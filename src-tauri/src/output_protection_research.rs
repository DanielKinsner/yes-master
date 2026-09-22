//! Explicitly opted-in research benchmark; absent from application builds.
use super::{finalize, finalize_prepared, prepare};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
#[ignore = "explicit frozen private whole-buffer reuse benchmark"]
fn prepared_measurement_reuse_benchmark() {
    let job_path = std::env::var("YES_MASTER_PREPARED_REUSE_JOB").unwrap();
    let output = std::env::var("YES_MASTER_PREPARED_REUSE_OUTPUT").unwrap();
    let job_path = Path::new(&job_path);
    let output = Path::new(&output);
    assert!(!output.exists());
    let job = read(job_path);
    assert_eq!(job["experiment"], "prepared-measurement-reuse-v1");
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"]
    );
    assert_eq!(
        sha(Path::new(job["baseline_summary"].as_str().unwrap())),
        job["baseline_summary_sha256"]
    );
    assert_eq!(job["cases"].as_array().unwrap().len(), 4);
    std::fs::create_dir_all(output).unwrap();
    let mut report = json!({"status":"running","experiment":job["experiment"],"job_sha256":sha(job_path),
        "rows":[],"source_preparation":[]});
    for case in job["cases"].as_array().unwrap() {
        let path = Path::new(case["prepared_path"].as_str().unwrap());
        let expected_path = Path::new(case["expected_path"].as_str().unwrap());
        assert_eq!(sha(path), case["prepared_sha256"]);
        assert_eq!(sha(expected_path), case["expected_sha256"]);
        let expected = crate::decode::decode_full(expected_path).unwrap();
        let start = Instant::now();
        let source = crate::decode::decode_full(path).unwrap();
        let decode_s = start.elapsed().as_secs_f64();
        assert_eq!((source.sample_rate, source.channels), (48000, 2));
        assert_eq!((expected.sample_rate, expected.channels), (48000, 2));
        assert_eq!(source.samples.len(), expected.samples.len());
        let start = Instant::now();
        let measurements = prepare(&source.samples, 48000, 2, None).unwrap();
        let prepare_s = start.elapsed().as_secs_f64();
        report["source_preparation"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":case["id"],
            "decode_s":decode_s,"prepare_s":prepare_s,"prepared_sha256":case["prepared_sha256"]}));
        for (round, order) in [[false, true], [true, false]].iter().enumerate() {
            for reused in order {
                let start = Instant::now();
                let mut samples = source.samples.clone();
                let protected = if *reused {
                    finalize_prepared(&mut samples, &measurements, 32, Some(-9.), -1., None)
                        .unwrap()
                } else {
                    finalize(&mut samples, 48000, 2, 32, Some(-9.), -1., None).unwrap()
                };
                let finalization_s = start.elapsed().as_secs_f64();
                let exact = samples
                    .iter()
                    .zip(&expected.samples)
                    .all(|(a, b)| a.to_bits() == b.to_bits());
                let row = json!({"id":case["id"],"round":round,"reused_measurements":reused,
                    "finalization_s":finalization_s,"exact_pcm":exact,"expected_sha256":case["expected_sha256"],
                    "frames":samples.len()/2,"rate":48000,"channels":2,"lufs":protected.lufs,
                    "peak":protected.true_peak_dbtp,"gain":protected.gain_lin});
                report["rows"].as_array_mut().unwrap().push(row);
                std::fs::write(
                    output.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
                assert!(
                    exact,
                    "complete delivered PCM must match its verified reference"
                );
                assert!(protected.true_peak_dbtp <= -1.);
                assert_eq!(
                    protected.lufs.to_bits(),
                    (case["expected_lufs"].as_f64().unwrap() as f32).to_bits()
                );
                assert_eq!(
                    protected.true_peak_dbtp.to_bits(),
                    (case["expected_peak"].as_f64().unwrap() as f32).to_bits()
                );
                assert_eq!(
                    protected.gain_lin.to_bits(),
                    (case["expected_gain"].as_f64().unwrap() as f32).to_bits()
                );
                println!(
                    "{} round {round} reused={reused}: {finalization_s:.6}s",
                    case["id"]
                );
            }
        }
    }
    assert_eq!(report["rows"].as_array().unwrap().len(), 16);
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
