use crate::types::*;
use std::path::{Path, PathBuf};

use tauri::Manager;

const SESSION_FILENAME: &str = "session.json";
const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[tauri::command]
pub async fn save_project(path: String, state: ProjectState) -> CommandResult<()> {
    if path.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    let p = Path::new(&path);
    if crate::files::has_parent_dir_component(p) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {path}"
        )));
    }
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CommandError::Io(e.to_string()))?;
    }
    write_session_atomic(p, &state)
        .inspect_err(|e| crate::diagnostics::error(format!("save_project failed for {path}: {e}")))
}

/// Load a project from an arbitrary `.ams.json` path picked via native
/// file-open dialog. Mirrors `save_project` — same path-traversal guard,
/// same JSON shape (ProjectState).
#[tauri::command]
pub async fn load_project(path: String) -> CommandResult<ProjectState> {
    if path.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    let p = Path::new(&path);
    if crate::files::has_parent_dir_component(p) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {path}"
        )));
    }
    if !p.exists() {
        return Err(CommandError::Io(format!(
            "project file does not exist: {path}"
        )));
    }
    read_session(p)
        .inspect_err(|e| crate::diagnostics::error(format!("load_project failed for {path}: {e}")))
}

#[tauri::command]
pub async fn autosave_session(state: ProjectState, app: tauri::AppHandle) -> CommandResult<()> {
    let path = autosave_path(&app)?;
    write_session_atomic(&path, &state)
        .inspect_err(|e| crate::diagnostics::error(format!("autosave failed: {e}")))
}

#[tauri::command]
pub async fn load_recent_session(app: tauri::AppHandle) -> CommandResult<Option<ProjectState>> {
    let path = autosave_path(&app)?;
    if !path.exists() {
        return Ok(None);
    }
    match read_session(&path) {
        Ok(state) if state.schema_version == SUPPORTED_SCHEMA_VERSION => Ok(Some(state)),
        // Starting with an empty app instead of crashing is right, but the
        // discarded session is the #1 "my tracks disappeared" support case —
        // record WHY it was skipped so a diagnostics report explains it.
        Ok(state) => {
            crate::diagnostics::warn(format!(
                "autosaved session skipped: unsupported schema_version {} (expected {})",
                state.schema_version, SUPPORTED_SCHEMA_VERSION
            ));
            Ok(None)
        }
        Err(e) => {
            crate::diagnostics::error(format!("autosaved session unreadable, starting empty: {e}"));
            Ok(None)
        }
    }
}

fn autosave_path(app: &tauri::AppHandle) -> CommandResult<PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| CommandError::Other(format!("app_data_dir: {e}")))?;
    std::fs::create_dir_all(&app_data).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(app_data.join(SESSION_FILENAME))
}

pub fn write_session_atomic(path: &Path, state: &ProjectState) -> CommandResult<()> {
    let json = serde_json::to_vec_pretty(state)
        .map_err(|e| CommandError::Other(format!("serialize session: {e}")))?;
    // serde_json writes non-finite floats as `null`, and `null` does not
    // parse back into an `f32` field — so a state carrying NaN/inf would
    // save as a file that can never be reloaded. A refused save is
    // recoverable; a session that silently stops reopening is data loss.
    serde_json::from_slice::<ProjectState>(&json).map_err(|e| {
        CommandError::Other(format!(
            "refusing to save a session that could not be reloaded: {e}"
        ))
    })?;
    let tmp_path = session_tmp_path(path);
    std::fs::write(&tmp_path, &json).map_err(|e| CommandError::Io(e.to_string()))?;
    std::fs::rename(&tmp_path, path).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(())
}

fn session_tmp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(SESSION_FILENAME);
    let tmp_name = format!("{name}.{}.tmp", uuid::Uuid::new_v4());
    path.parent()
        .map(|parent| parent.join(&tmp_name))
        .unwrap_or_else(|| PathBuf::from(tmp_name))
}

pub fn read_session(path: &Path) -> CommandResult<ProjectState> {
    let json = std::fs::read(path).map_err(|e| CommandError::Io(e.to_string()))?;
    let mut value: serde_json::Value = serde_json::from_slice(&json)
        .map_err(|e| CommandError::Other(format!("session parse: {e}")))?;
    clamp_floats_to_f32_range(&mut value);
    serde_json::from_value(value).map_err(|e| CommandError::Other(format!("session parse: {e}")))
}

/// A `.ams.json` number like `1e39` is valid JSON (finite as f64) but
/// overflows to `+inf` when serde converts it into an `f32` field — and a
/// non-finite float in `ProjectState` re-serializes as `null`, producing a
/// session file that can never be reloaded (hardening plan D3). Clamping
/// every float in the raw tree into f32-finite range before the typed
/// parse keeps loaded state finite for ALL current and future f32 fields;
/// semantic range clamps stay where they belong (`clamp_finite_or` and
/// friends at chain build). Integers and in-range floats are untouched.
fn clamp_floats_to_f32_range(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Number(n) if n.is_f64() => {
            if let Some(f) = n.as_f64() {
                if f.abs() > f64::from(f32::MAX) {
                    let clamped = f.clamp(-f64::from(f32::MAX), f64::from(f32::MAX));
                    if let Some(num) = serde_json::Number::from_f64(clamped) {
                        *value = serde_json::Value::Number(num);
                    }
                }
            }
        }
        serde_json::Value::Array(items) => {
            items.iter_mut().for_each(clamp_floats_to_f32_range);
        }
        serde_json::Value::Object(map) => {
            map.values_mut().for_each(clamp_floats_to_f32_range);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    fn project_state(id: &str) -> ProjectState {
        let track_id = TrackId(id.to_string());
        ProjectState {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            mode: ProjectMode::Track,
            tracks: vec![ImportedTrack {
                id: track_id.clone(),
                path: format!("C:/music/{id}.wav"),
                display_name: id.to_string(),
                source_format: "wav".to_string(),
                duration_seconds: Some(1.0),
                sample_rate: Some(44_100),
                channels: Some(2),
            }],
            track_order: vec![track_id],
            track_settings: std::collections::HashMap::new(),
            album_intent: None,
            album_arc_kind: AlbumArcKind::Cinematic,
            album_intensity: 1.0,
            album_title: String::new(),
            album_sample_rate: None,
            album_bit_depth: None,
            track_override_album: Vec::new(),
            last_saved_iso: Some(format!("2026-06-12T12:00:00Z-{id}")),
        }
    }

    #[test]
    fn non_finite_state_is_refused_at_save_instead_of_writing_unloadable_json() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("poisoned.ams.json");
        let mut state = project_state("poisoned");
        state.album_intensity = f32::INFINITY;

        let err = write_session_atomic(&path, &state).expect_err("must refuse");
        assert!(
            format!("{err:?}").contains("could not be reloaded"),
            "refusal should say why: {err:?}",
        );
        assert!(!path.exists(), "no file may be written on refusal");
    }

    #[test]
    fn out_of_f32_range_floats_in_the_file_load_clamped_and_finite() {
        // 1e39 is valid JSON but +inf as f32; the loader must clamp it so
        // the loaded state round-trips through save again.
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("overflow.ams.json");
        let mut state = project_state("overflow");
        state.album_intensity = 1.0;
        write_session_atomic(&path, &state).expect("write seed");
        let raw = std::fs::read_to_string(&path).expect("read seed");
        let poisoned = raw.replace("\"album_intensity\": 1.0", "\"album_intensity\": 1e39");
        assert_ne!(raw, poisoned, "fixture must actually poison the field");
        std::fs::write(&path, poisoned).expect("write poisoned");

        let loaded = read_session(&path).expect("hostile-but-valid JSON loads");
        assert!(
            loaded.album_intensity.is_finite(),
            "overflowing float must be clamped finite, got {}",
            loaded.album_intensity,
        );
        // And the loaded state must save again — the full round trip the
        // old code broke.
        let path2 = tmp.path().join("resaved.ams.json");
        write_session_atomic(&path2, &loaded).expect("resave succeeds");
        read_session(&path2).expect("resaved session reloads");
    }

    #[test]
    fn session_tmp_path_is_unique_and_adjacent_to_target() {
        let dir = PathBuf::from("projects");
        let target = dir.join("session.ams.json");

        let first = session_tmp_path(&target);
        let second = session_tmp_path(&target);

        assert_ne!(first, second);
        assert_eq!(first.parent(), target.parent());
        assert_eq!(second.parent(), target.parent());
        assert!(first
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("session.ams.json.")));
    }

    #[test]
    fn concurrent_distinct_project_saves_do_not_collide_on_tmp_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let first_path = tmp.path().join("first.ams.json");
        let second_path = tmp.path().join("second.ams.json");
        let barrier = Arc::new(Barrier::new(2));

        let first_state = project_state("first");
        let second_state = project_state("second");

        let first_barrier = barrier.clone();
        let first = std::thread::spawn(move || {
            first_barrier.wait();
            for _ in 0..25 {
                write_session_atomic(&first_path, &first_state).expect("write first");
            }
            read_session(&first_path).expect("read first")
        });

        let second_barrier = barrier.clone();
        let second = std::thread::spawn(move || {
            second_barrier.wait();
            for _ in 0..25 {
                write_session_atomic(&second_path, &second_state).expect("write second");
            }
            read_session(&second_path).expect("read second")
        });

        let first = first.join().expect("first thread");
        let second = second.join().expect("second thread");

        assert_eq!(first.tracks[0].id, TrackId("first".to_string()));
        assert_eq!(second.tracks[0].id, TrackId("second".to_string()));
    }
}
