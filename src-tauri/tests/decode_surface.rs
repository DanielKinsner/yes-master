//! Decode-surface pin (hardening plan D2).
//!
//! The import UI advertises a fixed extension list
//! (`src/lib/supported-formats.ts`: wav, mp3, m4a, aac, flac, ogg). Every
//! advertised format must actually decode end-to-end. Audit finding F7
//! claimed raw ADTS `.aac` was broken for lack of an `adts` cargo feature;
//! this test REFUTED that — symphonia 0.5's `aac` feature already
//! registers the AdtsReader — and stays as the permanent proof that the
//! UI's format promise holds.
//!
//! WAV is pinned unconditionally (generated with hound). The compressed
//! formats are generated with ffmpeg when it is available on the machine
//! (owner boxes and any CI runner with ffmpeg get full coverage); without
//! ffmpeg those cases skip with a notice rather than fail.

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use yes_master_lib::decode::decode_full;

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate one second of a 440 Hz stereo tone into `path` via ffmpeg.
fn ffmpeg_tone(path: &Path, extra_args: &[&str]) -> bool {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:duration=1:sample_rate=44100",
        "-ac",
        "2",
    ]);
    cmd.args(extra_args);
    cmd.arg(path);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

fn assert_decodes_one_second(path: &Path, label: &str) {
    let pcm = decode_full(path).unwrap_or_else(|e| panic!("{label} must decode: {e:?}"));
    assert!(pcm.sample_rate > 0, "{label}: sample rate");
    assert!(pcm.channels > 0, "{label}: channels");
    let frames = pcm.samples.len() / pcm.channels as usize;
    let seconds = frames as f32 / pcm.sample_rate as f32;
    assert!(
        (0.8..=1.3).contains(&seconds),
        "{label}: expected ~1 s of audio, got {seconds:.2} s",
    );
    let peak = pcm.samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    assert!(peak > 0.05, "{label}: decoded audio should be non-silent");
}

#[test]
fn wav_decodes() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("tone.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).expect("create");
    for i in 0..44_100 {
        let s = (0.3 * (i as f32 * 2.0 * std::f32::consts::PI * 440.0 / 44_100.0).sin() * 32_767.0)
            as i16;
        w.write_sample(s).expect("L");
        w.write_sample(s).expect("R");
    }
    w.finalize().expect("finalize");
    assert_decodes_one_second(&path, "wav");
}

#[test]
fn every_advertised_compressed_format_decodes() {
    if !ffmpeg_available() {
        eprintln!("SKIP: ffmpeg not available — compressed-format pins not run");
        return;
    }
    let tmp = TempDir::new().expect("tempdir");
    // (extension, extra ffmpeg args) — one entry per non-wav extension in
    // src/lib/supported-formats.ts AUDIO_EXTENSIONS.
    let cases: &[(&str, &[&str])] = &[
        ("mp3", &[]),
        ("m4a", &["-c:a", "aac"]),
        // Raw ADTS stream — the F7 audit claim said this couldn't decode;
        // it can (AdtsReader ships with the `aac` feature). Keep it pinned.
        ("aac", &["-c:a", "aac", "-f", "adts"]),
        ("flac", &[]),
        ("ogg", &["-c:a", "libvorbis"]),
    ];
    for (ext, args) in cases {
        let path = tmp.path().join(format!("tone.{ext}"));
        assert!(
            ffmpeg_tone(&path, args),
            "ffmpeg failed to generate the {ext} fixture",
        );
        assert_decodes_one_second(&path, ext);
    }
}
