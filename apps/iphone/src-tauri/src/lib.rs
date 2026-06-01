use std::path::Path;

use yes_master_lib::{
    engine, exports, files, AnalysisResult, CommandResult, ExportReport, ImportedTrack,
    MasteringSettings, QualityCheck, RenderJob, RenderKind, TrackId, WaveformPeaks,
};

#[tauri::command]
async fn iphone_import_track(path: String) -> CommandResult<ImportedTrack> {
    let mut tracks = files::import_tracks(vec![normalize_iphone_file_path(&path)]).await?;
    tracks
        .pop()
        .ok_or_else(|| yes_master_lib::CommandError::Other("no track imported".to_string()))
}

#[tauri::command]
async fn iphone_analyze_track(track_id: String, path: String) -> CommandResult<AnalysisResult> {
    let path = normalize_iphone_file_path(&path);
    let mut results = engine::analyze_tracks(vec![engine::AnalyzeRequest {
        id: TrackId(track_id),
        path,
    }])
    .await?;
    results
        .pop()
        .ok_or_else(|| yes_master_lib::CommandError::Other("no analysis produced".to_string()))
}

#[tauri::command]
async fn iphone_prepare_waveform(
    track_id: String,
    track_path: String,
    target_pixels: Option<u32>,
) -> CommandResult<WaveformPeaks> {
    yes_master_lib::audio::prepare_waveform(
        TrackId(track_id),
        normalize_iphone_file_path(&track_path),
        target_pixels,
    )
    .await
}

#[tauri::command]
async fn iphone_render_master(
    track_id: String,
    track_path: String,
    settings: MasteringSettings,
    output_path: String,
) -> CommandResult<RenderJob> {
    let track_path = normalize_iphone_file_path(&track_path);
    let output_path = normalize_iphone_file_path(&output_path);
    iphone_render_master_to_path(
        track_id,
        Path::new(&track_path),
        &settings,
        Path::new(&output_path),
    )
}

#[tauri::command]
async fn iphone_prepare_master_preview(
    track_id: String,
    track_path: String,
    settings: MasteringSettings,
) -> CommandResult<RenderJob> {
    let preview_dir = std::env::temp_dir().join("yes-master-iphone-previews");
    let track_path = normalize_iphone_file_path(&track_path);
    iphone_prepare_master_preview_in_dir(track_id, Path::new(&track_path), &settings, &preview_dir)
}

pub fn iphone_render_master_to_path(
    track_id: String,
    source_path: &Path,
    settings: &MasteringSettings,
    output_path: &Path,
) -> CommandResult<RenderJob> {
    let output_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(output_dir)
        .map_err(|e| yes_master_lib::CommandError::Io(e.to_string()))?;
    engine::mastering_render_to_path(
        TrackId(track_id),
        source_path,
        settings,
        output_dir,
        RenderKind::Master,
        output_path,
    )
}

pub fn iphone_prepare_master_preview_in_dir(
    track_id: String,
    source_path: &Path,
    settings: &MasteringSettings,
    preview_dir: &Path,
) -> CommandResult<RenderJob> {
    let output_path = preview_dir.join(format!(
        "{}-mastered-preview.wav",
        sanitize_preview_name(&track_id)
    ));
    iphone_render_master_to_path(track_id, source_path, settings, &output_path)
}

fn sanitize_preview_name(track_id: &str) -> String {
    let sanitized = track_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "track".to_string()
    } else {
        sanitized
    }
}

pub fn normalize_iphone_file_path(path: &str) -> String {
    percent_decode_path(&strip_file_url_prefix(path.trim()))
}

fn strip_file_url_prefix(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("file://localhost") {
        if rest.starts_with('/') {
            return rest.to_string();
        }
        return format!("/{rest}");
    }

    if let Some(rest) = path.strip_prefix("file://") {
        return rest.to_string();
    }

    path.to_string()
}

fn percent_decode_path(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut changed = false;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                decoded.push((high << 4) | low);
                index += 3;
                changed = true;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    if !changed {
        return path.to_string();
    }

    String::from_utf8(decoded).unwrap_or_else(|_| path.to_string())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[tauri::command]
async fn iphone_run_export_checks(
    report: ExportReport,
    source_analysis: Option<AnalysisResult>,
    settings: Option<MasteringSettings>,
) -> CommandResult<Vec<QualityCheck>> {
    Ok(exports::export_checks_for_report(
        &report,
        source_analysis.as_ref(),
        settings.as_ref(),
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            iphone_import_track,
            iphone_analyze_track,
            iphone_prepare_waveform,
            iphone_render_master,
            iphone_prepare_master_preview,
            iphone_run_export_checks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
