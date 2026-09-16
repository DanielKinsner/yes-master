//! Full-file device protection/reuse experiment, no audio device opened.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{decode, device_preparation::PreparedDevicePcm, peak_meter};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "input manifest, fresh output directory");
    let manifest = Path::new(&args[1]);
    let entries: Vec<Value> = serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    std::fs::create_dir_all(output).unwrap();
    let mut report = json!({"status":"running","manifest_sha256":sha(manifest),
        "estimator":peak_meter::VERSION,"rows":[],
        "scope":"whole-file post-conversion device gain/reuse core; retained file PCM, no production route integration or native output claim"});
    for entry in entries {
        let path = Path::new(entry["path"].as_str().unwrap());
        assert_eq!(sha(path), entry["sha256"].as_str().unwrap());
        let pcm = decode::decode_full(path).unwrap();
        for rate in [44100, 96000] {
            let start = Instant::now();
            let prepared =
                PreparedDevicePcm::new(&pcm.samples, pcm.sample_rate, pcm.channels, rate, None)
                    .unwrap();
            let prepare_s = start.elapsed().as_secs_f64();
            let ceiling = entry["ceiling"].as_f64().unwrap() as f32;
            let start = Instant::now();
            let delivery = prepared.finish(1., ceiling, None).unwrap();
            let verify_s = start.elapsed().as_secs_f64();
            let delivered_path =
                output.join(format!("{}-{rate}.wav", entry["id"].as_str().unwrap()));
            let mut writer = hound::WavWriter::create(
                &delivered_path,
                hound::WavSpec {
                    channels: pcm.channels,
                    sample_rate: rate,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .unwrap();
            for &sample in &delivery.samples {
                assert!(sample.is_finite());
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap();
            let mut edits = Vec::new();
            for desired_gain in [0.7079458, 0.3548134, 1.] {
                let start = Instant::now();
                let changed = prepared.finish(desired_gain, ceiling, None).unwrap();
                edits.push(json!({"desired_gain":desired_gain,"gain":changed.protection.gain_lin,
                    "whole_file_verify_s":start.elapsed().as_secs_f64(),"lufs":changed.protection.lufs,
                    "peak":changed.protection.true_peak_dbtp,"fresh_meter":changed.protection.used_fresh_meter}));
                if desired_gain == 1. {
                    assert_eq!(changed.samples, delivery.samples);
                }
            }
            let row = json!({"id":format!("{}-{rate}",entry["id"].as_str().unwrap()), "path":delivered_path,
                "sha256":sha(&delivered_path),"input_sha256":entry["sha256"],"source_rate":pcm.sample_rate,
                "device_rate":rate,"channels":pcm.channels,"frames":delivery.samples.len()/usize::from(pcm.channels),
                "ceiling":ceiling,"peak":delivery.protection.true_peak_dbtp,"lufs":delivery.protection.lufs,
                "gain":delivery.protection.gain_lin,"interval_db":delivery.protection.interval_db,
                "prepare_s":prepare_s,"verify_s":verify_s,"pcm_bytes":prepared.pcm_bytes(),"edits":edits,
                "fullscale_samples":delivery.samples.iter().filter(|x| x.abs()>=1.).count()});
            report["rows"].as_array_mut().unwrap().push(row);
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!("Protected {} at {rate}", entry["id"]);
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
