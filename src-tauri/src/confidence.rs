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

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::deep_analysis::{fisher_z_iqr, iqr, DeepAnalysis};

/// **Owner-calibration gate (master switch for Phase B §7.2), runtime-toggleable.**
/// Off by default: while disabled the chain resolves confidence to `None`
/// (→ `full()` → byte-identical Tier-1), so the provisional gating below has **no
/// audible effect**. Enable it at runtime — without a rebuild — via
/// [`set_confidence_gating_enabled`] (the `set_confidence_gating` Tauri command) or
/// the `YES_MASTER_CONFIDENCE_GATING` env seed ([`init_confidence_gating_from_env`]),
/// so the owner can A/B-calibrate and the gate-ON path stays testable. Enable it only
/// after a by-ear A/B has locked the constants in this file (handoff §7.6 / §10).
static CONFIDENCE_GATING: AtomicBool = AtomicBool::new(false);

/// Whether Phase B confidence gating is currently enabled (default `false`).
pub fn is_confidence_gating_enabled() -> bool {
    CONFIDENCE_GATING.load(Ordering::Relaxed)
}

/// Enable/disable Phase B confidence gating at runtime; returns the previous value.
pub fn set_confidence_gating_enabled(enabled: bool) -> bool {
    CONFIDENCE_GATING.swap(enabled, Ordering::Relaxed)
}

/// Seed the gate from the `YES_MASTER_CONFIDENCE_GATING` env var (`1`/`true`/`on`/
/// `yes` => enabled). Call once at startup so headless / fixture / dev runs can
/// enable Phase B without a UI; an unset var leaves the default (off).
pub fn init_confidence_gating_from_env() {
    if let Ok(v) = std::env::var("YES_MASTER_CONFIDENCE_GATING") {
        let on = matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "on" | "yes"
        );
        set_confidence_gating_enabled(on);
    }
}

/// Tauri command: enable/disable Phase B confidence gating at runtime (owner A/B
/// calibration — no rebuild). Returns the previous value.
#[tauri::command]
pub fn set_confidence_gating(enabled: bool) -> bool {
    set_confidence_gating_enabled(enabled)
}

/// Tauri command: read whether Phase B confidence gating is currently enabled.
#[tauri::command]
pub fn confidence_gating_enabled() -> bool {
    is_confidence_gating_enabled()
}

/// Resolve the per-axis confidence for one chain build, honoring the album gate and
/// an explicit `gating_enabled` flag. `None` => `full()` => byte-identical Tier-1.
/// Pure (the flag is a parameter) so both gate states are deterministically testable;
/// the app passes [`is_confidence_gating_enabled`].
pub fn resolve_source_confidence(
    deep: Option<&DeepAnalysis>,
    album: bool,
    gating_enabled: bool,
) -> Option<Confidence> {
    if album || !gating_enabled {
        return None;
    }
    deep.map(Confidence::from_deep)
}

// --- Provisional coverage thresholds (per-window "is the trait present?") ---
/// A window counts as **bright/problematic** when its 31-band harsh+sibilant
/// share exceeds this. Air-only sheen is deliberately not the same as harshness.
/// Provisional.
const BRIGHT_PROBLEM_WINDOW_SHARE: f32 = 0.35;
/// A window counts as **boomy** when its sample-rate-aware 31-band low share
/// exceeds this. Provisional.
const LOW_31_WINDOW_SHARE: f32 = 0.35;
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

    /// One-line per-axis confidence summary for the export receipt's "what Phase B
    /// confidence shaped this master" line. Each value is the reduce-only trim
    /// multiplier in `[0, 1]` (1.00 = full Tier-1 trim, lower = held back).
    pub fn digest(&self) -> String {
        format!(
            "bright {:.2} / low {:.2} / density {:.2} / width {:.2}",
            self.bright.confidence,
            self.low.confidence,
            self.density.confidence,
            self.width.confidence,
        )
    }

    /// Derive per-axis confidence from the Phase A deep analysis. Bright and low
    /// now consume the per-window 31-band detail (harsh/sibilant/tilt/low) instead
    /// of the old approximate 3-band helper, so Phase B can back off airy-but-not-
    /// harsh material while still acting on broad harshness or sibilance. Width
    /// uses the Fisher-z dispersion (bounded stat).
    pub fn from_deep(d: &DeepAnalysis) -> Self {
        let keys: Vec<f32> = d.windows.iter().map(|w| w.loudness_key).collect();
        let bright_problem: Vec<f32> = d.windows.iter().map(bright_problem_share).collect();
        let lows_31: Vec<f32> = d.windows.iter().map(|w| w.low_31).collect();
        let crests: Vec<f32> = d.windows.iter().map(|w| w.crest).collect();
        let corrs: Vec<f32> = d.windows.iter().map(|w| w.stereo_correlation).collect();
        let bright_problem_keyed = keyed_finite_values(&d.windows, bright_problem_share);
        let lows_keyed = keyed_finite_values(&d.windows, |w| w.low_31);
        Self {
            // Brightness/low now use 31-band per-window detail. Density/width
            // reuse the Phase A strata or Fisher-z dispersion.
            bright: axis_confidence(
                &bright_problem,
                &keys,
                |v| v > BRIGHT_PROBLEM_WINDOW_SHARE,
                iqr(&bright_problem_keyed),
                BRIGHT_DISP_FULL,
            ),
            low: axis_confidence(
                &lows_31,
                &keys,
                |v| v > LOW_31_WINDOW_SHARE,
                iqr(&lows_keyed),
                LOW_DISP_FULL,
            ),
            density: axis_confidence(
                &crests,
                &keys,
                |v| v < CREST_DENSE,
                d.crest.dispersion,
                CREST_DISP_FULL,
            ),
            // Width gates on STEREO CONTENT (finite correlation), not the mono
            // loudness key: a side-heavy / anti-phase window can be ~silent in the
            // mono downmix yet maximally wide, and must still count toward width
            // coverage (else Phase B under-trims widening on the phasiest material).
            // Dispersion = Fisher-z IQR over the same correlation-finite population.
            width: axis_confidence(
                &corrs,
                &corrs,
                |v| v.is_finite() && v < WIDTH_CORR_WIDE,
                fisher_z_iqr(&corrs),
                WIDTH_DISP_FULL,
            ),
        }
    }
}

fn bright_problem_share(w: &crate::deep_analysis::WindowMetrics) -> f32 {
    // 31-band harsh/sibilant detail is the primary trigger; spectral tilt is a
    // gentle sanity check so a low-heavy window with a little upper energy is not
    // treated the same as a genuinely upper-tilted harsh/sibilant window.
    let tilt_weight = ((w.tilt_31 + 1.0) * 0.5).clamp(0.25, 1.0);
    (w.harsh_31 + w.sibilant_31).clamp(0.0, 1.0) * tilt_weight
}

fn keyed_finite_values(
    windows: &[crate::deep_analysis::WindowMetrics],
    value: impl Fn(&crate::deep_analysis::WindowMetrics) -> f32,
) -> Vec<f32> {
    windows
        .iter()
        .filter(|w| w.loudness_key.is_finite())
        .map(value)
        .filter(|v| v.is_finite())
        .collect()
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
    fn resolve_source_confidence_respects_gate_and_album() {
        let deep = deep_from_tone(1_000.0, 2);
        // Gate OFF -> None even with a deep read present (byte-identical Tier-1).
        assert!(resolve_source_confidence(Some(&deep), false, false).is_none());
        // Gate ON + deep present -> Some. This is the gate-ON path made testable by
        // the runtime flag (the flag is an explicit parameter, no global mutation).
        assert!(resolve_source_confidence(Some(&deep), false, true).is_some());
        // Album is never adaptive -> None even with the gate on.
        assert!(resolve_source_confidence(Some(&deep), true, true).is_none());
        // Gate on but no deep read -> None.
        assert!(resolve_source_confidence(None, false, true).is_none());
        // NOTE: the default-OFF pin for the runtime gate lives in
        // tests/owner_gates_default.rs (pristine process). Asserting it here
        // would race: reference_tuning/fixture_matrix/profile_store tests flip
        // this global under ADAPTIVE_COMPRESSION_GATE_TEST_LOCK, which this
        // test does not hold.
    }

    #[test]
    fn confidence_digest_is_compact_per_axis() {
        assert_eq!(
            Confidence::full().digest(),
            "bright 1.00 / low 1.00 / density 1.00 / width 1.00"
        );
    }

    #[test]
    fn broad_consistent_harsh_source_has_high_brightness_confidence() {
        // A sustained 3 kHz tone lives in the 31-band harsh range in every
        // window, so Phase B should let the brightness trim act.
        let c = Confidence::from_deep(&deep_from_tone(3_000.0, 1));
        assert!(c.bright.coverage > 0.9, "coverage {:?}", c.bright);
        assert!(c.bright.consistency > 0.8, "consistency {:?}", c.bright);
        assert!(c.bright.confidence > 0.7, "confidence {:?}", c.bright);
    }

    #[test]
    fn broad_consistent_sibilant_source_has_high_brightness_confidence() {
        // Sibilance is distinct from harshness internally, but both are concrete
        // 31-band reasons to allow the Tier-1 brightness trim.
        let c = Confidence::from_deep(&deep_from_tone(7_000.0, 1));
        assert!(c.bright.coverage > 0.9, "coverage {:?}", c.bright);
        assert!(c.bright.confidence > 0.7, "confidence {:?}", c.bright);
    }

    #[test]
    fn airy_not_harsh_source_backs_off_brightness_confidence() {
        // The old coarse high-band trigger could treat a shiny 12 kHz air tone as
        // "bright" and trim preset air globally. The 31-band Phase-B input keeps
        // air-only material from looking like harshness or sibilance.
        let c = Confidence::from_deep(&deep_from_tone(12_000.0, 1));
        assert!(
            c.bright.coverage < 0.2,
            "air-only source should not look harsh/sibilant: {:?}",
            c.bright
        );
        assert!(
            c.bright.confidence < 0.2,
            "air-only source should back off brightness trim: {:?}",
            c.bright
        );
    }

    #[test]
    fn low_dominant_source_has_low_brightness_but_high_boom_confidence() {
        // A sustained LF tone: windows are boomy, not bright.
        let c = Confidence::from_deep(&deep_from_tone(80.0, 1));
        assert!(c.bright.coverage < 0.1, "bright coverage {:?}", c.bright);
        assert!(
            c.bright.confidence < 0.2,
            "bright confidence {:?}",
            c.bright
        );
        assert!(c.low.coverage > 0.9, "boom coverage {:?}", c.low);
        assert!(c.low.confidence > 0.7, "boom confidence {:?}", c.low);
    }

    #[test]
    fn low_confidence_consistency_ignores_silent_tail() {
        // A boomy track with a silent tail must not have its low CONSISTENCY diluted
        // by the silent windows (which are already excluded from coverage). Low
        // dispersion must use the same loudness-finite window population as coverage
        // (Codex review: coverage and consistency were calibrated off different sets).
        let sr = 48_000_u32;
        let omega = 2.0 * std::f32::consts::PI * 80.0 / sr as f32;
        let boomy: Vec<f32> = (0..(sr as usize * 2))
            .map(|i| 0.3 * (omega * i as f32).sin())
            .collect();
        let mut with_tail = boomy.clone();
        with_tail.extend(std::iter::repeat(0.0).take(sr as usize * 2)); // + 2 s silence

        let c_boomy = Confidence::from_deep(&DeepAnalysis::from_parts(
            [1.0 / 31.0; 31],
            scan_windows(&boomy, sr, 1),
        ));
        let c_tail = Confidence::from_deep(&DeepAnalysis::from_parts(
            [1.0 / 31.0; 31],
            scan_windows(&with_tail, sr, 1),
        ));
        assert!(
            (c_tail.low.consistency - c_boomy.low.consistency).abs() < 0.1,
            "silent tail diluted low consistency: boomy {} vs tail {}",
            c_boomy.low.consistency,
            c_tail.low.consistency,
        );
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
    fn anti_phase_stereo_keeps_width_confidence_nonzero() {
        // L = -R: the mono downmix is ~silent (loudness_key NEG_INF) but the stereo
        // field is maximally wide (correlation ~ -1). Width coverage must gate on
        // stereo content (finite correlation), NOT the mono loudness key — otherwise
        // Phase B under-trims widening on exactly the phasiest material (Codex review).
        let sr = 48_000_u32;
        let n = sr as usize * 3;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n)
            .flat_map(|i| {
                let s = 0.3 * (omega * i as f32).sin();
                [s, -s] // anti-phase L/R
            })
            .collect();
        let windows = scan_windows(&samples, sr, 2);
        assert!(!windows.is_empty());
        // sanity: mono is silent (loudness keys non-finite) but correlation is wide.
        assert!(windows.iter().all(|w| !w.loudness_key.is_finite()));
        assert!(windows
            .iter()
            .any(|w| w.stereo_correlation.is_finite() && w.stereo_correlation < 0.0));
        let c = Confidence::from_deep(&DeepAnalysis::from_parts([1.0 / 31.0; 31], windows));
        assert!(
            c.width.coverage > 0.5,
            "width coverage collapsed: {:?}",
            c.width
        );
        assert!(
            c.width.confidence > 0.5,
            "width confidence collapsed: {:?}",
            c.width
        );
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
