//! Opt-in, muted native callback evidence. Measures production MasteringSource
//! and meter work on a real CPAL output callback during whole-song preparation.
//! This does not certify installed UI, listening, or the AudioPlayer lifecycle.
use super::*;
use rodio::cpal::{self, traits::StreamTrait};
use serde_json::json;
use std::time::Instant;

#[test]
#[ignore = "requires local output device and restored private source; writes timing JSON only"]
fn mastering_quality_native_callback_bench() {
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
    let playback = Arc::new(
        crate::sample_rate::convert_interleaved(&pcm.samples, pcm.sample_rate, rate, pcm.channels)
            .unwrap(),
    );
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
    let mut source = MasteringSource::new(
        playback,
        pcm.channels,
        rate,
        crate::dsp::MasteringChain::new(rate, pcm.channels as usize, &settings),
        rx,
        peak.clone(),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        lufs.clone(),
        integrated,
        ring.clone(),
    );
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
            coeffs: crate::dsp::ChainCoeffs::from_settings(rate, &settings),
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
    let report = json!({"scope":"muted production MasteringSource on CPAL callback, concurrent preview landing and coefficient edits; not installed UI/lifecycle proof",
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
    // Retain timing misses, rather than converting a noisy machine into a fake
    // invariant test failure or silently claiming the requested block was granted.
    println!("native callback report: {}", out.display());
}
