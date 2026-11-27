//! Slice 5 regressions for auto gain staging during audition.
use super::*;
use crate::audio::tests::settings_with_intensity;
use crate::audio::{
    try_spawn_lufs_preview_worker, worker_readout, AudioCommand, DecodedCacheEntry,
    GainStagingOutcome, GainStagingWork, PreviewWorkRequest, PreviewWorkerGate,
};
use crate::decode::DecodedPcm;
use crate::export_format::ExportEncoding;
use crate::types::{AutoGainStagingSettings, LoudnessPriority, Preset};
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};

/// A song a −9 target has to limit: noise bursts over a low tone, six
/// seconds of 48 kHz stereo.
fn song() -> PlaybackPcm {
    let rate = 48_000_u32;
    let frames = rate as usize * 6;
    let mut seed = 0x2545_f491_u32;
    let mut samples = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let noise = (seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
        let t = i as f32 / rate as f32;
        let burst = if (i / 6_000) % 4 == 0 { 0.35 } else { 0.04 };
        let tone = 0.05 * (std::f32::consts::TAU * 110.0 * t).sin();
        samples.push(burst * noise + tone);
        samples.push(burst * noise * 0.8 + tone);
    }
    DecodedPcm {
        samples,
        sample_rate: rate,
        channels: 2,
    }
    .into()
}

/// Band-limited, music-like content for the estimate: low-passed noise
/// bursts (a one-pole at about 3 kHz) over two tones. White noise is not
/// used here: its energy up to Nyquist is exactly where the estimate's
/// low-pass peak reading is known to read low (see the Slice 8 note).
fn music_like() -> PlaybackPcm {
    let rate = 48_000_u32;
    let frames = rate as usize * 6;
    let mut seed = 0x1234_5679_u32;
    let a = (-std::f32::consts::TAU * 3_000.0 / rate as f32).exp();
    let (mut l, mut r) = (0.0_f32, 0.0_f32);
    let mut samples = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let noise = (seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
        l = a * l + (1.0 - a) * noise;
        r = a * r + (1.0 - a) * noise * 0.9;
        let t = i as f32 / rate as f32;
        let burst = if (i / 6_000) % 4 == 0 { 2.2 } else { 0.3 };
        let tones = 0.08 * (std::f32::consts::TAU * 110.0 * t).sin()
            + 0.05 * (std::f32::consts::TAU * 440.0 * t).sin();
        samples.push(burst * l + tones);
        samples.push(burst * r + tones);
    }
    DecodedPcm {
        samples,
        sample_rate: rate,
        channels: 2,
    }
    .into()
}

fn switched_on(target: f32, priority: LoudnessPriority) -> MasteringSettings {
    let mut s = settings_with_intensity(0.5);
    s.preset = Preset::Universal;
    s.advanced.lufs_offset_db = Some(target);
    s.advanced.ceiling_dbtp = Some(-1.0);
    s.advanced.auto_gain_staging = Some(AutoGainStagingSettings {
        enabled: true,
        priority,
    });
    crate::audio::output_route::preview_settings(&s, 48_000, ExportEncoding::default())
}

fn analysis() -> SourceLevel {
    SourceLevel {
        lufs_integrated: -20.0,
        lra: 8.0,
        true_peak_dbtp: -6.0,
    }
}

fn source(name: &str) -> (PathBuf, Option<SystemTime>) {
    (
        PathBuf::from(format!("/virtual/auto-gain-staging/{name}.wav")),
        None,
    )
}

#[test]
fn off_and_inert_settings_apply_nothing() {
    let (path, mtime) = source("inert");
    let src = Some((path.as_path(), mtime));
    let mut off = switched_on(-9.0, LoudnessPriority::Clean);
    off.advanced.auto_gain_staging = None;
    assert_eq!(
        plan(&off, false, Some(analysis()), src, None),
        LivePlan::Inert(None)
    );
    off.advanced.auto_gain_staging = Some(AutoGainStagingSettings::default());
    assert_eq!(
        plan(&off, false, Some(analysis()), src, None),
        LivePlan::Inert(None)
    );

    let on = switched_on(-9.0, LoudnessPriority::Clean);
    assert_eq!(
        plan(&on, true, Some(analysis()), src, None),
        LivePlan::Inert(Some(AutoGainStagingReadout::Album))
    );
    let mut no_target = on.clone();
    no_target.advanced.lufs_offset_db = None;
    no_target.delivery_profile = crate::types::DeliveryProfile::Custom;
    assert_eq!(
        plan(&no_target, false, Some(analysis()), src, None),
        LivePlan::Inert(Some(AutoGainStagingReadout::NoTarget))
    );
    let silent = SourceLevel {
        lufs_integrated: -90.0,
        lra: 0.0,
        true_peak_dbtp: -90.0,
    };
    assert_eq!(
        plan(&on, false, Some(silent), src, None),
        LivePlan::Inert(Some(AutoGainStagingReadout::Unusable))
    );
    let mut chained = on.clone();
    chained.advanced.gain_staging_resolved = Some(ResolvedGainStaging {
        level_in_db: 5.0,
        limiter_drive_db: 5.0,
        ..Default::default()
    });
    assert_eq!(apply(&LivePlan::Inert(None), &mut chained), None);
    assert_eq!(chained.advanced.gain_staging_resolved, None);
}

/// Contract rule 8: level in applies at once from the analysis, the drive
/// holds and never rises on an unverified value, and the state is never
/// settled while pending.
#[test]
fn pending_applies_level_in_at_once_and_never_raises_the_drive() {
    let (path, mtime) = source("pending");
    let src = Some((path.as_path(), mtime));
    let on = switched_on(-9.0, LoudnessPriority::Clean);
    let expected_level_in = crate::gain_staging::level_in_for(&analysis()).unwrap();
    for held_drive in [-6.0_f32, -0.5, 0.0, 0.7, 3.0, 9.5] {
        let held = ResolvedGainStaging {
            level_in_db: 1.25,
            limiter_drive_db: held_drive,
            limiter_load_db: 4.0,
            stopped_at_clean_limit: true,
            stopped_by_peak_check: false,
        };
        let LivePlan::Pending { key, pending } =
            plan(&on, false, Some(analysis()), src, Some(held))
        else {
            panic!("no resolution exists yet");
        };
        assert_eq!(key, Key::new(&path, mtime, &on).unwrap());
        assert_eq!(pending.level_in_db, expected_level_in, "level in at once");
        assert_eq!(pending.limiter_drive_db, held_drive, "the drive holds");
        assert!(!pending.stopped_at_clean_limit && !pending.stopped_by_peak_check);
        let mut chained = on.clone();
        let readout = apply(&LivePlan::Pending { key, pending }, &mut chained);
        assert_eq!(readout, Some(AutoGainStagingReadout::Measuring));
        assert_eq!(chained.active_gain_staging(), Some(pending));
        // Without the analysis, the chain keeps the level in it applies.
        let LivePlan::Pending { pending, .. } = plan(&on, false, None, src, Some(held)) else {
            panic!("pending");
        };
        assert_eq!(pending.level_in_db, 1.25);
        assert_eq!(pending.limiter_drive_db, held_drive);
    }
    // Nothing held (the switch was off, or a new song): no drive at all.
    let LivePlan::Pending { pending, .. } = plan(&on, false, Some(analysis()), src, None) else {
        panic!("pending");
    };
    assert_eq!(pending.limiter_drive_db, 0.0);
    assert_eq!(pending.level_in_db, expected_level_in);
}

/// Audition resolves exactly what the export resolves (A9): the same gains,
/// bit for bit, and the same delivered loudness.
#[test]
fn audition_resolves_exactly_what_the_export_resolves() {
    let pcm = song();
    for (name, target, priority) in [
        ("export-clean-9", -9.0, LoudnessPriority::Clean),
        ("export-reach-9", -9.0, LoudnessPriority::ReachTarget),
        ("export-clean-14", -14.0, LoudnessPriority::Clean),
    ] {
        let (path, mtime) = source(name);
        let settings = switched_on(target, priority);
        let key = Key::new(&path, mtime, &settings).unwrap();
        let cancel = AtomicBool::new(false);
        let entry = resolve_cached(&key, &pcm, &settings, None, &cancel)
            .unwrap()
            .expect("active");
        assert_eq!(
            entry.audition, entry.exact.gains,
            "no estimate: the exact gains"
        );
        let live = entry.exact;

        // The export's own path, as `mastering_render_format_with_cancel`
        // calls it for the default WAV encoding.
        let encoding = ExportEncoding::default();
        let mut export = settings.clone();
        export.volume_match = false;
        let rate = encoding.delivery_rate(export.effective_sample_rate(pcm.sample_rate));
        let bits = encoding.delivery_bits(export.effective_bit_depth());
        let outcome = crate::gain_staging::render_export(
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels,
            &mut export,
            rate,
            bits,
            None,
            &|_| {},
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            export.advanced.gain_staging_resolved,
            Some(live.gains),
            "{name}"
        );
        assert_eq!(outcome.measured.lufs, live.delivered_lufs, "{name}");
        assert_eq!(live.target_lufs, target);
        assert!(
            live.gains.limiter_drive_db != 0.0,
            "{name}: the song needs drive"
        );
        // Reused, not re-resolved, and settled for the live chain.
        assert_eq!(lookup(&key), Some(entry));
        let src = Some((path.as_path(), mtime));
        assert_eq!(
            plan(&settings, false, Some(analysis()), src, None),
            LivePlan::Settled(entry)
        );
        let mut toggled = settings.clone();
        toggled.volume_match = !toggled.volume_match;
        assert_eq!(
            plan(&toggled, false, Some(analysis()), src, None),
            LivePlan::Settled(entry),
            "Volume Match does not change the resolution"
        );
        let mut louder = settings.clone();
        louder.advanced.lufs_offset_db = Some(target + 1.0);
        assert!(matches!(
            plan(&louder, false, Some(analysis()), src, None),
            LivePlan::Pending { .. }
        ));
        let resaved = Some((path.as_path(), Some(SystemTime::UNIX_EPOCH)));
        assert!(matches!(
            plan(&settings, false, Some(analysis()), resaved, None),
            LivePlan::Pending { .. }
        ));
    }
}

/// Wait for a worker's resolution to reach the shared cache (it is stored
/// before any message is sent).
fn wait_for_resolution(key: &Key) -> LiveEntry {
    let start = std::time::Instant::now();
    loop {
        if let Some(hit) = lookup(key) {
            return hit;
        }
        assert!(start.elapsed().as_secs() < 120, "no resolution");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// The worker plays a fast estimate first (Slice 8), then resolves exactly;
/// Volume Match and the landing then measure the chain audition plays, so
/// Volume Match still compares at matched loudness.
#[test]
fn the_worker_resolves_first_and_measures_the_resolved_chain() {
    let pcm = song();
    let (path, mtime) = source("worker");
    let mut settings = switched_on(-9.0, LoudnessPriority::Clean);
    settings.volume_match = true;
    let source_lufs =
        crate::engine::measure_integrated_lufs(&pcm.samples, pcm.sample_rate, pcm.channels)
            .unwrap();
    settings.source_lufs_integrated = Some(source_lufs);
    let key = Key::new(&path, mtime, &settings).unwrap();
    let entry = DecodedCacheEntry {
        canonical_path: path.clone(),
        mtime,
        pcm: pcm.clone(),
    };
    let (tx, rx) = mpsc::channel::<AudioCommand>();
    let mut pending = settings.clone();
    pending.advanced.gain_staging_resolved = Some(ResolvedGainStaging {
        level_in_db: 1.0,
        ..Default::default()
    });
    assert!(try_spawn_lufs_preview_worker(
        Some(&entry),
        PreviewWorkRequest {
            settings: pending,
            generation: 5,
            track_epoch: 3,
            landing_enabled: true,
            device_rate: 48_000,
            gain_staging: Some(GainStagingWork {
                key: key.clone(),
                analysis: None,
            }),
        },
        &tx,
        &Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
    ));
    // First, the estimate: played at once, never finished, never measured.
    let estimate = match rx
        .recv_timeout(std::time::Duration::from_secs(120))
        .unwrap()
    {
        AudioCommand::PreviewLandingReady {
            settings: playing,
            gain,
            vm_gain,
            finished,
            gain_staging: Some(GainStagingOutcome::Estimated(estimate)),
            ..
        } => {
            assert!(!finished && gain.is_none() && vm_gain.is_none());
            assert_eq!(playing.advanced.gain_staging_resolved, Some(estimate.gains));
            estimate
        }
        _ => panic!("the estimate comes first"),
    };
    let expected = wait_for_resolution(&key);
    assert_eq!(
        expected.audition,
        audition_gains(&expected.exact, Some(&estimate)),
        "audition keeps the estimate's drive only within the threshold"
    );
    let mut resolved = settings.clone();
    resolved.advanced.gain_staging_resolved = Some(expected.audition);
    let mut vm = None;
    for stage in 0..2 {
        match rx
            .recv_timeout(std::time::Duration::from_secs(120))
            .unwrap()
        {
            AudioCommand::PreviewLandingReady {
                settings: measured,
                vm_gain,
                gain,
                finished,
                gain_staging,
                generation,
                track_epoch,
                ..
            } => {
                assert_eq!((track_epoch, generation), (3, 5));
                assert_eq!(gain_staging, Some(GainStagingOutcome::Resolved(expected)));
                assert_eq!(
                    measured.advanced.gain_staging_resolved,
                    Some(expected.audition)
                );
                assert_eq!(finished, stage == 1);
                if stage == 0 {
                    vm = vm_gain;
                } else {
                    assert!(gain.is_some_and(|g| g.is_finite() && g > 0.0));
                }
            }
            _ => panic!("unexpected command"),
        }
    }
    let vm = vm.expect("Volume Match first");
    let expected_vm = crate::engine::preview_volume_match_gain(
        &pcm.samples,
        pcm.sample_rate,
        pcm.channels,
        &resolved,
    )
    .unwrap();
    assert_eq!(vm, expected_vm, "Volume Match measures the resolved chain");
    let mut chain = crate::dsp::MasteringChain::new(pcm.sample_rate, 2, &resolved);
    chain.coeffs.volume_match_gain_lin = vm;
    let mut rendered = pcm.samples.to_vec();
    chain.process_interleaved(&mut rendered, 2);
    let matched =
        crate::engine::measure_integrated_lufs(&rendered, pcm.sample_rate, pcm.channels).unwrap();
    assert!(
        (matched - source_lufs).abs() <= 1.0,
        "matched {matched:.2} LUFS against the source's {source_lufs:.2}"
    );
}

/// A superseded resolution is cancelled, never stored, and never changes
/// what the line shows.
#[test]
fn a_superseded_resolution_is_discarded() {
    let pcm = song();
    let (path, mtime) = source("superseded");
    let first = switched_on(-9.0, LoudnessPriority::Clean);
    let second = switched_on(-11.0, LoudnessPriority::Clean);
    let first_key = Key::new(&path, mtime, &first).unwrap();
    let second_key = Key::new(&path, mtime, &second).unwrap();
    assert_ne!(first_key, second_key);

    let active = |gate: &PreviewWorkerGate| PreviewWorkRequest {
        settings: first.clone(),
        generation: 1,
        track_epoch: gate.epoch,
        landing_enabled: true,
        device_rate: 48_000,
        gain_staging: Some(GainStagingWork {
            key: first_key.clone(),
            analysis: None,
        }),
    };
    let mut gate = PreviewWorkerGate::default();
    gate.active = Some(active(&gate));
    gate.cancel_superseded_gain_staging(Some(&first_key));
    assert!(
        !gate.cancelled.load(Ordering::Relaxed),
        "the same settings keep it"
    );
    gate.cancel_superseded_gain_staging(Some(&second_key));
    assert!(
        gate.cancelled.load(Ordering::Relaxed),
        "new settings cancel it"
    );
    let mut gate = PreviewWorkerGate::default();
    gate.active = Some(active(&gate));
    gate.cancel_superseded_gain_staging(None);
    assert!(
        gate.cancelled.load(Ordering::Relaxed),
        "switching off cancels it"
    );

    // A cancelled resolution stores nothing.
    let cancel = AtomicBool::new(true);
    assert!(resolve_cached(&first_key, &pcm, &first, None, &cancel).is_err());
    assert_eq!(lookup(&first_key), None);

    // A result for an older generation never changes the line; the live
    // generation's does.
    let measuring = Some(AutoGainStagingReadout::Measuring);
    let resolution = Resolution {
        gains: ResolvedGainStaging {
            level_in_db: 2.0,
            limiter_drive_db: 3.0,
            limiter_load_db: 1.5,
            ..Default::default()
        },
        delivered_lufs: -9.1,
        target_lufs: -9.0,
    };
    let entry = LiveEntry {
        exact: resolution,
        audition: resolution.gains,
    };
    let outcome = Some(GainStagingOutcome::Resolved(entry));
    assert_eq!(worker_readout(measuring, 4, 5, outcome), measuring);
    assert_eq!(
        worker_readout(measuring, 5, 5, outcome),
        Some(resolution.readout())
    );
    // Slice 8: the estimate's line, only for the live generation.
    let estimated = Some(GainStagingOutcome::Estimated(resolution));
    assert_eq!(worker_readout(measuring, 4, 5, estimated), measuring);
    assert_eq!(
        worker_readout(measuring, 5, 5, estimated),
        Some(resolution.estimated_readout())
    );
    assert_eq!(
        worker_readout(measuring, 5, 5, Some(GainStagingOutcome::Inert)),
        Some(AutoGainStagingReadout::Unusable)
    );
    assert_eq!(worker_readout(measuring, 5, 5, None), measuring);
}

/// Slice 8, F3: the exact result replaces the estimate audition plays only
/// when their drives differ by the threshold or more; below it the chain's
/// coefficients stay exactly as they are, so nothing changes at all.
#[test]
fn refining_changes_nothing_below_the_threshold() {
    let exact = Resolution {
        gains: ResolvedGainStaging {
            level_in_db: 1.5,
            limiter_drive_db: 2.0,
            limiter_load_db: 3.0,
            stopped_at_clean_limit: false,
            stopped_by_peak_check: false,
        },
        delivered_lufs: -9.05,
        target_lufs: -9.0,
    };
    let near = |drive: f32| Resolution {
        gains: ResolvedGainStaging {
            limiter_drive_db: drive,
            limiter_load_db: 2.8,
            ..exact.gains
        },
        delivered_lufs: -9.1,
        target_lufs: -9.0,
    };
    let settings = switched_on(-9.0, LoudnessPriority::Clean);
    let coeffs = |gains: ResolvedGainStaging| {
        let mut s = settings.clone();
        s.advanced.gain_staging_resolved = Some(gains);
        crate::dsp::ChainCoeffs::from_settings(48_000, &s)
    };
    for drive in [1.71, 1.9, 2.0, 2.1, 2.29] {
        let estimate = near(drive);
        let audition = audition_gains(&exact, Some(&estimate));
        assert_eq!(audition.limiter_drive_db, drive, "kept: {drive}");
        assert_eq!(audition.level_in_db, exact.gains.level_in_db);
        assert_eq!(
            coeffs(audition),
            coeffs(estimate.gains),
            "the chain does not change: {drive}"
        );
    }
    for drive in [1.69, 1.0, 2.31, 4.0] {
        let audition = audition_gains(&exact, Some(&near(drive)));
        assert_eq!(audition, exact.gains, "replaced: {drive}");
        assert_ne!(coeffs(audition), coeffs(near(drive).gains));
    }
    assert_eq!(audition_gains(&exact, None), exact.gains);
    let other_level = Resolution {
        gains: ResolvedGainStaging {
            level_in_db: 1.0,
            ..near(2.0).gains
        },
        ..near(2.0)
    };
    assert_eq!(audition_gains(&exact, Some(&other_level)), exact.gains);
}

/// Slice 8: with Clean, the estimate follows the export's search to within
/// F1's 0.5 dB, and exports never use it (their resolution is the exact one,
/// pinned above). Reach target is pinned separately below.
#[test]
fn the_estimate_lands_near_the_exact_drive() {
    let pcm = music_like();
    for (name, target, priority) in [
        ("estimate-clean-9", -9.0, LoudnessPriority::Clean),
        ("estimate-clean-11", -11.0, LoudnessPriority::Clean),
        ("estimate-clean-14", -14.0, LoudnessPriority::Clean),
    ] {
        let (path, mtime) = source(name);
        let settings = switched_on(target, priority);
        let key = Key::new(&path, mtime, &settings).unwrap();
        let cancel = AtomicBool::new(false);
        let est = estimate(&key, &pcm, &settings, None, &cancel)
            .unwrap()
            .expect("active");
        let exact = resolve_cached(&key, &pcm, &settings, Some(&est), &cancel)
            .unwrap()
            .expect("active");
        let gap = est.gains.limiter_drive_db - exact.exact.gains.limiter_drive_db;
        assert_eq!(
            est.gains.level_in_db, exact.exact.gains.level_in_db,
            "{name}"
        );
        assert!(gap.abs() <= 0.5, "{name}: estimate {gap:+.3} dB from exact");
        assert!(gap <= 1.5, "{name}");
        // A cancelled estimate is an error, never a result.
        assert!(estimate(&key, &pcm, &settings, None, &AtomicBool::new(true)).is_err());
    }
}

/// Slice 8, Reach target on this bright signal: the export's search stops
/// early at its no-progress peak check, because the final check's qualified
/// peak (its finite-sinc half) holds the loudness down at low drives. Only
/// the qualified reading on every pass follows it. The lowpass reading,
/// alone or checked once on the last pass, overshoots the exact drive here
/// by about 10 dB (Checkpoint D's F1/F2 trade-off).
#[test]
fn only_the_qualified_reading_on_every_pass_follows_the_peak_check_stop() {
    use crate::gain_staging::{estimate_preview_with, EstimatePeak};
    let pcm = music_like();
    let settings = switched_on(-9.0, LoudnessPriority::ReachTarget);
    let (path, mtime) = source("estimate-reach-9");
    let key = Key::new(&path, mtime, &settings).unwrap();
    let cancel = AtomicBool::new(false);
    let exact = resolve_cached(&key, &pcm, &settings, None, &cancel)
        .unwrap()
        .expect("active")
        .exact;
    assert!(exact.gains.stopped_by_peak_check, "{exact:?}");
    let drive = |peak| {
        let (est, _) = estimate_preview_with(
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels,
            &settings,
            None,
            peak,
            0.0,
            None,
        )
        .unwrap()
        .expect("active");
        est.gains.limiter_drive_db - exact.gains.limiter_drive_db
    };
    let qualified = drive(EstimatePeak::Qualified);
    assert!(qualified.abs() <= 0.5, "qualified {qualified:+.3} dB");
    for peak in [EstimatePeak::Lowpass, EstimatePeak::Verified] {
        let gap = drive(peak);
        assert!(gap > 1.5, "{peak:?} {gap:+.3} dB: the recorded overshoot");
    }
}
