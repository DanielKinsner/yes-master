//! Tier-1 adaptive guardrails — defensive, analysis-driven trimming of preset
//! moves. See `docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md`.
//!
//! Principle: when a source ALREADY has a quality (bright / boomy / dense /
//! wide), trim the matching preset move toward neutral. Defensive means we only
//! ever REDUCE a positive preset move — we never add a boost, never flip a
//! sign, never touch a preset cut, never narrow a source. Per-axis caps plus a
//! character floor keep presets recognizable at any strength; one `strength`
//! value scales every trim.
//!
//! All triggers read level-invariant shares/ratios from [`SourceProfile`], so no
//! LUFS normalization is needed before comparison.

use crate::confidence::Confidence;
use crate::types::{MasteringSettings, SourceProfile, TrackId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Default Adapt Strength when the user hasn't set one (on by default). Owner
/// starting point (2026-06-03 listening): a moderate **0.5** — kept conservative
/// so adaptation doesn't homogenize presets (a louder default narrows the spread
/// between presets on a given source) and to under-act rather than over-neuter
/// while the deeper section-aware analysis / by-ear calibration lands.
pub const ADAPTIVE_STRENGTH_DEFAULT: f32 = 0.5;

// ---------------------------------------------------------------------------
// Neutral references & deadbands — PROVISIONAL, calibrate by ear.
//
// Mapped onto YES Master's own 6 analysis bands (sub 20-80, low 80-250,
// low_mid 250-800, mid 800-2500, presence 2500-6500, air 6500-16k Hz). Per the
// spec's "false precision" caveat these are interpretive starting points, kept
// deliberately wide/conservative so an imperfect default UNDER-acts rather than
// misfires. The single source of truth for tuning lives here.
// ---------------------------------------------------------------------------

/// Brightness trigger = presence + air share. No trim at/below this share.
/// Set ABOVE the natural pink-tilt share (presence+air ~= 0.278 for a 1/f
/// spectrum across our band edges) so a genuinely neutral master reads as zero
/// excess and keeps its air. Was 0.20, which over-trimmed neutral masters by
/// ~39% at default strength (reviews 2026-06-02/03). Tilt-vs-reference is the
/// planned principled replacement; see the finish plan.
const BRIGHT_DEADBAND: f32 = 0.30;
/// Brightness share above the deadband that maps to full (pre-cap) trim.
const BRIGHT_EXCESS_FULL: f32 = 0.12;

/// Boominess trigger = sub + low share. Wider deadband (documented bass variance).
const LOW_DEADBAND: f32 = 0.42;
/// Boominess share above the deadband that maps to full (pre-cap) trim.
const LOW_EXCESS_FULL: f32 = 0.15;

/// Density (P95-P10 dynamic range, dB): trim begins / reaches full.
const DENSITY_DR_SOFT_DB: f32 = 8.0;
const DENSITY_DR_FULL_DB: f32 = 3.0;
/// Density (LRA, LU): trim begins / reaches full (secondary signal).
const DENSITY_LRA_SOFT_LU: f32 = 6.0;
const DENSITY_LRA_FULL_LU: f32 = 3.0;

// Adaptive Compressor MVP already-mastered classifier. TBD-CALIBRATION:
// provisional runner-derived inputs only; owner locks by listening in AC-5.
const ALREADY_MASTERED_HOT_LUFS: f32 = -10.0; // TBD-CALIBRATION
const ALREADY_MASTERED_TRUE_PEAK_DBBTP: f32 = -1.2; // TBD-CALIBRATION
const ALREADY_MASTERED_LRA_LU: f32 = 6.0; // TBD-CALIBRATION
const ALREADY_MASTERED_BAND_PSR_DB: f32 = 8.0; // TBD-CALIBRATION
const BAND_PSR_SOFT_DB: f32 = 12.0; // TBD-CALIBRATION
const BAND_PSR_FULL_DB: f32 = 8.0; // TBD-CALIBRATION
const BAND_COMPRESSION_DENSITY_CAP: f32 = 0.45; // TBD-CALIBRATION
const BAND_THRESHOLD_LIFT_MAX_DB: f32 = 4.0; // TBD-CALIBRATION
const BAND_RATIO_EASE_CAP: f32 = 0.35; // TBD-CALIBRATION

/// Owner-calibration gate for the Adaptive Compressor MVP. Off by default so
/// every committed AC slice remains byte-identical until the listening session
/// explicitly flips it.
static ADAPTIVE_COMPRESSION: AtomicBool = AtomicBool::new(false);

pub fn is_adaptive_compression_enabled() -> bool {
    ADAPTIVE_COMPRESSION.load(Ordering::Relaxed)
}

pub fn set_adaptive_compression_enabled(enabled: bool) -> bool {
    ADAPTIVE_COMPRESSION.swap(enabled, Ordering::Relaxed)
}

pub fn init_adaptive_compression_from_env() {
    if let Ok(v) = std::env::var("YES_MASTER_ADAPTIVE_COMPRESSION") {
        let on = matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "on" | "yes"
        );
        set_adaptive_compression_enabled(on);
    }
}

#[tauri::command]
pub fn set_adaptive_compression(enabled: bool) -> bool {
    set_adaptive_compression_enabled(enabled)
}

#[tauri::command]
pub fn adaptive_compression_enabled() -> bool {
    is_adaptive_compression_enabled()
}

/// Width (L/R correlation): deadband edge / full-trim floor. Lower correlation
/// = wider, phasier source. Mono (`None`) never trims width.
pub(crate) const WIDTH_CORR_DEADBAND: f32 = 0.50;
const WIDTH_CORR_FULL: f32 = 0.20;

// Per-axis maximum trim caps (fraction of the preset move removed), applied
// AFTER strength scaling and independent of it — this is what guarantees a
// preset stays recognizable even at full strength on a strongly-matched source.
const EQ_CAP: f32 = 0.50;
const DENSITY_CAP: f32 = 0.60;
const WIDTH_CAP: f32 = 0.70;

/// Positive EQ boosts are never trimmed below this many dB, so a preset's tonal
/// character survives. (A boost already smaller than the floor is left as-is.)
const EQ_BOOST_FLOOR_DB: f32 = 0.5;

#[inline]
fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

/// Ramp for a "smaller value = more excess" trigger (dynamics, correlation):
/// 0 at/above `soft`, rising to 1 at/below `full`.
#[inline]
fn descending_ramp(value: f32, soft: f32, full: f32) -> f32 {
    if (soft - full).abs() < f32::EPSILON {
        return if value <= full { 1.0 } else { 0.0 };
    }
    clamp01((soft - value) / (soft - full))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlreadyMasteredStandDown {
    pub stand_down: f32,
    pub hot_loudness: bool,
    pub near_ceiling: bool,
    pub low_lra: bool,
    pub uniformly_low_psr: bool,
}

impl AlreadyMasteredStandDown {
    pub fn identity() -> Self {
        Self {
            stand_down: 0.0,
            hot_loudness: false,
            near_ceiling: false,
            low_lra: false,
            uniformly_low_psr: false,
        }
    }
}

/// Classify "already-mastered" inputs for the adaptive compressor stand-down.
/// Pure AC-1 plumbing: this does not alter the chain until AC-2 wires it behind
/// `YES_MASTER_ADAPTIVE_COMPRESSION`.
pub fn classify_already_mastered_stand_down(
    lufs_integrated: f32,
    true_peak_dbtp: f32,
    dynamic_range_lu: f32,
    band_psr: Option<crate::deep_analysis::BandPsrStats>,
) -> AlreadyMasteredStandDown {
    let hot_loudness = lufs_integrated.is_finite() && lufs_integrated >= ALREADY_MASTERED_HOT_LUFS;
    let near_ceiling =
        true_peak_dbtp.is_finite() && true_peak_dbtp >= ALREADY_MASTERED_TRUE_PEAK_DBBTP;
    let low_lra = dynamic_range_lu.is_finite()
        && dynamic_range_lu > 0.5
        && dynamic_range_lu <= ALREADY_MASTERED_LRA_LU;
    let uniformly_low_psr =
        band_psr.is_some_and(|psr| psr.all_bands_below(ALREADY_MASTERED_BAND_PSR_DB));
    let stand_down = if hot_loudness && near_ceiling && low_lra && uniformly_low_psr {
        1.0
    } else {
        0.0
    };

    AlreadyMasteredStandDown {
        stand_down,
        hot_loudness,
        near_ceiling,
        low_lra,
        uniformly_low_psr,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct BandCompressionGuard {
    pub density_mult: f32,
    pub threshold_lift_db: f32,
    pub ratio_mult: f32,
}

impl BandCompressionGuard {
    pub fn identity() -> Self {
        Self {
            density_mult: 1.0,
            threshold_lift_db: 0.0,
            ratio_mult: 1.0,
        }
    }

    fn from_amount(amount: f32) -> Self {
        let amount = clamp01(amount);
        if amount <= 1.0e-6 {
            return Self::identity();
        }
        Self {
            density_mult: 1.0 - amount * BAND_COMPRESSION_DENSITY_CAP,
            threshold_lift_db: amount * BAND_THRESHOLD_LIFT_MAX_DB,
            ratio_mult: 1.0 - amount * BAND_RATIO_EASE_CAP,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardReason {
    LowBandDense,
    MidBandDense,
    HighBandDense,
    AlreadyMastered,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompressionGuards {
    pub low: BandCompressionGuard,
    pub mid: BandCompressionGuard,
    pub high: BandCompressionGuard,
    pub stand_down: f32,
    pub reasons: Vec<GuardReason>,
}

impl CompressionGuards {
    pub fn identity() -> Self {
        Self {
            low: BandCompressionGuard::identity(),
            mid: BandCompressionGuard::identity(),
            high: BandCompressionGuard::identity(),
            stand_down: 0.0,
            reasons: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct CompressionBandPlan {
    pub threshold_db: f32,
    pub ratio: f32,
    pub density_mult: f32,
    pub threshold_lift_db: f32,
    pub ratio_mult: f32,
    pub adaptive: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompressionPlanReason {
    pub code: GuardReason,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompressionPlan {
    pub active: bool,
    pub low: CompressionBandPlan,
    pub mid: CompressionBandPlan,
    pub high: CompressionBandPlan,
    pub reasons: Vec<CompressionPlanReason>,
    pub guidance: Option<String>,
    pub digest: Option<String>,
}

/// Resolve per-band adaptive compressor guards from DeepAnalysis-derived PSR and
/// the already-mastered classifier. Pure AC-2 core: callers pass the rollout gate
/// explicitly so tests can prove gate-OFF identity without relying on process
/// globals.
pub fn resolve_compression_guards(
    band_psr: Option<crate::deep_analysis::BandPsrStats>,
    confidence: &Confidence,
    stand_down: AlreadyMasteredStandDown,
    strength: f32,
    gating_enabled: bool,
) -> Option<CompressionGuards> {
    if !gating_enabled {
        return None;
    }
    let strength = clamp01(strength);
    if strength <= 0.0 {
        return None;
    }
    let density_confidence = clamp01(confidence.density.confidence);
    if density_confidence <= 0.0 {
        return None;
    }

    let stand_down_amount = clamp01(stand_down.stand_down) * strength * density_confidence;
    let low_amount = band_guard_amount(
        band_psr.and_then(|p| p.low_p10_db),
        strength,
        density_confidence,
    );
    let mid_amount = band_guard_amount(
        band_psr.and_then(|p| p.mid_p10_db),
        strength,
        density_confidence,
    );
    let high_amount = band_guard_amount(
        band_psr.and_then(|p| p.high_p10_db),
        strength,
        density_confidence,
    );

    let low = BandCompressionGuard::from_amount(low_amount.max(stand_down_amount));
    let mid = BandCompressionGuard::from_amount(mid_amount.max(stand_down_amount));
    let high = BandCompressionGuard::from_amount(high_amount.max(stand_down_amount));
    if low == BandCompressionGuard::identity()
        && mid == BandCompressionGuard::identity()
        && high == BandCompressionGuard::identity()
        && stand_down_amount <= 1.0e-6
    {
        return None;
    }

    let mut reasons = Vec::new();
    if low_amount > 1.0e-6 {
        reasons.push(GuardReason::LowBandDense);
    }
    if mid_amount > 1.0e-6 {
        reasons.push(GuardReason::MidBandDense);
    }
    if high_amount > 1.0e-6 {
        reasons.push(GuardReason::HighBandDense);
    }
    if stand_down_amount > 1.0e-6 {
        reasons.push(GuardReason::AlreadyMastered);
    }

    Some(CompressionGuards {
        low,
        mid,
        high,
        stand_down: stand_down_amount,
        reasons,
    })
}

fn band_guard_amount(psr_db: Option<f32>, strength: f32, confidence: f32) -> f32 {
    let Some(psr_db) = psr_db.filter(|value| value.is_finite()) else {
        return 0.0;
    };
    descending_ramp(psr_db, BAND_PSR_SOFT_DB, BAND_PSR_FULL_DB) * strength * confidence
}

pub fn compression_plan_for_resolved_settings(settings: &MasteringSettings) -> CompressionPlan {
    let coeffs = crate::dsp::ChainCoeffs::from_settings(44_100, settings);
    let adaptive_strength = settings
        .advanced
        .adaptive_strength
        .unwrap_or(ADAPTIVE_STRENGTH_DEFAULT)
        .clamp(0.0, 1.0);
    let guards = settings
        .advanced
        .compression_guards
        .as_ref()
        .filter(|_| {
            matches!(
                settings.advanced.compression_mode,
                crate::types::CompressionMode::Preset
            )
        })
        .filter(|_| adaptive_strength > 0.0)
        .filter(|_| is_adaptive_compression_enabled());
    let confidence = settings
        .advanced
        .source_confidence
        .unwrap_or_default()
        .density
        .confidence
        .clamp(0.0, 1.0);
    let active = guards.is_some();
    let reasons = guards.map_or_else(Vec::new, |g| {
        g.reasons
            .iter()
            .copied()
            .map(|code| CompressionPlanReason {
                code,
                message: guard_reason_message(code).to_string(),
            })
            .collect()
    });
    let guidance = if reasons.is_empty() {
        None
    } else {
        Some(
            reasons
                .iter()
                .map(|reason| reason.message.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        )
    };
    let digest = guards.map(|g| compression_guard_digest(g, confidence));
    CompressionPlan {
        active,
        low: band_plan(
            coeffs.comp_low_threshold_db,
            coeffs.comp_low_ratio,
            guards.map(|g| &g.low),
        ),
        mid: band_plan(
            coeffs.comp_mid_threshold_db,
            coeffs.comp_mid_ratio,
            guards.map(|g| &g.mid),
        ),
        high: band_plan(
            coeffs.comp_high_threshold_db,
            coeffs.comp_high_ratio,
            guards.map(|g| &g.high),
        ),
        reasons,
        guidance,
        digest,
    }
}

fn band_plan(
    threshold_db: f32,
    ratio: f32,
    guard: Option<&BandCompressionGuard>,
) -> CompressionBandPlan {
    let guard = guard
        .copied()
        .unwrap_or_else(BandCompressionGuard::identity);
    CompressionBandPlan {
        threshold_db,
        ratio,
        density_mult: guard.density_mult,
        threshold_lift_db: guard.threshold_lift_db,
        ratio_mult: guard.ratio_mult,
        adaptive: guard != BandCompressionGuard::identity(),
    }
}

fn guard_reason_message(reason: GuardReason) -> &'static str {
    match reason {
        GuardReason::LowBandDense => "Low band is already dense - easing compression there.",
        GuardReason::MidBandDense => "Mid band is already dense - easing compression there.",
        GuardReason::HighBandDense => "High band is already dense - easing compression there.",
        GuardReason::AlreadyMastered => {
            "Source looks already mastered - easing compression across all bands."
        }
    }
}

fn compression_guard_digest(guards: &CompressionGuards, density_confidence: f32) -> String {
    fn eased_pct(guard: BandCompressionGuard) -> i32 {
        (((1.0 - guard.density_mult).clamp(0.0, 1.0) * 100.0).round()) as i32
    }
    format!(
        "compression eased low {}% / mid {}% / high {}%; stand-down {:.2}; density confidence {:.2}",
        eased_pct(guards.low),
        eased_pct(guards.mid),
        eased_pct(guards.high),
        guards.stand_down.clamp(0.0, 1.0),
        density_confidence.clamp(0.0, 1.0)
    )
}

/// Precomputed defensive trim multipliers for one source at one strength. Each
/// multiplier is in `[1 - cap, 1.0]`; `1.0` means "no trim on this axis".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceGuardrails {
    bright_mult: f32,
    low_mult: f32,
    density_mult: f32,
    width_mult: f32,
}

impl SourceGuardrails {
    /// Compute trims from a source profile at the given `strength` in `[0, 1]`.
    /// Callers skip this entirely when strength <= 0 (no profile => no trim).
    pub fn compute(profile: &SourceProfile, strength: f32) -> Self {
        // Tier-1 path: full confidence on every axis (no DeepAnalysis consulted).
        Self::compute_with_confidence(profile, strength, &Confidence::full())
    }

    /// Tier-2 Phase B: like [`compute`](Self::compute) but scales each axis's
    /// defensive excess by its per-axis `confidence` in `[0, 1]` (reduce-only —
    /// confidence never increases a trim beyond Tier-1). Full confidence reproduces
    /// `compute` byte-for-byte (`raw * 1.0 == raw`), so a source with no
    /// DeepAnalysis is unaffected. The anti-homogenization core: act strongly only
    /// where a quality is broad AND consistent (see [`crate::confidence`]).
    pub fn compute_with_confidence(
        profile: &SourceProfile,
        strength: f32,
        confidence: &Confidence,
    ) -> Self {
        let strength = clamp01(strength);
        let s = &profile.spectral_6;

        // already-bright -> trim air/high lift
        let brightness = s.presence + s.air;
        let bright_raw = clamp01((brightness - BRIGHT_DEADBAND) / BRIGHT_EXCESS_FULL)
            * confidence.bright.confidence;
        let bright_mult = 1.0 - (bright_raw * strength).min(EQ_CAP);

        // already-boomy -> trim low/sub lift
        let lowness = s.sub + s.low;
        let low_raw =
            clamp01((lowness - LOW_DEADBAND) / LOW_EXCESS_FULL) * confidence.low.confidence;
        let low_mult = 1.0 - (low_raw * strength).min(EQ_CAP);

        // already-dense -> soften compression (max of DR and LRA triggers)
        let dr_raw = descending_ramp(
            profile.dynamic_range_p95_p10_db,
            DENSITY_DR_SOFT_DB,
            DENSITY_DR_FULL_DB,
        );
        // A non-finite EBU LRA is sanitized to 0.0 upstream (analysis.rs), which
        // is indistinguishable from a real measurement and would otherwise read
        // as "maximally dense" and force a full trim. Treat <= 0.5 LU as
        // "unknown" — no real music master sits that low without the P95-P10 DR
        // also catching it — so the LRA ramp contributes nothing and density
        // rests on the P95-P10 measure alone.
        let lra_raw = if profile.dynamic_range_lu > 0.5 {
            descending_ramp(
                profile.dynamic_range_lu,
                DENSITY_LRA_SOFT_LU,
                DENSITY_LRA_FULL_LU,
            )
        } else {
            0.0
        };
        let density_raw = dr_raw.max(lra_raw) * confidence.density.confidence;
        let density_mult = 1.0 - (density_raw * strength).min(DENSITY_CAP);

        // already-wide -> trim widening (correlation primary; mono never trims)
        let width_excess = match profile.stereo_correlation {
            Some(c) => descending_ramp(c, WIDTH_CORR_DEADBAND, WIDTH_CORR_FULL),
            None => 0.0,
        };
        let width_raw = width_excess * confidence.width.confidence;
        let width_mult = 1.0 - (width_raw * strength).min(WIDTH_CAP);

        Self {
            bright_mult,
            low_mult,
            density_mult,
            width_mult,
        }
    }

    /// An identity (no-op) guardrail — every multiplier `1.0`.
    pub fn identity() -> Self {
        Self {
            bright_mult: 1.0,
            low_mult: 1.0,
            density_mult: 1.0,
            width_mult: 1.0,
        }
    }

    /// Trim a preset's high/air-region boost (3.5k / 6k / 12k bands). Reduce-
    /// only: cuts and zero pass through untouched; a positive boost is scaled by
    /// the bright multiplier but never pushed below the character floor.
    pub fn trim_bright_db(&self, preset_db: f32) -> f32 {
        floor_boost(preset_db, self.bright_mult)
    }

    /// Trim a preset's sub/low boost (80 Hz / 200 Hz bands). Reduce-only.
    pub fn trim_low_db(&self, preset_db: f32) -> f32 {
        floor_boost(preset_db, self.low_mult)
    }

    /// Scale the compression `density` macro `[0, 1]`. Reduce-only by
    /// construction (multiplier <= 1.0).
    pub fn scale_density(&self, density: f32) -> f32 {
        density * self.density_mult
    }

    /// Pull a preset stereo-width baseline toward neutral (1.0). Widths at or
    /// below 1.0 pass through untouched (we never narrow a source — that's
    /// corrective Tier-2); only the widening *above* 1.0 is trimmed.
    pub fn trim_width(&self, preset_width: f32) -> f32 {
        if preset_width <= 1.0 {
            return preset_width;
        }
        1.0 + (preset_width - 1.0) * self.width_mult
    }
}

/// Apply a reduce-only multiplier to a boost, honoring the character floor.
/// Negative or zero `preset_db` (a cut, or no move) passes through unchanged.
#[inline]
fn floor_boost(preset_db: f32, mult: f32) -> f32 {
    if preset_db <= 0.0 {
        return preset_db;
    }
    let trimmed = preset_db * mult;
    // Never trim below the floor; but if the original boost was already smaller
    // than the floor, leave it (don't raise it).
    trimmed.max(EQ_BOOST_FLOOR_DB.min(preset_db))
}

/// Read-only per-axis summary of what the guardrails did, for the "what was
/// trimmed and why" UI. `*_trim` are fractions removed from each preset move
/// `[0, cap]`; the `*_share` / `dynamic_range_db` / `stereo_correlation` fields
/// are the source context that drove them. NOTE: these are CHAIN trims, computed
/// before the post-chain LUFS-landing stage — label any UI accordingly.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct GuardrailReadout {
    pub active: bool,
    pub strength: f32,
    pub bright_trim: f32,
    pub low_trim: f32,
    pub density_trim: f32,
    pub width_trim: f32,
    pub brightness_share: f32,
    pub low_share: f32,
    pub dynamic_range_db: f32,
    /// Deadband thresholds the source must cross before an axis trims, surfaced so
    /// the UI can explain a `-0%` as "source in range" instead of looking broken.
    /// `brightness_share`/`low_share` trim when they exceed `bright_deadband`/
    /// `low_deadband`; width trims when `stereo_correlation` is BELOW
    /// `width_corr_deadband` (lower correlation = wider). `#[serde(default)]` for
    /// back-compat with older serialized readouts.
    #[serde(default)]
    pub bright_deadband: f32,
    #[serde(default)]
    pub low_deadband: f32,
    #[serde(default)]
    pub width_corr_deadband: f32,
    #[serde(default)]
    pub stereo_correlation: Option<f32>,
    /// Tier-2 Phase B: the per-axis confidence (coverage × consistency) that gated
    /// the trims, present only when confidence gating resolved a value (gate on +
    /// deep present). `None` => Tier-1 full confidence. Lets a calibration session
    /// SEE why each axis acted (or held back) by eye. `#[serde(default)]` for
    /// back-compat with older readouts.
    #[serde(default)]
    pub confidence: Option<crate::confidence::Confidence>,
}

/// Realized trim fraction (AFTER the +0.5 dB character floor) for a set of preset
/// EQ bands at `preset_scale`: how much of the total POSITIVE boost the guardrail
/// actually removed. The raw cap fraction (`1 - mult`) overstates near the floor;
/// this reads true for the by-ear calibration session (B8).
fn realized_eq_trim(bands: &[f32], preset_scale: f32, trim: impl Fn(f32) -> f32) -> f32 {
    let mut orig = 0.0_f32;
    let mut removed = 0.0_f32;
    for &db in bands {
        let boost = db * preset_scale;
        if boost > 0.0 {
            orig += boost;
            removed += boost - trim(boost);
        }
    }
    if orig > 1.0e-6 {
        removed / orig
    } else {
        0.0
    }
}

/// Compute the read-only guardrail summary for `settings`. Same gating as the
/// chain: a source profile must be present and `strength > 0`, else
/// `active = false`. The EQ trims are the REALIZED fractions (after the +0.5 dB
/// floor, computed against the actual preset bands); density/width are floor-free
/// and exact.
pub fn readout_for(settings: &MasteringSettings) -> GuardrailReadout {
    let strength = settings
        .advanced
        .adaptive_strength
        .unwrap_or(ADAPTIVE_STRENGTH_DEFAULT)
        .clamp(0.0, 1.0);
    match settings
        .advanced
        .source_profile
        .as_ref()
        .filter(|_| strength > 0.0)
    {
        Some(p) => {
            let g = SourceGuardrails::compute_with_confidence(
                p,
                strength,
                &settings.advanced.source_confidence.unwrap_or_default(),
            );
            let preset = crate::dsp::preset_calibration(&settings.preset);
            let preset_scale = 0.4 + 1.2 * settings.intensity.clamp(0.0, 1.0);
            GuardrailReadout {
                active: true,
                strength,
                bright_trim: realized_eq_trim(
                    &[preset.high_mid_db, preset.air_db, preset.sparkle_db],
                    preset_scale,
                    |o| g.trim_bright_db(o),
                ),
                low_trim: realized_eq_trim(
                    &[preset.sub_db, preset.low_shelf_db],
                    preset_scale,
                    |o| g.trim_low_db(o),
                ),
                density_trim: 1.0 - g.density_mult,
                width_trim: 1.0 - g.width_mult,
                brightness_share: p.spectral_6.presence + p.spectral_6.air,
                low_share: p.spectral_6.sub + p.spectral_6.low,
                dynamic_range_db: p.dynamic_range_p95_p10_db,
                bright_deadband: BRIGHT_DEADBAND,
                low_deadband: LOW_DEADBAND,
                width_corr_deadband: WIDTH_CORR_DEADBAND,
                stereo_correlation: p.stereo_correlation,
                confidence: settings.advanced.source_confidence,
            }
        }
        None => GuardrailReadout {
            active: false,
            strength,
            bright_trim: 0.0,
            low_trim: 0.0,
            density_trim: 0.0,
            width_trim: 0.0,
            brightness_share: 0.0,
            low_share: 0.0,
            dynamic_range_db: 0.0,
            bright_deadband: BRIGHT_DEADBAND,
            low_deadband: LOW_DEADBAND,
            width_corr_deadband: WIDTH_CORR_DEADBAND,
            stereo_correlation: None,
            confidence: None,
        },
    }
}

/// Tauri command: read-only adaptive-trim summary for the current settings.
/// B2: the backend resolves the effective profile from its store (keyed by
/// `track_id`) so the readout reflects the SAME profile the chain will apply;
/// album mode is non-adaptive. An FE-supplied profile on `settings` is honored as
/// an override, matching every other chain entry point.
#[tauri::command]
pub fn guardrail_readout(
    mut settings: MasteringSettings,
    track_id: Option<TrackId>,
    album: Option<bool>,
    profile_store: tauri::State<'_, std::sync::Arc<crate::profile_store::SourceProfileStore>>,
) -> GuardrailReadout {
    let album = album.unwrap_or(false);
    let cached = track_id.as_ref().and_then(|t| profile_store.get(t));
    let cached_deep = track_id.as_ref().and_then(|t| profile_store.get_deep(t));
    let cached_stand_down = track_id
        .as_ref()
        .and_then(|t| profile_store.get_stand_down(t));
    crate::profile_store::apply_resolved_profile(&mut settings, cached, album);
    crate::profile_store::apply_resolved_confidence(&mut settings, cached_deep.clone(), album);
    crate::profile_store::apply_resolved_compression_guards(
        &mut settings,
        cached_deep,
        cached_stand_down,
        album,
    );
    readout_for(&settings)
}

#[tauri::command]
pub fn resolve_compression_plan(
    mut settings: MasteringSettings,
    track_id: Option<TrackId>,
    album: Option<bool>,
    profile_store: tauri::State<'_, std::sync::Arc<crate::profile_store::SourceProfileStore>>,
) -> CompressionPlan {
    let album = album.unwrap_or(false);
    let cached = track_id.as_ref().and_then(|t| profile_store.get(t));
    let cached_deep = track_id.as_ref().and_then(|t| profile_store.get_deep(t));
    let cached_stand_down = track_id
        .as_ref()
        .and_then(|t| profile_store.get_stand_down(t));
    crate::profile_store::apply_resolved_profile(&mut settings, cached, album);
    crate::profile_store::apply_resolved_confidence(&mut settings, cached_deep.clone(), album);
    crate::profile_store::apply_resolved_compression_guards(
        &mut settings,
        cached_deep,
        cached_stand_down,
        album,
    );
    compression_plan_for_resolved_settings(&settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SourceProfile, SpectralBalance6};

    struct AdaptiveCompressionGateReset(bool);

    impl AdaptiveCompressionGateReset {
        fn set(enabled: bool) -> Self {
            Self(set_adaptive_compression_enabled(enabled))
        }
    }

    impl Drop for AdaptiveCompressionGateReset {
        fn drop(&mut self) {
            set_adaptive_compression_enabled(self.0);
        }
    }

    fn profile(
        presence: f32,
        air: f32,
        sub: f32,
        low: f32,
        dr: f32,
        lra: f32,
        corr: Option<f32>,
    ) -> SourceProfile {
        SourceProfile {
            spectral_6: SpectralBalance6 {
                sub,
                low,
                low_mid: 0.2,
                mid: 0.2,
                presence,
                air,
            },
            dynamic_range_p95_p10_db: dr,
            dynamic_range_lu: lra,
            stereo_correlation: corr,
            stereo_width: 1.0,
        }
    }

    /// A realistic pink-tilted neutral master (presence+air ~= 0.278, sub+low
    /// ~= 0.377 — the shares a true 1/f spectrum yields across our band edges),
    /// dynamic and well-correlated: nothing should trim. The old fixture
    /// (presence 0.08, air 0.05) was unrealistically dark and masked the
    /// neutral-master deadband misfire.
    fn neutral() -> SourceProfile {
        profile(0.143, 0.135, 0.207, 0.170, 10.0, 9.0, Some(0.8))
    }

    #[test]
    fn neutral_source_is_identity() {
        let g = SourceGuardrails::compute(&neutral(), 1.0);
        assert_eq!(g, SourceGuardrails::identity());
        assert_eq!(g.trim_bright_db(3.0), 3.0);
        assert_eq!(g.trim_low_db(3.0), 3.0);
        assert_eq!(g.scale_density(0.5), 0.5);
        assert_eq!(g.trim_width(1.1), 1.1);
    }

    #[test]
    fn deadband_means_no_action_just_inside() {
        // brightness exactly at the 0.30 deadband edge -> no trim.
        let p = profile(0.15, 0.15, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        assert_eq!(g.trim_bright_db(3.0), 3.0);
    }

    #[test]
    fn bright_source_trims_only_air() {
        // presence+air = 0.40, well past the 0.30 deadband.
        let p = profile(0.20, 0.20, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        let trimmed = g.trim_bright_db(3.0);
        assert!(trimmed < 3.0, "should trim a positive air boost: {trimmed}");
        assert!(
            trimmed >= 1.5 - 1e-6,
            "EQ cap limits removal to 50%: {trimmed}"
        );
        // low / density / width untouched.
        assert_eq!(g.trim_low_db(3.0), 3.0);
        assert_eq!(g.scale_density(0.5), 0.5);
        assert_eq!(g.trim_width(1.2), 1.2);
    }

    #[test]
    fn bright_trim_respects_floor_and_reduce_only() {
        let p = profile(0.25, 0.25, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        // a small boost stays at/above the +0.5 dB floor
        assert!(g.trim_bright_db(0.8) >= 0.5 - 1e-6);
        // a boost already below the floor is left as-is (never raised)
        assert_eq!(g.trim_bright_db(0.3), 0.3);
        // cuts and zero are never touched
        assert_eq!(g.trim_bright_db(-2.0), -2.0);
        assert_eq!(g.trim_bright_db(0.0), 0.0);
    }

    #[test]
    fn boomy_source_trims_only_low() {
        // sub+low = 0.55, past the 0.42 deadband.
        let p = profile(0.08, 0.05, 0.25, 0.30, 10.0, 9.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        assert!(g.trim_low_db(3.0) < 3.0);
        assert!(g.trim_low_db(3.0) >= 1.5 - 1e-6);
        assert_eq!(g.trim_bright_db(3.0), 3.0);
    }

    #[test]
    fn dense_source_softens_compression_capped() {
        // very low DR + LRA -> full density ramp, capped at 60%.
        let p = profile(0.08, 0.05, 0.08, 0.22, 2.0, 2.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        let d = g.scale_density(0.5);
        assert!(d < 0.5);
        assert!(d >= 0.5 * 0.4 - 1e-6, "cap keeps >=40% of density: {d}");
    }

    #[test]
    fn dense_via_lra_alone_still_trims() {
        // DR healthy but a GENUINE low LRA (2.0 LU) -> still soften (max of triggers).
        let p = profile(0.08, 0.05, 0.08, 0.22, 9.0, 2.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        assert!(g.scale_density(0.5) < 0.5);
    }

    #[test]
    fn missing_p95p10_does_not_alias_lra_into_the_db_ramp() {
        // When P95-P10 is unmeasured, from_analysis carries the 100.0 "no DR
        // trigger" sentinel; density then rests on the LRA ramp (LU thresholds),
        // never on an LU value aliased into the dB DR ramp (B11).
        let healthy = profile(0.08, 0.05, 0.08, 0.22, 100.0, 8.0, Some(0.8));
        assert_eq!(
            SourceGuardrails::compute(&healthy, 1.0).scale_density(0.5),
            0.5,
            "healthy LRA + unmeasured DR must not density-trim"
        );
        let dense = profile(0.08, 0.05, 0.08, 0.22, 100.0, 2.0, Some(0.8));
        assert!(
            SourceGuardrails::compute(&dense, 1.0).scale_density(0.5) < 0.5,
            "low LRA still softens via the LRA ramp"
        );
    }

    #[test]
    fn lra_sentinel_does_not_density_trim_a_dynamic_source() {
        // A non-finite EBU LRA is sanitized to 0.0 upstream. With a HEALTHY
        // P95-P10 DR, that sentinel must NOT be read as "maximally dense".
        let p = profile(0.08, 0.05, 0.08, 0.22, 10.0, 0.0, Some(0.8));
        let g = SourceGuardrails::compute(&p, 1.0);
        assert_eq!(
            g.scale_density(0.5),
            0.5,
            "LRA=0.0 sentinel must not density-trim a dynamic source"
        );
    }

    #[test]
    fn wide_source_trims_widening_capped_and_reduce_only() {
        // correlation 0.1 -> strongly wide.
        let p = profile(0.08, 0.05, 0.08, 0.22, 10.0, 9.0, Some(0.1));
        let g = SourceGuardrails::compute(&p, 1.0);
        let w = g.trim_width(1.5); // preset widens to 1.5
        assert!(
            w < 1.5 && w > 1.0,
            "pull toward neutral but not past it: {w}"
        );
        assert!(w >= 1.0 + 0.5 * 0.30 - 1e-6, "width cap keeps >=30%: {w}");
        // a narrowing preset (<=1.0) is never touched
        assert_eq!(g.trim_width(0.9), 0.9);
    }

    #[test]
    fn mono_source_never_trims_width() {
        let p = profile(0.08, 0.05, 0.08, 0.22, 10.0, 9.0, None);
        let g = SourceGuardrails::compute(&p, 1.0);
        assert_eq!(g.trim_width(1.5), 1.5);
    }

    #[test]
    fn strength_scales_trim_monotonically() {
        let p = profile(0.20, 0.20, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let gentle = SourceGuardrails::compute(&p, 0.3);
        let strong = SourceGuardrails::compute(&p, 1.0);
        assert!(strong.trim_bright_db(3.0) < gentle.trim_bright_db(3.0));
        assert!(
            gentle.trim_bright_db(3.0) < 3.0,
            "even gentle trims something"
        );
    }

    #[test]
    fn zero_strength_is_identity() {
        let p = profile(0.30, 0.30, 0.30, 0.30, 1.0, 1.0, Some(0.0));
        let g = SourceGuardrails::compute(&p, 0.0);
        assert_eq!(g, SourceGuardrails::identity());
    }

    #[test]
    fn full_confidence_is_byte_identical_to_tier1() {
        use crate::confidence::Confidence;
        // triggers every axis (bright + boomy + dense + wide).
        let p = profile(0.25, 0.25, 0.30, 0.30, 2.0, 2.0, Some(0.1));
        assert_eq!(
            SourceGuardrails::compute(&p, 1.0),
            SourceGuardrails::compute_with_confidence(&p, 1.0, &Confidence::full()),
        );
    }

    #[test]
    fn lower_confidence_reduces_trim_reduce_only() {
        use crate::confidence::Confidence;
        // brightness 0.35 sits just above the 0.30 deadband and BELOW the EQ cap,
        // so confidence scaling is observable (a capped trim would saturate).
        let p = profile(0.18, 0.17, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let full = SourceGuardrails::compute_with_confidence(&p, 1.0, &Confidence::full());
        let mut conf = Confidence::full();
        conf.bright.confidence = 0.5;
        let gated = SourceGuardrails::compute_with_confidence(&p, 1.0, &conf);
        // half confidence -> the air boost is trimmed LESS (kept closer to original).
        assert!(
            gated.trim_bright_db(3.0) > full.trim_bright_db(3.0),
            "confidence gating must REDUCE the trim (gated {} vs full {})",
            gated.trim_bright_db(3.0),
            full.trim_bright_db(3.0),
        );
        // never trims MORE than Tier-1 (reduce-only).
        assert!(gated.trim_bright_db(3.0) <= 3.0);
        // zero confidence on an axis -> that axis does not trim at all.
        conf.bright.confidence = 0.0;
        let off = SourceGuardrails::compute_with_confidence(&p, 1.0, &conf);
        assert_eq!(
            off.trim_bright_db(3.0),
            3.0,
            "zero confidence = no brightness trim"
        );
    }

    #[test]
    fn already_mastered_stand_down_requires_hot_limited_low_lra_and_uniform_low_psr() {
        use crate::deep_analysis::BandPsrStats;

        let already_mastered = classify_already_mastered_stand_down(
            -8.5,
            -0.7,
            4.2,
            Some(BandPsrStats {
                low_p10_db: Some(5.0),
                mid_p10_db: Some(4.5),
                high_p10_db: Some(5.5),
            }),
        );
        assert_eq!(already_mastered.stand_down, 1.0);
        assert!(already_mastered.hot_loudness);
        assert!(already_mastered.near_ceiling);
        assert!(already_mastered.low_lra);
        assert!(already_mastered.uniformly_low_psr);

        let clean_dynamic = classify_already_mastered_stand_down(
            -14.0,
            -3.0,
            11.0,
            Some(BandPsrStats {
                low_p10_db: Some(12.0),
                mid_p10_db: Some(13.0),
                high_p10_db: Some(12.5),
            }),
        );
        assert_eq!(clean_dynamic.stand_down, 0.0);
        assert!(!clean_dynamic.hot_loudness);
        assert!(!clean_dynamic.near_ceiling);
        assert!(!clean_dynamic.low_lra);
        assert!(!clean_dynamic.uniformly_low_psr);
    }

    #[test]
    fn already_mastered_stand_down_requires_all_three_psr_bands() {
        use crate::deep_analysis::BandPsrStats;

        let missing_high = classify_already_mastered_stand_down(
            -8.5,
            -0.7,
            4.2,
            Some(BandPsrStats {
                low_p10_db: Some(5.0),
                mid_p10_db: Some(4.5),
                high_p10_db: None,
            }),
        );

        assert_eq!(missing_high.stand_down, 0.0);
        assert!(
            !missing_high.uniformly_low_psr,
            "short/partial-band reads must not classify as already-mastered"
        );
    }

    #[test]
    fn compression_guards_resolve_dense_bands_reduce_only_when_gate_on() {
        use crate::confidence::Confidence;
        use crate::deep_analysis::BandPsrStats;

        let guards = resolve_compression_guards(
            Some(BandPsrStats {
                low_p10_db: Some(5.0),
                mid_p10_db: Some(13.0),
                high_p10_db: Some(4.0),
            }),
            &Confidence::full(),
            AlreadyMasteredStandDown::identity(),
            1.0,
            true,
        )
        .expect("dense low/high bands should resolve guards");

        assert!(guards.low.density_mult < 1.0);
        assert!(guards.low.threshold_lift_db > 0.0);
        assert!(guards.low.ratio_mult < 1.0);
        assert_eq!(guards.mid, BandCompressionGuard::identity());
        assert!(guards.high.density_mult < 1.0);
        assert!(
            guards.reasons.contains(&GuardReason::LowBandDense)
                && guards.reasons.contains(&GuardReason::HighBandDense)
        );
    }

    #[test]
    fn compression_guards_stay_identity_for_dynamic_or_low_confidence_sources() {
        use crate::confidence::{AxisConfidence, Confidence};
        use crate::deep_analysis::BandPsrStats;

        let dynamic = resolve_compression_guards(
            Some(BandPsrStats {
                low_p10_db: Some(14.0),
                mid_p10_db: Some(15.0),
                high_p10_db: Some(16.0),
            }),
            &Confidence::full(),
            AlreadyMasteredStandDown::identity(),
            1.0,
            true,
        );
        assert_eq!(
            dynamic, None,
            "dynamic/high-PSR source should stay preset-flat"
        );

        let mut low_confidence = Confidence::full();
        low_confidence.density = AxisConfidence {
            coverage: 0.0,
            consistency: 1.0,
            confidence: 0.0,
        };
        let gated_out = resolve_compression_guards(
            Some(BandPsrStats {
                low_p10_db: Some(4.0),
                mid_p10_db: Some(4.0),
                high_p10_db: Some(4.0),
            }),
            &low_confidence,
            AlreadyMasteredStandDown::identity(),
            1.0,
            true,
        );
        assert_eq!(
            gated_out, None,
            "zero density confidence should hold adaptation at identity"
        );
    }

    #[test]
    fn compression_guards_stand_down_source_gets_maximum_easing() {
        use crate::confidence::Confidence;

        let guards = resolve_compression_guards(
            None,
            &Confidence::full(),
            AlreadyMasteredStandDown {
                stand_down: 1.0,
                hot_loudness: true,
                near_ceiling: true,
                low_lra: true,
                uniformly_low_psr: true,
            },
            1.0,
            true,
        )
        .expect("stand-down should resolve guards even without band PSR buckets");

        assert_eq!(guards.stand_down, 1.0);
        assert_eq!(guards.low, guards.mid);
        assert_eq!(guards.mid, guards.high);
        assert!(guards.low.density_mult <= 0.65);
        assert!(guards.low.threshold_lift_db >= 3.0);
        assert!(guards.low.ratio_mult < 0.80);
        assert!(guards.reasons.contains(&GuardReason::AlreadyMastered));
    }

    #[test]
    fn compression_guards_gate_off_is_none_even_with_dense_inputs() {
        use crate::confidence::Confidence;
        use crate::deep_analysis::BandPsrStats;

        let guards = resolve_compression_guards(
            Some(BandPsrStats {
                low_p10_db: Some(4.0),
                mid_p10_db: Some(4.0),
                high_p10_db: Some(4.0),
            }),
            &Confidence::full(),
            AlreadyMasteredStandDown::identity(),
            1.0,
            false,
        );

        assert_eq!(guards, None);
    }

    #[test]
    fn compression_plan_reports_backend_resolved_adaptive_values() {
        use crate::confidence::Confidence;

        let _gate = AdaptiveCompressionGateReset::set(true);
        let mut settings = settings_with(None, Some(1.0));
        settings.advanced.source_confidence = Some(Confidence::full());
        settings.advanced.compression_guards = Some(CompressionGuards {
            low: BandCompressionGuard {
                density_mult: 0.80,
                threshold_lift_db: 2.0,
                ratio_mult: 0.90,
            },
            mid: BandCompressionGuard::identity(),
            high: BandCompressionGuard {
                density_mult: 0.75,
                threshold_lift_db: 2.4,
                ratio_mult: 0.88,
            },
            stand_down: 0.25,
            reasons: vec![GuardReason::LowBandDense, GuardReason::HighBandDense],
        });

        let plan = compression_plan_for_resolved_settings(&settings);

        assert!(plan.active);
        assert!(plan.low.adaptive);
        assert!(!plan.mid.adaptive);
        assert!(plan.high.adaptive);
        assert!(
            plan.low.threshold_db > plan.mid.threshold_db,
            "low threshold should be softened by the backend guard plan: {:?}",
            plan
        );
        assert!(
            plan.low.ratio < plan.mid.ratio,
            "low ratio should be eased by the backend guard plan: {:?}",
            plan
        );
        assert!(
            plan.guidance
                .as_deref()
                .unwrap_or_default()
                .contains("Low band is already dense"),
            "{plan:?}"
        );
        assert!(
            plan.digest
                .as_deref()
                .unwrap_or_default()
                .contains("compression eased low 20%"),
            "{plan:?}"
        );
    }

    fn settings_with(
        profile: Option<SourceProfile>,
        strength: Option<f32>,
    ) -> crate::types::MasteringSettings {
        crate::types::MasteringSettings {
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
                adaptive_strength: strength,
                source_profile: profile,
                ..Default::default()
            },
        }
    }

    fn confidence_for_tone(freq: f32) -> crate::confidence::Confidence {
        let sr = 48_000_u32;
        let n = sr as usize * 3;
        let omega = 2.0 * std::f32::consts::PI * freq / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let windows = crate::deep_analysis::scan_windows(&samples, sr, 1);
        let deep = crate::deep_analysis::DeepAnalysis::from_parts([1.0 / 31.0; 31], windows);
        crate::confidence::Confidence::from_deep(&deep)
    }

    #[test]
    fn readout_reports_trims_and_context_when_active() {
        // bright (presence+air 0.45) + dense (DR/LRA 2) + wide (corr 0.1); not boomy.
        let p = profile(0.25, 0.20, 0.08, 0.22, 2.0, 2.0, Some(0.1));
        let r = readout_for(&settings_with(Some(p), Some(1.0)));
        assert!(r.active);
        assert!(r.bright_trim > 0.0); // Universal carries a positive air boost
        assert!(r.density_trim > 0.0);
        assert!(r.width_trim > 0.0);
        assert_eq!(
            r.low_trim, 0.0,
            "sub+low 0.30 < 0.42 deadband -> no low trim"
        );
        assert!((r.brightness_share - 0.45).abs() < 1e-6);
        assert_eq!(r.stereo_correlation, Some(0.1));
        // Deadband thresholds are surfaced so the UI can explain a -0% axis.
        assert_eq!(r.bright_deadband, BRIGHT_DEADBAND);
        assert_eq!(r.low_deadband, LOW_DEADBAND);
        assert_eq!(r.width_corr_deadband, WIDTH_CORR_DEADBAND);
    }

    #[test]
    fn readout_carries_deadbands_even_when_inactive() {
        // So the UI can always render "share vs threshold", even at strength 0.
        let p = profile(0.25, 0.20, 0.08, 0.22, 2.0, 2.0, Some(0.1));
        let r = readout_for(&settings_with(Some(p), Some(0.0)));
        assert!(!r.active);
        assert_eq!(r.bright_deadband, BRIGHT_DEADBAND);
        assert_eq!(r.low_deadband, LOW_DEADBAND);
        assert_eq!(r.width_corr_deadband, WIDTH_CORR_DEADBAND);
    }

    #[test]
    fn readout_inactive_without_profile_or_at_zero_strength() {
        assert!(!readout_for(&settings_with(None, Some(1.0))).active);
        let p = profile(0.25, 0.20, 0.08, 0.22, 2.0, 2.0, Some(0.1));
        assert!(!readout_for(&settings_with(Some(p), Some(0.0))).active);
    }

    #[test]
    fn realized_eq_trim_respects_the_character_floor() {
        // A 0.8 dB boost at multiplier 0.5 floors at +0.5 dB -> 0.3 dB removed =
        // 37.5% realized, NOT the 50% the raw cap fraction would report (B8).
        let realized = realized_eq_trim(&[0.8], 1.0, |o| floor_boost(o, 0.5));
        assert!((realized - 0.375).abs() < 1.0e-4, "realized={realized}");
    }

    #[test]
    fn readout_honors_source_confidence_reduce_only() {
        use crate::confidence::{AxisConfidence, Confidence};
        // brightness 0.35 sits above the 0.30 deadband but below the EQ cap, so the
        // confidence scaling is observable in the realized trim.
        let p = profile(0.18, 0.17, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let mut s = settings_with(Some(p), Some(1.0)); // source_confidence None => full
        let full = readout_for(&s);
        assert!(
            full.bright_trim > 0.0,
            "Universal air boost should trim at full confidence"
        );
        // Inject a low-confidence brightness axis (scattered/ambiguous trait).
        let mut conf = Confidence::full();
        conf.bright = AxisConfidence {
            coverage: 0.3,
            consistency: 0.5,
            confidence: 0.15,
        };
        s.advanced.source_confidence = Some(conf);
        let gated = readout_for(&s);
        assert!(
            gated.bright_trim < full.bright_trim,
            "confidence gating must REDUCE the realized bright trim (gated {} vs full {})",
            gated.bright_trim,
            full.bright_trim,
        );
        assert!(gated.bright_trim >= 0.0);
    }

    #[test]
    fn readout_uses_31band_confidence_to_distinguish_harsh_from_air() {
        let p = profile(0.20, 0.20, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let mut s = settings_with(Some(p), Some(1.0));
        let full = readout_for(&s);
        assert!(full.bright_trim > 0.0);

        s.advanced.source_confidence = Some(confidence_for_tone(12_000.0));
        let airy = readout_for(&s);
        assert!(
            airy.bright_trim < full.bright_trim * 0.25,
            "air-only 31-band read should back off bright trim: airy {} full {}",
            airy.bright_trim,
            full.bright_trim
        );

        s.advanced.source_confidence = Some(confidence_for_tone(3_000.0));
        let harsh = readout_for(&s);
        assert!(
            harsh.bright_trim > full.bright_trim * 0.70,
            "harsh 31-band read should preserve most bright trim: harsh {} full {}",
            harsh.bright_trim,
            full.bright_trim
        );
    }

    #[test]
    fn readout_surfaces_source_confidence_for_eye_validation() {
        use crate::confidence::Confidence;
        let p = profile(0.18, 0.17, 0.08, 0.22, 10.0, 9.0, Some(0.8));
        let mut s = settings_with(Some(p), Some(1.0));
        // No confidence resolved (gate off / Tier-1) -> readout carries None.
        assert_eq!(readout_for(&s).confidence, None);
        // A resolved confidence is surfaced verbatim, so a calibration session can
        // read coverage/consistency/confidence per axis by eye.
        let conf = Confidence::full();
        s.advanced.source_confidence = Some(conf);
        assert_eq!(readout_for(&s).confidence, Some(conf));
        // An inactive readout (no profile) carries None too.
        assert_eq!(
            readout_for(&settings_with(None, Some(1.0))).confidence,
            None
        );
    }
}
