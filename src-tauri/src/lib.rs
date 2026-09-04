pub mod album;
pub mod album_render;
pub mod analysis;
pub mod audio;
pub mod confidence;
pub mod decode;
pub mod deep_analysis;
pub mod demo;
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

/// Default main-window size from `tauri.conf.json` (`app.windows[0]`), in
/// logical pixels. Named consts rather than a runtime config read: the
/// maximize decision is a pure function that unit tests drive directly, and
/// nothing on the launch path should wait on a config lookup.
#[cfg(feature = "app-runner")]
const DEFAULT_WINDOW_WIDTH: f64 = 1920.0;
#[cfg(feature = "app-runner")]
const DEFAULT_WINDOW_HEIGHT: f64 = 1080.0;

/// True when the monitor's logical work area cannot hold the configured
/// default window. Logical, not physical: Windows display scaling shrinks
/// the usable canvas (1080p at 150 % is 1280×720 logical) and the app is
/// composed for 1920×1080 logical (tauri.conf.json).
#[cfg(feature = "app-runner")]
pub(crate) fn should_maximize_for_work_area(
    work_w: f64,
    work_h: f64,
    default_w: f64,
    default_h: f64,
) -> bool {
    work_w < default_w || work_h < default_h
}

/// Layout floor from `tauri.conf.json` (`minWidth` / `minHeight`), logical
/// pixels. The console is composed down to exactly this size.
#[cfg(feature = "app-runner")]
const LAYOUT_FLOOR_WIDTH: f64 = 1360.0;
#[cfg(feature = "app-runner")]
const LAYOUT_FLOOR_HEIGHT: f64 = 740.0;
/// Never zoom the console below this. Past it the type is smaller than the
/// scrollbar it replaces; the owner's call is to revert tier 2 rather than
/// tune it (D4, 2026-09-01).
#[cfg(feature = "app-runner")]
const MIN_FIT_ZOOM: f64 = 0.8;

/// D4 tier 2: the NATIVE webview zoom factor that fits the layout floor into
/// a work area smaller than it. 1.0 whenever the floor already fits — never
/// above 1.0 — and clamped at `MIN_FIT_ZOOM` below. Native zoom (what
/// Ctrl+minus does in a browser) stays crisp and keeps CSS breakpoints
/// honest because CSS pixels scale with it; the CSS `zoom` property is the
/// thing `useWebviewZoomShortcuts` deliberately pins to 1, and it is not
/// touched here.
#[cfg(feature = "app-runner")]
pub(crate) fn fit_zoom_for_work_area(work_w: f64, work_h: f64) -> f64 {
    // Garbage in, no zoom: a non-finite or non-positive work area is not a
    // reason to shrink the console.
    if !work_w.is_finite() || !work_h.is_finite() || work_w <= 0.0 || work_h <= 0.0 {
        return 1.0;
    }
    let factor = (work_w / LAYOUT_FLOOR_WIDTH)
        .min(work_h / LAYOUT_FLOOR_HEIGHT)
        .min(1.0);
    factor.max(MIN_FIT_ZOOM)
}

/// Choose the dimensions that actually contain the webview. A maximized
/// decorated window's client area is shorter than the monitor work area on
/// Windows because the native title bar and borders are outside the client.
/// Fall back to the monitor work area when the client measurement is missing
/// or invalid so display probing can never block launch.
#[cfg(feature = "app-runner")]
pub(crate) fn fit_zoom_for_available_area(
    work_w: f64,
    work_h: f64,
    client_area: Option<(f64, f64)>,
) -> f64 {
    let client_area = client_area.filter(|(client_w, client_h)| {
        client_w.is_finite() && client_h.is_finite() && *client_w > 0.0 && *client_h > 0.0
    });
    let (available_w, available_h) = client_area.unwrap_or((work_w, work_h));
    fit_zoom_for_work_area(available_w, available_h)
}

/// D4 (owner decision 2026-09-01), tiers 1 and 2: open at the configured
/// 1920×1080, centred, when the primary monitor's logical work area holds
/// it; maximized when it does not; and, below the 1360×740 layout floor,
/// natively zoomed so the whole console still fits. Every error path is
/// ignored — a missing monitor, an odd scale factor, a failed maximize or
/// zoom must never block launch.
#[cfg(feature = "app-runner")]
fn fit_main_window_to_display(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };
    let scale = monitor.scale_factor();
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }
    let work = monitor.work_area().size;
    let work_w = f64::from(work.width) / scale;
    let work_h = f64::from(work.height) / scale;
    let maximized =
        should_maximize_for_work_area(work_w, work_h, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
            && window.maximize().is_ok();
    if maximized {
        crate::diagnostics::info(format!(
            "window maximized: work area {work_w:.0}x{work_h:.0} logical < {DEFAULT_WINDOW_WIDTH:.0}x{DEFAULT_WINDOW_HEIGHT:.0}"
        ));
    }

    // `work_area` describes the maximized OUTER window. The layout lives in
    // the client area, which excludes the native title bar and borders. At
    // 300% Windows scaling that difference was enough for Standard's bottom
    // intensity controls to remain clipped even though the work-area zoom
    // arithmetic was correct. Tauri's `inner_size` is explicitly the client
    // area; read it only after maximize so it describes the final viewport.
    let client_area = maximized
        .then(|| window.inner_size().ok())
        .flatten()
        .map(|client| {
            (
                f64::from(client.width) / scale,
                f64::from(client.height) / scale,
            )
        });
    let zoom = fit_zoom_for_available_area(work_w, work_h, client_area);
    if zoom < 1.0 && window.set_zoom(zoom).is_ok() {
        if let Some((client_w, client_h)) = client_area {
            crate::diagnostics::info(format!(
                "window zoom {zoom:.3}: client area {client_w:.0}x{client_h:.0} logical within work area {work_w:.0}x{work_h:.0} < floor {LAYOUT_FLOOR_WIDTH:.0}x{LAYOUT_FLOOR_HEIGHT:.0}"
            ));
        } else {
            crate::diagnostics::info(format!(
                "window zoom {zoom:.3}: client area unavailable; work-area fallback {work_w:.0}x{work_h:.0} logical < floor {LAYOUT_FLOOR_WIDTH:.0}x{LAYOUT_FLOOR_HEIGHT:.0}"
            ));
        }
    }
}

#[cfg(all(test, feature = "app-runner"))]
mod window_fit_tests {
    use super::{
        fit_zoom_for_available_area, fit_zoom_for_work_area, should_maximize_for_work_area,
        DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, MIN_FIT_ZOOM,
    };

    fn maximize(work_w: f64, work_h: f64) -> bool {
        should_maximize_for_work_area(work_w, work_h, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }

    #[test]
    fn fit_zoom_for_1080p_at_150_percent_is_width_limited() {
        // 1280/1360 = 0.941… is the tighter ratio (720/740 = 0.973…).
        let zoom = fit_zoom_for_work_area(1280.0, 720.0);
        assert!((zoom - 1280.0 / 1360.0).abs() < 1e-9, "{zoom}");
        assert!(zoom > 0.94 && zoom < 0.942, "{zoom}");
    }

    #[test]
    fn fit_zoom_prefers_the_post_maximize_client_area() {
        // Owner's 4K / 300% repro: Windows reported a 1280x672 work area,
        // but native title-bar chrome leaves a shorter webview client area.
        // Fitting the outer work area produced 0.908 and clipped Standard's
        // intensity quick-set buttons; the client height must drive the fit.
        let zoom = fit_zoom_for_available_area(1280.0, 672.0, Some((1280.0, 640.0)));
        assert!((zoom - 640.0 / 740.0).abs() < 1e-9, "{zoom}");
    }

    #[test]
    fn fit_zoom_is_unity_whenever_the_floor_fits() {
        assert_eq!(fit_zoom_for_work_area(1536.0, 864.0), 1.0);
        assert_eq!(fit_zoom_for_work_area(1920.0, 1080.0), 1.0);
        assert_eq!(fit_zoom_for_work_area(2560.0, 1440.0), 1.0);
    }

    #[test]
    fn fit_zoom_clamps_at_the_minimum() {
        assert_eq!(fit_zoom_for_work_area(1000.0, 600.0), MIN_FIT_ZOOM);
    }

    #[test]
    fn fit_zoom_leaves_degenerate_work_areas_alone() {
        // Garbage in, no zoom — never a zero, NaN, or clamped factor.
        assert_eq!(fit_zoom_for_work_area(0.0, 0.0), 1.0);
        assert_eq!(fit_zoom_for_work_area(-1.0, 720.0), 1.0);
        assert_eq!(fit_zoom_for_work_area(f64::NAN, f64::NAN), 1.0);
        assert_eq!(fit_zoom_for_work_area(f64::INFINITY, 720.0), 1.0);
    }

    #[test]
    fn fit_zoom_still_fits_1080p_at_175_percent() {
        // 1097×617 logical → 0.807, above the clamp: the console fits. At
        // 200 % (960×540) it would clamp, which is the documented caveat.
        let zoom = fit_zoom_for_work_area(1097.0, 617.0);
        assert!(zoom > MIN_FIT_ZOOM && zoom < 0.81, "{zoom}");
        assert_eq!(fit_zoom_for_work_area(960.0, 540.0), MIN_FIT_ZOOM);
    }

    #[test]
    fn laptop_1080p_at_125_percent_maximizes() {
        assert!(maximize(1536.0, 864.0));
    }

    #[test]
    fn laptop_1080p_at_150_percent_maximizes() {
        assert!(maximize(1280.0, 720.0));
    }

    #[test]
    fn exact_default_fits_without_maximizing() {
        assert!(!maximize(1920.0, 1080.0));
    }

    #[test]
    fn qhd_fits_without_maximizing() {
        assert!(!maximize(2560.0, 1440.0));
    }

    #[test]
    fn short_work_area_maximizes_even_when_wide_enough() {
        // Width fits; a taskbar has eaten the bottom rows.
        assert!(maximize(1920.0, 1000.0));
    }
}

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
        .plugin(tauri_plugin_opener::init())
        .manage(profile_store)
        .manage(player)
        .manage(engine::RenderJobRegistry::default())
        .manage(UpdateAvailability::default())
        .setup(|app| {
            // Local-first diagnostics: rotating log + panic capture. Init
            // first so everything below (sweep included) can log.
            if let Ok(app_data) = app.path().app_data_dir() {
                crate::diagnostics::init(app_data.join("logs"));
            }
            crate::diagnostics::install_panic_hook();
            crate::diagnostics::info(format!(
                "YES Master {} (build {}) starting ({} {})",
                env!("CARGO_PKG_VERSION"),
                BUILD_STAMP,
                std::env::consts::OS,
                std::env::consts::ARCH
            ));
            // D4 tier 1: maximize when the display cannot hold 1920×1080
            // logical. Runs after diagnostics init so the decision is logged.
            fit_main_window_to_display(app.handle());
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
            exports::suggest_export_filename,
            build_info,
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
            demo::prepare_demo_track,
            install_update,
            available_update_version,
            open_release_page,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Owner finding 2026-07-08 ("what am I running?"): every dev build says
/// "0.9.0", so hand-testing cannot tell builds apart and stale installs have
/// burned test sessions. `YES_BUILD_STAMP` is set by build.rs to
/// `<git-short-hash>[+] · <build time>` (`+` = dirty tree). Shown in the Help
/// dialog and the startup log line.
#[cfg(feature = "app-runner")]
const BUILD_STAMP: &str = env!("YES_BUILD_STAMP");

/// Version + build stamp for the Help dialog — the user-facing answer to
/// "which build am I actually running?".
#[cfg(feature = "app-runner")]
#[tauri::command]
fn build_info() -> String {
    format!("{} · build {}", env!("CARGO_PKG_VERSION"), BUILD_STAMP)
}

/// The ONLY URL the desktop app will ever hand to the system opener (audit
/// L-03). Private + zero-argument command by design: a frontend cannot pass
/// a URL, so a compromised renderer cannot steer the opener elsewhere. Must
/// stay byte-identical to src/lib/release-links.ts (release-readiness test
/// pins both sources).
#[cfg(feature = "app-runner")]
const RELEASES_INDEX_URL: &str = "https://github.com/DanielKinsner/yes-master/releases";

/// Manual recovery for a failed update install: open the fixed GitHub
/// Releases index in the system browser. See [`RELEASES_INDEX_URL`].
#[cfg(feature = "app-runner")]
#[tauri::command]
fn open_release_page(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(RELEASES_INDEX_URL, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Latched updater availability (audit L-02). `updater:available` is an
/// edge-triggered emit: if the network check wins the race against React's
/// listener registration, Tauri drops the event and the notice is lost until
/// the next app restart. The version is latched here BEFORE the emit so the
/// frontend can query it at any time; reading never consumes the latch.
#[cfg(feature = "app-runner")]
#[derive(Default)]
struct UpdateAvailability(std::sync::Mutex<Option<String>>);

#[cfg(feature = "app-runner")]
impl UpdateAvailability {
    fn record(&self, version: String) -> Result<(), String> {
        let mut slot = self
            .0
            .lock()
            .map_err(|_| "update availability state poisoned".to_string())?;
        *slot = Some(version);
        Ok(())
    }

    fn current(&self) -> Result<Option<String>, String> {
        let slot = self
            .0
            .lock()
            .map_err(|_| "update availability state poisoned".to_string())?;
        Ok(slot.clone())
    }
}

/// Frontend query for the latched availability. Registered alongside the
/// event so detection before, during, or after listener registration all
/// resolve to the same single notice.
#[cfg(feature = "app-runner")]
#[tauri::command]
fn available_update_version(
    state: tauri::State<'_, UpdateAvailability>,
) -> Result<Option<String>, String> {
    state.current()
}

/// Background update check (Slice 7). Pulls the updater manifest from GitHub
/// Releases (the endpoint + public key live in `tauri.conf.json`
/// `plugins.updater`). If a newer signed release exists it latches the version
/// (see [`UpdateAvailability`]) and emits `updater:available` so the UI can
/// surface it; the update is NOT auto-downloaded or applied (that stays a user
/// choice). Any network or plugin error bubbles to the caller, which logs it
/// silently — so the app is fully usable with no network.
#[cfg(feature = "app-runner")]
async fn run_update_check(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    use tauri_plugin_updater::UpdaterExt;
    match app.updater()?.check().await? {
        Some(update) => {
            crate::diagnostics::info(format!("update available: {}", update.version));
            // Latch first, emit second: a frontend that misses the emit can
            // still recover the version by query; one that catches both paths
            // funnels them through the same idempotent setter.
            if let Err(e) = app
                .state::<UpdateAvailability>()
                .record(update.version.clone())
            {
                crate::diagnostics::warn(format!("update availability latch failed: {e}"));
            }
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
#[cfg(all(test, feature = "app-runner"))]
mod update_availability_tests {
    use super::UpdateAvailability;

    #[test]
    fn latched_version_survives_repeated_reads() {
        let store = UpdateAvailability::default();
        assert_eq!(store.current().expect("empty read"), None);

        store.record("0.9.3".to_string()).expect("record");
        assert_eq!(
            store.current().expect("first read"),
            Some("0.9.3".to_string())
        );
        // Reading must never consume the latch: the frontend may query at
        // any time after (or instead of) the startup event, more than once.
        assert_eq!(
            store.current().expect("second read"),
            Some("0.9.3".to_string())
        );
    }
}

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
