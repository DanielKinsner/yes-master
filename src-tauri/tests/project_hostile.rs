//! Hostile session/project JSON corpus (hardening plan D3).
//!
//! `.ams.json` files arrive from a native file-open dialog — anything on
//! disk can be handed to `load_project`, and the autosaved `session.json`
//! can be corrupted by crashes, sync tools, or hand edits. The contract:
//! malformed bytes produce a typed `CommandError` (never a panic, never a
//! stack overflow), and a *well-formed* project carrying hostile values
//! must not poison the audio chain — extreme numbers get clamped or
//! neutralized on the way in (`clamp_finite_or` / `sanitize_shelf_gain_db`
//! in dsp.rs), so the render stays finite.

use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;
use yes_master_lib::dsp::MasteringChain;
use yes_master_lib::project::{load_project, read_session, save_project};
use yes_master_lib::types::ProjectState;

fn write_file(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(bytes).expect("write");
    path
}

fn load(path: &std::path::Path) -> Result<ProjectState, yes_master_lib::types::CommandError> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("tokio runtime")
        .block_on(load_project(path.to_string_lossy().into_owned()))
}

// ---------------------------------------------------------------------------
// Malformed bytes → typed error, never a panic
// ---------------------------------------------------------------------------

#[test]
fn zero_byte_project_file_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(&tmp, "empty.ams.json", b"");
    assert!(load(&path).is_err(), "zero-byte project must error");
}

#[test]
fn garbage_bytes_error() {
    let tmp = TempDir::new().expect("tempdir");
    let garbage: Vec<u8> = (0..8192u32)
        .map(|i| (i.wrapping_mul(31) % 251) as u8)
        .collect();
    let path = write_file(&tmp, "garbage.ams.json", &garbage);
    assert!(load(&path).is_err(), "binary garbage must error");
}

#[test]
fn truncated_json_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(
        &tmp,
        "truncated.ams.json",
        br#"{"schema_version": 1, "mode": "tra"#,
    );
    assert!(load(&path).is_err(), "truncated JSON must error");
}

#[test]
fn type_confusion_errors() {
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(
        &tmp,
        "types.ams.json",
        br#"{"schema_version": "one", "mode": 42, "tracks": "none"}"#,
    );
    assert!(load(&path).is_err(), "wrong field types must error");
}

#[test]
fn nan_literal_is_rejected_as_invalid_json() {
    // JSON has no NaN literal; a file carrying one must fail the parse
    // (typed error) rather than smuggle a NaN into MasteringSettings.
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(&tmp, "nan.ams.json", br#"{"schema_version": NaN}"#);
    assert!(load(&path).is_err(), "NaN literal must be a parse error");
}

#[test]
fn deep_nesting_bomb_errors_without_stack_overflow() {
    // 100k opening brackets: serde_json's recursion limit must convert
    // this into a typed error, not a stack overflow (which would abort
    // the whole process — the classic parser DoS).
    let tmp = TempDir::new().expect("tempdir");
    let mut bomb = Vec::with_capacity(200_012);
    bomb.extend_from_slice(br#"{"schema_version": "#);
    bomb.extend(std::iter::repeat(b'[').take(100_000));
    let path = write_file(&tmp, "bomb.ams.json", &bomb);
    assert!(load(&path).is_err(), "nesting bomb must be a typed error");
}

#[test]
fn empty_and_traversal_paths_are_rejected() {
    assert!(load(&PathBuf::new()).is_err(), "empty path must error");
    // Forward slashes are path separators on every OS; backslashes are
    // separators only on Windows (on Unix they're ordinary filename bytes,
    // so "..\\.." is a single component and not a traversal there).
    let evil = PathBuf::from("../../evil.ams.json");
    let err = load(&evil).expect_err("traversal path must error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("traversal"),
        "traversal rejection should be explicit, got: {msg}",
    );
    #[cfg(windows)]
    {
        let evil = PathBuf::from("..\\..\\evil.ams.json");
        let err = load(&evil).expect_err("backslash traversal path must error");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("traversal"),
            "traversal rejection should be explicit, got: {msg}",
        );
    }
}

// ---------------------------------------------------------------------------
// Well-formed but hostile values → clamped, and the chain stays finite
// ---------------------------------------------------------------------------

/// A syntactically valid project whose settings carry the worst numbers
/// JSON can express: 1e39 overflows f32 to +inf on deserialize, gains sit
/// nine decades past the UI range, intensity is far above the slider.
const HOSTILE_SETTINGS_PROJECT: &str = r#"{
  "schema_version": 1,
  "mode": "track",
  "tracks": [{
    "id": "t1",
    "path": "C:/music/song.wav",
    "display_name": "song",
    "source_format": "wav",
    "duration_seconds": 1.0,
    "sample_rate": 44100,
    "channels": 2
  }],
  "track_order": ["t1"],
  "track_settings": {
    "t1": {
      "preset": {"kind": "loud"},
      "intensity": 99.0,
      "eq_sub_db": 1e39,
      "eq_low_db": -1e39,
      "eq_low_mid_db": 480.0,
      "eq_mid_db": -480.0,
      "eq_high_mid_db": 1e30,
      "eq_high_db": -1e30,
      "eq_sparkle_db": 1e39,
      "volume_match": false,
      "source_lufs_integrated": null,
      "input_gain_db": 1e9,
      "output_gain_db": -1e9,
      "delivery_profile": "custom",
      "album": null,
      "advanced": {
        "stereo_width": 5.0,
        "warmth": -3.0,
        "presence_air": 7.0,
        "compression_density": 9.0
      }
    }
  },
  "album_intent": null,
  "album_arc_kind": "cinematic",
  "album_intensity": 1.0,
  "album_title": "",
  "album_sample_rate": null,
  "album_bit_depth": null,
  "track_override_album": [],
  "last_saved_iso": null
}"#;

#[test]
fn hostile_settings_load_and_render_finite_audio() {
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(
        &tmp,
        "hostile.ams.json",
        HOSTILE_SETTINGS_PROJECT.as_bytes(),
    );
    let state = load(&path).expect("well-formed JSON must load");

    let settings = state
        .track_settings
        .get("t1")
        .expect("settings present")
        .clone();

    // The chain must swallow these values (clamp_finite_or, shelf-gain
    // sanitization, width/warmth clamps) and still produce finite audio.
    let sr = 48_000_u32;
    let mut chain = MasteringChain::new(sr, 2, &settings);
    let mut buf: Vec<f32> = (0..sr as usize)
        .flat_map(|n| {
            let s = 0.25 * (2.0 * std::f32::consts::PI * 220.0 * n as f32 / sr as f32).sin();
            [s, s]
        })
        .collect();
    chain.process_interleaved(&mut buf, 2);
    chain.flush_render_tail(&mut buf, 2);

    for (i, s) in buf.iter().enumerate() {
        assert!(
            s.is_finite(),
            "hostile session poisoned the chain: non-finite sample at {i}",
        );
    }
    let peak = buf.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    assert!(
        peak <= 4.0,
        "hostile session produced runaway gain: peak {peak} (linear)",
    );
}

// ---------------------------------------------------------------------------
// U13 — enormous FINITE values (not NaN/Inf) on the compressor and delivery
// surfaces.
//
// The corpus above proves NaN/Inf are neutralized. Finite-but-absurd is a
// different failure mode and reaches the chain through a different door: a
// hand-edited or sync-corrupted `.ams.json` can carry a 1e12 ratio or a
// 5-hour release, which are perfectly valid f32s. `finite_or` passes those
// straight through.
//
// Compatibility behavior pinned by these tests: **preserved in file, clamped
// at execution.** The project file keeps whatever it carried (so nothing is
// silently rewritten under the user, and a file authored by a future version
// with a wider range is not destroyed), and the chain clamps to the current UI
// range when it builds coefficients. Load does not normalize, and load does not
// reject.
// ---------------------------------------------------------------------------

/// Every per-band compressor override at an absurd-but-finite value, plus a
/// delivery target and ceiling far outside anything the UI can produce.
const HOSTILE_FINITE_COMPRESSOR_PROJECT: &str = r#"{
  "schema_version": 1,
  "mode": "track",
  "tracks": [{
    "id": "t1",
    "path": "C:/music/song.wav",
    "display_name": "song",
    "source_format": "wav",
    "duration_seconds": 1.0,
    "sample_rate": 44100,
    "channels": 2
  }],
  "track_order": ["t1"],
  "track_settings": {
    "t1": {
      "preset": {"kind": "universal"},
      "intensity": 0.5,
      "eq_sub_db": 0.0,
      "eq_low_db": 0.0,
      "eq_low_mid_db": 0.0,
      "eq_mid_db": 0.0,
      "eq_high_mid_db": 0.0,
      "eq_high_db": 0.0,
      "eq_sparkle_db": 0.0,
      "volume_match": false,
      "source_lufs_integrated": null,
      "input_gain_db": 0.0,
      "output_gain_db": 0.0,
      "delivery_profile": "custom",
      "album": null,
      "advanced": {
        "compression_mode": "manual",
        "lufs_offset_db": 1.0e12,
        "ceiling_dbtp": 9.0e11,
        "compression_low_threshold_db": -9.0e11,
        "compression_low_ratio": 1.0e12,
        "compression_low_attack_ms": 1.0e12,
        "compression_low_release_ms": 1.0e12,
        "compression_mid_threshold_db": 9.0e11,
        "compression_mid_ratio": 5.0e11,
        "compression_mid_attack_ms": 8.64e7,
        "compression_mid_release_ms": 8.64e7,
        "compression_high_threshold_db": -1.0e12,
        "compression_high_ratio": 2.0e12,
        "compression_high_attack_ms": 0.0,
        "compression_high_release_ms": 0.0
      }
    }
  },
  "album_intent": null,
  "album_arc_kind": "cinematic",
  "album_intensity": 1.0,
  "album_title": "",
  "album_sample_rate": null,
  "album_bit_depth": null,
  "track_override_album": [],
  "last_saved_iso": null
}"#;

fn render_sine_through(settings: &yes_master_lib::types::MasteringSettings) -> Vec<f32> {
    let sr = 48_000_u32;
    let mut chain = MasteringChain::new(sr, 2, settings);
    let mut buf: Vec<f32> = (0..sr as usize)
        .flat_map(|n| {
            let s = 0.25 * (2.0 * std::f32::consts::PI * 220.0 * n as f32 / sr as f32).sin();
            [s, s]
        })
        .collect();
    chain.process_interleaved(&mut buf, 2);
    chain.flush_render_tail(&mut buf, 2);
    buf
}

#[test]
fn enormous_finite_compressor_values_render_finite_bounded_audio() {
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(
        &tmp,
        "hostile-finite.ams.json",
        HOSTILE_FINITE_COMPRESSOR_PROJECT.as_bytes(),
    );
    let state = load(&path).expect("well-formed JSON must load");
    let settings = state
        .track_settings
        .get("t1")
        .expect("settings present")
        .clone();

    let buf = render_sine_through(&settings);

    for (i, s) in buf.iter().enumerate() {
        assert!(
            s.is_finite(),
            "enormous finite compressor values poisoned the chain at sample {i}",
        );
    }
    let peak = buf.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    assert!(
        peak <= 4.0,
        "enormous finite compressor values produced runaway gain: peak {peak} (linear)",
    );
    // A silent output would also be "finite and bounded" while being a
    // different bug, so require the chain to still pass audio.
    assert!(
        peak > 1.0e-4,
        "chain collapsed to silence under hostile finite values: peak {peak}",
    );
}

#[test]
fn hostile_finite_values_are_preserved_in_the_file_not_normalized_on_load() {
    // The compatibility decision, pinned: load neither rewrites nor rejects.
    // Clamping happens at execution. If this ever flips to normalize-on-load,
    // this test is where that decision gets made deliberately.
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(
        &tmp,
        "preserve.ams.json",
        HOSTILE_FINITE_COMPRESSOR_PROJECT.as_bytes(),
    );
    let state = load(&path).expect("loads");
    let advanced = &state
        .track_settings
        .get("t1")
        .expect("settings present")
        .advanced;

    assert_eq!(
        advanced.compression_low_ratio,
        Some(1.0e12),
        "load must preserve the stored value verbatim",
    );
    assert_eq!(advanced.compression_low_attack_ms, Some(1.0e12));
    assert_eq!(advanced.lufs_offset_db, Some(1.0e12));
}

#[test]
fn hostile_finite_values_round_trip_unchanged_through_save() {
    let tmp = TempDir::new().expect("tempdir");
    let seed = write_file(
        &tmp,
        "seed-finite.ams.json",
        HOSTILE_FINITE_COMPRESSOR_PROJECT.as_bytes(),
    );
    let state = load(&seed).expect("seed loads");
    let out = tmp.path().join("roundtrip-finite.ams.json");

    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("tokio runtime")
        .block_on(save_project(
            out.to_string_lossy().into_owned(),
            state.clone(),
        ))
        .expect("save succeeds");

    let reloaded = read_session(&out).expect("reload succeeds");
    let advanced = &reloaded
        .track_settings
        .get("t1")
        .expect("settings present")
        .advanced;
    assert_eq!(
        advanced.compression_low_ratio,
        Some(1.0e12),
        "save/load must not quietly normalize a hostile value",
    );
}

#[test]
fn in_range_compressor_overrides_are_untouched_by_the_hostile_clamps() {
    // The clamps must bound only what the UI cannot produce. Anything the user
    // can actually dial has to render byte-identically, or "hostile bounds"
    // would have become a silent retune.
    let tmp = TempDir::new().expect("tempdir");
    let path = write_file(
        &tmp,
        "in-range.ams.json",
        HOSTILE_FINITE_COMPRESSOR_PROJECT
            .replace(
                "\"compression_low_threshold_db\": -9.0e11",
                "\"compression_low_threshold_db\": -24.0",
            )
            .replace(
                "\"compression_low_ratio\": 1.0e12",
                "\"compression_low_ratio\": 3.5",
            )
            .replace(
                "\"compression_low_attack_ms\": 1.0e12",
                "\"compression_low_attack_ms\": 20.0",
            )
            .replace(
                "\"compression_low_release_ms\": 1.0e12",
                "\"compression_low_release_ms\": 250.0",
            )
            .replace(
                "\"compression_mid_threshold_db\": 9.0e11",
                "\"compression_mid_threshold_db\": -18.0",
            )
            .replace(
                "\"compression_mid_ratio\": 5.0e11",
                "\"compression_mid_ratio\": 2.0",
            )
            .replace(
                "\"compression_mid_attack_ms\": 8.64e7",
                "\"compression_mid_attack_ms\": 15.0",
            )
            .replace(
                "\"compression_mid_release_ms\": 8.64e7",
                "\"compression_mid_release_ms\": 200.0",
            )
            .replace(
                "\"compression_high_threshold_db\": -1.0e12",
                "\"compression_high_threshold_db\": -12.0",
            )
            .replace(
                "\"compression_high_ratio\": 2.0e12",
                "\"compression_high_ratio\": 1.8",
            )
            .replace(
                "\"compression_high_attack_ms\": 0.0",
                "\"compression_high_attack_ms\": 10.0",
            )
            .replace(
                "\"compression_high_release_ms\": 0.0",
                "\"compression_high_release_ms\": 120.0",
            )
            .replace("\"lufs_offset_db\": 1.0e12", "\"lufs_offset_db\": -14.0")
            .replace("\"ceiling_dbtp\": 9.0e11", "\"ceiling_dbtp\": -1.0")
            .as_bytes(),
    );
    let state = load(&path).expect("loads");
    let settings = state
        .track_settings
        .get("t1")
        .expect("settings present")
        .clone();

    let rendered = render_sine_through(&settings);

    // Reference render built directly from the same in-range values, i.e. the
    // path that never goes near a clamp.
    let mut reference_settings = settings.clone();
    reference_settings.advanced.compression_low_ratio = Some(3.5);
    let reference = render_sine_through(&reference_settings);

    assert_eq!(rendered.len(), reference.len());
    for (i, (a, b)) in rendered.iter().zip(reference.iter()).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "in-range override render drifted at sample {i}",
        );
    }
}

// ---------------------------------------------------------------------------
// Round-trip: save always re-loads (the autosave promise)
// ---------------------------------------------------------------------------

#[test]
fn save_then_load_round_trips_via_the_command_surface() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("roundtrip.ams.json");
    let state = {
        let tmp_load = write_file(&tmp, "seed.ams.json", HOSTILE_SETTINGS_PROJECT.as_bytes());
        load(&tmp_load).expect("seed loads")
    };

    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("tokio runtime")
        .block_on(save_project(
            path.to_string_lossy().into_owned(),
            state.clone(),
        ))
        .expect("save succeeds");

    let reloaded = read_session(&path).expect("reload succeeds");
    assert_eq!(reloaded.schema_version, state.schema_version);
    assert_eq!(reloaded.tracks.len(), 1);
    assert!(
        reloaded.track_settings.contains_key("t1"),
        "settings survive the round trip",
    );
}
