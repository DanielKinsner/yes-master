//! Tier-2 Phase B — confidence / coverage gating (anti-homogenization core).
//!
//! "A measures, B decides": Phase A produced a [`DeepAnalysis`] (a per-window
//! series plus loudness-stratified aggregates). This module turns that into a per-axis
//! **confidence** in `[0, 1]` for the four Tier-1 guardrail axes (brightness,
//! low/boom, density, width). Phase B (guardrails) scales each defensive trim by
//! its axis confidence — **reduce-only** — so the engine acts strongly only where
//! a source quality is **broad** (high coverage) AND **consistent** (low
//! dispersion), and stands off on neutral / ambiguous / scattered sources. That is
//! the direct fix for preset homogenization: neutral material gets small trims, so
//! presets stay distinct.
//!
//! Each axis pairs **coverage** (the fraction of loudness-finite windows that
//! exhibit the trait) with **consistency** (`1 - dispersion/scale`, IQR-based — or
//! Fisher-z IQR for the bounded correlation axis — clamped to `[0, 1]`) into
//! `confidence = coverage * consistency`; both must be high for the engine to act.
//!
//! **Byte-identity:** [`Confidence::full`] (all `1.0`) reproduces Tier-1 exactly,
//! so a source with no `DeepAnalysis` (short clip / mobile) is unaffected.
//!
//! ⚠️ **ALL CONSTANTS HERE ARE PROVISIONAL.** They have no audible effect until the
//! guardrail application AND a non-zero Adapt Strength; the owner locks them by ear
//! (handoff §7.6 / §10, slow-fixture lane). The single source of truth for Phase B
//! tuning lives here.

use crate::deep_analysis::{iqr, DeepAnalysis};

// --- Provisional coverage thresholds (per-window "is the trait present?") ---
/// A window counts as **bright** when its 3-band `high` share exceeds this
/// (a flat/neutral window sits near 0.33). Provisional.
const BRIGHT_WINDOW_SHARE: f32 = 0.40;
/// A window counts as **boomy** when its 3-band `low` share exceeds this. Provisional.
const LOW_WINDOW_SHARE: f32 = 0.40;
/// A window counts as **dense** when its crest (peak/RMS, linear) is BELOW this
/// (a sine sits ~1.41; limited masters trend lower). Provisional.
const CREST_DENSE: f32 = 2.2;
/// A window counts as **wide** when its L/R correlation is below the Tier-1 width
/// deadband (reused so per-window and whole-track agree on "wide").
const WIDTH_CORR_WIDE: f32 = crate::guardrails::WIDTH_CORR_DEADBAND;

// --- Provisional dispersion scales (the IQR at which consistency reaches 0) ---
const BRIGHT_DISP_FULL: f32 = 0.10;
const LOW_DISP_FULL: f32 = 0.10;
const CREST_DISP_FULL: f32 = 1.2;
/// Fisher-z IQR scale for the bounded correlation axis.
const WIDTH_DISP_FULL: f32 = 0.6;

/// Coverage + consistency + their product for one adaptive axis. `confidence` is
/// what the guardrails consume; `coverage`/`consistency` are retained for the
/// future "what we found" readout (Phase C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisConfidence {
    /// Fraction of loudness-finite windows exhibiting the trait, `[0, 1]`.
    pub coverage: f32,
    /// `1 - dispersion/scale`, clamped `[0, 1]`. High = the trait is a stable
    /// property; low = scattered.
    pub consistency: f32,
    /// `coverage * consistency` — the reduce-only trim multiplier the guardrails apply.
    pub confidence: f32,
}

impl AxisConfidence {
    /// Identity: act at full Tier-1 strength (used when no DeepAnalysis is present).
    pub const FULL: Self = Self {
        coverage: 1.0,
        consistency: 1.0,
        confidence: 1.0,
    };
}

/// Per-axis confidence for the four Tier-1 guardrail axes, derived from a
/// `DeepAnalysis`. `full()` is the byte-identity default (no deep read).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Confidence {
    pub bright: AxisConfidence,
    pub low: AxisConfidence,
    pub density: AxisConfidence,
    pub width: AxisConfidence,
}

impl Default for Confidence {
    fn default() -> Self {
        Self::full()
    }
}

impl Confidence {
    /// All axes at full confidence — reproduces Tier-1 exactly (byte-identity).
    pub fn full() -> Self {
        Self {
            bright: AxisConfidence::FULL,
            low: AxisConfidence::FULL,
            density: AxisConfidence::FULL,
            width: AxisConfidence::FULL,
        }
    }

    /// Derive per-axis confidence from the Phase A deep analysis. Coverage is the
    /// fraction of loudness-finite windows exhibiting each trait; consistency comes
    /// from the per-axis dispersion (the stored strata IQR where Phase A retained
    /// it; a fresh IQR over the per-window `low` share for the boom axis, which has
    /// no stored stratum). Width uses the Fisher-z dispersion (bounded stat).
    pub fn from_deep(d: &DeepAnalysis) -> Self {
        let keys: Vec<f32> = d.windows.iter().map(|w| w.loudness_key).collect();
        let highs: Vec<f32> = d.windows.iter().map(|w| w.high).collect();
        let lows: Vec<f32> = d.windows.iter().map(|w| w.low).collect();
        let crests: Vec<f32> = d.windows.iter().map(|w| w.crest).collect();
        let corrs: Vec<f32> = d.windows.iter().map(|w| w.stereo_correlation).collect();
        Self {
            // brightness/density/width reuse the Phase A strata dispersion (IQR);
            // the boom axis has no stored stratum, so its IQR is computed here.
            bright: axis_confidence(
                &highs,
                &keys,
                |v| v > BRIGHT_WINDOW_SHARE,
                d.brightness.dispersion,
                BRIGHT_DISP_FULL,
            ),
            low: axis_confidence(
                &lows,
                &keys,
                |v| v > LOW_WINDOW_SHARE,
                iqr(&lows),
                LOW_DISP_FULL,
            ),
            density: axis_confidence(
                &crests,
                &keys,
                |v| v < CREST_DENSE,
                d.crest.dispersion,
                CREST_DISP_FULL,
            ),
            width: axis_confidence(
                &corrs,
                &keys,
                |v| v.is_finite() && v < WIDTH_CORR_WIDE,
                d.stereo_correlation.dispersion, // Fisher-z IQR (set in from_parts)
                WIDTH_DISP_FULL,
            ),
        }
    }
}

/// Coverage (fraction of loudness-finite windows passing `trait_present`) and
/// consistency (`1 - dispersion/dispersion_full`, clamped) for one axis.
fn axis_confidence(
    vals: &[f32],
    keys: &[f32],
    trait_present: impl Fn(f32) -> bool,
    dispersion: f32,
    dispersion_full: f32,
) -> AxisConfidence {
    let mut total = 0usize;
    let mut hits = 0usize;
    for (v, k) in vals.iter().zip(keys.iter()) {
        if !k.is_finite() {
            continue; // silent / non-finite window — excluded, matching the strata.
        }
        total += 1;
        if trait_present(*v) {
            hits += 1;
        }
    }
    let coverage = if total == 0 {
        0.0
    } else {
        hits as f32 / total as f32
    };
    let consistency = if dispersion_full <= 0.0 {
        1.0
    } else {
        (1.0 - dispersion / dispersion_full).clamp(0.0, 1.0)
    };
    AxisConfidence {
        coverage,
        consistency,
        confidence: coverage * consistency,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_analysis::{scan_windows, DeepAnalysis};

    /// Build a DeepAnalysis from a mono sine at `freq` Hz (steady → low dispersion).
    fn deep_from_tone(freq: f32, channels: usize) -> DeepAnalysis {
        let sr = 48_000_u32;
        let n = sr as usize * 3;
        let omega = 2.0 * std::f32::consts::PI * freq / sr as f32;
        let mono: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let samples: Vec<f32> = if channels > 1 {
            mono.iter().flat_map(|&s| [s, s]).collect()
        } else {
            mono
        };
        let windows = scan_windows(&samples, sr, channels);
        DeepAnalysis::from_parts([1.0 / 31.0; 31], windows)
    }

    #[test]
    fn full_confidence_is_identity() {
        let f = Confidence::full();
        for a in [f.bright, f.low, f.density, f.width] {
            assert_eq!(a.confidence, 1.0);
            assert_eq!(a.coverage, 1.0);
            assert_eq!(a.consistency, 1.0);
        }
        assert_eq!(Confidence::default(), Confidence::full());
    }

    #[test]
    fn broad_consistent_bright_source_has_high_brightness_confidence() {
        // A sustained HF-dominant tone reads "bright" in every window with near-zero
        // dispersion -> high coverage AND high consistency.
        let c = Confidence::from_deep(&deep_from_tone(12_000.0, 1));
        assert!(c.bright.coverage > 0.9, "coverage {:?}", c.bright);
        assert!(c.bright.consistency > 0.8, "consistency {:?}", c.bright);
        assert!(c.bright.confidence > 0.7, "confidence {:?}", c.bright);
    }

    #[test]
    fn low_dominant_source_has_low_brightness_but_high_boom_confidence() {
        // A sustained LF tone: windows are boomy, not bright.
        let c = Confidence::from_deep(&deep_from_tone(80.0, 1));
        assert!(c.bright.coverage < 0.1, "bright coverage {:?}", c.bright);
        assert!(c.bright.confidence < 0.2, "bright confidence {:?}", c.bright);
        assert!(c.low.coverage > 0.9, "boom coverage {:?}", c.low);
        assert!(c.low.confidence > 0.7, "boom confidence {:?}", c.low);
    }

    #[test]
    fn mono_source_has_zero_width_confidence() {
        // Mono windows carry NaN correlation -> never "wide" -> zero coverage, so
        // width confidence is 0 and the width trim is never scaled up by Phase B.
        let c = Confidence::from_deep(&deep_from_tone(1_000.0, 1));
        assert_eq!(c.width.coverage, 0.0, "{:?}", c.width);
        assert_eq!(c.width.confidence, 0.0, "{:?}", c.width);
    }

    #[test]
    fn confidence_is_bounded_unit_interval() {
        let c = Confidence::from_deep(&deep_from_tone(3_000.0, 2));
        for a in [c.bright, c.low, c.density, c.width] {
            assert!((0.0..=1.0).contains(&a.coverage), "{a:?}");
            assert!((0.0..=1.0).contains(&a.consistency), "{a:?}");
            assert!((0.0..=1.0).contains(&a.confidence), "{a:?}");
        }
    }
}
