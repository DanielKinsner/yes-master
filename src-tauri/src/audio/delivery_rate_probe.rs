//! Opt-in offline capture of the actual MasteringSource and Rodio device SRC.
//! No output device or listening verdict; separate native probes cover those.
use super::*;
use crate::{dsp::MasteringChain, export_format::ExportEncoding, peak_meter};
use rodio::source::UniformSourceIterator;
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
                let mut witnesses = json!(null);
                if rate == 96000 && requested == 44100 && device == 48000 {
                    let folder = path.with_extension("witness");
                    assert!(!folder.exists());
                    std::fs::create_dir(&folder).unwrap();
                    witnesses = json!({"folder":folder,
                        "pre_sha256":write_witness(&folder.join("aligned-pre.wav"), &pre, device),
                        "post_sha256":write_witness(&folder.join("aligned-post.wav"), &post, device),
                        "cap_sha256":write_witness(&folder.join("aligned-capped.wav"), &protected, device)});
                }
                rows.push(json!({"source_rate":rate,"frequency":frequency,"requested_rate":requested,
                    "device_rate":device,"export_gain":landing.gain_lin,"pre_src_gain":measured(&pre,device),
                    "post_src_gain":measured(&post,device),"capped":measured(&protected,device),
                    "additional_device_cap_gain":cap.gain_lin,"witnesses":witnesses}));
            }
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&json!({"status":"complete",
        "scope":"offline finite-source equivalence and gain placement; no native playback or policy adoption",
        "rows":rows})).unwrap()).unwrap();
}
