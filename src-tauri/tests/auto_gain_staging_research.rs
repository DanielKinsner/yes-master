//! Tests for the auto gain staging research runner (plan Phase A). The
//! runner lives in `examples/auto_gain_staging_research/`; its core module is
//! compiled here as well so these properties are pinned in the normal suite.

#[path = "../examples/auto_gain_staging_research/research.rs"]
mod research;
#[allow(dead_code)]
#[path = "../src/sample_rate.rs"]
mod sample_rate;

use research::*;
use yes_master_lib::{types, Preset};

/// Deterministic stereo test music: pink-ish noise with drum-like bursts and
/// a slow level swell, so the compressor, saturation and limiter all work.
fn test_music(rate: u32, seconds: f64, peak: f32) -> Vec<f32> {
    let frames = (f64::from(rate) * seconds) as usize;
    let mut out = Vec::with_capacity(frames * 2);
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut rnd = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
    };
    let (mut b0, mut b1, mut b2) = (0.0_f32, 0.0_f32, 0.0_f32);
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        let w = rnd();
        b0 = 0.997 * b0 + 0.029 * w;
        b1 = 0.985 * b1 + 0.032 * w;
        b2 = 0.950 * b2 + 0.048 * w;
        let pink = b0 + b1 + b2 + 0.02 * w;
        let beat = (t * 2.0).fract();
        let hit = if beat < 0.03 {
            (1.0 - beat / 0.03) * (std::f32::consts::TAU * 60.0 * t).sin()
        } else {
            0.0
        };
        let swell = 0.6 + 0.4 * (std::f32::consts::TAU * 0.25 * t).sin();
        out.push(swell * (0.5 * pink + 0.8 * hit));
        out.push(swell * (0.45 * pink + 0.8 * hit) + 0.05 * rnd());
    }
    let max = out.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    for s in out.iter_mut() {
        *s *= peak / max;
    }
    out
}

#[test]
fn split_render_is_bit_identical_to_the_production_chain() {
    for rate in [44_100, 48_000] {
        for peak in [0.2_f32, 0.99] {
            let x = test_music(rate, 3.0, peak);
            for (_, preset) in standard_styles() {
                let settings = standard_export_settings(preset, -9.0, None, Some(-14.0));
                assert!(
                    decomposition_matches_chain(&x, rate, 2, &settings),
                    "rate {rate} peak {peak}"
                );
            }
        }
    }
}

#[test]
fn standard_settings_follow_the_standard_export_recipe() {
    let s = standard_export_settings(Preset::Tape, -11.0, None, None);
    assert_eq!(s.effective_target_lufs(), Some(-11.0));
    assert_eq!(s.effective_ceiling_dbtp(), -1.0);
    assert_eq!(s.effective_bit_depth(), 24);
    assert_eq!(s.effective_sample_rate(48_000), 44_100);
    assert!(!s.volume_match);
    assert_eq!(s.intensity, 0.5);
}

#[test]
fn the_detector_replica_predicts_the_limiters_load() {
    let rate = 44_100;
    let x = test_music(rate, 3.0, 0.5);
    let settings = standard_export_settings(Preset::Universal, -9.0, None, None);
    let stage = stage_pass(&x, rate, 2, &settings, 0.0);
    let peak_db = lin_to_db(limiter_detector_peak(&stage.pre, 2));
    let idle = limiter_pass(&stage, -1.0, -1.0 - peak_db - 0.01);
    assert_eq!(idle.max_gain_reduction_db, 0.0);
    for over in [0.5_f32, 3.0, 6.0] {
        let driven = limiter_pass(&stage, -1.0, -1.0 - peak_db + over);
        assert!(
            (driven.max_gain_reduction_db - over).abs() < 0.01,
            "over {over}: measured {}",
            driven.max_gain_reduction_db
        );
    }
}

#[test]
fn level_in_follows_contract_rule_two() {
    let nominal = Nominal {
        lufs: -12.3,
        peak_dbtp: -1.9,
    };
    // The reference song itself: exactly zero.
    assert_eq!(level_in_db(nominal, -12.3, -1.9), Some(0.0));
    // A loud master: the loudness term binds (turn down 3 dB).
    assert_eq!(level_in_db(nominal, -9.3, -0.1), Some(-3.0));
    // A quiet, dynamic mix: the peak term binds (only +2 dB, not +8).
    let quiet = level_in_db(nominal, -20.3, -3.9).unwrap();
    assert!((quiet - 2.0).abs() < 1e-5);
    // The ±24 dB input range.
    assert_eq!(level_in_db(nominal, -60.0, -50.0), Some(24.0));
    // Inert without a usable measurement (A7).
    assert_eq!(level_in_db(nominal, f32::NEG_INFINITY, -3.0), None);
    assert_eq!(level_in_db(nominal, -80.0, -60.0), None);
    assert_eq!(level_in_db(nominal, -14.0, f32::NAN), None);
}

#[test]
fn the_drive_never_lands_above_target_and_stops_at_the_clean_limit() {
    let rate = 44_100;
    let x = test_music(rate, 4.0, 0.3);
    let settings = standard_export_settings(Preset::Universal, -9.0, None, None);
    let stage = stage_pass(&x, rate, 2, &settings, 0.0);
    let base = loudness(&stage.pre[..stage.frames * 2], rate, 2);

    // Reachable with the limiter idle: the drive is exactly the difference.
    let (r, render) = resolve_drive(&stage, base - 3.0, -1.0, Some(6.0), 0.2, 4);
    assert!(r.limiter_idle && r.resolved && r.limiter_passes == 1);
    assert_eq!(r.drive_db, (base - 3.0) - base);
    assert_eq!(render.max_gain_reduction_db, 0.0);

    for target in [-14.0_f32, -11.0, -9.0, -6.0] {
        for clean in [Some(6.0_f32), Some(2.0), None] {
            let (r, render) = resolve_drive(&stage, target, -1.0, clean, 0.2, 4);
            for (_, l) in &r.trace {
                assert!(*l <= target + 0.01, "target {target}: {l}");
            }
            if let Some(limit) = clean {
                assert!(
                    render.max_gain_reduction_db <= limit + 0.01,
                    "load {} over {limit}",
                    render.max_gain_reduction_db
                );
            }
            assert!(r.resolved || r.stopped_at_clean_limit || r.limiter_passes == 4);
        }
    }
}

#[test]
fn the_delivered_drive_never_lands_above_target_and_stops_at_the_clean_limit() {
    let rate = 48_000; // includes the export's conversion to 44.1 kHz
    let x = test_music(rate, 4.0, 0.3);
    let settings = standard_export_settings(Preset::Clarity, -9.0, None, None);
    let stage = stage_pass(&x, rate, 2, &settings, 0.0);
    for target in [-14.0_f32, -9.0, -6.0] {
        for clean in [Some(6.0_f32), Some(2.0)] {
            let (r, load, samples, facts) =
                resolve_drive_delivered(&stage, target, clean, 0.2, 4).expect("delivers");
            for (_, delivered) in &r.trace {
                assert!(*delivered <= target + 0.01, "target {target}: {delivered}");
            }
            assert!(load <= clean.unwrap() + 0.01, "load {load}");
            assert!(facts.all_finite && facts.under_ceiling);
            assert_eq!(samples.len() % 2, 0);
            assert!(r.resolved || r.stopped_at_clean_limit || r.limiter_passes == 4);
            if r.resolved {
                assert!(facts.lufs >= target - 0.2);
            }
        }
    }
}

#[test]
fn the_turn_down_never_raises_the_level() {
    let rate = 44_100;
    let x = test_music(rate, 4.0, 0.99);
    let settings = standard_export_settings(Preset::Oomph, -14.0, None, None);
    let t = limiter_pass(&stage_pass(&x, rate, 2, &settings, 0.0), -1.0, 0.0);
    let t_lufs = loudness(&t.samples, rate, 2);
    let target = t_lufs - 4.0;
    let (r, render) = resolve_turn_down(&x, rate, 2, &settings, t_lufs, target, 0.2, 4);
    assert!(render.is_some());
    assert!(r.pre_gain_db < 0.0);
    for (gain, _) in &r.trace {
        assert!(*gain <= 0.0);
    }
    assert!(r.resolved, "{:?}", r.trace);
    // No overshoot: S is T.
    let (r, render) = resolve_turn_down(&x, rate, 2, &settings, t_lufs, t_lufs + 1.0, 0.2, 4);
    assert!(render.is_none() && r.pre_gain_db == 0.0 && r.passes == 1);
}

#[test]
fn find_peaks_matches_scipy() {
    let x = [0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
    assert_eq!(find_peaks(&x, 1), vec![1, 3, 5]);
    assert_eq!(find_peaks(&x, 3), vec![1, 5]);
    // A plateau peaks at its middle (left of centre for even widths).
    assert_eq!(find_peaks(&[0.0, 2.0, 2.0, 0.0], 1), vec![1]);
    assert_eq!(find_peaks(&[0.0, 2.0, 2.0, 2.0, 0.0], 1), vec![2]);
    // Edges and rising tails are not peaks.
    assert!(find_peaks(&[3.0, 1.0, 2.0], 1).is_empty());
}

#[test]
fn percentile_matches_numpy() {
    let v = [4.0, 1.0, 3.0, 2.0];
    assert!((percentile(&v, 10.0).unwrap() - 1.3).abs() < 1e-12);
    assert!((median(&v).unwrap() - 2.5).abs() < 1e-12);
    assert!(percentile(&[], 50.0).is_none());
}

#[test]
fn welch_bands_locate_a_tone_and_cancel_level() {
    let rate = ANALYSIS_RATE;
    let tone: Vec<f32> = (0..rate as usize * 3)
        .flat_map(|i| {
            let s = 0.5 * (std::f32::consts::TAU * 1500.0 * i as f32 / rate as f32).sin();
            [s, s]
        })
        .collect();
    let anchors = source_anchors(&tone, 2);
    let a = shape(&tone, 2, &anchors);
    let loudest = (0..9)
        .max_by(|i, j| a.tone_bands_db[*i].total_cmp(&a.tone_bands_db[*j]))
        .unwrap();
    assert_eq!(loudest, 5, "1.5 kHz is in the 1–2 kHz band");
    // A pure gain change leaves every tone band unchanged.
    let quieter: Vec<f32> = tone.iter().map(|s| s * 0.5).collect();
    let b = shape(&quieter, 2, &anchors);
    let change = compare(&a, &anchors, &b);
    for d in change.tone_band_delta_db {
        assert!(d.abs() < 1e-6, "{d}");
    }
    assert!(change.side_mid_delta_db.unwrap().abs() < 1e-9);
}

fn outcome(lufs: f32, plr: f32, landing: f32, stopped: bool) -> DesignOutcome {
    DesignOutcome {
        lufs,
        true_peak_dbtp: lufs + plr,
        plr_db: plr,
        landing_db: landing,
        all_finite: true,
        under_ceiling: true,
        stopped_at_clean_limit: stopped,
        tone_band_delta_db: [0.0; 9],
    }
}

fn case(t: DesignOutcome, g: DesignOutcome, reference: bool) -> RuleCase {
    RuleCase {
        song: "x".into(),
        reference_song: reference,
        source_lufs: -10.0,
        style: "Universal".into(),
        target: -9.0,
        level_in_db: Some(0.0),
        t: Some(t.clone()),
        s: Some(t),
        g: Some(g),
    }
}

#[test]
fn the_decision_rule_reads_cases_as_written() {
    let distinct = vec![DistinctCase {
        song: "x".into(),
        source_lufs: -10.0,
        target: -9.0,
        t: Some(1.0),
        s: Some(1.0),
        g: Some(1.2),
    }];
    // G gains 1.5 dB PLR where T turned down 2 dB; one quiet miss reached.
    let cases = vec![
        case(
            outcome(-9.0, 8.0, -2.0, false),
            outcome(-9.0, 9.5, 0.0, false),
            true,
        ),
        case(
            outcome(-11.0, 10.0, 0.0, false),
            outcome(-9.6, 7.0, 0.0, true),
            false,
        ),
    ];
    let g = evaluate_rule(&cases, &distinct, Design::G);
    assert!(g.iter().all(|c| c.pass), "{g:?}");
    let s = evaluate_rule(&cases, &distinct, Design::S);
    assert_eq!(select_design(&g, &s), "G");

    // A 0.11 dB PLR loss where both land fails R1.
    let cases = vec![case(
        outcome(-9.0, 8.0, -2.0, false),
        outcome(-9.0, 7.89, 0.0, false),
        true,
    )];
    let g = evaluate_rule(&cases, &distinct, Design::G);
    assert!(!g.iter().find(|c| c.id == "R1").unwrap().pass);
    // S (equal to T here) gains nothing, so it fails R3: neither qualifies.
    let s = evaluate_rule(&cases, &distinct, Design::S);
    assert!(!s.iter().find(|c| c.id == "R3").unwrap().pass);
    assert_eq!(select_design(&g, &s), "neither");

    // A T miss that G neither reaches nor stops at the clean limit fails R4.
    let cases = vec![case(
        outcome(-11.0, 10.0, 0.0, false),
        outcome(-10.0, 9.0, 0.0, false),
        false,
    )];
    let g = evaluate_rule(&cases, &distinct, Design::G);
    assert!(!g.iter().find(|c| c.id == "R4").unwrap().pass);
}
