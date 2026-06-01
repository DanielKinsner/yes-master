use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use tauri::{Emitter, Manager};
use yes_master_lib::{
    audio::AudioPlayer, engine, exports, files, AnalysisResult, CommandResult, ExportReport,
    ImportedTrack, MasteringSettings, PlaybackTick, QualityCheck, RenderJob, RenderKind, TrackId,
    WaveformPeaks,
};

/// iOS audio-session setup. cpal/rodio open a RemoteIO output unit, but iOS
/// produces no audible output until the app sets the shared `AVAudioSession`
/// category to `.playback` and activates it. Without this, playback is silent
/// (and, on some devices/iOS versions, starting the unit can fail outright).
///
/// We avoid a link-time AVFoundation dependency — Xcode links the Rust
/// staticlib and would otherwise need the framework added to its build
/// settings, which `tauri ios` regeneration would drop — by `dlopen`-ing
/// AVFoundation at runtime (registering the `AVAudioSession` class) and
/// `dlsym`-ing the category constant rather than hard-coding its raw value.
#[cfg(target_os = "ios")]
mod ios_audio {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::Emitter;

    static AUDIO_SESSION_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);

    pub fn configure(app: &tauri::AppHandle) {
        activate_playback_session(Some(app));
    }

    /// Set the shared AVAudioSession to `.playback` and activate it. Idempotent
    /// — safe to re-call to recover after an interruption, a route change, or
    /// returning from the background, where iOS deactivates the session and the
    /// app would otherwise be silent for the rest of its lifetime.
    pub fn activate_playback_session(app: Option<&tauri::AppHandle>) {
        unsafe {
            let handle = libc::dlopen(
                c"/System/Library/Frameworks/AVFoundation.framework/AVFoundation".as_ptr(),
                libc::RTLD_NOW,
            );
            if handle.is_null() {
                log::warn!("AVFoundation dlopen failed; iPhone playback will be silent");
                return;
            }

            // `AVAudioSessionCategoryPlayback` is exported as `NSString * const`,
            // i.e. dlsym hands back the address of the pointer — deref once.
            let category_sym = libc::dlsym(handle, c"AVAudioSessionCategoryPlayback".as_ptr());
            if category_sym.is_null() {
                log::warn!("AVAudioSessionCategoryPlayback symbol missing; skipping session setup");
                return;
            }
            let category: *const AnyObject = *(category_sym as *const *const AnyObject);

            let session: *mut AnyObject = msg_send![class!(AVAudioSession), sharedInstance];
            if session.is_null() {
                log::warn!("AVAudioSession sharedInstance was nil");
                return;
            }

            let mut category_error: *mut AnyObject = std::ptr::null_mut();
            let mut active_error: *mut AnyObject = std::ptr::null_mut();
            let category_ok: bool =
                msg_send![session, setCategory: category, error: &mut category_error];
            let active_ok: bool = msg_send![session, setActive: true, error: &mut active_error];
            log::info!(
                "AVAudioSession activated (category_ok={category_ok}, active_ok={active_ok})"
            );
            if !category_ok || !active_ok {
                let detail = error_description(if !active_ok {
                    active_error
                } else {
                    category_error
                })
                .unwrap_or_else(|| "iOS did not return an audio-session error detail".to_string());
                let warning = format!(
                    "iPhone audio could not be fully activated. Playback may be silent until the app is restarted. {detail}"
                );
                log::warn!("{warning}");
                emit_warning_once(app, warning);
            }
        }
    }

    unsafe fn error_description(error: *mut AnyObject) -> Option<String> {
        if error.is_null() {
            return None;
        }
        let description: *mut AnyObject = msg_send![error, localizedDescription];
        nsstring_to_string(description)
    }

    unsafe fn nsstring_to_string(value: *mut AnyObject) -> Option<String> {
        if value.is_null() {
            return None;
        }
        let utf8: *const libc::c_char = msg_send![value, UTF8String];
        if utf8.is_null() {
            return None;
        }
        Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
    }

    fn emit_warning_once(app: Option<&tauri::AppHandle>, warning: String) {
        let Some(app) = app else {
            return;
        };
        if AUDIO_SESSION_WARNING_EMITTED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let _ = app.emit("iphone:audio-session-warning", warning);
        }
    }
}

#[tauri::command]
async fn iphone_import_track(path: String, app: tauri::AppHandle) -> CommandResult<ImportedTrack> {
    let normalized = normalize_iphone_file_path(&path);
    log::info!(
        "iphone_import_track: raw={path:?} normalized={normalized:?} exists={}",
        Path::new(&normalized).exists()
    );
    // Copy the picked file into an app-owned dir so analyze/waveform/play/render
    // all read a path the app controls. The document picker delivers files into
    // tmp/<bundle>-Inbox/, which iOS can purge under storage pressure or between
    // launches (every later File::open then fails), and a same-basename
    // re-import overwrites the prior Inbox copy. A durable copy removes both.
    let durable = copy_into_app_imports(&app, &normalized);
    let mut tracks = files::import_tracks(vec![durable]).await?;
    tracks
        .pop()
        .ok_or_else(|| yes_master_lib::CommandError::Other("no track imported".to_string()))
}

/// Copy `source` into `app_data_dir()/imports/<hash>/<original-name>` and return
/// that path. The per-source-hash subdir keeps the original filename (so the
/// display name and extension survive) while avoiding same-basename collisions.
/// On any failure (including an already-missing source) it returns the original
/// path unchanged so the real error surfaces downstream rather than being masked
/// here.
fn copy_into_app_imports(app: &tauri::AppHandle, source: &str) -> String {
    use std::hash::{Hash, Hasher};

    let source_path = Path::new(source);
    if !source_path.exists() {
        return source.to_string();
    }
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("import.audio");

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    let bucket = format!("{:016x}", hasher.finish());

    let dest_dir = match app.path().app_data_dir() {
        Ok(dir) => dir.join("imports").join(bucket),
        Err(error) => {
            log::warn!("app_data_dir unavailable, using picked path: {error}");
            return source.to_string();
        }
    };
    if let Err(error) = std::fs::create_dir_all(&dest_dir) {
        log::warn!("could not create imports dir, using picked path: {error}");
        return source.to_string();
    }
    let dest = dest_dir.join(file_name);
    match std::fs::copy(source_path, &dest) {
        Ok(_) => dest.to_string_lossy().to_string(),
        Err(error) => {
            log::warn!("could not copy import to app dir, using picked path: {error}");
            source.to_string()
        }
    }
}

#[tauri::command]
async fn iphone_analyze_track(track_id: String, path: String) -> CommandResult<AnalysisResult> {
    let path = normalize_iphone_file_path(&path);
    let mut results = tauri::async_runtime::spawn_blocking(move || {
        tauri::async_runtime::block_on(engine::analyze_tracks(vec![engine::AnalyzeRequest {
            id: TrackId(track_id),
            path,
        }]))
    })
    .await
    .map_err(|error| {
        yes_master_lib::CommandError::Other(format!("analyze task failed: {error}"))
    })??;
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
    let normalized = normalize_iphone_file_path(&track_path);
    log::info!(
        "iphone_prepare_waveform: raw={track_path:?} normalized={normalized:?} exists={} pixels={target_pixels:?}",
        Path::new(&normalized).exists()
    );
    let decode_track_id = TrackId(track_id);
    let decode_path = normalized.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        tauri::async_runtime::block_on(yes_master_lib::audio::prepare_waveform(
            decode_track_id,
            decode_path,
            target_pixels,
        ))
    })
    .await
    .map_err(|error| {
        yes_master_lib::CommandError::Other(format!("waveform task failed: {error}"))
    })?;
    match &result {
        Ok(peaks) => log::info!(
            "iphone_prepare_waveform ok: channels={} first_len={}",
            peaks.channels.len(),
            peaks.channels.first().map(Vec::len).unwrap_or(0)
        ),
        Err(error) => log::error!("iphone_prepare_waveform FAILED for {normalized:?}: {error:?}"),
    }
    result
}

#[tauri::command]
async fn iphone_render_master(
    track_id: String,
    track_path: String,
    settings: MasteringSettings,
    output_path: String,
    app: tauri::AppHandle,
) -> CommandResult<RenderJob> {
    let track_path = normalize_iphone_file_path(&track_path);
    // On iOS the frontend hands us a bare filename; land the master in the app's
    // Files-visible Documents/YES Master folder rather than a save()-dialog
    // scoped URL (which exports a 0-byte placeholder the real bytes never reach).
    let output_path = resolve_export_path(&app, &normalize_iphone_file_path(&output_path))?;
    iphone_render_master_to_path(track_id, Path::new(&track_path), &settings, &output_path)
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
        preview_lufs_landing.unwrap_or(false),
    )
}

#[tauri::command]
async fn iphone_update_chain(
    settings: MasteringSettings,
    preview_lufs_landing: Option<bool>,
    player: tauri::State<'_, Arc<AudioPlayer>>,
) -> CommandResult<()> {
    player.update_chain(settings, preview_lufs_landing.unwrap_or(false))
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
    let job = engine::mastering_render_to_path(
        TrackId(track_id),
        source_path,
        settings,
        output_dir,
        RenderKind::Master,
        output_path,
    )?;
    // Don't let the UI claim "Master ready" unless real bytes landed — the iOS
    // export path historically produced empty/placeholder files.
    let bytes = std::fs::metadata(output_path)
        .map(|meta| meta.len())
        .unwrap_or(0);
    if bytes == 0 {
        return Err(yes_master_lib::CommandError::Other(format!(
            "export produced no audio at {}",
            output_path.display()
        )));
    }
    Ok(job)
}

/// Resolve the export destination. An absolute path (desktop / tests) is used
/// as-is; a bare filename (iOS simple-mode) lands in the app's Files-visible
/// `Documents/YES Master` folder so the user can actually retrieve the master.
fn resolve_export_path(app: &tauri::AppHandle, output_path: &str) -> CommandResult<PathBuf> {
    let candidate = Path::new(output_path);
    if candidate.is_absolute() {
        return Ok(candidate.to_path_buf());
    }
    let file_name = candidate
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("YES-Master.wav");
    let dir = app
        .path()
        .document_dir()
        .map_err(|error| {
            yes_master_lib::CommandError::Other(format!("export document_dir: {error}"))
        })?
        .join("YES Master");
    Ok(dir.join(sanitize_export_file_name(file_name)))
}

/// Strip path separators and force a `.wav` extension so a user-supplied name
/// can't escape the export folder or land without a usable extension.
fn sanitize_export_file_name(file_name: &str) -> String {
    let base = file_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_name)
        .trim();
    let cleaned = if base.is_empty() {
        "YES-Master.wav"
    } else {
        base
    };
    if cleaned.to_ascii_lowercase().ends_with(".wav") {
        cleaned.to_string()
    } else {
        format!("{cleaned}.wav")
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

#[tauri::command]
async fn iphone_reactivate_audio_session(app: tauri::AppHandle) -> CommandResult<()> {
    // Re-activate the AVAudioSession after the app returns to the foreground —
    // a call / Siri / route change deactivates it, otherwise leaving playback
    // silent for the rest of the app's lifetime. No-op off iOS.
    #[cfg(target_os = "ios")]
    ios_audio::activate_playback_session(Some(&app));
    #[cfg(not(target_os = "ios"))]
    let _ = app;
    Ok(())
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
            // Configure the audio session AFTER the log plugin so its result is
            // visible in the device console, and before the output stream opens
            // on first play.
            #[cfg(target_os = "ios")]
            ios_audio::configure(app.handle());
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
            iphone_play_track,
            iphone_play_master,
            iphone_update_chain,
            iphone_pause_playback,
            iphone_resume_playback,
            iphone_stop_playback,
            iphone_seek_playback,
            iphone_run_export_checks,
            iphone_reactivate_audio_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
