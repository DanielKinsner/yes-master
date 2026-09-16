//! Whole-file downstream device conversion of retained, protected file PCM.
//! No native stream is opened and no production gain policy is changed.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{decode, engine, peak_meter};
#[path = "../src/quality_source.rs"]
#[allow(dead_code)]
mod quality_source;

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "input manifest JSON, fresh output directory");
    let entries: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    std::fs::create_dir_all(output).unwrap();
    let mut report = json!({"status":"running","manifest_sha256":sha(Path::new(&args[1])),
        "estimator":peak_meter::VERSION,"rows":[],
        "scope":"actual streaming converter, full protected file PCM to simulated device rates; no native device opening, no scalar reordering or production cap claim"});
    for entry in entries {
        let path = Path::new(entry["path"].as_str().unwrap());
        assert_eq!(sha(path), entry["sha256"].as_str().unwrap());
        let pcm = decode::decode_full(path).unwrap();
        let original_lufs =
            engine::measure_integrated_lufs(&pcm.samples, pcm.sample_rate, pcm.channels).unwrap();
        for rate in [44100_u32, 96000] {
            let start = Instant::now();
            let source = rodio::buffer::SamplesBuffer::new(
                pcm.channels,
                pcm.sample_rate,
                pcm.samples.clone(),
            );
            let source = quality_source::QualitySource::new(source, rate).unwrap();
            let failure = source.error_slot();
            let samples: Vec<f32> = source.collect();
            assert_eq!(failure.load(std::sync::atomic::Ordering::Acquire), 0);
            let src_s = start.elapsed().as_secs_f64();
            let frames = samples.len() / usize::from(pcm.channels);
            let expected = (pcm.samples.len() / usize::from(pcm.channels) * rate as usize)
                .div_ceil(pcm.sample_rate as usize);
            assert_eq!(frames, expected);
            let start = Instant::now();
            let peak = peak_meter::measure(&samples, usize::from(pcm.channels), || false).unwrap();
            let meter_s = start.elapsed().as_secs_f64();
            let lufs = engine::measure_integrated_lufs(&samples, rate, pcm.channels).unwrap();
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
            for &sample in &samples {
                assert!(sample.is_finite());
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap();
            let record = json!({"id":format!("{}-{rate}",entry["id"].as_str().unwrap()),"path":delivered_path,
                "sha256":sha(&delivered_path),"input_sha256":entry["sha256"],"source_rate":pcm.sample_rate,
                "device_rate":rate,"channels":pcm.channels,"frames":frames,"ceiling":entry["ceiling"],
                "peak":peak,"lufs":lufs,"lufs_delta":lufs-original_lufs,"src_s":src_s,"meter_s":meter_s,
                "fullscale_samples":samples.iter().filter(|x| x.abs()>=1.).count()});
            println!("Converted {} to {rate}", entry["id"]);
            report["rows"].as_array_mut().unwrap().push(record);
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
