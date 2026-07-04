pub mod album;
pub mod album_render;
pub mod analysis;
pub mod audio;
pub mod confidence;
pub mod decode;
pub mod deep_analysis;
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
        .manage(profile_store)
        .manage(player)
        .setup(|app| {
            // D5: reclaim render tmp files stranded by a process kill /
            // OS shutdown mid-render (no render can be running yet).
            if let Ok(app_data) = app.path().app_data_dir() {
                let removed = crate::engine::sweep_orphaned_render_tmp(&app_data);
                if removed > 0 {
                    eprintln!("swept {removed} orphaned render tmp file(s)");
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
                loop {
                    std::thread::sleep(Duration::from_millis(50));
                    let Ok(snap) = player_state.snapshot() else {
                        continue;
                    };
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
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            files::import_tracks,
            engine::analyze_tracks,
            engine::render_track_preview,
            engine::render_track_master,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
