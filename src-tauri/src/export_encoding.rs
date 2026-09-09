//! Packaged desktop encoder. Inputs and intermediate outputs belong to the job;
//! nothing reaches its destination until encoded bytes decode and measure.
use crate::export_format::ExportEncoding;
use crate::types::{CommandError, CommandResult};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

fn error(e: impl std::fmt::Display) -> CommandError {
    CommandError::Render(format!("Audio export: {e}"))
}
fn check_cancel(cancel: Option<&AtomicBool>) -> CommandResult<()> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        Err(error("cancelled"))
    } else {
        Ok(())
    }
}

pub struct Encoder {
    path: PathBuf,
}

impl Encoder {
    #[cfg(feature = "app-runner")]
    pub fn packaged() -> CommandResult<Self> {
        use sha2::{Digest, Sha256};
        let expected = option_env!("YES_MASTER_ENCODER_SHA256")
            .filter(|value| value.len() == 64)
            .ok_or_else(|| error("This build has no qualified audio encoder package"))?;
        let filename = if cfg!(windows) {
            "yes-master-encoder.exe"
        } else {
            "yes-master-encoder"
        };
        let path = std::env::current_exe()
            .map_err(error)?
            .parent()
            .ok_or_else(|| error("Missing application directory"))?
            .join(filename);
        // Local cargo tests/dev use the same staged, hash-bound artifact. Release
        // builds resolve only beside the installed executable, never PATH/env.
        #[cfg(debug_assertions)]
        let path = if path.exists() {
            path
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(env!("YES_MASTER_ENCODER_FILENAME"))
        };
        let bytes = std::fs::read(&path)
            .map_err(|e| error(format!("Packaged encoder unavailable: {e}")))?;
        if format!("{:x}", Sha256::digest(&bytes)) != expected {
            return Err(error(
                "Packaged encoder hash mismatch; reinstall the verified application",
            ));
        }
        Ok(Self { path })
    }

    #[cfg(not(feature = "app-runner"))]
    pub fn packaged() -> CommandResult<Self> {
        Err(error(
            "This encoding is available only in the desktop application",
        ))
    }

    pub fn encode_staged(
        &self,
        input: &Path,
        output: &Path,
        encoding: ExportEncoding,
        bits: u16,
        cancel: Option<&AtomicBool>,
    ) -> CommandResult<()> {
        encoding.validate()?;
        check_cancel(cancel)?;
        let channels = hound::WavReader::open(input)
            .map_err(error)?
            .spec()
            .channels;
        let layout = match channels {
            1 => "mono",
            2 => "stereo",
            _ => return Err(error("Unsupported channel layout")),
        };
        let mut command = Command::new(&self.path);
        command
            .args([
                "-v",
                "error",
                "-nostdin",
                "-n",
                "-threads",
                "1",
                "-channel_layout",
                layout,
                "-i",
            ])
            .arg(input)
            .args(["-map", "0:a:0", "-map_metadata", "-1", "-threads", "1"]);
        match encoding {
            ExportEncoding::Flac => {
                command.args(["-c:a", "flac", "-compression_level", "5"]);
            }
            ExportEncoding::Aiff => {
                command.args(["-c:a", if bits == 16 { "pcm_s16be" } else { "pcm_s24be" }]);
            }
            ExportEncoding::M4a { bitrate_kbps } | ExportEncoding::Aac { bitrate_kbps } => {
                command
                    .args(["-c:a", "aac", "-profile:a", "aac_low", "-b:a"])
                    .arg(format!("{bitrate_kbps}k"));
            }
            ExportEncoding::Ogg { quality } => {
                command
                    .args(["-c:a", "libvorbis", "-q:a"])
                    .arg(quality.to_string())
                    .args(["-page_duration", "1"]);
            }
            _ => return Err(error("WAV/MP3 must use their existing writers")),
        }
        command.arg(output);
        self.execute(command, cancel)
    }

    fn execute(&self, mut command: Command, cancel: Option<&AtomicBool>) -> CommandResult<()> {
        check_cancel(cancel)?;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(error)?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| error("Missing encoder error pipe"))?;
        let drain = std::thread::spawn(move || {
            let mut captured = Vec::new();
            let mut chunk = [0_u8; 4096];
            while let Ok(count) = stderr.read(&mut chunk) {
                if count == 0 {
                    break;
                }
                let retain = count.min(65_536_usize.saturating_sub(captured.len()));
                captured.extend_from_slice(&chunk[..retain]);
            }
            String::from_utf8_lossy(&captured).into_owned()
        });
        let outcome = loop {
            if let Err(e) = check_cancel(cancel) {
                break Err(e);
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(15)),
                Err(e) => break Err(error(e)),
            }
        };
        if outcome.is_err() {
            let _ = child.kill();
        }
        let _ = child.wait(); // Always reap, including cancellation and pipe/error paths.
        let stderr = drain
            .join()
            .map_err(|_| error("Encoder error reader failed"))?;
        let status = outcome?;
        if !status.success() {
            return Err(error(format!("Encoder failed ({status}): {stderr}")));
        }
        check_cancel(cancel)
    }
}

pub struct EncodedFile {
    pub path: PathBuf,
    pub measurements: (f32, f32, f32),
}

/// Container-aware read-back. FFmpeg honors AAC edit lists; Symphonia 0.5's
/// AAC decoder does not remove priming. A private float WAV keeps memory bounded
/// while measuring the actual decoded bytes, including ADTS's unavoidable pad.
pub fn measure(
    encoder: &Encoder,
    input: &Path,
    cancel: Option<&AtomicBool>,
) -> CommandResult<crate::mp3::DecodedMeasurements> {
    let scratch = tempfile::tempdir().map_err(error)?;
    let decoded = scratch.path().join("readback.wav");
    let mut command = Command::new(&encoder.path);
    command
        .args(["-v", "error", "-nostdin", "-n", "-threads", "1", "-i"])
        .arg(input)
        .args(["-map", "0:a:0", "-c:a", "pcm_f32le", "-threads", "1"])
        .arg(&decoded);
    encoder.execute(command, cancel)?;
    let mut reader = hound::WavReader::open(&decoded).map_err(error)?;
    let spec = reader.spec();
    let mut meter = ebur128::EbuR128::new(
        u32::from(spec.channels),
        spec.sample_rate,
        ebur128::Mode::I | ebur128::Mode::LRA | ebur128::Mode::TRUE_PEAK,
    )
    .map_err(error)?;
    let mut source = reader.samples::<f32>();
    let mut buffer = Vec::with_capacity(4096 * usize::from(spec.channels));
    let mut frames = 0;
    loop {
        check_cancel(cancel)?;
        buffer.clear();
        for sample in source.by_ref().take(buffer.capacity()) {
            let sample = sample.map_err(error)?;
            if !sample.is_finite() {
                return Err(error("Non-finite decoded sample"));
            }
            buffer.push(sample);
        }
        if buffer.is_empty() {
            break;
        }
        if buffer.len() % usize::from(spec.channels) != 0 {
            return Err(error("Incomplete decoded frame"));
        }
        frames += buffer.len() as u64 / u64::from(spec.channels);
        meter.add_frames_f32(&buffer).map_err(error)?;
    }
    if frames == 0 {
        return Err(error("No delivered audio frames"));
    }
    let lufs = crate::analysis::sanitize_lufs(meter.loudness_global().map_err(error)? as f32);
    let peak = (0..u32::from(spec.channels))
        .map(|channel| meter.true_peak(channel).map_err(error))
        .collect::<CommandResult<Vec<_>>>()?
        .into_iter()
        .fold(0.0_f64, f64::max);
    let lra = meter.loudness_range().map_err(error)? as f32;
    Ok(crate::mp3::DecodedMeasurements {
        measurements: (
            lufs,
            if peak > 0.0 {
                (20.0 * peak.log10()) as f32
            } else {
                -60.0
            },
            if lra.is_finite() { lra } else { 0.0 },
        ),
        sample_rate: spec.sample_rate,
        channels: u32::from(spec.channels),
        frames,
    })
}

/// Input WAV is already at the selected delivery precision/rate. It is never
/// mastered again or dithered by the encoder. The containing temporary directory
/// owns every intermediate and cleans them on every error/cancellation path.
pub fn deliver_staged(
    encoder: &Encoder,
    input: &Path,
    destination: &Path,
    encoding: ExportEncoding,
    cancel: Option<&AtomicBool>,
) -> CommandResult<EncodedFile> {
    check_cancel(cancel)?;
    let mut input_reader = hound::WavReader::open(input).map_err(error)?;
    let spec = input_reader.spec();
    if encoding.delivery_rate(spec.sample_rate) != spec.sample_rate
        || !(1..=2).contains(&spec.channels)
    {
        return Err(error("Unsupported delivery rate/channel layout"));
    }
    if !encoding.is_lossy()
        && (spec.sample_format != hound::SampleFormat::Int
            || !matches!(spec.bits_per_sample, 16 | 24))
    {
        return Err(error(
            "Lossless encoding requires the selected integer PCM representation",
        ));
    }
    let expected_frames = input_reader.duration();
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let scratch = tempfile::Builder::new()
        .prefix(".yes-export-")
        .tempdir_in(parent)
        .map_err(error)?;
    let padded;
    let encoder_input = if matches!(encoding, ExportEncoding::Aac { .. }) && expected_frames < 2048
    {
        if spec.sample_format != hound::SampleFormat::Float || spec.bits_per_sample != 32 {
            return Err(error("AAC staging requires float PCM"));
        }
        padded = scratch.path().join("padded.wav");
        let mut writer = hound::WavWriter::create(&padded, spec).map_err(error)?;
        for sample in input_reader.samples::<f32>() {
            writer.write_sample(sample.map_err(error)?).map_err(error)?;
        }
        for _ in expected_frames..2048 {
            for _ in 0..spec.channels {
                writer.write_sample(0.0_f32).map_err(error)?;
            }
        }
        writer.finalize().map_err(error)?;
        padded.as_path()
    } else {
        input
    };
    let encoded = scratch
        .path()
        .join("encoded")
        .with_extension(encoding.extension());
    encoder.encode_staged(
        encoder_input,
        &encoded,
        encoding,
        spec.bits_per_sample,
        cancel,
    )?;
    let measured = measure(encoder, &encoded, cancel)?;
    if measured.sample_rate != spec.sample_rate || measured.channels != u32::from(spec.channels) {
        return Err(error(
            "Encoded file has an unexpected sample rate/channel count",
        ));
    }
    let expected = u64::from(expected_frames);
    let (minimum, maximum) = match encoding {
        ExportEncoding::M4a { .. } => (expected, expected + 1023),
        ExportEncoding::Aac { .. } => (expected + 1024, expected + 3071),
        _ => (expected, expected),
    };
    if !(minimum..=maximum).contains(&measured.frames) {
        return Err(error(format!(
            "Decoded file duration failed verification: {} frames, expected {minimum}..={maximum}",
            measured.frames
        )));
    }
    // Read-back validates actual bytes before atomically acquiring a destination.
    check_cancel(cancel)?;
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&encoded)
        .map_err(error)?
        .sync_all()
        .map_err(error)?;
    let saved = crate::wav_writer::finalize_never_overwrite(&encoded, destination)?;
    if let Err(e) = check_cancel(cancel) {
        let _ = std::fs::remove_file(&saved);
        return Err(e);
    }
    Ok(EncodedFile {
        path: saved,
        measurements: measured.measurements,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn write(
    encoder: &Encoder,
    destination: &Path,
    samples: &[f32],
    rate: u32,
    channels: u16,
    bits: u16,
    encoding: ExportEncoding,
    cancel: Option<&AtomicBool>,
) -> CommandResult<EncodedFile> {
    check_cancel(cancel)?;
    let scratch = tempfile::tempdir().map_err(error)?;
    let staged = crate::wav_writer::write_wav_with_cancel(
        &scratch.path().join("delivery.wav"),
        samples,
        rate,
        channels,
        bits,
        cancel,
    )?;
    check_cancel(cancel)?;
    deliver_staged(encoder, &staged, destination, encoding, cancel)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "subprocess fixture invoked only by cancellation test"]
    fn encoder_child_probe() {
        let Ok(marker) = std::env::var("YES_ENCODER_CHILD_PROBE") else {
            return;
        };
        std::fs::write(marker, b"started").unwrap();
        use std::io::Write;
        let mut stderr = std::io::stderr().lock();
        for _ in 0..1024 {
            let _ = stderr.write_all(&[b'x'; 4096]);
        }
        std::thread::sleep(std::time::Duration::from_secs(30));
    }

    #[test]
    fn cancellation_drains_errors_and_reaps_a_started_child() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("started");
        let cancel = AtomicBool::new(false);
        let exe = std::env::current_exe().unwrap();
        let encoder = Encoder { path: exe.clone() };
        let started = std::time::Instant::now();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                while !marker.exists() && started.elapsed().as_secs() < 10 {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                cancel.store(true, Ordering::Relaxed);
            });
            let mut command = Command::new(exe);
            command
                .args([
                    "--exact",
                    "export_encoding::tests::encoder_child_probe",
                    "--ignored",
                    "--nocapture",
                ])
                .env("YES_ENCODER_CHILD_PROBE", &marker);
            let result = encoder.execute(command, Some(&cancel));
            assert!(result.unwrap_err().to_string().contains("cancelled"));
        });
        assert!(
            marker.exists(),
            "the child must actually start before cancellation"
        );
        assert!(
            started.elapsed().as_secs() < 15,
            "child was not promptly killed/reaped"
        );
    }
}
