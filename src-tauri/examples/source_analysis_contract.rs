//! Fresh production analysis of actual and normalized gain-copy sources.
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{decode, deep_analysis, dsp, files, types};
#[allow(dead_code)]
#[path = "../src/analysis.rs"]
mod production_analysis;

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn pcm_sha(samples: &[f32]) -> String {
    let mut hasher = Sha256::new();
    for v in samples {
        hasher.update(v.to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}
fn write_wav(path: &Path, samples: &[f32], rate: u32) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for v in samples {
        writer.write_sample(*v).unwrap();
    }
    writer.finalize().unwrap();
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let job_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    assert!(!output.exists());
    let job: Value = serde_json::from_slice(&std::fs::read(job_path).unwrap()).unwrap();
    assert_eq!(job["experiment"], "source-analysis-contract-v1");
    assert_eq!(
        sha(Path::new(job["analysis_source"].as_str().unwrap())),
        job["analysis_source_sha256"]
    );
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"]
    );
    std::fs::create_dir_all(output).unwrap();
    let mut report =
        json!({"status":"running","job_sha256":sha(job_path),"rows":[],"preparation":[]});
    for case in job["cases"].as_array().unwrap() {
        let started = Instant::now();
        let source = Path::new(case["source"].as_str().unwrap());
        assert_eq!(sha(source), case["source_sha256"]);
        let pcm = decode::decode_full(source).unwrap();
        assert_eq!(pcm.channels, 2);
        let decode_s = started.elapsed().as_secs_f64();
        let mut preparation = json!({"case":case["id"],"decode_s":decode_s,"inputs":[]});
        for reference in case["references"].as_array().unwrap() {
            let copy_db = reference["source_gain_db"].as_f64().unwrap();
            for normalized in [false, true] {
                let started = Instant::now();
                let gain = 10_f64.powf(copy_db / 20.);
                let mut samples: Vec<f32> = pcm
                    .samples
                    .iter()
                    .map(|v| (f64::from(*v) * gain) as f32)
                    .collect();
                if normalized {
                    let peak = samples
                        .iter()
                        .map(|v| f64::from(v.abs()))
                        .fold(0., f64::max);
                    assert!(peak > 0.);
                    for v in &mut samples {
                        *v = (f64::from(*v) / peak) as f32;
                    }
                }
                let domain = if normalized { "normalized" } else { "actual" };
                let id = format!("{}-{domain}-copy{copy_db}", case["id"].as_str().unwrap());
                let hash = pcm_sha(&samples);
                if normalized {
                    assert_eq!(hash, reference["input_pcm_sha256"]);
                }
                let path = if copy_db == 0. && !normalized {
                    source.to_path_buf()
                } else {
                    let path = output.join(format!("{id}.wav"));
                    write_wav(&path, &samples, pcm.sample_rate);
                    path
                };
                let mut ebu = EbuR128::new(2, pcm.sample_rate, Mode::I).unwrap();
                ebu.add_frames_f32(&samples).unwrap();
                let native_lufs = ebu.loudness_global().unwrap();
                let input_s = started.elapsed().as_secs_f64();
                drop(samples);
                let started = Instant::now();
                let analysis =
                    production_analysis::analyze_one(types::TrackId(id.clone()), &path, true)
                        .unwrap();
                let analysis_s = started.elapsed().as_secs_f64();
                let profile = types::SourceProfile::from_analysis(&analysis);
                let mut settings: types::MasteringSettings =
                    serde_json::from_value(case["settings"].clone()).unwrap();
                settings.advanced.source_profile = profile;
                settings.source_lufs_integrated = Some(analysis.lufs_integrated);
                let coeffs = dsp::ChainCoeffs::from_settings(pcm.sample_rate, &settings);
                let deep = analysis.deep_analysis.as_ref().map(|d| {
                    json!({
                    "windows":d.windows.len(),"bands_31":d.bands_31.to_vec()})
                });
                report["rows"].as_array_mut().unwrap().push(json!({
                    "id":id,"case":case["id"],"source_gain_db":copy_db,"domain":domain,
                    "path":path,"sha256":sha(&path),"pcm_sha256":hash,"frames":pcm.samples.len()/2,"rate":pcm.sample_rate,
                    "native_lufs":native_lufs.is_finite().then_some(native_lufs),
                    "native_availability":if native_lufs.is_finite() {"finite"} else {"unavailable"},
                    "analysis":analysis,"deep":deep,"profile":profile,"resolved_settings":settings,
                    "coefficients":format!("{coeffs:?}"),"analysis_s":analysis_s}));
                preparation["inputs"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"id":id,"input_s":input_s}));
                std::fs::write(
                    output.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
                println!("Completed {id}");
            }
        }
        report["preparation"]
            .as_array_mut()
            .unwrap()
            .push(preparation);
    }
    assert_eq!(report["rows"].as_array().unwrap().len(), 24);
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
