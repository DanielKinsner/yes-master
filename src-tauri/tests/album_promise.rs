//! Album promise proofs (owner decision 2026-07-03 D4).
//!
//! The Album Master promise is "one coherent record": consistent loudness,
//! one delivery format, honest per-track receipts, nothing silently
//! altered. These tests pin each clause mechanically so a regression in
//! any of them fails CI rather than a customer's record.

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

fn input(id: &str, path: &Path) -> AlbumTrackRenderInput {
    AlbumTrackRenderInput {
        track_id: TrackId(id.to_string()),
        source_path: path.to_string_lossy().to_string(),
        settings: album_intent_settings(),
        override_album: false,
    }
}

/// Promise clause: CONSISTENT LOUDNESS. Sources at wildly different input
/// levels (-26 dBFS peak to -3 dBFS peak), a flat arc, and one album
/// intent must land every track at the album target — and, just as
/// importantly, land them at the SAME level as each other.
#[test]
fn album_tracks_land_consistently_despite_divergent_source_levels() {
    let tmp = TempDir::new().expect("tempdir");
    let sr: u32 = 48_000;

    let specs = [("quiet", 0.05_f32), ("mid", 0.10), ("hot", 0.70)];
    let mut inputs = Vec::new();
    let mut entries = Vec::new();
    for (i, (id, amp)) in specs.iter().enumerate() {
        let path = tmp.path().join(format!("{id}.wav"));
        write_stereo_sine_wav(&path, sr, 4.0, *amp);
        inputs.push(input(id, &path));
        entries.push(entry(id, (i + 1) as u32));
    }

    let plan = AlbumPlan {
        title: "Consistency Proof".to_string(),
        arc: AlbumArc::Custom {
            lufs_offsets: vec![0.0, 0.0, 0.0],
        },
        tracks: entries,
        transitions: vec![],
        intensity: 1.0,
        delivery_sample_rate: None,
        delivery_bit_depth: None,
    };

    let out_dir = tmp.path().join("out");
    let report = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan,
            tracks: inputs,
        },
        &out_dir,
        None,
    )
    .expect("album render");

    let mut landed = Vec::new();
    for record in &report.tracks {
        let lufs =
            measure_integrated_lufs_at_path(Path::new(&record.output_path)).expect("measure");
        assert!(
            (lufs - (-14.0)).abs() < 0.5,
            "track {} should land at the -14 LUFS album target, got {lufs:.2}",
            record.track_id.as_str(),
        );
        landed.push(lufs);
    }
    let spread = landed.iter().cloned().fold(f32::MIN, f32::max)
        - landed.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        spread < 0.6,
        "album tracks should land within 0.6 LU of each other (one coherent \
         record), got spread {spread:.2} LU across {landed:?}",
    );
}

/// Promise clause: HONEST RECEIPTS. Everything manifest.json claims about
/// the render must match independently re-measured reality: files exist,
/// WAV specs match the claimed delivery format, per-track measured_lufs
/// matches a fresh measurement of the actual file, and the override flag
/// is recorded per track.
#[test]
fn album_manifest_claims_match_remeasured_reality() {
    let tmp = TempDir::new().expect("tempdir");
    let sr: u32 = 44_100;

    let a_path = tmp.path().join("a.wav");
    let b_path = tmp.path().join("b.wav");
    write_stereo_sine_wav(&a_path, sr, 3.0, 0.10);
    write_stereo_sine_wav(&b_path, sr, 3.0, 0.20);

    let mut own = album_intent_settings();
    own.delivery_profile = DeliveryProfile::Custom;
    own.advanced.lufs_offset_db = Some(-12.0);
    own.advanced.ceiling_dbtp = Some(-1.0);

    let mut b_input = input("b", &b_path);
    b_input.settings = own;
    b_input.override_album = true;

    let plan = AlbumPlan {
        title: "Manifest Truth".to_string(),
        arc: AlbumArc::Custom {
            lufs_offsets: vec![0.0, 0.0],
        },
        tracks: vec![entry("a", 1), entry("b", 2)],
        transitions: vec![],
        intensity: 1.0,
        delivery_sample_rate: Some(48_000),
        delivery_bit_depth: Some(24),
    };

    let out_dir = tmp.path().join("out");
    let report = render_album_plan_impl(
        &AlbumPlanRenderRequest {
            plan,
            tracks: vec![input("a", &a_path), b_input],
        },
        &out_dir,
        None,
    )
    .expect("album render");

    // The manifest file the report points at must exist and parse.
    let manifest_raw =
        std::fs::read_to_string(&report.manifest_path).expect("manifest.json exists");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("manifest JSON");

    // Claimed delivery format matches the report and the actual album WAV.
    assert_eq!(manifest["sample_rate"], 48_000);
    assert_eq!(manifest["bit_depth"], 24);
    assert_eq!(manifest["channels"], 2);
    let album_wav = manifest["album_wav_path"].as_str().expect("album path");
    let album_spec = hound::WavReader::open(album_wav)
        .expect("album wav exists")
        .spec();
    assert_eq!(album_spec.sample_rate, 48_000);
    assert_eq!(album_spec.bits_per_sample, 24);
    assert_eq!(album_spec.channels, 2);

    // Per-track claims: file exists, spec matches, LUFS matches an
    // independent re-measurement, override flag is recorded.
    let tracks = manifest["tracks"].as_array().expect("tracks array");
    assert_eq!(tracks.len(), 2);
    for t in tracks {
        let out_path = t["output_path"].as_str().expect("output_path");
        let spec = hound::WavReader::open(out_path)
            .expect("per-track wav exists")
            .spec();
        assert_eq!(spec.sample_rate, 48_000, "{out_path} sample rate");
        assert_eq!(spec.bits_per_sample, 24, "{out_path} bit depth");
        assert_eq!(spec.channels, 2, "{out_path} channels");

        let claimed = t["measured_lufs"].as_f64().expect("measured_lufs") as f32;
        let actual = measure_integrated_lufs_at_path(Path::new(out_path)).expect("re-measure");
        assert!(
            (claimed - actual).abs() < 0.1,
            "{out_path}: manifest claims {claimed:.2} LUFS but the file \
             measures {actual:.2}",
        );

        let is_b = t["track_id"] == "b";
        assert_eq!(
            t["override_album"],
            serde_json::Value::Bool(is_b),
            "override flag must be recorded truthfully for {}",
            t["track_id"],
        );
    }

    // The overridden track really landed at its own -12 target while the
    // album-intent track landed at -14 (double-checking the manifest's
    // numbers describe the exemption honestly).
    let lufs_of = |id: &str| {
        tracks
            .iter()
            .find(|t| t["track_id"] == id)
            .and_then(|t| t["measured_lufs"].as_f64())
            .expect("track lufs") as f32
    };
    assert!((lufs_of("a") - (-14.0)).abs() < 0.5, "a: {}", lufs_of("a"));
    assert!((lufs_of("b") - (-12.0)).abs() < 0.5, "b: {}", lufs_of("b"));
}
