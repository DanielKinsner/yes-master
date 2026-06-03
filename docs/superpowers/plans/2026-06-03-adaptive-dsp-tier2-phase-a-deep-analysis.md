# Adaptive DSP Tier-2 Phase A — Deep Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a backend-internal `DeepAnalysis` (whole-track 31-band tonal curve + a retained per-window time series + loudness-stratified aggregates), cached per track, *without changing any existing value or audible behavior*.

**Architecture:** Additive. The existing whole-track spectral pass gains a *parallel* 31-band accumulation (the 6-band keeps its own direct accumulation, byte-exact). A new short-window pass produces an ordered per-window series. Both land in a new `DeepAnalysis`, attached to `AnalysisResult` via a `#[serde(skip)]` field and moved into a new `Arc`-backed store map beside `SourceProfileStore`. No frontend / wire-contract changes.

**Tech Stack:** Rust (`src-tauri`), `rustfft`, `ebur128`, serde; vitest/cargo test. Spec: `docs/superpowers/specs/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis-design.md` (v2.1).

**Branch:** `fix/adversarial-type-review-2026-06-03`.

**Verification lanes (run per task as noted):**
- Rust unit: `cd src-tauri; cargo test --lib --target-dir target/codex-rc`
- Rust full: `cd src-tauri; cargo test --target-dir target/codex-rc`
- Lint: `cd src-tauri; cargo clippy --all-targets --target-dir target/codex-rc`
- Frontend (only Task 8 touches a shared type's wire view — should stay unchanged): `npm test`; `npx tsc -b --pretty false`
- iPhone bridge (shared types changed): `cd apps/iphone-native/rust; cargo check --all-targets`
- Use `--target-dir target/codex-rc` whenever the desktop app may be running (it locks `target/debug/yes-master.exe`).

**DECIDE defaults baked in (flip if the owner says so):** mobile = gate the deep scan OFF the iPhone FFI path; persistence = per-session (in-memory only).

---

## File structure

- **Create** `src-tauri/src/deep_analysis.rs` — `DeepAnalysis`, `WindowMetrics`, `AxisStrata`, all tuning constants, the strata/IQR/Fisher-z/momentary-PSR helpers, and the short-window scan. One responsibility: *measure the time-varying series and aggregate it*. Backend-internal; never in `bindings.ts`.
- **Modify** `src-tauri/src/analysis.rs` — (a) golden 6-band test (Task 1); (b) add a parallel 31-band accumulation to `compute_spectral_balance_6band`'s sibling so the long pass yields the 31-band curve (Task 6); (c) call the short scan + assemble `DeepAnalysis` in `analyze_one` (Task 7).
- **Modify** `src-tauri/src/types.rs` — add `#[serde(skip)] deep_analysis: Option<Arc<DeepAnalysis>>` to `AnalysisResult` (Task 7). The ~10 fixture literals each gain a one-line `deep_analysis: None,`.
- **Modify** `src-tauri/src/profile_store.rs` — add `Arc<DeepAnalysis>` map, `insert_deep`/`get_deep`, extend both clear paths, fixed lock order (Task 8).
- **Modify** `src-tauri/src/engine.rs` — move `DeepAnalysis` off the result into the store in `populate_profile_store` (Task 8).
- **Modify** `src-tauri/src/lib.rs` — `mod deep_analysis;` (Task 2).
- **Modify** `apps/iphone-native/rust/src/lib.rs` — gate the deep scan off (use the existing analyze path; do not request `DeepAnalysis`) (Task 9).
- **Modify** the ~10 `AnalysisResult` literals in tests to use the new constructor or add `deep_analysis: None` (Task 7).

---

## Task 1: Golden 6-band test (tests-first — locks "changes nothing")

**Files:**
- Test: `src-tauri/src/analysis.rs` (the `#[cfg(test)] mod tests` block, near the other `spectral_balance_6band_*` tests ~line 715)

- [ ] **Step 1: Write the byte-exact golden test against today's code**

In `analysis.rs` tests module:

```rust
/// Byte-exact lock on the 6-band output (adversarial review must-fix #1). The
/// other 6-band tests are relative (sums-to-unity / mid>0.5 / low>bright) and
/// would NOT catch a value shift. Phase A adds a PARALLEL 31-band accumulation;
/// this asserts the 6 floats are bit-identical before and after that change.
/// If a future Phase B intentionally rerolls the 6-band from the 31-band, THIS
/// test is the gate that forces an explicit decision.
#[test]
fn spectral_balance_6band_is_byte_exact_golden() {
    let sr = 48_000_u32;
    // Deterministic broadband fixture: summed sines across the bands.
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
    // Capture the CURRENT values, then assert exact equality. Fill these from
    // the Step-2 run output (they are the golden constants).
    let golden = [
        bal.sub, bal.low, bal.low_mid, bal.mid, bal.presence, bal.air,
    ];
    assert_eq!([bal.sub, bal.low, bal.low_mid, bal.mid, bal.presence, bal.air], golden);
}
```

- [ ] **Step 2: Run it and capture the real golden values**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc spectral_balance_6band_is_byte_exact_golden -- --nocapture`
Expected: PASS (the self-comparison is trivially true). Then add a temporary `eprintln!("{golden:?}");` , re-run, copy the printed six `f32` values, and **replace** the `let golden = [bal.sub, ...]` line with the literal array, e.g.:

```rust
    let golden: [f32; 6] = [0.0_f32, /* paste the 6 printed values */];
```

Remove the `eprintln!`. Now the test pins the actual numbers, not a tautology.

- [ ] **Step 3: Re-run to confirm the pinned golden passes**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc spectral_balance_6band_is_byte_exact_golden`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/analysis.rs
git commit -m "test(analysis): byte-exact golden lock on 6-band (Phase A pre-refactor)"
```

---

## Task 2: `deep_analysis` module — types + constants

**Files:**
- Create: `src-tauri/src/deep_analysis.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod deep_analysis;`)

- [ ] **Step 1: Create the module with types + constants**

```rust
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
```

- [ ] **Step 2: Register the module**

In `src-tauri/src/lib.rs`, add alongside the other `mod` lines:

```rust
pub mod deep_analysis;
```

- [ ] **Step 3: Confirm it compiles**

Run: `cd src-tauri; cargo check --lib --target-dir target/codex-rc`
Expected: success (warnings about unused items are fine until later tasks).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/deep_analysis.rs src-tauri/src/lib.rs
git commit -m "feat(deep-analysis): Phase A module skeleton — types + tuning constants"
```

---

## Task 3: Strata + dispersion helpers (pure, fully unit-tested)

**Files:**
- Modify: `src-tauri/src/deep_analysis.rs`

- [ ] **Step 1: Write failing tests for the aggregator helpers**

Add a `#[cfg(test)] mod tests` to `deep_analysis.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool { (a - b).abs() < 1e-5 }

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
        let corr = [0.1_f32, 0.5, 0.9, -0.2, f32::NAN];
        let d = fisher_z_iqr(&corr);
        assert!(d.is_finite() && d >= 0.0);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc deep_analysis::tests`
Expected: FAIL (`percentile_sorted` / `iqr` / `axis_strata` / `fisher_z_iqr` not found).

- [ ] **Step 3: Implement the helpers**

Add to `deep_analysis.rs` (module level):

```rust
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
/// key is non-finite are excluded from ALL strata (§5.1). Deterministic: stable
/// sort key (loudness, index) via enumerate + total_cmp.
pub(crate) fn axis_strata(vals: &[f32], keys: &[f32]) -> AxisStrata {
    let mut finite: Vec<(f32, f32)> = vals
        .iter()
        .zip(keys.iter())
        .filter(|(_, k)| k.is_finite())
        .map(|(v, k)| (*k, *v))
        .collect();
    if finite.is_empty() {
        return AxisStrata { whole: 0.0, loud: 0.0, body: 0.0, dispersion: 0.0 };
    }
    let whole = finite.iter().map(|(_, v)| *v as f64).sum::<f64>() as f32 / finite.len() as f32;
    let vals_only: Vec<f32> = finite.iter().map(|(_, v)| *v).collect();
    let dispersion = iqr(&vals_only);
    // sort ascending by (loudness key, original order preserved by stable sort)
    finite.sort_by(|a, b| a.0.total_cmp(&b.0));
    // loud = mean of the top LOUD_STRATUM_FRACTION by key
    let loud_n = ((finite.len() as f32 * LOUD_STRATUM_FRACTION).ceil() as usize).max(1);
    let loud = finite[finite.len() - loud_n..]
        .iter()
        .map(|(_, v)| *v as f64)
        .sum::<f64>() as f32
        / loud_n as f32;
    // body = mean of values whose key sits in [p25,p75] of keys
    let lo_idx = (finite.len() as f32 * BODY_PCTL_LO).floor() as usize;
    let hi_idx = ((finite.len() as f32 * BODY_PCTL_HI).ceil() as usize).min(finite.len());
    let body_slice = &finite[lo_idx..hi_idx.max(lo_idx + 1).min(finite.len())];
    let body = body_slice.iter().map(|(_, v)| *v as f64).sum::<f64>() as f32
        / body_slice.len().max(1) as f32;
    AxisStrata { whole, loud, body, dispersion }
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
```

- [ ] **Step 4: Run to verify pass**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc deep_analysis::tests`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/deep_analysis.rs
git commit -m "feat(deep-analysis): strata + IQR/Fisher-z aggregators with tests"
```

---

## Task 4: Short-window time pass → per-window series

**Files:**
- Modify: `src-tauri/src/deep_analysis.rs`

- [ ] **Step 1: Write the failing test**

```rust
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
```

- [ ] **Step 2: Run to verify failure**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc scan_windows`
Expected: FAIL (`scan_windows` not found).

- [ ] **Step 3: Implement `scan_windows`**

```rust
use crate::dsp::BiquadCoeffs; // existing K-weighting filters (k_weighting_pre/rlb)

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
```

Add the per-window measurement (uses the existing 3-band bucketing from `compute_spectral_balance` logic — call it on the window slice; and an inline K-weighted loudness):

```rust
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
    let loudness_key = if avail >= mom_frames / 2 {
        kweighted_lufs(samples, channels, mom_lo, mom_hi, pre, rlb)
    } else {
        f32::NEG_INFINITY
    };

    // 3-band tonal on the window (reuse the crate helper on a mono slice copy).
    let mono_slice: Vec<f32> = (0..window)
        .map(|f| {
            let base = (start + f) * channels;
            if channels > 1 { 0.5 * (samples[base] + samples[base + 1]) } else { samples[base] }
        })
        .collect();
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
```

Add `kweighted_lufs` (K-weight the mono span through the two biquads, mean-square → LUFS) and `stereo_window` (side/mid energy + Pearson). Provide full bodies:

```rust
fn kweighted_lufs(
    samples: &[f32], channels: usize, lo: usize, hi: usize,
    pre: &BiquadCoeffs, rlb: &BiquadCoeffs,
) -> f32 {
    let mut s1 = crate::dsp::BiquadState::default();
    let mut s2 = crate::dsp::BiquadState::default();
    let mut sum_sq = 0.0_f64;
    let mut count = 0usize;
    for f in lo..hi {
        let base = f * channels;
        let mono = if channels > 1 { 0.5 * (samples[base] + samples[base + 1]) } else { samples[base] };
        let y = s2.process(rlb, s1.process(pre, mono));
        sum_sq += (y as f64) * (y as f64);
        count += 1;
    }
    if count == 0 { return f32::NEG_INFINITY; }
    let ms = sum_sq / count as f64;
    if ms <= 0.0 { return f32::NEG_INFINITY; }
    (-0.691 + 10.0 * ms.log10()) as f32
}

fn stereo_window(
    samples: &[f32], channels: usize, start: usize, window: usize,
    sum_l: f64, sum_r: f64,
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
    let corr = if denom > 1e-12 { (cov / denom).clamp(-1.0, 1.0) as f32 } else { 1.0 };
    let width = if mid_e + side_e > 1e-12 { (side_e / (mid_e + side_e)) as f32 } else { 0.0 };
    (width, corr)
}
```

NOTE: this requires `compute_spectral_balance` to be `pub(crate)`. Change its signature in `analysis.rs:248` from `fn compute_spectral_balance` to `pub(crate) fn compute_spectral_balance`. Also ensure `BiquadCoeffs::k_weighting_pre/rlb` and `BiquadState` are `pub` (they are used by tests already — confirm; if not, make `pub`).

- [ ] **Step 4: Run to verify pass**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc scan_windows`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/deep_analysis.rs src-tauri/src/analysis.rs
git commit -m "feat(deep-analysis): short-window time pass -> ordered per-window series"
```

---

## Task 5: Assemble `DeepAnalysis` from the series (strata + momentary-PSR)

**Files:**
- Modify: `src-tauri/src/deep_analysis.rs`

- [ ] **Step 1: Write the failing test (incl. momentary-PSR derivable)**

```rust
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
```

- [ ] **Step 2: Run to verify failure**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc deep_from_series`
Expected: FAIL (`DeepAnalysis::from_parts` not found).

- [ ] **Step 3: Implement `from_parts` + harsh/sibilant derivation**

```rust
impl DeepAnalysis {
    /// Assemble from the 31-band whole-track curve + the per-window series.
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
        // correlation strata: means use raw corr; dispersion uses Fisher-z IQR.
        let corr_vals: Vec<f32> = windows.iter().map(|w| w.stereo_correlation).collect();
        let mut stereo_correlation = axis_strata(&corr_vals, &keys);
        stereo_correlation.dispersion = fisher_z_iqr(&corr_vals);
        let (harsh_share, sibilant_share) = harsh_sibilant_from_bands(&bands_31);
        Self {
            bands_31, harsh_share, sibilant_share, windows,
            loudness, crest, brightness, stereo_width, stereo_correlation,
        }
    }
}

/// Sum band shares whose center frequency falls in the harsh / sibilant ranges.
/// `bands_31` are one-third-octave shares from 20 Hz upward (see Task 6 for the
/// edge table); `band_center_hz(i)` returns each band's nominal center.
fn harsh_sibilant_from_bands(bands: &[f32; 31]) -> (f32, f32) {
    let mut harsh = 0.0;
    let mut sib = 0.0;
    for (i, &share) in bands.iter().enumerate() {
        let c = band_center_hz(i);
        if (HARSH_LO_HZ..HARSH_HI_HZ).contains(&c) { harsh += share; }
        if (SIBILANT_LO_HZ..SIBILANT_HI_HZ).contains(&c) { sib += share; }
    }
    (harsh, sib)
}
```

`band_center_hz` and the 31-band edge table are defined in Task 6 (the long pass); reference it here. If implementing Task 5 before Task 6, add a temporary `pub(crate) fn band_center_hz(i: usize) -> f32 { 1000.0 }` stub and replace it in Task 6 — but prefer doing Task 6 first if order allows.

- [ ] **Step 4: Run to verify pass**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc deep_from_series`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/deep_analysis.rs
git commit -m "feat(deep-analysis): assemble DeepAnalysis (strata + harsh/sibilant + PSR-ready)"
```

---

## Task 6: 31-band parallel accumulation in the long pass

**Files:**
- Modify: `src-tauri/src/analysis.rs` (add a sibling that returns the 31-band curve; the 6-band fn stays byte-exact)
- Modify: `src-tauri/src/deep_analysis.rs` (real `band_center_hz`)

- [ ] **Step 1: Write the failing test**

In `analysis.rs` tests:

```rust
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
```

- [ ] **Step 2: Run to verify failure**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc spectral_balance_31band`
Expected: FAIL (`compute_spectral_balance_31band` not found).

- [ ] **Step 3: Implement the 31-band sibling + `band_center_hz`**

Add `compute_spectral_balance_31band` to `analysis.rs` — same Welch windowing as `compute_spectral_balance_6band` (copy its window-sizing + FFT loop), but bucket into 31 one-third-octave bands using IEC nominal centers 25 Hz … 16 kHz. It is a SEPARATE function; **do not touch `compute_spectral_balance_6band`** (the golden test from Task 1 must stay green).

```rust
/// IEC 61260 nominal one-third-octave centers, 25 Hz … 16 kHz (31 bands).
pub(crate) const THIRD_OCTAVE_CENTERS: [f32; 31] = [
    25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0,
    400.0, 500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0,
    4000.0, 5000.0, 6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 20000.0, 0.0,
];
// (Indices 29/30 are headroom/padding to keep a fixed [f32;31]; centers beyond
//  Nyquist/16k contribute ~0. Edge = geometric mean of adjacent centers.)
```

In `deep_analysis.rs`, replace the `band_center_hz` stub:

```rust
pub fn band_center_hz(i: usize) -> f32 {
    *crate::analysis::THIRD_OCTAVE_CENTERS.get(i).unwrap_or(&0.0)
}
```

Concretely: **copy `compute_spectral_balance_6band` (analysis.rs:316–397) verbatim** into a new `pub(crate) fn compute_spectral_balance_31band(...) -> Option<[f32; 31]>`, then change exactly three things, leaving window sizing / Hann / Welch sliding accumulation / the `total <= 1e-12 → None` gate **identical**:
1. Replace `let mut bands = [0.0_f64; 6];` with `[0.0_f64; 31]`.
2. Replace the 6-edge `if/else if` band-assignment with a one-third-octave lookup: a bin at `freq` goes to band `i` when `freq` falls in `[edge(i), edge(i+1))`, where `edge(i)` is the geometric mean of adjacent `THIRD_OCTAVE_CENTERS` (i.e. `(C[i-1]*C[i]).sqrt()`); bins below `edge(0)` or above the top edge are skipped (`continue`), same as today's out-of-range bins.
3. Return `Some([... bands[0]/total ... bands[30]/total ...])` (or build the array in a loop).
Define a small `fn third_octave_band(freq: f32) -> Option<usize>` next to it for the edge lookup. Do **not** modify `compute_spectral_balance_6band`.

- [ ] **Step 4: Run to verify pass + golden still green**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc spectral_balance`
Expected: PASS — including `spectral_balance_6band_is_byte_exact_golden` (proves the 6-band is untouched).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/analysis.rs src-tauri/src/deep_analysis.rs
git commit -m "feat(deep-analysis): 31-band one-third-octave parallel pass (6-band untouched)"
```

---

## Task 7: Wire into `analyze_one` via a `serde(skip)` field on `AnalysisResult`

**Files:**
- Modify: `src-tauri/src/types.rs` (`AnalysisResult` gains the `serde(skip)` field)
- Modify: `src-tauri/src/analysis.rs` (`analyze_one` builds + attaches `DeepAnalysis`)
- Modify: all `AnalysisResult { .. }` literals in tests (add `deep_analysis: None,`)

- [ ] **Step 1: Add the field + test constructor (compile-driven)**

In `types.rs`, add to `AnalysisResult` (after `energy_density_score`):

```rust
    /// Tier-2 Phase A backend-internal deep analysis. NEVER serialized
    /// (`serde(skip)` → off the wire; defaults to None on deserialize). Moved
    /// into the profile store by `populate_profile_store`.
    #[serde(skip)]
    pub deep_analysis: Option<std::sync::Arc<crate::deep_analysis::DeepAnalysis>>,
```

The field defaults to `None` on deserialize (`serde(skip)`), but Rust struct
*literals* must still name it — so every `AnalysisResult { .. }` test fixture
needs `deep_analysis: None,` (handled in Step 2). No lib-level constructor is
added (it would duplicate the existing `stub_analysis_with` in `contracts.rs`);
the churn is a one-line addition per site.

- [ ] **Step 2: Run to see every broken literal**

Run: `cd src-tauri; cargo build --tests --target-dir target/codex-rc 2>&1 | rg "missing field .deep_analysis"`
Expected: a list of ~10 sites (`contracts.rs:45/65/...`, `album_render.rs:73`, `album.rs:537`, `fixture_matrix.rs:452`, `album_character_bias.rs:73`, `album_sample_rate.rs:57`, `album_arc_trace.rs:66`). Add `deep_analysis: None,` to each.

- [ ] **Step 3: Build the `DeepAnalysis` in `analyze_one` and attach it**

In `analysis.rs::analyze_one`, after the existing Phase A5 block (~line 104), add:

```rust
    // Tier-2 Phase A: dual-resolution deep analysis (additive; never on the wire).
    let deep_analysis = {
        let bands31 = compute_spectral_balance_31band(
            &pcm.samples, pcm.sample_rate, pcm.channels as usize,
        );
        let windows = crate::deep_analysis::scan_windows(
            &pcm.samples, pcm.sample_rate, pcm.channels as usize,
        );
        match bands31 {
            Some(bands) if !windows.is_empty() => Some(std::sync::Arc::new(
                crate::deep_analysis::DeepAnalysis::from_parts(bands, windows),
            )),
            _ => None, // too short / silent → DeepAnalysis absent (SourceProfile still derives)
        }
    };
```

Then add `deep_analysis,` to the `AnalysisResult { .. }` constructor (~line 154+).

- [ ] **Step 4: Verify it builds, all tests pass, golden still green**

Run: `cd src-tauri; cargo test --target-dir target/codex-rc`
Expected: PASS (incl. the golden + the new deep-analysis tests). Add a test asserting `analyze_one`/`analyze_tracks_core` yields `deep_analysis.is_some()` for a ≥1 s synthetic stereo wav, and `None` for a < SHORT_WINDOW clip.

- [ ] **Step 5: Frontend + iPhone contract checks (must be unchanged)**

Run: `npx tsc -b --pretty false` (PASS), `npm test` (PASS), `cd apps/iphone-native/rust; cargo check --all-targets` (PASS — the new field is `serde(skip)`, so the bridge is unaffected).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/types.rs src-tauri/src/analysis.rs src-tauri/tests src-tauri/src
git commit -m "feat(deep-analysis): produce DeepAnalysis in analyze_one (serde(skip) on AnalysisResult)"
```

---

## Task 8: Cache in the profile store (Arc map, both clear paths, lock order)

**Files:**
- Modify: `src-tauri/src/profile_store.rs`
- Modify: `src-tauri/src/engine.rs` (`populate_profile_store`)

- [ ] **Step 1: Write failing store tests**

In `profile_store.rs` tests:

```rust
// Concrete helper (no stubs): build a real DeepAnalysis from a synthesized sine.
fn make_test_deep() -> crate::deep_analysis::DeepAnalysis {
    let sr = 48_000_u32;
    let n = sr as usize * 2;
    let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
    let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
    let windows = crate::deep_analysis::scan_windows(&samples, sr, 1);
    crate::deep_analysis::DeepAnalysis::from_parts([1.0 / 31.0; 31], windows)
}

#[test]
fn deep_store_insert_get_and_prune() {
    let store = SourceProfileStore::default();
    let id = TrackId("t".into());
    assert!(store.get_deep(&id).is_none());
    store.insert_deep(id.clone(), Some(std::sync::Arc::new(make_test_deep())));
    assert!(store.get_deep(&id).is_some());
    // soft clear
    store.insert_deep(id.clone(), None);
    assert!(store.get_deep(&id).is_none());
    // prune clears deep too
    store.insert_deep(id.clone(), Some(std::sync::Arc::new(make_test_deep())));
    prune_failed_profiles(&store, std::slice::from_ref(&id), &[]);
    assert!(store.get_deep(&id).is_none());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cd src-tauri; cargo test --lib --target-dir target/codex-rc deep_store`
Expected: FAIL (`get_deep`/`insert_deep` not found).

- [ ] **Step 3: Implement the second map + methods + extend clear paths**

In `profile_store.rs`, add to `SourceProfileStore`:

```rust
    by_track_deep: Mutex<HashMap<TrackId, std::sync::Arc<crate::deep_analysis::DeepAnalysis>>>,
```

Methods (fixed lock order: SourceProfile map first, never hold both at once):

```rust
    pub fn insert_deep(&self, track_id: TrackId, deep: Option<std::sync::Arc<crate::deep_analysis::DeepAnalysis>>) {
        if let Ok(mut g) = self.by_track_deep.lock() {
            match deep { Some(d) => { g.insert(track_id, d); }, None => { g.remove(&track_id); } }
        }
    }
    pub fn get_deep(&self, track_id: &TrackId) -> Option<std::sync::Arc<crate::deep_analysis::DeepAnalysis>> {
        self.by_track_deep.lock().ok().and_then(|g| g.get(track_id).cloned())
    }
```

Extend `prune_failed_profiles` to also `store.insert_deep(id.clone(), None)` for pruned ids (never holding both locks — call the two `insert`/`insert_deep` sequentially).

- [ ] **Step 4: Move DeepAnalysis into the store in `populate_profile_store`**

In `engine.rs::populate_profile_store`, alongside the existing `set(...)`:

```rust
    for result in results {
        profile_store.set(result.track_id.clone(), SourceProfile::from_analysis(result));
        profile_store.insert_deep(result.track_id.clone(), result.deep_analysis.clone());
    }
```

(The soft `set(_, None)` path: when `from_analysis` is None, also `insert_deep(id, None)` — `result.deep_analysis` is already None there, so a single line covers it.)

- [ ] **Step 5: Run all tests**

Run: `cd src-tauri; cargo test --target-dir target/codex-rc` — Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/profile_store.rs src-tauri/src/engine.rs
git commit -m "feat(deep-analysis): Arc-backed DeepAnalysis store map; both clear paths, fixed lock order"
```

---

## Task 9: Gate the deep scan off the iPhone FFI path (DECIDE default)

**Files:**
- Modify: `src-tauri/src/analysis.rs` (a flag param or a separate entry), `apps/iphone-native/rust/src/lib.rs`

- [ ] **Step 1: Gate design (decided): a private `deep: bool` on `analyze_one`, wrapped by two public entry points**

To keep `analyze_tracks_core`'s signature stable (the iPhone bridge calls it; `contracts.rs:136` pins it — the recurring cross-crate lesson), do NOT add a param to `analyze_tracks_core`. Instead:
- Give the private `analyze_one` a `deep: bool` parameter; wrap the Task-7 deep block in `if deep { ... } else { None }`.
- Keep `analyze_tracks_core(tracks)` unchanged — it calls `analyze_one(.., deep = true)`.
- Add `analyze_tracks_core_lite(tracks)` (same body, `deep = false`) for the mobile bridge.

- [ ] **Step 2: Implement**

Add the `deep: bool` param to `analyze_one`; update its internal callers. `analyze_tracks_core` passes `true`; new `analyze_tracks_core_lite` passes `false`. Existing desktop callers/tests and `analyze_tracks_core`'s signature are unchanged.

- [ ] **Step 3: Point the iPhone bridge at the lite path**

In `apps/iphone-native/rust/src/lib.rs`, change the `analyze_tracks_core(...)` call to `analyze_tracks_core_lite(...)`.

- [ ] **Step 4: Verify both crates**

Run: `cd src-tauri; cargo test --target-dir target/codex-rc` (PASS); `cd apps/iphone-native/rust; cargo check --all-targets` (PASS).

- [ ] **Step 5: Add the mobile benchmark**

In `apps/iphone-native/YESMasterNativeTests/` add a test that analyzes a bundled fixture and prints elapsed ms (records the real cost; informs whether the §8 cap ever fires). If no Swift CI here, add a `#[test]` in the bridge crate timing `analyze_tracks_core(deep=true)` on a synthetic buffer with `eprintln!` of elapsed ms.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/analysis.rs src-tauri/src/engine.rs apps/iphone-native/rust/src/lib.rs apps/iphone-native
git commit -m "feat(deep-analysis): gate deep scan off the mobile path; add cost benchmark"
```

---

## Task 10: Integration tests — short-clip regimes + stratification/temporal

**Files:**
- Test: `src-tauri/tests/contracts.rs` (or a new `src-tauri/tests/deep_analysis_integration.rs`)

- [ ] **Step 1: Write the regime + behavior tests**

```rust
#[tokio::test]
async fn deep_analysis_present_for_normal_track_absent_for_short_clip() {
    // normal: ≥1 s stereo sine → analyze_tracks_core yields deep_analysis.is_some()
    // short: < 16384 frames but ≥1024 → deep absent BUT SourceProfile still derivable
    //        (assert from_analysis(&result).is_some() and result.deep_analysis.is_none())
    // tiny: < 1024 frames → both absent
}

#[test]
fn stratification_loud_vs_body_and_ordered_series_distinguishes_sustained_vs_scattered() {
    // (a) bright LOUD section + dark body → DeepAnalysis.brightness.loud >> .body
    // (b) sustained-bright vs scattered-bright with SAME loudness multiset →
    //     the retained `windows` series differs (assert the ordered high-share
    //     sequences are not equal) even though strata/dispersion match.
}
```

Implement using `engine::analyze_tracks_core` on synthesized wavs (reuse `write_sine_wav` / `write_sine_wav_at_amplitude` helpers already in `contracts.rs`); for the bright-loud/dark-body fixture, concatenate a loud high-frequency segment + a quiet low-frequency body.

- [ ] **Step 2: Run to verify failure, then implement fixtures until green**

Run: `cd src-tauri; cargo test --target-dir target/codex-rc deep_analysis`
Expected: FAIL → build the fixtures → PASS.

- [ ] **Step 3: Full verification sweep**

Run, all expected green:
- `cd src-tauri; cargo test --target-dir target/codex-rc`
- `cd src-tauri; cargo clippy --all-targets --target-dir target/codex-rc` (0 warnings)
- `npx tsc -b --pretty false`; `npm test`
- `cd apps/iphone-native/rust; cargo check --all-targets`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tests
git commit -m "test(deep-analysis): short-clip regimes + stratification/temporal integration"
```

---

## Acceptance (maps to spec §13)

- `DeepAnalysis` produced once per analyze, cached behind `Arc`, holding the ordered per-window series, 31-band curve, loud/body/whole strata, IQR dispersion (Fisher-z for correlation), per-window sample peak + loudness key (momentary-PSR-ready).
- `body`, dispersion, loudness key defined by constant.
- 6-band byte-exact (golden test); all existing tests green; both store maps invalidate in lockstep on both clear paths; lock order fixed.
- Wire contract unchanged (`serde(skip)`); iPhone bridge compiles; mobile scan gated off + benchmark recorded; `MAX_SCAN_WINDOWS` enforced.

## Notes for the executor

- Always use `--target-dir target/codex-rc` for cargo if the desktop app might be running.
- Keep `compute_spectral_balance_6band` untouched — the golden test (Task 1) is the tripwire.
- `DeepAnalysis` and its fields are backend-only; do NOT add anything to `bindings.ts`.
- If a task's exact numeric helper (percentile/IQR) needs a different interpolation to satisfy a test, match the test — the tests are the behavioral contract.
