use crate::types::*;
use std::path::{Path, PathBuf};

use tauri::Manager;

const PRESETS_FILENAME: &str = "user_presets.json";
const PRESETS_TMP_FILENAME: &str = "user_presets.json.tmp";

#[tauri::command]
pub async fn save_user_preset(
    name: String,
    kind: PresetKind,
    settings: MasteringSettings,
    app: tauri::AppHandle,
) -> CommandResult<UserPreset> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(CommandError::Other(
            "preset name cannot be empty".to_string(),
        ));
    }
    let path = presets_path(&app)?;
    let mut presets = read_presets(&path).unwrap_or_default();
    let preset = UserPreset {
        id: uuid::Uuid::new_v4().to_string(),
        name: trimmed,
        kind,
        settings,
        created_at_iso: ISO_PLACEHOLDER.to_string(),
    };
    presets.push(preset.clone());
    write_presets(&path, &presets)?;
    Ok(preset)
}

#[tauri::command]
pub async fn list_user_presets(app: tauri::AppHandle) -> CommandResult<Vec<UserPreset>> {
    let path = presets_path(&app)?;
    Ok(read_presets(&path).unwrap_or_default())
}

#[tauri::command]
pub async fn delete_user_preset(id: String, app: tauri::AppHandle) -> CommandResult<()> {
    if id.is_empty() {
        return Err(CommandError::Other("preset id cannot be empty".to_string()));
    }
    let path = presets_path(&app)?;
    let mut presets = read_presets(&path).unwrap_or_default();
    let before = presets.len();
    presets.retain(|p| p.id != id);
    if presets.len() == before {
        // Idempotent: deleting a missing preset is a no-op success.
        return Ok(());
    }
    write_presets(&path, &presets)
}

fn presets_path(app: &tauri::AppHandle) -> CommandResult<PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| CommandError::Other(format!("app_data_dir: {e}")))?;
    std::fs::create_dir_all(&app_data).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(app_data.join(PRESETS_FILENAME))
}

pub fn read_presets(path: &Path) -> CommandResult<Vec<UserPreset>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = std::fs::read(path).map_err(|e| CommandError::Io(e.to_string()))?;
    serde_json::from_slice(&json)
        .map_err(|e| CommandError::Other(format!("presets parse: {e}")))
}

pub fn write_presets(path: &Path, presets: &[UserPreset]) -> CommandResult<()> {
    let json = serde_json::to_vec_pretty(presets)
        .map_err(|e| CommandError::Other(format!("serialize presets: {e}")))?;
    let tmp_path = path
        .parent()
        .map(|p| p.join(PRESETS_TMP_FILENAME))
        .unwrap_or_else(|| PathBuf::from(PRESETS_TMP_FILENAME));
    std::fs::write(&tmp_path, &json).map_err(|e| CommandError::Io(e.to_string()))?;
    std::fs::rename(&tmp_path, path).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(())
}
