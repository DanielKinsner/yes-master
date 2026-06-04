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
            let g = SourceGuardrails::compute(p, strength);
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
    let cached = track_id.as_ref().and_then(|t| profile_store.get(t));
    crate::profile_store::apply_resolved_profile(&mut settings, cached, album.unwrap_or(false));
    readout_for(&settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SourceProfile, SpectralBalance6};

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
}
