//! Research-only immutable prepared-PCM reuse; not an application cache.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};
use yes_master_lib::{decode, output_protection};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let job_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job = read(job_path);
    assert_eq!(job["experiment"], "prepared-target-reuse-v1");
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
    let mut report = json!({"status":"running","experiment":job["experiment"],
        "job_sha256":sha(job_path),"rows":[]});
    for case in job["cases"].as_array().unwrap() {
        let path = Path::new(case["prepared_path"].as_str().unwrap());
        assert_eq!(sha(path), case["prepared_sha256"]);
        let start = Instant::now();
        let decoded = decode::decode_full(path).unwrap();
        assert_eq!(decoded.sample_rate, 48000);
        assert_eq!(decoded.channels, 2);
        assert_eq!(
            decoded.samples.len() / 2,
            case["frames"].as_u64().unwrap() as usize
        );
        let prepared = Arc::new(decoded.samples);
        let decode_s = start.elapsed().as_secs_f64();
        // The map lives inside the bound preparation, so its source/algorithm,
        // rate/channels/format/ceiling cannot change underneath a target key.
        let mut cache: HashMap<u32, Arc<Vec<f32>>> = HashMap::new();
        cache.insert((-14_f32).to_bits(), Arc::clone(&prepared));
        let start = Instant::now();
        let mut changed = prepared.as_ref().clone();
        let protected =
            output_protection::finalize(&mut changed, 48000, 2, 32, Some(-9.), -1., None).unwrap();
        let new_target_s = start.elapsed().as_secs_f64();
        assert!(protected.true_peak_dbtp <= -1.);
        assert!(changed.iter().all(|sample| sample.is_finite()));
        let peak = prepared
            .iter()
            .map(|sample| sample.abs())
            .fold(0_f32, f32::max);
        let error = prepared
            .iter()
            .zip(&changed)
            .map(|(a, b)| (f64::from(*a) * f64::from(protected.gain_lin) - f64::from(*b)).abs())
            .fold(0_f64, f64::max);
        let normalized_scalar_error = error / (f64::from(peak) * f64::from(protected.gain_lin));
        assert!(normalized_scalar_error <= 1e-6);
        let changed = Arc::new(changed);
        cache.insert((-9_f32).to_bits(), Arc::clone(&changed));
        let start = Instant::now();
        let old_hit = Arc::clone(cache.get(&(-14_f32).to_bits()).unwrap());
        let prior_target_lookup_s = start.elapsed().as_secs_f64();
        assert!(Arc::ptr_eq(&old_hit, &prepared));
        let start = Instant::now();
        let new_hit = Arc::clone(cache.get(&(-9_f32).to_bits()).unwrap());
        let repeat_target_lookup_s = start.elapsed().as_secs_f64();
        assert!(Arc::ptr_eq(&new_hit, &changed));
        let path = output.join(format!("{}-reuse-t9.wav", case["id"].as_str().unwrap()));
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
        for &sample in changed.iter() {
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
        report["rows"].as_array_mut().unwrap().push(json!({"id":case["id"],"case":case["case"],
            "preset_id":case["preset_id"],"path":path,"sha256":sha(&path),"prepared_sha256":case["prepared_sha256"],
            "prepared_id":case["prepared_id"],"source_sha256":case["source_sha256"],"frames":changed.len()/2,
            "rate":48000,"channels":2,"lufs":protected.lufs,"peak":protected.true_peak_dbtp,"ceiling":-1.,
            "target":-9.,"gain":protected.gain_lin,"normalized_scalar_error":normalized_scalar_error,
            "decode_s":decode_s,"new_target_s":new_target_s,"prior_target_lookup_s":prior_target_lookup_s,
            "repeat_target_lookup_s":repeat_target_lookup_s,"cache_arc_identity_pass":true,"finalizer_calls":1}));
        std::fs::write(
            output.join("report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("Completed {}", case["id"]);
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
