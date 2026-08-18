//! Hostile-input pins for the album render path (hardening plan A5).
//!
//! Every malformed request must produce a typed, honest `CommandError` —
//! never a panic, never silent partial output. These tests pin the
//! existing defenses so they cannot regress.

use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use yes_master_lib::album_render::render_album_plan_impl;
use yes_master_lib::engine::{AlbumPlanRenderRequest, AlbumTrackRenderInput};
use yes_master_lib::types::{
    AdvancedSettings, AlbumArc, AlbumPlan, AlbumTrackEntry, CommandError, DeliveryProfile,
    MasteringSettings, Preset, TrackId, TrackRole,
};

fn write_stereo_sine_wav(path: &PathBuf, sample_rate: u32, duration_sec: f32, amplitude: f32) {
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).expect("create wav");
    let n_frames = (sample_rate as f32 * duration_sec) as u32;
    let omega = 2.0 * std::f32::consts::PI * 1_000.0 / sample_rate as f32;
    for i in 0..n_frames {
        let v = amplitude * (omega * i as f32).sin();
        let s = (v.clamp(-1.0, 1.0) * 32_767.0).round() as i16;
        writer.write_sample(s).expect("write L");
        writer.write_sample(s).expect("write R");
    }
    writer.finalize().expect("finalize source wav");
}

fn settings() -> MasteringSettings {
    MasteringSettings {
        preset: Preset::Universal,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: yes_master_lib::EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::StreamingUniversal,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

fn entry(id: &str, position: u32) -> AlbumTrackEntry {
    AlbumTrackEntry {
        track_id: TrackId(id.to_string()),
        position,
        role: TrackRole::AlbumTrack,
        role_locked: false,
        arc_lufs_offset_db: 0.0,
        intensity_scale: 1.0,
        album_character: None,
    }
}

fn input(id: &str, source_path: &str) -> AlbumTrackRenderInput {
    AlbumTrackRenderInput {
        track_id: TrackId(id.to_string()),
        source_path: source_path.to_string(),
        settings: settings(),
        override_album: false,
    }
}

fn plan(entries: Vec<AlbumTrackEntry>) -> AlbumPlan {
    let n = entries.len();
    AlbumPlan {
        title: "Hostile".to_string(),
        arc: AlbumArc::Custom {
            lufs_offsets: vec![0.0; n],
        },
        tracks: entries,
        transitions: vec![],
        intensity: 1.0,
        delivery_sample_rate: None,
        delivery_bit_depth: None,
    }
}

#[test]
fn zero_track_plan_is_a_typed_error() {
    let tmp = TempDir::new().expect("tempdir");
    let err = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan: plan(vec![]),
            tracks: vec![],
        },
        &tmp.path().join("out"),
        None,
    )
    .expect_err("empty plan must error");
    assert!(
        matches!(err, CommandError::Other(ref m) if m.contains("no tracks")),
        "unexpected error: {err:?}"
    );
}

#[test]
fn plan_referencing_unknown_track_is_a_typed_error() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("real.wav");
    write_stereo_sine_wav(&src, 44_100, 1.0, 0.1);
    // Plan asks for "ghost" but only "real" was provided.
    let err = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan: plan(vec![entry("ghost", 1)]),
            tracks: vec![input("real", &src.to_string_lossy())],
        },
        &tmp.path().join("out"),
        None,
    )
    .expect_err("unknown track_id must error");
    assert!(
        matches!(err, CommandError::Other(ref m) if m.contains("ghost")),
        "error should name the offending track_id: {err:?}"
    );
}

#[test]
fn missing_source_file_is_a_typed_error() {
    let tmp = TempDir::new().expect("tempdir");
    let gone = tmp.path().join("never-existed.wav");
    let err = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan: plan(vec![entry("gone", 1)]),
            tracks: vec![input("gone", &gone.to_string_lossy())],
        },
        &tmp.path().join("out"),
        None,
    )
    .expect_err("missing source must error");
    assert!(
        matches!(err, CommandError::Io(ref m) if m.contains("source not found")),
        "unexpected error: {err:?}"
    );
}

#[test]
fn empty_source_path_is_a_typed_error() {
    // The frontend falls back to "" when a plan entry has no matching
    // imported track (useTrackMaster.exportAlbumPlan) — the backend must
    // catch it before any decode attempt.
    let tmp = TempDir::new().expect("tempdir");
    let err = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan: plan(vec![entry("blank", 1)]),
            tracks: vec![input("blank", "")],
        },
        &tmp.path().join("out"),
        None,
    )
    .expect_err("empty source path must error");
    assert!(
        matches!(err, CommandError::InvalidPath(ref m) if m.contains("empty path")),
        "unexpected error: {err:?}"
    );
}

#[test]
fn mid_album_failure_leaves_no_partial_per_track_files() {
    // Track 1 renders fine; track 2 is a structurally valid WAV containing
    // zero samples, which passes the header probe but fails in-loop with
    // "no samples decoded" — AFTER track 1's per-track WAV was written.
    // A failed album export must not leave that orphan behind.
    let tmp = TempDir::new().expect("tempdir");
    let good = tmp.path().join("good.wav");
    write_stereo_sine_wav(&good, 44_100, 1.0, 0.1);
    let empty = tmp.path().join("empty.wav");
    {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        WavWriter::create(&empty, spec)
            .expect("create empty wav")
            .finalize()
            .expect("finalize empty wav");
    }

    let out_dir = tmp.path().join("out");
    let err = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan: plan(vec![entry("good", 1), entry("empty", 2)]),
            tracks: vec![
                input("good", &good.to_string_lossy()),
                input("empty", &empty.to_string_lossy()),
            ],
        },
        &out_dir,
        None,
    )
    .expect_err("zero-sample track must fail the album render");
    assert!(
        matches!(err, CommandError::Decode(ref m) if m.contains("no samples decoded")),
        "unexpected error: {err:?}"
    );

    // No orphaned per-track WAVs (or any files) left in the export folder.
    let leftovers: Vec<String> = std::fs::read_dir(&out_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        leftovers.is_empty(),
        "failed export must not leave partial output behind, found: {leftovers:?}"
    );
}

#[test]
fn duplicate_plan_entries_render_the_same_source_twice_coherently() {
    // Two plan entries sharing one track_id (a "reprise") is tolerated:
    // the same source renders at both positions with distinct filenames.
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("reprise.wav");
    write_stereo_sine_wav(&src, 44_100, 1.0, 0.1);
    let report = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan: plan(vec![entry("reprise", 1), entry("reprise", 2)]),
            tracks: vec![input("reprise", &src.to_string_lossy())],
        },
        &tmp.path().join("out"),
        None,
    )
    .expect("duplicate entries should render");
    assert_eq!(report.tracks.len(), 2);
    assert_ne!(
        report.tracks[0].output_path, report.tracks[1].output_path,
        "both renders must exist under distinct names"
    );
    assert!(Path::new(&report.tracks[0].output_path).exists());
    assert!(Path::new(&report.tracks[1].output_path).exists());
}
