use crate::album_render::render_album_plan_impl;
use crate::analysis::{analyze_one, nudge_role_by_position, sanitize_lufs};
use crate::sample_rate::convert_interleaved;
use crate::types::*;
use crate::wav_writer::write_wav;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ebur128::{EbuR128, Mode};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub id: TrackId,
    pub path: String,
}

/// Phase 12.1 export progress — emitted on the "render:progress" Tauri event
/// channel during `render_track_master` / `render_track_preview` so the
/// frontend can render a real progress bar.
#[derive(Debug, Serialize, Clone)]
pub struct RenderProgress {
    pub track_id: TrackId,
    pub kind: RenderKind,
    pub fraction: f32,
}

#[tauri::command]
pub async fn analyze_tracks(tracks: Vec<AnalyzeRequest>) -> CommandResult<Vec<AnalysisResult>> {
    let total = tracks.len();
    let mut out = Vec::with_capacity(total);
    let mut failures: Vec<(TrackId, String)> = Vec::new();
    for (index, req) in tracks.into_iter().enumerate() {
        match analyze_one(req.id.clone(), Path::new(&req.path)) {
            Ok(mut result) => {
                nudge_role_by_position(&mut result, index, total);
                out.push(result);
            }
            Err(e) => {
                failures.push((req.id, e.to_string()));
            }
        }
    }
    // Partial-success policy: if every track failed, surface the first error
    // (otherwise the frontend has no signal at all). If at least one succeeded,
    // return the successes and log the failures — session restore and bulk
    // imports can keep working when one source file has moved.
    if out.is_empty() && !failures.is_empty() {
        let (_, msg) = &failures[0];
        return Err(CommandError::Other(format!(
            "analyze failed for all tracks: {msg}"
        )));
    }
    for (id, msg) in failures {
        eprintln!("analyze_tracks: skipping {} — {}", id.as_str(), msg);
    }
    Ok(out)
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
// The audio.rs live-preview helper uses the pure-math tier directly
// because it returns a gain scalar rather than mutating samples, and
// because its ebur128 setup is measured on a 8 s window (perf
// optimization that the offline render paths intentionally don't share).
// ============================================================================

/// Compute the LUFS-landing delta in dB given pre-measured loudness +
/// true peak. Downward delta applies in full (the limiter already
/// capped peaks at ceiling, so attenuating only moves them further
/// away). Upward delta is bounded by the residual true-peak headroom
/// below the user's ceiling. Returns 0.0 when:
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
    let headroom_db = (ceiling_dbtp - measured_true_peak_dbtp).max(0.0);
    let applied_delta_db = if delta_db < 0.0 {
        delta_db
    } else {
        delta_db.min(headroom_db)
    };
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

#[tauri::command]
pub async fn render_track_preview(
    track_id: TrackId,
    track_path: String,
    settings: MasteringSettings,
    app: tauri::AppHandle,
) -> CommandResult<RenderJob> {
    let out_dir = render_output_dir(&app, RenderKind::Preview)?;
    let track_id_for_progress = track_id.clone();
    let app_for_progress = app.clone();
    let on_progress = move |fraction: f32| {
        let _ = app_for_progress.emit(
            "render:progress",
            RenderProgress {
                track_id: track_id_for_progress.clone(),
                kind: RenderKind::Preview,
                fraction,
            },
        );
    };
    mastering_render_with_progress(
        track_id,
        Path::new(&track_path),
        &settings,
        &out_dir,
        RenderKind::Preview,
        Some(&on_progress),
        None,
    )
}

#[tauri::command]
pub async fn render_track_master(
    track_id: TrackId,
    track_path: String,
    settings: MasteringSettings,
    output_path: Option<String>,
    app: tauri::AppHandle,
) -> CommandResult<RenderJob> {
    let out_dir = render_output_dir(&app, RenderKind::Master)?;
    let explicit_output_path = output_path.as_deref().map(Path::new);
    let track_id_for_progress = track_id.clone();
    let app_for_progress = app.clone();
    let on_progress = move |fraction: f32| {
        let _ = app_for_progress.emit(
            "render:progress",
            RenderProgress {
                track_id: track_id_for_progress.clone(),
                kind: RenderKind::Master,
                fraction,
            },
        );
    };
    mastering_render_with_progress(
        track_id,
        Path::new(&track_path),
        &settings,
        &out_dir,
        RenderKind::Master,
        Some(&on_progress),
        explicit_output_path,
    )
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
// Sample-rate / channel-count mismatches between tracks fail with a
// clear error — resampling is deferred to a future phase.
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AlbumTrackRenderInput {
    pub track_id: TrackId,
    pub source_path: String,
    pub settings: MasteringSettings,
}

#[derive(Debug, Deserialize)]
pub struct AlbumPlanRenderRequest {
    pub plan: AlbumPlan,
    pub tracks: Vec<AlbumTrackRenderInput>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AlbumTrackRenderRecord {
    pub track_id: TrackId,
    pub position: u32,
    pub output_path: String,
    pub measured_lufs: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct AlbumRenderReport {
    pub album_wav_path: String,
    pub manifest_path: String,
    pub tracks: Vec<AlbumTrackRenderRecord>,
}

#[derive(Debug, Deserialize)]
pub struct PlanAlbumRequest {
    pub title: String,
    pub analyses: Vec<AnalysisResult>,
    pub durations: Vec<f64>,
    pub arc: AlbumArc,
    pub intensity: f32,
}

/// Phase B Step 4: thin Tauri wrapper around `album::build_album_plan`.
/// Lets the frontend pick (arc, intensity) and immediately receive the
/// per-track plan without duplicating the math in TypeScript.
#[tauri::command]
pub async fn plan_album(request: PlanAlbumRequest) -> CommandResult<AlbumPlan> {
    let refs: Vec<&AnalysisResult> = request.analyses.iter().collect();
    Ok(crate::album::build_album_plan(
        request.title,
        &refs,
        &request.durations,
        request.arc,
        request.intensity,
    ))
}

#[tauri::command]
pub async fn render_album_plan(
    request: AlbumPlanRenderRequest,
    output_dir: Option<String>,
    app: tauri::AppHandle,
) -> CommandResult<AlbumRenderReport> {
    let out_dir = match output_dir {
        Some(path) => explicit_output_dir(Path::new(&path))?,
        None => render_output_dir(&app, RenderKind::Album)?,
    };
    let app_for_progress = app.clone();
    let on_progress = move |fraction: f32| {
        let _ = app_for_progress.emit(
            "render:progress",
            RenderProgress {
                track_id: TrackId(String::new()),
                kind: RenderKind::Album,
                fraction,
            },
        );
    };
    render_album_plan_impl(&request, &out_dir, Some(&on_progress))
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

    let pcm = crate::decode::decode_full(source_path)?;
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
    if let Some(cb) = on_progress {
        cb(0.0);
    }
    while processed < total_samples {
        let end = (processed + chunk_samples).min(total_samples);
        chain.process_interleaved(&mut samples[processed..end], channels_max);
        processed = end;
        if let Some(cb) = on_progress {
            let fraction = processed as f32 / total_samples.max(1) as f32;
            cb(fraction.min(1.0));
        }
    }

    let rendered_sample_rate = render_settings.effective_sample_rate(pcm.sample_rate);
    if rendered_sample_rate != pcm.sample_rate {
        samples = convert_interleaved(
            &samples,
            pcm.sample_rate,
            rendered_sample_rate,
            pcm.channels,
        )?;
    }

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
    let measurements = RenderedMeasurements {
        lufs_integrated: measured_lufs,
        true_peak_dbtp: measured_true_peak_dbtp,
        dynamic_range_lu: if lra.is_finite() { lra } else { 0.0 },
        sample_rate: rendered_sample_rate,
        bit_depth,
    };
    let out_path = match output_path {
        Some(path) => explicit_output_path(path)?,
        None => unique_output_path(out_dir, source_path, &track_id, kind)?,
    };
    write_wav(
        &out_path,
        &samples,
        rendered_sample_rate,
        pcm.channels,
        bit_depth,
    )?;
    if let Some(cb) = on_progress {
        cb(1.0);
    }

    Ok(RenderJob {
        id: uuid::Uuid::new_v4().to_string(),
        kind,
        target_tracks: vec![track_id],
        status: JobStatus::Done,
        progress: 1.0,
        started_at_iso: now_iso(),
        output_paths: vec![out_path.to_string_lossy().to_string()],
        measurements: Some(measurements),
    })
}

fn explicit_output_path(path: &Path) -> CommandResult<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(CommandError::InvalidPath("empty output path".to_string()));
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
    Ok(path.to_path_buf())
}

fn explicit_output_dir(path: &Path) -> CommandResult<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(CommandError::InvalidPath(
            "empty output directory".to_string(),
        ));
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
    fn explicit_output_path_creates_parent_for_native_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let chosen = tmp.path().join("Masters").join("track master.wav");

        let out_path = explicit_output_path(&chosen).expect("explicit output path");

        assert_eq!(out_path, chosen);
        assert!(
            chosen.parent().expect("parent").is_dir(),
            "selected output parent should be created"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn explicit_output_path_creates_parent_for_windows_backslash_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let chosen = format!(
            "{}\\Masters\\track master.wav",
            tmp.path().to_string_lossy()
        );
        let chosen = PathBuf::from(chosen);

        let out_path = explicit_output_path(&chosen).expect("explicit output path");

        assert_eq!(out_path, chosen);
        assert!(
            out_path.parent().expect("parent").is_dir(),
            "Windows backslash output parent should be created"
        );
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
    fn ceiling_bounded_landing_skips_negligible_delta() {
        // measured -14.00005, target -14. Delta = -5e-5, abs < 1e-4.
        let applied = ceiling_bounded_landing_delta_db(-14.00005, -1.0, -14.0, -1.0);
        assert_eq!(
            applied, 0.0,
            "delta below the ±1e-4 dB noise threshold should produce zero; got {applied}"
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
