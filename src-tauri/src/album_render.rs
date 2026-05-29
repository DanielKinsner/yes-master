use crate::analysis::{
    compute_dynamic_range_p95_p10, compute_energy_density_score, compute_spectral_balance_6band,
    compute_transient_flux,
};
use crate::engine::{
    measure_and_apply_ceiling_bounded_landing, measure_integrated_lufs, AlbumPlanRenderRequest,
    AlbumRenderReport, AlbumTrackRenderInput, AlbumTrackRenderRecord,
};
use crate::types::*;
use crate::wav_writer::{wav_spec, write_samples_into_writer, write_wav};
use serde::Serialize;
use std::path::{Path, PathBuf};
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

/// Shadow a per-track `MasteringSettings` with the album plan's offsets:
///   * advanced.lufs_offset_db is REPLACED with
///     `effective_target_lufs() + arc_lufs_offset_db` so the per-track
///     render lands at the arc-modulated target.
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
    shadowed.advanced.lufs_offset_db = Some(base_target + entry.arc_lufs_offset_db);
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

    shadowed
}

pub fn render_album_plan_impl(
    request: &AlbumPlanRenderRequest,
    out_dir: &Path,
    on_progress: Option<&dyn Fn(f32)>,
) -> CommandResult<AlbumRenderReport> {
    if request.plan.tracks.is_empty() {
        return Err(CommandError::Other("AlbumPlan has no tracks".to_string()));
    }
    // Lookup: TrackId -> (source_path, settings).
    let settings_by_id: std::collections::HashMap<&str, &AlbumTrackRenderInput> = request
        .tracks
        .iter()
        .map(|t| (t.track_id.as_str(), t))
        .collect();

    let bit_depth = request
        .plan
        .tracks
        .first()
        .and_then(|t| settings_by_id.get(t.track_id.as_str()))
        .map(|input| input.settings.effective_bit_depth())
        .unwrap_or(24);

    std::fs::create_dir_all(out_dir).map_err(|e| CommandError::Io(e.to_string()))?;

    let total_tracks = request.plan.tracks.len();
    if let Some(cb) = on_progress {
        cb(0.0);
    }

    // Two passes:
    //   Pass 1 - decode + render each track into samples in memory, write
    //   the per-track WAV with NN-<title>.wav name, measure post-render
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
    let mut common_sr: u32 = 0;
    let mut common_channels: u16 = 0;

    for (i, entry) in request.plan.tracks.iter().enumerate() {
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
        let pcm = crate::decode::decode_full(path)?;
        if pcm.samples.is_empty() {
            return Err(CommandError::Decode(format!(
                "no samples decoded from {}",
                input.source_path
            )));
        }
        if i == 0 {
            common_sr = pcm.sample_rate;
            common_channels = pcm.channels.max(1);
        } else if pcm.sample_rate != common_sr {
            return Err(CommandError::Other(format!(
                "album sample-rate mismatch on {}: {} Hz vs album {} Hz (resampling not yet supported)",
                input.source_path, pcm.sample_rate, common_sr
            )));
        } else if pcm.channels != common_channels {
            return Err(CommandError::Other(format!(
                "album channel mismatch on {}: {} ch vs album {} ch",
                input.source_path, pcm.channels, common_channels
            )));
        }

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
            let lufs = measure_integrated_lufs(&pcm.samples, pcm.sample_rate, pcm.channels)
                .unwrap_or(-30.0);
            let spec6 = compute_spectral_balance_6band(
                &pcm.samples,
                pcm.sample_rate,
                pcm.channels as usize,
            );
            let dr =
                compute_dynamic_range_p95_p10(&pcm.samples, pcm.sample_rate, pcm.channels as usize);
            let tflux =
                compute_transient_flux(&pcm.samples, pcm.sample_rate, pcm.channels as usize);
            compute_energy_density_score(lufs, spec6.as_ref(), dr, tflux)
        };
        let energy_density = energy_density_score.unwrap_or(0.5);
        let shadowed = apply_album_shadow(
            &input.settings,
            entry,
            request.plan.intensity,
            curve_value,
            energy_density,
        );
        let mut shadowed = shadowed;
        shadowed.volume_match = false;
        let mut samples = pcm.samples;
        let channels_usize = pcm.channels.max(1) as usize;
        let mut chain = crate::dsp::MasteringChain::new(pcm.sample_rate, channels_usize, &shadowed);
        const CHUNK_FRAMES: usize = 4096;
        let chunk_samples = CHUNK_FRAMES * channels_usize;
        let track_total = samples.len();
        let mut processed = 0;
        while processed < track_total {
            let end = (processed + chunk_samples).min(track_total);
            chain.process_interleaved(&mut samples[processed..end], channels_usize);
            processed = end;
            if let Some(cb) = on_progress {
                let within_track = processed as f32 / track_total.max(1) as f32;
                let overall = (i as f32 + within_track) / total_tracks.max(1) as f32;
                cb(overall.min(1.0));
            }
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
            pcm.sample_rate,
            pcm.channels,
            &shadowed,
        )?;

        // Per-track WAV named NN-<sanitized_title>.wav.
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("track");
        let safe = sanitize_for_filename(stem);
        let per_track_name = format!("{:02}-{}.wav", entry.position, safe);
        let per_track_path = out_dir.join(&per_track_name);
        write_wav(
            &per_track_path,
            &samples,
            pcm.sample_rate,
            pcm.channels,
            bit_depth,
        )?;

        let measured_lufs = measure_integrated_lufs(&samples, pcm.sample_rate, pcm.channels)?;
        track_records.push(AlbumTrackRenderRecord {
            track_id: entry.track_id.clone(),
            position: entry.position,
            output_path: per_track_path.to_string_lossy().to_string(),
            measured_lufs,
        });
        rendered_samples.push(samples);
    }

    // Pass 2 - assemble the continuous album.wav, inserting silence
    // frames per TransitionSpec.
    let album_path = unique_album_path(out_dir)?;
    let spec = wav_spec(common_channels, common_sr, bit_depth)?;
    let mut album_writer =
        hound::WavWriter::create(&album_path, spec).map_err(|e| CommandError::Io(e.to_string()))?;
    for (i, samples) in rendered_samples.iter().enumerate() {
        write_samples_into_writer(&mut album_writer, samples, bit_depth)?;
        if i + 1 < rendered_samples.len() {
            // Transition slot between track i and track i+1.
            if let Some(t) = request.plan.transitions.get(i) {
                if matches!(t.kind, TransitionKind::Gap) {
                    let gap_seconds = t.duration_seconds.clamp(0.0, 5.0);
                    let gap_frames = (gap_seconds * common_sr as f32) as usize;
                    let gap_samples = gap_frames * common_channels as usize;
                    let zeros = vec![0.0_f32; gap_samples];
                    write_samples_into_writer(&mut album_writer, &zeros, bit_depth)?;
                }
            }
        }
    }
    album_writer
        .finalize()
        .map_err(|e| CommandError::Io(e.to_string()))?;

    // Manifest.
    let manifest_path = out_dir.join("manifest.json");
    let manifest = AlbumManifest {
        plan: &request.plan,
        rendered_at_iso: now_iso(),
        sample_rate: common_sr,
        channels: common_channels,
        bit_depth,
        album_wav_path: &album_path.to_string_lossy(),
        tracks: &track_records,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| CommandError::Other(format!("manifest serde: {e}")))?;
    std::fs::write(&manifest_path, manifest_json).map_err(|e| CommandError::Io(e.to_string()))?;

    if let Some(cb) = on_progress {
        cb(1.0);
    }

    Ok(AlbumRenderReport {
        album_wav_path: album_path.to_string_lossy().to_string(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        tracks: track_records,
    })
}

fn unique_album_path(out_dir: &Path) -> CommandResult<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let candidate = out_dir.join(format!("album_continuous_{ts}.wav"));
    if !candidate.exists() {
        return Ok(candidate);
    }
    for n in 1..1000 {
        let alt = out_dir.join(format!("album_continuous_{ts}_{n}.wav"));
        if !alt.exists() {
            return Ok(alt);
        }
    }
    Err(CommandError::Io(
        "could not generate unique album path".to_string(),
    ))
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

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
}
