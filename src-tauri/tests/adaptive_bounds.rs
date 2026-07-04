//! Adaptive-ruin proofs (hardening plan B1/B3, owner fear #2).
//!
//! The owner's stated nightmare: "we analyze their track, assume the
//! intended high frequencies are unintentional, and bury them." These
//! property sweeps prove the Tier-1 guardrail math CANNOT do that: it is
//! reduce-only (a preset boost can be softened, never converted to a cut;
//! cuts and zero pass through bit-exact), capped per axis (a preset stays
//! recognizable at any strength), floored (boosts never trim below the
//! 0.5 dB character floor), monotone in strength, and inert at strength 0
//! or zero confidence.
//!
//! The caps/floors asserted here (EQ 0.50, density 0.60, width 0.70,
//! 0.5 dB boost floor) are the owner-listened Tier-1 constants
//! (accepted 2026-06-11) — if one changes, this test failing is the
//! intended tripwire demanding a new listening note.

use yes_master_lib::confidence::{AxisConfidence, Confidence};
use yes_master_lib::guardrails::SourceGuardrails;
use yes_master_lib::types::{SourceProfile, SpectralBalance6};

const EQ_CAP: f32 = 0.50;
const DENSITY_CAP: f32 = 0.60;
const WIDTH_CAP: f32 = 0.70;
const EQ_BOOST_FLOOR_DB: f32 = 0.5;

/// Deterministic LCG so the sweep is reproducible without a proptest dep.
struct Lcg(u32);
impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
        ((self.0 >> 16) & 0x7FFF) as f32 / 32_767.0
    }
    fn in_range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

fn random_profile(rng: &mut Lcg) -> SourceProfile {
    // Spectral shares roughly normalized like real analyses, but pushed to
    // extremes often enough to exercise every ramp.
    let raw: Vec<f32> = (0..6).map(|_| rng.next_f32().powi(2)).collect();
    let sum: f32 = raw.iter().sum::<f32>().max(1.0e-6);
    SourceProfile {
        spectral_6: SpectralBalance6 {
            sub: raw[0] / sum,
            low: raw[1] / sum,
            low_mid: raw[2] / sum,
            mid: raw[3] / sum,
            presence: raw[4] / sum,
            air: raw[5] / sum,
        },
        dynamic_range_p95_p10_db: rng.in_range(0.0, 24.0),
        dynamic_range_lu: rng.in_range(0.0, 20.0),
        stereo_correlation: if rng.next_f32() < 0.2 {
            None
        } else {
            Some(rng.in_range(-1.0, 1.0))
        },
        stereo_width: rng.in_range(0.0, 2.0),
    }
}

#[test]
fn strength_zero_is_identity_for_every_profile() {
    let mut rng = Lcg(0xA5A5_0001);
    for _ in 0..2_000 {
        let p = random_profile(&mut rng);
        let g = SourceGuardrails::compute(&p, 0.0);
        for boost in [-4.0_f32, -0.3, 0.0, 0.2, 1.5, 6.0] {
            assert_eq!(g.trim_bright_db(boost), boost, "bright at strength 0");
            assert_eq!(g.trim_low_db(boost), boost, "low at strength 0");
        }
        for d in [0.0_f32, 0.3, 1.0] {
            assert_eq!(g.scale_density(d), d, "density at strength 0");
        }
        for w in [0.5_f32, 1.0, 1.7] {
            assert_eq!(g.trim_width(w), w, "width at strength 0");
        }
    }
}

#[test]
fn adaptation_is_reduce_only_capped_and_floored() {
    let mut rng = Lcg(0xA5A5_0002);
    for _ in 0..5_000 {
        let p = random_profile(&mut rng);
        let strength = rng.next_f32();
        let g = SourceGuardrails::compute(&p, strength);

        // EQ: cuts and zero pass through bit-exact; boosts soften at most
        // to (1 - cap) x boost and never below the character floor.
        for cut in [-6.0_f32, -1.0, -0.01, 0.0] {
            assert_eq!(g.trim_bright_db(cut), cut, "cuts must pass through");
            assert_eq!(g.trim_low_db(cut), cut, "cuts must pass through");
        }
        for boost in [0.1_f32, 0.5, 1.0, 2.5, 6.0] {
            for trimmed in [g.trim_bright_db(boost), g.trim_low_db(boost)] {
                assert!(trimmed <= boost, "reduce-only: {trimmed} > {boost}");
                assert!(
                    trimmed >= boost * (1.0 - EQ_CAP) - 1.0e-6,
                    "EQ cap violated: {boost} dB trimmed to {trimmed}",
                );
                assert!(
                    trimmed >= EQ_BOOST_FLOOR_DB.min(boost) - 1.0e-6,
                    "character floor violated: {boost} dB trimmed to {trimmed}",
                );
            }
        }

        // Density: reduce-only, capped, stays in [0, 1] for legal input.
        for d in [0.0_f32, 0.2, 0.6, 1.0] {
            let scaled = g.scale_density(d);
            assert!(scaled <= d + 1.0e-6, "density reduce-only");
            assert!(
                scaled >= d * (1.0 - DENSITY_CAP) - 1.0e-6,
                "density cap violated: {d} scaled to {scaled}",
            );
            assert!((0.0..=1.0).contains(&scaled), "density out of range");
        }

        // Width: at-or-below neutral passes through; widening trims toward
        // 1.0 but never past the cap and never below neutral.
        for w in [0.0_f32, 0.6, 1.0] {
            assert_eq!(g.trim_width(w), w, "narrow/neutral width passes through");
        }
        for w in [1.1_f32, 1.5, 2.0] {
            let trimmed = g.trim_width(w);
            assert!(trimmed <= w + 1.0e-6, "width reduce-only");
            assert!(
                trimmed >= 1.0 + (w - 1.0) * (1.0 - WIDTH_CAP) - 1.0e-6,
                "width cap violated: {w} trimmed to {trimmed}",
            );
            assert!(trimmed >= 1.0, "widening trim must never narrow below neutral");
        }
    }
}

#[test]
fn adaptation_is_monotone_in_strength() {
    let mut rng = Lcg(0xA5A5_0003);
    for _ in 0..2_000 {
        let p = random_profile(&mut rng);
        let s1 = rng.next_f32();
        let s2 = rng.next_f32();
        let (lo, hi) = if s1 <= s2 { (s1, s2) } else { (s2, s1) };
        let g_lo = SourceGuardrails::compute(&p, lo);
        let g_hi = SourceGuardrails::compute(&p, hi);
        assert!(
            g_hi.trim_bright_db(3.0) <= g_lo.trim_bright_db(3.0) + 1.0e-6,
            "more strength must never mean more brightness",
        );
        assert!(
            g_hi.trim_low_db(3.0) <= g_lo.trim_low_db(3.0) + 1.0e-6,
            "more strength must never mean more low boost",
        );
        assert!(
            g_hi.scale_density(0.8) <= g_lo.scale_density(0.8) + 1.0e-6,
            "more strength must never mean more compression density",
        );
        assert!(
            g_hi.trim_width(1.6) <= g_lo.trim_width(1.6) + 1.0e-6,
            "more strength must never mean more widening",
        );
    }
}

#[test]
fn zero_confidence_is_identity_and_confidence_never_deepens_a_trim() {
    let axis = |c: f32| AxisConfidence {
        coverage: c,
        consistency: c,
        confidence: c,
    };
    let zero = Confidence {
        bright: axis(0.0),
        low: axis(0.0),
        density: axis(0.0),
        width: axis(0.0),
    };
    let mut rng = Lcg(0xA5A5_0004);
    for _ in 0..2_000 {
        let p = random_profile(&mut rng);
        let strength = rng.next_f32().max(0.05);

        // Zero confidence on every axis => no trims at all.
        let g0 = SourceGuardrails::compute_with_confidence(&p, strength, &zero);
        assert_eq!(g0, SourceGuardrails::identity(), "zero confidence must be inert");

        // Full confidence reproduces Tier-1 compute exactly (documented).
        let gf = SourceGuardrails::compute_with_confidence(&p, strength, &Confidence::full());
        assert_eq!(gf, SourceGuardrails::compute(&p, strength));

        // Partial confidence can only trim LESS than full confidence.
        let c = rng.next_f32();
        let partial = Confidence {
            bright: axis(c),
            low: axis(c),
            density: axis(c),
            width: axis(c),
        };
        let gp = SourceGuardrails::compute_with_confidence(&p, strength, &partial);
        assert!(
            gp.trim_bright_db(3.0) >= gf.trim_bright_db(3.0) - 1.0e-6,
            "partial confidence must never trim deeper than full",
        );
        assert!(gp.scale_density(0.8) >= gf.scale_density(0.8) - 1.0e-6);
        assert!(gp.trim_width(1.6) >= gf.trim_width(1.6) - 1.0e-6);
    }
}

#[test]
fn mono_sources_never_lose_width() {
    let mut rng = Lcg(0xA5A5_0005);
    for _ in 0..1_000 {
        let mut p = random_profile(&mut rng);
        p.stereo_correlation = None;
        let g = SourceGuardrails::compute(&p, 1.0);
        for w in [1.0_f32, 1.3, 2.0] {
            assert_eq!(g.trim_width(w), w, "mono must never trigger a width trim");
        }
    }
}

/// The owner's named nightmare, encoded directly: a maximally bright
/// source at full strength. The engine may decline to brighten further
/// (soften the preset's boost toward the floor) but can NEVER cut the
/// track's own high end: a zero move stays zero, a preset cut stays
/// exactly the user's cut, and a boost never leaves the [floor, boost]
/// band.
#[test]
fn intentional_brightness_is_never_buried() {
    let p = SourceProfile {
        spectral_6: SpectralBalance6 {
            sub: 0.02,
            low: 0.03,
            low_mid: 0.05,
            mid: 0.10,
            presence: 0.40,
            air: 0.40, // presence + air = 0.80: far beyond the deadband
        },
        dynamic_range_p95_p10_db: 4.0,
        dynamic_range_lu: 3.0,
        stereo_correlation: Some(0.1),
        stereo_width: 1.8,
    };
    let g = SourceGuardrails::compute(&p, 1.0);

    // No preset move => the engine adds nothing and removes nothing.
    assert_eq!(g.trim_bright_db(0.0), 0.0);
    // The user's own cut is sacred.
    assert_eq!(g.trim_bright_db(-2.0), -2.0);
    // A preset boost is softened but bounded: never below half (EQ cap),
    // never below the 0.5 dB character floor, never negative.
    let softened = g.trim_bright_db(3.0);
    assert!(softened >= 1.5 - 1.0e-6, "cap: got {softened}");
    assert!(softened <= 3.0, "reduce-only: got {softened}");
    let tiny = g.trim_bright_db(0.3);
    assert!(
        (0.3 - tiny).abs() < 1.0e-6,
        "a boost already under the floor is left as-is, got {tiny}",
    );
}
