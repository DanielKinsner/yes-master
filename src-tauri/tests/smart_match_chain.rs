//! Smart Match plan, Slice 4: the setting and the chain.
//!
//! - Identity (contract rule 9): no reference, switched off, amount zero and a
//!   self-match leave every preset's coefficients and rendered samples exactly
//!   as they are without Smart Match.
//! - Realization accuracy (gate G4): the six-band change measured through the
//!   chain's real coefficients is within 0.5 dB of the decision's correction.
//! - Level invariance (gate G5), Album inertness, user-moved band frequencies,
//!   presets never carrying a reference, and project save/load.

use std::collections::HashMap;

use yes_master_lib::dsp::{BiquadCoeffs, ChainCoeffs, MasteringChain};
use yes_master_lib::smart_match::{
    band_power_change_db, decide_for_settings, measure_shares, BOOST_CAP_DB, CUT_CAP_DB,
    TYPICAL_SHAPE_DB,
};
use yes_master_lib::{
    AdvancedSettings, DeliveryProfile, EqBandFrequencies, ImportedTrack, MasteringSettings, Preset,
    PresetKind, ProjectMode, ProjectState, SmartMatchReference, SmartMatchSettings, SourceProfile,
    SpectralBalance6, TrackId,
};

const RATES: [u32; 3] = [44_100, 48_000, 96_000];
/// Measurement grid for G4: far finer than the solver's 12 points per band.
const MEASURE_GRID: usize = 240;

fn all_presets() -> Vec<Preset> {
    vec![
        Preset::Universal,
        Preset::Clarity,
        Preset::Tape,
        Preset::Spatial,
        Preset::Oomph,
        Preset::Warmth,
        Preset::Punch,
        Preset::Loud,
        Preset::Custom {
            id: "custom".into(),
        },
    ]
}

fn shares_from_db(levels: [f64; 6]) -> SpectralBalance6 {
    let lin = levels.map(|d| 10f64.powf(d / 10.0));
    let total: f64 = lin.iter().sum();
    let s = lin.map(|v| (v / total) as f32);
    SpectralBalance6 {
        sub: s[0],
        low: s[1],
        low_mid: s[2],
        mid: s[3],
        presence: s[4],
        air: s[5],
    }
}

fn profile(shares: SpectralBalance6) -> SourceProfile {
    SourceProfile {
        spectral_6: shares,
        dynamic_range_p95_p10_db: 12.0,
        dynamic_range_lu: 6.0,
        stereo_correlation: Some(0.6),
        stereo_width: 0.3,
    }
}

/// A preset at default Intensity with a resolved track profile: the state the
/// live chain builds from once analysis has landed.
fn settings_for(preset: Preset, track: SpectralBalance6) -> MasteringSettings {
    MasteringSettings {
        preset,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings {
            source_profile: Some(profile(track)),
            ..Default::default()
        },
    }
}

fn with_match(
    mut settings: MasteringSettings,
    reference: SpectralBalance6,
    enabled: bool,
    amount: f32,
) -> MasteringSettings {
    settings.advanced.smart_match = Some(SmartMatchSettings {
        reference: Some(SmartMatchReference {
            name: "Reference".into(),
            spectral_6: reference,
        }),
        enabled,
        amount,
    });
    settings
}

/// The EQ cascade's real coefficients, in chain order.
fn eq_cascade(c: &ChainCoeffs) -> Vec<BiquadCoeffs> {
    let mut v = vec![
        c.sub_highpass,
        c.sub_highpass,
        c.sub,
        c.low,
        c.low_mid,
        c.mid,
        c.high_mid,
        c.high,
        c.sparkle,
        c.warmth,
        c.presence_air,
    ];
    if c.smart_match_active {
        v.push(c.smart_match_bass);
    }
    v
}

fn pink_stereo(frames: usize) -> Vec<f32> {
    let mut state: u32 = 0x1234_5678;
    let (mut b0, mut b1, mut b2) = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut out = Vec::with_capacity(frames * 2);
    for _ in 0..frames {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let w = ((state >> 16) & 0x7FFF) as f32 / 32_768.0 - 0.5;
        b0 = 0.99765 * b0 + w * 0.099_046;
        b1 = 0.96300 * b1 + w * 0.296_516_4;
        b2 = 0.57000 * b2 + w * 1.052_691_3;
        let p = (b0 + b1 + b2 + w * 0.1848) * 0.08;
        out.push(p);
        out.push(p * 0.9);
    }
    out
}

fn render(sample_rate: u32, settings: &MasteringSettings, input: &[f32]) -> Vec<f32> {
    let mut chain = MasteringChain::new(sample_rate, 2, settings);
    let mut out = input.to_vec();
    for frame in out.chunks_mut(2) {
        chain.process_frame_inplace(frame);
    }
    out
}

fn typical() -> SpectralBalance6 {
    shares_from_db(TYPICAL_SHAPE_DB)
}

/// A track a little off typical in every band, so guardrails and the
/// missing-band rule see a realistic profile.
fn track_shares() -> SpectralBalance6 {
    shares_from_db([-6.0, -5.5, -7.8, -10.0, -16.4, -22.5])
}

#[test]
fn off_states_leave_every_preset_bit_identical() {
    let track = track_shares();
    let other = shares_from_db([-8.0, -4.0, -9.0, -10.5, -14.0, -19.0]);
    let input = pink_stereo(12_000);
    for preset in all_presets() {
        for sr in RATES {
            let plain = settings_for(preset.clone(), track);
            let baseline = ChainCoeffs::from_settings(sr, &plain);
            let baseline_audio = render(sr, &plain, &input);
            let mut none = plain.clone();
            none.advanced.smart_match = Some(SmartMatchSettings::default());
            let cases = [
                ("no reference", none),
                ("switched off", with_match(plain.clone(), other, false, 0.5)),
                ("amount zero", with_match(plain.clone(), other, true, 0.0)),
                ("self-match", with_match(plain.clone(), track, true, 1.0)),
            ];
            for (label, settings) in cases {
                let coeffs = ChainCoeffs::from_settings(sr, &settings);
                assert_eq!(
                    coeffs, baseline,
                    "{preset:?} @ {sr}: {label} changed coefficients"
                );
                assert!(!coeffs.smart_match_active, "{label} must stay idle");
                let audio = render(sr, &settings, &input);
                assert!(
                    audio
                        .iter()
                        .zip(&baseline_audio)
                        .all(|(a, b)| a.to_bits() == b.to_bits()),
                    "{preset:?} @ {sr}: {label} changed rendered samples"
                );
            }
        }
    }
}

#[test]
fn an_active_match_changes_the_chain_and_switching_off_restores_it() {
    let track = track_shares();
    let reference = shares_from_db([-9.0, -4.0, -6.0, -11.0, -17.0, -18.0]);
    let plain = settings_for(Preset::Universal, track);
    let on = with_match(plain.clone(), reference, true, 0.5);
    let coeffs = ChainCoeffs::from_settings(48_000, &on);
    let baseline = ChainCoeffs::from_settings(48_000, &plain);
    assert_ne!(coeffs, baseline);
    // With the Advanced shelves idle, the corrective filter takes the warmth
    // slot and no extra stage runs.
    assert!(!coeffs.smart_match_active);
    assert_ne!(coeffs.warmth, baseline.warmth);
    assert_eq!(coeffs.presence_air, baseline.presence_air);
    let off = with_match(plain.clone(), reference, false, 0.5);
    assert_eq!(
        ChainCoeffs::from_settings(48_000, &off),
        ChainCoeffs::from_settings(48_000, &plain)
    );
}

/// Settings whose decision corrects by exactly `target` dB per band (when the
/// target's median is zero, no cap binds and nothing is in the dead zone).
fn settings_with_correction(preset: Preset, target: [f64; 6]) -> MasteringSettings {
    let track = typical();
    let mut levels = TYPICAL_SHAPE_DB;
    for (l, t) in levels.iter_mut().zip(target) {
        *l += t;
    }
    with_match(
        settings_for(preset, track),
        shares_from_db(levels),
        true,
        1.0,
    )
}

/// The three places the corrective filter can run: the idle warmth slot, the
/// idle air slot (Warmth in use), or its own stage (both sliders in use).
fn with_slot(mut settings: MasteringSettings, slot: usize) -> MasteringSettings {
    match slot {
        0 => {}
        1 => settings.advanced.warmth = Some(0.5),
        _ => {
            settings.advanced.warmth = Some(0.5);
            settings.advanced.presence_air = Some(0.25);
        }
    }
    settings
}

#[test]
fn the_corrective_filter_uses_an_idle_slot_before_its_own_stage() {
    let target = [2.0, -2.0, 0.0, 0.0, 0.0, 0.0];
    let base = settings_with_correction(Preset::Universal, target);
    for slot in 0..3 {
        let s = with_slot(base.clone(), slot);
        let mut plain = s.clone();
        plain.advanced.smart_match = None;
        let on = ChainCoeffs::from_settings(48_000, &s);
        let off = ChainCoeffs::from_settings(48_000, &plain);
        assert_eq!(on.smart_match_active, slot == 2, "slot case {slot}");
        match slot {
            0 => assert_ne!(on.warmth, off.warmth),
            1 => {
                assert_eq!(on.warmth, off.warmth, "the user's Warmth stays as set");
                assert_ne!(on.presence_air, off.presence_air);
            }
            _ => {
                assert_eq!(on.warmth, off.warmth);
                assert_eq!(on.presence_air, off.presence_air);
            }
        }
        assert_realized(48_000, &s, &format!("slot case {slot}"));
    }
}

fn assert_realized(sr: u32, settings: &MasteringSettings, label: &str) -> f64 {
    let decision = decide_for_settings(settings);
    let mut plain = settings.clone();
    plain.advanced.smart_match = None;
    let before = eq_cascade(&ChainCoeffs::from_settings(sr, &plain));
    let after = eq_cascade(&ChainCoeffs::from_settings(sr, settings));
    let change = band_power_change_db(sr, &before, &after, MEASURE_GRID);
    let mut worst = 0.0_f64;
    for b in 0..6 {
        let err = (change[b] - f64::from(decision.correction_db[b])).abs();
        worst = worst.max(err);
        assert!(
            err <= 0.5,
            "{label} @ {sr}: band {b} realized {:.3} dB vs intended {:.3} dB",
            change[b],
            decision.correction_db[b]
        );
    }
    worst
}

#[test]
fn g4_realization_is_within_half_a_db_for_cap_and_mixed_cases() {
    let mut worst = 0.0_f64;
    for sr in RATES {
        // Every single-band cap case.
        for band in 0..6 {
            for value in [f64::from(BOOST_CAP_DB), -f64::from(CUT_CAP_DB)] {
                let mut target = [0.0; 6];
                target[band] = value;
                for preset in [Preset::Universal, Preset::Oomph, Preset::Clarity] {
                    let s = settings_with_correction(preset.clone(), target);
                    let d = decide_for_settings(&s);
                    assert!(
                        (f64::from(d.correction_db[band]) - value).abs() < 1e-3,
                        "setup: band {band} wanted {value}, decided {:?}",
                        d.correction_db
                    );
                    worst = worst.max(assert_realized(
                        sr,
                        &s,
                        &format!("{preset:?} band {band} {value:+}"),
                    ));
                }
            }
        }
        // Twenty seeded mixed cases.
        let mut seed = 0x6A7E_u64;
        for case in 0..20 {
            let mut target = [0.0; 6];
            for t in target.iter_mut() {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u = (seed >> 11) as f64 / (1u64 << 53) as f64;
                *t = -4.0 + 7.0 * u;
            }
            // Rotate through the three filter placements.
            let s = with_slot(
                settings_with_correction(Preset::Universal, target),
                case % 3,
            );
            worst = worst.max(assert_realized(sr, &s, &format!("mixed case {case}")));
        }
    }
    eprintln!("G4 worst realization error: {worst:.4} dB");
}

#[test]
fn moved_band_frequencies_never_give_the_wrong_sign() {
    let ranges = EqBandFrequencies::RANGES;
    let mut seed = 0xBA4D_u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    for case in 0..40 {
        let pick = |u: f64, (lo, hi): (f32, f32)| lo + (hi - lo) * u as f32;
        let bands = EqBandFrequencies {
            sub_hz: pick(next(), ranges[0]),
            low_hz: pick(next(), ranges[1]),
            low_mid_hz: pick(next(), ranges[2]),
            mid_hz: pick(next(), ranges[3]),
            high_mid_hz: pick(next(), ranges[4]),
            high_hz: pick(next(), ranges[5]),
            sparkle_hz: pick(next(), ranges[6]),
        };
        let mut target = [0.0; 6];
        if case < 12 {
            target[case % 6] = if case < 6 { 3.0 } else { -4.0 };
        } else {
            for t in target.iter_mut() {
                *t = -4.0 + 7.0 * next();
            }
        }
        for sr in RATES {
            let mut s = settings_with_correction(Preset::Universal, target);
            s.eq_bands = bands;
            let decision = decide_for_settings(&s);
            let mut plain = s.clone();
            plain.advanced.smart_match = None;
            let change = band_power_change_db(
                sr,
                &eq_cascade(&ChainCoeffs::from_settings(sr, &plain)),
                &eq_cascade(&ChainCoeffs::from_settings(sr, &s)),
                MEASURE_GRID,
            );
            for b in 0..6 {
                let intended = f64::from(decision.correction_db[b]);
                if intended.abs() >= 0.5 {
                    assert_eq!(
                        change[b].signum(),
                        intended.signum(),
                        "case {case} @ {sr} with {bands:?}: band {b} realized {:.2} vs {intended:.2}",
                        change[b]
                    );
                }
            }
        }
    }
}

/// A two-second stereo signal with energy in every band.
fn tone_mix(sample_rate: u32) -> Vec<f32> {
    let freqs = [45.0_f32, 140.0, 450.0, 1400.0, 4000.0, 10_000.0];
    let amps = [0.20_f32, 0.25, 0.15, 0.10, 0.06, 0.03];
    let frames = sample_rate as usize * 2;
    let mut out = Vec::with_capacity(frames * 2);
    for n in 0..frames {
        let t = n as f32 / sample_rate as f32;
        let v: f32 = freqs
            .iter()
            .zip(amps)
            .map(|(f, a)| a * (std::f32::consts::TAU * f * t).sin())
            .sum();
        out.push(v);
        out.push(v * 0.95);
    }
    out
}

#[test]
fn g5_level_changes_of_twelve_db_leave_the_correction_unchanged() {
    let sr = 48_000;
    let audio = tone_mix(sr);
    let reference_audio: Vec<f32> = pink_stereo(sr as usize * 2);
    let measure = |samples: &[f32], gain_db: f32| {
        let g = 10f32.powf(gain_db / 20.0);
        let scaled: Vec<f32> = samples.iter().map(|s| s * g).collect();
        measure_shares(&scaled, sr, 2).expect("measurable")
    };
    let base_track = measure(&audio, 0.0);
    let base_reference = measure(&reference_audio, 0.0);
    let correction = |track: SpectralBalance6, reference: SpectralBalance6| {
        decide_for_settings(&with_match(
            settings_for(Preset::Universal, track),
            reference,
            true,
            1.0,
        ))
        .correction_db
    };
    let base = correction(base_track, base_reference);
    assert!(base.iter().any(|c| *c != 0.0), "the test needs a real gap");
    for gain in [12.0, -12.0] {
        for (track, reference) in [
            (measure(&audio, gain), base_reference),
            (base_track, measure(&reference_audio, gain)),
        ] {
            let moved = correction(track, reference);
            for b in 0..6 {
                assert!(
                    (moved[b] - base[b]).abs() <= 0.1,
                    "gain {gain}: band {b} {} vs {}",
                    moved[b],
                    base[b]
                );
            }
        }
    }
}

#[test]
fn album_is_inert_even_with_a_supplied_profile() {
    let track = track_shares();
    let reference = shares_from_db([-9.0, -4.0, -6.0, -11.0, -17.0, -18.0]);
    let mut settings = with_match(settings_for(Preset::Universal, track), reference, true, 1.0);
    // Album resolves to no profile, even when one was supplied (rule 7).
    yes_master_lib::profile_store::apply_resolved_profile(
        &mut settings,
        Some(profile(track)),
        true,
    );
    let coeffs = ChainCoeffs::from_settings(48_000, &settings);
    assert!(!coeffs.smart_match_active);
    let mut plain = settings.clone();
    plain.advanced.smart_match = None;
    assert_eq!(coeffs, ChainCoeffs::from_settings(48_000, &plain));
}

#[test]
fn smart_match_works_with_adapt_strength_at_zero() {
    let track = track_shares();
    let reference = shares_from_db([-9.0, -4.0, -6.0, -11.0, -17.0, -18.0]);
    let mut settings = with_match(settings_for(Preset::Universal, track), reference, true, 0.5);
    settings.advanced.adaptive_strength = Some(0.0);
    let d = decide_for_settings(&settings);
    assert!(d.changes_anything());
    let mut plain = settings.clone();
    plain.advanced.smart_match = None;
    assert_ne!(
        ChainCoeffs::from_settings(48_000, &settings),
        ChainCoeffs::from_settings(48_000, &plain)
    );
    // The user's knobs never change the decision (M14).
    let mut knobs = settings.clone();
    knobs.eq_low_db = 4.0;
    knobs.intensity = 0.9;
    knobs.preset = Preset::Loud;
    knobs.advanced.adaptive_strength = Some(1.0);
    assert_eq!(decide_for_settings(&knobs), d);
}

#[test]
fn a_saved_preset_never_contains_a_reference() {
    let dir = std::env::temp_dir().join(format!("ym-smart-match-presets-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("user_presets.json");
    let _ = std::fs::remove_file(&path);
    let settings = with_match(
        settings_for(Preset::Clarity, track_shares()),
        typical(),
        true,
        0.7,
    );
    let saved = yes_master_lib::settings::save_user_preset_at_path(
        &path,
        "Mine",
        PresetKind::Track,
        settings,
    )
    .unwrap();
    assert!(saved.settings.advanced.smart_match.is_none());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.contains("smart_match"), "the file must not carry it");

    // A hand-edited presets file cannot smuggle one in either.
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    value[0]["settings"]["advanced"]["smart_match"] = serde_json::json!({
        "reference": {"name": "x", "spectral_6": {
            "sub": 0.2, "low": 0.2, "low_mid": 0.2, "mid": 0.2, "presence": 0.1, "air": 0.1
        }},
        "enabled": true,
        "amount": 1.0
    });
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let listed = yes_master_lib::settings::read_presets(&path).unwrap();
    assert!(listed
        .iter()
        .all(|p| p.settings.advanced.smart_match.is_none()));
    let _ = std::fs::remove_dir_all(&dir);
}

fn project_with(settings: MasteringSettings) -> ProjectState {
    let id = TrackId("t1".into());
    let mut track_settings = HashMap::new();
    track_settings.insert(id.0.clone(), settings);
    ProjectState {
        schema_version: 1,
        mode: ProjectMode::Track,
        tracks: vec![ImportedTrack {
            id: id.clone(),
            path: "C:/music/t1.wav".into(),
            display_name: "t1".into(),
            source_format: "wav".into(),
            duration_seconds: Some(1.0),
            sample_rate: Some(44_100),
            channels: Some(2),
        }],
        track_order: vec![id],
        track_settings,
        album_intent: None,
        album_arc_kind: Default::default(),
        album_intensity: 1.0,
        album_title: String::new(),
        album_sample_rate: None,
        album_bit_depth: None,
        track_override_album: Vec::new(),
        selected_track_id: None,
        view_by_track_id: HashMap::new(),
        last_saved_iso: None,
    }
}

#[test]
fn a_project_round_trips_the_reference_and_amount_without_the_file() {
    let dir = std::env::temp_dir().join(format!("ym-smart-match-project-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("song.ams.json");
    let mut settings = with_match(
        settings_for(Preset::Tape, track_shares()),
        typical(),
        true,
        0.75,
    );
    // The profile is backend-resolved; it is not what the project keeps.
    settings.advanced.source_profile = None;
    yes_master_lib::project::write_session_atomic(&path, &project_with(settings.clone())).unwrap();
    let loaded = yes_master_lib::project::read_session(&path).unwrap();
    let state = loaded.track_settings["t1"].advanced.smart_match.clone();
    assert_eq!(state, settings.advanced.smart_match);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains(":\\\\") && !text.contains("C:/music/ref"),
        "no reference path is stored"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_project_without_a_reference_keeps_the_old_shape() {
    let dir = std::env::temp_dir().join(format!("ym-smart-match-shape-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("song.ams.json");
    let mut settings = settings_for(Preset::Universal, track_shares());
    settings.advanced.source_profile = None;
    yes_master_lib::project::write_session_atomic(&path, &project_with(settings)).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains("smart_match"),
        "a project with no Smart Match state must be written exactly as before"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
