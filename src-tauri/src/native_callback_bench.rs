//! Opt-in, muted native callback evidence. Measures production MasteringSource
//! and meter work on a real CPAL output callback during whole-song preparation.
//! Separate probes cover callback timing and AudioPlayer lifecycle. Neither
//! certifies installed UI or listening.
use super::*;
use rodio::cpal::{self, traits::StreamTrait};
use serde_json::json;
use std::time::Instant;

#[test]
#[ignore = "muted production Mastered cascade, playing/paused rate edits and A/B over restored source"]
fn mastering_quality_mastered_rate_lifecycle() {
    use sha2::Digest;
    assert!(std::env::var_os("YES_MASTER_BENCH_MUTE").is_some());
    let source = PathBuf::from(std::env::var("YES_MASTER_BENCH_FILE").unwrap());
    let output = PathBuf::from(std::env::var("YES_MASTER_LIFECYCLE_REPORT").unwrap());
    assert!(!output.exists());
    let player = AudioPlayer::new();
    let track = TrackId("native-mastered-conversion".into());
    let wait = |check: &dyn Fn(&PlaybackSnapshot) -> bool, seconds| {
        let start = Instant::now();
        loop {
            let snapshot = player.snapshot().unwrap();
            assert!(
                snapshot.playback_error.is_none() && !snapshot.device_lost,
                "{snapshot:?}"
            );
            if check(&snapshot) {
                return snapshot;
            }
            assert!(
                start.elapsed() < Duration::from_secs(seconds),
                "{snapshot:?}"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    let mut rows = Vec::new();
    let mut settings = tests::settings_with_intensity(0.75);
    settings.delivery_profile = DeliveryProfile::Custom;
    settings.advanced.target_sample_rate = Some(96000);
    settings.advanced.lufs_offset_db = Some(-14.);
    settings.advanced.bit_depth = Some(32);
    let start = Instant::now();
    player
        .play_master(track.clone(), &source, settings.clone(), 30., false, false)
        .unwrap();
    let accepted = start.elapsed().as_secs_f64();
    wait(
        &|s| s.is_playing && s.position_sec > 30.03 && s.peak_dbfs > SILENCE_DBFS,
        3,
    );
    rows.push(json!({"event":"cold_mastered_without_landing","accepted_s":accepted,"first_meter_s":start.elapsed().as_secs_f64()}));
    for paused in [false, true] {
        if paused {
            player.pause();
            wait(&|s| !s.is_playing, 3);
        }
        for file_rate in [44100, 48000, 96000] {
            let before = player.snapshot().unwrap();
            settings.advanced.target_sample_rate = Some(file_rate);
            let start = Instant::now();
            player.update_chain(settings.clone(), false, false).unwrap();
            let after = wait(
                &|s| s.play_generation > before.play_generation && s.is_playing != paused,
                3,
            );
            assert!(after.position_sec >= before.position_sec - 0.01);
            if paused {
                assert!((after.position_sec - before.position_sec).abs() < 0.01);
                std::thread::sleep(Duration::from_millis(100));
                assert!(!player.snapshot().unwrap().is_playing);
            }
            rows.push(json!({"event":"rate_edit","rate":file_rate,"paused":paused,"observed_s":start.elapsed().as_secs_f64(),
                "before":before.position_sec,"after":after.position_sec}));
        }
    }
    player.seek(90.).unwrap();
    wait(&|s| !s.is_playing && (s.position_sec - 90.).abs() < 0.01, 3);
    let start = Instant::now();
    player.resume();
    wait(
        &|s| s.is_playing && s.position_sec > 90.03 && s.peak_dbfs > SILENCE_DBFS,
        3,
    );
    rows.push(json!({"event":"paused_seek_resume","observed_s":start.elapsed().as_secs_f64()}));
    for mastered in [false, true, false, true] {
        let before = player.snapshot().unwrap();
        let start = Instant::now();
        if mastered {
            player
                .play_master(
                    track.clone(),
                    &source,
                    settings.clone(),
                    before.position_sec,
                    false,
                    false,
                )
                .unwrap();
        } else {
            player
                .play_track(track.clone(), &source, before.position_sec)
                .unwrap();
        }
        let after = wait(
            &|s| s.play_generation > before.play_generation && s.is_playing,
            3,
        );
        assert!(after.position_sec >= before.position_sec - 0.01);
        rows.push(
            json!({"event":"ab","mastered":mastered,"observed_s":start.elapsed().as_secs_f64(),
            "before":before.position_sec,"after":after.position_sec}),
        );
    }
    let start = Instant::now();
    player.update_chain(settings.clone(), true, false).unwrap();
    wait(&|s| s.landing_pending, 3);
    let first_pending = start.elapsed().as_secs_f64();
    wait(&|s| !s.landing_pending && s.is_playing, 180);
    rows.push(json!({"event":"first_landing_preparation","first_pending_s":first_pending,"settled_snapshot_s":start.elapsed().as_secs_f64()}));
    for target in [-23., -14.] {
        settings.advanced.lufs_offset_db = Some(target);
        let start = Instant::now();
        let before = player.snapshot().unwrap().position_sec;
        player.update_chain(settings.clone(), true, false).unwrap();
        std::thread::sleep(Duration::from_millis(75));
        wait(
            &|s| !s.landing_pending && s.is_playing && s.position_sec > before,
            180,
        );
        rows.push(json!({"event":"target_edit","target":target,"settled_snapshot_s":start.elapsed().as_secs_f64()}));
    }
    player.stop();
    wait(&|s| !s.is_loaded, 3);
    std::fs::write(output,serde_json::to_vec_pretty(&json!({"status":"complete",
        "scope":"muted actual Mastered production rate route; snapshot timings under research load, no codec/device-cap/installed/listening claim",
        "source_sha256":format!("{:x}",sha2::Sha256::digest(std::fs::read(source).unwrap())),"rows":rows})).unwrap()).unwrap();
}

#[test]
#[ignore = "muted production Original playback, seek and device changes over a restored source"]
fn mastering_quality_original_native_lifecycle() {
    use sha2::Digest;
    assert!(std::env::var_os("YES_MASTER_BENCH_MUTE").is_some());
    let source = PathBuf::from(std::env::var("YES_MASTER_BENCH_FILE").unwrap());
    let output = PathBuf::from(std::env::var("YES_MASTER_LIFECYCLE_REPORT").unwrap());
    assert!(!output.exists());
    let player = AudioPlayer::new();
    let track = TrackId("native-original-conversion".into());
    let wait = |check: &dyn Fn(&PlaybackSnapshot) -> bool| {
        let start = Instant::now();
        loop {
            let snapshot = player.snapshot().unwrap();
            assert!(
                snapshot.playback_error.is_none() && !snapshot.device_lost,
                "{snapshot:?}"
            );
            if check(&snapshot) {
                return snapshot;
            }
            assert!(start.elapsed() < Duration::from_secs(3), "{snapshot:?}");
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    let mut rows = Vec::new();
    let start = Instant::now();
    player.play_track(track.clone(), &source, 30.).unwrap();
    let accepted = start.elapsed().as_secs_f64();
    wait(&|s| s.is_playing && s.position_sec > 30.03 && s.peak_dbfs > SILENCE_DBFS);
    rows.push(json!({"event":"cold_original_and_meter","accepted_s":accepted,"observed_s":start.elapsed().as_secs_f64()}));
    let start = Instant::now();
    player.seek(65.).unwrap();
    wait(&|s| s.is_playing && s.position_sec > 65.03 && s.peak_dbfs > SILENCE_DBFS);
    rows.push(json!({"event":"playing_seek","observed_s":start.elapsed().as_secs_f64()}));
    player.pause();
    wait(&|s| !s.is_playing);
    player.seek(90.).unwrap();
    wait(&|s| !s.is_playing && (s.position_sec - 90.).abs() < 0.05);
    let start = Instant::now();
    player.resume();
    wait(&|s| s.is_playing && s.position_sec > 90.03);
    rows.push(json!({"event":"paused_seek_resume","observed_s":start.elapsed().as_secs_f64()}));
    let before = player.snapshot().unwrap();
    let start = Instant::now();
    player
        .play_track(track.clone(), &source, before.position_sec)
        .unwrap();
    let after = wait(&|s| s.play_generation > before.play_generation && s.is_playing);
    assert!(after.position_sec >= before.position_sec - 0.05);
    rows.push(
        json!({"event":"same_source_swap","observed_s":start.elapsed().as_secs_f64(),
        "before":before.position_sec,"after":after.position_sec}),
    );
    let device = player
        .list_output_devices()
        .unwrap()
        .into_iter()
        .find(|d| d.is_default)
        .unwrap();
    for selection in [Some(device.id), None] {
        let start = Instant::now();
        player.set_output_device(selection.clone()).unwrap();
        wait(&|s| !s.is_loaded && !s.is_playing);
        rows.push(json!({"event":"device_reopen","selection":selection,"observed_s":start.elapsed().as_secs_f64()}));
        player.play_track(track.clone(), &source, 30.).unwrap();
        wait(&|s| {
            s.is_loaded && s.is_playing && s.position_sec > 30.03 && s.peak_dbfs > SILENCE_DBFS
        });
    }
    player.stop();
    wait(&|s| !s.is_loaded);
    std::fs::write(output, serde_json::to_vec_pretty(&json!({"status":"complete",
        "scope":"muted actual AudioPlayer Original route; snapshot-observed timings under research load; no Mastered/installed/listening proof",
        "source_sha256":format!("{:x}", sha2::Sha256::digest(std::fs::read(source).unwrap())),"rows":rows})).unwrap()).unwrap();
}

#[test]
#[ignore = "muted native AudioPlayer lifecycle diagnostic over two restored sources"]
fn mastering_quality_native_lifecycle_bench() {
    assert!(
        std::env::var_os("YES_MASTER_BENCH_MUTE").is_some(),
        "set the test-only sink mute"
    );
    let source = PathBuf::from(std::env::var("YES_MASTER_BENCH_FILE").unwrap());
    let other = PathBuf::from(std::env::var("YES_MASTER_BENCH_OTHER_FILE").unwrap());
    let output = PathBuf::from(std::env::var("YES_MASTER_LIFECYCLE_REPORT").unwrap());
    assert!(!output.exists());
    let saved: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("YES_MASTER_BENCH_PREPARATION").unwrap()).unwrap(),
    )
    .unwrap();
    let settings: MasteringSettings = serde_json::from_value(saved["settings"].clone()).unwrap();
    let mut rows = Vec::new();
    let mut record = |row| {
        rows.push(row);
        std::fs::write(
            &output,
            serde_json::to_vec_pretty(&json!({
                "status":"in_progress","source_sha256":saved["source_sha256"],"rows":rows,
            }))
            .unwrap(),
        )
        .unwrap();
    };
    let player = AudioPlayer::new();
    let track = TrackId("native-lifecycle-a".into());
    let wait = |check: &dyn Fn(&PlaybackSnapshot) -> bool| {
        let start = Instant::now();
        loop {
            let s = player.snapshot().unwrap();
            if check(&s) {
                return s;
            }
            assert!(
                start.elapsed() < Duration::from_secs(30),
                "native snapshot did not settle: {s:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    let start = Instant::now();
    player.play_track(track.clone(), &source, 30.0).unwrap();
    wait(&|s| s.is_playing && s.position_sec > 30.0 && s.peak_dbfs > SILENCE_DBFS);
    record(
        json!({"event":"cold_original_and_first_meter","seconds":start.elapsed().as_secs_f64()}),
    );
    let start = Instant::now();
    let before_snapshot = player.snapshot().unwrap();
    let before = before_snapshot.position_sec;
    player
        .play_master(
            track.clone(),
            &source,
            settings.clone(),
            before,
            true,
            false,
        )
        .unwrap();
    let accepted = start.elapsed().as_secs_f64();
    let after = wait(&|s| {
        s.play_generation > before_snapshot.play_generation && s.is_playing && !s.landing_pending
    });
    record(
        json!({"event":"first_master","accept_s":accepted,"settle_s":start.elapsed().as_secs_f64(),
        "before_position":before,"after_position":after.position_sec,"device_lost":after.device_lost}),
    );
    let mut edits = Vec::new();
    let mut s = settings.clone();
    s.advanced.lufs_offset_db = Some(-14.0);
    edits.push(("target", s));
    let mut s = settings.clone();
    s.intensity = 0.5;
    edits.push(("intensity", s));
    let mut s = settings.clone();
    s.eq_high_db = 1.0;
    edits.push(("eq", s));
    let mut s = settings.clone();
    s.advanced.compression_density = Some(0.2);
    edits.push(("density", s));
    let mut s = settings.clone();
    s.advanced.adaptive_strength = Some(1.0);
    edits.push(("adapt", s));
    let mut s = settings.clone();
    s.preset = Preset::Loud;
    edits.push(("preset", s));
    edits.push(("return_initial", settings.clone()));
    for (event, s) in edits {
        let start = Instant::now();
        let before = player.snapshot().unwrap().position_sec;
        player.update_chain(s, true, false).unwrap();
        // Snapshot cadence bounds fast hits; do not call this an exact worker timestamp.
        std::thread::sleep(Duration::from_millis(60));
        let pending_observed = player.snapshot().unwrap().landing_pending;
        // A fast cache hit can precede the next 50 ms transport snapshot.
        // Require observed playback progress as well as level readiness.
        let after = wait(&|s| !s.landing_pending && s.position_sec > before);
        assert!(after.is_playing && !after.device_lost && after.position_sec > before);
        record(
            json!({"event":event,"observed_settle_s":start.elapsed().as_secs_f64(),
            "pending_observed":pending_observed,"position_advanced_s":after.position_sec-before}),
        );
    }
    let start = Instant::now();
    for i in 0..30 {
        let mut s = settings.clone();
        s.eq_high_db = i as f32 * 0.1;
        player.update_chain(s, true, false).unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(60));
    wait(&|s| !s.landing_pending);
    record(json!({"event":"rapid_30_edits","settle_s":start.elapsed().as_secs_f64()}));
    for kind in ["original", "mastered"] {
        let before_snapshot = player.snapshot().unwrap();
        let before = before_snapshot.position_sec;
        let start = Instant::now();
        if kind == "original" {
            player.play_track(track.clone(), &source, before).unwrap();
        } else {
            player
                .play_master(
                    track.clone(),
                    &source,
                    settings.clone(),
                    before,
                    true,
                    false,
                )
                .unwrap();
        }
        let after = wait(&|s| {
            s.play_generation > before_snapshot.play_generation
                && s.is_playing
                && !s.landing_pending
        });
        assert!(after.position_sec >= before - 0.05 && !after.device_lost);
        record(
            json!({"event":kind,"seconds":start.elapsed().as_secs_f64(),"before":before,"after":after.position_sec}),
        );
    }
    let device = player
        .list_output_devices()
        .unwrap()
        .into_iter()
        .find(|d| d.is_default)
        .unwrap();
    for id in [Some(device.id), None] {
        let start = Instant::now();
        player.set_output_device(id.clone()).unwrap();
        // Existing device-selection contract closes current playback and opens
        // an unloaded stream for the next audition (APP_BEHAVIOR.md).
        let snapshot = wait(&|s| !s.is_loaded && !s.is_playing && !s.device_lost);
        record(
            json!({"event":"device_route_reopen","selection":id,"seconds":start.elapsed().as_secs_f64(),"playing":snapshot.is_playing}),
        );
    }
    let start = Instant::now();
    let other_id = TrackId("native-lifecycle-b".into());
    player.play_track(other_id.clone(), &other, 30.0).unwrap();
    wait(&|s| s.track_id.as_ref() == Some(&other_id) && s.is_playing && s.position_sec > 30.0);
    record(json!({"event":"source_switch","seconds":start.elapsed().as_secs_f64()}));
    player
        .play_master(track, &source, settings.clone(), 30.0, true, false)
        .unwrap();
    let mut s = settings;
    s.eq_high_db = 4.1;
    player.update_chain(s, true, false).unwrap();
    std::thread::sleep(Duration::from_millis(60));
    let pending = player.snapshot().unwrap().landing_pending;
    let start = Instant::now();
    player.stop();
    wait(&|s| !s.is_playing && !s.landing_pending);
    record(
        json!({"event":"stop_pending_preparation","pending_observed":pending,"seconds":start.elapsed().as_secs_f64()}),
    );
    std::fs::write(output,serde_json::to_vec_pretty(&json!({"scope":"muted real AudioPlayer API, native output; snapshot-observed settling, no installed UI or listening claim",
        "source_sha256":saved["source_sha256"],"rows":rows})).unwrap()).unwrap();
}

#[test]
#[ignore = "requires local output device and restored private source; writes timing JSON only"]
fn mastering_quality_native_callback_bench() {
    callback_bench(false);
}

#[test]
#[ignore = "muted native streaming SRC candidate; requires restored private source and output device"]
fn mastering_quality_streaming_callback_bench() {
    callback_bench(true);
}

fn callback_bench(streaming_src: bool) {
    let path = std::env::var("YES_MASTER_BENCH_FILE").expect("source path");
    let out = PathBuf::from(std::env::var("YES_MASTER_CALLBACK_REPORT").expect("fresh JSON path"));
    assert!(!out.exists());
    let prepare = Instant::now();
    let pcm = decode_full(Path::new(&path)).unwrap();
    let decode_s = prepare.elapsed().as_secs_f64();
    let host = cpal::default_host();
    let device = host.default_output_device().expect("native output device");
    let default = device.default_output_config().unwrap();
    assert_eq!(
        default.sample_format(),
        cpal::SampleFormat::F32,
        "this probe currently requires float32 native output"
    );
    assert_eq!(
        default.channels(),
        pcm.channels,
        "do not silently introduce channel conversion"
    );
    let mut config: cpal::StreamConfig = default.into();
    config.buffer_size = cpal::BufferSize::Fixed(256);
    let rate = config.sample_rate.0;
    let start = Instant::now();
    let playback = Arc::new(if streaming_src {
        pcm.samples.clone()
    } else {
        crate::sample_rate::convert_interleaved(&pcm.samples, pcm.sample_rate, rate, pcm.channels)
            .unwrap()
    });
    let chain_rate = if streaming_src { pcm.sample_rate } else { rate };
    let playback_src_s = start.elapsed().as_secs_f64();
    let mut settings = tests::settings_with_intensity(0.75);
    settings.advanced = AdvancedSettings::default();
    settings.delivery_profile = DeliveryProfile::Custom;
    settings.advanced.lufs_offset_db = Some(-9.0);
    settings.advanced.target_sample_rate = Some(48_000);
    settings.advanced.ceiling_dbtp = Some(-1.0);
    let initial_settings = settings.clone();
    let peak = Arc::new(AtomicU32::new(0));
    let lufs = Arc::new(AtomicI32::new(i32::MIN));
    let integrated = Arc::new(AtomicI32::new(i32::MIN));
    let ring = Arc::new(SpectrumRing::new());
    let (tx, rx) = mpsc::channel();
    let source = MasteringSource::new(
        playback,
        pcm.channels,
        chain_rate,
        crate::dsp::MasteringChain::new(chain_rate, pcm.channels as usize, &settings),
        rx,
        peak.clone(),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        lufs.clone(),
        integrated,
        ring.clone(),
    );
    let construct_start = Instant::now();
    let (mut source, stream_error): (Box<dyn Iterator<Item = f32> + Send>, _) = if streaming_src {
        let file_rate = std::env::var("YES_MASTER_CALLBACK_FILE_RATE")
            .ok()
            .map(|rate| rate.parse().unwrap())
            .unwrap_or(settings.effective_sample_rate(pcm.sample_rate));
        let (metered, slot) = output_route::mastered_source(
            source,
            file_rate,
            rate,
            crate::sources::FadeEnvelope::inactive(),
        )
        .unwrap();
        (Box::new(metered), Some(slot))
    } else {
        (Box::new(source), None)
    };
    let streaming_construction_s = construct_start.elapsed().as_secs_f64();
    // Fixed allocation before the callback; atomic slots avoid callback locks.
    let slots: Arc<Vec<[AtomicU64; 3]>> = Arc::new(
        (0..16_384)
            .map(|_| std::array::from_fn(|_| AtomicU64::new(0)))
            .collect(),
    );
    let count = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));
    let exhausted = Arc::new(AtomicU64::new(0));
    let epoch = Instant::now();
    let (callback_slots, callback_count, callback_errors, callback_exhausted) = (
        slots.clone(),
        count.clone(),
        errors.clone(),
        exhausted.clone(),
    );
    let channels = usize::from(pcm.channels);
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                let start = Instant::now();
                for sample in data.iter_mut() {
                    if let Some(processed) = source.next() {
                        std::hint::black_box(processed);
                    } else {
                        callback_exhausted.fetch_add(1, Ordering::Relaxed);
                    }
                    // Mute AFTER production processing and meters; no listening claim.
                    *sample = 0.0;
                }
                let ns = start.elapsed().as_nanos() as u64;
                let index = callback_count.fetch_add(1, Ordering::Relaxed) as usize;
                if let Some(slot) = callback_slots.get(index) {
                    slot[0].store(ns, Ordering::Relaxed);
                    slot[1].store((data.len() / channels) as u64, Ordering::Relaxed);
                    slot[2].store(
                        start.duration_since(epoch).as_nanos() as u64,
                        Ordering::Relaxed,
                    );
                }
            },
            move |_| {
                callback_errors.fetch_add(1, Ordering::Relaxed);
            },
            None,
        )
        .unwrap();
    stream.play().unwrap();
    let meter_start = Instant::now();
    while peak.load(Ordering::Relaxed) == 0 && meter_start.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(10));
    }
    let first_meter_s = meter_start.elapsed().as_secs_f64();
    let (done_tx, done_rx) = mpsc::channel();
    let worker_pcm = pcm.clone();
    let mut worker_settings = settings.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    let worker = std::thread::spawn(move || {
        let mut rows = Vec::new();
        for target in [-14.0, -9.0, -12.0] {
            worker_settings.advanced.lufs_offset_db = Some(target);
            let start = Instant::now();
            let result = crate::engine::preview_landing_with_cancel(
                &worker_pcm.samples,
                worker_pcm.sample_rate,
                worker_pcm.channels,
                &worker_settings,
                Some(&worker_cancel),
            );
            rows.push(json!({"target":target,"seconds":start.elapsed().as_secs_f64(),"ok":result.is_ok(),
                "error":result.err().map(|e|e.to_string()),"cancel_requested":worker_cancel.load(Ordering::Relaxed)}));
            if worker_cancel.load(Ordering::Relaxed) {
                break;
            }
        }
        done_tx.send(rows).unwrap();
    });
    let mut spectrum = SpectrumAnalyzer::new(rate);
    let mut snapshots = 0;
    let mut lufs_updates = 0;
    let mut previous_lufs = i32::MIN;
    let edit_start = Instant::now();
    for generation in 1..=120 {
        settings.eq_high_db = if generation % 2 == 0 { 1.0 } else { -1.0 };
        tx.send(LiveCoeffUpdate {
            generation,
            coeffs: crate::dsp::ChainCoeffs::from_settings(chain_rate, &settings),
        })
        .unwrap();
        std::hint::black_box(spectrum.compute(&ring));
        snapshots += 1;
        let current = lufs.load(Ordering::Relaxed);
        if current != previous_lufs {
            lufs_updates += 1;
            previous_lufs = current;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    cancel.store(true, Ordering::SeqCst);
    let cancel_start = Instant::now();
    let worker_rows = done_rx.recv_timeout(Duration::from_secs(30)).unwrap();
    let join_after_cancel_s = cancel_start.elapsed().as_secs_f64();
    worker.join().unwrap();
    drop(stream);
    let count = count.load(Ordering::Relaxed) as usize;
    let rows: Vec<_> = slots
        .iter()
        .take(count.min(slots.len()))
        .map(|slot| std::array::from_fn::<_, 3, _>(|i| slot[i].load(Ordering::Relaxed)))
        .collect();
    let deadline_misses = rows
        .iter()
        .filter(|r| r[0] as f64 > r[1] as f64 / rate as f64 * 1e9)
        .count();
    let report = json!({"scope":"muted MasteringSource on CPAL callback, concurrent preview landing and coefficient edits; production file/device streaming SRC when enabled; not installed UI/lifecycle proof",
        "streaming_src_candidate":streaming_src,"source_rate":pcm.sample_rate,"chain_rate":chain_rate,
        "file_rate_override":std::env::var("YES_MASTER_CALLBACK_FILE_RATE").ok(),
        "streaming_construction_s":streaming_construction_s,"streaming_error_code":stream_error.as_ref().map(|slot|slot.load(Ordering::Acquire)),
        "initial_settings":initial_settings,"coefficient_edits":120,
        "device":device.name().unwrap_or_default(),"sample_rate":rate,"channels":channels,
        "requested_frames":256,"granted_frames_min":rows.iter().map(|r|r[1]).min(),"granted_frames_max":rows.iter().map(|r|r[1]).max(),
        "decode_s":decode_s,"playback_src_s":playback_src_s,"first_peak_meter_s":first_meter_s,
        "duration_s":edit_start.elapsed().as_secs_f64(),"snapshots":snapshots,"lufs_updates":lufs_updates,
        "worker":worker_rows,"join_after_cancel_s":join_after_cancel_s,"callbacks":count,
        "deadline_misses":deadline_misses,"device_errors":errors.load(Ordering::Relaxed),
        "exhausted_samples":exhausted.load(Ordering::Relaxed),"callback_rows_ns_frames_start_ns":rows});
    std::fs::write(&out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    assert!(count > 0 && count <= slots.len());
    assert_eq!(errors.load(Ordering::Relaxed), 0);
    assert_eq!(exhausted.load(Ordering::Relaxed), 0);
    assert!(stream_error
        .as_ref()
        .is_none_or(|slot| slot.load(Ordering::Acquire) == 0));
    // Retain timing misses, rather than converting a noisy machine into a fake
    // invariant test failure or silently claiming the requested block was granted.
    println!("native callback report: {}", out.display());
}
