use crate::types::*;
use ebur128::{EbuR128, Mode};
use std::path::Path;

/// Phase 9.2: when a track's per-track role inference is weak, nudge first/
/// last positions toward Opener/Closer respectively. Strong-confidence roles
/// (e.g. an obvious Single at track 1) are left alone — the per-track signal
/// dominates when it's clear.
pub fn nudge_role_by_position(result: &mut AnalysisResult, index: usize, total: usize) {
    if total <= 1 {
        return;
    }
    let weak = matches!(
        result.role_confidence,
        Some(InferenceConfidence::Unsure) | None
    );
    let mid_default = matches!(result.role_confidence, Some(InferenceConfidence::Moderate))
        && matches!(result.inferred_role, Some(TrackRole::AlbumTrack));
    let eligible = weak || mid_default;
    if !eligible {
        return;
    }
    if index == 0 {
        result.inferred_role = Some(TrackRole::Opener);
        result.role_confidence = Some(InferenceConfidence::Moderate);
    } else if index == total - 1 {
        result.inferred_role = Some(TrackRole::Closer);
        result.role_confidence = Some(InferenceConfidence::Moderate);
    }
}

pub(crate) fn analyze_one(
    track_id: TrackId,
    path: &Path,
    deep: bool,
) -> CommandResult<AnalysisResult> {
    analyze_one_with_progress(track_id, path, deep, &|_, _| {})
}

/// `analyze_one` with stage callbacks at the REAL phase boundaries, so the
/// UI's analysis progress reports actual work instead of a paced timer
/// (mirrors the render path's real `render:progress`). `progress(frac,
/// label)` gets a 0..=1 fraction within THIS track; weights approximate
/// relative cost (decode dominates for long compressed sources).
pub(crate) fn analyze_one_with_progress(
    track_id: TrackId,
    path: &Path,
    deep: bool,
    progress: &dyn Fn(f32, &'static str),
) -> CommandResult<AnalysisResult> {
    if crate::files::has_parent_dir_component(path) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {}",
            path.display()
        )));
    }
    if !path.exists() {
        return Err(CommandError::Io(format!(
            "source file not found: {}",
            path.display()
        )));
    }

    progress(0.0, "Analyzing audio");
    let pcm = crate::decode::decode_full(path)?;
    if pcm.samples.is_empty() {
        return Err(CommandError::Decode("no samples decoded".to_string()));
    }

    progress(0.35, "Checking dynamics");
    let channels_u32 = u32::from(pcm.channels.max(1));
    let mut ebu = EbuR128::new(
        channels_u32,
        pcm.sample_rate,
        Mode::I | Mode::LRA | Mode::TRUE_PEAK,
    )
    .map_err(|e| CommandError::Other(format!("ebur128 init: {e}")))?;
    ebu.add_frames_f32(&pcm.samples)
        .map_err(|e| CommandError::Other(format!("ebur128 feed: {e}")))?;

    let lufs_integrated = sanitize_lufs(
        ebu.loudness_global()
            .map_err(|e| CommandError::Other(format!("ebur128 global: {e}")))? as f32,
    );
    let lra = ebu
        .loudness_range()
        .map_err(|e| CommandError::Other(format!("ebur128 lra: {e}")))? as f32;

    let mut peak_lin: f64 = 0.0;
    for ch in 0..channels_u32 {
        let tp = ebu
            .true_peak(ch)
            .map_err(|e| CommandError::Other(format!("ebur128 tp: {e}")))?;
        if tp > peak_lin {
            peak_lin = tp;
        }
    }
    let true_peak_dbtp = if peak_lin > 0.0 {
        (20.0 * peak_lin.log10()) as f32
    } else {
        -60.0
    };

    progress(0.55, "Evaluating stereo field");
    let stereo_width = compute_stereo_width(&pcm.samples, pcm.channels as usize);
    let spectral_balance = compute_spectral_balance(&pcm.samples, pcm.channels as usize);
    let transient_density = compute_transient_density(&pcm.samples, pcm.channels as usize);

    progress(0.65, "Reading tonal balance");
    // Phase A5: richer measurements. All Optional — they degrade
    // gracefully when the signal is too short or silent.
    let spectral_balance_6band =
        compute_spectral_balance_6band(&pcm.samples, pcm.sample_rate, pcm.channels as usize);
    let transient_flux =
        compute_transient_flux(&pcm.samples, pcm.sample_rate, pcm.channels as usize);
    let stereo_correlation = compute_stereo_correlation(&pcm.samples, pcm.channels as usize);
    let dynamic_range_p95_p10_db =
        compute_dynamic_range_p95_p10(&pcm.samples, pcm.sample_rate, pcm.channels as usize);
    let lufs_short_term_max_3s =
        compute_short_term_max_lufs(&pcm.samples, pcm.sample_rate, pcm.channels);
    let energy_density_score = compute_energy_density_score(
        lufs_integrated,
        spectral_balance_6band.as_ref(),
        dynamic_range_p95_p10_db,
        transient_flux,
    );

    progress(0.8, "Building mastering context");
    // Tier-2 Phase A: dual-resolution deep analysis (additive; never on the wire).
    // Task 9: callers can still pass `deep == false` for low-cost/mobile-lite
    // analysis, but desktop and the current iPhone native bridge both use the
    // deep-capable path when they need adaptive render/live parity.
    let deep_analysis = if deep {
        let bands31 =
            compute_spectral_balance_31band(&pcm.samples, pcm.sample_rate, pcm.channels as usize);
        let windows = crate::deep_analysis::scan_windows(
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels as usize,
        );
        match bands31 {
            Some(bands) if !windows.is_empty() => Some(std::sync::Arc::new(
                crate::deep_analysis::DeepAnalysis::from_parts(bands, windows),
            )),
            _ => None, // too short / silent -> DeepAnalysis absent (SourceProfile still derives)
        }
    } else {
        None
    };

    // Prefer the true 3 s short-term max from ebur128 Mode::S when
    // available; fall back to the prior estimate (integrated + LRA / 2)
    // for short signals where Mode::S doesn't have enough material.
    let short_term_max = lufs_short_term_max_3s.unwrap_or_else(|| {
        if lra.is_finite() {
            lufs_integrated + (lra * 0.5).max(0.0)
        } else {
            lufs_integrated
        }
    });

    let recommended_universal = MasteringSettings {
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
        delivery_profile: DeliveryProfile::default(),
        album: None,
        advanced: AdvancedSettings {
            lufs_offset_db: Some(-14.0 - lufs_integrated),
            ceiling_dbtp: Some(-1.0),
            bit_depth: Some(24),
            target_sample_rate: Some(pcm.sample_rate),
            ..Default::default()
        },
    };

    let duration_sec = if pcm.sample_rate > 0 && pcm.channels > 0 {
        (pcm.samples.len() as f64) / (pcm.channels.max(1) as f64 * pcm.sample_rate as f64)
    } else {
        0.0
    };
    // Phase A5: role inference prefers transient_flux when available
    // (spectral-flux is a stronger Single-track signal than ZCR), and
    // falls back to transient_density for backward compatibility.
    let role_transient_signal = transient_flux.unwrap_or(transient_density);
    let (role, role_conf) = infer_role(lufs_integrated, role_transient_signal, duration_sec);
    let (character, character_conf) = infer_character(&spectral_balance, transient_density);

    progress(1.0, "Building mastering context");
    Ok(AnalysisResult {
        track_id,
        lufs_integrated,
        lufs_short_term_max: short_term_max,
        true_peak_dbtp,
        dynamic_range_lu: if lra.is_finite() { lra } else { 0.0 },
        spectral_balance,
        transient_density,
        stereo_width,
        recommended_universal,
        measured_at_iso: now_iso(),
        inferred_role: Some(role),
        role_confidence: Some(role_conf),
        inferred_character: Some(character),
        character_confidence: Some(character_conf),
        spectral_balance_6band,
        transient_flux,
        stereo_correlation,
        dynamic_range_p95_p10_db,
        lufs_short_term_max_3s,
        energy_density_score,
        deep_analysis,
    })
}

fn infer_role(
    lufs: f32,
    transient_density: f32,
    duration_sec: f64,
) -> (TrackRole, InferenceConfidence) {
    // Interlude: short and quiet/sparse.
    if duration_sec > 0.0 && duration_sec < 90.0 && transient_density < 0.4 {
        return (TrackRole::Interlude, InferenceConfidence::Moderate);
    }
    // Single / banger: loud and dense.
    if lufs.is_finite() && lufs > -10.0 && transient_density > 0.6 {
        return (TrackRole::Single, InferenceConfidence::Strong);
    }
    // Ballad: quiet and sparse.
    if lufs.is_finite() && lufs < -16.0 && transient_density < 0.4 {
        return (TrackRole::Ballad, InferenceConfidence::Moderate);
    }
    // Default fallback.
    (TrackRole::AlbumTrack, InferenceConfidence::Unsure)
}

fn infer_character(
    spectral: &SpectralBalance,
    transient_density: f32,
) -> (TrackCharacter, InferenceConfidence) {
    if spectral.high > 0.45 {
        return (TrackCharacter::Bright, InferenceConfidence::Strong);
    }
    if spectral.high < 0.15 {
        return (TrackCharacter::Dark, InferenceConfidence::Moderate);
    }
    if transient_density > 0.65 {
        return (TrackCharacter::Dense, InferenceConfidence::Moderate);
    }
    if transient_density < 0.25 {
        return (TrackCharacter::Sparse, InferenceConfidence::Moderate);
    }
    (TrackCharacter::Balanced, InferenceConfidence::Unsure)
}

pub(crate) fn sanitize_lufs(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        -70.0
    }
}

fn compute_stereo_width(samples: &[f32], channels: usize) -> f32 {
    if channels < 2 {
        return 0.0;
    }
    let mut mid_sq = 0.0_f64;
    let mut side_sq = 0.0_f64;
    for frame in samples.chunks(channels) {
        let l = f64::from(*frame.first().unwrap_or(&0.0));
        let r = f64::from(*frame.get(1).unwrap_or(&0.0));
        let m = (l + r) * 0.5;
        let s = (l - r) * 0.5;
        mid_sq += m * m;
        side_sq += s * s;
    }
    let total = mid_sq + side_sq;
    if total > 0.0 {
        (side_sq / total) as f32
    } else {
        0.0
    }
}

pub(crate) fn compute_spectral_balance(samples: &[f32], channels: usize) -> SpectralBalance {
    if samples.is_empty() || channels == 0 {
        return SpectralBalance {
            low: 0.33,
            mid: 0.34,
            high: 0.33,
        };
    }
    // Simple band split via first-order RC filters. Phase 11b can replace with
    // Linkwitz-Riley crossovers or FFT for sharper bands.
    let mut low_lp_state = 0.0_f64;
    let mut high_lp_state = 0.0_f64;
    let mut low_sq = 0.0_f64;
    let mut mid_sq = 0.0_f64;
    let mut high_sq = 0.0_f64;

    // Assume 44.1 kHz reference; the bands are approximate either way.
    let low_alpha = 0.015; // ~100 Hz first-order LP at 44.1k
    let high_alpha = 0.45; // ~3 kHz first-order LP boundary for mid/high split

    for frame in samples.chunks(channels) {
        let mut mono = 0.0_f64;
        for &s in frame.iter() {
            mono += f64::from(s);
        }
        mono /= channels as f64;

        low_lp_state += low_alpha * (mono - low_lp_state);
        high_lp_state += high_alpha * (mono - high_lp_state);

        let low = low_lp_state;
        let mid = high_lp_state - low_lp_state;
        let high = mono - high_lp_state;

        low_sq += low * low;
        mid_sq += mid * mid;
        high_sq += high * high;
    }

    let total = low_sq + mid_sq + high_sq;
    if total > 0.0 {
        SpectralBalance {
            low: (low_sq / total) as f32,
            mid: (mid_sq / total) as f32,
            high: (high_sq / total) as f32,
        }
    } else {
        SpectralBalance {
            low: 0.33,
            mid: 0.34,
            high: 0.33,
        }
    }
}

// ============================================================================
// Phase A5: richer pre-mastering analysis. Implementations ported from
// Codex's `src/album_mastering_studio/analysis.py`. Originally these fed only
// role / character inference and album-arc planning. As of the Tier-1 adaptive
// guardrails, `spectral_balance_6band` (plus dynamic_range_p95_p10 / LRA /
// stereo_correlation) ALSO drives the adaptive EQ / density / width trims on the
// Track-Master audition and export path via `SourceProfile::from_analysis` — so
// their accuracy now affects rendered quality, not just metadata.
// ============================================================================

/// IEC 61260 nominal one-third-octave centers, 25 Hz … 20 kHz (30 nominal
/// centers + 1 padding slot).
/// Index 29 = 20 kHz, index 30 = 0.0 (padding to keep a fixed [f32; 31]).
pub(crate) const THIRD_OCTAVE_CENTERS: [f32; 31] = [
    25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0, 500.0,
    630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0,
    10000.0, 12500.0, 16000.0, 20000.0, 0.0,
];

/// Map a bin frequency to its one-third-octave band index (None if outside the
/// 25 Hz…~22 kHz coverage or in a padding slot). Deterministic.
/// Note: the nominal (rounded) centers leave small gaps/overlaps at band edges,
/// so an occasional out-of-band bin is dropped via `continue` — acceptable
/// because the tonal curve is re-normalized over captured energy (same way the
/// 6-band pass already drops out-of-range bins).
pub(crate) fn third_octave_band(freq: f32) -> Option<usize> {
    // A one-third-octave band spans 2^(1/3), so its edges sit at center ÷/× the
    // half-bandwidth ratio 2^(1/6) ≈ 1.1225 (= sqrt of the 2^(1/3) span).
    const HALF_STEP: f32 = 1.122_462_f32; // 2^(1/6)
    for (i, &c) in THIRD_OCTAVE_CENTERS.iter().enumerate() {
        if c <= 0.0 {
            continue; // padding slot
        }
        let lo = c / HALF_STEP;
        let hi = c * HALF_STEP;
        if freq >= lo && freq < hi {
            return Some(i);
        }
    }
    None
}

/// 6-band spectral balance via Hann-windowed FFT. Returns `None` if the
/// signal is too short for a meaningful FFT (< 1024 frames after
/// power-of-two truncation) or has no energy.
// NOTE: byte-exact/golden-pinned (see spectral_balance_6band_is_byte_exact_golden).
// The 31-band sibling is a deliberate copy, not a shared abstraction — keep them
// separate.
pub(crate) fn compute_spectral_balance_6band(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
) -> Option<crate::types::SpectralBalance6> {
    if channels == 0 || samples.is_empty() || sample_rate == 0 {
        return None;
    }
    let total_frames = samples.len() / channels;
    // Welch-style WHOLE-TRACK average. The largest power-of-two window <= the
    // track length (hard-capped at `1 << 18` ~= 5.5 s @48k for FFT cost and
    // low-frequency resolution) is slid across the ENTIRE track at 50% overlap,
    // and per-band power is accumulated over every window. This keeps the tonal
    // read representative of the whole track — matching the whole-track DR / LRA /
    // correlation measures — instead of only the leading ~5.5 s, so a bright (or
    // dark) intro can no longer bias the adaptive guardrail trims for the whole
    // song. A track no longer than one window collapses to a single window,
    // identical to the prior leading-window read (keeps stationary fixtures pinned).
    let mut fft_size = 1_usize;
    while fft_size * 2 <= total_frames && fft_size < 1 << 18 {
        fft_size *= 2;
    }
    if fft_size < 1024 {
        return None;
    }

    use rustfft::num_complex::Complex;
    use rustfft::FftPlanner;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let bins = fft_size / 2;
    let bin_hz = sample_rate as f32 / fft_size as f32;
    // Edge frequencies for the 6 bands (sub / low / low_mid / mid /
    // presence / air). Top edge clamped to min(Nyquist, 16 kHz).
    let top = (sample_rate as f32 / 2.0).min(16_000.0);
    let edges = [20.0, 80.0, 250.0, 800.0, 2500.0, 6500.0, top];
    let two_pi = 2.0 * std::f32::consts::PI;
    let hop = (fft_size / 2).max(1);
    let mut bands = [0.0_f64; 6];
    let mut buf: Vec<Complex<f32>> = Vec::with_capacity(fft_size);
    let mut start = 0_usize;
    while start + fft_size <= total_frames {
        buf.clear();
        for i in 0..fft_size {
            let mut mono = 0.0_f32;
            let frame_start = (start + i) * channels;
            for c in 0..channels {
                mono += samples[frame_start + c];
            }
            mono /= channels as f32;
            // Hann window — reduces spectral leakage.
            let w = 0.5 * (1.0 - (two_pi * i as f32 / (fft_size as f32 - 1.0)).cos());
            buf.push(Complex {
                re: mono * w,
                im: 0.0,
            });
        }
        fft.process(&mut buf);
        for (bin, c) in buf.iter().copied().enumerate().take(bins).skip(1) {
            let freq = bin as f32 * bin_hz;
            let idx = if freq >= edges[0] && freq < edges[1] {
                0
            } else if freq >= edges[1] && freq < edges[2] {
                1
            } else if freq >= edges[2] && freq < edges[3] {
                2
            } else if freq >= edges[3] && freq < edges[4] {
                3
            } else if freq >= edges[4] && freq < edges[5] {
                4
            } else if freq >= edges[5] && freq < edges[6] {
                5
            } else {
                continue;
            };
            bands[idx] += (c.re as f64) * (c.re as f64) + (c.im as f64) * (c.im as f64);
        }
        start += hop;
    }
    let total: f64 = bands.iter().sum();
    if total <= 1.0e-12 {
        return None;
    }
    Some(crate::types::SpectralBalance6 {
        sub: (bands[0] / total) as f32,
        low: (bands[1] / total) as f32,
        low_mid: (bands[2] / total) as f32,
        mid: (bands[3] / total) as f32,
        presence: (bands[4] / total) as f32,
        air: (bands[5] / total) as f32,
    })
}

/// 31-band one-third-octave tonal curve. Mirrors `compute_spectral_balance_6band`'s
/// Welch sliding-window FFT pass verbatim, but accumulates per-bin power into
/// one-third-octave bands (via `third_octave_band`) and normalizes to a fixed
/// `[f32; 31]` summing to ~1. This is the finer-resolution whole-track tonal curve
/// for `DeepAnalysis` (Phase A long-pass; backend-internal — never serialized).
// Deliberately a standalone copy of the 6-band Welch loop — do NOT factor the two
// together: compute_spectral_balance_6band is byte-exact/golden-pinned and shared
// extraction would risk silent byte-drift.
pub(crate) fn compute_spectral_balance_31band(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
) -> Option<[f32; 31]> {
    if channels == 0 || samples.is_empty() || sample_rate == 0 {
        return None;
    }
    let total_frames = samples.len() / channels;
    let mut fft_size = 1_usize;
    while fft_size * 2 <= total_frames && fft_size < 1 << 18 {
        fft_size *= 2;
    }
    if fft_size < 1024 {
        return None;
    }

    use rustfft::num_complex::Complex;
    use rustfft::FftPlanner;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let bins = fft_size / 2;
    let bin_hz = sample_rate as f32 / fft_size as f32;
    let two_pi = 2.0 * std::f32::consts::PI;
    let hop = (fft_size / 2).max(1);
    let mut bands = [0.0_f64; 31];
    let mut buf: Vec<Complex<f32>> = Vec::with_capacity(fft_size);
    let mut start = 0_usize;
    while start + fft_size <= total_frames {
        buf.clear();
        for i in 0..fft_size {
            let mut mono = 0.0_f32;
            let frame_start = (start + i) * channels;
            for c in 0..channels {
                mono += samples[frame_start + c];
            }
            mono /= channels as f32;
            // Hann window — reduces spectral leakage.
            let w = 0.5 * (1.0 - (two_pi * i as f32 / (fft_size as f32 - 1.0)).cos());
            buf.push(Complex {
                re: mono * w,
                im: 0.0,
            });
        }
        fft.process(&mut buf);
        for (bin, c) in buf.iter().copied().enumerate().take(bins).skip(1) {
            let freq = bin as f32 * bin_hz;
            let Some(idx) = third_octave_band(freq) else {
                continue;
            };
            bands[idx] += (c.re as f64) * (c.re as f64) + (c.im as f64) * (c.im as f64);
        }
        start += hop;
    }
    let total: f64 = bands.iter().sum();
    if total <= 1.0e-12 {
        return None;
    }
    let mut out = [0.0_f32; 31];
    for (o, b) in out.iter_mut().zip(bands.iter()) {
        *o = (*b / total) as f32;
    }
    Some(out)
}

/// Pearson correlation between L and R channels. `None` for mono.
fn compute_stereo_correlation(samples: &[f32], channels: usize) -> Option<f32> {
    if channels < 2 || samples.is_empty() {
        return None;
    }
    let n = samples.len() / channels;
    if n < 16 {
        return None;
    }
    // Two-pass for numerical stability.
    let mut sum_l = 0.0_f64;
    let mut sum_r = 0.0_f64;
    for frame in samples.chunks_exact(channels) {
        sum_l += frame[0] as f64;
        sum_r += frame[1] as f64;
    }
    let inv_n = 1.0 / n as f64;
    let mean_l = sum_l * inv_n;
    let mean_r = sum_r * inv_n;
    let mut cov = 0.0_f64;
    let mut var_l = 0.0_f64;
    let mut var_r = 0.0_f64;
    for frame in samples.chunks_exact(channels) {
        let dl = frame[0] as f64 - mean_l;
        let dr = frame[1] as f64 - mean_r;
        cov += dl * dr;
        var_l += dl * dl;
        var_r += dr * dr;
    }
    let denom = (var_l * var_r).sqrt();
    if denom > 1.0e-12 {
        Some((cov / denom).clamp(-1.0, 1.0) as f32)
    } else {
        None
    }
}

/// Dynamic range as P95 minus P10 of RMS-block dB values. 100 ms windows
/// at 50 ms hop. Better "how dynamic does this feel" than crest factor.
pub(crate) fn compute_dynamic_range_p95_p10(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
) -> Option<f32> {
    if channels == 0 || samples.is_empty() || sample_rate == 0 {
        return None;
    }
    let frames_per_block = (sample_rate as f32 * 0.1) as usize;
    let frames_per_hop = (sample_rate as f32 * 0.05) as usize;
    if frames_per_hop == 0 {
        return None;
    }
    let window = frames_per_block * channels;
    let hop = frames_per_hop * channels;
    if samples.len() < window {
        return None;
    }
    let mut rms_db: Vec<f32> = Vec::with_capacity(samples.len() / hop);
    let mut pos = 0;
    while pos + window <= samples.len() {
        let chunk = &samples[pos..pos + window];
        let mut sum_sq = 0.0_f64;
        for &s in chunk {
            sum_sq += (s as f64) * (s as f64);
        }
        let rms = (sum_sq / chunk.len() as f64).sqrt();
        if rms > 1.0e-9 {
            rms_db.push((20.0 * rms.log10()) as f32);
        }
        pos += hop;
    }
    if rms_db.len() < 4 {
        return None;
    }
    rms_db.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p10 = rms_db[(rms_db.len() * 10) / 100];
    let p95 = rms_db[((rms_db.len() * 95) / 100).min(rms_db.len() - 1)];
    Some(p95 - p10)
}

/// Maximum short-term LUFS via ebur128 Mode::S (3 s sliding window).
/// Feeds the signal in ~100 ms chunks and samples loudness_shortterm()
/// at each boundary, returning the max.
fn compute_short_term_max_lufs(samples: &[f32], sample_rate: u32, channels: u16) -> Option<f32> {
    if samples.is_empty() || channels == 0 {
        return None;
    }
    let mut ebu = ebur128::EbuR128::new(u32::from(channels), sample_rate, ebur128::Mode::S).ok()?;
    let chunk_frames = (sample_rate / 10).max(1) as usize;
    let chunk_samples = chunk_frames * channels as usize;
    let mut max_st = f32::NEG_INFINITY;
    let mut pos = 0;
    while pos < samples.len() {
        let end = (pos + chunk_samples).min(samples.len());
        ebu.add_frames_f32(&samples[pos..end]).ok()?;
        if let Ok(st) = ebu.loudness_shortterm() {
            let v = st as f32;
            if v.is_finite() && v > max_st {
                max_st = v;
            }
        }
        pos = end;
    }
    if max_st.is_finite() {
        Some(max_st)
    } else {
        None
    }
}

/// Spectral-flux transient density. 40 ms windows at 10 ms hop; the
/// positive (one-sided) flux of the RMS envelope is averaged and
/// normalized by mean RMS. Higher = more percussive.
pub(crate) fn compute_transient_flux(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
) -> Option<f32> {
    if channels == 0 || samples.is_empty() || sample_rate == 0 {
        return None;
    }
    let frames_per_window = (sample_rate as f32 * 0.04) as usize;
    let frames_per_hop = (sample_rate as f32 * 0.01) as usize;
    if frames_per_hop == 0 {
        return None;
    }
    let window = frames_per_window * channels;
    let hop = frames_per_hop * channels;
    if samples.len() < window {
        return None;
    }
    let mut rms: Vec<f64> = Vec::with_capacity(samples.len() / hop);
    let mut pos = 0;
    while pos + window <= samples.len() {
        let chunk = &samples[pos..pos + window];
        let mut sum_sq = 0.0_f64;
        for &s in chunk {
            sum_sq += (s as f64) * (s as f64);
        }
        rms.push((sum_sq / chunk.len() as f64).sqrt());
        pos += hop;
    }
    if rms.len() < 4 {
        return None;
    }
    let mean_rms: f64 = rms.iter().sum::<f64>() / rms.len() as f64;
    if mean_rms <= 1.0e-9 {
        return None;
    }
    let mut positive_flux = 0.0_f64;
    let mut count = 0_usize;
    for w in rms.windows(2) {
        let diff = w[1] - w[0];
        if diff > 0.0 {
            positive_flux += diff;
            count += 1;
        }
    }
    if count == 0 {
        return None;
    }
    Some(((positive_flux / count as f64) / mean_rms) as f32)
}

/// Composite "how hot does this mix feel" score in `[0, 1]`. Weighted
/// combination of loudness, brightness, density, transient flux per
/// Codex's analysis.py formula. Requires the 6-band spectral balance,
/// dynamic range, and transient flux — returns `None` if any input is
/// missing.
pub(crate) fn compute_energy_density_score(
    lufs_integrated: f32,
    spectral_6: Option<&crate::types::SpectralBalance6>,
    dynamic_range_p95_p10_db: Option<f32>,
    transient_flux: Option<f32>,
) -> Option<f32> {
    let spec = spectral_6?;
    let dr = dynamic_range_p95_p10_db?;
    let flux = transient_flux?;
    // Loudness term: -30 LUFS → 0, 0 LUFS → 1. Clamped.
    let loudness_norm = ((lufs_integrated + 30.0) / 30.0).clamp(0.0, 1.0);
    // Brightness term: presence + air share, scaled.
    let brightness_norm = ((spec.presence + spec.air) * 2.0).clamp(0.0, 1.0);
    // Density: low dynamic range → high density. 12 LU as the soft anchor.
    let density_norm = (1.0 - dr / 12.0).clamp(0.0, 1.0);
    // Transient flux already in roughly [0, 1] for typical content.
    let transient_norm = flux.clamp(0.0, 1.0);
    Some(
        0.44 * loudness_norm + 0.21 * brightness_norm + 0.23 * density_norm + 0.12 * transient_norm,
    )
}

fn compute_transient_density(samples: &[f32], channels: usize) -> f32 {
    if samples.is_empty() || channels == 0 {
        return 0.0;
    }
    // Crude zero-crossing-based proxy on the mono mix. Phase 11b can replace
    // with a real onset detector.
    let mut prev = 0.0_f32;
    let mut crossings = 0_u64;
    let mut frames = 0_u64;
    for frame in samples.chunks(channels) {
        let mut mono = 0.0;
        for &s in frame.iter() {
            mono += s;
        }
        mono /= channels as f32;
        if (mono >= 0.0) != (prev >= 0.0) && (mono - prev).abs() > 0.005 {
            crossings += 1;
        }
        prev = mono;
        frames += 1;
    }
    if frames == 0 {
        return 0.0;
    }
    // Normalize to a 0..1 range; ~4000 crossings/sec is dense (typical drums).
    let rate = crossings as f32 / frames as f32;
    (rate * 50.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spectral flux should read materially higher on a percussive
    /// signal (impulse train) than on a sustained one (continuous sine
    /// at the same average level). This is the core "is the new
    /// transient_flux actually better than the prior ZCR proxy" check.
    #[test]
    fn transient_flux_higher_on_percussive_than_sustained() {
        let sr = 48_000_u32;
        let n = (sr as f32 * 2.0) as usize;
        // Percussive: short bursts of high-amplitude sine at 5 Hz rate
        // (one click every 200 ms). Each burst is 10 ms of 1 kHz sine
        // followed by silence. The RMS envelope oscillates strongly.
        let burst_len = (sr as f32 * 0.01) as usize;
        let burst_period = (sr as f32 * 0.2) as usize;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let percussive: Vec<f32> = (0..n)
            .map(|i| {
                let phase = i % burst_period;
                if phase < burst_len {
                    0.5 * (omega * i as f32).sin()
                } else {
                    0.0
                }
            })
            .collect();
        // Sustained: continuous 1 kHz sine at lower amplitude so the
        // average level roughly matches the percussive signal. RMS
        // envelope is essentially flat.
        let sustained: Vec<f32> = (0..n).map(|i| 0.1 * (omega * i as f32).sin()).collect();

        let flux_p = compute_transient_flux(&percussive, sr, 1).expect("percussive flux");
        let flux_s = compute_transient_flux(&sustained, sr, 1).expect("sustained flux");
        assert!(
            flux_p > flux_s * 5.0,
            "percussive flux ({:.3}) should be >>5x sustained flux ({:.3})",
            flux_p,
            flux_s
        );
    }

    /// Stereo correlation: identical L/R reads ~+1.0; inverted L/R
    /// reads ~-1.0; decorrelated reads ~0.
    #[test]
    fn stereo_correlation_identical_inverted_decorrelated() {
        let n = 480_000;
        let omega = 2.0 * std::f32::consts::PI * 440.0 / 48_000.0;
        let identical: Vec<f32> = (0..n)
            .flat_map(|i| {
                let v = 0.3 * (omega * i as f32).sin();
                [v, v]
            })
            .collect();
        let inverted: Vec<f32> = (0..n)
            .flat_map(|i| {
                let v = 0.3 * (omega * i as f32).sin();
                [v, -v]
            })
            .collect();

        let c_id = compute_stereo_correlation(&identical, 2).expect("identical");
        let c_inv = compute_stereo_correlation(&inverted, 2).expect("inverted");
        assert!(
            (c_id - 1.0).abs() < 1.0e-3,
            "identical L/R should correlate ~+1.0; got {}",
            c_id
        );
        assert!(
            (c_inv + 1.0).abs() < 1.0e-3,
            "inverted L/R should correlate ~-1.0; got {}",
            c_inv
        );
    }

    /// Stereo correlation returns None for mono input.
    #[test]
    fn stereo_correlation_none_for_mono() {
        let mono: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        assert!(compute_stereo_correlation(&mono, 1).is_none());
    }

    /// 6-band spectral balance fractions sum to ~1.0.
    #[test]
    fn spectral_balance_6band_sums_to_unity() {
        let sr = 48_000_u32;
        let n = (sr as f32 * 1.5) as usize; // 1.5 s → 65536 frames after pow2 truncation
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let bal = compute_spectral_balance_6band(&samples, sr, 1).expect("balance");
        let total = bal.sub + bal.low + bal.low_mid + bal.mid + bal.presence + bal.air;
        assert!(
            (total - 1.0).abs() < 0.01,
            "6-band fractions should sum to ~1.0; got {} (sub={}, low={}, low_mid={}, mid={}, presence={}, air={})",
            total,
            bal.sub,
            bal.low,
            bal.low_mid,
            bal.mid,
            bal.presence,
            bal.air
        );
    }

    /// A 1 kHz pure tone should concentrate its energy in the `mid`
    /// band (800–2500 Hz) — sanity check that the band edges actually
    /// map correctly.
    #[test]
    fn spectral_balance_6band_1khz_concentrates_in_mid() {
        let sr = 48_000_u32;
        let n = (sr as f32 * 1.5) as usize;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let bal = compute_spectral_balance_6band(&samples, sr, 1).expect("balance");
        assert!(
            bal.mid > 0.5,
            "1 kHz sine should put majority of energy in mid band; got mid={}",
            bal.mid
        );
    }

    /// Regression (adversarial review F1, 2026-06-03): the 6-band tonal read must
    /// reflect the WHOLE track, not just the first window. A bright intro
    /// followed by a longer dark body must read as dark (the body dominates by
    /// energy), so the adaptive guardrails don't trim the high end based on an
    /// unrepresentative intro. The pre-fix single-leading-window FFT read only the
    /// first ~5.5 s and would have reported this track as bright.
    #[test]
    fn spectral_balance_6band_reflects_whole_track_not_just_intro() {
        let sr = 48_000_u32;
        // 7 s bright intro (10 kHz -> air band), then 17 s dark body (100 Hz ->
        // low band). The first window (~5.5 s) sees only the bright intro;
        // whole-track averaging is energy-dominated by the longer dark body.
        let intro_n = (sr as f32 * 7.0) as usize;
        let body_n = (sr as f32 * 17.0) as usize;
        let w_hi = 2.0 * std::f32::consts::PI * 10_000.0 / sr as f32;
        let w_lo = 2.0 * std::f32::consts::PI * 100.0 / sr as f32;
        let mut samples: Vec<f32> = Vec::with_capacity(intro_n + body_n);
        for i in 0..intro_n {
            samples.push(0.3 * (w_hi * i as f32).sin());
        }
        for i in 0..body_n {
            samples.push(0.3 * (w_lo * i as f32).sin());
        }
        let bal = compute_spectral_balance_6band(&samples, sr, 1).expect("balance");
        let bright = bal.presence + bal.air;
        let low = bal.sub + bal.low;
        assert!(
            low > bright,
            "whole-track read must be dark-dominated by the body; got low={low} bright={bright} \
             (a first-window read of the bright intro would report the opposite)"
        );
        assert!(
            low > 0.5,
            "the 17 s dark body should dominate the 7 s bright intro; got low={low}"
        );
    }

    fn spectral_balance_6band_snapshot_fixture() -> [f32; 6] {
        let sr = 48_000_u32;
        // Deterministic broadband fixture: summed sines exciting four of the six
        // bands (sub, low_mid, presence, air); `low`/`mid` stay near-zero so the
        // lock pins those too.
        let n = sr as usize * 2; // 2 s, mono
        let mut samples = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f32 / sr as f32;
            let s = 0.20 * (2.0 * std::f32::consts::PI * 60.0 * t).sin()
                + 0.15 * (2.0 * std::f32::consts::PI * 500.0 * t).sin()
                + 0.12 * (2.0 * std::f32::consts::PI * 3_000.0 * t).sin()
                + 0.08 * (2.0 * std::f32::consts::PI * 9_000.0 * t).sin();
            samples.push(s);
        }
        let bal = compute_spectral_balance_6band(&samples, sr, 1).expect("balance");
        [
            bal.sub,
            bal.low,
            bal.low_mid,
            bal.mid,
            bal.presence,
            bal.air,
        ]
    }

    fn write_f32le(path: &std::path::Path, samples: &[f32]) {
        let mut bytes = Vec::with_capacity(std::mem::size_of_val(samples));
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        std::fs::write(path, bytes).expect("write f32 diagnostic buffer");
    }

    #[test]
    #[ignore = "writes raw snapshot buffers for cross-platform CI comparison"]
    fn snapshot_diagnostics_write_analysis_buffers() {
        let out_dir = std::env::var("SNAPSHOT_DIAGNOSTIC_DIR")
            .expect("SNAPSHOT_DIAGNOSTIC_DIR must point at an artifact directory");
        let out_dir = std::path::PathBuf::from(out_dir).join("analysis");
        std::fs::create_dir_all(&out_dir).expect("create analysis diagnostic directory");

        let values = spectral_balance_6band_snapshot_fixture();
        write_f32le(&out_dir.join("spectral_balance_6band.f32le"), &values);
        std::fs::write(
            out_dir.join("manifest.json"),
            format!(
                "{{\"case\":\"spectral_balance_6band\",\"sample_count\":{},\"values\":{:?}}}\n",
                values.len(),
                values
            ),
        )
        .expect("write analysis diagnostic manifest");
    }

    /// One OS/arch-independent 6-band reference. The former per-OS split existed
    /// only because libm rounds these ~6e-8 differently on macOS-arm64; the
    /// comparison below uses a scale-aware tolerance so a single reference passes
    /// on every OS/arch (an Intel Mac no longer spuriously fails).
    fn reference_spectral_balance_6band() -> [f32; 6] {
        [
            0.48019287,
            3.2878075e-10,
            0.27010766,
            8.422937e-9,
            0.17286925,
            0.07683022,
        ]
    }

    /// Value-tolerance lock on the 6-band output (adversarial review must-fix #1).
    /// The other 6-band tests are relative (sums-to-unity / mid>0.5 / low>bright)
    /// and would NOT catch a value shift. This pins the 6 values within a tight
    /// tolerance — well above the ~6e-8 platform rounding, far below any
    /// structural drift — so it still forces an explicit decision if a future
    /// change rerolls the 6-band from the 31-band.
    #[test]
    fn spectral_balance_6band_matches_golden() {
        let observed = spectral_balance_6band_snapshot_fixture();
        let reference = reference_spectral_balance_6band();
        for (i, (a, b)) in observed.iter().zip(reference).enumerate() {
            let tol = 1.0e-6 + 1.0e-5 * b.abs();
            assert!(
                (a - b).abs() <= tol,
                "6-band index {i} drifted: {a} vs reference {b} (delta {}, tol {tol}); \
                 investigate DSP drift before regenerating",
                (a - b).abs()
            );
        }
    }

    #[test]
    fn spectral_balance_31band_sums_to_unity_and_rolls_up_near_6band() {
        let sr = 48_000_u32;
        let n = sr as usize * 2;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let bands = compute_spectral_balance_31band(&samples, sr, 1).expect("31");
        let total: f32 = bands.iter().sum();
        assert!((total - 1.0).abs() < 0.02, "sum={total}");
        // 1 kHz tone concentrates in the bands around 1 kHz (mid region).
        let mid_energy: f32 = (0..31)
            .filter(|&i| (800.0..2500.0).contains(&crate::deep_analysis::band_center_hz(i)))
            .map(|i| bands[i])
            .sum();
        assert!(mid_energy > 0.5, "mid_energy={mid_energy}");
    }

    /// Dynamic-range P95-P10 should be small for a sine at constant amplitude
    /// and large for a square-envelope amplitude-modulated signal.
    #[test]
    fn dynamic_range_p95_p10_responds_to_amplitude_swings() {
        let sr = 48_000_u32;
        let n = (sr as f32 * 2.0) as usize;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let flat: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        // Modulated: alternate 0.5 s loud / 0.5 s quiet (-30 dB).
        let half = sr as usize / 2;
        let mod_signal: Vec<f32> = (0..n)
            .map(|i| {
                let amp = if (i / half) % 2 == 0 { 0.3 } else { 0.01 };
                amp * (omega * i as f32).sin()
            })
            .collect();
        let dr_flat = compute_dynamic_range_p95_p10(&flat, sr, 1).expect("flat");
        let dr_mod = compute_dynamic_range_p95_p10(&mod_signal, sr, 1).expect("mod");
        assert!(
            dr_mod > dr_flat + 15.0,
            "modulated signal should have much wider P95-P10 spread; flat={} mod={}",
            dr_flat,
            dr_mod
        );
    }
}
