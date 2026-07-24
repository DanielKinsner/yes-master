//! U13 — adaptive DSP characterization across source classes.
//!
//! ## What this answers
//!
//! The shipped `preset_fingerprint.rs` harness measures the presets on the
//! NON-adaptive chain: `AdvancedSettings::default()` carries no `SourceProfile`,
//! so the guardrails never engage. That leaves the actual product question
//! unmeasured — **once the guardrails start trimming, do the presets stay
//! distinguishable, and do they stay safe?** Adaptive trims only ever reduce
//! preset moves, so the honest worry is convergence: eight presets all trimmed
//! toward the same place stop being eight presets.
//!
//! This file measures that. It does **not** act on it. Per the plan, a collapse
//! is reported and routed to the owner listening gate (U15), never tuned away
//! here.
//!
//! ## PRE-REGISTRATION
//!
//! Everything in this section was written and committed **before any adaptive
//! number was computed or read.** That ordering is the whole point: if the
//! metric or threshold were chosen after seeing results, a convenient one could
//! be selected and the "characterization" would be decoration.
//!
//! **Metric — inherited, not chosen.** The distinctness metric is the existing
//! `character_distance` over the existing `Fingerprint`, exactly as shipped in
//! `preset_fingerprint.rs`. No new metric is defined here. This is deliberate:
//! an inherited metric cannot have been reverse-engineered to produce a
//! flattering result.
//!
//! **Source-profile partitions.** Five classes, four probed plus one holdout.
//! Shares are set relative to the real guardrail deadbands in `guardrails.rs`
//! (`BRIGHT_DEADBAND` 0.30 on presence+air, `LOW_DEADBAND` 0.42 on sub+low,
//! density ramps at P95-P10 8→3 dB and LRA 6→3 LU, `WIDTH_CORR_DEADBAND` 0.50
//! on L/R correlation) so each class genuinely crosses its trigger. A partition
//! that sat inside the deadband would render identically to the non-adaptive
//! path and prove nothing.
//!
//!   * `neutral` — control; inside every deadband. Establishes the baseline.
//!   * `bright`  — presence+air well above 0.30.
//!   * `boomy`   — sub+low well above 0.42.
//!   * `dense`   — P95-P10 and LRA at/below their full-trim floors.
//!   * `wide`    — correlation well below 0.50.  **← HOLDOUT**
//!
//! **Holdout.** `wide` takes no part in any threshold decision. It exists only
//! to check that a conclusion drawn from the other four generalizes. If a floor
//! is ever pinned, it must hold on the holdout too.
//!
//! **Thresholds, fixed in advance.**
//!
//!   * *Safety* (hard assertions, no judgement): every adapted render must stay
//!     inside the SAME caps the non-adaptive harness already enforces —
//!     |band tilt| ≤ 6 dB, drum crest ≥ 5 dB, drum PSR ≥ 4 dB, THD proxy
//!     ≤ −20 dB, everything finite. Guardrails reduce moves, so any adapted
//!     render breaching a cap the non-adaptive render clears would be a genuine
//!     defect, not a taste question.
//!   * *Distinctness candidate floor*: **1.0** — the same value
//!     `preset_fingerprint.rs` already enforces on the non-adaptive path. Not a
//!     new number.
//!   * *Pin condition, decided before looking*: the floor is pinned **only if**
//!     the minimum pairwise distance observed across **all five** partitions
//!     (holdout included) is ≥ `1.0 × 1.2` = **1.2**, i.e. at least 20 %
//!     headroom over the floor being pinned. Between 1.0 and 1.2 → report, do
//!     not pin (insufficient margin to distinguish a real floor from noise).
//!     Below 1.0 → **collapse**: report it, change nothing, route to U15.
//!
//! ## Boundaries
//!
//! Nothing here changes a preset coefficient, calibration constant, loudness
//! target, limiter value, Volume Match, or audition timing. This file only
//! measures.

mod fingerprint_kit;

use fingerprint_kit::{
    character_distance, compute_fingerprint_with_profile, fixtures, master_with_profile,
    Fingerprint, FACTORY_PRESETS, TEST_INTENSITY,
};
use yes_master_lib::types::{SourceProfile, SpectralBalance6};

// ---------------------------------------------------------------------------
// Pre-registered partitions
// ---------------------------------------------------------------------------

/// Candidate floor — the value already enforced on the non-adaptive path.
const CANDIDATE_FLOOR: f32 = 1.0;
/// Required headroom before the candidate floor may be pinned.
const PIN_HEADROOM_FACTOR: f32 = 1.2;

struct Partition {
    name: &'static str,
    /// Excluded from every threshold decision; used only to check
    /// generalization.
    holdout: bool,
    profile: SourceProfile,
}

fn balance(
    sub: f32,
    low: f32,
    low_mid: f32,
    mid: f32,
    presence: f32,
    air: f32,
) -> SpectralBalance6 {
    SpectralBalance6 {
        sub,
        low,
        low_mid,
        mid,
        presence,
        air,
    }
}

fn partitions() -> Vec<Partition> {
    vec![
        Partition {
            name: "neutral",
            holdout: false,
            // presence+air = 0.22 (< 0.30), sub+low = 0.32 (< 0.42),
            // DR and LRA above their soft knees, correlation above 0.50.
            profile: SourceProfile {
                spectral_6: balance(0.10, 0.22, 0.24, 0.22, 0.14, 0.08),
                dynamic_range_p95_p10_db: 12.0,
                dynamic_range_lu: 9.0,
                stereo_correlation: Some(0.80),
                stereo_width: 0.9,
            },
        },
        Partition {
            name: "bright",
            holdout: false,
            // presence+air = 0.44, comfortably past BRIGHT_DEADBAND + FULL.
            profile: SourceProfile {
                spectral_6: balance(0.06, 0.14, 0.16, 0.20, 0.26, 0.18),
                dynamic_range_p95_p10_db: 12.0,
                dynamic_range_lu: 9.0,
                stereo_correlation: Some(0.80),
                stereo_width: 0.9,
            },
        },
        Partition {
            name: "boomy",
            holdout: false,
            // sub+low = 0.58, comfortably past LOW_DEADBAND + FULL.
            profile: SourceProfile {
                spectral_6: balance(0.24, 0.34, 0.20, 0.12, 0.07, 0.03),
                dynamic_range_p95_p10_db: 12.0,
                dynamic_range_lu: 9.0,
                stereo_correlation: Some(0.80),
                stereo_width: 0.9,
            },
        },
        Partition {
            name: "dense",
            holdout: false,
            // At/below both density full-trim floors (3 dB / 3 LU).
            profile: SourceProfile {
                spectral_6: balance(0.10, 0.22, 0.24, 0.22, 0.14, 0.08),
                dynamic_range_p95_p10_db: 2.5,
                dynamic_range_lu: 2.5,
                stereo_correlation: Some(0.80),
                stereo_width: 0.9,
            },
        },
        Partition {
            name: "wide",
            holdout: true,
            // Correlation well below WIDTH_CORR_DEADBAND (0.50).
            profile: SourceProfile {
                spectral_6: balance(0.10, 0.22, 0.24, 0.22, 0.14, 0.08),
                dynamic_range_p95_p10_db: 12.0,
                dynamic_range_lu: 9.0,
                stereo_correlation: Some(0.05),
                stereo_width: 1.6,
            },
        },
    ]
}

// ---------------------------------------------------------------------------
// Pre-registered safety caps — identical to the non-adaptive harness.
// ---------------------------------------------------------------------------

const MAX_ABS_BAND_TILT_DB: f32 = 6.0;
const MIN_DRUM_CREST_DB: f32 = 5.0;
const MIN_DRUM_PSR_DB: f32 = 4.0;
const MAX_THD_PROXY_DB: f32 = -20.0;

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

struct PartitionResult {
    name: &'static str,
    holdout: bool,
    rows: Vec<(&'static str, Fingerprint)>,
}

fn characterize() -> Vec<PartitionResult> {
    let fx = fixtures();
    partitions()
        .into_iter()
        .map(|partition| PartitionResult {
            name: partition.name,
            holdout: partition.holdout,
            rows: FACTORY_PRESETS
                .iter()
                .map(|(name, preset)| {
                    (
                        *name,
                        compute_fingerprint_with_profile(
                            preset.clone(),
                            fx,
                            TEST_INTENSITY,
                            Some(partition.profile),
                        ),
                    )
                })
                .collect(),
        })
        .collect()
}

fn closest_pair(rows: &[(&'static str, Fingerprint)]) -> (&'static str, &'static str, f32) {
    let mut best = ("", "", f32::INFINITY);
    for i in 0..rows.len() {
        for j in (i + 1)..rows.len() {
            let d = character_distance(&rows[i].1, &rows[j].1);
            if d < best.2 {
                best = (rows[i].0, rows[j].0, d);
            }
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Anti-drift guard for the copied kit
// ---------------------------------------------------------------------------

#[test]
fn kit_reproduces_the_committed_golden() {
    // `tests/common/mod.rs` is a guarded copy of the helpers in
    // preset_fingerprint.rs. This is what stops the copy drifting: the kit's
    // non-adaptive fingerprints must still reproduce the committed golden that
    // gates every preset retune.
    let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/preset_fingerprint.json");
    let text = std::fs::read_to_string(&golden_path).expect("read committed golden");
    let golden: serde_json::Value = serde_json::from_str(&text).expect("parse golden");
    let golden_rows = golden.as_array().expect("golden is an array");

    let fx = fixtures();
    for (name, preset) in fingerprint_kit::FACTORY_PRESETS.iter() {
        let fp = compute_fingerprint_with_profile(preset.clone(), fx, TEST_INTENSITY, None);
        let row = golden_rows
            .iter()
            .find(|r| r["preset"] == *name)
            .unwrap_or_else(|| panic!("golden missing preset {name}"));

        for (i, tilt) in fp.band_tilt_db.iter().enumerate() {
            let expected = row["band_tilt_db"][i].as_f64().expect("tilt") as f32;
            assert!(
                (tilt - expected).abs() <= 0.25,
                "kit drifted from the golden: {name} band {i} tilt {tilt} vs {expected}",
            );
        }
        for (field, actual) in [
            ("landed_lufs_pink", fp.landed_lufs_pink),
            ("crest_db_drums", fp.crest_db_drums),
            ("width_delta_db", fp.width_delta_db),
        ] {
            let expected = row[field].as_f64().expect(field) as f32;
            assert!(
                (actual - expected).abs() <= 0.25,
                "kit drifted from the golden: {name} {field} {actual} vs {expected}",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Safety — hard assertions, no judgement involved
// ---------------------------------------------------------------------------

#[test]
fn adapted_renders_stay_finite_and_within_the_non_adaptive_safety_caps() {
    for partition in characterize() {
        for (name, fp) in &partition.rows {
            let label = format!("{}/{}", partition.name, name);

            for (i, tilt) in fp.band_tilt_db.iter().enumerate() {
                assert!(tilt.is_finite(), "{label}: band {i} tilt is not finite");
                assert!(
                    tilt.abs() <= MAX_ABS_BAND_TILT_DB,
                    "{label}: band {i} tilt {tilt} dB exceeds the {MAX_ABS_BAND_TILT_DB} dB cap. \
                     Guardrails only ever REDUCE preset moves, so an adapted render breaching a \
                     cap the non-adaptive render clears is a defect, not a taste question.",
                );
            }
            assert!(
                fp.crest_db_drums >= MIN_DRUM_CREST_DB,
                "{label}: drum crest {} dB below the {MIN_DRUM_CREST_DB} dB floor",
                fp.crest_db_drums,
            );
            assert!(
                fp.psr_db_drums >= MIN_DRUM_PSR_DB,
                "{label}: drum PSR {} dB below the {MIN_DRUM_PSR_DB} dB floor",
                fp.psr_db_drums,
            );
            assert!(
                fp.thd_proxy_db <= MAX_THD_PROXY_DB,
                "{label}: THD proxy {} dB above the {MAX_THD_PROXY_DB} dB cap",
                fp.thd_proxy_db,
            );
            assert!(
                fp.landed_lufs_pink.is_finite() && fp.width_delta_db.is_finite(),
                "{label}: non-finite aggregate metric",
            );
        }
    }
}

#[test]
fn no_adapted_render_clips() {
    let fx = fixtures();
    for partition in partitions() {
        for (name, preset) in fingerprint_kit::FACTORY_PRESETS.iter() {
            let out = master_with_profile(
                &fx.drums,
                preset.clone(),
                TEST_INTENSITY,
                Some(partition.profile),
            );
            let peak = out.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
            assert!(
                peak <= 1.0,
                "{}/{name}: adapted render clips (peak {peak} linear)",
                partition.name,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Distinctness — REPORT FIRST. The pin condition is evaluated, not assumed.
// ---------------------------------------------------------------------------

#[test]
fn adaptive_distinctness_report() {
    let results = characterize();

    let mut observed_min = f32::INFINITY;
    let mut report = String::from(
        "\n=== U13 adaptive distinctness — closest preset pair per source class ===\n",
    );
    for partition in &results {
        let (a, b, d) = closest_pair(&partition.rows);
        let tag = if partition.holdout { " [HOLDOUT]" } else { "" };
        report.push_str(&format!(
            "  {:<8}{tag:<11} closest: {a} <-> {b}  distance {d:.3}\n",
            partition.name,
        ));
        observed_min = observed_min.min(d);
    }

    let pin_threshold = CANDIDATE_FLOOR * PIN_HEADROOM_FACTOR;
    report.push_str(&format!(
        "\n  observed minimum across ALL partitions: {observed_min:.3}\n  \
         candidate floor {CANDIDATE_FLOOR:.2}, pin requires >= {pin_threshold:.2}\n",
    ));
    report.push_str(if observed_min < CANDIDATE_FLOOR {
        "  VERDICT: COLLAPSE — below the non-adaptive floor. Report to U15; do NOT tune here.\n"
    } else if observed_min < pin_threshold {
        "  VERDICT: PASSES the floor but with < 20% headroom. Reported, deliberately NOT pinned.\n"
    } else {
        "  VERDICT: robust — floor pinned by the test below.\n"
    });
    println!("{report}");

    // The pre-registered collapse condition. This is the only assertion the
    // report itself makes, and it is the one written before looking.
    assert!(
        observed_min >= CANDIDATE_FLOOR,
        "{report}\nAdaptive distinctness COLLAPSED below the non-adaptive floor \
         ({observed_min:.3} < {CANDIDATE_FLOOR:.2}). Per the plan this routes to the owner \
         listening gate (U15) as evidence — it does NOT authorize a retune.",
    );
}

/// The floor, pinned because the pre-registered condition was met.
///
/// FIRST RUN, 2026-07-24 — observed minimum **1.509** across all five
/// partitions (bright was tightest; the `wide` holdout came in at 1.604), which
/// clears the 1.2 pin threshold, so the candidate floor is now enforced.
///
/// Pinned at the CANDIDATE value (1.0), deliberately **not** at the observed
/// 1.509. Pinning to what was measured is the after-the-fact threshold
/// selection the pre-registration exists to prevent, and it would convert
/// ordinary drift into a spurious failure. 1.0 is the same floor the
/// non-adaptive path already enforces; the ~50 % headroom is the finding, not
/// the threshold.
const ADAPTED_MIN_PAIRWISE_DISTANCE: f32 = CANDIDATE_FLOOR;

#[test]
fn every_preset_pair_stays_distinct_under_every_source_class() {
    for partition in characterize() {
        for i in 0..partition.rows.len() {
            for j in (i + 1)..partition.rows.len() {
                let (a, fa) = &partition.rows[i];
                let (b, fb) = &partition.rows[j];
                let d = character_distance(fa, fb);
                assert!(
                    d >= ADAPTED_MIN_PAIRWISE_DISTANCE,
                    "{}/{a} vs {b}: character distance {d:.3} fell below the \
                     {ADAPTED_MIN_PAIRWISE_DISTANCE} floor. Two presets have converged under \
                     adaptive trimming. Do NOT retune to satisfy this — capture a listening note \
                     first (R15/U15).",
                    partition.name,
                );
            }
        }
    }
}

/// Owner-readable adaptive table for a listening sitting. Mirrors
/// `preset_fingerprint.rs::write_owner_fingerprint_report`.
///
/// `cargo test --test preset_adaptive_characterization write_adaptive_report -- --ignored`
#[test]
#[ignore = "diagnostic: writes an owner report, not a gate"]
fn write_adaptive_report() {
    use std::fmt::Write as _;

    let results = characterize();
    let mut md = String::from(
        "# Adaptive preset characterization (U13)\n\n\
         Synthetic fixtures, default intensity 0.5, per source class. Distances use the \
         shipped `character_distance` metric — the same one that gates the non-adaptive path.\n\n\
         **This is mechanical evidence, not a listening verdict.** It selects what to listen \
         to; it cannot approve or reject a preset.\n\n",
    );

    for partition in &results {
        let (a, b, d) = closest_pair(&partition.rows);
        let _ = writeln!(
            md,
            "## {}{}\n\nClosest pair: **{a} ↔ {b}** at {d:.3}\n\n\
             | Preset | sub | low | low-mid | mid | presence | air | crest dB | width dB | LUFS |\n\
             |---|---|---|---|---|---|---|---|---|---|",
            partition.name,
            if partition.holdout { " (holdout)" } else { "" },
        );
        for (name, fp) in &partition.rows {
            let t = fp.band_tilt_db;
            let _ = writeln!(
                md,
                "| {name} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
                t[0], t[1], t[2], t[3], t[4], t[5],
                fp.crest_db_drums, fp.width_delta_db, fp.landed_lufs_pink,
            );
        }
        md.push('\n');
    }

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-output/adaptive-characterization");
    std::fs::create_dir_all(&dir).expect("create report dir");
    let path = dir.join("adaptive-characterization.md");
    std::fs::write(&path, md).expect("write report");
    println!("Adaptive characterization report: {}", path.display());
}
