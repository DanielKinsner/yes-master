use crate::analysis::{analyze_one_with_progress, nudge_role_by_position, sanitize_lufs};
use crate::sample_rate::convert_interleaved;
use crate::types::*;
use crate::wav_writer::write_wav;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use ebur128::{EbuR128, Mode};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeRequest {
    pub id: TrackId,
    pub path: String,
}

/// Phase 12.1 export progress — emitted on the "render:progress" Tauri event
/// channel during `render_track_master` / `render_track_preview` so the
/// frontend can render a real progress bar.
#[derive(Debug, Serialize, Clone)]
pub struct RenderProgress {
    pub job_id: String,
    pub track_id: TrackId,
    pub kind: RenderKind,
    pub fraction: f32,
}

#[derive(Default)]
pub struct RenderJobRegistry {
    jobs: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl RenderJobRegistry {
    pub fn register(&self, job_id: String) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        let mut jobs = self
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        jobs.insert(job_id, flag.clone());
        flag
    }

    pub fn cancel(&self, job_id: &str) {
        let jobs = self
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(flag) = jobs.get(job_id) {
            flag.store(true, Ordering::SeqCst);
        }
    }

    pub fn remove(&self, job_id: &str) {
        let mut jobs = self
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        jobs.remove(job_id);
    }
}

#[tauri::command]
pub async fn cancel_render(
    job_id: String,
    render_jobs: tauri::State<'_, RenderJobRegistry>,
) -> CommandResult<()> {
    crate::diagnostics::info(format!("render cancel requested: job {job_id}"));
    render_jobs.cancel(&job_id);
    Ok(())
}

pub fn analysis_progress_event(batch_id: &str, fraction: f32, label: &str) -> AnalysisProgress {
    AnalysisProgress {
        batch_id: batch_id.to_string(),
        fraction,
        label: label.to_string(),
    }
}

#[tauri::command]
pub async fn analyze_tracks(
    app: tauri::AppHandle,
    tracks: Vec<AnalyzeRequest>,
    batch_id: Option<String>,
    profile_store: tauri::State<'_, std::sync::Arc<crate::profile_store::SourceProfileStore>>,
) -> CommandResult<Vec<AnalysisResult>> {
    let requested_ids: Vec<TrackId> = tracks.iter().map(|r| r.id.clone()).collect();
    let batch_id = batch_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let profile_store = profile_store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = analyze_tracks_core_with_progress_sync(tracks, |fraction, label| {
            let _ = app.emit(
                "analysis:progress",
                analysis_progress_event(&batch_id, fraction, label),
            );
        });
        match &result {
            Ok(results) => {
                // B2: the backend is the SINGLE point that derives the adaptive source
                // profile (kills the dual TS/Rust mapper). The render / readout commands
                // and the live chain resolve from this store instead of an FE-injected
                // profile.
                populate_profile_store(&profile_store, results);
                // Evict stale profiles for any requested track that HARD-failed analysis
                // (skipped under the partial-success policy). Otherwise a re-analysis
                // whose source moved / was replaced under a persisted TrackId would keep
                // adapting the audition from old audio.
                let succeeded: Vec<TrackId> = results.iter().map(|r| r.track_id.clone()).collect();
                crate::profile_store::prune_failed_profiles(
                    &profile_store,
                    &requested_ids,
                    &succeeded,
                );
            }
            Err(_) => {
                // Every requested track failed — evict any stale profiles outright.
                crate::profile_store::prune_failed_profiles(&profile_store, &requested_ids, &[]);
            }
        }
        result
    })
    .await
    .map_err(|e| CommandError::Other(format!("analyze task: {e}")))?
}

/// Analysis core, free of the Tauri `State` so unit / contract tests can call it
/// directly. The `analyze_tracks` command wraps this and populates the
/// backend-owned profile store from the results. The Tier-2 deep scan is
/// always ON: desktop and the iPhone bridge both go through here, and the
/// bridge resolves the adaptive profile + confidence from `DeepAnalysis`.
/// (A `_lite` deep-scan-off variant existed for a never-shipped mobile
/// battery path; deleted 2026-06-09, owner-approved.)
pub async fn analyze_tracks_core(
    tracks: Vec<AnalyzeRequest>,
) -> CommandResult<Vec<AnalysisResult>> {
    analyze_tracks_core_with_progress_sync(tracks, |_, _| {})
}

/// `analyze_tracks_core` with a batch-level progress callback. The
/// per-track 0..=1 stage fraction from `analyze_one_with_progress` is
/// rescaled across the batch: track i of n spans [i/n, (i+1)/n].
pub async fn analyze_tracks_core_with_progress(
    tracks: Vec<AnalyzeRequest>,
    progress: impl Fn(f32, &str),
) -> CommandResult<Vec<AnalysisResult>> {
    analyze_tracks_core_with_progress_sync(tracks, progress)
}

pub fn analyze_tracks_core_with_progress_sync(
    tracks: Vec<AnalyzeRequest>,
    progress: impl Fn(f32, &str),
) -> CommandResult<Vec<AnalysisResult>> {
    let total = tracks.len();
    let mut out = Vec::with_capacity(total);
    let mut failures: Vec<(TrackId, CommandError)> = Vec::new();
    for (index, req) in tracks.into_iter().enumerate() {
        let track_progress = |frac: f32, label: &'static str| {
            if total > 0 {
                progress((index as f32 + frac) / total as f32, label);
            }
        };
        match analyze_one_with_progress(req.id.clone(), Path::new(&req.path), true, &track_progress)
        {
            Ok(mut result) => {
                nudge_role_by_position(&mut result, index, total);
                out.push(result);
            }
            Err(e) => {
                failures.push((req.id, e));
            }
        }
    }
    // Partial-success policy: if every track failed, surface the first error
    // (otherwise the frontend has no signal at all). If at least one succeeded,
    // return the successes and log the failures — session restore and bulk
    // imports can keep working when one source file has moved.
    // `with_context` keeps the first error's VARIANT so the mobile bridges'
    // machine-readable error code reflects the real class (decode/io/...),
    // not a blanket `Other`.
    if out.is_empty() && !failures.is_empty() {
        let (_, first) = failures.swap_remove(0);
        return Err(first.with_context("analyze failed for all tracks"));
    }
    for (id, error) in failures {
        eprintln!("analyze_tracks: skipping {} — {}", id.as_str(), error);
    }
    Ok(out)
}

/// Derive each result's adaptive [`SourceProfile`] and cache it in the
/// backend-owned store (B2). `set` clears any stale entry when a re-analysis can
/// no longer derive one (e.g. a source that became too short). Pure over a plain
/// `&SourceProfileStore` so it's testable without the Tauri runtime.
pub fn populate_profile_store(
    profile_store: &crate::profile_store::SourceProfileStore,
    results: &[AnalysisResult],
) {
    for result in results {
        profile_store.set(
            result.track_id.clone(),
            SourceProfile::from_analysis(result),
        );
        // Move the (Arc-shared) DeepAnalysis into the store alongside the profile.
        // `None` (e.g. the soft "too short" path) clears any stale entry, keeping
        // the two maps in lockstep.
        profile_store.insert_deep(result.track_id.clone(), result.deep_analysis.clone());
        let band_psr = result
            .deep_analysis
            .as_deref()
            .and_then(crate::deep_analysis::band_psr_p10_db);
        profile_store.set_stand_down(
            result.track_id.clone(),
            Some(crate::guardrails::classify_already_mastered_stand_down(
                result.lufs_integrated,
                result.true_peak_dbtp,
                result.dynamic_range_lu,
                band_psr,
            )),
        );
    }
}

// ============================================================================
// Ceiling-bounded LUFS landing — shared helpers used by every render path.
//
// Pre-extraction, this math was duplicated across render paths
// (mastering_render_with_progress and render_album_plan_impl) plus a shape
// variant in audio.rs::export_landing_gain_lin_for_preview. The B6
// ceiling-bounded behavior shipped as near-identical blocks, and the
// album-plan copy was missed for almost a full session — exactly the drift the
// extraction is meant to prevent.
//
// Two-tier API:
//   * `ceiling_bounded_landing_delta_db`: pure math. Computes the
//     applied delta in dB given pre-measured LUFS+TP and the target/
//     ceiling. Returned value is 0.0 when the landing is a no-op
//     (silent signal, near-zero delta, or no headroom for upward push).
//   * `apply_ceiling_bounded_landing_with_measurements`: math + in-place
//     gain multiply. Returns the applied delta in dB so callers that
//     track post-landing measurements (e.g. the track-export receipt)
//     can shift their tracked LUFS+TP by the same amount.
//   * `measure_and_apply_ceiling_bounded_landing`: full ebur128 pass +
//     apply. For callers that don't already have LUFS+TP measurements
//     in hand (album-simple, album-plan).
//
// The audio.rs live-preview helper delegates to `preview_landing` below and
// returns its gain scalar, so desktop and phone audition measure the same
// render-rate window before applying the pure landing math.
// ============================================================================

/// Compute the LUFS-landing delta in dB given pre-measured loudness +
/// true peak. The ceiling is a delivery spec, not advice: the applied
/// delta is capped so the post-landing true peak never exceeds it —
/// including a NEGATIVE delta (a uniform safety trim, level-only) when
/// the chain output itself carries inter-sample peaks above the
/// ceiling. The limiter caps SAMPLE peaks; hard-driven transient
/// material can overshoot the ceiling by >1 dB of true peak (B2
/// landing-matrix finding, 2026-07-03: synthetic drums × Loud ×
/// LoudRock measured +1.68 dB over a −1 dBTP ceiling pre-fix), which
/// would clip in every streaming platform's re-encode. Returns 0.0
/// when:
///
///   * the target or measurement is non-finite, or the signal is
///     effectively silent (measured_lufs <= -70 LUFS),
///   * the applied delta would be within ±1e-4 dB of zero (numerical
///     no-op — skip the gain multiply entirely).
///
/// The earlier refuse-upward policy (citing the Sonible / Ozone /
/// Mastering The Mix industry survey) was retired during B6 in favor
/// of letting the user push toward their stated target. The live
/// Export LUFS preview shows the resulting level in real time, so
/// what the user hears is what export writes — no hidden cap.
pub(crate) fn ceiling_bounded_landing_delta_db(
    measured_lufs: f32,
    measured_true_peak_dbtp: f32,
    target_lufs: f32,
    ceiling_dbtp: f32,
) -> f32 {
    if !target_lufs.is_finite() || !measured_lufs.is_finite() || measured_lufs <= -70.0 {
        return 0.0;
    }
    let delta_db = target_lufs - measured_lufs;
    // Max delta that keeps post-landing true peak at or below the
    // ceiling. Negative when the chain output is already over it —
    // the trim case. With an unknown TP, allow attenuation but never
    // boost (the pre-fix conservative behavior).
    let tp_cap_db = if measured_true_peak_dbtp.is_finite() {
        ceiling_dbtp - measured_true_peak_dbtp
    } else {
        0.0
    };
    let applied_delta_db = delta_db.min(tp_cap_db);
    if applied_delta_db.abs() > 1.0e-4 {
        applied_delta_db
    } else {
        0.0
    }
}

/// Apply ceiling-bounded LUFS landing in-place to a sample slice given
/// pre-measured loudness + true peak. Returns the applied delta in
/// dB (0.0 if no gain was applied) so callers that track post-landing
/// measurements can shift them by the same amount via
/// `measured_lufs += applied; measured_true_peak_dbtp += applied;`.
///
/// Under a uniform linear gain `g`, integrated LUFS and true-peak
/// both shift by exactly `20·log10(g)` dB — so callers never need to
/// re-run the ebur128 pass after scaling.
fn apply_ceiling_bounded_landing_with_measurements(
    samples: &mut [f32],
    measured_lufs: f32,
    measured_true_peak_dbtp: f32,
    target_lufs: f32,
    ceiling_dbtp: f32,
) -> f32 {
    let applied_delta_db = ceiling_bounded_landing_delta_db(
        measured_lufs,
        measured_true_peak_dbtp,
        target_lufs,
        ceiling_dbtp,
    );
    if applied_delta_db != 0.0 {
        let gain_lin = 10.0_f32.powf(applied_delta_db / 20.0);
        for s in samples.iter_mut() {
            *s *= gain_lin;
        }
    }
    applied_delta_db
}

/// Full-stack ceiling-bounded LUFS landing: measure integrated LUFS +
/// BS.1770 true peak via ebur128, compute the bounded delta, apply in
/// place. Used by render paths that don't already have measurements
/// in hand (album-simple, album-plan). The track-export path measures
/// separately so it can also feed the receipt's `RenderedMeasurements`,
/// and routes through `apply_ceiling_bounded_landing_with_measurements`
/// directly.
pub(crate) fn measure_and_apply_ceiling_bounded_landing(
    samples: &mut [f32],
    sample_rate: u32,
    channels: u16,
    settings: &MasteringSettings,
) -> CommandResult<()> {
    let Some(target_lufs) = settings.effective_target_lufs() else {
        return Ok(());
    };
    if !target_lufs.is_finite() {
        return Ok(());
    }
    let channels_u32 = u32::from(channels.max(1));
    let mut ebu = EbuR128::new(channels_u32, sample_rate, Mode::I | Mode::TRUE_PEAK)
        .map_err(|e| CommandError::Render(format!("ebur128 init: {e}")))?;
    ebu.add_frames_f32(samples)
        .map_err(|e| CommandError::Render(format!("ebur128 feed: {e}")))?;
    let measured_lufs = sanitize_lufs(
        ebu.loudness_global()
            .map_err(|e| CommandError::Render(format!("ebur128 global: {e}")))? as f32,
    );
    let mut peak_lin: f64 = 0.0;
    for ch in 0..channels_u32 {
        let tp = ebu
            .true_peak(ch)
            .map_err(|e| CommandError::Render(format!("ebur128 tp: {e}")))?;
        if tp > peak_lin {
            peak_lin = tp;
        }
    }
    let measured_true_peak_dbtp = if peak_lin > 0.0 {
        (20.0 * peak_lin.log10()) as f32
    } else {
        -60.0
    };
    let ceiling_dbtp = settings.effective_ceiling_dbtp();
    apply_ceiling_bounded_landing_with_measurements(
        samples,
        measured_lufs,
        measured_true_peak_dbtp,
        target_lufs,
        ceiling_dbtp,
    );
    Ok(())
}

/// Measure post-render integrated loudness (BS.1770) of an interleaved f32
/// buffer. Returns the raw ebur128 reading — callers should treat values
/// below -70 LUFS as "effectively silent" and skip downstream gain math, the
/// same way `analyze_tracks` does. Used by the LUFS-landing stage in
/// `mastering_render_with_progress` and by contract tests that verify the
/// landing actually lands.
pub fn measure_integrated_lufs(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> CommandResult<f32> {
    let channels_u32 = u32::from(channels.max(1));
    let mut ebu = EbuR128::new(channels_u32, sample_rate, Mode::I)
        .map_err(|e| CommandError::Render(format!("ebur128 init: {e}")))?;
    ebu.add_frames_f32(samples)
        .map_err(|e| CommandError::Render(format!("ebur128 feed: {e}")))?;
    Ok(ebu
        .loudness_global()
        .map_err(|e| CommandError::Render(format!("ebur128 global: {e}")))? as f32)
}

/// File-path variant: decodes the WAV (or any supported format) via the same
/// pipeline `analyze_tracks` uses, then measures integrated LUFS. Convenience
/// for contract tests that want to read back the rendered output's loudness.
pub fn measure_integrated_lufs_at_path(path: &Path) -> CommandResult<f32> {
    let pcm = crate::decode::decode_full(path)?;
    measure_integrated_lufs(&pcm.samples, pcm.sample_rate, pcm.channels)
}

/// Result of a live-preview loudness landing measurement.
pub struct PreviewLanding {
    /// Linear gain to apply after the chain so the master lands near target.
    pub gain_lin: f32,
    /// Resulting integrated LUFS of the mastered window after the landing gain —
    /// used for audition Volume Match. `f32::NEG_INFINITY` when unavailable
    /// (no target, or an effectively-silent window).
    pub mastered_lufs: f32,
}

/// Compute the live-preview loudness landing for `settings`: process a
/// representative ~8 s window through the chain, measure integrated LUFS +
/// BS.1770 true peak, and route through the SAME `ceiling_bounded_landing_delta_db`
/// the export path uses — so the live preview lands at the same level the full
/// render will. Mirrors the desktop preview path (`audio.rs`). Returns unity gain
/// when there's no loudness target or the window is effectively silent. This is
/// the off-audio-thread measurement the iPhone bridge calls on settings changes.
pub fn preview_landing(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    settings: &MasteringSettings,
) -> CommandResult<PreviewLanding> {
    let mut render_settings = settings.clone();
    render_settings.volume_match = false;
    let unity = PreviewLanding {
        gain_lin: 1.0,
        mastered_lufs: f32::NEG_INFINITY,
    };
    let Some(target_lufs) = render_settings.effective_target_lufs() else {
        return Ok(unity);
    };
    if !target_lufs.is_finite() {
        return Ok(unity);
    }

    let (rendered, rendered_sample_rate) =
        render_preview_landing_window(samples, sample_rate, channels, &render_settings)?;

    let channels_u32 = u32::from(channels.max(1));
    let mut ebu = EbuR128::new(
        channels_u32,
        rendered_sample_rate,
        Mode::I | Mode::TRUE_PEAK,
    )
    .map_err(|e| CommandError::Render(format!("ebur128 init: {e}")))?;
    ebu.add_frames_f32(&rendered)
        .map_err(|e| CommandError::Render(format!("ebur128 feed: {e}")))?;
    let measured = sanitize_lufs(
        ebu.loudness_global()
            .map_err(|e| CommandError::Render(format!("ebur128 global: {e}")))? as f32,
    );
    if !measured.is_finite() || measured <= -70.0 {
        return Ok(unity);
    }
    let mut peak_lin: f64 = 0.0;
    for ch in 0..channels_u32 {
        let tp = ebu
            .true_peak(ch)
            .map_err(|e| CommandError::Render(format!("ebur128 tp: {e}")))?;
        if tp > peak_lin {
            peak_lin = tp;
        }
    }
    let measured_true_peak_dbtp = if peak_lin > 0.0 {
        (20.0 * peak_lin.log10()) as f32
    } else {
        -60.0
    };
    let ceiling_dbtp = render_settings.effective_ceiling_dbtp();
    let applied_delta_db = ceiling_bounded_landing_delta_db(
        measured,
        measured_true_peak_dbtp,
        target_lufs,
        ceiling_dbtp,
    );
    let gain_lin = if applied_delta_db != 0.0 {
        10.0_f32.powf(applied_delta_db / 20.0)
    } else {
        1.0
    };
    Ok(PreviewLanding {
        gain_lin,
        mastered_lufs: measured + applied_delta_db,
    })
}

/// Compute audition-only Volume Match for live preview by measuring the same
/// representative window before and after the mastering chain. The gain is
/// attenuation-only: if the mastered side is not louder than the source, it
/// returns unity rather than boosting.
pub(crate) fn preview_volume_match_gain(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    settings: &MasteringSettings,
) -> CommandResult<f32> {
    let mut render_settings = settings.clone();
    render_settings.volume_match = false;
    let source_window = preview_landing_window(samples, sample_rate, channels);
    let source_lufs = sanitize_lufs(measure_integrated_lufs(
        &source_window,
        sample_rate,
        channels,
    )?);
    if !source_lufs.is_finite() || source_lufs <= -70.0 {
        return Ok(1.0);
    }

    let mut mastered_window = source_window.clone();
    let channels_usize = channels.max(1) as usize;
    let mut chain = crate::dsp::MasteringChain::new(sample_rate, channels_usize, &render_settings);
    chain.process_interleaved(&mut mastered_window, channels_usize);
    let mastered_lufs = sanitize_lufs(measure_integrated_lufs(
        &mastered_window,
        sample_rate,
        channels,
    )?);
    if !mastered_lufs.is_finite() || mastered_lufs <= source_lufs {
        return Ok(1.0);
    }

    let attenuation_db = (source_lufs - mastered_lufs).clamp(-24.0, 0.0);
    Ok(10.0_f32.powf(attenuation_db / 20.0))
}

/// The representative ~8 s middle window the preview landing measures, instead
/// of the full track (≈15-20× cheaper, within ~0.5 dB of full-track for normal
/// music). Same centering math as the desktop preview path.
fn preview_landing_window(samples: &[f32], sample_rate: u32, channels: u16) -> Vec<f32> {
    const PREVIEW_WINDOW_SECS: f32 = 8.0;
    let channels_usize = channels.max(1) as usize;
    let total_frames = samples.len() / channels_usize;
    let window_frames = ((PREVIEW_WINDOW_SECS * sample_rate as f32) as usize).min(total_frames);
    let start_frame = total_frames.saturating_sub(window_frames) / 2;
    let start = start_frame * channels_usize;
    let end = ((start_frame + window_frames) * channels_usize).min(samples.len());
    samples[start..end].to_vec()
}

fn render_preview_landing_window(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    settings: &MasteringSettings,
) -> CommandResult<(Vec<f32>, u32)> {
    let channels_usize = channels.max(1) as usize;
    let mut rendered = preview_landing_window(samples, sample_rate, channels);
    let mut chain = crate::dsp::MasteringChain::new(sample_rate, channels_usize, settings);
    chain.process_interleaved(&mut rendered, channels_usize);

    let rendered_sample_rate = settings.effective_sample_rate(sample_rate);
    if rendered_sample_rate != sample_rate {
        rendered = convert_interleaved(&rendered, sample_rate, rendered_sample_rate, channels)?;
    }

    Ok((rendered, rendered_sample_rate))
}

#[tauri::command]
pub async fn render_track_preview(
    track_id: TrackId,
    track_path: String,
    mut settings: MasteringSettings,
    app: tauri::AppHandle,
    profile_store: tauri::State<'_, std::sync::Arc<crate::profile_store::SourceProfileStore>>,
    render_jobs: tauri::State<'_, RenderJobRegistry>,
) -> CommandResult<RenderJob> {
    // B2: the backend owns the adaptive profile. Track Master is the adaptive
    // surface (album renders via render_album_plan), so album = false; any
    // FE-supplied profile is honored as an override.
    let cached_deep = profile_store.get_deep(&track_id);
    crate::profile_store::apply_resolved_profile(
        &mut settings,
        profile_store.get(&track_id),
        false,
    );
    crate::profile_store::apply_resolved_confidence(&mut settings, cached_deep.clone(), false);
    crate::profile_store::apply_resolved_compression_guards(
        &mut settings,
        cached_deep,
        profile_store.get_stand_down(&track_id),
        false,
    );
    let out_dir = render_output_dir(&app, RenderKind::Preview)?;
    let job_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = render_jobs.register(job_id.clone());
    let track_id_for_progress = track_id.clone();
    let job_id_for_progress = job_id.clone();
    let app_for_progress = app.clone();
    let on_progress = move |fraction: f32| {
        let _ = app_for_progress.emit(
            "render:progress",
            RenderProgress {
                job_id: job_id_for_progress.clone(),
                track_id: track_id_for_progress.clone(),
                kind: RenderKind::Preview,
                fraction,
            },
        );
    };
    let job_id_for_render = job_id.clone();
    let join_result = tauri::async_runtime::spawn_blocking(move || {
        mastering_render_with_cancel(
            track_id,
            Path::new(&track_path),
            &settings,
            &out_dir,
            RenderKind::Preview,
            RenderJobOptions {
                on_progress: Some(&on_progress),
                output_path: None,
                job_id: Some(&job_id_for_render),
                cancel_flag: Some(cancel_flag.as_ref()),
            },
        )
    })
    .await;
    render_jobs.remove(&job_id);
    join_result.map_err(|e| CommandError::Other(format!("preview render task: {e}")))?
}

#[tauri::command]
pub async fn render_track_master(
    track_id: TrackId,
    track_path: String,
    mut settings: MasteringSettings,
    output_path: Option<String>,
    app: tauri::AppHandle,
    profile_store: tauri::State<'_, std::sync::Arc<crate::profile_store::SourceProfileStore>>,
    render_jobs: tauri::State<'_, RenderJobRegistry>,
) -> CommandResult<RenderJob> {
    // B2: backend-owned profile (override > backend-derived cache; album = false
    // because album exports go through render_album_plan, which strips it).
    let cached_deep = profile_store.get_deep(&track_id);
    crate::profile_store::apply_resolved_profile(
        &mut settings,
        profile_store.get(&track_id),
        false,
    );
    crate::profile_store::apply_resolved_confidence(&mut settings, cached_deep.clone(), false);
    crate::profile_store::apply_resolved_compression_guards(
        &mut settings,
        cached_deep,
        profile_store.get_stand_down(&track_id),
        false,
    );
    let out_dir = render_output_dir(&app, RenderKind::Master)?;
    let job_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = render_jobs.register(job_id.clone());
    let explicit_output_path = output_path.map(PathBuf::from);
    let track_id_for_progress = track_id.clone();
    let job_id_for_progress = job_id.clone();
    let app_for_progress = app.clone();
    let on_progress = move |fraction: f32| {
        let _ = app_for_progress.emit(
            "render:progress",
            RenderProgress {
                job_id: job_id_for_progress.clone(),
                track_id: track_id_for_progress.clone(),
                kind: RenderKind::Master,
                fraction,
            },
        );
    };
    let job_id_for_render = job_id.clone();
    let join_result = tauri::async_runtime::spawn_blocking(move || {
        mastering_render_with_cancel(
            track_id,
            Path::new(&track_path),
            &settings,
            &out_dir,
            RenderKind::Master,
            RenderJobOptions {
                on_progress: Some(&on_progress),
                output_path: explicit_output_path.as_deref(),
                job_id: Some(&job_id_for_render),
                cancel_flag: Some(cancel_flag.as_ref()),
            },
        )
    })
    .await;
    render_jobs.remove(&job_id);
    let outcome =
        join_result.map_err(|e| CommandError::Other(format!("master render task: {e}")))?;
    match &outcome {
        Ok(job) => crate::diagnostics::info(format!(
            "render master finished: job {job_id} status {:?} -> {:?}",
            job.status, job.output_paths
        )),
        Err(e) => crate::diagnostics::error(format!("render master failed: job {job_id}: {e}")),
    }
    outcome
}

// ============================================================================
// Phase B Step 3: AlbumPlan-driven render path.
//
// Consumes an AlbumPlan + per-track settings + per-track source paths and
// produces:
//   1. NN per-track WAVs named NN-<sanitized_title>.wav
//   2. one continuous album.wav with TransitionSpec silence between tracks
//   3. manifest.json documenting the plan + per-track output paths +
//      post-render measured integrated LUFS for each track
//
// Each track's `MasteringSettings` is shadowed by the plan's
// `arc_lufs_offset_db` (added to the effective LUFS target) and
// `intensity_scale` (multiplied onto `settings.intensity`).
//
// Album export resolves one delivery sample rate and channel count up front;
// mixed-rate tracks are resampled, and mono/stereo/above-stereo sources are
// converted or folded into that album delivery shape.
// ============================================================================

// `Serialize` is added so the Rust↔TS wire-drift gate (tests/wire_shape.rs)
// can emit a canonical sample of this album-render input; production only ever
// deserializes it.
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumTrackRenderInput {
    pub track_id: TrackId,
    pub source_path: String,
    pub settings: MasteringSettings,
    /// Owner decision 2026-07-03 D9 — "full sound exemption": when true this
    /// track renders with its own settings and its own loudness target (no
    /// arc offset, no character bias, no album intensity scale). The album
    /// delivery format (sample rate / bit depth / channel count) still
    /// applies so the record stays one coherent deliverable. Serde-default
    /// so older payloads deserialize as "follows album intent."
    #[serde(default)]
    pub override_album: bool,
}

#[derive(Debug, Deserialize)]
pub struct AlbumPlanRenderRequest {
    pub plan: AlbumPlan,
    pub tracks: Vec<AlbumTrackRenderInput>,
}

/// Reject empty or `..`-bearing per-track source paths before any album render
/// decodes them, mirroring the guard `mastering_render_with_progress` applies to
/// single-track sources. Album rendering decodes each `source_path` directly, so
/// without this the album path was the one place a traversal path slipped through.
pub(crate) fn validate_album_source_paths(tracks: &[AlbumTrackRenderInput]) -> CommandResult<()> {
    for input in tracks {
        if input.source_path.is_empty() {
            return Err(CommandError::InvalidPath("empty path".to_string()));
        }
        if crate::files::has_parent_dir_component(Path::new(&input.source_path)) {
            return Err(CommandError::InvalidPath(format!(
                "path traversal not allowed: {}",
                input.source_path
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct AlbumTrackRenderRecord {
    pub track_id: TrackId,
    pub position: u32,
    pub output_path: String,
    pub measured_lufs: f32,
    pub source_sample_rate: u32,
    pub rendered_sample_rate: u32,
    pub source_channels: u16,
    pub rendered_channels: u16,
    /// True when the track rendered with its own settings/target instead of
    /// the album intent (D9 full sound exemption) — surfaced in the manifest
    /// and export report so the receipt never hides it.
    pub override_album: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct AlbumRenderReport {
    pub job_id: String,
    pub status: JobStatus,
    pub album_wav_path: String,
    pub manifest_path: String,
    pub requested_sample_rate: Option<u32>,
    pub rendered_sample_rate: u32,
    pub source_sample_rates: Vec<u32>,
    pub bit_depth: u16,
    pub rendered_channels: u16,
    pub source_channels: Vec<u16>,
    pub tracks: Vec<AlbumTrackRenderRecord>,
}

#[derive(Debug, Deserialize)]
pub struct PlanAlbumRequest {
    pub title: String,
    pub analyses: Vec<AnalysisResult>,
    pub durations: Vec<f64>,
    pub arc: AlbumArc,
    pub intensity: f32,
    #[serde(default)]
    pub delivery_sample_rate: Option<u32>,
    #[serde(default)]
    pub delivery_bit_depth: Option<u16>,
}

/// Phase B Step 4: thin Tauri wrapper around `album::build_album_plan`.
/// Lets the frontend pick (arc, intensity) and immediately receive the
/// per-track plan without duplicating the math in TypeScript.
#[tauri::command]
pub async fn plan_album(request: PlanAlbumRequest) -> CommandResult<AlbumPlan> {
    let refs: Vec<&AnalysisResult> = request.analyses.iter().collect();
    let mut plan = crate::album::build_album_plan(
        request.title,
        &refs,
        &request.durations,
        request.arc,
        request.intensity,
    );
    plan.delivery_sample_rate = request.delivery_sample_rate;
    plan.delivery_bit_depth = request.delivery_bit_depth;
    Ok(plan)
}

#[tauri::command]
pub async fn render_album_plan(
    request: AlbumPlanRenderRequest,
    output_dir: Option<String>,
    app: tauri::AppHandle,
    render_jobs: tauri::State<'_, RenderJobRegistry>,
) -> CommandResult<AlbumRenderReport> {
    let out_dir = match output_dir {
        Some(path) => explicit_output_dir(Path::new(&path))?,
        None => render_output_dir(&app, RenderKind::Album)?,
    };
    let job_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = render_jobs.register(job_id.clone());
    let job_id_for_progress = job_id.clone();
    let app_for_progress = app.clone();
    let on_progress = move |fraction: f32| {
        let _ = app_for_progress.emit(
            "render:progress",
            RenderProgress {
                job_id: job_id_for_progress.clone(),
                track_id: TrackId(String::new()),
                kind: RenderKind::Album,
                fraction,
            },
        );
    };
    let job_id_for_render = job_id.clone();
    let join_result = tauri::async_runtime::spawn_blocking(move || {
        crate::album_render::render_album_plan_impl_with_cancel(
            &request,
            &out_dir,
            Some(&on_progress),
            Some(cancel_flag.as_ref()),
            Some(&job_id_for_render),
        )
    })
    .await;
    render_jobs.remove(&job_id);
    let outcome =
        join_result.map_err(|e| CommandError::Other(format!("album render task: {e}")))?;
    match &outcome {
        Ok(report) => crate::diagnostics::info(format!(
            "render album finished: job {job_id} status {:?}, {} track(s)",
            report.status,
            report.tracks.len()
        )),
        Err(e) => crate::diagnostics::error(format!("render album failed: job {job_id}: {e}")),
    }
    outcome
}

pub fn render_output_dir(app: &tauri::AppHandle, kind: RenderKind) -> CommandResult<PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| CommandError::Other(format!("app_data_dir: {e}")))?;
    let leaf = match kind {
        RenderKind::Preview => "previews",
        RenderKind::Master => "masters",
        RenderKind::Album => "albums",
    };
    let dir = app_data.join("renders").join(leaf);
    std::fs::create_dir_all(&dir).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(dir)
}

/// Startup sweep for orphaned render temp files (D5 runtime-abuse review).
/// Every render writes through a `{name}.{uuid}.tmp` sibling and renames on
/// success; the in-process error paths remove their own tmp, but a process
/// kill / OS shutdown mid-render bypasses destructors and strands the file
/// forever — nothing else ever touches it. At launch no render is running,
/// so any `.tmp` under the app's own `renders/` tree is guaranteed orphaned.
/// Only files whose name ends in `.tmp` inside the three render leaf dirs
/// are considered; user-chosen export locations are never swept.
pub fn sweep_orphaned_render_tmp(app_data: &Path) -> usize {
    let mut removed = 0;
    for leaf in ["previews", "masters", "albums"] {
        let dir = app_data.join("renders").join(leaf);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_tmp = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".tmp"));
            if is_tmp && path.is_file() && std::fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
    }
    removed
}

pub fn mastering_render(
    track_id: TrackId,
    source_path: &Path,
    settings: &MasteringSettings,
    out_dir: &Path,
    kind: RenderKind,
) -> CommandResult<RenderJob> {
    mastering_render_with_progress(track_id, source_path, settings, out_dir, kind, None, None)
}

pub fn mastering_render_to_path(
    track_id: TrackId,
    source_path: &Path,
    settings: &MasteringSettings,
    out_dir: &Path,
    kind: RenderKind,
    output_path: &Path,
) -> CommandResult<RenderJob> {
    mastering_render_with_progress(
        track_id,
        source_path,
        settings,
        out_dir,
        kind,
        None,
        Some(output_path),
    )
}

/// Same as `mastering_render` but accepts an optional progress callback that
/// fires after each ~4096-frame chunk with the current 0.0–1.0 fraction.
/// Phase 12.1 perf — `render_track_master` / `render_track_preview` thread an
/// AppHandle-emitting closure through here so
/// the frontend can render a real progress bar instead of an indeterminate
/// "Rendering…" message. Contract tests pass `None` and ignore progress.
pub fn mastering_render_with_progress(
    track_id: TrackId,
    source_path: &Path,
    settings: &MasteringSettings,
    out_dir: &Path,
    kind: RenderKind,
    on_progress: Option<&dyn Fn(f32)>,
    output_path: Option<&Path>,
) -> CommandResult<RenderJob> {
    mastering_render_with_cancel(
        track_id,
        source_path,
        settings,
        out_dir,
        kind,
        RenderJobOptions {
            on_progress,
            output_path,
            job_id: None,
            cancel_flag: None,
        },
    )
}

#[derive(Default)]
pub struct RenderJobOptions<'a> {
    pub on_progress: Option<&'a dyn Fn(f32)>,
    pub output_path: Option<&'a Path>,
    pub job_id: Option<&'a str>,
    pub cancel_flag: Option<&'a AtomicBool>,
}

pub fn mastering_render_with_cancel(
    track_id: TrackId,
    source_path: &Path,
    settings: &MasteringSettings,
    out_dir: &Path,
    kind: RenderKind,
    options: RenderJobOptions<'_>,
) -> CommandResult<RenderJob> {
    let job_id = options
        .job_id
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let started_at_iso = now_iso();
    let source_path_str = source_path.to_string_lossy().to_string();
    if source_path_str.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    if crate::files::has_parent_dir_component(source_path) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {source_path_str}"
        )));
    }
    if !source_path.exists() {
        return Err(CommandError::Io(format!(
            "source file not found: {source_path_str}"
        )));
    }
    let out_path = match options.output_path {
        Some(path) => explicit_output_path(path, source_path)?,
        None => unique_output_path(out_dir, source_path, &track_id, kind)?,
    };

    // Per-stage wall-clock markers, logged at the end of the render. F9/F11
    // (owner smoke): a "15-minute export" report is undiagnosable without
    // knowing WHICH stage ate the time — decode vs chain vs SRC vs measure
    // vs the write to the chosen destination.
    let t_render_start = std::time::Instant::now();
    let stage_ms = |t: std::time::Instant| t.elapsed().as_millis();

    let t_stage = std::time::Instant::now();
    let pcm = crate::decode::decode_full(source_path)?;
    let decode_ms = stage_ms(t_stage);
    if pcm.samples.is_empty() {
        return Err(CommandError::Decode(
            "no samples decoded from source".to_string(),
        ));
    }
    let channels = pcm.channels as usize;
    let channels_max = channels.max(1);
    let mut samples = pcm.samples;
    let mut render_settings = settings.clone();
    render_settings.volume_match = false;
    let mut chain =
        crate::dsp::MasteringChain::new(pcm.sample_rate, channels_max, &render_settings);

    // Process in 4096-frame chunks (~93 ms at 44.1 kHz) so progress callbacks
    // fire ~10 times per second. The chain's per-frame state (limiter
    // lookahead, biquad memory) flows through chunk boundaries because we
    // call into the same `chain` instance for each chunk.
    const CHUNK_FRAMES: usize = 4096;
    let chunk_samples = CHUNK_FRAMES * channels_max;
    let total_samples = samples.len();
    let mut processed = 0;
    if let Some(cb) = options.on_progress {
        cb(0.0);
    }
    if render_cancelled(options.cancel_flag) {
        return Ok(cancelled_render_job(
            job_id,
            kind,
            vec![track_id],
            0.0,
            started_at_iso,
        ));
    }
    let t_stage = std::time::Instant::now();
    // Progress emits are throttled to ≥1% fraction steps (or the final 1.0):
    // the chain runs many times faster than realtime, so an unthrottled emit
    // per 4096-frame chunk produced hundreds of IPC events per second — pure
    // overhead for a progress bar nobody can read that fast (F9).
    let mut last_emitted_fraction = 0.0_f32;
    while processed < total_samples {
        if render_cancelled(options.cancel_flag) {
            return Ok(cancelled_render_job(
                job_id,
                kind,
                vec![track_id],
                processed as f32 / total_samples.max(1) as f32,
                started_at_iso,
            ));
        }
        let end = (processed + chunk_samples).min(total_samples);
        chain.process_interleaved(&mut samples[processed..end], channels_max);
        processed = end;
        if let Some(cb) = options.on_progress {
            let fraction = (processed as f32 / total_samples.max(1) as f32).min(1.0);
            if fraction >= 1.0 || fraction - last_emitted_fraction >= 0.01 {
                last_emitted_fraction = fraction;
                cb(fraction);
            }
        }
    }
    let chain_ms = stage_ms(t_stage);
    if render_cancelled(options.cancel_flag) {
        return Ok(cancelled_render_job(
            job_id,
            kind,
            vec![track_id],
            1.0,
            started_at_iso,
        ));
    }
    // RS-09 fix (2026-07-03): drain the limiter's lookahead so the final
    // ~3 ms of the master isn't dropped and the output stays sample-aligned
    // with the source. Must run BEFORE sample-rate conversion/measurement.
    let t_stage = std::time::Instant::now();
    chain.flush_render_tail(&mut samples, channels_max);

    let rendered_sample_rate = render_settings.effective_sample_rate(pcm.sample_rate);
    if rendered_sample_rate != pcm.sample_rate {
        samples = convert_interleaved(
            &samples,
            pcm.sample_rate,
            rendered_sample_rate,
            pcm.channels,
        )?;
    }
    let tail_and_src_ms = stage_ms(t_stage);
    let t_stage = std::time::Instant::now();

    // Single full BS.1770 pass over the post-chain, post-SRC samples — used both to
    // decide LUFS landing and to populate the rendered-output measurements
    // for the export receipt (Codex audit 2026-05-13 P0: the receipt must
    // describe the rendered output, not the source analysis).
    //
    // We measure once and shift the result mathematically if landing applies.
    // Under a uniform linear gain `g`, integrated LUFS and true-peak both
    // shift by exactly `20·log10(g)` dB, and LRA (a range between gated
    // loudness percentiles) is preserved. So we never need to re-run the
    // ~25 MB-per-track ebur128 pass after scaling.
    let channels_u32 = u32::from(pcm.channels.max(1));
    let mut ebu = EbuR128::new(
        channels_u32,
        rendered_sample_rate,
        Mode::I | Mode::LRA | Mode::TRUE_PEAK,
    )
    .map_err(|e| CommandError::Render(format!("ebur128 init: {e}")))?;
    ebu.add_frames_f32(&samples)
        .map_err(|e| CommandError::Render(format!("ebur128 feed: {e}")))?;
    let mut measured_lufs = sanitize_lufs(
        ebu.loudness_global()
            .map_err(|e| CommandError::Render(format!("ebur128 global: {e}")))? as f32,
    );
    let lra = ebu
        .loudness_range()
        .map_err(|e| CommandError::Render(format!("ebur128 lra: {e}")))? as f32;
    let mut peak_lin: f64 = 0.0;
    for ch in 0..channels_u32 {
        let tp = ebu
            .true_peak(ch)
            .map_err(|e| CommandError::Render(format!("ebur128 tp: {e}")))?;
        if tp > peak_lin {
            peak_lin = tp;
        }
    }
    let mut measured_true_peak_dbtp = if peak_lin > 0.0 {
        (20.0 * peak_lin.log10()) as f32
    } else {
        -60.0
    };
    let measure_ms = stage_ms(t_stage);

    // Ceiling-bounded LUFS landing. Routes through the shared helper
    // with the LUFS+TP we already measured for the receipt. The
    // helper returns the applied delta in dB so we can shift the
    // tracked measurements (which feed `RenderedMeasurements`) in
    // lockstep — under a uniform linear gain, integrated LUFS and
    // true-peak both shift by exactly the same dB amount, so no
    // second ebur128 pass is needed.
    if let Some(target_lufs) = render_settings.effective_target_lufs() {
        let ceiling_dbtp = render_settings.effective_ceiling_dbtp();
        let applied_delta_db = apply_ceiling_bounded_landing_with_measurements(
            &mut samples,
            measured_lufs,
            measured_true_peak_dbtp,
            target_lufs,
            ceiling_dbtp,
        );
        if applied_delta_db != 0.0 {
            measured_lufs += applied_delta_db;
            measured_true_peak_dbtp += applied_delta_db;
        }
    }

    let bit_depth = render_settings.effective_bit_depth();
    // B5 — record what adaptation actually produced this master. `render_settings`
    // already carries the backend-resolved profile (B2), so this mirrors the
    // chain's own gating: a digest is recorded only when a profile was present AND
    // the strength was non-zero (otherwise the guardrails were inert).
    let effective_adaptive_strength = render_settings
        .advanced
        .adaptive_strength
        .unwrap_or(crate::guardrails::ADAPTIVE_STRENGTH_DEFAULT)
        .clamp(0.0, 1.0);
    let source_profile_digest = render_settings
        .advanced
        .source_profile
        .as_ref()
        .filter(|_| effective_adaptive_strength > 0.0)
        .map(|p| p.digest());
    // Tier-2 Phase B traceability: record the per-axis confidence that gated the
    // trims (present only when confidence resolved — gate on + deep + strength > 0).
    let confidence_digest = render_settings
        .advanced
        .source_confidence
        .as_ref()
        .filter(|_| render_settings.advanced.source_profile.is_some())
        .filter(|_| effective_adaptive_strength > 0.0)
        .map(|c| c.digest());
    let compression_digest =
        crate::guardrails::compression_plan_for_resolved_settings(&render_settings).digest;
    let measurements = RenderedMeasurements {
        lufs_integrated: measured_lufs,
        true_peak_dbtp: measured_true_peak_dbtp,
        dynamic_range_lu: if lra.is_finite() { lra } else { 0.0 },
        sample_rate: rendered_sample_rate,
        bit_depth,
        effective_adaptive_strength,
        source_profile_digest,
        confidence_digest,
        compression_digest,
    };
    if render_cancelled(options.cancel_flag) {
        return Ok(cancelled_render_job(
            job_id,
            kind,
            vec![track_id],
            1.0,
            started_at_iso,
        ));
    }
    // If the chosen path gained a file while we rendered (overlapping
    // export), the write diverts to a `__{n}` sibling — report where the
    // render actually landed.
    let t_stage = std::time::Instant::now();
    let actual_out_path = write_wav(
        &out_path,
        &samples,
        rendered_sample_rate,
        pcm.channels,
        bit_depth,
    )?;
    let write_ms = stage_ms(t_stage);
    if render_cancelled(options.cancel_flag) {
        let _ = std::fs::remove_file(&actual_out_path);
        return Ok(cancelled_render_job(
            job_id,
            kind,
            vec![track_id],
            1.0,
            started_at_iso,
        ));
    }
    if let Some(cb) = options.on_progress {
        cb(1.0);
    }
    // One line per render; names the slow stage AND the destination so a
    // diagnostics report alone can answer "why did this export take 15
    // minutes" (write time is where OneDrive/USB destinations show up).
    crate::diagnostics::info(format!(
        "render {kind:?} job {job_id}: decode {decode_ms}ms, chain {chain_ms}ms, \
         tail+src {tail_and_src_ms}ms, measure {measure_ms}ms, write {write_ms}ms, \
         total {}ms -> {}",
        t_render_start.elapsed().as_millis(),
        actual_out_path
            .parent()
            .unwrap_or(&actual_out_path)
            .display()
    ));

    Ok(RenderJob {
        id: job_id.clone(),
        job_id,
        kind,
        target_tracks: vec![track_id],
        status: JobStatus::Done,
        progress: 1.0,
        started_at_iso,
        output_paths: vec![actual_out_path.to_string_lossy().to_string()],
        measurements: Some(measurements),
    })
}

pub fn render_cancelled(cancel_flag: Option<&AtomicBool>) -> bool {
    cancel_flag.is_some_and(|flag| flag.load(Ordering::SeqCst))
}

pub fn cancelled_render_job(
    job_id: String,
    kind: RenderKind,
    target_tracks: Vec<TrackId>,
    progress: f32,
    started_at_iso: String,
) -> RenderJob {
    RenderJob {
        id: job_id.clone(),
        job_id,
        kind,
        target_tracks,
        status: JobStatus::Cancelled,
        progress: progress.clamp(0.0, 1.0),
        started_at_iso,
        output_paths: Vec::new(),
        measurements: None,
    }
}

pub(crate) fn comparable_existing_or_parent_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }

    let Some(file_name) = path.file_name() else {
        return path.to_path_buf();
    };
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    parent
        .canonicalize()
        .map(|p| p.join(file_name))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn explicit_output_path(path: &Path, source_path: &Path) -> CommandResult<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(CommandError::InvalidPath("empty output path".to_string()));
    }
    if crate::files::has_parent_dir_component(path) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {}",
            path.to_string_lossy()
        )));
    }
    if path.file_name().is_none() {
        return Err(CommandError::InvalidPath(format!(
            "output path must include a file name: {}",
            path.to_string_lossy()
        )));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| CommandError::Io(e.to_string()))?;
        }
    }
    if comparable_existing_or_parent_path(path) == comparable_existing_or_parent_path(source_path) {
        return Err(CommandError::InvalidPath(
            "output path would overwrite the source file".to_string(),
        ));
    }
    // Non-negotiable: never overwrite a prior render by default. If the
    // chosen path already exists (and isn't the source, handled above —
    // e.g. the user accepted "Replace" in the save dialog), save to a
    // unique sibling instead. Same `__{n}` collision suffix as
    // `unique_output_path` so the two render paths stay consistent.
    if path.exists() {
        // Same `__{n}` sibling convention the finalize step uses — the
        // user's chosen stem/extension stay recognizable.
        return crate::wav_writer::unique_sibling(path);
    }
    Ok(path.to_path_buf())
}

fn explicit_output_dir(path: &Path) -> CommandResult<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(CommandError::InvalidPath(
            "empty output directory".to_string(),
        ));
    }
    if crate::files::has_parent_dir_component(path) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {}",
            path.to_string_lossy()
        )));
    }
    std::fs::create_dir_all(path).map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(path.to_path_buf())
}

pub(crate) fn unique_output_path(
    out_dir: &Path,
    source: &Path,
    track_id: &TrackId,
    kind: RenderKind,
) -> CommandResult<PathBuf> {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("track");
    let kind_tag = match kind {
        RenderKind::Preview => "preview",
        RenderKind::Master => "master",
        RenderKind::Album => "album",
    };
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let id_short = track_id.as_str().chars().take(8).collect::<String>();
    let filename = format!("{stem}__{kind_tag}__{id_short}__{ts}.wav");
    let candidate = out_dir.join(&filename);
    if !candidate.exists() {
        return Ok(candidate);
    }
    for n in 1..1000 {
        let alt = out_dir.join(format!("{stem}__{kind_tag}__{id_short}__{ts}__{n}.wav"));
        if !alt.exists() {
            return Ok(alt);
        }
    }
    Err(CommandError::Io(
        "could not generate unique output path".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_output_dir_creates_selected_album_folder() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let chosen = tmp.path().join("Album Masters").join("Round 1");

        let out_dir = explicit_output_dir(&chosen).expect("explicit output dir");

        assert_eq!(out_dir, chosen);
        assert!(out_dir.is_dir(), "selected album folder should be created");
    }

    #[test]
    fn explicit_output_dir_rejects_empty_path() {
        let err = explicit_output_dir(Path::new("")).expect_err("empty dir should fail");

        assert!(
            matches!(err, CommandError::InvalidPath(ref message) if message == "empty output directory"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn explicit_output_dir_rejects_path_traversal() {
        let err = explicit_output_dir(Path::new("../escape/Album"))
            .expect_err("traversal dir should fail");

        assert!(
            matches!(err, CommandError::InvalidPath(ref m) if m.contains("path traversal not allowed")),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn explicit_output_path_rejects_path_traversal() {
        let err = explicit_output_path(Path::new("../escape/out.wav"), Path::new("source.wav"))
            .expect_err("traversal output path should fail");

        assert!(
            matches!(err, CommandError::InvalidPath(ref m) if m.contains("path traversal not allowed")),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn explicit_output_path_creates_parent_for_native_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let source = tmp.path().join("source.wav");
        std::fs::write(&source, b"source").expect("write source");
        let chosen = tmp.path().join("Masters").join("track master.wav");

        let out_path = explicit_output_path(&chosen, &source).expect("explicit output path");

        assert_eq!(out_path, chosen);
        assert!(
            chosen.parent().expect("parent").is_dir(),
            "selected output parent should be created"
        );
    }

    /// L15 regression: an explicit output path that collides with an
    /// existing prior render must NOT overwrite it. The function returns a
    /// unique sibling in the same directory that does not yet exist, so a
    /// user accepting "Replace" in the save dialog never clobbers a prior
    /// master.
    #[test]
    fn explicit_output_path_uniquifies_when_target_already_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let source = tmp.path().join("source.wav");
        std::fs::write(&source, b"source").expect("write source");
        let chosen = tmp.path().join("track master.wav");
        std::fs::write(&chosen, b"prior render").expect("write prior render");

        let out_path = explicit_output_path(&chosen, &source).expect("explicit output path");

        assert_ne!(
            out_path, chosen,
            "must not return the colliding path that holds a prior render"
        );
        assert_eq!(
            out_path.parent(),
            chosen.parent(),
            "unique sibling must live in the same directory"
        );
        assert!(
            !out_path.exists(),
            "returned path must not yet exist (it was uniquified)"
        );
        assert!(
            chosen.exists(),
            "the prior render must be left intact, not overwritten"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn explicit_output_path_creates_parent_for_windows_backslash_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let source = tmp.path().join("source.wav");
        std::fs::write(&source, b"source").expect("write source");
        let chosen = format!(
            "{}\\Masters\\track master.wav",
            tmp.path().to_string_lossy()
        );
        let chosen = PathBuf::from(chosen);

        let out_path = explicit_output_path(&chosen, &source).expect("explicit output path");

        assert_eq!(out_path, chosen);
        assert!(
            out_path.parent().expect("parent").is_dir(),
            "Windows backslash output parent should be created"
        );
    }

    #[test]
    fn preview_landing_window_uses_effective_render_sample_rate() {
        let source_rate = 48_000u32;
        let render_rate = 44_100u32;
        let channels = 2u16;
        let frames = source_rate as usize * 8;
        let omega = std::f32::consts::TAU * 440.0 / source_rate as f32;
        let mut samples = Vec::with_capacity(frames * channels as usize);
        for i in 0..frames {
            let sample = 0.25 * (omega * i as f32).sin();
            for _ in 0..channels {
                samples.push(sample);
            }
        }
        let settings = MasteringSettings {
            preset: Preset::Universal,
            intensity: 0.5,
            eq_sub_db: 0.0,
            eq_low_db: 0.0,
            eq_low_mid_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_mid_db: 0.0,
            eq_high_db: 0.0,
            eq_sparkle_db: 0.0,
            volume_match: false,
            source_lufs_integrated: None,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            delivery_profile: DeliveryProfile::Custom,
            album: None,
            advanced: AdvancedSettings {
                lufs_offset_db: Some(-14.0),
                ceiling_dbtp: Some(-1.0),
                target_sample_rate: Some(render_rate),
                ..Default::default()
            },
        };

        let (rendered, measured_rate) =
            render_preview_landing_window(&samples, source_rate, channels, &settings)
                .expect("render preview landing window");

        assert_eq!(measured_rate, render_rate);
        let rendered_frames = rendered.len() / channels as usize;
        let expected_frames = render_rate as usize * 8;
        assert!(
            rendered_frames.abs_diff(expected_frames) <= 1,
            "landing measurement window should be resampled to render rate; \
             got {rendered_frames} frames, expected {expected_frames}"
        );
        let landing =
            preview_landing(&samples, source_rate, channels, &settings).expect("preview landing");
        assert!(landing.gain_lin.is_finite());
        assert!(landing.mastered_lufs.is_finite());
    }

    // ========================================================================
    // ceiling_bounded_landing_delta_db — mechanical gates for the shared
    // landing math now used by all four render/preview paths. Tests
    // exercise the pure math via input/output pairs so a future change
    // to the formula can't silently shift behavior on any single caller.
    // ========================================================================

    /// Downward delta applies in full (the limiter has already capped
    /// peaks at ceiling, so attenuating only moves them further away).
    #[test]
    fn ceiling_bounded_landing_downward_applies_full_delta() {
        // measured -10 LUFS, peak -1 dBTP, target -14 LUFS, ceiling -1.
        // delta = target - measured = -4. Should apply in full.
        let applied = ceiling_bounded_landing_delta_db(-10.0, -1.0, -14.0, -1.0);
        assert!(
            (applied - -4.0).abs() < 1.0e-6,
            "downward delta should apply in full; got {applied}"
        );
    }

    /// Upward delta applies in full when there's headroom below ceiling.
    /// Verifies the post-B6 "let the slider push upward when safe"
    /// behavior is preserved through the extraction.
    #[test]
    fn ceiling_bounded_landing_upward_uses_full_headroom_when_available() {
        // measured -23 LUFS, peak -15 dBTP, target -14, ceiling -1.
        // delta = +9; headroom = 14. Push the full +9.
        let applied = ceiling_bounded_landing_delta_db(-23.0, -15.0, -14.0, -1.0);
        assert!(
            (applied - 9.0).abs() < 1.0e-6,
            "upward delta should apply in full when headroom > delta; got {applied}"
        );
    }

    /// Upward delta is clamped by ceiling headroom — verifies the cap
    /// fires when the chain already pushed peaks near the ceiling.
    #[test]
    fn ceiling_bounded_landing_upward_clamped_by_ceiling_headroom() {
        // measured -10 LUFS, peak -3 dBTP, target -6, ceiling -1.
        // delta = +4; headroom = 2. Push only +2.
        let applied = ceiling_bounded_landing_delta_db(-10.0, -3.0, -6.0, -1.0);
        assert!(
            (applied - 2.0).abs() < 1.0e-6,
            "upward delta should clamp to ceiling headroom; got {applied}"
        );
    }

    /// Upward delta with zero headroom (post-chain peak already at
    /// ceiling) clamps to zero — no push, no change. This is the
    /// "slider feels inert on already-limiter-slammed material" case,
    /// which is the spec-correct behavior.
    #[test]
    fn ceiling_bounded_landing_upward_zero_when_no_headroom() {
        // measured -10 LUFS, peak -1 dBTP (at ceiling), target -6.
        // delta = +4; headroom = 0. Push zero.
        let applied = ceiling_bounded_landing_delta_db(-10.0, -1.0, -6.0, -1.0);
        assert_eq!(
            applied, 0.0,
            "no headroom should produce zero applied delta; got {applied}"
        );
    }

    /// Silent signal (-70 LUFS gate) bypasses landing entirely.
    /// Pre-extraction, every duplicate copy of the math had the
    /// `measured_lufs > -70.0` guard. Verifies the extracted helper
    /// inherits it.
    #[test]
    fn ceiling_bounded_landing_skips_silent_signal() {
        let applied = ceiling_bounded_landing_delta_db(-80.0, -60.0, -14.0, -1.0);
        assert_eq!(
            applied, 0.0,
            "silent signal (-70 LUFS gate) should produce zero delta; got {applied}"
        );
    }

    /// Non-finite target or measurement bypasses landing — silent
    /// guard against NaN propagation into the gain stage.
    #[test]
    fn ceiling_bounded_landing_skips_non_finite_inputs() {
        assert_eq!(
            ceiling_bounded_landing_delta_db(f32::NAN, -1.0, -14.0, -1.0),
            0.0,
            "NaN measured_lufs should produce zero delta"
        );
        assert_eq!(
            ceiling_bounded_landing_delta_db(-10.0, -1.0, f32::NAN, -1.0),
            0.0,
            "NaN target should produce zero delta"
        );
        assert_eq!(
            ceiling_bounded_landing_delta_db(-10.0, -1.0, f32::INFINITY, -1.0),
            0.0,
            "infinite target should produce zero delta"
        );
    }

    /// Near-zero delta (chain already lands at target within 1e-4 dB)
    /// produces zero so the gain multiply is skipped entirely.
    /// Prevents tiny floating-point noise from triggering a
    /// near-identity gain pass over every sample.
    #[test]
    fn sweep_orphaned_render_tmp_removes_only_tmp_files_in_render_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let app_data = tmp.path();
        let masters = app_data.join("renders").join("masters");
        let albums = app_data.join("renders").join("albums");
        std::fs::create_dir_all(&masters).expect("masters dir");
        std::fs::create_dir_all(&albums).expect("albums dir");

        let orphan1 = masters.join("song.wav.5a3e.tmp");
        let orphan2 = albums.join("album.wav.9f01.tmp");
        let keeper_wav = masters.join("song.wav");
        let keeper_elsewhere = app_data.join("not-renders.tmp");
        for p in [&orphan1, &orphan2, &keeper_wav, &keeper_elsewhere] {
            std::fs::write(p, b"x").expect("write fixture");
        }

        let removed = sweep_orphaned_render_tmp(app_data);

        assert_eq!(removed, 2, "exactly the two orphans are swept");
        assert!(!orphan1.exists() && !orphan2.exists());
        assert!(keeper_wav.exists(), "finished renders must survive");
        assert!(
            keeper_elsewhere.exists(),
            "files outside the render dirs are never touched",
        );
    }

    #[test]
    fn ceiling_bounded_landing_skips_negligible_delta() {
        // measured -14.00005, target -14. Delta = -5e-5, abs < 1e-4.
        let applied = ceiling_bounded_landing_delta_db(-14.00005, -1.0, -14.0, -1.0);
        assert_eq!(
            applied, 0.0,
            "delta below the ±1e-4 dB noise threshold should produce zero; got {applied}"
        );
    }

    /// B2 landing-matrix fix (2026-07-03): when the chain output's true
    /// peak already exceeds the ceiling (inter-sample overshoot on
    /// transient material — the limiter caps sample peaks only), the
    /// landing must apply a NEGATIVE safety trim down to the ceiling,
    /// even when the loudness target is asking for a boost.
    #[test]
    fn ceiling_bounded_landing_trims_down_when_true_peak_exceeds_ceiling() {
        // Boost wanted (+3.5 dB toward target) but TP is 1.68 dB OVER
        // the ceiling: the applied delta must be −1.68 (trim), not 0.
        let applied = ceiling_bounded_landing_delta_db(-14.0, 0.68, -10.5, -1.0);
        assert!(
            (applied - -1.68).abs() < 1.0e-4,
            "expected a −1.68 dB safety trim; got {applied}"
        );

        // Pull-down case that STILL leaves TP over the ceiling: the trim
        // must go past the target delta to honor the ceiling.
        let applied = ceiling_bounded_landing_delta_db(-13.5, 0.5, -14.0, -1.0);
        assert!(
            (applied - -1.5).abs() < 1.0e-4,
            "ceiling outranks target: expected −1.5 dB, got {applied}"
        );

        // TP safely under the ceiling: boosts remain headroom-bounded
        // exactly as before.
        let applied = ceiling_bounded_landing_delta_db(-20.0, -4.0, -14.0, -1.0);
        assert!(
            (applied - 3.0).abs() < 1.0e-4,
            "boost must cap at TP headroom (3 dB); got {applied}"
        );
    }

    /// Apply-in-place returns the same delta the math core would
    /// compute and ALSO mutates the sample buffer by the corresponding
    /// linear gain. Wraps the math core's contract plus the in-place
    /// step the render paths depend on.
    #[test]
    fn apply_with_measurements_mutates_samples_and_returns_delta() {
        // Construct a sample buffer at uniform amplitude 0.5. Apply
        // a -6 dB landing (measured -10 LUFS, target -16, plenty of
        // headroom — but delta is downward so headroom doesn't bind).
        let mut samples = vec![0.5_f32; 1024];
        let applied =
            apply_ceiling_bounded_landing_with_measurements(&mut samples, -10.0, -1.0, -16.0, -1.0);
        assert!(
            (applied - -6.0).abs() < 1.0e-6,
            "expected -6 dB applied delta; got {applied}"
        );
        // -6 dB linear ≈ 0.501. Each sample = 0.5 * 0.501 ≈ 0.2506.
        let expected_lin = 10.0_f32.powf(-6.0 / 20.0);
        let expected_sample = 0.5_f32 * expected_lin;
        for s in &samples {
            assert!(
                (s - expected_sample).abs() < 1.0e-5,
                "sample mutation should match the linear-gain of applied delta; \
                 got {s}, expected {expected_sample}"
            );
        }
    }

    /// Apply-in-place returns 0.0 and leaves samples untouched when
    /// the math core would no-op. Verifies the contract: callers can
    /// use `if applied != 0.0` to decide whether to mutate downstream
    /// state (e.g. the track-export receipt's tracked LUFS).
    #[test]
    fn apply_with_measurements_is_a_noop_when_delta_is_zero() {
        let mut samples = vec![0.5_f32; 32];
        let original = samples.clone();
        // Silent signal → math returns 0.
        let applied = apply_ceiling_bounded_landing_with_measurements(
            &mut samples,
            -80.0,
            -60.0,
            -14.0,
            -1.0,
        );
        assert_eq!(applied, 0.0);
        assert_eq!(samples, original, "samples must not be mutated on no-op");
    }

    /// B4: every production *_iso field now reads from `now_iso()` instead
    /// of the frozen `ISO_PLACEHOLDER`. Verifies the helper returns a
    /// real RFC 3339 timestamp near the current time, and explicitly
    /// confirms it does NOT return the placeholder. Test fixtures still
    /// use `ISO_PLACEHOLDER` for deterministic AnalysisResult construction.
    #[test]
    fn now_iso_returns_current_rfc3339_timestamp_not_placeholder() {
        let ts = now_iso();
        let parsed = chrono::DateTime::parse_from_rfc3339(&ts)
            .expect("now_iso must return a valid RFC 3339 timestamp");
        let now = chrono::Utc::now();
        let diff_seconds = (now - parsed.with_timezone(&chrono::Utc))
            .num_seconds()
            .abs();
        assert!(
            diff_seconds < 5,
            "now_iso timestamp ({ts}) should be near now (within 5 s), got {diff_seconds}s drift"
        );
        assert_ne!(
            ts, ISO_PLACEHOLDER,
            "now_iso must return a real current timestamp, not the frozen test placeholder"
        );
    }
}
