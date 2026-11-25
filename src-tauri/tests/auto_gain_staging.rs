//! Auto gain staging (plan `docs/plans/2026-11-24-auto-gain-staging-execution-plan.md`),
//! Slices 2 and 3: the setting, the chain stages, export resolution and the
//! receipt. Each test names the contract rule it pins.

mod common;

#[allow(dead_code)]
#[path = "../src/sample_rate.rs"]
mod sample_rate;

use common::default_master_settings;
use yes_master_lib::dsp::{ChainCoeffs, MasteringChain};
use yes_master_lib::gain_staging::{
    level_in_db, receipt_digest, render_export, CLEAN_LIMIT_DB, MAX_PASSES, NOMINAL_LUFS,
    NOMINAL_PEAK_DBTP, TARGET_TOLERANCE_LU,
};
use yes_master_lib::types::{
    self, AlbumArc, AlbumArcKind, AlbumPlan, AutoGainStagingSettings, DeliveryProfile,
    LoudnessPriority, MasteringSettings, Preset, ResolvedGainStaging, SmartMatchReference,
    SmartMatchSettings, SpectralBalance6, GAIN_STAGING_RANGE_DB,
};

const FACTORY: [Preset; 8] = [
    Preset::Universal,
    Preset::Clarity,
    Preset::Tape,
    Preset::Spatial,
    Preset::Oomph,
    Preset::Warmth,
    Preset::Punch,
    Preset::Loud,
];

/// Deterministic stereo test music: pink-ish noise with drum-like bursts and
/// a slow swell, so the compressor, saturation and limiter all work.
fn music(rate: u32, seconds: f64, peak: f32) -> Vec<f32> {
    let frames = (f64::from(rate) * seconds) as usize;
    let mut out = Vec::with_capacity(frames * 2);
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
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

fn settings(preset: Preset, target: Option<f32>) -> MasteringSettings {
    let mut s = default_master_settings();
    s.preset = preset;
    s.delivery_profile = DeliveryProfile::Custom;
    s.advanced.lufs_offset_db = target;
    s.advanced.ceiling_dbtp = Some(-1.0);
    s.advanced.bit_depth = Some(24);
    s.advanced.target_sample_rate = Some(44_100);
    s
}

fn switched(mut s: MasteringSettings, priority: LoudnessPriority) -> MasteringSettings {
    s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
        enabled: true,
        priority,
    });
    s
}

fn resolved(level_in_db: f32, limiter_drive_db: f32) -> Option<ResolvedGainStaging> {
    Some(ResolvedGainStaging {
        level_in_db,
        limiter_drive_db,
        ..Default::default()
    })
}

fn album() -> AlbumPlan {
    AlbumPlan {
        title: "Album".into(),
        arc: AlbumArc::Preset {
            preset: AlbumArcKind::Cinematic,
        },
        tracks: Vec::new(),
        transitions: Vec::new(),
        intensity: 1.0,
        delivery_sample_rate: None,
        delivery_bit_depth: None,
    }
}

fn render(samples: &[f32], rate: u32, s: &MasteringSettings) -> Vec<f32> {
    let mut chain = MasteringChain::new(rate, 2, s);
    let mut out = samples.to_vec();
    chain.process_interleaved(&mut out, 2);
    chain.flush_render_tail(&mut out, 2);
    out
}

fn bits_equal(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

/// The export's own delivery after the chain: conversion to 44.1 kHz, then
/// the qualified finalizer, as `mastering_render_format_with_cancel` runs it.
fn deliver(rendered: Vec<f32>, rate: u32, target: f32) -> (Vec<f32>, f32) {
    let mut out = if rate != 44_100 {
        sample_rate::convert_interleaved(&rendered, rate, 44_100, 2).unwrap()
    } else {
        rendered
    };
    let p = yes_master_lib::output_protection::finalize(
        &mut out,
        44_100,
        2,
        24,
        Some(target),
        -1.0,
        None,
    )
    .unwrap();
    (out, p.lufs)
}

fn export(
    samples: &[f32],
    rate: u32,
    s: &mut MasteringSettings,
) -> Option<yes_master_lib::gain_staging::ExportOutcome> {
    render_export(samples, rate, 2, s, 44_100, 24, None, &|_| {}).unwrap()
}

// ---------------------------------------------------------------------------
// Rule 7 (identity) and rule 6 (activation)
// ---------------------------------------------------------------------------

#[test]
fn off_or_inert_is_bit_identical_for_every_preset() {
    let x = music(44_100, 2.0, 0.6);
    for preset in FACTORY {
        let base = settings(preset.clone(), Some(-11.0));
        let base_coeffs = ChainCoeffs::from_settings(44_100, &base);
        let base_out = render(&x, 44_100, &base);
        let mut variants = Vec::new();
        // Switch present but off, even with stale resolved gains.
        let mut off = base.clone();
        off.advanced.auto_gain_staging = Some(AutoGainStagingSettings::default());
        off.advanced.gain_staging_resolved = resolved(-3.0, 2.0);
        variants.push(("off", off));
        // On, nothing resolved (live audition until Slice 5).
        variants.push((
            "unresolved",
            switched(base.clone(), LoudnessPriority::Clean),
        ));
        // On, but resolved to exactly zero.
        let mut zero = switched(base.clone(), LoudnessPriority::Clean);
        zero.advanced.gain_staging_resolved = resolved(0.0, 0.0);
        variants.push(("zero", zero));
        // On with gains, but no loudness target.
        let mut no_target = switched(base.clone(), LoudnessPriority::Clean);
        no_target.advanced.lufs_offset_db = None;
        no_target.advanced.gain_staging_resolved = resolved(-3.0, 2.0);
        let mut no_target_base = base.clone();
        no_target_base.advanced.lufs_offset_db = None;
        // On with gains, in Album mode.
        let mut in_album = switched(base.clone(), LoudnessPriority::Clean);
        in_album.album = Some(album());
        in_album.advanced.gain_staging_resolved = resolved(-3.0, 2.0);
        let mut album_base = base.clone();
        album_base.album = Some(album());
        for (name, v) in &variants {
            assert_eq!(ChainCoeffs::from_settings(44_100, v), base_coeffs, "{name}");
            assert!(bits_equal(&render(&x, 44_100, v), &base_out), "{name}");
            assert!(!ChainCoeffs::from_settings(44_100, v).limiter_drive_active);
        }
        for (name, v, b) in [
            ("no target", &no_target, &no_target_base),
            ("album", &in_album, &album_base),
        ] {
            assert_eq!(
                ChainCoeffs::from_settings(44_100, v),
                ChainCoeffs::from_settings(44_100, b),
                "{name}"
            );
            assert!(
                bits_equal(&render(&x, 44_100, v), &render(&x, 44_100, b)),
                "{name}"
            );
        }
    }
}

#[test]
fn activation_needs_the_switch_a_target_track_master_and_finite_gains() {
    let on = |mut s: MasteringSettings, r: Option<ResolvedGainStaging>| {
        s = switched(s, LoudnessPriority::Clean);
        s.advanced.gain_staging_resolved = r;
        s
    };
    let base = settings(Preset::Universal, Some(-9.0));
    assert!(on(base.clone(), resolved(1.0, 2.0))
        .active_gain_staging()
        .is_some());
    let mut off = on(base.clone(), resolved(1.0, 2.0));
    off.advanced.auto_gain_staging = None;
    assert!(off.active_gain_staging().is_none());
    let mut no_target = on(base.clone(), resolved(1.0, 2.0));
    no_target.advanced.lufs_offset_db = None;
    assert!(no_target.active_gain_staging().is_none());
    let mut in_album = on(base.clone(), resolved(1.0, 2.0));
    in_album.album = Some(album());
    assert!(in_album.active_gain_staging().is_none());
    assert!(on(base.clone(), resolved(f32::NAN, 0.0))
        .active_gain_staging()
        .is_none());
    assert!(on(base, None).active_gain_staging().is_none());
}

// ---------------------------------------------------------------------------
// Rules 2 and 3: where the gains act in the chain
// ---------------------------------------------------------------------------

#[test]
fn level_in_joins_the_input_gain_and_the_drive_sits_before_the_limiter() {
    let base = settings(Preset::Tape, Some(-9.0));
    let mut on = switched(base.clone(), LoudnessPriority::Clean);
    on.advanced.gain_staging_resolved = resolved(-3.0, 2.0);
    let b = ChainCoeffs::from_settings(48_000, &base);
    let c = ChainCoeffs::from_settings(48_000, &on);
    let base_db = 20.0 * b.input_gain_lin.log10();
    let on_db = 20.0 * c.input_gain_lin.log10();
    assert!((on_db - base_db + 3.0).abs() < 1e-4, "{base_db} {on_db}");
    assert!(c.limiter_drive_active);
    assert!((c.limiter_drive_lin - 10.0_f32.powf(0.1)).abs() < 1e-6);
    // Everything else is the Style's own coefficient set.
    let mut c2 = c;
    c2.input_gain_lin = b.input_gain_lin;
    c2.limiter_drive_lin = 1.0;
    c2.limiter_drive_active = false;
    assert_eq!(c2, b);
    // The ±24 dB range bounds both gains.
    on.advanced.gain_staging_resolved = resolved(-60.0, 60.0);
    let c = ChainCoeffs::from_settings(48_000, &on);
    assert!((20.0 * c.input_gain_lin.log10() - (base_db - GAIN_STAGING_RANGE_DB)).abs() < 1e-3);
    assert!((20.0 * c.limiter_drive_lin.log10() - GAIN_STAGING_RANGE_DB).abs() < 1e-3);
}

#[test]
fn the_reference_level_gives_exactly_zero_level_in() {
    assert_eq!(level_in_db(NOMINAL_LUFS, NOMINAL_PEAK_DBTP), Some(0.0));
}

/// A6 on the real file: the slow fixture lane with `AMS_REAL_FIXTURE_PATH` set
/// to the owner's reference song.
#[test]
fn the_reference_song_measures_to_exactly_zero_level_in() {
    let enabled = std::env::var("AMS_RUN_REAL_FIXTURE").is_ok_and(|v| !v.is_empty());
    let Some(path) = std::env::var_os("AMS_REAL_FIXTURE_PATH").map(std::path::PathBuf::from) else {
        eprintln!("skipping: set AMS_RUN_REAL_FIXTURE and AMS_REAL_FIXTURE_PATH");
        return;
    };
    let is_reference = path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "It\u{2019}s a coat-original-test.wav");
    if !enabled || !is_reference {
        eprintln!("skipping: the fixture is not the reference song");
        return;
    }
    let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
    let level =
        yes_master_lib::analysis::measure_source_level(&pcm.samples, pcm.sample_rate, pcm.channels)
            .unwrap();
    assert_eq!(level.lufs_integrated, NOMINAL_LUFS);
    assert_eq!(level.true_peak_dbtp, NOMINAL_PEAK_DBTP);
    assert_eq!(
        yes_master_lib::gain_staging::level_in_for(&level),
        Some(0.0)
    );
}

#[test]
fn smart_matchs_decision_is_unchanged_with_the_switch_on() {
    let shares = |air: f32| SpectralBalance6 {
        sub: 0.2,
        low: 0.25,
        low_mid: 0.2,
        mid: 0.15,
        presence: 0.15 - air,
        air: 0.05 + air,
    };
    let x = music(44_100, 2.0, 0.5);
    let track = yes_master_lib::smart_match::measure_shares(&x, 44_100, 2).unwrap();
    let mut base = settings(Preset::Universal, Some(-9.0));
    base.advanced.source_profile = Some(yes_master_lib::types::SourceProfile {
        spectral_6: track,
        dynamic_range_p95_p10_db: 8.0,
        dynamic_range_lu: 6.0,
        stereo_correlation: Some(0.8),
        stereo_width: 0.3,
    });
    base.advanced.smart_match = Some(SmartMatchSettings {
        reference: Some(SmartMatchReference {
            name: "Ref".into(),
            spectral_6: shares(0.04),
        }),
        enabled: true,
        amount: 0.5,
    });
    let off = yes_master_lib::smart_match::decide_for_settings(&base);
    for priority in [LoudnessPriority::Clean, LoudnessPriority::ReachTarget] {
        let mut on = switched(base.clone(), priority);
        on.advanced.gain_staging_resolved = resolved(-2.5, 3.0);
        assert_eq!(yes_master_lib::smart_match::decide_for_settings(&on), off);
        let c_on = ChainCoeffs::from_settings(44_100, &on);
        let c_off = ChainCoeffs::from_settings(44_100, &base);
        assert_eq!(c_on.smart_match_bass, c_off.smart_match_bass);
        assert_eq!(c_on.smart_match_slot, c_off.smart_match_slot);
    }
}

// ---------------------------------------------------------------------------
// Slice 3: export resolution (rules 3, 4, 5, 6 and 9)
// ---------------------------------------------------------------------------

/// Rule 9: the delivered audio is exactly what a chain built from the
/// resolved gains renders, through the export's own conversion and finalizer.
#[test]
fn the_export_equals_a_chain_built_from_the_resolved_gains() {
    for (rate, peak, target) in [
        (48_000, 0.2_f32, -9.0_f32),
        (44_100, 0.99, -14.0),
        (44_100, 0.3, -11.0),
    ] {
        let x = music(rate, 3.0, peak);
        let mut s = switched(
            settings(Preset::Universal, Some(target)),
            LoudnessPriority::Clean,
        );
        let outcome = export(&x, rate, &mut s).expect("active");
        let r = s.advanced.gain_staging_resolved.expect("resolved");
        let mut built = s.clone();
        built.advanced.gain_staging_resolved = Some(r);
        let (expected, lufs) = deliver(render(&x, rate, &built), rate, target);
        assert!(
            bits_equal(&outcome.samples, &expected),
            "rate {rate} target {target}"
        );
        assert_eq!(outcome.measured.lufs, lufs);
    }
}

/// Rule 3: when the limiter can stay idle the drive is exactly the
/// difference, in one pass, and the limiter does nothing.
#[test]
fn the_drive_is_the_difference_when_the_limiter_can_stay_idle() {
    let x = music(44_100, 3.0, 0.99);
    let mut s = switched(
        settings(Preset::Universal, Some(-16.0)),
        LoudnessPriority::Clean,
    );
    let outcome = export(&x, 44_100, &mut s).expect("active");
    let r = s.advanced.gain_staging_resolved.unwrap();
    assert_eq!(outcome.passes, 1);
    assert_eq!(r.limiter_load_db, 0.0);
    assert!((outcome.measured.lufs + 16.0).abs() <= TARGET_TOLERANCE_LU);
}

/// Rules 4 and 5: never above the target; Clean never takes more than the
/// clean limit, and stops there short of a target it cannot reach.
#[test]
fn clean_never_exceeds_the_clean_limit_or_lands_above_the_target() {
    let x = music(44_100, 3.0, 0.15);
    for target in [-14.0_f32, -11.0, -9.0, -6.0] {
        let mut s = switched(
            settings(Preset::Clarity, Some(target)),
            LoudnessPriority::Clean,
        );
        let outcome = export(&x, 44_100, &mut s).expect("active");
        let r = s.advanced.gain_staging_resolved.unwrap();
        assert!(
            outcome.measured.lufs <= target + TARGET_TOLERANCE_LU,
            "{target}"
        );
        assert!(
            r.limiter_load_db <= CLEAN_LIMIT_DB + 0.01,
            "{target}: {r:?}"
        );
        let reached = outcome.measured.lufs >= target - TARGET_TOLERANCE_LU;
        assert!(
            reached || r.stopped_at_clean_limit || outcome.passes == MAX_PASSES,
            "{target}: {r:?}"
        );
        assert!(outcome.samples.iter().all(|v| v.is_finite()));
        assert!(outcome.measured.true_peak_dbtp <= -1.0);
    }
}

/// A4: Reach target goes past the clean limit where Clean stops at it.
#[test]
fn reach_target_goes_past_the_clean_limit() {
    let x = music(44_100, 3.0, 0.1);
    let mut clean = switched(
        settings(Preset::Universal, Some(-6.0)),
        LoudnessPriority::Clean,
    );
    let clean_out = export(&x, 44_100, &mut clean).unwrap();
    let c = clean.advanced.gain_staging_resolved.unwrap();
    assert!(c.stopped_at_clean_limit, "{c:?}");
    let mut reach = switched(
        settings(Preset::Universal, Some(-6.0)),
        LoudnessPriority::ReachTarget,
    );
    let reach_out = export(&x, 44_100, &mut reach).unwrap();
    let r = reach.advanced.gain_staging_resolved.unwrap();
    assert!(!r.stopped_at_clean_limit);
    assert!(r.limiter_load_db > CLEAN_LIMIT_DB, "{r:?}");
    assert!(reach_out.measured.lufs > clean_out.measured.lufs + 0.5);
    assert!(reach_out.measured.lufs <= -6.0 + TARGET_TOLERANCE_LU);
}

/// Rule 6: switch off, no target, Album and an unusable measurement are
/// inert; the caller then renders exactly as today.
#[test]
fn inert_cases_do_not_resolve() {
    let x = music(44_100, 2.0, 0.5);
    let base = settings(Preset::Universal, Some(-9.0));
    let mut off = base.clone();
    assert!(export(&x, 44_100, &mut off).is_none());
    off.advanced.auto_gain_staging = Some(AutoGainStagingSettings::default());
    assert!(export(&x, 44_100, &mut off).is_none());
    let mut no_target = switched(base.clone(), LoudnessPriority::Clean);
    no_target.advanced.lufs_offset_db = None;
    assert!(export(&x, 44_100, &mut no_target).is_none());
    let mut in_album = switched(base.clone(), LoudnessPriority::Clean);
    in_album.album = Some(album());
    assert!(export(&x, 44_100, &mut in_album).is_none());
    let mut silent = switched(base, LoudnessPriority::Clean);
    assert!(export(&vec![0.0; 88_200], 44_100, &mut silent).is_none());
    assert!(silent.advanced.gain_staging_resolved.is_none());
}

/// E8: hostile sources never panic, never produce non-finite audio, and
/// resolve to inert or to a bounded gain.
#[test]
fn hostile_sources_are_inert_or_bounded() {
    let clipped: Vec<f32> = (0..88_200)
        .map(|i| if (i / 40) % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let mono_as_stereo: Vec<f32> = music(44_100, 2.0, 0.5)
        .chunks_exact(2)
        .flat_map(|f| [f[0], f[0]])
        .collect();
    let very_quiet: Vec<f32> = music(44_100, 2.0, 1.0e-4);
    let cases: [(&str, Vec<f32>); 5] = [
        ("silent", vec![0.0; 88_200]),
        ("very short", music(44_100, 0.05, 0.5)),
        ("clipped", clipped),
        ("mono", mono_as_stereo),
        ("extremely quiet", very_quiet),
    ];
    for priority in [LoudnessPriority::Clean, LoudnessPriority::ReachTarget] {
        for (name, x) in &cases {
            let mut s = switched(settings(Preset::Loud, Some(-9.0)), priority);
            match export(x, 44_100, &mut s) {
                None => assert!(s.advanced.gain_staging_resolved.is_none(), "{name}"),
                Some(outcome) => {
                    let r = s.advanced.gain_staging_resolved.unwrap();
                    assert!(outcome.samples.iter().all(|v| v.is_finite()), "{name}");
                    assert!(r.level_in_db.abs() <= GAIN_STAGING_RANGE_DB, "{name}");
                    assert!(r.limiter_drive_db.abs() <= GAIN_STAGING_RANGE_DB, "{name}");
                    assert!(r.limiter_load_db.is_finite(), "{name}");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Receipt (locked copy, rule 9)
// ---------------------------------------------------------------------------

#[test]
fn receipt_lines_follow_the_locked_copy() {
    let base = settings(Preset::Universal, Some(-9.0));
    assert_eq!(receipt_digest(&base), None);
    let mut off = base.clone();
    off.advanced.auto_gain_staging = Some(AutoGainStagingSettings::default());
    assert_eq!(
        receipt_digest(&off).as_deref(),
        Some("Auto gain staging: off")
    );
    let mut on = switched(base.clone(), LoudnessPriority::Clean);
    assert_eq!(
        receipt_digest(&on).as_deref(),
        Some("Auto gain staging: off")
    );
    on.advanced.gain_staging_resolved = Some(ResolvedGainStaging {
        level_in_db: -3.26,
        limiter_drive_db: 0.0,
        limiter_load_db: 0.0,
        stopped_at_clean_limit: false,
        stopped_by_peak_check: false,
    });
    assert_eq!(
        receipt_digest(&on).as_deref(),
        Some("Auto gain staging: level in \u{2212}3.3 dB \u{b7} limiter drive 0.0 dB \u{b7} limiting 0.0 dB")
    );
    assert_eq!(yes_master_lib::gain_staging::receipt_target_lufs(&on), None);
    on.advanced.gain_staging_resolved = Some(ResolvedGainStaging {
        level_in_db: 2.1,
        limiter_drive_db: 4.5,
        limiter_load_db: 6.0,
        stopped_at_clean_limit: true,
        stopped_by_peak_check: false,
    });
    assert_eq!(
        receipt_digest(&on).as_deref(),
        Some("Auto gain staging: level in +2.1 dB \u{b7} limiter drive +4.5 dB \u{b7} limiting 6.0 dB \u{b7} stopped at the clean limit")
    );
    assert_eq!(
        yes_master_lib::gain_staging::receipt_target_lufs(&on),
        Some(-9.0)
    );
}

fn write_wav(path: &std::path::Path, rate: u32, samples: &[f32]) {
    let mut w = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for s in samples {
        w.write_sample(*s).unwrap();
    }
    w.finalize().unwrap();
}

/// A real export carries the line and the stopped-at target from the render;
/// a song that never used the feature writes neither key, and switching it
/// off renders exactly as today.
#[test]
fn a_real_export_carries_the_receipt_line() {
    let dir = std::env::temp_dir().join(format!("ym-ags-receipt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("quiet.wav");
    write_wav(&source, 44_100, &music(44_100, 4.0, 0.08));
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let run = |s: &MasteringSettings, name: &str| {
        yes_master_lib::engine::mastering_render(
            yes_master_lib::TrackId(name.into()),
            &source,
            s,
            &out,
            yes_master_lib::RenderKind::Master,
        )
        .unwrap()
    };
    let base = settings(Preset::Universal, Some(-6.0));
    let plain = run(&base, "plain");
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("auto_gain_staging"), "{json}");
    let mut off = base.clone();
    off.advanced.auto_gain_staging = Some(AutoGainStagingSettings::default());
    let off_job = run(&off, "off");
    let m = off_job.measurements.unwrap();
    assert_eq!(
        m.auto_gain_staging_digest.as_deref(),
        Some("Auto gain staging: off")
    );
    let plain_m = plain.measurements.unwrap();
    assert_eq!(m.lufs_integrated, plain_m.lufs_integrated);
    assert_eq!(m.true_peak_dbtp, plain_m.true_peak_dbtp);
    let on = switched(base, LoudnessPriority::Clean);
    let on_job = run(&on, "on");
    let m = on_job.measurements.unwrap();
    let line = m.auto_gain_staging_digest.clone().unwrap();
    assert!(line.starts_with("Auto gain staging: level in +"), "{line}");
    assert!(
        line.ends_with(" \u{b7} stopped at the clean limit"),
        "{line}"
    );
    assert_eq!(m.auto_gain_staging_target_lufs, Some(-6.0));
    assert!(m.lufs_integrated > plain_m.lufs_integrated + 1.0, "{m:?}");
    assert!(m.true_peak_dbtp <= -1.0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_settings_are_omitted_when_absent_and_round_trip_when_present() {
    let base = settings(Preset::Universal, Some(-9.0));
    let json = serde_json::to_value(&base).unwrap();
    assert!(json["advanced"].get("auto_gain_staging").is_none());
    assert!(json["advanced"].get("gain_staging_resolved").is_none());
    let mut on = switched(base, LoudnessPriority::ReachTarget);
    on.advanced.gain_staging_resolved = resolved(1.0, 2.0);
    let json = serde_json::to_value(&on).unwrap();
    assert_eq!(
        json["advanced"]["auto_gain_staging"],
        serde_json::json!({"enabled": true, "priority": "reach_target"})
    );
    assert!(json["advanced"].get("gain_staging_resolved").is_none());
    let back: MasteringSettings = serde_json::from_value(json).unwrap();
    assert_eq!(
        back.advanced.auto_gain_staging,
        on.advanced.auto_gain_staging
    );
    assert!(back.advanced.gain_staging_resolved.is_none());
    // A frontend cannot smuggle resolved gains in.
    let mut hostile = serde_json::to_value(&on).unwrap();
    hostile["advanced"]["gain_staging_resolved"] =
        serde_json::json!({"level_in_db": 20.0, "limiter_drive_db": 20.0});
    let parsed: MasteringSettings = serde_json::from_value(hostile).unwrap();
    assert!(parsed.advanced.gain_staging_resolved.is_none());
}

// ---------------------------------------------------------------------------
// Checkpoint B item 3: `Limiter::detector_level` is pinned to the limiter
// ---------------------------------------------------------------------------

/// Signals that stress the limiter's detector in different ways: broadband
/// noise, a near-Nyquist tone (intersample peaks well above the samples),
/// sparse impulses, and a hard-clipped square-ish wave.
fn detector_signal(kind: usize, rate: u32, channels: usize) -> Vec<f32> {
    let frames = rate as usize / 2;
    let mut seed = 0x9e37_79b9_u32 ^ (kind as u32 * 7919 + channels as u32);
    let mut rnd = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    };
    let mut out = vec![0.0_f32; frames * channels];
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        for c in 0..channels {
            let phase = 0.37 * c as f32;
            out[i * channels + c] = match kind {
                0 => 0.5 * rnd(),
                1 => 0.7 * (std::f32::consts::TAU * 0.45 * rate as f32 * t + phase).sin(),
                2 => {
                    if (i + 97 * c) % 2_411 == 0 {
                        0.9
                    } else {
                        0.01 * rnd()
                    }
                }
                _ => (3.0 * (std::f32::consts::TAU * 1_013.0 * t + phase).sin()).clamp(-0.8, 0.8),
            };
        }
    }
    out
}

/// The limiter's largest gain reduction over `x` scaled by `gain`, with the
/// limiter built exactly as the chain builds it.
fn limiter_reduction_db(x: &[f32], rate: u32, channels: usize, gain: f32) -> f32 {
    let mut s = settings(Preset::Universal, Some(-9.0));
    s.advanced.ceiling_dbtp = Some(-1.0);
    let mut limiter = MasteringChain::fresh_limiter(rate, channels, &s);
    let mut frame = vec![0.0_f32; channels];
    let mut reduction = 0.0_f32;
    for chunk in x.chunks_exact(channels) {
        for (f, v) in frame.iter_mut().zip(chunk) {
            *f = v * gain;
        }
        limiter.process_frame_inplace(&mut frame);
        reduction = reduction.max(limiter.gain_reduction_db());
    }
    reduction
}

#[test]
fn the_limiter_level_helper_matches_the_limiters_own_detection() {
    let ceiling = 10.0_f32.powf(-1.0 / 20.0);
    for rate in [44_100_u32, 48_000, 96_000] {
        for channels in [1_usize, 2, 6] {
            for kind in 0..4 {
                let x = detector_signal(kind, rate, channels);
                let level = yes_master_lib::dsp::Limiter::detector_level(&x, channels);
                assert!(level > 0.0);
                let at = ceiling / level;
                let case = format!("rate {rate}, {channels} ch, signal {kind}");
                // Just under the level the limiter reacts to: nothing reduced.
                assert_eq!(
                    limiter_reduction_db(&x, rate, channels, at * 0.9999),
                    0.0,
                    "{case}"
                );
                // Exactly at it: nothing beyond float rounding.
                let exact = limiter_reduction_db(&x, rate, channels, at);
                assert!(exact < 1e-4, "{case}: {exact} dB at the ceiling");
                // 0.1% (0.0087 dB) over it: the limiter takes about that much.
                let over = limiter_reduction_db(&x, rate, channels, at * 1.001);
                assert!(
                    (over - 0.008_68).abs() < 2e-3,
                    "{case}: {over} dB just over the ceiling"
                );
                // A full dB over it: about a dB.
                let db = limiter_reduction_db(&x, rate, channels, at * 10.0_f32.powf(0.05));
                assert!((db - 1.0).abs() < 2e-3, "{case}: {db} dB one dB over");
            }
        }
    }
}
