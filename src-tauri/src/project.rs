use crate::types::*;

#[tauri::command]
pub async fn save_project(path: String, state: ProjectState) -> CommandResult<()> {
    if path.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    let _ = state;
    Ok(())
}

#[tauri::command]
pub async fn autosave_session(state: ProjectState) -> CommandResult<()> {
    let _ = state;
    Ok(())
}

#[tauri::command]
pub async fn load_recent_session() -> CommandResult<Option<ProjectState>> {
    Ok(Some(ProjectState {
        schema_version: 1,
        mode: ProjectMode::Track,
        tracks: Vec::new(),
        track_order: Vec::new(),
        track_settings: std::collections::HashMap::new(),
        album_intent: None,
        last_saved_iso: Some(ISO_PLACEHOLDER.to_string()),
    }))
}
