use crate::types::*;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::Manager;

const PRESETS_FILENAME: &str = "user_presets.json";

/// Serializes the read-modify-write in `save_user_preset` / `delete_user_preset`
/// so two concurrent command invocations can't each read the same list, mutate
/// it independently, and have the last writer silently drop the other's change.
static PRESETS_LOCK: Mutex<()> = Mutex::new(());

#[tauri::command]
pub async fn save_user_preset(
    name: String,
    kind: PresetKind,
    settings: MasteringSettings,
    app: tauri::AppHandle,
) -> CommandResult<UserPreset> {
    let _guard = PRESETS_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let path = presets_path(&app)?;
    save_user_preset_at_path(&path, &name, kind, settings)
}

#[tauri::command]
pub async fn list_user_presets(app: tauri::AppHandle) -> CommandResult<Vec<UserPreset>> {
    let path = presets_path(&app)?;
    Ok(read_presets(&path).unwrap_or_default())
}

#[tauri::command]
pub async fn delete_user_preset(id: String, app: tauri::AppHandle) -> CommandResult<()> {
    let _guard = PRESETS_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let path = presets_path(&app)?;
    delete_user_preset_at_path(&path, &id)
}

fn presets_path(app: &tauri::AppHandle) -> CommandResult<PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| CommandError::Other(format!("app_data_dir: {e}")))?;
    std::fs::create_dir_all(&app_data).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(app_data.join(PRESETS_FILENAME))
}

pub fn save_user_preset_at_path(
    path: &Path,
    name: &str,
    kind: PresetKind,
    settings: MasteringSettings,
) -> CommandResult<UserPreset> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(CommandError::Other(
            "preset name cannot be empty".to_string(),
        ));
    }
    let mut presets = read_presets(path)?;
    let preset = UserPreset {
        id: uuid::Uuid::new_v4().to_string(),
        name: trimmed,
        kind,
        settings,
        created_at_iso: now_iso(),
    };
    presets.push(preset.clone());
    write_presets(path, &presets)?;
    Ok(preset)
}

pub fn delete_user_preset_at_path(path: &Path, id: &str) -> CommandResult<()> {
    if id.is_empty() {
        return Err(CommandError::Other("preset id cannot be empty".to_string()));
    }
    let mut presets = read_presets(path)?;
    let before = presets.len();
    presets.retain(|p| p.id != id);
    if presets.len() == before {
        // Idempotent: deleting a missing preset is a no-op success.
        return Ok(());
    }
    write_presets(path, &presets)
}

pub fn read_presets(path: &Path) -> CommandResult<Vec<UserPreset>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = std::fs::read(path).map_err(|e| CommandError::Io(e.to_string()))?;
    serde_json::from_slice(&json).map_err(|e| CommandError::Other(format!("presets parse: {e}")))
}

pub fn write_presets(path: &Path, presets: &[UserPreset]) -> CommandResult<()> {
    let json = serde_json::to_vec_pretty(presets)
        .map_err(|e| CommandError::Other(format!("serialize presets: {e}")))?;
    // Unique per-write tmp name so a concurrent writer can't collide on (and
    // corrupt) a shared scratch file before the atomic rename.
    let tmp_name = format!("{PRESETS_FILENAME}.{}.tmp", uuid::Uuid::new_v4());
    let tmp_path = path
        .parent()
        .map(|p| p.join(&tmp_name))
        .unwrap_or_else(|| PathBuf::from(&tmp_name));
    std::fs::write(&tmp_path, &json).map_err(|e| CommandError::Io(e.to_string()))?;
    std::fs::rename(&tmp_path, path).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(())
}
