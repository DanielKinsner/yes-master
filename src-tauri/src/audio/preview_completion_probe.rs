//! Opt-in real-device evidence through a complete cold preview measurement.
//! Captures production PCM before muting; does not change device preferences.
use super::*;
use rodio::{cpal, cpal::traits::StreamTrait, Source};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::Instant;

#[test]
#[ignore = "muted native completion probe: YES_MASTER_COMPLETION_INPUT and fresh YES_MASTER_COMPLETION_OUTPUT directory"]
fn cold_preset_completion() {
    let input = PathBuf::from(std::env::var("YES_MASTER_COMPLETION_INPUT").unwrap());
    let out = PathBuf::from(std::env::var("YES_MASTER_COMPLETION_OUTPUT").unwrap());
    assert!(!out.exists());
    std::fs::create_dir_all(&out).unwrap();
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&input).unwrap()).unwrap();
    let path = Path::new(config["source"]["path"].as_str().unwrap());
    let source_hash = format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()));
    assert_eq!(source_hash, config["source_sha256"].as_str().unwrap());
    let mut requested: MasteringSettings =
        serde_json::from_value(config["saved_settings"].clone()).unwrap();
    let analysis = crate::analysis::analyze_one(TrackId("completion".into()), path, false).unwrap();
    requested.advanced.source_profile = SourceProfile::from_analysis(&analysis);
    requested.volume_match = false;
    let pcm: PlaybackPcm = decode_full(path).unwrap().into();
    let settings = output_route::preview_settings(
        &requested,
        pcm.sample_rate,
        crate::export_format::ExportEncoding::Wav,
    );
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let output_config = device.default_output_config().unwrap();
    assert_eq!(output_config.sample_format(), cpal::SampleFormat::F32);
    assert_eq!(output_config.channels(), pcm.channels);
    let rate = output_config.sample_rate().0;
    let channels = usize::from(pcm.channels);
    let mut initial = settings.clone();
    initial.preset = Preset::Tape;
    let peak = Arc::new(AtomicU32::new(0));
    let (tx, rx) = mpsc::channel();
    let source = MasteringSource::new(
        pcm.samples.clone(),
        pcm.channels,
        pcm.sample_rate,
        crate::dsp::MasteringChain::new(pcm.sample_rate, channels, &initial),
        rx,
        peak.clone(),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicI32::new(i32::MIN)),
        Arc::new(AtomicI32::new(i32::MIN)),
        Arc::new(SpectrumRing::new()),
    )
    .with_initial_revision(1);
    let mailbox = gain_stage::GainMailbox::new(gain_stage::GainPlan {
        revision: 1,
        raw_revision: 1,
        landing: 1.,
        volume_match: 1.,
    });
    let (mut source, failure) = output_route::mastered_source(
        source,
        settings.effective_sample_rate(pcm.sample_rate),
        rate,
        crate::sources::FadeEnvelope::inactive(),
        mailbox.clone(),
    )
    .unwrap();
    let applied = source.revision_slot();
    source.try_seek(Duration::from_secs(30)).unwrap();
    // Storage is fixed before opening the stream. The callback never locks,
    // logs, grows a buffer or writes a file.
    let capture: Arc<Vec<AtomicU32>> = Arc::new(
        (0..rate as usize * channels * 120)
            .map(|_| AtomicU32::new(0))
            .collect(),
    );
    let slots: Arc<Vec<[AtomicU64; 4]>> = Arc::new(
        (0..32768)
            .map(|_| std::array::from_fn(|_| AtomicU64::new(0)))
            .collect(),
    );
    let samples = Arc::new(AtomicU64::new(0));
    let calls = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));
    let exhausted = Arc::new(AtomicU64::new(0));
    let epoch = Instant::now();
    let (cb_capture, cb_slots, cb_samples, cb_calls, cb_errors, cb_exhausted) = (
        capture.clone(),
        slots.clone(),
        samples.clone(),
        calls.clone(),
        errors.clone(),
        exhausted.clone(),
    );
    let stream = device
        .build_output_stream(
            &output_config.config(),
            move |data: &mut [f32], _| {
                let start = Instant::now();
                let offset = cb_samples.load(Ordering::Relaxed) as usize;
                for (index, value) in data.iter_mut().enumerate() {
                    let processed = source.next().unwrap_or_else(|| {
                        cb_exhausted.fetch_add(1, Ordering::Relaxed);
                        0.
                    });
                    if let Some(slot) = cb_capture.get(offset + index) {
                        slot.store(processed.to_bits(), Ordering::Relaxed);
                    }
                    *value = 0.;
                }
                let elapsed = start.elapsed().as_nanos() as u64;
                cb_samples.store((offset + data.len()) as u64, Ordering::Relaxed);
                let index = cb_calls.fetch_add(1, Ordering::Relaxed) as usize;
                if let Some(slot) = cb_slots.get(index) {
                    for (target, value) in slot.iter().zip([
                        elapsed,
                        (data.len() / channels) as u64,
                        start.duration_since(epoch).as_nanos() as u64,
                        offset as u64,
                    ]) {
                        target.store(value, Ordering::Relaxed);
                    }
                }
            },
            move |_| {
                cb_errors.fetch_add(1, Ordering::Relaxed);
            },
            None,
        )
        .unwrap();
    stream.play().unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let edit_s = epoch.elapsed().as_secs_f64();
    let mut next = crate::dsp::ChainCoeffs::from_settings(pcm.sample_rate, &settings);
    next.volume_match_gain_lin = 1.;
    mailbox.publish(gain_stage::GainPlan {
        revision: 2,
        raw_revision: 2,
        landing: 1.,
        volume_match: 1.,
    });
    tx.send(LiveCoeffUpdate {
        generation: 2,
        coeffs: next,
    })
    .unwrap();
    let preparation = Instant::now();
    let measured =
        preparation_cache::measure_for_device(&pcm, &settings, rate, &AtomicBool::new(false))
            .unwrap();
    let preparation_s = preparation.elapsed().as_secs_f64();
    let ready_s = epoch.elapsed().as_secs_f64();
    let ready_sample = samples.load(Ordering::Acquire);
    next.export_landing_gain_lin = measured.file_gain;
    mailbox.publish(gain_stage::GainPlan {
        revision: 3,
        raw_revision: 3,
        landing: measured.device_gain,
        volume_match: 1.,
    });
    tx.send(LiveCoeffUpdate {
        generation: 3,
        coeffs: next,
    })
    .unwrap();
    let timeout = Instant::now();
    while applied.load(Ordering::Acquire) != 3 {
        assert!(
            timeout.elapsed() < Duration::from_secs(3),
            "final gain did not reach output"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    let applied_s = epoch.elapsed().as_secs_f64();
    std::thread::sleep(Duration::from_secs(2));
    drop(stream);
    let count = calls.load(Ordering::Relaxed) as usize;
    let sample_count = samples.load(Ordering::Relaxed) as usize;
    assert!(count > 0 && count <= slots.len() && sample_count <= capture.len());
    let rows: Vec<[u64; 4]> = slots
        .iter()
        .take(count)
        .map(|slot| std::array::from_fn(|i| slot[i].load(Ordering::Relaxed)))
        .collect();
    let deadline_misses = rows
        .iter()
        .filter(|r| r[0] as f64 > r[1] as f64 / rate as f64 * 1e9)
        .count();
    let boundary_misses = rows
        .iter()
        .filter(|r| {
            let time = r[2] as f64 / 1e9;
            time >= ready_s - 0.25
                && time <= applied_s + 0.25
                && r[0] as f64 > r[1] as f64 / rate as f64 * 1e9
        })
        .count();
    let mut writer = hound::WavWriter::create(
        out.join("capture.wav"),
        hound::WavSpec {
            channels: pcm.channels,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for slot in capture.iter().take(sample_count) {
        writer
            .write_sample(f32::from_bits(slot.load(Ordering::Relaxed)))
            .unwrap();
    }
    writer.finalize().unwrap();
    let report = json!({"status":"complete", "scope":"production DSP/SRC/gain/meter sources on muted default-device callback through cold measurement and final publication; separate from AudioPlayer command lifecycle and installed listening",
        "source_sha256":source_hash,"requested_settings":requested,"resolved_settings":settings,
        "source_rate":pcm.sample_rate,"device_rate":rate,"device":device.name().unwrap(),
        "edit_s":edit_s,"preparation_s":preparation_s,"ready_s":ready_s,"applied_s":applied_s,
        "application_after_ready_ms":(applied_s-ready_s)*1000.,"ready_sample":ready_sample,
        "file_gain":measured.file_gain,"device_gain":measured.device_gain,
        "callbacks":count,"deadline_misses":deadline_misses,"completion_boundary_misses":boundary_misses,
        "device_errors":errors.load(Ordering::Relaxed),"exhausted_samples":exhausted.load(Ordering::Relaxed),
        "stream_failure":failure.load(Ordering::Acquire),"callback_ns_frames_start_ns_sample_offset":rows});
    std::fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("completion preparation={preparation_s:.3}s handoff={:.3}ms callbacks={count} misses={deadline_misses} boundary_misses={boundary_misses}", (applied_s-ready_s)*1000.);
    assert_eq!(errors.load(Ordering::Relaxed), 0);
    assert_eq!(exhausted.load(Ordering::Relaxed), 0);
    assert_eq!(failure.load(Ordering::Acquire), 0);
}
