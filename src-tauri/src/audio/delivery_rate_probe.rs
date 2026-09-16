//! Opt-in offline capture of the actual MasteringSource and Rodio device SRC.
//! No output device or listening verdict; separate native probes cover those.
use super::*;
use crate::{dsp::MasteringChain, export_format::ExportEncoding, peak_meter};
use rodio::source::UniformSourceIterator;
use rodio::Source as _;
use serde_json::json;
use sha2::{Digest, Sha256};

fn write_witness(path: &Path, samples: &[f32], rate: u32) -> String {
    assert!(!path.exists());
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for &sample in samples {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn source(
    samples: Vec<f32>,
    rate: u32,
    settings: &MasteringSettings,
    gain: f32,
) -> MasteringSource {
    let (_, receive) = mpsc::channel();
    let mut chain = MasteringChain::new(rate, 2, settings);
    chain.coeffs.export_landing_gain_lin = gain;
    MasteringSource::new(
        samples,
        2,
        rate,
        chain,
        receive,
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicU32::new(0)),
        Arc::new(AtomicI32::new(i32::MIN)),
        Arc::new(AtomicI32::new(i32::MIN)),
        Arc::new(SpectrumRing::new()),
    )
    .with_initial_revision(1)
}

fn gains(landing: f32, volume_match: f32) -> Arc<gain_stage::GainMailbox> {
    gain_stage::GainMailbox::new(gain_stage::GainPlan {
        revision: 1,
        raw_revision: 1,
        landing,
        volume_match,
    })
}

fn measured(x: &[f32], rate: u32) -> serde_json::Value {
    let peak = peak_meter::measure(x, 2, || false).unwrap();
    let mut ebu = ebur128::EbuR128::new(2, rate, ebur128::Mode::I).unwrap();
    ebu.add_frames_f32(x).unwrap();
    let lufs = ebu.loudness_global().unwrap();
    let skip = (rate / 10) as usize * 2;
    let interior = &x[skip..x.len() - skip];
    let rms = interior.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / interior.len() as f64;
    json!({"lufs":lufs.is_finite().then_some(lufs),"peak_dbtp":peak.upper_dbtp(),
        "interior_rms_dbfs":10.*rms.max(1e-30).log10(),"frames":x.len()/2})
}

#[test]
fn canonical_mastered_route_matches_finite_offline_cascade() {
    for rate in [44100, 48000, 96000] {
        for file in [44100, 48000, 96000] {
            for device in [44100, 48000] {
                for frames in [1, 137, 2053, 48007] {
                    let samples: Vec<f32> = (0..frames)
                        .flat_map(|i| {
                            let x = (i as f32 * 0.73).sin() * 0.4;
                            [x, -x * 0.7]
                        })
                        .collect();
                    let settings = tests::settings_with_intensity(0.75);
                    let gain = 1.7;
                    let mut raw = samples.clone();
                    let mut chain = MasteringChain::new(rate, 2, &settings);
                    chain.coeffs.export_landing_gain_lin = gain;
                    chain.coeffs.volume_match_gain_lin = 1.;
                    chain.process_interleaved(&mut raw, 2);
                    chain.flush_render_tail(&mut raw, 2);
                    let file_pcm =
                        crate::sample_rate::convert_interleaved(&raw, rate, file, 2).unwrap();
                    let mut expected =
                        crate::sample_rate::convert_interleaved(&file_pcm, file, device, 2)
                            .unwrap();
                    for sample in &mut expected {
                        *sample *= 0.73;
                        *sample *= 0.37;
                    }
                    let (live, failure) = output_route::mastered_source(
                        source(samples, rate, &settings, gain),
                        file,
                        device,
                        crate::sources::FadeEnvelope::inactive(),
                        gains(0.73, 0.37),
                    )
                    .unwrap();
                    let actual: Vec<f32> = live.collect();
                    assert_eq!(
                        actual, expected,
                        "{rate}->{file}->{device}, {frames} frames"
                    );
                    assert_eq!(failure.load(Ordering::Acquire), 0);
                }
            }
        }
    }
}

#[test]
fn prepared_source_gain_matches_streaming_pcm_at_the_existing_small_gain_shortcut() {
    for (rate, file, device) in [(44100, 96000, 48000), (48000, 48000, 44100)] {
        let samples: Vec<f32> = (0..8193 * 2)
            .map(|i| 0.3 * (i as f32 * 0.47).sin())
            .collect();
        let mut settings = tests::settings_with_intensity(0.75);
        settings.advanced.target_sample_rate = Some(file);
        settings.delivery_profile = DeliveryProfile::Custom;
        let prepared =
            crate::engine::prepare_preview_audio(&samples, rate, 2, &settings, None).unwrap();
        for gain in [0., 0.5, 0.9999, 0.99999, 1., 1.00001, 1.0001, 2.] {
            let applied = preparation_cache::applied_source_landing(gain);
            let expected = prepared
                .device_pcm_with_landing(applied, device, None)
                .unwrap();
            let (stream, failure) = output_route::mastered_source(
                source(samples.clone(), rate, &settings, gain),
                file,
                device,
                crate::sources::FadeEnvelope::inactive(),
                gains(1., 1.),
            )
            .unwrap();
            assert_eq!(
                stream.collect::<Vec<_>>(),
                expected,
                "source gain {gain}: {rate}->{file}->{device}"
            );
            assert_eq!(failure.load(Ordering::Acquire), 0);
        }
    }
}

#[test]
#[ignore = "full restored-source PCM comparison and peak check through production post-device gain; fresh manifest/report required"]
fn mastering_quality_protected_device_whole_files() {
    let manifest = PathBuf::from(std::env::var("YES_MASTER_DEVICE_LIVE_INPUTS").unwrap());
    let output = PathBuf::from(std::env::var("YES_MASTER_DEVICE_LIVE_OUTPUT").unwrap());
    assert!(!output.exists());
    std::fs::create_dir_all(&output).unwrap();
    let entries: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    let mut report = json!({"status":"running", "rows":[], "estimator":peak_meter::VERSION,
        "manifest_sha256":format!("{:x}",Sha256::digest(std::fs::read(manifest).unwrap())),
        "scope":"whole original source, current compensated DSP -> file SRC -> device SRC -> verified correction -> meter, exact PCM equality to independently traversed prepared buffer; no opened device or listening claim"});
    for entry in entries {
        let path = Path::new(entry["source"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
            entry["sha256"].as_str().unwrap()
        );
        let pcm = decode_full(path).unwrap();
        assert_eq!(pcm.channels, 2);
        let settings: MasteringSettings =
            serde_json::from_value(entry["settings"].clone()).unwrap();
        let started = std::time::Instant::now();
        let prepared = crate::engine::prepare_preview_audio(
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels,
            &settings,
            None,
        )
        .unwrap();
        let file = prepared.finish(&settings, None).unwrap();
        let file_prepare_s = started.elapsed().as_secs_f64();
        for rate in [44100, 96000] {
            let started = std::time::Instant::now();
            let device = crate::device_preparation::PreparedDevicePcm::new(
                prepared.raw_pcm(),
                settings.effective_sample_rate(pcm.sample_rate),
                pcm.channels,
                rate,
                None,
            )
            .unwrap();
            let applied_gain = preparation_cache::applied_source_landing(file.gain_lin);
            let desired_correction = if applied_gain == 0. {
                1.
            } else {
                file.gain_lin / applied_gain
            };
            let actual = prepared
                .device_pcm_with_landing(applied_gain, rate, None)
                .unwrap();
            let delivered = device
                .finish_converted_gain(
                    actual,
                    applied_gain,
                    desired_correction,
                    settings.effective_ceiling_dbtp(),
                    None,
                )
                .unwrap();
            let device_prepare_s = started.elapsed().as_secs_f64();
            let started = std::time::Instant::now();
            let (live, failure) = output_route::mastered_source(
                source(
                    pcm.samples.clone(),
                    pcm.sample_rate,
                    &settings,
                    file.gain_lin,
                ),
                settings.effective_sample_rate(pcm.sample_rate),
                rate,
                crate::sources::FadeEnvelope::inactive(),
                gains(delivered.protection.gain_lin, 1.),
            )
            .unwrap();
            let actual: Vec<f32> = live.collect();
            assert_eq!(failure.load(Ordering::Acquire), 0);
            assert_eq!(actual.len(), delivered.samples.len());
            let differences = actual
                .iter()
                .zip(&delivered.samples)
                .filter(|(a, b)| a.to_bits() != b.to_bits())
                .count();
            assert_eq!(
                differences, 0,
                "whole-file production PCM differs from measured plan"
            );
            let route_s = started.elapsed().as_secs_f64();
            let path = output.join(format!("{}-{rate}.wav", entry["id"].as_str().unwrap()));
            let sha256 = write_witness(&path, &actual, rate);
            report["rows"].as_array_mut().unwrap().push(json!({"id":entry["id"],"path":path,"sha256":sha256,
                "source_sha256":entry["sha256"],"device_rate":rate,"file_rate":settings.effective_sample_rate(pcm.sample_rate),
                "source_rate":pcm.sample_rate,"channels":2,"frames":actual.len()/2,
                "peak":delivered.protection.true_peak_dbtp,"ceiling":settings.effective_ceiling_dbtp(),
                "lufs":delivered.protection.lufs,"file_gain":file.gain_lin,"device_gain":delivered.protection.gain_lin,
                "pcm_differences":differences,"file_prepare_s":file_prepare_s,"device_prepare_s":device_prepare_s,"route_s":route_s}));
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "whole-file production cascade rate measurement; set YES_MASTER_RATE_REPORT"]
fn mastering_quality_canonical_live_rates() {
    let path = PathBuf::from(std::env::var("YES_MASTER_RATE_REPORT").unwrap());
    assert!(!path.exists());
    let mut rows = Vec::new();
    for rate in [44100, 48000, 96000] {
        for frequency in [997_f32, rate as f32 * 0.4] {
            let samples: Vec<f32> = (0..rate * 2)
                .flat_map(|i| {
                    let x =
                        (i as f32 * frequency * std::f32::consts::TAU / rate as f32).sin() * 0.4;
                    [x, x * 0.8]
                })
                .collect();
            for file in [44100, 48000, 96000] {
                for target in [None, Some(-14.)] {
                    let settings: MasteringSettings = serde_json::from_value(json!({
                        "preset":{"kind":"custom","id":"canonical-rate-probe"},"intensity":0.,"volume_match":false,
                        "eq_low_db":0.,"eq_mid_db":0.,"eq_high_db":0.,"delivery_profile":"custom",
                        "advanced":{"lufs_offset_db":target,"ceiling_dbtp":-1.,"bit_depth":32,
                            "compression_mode":"off","warmth":0.,"target_sample_rate":file}
                    })).unwrap();
                    let landing =
                        crate::engine::preview_landing(&samples, rate, 2, &settings).unwrap();
                    for device in [44100, 48000] {
                        for enabled in [false, true] {
                            let gain = if enabled { landing.gain_lin } else { 1. };
                            let (live, failure) = output_route::mastered_source(
                                source(samples.clone(), rate, &settings, 1.),
                                file,
                                device,
                                crate::sources::FadeEnvelope::inactive(),
                                gains(gain, 1.),
                            )
                            .unwrap();
                            let actual: Vec<f32> = live.collect();
                            assert_eq!(failure.load(Ordering::Acquire), 0);
                            let mut row = json!({"source_rate":rate,"frequency":frequency,"file_rate":file,"device_rate":device,
                                "target":target,"preview_enabled":enabled,"gain":gain,"live":measured(&actual,device)});
                            if rate == 96000
                                && file == 44100
                                && device == 48000
                                && frequency > 30000.
                                && enabled
                                && target.is_some()
                            {
                                let witness = path.with_extension("wav");
                                row["witness"] = json!({"path":witness,"sha256":write_witness(&witness,&actual,device)});
                            }
                            rows.push(row);
                        }
                    }
                }
            }
        }
    }
    std::fs::write(path,serde_json::to_vec_pretty(&json!({"status":"complete","peak_version":peak_meter::VERSION,
        "scope":"actual production finite Mastered cascade, post-device gain placement; file-gain-only diagnostic, no codec resolution or device-gain-cap claim","rows":rows})).unwrap()).unwrap();
}

#[test]
#[ignore = "offline device/delivery-rate qualification; set YES_MASTER_RATE_REPORT"]
fn mastering_quality_device_delivery_rates() {
    let path = PathBuf::from(std::env::var("YES_MASTER_RATE_REPORT").unwrap());
    assert!(!path.exists(), "preserve completed evidence");
    let mut rows = Vec::new();
    for source_rate in [44100, 48000, 96000] {
        for (signal, frequency) in [("mid", 997_f32), ("high", source_rate as f32 * 0.4)] {
            let samples: Vec<f32> = (0..source_rate * 2)
                .flat_map(|n| {
                    let x = 0.4
                        * (n as f32 * frequency * std::f32::consts::TAU / source_rate as f32).sin();
                    [x, x * 0.8]
                })
                .collect();
            for requested_rate in [44100, 48000, 96000] {
                let mut settings: MasteringSettings = serde_json::from_value(json!({
                    "preset":{"kind":"custom","id":"rate-probe"},"intensity":0.,"volume_match":false,
                    "eq_low_db":0.,"eq_mid_db":0.,"eq_high_db":0.,"delivery_profile":"custom",
                    "advanced":{"lufs_offset_db":-14.,"ceiling_dbtp":-1.,"bit_depth":32,
                        "compression_mode":"off","warmth":0.,"target_sample_rate":requested_rate}
                }))
                .unwrap();
                for target in [None, Some(-14.)] {
                    settings.advanced.lufs_offset_db = target;
                    let landing =
                        crate::engine::preview_landing(&samples, source_rate, 2, &settings)
                            .unwrap();
                    let mut raw = samples.clone();
                    let mut chain = MasteringChain::new(source_rate, 2, &settings);
                    chain.process_interleaved(&mut raw, 2);
                    chain.flush_render_tail(&mut raw, 2);
                    let mut delivered = crate::sample_rate::convert_interleaved(
                        &raw,
                        source_rate,
                        requested_rate,
                        2,
                    )
                    .unwrap();
                    let protected = crate::output_protection::finalize(
                        &mut delivered,
                        requested_rate,
                        2,
                        32,
                        target,
                        -1.,
                        None,
                    )
                    .unwrap();
                    let delivery = measured(&delivered, requested_rate);
                    assert!(protected.true_peak_dbtp <= -1. + 1e-5);
                    // These are the actual settled/raw preview plan semantics.
                    for preview in [false, true] {
                        let gain = if preview { landing.gain_lin } else { 1. };
                        for device_rate in [44100, 48000] {
                            let live = source(samples.clone(), source_rate, &settings, gain);
                            // Pull samples like the mixer. Vec::collect asks
                            // Rodio 0.20's channel adapter for size_hint after
                            // the first sample; its zero lower bound can then
                            // underflow. Native callback iteration does not.
                            let converted =
                                UniformSourceIterator::<_, f32>::new(live, 2, device_rate);
                            let mut device = Vec::with_capacity(device_rate as usize * 4 + 4);
                            for sample in converted {
                                device.push(sample);
                            }
                            let live_measure = measured(&device, device_rate);
                            let quality_source = crate::quality_source::QualitySource::new(
                                source(samples.clone(), source_rate, &settings, gain),
                                device_rate,
                            )
                            .unwrap();
                            let quality: Vec<f32> = quality_source.collect();
                            let quality_measure = measured(&quality, device_rate);
                            let mut witnesses = serde_json::Value::Null;
                            if source_rate == 96000
                                && requested_rate == 44100
                                && device_rate == 48000
                                && signal == "high"
                                && target == Some(-14.)
                                && preview
                            {
                                let folder = path.with_extension("witness");
                                assert!(!folder.exists());
                                std::fs::create_dir(&folder).unwrap();
                                witnesses = json!({"folder":folder,
                                    "source_sha256":write_witness(&folder.join("source.wav"),&samples,source_rate),
                                    "legacy_live_sha256":write_witness(&folder.join("legacy-live.wav"),&device,device_rate),
                                    "quality_live_sha256":write_witness(&folder.join("quality-live.wav"),&quality,device_rate),
                                    "export_sha256":write_witness(&folder.join("export.wav"),&delivered,requested_rate)});
                            }
                            rows.push(json!({"signal":signal,"frequency":frequency,"source_rate":source_rate,
                                "requested_rate":requested_rate,"device_rate":device_rate,"target":target,
                                "preview_lufs":preview,"landing_gain":gain,"predicted_lufs":landing.mastered_lufs,
                                "live":live_measure,"quality_live":quality_measure,"protected_pcm":delivery,"witnesses":witnesses,
                                "codec_rates":{"mp3":ExportEncoding::Mp3{bitrate_kbps:320}.delivery_rate(requested_rate),
                                    "m4a":ExportEncoding::M4a{bitrate_kbps:256}.delivery_rate(requested_rate)}}));
                        }
                    }
                }
            }
        }
    }
    std::fs::write(path,serde_json::to_vec_pretty(&json!({"status":"complete",
        "scope":"actual MasteringSource and Rodio UniformSourceIterator collected offline; final file flush differs from live finite stop; fixed settings, no device/OS/listening proof",
        "peak_version":peak_meter::VERSION,"rows":rows})).unwrap()).unwrap();
}

#[test]
#[ignore = "offline finite-alignment/gain-stage witness; set YES_MASTER_RATE_REPORT"]
fn mastering_quality_finite_live_gain_stages() {
    let path = PathBuf::from(std::env::var("YES_MASTER_RATE_REPORT").unwrap());
    assert!(!path.exists());
    let mut rows = Vec::new();
    for (rate, frequency) in [(96000, 38400_f32), (44100, 997_f32)] {
        let samples: Vec<f32> = (0..rate * 2)
            .flat_map(|n| {
                let x = 0.4 * (n as f32 * frequency * std::f32::consts::TAU / rate as f32).sin();
                [x, x * 0.8]
            })
            .collect();
        for requested in [44100, 48000] {
            let settings: MasteringSettings = serde_json::from_value(json!({
                "preset":{"kind":"custom","id":"rate-probe"},"intensity":0.,"volume_match":false,
                "eq_low_db":0.,"eq_mid_db":0.,"eq_high_db":0.,"delivery_profile":"custom",
                "advanced":{"lufs_offset_db":-14.,"ceiling_dbtp":-1.,"bit_depth":32,
                    "compression_mode":"off","warmth":0.,"target_sample_rate":requested}
            }))
            .unwrap();
            let landing = crate::engine::preview_landing(&samples, rate, 2, &settings).unwrap();
            let mut raw = samples.clone();
            let mut chain = MasteringChain::new(rate, 2, &settings);
            chain.process_interleaved(&mut raw, 2);
            chain.flush_render_tail(&mut raw, 2);
            for device in [44100, 48000] {
                let neutral: Vec<f32> = crate::quality_source::QualitySource::new(
                    source(samples.clone(), rate, &settings, 1.).with_render_alignment(),
                    device,
                )
                .unwrap()
                .collect();
                let offline =
                    crate::sample_rate::convert_interleaved(&raw, rate, device, 2).unwrap();
                assert_eq!(neutral, offline, "aligned live/offline samples differ");
                let pre: Vec<f32> = crate::quality_source::QualitySource::new(
                    source(samples.clone(), rate, &settings, landing.gain_lin)
                        .with_render_alignment(),
                    device,
                )
                .unwrap()
                .collect();
                let post: Vec<f32> = neutral.iter().map(|x| x * landing.gain_lin).collect();
                let mut protected = post.clone();
                let cap = crate::output_protection::finalize(
                    &mut protected,
                    device,
                    2,
                    32,
                    None,
                    -1.,
                    None,
                )
                .unwrap();
                assert!(cap.true_peak_dbtp <= -1. + 1e-5);
                // Audition the requested file-rate signal, then convert that
                // same signal to the device, as an external file player does.
                // Compare this route independently of direct source->device.
                let mut file_rate =
                    crate::sample_rate::convert_interleaved(&raw, rate, requested, 2).unwrap();
                for sample in &mut file_rate {
                    *sample *= landing.gain_lin;
                }
                let cascade_reference =
                    crate::sample_rate::convert_interleaved(&file_rate, requested, device, 2)
                        .unwrap();
                let file_stream = crate::quality_source::QualitySource::new(
                    source(samples.clone(), rate, &settings, 1.).with_render_alignment(),
                    requested,
                )
                .unwrap()
                .amplify(landing.gain_lin);
                let cascade: Vec<f32> =
                    crate::quality_source::QualitySource::new(file_stream, device)
                        .unwrap()
                        .collect();
                assert_eq!(
                    cascade, cascade_reference,
                    "file-rate cascade differs from the rendered signal"
                );
                let mut cascade_capped = cascade.clone();
                let cascade_cap = crate::output_protection::finalize(
                    &mut cascade_capped,
                    device,
                    2,
                    32,
                    None,
                    -1.,
                    None,
                )
                .unwrap();
                assert!(cascade_cap.true_peak_dbtp <= -1. + 1e-5);
                let mut witnesses = json!(null);
                if rate == 96000 && requested == 44100 && device == 48000 {
                    let folder = path.with_extension("witness");
                    assert!(!folder.exists());
                    std::fs::create_dir(&folder).unwrap();
                    witnesses = json!({"folder":folder,
                        "pre_sha256":write_witness(&folder.join("aligned-pre.wav"), &pre, device),
                        "post_sha256":write_witness(&folder.join("aligned-post.wav"), &post, device),
                        "cap_sha256":write_witness(&folder.join("aligned-capped.wav"), &protected, device),
                        "cascade_sha256":write_witness(&folder.join("file-rate-cascade.wav"), &cascade, device),
                        "cascade_capped_sha256":write_witness(&folder.join("file-rate-cascade-capped.wav"), &cascade_capped, device)});
                }
                rows.push(json!({"source_rate":rate,"frequency":frequency,"requested_rate":requested,
                    "device_rate":device,"export_gain":landing.gain_lin,"pre_src_gain":measured(&pre,device),
                    "post_src_gain":measured(&post,device),"capped":measured(&protected,device),
                    "additional_device_cap_gain":cap.gain_lin,"file_rate_cascade":measured(&cascade,device),
                    "file_rate_cascade_capped":measured(&cascade_capped,device),
                    "cascade_additional_gain":cascade_cap.gain_lin,"witnesses":witnesses}));
            }
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&json!({"status":"complete",
        "scope":"offline finite-source equivalence and gain placement; no native playback or policy adoption",
        "rows":rows})).unwrap()).unwrap();
}
