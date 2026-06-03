//! Tier-2 Phase A — backend-internal deep analysis. NEVER serialized to TS.
//! Additive: produced at analysis time, cached beside SourceProfile, consumed
//! (later) by Phase B. See docs/superpowers/specs/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis-design.md

use crate::dsp::BiquadCoeffs; // existing K-weighting filters (k_weighting_pre/rlb)

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

/// One short-window's time-varying measurements (ordered series, §4.2).
#[derive(Debug, Clone, Copy)]
pub struct WindowMetrics {
    /// Momentary-style K-weighted loudness key (LUFS-like dB). `NEG_INFINITY`
    /// for a silent/non-finite window (excluded from every stratum, §5.1).
    pub loudness_key: f32,
    /// Linear sample peak over the window (for momentary-PSR + crest, §5.3).
    pub sample_peak: f32,
    /// Crest = peak / RMS (linear) over the window.
    pub crest: f32,
    /// Side / (mid + side) energy ratio over the window.
    pub stereo_width: f32,
    /// L/R Pearson correlation over the window; `NaN` for mono.
    pub stereo_correlation: f32,
    /// 3-band tonal shares (low/mid/high) for temporal brightness clumping.
    pub low: f32,
    pub mid: f32,
    pub high: f32,
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
/// key is non-finite are excluded from ALL strata (§5.1). Deterministic: tuples
/// carry the original (finite-filtered) window index and sort on `(key, index)`
/// via `total_cmp` then `cmp`, so the tiebreak is explicit and robust even under
/// an unstable sort.
pub(crate) fn axis_strata(vals: &[f32], keys: &[f32]) -> AxisStrata {
    let mut finite: Vec<(f32, f32, usize)> = vals
        .iter()
        .zip(keys.iter())
        .filter(|(_, k)| k.is_finite())
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
    let body =
        body_slice.iter().map(|(_, v, _)| *v as f64).sum::<f64>() as f32 / body_slice.len().max(1) as f32;
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
    pub bands_31: [f32; 31],
    /// Harsh / sibilant shares derived from `bands_31` (tunable edges).
    pub harsh_share: f32,
    pub sibilant_share: f32,
    /// Retained ordered per-window series (§4.3).
    pub windows: Vec<WindowMetrics>,
    /// Per-axis strata + dispersion derived from `windows`.
    pub loudness: AxisStrata,
    pub crest: AxisStrata,
    pub brightness: AxisStrata, // per-window `high` 3-band share
    pub stereo_width: AxisStrata,
    pub stereo_correlation: AxisStrata,
}

/// Short-window time pass. Returns the ordered per-window series. `channels`
/// interleaved samples. Mono = (L+R)/2 for spectral/loudness; L/R kept for the
/// stereo axis. Caps at MAX_SCAN_WINDOWS by widening the hop.
pub fn scan_windows(samples: &[f32], sample_rate: u32, channels: usize) -> Vec<WindowMetrics> {
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

    let mut out = Vec::new();
    let mut start = 0usize;
    while start + SHORT_WINDOW <= total_frames {
        out.push(measure_window(
            samples, channels, start, SHORT_WINDOW, sample_rate, mom_frames, &pre, &rlb,
        ));
        start += hop;
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn measure_window(
    samples: &[f32],
    channels: usize,
    start: usize,
    window: usize,
    _sample_rate: u32,
    mom_frames: usize,
    pre: &BiquadCoeffs,
    rlb: &BiquadCoeffs,
) -> WindowMetrics {
    // mono sum + L/R for this window
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
    }
    let rms = (sum_sq / window as f64).sqrt() as f32;
    let crest = if rms > 1e-9 { peak / rms } else { 1.0 };

    // momentary K-weighted loudness over min(400ms, available) centered span.
    let half = mom_frames / 2;
    let mom_lo = start.saturating_sub(half);
    let total = samples.len() / channels;
    let mom_hi = (start + window / 2 + half).min(total);
    let avail = mom_hi.saturating_sub(mom_lo);
    // Require >= half the 400 ms span present, else the momentary loudness read
    // is unreliable -> exclude this window (NEG_INFINITY).
    let loudness_key = if avail >= mom_frames / 2 {
        kweighted_lufs(samples, channels, mom_lo, mom_hi, pre, rlb)
    } else {
        f32::NEG_INFINITY
    };

    // 3-band tonal on the window (reuse the crate helper on a mono slice copy).
    // TODO(phase-b/perf): allocates a mono copy per window just to reuse
    // compute_spectral_balance; consider a borrowed/iterator view if window
    // counts grow.
    let mono_slice: Vec<f32> = (0..window)
        .map(|f| {
            let base = (start + f) * channels;
            if channels > 1 {
                0.5 * (samples[base] + samples[base + 1])
            } else {
                samples[base]
            }
        })
        .collect();
    // NOTE: compute_spectral_balance is sample-rate-agnostic (bakes in a ~44.1k
    // band reference), so this 3-band read carries a small SR skew at 48k+.
    // Consistent/deterministic and matches infer_character's tone read
    // elsewhere; that inherited assumption is why _sample_rate is currently
    // unused here. Revisit for Phase B if finer banding is needed.
    let three = crate::analysis::compute_spectral_balance(&mono_slice, 1);

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
    fn scan_windows_caps_at_max_windows() {
        // Build a signal long enough to exceed MAX_SCAN_WINDOWS at 50% hop, assert
        // the cap holds by striding the hop.
        let sr = 48_000_u32;
        let needed = (MAX_SCAN_WINDOWS + 100) * SHORT_HOP + SHORT_WINDOW;
        let samples = vec![0.1_f32; needed];
        let windows = scan_windows(&samples, sr, 1);
        assert!(windows.len() <= MAX_SCAN_WINDOWS, "got {}", windows.len());
    }
}
