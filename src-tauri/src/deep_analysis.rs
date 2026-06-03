//! Tier-2 Phase A — backend-internal deep analysis. NEVER serialized to TS.
//! Additive: produced at analysis time, cached beside SourceProfile, consumed
//! (later) by Phase B. See docs/superpowers/specs/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis-design.md

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
}
