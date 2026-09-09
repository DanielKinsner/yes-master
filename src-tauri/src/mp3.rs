//! Desktop MP3 delivery. The mastering chain supplies final float PCM; LAME
//! only encodes it. Receipts measure a fresh decode of the delivered bytes.
use crate::types::{CommandError, CommandResult};
#[cfg(feature = "app-runner")]
use mp3lame_encoder::{Bitrate, Builder, FlushGap, InterleavedPcm, MonoPcm, Quality, VbrMode};
use std::io::Write;
#[cfg(feature = "app-runner")]
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn delivery_rate(rate: u32) -> u32 {
    if rate % 44_100 == 0 {
        44_100
    } else if rate == 32_000 {
        32_000
    } else {
        48_000
    }
}

pub fn validate_bitrate(kbps: u16) -> CommandResult<()> {
    if matches!(kbps, 128 | 192 | 256 | 320) {
        Ok(())
    } else {
        Err(CommandError::Render(
            "MP3 bitrate must be 128, 192, 256 or 320 kbps".into(),
        ))
    }
}

fn error(e: impl std::fmt::Display) -> CommandError {
    CommandError::Render(format!("MP3: {e}"))
}
fn check_cancel(cancel: Option<&AtomicBool>) -> CommandResult<()> {
    if cancel.is_some_and(|v| v.load(Ordering::Relaxed)) {
        Err(CommandError::Other("MP3 export cancelled".into()))
    } else {
        Ok(())
    }
}

/// Album assembly stays lossless until its single final encode. Private float
/// WAV staging reuses the proven bounded album assembler, including gaps and
/// per-track overrides; MP3 tracks are never decoded/re-encoded into the album.
pub fn render_album(
    request: &crate::engine::AlbumPlanRenderRequest,
    out_dir: &Path,
    progress: Option<&dyn Fn(f32)>,
    cancel: Option<&AtomicBool>,
    job_id: Option<&str>,
    bitrate: Option<u16>,
) -> CommandResult<crate::engine::AlbumRenderReport> {
    let Some(kbps) = bitrate else {
        return crate::album_render::render_album_plan_impl_with_cancel(
            request, out_dir, progress, cancel, job_id,
        );
    };
    validate_bitrate(kbps)?;
    crate::engine::validate_album_source_paths(&request.tracks)?;
    if crate::files::has_parent_dir_component(out_dir) {
        return Err(error("invalid output directory"));
    }
    let mut staged_request = request.clone();
    let requested_rate = match request.plan.delivery_sample_rate {
        Some(rate) => rate,
        None => request
            .tracks
            .iter()
            .map(|t| crate::decode::probe_sample_rate(Path::new(&t.source_path)))
            .collect::<CommandResult<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(48_000),
    };
    staged_request.plan.delivery_sample_rate = Some(delivery_rate(requested_rate));
    staged_request.plan.delivery_bit_depth = Some(32);
    let scratch = tempfile::tempdir().map_err(error)?;
    let staged_progress = |fraction: f32| {
        if let Some(cb) = progress {
            cb(fraction * 0.8);
        }
    };
    let mut report = crate::album_render::render_album_plan_impl_with_cancel(
        &staged_request,
        scratch.path(),
        Some(&staged_progress),
        cancel,
        job_id,
    )?;
    if matches!(report.status, crate::types::JobStatus::Cancelled) {
        return Ok(report);
    }
    std::fs::create_dir_all(out_dir).map_err(error)?;
    let destination = crate::album_render::unique_export_subdir(out_dir, &request.plan.title)?;
    let metadata = destination.join("metadata");
    let mut written = Vec::new();
    let result = (|| {
        std::fs::create_dir(&metadata).map_err(error)?;
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report.manifest_path).map_err(error)?)
                .map_err(error)?;
        let total = report.tracks.len() + 1;
        for (index, track) in report.tracks.iter_mut().enumerate() {
            check_cancel(cancel)?;
            let input = Path::new(&track.output_path);
            let output = destination
                .join(input.file_name().ok_or_else(|| error("track filename"))?)
                .with_extension("mp3");
            let mut reader = hound::WavReader::open(input).map_err(error)?;
            let saved = write(
                &output,
                reader.samples::<f32>().map(|s| s.map_err(error)),
                report.rendered_sample_rate,
                report.rendered_channels,
                kbps,
                cancel,
            )?;
            written.push(saved.clone());
            let (lufs, tp, _) = measure(&saved, cancel)?;
            track.output_path = saved.to_string_lossy().into_owned();
            track.measured_lufs = lufs;
            track.true_peak_dbtp = tp;
            if let Some(cb) = progress {
                cb(0.8 + 0.2 * (index + 1) as f32 / total as f32);
            }
        }
        let mut reader = hound::WavReader::open(&report.album_wav_path).map_err(error)?;
        let saved = write(
            &destination.join("album.mp3"),
            reader.samples::<f32>().map(|s| s.map_err(error)),
            report.rendered_sample_rate,
            report.rendered_channels,
            kbps,
            cancel,
        )?;
        written.push(saved.clone());
        let (lufs, tp, lra) = measure(&saved, cancel)?;
        report.album_wav_path = saved.to_string_lossy().into_owned();
        report.mp3_bitrate_kbps = Some(kbps);
        report.delivered_format = Some(
            crate::export_format::ExportEncoding::Mp3 { bitrate_kbps: kbps }.delivered(
                report.rendered_sample_rate,
                report.rendered_channels,
                32,
                request.plan.delivery_sample_rate,
                crate::album_encoding::requested_bit_depth(request),
            ),
        );
        report.bit_depth = 0;
        report.requested_sample_rate = request.plan.delivery_sample_rate;
        report.manifest_path = metadata
            .join("manifest.json")
            .to_string_lossy()
            .into_owned();
        manifest["format"] = serde_json::json!("mp3");
        manifest["mp3_bitrate_kbps"] = serde_json::json!(kbps);
        manifest["bit_depth"] = serde_json::Value::Null;
        manifest["plan"] = serde_json::to_value(&request.plan).map_err(error)?;
        manifest["album_wav_path"] = serde_json::json!(report.album_wav_path);
        manifest["album_measurements"] =
            serde_json::json!({"lufs_integrated":lufs,"true_peak_dbtp":tp,"dynamic_range_lu":lra});
        manifest["tracks"] = serde_json::to_value(&report.tracks).map_err(error)?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&report.manifest_path)
            .map_err(error)?;
        written.push(PathBuf::from(&report.manifest_path));
        file.write_all(&serde_json::to_vec_pretty(&manifest).map_err(error)?)
            .map_err(error)?;
        check_cancel(cancel)?;
        if let Some(cb) = progress {
            cb(1.0);
        }
        Ok(())
    })();
    if let Err(err) = result {
        for path in written {
            let _ = std::fs::remove_file(path);
        }
        let _ = std::fs::remove_dir(metadata);
        let _ = std::fs::remove_dir(destination);
        if cancel.is_some_and(|v| v.load(Ordering::Relaxed)) {
            report.status = crate::types::JobStatus::Cancelled;
            report.album_wav_path.clear();
            report.manifest_path.clear();
            report.tracks.clear();
            return Ok(report);
        }
        return Err(err);
    }
    Ok(report)
}

/// Bounded encoder buffers, atomic non-overwriting finalization, and gapless
/// delay/padding metadata. Input must already be at its delivery sample rate.
#[cfg(feature = "app-runner")]
pub fn write(
    path: &Path,
    samples: impl Iterator<Item = CommandResult<f32>>,
    rate: u32,
    channels: u16,
    kbps: u16,
    cancel: Option<&AtomicBool>,
) -> CommandResult<PathBuf> {
    if !(1..=2).contains(&channels) || delivery_rate(rate) != rate {
        return Err(error("unsupported delivery rate or channels"));
    }
    validate_bitrate(kbps)?;
    let bitrate = match kbps {
        128 => Bitrate::Kbps128,
        192 => Bitrate::Kbps192,
        256 => Bitrate::Kbps256,
        _ => Bitrate::Kbps320,
    };
    check_cancel(cancel)?;
    let tmp = crate::wav_writer::unique_tmp_path(path)?;
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(error)?;
        let mut encoder = Builder::new()
            .ok_or_else(|| error("encoder allocation"))?
            .with_num_channels(channels as u8)
            .map_err(error)?
            .with_sample_rate(rate)
            .map_err(error)?
            .with_brate(bitrate)
            .map_err(error)?
            .with_quality(Quality::Best)
            .map_err(error)?
            .with_vbr_mode(VbrMode::Off)
            .map_err(error)?
            .with_to_write_vbr_tag(true)
            .map_err(error)?
            .build()
            .map_err(error)?;
        let mut input = Vec::with_capacity(8192);
        let mut output = Vec::with_capacity(32768);
        let mut source = samples;
        loop {
            check_cancel(cancel)?;
            input.clear();
            for sample in source.by_ref().take(4096 * channels as usize) {
                let sample = sample?;
                if !sample.is_finite() {
                    return Err(error("non-finite master sample"));
                }
                input.push(sample);
            }
            if input.is_empty() {
                break;
            }
            if input.len() % channels as usize != 0 {
                return Err(error("incomplete audio frame"));
            }
            output.clear();
            output.reserve(32768);
            if channels == 1 {
                encoder
                    .encode_to_vec(MonoPcm(&input), &mut output)
                    .map_err(error)?;
            } else {
                encoder
                    .encode_to_vec(InterleavedPcm(&input), &mut output)
                    .map_err(error)?;
            }
            file.write_all(&output).map_err(error)?;
        }
        output.clear();
        output.reserve(32768);
        encoder
            .flush_to_vec::<FlushGap>(&mut output)
            .map_err(error)?;
        file.write_all(&output).map_err(error)?;
        output.clear();
        output.reserve(32768);
        encoder
            .lame_tag_encode_to_vec(&mut output)
            .ok_or_else(|| error("missing gapless tag"))?;
        file.seek(SeekFrom::Start(0)).map_err(error)?;
        file.write_all(&output).map_err(error)?;
        file.sync_all().map_err(error)?;
        drop(file);
        check_cancel(cancel)?;
        crate::wav_writer::finalize_never_overwrite(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(not(feature = "app-runner"))]
pub fn write(
    _path: &Path,
    _samples: impl Iterator<Item = CommandResult<f32>>,
    _rate: u32,
    _channels: u16,
    _kbps: u16,
    _cancel: Option<&AtomicBool>,
) -> CommandResult<PathBuf> {
    Err(error("MP3 delivery is available in the desktop app"))
}

/// Stream the delivered MP3 back through a separate decoder and BS.1770 meter.
/// This remains bounded for continuous albums as well as individual tracks.
pub fn measure(path: &Path, cancel: Option<&AtomicBool>) -> CommandResult<(f32, f32, f32)> {
    Ok(measure_details(path, cancel)?.measurements)
}

pub struct DecodedMeasurements {
    pub measurements: (f32, f32, f32),
    pub sample_rate: u32,
    pub channels: u32,
    pub frames: u64,
}

pub fn measure_details(
    path: &Path,
    cancel: Option<&AtomicBool>,
) -> CommandResult<DecodedMeasurements> {
    use symphonia::core::{
        audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
        meta::MetadataOptions, probe::Hint,
    };
    let file = std::fs::File::open(path).map_err(error)?;
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|value| value.to_str()) {
        hint.with_extension(extension);
    }
    let mut format = symphonia::default::get_probe()
        .format(
            &hint,
            MediaSourceStream::new(Box::new(file), Default::default()),
            &FormatOptions {
                enable_gapless: true,
                ..Default::default()
            },
            &MetadataOptions::default(),
        )
        .map_err(error)?
        .format;
    let track = format
        .default_track()
        .ok_or_else(|| error("no audio stream"))?;
    let id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(error)?;
    let mut state: Option<(ebur128::EbuR128, u32, u32)> = None;
    let mut frames = 0_u64;
    loop {
        check_cancel(cancel)?;
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(e) => return Err(error(e)),
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = decoder.decode(&packet).map_err(error)?;
        if state.is_none() {
            let rate = decoded.spec().rate;
            let channels = decoded.spec().channels.count() as u32;
            let meter = ebur128::EbuR128::new(
                channels,
                rate,
                ebur128::Mode::I | ebur128::Mode::LRA | ebur128::Mode::TRUE_PEAK,
            )
            .map_err(error)?;
            state = Some((meter, rate, channels));
        }
        let (meter, rate, channels) = state.as_mut().ok_or_else(|| error("no decoded format"))?;
        if decoded.spec().rate != *rate || decoded.spec().channels.count() as u32 != *channels {
            return Err(error("delivered stream changes rate or channel count"));
        }
        let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        samples.copy_interleaved_ref(decoded);
        if samples.samples().iter().any(|s| !s.is_finite()) {
            return Err(error("non-finite delivered sample"));
        }
        frames += samples.samples().len() as u64 / u64::from(*channels);
        meter.add_frames_f32(samples.samples()).map_err(error)?;
    }
    let (meter, rate, channels) = state.ok_or_else(|| error("no delivered audio frames"))?;
    let lufs = crate::analysis::sanitize_lufs(meter.loudness_global().map_err(error)? as f32);
    let peak = (0..channels)
        .map(|ch| meter.true_peak(ch).map_err(error))
        .collect::<CommandResult<Vec<_>>>()?
        .into_iter()
        .fold(0.0_f64, f64::max);
    let tp = if peak > 0.0 {
        (20.0 * peak.log10()) as f32
    } else {
        -60.0
    };
    let lra = meter.loudness_range().map_err(error)? as f32;
    if frames == 0 {
        return Err(error("no delivered audio frames"));
    }
    Ok(DecodedMeasurements {
        measurements: (lufs, tp, if lra.is_finite() { lra } else { 0.0 }),
        sample_rate: rate,
        channels,
        frames,
    })
}

#[cfg(all(test, feature = "app-runner"))]
mod tests {
    use super::*;
    #[test]
    fn all_delivery_bitrates_decode_and_preserve_level_in_mono_and_stereo() {
        let dir = tempfile::tempdir().unwrap();
        for rate in [32_000, 44_100, 48_000] {
            for channels in [1, 2] {
                let samples: Vec<f32> = (0..rate * 2)
                    .flat_map(|n| {
                        let sample =
                            (n as f32 * 997.0 * std::f32::consts::TAU / rate as f32).sin() * 0.15;
                        std::iter::repeat_n(sample, channels as usize)
                    })
                    .collect();
                let reference =
                    crate::wav_writer::measure_delivery(&samples, rate, channels, 32).unwrap();
                for kbps in [128, 192, 256, 320] {
                    let path = dir.path().join(format!("{rate}-{channels}-{kbps}.mp3"));
                    let saved = write(
                        &path,
                        samples.iter().copied().map(Ok),
                        rate,
                        channels,
                        kbps,
                        None,
                    )
                    .unwrap();
                    let delivered = measure(&saved, None).unwrap();
                    assert!(
                        (delivered.0 - reference.0).abs() < 0.5,
                        "{rate}/{channels}/{kbps}: {delivered:?} versus {reference:?}"
                    );
                    let size = std::fs::metadata(&saved).unwrap().len();
                    assert!((size as f64 / (kbps as f64 * 1000.0 / 8.0 * 2.0) - 1.0).abs() < 0.12);
                }
            }
        }
    }
    #[test]
    fn collision_and_cancellation_preserve_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("master.mp3");
        std::fs::write(&path, b"existing master").unwrap();
        let saved = write(
            &path,
            std::iter::repeat_n(Ok(0.1), 48_000),
            48_000,
            1,
            320,
            None,
        )
        .unwrap();
        assert_ne!(saved, path);
        assert_eq!(std::fs::read(&path).unwrap(), b"existing master");
        let cancel = AtomicBool::new(false);
        let samples = (0..48_000).map(|n| {
            if n == 5000 {
                cancel.store(true, Ordering::Relaxed);
            }
            Ok(0.1)
        });
        assert!(write(&path, samples, 48_000, 1, 320, Some(&cancel)).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        assert!(validate_bitrate(64).is_err());
    }
}
