pub mod album;
pub mod album_render;
pub mod analysis;
pub mod audio;
pub mod confidence;
pub mod decode;
pub mod deep_analysis;
pub mod diagnostics;
pub mod dsp;
pub mod engine;
pub(crate) mod evidence_lanes;
pub mod exports;
pub mod files;
pub mod fixture_matrix;
pub mod guardrails;
pub mod profile_store;
pub mod project;
pub mod reference_tuning;
pub mod sample_rate;
pub mod settings;
pub mod sources;
pub mod spectrum;
pub mod types;
pub mod wav_writer;

pub use types::*;

#[cfg(feature = "app-runner")]
use std::sync::Arc;
#[cfg(feature = "app-runner")]
use std::time::Duration;

#[cfg(feature = "app-runner")]
use tauri::{Emitter, Manager};

#[cfg(feature = "app-runner")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Seed the Phase B confidence gate from the environment (stays off unless
    // YES_MASTER_CONFIDENCE_GATING is set) before any chain resolves confidence.
    crate::confidence::init_confidence_gating_from_env();
    crate::guardrails::init_adaptive_compression_from_env();
    // Album character system (owner decision 2026-07-03): off unless
    // YES_MASTER_ALBUM_CHARACTER is set — pending owner listening signoff.
    crate::album::init_album_character_from_env();
    let player = Arc::new(audio::AudioPlayer::new());
    // B2: the backend owns deriving the adaptive source profile. The same store
    // instance is shared with the audio thread (owned by `player`) and managed
    // here so the analysis / render / readout commands resolve from it too.
    let profile_store = player.profile_store();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(profile_store)
        .manage(player)
        .manage(engine::RenderJobRegistry::default())
        .setup(|app| {
            // Local-first diagnostics: rotating log + panic capture. Init
            // first so everything below (sweep included) can log.
            if let Ok(app_data) = app.path().app_data_dir() {
                crate::diagnostics::init(app_data.join("logs"));
            }
            crate::diagnostics::install_panic_hook();
            crate::diagnostics::info(format!(
                "YES Master {} starting ({} {})",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH
            ));
            // D5: reclaim render tmp files stranded by a process kill /
            // OS shutdown mid-render (no render can be running yet).
            if let Ok(app_data) = app.path().app_data_dir() {
                let removed = crate::engine::sweep_orphaned_render_tmp(&app_data);
                if removed > 0 {
                    crate::diagnostics::info(format!(
                        "swept {removed} orphaned render tmp file(s)"
                    ));
                }
            }
            let app_handle = app.handle().clone();
            let player_state = app.state::<Arc<audio::AudioPlayer>>().inner().clone();
            std::thread::spawn(move || {
                // Edge-triggered "landing loudness…" signal: the boolean
                // payload flips true while the live chain plays with a
                // still-measuring landing gain, false when the corrective
                // gain crossfades in. Emitted only on change.
                let mut last_landing_status: Option<LandingStatus> = None;
                let mut device_loss_detector = audio::PlaybackDeviceLossDetector::new();
                // Mirrors AudioThreadState::device_loss_skips: the audio
                // thread only bumps the counter; THIS thread does the log
                // file I/O (diagnostics.rs discipline — the audio thread
                // stays untouched).
                let mut last_device_loss_skips: u64 = 0;
                let mut last_device_lost_latched = false;
                loop {
                    std::thread::sleep(Duration::from_millis(50));
                    let Ok(snap) = player_state.snapshot() else {
                        continue;
                    };
                    if snap.device_loss_skips != last_device_loss_skips {
                        last_device_loss_skips = snap.device_loss_skips;
                        crate::diagnostics::info(
                            "skipped stale device-loss mark (play completed within the stall window)",
                        );
                    }
                    // The FE banner is driven by the LATCH edge, not the
                    // detector's raw decision: the audio thread can skip a
                    // stale mark (a play completed within the stall window),
                    // and emitting at detection time popped a transient
                    // banner for exactly that case (2026-07-06 audit). A
                    // real loss latches within a tick or two of the mark, so
                    // the banner still arrives promptly; a dismissed latch
                    // re-arms the edge for the next genuine loss.
                    if snap.device_lost && !last_device_lost_latched {
                        let _ = app_handle.emit(
                            audio::PLAYBACK_DEVICE_LOST_EVENT,
                            crate::types::PlaybackDeviceLost {
                                track_id: snap.track_id.clone(),
                                position_sec: snap.position_sec,
                            },
                        );
                    }
                    last_device_lost_latched = snap.device_lost;
                    // Landing status is evaluated BEFORE the is_loaded gate:
                    // an unload (stop / track removal) must still flip the
                    // frontend back to false or the note leaks onto idle.
                    let landing_status = LandingStatus {
                        track_id: snap.track_id.clone(),
                        pending: snap.is_loaded && snap.landing_pending,
                    };
                    if last_landing_status.as_ref() != Some(&landing_status) {
                        last_landing_status = Some(landing_status.clone());
                        let _ = app_handle.emit("landing:status", landing_status);
                    }
                    match device_loss_detector.observe(&snap) {
                        audio::PlaybackDeviceLossDecision::Healthy => {}
                        audio::PlaybackDeviceLossDecision::DeviceLost(event) => {
                            crate::diagnostics::warn(format!(
                                "playback device lost (track {:?} at {:.2}s)",
                                event.track_id, event.position_sec
                            ));
                            // No FE emit here — the audio thread may still
                            // SKIP this mark as stale. The latch-edge check
                            // above emits once the mark actually applies.
                            player_state.mark_device_lost();
                            continue;
                        }
                        audio::PlaybackDeviceLossDecision::SuppressStalledTick => continue,
                    }
                    if !snap.is_loaded {
                        continue;
                    }
                    let tick = PlaybackTick {
                        track_id: snap.track_id,
                        position_sec: snap.position_sec,
                        is_playing: snap.is_playing,
                        is_loaded: snap.is_loaded,
                        peak_dbfs: snap.peak_dbfs,
                        device_lost: snap.device_lost,
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
                }
            });

            // Slice 7: check GitHub Releases for a newer version in the
            // background. Any failure (offline, GitHub unreachable, no release
            // published yet) is logged and swallowed — the updater never blocks
            // or interrupts launch, matching the local-first promise.
            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_update_check(update_handle).await {
                    crate::diagnostics::info(format!("update check skipped: {e}"));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            files::import_tracks,
            engine::analyze_tracks,
            engine::render_track_preview,
            engine::render_track_master,
            engine::cancel_render,
            engine::plan_album,
            engine::render_album_plan,
            audio::prepare_waveform,
            audio::list_audio_output_devices,
            audio::set_audio_output_device,
            audio::play_track,
            audio::play_master,
            audio::update_chain,
            audio::prewarm_decode,
            audio::pause_playback,
            audio::resume_playback,
            audio::clear_device_lost,
            audio::stop_playback,
            audio::seek_playback,
            audio::set_loop_region,
            exports::run_export_checks,
            guardrails::guardrail_readout,
            guardrails::resolve_compression_plan,
            exports::open_output,
            profile_store::evict_source_profile,
            confidence::set_confidence_gating,
            confidence::confidence_gating_enabled,
            guardrails::set_adaptive_compression,
            guardrails::adaptive_compression_enabled,
            project::save_project,
            project::autosave_session,
            project::load_recent_session,
            project::load_project,
            settings::save_user_preset,
            settings::list_user_presets,
            settings::delete_user_preset,
            diagnostics::save_diagnostics_report,
            install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Background update check (Slice 7). Pulls the updater manifest from GitHub
/// Releases (the endpoint + public key live in `tauri.conf.json`
/// `plugins.updater`). If a newer signed release exists it emits
/// `updater:available` with the version so the UI can surface it; the update is
/// NOT auto-downloaded or applied (that stays a user choice). Any network or
/// plugin error bubbles to the caller, which logs it silently — so the app is
/// fully usable with no network.
#[cfg(feature = "app-runner")]
async fn run_update_check(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    use tauri_plugin_updater::UpdaterExt;
    match app.updater()?.check().await? {
        Some(update) => {
            crate::diagnostics::info(format!("update available: {}", update.version));
            let _ = app.emit("updater:available", update.version);
        }
        None => crate::diagnostics::info("update check: up to date"),
    }
    Ok(())
}

/// Slice 7b: download + install the available update, then relaunch. Fired only
/// from the user clicking the update toast's action — never automatically. The
/// caller (frontend) keeps this disabled while an export/render runs, so an
/// in-progress job is never interrupted. Re-checks so it has the concrete
/// update to install; if nothing is available (already current) it is a no-op.
/// On success `restart()` never returns; any failure is returned as a string so
/// the toast can dismiss gracefully.
#[cfg(feature = "app-runner")]
#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(update) = update {
        crate::diagnostics::info(format!("installing update {}", update.version));
        update
            .download_and_install(|_chunk, _total| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}
