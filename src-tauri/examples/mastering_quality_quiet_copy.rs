//! Frozen quiet-copy diagnostic. Never invoked by production mastering.
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
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job = read(job_path);
    assert_eq!(job["experiment"], "quiet-copy-bidirectional-v1");
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"]
    );
    assert_eq!(job["cases"].as_array().unwrap().len(), 4);
    std::fs::create_dir_all(output).unwrap();
    let session = Instant::now();
    let mut report = json!({"status":"running","experiment":job["experiment"],
        "job_sha256":sha(job_path),"rows":[],"source_preparation":[]});
    for case in job["cases"].as_array().unwrap() {
        let name = case["id"].as_str().unwrap();
        assert!(matches!(name, "piano" | "imaginal" | "metal" | "rich"));
        let source = Path::new(case["source"].as_str().unwrap());
        assert_eq!(sha(source), case["source_sha256"]);
        let start = Instant::now();
        let pcm = decode::decode_full(source).unwrap();
        assert_eq!(pcm.channels, 2);
        let decode_s = start.elapsed().as_secs_f64();
        let reference_rows = case["originals"].as_array().unwrap();
        assert_eq!(reference_rows.len(), 4);
        let zero = reference_rows
            .iter()
            .find(|r| r["policy"] == "single" && r["offset_db"] == 0.)
            .unwrap();
        let settings: types::MasteringSettings =
            serde_json::from_value(zero["requested_settings"].clone()).unwrap();
        assert_eq!(settings.effective_target_lufs(), Some(-14.));
        assert_eq!(settings.intensity, 0.5);
        assert_eq!(settings.input_gain_db, 0.);
        assert!(!settings.volume_match);
        let old_drive = zero["drive_db"].as_f64().unwrap() as f32;
        assert_eq!(
            coefficients(&settings, pcm.sample_rate, old_drive),
            zero["render"]["base_coefficients"]
        );
        let mut preparation = json!({"case":name,"decode_s":decode_s,"copies":[]});
        for copy_db in [-40_f64, -80.] {
            let start = Instant::now();
            let gain = 10_f64.powf(copy_db / 20.);
            let copied: Vec<f32> = pcm
                .samples
                .iter()
                .map(|s| (f64::from(*s) * gain) as f32)
                .collect();
            let mut hasher = Sha256::new();
            for sample in &copied {
                hasher.update(sample.to_le_bytes());
            }
            let copy_sha256 = format!("{:x}", hasher.finalize());
            let peak = copied.iter().map(|s| f64::from(s.abs())).fold(0., f64::max);
            assert!(peak > 0.);
            let normalized: Vec<f32> = copied
                .iter()
                .map(|s| (f64::from(*s) / peak) as f32)
                .collect();
            let normalized_lufs = lufs(&normalized, pcm.sample_rate).unwrap();
            let actual_lufs = lufs(&copied, pcm.sample_rate);
            let descriptor_s = start.elapsed().as_secs_f64();
            let drive = -14_f32
                - normalized_lufs as f32
                - case["preset_push_db"].as_f64().unwrap() as f32
                - case["saturation_small_signal_db"].as_f64().unwrap() as f32;
            let descriptor = json!({"source_gain_db":copy_db,"copy_pcm_sha256":copy_sha256,
                "actual_source_lufs":actual_lufs,"normalized_lufs":normalized_lufs,
                "normalization_db":-20.*peak.log10(),"drive_db":drive,"descriptor_s":descriptor_s});
            preparation["copies"]
                .as_array_mut()
                .unwrap()
                .push(descriptor.clone());
            for (policy, offset) in [
                ("control", 0_f32),
                ("single", 0.),
                ("single", -3.),
                ("single", 3.),
            ] {
                let reference = reference_rows
                    .iter()
                    .find(|r| r["policy"] == policy && r["offset_db"] == offset)
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&settings).unwrap(),
                    reference["requested_settings"]
                );
                let use_drive = if policy == "control" { 0. } else { drive };
                let input = if policy == "control" {
                    &copied
                } else {
                    &normalized
                };
                let coeffs = coefficients(&settings, pcm.sample_rate, use_drive);
                let (raw, render) = render(
                    input,
                    pcm.sample_rate,
                    &settings,
                    use_drive,
                    offset,
                    &coeffs,
                );
                let start = Instant::now();
                let mut delivered =
                    sample_rate::convert_interleaved(&raw, pcm.sample_rate, 48000, 2).unwrap();
                let src_s = start.elapsed().as_secs_f64();
                assert_eq!(
                    delivered.len() / 2,
                    reference["frames"].as_u64().unwrap() as usize
                );
                let start = Instant::now();
                let protected = output_protection::finalize(
                    &mut delivered,
                    48000,
                    2,
                    32,
                    Some(-14.),
                    -1.,
                    None,
                )
                .unwrap();
                let finalize_s = start.elapsed().as_secs_f64();
                assert!(protected.true_peak_dbtp <= -1. && delivered.iter().all(|v| v.is_finite()));
                let id = format!("{name}-copy{copy_db}-{policy}-offset{offset}");
                let path = output.join(format!("{id}.wav"));
                assert!(!path.exists());
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
                for &sample in &delivered {
                    writer.write_sample(sample).unwrap();
                }
                writer.finalize().unwrap();
                let pcm_difference = if policy == "single" {
                    let old_path = Path::new(reference["path"].as_str().unwrap());
                    assert_eq!(sha(old_path), reference["sha256"]);
                    let old = decode::decode_full(old_path).unwrap();
                    assert_eq!(
                        (old.sample_rate, old.channels, old.samples.len()),
                        (48000, 2, delivered.len())
                    );
                    let maximum = delivered
                        .iter()
                        .zip(&old.samples)
                        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs())
                        .fold(0., f64::max);
                    Some(maximum)
                } else {
                    None
                };
                let delivered_lufs = protected.lufs.is_finite().then_some(protected.lufs);
                report["rows"].as_array_mut().unwrap().push(json!({"id":id,"case":name,"policy":policy,
                    "offset_db":offset,"source_gain_db":copy_db,"descriptor":descriptor,"source":source,
                    "source_sha256":case["source_sha256"],"path":path,"sha256":sha(&path),"requested_settings":settings,
                    "drive_db":use_drive,"render":render,"src_s":src_s,"finalize_s":finalize_s,
                    "frames":delivered.len()/2,"rate":48000,"channels":2,"ceiling":-1.,"peak":protected.true_peak_dbtp,
                    "lufs":delivered_lufs,"loudness_unavailable_reason":if delivered_lufs.is_none(){Some("below_integrated_gate")}else{None},
                    "gain":protected.gain_lin,"original_id":reference["id"],"original_sha256":reference["sha256"],
                    "maximum_delivered_copy_difference":pcm_difference}));
                std::fs::write(
                    output.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
                println!("Completed {id}");
            }
        }
        report["source_preparation"]
            .as_array_mut()
            .unwrap()
            .push(preparation);
    }
    assert_eq!(report["rows"].as_array().unwrap().len(), 32);
    report["status"] = json!("complete");
    report["total_wall_s"] = json!(session.elapsed().as_secs_f64());
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
