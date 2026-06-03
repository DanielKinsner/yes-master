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
