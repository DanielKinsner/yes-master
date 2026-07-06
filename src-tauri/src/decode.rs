//! Symphonia-backed PCM decode utilities.
//!
//! Two entry points:
//! - [`decode_full`] reads the entire file into an interleaved `f32` buffer
//!   plus its sample rate / channel count. Used by both the audio thread
//!   (live playback / preview) and the engine's render paths (export).
//! - [`decode_to_peaks`] reads the file once and produces a downsampled
//!   max-abs-per-pixel peak envelope for the waveform UI, without keeping
//!   the full PCM in memory.
//!
//! Both helpers share the same Symphonia probe / format / decoder loop;
//! the difference is only what they accumulate from each packet.

use std::path::Path;

use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::types::{CommandError, CommandResult};

pub struct DecodedPeaks {
    pub channels: Vec<Vec<f32>>,
    pub samples_per_pixel: u32,
    pub total_samples: u64,
    pub sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct DecodedPcm {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbedAudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

pub const MIN_WAVEFORM_PIXELS: u32 = 64;
pub const MAX_WAVEFORM_PIXELS: u32 = 16_384;

pub fn clamp_waveform_target_pixels(target_pixels: u32) -> u32 {
    target_pixels.clamp(MIN_WAVEFORM_PIXELS, MAX_WAVEFORM_PIXELS)
}

fn estimated_decode_capacity(
    n_frames: Option<u64>,
    channel_count: u16,
    file_size_bytes: u64,
) -> usize {
    let Some(frames) = n_frames else {
        return 0;
    };
    let claimed = frames.saturating_mul(u64::from(channel_count));
    let file_bound = file_size_bytes.saturating_mul(8);
    claimed.min(file_bound).min(usize::MAX as u64) as usize
}

/// Validated sample rate from codec params. `Some(0)` is crafted-file
/// garbage — reject it here rather than let a zero rate reach filter
/// coefficient math or duration division downstream (D1, 2026-07-03);
/// `None` falls back to 44.1 kHz (some containers omit the field).
fn validated_sample_rate(rate: Option<u32>) -> CommandResult<u32> {
    match rate {
        Some(0) => Err(CommandError::Decode(
            "invalid sample rate 0 in source header".to_string(),
        )),
        Some(r) => Ok(r),
        None => Ok(44_100),
    }
}

fn decoded_channel_count(decoded: &AudioBufferRef<'_>) -> CommandResult<u16> {
    let channels = decoded.spec().channels.count().max(1);
    u16::try_from(channels)
        .map_err(|_| CommandError::Decode(format!("unsupported decoded channel count {channels}")))
}

fn reconcile_decoded_channel_count(observed: &mut Option<u16>, actual: u16) -> CommandResult<u16> {
    match *observed {
        Some(previous) if previous == actual => Ok(previous),
        Some(previous) => Err(CommandError::Decode(format!(
            "decoded channel count changed from {previous} to {actual}"
        ))),
        None => {
            *observed = Some(actual);
            Ok(actual)
        }
    }
}

/// Panic boundary for the untrusted-parser layer (D1, 2026-07-03). A
/// crafted file can panic INSIDE symphonia before any of our validation
/// runs (proven: a 0 Hz fmt chunk asserts in symphonia-core units.rs) —
/// and a panic in a command thread is a crash a hostile file can trigger
/// at will. Convert it into the same typed `CommandError::Decode` any
/// other malformed file gets. (Cargo profiles keep the default unwind
/// strategy, so `catch_unwind` is effective in dev and release.)
fn decode_panic_boundary<T>(
    path: &Path,
    body: impl FnOnce() -> CommandResult<T>,
) -> CommandResult<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).unwrap_or_else(|_| {
        Err(CommandError::Decode(format!(
            "malformed audio file: {}",
            path.display()
        )))
    })
}

pub fn decode_full(path: &Path) -> CommandResult<DecodedPcm> {
    decode_panic_boundary(path, || decode_full_inner(path)).inspect_err(|e| {
        crate::diagnostics::warn(format!("decode failed for {}: {e}", path.display()))
    })
}

fn decode_full_inner(path: &Path) -> CommandResult<DecodedPcm> {
    let file_size_bytes = std::fs::metadata(path)
        .map_err(|e| CommandError::Io(e.to_string()))?
        .len();
    let file = std::fs::File::open(path).map_err(|e| CommandError::Io(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| CommandError::Decode(e.to_string()))?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| CommandError::Decode("no decodable track".to_string()))?;
    let stream_track_id = track.id;
    let sample_rate = validated_sample_rate(track.codec_params.sample_rate)?;
    let channel_count = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2)
        .max(1) as u16;
    let estimated_capacity =
        estimated_decode_capacity(track.codec_params.n_frames, channel_count, file_size_bytes);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| CommandError::Decode(e.to_string()))?;

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut observed_channel_count: Option<u16> = None;
    let mut samples: Vec<f32> = Vec::with_capacity(estimated_capacity);

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(CommandError::Decode(e.to_string())),
        };
        if packet.track_id() != stream_track_id {
            continue;
        }
        let decoded: AudioBufferRef = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::IoError(_)) => continue,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(CommandError::Decode(e.to_string())),
        };
        let actual_channel_count = decoded_channel_count(&decoded)?;
        reconcile_decoded_channel_count(&mut observed_channel_count, actual_channel_count)?;
        if sample_buf.is_none() {
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }
        let sbuf = sample_buf.as_mut().unwrap();
        sbuf.copy_interleaved_ref(decoded);
        // Hostile-input boundary (D1, 2026-07-03): a float WAV can carry
        // any bit pattern — NaN/±Inf here would poison analysis, the live
        // chain, and the rendered file (NaN × gain stays NaN all the way
        // to the WAV writer). Zero non-finite samples and clamp to ±64
        // (+36 dBFS, far beyond any real program material) so a crafted
        // file can't blow out the engine's f32 headroom either.
        samples.extend(sbuf.samples().iter().map(|s| {
            if s.is_finite() {
                s.clamp(-64.0, 64.0)
            } else {
                0.0
            }
        }));
    }

    let channels = observed_channel_count.unwrap_or(channel_count);
    // Above-stereo sources fold to stereo AT DECODE (owner decision,
    // 2026-07-06 audit): the mastering chain is stereo-native — the multiband
    // compressor and width stage process channels 0-1 only — so decoding a
    // 5.1 file as-is mastered the front pair (compression + makeup gain) and
    // passed center/surrounds through untouched. Folding here keeps
    // analysis, audition, track render, and album render all processing the
    // SAME signal, using the same tested downmix as album stereo delivery
    // (LFE excluded). Header probes (probe_audio_format) still report the
    // file's real channel count, so import UI and album records stay honest.
    let (samples, channels) = fold_interleaved_above_stereo_to_stereo(samples, channels);
    Ok(DecodedPcm {
        samples,
        sample_rate,
        channels,
    })
}

pub(crate) const SURROUND_DOWNMIX_GAIN: f32 = 0.707_106_77;

fn add_downmix_sample(acc: &mut f32, weight_sum: &mut f32, sample: f32, weight: f32) {
    *acc += sample * weight;
    *weight_sum += weight;
}

/// Fold one interleaved frame of any channel count down to stereo. Canonical
/// fold for the whole engine: applied at decode for every imported source,
/// and reused by the album path's `convert_channel_count` for direct-PCM
/// callers. Common 5.1 order (L, R, C, LFE, Ls, Rs) is assumed for 6+
/// channels; LFE is intentionally not folded into stereo delivery.
pub(crate) fn downmix_frame_to_stereo(frame: &[f32]) -> [f32; 2] {
    if frame.is_empty() {
        return [0.0, 0.0];
    }
    if frame.len() == 1 {
        return [frame[0], frame[0]];
    }
    if frame.len() == 2 {
        return [frame[0], frame[1]];
    }

    let mut left = 0.0;
    let mut right = 0.0;
    let mut left_weight = 0.0;
    let mut right_weight = 0.0;

    add_downmix_sample(&mut left, &mut left_weight, frame[0], 1.0);
    add_downmix_sample(&mut right, &mut right_weight, frame[1], 1.0);

    match frame.len() {
        3 => {
            add_downmix_sample(&mut left, &mut left_weight, frame[2], SURROUND_DOWNMIX_GAIN);
            add_downmix_sample(
                &mut right,
                &mut right_weight,
                frame[2],
                SURROUND_DOWNMIX_GAIN,
            );
        }
        4 => {
            add_downmix_sample(&mut left, &mut left_weight, frame[2], SURROUND_DOWNMIX_GAIN);
            add_downmix_sample(
                &mut right,
                &mut right_weight,
                frame[3],
                SURROUND_DOWNMIX_GAIN,
            );
        }
        5 => {
            add_downmix_sample(&mut left, &mut left_weight, frame[2], SURROUND_DOWNMIX_GAIN);
            add_downmix_sample(
                &mut right,
                &mut right_weight,
                frame[2],
                SURROUND_DOWNMIX_GAIN,
            );
            add_downmix_sample(&mut left, &mut left_weight, frame[3], SURROUND_DOWNMIX_GAIN);
            add_downmix_sample(
                &mut right,
                &mut right_weight,
                frame[4],
                SURROUND_DOWNMIX_GAIN,
            );
        }
        _ => {
            add_downmix_sample(&mut left, &mut left_weight, frame[2], SURROUND_DOWNMIX_GAIN);
            add_downmix_sample(
                &mut right,
                &mut right_weight,
                frame[2],
                SURROUND_DOWNMIX_GAIN,
            );
            // Common 5.1 order is L, R, C, LFE, Ls, Rs. LFE is intentionally
            // not folded into stereo delivery.
            add_downmix_sample(&mut left, &mut left_weight, frame[4], SURROUND_DOWNMIX_GAIN);
            add_downmix_sample(
                &mut right,
                &mut right_weight,
                frame[5],
                SURROUND_DOWNMIX_GAIN,
            );
            for (offset, sample) in frame.iter().copied().enumerate().skip(6) {
                if offset % 2 == 0 {
                    add_downmix_sample(&mut left, &mut left_weight, sample, SURROUND_DOWNMIX_GAIN);
                } else {
                    add_downmix_sample(
                        &mut right,
                        &mut right_weight,
                        sample,
                        SURROUND_DOWNMIX_GAIN,
                    );
                }
            }
        }
    }

    [left / left_weight.max(1.0), right / right_weight.max(1.0)]
}

/// Fold an interleaved above-stereo buffer down to interleaved stereo; mono
/// and stereo buffers pass through untouched. Returns the (possibly new)
/// buffer and its channel count.
fn fold_interleaved_above_stereo_to_stereo(samples: Vec<f32>, channels: u16) -> (Vec<f32>, u16) {
    if channels <= 2 {
        return (samples, channels);
    }
    let ch = channels as usize;
    let frames = samples.len() / ch;
    let mut folded = Vec::with_capacity(frames * 2);
    for frame in samples.chunks_exact(ch) {
        folded.extend_from_slice(&downmix_frame_to_stereo(frame));
    }
    (folded, 2)
}

pub fn decode_to_peaks(path: &Path, target_pixels: u32) -> CommandResult<DecodedPeaks> {
    decode_panic_boundary(path, || decode_to_peaks_inner(path, target_pixels))
}

fn decode_to_peaks_inner(path: &Path, target_pixels: u32) -> CommandResult<DecodedPeaks> {
    let target_pixels = clamp_waveform_target_pixels(target_pixels);
    let file = std::fs::File::open(path).map_err(|e| CommandError::Io(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| CommandError::Decode(e.to_string()))?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| CommandError::Decode("no decodable track".to_string()))?;
    let stream_track_id = track.id;
    let sample_rate = validated_sample_rate(track.codec_params.sample_rate)?;
    let channel_count = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2)
        .max(1);
    let total_frames = track.codec_params.n_frames.unwrap_or(0);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| CommandError::Decode(e.to_string()))?;

    let samples_per_pixel = if total_frames > 0 {
        ((total_frames as f64 / target_pixels as f64).ceil() as u32).max(1)
    } else {
        (sample_rate / 50).max(1)
    };

    let mut channel_peaks: Vec<Vec<f32>> = Vec::new();
    let mut running_max: Vec<f32> = Vec::new();
    let mut observed_channel_count: Option<u16> = None;
    let mut window_frames: u64 = 0;
    let mut total_decoded_frames: u64 = 0;
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(CommandError::Decode(e.to_string())),
        };
        if packet.track_id() != stream_track_id {
            continue;
        }
        let decoded: AudioBufferRef = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::IoError(_)) => continue,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(CommandError::Decode(e.to_string())),
        };
        let actual_channel_count = decoded_channel_count(&decoded)?;
        let decoded_channels =
            reconcile_decoded_channel_count(&mut observed_channel_count, actual_channel_count)?
                as usize;
        if channel_peaks.is_empty() {
            channel_peaks = vec![Vec::with_capacity(target_pixels as usize); decoded_channels];
            running_max = vec![0.0; decoded_channels];
        }
        if sample_buf.is_none() {
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }
        let sbuf = sample_buf.as_mut().unwrap();
        sbuf.copy_interleaved_ref(decoded);
        let samples = sbuf.samples();
        let frames = samples.len() / decoded_channels.max(1);
        total_decoded_frames += frames as u64;

        for frame in 0..frames {
            for ch in 0..decoded_channels {
                let v = samples[frame * decoded_channels + ch].abs();
                if v > running_max[ch] {
                    running_max[ch] = v;
                }
            }
            window_frames += 1;
            if window_frames >= u64::from(samples_per_pixel) {
                for ch in 0..decoded_channels {
                    channel_peaks[ch].push(running_max[ch]);
                    running_max[ch] = 0.0;
                }
                window_frames = 0;
            }
        }
    }

    if channel_peaks.is_empty() {
        channel_peaks = vec![Vec::new(); channel_count];
    }

    if window_frames > 0 {
        for ch in 0..channel_peaks.len() {
            channel_peaks[ch].push(running_max[ch]);
        }
    }

    Ok(DecodedPeaks {
        channels: channel_peaks,
        samples_per_pixel,
        total_samples: total_decoded_frames,
        sample_rate,
    })
}

/// Read just the container/codec header to learn the source format without
/// decoding any audio. Used by the album render path to resolve album-wide
/// delivery format before any track is processed.
pub fn probe_audio_format(path: &Path) -> CommandResult<ProbedAudioFormat> {
    decode_panic_boundary(path, || probe_audio_format_inner(path))
}

fn probe_audio_format_inner(path: &Path) -> CommandResult<ProbedAudioFormat> {
    let file = std::fs::File::open(path).map_err(|e| CommandError::Io(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| CommandError::Decode(e.to_string()))?;
    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| CommandError::Decode("no decodable track".to_string()))?;
    Ok(ProbedAudioFormat {
        sample_rate: validated_sample_rate(track.codec_params.sample_rate)?,
        channels: track
            .codec_params
            .channels
            .map(|c| c.count())
            .unwrap_or(2)
            .max(1) as u16,
    })
}

/// Read just the container/codec header to learn the source sample rate
/// without decoding any audio. Kept as the narrow helper for existing callers.
pub fn probe_sample_rate(path: &Path) -> CommandResult<u32> {
    Ok(probe_audio_format(path)?.sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_silence_wav(path: &Path, sample_rate: u32) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).expect("create");
        for _ in 0..1024 {
            w.write_sample(0_i16).expect("write");
        }
        w.finalize().expect("finalize");
    }

    fn write_overclaimed_wav(path: &Path) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&44_100_u32.to_le_bytes());
        bytes.extend_from_slice(&(44_100_u32 * 2 * 2).to_le_bytes());
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&[0_u8; 64]);
        std::fs::write(path, bytes).expect("write overclaimed wav");
    }

    #[test]
    fn estimated_decode_capacity_is_bounded_by_file_size() {
        assert_eq!(
            estimated_decode_capacity(Some(u32::MAX as u64), 2, 256),
            2_048
        );
        assert_eq!(estimated_decode_capacity(Some(10), 2, 10_000), 20);
        assert_eq!(estimated_decode_capacity(None, 2, 256), 0);
    }

    #[test]
    fn decoded_channel_count_reconciliation_rejects_mid_stream_changes() {
        let mut observed = None;
        assert_eq!(
            reconcile_decoded_channel_count(&mut observed, 1).expect("first channel count"),
            1
        );
        assert_eq!(
            reconcile_decoded_channel_count(&mut observed, 1).expect("same channel count"),
            1
        );

        let err = reconcile_decoded_channel_count(&mut observed, 2).expect_err("changed channels");
        assert!(matches!(
            err,
            CommandError::Decode(message)
                if message.contains("decoded channel count changed from 1 to 2")
        ));
    }

    #[test]
    fn decode_full_returns_for_tiny_wav_with_absurd_frame_claim() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("overclaimed.wav");
        write_overclaimed_wav(&p);
        let result = std::panic::catch_unwind(|| decode_full(&p));
        assert!(result.is_ok(), "decode_full panicked");
    }

    #[test]
    fn probe_sample_rate_reads_header_rate() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("probe.wav");
        write_silence_wav(&p, 44_100);
        assert_eq!(probe_sample_rate(&p).expect("probe"), 44_100);
    }

    #[test]
    fn probe_audio_format_reads_header_rate_and_channels() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("probe-format.wav");
        write_silence_wav(&p, 48_000);
        let probed = probe_audio_format(&p).expect("probe format");
        assert_eq!(probed.sample_rate, 48_000);
        assert_eq!(probed.channels, 1);
    }

    // Unsupported / garbage input must surface as a Decode error, never a
    // panic, on every entry point.
    #[test]
    fn garbage_input_is_a_decode_error_not_a_panic() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("garbage.wav");
        std::fs::write(&p, b"this is definitely not an audio container").expect("write garbage");
        assert!(matches!(decode_full(&p), Err(CommandError::Decode(_))));
        assert!(matches!(
            decode_to_peaks(&p, 100),
            Err(CommandError::Decode(_))
        ));
        assert!(matches!(
            probe_sample_rate(&p),
            Err(CommandError::Decode(_))
        ));
    }

    #[test]
    fn decode_full_round_trips_a_known_wav() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("silence.wav");
        write_silence_wav(&p, 44_100);
        let pcm = decode_full(&p).expect("decode");
        assert_eq!(pcm.channels, 1);
        assert_eq!(pcm.sample_rate, 44_100);
        assert_eq!(pcm.samples.len(), 1024);
    }

    /// 2026-07-06 audit: the mastering chain is stereo-native (multiband
    /// compressor and width process channels 0-1 only), so a 5.1 source
    /// decoded as-is got its front pair mastered while center/surrounds
    /// passed through untouched. Decode now folds above-stereo sources to
    /// stereo with the same downmix the album delivery path uses.
    #[test]
    fn decode_full_folds_above_stereo_sources_to_stereo() {
        // One constant 5.1 frame (L, R, C, LFE, Ls, Rs), repeated.
        const FRAME_I16: [i16; 6] = [8_192, -4_096, 6_144, 16_384, 2_048, -2_048];
        const FRAMES: usize = 256;
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("surround.wav");
        let spec = hound::WavSpec {
            channels: 6,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&p, spec).expect("create");
        for _ in 0..FRAMES {
            for s in FRAME_I16 {
                w.write_sample(s).expect("write");
            }
        }
        w.finalize().expect("finalize");

        let pcm = decode_full(&p).expect("decode");
        assert_eq!(
            pcm.channels, 2,
            "above-stereo must fold to stereo at decode"
        );
        assert_eq!(pcm.samples.len(), FRAMES * 2);

        // The fold must be the shared album downmix applied to the decoded
        // (i16-quantized) frame — LFE excluded, surrounds at -3 dB.
        let frame_f32: Vec<f32> = FRAME_I16.iter().map(|&s| s as f32 / 32_768.0).collect();
        let expected = downmix_frame_to_stereo(&frame_f32);
        let mid = FRAMES; // an even sample index = a left sample mid-buffer
        assert!(
            (pcm.samples[mid] - expected[0]).abs() < 1.0e-3
                && (pcm.samples[mid + 1] - expected[1]).abs() < 1.0e-3,
            "fold must match downmix_frame_to_stereo: got ({}, {}), want ({}, {})",
            pcm.samples[mid],
            pcm.samples[mid + 1],
            expected[0],
            expected[1]
        );

        // Header probes keep reporting the file's real channel count — the
        // import UI and album records rely on that for honesty.
        let probed = probe_audio_format(&p).expect("probe");
        assert_eq!(probed.channels, 6);
    }
}
