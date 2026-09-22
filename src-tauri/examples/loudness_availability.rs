//! Offline meter-only availability diagnostic; no mastering or app integration.
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3, "job.json report.json");
    let job_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job: Value = serde_json::from_slice(&std::fs::read(job_path).unwrap()).unwrap();
    let mut report = json!({"status":"running", "job_sha256":sha(job_path), "rows":[]});
    for file in job["files"].as_array().unwrap() {
        let started = Instant::now();
        let path = Path::new(file["path"].as_str().unwrap());
        assert_eq!(sha(path), file["sha256"].as_str().unwrap());
        let mut reader = hound::WavReader::open(path).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, 48000);
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_format, hound::SampleFormat::Float);
        let samples: Vec<f32> = reader.samples::<f32>().map(Result::unwrap).collect();
        assert!(samples.iter().all(|v| v.is_finite()));
        assert_eq!(samples.len() as u64 / 2, file["frames"].as_u64().unwrap());
        let mut meter = EbuR128::new(2, 48000, Mode::I | Mode::M).unwrap();
        let mut momentary_max = f64::NEG_INFINITY;
        for chunk in samples.chunks(9600) {
            meter.add_frames_f32(chunk).unwrap();
            if chunk.len() == 9600 {
                momentary_max = momentary_max.max(meter.loudness_momentary().unwrap());
            }
        }
        let integrated = meter.loudness_global().unwrap();
        let state = if integrated.is_finite() {
            "finite"
        } else if integrated == f64::NEG_INFINITY {
            "unavailable"
        } else {
            "error"
        };
        let row = json!({"id":file["id"], "sha256":file["sha256"],
            "frames":samples.len()/2,"rate":48000,"channels":2,
            "integrated_state":state,"lufs":finite(integrated),
            "relative_threshold":finite(meter.relative_threshold().unwrap()),
            "gating_block_count_and_energy":meter.gating_block_count_and_energy(),
            "maximum_momentary_lufs":finite(momentary_max),"meter_s":started.elapsed().as_secs_f64()});
        println!("{} {} {}", row["id"], row["integrated_state"], row["lufs"]);
        report["rows"].as_array_mut().unwrap().push(row);
        std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    report["status"] = json!("complete");
    std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
