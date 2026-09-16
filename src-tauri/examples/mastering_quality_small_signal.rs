//! Low-drive response diagnostic using retained settings; no selector or preset change.
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

fn write(path: &Path, samples: &[f32]) {
    assert!(!path.exists());
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for &sample in samples {
        assert!(sample.is_finite());
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}

fn render(
    input: &[f32],
    rate: u32,
    settings: &types::MasteringSettings,
    drive: f32,
    offset: f32,
    expected_coefficients: &str,
) -> (Vec<f32>, Value) {
    let mut chain = MasteringChain::new(rate, 2, settings);
    chain.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
    assert_eq!(format!("{:?}", chain.coeffs), expected_coefficients);
    let factor = 10_f32.powf(offset / 20.);
    chain.coeffs.input_gain_lin *= factor;
    let delay = chain.latency_frames();
    let mut samples = input.to_vec();
    samples.resize(samples.len() + delay * 2, 0.);
    let mut limiter_max_db = 0_f32;
    let start = Instant::now();
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
        json!({"chain_compensation_and_observation_s":start.elapsed().as_secs_f64(),
        "extra_input_gain":factor,"limiter_max_db":limiter_max_db,
        "compressor_max_db_rounded_down_0_01":compressor_max_db,
        "base_coefficients":expected_coefficients}),
    )
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "job.json fresh-output-directory");
    let job_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job = read(job_path);
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"].as_str().unwrap()
    );
    let baseline_path = Path::new(job["baseline_report"].as_str().unwrap());
    assert_eq!(
        sha(baseline_path),
        job["baseline_report_sha256"].as_str().unwrap()
    );
    let baseline = read(baseline_path);
    assert_eq!(baseline["status"], "complete");
    std::fs::create_dir_all(output).unwrap();
    let mut report = json!({"status":"running","job_sha256":sha(job_path),"rows":[],"convergence":[],
        "maximum_allowed_low_drive_difference":1e-5,
        "scope":"Low-drive response diagnostic, compensated after unchanged production chain; no expanded C grid or sonic adoption"});
    for case in job["cases"].as_array().unwrap() {
        let prior_path = Path::new(case["report"].as_str().unwrap());
        assert_eq!(sha(prior_path), case["report_sha256"].as_str().unwrap());
        let prior = read(prior_path);
        assert_eq!(prior["status"], "complete");
        let retained = prior["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == case["candidate"])
            .unwrap();
        assert_eq!(retained["source_gain_db"], 0.);
        let source = Path::new(prior["source"].as_str().unwrap());
        assert_eq!(sha(source), prior["source_sha256"].as_str().unwrap());
        let pcm = decode::decode_full(source).unwrap();
        assert_eq!(pcm.channels, 2);
        let mut input = pcm.samples;
        if retained["policy"] != "control" {
            assert_eq!(retained["policy"], "single");
            let peak = input.iter().map(|x| f64::from(x.abs())).fold(0., f64::max);
            for sample in &mut input {
                *sample = (f64::from(*sample) / peak) as f32;
            }
        }
        let settings: types::MasteringSettings =
            serde_json::from_value(retained["requested_settings"].clone()).unwrap();
        let drive = retained["operating_drive_db"].as_f64().unwrap() as f32;
        let coefficients = retained["render"]["coefficients"].as_str().unwrap();
        let control = baseline["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["case"] == case["id"] && row["variant"] == "current")
            .unwrap();
        let mut lower: Option<Vec<f32>> = None;
        for offset in [0., -24., -48.] {
            let (raw, render) = render(
                &input,
                pcm.sample_rate,
                &settings,
                drive,
                offset,
                coefficients,
            );
            if offset == -48. {
                let previous = lower.take().unwrap();
                assert_eq!(previous.len(), raw.len());
                let maximum = previous
                    .iter()
                    .zip(&raw)
                    .map(|(&a, &b)| (f64::from(a) - f64::from(b)).abs())
                    .fold(0., f64::max);
                report["convergence"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"case":case["id"],"maximum":maximum,"pass":maximum<=1e-5}));
            } else if offset == -24. {
                lower = Some(raw.clone());
            }
            let start = Instant::now();
            let mut delivered =
                sample_rate::convert_interleaved(&raw, pcm.sample_rate, 48000, 2).unwrap();
            let src_s = start.elapsed().as_secs_f64();
            let start = Instant::now();
            let protection = output_protection::finalize(
                &mut delivered,
                48000,
                2,
                32,
                settings.effective_target_lufs(),
                settings.effective_ceiling_dbtp(),
                None,
            )
            .unwrap();
            let finalize_s = start.elapsed().as_secs_f64();
            assert!(protection.true_peak_dbtp <= settings.effective_ceiling_dbtp());
            let path = if offset == 0. {
                let path = Path::new(control["path"].as_str().unwrap());
                assert_eq!(sha(path), control["sha256"].as_str().unwrap());
                let reference = decode::decode_full(path).unwrap();
                assert_eq!(delivered.len(), reference.samples.len());
                assert!(
                    delivered
                        .iter()
                        .zip(&reference.samples)
                        .all(|(a, b)| a.to_bits() == b.to_bits()),
                    "zero-offset must reproduce retained E production control exactly"
                );
                path.to_owned()
            } else {
                let path = output.join(format!(
                    "{}-m{}.wav",
                    case["id"].as_str().unwrap(),
                    -offset as u32
                ));
                write(&path, &delivered);
                path
            };
            report["rows"].as_array_mut().unwrap().push(json!({"case":case["id"],"offset_db":offset,
                "path":path,"sha256":sha(&path),"source":source,"source_sha256":prior["source_sha256"],
                "requested_settings":settings,"drive":drive,"render":render,"src_s":src_s,"finalize_s":finalize_s,
                "frames":delivered.len()/2,"rate":48000,"channels":2,"gain":protection.gain_lin,
                "lufs":protection.lufs,"peak":protection.true_peak_dbtp,"ceiling":settings.effective_ceiling_dbtp(),
                "reused_exact_control":offset==0.}));
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!("Completed {} offset {offset}", case["id"]);
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
