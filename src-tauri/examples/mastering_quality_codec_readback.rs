//! Remeasure retained MP3 bytes after a decoder correction, preserving all old evidence.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{mp3, peak_meter};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let old = Path::new(&args[1]);
    let mut report: Value = serde_json::from_slice(&std::fs::read(old).unwrap()).unwrap();
    assert_eq!(report["status"], "complete");
    let out = Path::new(&args[2]);
    assert!(!out.exists());
    std::fs::create_dir(out).unwrap();
    report["prior_report_sha256"] = json!(sha(old));
    report["mp3_decoder"] = json!(mp3::READBACK_DECODER);
    report["estimator"] = json!(peak_meter::VERSION);
    report["status"] = json!("in_progress");
    let count = report["rows"].as_array().unwrap().len();
    for i in 0..count {
        let row = &mut report["rows"][i];
        assert!(row.get("error").is_none());
        if row["encoding"]["format"] != "mp3" {
            continue;
        }
        let path = Path::new(row["path"].as_str().unwrap());
        assert_eq!(row["sha256"].as_str().unwrap(), sha(path));
        let start = Instant::now();
        let result = mp3::measure_details(path, None).unwrap();
        assert_eq!(result.frames, row["frames"].as_u64().unwrap());
        assert_eq!(u64::from(result.sample_rate), row["rate"].as_u64().unwrap());
        assert_eq!(
            u64::from(result.channels),
            row["channels"].as_u64().unwrap()
        );
        row["remeasure_s"] = json!(start.elapsed().as_secs_f64());
        row["prior_decoded_peak"] = row["decoded_peak"].clone();
        row["decoded_lufs"] = json!(result.measurements.0);
        row["decoded_peak"] = json!(result.measurements.1);
        row["decoded_lra"] = json!(result.measurements.2);
        row["growth_db"] =
            json!(f64::from(result.measurements.1) - row["input_peak"].as_f64().unwrap());
        row["ceiling_miss"] = json!(result.measurements.1 > -1. + 1e-5);
        row["target_miss_lu"] = json!(result.measurements.0 + 3.);
        std::fs::write(
            out.join("report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    report["status"] = json!("complete");
    std::fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
