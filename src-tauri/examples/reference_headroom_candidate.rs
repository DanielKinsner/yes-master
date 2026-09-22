//! Frozen reference-headroom candidate. Never invoked by production mastering.
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::Ordering, time::Instant};
use yes_master_lib::{decode, dsp::MasteringChain, output_protection, types};
#[path = "../src/sample_rate.rs"]
mod sample_rate;

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn lufs(samples: &[f32], rate: u32) -> Option<f64> {
    let mut meter = EbuR128::new(2, rate, Mode::I).unwrap();
    meter.add_frames_f32(samples).unwrap();
    let value = meter.loudness_global().unwrap();
    value.is_finite().then_some(value)
}
fn coefficients(settings: &types::MasteringSettings, rate: u32, drive: f32) -> String {
    let mut chain = MasteringChain::new(rate, 2, settings);
    chain.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
    format!("{:?}", chain.coeffs)
}
fn render(
    input: &[f32],
    rate: u32,
    settings: &types::MasteringSettings,
    drive: f32,
    offset: f32,
    coefficients: &str,
) -> (Vec<f32>, Value) {
    let mut chain = MasteringChain::new(rate, 2, settings);
    chain.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
    assert_eq!(format!("{:?}", chain.coeffs), coefficients);
    let factor = 10_f32.powf(offset / 20.);
    chain.coeffs.input_gain_lin *= factor;
    let delay = chain.latency_frames();
    let mut samples = input.to_vec();
    samples.resize(samples.len() + delay * 2, 0.);
    let start = Instant::now();
    let mut limiter_max_db = 0_f32;
    for frame in samples.chunks_exact_mut(2) {
        chain.process_frame_inplace(frame);
        limiter_max_db = limiter_max_db.max(chain.limiter.gain_reduction_db());
    }
    samples.copy_within(delay * 2.., 0);
    samples.truncate(input.len());
    for sample in &mut samples {
        *sample = (f64::from(*sample) / f64::from(factor)) as f32;
    }
    let compressor_max_db: Vec<_> = [
        &chain.gr_snapshots.low,
        &chain.gr_snapshots.mid,
        &chain.gr_snapshots.high,
    ]
    .iter()
    .map(|slot| slot.load(Ordering::Relaxed) as f64 / 100.)
    .collect();
    (
        samples,
        json!({"chain_s":start.elapsed().as_secs_f64(),"limiter_max_db":limiter_max_db,
        "compressor_max_db_rounded_down_0_01":compressor_max_db,"base_coefficients":coefficients}),
    )
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let job_path = Path::new(&args[1]);
    let directory = Path::new(&args[2]);
    assert!(!directory.exists());
    let job = read(job_path);
    assert_eq!(job["experiment"], "reference-headroom-candidate-v1");
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"]
    );
    std::fs::create_dir_all(directory).unwrap();
    let mut report =
        json!({"status":"running","job_sha256":sha(job_path),"rows":[],"source_preparation":[]});
    for case in job["cases"].as_array().unwrap() {
        let source = Path::new(case["source"].as_str().unwrap());
        assert_eq!(sha(source), case["source_sha256"]);
        let start = Instant::now();
        let pcm = decode::decode_full(source).unwrap();
        assert_eq!(pcm.channels, 2);
        let decode_s = start.elapsed().as_secs_f64();
        let settings: types::MasteringSettings =
            serde_json::from_value(case["settings"].clone()).unwrap();
        assert_eq!(settings.effective_target_lufs(), Some(-14.));
        assert_eq!(settings.intensity, 0.5);
        assert_eq!(settings.input_gain_db, 0.);
        assert_eq!(settings.output_gain_db, 0.);
        assert!(!settings.volume_match);
        assert_eq!(
            coefficients(
                &settings,
                pcm.sample_rate,
                case["original_drive"].as_f64().unwrap() as f32
            ),
            case["original_coefficients"]
        );
        let chain = MasteringChain::new(pcm.sample_rate, 2, &settings);
        let push = case["preset_push_db"].as_f64().unwrap();
        let saturation = case["saturation_small_signal_db"].as_f64().unwrap();
        let sat_drive = 1. + f64::from(chain.coeffs.saturation_amount) * 2.;
        let actual_sat = if chain.coeffs.saturation_amount > 0. {
            20. * (sat_drive / sat_drive.tanh().max(1e-3)).log10()
        } else {
            0.
        };
        assert!((20. * f64::from(chain.coeffs.input_gain_lin).log10() - push).abs() < 1e-5);
        assert!((actual_sat - saturation).abs() < 1e-5);
        let mut original: Option<Vec<f32>> = None;
        let mut descriptors = Vec::new();
        for reference in case["references"].as_array().unwrap() {
            let copy_db = reference["source_gain_db"].as_f64().unwrap();
            assert_eq!(
                sha(Path::new(reference["path"].as_str().unwrap())),
                reference["sha256"]
            );
            let start = Instant::now();
            let gain = 10_f64.powf(copy_db / 20.);
            let mut input: Vec<f32> = pcm
                .samples
                .iter()
                .map(|v| (f64::from(*v) * gain) as f32)
                .collect();
            let actual_source_lufs = lufs(&input, pcm.sample_rate);
            let peak = input.iter().map(|v| f64::from(v.abs())).fold(0., f64::max);
            assert!(peak > 0.);
            for sample in &mut input {
                *sample = (f64::from(*sample) / peak) as f32;
            }
            let normalized_lufs = lufs(&input, pcm.sample_rate).unwrap();
            let reference_peak = reference["peak"].as_f64().unwrap();
            assert!(reference_peak > 0.);
            let drive = (-1. - 20. * reference_peak.log10() - push - saturation) as f32;
            let descriptor = json!({"source_gain_db":copy_db,"actual_source_lufs":actual_source_lufs,
                "normalized_lufs":normalized_lufs,"normalization_db":-20.*peak.log10(),
                "static_reference_peak":reference_peak,"preset_push_db":push,"saturation_small_signal_db":saturation,
                "drive_db":drive,"descriptor_s":start.elapsed().as_secs_f64()});
            descriptors.push(descriptor.clone());
            let coeffs = coefficients(&settings, pcm.sample_rate, drive);
            let (raw, render) = render(&input, pcm.sample_rate, &settings, drive, 0., &coeffs);
            let start = Instant::now();
            let mut delivered =
                sample_rate::convert_interleaved(&raw, pcm.sample_rate, 48000, 2).unwrap();
            let src_s = start.elapsed().as_secs_f64();
            assert_eq!(
                delivered.len() / 2,
                case["delivered_frames"].as_u64().unwrap() as usize
            );
            let start = Instant::now();
            let protected =
                output_protection::finalize(&mut delivered, 48000, 2, 32, Some(-14.), -1., None)
                    .unwrap();
            let finalize_s = start.elapsed().as_secs_f64();
            assert!(protected.true_peak_dbtp <= -1. && delivered.iter().all(|v| v.is_finite()));
            let difference = original.as_ref().map(|prior| {
                prior
                    .iter()
                    .zip(&delivered)
                    .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs())
                    .fold(0., f64::max)
            });
            if copy_db == 0. {
                original = Some(delivered.clone());
            }
            let id = format!("{}-copy{copy_db}-headroom", case["id"].as_str().unwrap());
            let path = directory.join(format!("{id}.wav"));
            let mut writer = hound::WavWriter::create(
                &path,
                hound::WavSpec {
                    channels: 2,
                    sample_rate: 48000,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .unwrap();
            for value in &delivered {
                writer.write_sample(*value).unwrap();
            }
            writer.finalize().unwrap();
            report["rows"].as_array_mut().unwrap().push(json!({"id":id,"case":case["id"],"policy":"headroom",
                "source_gain_db":copy_db,"source":source,"source_sha256":case["source_sha256"],
                "requested_settings":settings,"reference_id":reference["id"],"reference_sha256":reference["sha256"],
                "drive_db":drive,"descriptor":descriptor,"render":render,"src_s":src_s,"finalize_s":finalize_s,
                "path":path,"sha256":sha(&path),"frames":delivered.len()/2,"rate":48000,"channels":2,"ceiling":-1.,
                "peak":protected.true_peak_dbtp,"lufs":protected.lufs.is_finite().then_some(protected.lufs),
                "gain":protected.gain_lin,"maximum_copy_difference":difference}));
            std::fs::write(
                directory.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!("Completed {id}");
        }
        report["source_preparation"]
            .as_array_mut()
            .unwrap()
            .push(json!({"case":case["id"],"decode_s":decode_s,"descriptors":descriptors}));
    }
    assert_eq!(report["rows"].as_array().unwrap().len(), 12);
    report["status"] = json!("complete");
    std::fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
