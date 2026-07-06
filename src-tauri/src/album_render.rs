use crate::analysis::{
    compute_dynamic_range_p95_p10, compute_energy_density_score, compute_spectral_balance_6band,
    compute_transient_flux, sanitize_lufs,
};
use crate::engine::{
    comparable_existing_or_parent_path, measure_and_apply_ceiling_bounded_landing,
    measure_integrated_lufs, render_cancelled, AlbumPlanRenderRequest, AlbumRenderReport,
    AlbumTrackRenderInput, AlbumTrackRenderRecord,
};
use crate::sample_rate::convert_interleaved;
use crate::types::*;
use crate::wav_writer::{
    finalize_never_overwrite, unique_tmp_path, wav_spec, write_samples_into_writer, write_wav,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};

/// Resolve the album-wide delivery sample rate. An explicit request wins;
/// otherwise Auto = the highest source rate among the tracks (quality-safe:
/// never forces a needless downsample, deterministic for mixed sources).
/// Empty source list falls back to 48 kHz.
fn resolve_album_sample_rate(requested: Option<u32>, source_rates: &[u32]) -> u32 {
    if let Some(rate) = requested {
        return rate;
    }
    source_rates.iter().copied().max().unwrap_or(48_000)
}

/// Resolve the album-wide output channel count. Auto keeps all-mono albums
/// mono, but mixed mono/stereo albums render stereo so stereo sources are not
/// downmixed and mono sources can be safely duplicated. The product is
/// stereo-only (BS.1770 loudness uses stereo channel weights), so the result is
/// capped at 2: a source with more than two channels is folded to stereo (at
/// decode since 2026-07-06; `convert_channel_count` keeps the same fold for
/// direct-PCM callers) rather than producing a pseudo-surround album whose
/// loudness would land under multichannel weights and not compare to other
/// masters.
fn resolve_album_channels(source_channels: &[u16]) -> u16 {
    source_channels
        .iter()
        .copied()
        .max()
        .unwrap_or(2)
        .clamp(1, 2)
}

// The canonical stereo fold (`downmix_frame_to_stereo`) lives in
// `crate::decode` since 2026-07-06: it runs at decode for every imported
// source, so above-stereo PCM normally never reaches this module.
// `convert_channel_count` keeps its above-stereo branch (same helper) for
// direct-PCM callers.
use crate::decode::downmix_frame_to_stereo;

fn convert_channel_count(
    samples: Vec<f32>,
    source_channels: u16,
    target_channels: u16,
) -> CommandResult<Vec<f32>> {
    let source_channels = source_channels.max(1) as usize;
    let target_channels = target_channels.max(1) as usize;
    if source_channels == target_channels {
        return Ok(samples);
    }
    if samples.len() % source_channels != 0 {
        return Err(CommandError::Other(format!(
            "sample count {} is not divisible by source channel count {}",
            samples.len(),
            source_channels
        )));
    }

    let frames = samples.len() / source_channels;
    let mut converted = Vec::with_capacity(frames * target_channels);
    for frame in samples.chunks_exact(source_channels) {
        if target_channels > source_channels {
            if source_channels == 1 {
                converted.extend(std::iter::repeat(frame[0]).take(target_channels));
            } else {
                converted.extend_from_slice(frame);
                let fill = *frame.last().unwrap_or(&0.0);
                converted.extend(std::iter::repeat(fill).take(target_channels - source_channels));
            }
        } else if target_channels == 2 && source_channels > 2 {
            converted.extend_from_slice(&downmix_frame_to_stereo(frame));
        } else {
            for out_ch in 0..target_channels {
                let start = out_ch * source_channels / target_channels;
                let end = ((out_ch + 1) * source_channels / target_channels)
                    .max(start + 1)
                    .min(source_channels);
                let sum: f32 = frame[start..end].iter().copied().sum();
                converted.push(sum / (end - start) as f32);
            }
        }
    }
    Ok(converted)
}

#[derive(Debug, Serialize)]
struct AlbumManifest<'a> {
    plan: &'a AlbumPlan,
    rendered_at_iso: String,
    sample_rate: u32,
    channels: u16,
    bit_depth: u16,
    album_wav_path: &'a str,
    tracks: &'a [AlbumTrackRenderRecord],
}

/// Sanitize a string into a safe file-name component. Replaces any
/// character outside `[A-Za-z0-9._-]` with `_`. Empty input becomes
/// `"untitled"`.
fn sanitize_for_filename(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed: String = cleaned.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed
    }
}

fn reject_album_output_source_collision(
    candidate: &Path,
    source_paths: &[PathBuf],
) -> CommandResult<()> {
    let comparable_candidate = comparable_existing_or_parent_path(candidate);
    if source_paths
        .iter()
        .any(|source| comparable_candidate == comparable_existing_or_parent_path(source))
    {
        return Err(CommandError::InvalidPath(
            "album output path would overwrite a source file".to_string(),
        ));
    }
    Ok(())
}

fn unique_child_path_avoiding_sources(
    out_dir: &Path,
    base_name: &str,
    source_paths: &[PathBuf],
) -> CommandResult<PathBuf> {
    let candidate = out_dir.join(base_name);
    reject_album_output_source_collision(&candidate, source_paths)?;
    if !candidate.exists() {
        return Ok(candidate);
    }

    let base = Path::new(base_name);
    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("render");
    let extension = base.extension().and_then(|s| s.to_str());

    for n in 2..1000 {
        let name = match extension {
            Some(ext) if !ext.is_empty() => format!("{stem}-{n}.{ext}"),
            _ => format!("{stem}-{n}"),
        };
        let alt = out_dir.join(name);
        reject_album_output_source_collision(&alt, source_paths)?;
        if !alt.exists() {
            return Ok(alt);
        }
    }

    Err(CommandError::Io(format!(
        "could not generate unique path for {base_name}"
    )))
}

/// Shadow a per-track `MasteringSettings` with the album plan's offsets:
///   * advanced.lufs_offset_db is REPLACED with
///     `effective_target_lufs() + arc_lufs_offset_db` so the per-track
///     render lands at the arc-modulated target.
///   * delivery intent is preserved as Custom-equivalent advanced fields so
///     the arc target is not shadowed by a non-Custom profile.
///   * intensity is multiplied by intensity_scale (clamped to [0, 1.5]).
fn apply_album_shadow(
    settings: &MasteringSettings,
    entry: &AlbumTrackEntry,
    album_intensity: f32,
    curve_value: f32,
    energy_density: f32,
) -> MasteringSettings {
    let mut shadowed = settings.clone();
    let base_target = shadowed.effective_target_lufs().unwrap_or(-14.0);
    let ceiling_dbtp = shadowed.effective_ceiling_dbtp();
    let bit_depth = shadowed.effective_bit_depth();
    let target_sample_rate = shadowed.requested_delivery_sample_rate();

    shadowed.delivery_profile = DeliveryProfile::Custom;
    shadowed.advanced.lufs_offset_db = Some(base_target + entry.arc_lufs_offset_db);
    shadowed.advanced.ceiling_dbtp = Some(ceiling_dbtp);
    shadowed.advanced.bit_depth = Some(bit_depth);
    shadowed.advanced.target_sample_rate = target_sample_rate;
    shadowed.intensity = (shadowed.intensity * entry.intensity_scale).clamp(0.0, 1.5);

    // Phase B+ Step 7: apply the per-character mastering bias on top of
    // the user's per-track settings. EQ band offsets add to the existing
    // user EQ; width / warmth coerce None to a neutral baseline (1.0 /
    // 0.0) before the offset lands; intensity gets a final bias add then
    // re-clamp.
    let bias = crate::album::mastering_bias_for(
        entry.album_character,
        energy_density,
        curve_value,
        album_intensity,
    );
    shadowed.eq_low_db += bias.low_end_db;
    shadowed.eq_low_mid_db += bias.low_mid_db;
    shadowed.eq_mid_db += bias.presence_db;
    shadowed.eq_high_db += bias.air_db;
    if bias.width_offset.abs() > 1.0e-4 {
        let base_width = shadowed.advanced.width.unwrap_or(1.0);
        shadowed.advanced.width = Some((base_width + bias.width_offset).clamp(0.0, 2.0));
    }
    if bias.warmth_offset.abs() > 1.0e-4 {
        let base_warmth = shadowed.advanced.warmth.unwrap_or(0.0);
        shadowed.advanced.warmth = Some((base_warmth + bias.warmth_offset).clamp(0.0, 1.0));
    }
    shadowed.intensity = (shadowed.intensity + bias.intensity_offset).clamp(0.0, 1.5);

    // Album Master is non-adaptive (owner decision): strip adaptive internals so
    // a stale / hand-built / API payload can't make album renders adaptive (B1/B9).
    shadowed.advanced.source_profile = None;
    shadowed.advanced.source_confidence = None;
    shadowed.advanced.compression_guards = None;

    shadowed
}

/// Owner decision 2026-07-03 D9 — "full sound exemption" for an overridden
/// track: its own settings and loudness target render unmodified. No arc
/// offset, no character bias, no album intensity scale. Album Master's
/// non-adaptive rule still applies (adaptive internals stripped, mirroring
/// `apply_album_shadow`), and the album delivery format is enforced outside
/// the settings (resample / channel fold / bit depth), so the record stays
/// one coherent deliverable.
fn override_exempt_settings(settings: &MasteringSettings) -> MasteringSettings {
    let mut own = settings.clone();
    own.advanced.source_profile = None;
    own.advanced.source_confidence = None;
    own.advanced.compression_guards = None;
    own
}

#[cfg(test)]
mod adaptive_scope_tests {
    use super::*;

    fn settings_with_profile(profile: Option<crate::types::SourceProfile>) -> MasteringSettings {
        MasteringSettings {
            preset: crate::types::Preset::Universal,
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
            delivery_profile: crate::types::DeliveryProfile::Custom,
            album: None,
            advanced: crate::types::AdvancedSettings {
                source_profile: profile,
                ..Default::default()
            },
        }
    }

    fn album_entry() -> AlbumTrackEntry {
        AlbumTrackEntry {
            track_id: crate::types::TrackId("t".to_string()),
            position: 1,
            role: crate::types::TrackRole::AlbumTrack,
            role_locked: false,
            arc_lufs_offset_db: 0.0,
            intensity_scale: 1.0,
            album_character: None,
        }
    }

    #[test]
    fn album_shadow_strips_source_profile_so_album_stays_unadapted() {
        // Owner decision: Album Master is non-adaptive. apply_album_shadow STRIPS
        // any source_profile, so even a stale / hand-built payload carrying one
        // renders flat (B1/B9). Track Master is the adaptive surface.
        let none = apply_album_shadow(&settings_with_profile(None), &album_entry(), 0.5, 0.0, 0.5);
        assert!(none.advanced.source_profile.is_none());

        let bright = crate::types::SourceProfile {
            spectral_6: crate::types::SpectralBalance6 {
                sub: 0.1,
                low: 0.2,
                low_mid: 0.2,
                mid: 0.2,
                presence: 0.15,
                air: 0.15,
            },
            dynamic_range_p95_p10_db: 8.0,
            dynamic_range_lu: 8.0,
            stereo_correlation: Some(0.5),
            stereo_width: 1.0,
        };
        let stripped = apply_album_shadow(
            &settings_with_profile(Some(bright)),
            &album_entry(),
            0.5,
            0.0,
            0.5,
        );
        assert!(
            stripped.advanced.source_profile.is_none(),
            "album shadow must strip an incoming profile"
        );
    }

    #[test]
    fn album_shadow_applies_arc_target_under_profile_delivery_intent() {
        let mut settings = settings_with_profile(None);
        settings.delivery_profile = crate::types::DeliveryProfile::StreamingUniversal;
        let mut entry = album_entry();
        entry.arc_lufs_offset_db = 1.5;

        let shadowed = apply_album_shadow(&settings, &entry, 0.5, 0.0, 0.5);

        assert_eq!(shadowed.effective_target_lufs(), Some(-12.5));
        assert!((shadowed.effective_ceiling_dbtp() - -1.0).abs() < 1.0e-6);
        assert_eq!(shadowed.effective_bit_depth(), 24);
        assert_eq!(shadowed.effective_sample_rate(96_000), 48_000);
    }

    #[test]
    fn album_shadow_strips_backend_internal_adaptive_fields() {
        let bright = crate::types::SourceProfile {
            spectral_6: crate::types::SpectralBalance6 {
                sub: 0.1,
                low: 0.2,
                low_mid: 0.2,
                mid: 0.2,
                presence: 0.15,
                air: 0.15,
            },
            dynamic_range_p95_p10_db: 8.0,
            dynamic_range_lu: 8.0,
            stereo_correlation: Some(0.5),
            stereo_width: 1.0,
        };
        let mut settings = settings_with_profile(Some(bright));
        settings.advanced.source_confidence = Some(crate::confidence::Confidence::full());
        settings.advanced.compression_guards =
            Some(crate::guardrails::CompressionGuards::identity());

        let stripped = apply_album_shadow(&settings, &album_entry(), 0.5, 0.0, 0.5);

        assert!(stripped.advanced.source_profile.is_none());
        assert!(
            stripped.advanced.source_confidence.is_none(),
            "album shadow must strip backend-only source confidence"
        );
        assert!(
            stripped.advanced.compression_guards.is_none(),
            "album shadow must strip backend-only compression guards"
        );
    }

    #[test]
    fn override_exempt_settings_preserves_user_intent_and_strips_adaptive() {
        let bright = crate::types::SourceProfile {
            spectral_6: crate::types::SpectralBalance6 {
                sub: 0.1,
                low: 0.2,
                low_mid: 0.2,
                mid: 0.2,
                presence: 0.15,
                air: 0.15,
            },
            dynamic_range_p95_p10_db: 8.0,
            dynamic_range_lu: 8.0,
            stereo_correlation: Some(0.5),
            stereo_width: 1.0,
        };
        let mut settings = settings_with_profile(Some(bright));
        settings.delivery_profile = crate::types::DeliveryProfile::StreamingUniversal;
        settings.eq_high_db = 2.5;
        settings.intensity = 0.9;
        settings.advanced.source_confidence = Some(crate::confidence::Confidence::full());
        settings.advanced.compression_guards =
            Some(crate::guardrails::CompressionGuards::identity());

        let own = override_exempt_settings(&settings);

        // D9: the user's sound intent is untouched — own delivery target,
        // own EQ, own intensity; no arc replacement, no bias, no scale.
        assert!(matches!(
            own.delivery_profile,
            crate::types::DeliveryProfile::StreamingUniversal
        ));
        assert_eq!(own.effective_target_lufs(), Some(-14.0));
        assert!((own.eq_high_db - 2.5).abs() < 1.0e-6);
        assert!((own.intensity - 0.9).abs() < 1.0e-6);
        // Album Master stays non-adaptive even for overridden tracks.
        assert!(own.advanced.source_profile.is_none());
        assert!(own.advanced.source_confidence.is_none());
        assert!(own.advanced.compression_guards.is_none());
    }

    #[test]
    fn album_source_path_validation_rejects_traversal() {
        let inputs = vec![crate::engine::AlbumTrackRenderInput {
            track_id: crate::types::TrackId("t".to_string()),
            source_path: "../escape/track.wav".to_string(),
            settings: settings_with_profile(None),
            override_album: false,
        }];
        let err = crate::engine::validate_album_source_paths(&inputs)
            .expect_err("traversal source path must be rejected");
        assert!(
            matches!(err, CommandError::InvalidPath(ref m) if m.contains("path traversal not allowed")),
            "unexpected error: {err:?}"
        );
    }
}

pub fn render_album_plan_impl(
    request: &AlbumPlanRenderRequest,
    out_dir: &Path,
    on_progress: Option<&dyn Fn(f32)>,
) -> CommandResult<AlbumRenderReport> {
    render_album_plan_impl_with_cancel(request, out_dir, on_progress, None, None)
}

pub fn render_album_plan_impl_with_cancel(
    request: &AlbumPlanRenderRequest,
    out_dir: &Path,
    on_progress: Option<&dyn Fn(f32)>,
    cancel_flag: Option<&AtomicBool>,
    job_id: Option<&str>,
) -> CommandResult<AlbumRenderReport> {
    let job_id = job_id
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if request.plan.tracks.is_empty() {
        return Err(CommandError::Other("AlbumPlan has no tracks".to_string()));
    }
    crate::engine::validate_album_source_paths(&request.tracks)?;
    // Lookup: TrackId -> (source_path, settings).
    let settings_by_id: std::collections::HashMap<&str, &AlbumTrackRenderInput> = request
        .tracks
        .iter()
        .map(|t| (t.track_id.as_str(), t))
        .collect();
    let source_paths: Vec<PathBuf> = request
        .tracks
        .iter()
        .map(|input| PathBuf::from(&input.source_path))
        .collect();

    // Resolve the single album delivery format BEFORE processing any track.
    // Sample rate: explicit request wins, else Auto = highest source rate
    // (probed cheaply from headers). Bit depth: explicit request wins, else
    // the historical first-track effective_bit_depth().
    let mut source_rates: Vec<u32> = Vec::with_capacity(request.plan.tracks.len());
    let mut source_channels: Vec<u16> = Vec::with_capacity(request.plan.tracks.len());
    for t in &request.plan.tracks {
        if let Some(input) = settings_by_id.get(t.track_id.as_str()) {
            let path = Path::new(&input.source_path);
            // Check existence here, BEFORE the format probe: the probe
            // otherwise surfaces a raw OS error ("os error 2") that never
            // names which of the album's sources is missing.
            if !path.exists() {
                return Err(CommandError::Io(format!(
                    "source not found: {}",
                    input.source_path
                )));
            }
            let probed = crate::decode::probe_audio_format(path)?;
            source_rates.push(probed.sample_rate);
            source_channels.push(probed.channels);
        }
    }
    let album_sample_rate =
        resolve_album_sample_rate(request.plan.delivery_sample_rate, &source_rates);
    let album_channels = resolve_album_channels(&source_channels);
    let bit_depth = request.plan.delivery_bit_depth.unwrap_or_else(|| {
        request
            .plan
            .tracks
            .first()
            .and_then(|t| settings_by_id.get(t.track_id.as_str()))
            .map(|input| input.settings.effective_bit_depth())
            .unwrap_or(24)
    });

    std::fs::create_dir_all(out_dir).map_err(|e| CommandError::Io(e.to_string()))?;

    let total_tracks = request.plan.tracks.len();
    if let Some(cb) = on_progress {
        cb(0.0);
    }
    if render_cancelled(cancel_flag) {
        return Ok(cancelled_album_report(
            job_id,
            request,
            album_sample_rate,
            source_rates,
            bit_depth,
            album_channels,
            source_channels,
        ));
    }

    // Two passes:
    //   Pass 1 - decode + render each track into samples in memory, write
    //   the per-track WAV named NN-<source-file-stem>.wav (the album title
    //   is NOT used in filenames today — owner smoke F13; naming scheme is
    //   an open owner decision in docs/OPEN_THREADS_AND_DECISIONS.md),
    //   measure post-render
    //   LUFS, and remember the rendered samples + transition spec for the
    //   continuous writer in pass 2. Memory cost is the full album in f32;
    //   for a typical 60-min album at 48k stereo that's ~1.3 GB which is
    //   acceptable on modern desktop. Future optimization can stream
    //   directly without staging.
    //
    //   Pass 2 - open the album writer, stream each track's samples in,
    //   inject Gap silence frames per TransitionSpec, finalize.
    let mut rendered_samples: Vec<Vec<f32>> = Vec::with_capacity(total_tracks);
    let mut track_records: Vec<AlbumTrackRenderRecord> = Vec::with_capacity(total_tracks);

    // Per-stage wall-time aggregates across all tracks (F9/F11, mirrors the
    // track-render path): one diagnostics line at the end names the slow
    // stage so a report alone can answer "why did this album export take 15
    // minutes". Progress emits are throttled to ≥1% overall steps below —
    // the chain runs far faster than realtime, so an emit per 4096-frame
    // chunk flooded IPC with events nobody can read.
    let t_render_start = std::time::Instant::now();
    let mut decode_ms: u128 = 0;
    let mut chain_ms: u128 = 0;
    let mut src_land_ms: u128 = 0;
    let mut track_write_ms: u128 = 0;
    let mut last_emitted_overall = 0.0_f32;

    // Pass-1 hostile-input hygiene (2026-07-03 A5b): if any track fails
    // mid-album, best-effort remove the per-track WAVs already written so a
    // failed export can't leave misleading partial output behind. (Pass 2
    // already cleans its own tmp file on failure.)
    let mut written_paths: Vec<PathBuf> = Vec::with_capacity(total_tracks);
    let pass1_result = (|| -> CommandResult<()> {
        for (i, entry) in request.plan.tracks.iter().enumerate() {
            if render_cancelled(cancel_flag) {
                return Err(CommandError::Other("album render cancelled".to_string()));
            }
            let input = settings_by_id
                .get(entry.track_id.as_str())
                .copied()
                .ok_or_else(|| {
                    CommandError::Other(format!(
                        "AlbumPlan references track_id {} but no settings/path was provided",
                        entry.track_id.as_str()
                    ))
                })?;
            let path = Path::new(&input.source_path);
            if !path.exists() {
                return Err(CommandError::Io(format!(
                    "source not found: {}",
                    input.source_path
                )));
            }
            let t_decode = std::time::Instant::now();
            let pcm = crate::decode::decode_full(path)?;
            decode_ms += t_decode.elapsed().as_millis();
            if pcm.samples.is_empty() {
                return Err(CommandError::Decode(format!(
                    "no samples decoded from {}",
                    input.source_path
                )));
            }
            let source_channel_count = pcm.channels.max(1);
            // Decode folds above-stereo sources to stereo, so pcm.channels is
            // the PROCESSING layout; the render record reports the file's
            // real channel count from the header probe (source_channels vec
            // above) so per-track receipts stay honest about fold-downs.
            let file_channel_count = source_channels
                .get(i)
                .copied()
                .unwrap_or(source_channel_count)
                .max(1);

            let shadowed = if input.override_album {
                // D9 full sound exemption: skip the arc/bias shadowing (and the
                // energy measurement that only feeds the bias) entirely.
                override_exempt_settings(&input.settings)
            } else {
                // Per-track curve value for the per-character mastering bias.
                // For Preset arcs we resample the 6-point curve to actual track
                // count; for Custom arcs we use a neutral 0.5 (no curve-driven
                // air-band swing in the bias).
                let curve_value = match &request.plan.arc {
                    AlbumArc::Preset { preset } => {
                        let curve = crate::album::resample_arc_curve(preset.curve(), total_tracks);
                        curve.get(i).copied().unwrap_or(0.5)
                    }
                    AlbumArc::Custom { .. } => 0.5,
                };
                // B1: compute per-track energy density from the decoded PCM so the
                // album-arc character-bias presence-band energy-gate uses the same
                // signal as the analysis path. Pre-B1 this was hardcoded to 0.5,
                // dead-coding the gate in the album EXPORT path while
                // `analyze_tracks` computed real values.
                //
                // Four measurements: integrated LUFS, 6-band spectral balance,
                // dynamic range (p95-p10), transient flux. Falls back to 0.5
                // (the prior literal, treated as "neutral") if any input is
                // unavailable - matches `compute_energy_density_score`'s contract.
                let energy_density_score = {
                    let lufs = measure_integrated_lufs(
                        &pcm.samples,
                        pcm.sample_rate,
                        source_channel_count,
                    )
                    .unwrap_or(-30.0);
                    let spec6 = compute_spectral_balance_6band(
                        &pcm.samples,
                        pcm.sample_rate,
                        source_channel_count as usize,
                    );
                    let dr = compute_dynamic_range_p95_p10(
                        &pcm.samples,
                        pcm.sample_rate,
                        source_channel_count as usize,
                    );
                    let tflux = compute_transient_flux(
                        &pcm.samples,
                        pcm.sample_rate,
                        source_channel_count as usize,
                    );
                    compute_energy_density_score(lufs, spec6.as_ref(), dr, tflux)
                };
                let energy_density = energy_density_score.unwrap_or(0.5);
                apply_album_shadow(
                    &input.settings,
                    entry,
                    request.plan.intensity,
                    curve_value,
                    energy_density,
                )
            };
            let mut shadowed = shadowed;
            shadowed.volume_match = false;
            let mut samples = pcm.samples;
            let channels_usize = source_channel_count as usize;
            let mut chain =
                crate::dsp::MasteringChain::new(pcm.sample_rate, channels_usize, &shadowed);
            const CHUNK_FRAMES: usize = 4096;
            let chunk_samples = CHUNK_FRAMES * channels_usize;
            let track_total = samples.len();
            let mut processed = 0;
            let t_chain = std::time::Instant::now();
            while processed < track_total {
                if render_cancelled(cancel_flag) {
                    return Err(CommandError::Other("album render cancelled".to_string()));
                }
                let end = (processed + chunk_samples).min(track_total);
                chain.process_interleaved(&mut samples[processed..end], channels_usize);
                processed = end;
                if let Some(cb) = on_progress {
                    let within_track = processed as f32 / track_total.max(1) as f32;
                    let overall = ((i as f32 + within_track) / total_tracks.max(1) as f32).min(1.0);
                    // ≥1% throttle (F9) — the final 1.0 is emitted once after
                    // pass 2, so no terminal event is lost to the threshold.
                    if overall - last_emitted_overall >= 0.01 {
                        last_emitted_overall = overall;
                        cb(overall);
                    }
                }
            }
            // RS-09 fix (2026-07-03): drain the limiter's lookahead so each
            // album track keeps its final ~3 ms and stays sample-aligned —
            // and gap/butt-splice transitions in the continuous album.wav
            // sit exactly where the plan says. Before SRC/measure, like the
            // track-master path.
            chain.flush_render_tail(&mut samples, channels_usize);
            chain_ms += t_chain.elapsed().as_millis();
            let t_src_land = std::time::Instant::now();

            // Resample this track from its source rate to the album delivery
            // rate. Ordering mirrors Track Master: chain -> SRC -> measure ->
            // land. `convert_interleaved` would copy even on a match, so guard
            // it to avoid a needless full-buffer clone on already-matching tracks.
            if pcm.sample_rate != album_sample_rate {
                samples = convert_interleaved(
                    &samples,
                    pcm.sample_rate,
                    album_sample_rate,
                    source_channel_count,
                )?;
            }

            let mut rendered_channel_count = source_channel_count;
            if rendered_channel_count != album_channels {
                samples = convert_channel_count(samples, rendered_channel_count, album_channels)?;
                rendered_channel_count = album_channels;
            }

            // Per-track ceiling-bounded LUFS landing on the album-plan
            // path. `shadowed.effective_target_lufs()` is the arc-modulated
            // target (per-track LUFS offset baked into the shadow), so each
            // track lands at its arc-curve-determined target rather than
            // the raw album-intent target - preserving the album-arc story.
            // The B6 ceiling-bounded math is shared with the track-export
            // and album-simple paths via the helper.
            measure_and_apply_ceiling_bounded_landing(
                &mut samples,
                album_sample_rate,
                rendered_channel_count,
                &shadowed,
            )?;
            src_land_ms += t_src_land.elapsed().as_millis();

            // Per-track WAV named NN-<sanitized SOURCE-FILE stem>.wav. There
            // is no per-track title field anywhere in the album model, and
            // `plan.title` (the album title) reaches only manifest.json —
            // owner smoke F13; scheme decision tracked in
            // docs/OPEN_THREADS_AND_DECISIONS.md.
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("track");
            let safe = sanitize_for_filename(stem);
            let per_track_name = format!("{:02}-{}.wav", entry.position, safe);
            let per_track_path =
                unique_child_path_avoiding_sources(out_dir, &per_track_name, &source_paths)?;
            // write_wav diverts to a `__{n}` sibling if the chosen path
            // gained a file mid-render; track the ACTUAL path for both
            // the record and the on-error cleanup sweep.
            let t_write = std::time::Instant::now();
            let per_track_path = write_wav(
                &per_track_path,
                &samples,
                album_sample_rate,
                rendered_channel_count,
                bit_depth,
            )?;
            track_write_ms += t_write.elapsed().as_millis();
            written_paths.push(per_track_path.clone());
            if render_cancelled(cancel_flag) {
                return Err(CommandError::Other("album render cancelled".to_string()));
            }

            let measured_lufs = sanitize_lufs(measure_integrated_lufs(
                &samples,
                album_sample_rate,
                rendered_channel_count,
            )?);
            track_records.push(AlbumTrackRenderRecord {
                track_id: entry.track_id.clone(),
                position: entry.position,
                output_path: per_track_path.to_string_lossy().to_string(),
                measured_lufs,
                source_sample_rate: pcm.sample_rate,
                rendered_sample_rate: album_sample_rate,
                source_channels: file_channel_count,
                rendered_channels: rendered_channel_count,
                override_album: input.override_album,
            });
            rendered_samples.push(samples);
        }
        Ok(())
    })();
    if let Err(err) = pass1_result {
        for p in &written_paths {
            let _ = std::fs::remove_file(p);
        }
        if render_cancelled(cancel_flag) {
            return Ok(cancelled_album_report(
                job_id,
                request,
                album_sample_rate,
                source_rates,
                bit_depth,
                album_channels,
                source_channels,
            ));
        }
        return Err(err);
    }

    // Pass 2 - assemble the continuous album.wav, inserting silence
    // frames per TransitionSpec. Everything from here through the
    // manifest write shares one error exit that sweeps `written_paths`
    // — the no-partial-output promise (A5b) covers pass-2 and manifest
    // failures too, not just pass-1 (D5 runtime-abuse review finding):
    // a failed album export must not leave the per-track WAVs behind.
    if render_cancelled(cancel_flag) {
        for p in &written_paths {
            let _ = std::fs::remove_file(p);
        }
        return Ok(cancelled_album_report(
            job_id,
            request,
            album_sample_rate,
            source_rates,
            bit_depth,
            album_channels,
            source_channels,
        ));
    }
    let t_pass2 = std::time::Instant::now();
    let pass2_result = (|| -> CommandResult<(PathBuf, PathBuf)> {
        let album_path = unique_album_path(out_dir, &source_paths)?;
        let spec = wav_spec(album_channels, album_sample_rate, bit_depth)?;
        let album_tmp_path = unique_tmp_path(&album_path)?;
        let album_write_result = (|| -> CommandResult<()> {
            // 1 MiB BufWriter (hound defaults to 8 KiB): the continuous album
            // file is the longest write in the app — at 8 KiB it flushes once
            // per ~11 ms of audio, which is exactly where a sync-monitored or
            // slow destination bleeds time (owner smoke F9/F11). Byte output
            // is unchanged; only flush cadence differs.
            let album_file = std::fs::File::create(&album_tmp_path)
                .map_err(|e| CommandError::Io(e.to_string()))?;
            let album_buf = std::io::BufWriter::with_capacity(1 << 20, album_file);
            let mut album_writer = hound::WavWriter::new(album_buf, spec)
                .map_err(|e| CommandError::Io(e.to_string()))?;
            for (i, samples) in rendered_samples.iter().enumerate() {
                if render_cancelled(cancel_flag) {
                    return Err(CommandError::Other("album render cancelled".to_string()));
                }
                write_samples_into_writer(&mut album_writer, samples, bit_depth)?;
                if i + 1 < rendered_samples.len() {
                    // Transition slot between track i and track i+1.
                    if let Some(t) = request.plan.transitions.get(i) {
                        if matches!(t.kind, TransitionKind::Gap) {
                            let gap_seconds = t.duration_seconds.clamp(0.0, 5.0);
                            let gap_frames = (gap_seconds * album_sample_rate as f32) as usize;
                            let gap_samples = gap_frames * album_channels as usize;
                            let zeros = vec![0.0_f32; gap_samples];
                            write_samples_into_writer(&mut album_writer, &zeros, bit_depth)?;
                        }
                    }
                }
            }
            album_writer
                .finalize()
                .map_err(|e| CommandError::Io(e.to_string()))?;
            Ok(())
        })();
        if let Err(err) = album_write_result {
            let _ = std::fs::remove_file(&album_tmp_path);
            return Err(err);
        }
        if render_cancelled(cancel_flag) {
            let _ = std::fs::remove_file(&album_tmp_path);
            return Err(CommandError::Other("album render cancelled".to_string()));
        }
        let album_path = match finalize_never_overwrite(&album_tmp_path, &album_path) {
            Ok(p) => p,
            Err(err) => {
                let _ = std::fs::remove_file(&album_tmp_path);
                return Err(err);
            }
        };
        // From here the continuous file exists on disk — track it so a
        // manifest failure sweeps it along with the per-track WAVs.
        written_paths.push(album_path.clone());

        let manifest_path =
            unique_child_path_avoiding_sources(out_dir, "manifest.json", &source_paths)?;
        let manifest = AlbumManifest {
            plan: &request.plan,
            rendered_at_iso: now_iso(),
            sample_rate: album_sample_rate,
            channels: album_channels,
            bit_depth,
            album_wav_path: &album_path.to_string_lossy(),
            tracks: &track_records,
        };
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| CommandError::Other(format!("manifest serde: {e}")))?;
        written_paths.push(manifest_path.clone());
        std::fs::write(&manifest_path, manifest_json)
            .map_err(|e| CommandError::Io(e.to_string()))?;
        if render_cancelled(cancel_flag) {
            return Err(CommandError::Other("album render cancelled".to_string()));
        }
        Ok((album_path, manifest_path))
    })();
    let (album_path, manifest_path) = match pass2_result {
        Ok(paths) => paths,
        Err(err) => {
            for p in &written_paths {
                let _ = std::fs::remove_file(p);
            }
            if render_cancelled(cancel_flag) {
                return Ok(cancelled_album_report(
                    job_id,
                    request,
                    album_sample_rate,
                    source_rates,
                    bit_depth,
                    album_channels,
                    source_channels,
                ));
            }
            return Err(err);
        }
    };

    if let Some(cb) = on_progress {
        cb(1.0);
    }

    // One line per album render, mirroring the track path (F9/F11): names
    // the slow stage AND the destination so a diagnostics report alone can
    // answer "why did this album export take 15 minutes". Stages not listed
    // (post-write measurement, planning) are the total-minus-sum remainder.
    crate::diagnostics::info(format!(
        "render album job {job_id}: {total_tracks} tracks, decode {decode_ms}ms, \
         chain {chain_ms}ms, src+land {src_land_ms}ms, track writes {track_write_ms}ms, \
         continuous+manifest {}ms, total {}ms -> {}",
        t_pass2.elapsed().as_millis(),
        t_render_start.elapsed().as_millis(),
        out_dir.display()
    ));

    Ok(AlbumRenderReport {
        job_id,
        status: JobStatus::Done,
        album_wav_path: album_path.to_string_lossy().to_string(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        requested_sample_rate: request.plan.delivery_sample_rate,
        rendered_sample_rate: album_sample_rate,
        source_sample_rates: source_rates,
        bit_depth,
        rendered_channels: album_channels,
        source_channels,
        tracks: track_records,
    })
}

fn cancelled_album_report(
    job_id: String,
    request: &AlbumPlanRenderRequest,
    album_sample_rate: u32,
    source_rates: Vec<u32>,
    bit_depth: u16,
    album_channels: u16,
    source_channels: Vec<u16>,
) -> AlbumRenderReport {
    AlbumRenderReport {
        job_id,
        status: JobStatus::Cancelled,
        album_wav_path: String::new(),
        manifest_path: String::new(),
        requested_sample_rate: request.plan.delivery_sample_rate,
        rendered_sample_rate: album_sample_rate,
        source_sample_rates: source_rates,
        bit_depth,
        rendered_channels: album_channels,
        source_channels,
        tracks: Vec::new(),
    }
}

fn unique_album_path(out_dir: &Path, source_paths: &[PathBuf]) -> CommandResult<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    unique_child_path_avoiding_sources(out_dir, &format!("album_continuous_{ts}.wav"), source_paths)
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use crate::decode::SURROUND_DOWNMIX_GAIN;

    #[test]
    fn explicit_request_overrides_sources() {
        assert_eq!(
            resolve_album_sample_rate(Some(44_100), &[48_000, 96_000]),
            44_100
        );
    }

    #[test]
    fn auto_picks_highest_source_rate() {
        assert_eq!(
            resolve_album_sample_rate(None, &[44_100, 48_000, 44_100]),
            48_000
        );
    }

    #[test]
    fn auto_with_no_sources_falls_back_to_48k() {
        assert_eq!(resolve_album_sample_rate(None, &[]), 48_000);
    }

    #[test]
    fn all_mono_sources_render_mono() {
        assert_eq!(resolve_album_channels(&[1, 1]), 1);
    }

    #[test]
    fn mixed_mono_stereo_sources_render_stereo() {
        assert_eq!(resolve_album_channels(&[1, 2]), 2);
    }

    #[test]
    fn mono_to_stereo_duplicates_samples() {
        let converted = convert_channel_count(vec![0.25, -0.5], 1, 2).expect("convert");
        assert_eq!(converted, vec![0.25, 0.25, -0.5, -0.5]);
    }

    #[test]
    fn more_than_stereo_sources_cap_at_stereo() {
        // The product is stereo-only: a >2ch source must not push the album to
        // a pseudo-surround channel count whose loudness lands under
        // multichannel BS.1770 weights. It folds to stereo instead.
        assert_eq!(resolve_album_channels(&[6, 2]), 2);
        assert_eq!(resolve_album_channels(&[1, 6]), 2);
    }

    #[test]
    fn surround_source_downmixes_to_stereo() {
        // Common 5.1 frame order: L, R, C, LFE, Ls, Rs. The LFE channel is
        // excluded from the stereo fold and the front L/R pair stays anchored.
        let converted =
            convert_channel_count(vec![1.0, 10.0, 3.0, 100.0, 5.0, 7.0], 6, 2).expect("convert");
        let weight = 1.0 + (SURROUND_DOWNMIX_GAIN * 2.0);
        let expected_left =
            (1.0 + (3.0 * SURROUND_DOWNMIX_GAIN) + (5.0 * SURROUND_DOWNMIX_GAIN)) / weight;
        let expected_right =
            (10.0 + (3.0 * SURROUND_DOWNMIX_GAIN) + (7.0 * SURROUND_DOWNMIX_GAIN)) / weight;
        assert!((converted[0] - expected_left).abs() < 0.000_001);
        assert!((converted[1] - expected_right).abs() < 0.000_001);
    }

    #[test]
    fn quad_source_preserves_front_pair_while_folding_rears() {
        let converted = convert_channel_count(vec![1.0, 10.0, 5.0, 7.0], 4, 2).expect("convert");
        let weight = 1.0 + SURROUND_DOWNMIX_GAIN;
        let expected_left = (1.0 + (5.0 * SURROUND_DOWNMIX_GAIN)) / weight;
        let expected_right = (10.0 + (7.0 * SURROUND_DOWNMIX_GAIN)) / weight;
        assert!((converted[0] - expected_left).abs() < 0.000_001);
        assert!((converted[1] - expected_right).abs() < 0.000_001);
    }
}

#[cfg(test)]
mod unique_path_tests {
    use super::*;
    use std::fs;

    // AGENTS.md non-negotiable: exports never overwrite a prior render. The
    // continuous-album writer has its own uniqueness helper, so pin that a
    // second allocation in the same directory dodges an existing file and
    // both renders survive on disk.
    #[test]
    fn unique_album_path_never_overwrites_a_prior_render() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path();

        let first = unique_album_path(out, &[]).expect("first path");
        fs::write(&first, b"first").expect("write first");

        let second = unique_album_path(out, &[]).expect("second path");
        assert_ne!(first, second, "second render must not reuse the first path");
        fs::write(&second, b"second").expect("write second");

        // Nothing was overwritten: both files exist and keep their contents.
        assert!(first.exists());
        assert!(second.exists());
        assert_eq!(fs::read(&first).expect("read first"), b"first");

        // Naming contract (timestamp value intentionally not asserted).
        for path in [&first, &second] {
            let name = path.file_name().unwrap().to_string_lossy();
            assert!(
                name.starts_with("album_continuous_"),
                "unexpected prefix: {name}"
            );
            assert!(name.ends_with(".wav"), "unexpected suffix: {name}");
        }
    }

    #[test]
    fn album_child_path_rejects_source_path_collision_instead_of_suffixing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path();
        let source = out.join("album_continuous_source.wav");
        fs::write(&source, b"source").expect("write source");

        let err = unique_child_path_avoiding_sources(
            out,
            "album_continuous_source.wav",
            std::slice::from_ref(&source),
        )
        .expect_err("album child path must reject a source collision");

        assert!(
            matches!(err, CommandError::InvalidPath(ref message) if message == "album output path would overwrite a source file"),
            "unexpected error: {err:?}"
        );
        assert_eq!(
            fs::read(&source).expect("read source"),
            b"source",
            "source bytes must remain untouched"
        );
    }
}
