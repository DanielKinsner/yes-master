use crate::types::*;
use std::path::Path;

#[tauri::command]
pub async fn import_tracks(paths: Vec<String>) -> CommandResult<Vec<ImportedTrack>> {
    paths
        .into_iter()
        .map(|p| import_one(&p))
        .collect()
}

fn import_one(path_str: &str) -> CommandResult<ImportedTrack> {
    if path_str.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    if path_str.contains("..") {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {path_str}"
        )));
    }
    let path = Path::new(path_str);
    let display_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    let source_format = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    Ok(ImportedTrack {
        id: TrackId::new(),
        path: path_str.to_string(),
        display_name,
        source_format,
        duration_seconds: None,
        sample_rate: None,
        channels: None,
    })
}
