//! Hostile decode-input corpus (hardening plan D1).
//!
//! Anything a stranger can name `.wav` must produce a typed
//! `CommandError` or sane, finite PCM — never a panic, never NaN/Inf
//! escaping into the engine, never an unbounded allocation. All fixtures
//! are generated in-test (no binaries in git).

use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use yes_master_lib::decode::{decode_full, probe_audio_format};

mod common;
use common::crafted_wav;

fn write_bytes(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(bytes).expect("write");
    path
}

/// The invariant every hostile case must satisfy: no panic (implicit),
/// and EITHER a typed error OR finite, bounded PCM.
fn assert_error_or_sane(path: &Path, label: &str) {
    match decode_full(path) {
        Err(_) => {} // typed error is a fine outcome
        Ok(pcm) => {
            assert!(pcm.sample_rate > 0, "{label}: zero sample rate escaped");
            assert!(pcm.channels > 0, "{label}: zero channels escaped");
            assert!(
                pcm.samples.len() < 50_000_000,
                "{label}: absurd allocation ({} samples) from a tiny file",
                pcm.samples.len(),
            );
            for (i, s) in pcm.samples.iter().enumerate() {
                assert!(s.is_finite(), "{label}: non-finite sample at {i}");
                assert!(s.abs() <= 64.0, "{label}: unclamped sample {s} at {i}");
            }
        }
    }
}

#[test]
fn zero_byte_file() {
    let tmp = TempDir::new().expect("tempdir");
    let path = write_bytes(&tmp, "empty.wav", &[]);
    assert!(decode_full(&path).is_err(), "zero-byte file must error");
    assert!(probe_audio_format(&path).is_err());
}

#[test]
fn garbage_bytes() {
    let tmp = TempDir::new().expect("tempdir");
    let garbage: Vec<u8> = (0..4096u32)
        .map(|i| (i.wrapping_mul(37) % 251) as u8)
        .collect();
    let path = write_bytes(&tmp, "garbage.wav", &garbage);
    assert_error_or_sane(&path, "garbage");
}

#[test]
fn truncated_riff_header() {
    let tmp = TempDir::new().expect("tempdir");
    let full = crafted_wav(2, 44_100, 16, &[0u8; 400], None);
    let path = write_bytes(&tmp, "truncated.wav", &full[..30]);
    assert_error_or_sane(&path, "truncated header");
}

#[test]
fn zero_channel_fmt() {
    let tmp = TempDir::new().expect("tempdir");
    let bytes = crafted_wav(0, 44_100, 16, &[0u8; 400], None);
    let path = write_bytes(&tmp, "zero-ch.wav", &bytes);
    assert_error_or_sane(&path, "zero channels");
}

#[test]
fn zero_sample_rate_fmt() {
    let tmp = TempDir::new().expect("tempdir");
    let bytes = crafted_wav(2, 0, 16, &[0u8; 400], None);
    let path = write_bytes(&tmp, "zero-rate.wav", &bytes);
    // Whatever symphonia thinks of a 0 Hz fmt, a 0 rate must never
    // escape into the engine (filter math would divide by it).
    assert_error_or_sane(&path, "zero sample rate");
    if let Ok(probed) = probe_audio_format(&path) {
        assert!(probed.sample_rate > 0, "probe let a 0 Hz rate escape");
    }
}

#[test]
fn absurd_sample_rate_fmt() {
    let tmp = TempDir::new().expect("tempdir");
    let bytes = crafted_wav(2, 4_294_000_000, 16, &[0u8; 4000], None);
    let path = write_bytes(&tmp, "absurd-rate.wav", &bytes);
    assert_error_or_sane(&path, "absurd sample rate");
}

#[test]
fn data_chunk_length_lie() {
    // Claims ~2 GB of data, ships 400 bytes: decode must stay bounded by
    // the real file size (no OOM pre-allocation, no hang).
    let tmp = TempDir::new().expect("tempdir");
    let bytes = crafted_wav(2, 44_100, 16, &[0u8; 400], Some(0x7FFF_FFF0));
    let path = write_bytes(&tmp, "length-lie.wav", &bytes);
    assert_error_or_sane(&path, "data length lie");
}

#[test]
fn nan_and_inf_in_float_wav_are_sanitized() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("nan.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44_100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(&path, spec).expect("create");
    for s in [
        0.5_f32,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        1.0e30,
        -0.25,
    ] {
        w.write_sample(s).expect("write");
    }
    w.finalize().expect("finalize");

    let pcm = decode_full(&path).expect("float wav decodes");
    assert_eq!(pcm.samples.len(), 6);
    for (i, s) in pcm.samples.iter().enumerate() {
        assert!(s.is_finite(), "sample {i} must be finite, got {s}");
    }
    // Real content is preserved, garbage is neutralized, huge finite
    // values are clamped into the engine's headroom.
    assert!((pcm.samples[0] - 0.5).abs() < 1.0e-6);
    assert_eq!(pcm.samples[1], 0.0, "NaN must become silence");
    assert_eq!(pcm.samples[2], 0.0, "+Inf must become silence");
    assert_eq!(pcm.samples[3], 0.0, "-Inf must become silence");
    assert_eq!(pcm.samples[4], 64.0, "1e30 must clamp to +64");
    assert!((pcm.samples[5] - (-0.25)).abs() < 1.0e-6);
}
