//! Bounded offline development runner; never called by production mastering.
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

// Same frame processing, alignment, compensation and diagnostics as the frozen
// lower-grid runner. Reproduction anchors below verify the resulting whole PCM.
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
    assert_eq!(args.len(), 3, "job.json fresh-output-directory");
    let job_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job = read(job_path);
    let followup = job["experiment"] == "flagged-lower-drive-v1";
    let broader = job["experiment"] == "broader-first-v1";
    assert!(followup || broader || job["experiment"] == "single-first-development-v1");
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"]
    );
    let baseline = if followup {
        let path = Path::new(job["baseline_report"].as_str().unwrap());
        assert_eq!(sha(path), job["baseline_report_sha256"]);
        let baseline = read(path);
        assert_eq!(baseline["status"], "complete");
        assert_eq!(baseline["experiment"], "single-first-development-v1");
        let check_path = Path::new(job["baseline_independent"].as_str().unwrap());
        assert_eq!(sha(check_path), job["baseline_independent_sha256"]);
        let checks = read(check_path);
        assert_eq!(checks["status"], "complete");
        assert_eq!(checks["all_pass"], true);
        assert_eq!(checks["native_sha256"], sha(path));
        Some(baseline)
    } else {
        None
    };
    std::fs::create_dir_all(output).unwrap();
    let session = Instant::now();
    let mut report = json!({"status":"running","experiment":job["experiment"],
        "job_sha256":sha(job_path),"rows":[],"source_preparation":[],
        "scope":"known-source offline development; no production policy or app latency claim"});
    for case in job["cases"].as_array().unwrap() {
        if broader {
            assert!(matches!(case["id"].as_str().unwrap(), "funk" | "rich"));
            assert_eq!(case["candidates"].as_array().unwrap().len(), 8);
        }
        if followup {
            assert!(matches!(case["id"].as_str().unwrap(), "metal" | "rich"));
            assert_eq!(case["candidates"].as_array().unwrap().len(), 5);
        }
        let prior_path = Path::new(case["report"].as_str().unwrap());
        assert_eq!(sha(prior_path), case["report_sha256"]);
        let prior = read(prior_path);
        assert_eq!(prior["status"], "complete");
        let source = Path::new(prior["source"].as_str().unwrap());
        assert_eq!(sha(source), prior["source_sha256"]);
        let start = Instant::now();
        let pcm = decode::decode_full(source).unwrap();
        assert_eq!(pcm.channels, 2);
        let decode_s = start.elapsed().as_secs_f64();
        let start = Instant::now();
        let peak = pcm
            .samples
            .iter()
            .map(|s| f64::from(s.abs()))
            .fold(0., f64::max);
        assert!(peak > 0.);
        let normalized: Vec<f32> = pcm
            .samples
            .iter()
            .map(|s| (f64::from(*s) / peak) as f32)
            .collect();
        report["source_preparation"].as_array_mut().unwrap().push(json!({"case":case["id"],
            "decode_s":decode_s,"normalization_s":start.elapsed().as_secs_f64(),"source_sha256":prior["source_sha256"]}));
        for requested in case["candidates"].as_array().unwrap() {
            let retained = prior["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["id"] == requested["id"])
                .unwrap();
            assert_eq!(retained["source_gain_db"], 0.);
            let policy = retained["policy"].as_str().unwrap();
            assert!(matches!(policy, "control" | "single"));
            let input = if policy == "single" {
                &normalized
            } else {
                &pcm.samples
            };
            let settings: types::MasteringSettings =
                serde_json::from_value(retained["requested_settings"].clone()).unwrap();
            let drive = retained["operating_drive_db"].as_f64().unwrap() as f32;
            let offset = requested["offset_db"].as_f64().unwrap() as f32;
            if followup {
                assert!([0., -3., -6., -12.].contains(&offset));
                assert!(policy == "single" || offset == 0.);
            } else {
                assert_eq!(offset, 0., "single-first freezes zero-offset candidates");
            }
            let id = if broader {
                assert!(matches!(
                    retained["preset_id"].as_str().unwrap(),
                    "universal50" | "loud75"
                ));
                assert!([-14., -9.].contains(&settings.effective_target_lufs().unwrap()));
                requested["output_id"].as_str().unwrap().to_owned()
            } else if offset == 0. {
                format!("{}-{policy}", case["id"].as_str().unwrap())
            } else {
                format!("{}-{policy}-m{}", case["id"].as_str().unwrap(), -offset)
            };
            assert!(!report["rows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["id"] == id));
            if let Some(reuse_id) = requested["reuse_row_id"].as_str() {
                assert!(followup && offset == 0.);
                assert_eq!(reuse_id, id);
                let mut row = baseline.as_ref().unwrap()["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|row| row["id"] == reuse_id)
                    .unwrap()
                    .clone();
                assert_eq!(row["source_sha256"], prior["source_sha256"]);
                // Compare the exact typed settings consumed by DSP. Parsing a
                // JSON f64 value and serializing an f32 field can differ in the
                // last decimal digit while preserving every processing bit.
                let baseline_settings: types::MasteringSettings =
                    serde_json::from_value(row["requested_settings"].clone()).unwrap();
                assert_eq!(
                    serde_json::to_value(&baseline_settings).unwrap(),
                    serde_json::to_value(&settings).unwrap()
                );
                assert_eq!(
                    row["render"]["base_coefficients"],
                    retained["render"]["coefficients"]
                );
                assert_eq!(row["policy"], policy);
                assert_eq!(row["drive_db"].as_f64().unwrap() as f32, drive);
                assert_eq!(row["offset_db"], 0.);
                assert_eq!(sha(Path::new(row["path"].as_str().unwrap())), row["sha256"]);
                row["reused_validated_baseline"] = json!(true);
                row["baseline_report_sha256"] = job["baseline_report_sha256"].clone();
                report["rows"].as_array_mut().unwrap().push(row);
                continue;
            }
            assert!(
                !followup || offset != 0.,
                "follow-up must reuse its verified zero-offset outputs"
            );
            let (raw, metrics) = render(
                input,
                pcm.sample_rate,
                &settings,
                drive,
                offset,
                retained["render"]["coefficients"].as_str().unwrap(),
            );
            let start = Instant::now();
            let mut delivered =
                sample_rate::convert_interleaved(&raw, pcm.sample_rate, 48000, 2).unwrap();
            let src_s = start.elapsed().as_secs_f64();
            assert_eq!(
                delivered.len() / 2,
                ((pcm.samples.len() / 2) as u128 * 48000).div_ceil(u128::from(pcm.sample_rate))
                    as usize
            );
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
            assert!(delivered.iter().all(|s| s.is_finite()));
            let reused = requested["expected_path"].as_str();
            let path = if let Some(path) = reused {
                let path = Path::new(path);
                assert_eq!(sha(path), requested["expected_sha256"]);
                let expected = decode::decode_full(path).unwrap();
                assert_eq!(expected.sample_rate, 48000);
                assert_eq!(expected.channels, 2);
                assert_eq!(expected.samples.len(), delivered.len());
                assert!(
                    delivered
                        .iter()
                        .zip(expected.samples)
                        .all(|(a, b)| a.to_bits() == b.to_bits()),
                    "exact retained anchor changed"
                );
                path.to_owned()
            } else {
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
                path
            };
            report["rows"].as_array_mut().unwrap().push(json!({"id":id,
                "case":case["id"],"policy":policy,"preset_id":retained["preset_id"],
                "comparison_group":requested["comparison_group"],"path":path,"sha256":sha(&path),"source_sha256":prior["source_sha256"],
                "source":source,"requested_settings":settings,"drive_db":drive,"offset_db":offset,
                "render":metrics,"src_s":src_s,"finalize_s":finalize_s,"frames":delivered.len()/2,"rate":48000,"channels":2,
                "lufs":protection.lufs,"peak":protection.true_peak_dbtp,"ceiling":settings.effective_ceiling_dbtp(),
                "gain":protection.gain_lin,"reused_exact_anchor":reused.is_some(),"reused_validated_baseline":false}));
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!("Completed {id}");
        }
    }
    assert_eq!(
        report["rows"].as_array().unwrap().len(),
        if followup {
            10
        } else if broader {
            16
        } else {
            12
        }
    );
    report["status"] = json!("complete");
    report["total_wall_s"] = json!(session.elapsed().as_secs_f64());
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
