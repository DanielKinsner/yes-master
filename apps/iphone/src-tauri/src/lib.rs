use std::{path::Path, sync::Arc, time::Duration};

use tauri::{Emitter, Manager};
use yes_master_lib::{
    audio::AudioPlayer, engine, exports, files, AnalysisResult, CommandResult, ExportReport,
    ImportedTrack, MasteringSettings, PlaybackTick, QualityCheck, RenderJob, RenderKind, TrackId,
    WaveformPeaks,
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

#[tauri::command]
async fn iphone_play_track(
    track_id: String,
    track_path: String,
    start_position_sec: Option<f64>,
    player: tauri::State<'_, Arc<AudioPlayer>>,
) -> CommandResult<()> {
    let track_path = normalize_iphone_file_path(&track_path);
    player.play_track(
        TrackId(track_id),
        Path::new(&track_path),
        start_position_sec.unwrap_or(0.0),
    )
}

#[tauri::command]
async fn iphone_play_master(
    track_id: String,
    track_path: String,
    settings: MasteringSettings,
    start_position_sec: Option<f64>,
    preview_lufs_landing: Option<bool>,
    player: tauri::State<'_, Arc<AudioPlayer>>,
) -> CommandResult<()> {
    let track_path = normalize_iphone_file_path(&track_path);
    player.play_master(
        TrackId(track_id),
        Path::new(&track_path),
        settings,
        start_position_sec.unwrap_or(0.0),
        preview_lufs_landing.unwrap_or(true),
    )
}

#[tauri::command]
async fn iphone_update_chain(
    settings: MasteringSettings,
    preview_lufs_landing: Option<bool>,
    player: tauri::State<'_, Arc<AudioPlayer>>,
) -> CommandResult<()> {
    player.update_chain(settings, preview_lufs_landing.unwrap_or(true))
}

#[tauri::command]
async fn iphone_pause_playback(player: tauri::State<'_, Arc<AudioPlayer>>) -> CommandResult<()> {
    player.pause();
    Ok(())
}

#[tauri::command]
async fn iphone_resume_playback(player: tauri::State<'_, Arc<AudioPlayer>>) -> CommandResult<()> {
    player.resume();
    Ok(())
}

#[tauri::command]
async fn iphone_stop_playback(player: tauri::State<'_, Arc<AudioPlayer>>) -> CommandResult<()> {
    player.stop();
    Ok(())
}

#[tauri::command]
async fn iphone_seek_playback(
    position_sec: f64,
    player: tauri::State<'_, Arc<AudioPlayer>>,
) -> CommandResult<()> {
    if !position_sec.is_finite() || position_sec < 0.0 {
        return Err(yes_master_lib::CommandError::Other(format!(
            "invalid seek position: {position_sec}"
        )));
    }
    player.seek(position_sec)
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
    let player = Arc::new(AudioPlayer::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(player)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let app_handle = app.handle().clone();
            let player_state = app.state::<Arc<AudioPlayer>>().inner().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(50));
                let snap = player_state.snapshot();
                if !snap.is_loaded {
                    continue;
                }
                let tick = PlaybackTick {
                    track_id: snap.track_id,
                    position_sec: snap.position_sec,
                    is_playing: snap.is_playing,
                    is_loaded: snap.is_loaded,
                    peak_dbfs: snap.peak_dbfs,
                    peak_left_dbfs: snap.peak_left_dbfs,
                    peak_right_dbfs: snap.peak_right_dbfs,
                    gr_low_db: snap.gr_low_db,
                    gr_mid_db: snap.gr_mid_db,
                    gr_high_db: snap.gr_high_db,
                    lufs_momentary: snap.lufs_momentary,
                    lufs_integrated: snap.lufs_integrated,
                    spectrum_db: snap.spectrum_db,
                };
                let _ = app_handle.emit("playback:tick", tick);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            iphone_import_track,
            iphone_analyze_track,
            iphone_prepare_waveform,
            iphone_render_master,
            iphone_prepare_master_preview,
            iphone_play_track,
            iphone_play_master,
            iphone_update_chain,
            iphone_pause_playback,
            iphone_resume_playback,
            iphone_stop_playback,
            iphone_seek_playback,
            iphone_run_export_checks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
