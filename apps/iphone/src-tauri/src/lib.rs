use std::path::Path;

use yes_master_lib::{
  engine, exports, files, AnalysisResult, CommandResult, ExportReport, ImportedTrack,
  MasteringSettings, QualityCheck, RenderJob, RenderKind, TrackId,
};

#[tauri::command]
async fn iphone_import_track(path: String) -> CommandResult<ImportedTrack> {
  let mut tracks = files::import_tracks(vec![path]).await?;
  tracks
    .pop()
    .ok_or_else(|| yes_master_lib::CommandError::Other("no track imported".to_string()))
}

#[tauri::command]
async fn iphone_analyze_track(
  track_id: String,
  path: String,
) -> CommandResult<AnalysisResult> {
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
async fn iphone_render_master(
  track_id: String,
  track_path: String,
  settings: MasteringSettings,
  output_path: String,
) -> CommandResult<RenderJob> {
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
  iphone_prepare_master_preview_in_dir(
    track_id,
    Path::new(&track_path),
    &settings,
    &preview_dir,
  )
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
      iphone_render_master,
      iphone_prepare_master_preview,
      iphone_run_export_checks,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
