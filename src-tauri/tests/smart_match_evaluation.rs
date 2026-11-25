//! Smart Match plan, Slice 6: the evaluation runner's own building blocks.
//! The runner's verdicts are only as good as its independent equalizer and
//! its gate arithmetic, so both are pinned here on synthetic audio.

use yes_master_lib::smart_match::{gap_db, measure_shares, BAND_COUNT};
use yes_master_lib::smart_match_evaluation::{
    apply_tone_change, levels_db, loudest_window, summarize_known_answers, CaseResult, SongGroup,
    ToneChange,
};

fn noise(sample_rate: u32, seconds: f64) -> Vec<f32> {
    let frames = (f64::from(sample_rate) * seconds) as usize;
    let mut state: u32 = 0x2468_ACE1;
    let (mut b0, mut b1, mut b2) = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut out = Vec::with_capacity(frames * 2);
    for _ in 0..frames {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let w = ((state >> 16) & 0x7FFF) as f32 / 32_768.0 - 0.5;
        b0 = 0.99765 * b0 + w * 0.099_046;
        b1 = 0.96300 * b1 + w * 0.296_516_4;
        b2 = 0.57000 * b2 + w * 1.052_691_3;
        let p = (b0 + b1 + b2 + w * 0.1848) * 0.1;
        out.push(p);
        out.push(p * 0.8);
    }
    out
}

#[test]
fn the_equalizer_passes_audio_unchanged_at_zero_gain() {
    let x = noise(44_100, 3.0);
    let y = apply_tone_change(&x, 44_100, 2, &ToneChange::Band { band: 2, db: 0.0 });
    let worst = x
        .iter()
        .zip(&y)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(worst < 1e-5, "worst sample difference {worst}");
}

#[test]
fn a_band_change_is_measured_where_it_was_made() {
    let sr = 48_000;
    let x = noise(sr, 20.0);
    let original = measure_shares(&x, sr, 2).unwrap();
    for band in 0..BAND_COUNT {
        let y = apply_tone_change(&x, sr, 2, &ToneChange::Band { band, db: 4.0 });
        let copy = measure_shares(&y, sr, 2).unwrap();
        // Reference (original) minus track (copy): −4 dB in the changed band.
        let gap = gap_db(&levels_db(&copy), &levels_db(&original));
        for (b, g) in gap.iter().enumerate() {
            if b == band {
                assert!((g + 4.0).abs() < 0.3, "band {band}: changed band gap {g}");
            } else {
                assert!(g.abs() < 0.3, "band {band}: band {b} moved by {g}");
            }
        }
    }
}

#[test]
fn the_tilt_is_one_db_per_octave_around_one_kilohertz() {
    let t = ToneChange::Tilt { db_per_octave: 1.0 };
    assert!((t.gain_db(2_000.0, 48_000) - 1.0).abs() < 1e-9);
    assert!((t.gain_db(250.0, 48_000) + 2.0).abs() < 1e-9);
    assert_eq!(t.changed_bands(), [true; BAND_COUNT]);
}

#[test]
fn the_loudest_window_is_found() {
    let sr = 8_000;
    let mut x = vec![0.01_f32; sr as usize * 100 * 2];
    // A loud stretch from 60 s to 95 s.
    for v in x
        .iter_mut()
        .skip(sr as usize * 60 * 2)
        .take(sr as usize * 35 * 2)
    {
        *v = 0.5;
    }
    let w = loudest_window(&x, sr, 2, 30.0);
    assert_eq!(w.len(), sr as usize * 30 * 2);
    assert!(w.iter().all(|v| *v == 0.5));
}

fn case(gap: f64, correction: f32, fade: f32, ratio: Option<f64>) -> CaseResult {
    let mut changed = [false; BAND_COUNT];
    changed[1] = true;
    let mut measured = [0.0; BAND_COUNT];
    measured[1] = gap;
    let mut corr = [0.0; BAND_COUNT];
    corr[1] = correction;
    let mut f = [1.0; BAND_COUNT];
    f[1] = fade;
    let mut r = [None; BAND_COUNT];
    r[1] = ratio;
    CaseResult {
        song: "s".into(),
        group: SongGroup::LibraryBuild,
        case: "bass".into(),
        changed_bands: changed,
        copy_shares: None,
        measured_gap_db: measured,
        correction_db: corr,
        capped: [false; BAND_COUNT],
        fade: f,
        remaining_ratio: r,
        remaining_ratio_vs_rendered_original: r,
        eq_stage_ratio: r,
    }
}

#[test]
fn the_gate_arithmetic_passes_and_fails_where_it_should() {
    // Correct direction and size, remaining gap 50%.
    let good = summarize_known_answers(&[
        case(-4.0, -2.0, 1.0, Some(0.5)),
        case(3.0, 1.5, 1.0, Some(0.45)),
    ]);
    assert!(good.g3_pass && good.g6_pass, "{good:?}");
    // Wrong direction fails G3.
    let wrong = summarize_known_answers(&[case(-4.0, 2.0, 1.0, Some(0.5))]);
    assert!(!wrong.g3_pass);
    assert_eq!(wrong.wrong_direction, 1);
    // A remaining gap outside 35–65% fails G6.
    let short = summarize_known_answers(&[case(-4.0, -2.0, 1.0, Some(0.8))]);
    assert!(!short.g6_pass);
    // A faded boost is reported, not graded.
    let faded =
        summarize_known_answers(&[case(4.0, 0.0, 0.0, None), case(-4.0, -2.0, 1.0, Some(0.5))]);
    assert_eq!(faded.faded_bands, 1);
    assert_eq!(faded.eligible_bands, 1);
    assert!(faded.g3_pass);
}
