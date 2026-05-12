use crate::types::*;

#[tauri::command]
pub async fn save_user_preset(name: String, kind: PresetKind, settings: MasteringSettings) -> CommandResult<UserPreset> {
    if name.trim().is_empty() {
        return Err(CommandError::Other("preset name cannot be empty".to_string()));
    }
    Ok(UserPreset {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        kind,
        settings,
        created_at_iso: ISO_PLACEHOLDER.to_string(),
    })
}

#[tauri::command]
pub async fn list_user_presets() -> CommandResult<Vec<UserPreset>> {
    Ok(Vec::new())
}
