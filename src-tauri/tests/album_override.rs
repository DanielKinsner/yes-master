//! Owner decision 2026-07-03 D9 — Override = "full sound exemption".
//!
//! The album Override toggle promises "its own settings will be applied at
//! export." Pre-fix, `track_override_album` never reached the Rust render
//! path, so an overridden track's personal settings were still shadowed
//! with the arc LUFS offset (and, pre-gate, character biases). This test
//! pins the fixed contract end-to-end:
//!
//!   * An overridden track lands at its OWN loudness target, ignoring the
//!     album arc offset entirely.
//!   * A non-overridden sibling still follows the arc-shifted album
//!     intent.
//!   * The render report (and therefore the manifest) marks each track's
//!     override state honestly.

use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use yes_master_lib::album_render::render_album_plan_impl;
use yes_master_lib::engine::{
    measure_integrated_lufs_at_path, AlbumPlanRenderRequest, AlbumTrackRenderInput,
};
use yes_master_lib::types::{
    AdvancedSettings, AlbumArc, AlbumPlan, AlbumTrackEntry, DeliveryProfile, MasteringSettings,
    Preset, TrackId, TrackRole,
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

/// Album intent: Streaming Universal (-14 LUFS / -1 dBTP).
fn album_intent_settings() -> MasteringSettings {
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
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::StreamingUniversal,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

/// The overridden track's OWN settings: Custom delivery with an explicit
/// -11 LUFS target, 3 dB above the album intent — far enough from both
/// the album target (-14) and the arc-shifted target (-17) that a landing
/// anywhere else fails the assertion unambiguously.
fn own_settings() -> MasteringSettings {
    let mut s = album_intent_settings();
    s.delivery_profile = DeliveryProfile::Custom;
    s.advanced.lufs_offset_db = Some(-11.0);
    s.advanced.ceiling_dbtp = Some(-1.0);
    s
}

fn entry(id: &str, position: u32, arc_offset: f32) -> AlbumTrackEntry {
    AlbumTrackEntry {
        track_id: TrackId(id.to_string()),
        position,
        role: TrackRole::AlbumTrack,
        role_locked: false,
        arc_lufs_offset_db: arc_offset,
        intensity_scale: 1.0,
        album_character: None,
    }
}

#[test]
fn override_track_lands_at_own_target_while_sibling_follows_arc() {
    let tmp = TempDir::new().expect("tempdir");
    let sr: u32 = 48_000;

    let own_path = tmp.path().join("override-me.wav");
    let follow_path = tmp.path().join("follows-album.wav");
    write_stereo_sine_wav(&own_path, sr, 4.0, 0.10);
    write_stereo_sine_wav(&follow_path, sr, 4.0, 0.10);

    // Both plan entries carry a -3 dB arc offset. If the override leaks,
    // the overridden track lands near -17 (album -14 + arc -3) or -14
    // (own target replaced) instead of its own -11.
    let plan = AlbumPlan {
        title: "Override Exemption Test".to_string(),
        arc: AlbumArc::Custom {
            lufs_offsets: vec![-3.0, -3.0],
        },
        tracks: vec![entry("own", 1, -3.0), entry("follows", 2, -3.0)],
        transitions: vec![],
        intensity: 1.0,
        delivery_sample_rate: None,
        delivery_bit_depth: None,
    };

    let request = AlbumPlanRenderRequest {
        plan,
        tracks: vec![
            AlbumTrackRenderInput {
                track_id: TrackId("own".to_string()),
                source_path: own_path.to_string_lossy().to_string(),
                settings: own_settings(),
                override_album: true,
            },
            AlbumTrackRenderInput {
                track_id: TrackId("follows".to_string()),
                source_path: follow_path.to_string_lossy().to_string(),
                settings: album_intent_settings(),
                override_album: false,
            },
        ],
    };

    let out_dir = tmp.path().join("out");
    let report = render_album_plan_impl(&request, &out_dir, None).expect("album plan render");
    assert_eq!(report.tracks.len(), 2);

    let mut by_pos = report.tracks.clone();
    by_pos.sort_by_key(|t| t.position);

    // Honest receipt: the report marks override state per track.
    assert!(
        by_pos[0].override_album,
        "report must mark the overridden track"
    );
    assert!(
        !by_pos[1].override_album,
        "report must not mark the album-intent track"
    );

    // D9: the overridden track lands at its OWN -11 target — not the
    // album's -14, not the arc-shifted -17.
    let own_lufs =
        measure_integrated_lufs_at_path(Path::new(&by_pos[0].output_path)).expect("own LUFS");
    assert!(
        (own_lufs - (-11.0)).abs() < 0.5,
        "override track should land at its own -11 LUFS target, got {own_lufs:.2}"
    );

    // The sibling still follows the album intent + arc: -14 - 3 = -17.
    let follow_lufs =
        measure_integrated_lufs_at_path(Path::new(&by_pos[1].output_path)).expect("follow LUFS");
    assert!(
        (follow_lufs - (-17.0)).abs() < 0.5,
        "album-intent track should land at the arc-shifted -17 LUFS, got {follow_lufs:.2}"
    );
}
