//! Existing-analysis recipe replay and post-hoc LRA isolation. No audio output.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use yes_master_lib::{dsp::MasteringChain, types};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 5);
    let analysis_path = Path::new(&args[1]);
    let headroom_path = Path::new(&args[2]);
    let job_path = Path::new(&args[3]);
    let output = Path::new(&args[4]);
    assert!(!output.exists());
    let analysis = read(analysis_path);
    let headroom = read(headroom_path);
    let job = read(job_path);
    assert_eq!(analysis["status"], "complete");
    assert_eq!(headroom["status"], "complete");
    assert_eq!(analysis["job_sha256"], sha(job_path));
    let rows = analysis["rows"].as_array().unwrap();
    let mut replay = Vec::new();
    let mut isolation = Vec::new();
    for row in rows {
        let rate = row["rate"].as_u64().unwrap() as u32;
        if row["domain"] == "normalized" {
            let prior = headroom["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["case"] == row["case"] && p["source_gain_db"] == row["source_gain_db"])
                .unwrap();
            let case = job["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == row["case"])
                .unwrap();
            let reference = case["references"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["source_gain_db"] == row["source_gain_db"])
                .unwrap();
            assert_eq!(row["pcm_sha256"], reference["input_pcm_sha256"]);
            assert_eq!(prior["reference_sha256"], reference["sha256"]);
            let mut settings: types::MasteringSettings =
                serde_json::from_value(prior["requested_settings"].clone()).unwrap();
            settings.advanced.source_profile =
                serde_json::from_value(row["profile"].clone()).unwrap();
            settings.source_lufs_integrated =
                Some(row["analysis"]["lufs_integrated"].as_f64().unwrap() as f32);
            let mut chain = MasteringChain::new(rate, 2, &settings);
            chain.coeffs.input_gain_lin *=
                10_f32.powf(prior["drive_db"].as_f64().unwrap() as f32 / 20.);
            let coefficients = format!("{:?}", chain.coeffs);
            replay.push(json!({"analysis_id":row["id"],"master_id":prior["id"],"master_sha256":prior["sha256"],
                "normalized_pcm_sha256":row["pcm_sha256"],"coefficients":coefficients,
                "exact_coefficient_match":coefficients == prior["render"]["base_coefficients"].as_str().unwrap()}));
        } else if row["source_gain_db"].as_f64().unwrap() != 0. {
            let prior = rows
                .iter()
                .find(|p| {
                    p["case"] == row["case"]
                        && p["domain"] == "actual"
                        && p["source_gain_db"].as_f64().unwrap() == 0.
                })
                .unwrap();
            let mut settings: types::MasteringSettings =
                serde_json::from_value(row["resolved_settings"].clone()).unwrap();
            let original_lra = prior["profile"]["dynamic_range_lu"].as_f64().unwrap() as f32;
            settings
                .advanced
                .source_profile
                .as_mut()
                .unwrap()
                .dynamic_range_lu = original_lra;
            let coefficients = format!("{:?}", MasteringChain::new(rate, 2, &settings).coeffs);
            isolation.push(json!({"analysis_id":row["id"],"original_lra":original_lra,"actual_lra":row["profile"]["dynamic_range_lu"],
                "observed_coefficients_changed":row["coefficients"] != prior["coefficients"],"substituted_coefficients":coefficients,
                "exact_original_match_after_lra_restore":coefficients == prior["coefficients"].as_str().unwrap()}));
        }
    }
    assert_eq!(replay.len(), 12);
    assert_eq!(isolation.len(), 8);
    let result = json!({"status":"complete","analysis_sha256":sha(analysis_path),"headroom_sha256":sha(headroom_path),
        "job_sha256":sha(job_path),"replay":replay,"lra_isolation":isolation,
        "scope":"Recipe identity and post-hoc LRA isolation only; no new audio, analysis or shipping policy."});
    std::fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!("Recorded twelve recipes and eight LRA substitutions");
}
