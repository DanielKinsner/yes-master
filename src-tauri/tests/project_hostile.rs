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
