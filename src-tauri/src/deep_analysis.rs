//! Tier-2 Phase A — backend-internal deep analysis. NEVER serialized to TS.
//! Additive: produced at analysis time, cached beside SourceProfile, consumed
//! (later) by Phase B. See docs/superpowers/specs/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis-design.md

use std::sync::Arc;

use crate::dsp::BiquadCoeffs; // existing K-weighting filters (k_weighting_pre/rlb)
use rustfft::{num_complex::Complex, Fft, FftPlanner};

// ---- Tuning constants (single source of truth) ----
/// Short time-pass window (~0.34 s @48k) and 50% hop.
pub const SHORT_WINDOW: usize = 16_384;
pub const SHORT_HOP: usize = 8_192;
/// Momentary loudness-key integration span (ms), §5.2.
pub const MOMENTARY_MS: f32 = 400.0;
/// Loud stratum = top fraction by loudness key; body = central percentile band.
pub const LOUD_STRATUM_FRACTION: f32 = 0.15;
pub const BODY_PCTL_LO: f32 = 0.25;
pub const BODY_PCTL_HI: f32 = 0.75;
/// Hard cap on short-pass windows (~12 min @48k / 8192 hop); stride beyond it.
pub const MAX_SCAN_WINDOWS: usize = 4_200;
/// Harsh / sibilant band edges (Hz), tunable. §5 / spec.
pub const HARSH_LO_HZ: f32 = 2_000.0;
pub const HARSH_HI_HZ: f32 = 5_000.0;
pub const SIBILANT_LO_HZ: f32 = 5_000.0;
pub const SIBILANT_HI_HZ: f32 = 9_000.0;
pub const AIR_LO_HZ: f32 = 9_000.0;
pub const AIR_HI_HZ: f32 = 16_000.0;
/// Short-window 31-band proxy FFT. Smaller than `SHORT_WINDOW` for cost, centered
/// inside each retained window so the Phase-B temporal trigger is sample-rate-aware
/// without turning deep analysis into a full mastering render.
const WINDOW_DETAIL_FFT: usize = 4_096;

/// One short-window's time-varying measurements (ordered series, §4.2).
#[derive(Debug, Clone, Copy)]
pub struct WindowMetrics {
    /// Momentary-style K-weighted loudness key (LUFS-like dB). `NEG_INFINITY`
    /// for a silent/non-finite window (excluded from every stratum, §5.1).
    pub loudness_key: f32,
    /// Linear peak of the mono downmix `0.5*(L+R)` over the window (same downmix
    /// as `loudness_key`/`crest`, so momentary-PSR is self-consistent; understates
    /// hard-panned/decorrelated material vs a per-channel `max(|L|,|R|)`). For
    /// momentary-PSR + crest (§5.3).
    pub sample_peak: f32,
    /// Crest = peak / RMS (linear) over the window.
    pub crest: f32,
    /// Side / (mid + side) energy ratio over the window.
    pub stereo_width: f32,
    /// L/R Pearson correlation over the window; `NaN` for mono.
    pub stereo_correlation: f32,
    /// 3-band tonal shares (low/mid/high). Retained for Phase-A diagnostics
    /// and historical tests only — Phase-B confidence consumes the 31-band
    /// detail fields below, not these.
    pub low: f32,
    pub mid: f32,
    pub high: f32,
    /// Sample-rate-aware energy shares rolled up to the multiband compressor's
    /// actual low/mid/high crossover bands.
    pub comp_low_31: f32,
    pub comp_mid_31: f32,
    pub comp_high_31: f32,
    /// Sample-rate-aware one-third-octave detail derived from the same short
    /// window. These feed Phase-B confidence so harsh/sibilant/air/bass behavior
    /// no longer depends on the old approximate 3-band helper.
    pub low_31: f32,
    pub harsh_31: f32,
    pub sibilant_31: f32,
    pub air_31: f32,
    /// High-detail minus low-detail balance, roughly [-1, 1].
    pub tilt_31: f32,
}

/// Loudness-stratified aggregate of one axis over the finite-window set (§4.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStrata {
    pub whole: f32,
    pub loud: f32,
    pub body: f32,
    /// IQR of the per-window values (Fisher-z IQR for correlation), §5.4.
    pub dispersion: f32,
}

/// Linear-interpolated percentile of an already-sorted ascending slice.
/// `p` in [0,1]. Empty slice → 0.0.
pub(crate) fn percentile_sorted(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f32;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f32;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

/// IQR (p75 − p25) of finite values. Non-finite dropped. Sorts a copy (f32
/// total order via total_cmp for determinism).
pub(crate) fn iqr(values: &[f32]) -> f32 {
    let mut v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.len() < 2 {
        return 0.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    percentile_sorted(&v, 0.75) - percentile_sorted(&v, 0.25)
}

/// Nominal center (Hz) of one-third-octave band `i` (see analysis::THIRD_OCTAVE_CENTERS).
pub fn band_center_hz(i: usize) -> f32 {
    *crate::analysis::THIRD_OCTAVE_CENTERS.get(i).unwrap_or(&0.0)
}

/// Strata for one axis: `vals[i]` paired with loudness `keys[i]`. Windows whose
/// key OR value is non-finite are excluded from ALL strata (§5.1) — dropping the
/// value too keeps a NaN (e.g. mono `stereo_correlation`) from poisoning the f64
/// means; an axis with no finite values (mono correlation) yields an all-zero
/// `AxisStrata`. Deterministic: tuples
/// carry the original (finite-filtered) window index and sort on `(key, index)`
/// via `total_cmp` then `cmp`, so the tiebreak is explicit and robust even under
/// an unstable sort.
pub(crate) fn axis_strata(vals: &[f32], keys: &[f32]) -> AxisStrata {
    let mut finite: Vec<(f32, f32, usize)> = vals
        .iter()
        .zip(keys.iter())
        .filter(|(v, k)| k.is_finite() && v.is_finite())
        .enumerate()
        .map(|(i, (v, k))| (*k, *v, i))
        .collect();
    if finite.is_empty() {
        return AxisStrata {
            whole: 0.0,
            loud: 0.0,
            body: 0.0,
            dispersion: 0.0,
        };
    }
    let whole = finite.iter().map(|(_, v, _)| *v as f64).sum::<f64>() as f32 / finite.len() as f32;
    let vals_only: Vec<f32> = finite.iter().map(|(_, v, _)| *v).collect();
    let dispersion = iqr(&vals_only);
    // sort ascending by (loudness key, original index) — explicit, total-order tiebreak.
    finite.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.2.cmp(&b.2)));
    // loud = mean of the top LOUD_STRATUM_FRACTION by key
    let loud_n = ((finite.len() as f32 * LOUD_STRATUM_FRACTION).ceil() as usize).max(1);
    let loud = finite[finite.len() - loud_n..]
        .iter()
        .map(|(_, v, _)| *v as f64)
        .sum::<f64>() as f32
        / loud_n as f32;
    // body = mean of values whose key sits in [p25,p75] of keys
    let lo_idx = (finite.len() as f32 * BODY_PCTL_LO).floor() as usize;
    let hi_idx = ((finite.len() as f32 * BODY_PCTL_HI).ceil() as usize).min(finite.len());
    // body = central [p25,p75] index band; guard a degenerate empty span.
    let body_end = hi_idx.max(lo_idx + 1).min(finite.len());
    let body_slice = &finite[lo_idx..body_end];
    let body = body_slice.iter().map(|(_, v, _)| *v as f64).sum::<f64>() as f32
        / body_slice.len().max(1) as f32;
    AxisStrata {
        whole,
        loud,
        body,
        dispersion,
    }
}

/// IQR of Fisher-z-transformed correlation values (variance of a bounded
/// [−1,1] stat is meaningless). NaN (mono windows) dropped.
pub(crate) fn fisher_z_iqr(corr: &[f32]) -> f32 {
    let z: Vec<f32> = corr
        .iter()
        .copied()
        .filter(|c| c.is_finite())
        .map(|c| {
            let c = c.clamp(-0.999_999, 0.999_999);
            0.5 * ((1.0 + c) / (1.0 - c)).ln()
        })
        .collect();
    iqr(&z)
}

/// Backend-internal deep analysis (§6). Holds the ordered series + aggregates +
/// the 31-band whole-track tonal curve. Not `Copy` (carries a `Vec`); store via
/// `Arc`. Only `Debug` is derived (Arc field needs it); never Serialize.
#[derive(Debug, Clone)]
pub struct DeepAnalysis {
    /// 31-band one-third-octave whole-track shares (sum ~1.0), long pass (§4.1).
    /// (Whole-track `harsh_share`/`sibilant_share` were computed here but never
    /// read — deleted 2026-07-03, audit Batch H tail. The LIVE per-window
    /// `harsh_31`/`sibilant_31` on `WindowMetrics` are what Phase-B confidence
    /// consumes; re-derive whole-track shares from `bands_31` if Tier-2 ever
    /// wants them.)
    pub bands_31: [f32; 31],
    /// Retained ordered per-window series (§4.3).
    pub windows: Vec<WindowMetrics>,
    /// Per-axis strata + dispersion derived from `windows`.
    pub loudness: AxisStrata,
    pub crest: AxisStrata,
    pub brightness: AxisStrata, // per-window `high` 3-band share
    pub stereo_width: AxisStrata,
    pub stereo_correlation: AxisStrata,
}

impl DeepAnalysis {
    /// Assemble from the 31-band whole-track curve + the per-window series.
    /// Every per-axis aggregate is loudness-stratified over the SAME loudness
    /// keys (so loud/body refer to the same windows across axes). Correlation
    /// dispersion uses Fisher-z IQR (variance of a bounded [-1,1] stat is
    /// meaningless); its whole/loud/body means use raw correlation.
    pub fn from_parts(bands_31: [f32; 31], windows: Vec<WindowMetrics>) -> Self {
        let keys: Vec<f32> = windows.iter().map(|w| w.loudness_key).collect();
        let pick = |f: fn(&WindowMetrics) -> f32| -> AxisStrata {
            let vals: Vec<f32> = windows.iter().map(f).collect();
            axis_strata(&vals, &keys)
        };
        let loudness = pick(|w| w.loudness_key);
        let crest = pick(|w| w.crest);
        let brightness = pick(|w| w.high);
        let stereo_width = pick(|w| w.stereo_width);
        // correlation strata: means use raw corr (NaN values dropped by axis_strata,
        // so a mono track -> all-NaN -> all-zero strata); dispersion uses Fisher-z IQR.
        let corr_vals: Vec<f32> = windows.iter().map(|w| w.stereo_correlation).collect();
        let mut stereo_correlation = axis_strata(&corr_vals, &keys);
        stereo_correlation.dispersion = fisher_z_iqr(&corr_vals);
        Self {
            bands_31,
            windows,
            loudness,
            crest,
            brightness,
            stereo_width,
            stereo_correlation,
        }
    }
}

/// Low-percentile momentary PSR per compressor band. `None` for a band means
/// the deep-analysis window set never had enough energy in that band to make a
/// band-specific read defensible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandPsrStats {
    pub low_p10_db: Option<f32>,
    pub mid_p10_db: Option<f32>,
    pub high_p10_db: Option<f32>,
}

impl BandPsrStats {
    pub fn all_bands_below(&self, threshold_db: f32) -> bool {
        [self.low_p10_db, self.mid_p10_db, self.high_p10_db]
            .into_iter()
            .all(|psr| psr.is_some_and(|psr| psr <= threshold_db))
    }
}

const BAND_PSR_MIN_SHARE: f32 = 0.10;

/// Momentary peak-to-short-term loudness ratio for one deep-analysis window.
pub fn momentary_psr_db(window: &WindowMetrics) -> Option<f32> {
    if !window.loudness_key.is_finite()
        || !window.sample_peak.is_finite()
        || window.sample_peak <= 0.0
    {
        return None;
    }
    Some(20.0 * window.sample_peak.max(1.0e-9).log10() - window.loudness_key)
}

/// Derive low-percentile PSR per compressor band from the retained window
/// series. The PSR value is measured per window; the sample-rate-aware 31-band
/// compressor shares decide which band buckets that window can represent.
pub fn band_psr_p10_db(deep: &DeepAnalysis) -> Option<BandPsrStats> {
    let mut low = Vec::new();
    let mut mid = Vec::new();
    let mut high = Vec::new();

    for window in &deep.windows {
        let Some(psr) = momentary_psr_db(window) else {
            continue;
        };
        if window.comp_low_31 >= BAND_PSR_MIN_SHARE {
            low.push(psr);
        }
        if window.comp_mid_31 >= BAND_PSR_MIN_SHARE {
            mid.push(psr);
        }
        if window.comp_high_31 >= BAND_PSR_MIN_SHARE {
            high.push(psr);
        }
    }

    let stats = BandPsrStats {
        low_p10_db: percentile_finite_unsorted(&mut low, 0.10),
        mid_p10_db: percentile_finite_unsorted(&mut mid, 0.10),
        high_p10_db: percentile_finite_unsorted(&mut high, 0.10),
    };
    if stats.low_p10_db.is_none() && stats.mid_p10_db.is_none() && stats.high_p10_db.is_none() {
        None
    } else {
        Some(stats)
    }
}

fn percentile_finite_unsorted(values: &mut Vec<f32>, p: f32) -> Option<f32> {
    values.retain(|value| value.is_finite());
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    Some(percentile_sorted(values, p))
}

/// Short-window time pass. Returns the ordered per-window series. `channels`
/// interleaved samples. Mono = (L+R)/2 for spectral/loudness; L/R kept for the
/// stereo axis. Caps at MAX_SCAN_WINDOWS by widening the hop.
pub fn scan_windows(samples: &[f32], sample_rate: u32, channels: usize) -> Vec<WindowMetrics> {
    scan_windows_with_progress(samples, sample_rate, channels, &mut |_| {})
}

/// `scan_windows` with a 0..=1 progress callback, invoked every few dozen
/// windows (2026-08-19: the deep scan is the costliest analysis stage and
/// used to report nothing between its start and end, so the UI bar parked
/// at 80% and read as stuck). Identical output to `scan_windows`.
pub fn scan_windows_with_progress(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
    progress: &mut dyn FnMut(f32),
) -> Vec<WindowMetrics> {
    if channels == 0 || sample_rate == 0 {
        return Vec::new();
    }
    let total_frames = samples.len() / channels;
    if total_frames < SHORT_WINDOW {
        return Vec::new();
    }
    // Choose hop so the window count stays <= MAX_SCAN_WINDOWS.
    let positions = total_frames.saturating_sub(SHORT_WINDOW) / SHORT_HOP + 1;
    let hop = if positions > MAX_SCAN_WINDOWS {
        // stride: spread MAX_SCAN_WINDOWS windows across the track.
        ((total_frames - SHORT_WINDOW) / (MAX_SCAN_WINDOWS - 1)).max(1)
    } else {
        SHORT_HOP
    };

    // K-weighting biquads (reuse the existing BS.1770 pre-filters) for the
    // momentary loudness key. The 400 ms span is clamped to available samples.
    let pre = BiquadCoeffs::k_weighting_pre(sample_rate);
    let rlb = BiquadCoeffs::k_weighting_rlb(sample_rate);
    let mom_frames = ((MOMENTARY_MS / 1000.0) * sample_rate as f32) as usize;
    let detail_fft_size = WINDOW_DETAIL_FFT.min(SHORT_WINDOW);
    let mut planner = FftPlanner::<f32>::new();
    let detail_fft = planner.plan_fft_forward(detail_fft_size);

    let expected = (total_frames.saturating_sub(SHORT_WINDOW) / hop + 1).max(1);
    // Report roughly every 1/32 of the scan (and at least every 32 windows).
    let report_every = (expected / 32).max(1);
    let mut out = Vec::with_capacity(expected);
    let mut mono_scratch = Vec::with_capacity(SHORT_WINDOW);
    let mut start = 0usize;
    while start + SHORT_WINDOW <= total_frames {
        out.push(measure_window(
            samples,
            channels,
            start,
            SHORT_WINDOW,
            sample_rate,
            mom_frames,
            &pre,
            &rlb,
            &detail_fft,
            &mut mono_scratch,
        ));
        if out.len() % report_every == 0 {
            progress((out.len() as f32 / expected as f32).min(1.0));
        }
        start += hop;
    }
    out
}

/// Frame interval `[lo, hi)` for the per-window momentary loudness key: a span of
/// `mom_frames` (~400 ms) centered on the window CENTER (`start + window/2`),
/// clamped to `[0, total)` at the track edges (shrinks, never zero-pads — spec §5.2).
pub(crate) fn momentary_span(
    start: usize,
    window: usize,
    mom_frames: usize,
    total: usize,
) -> (usize, usize) {
    let half = mom_frames / 2;
    let center = start + window / 2;
    let lo = center.saturating_sub(half);
    let hi = (center + half).min(total);
    (lo, hi)
}

#[allow(clippy::too_many_arguments)]
fn measure_window(
    samples: &[f32],
    channels: usize,
    start: usize,
    window: usize,
    sample_rate: u32,
    mom_frames: usize,
    pre: &BiquadCoeffs,
    rlb: &BiquadCoeffs,
    detail_fft: &Arc<dyn Fft<f32>>,
    mono_scratch: &mut Vec<f32>,
) -> WindowMetrics {
    // mono sum + L/R for this window
    mono_scratch.clear();
    let mut peak = 0.0_f32;
    let mut sum_sq = 0.0_f64;
    let mut sum_l = 0.0_f64;
    let mut sum_r = 0.0_f64;
    for f in 0..window {
        let base = (start + f) * channels;
        let l = samples[base];
        let r = if channels > 1 { samples[base + 1] } else { l };
        let mono = if channels > 1 { 0.5 * (l + r) } else { l };
        peak = peak.max(mono.abs());
        sum_sq += (mono as f64) * (mono as f64);
        sum_l += l as f64;
        sum_r += r as f64;
        mono_scratch.push(mono);
    }
    let rms = (sum_sq / window as f64).sqrt() as f32;
    let crest = if rms > 1e-9 { peak / rms } else { 1.0 };

    // momentary K-weighted loudness over a ~400 ms span centered on the window
    // (clamped at the track edges; see `momentary_span`).
    let total = samples.len() / channels;
    let (mom_lo, mom_hi) = momentary_span(start, window, mom_frames, total);
    let avail = mom_hi.saturating_sub(mom_lo);
    // Require >= half the 400 ms span present, else the momentary loudness read
    // is unreliable -> exclude this window (NEG_INFINITY).
    let loudness_key = if avail >= mom_frames / 2 {
        kweighted_lufs(samples, channels, mom_lo, mom_hi, pre, rlb)
    } else {
        f32::NEG_INFINITY
    };

    let mono_slice = mono_scratch.as_slice();
    // The 3-band read is retained for Phase-A diagnostics and historical tests.
    // Phase-B confidence consumes the sample-rate-aware 31-band detail below.
    let three = crate::analysis::compute_spectral_balance(mono_slice, sample_rate, 1);
    let detail = window_detail_features(mono_slice, sample_rate, detail_fft);

    // stereo: width = side/(mid+side); correlation via two-pass.
    let (width, corr) = if channels > 1 {
        stereo_window(samples, channels, start, window, sum_l, sum_r)
    } else {
        (0.0, f32::NAN)
    };

    WindowMetrics {
        loudness_key,
        sample_peak: peak,
        crest,
        stereo_width: width,
        stereo_correlation: corr,
        low: three.low,
        mid: three.mid,
        high: three.high,
        comp_low_31: detail.comp_low,
        comp_mid_31: detail.comp_mid,
        comp_high_31: detail.comp_high,
        low_31: detail.low,
        harsh_31: detail.harsh,
        sibilant_31: detail.sibilant,
        air_31: detail.air,
        tilt_31: detail.tilt,
    }
}

#[derive(Debug, Clone, Copy)]
struct WindowDetail {
    comp_low: f32,
    comp_mid: f32,
    comp_high: f32,
    low: f32,
    harsh: f32,
    sibilant: f32,
    air: f32,
    tilt: f32,
}

fn window_detail_features(
    mono_slice: &[f32],
    sample_rate: u32,
    fft: &Arc<dyn Fft<f32>>,
) -> WindowDetail {
    let fft_size = fft.len();
    if sample_rate == 0 || mono_slice.len() < fft_size || fft_size < 1024 {
        return WindowDetail {
            comp_low: 0.0,
            comp_mid: 0.0,
            comp_high: 0.0,
            low: 0.0,
            harsh: 0.0,
            sibilant: 0.0,
            air: 0.0,
            tilt: 0.0,
        };
    }

    let start = (mono_slice.len() - fft_size) / 2;
    let two_pi = 2.0 * std::f32::consts::PI;
    let mut buf: Vec<Complex<f32>> = Vec::with_capacity(fft_size);
    for i in 0..fft_size {
        let w = 0.5 * (1.0 - (two_pi * i as f32 / (fft_size as f32 - 1.0)).cos());
        buf.push(Complex {
            re: mono_slice[start + i] * w,
            im: 0.0,
        });
    }
    fft.process(&mut buf);

    let bins = fft_size / 2;
    let bin_hz = sample_rate as f32 / fft_size as f32;
    let mut low = 0.0_f64;
    let mut comp_low = 0.0_f64;
    let mut comp_mid = 0.0_f64;
    let mut comp_high = 0.0_f64;
    let mut harsh = 0.0_f64;
    let mut sibilant = 0.0_f64;
    let mut air = 0.0_f64;
    let mut upper = 0.0_f64;
    let mut total = 0.0_f64;
    for (bin, c) in buf.iter().copied().enumerate().take(bins).skip(1) {
        let freq = bin as f32 * bin_hz;
        let Some(idx) = crate::analysis::third_octave_band(freq) else {
            continue;
        };
        let center = band_center_hz(idx);
        let power = (c.re as f64) * (c.re as f64) + (c.im as f64) * (c.im as f64);
        total += power;
        if center < crate::dsp::LR4_CROSSOVER_LOW_HZ {
            comp_low += power;
        } else if center < crate::dsp::LR4_CROSSOVER_HIGH_HZ {
            comp_mid += power;
        } else {
            comp_high += power;
        }
        if (20.0..250.0).contains(&center) {
            low += power;
        }
        if (HARSH_LO_HZ..HARSH_HI_HZ).contains(&center) {
            harsh += power;
        }
        if (SIBILANT_LO_HZ..SIBILANT_HI_HZ).contains(&center) {
            sibilant += power;
        }
        if (AIR_LO_HZ..AIR_HI_HZ).contains(&center) {
            air += power;
        }
        if (2_500.0..AIR_HI_HZ).contains(&center) {
            upper += power;
        }
    }
    if total <= 1.0e-12 {
        return WindowDetail {
            comp_low: 0.0,
            comp_mid: 0.0,
            comp_high: 0.0,
            low: 0.0,
            harsh: 0.0,
            sibilant: 0.0,
            air: 0.0,
            tilt: 0.0,
        };
    }
    let low = (low / total) as f32;
    let comp_low = (comp_low / total) as f32;
    let comp_mid = (comp_mid / total) as f32;
    let comp_high = (comp_high / total) as f32;
    let harsh = (harsh / total) as f32;
    let sibilant = (sibilant / total) as f32;
    let air = (air / total) as f32;
    let upper = (upper / total) as f32;
    WindowDetail {
        comp_low,
        comp_mid,
        comp_high,
        low,
        harsh,
        sibilant,
        air,
        tilt: (upper - low).clamp(-1.0, 1.0),
    }
}

fn kweighted_lufs(
    samples: &[f32],
    channels: usize,
    lo: usize,
    hi: usize,
    pre: &BiquadCoeffs,
    rlb: &BiquadCoeffs,
) -> f32 {
    let mut s1 = crate::dsp::BiquadState::default();
    let mut s2 = crate::dsp::BiquadState::default();
    let mut sum_sq = 0.0_f64;
    let mut count = 0usize;
    for f in lo..hi {
        let base = f * channels;
        let mono = if channels > 1 {
            0.5 * (samples[base] + samples[base + 1])
        } else {
            samples[base]
        };
        let y = s2.process(rlb, s1.process(pre, mono));
        sum_sq += (y as f64) * (y as f64);
        count += 1;
    }
    if count == 0 {
        return f32::NEG_INFINITY;
    }
    let ms = sum_sq / count as f64;
    if ms <= 0.0 {
        return f32::NEG_INFINITY;
    }
    // BS.1770 LUFS calibration offset (see dsp::k_weighting_rlb).
    (-0.691 + 10.0 * ms.log10()) as f32
}

fn stereo_window(
    samples: &[f32],
    channels: usize,
    start: usize,
    window: usize,
    sum_l: f64,
    sum_r: f64,
) -> (f32, f32) {
    let n = window as f64;
    let mean_l = sum_l / n;
    let mean_r = sum_r / n;
    let (mut cov, mut var_l, mut var_r, mut mid_e, mut side_e) = (0.0, 0.0, 0.0, 0.0_f64, 0.0_f64);
    for f in 0..window {
        let base = (start + f) * channels;
        let l = samples[base] as f64;
        let r = samples[base + 1] as f64;
        let dl = l - mean_l;
        let dr = r - mean_r;
        cov += dl * dr;
        var_l += dl * dl;
        var_r += dr * dr;
        let mid = 0.5 * (l + r);
        let side = 0.5 * (l - r);
        mid_e += mid * mid;
        side_e += side * side;
    }
    let denom = (var_l * var_r).sqrt();
    let corr = if denom > 1e-12 {
        (cov / denom).clamp(-1.0, 1.0) as f32
    } else {
        1.0
    };
    let width = if mid_e + side_e > 1e-12 {
        (side_e / (mid_e + side_e)) as f32
    } else {
        0.0
    };
    (width, corr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn window_metric_bits(window: &WindowMetrics) -> [u32; 16] {
        [
            window.loudness_key.to_bits(),
            window.sample_peak.to_bits(),
            window.crest.to_bits(),
            window.stereo_width.to_bits(),
            window.stereo_correlation.to_bits(),
            window.low.to_bits(),
            window.mid.to_bits(),
            window.high.to_bits(),
            window.comp_low_31.to_bits(),
            window.comp_mid_31.to_bits(),
            window.comp_high_31.to_bits(),
            window.low_31.to_bits(),
            window.harsh_31.to_bits(),
            window.sibilant_31.to_bits(),
            window.air_31.to_bits(),
            window.tilt_31.to_bits(),
        ]
    }

    #[test]
    fn percentile_of_sorted_is_linear_interpolated() {
        let v = [0.0_f32, 1.0, 2.0, 3.0, 4.0];
        assert!(approx(percentile_sorted(&v, 0.0), 0.0));
        assert!(approx(percentile_sorted(&v, 1.0), 4.0));
        assert!(approx(percentile_sorted(&v, 0.5), 2.0));
    }

    #[test]
    fn iqr_is_p75_minus_p25() {
        let v = [0.0_f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        // p25=1.75, p75=5.25 -> IQR=3.5
        assert!(approx(iqr(&v), 3.5));
    }

    #[test]
    fn strata_excludes_non_finite_and_computes_loud_body() {
        // values paired with loudness keys; one silent window must be dropped.
        let vals = [10.0_f32, 20.0, 30.0, 40.0, 99.0];
        let keys = [-10.0_f32, -8.0, -6.0, -4.0, f32::NEG_INFINITY];
        let s = axis_strata(&vals, &keys);
        // silent (99.0) excluded -> whole over [10,20,30,40] = 25.0
        assert!(approx(s.whole, 25.0));
        // loud = top 15% by key among finite -> at least the single loudest (40.0)
        assert!(approx(s.loud, 40.0));
        // body = 25th-75th pctl band by key -> middle values present, 99 absent
        assert!(s.body > 10.0 && s.body < 40.0);
        assert!(s.dispersion.is_finite());
    }

    #[test]
    fn fisher_z_iqr_handles_bounded_correlation() {
        // NaN dropped; remaining four z-transformed and IQR'd. Pinned value.
        let corr = [0.1_f32, 0.5, 0.9, -0.2, f32::NAN];
        let d = fisher_z_iqr(&corr);
        assert!(d.is_finite() && d >= 0.0);
        assert!(approx(d, 0.755_466_1));
    }

    #[test]
    fn fisher_z_iqr_clamp_keeps_perfect_correlation_finite() {
        // Perfect ±1.0 correlations would push z to ±inf without the ±0.999999
        // clamp, making the IQR non-finite. With the clamp the result stays finite.
        let corr = [0.1_f32, 0.5, 0.9, -0.2, 1.0, -1.0];
        let d = fisher_z_iqr(&corr);
        assert!(d.is_finite() && d >= 0.0);
    }

    #[test]
    fn percentile_sorted_handles_empty_and_single() {
        assert!(approx(percentile_sorted(&[], 0.5), 0.0));
        assert!(approx(percentile_sorted(&[42.0_f32], 0.5), 42.0));
    }

    #[test]
    fn iqr_short_input_and_drops_non_finite() {
        // len < 2 -> 0.0
        assert!(approx(iqr(&[1.0_f32]), 0.0));
        // a stray NaN is dropped, leaving the same IQR as the finite-only case.
        let v = [0.0_f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, f32::NAN];
        assert!(approx(iqr(&v), 3.5));
    }

    #[test]
    fn axis_strata_all_non_finite_keys_returns_zero() {
        let s = axis_strata(&[1.0_f32, 2.0], &[f32::NEG_INFINITY, f32::NAN]);
        assert_eq!(
            s,
            AxisStrata {
                whole: 0.0,
                loud: 0.0,
                body: 0.0,
                dispersion: 0.0,
            }
        );
    }

    #[test]
    fn scan_windows_produces_ordered_series_and_drops_short_tail() {
        let sr = 48_000_u32;
        // 1.5 s mono so a couple of 16384 windows (50% hop) fit.
        let n = (sr as f32 * 1.5) as usize;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let windows = scan_windows(&samples, sr, 1);
        assert!(!windows.is_empty(), "at least one full window fits");
        for w in &windows {
            assert!(w.sample_peak > 0.0 && w.sample_peak <= 1.0);
            assert!(w.crest >= 1.0); // peak >= rms
            assert!((w.low + w.mid + w.high - 1.0).abs() < 0.05);
            // mono → correlation is NaN, width 0
            assert!(w.stereo_correlation.is_nan());
        }
    }

    #[test]
    fn scan_windows_matches_fixed_full_metric_fixture() {
        let sr = 48_000_u32;
        let frames = SHORT_WINDOW + SHORT_HOP;
        let mut samples = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            let l = (((i * 17 + 11) % 257) as f32 / 128.0 - 1.0) * 0.41;
            let r = (((i * 29 + 7) % 263) as f32 / 131.0 - 1.0) * 0.37;
            samples.push(l);
            samples.push(r);
        }

        let windows = scan_windows(&samples, sr, 2);
        // One OS/arch-independent reference, stored as f32 bits. The 31-band
        // detail metrics derive from libm transcendentals (sin/cos/powf), so a
        // few round 1 ULP (~3e-8) differently on macOS-arm64 — the DSP itself is
        // platform-identical. The assert below compares the metric VALUES within
        // a tolerance that clears that platform delta, so these bits do not need
        // a per-OS variant (see the tolerance comparison after the array).
        let expected_windows: [[u32; 16]; 2] = [
            [
                3_242_875_236,
                1_053_273_620,
                1_075_574_712,
                1_056_964_498,
                920_402_093,
                979_870_399,
                1_048_248_139,
                1_061_226_053,
                778_431_265,
                1_052_326_102,
                1_059_283_861,
                919_440_903,
                1_051_980_982,
                1_053_122_930,
                1_042_109_556,
                1_063_507_546,
            ],
            [
                3_242_887_790,
                1_053_226_234,
                1_075_533_567,
                1_056_951_073,
                978_623_375,
                979_258_547,
                1_048_205_208,
                1_061_237_383,
                781_233_218,
                1_051_673_207,
                1_059_610_309,
                952_033_446,
                1_051_577_123,
                1_052_679_867,
                1_041_752_903,
                1_062_993_847,
            ],
        ];
        // Single OS/arch-independent reference (`expected_windows` above). The
        // former per-OS `[u32;16]` split existed ONLY because libm rounds a few
        // 31-band metrics 1 ULP (~3e-8) differently on macOS-arm64 — the DSP is
        // platform-identical. Compare the metric VALUES within a scale-aware
        // tolerance that clears the platform delta but is far below any
        // structural drift, so one reference passes on every OS/arch and an
        // Intel Mac no longer spuriously fails.
        assert_eq!(windows.len(), expected_windows.len());
        for (window, reference) in windows.iter().zip(expected_windows) {
            let observed = window_metric_bits(window);
            for (i, (obs_bits, ref_bits)) in observed.iter().zip(reference).enumerate() {
                let a = f32::from_bits(*obs_bits);
                let b = f32::from_bits(ref_bits);
                if a.is_nan() && b.is_nan() {
                    continue;
                }
                let tol = 1.0e-5 + 1.0e-5 * b.abs();
                assert!(
                    (a - b).abs() <= tol,
                    "deep-analysis window metric {i} drifted: {a} vs reference {b} \
                     (delta {}, tol {tol}); investigate DSP drift before regenerating",
                    (a - b).abs()
                );
            }
        }
    }

    #[test]
    fn scan_windows_carries_31band_harsh_sibilant_and_air_detail() {
        let sr = 48_000_u32;
        let make = |freq: f32| {
            let n = sr as usize * 2;
            let omega = 2.0 * std::f32::consts::PI * freq / sr as f32;
            let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
            scan_windows(&samples, sr, 1)
        };

        let harsh = make(3_000.0);
        let sibilant = make(7_000.0);
        let air = make(12_000.0);
        assert!(harsh.iter().all(|w| w.harsh_31 > 0.5), "{harsh:?}");
        assert!(sibilant.iter().all(|w| w.sibilant_31 > 0.5), "{sibilant:?}");
        assert!(air.iter().all(|w| w.air_31 > 0.5), "{air:?}");
        assert!(
            air.iter().all(|w| w.harsh_31 + w.sibilant_31 < 0.2),
            "air-only windows should not look harsh/sibilant: {air:?}"
        );
    }

    #[test]
    fn deep_from_series_builds_strata_and_psr_is_derivable() {
        let sr = 48_000_u32;
        let n = sr as usize * 3;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let windows = scan_windows(&samples, sr, 1);
        let bands = [1.0_f32 / 31.0; 31]; // placeholder curve for this unit test
        let da = DeepAnalysis::from_parts(bands, windows.clone());
        assert!(da.loudness.whole.is_finite());
        assert!(da.crest.whole >= 1.0);
        // momentary-PSR per window = sample_peak(dB) - loudness_key; derivable, finite.
        let w = windows.iter().find(|w| w.loudness_key.is_finite()).unwrap();
        let psr = 20.0 * w.sample_peak.max(1e-9).log10() - w.loudness_key;
        assert!(psr.is_finite());
    }

    #[test]
    fn band_psr_rollup_distinguishes_dense_tone_from_transient_clicks() {
        let sr = 48_000_u32;
        let seconds = 3;
        let n = sr as usize * seconds;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let dense_tone: Vec<f32> = (0..n).map(|i| 0.35 * (omega * i as f32).sin()).collect();
        let dense = DeepAnalysis::from_parts([1.0 / 31.0; 31], scan_windows(&dense_tone, sr, 1));
        let dense_psr = band_psr_p10_db(&dense).expect("dense tone PSR");
        let dense_mid = dense_psr.mid_p10_db.expect("dense tone mid-band PSR");

        let mut clicks = vec![0.0_f32; n];
        for i in (0..n).step_by(sr as usize / 4) {
            clicks[i] = 0.95;
            if i + 1 < clicks.len() {
                clicks[i + 1] = -0.95;
            }
        }
        let transient = DeepAnalysis::from_parts([1.0 / 31.0; 31], scan_windows(&clicks, sr, 1));
        let transient_psr = band_psr_p10_db(&transient).expect("transient PSR");
        let transient_max = [
            transient_psr.low_p10_db,
            transient_psr.mid_p10_db,
            transient_psr.high_p10_db,
        ]
        .into_iter()
        .flatten()
        .fold(f32::NEG_INFINITY, f32::max);

        assert!(
            dense_mid < 8.0,
            "steady dense tone should have a low mid-band PSR, got {dense_mid}"
        );
        assert!(
            transient_max > dense_mid + 6.0,
            "transient clicks should retain much higher PSR than dense tone: \
             dense={dense_mid}, transient={transient_max}"
        );
    }

    #[test]
    fn scan_windows_caps_at_max_windows() {
        // Build a signal long enough to exceed MAX_SCAN_WINDOWS at 50% hop, assert
        // the cap holds by striding the hop.
        let sr = 48_000_u32;
        let needed = (MAX_SCAN_WINDOWS + 100) * SHORT_HOP + SHORT_WINDOW;
        let samples = vec![0.1_f32; needed];
        let windows = scan_windows(&samples, sr, 1);
        assert!(windows.len() <= MAX_SCAN_WINDOWS, "got {}", windows.len());
    }

    #[test]
    fn momentary_span_is_window_centered_and_400ms() {
        // The per-window loudness key must integrate a span of mom_frames (~400 ms)
        // CENTERED on the window center (start + window/2), per spec §5.2 / handoff §3.2
        // ("a single transient can't crown a window 'loudest'"). Pins both the width
        // and the centering so the span geometry cannot silently drift again.
        let mom_frames = 19_200; // 400 ms @ 48k
        let start = 1_000_000;
        let total = 100_000_000; // far from both edges -> no clamping
        let (lo, hi) = momentary_span(start, SHORT_WINDOW, mom_frames, total);
        let center = start + SHORT_WINDOW / 2;
        assert_eq!(
            hi - lo,
            mom_frames,
            "span must be exactly mom_frames (400 ms) wide"
        );
        assert_eq!(
            (lo + hi) / 2,
            center,
            "span must be centered on the window center"
        );
        assert_eq!(lo, center - mom_frames / 2);
        assert_eq!(hi, center + mom_frames / 2);
    }

    #[test]
    fn momentary_span_clamps_at_track_edges_without_zero_padding() {
        let mom_frames = 19_200;
        // Window at the very start: low bound clamps to 0 (shrink, not zero-pad).
        let (lo, hi) = momentary_span(0, SHORT_WINDOW, mom_frames, 100_000);
        assert_eq!(lo, 0);
        assert_eq!(hi, SHORT_WINDOW / 2 + mom_frames / 2);
        // Window near the end: high bound clamps to total.
        let total = SHORT_WINDOW + 1_000;
        let start = total - SHORT_WINDOW;
        let (lo2, hi2) = momentary_span(start, SHORT_WINDOW, mom_frames, total);
        assert_eq!(hi2, total);
        assert!(lo2 < hi2);
    }

    #[test]
    fn axis_strata_drops_non_finite_values_not_just_keys() {
        // A window with a finite loudness KEY but a non-finite VALUE must be
        // excluded so it can't poison the f64 means (previously only the key was
        // checked, so a NaN value propagated to whole/loud/body).
        let vals = [10.0_f32, f32::NAN, 30.0, 40.0];
        let keys = [-10.0_f32, -8.0, -6.0, -4.0];
        let s = axis_strata(&vals, &keys);
        // NaN value dropped -> whole = mean over [10, 30, 40].
        assert!(s.whole.is_finite(), "whole was {}", s.whole);
        assert!(
            (s.whole - (10.0 + 30.0 + 40.0) / 3.0).abs() < 1e-4,
            "whole was {}",
            s.whole
        );
        assert!(s.loud.is_finite() && s.body.is_finite() && s.dispersion.is_finite());
    }

    #[test]
    fn mono_track_correlation_strata_are_finite() {
        // A mono source sets every window's stereo_correlation to NaN (degenerate).
        // The correlation strata means must NOT inherit that NaN — it would poison
        // every Phase B comparison (all IEEE compares against NaN are false).
        let sr = 48_000_u32;
        let n = sr as usize * 3;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let windows = scan_windows(&samples, sr, 1);
        assert!(!windows.is_empty());
        assert!(
            windows.iter().all(|w| w.stereo_correlation.is_nan()),
            "mono windows should carry NaN correlation"
        );
        let da = DeepAnalysis::from_parts([1.0 / 31.0; 31], windows);
        assert!(
            da.stereo_correlation.whole.is_finite(),
            "whole was {}",
            da.stereo_correlation.whole
        );
        assert!(
            da.stereo_correlation.loud.is_finite(),
            "loud was {}",
            da.stereo_correlation.loud
        );
        assert!(
            da.stereo_correlation.body.is_finite(),
            "body was {}",
            da.stereo_correlation.body
        );
        assert!(da.stereo_correlation.dispersion.is_finite());
    }
}
