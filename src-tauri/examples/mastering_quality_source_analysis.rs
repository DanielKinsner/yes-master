//! Analysis-only C1 gain-copy diagnostic; never changes production policy.
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{decode, dsp::ChainCoeffs, engine, profile_store, types::*};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn write(path: &Path, samples: &[f32], rate: u32, channels: u16) {
    assert!(!path.exists());
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate: rate,
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

fn resolved(analysis: &AnalysisResult, rate: u32, template: &MasteringSettings) -> Value {
    let mut settings = template.clone();
    settings.advanced.source_profile = SourceProfile::from_analysis(analysis);
    profile_store::apply_resolved_confidence(&mut settings, analysis.deep_analysis.clone(), false);
    let psr = analysis
        .deep_analysis
        .as_deref()
        .and_then(yes_master_lib::deep_analysis::band_psr_p10_db);
    let guard = yes_master_lib::guardrails::classify_already_mastered_stand_down(
        analysis.lufs_integrated,
        analysis.true_peak_dbtp,
        analysis.dynamic_range_lu,
        psr,
    );
    profile_store::apply_resolved_compression_guards(
        &mut settings,
        analysis.deep_analysis.clone(),
        Some(guard),
        false,
    );
    settings.source_lufs_integrated = Some(analysis.lufs_integrated);
    let mut rows = Vec::new();
    for (preset, intensity) in [("universal", 0.75), ("universal", 0.5), ("loud", 0.75)] {
        settings.preset = serde_json::from_value(json!({"kind":preset})).unwrap();
        settings.intensity = intensity;
        rows.push(json!({"preset":preset,"intensity":intensity,
            "coefficients":format!("{:?}", ChainCoeffs::from_settings(rate, &settings))}));
    }
    json!({"profile":SourceProfile::from_analysis(analysis),"stand_down_diagnostic":{
        "stand_down":guard.stand_down,"hot_loudness":guard.hot_loudness,
        "near_ceiling":guard.near_ceiling,"low_lra":guard.low_lra,
        "uniformly_low_psr":guard.uniformly_low_psr},"rows":rows})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "manifest.json specification.md fresh-output-dir"
    );
    let manifest = Path::new(&args[1]);
    let specification = Path::new(&args[2]);
    let output = Path::new(&args[3]);
    assert!(!output.exists());
    let entries: Vec<Value> = serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    std::fs::create_dir_all(output).unwrap();
    let mut report = json!({"status":"running","manifest_sha256":sha(manifest),
        "specification_sha256":sha(specification),"rows":[],
        "scope":"Actual complete production analysis of original/gain copies and normalized descriptors; no DSP render or analysis-policy adoption; concurrent wall times"});
    for entry in entries {
        let path = Path::new(entry["source"].as_str().unwrap());
        assert_eq!(sha(path), entry["sha256"].as_str().unwrap());
        let template: MasteringSettings =
            serde_json::from_value(entry["settings"].clone()).unwrap();
        let start = Instant::now();
        let pcm = decode::decode_full(path).unwrap();
        let decode_s = start.elapsed().as_secs_f64();
        for gain_db in [0., -40., -80.] {
            for normalized in [false, true] {
                let start = Instant::now();
                let gain = 10_f32.powf(gain_db / 20.);
                let mut samples: Vec<_> = pcm.samples.iter().map(|x| x * gain).collect();
                let peak = samples
                    .iter()
                    .map(|x| f64::from(x.abs()))
                    .fold(0_f64, f64::max);
                assert!(peak.is_finite() && peak > 0.);
                if normalized {
                    for sample in &mut samples {
                        *sample = (f64::from(*sample) / peak) as f32;
                    }
                }
                let id = format!(
                    "{}-{}-{}",
                    entry["id"].as_str().unwrap(),
                    -gain_db as i32,
                    if normalized { "normalized" } else { "actual" }
                );
                let transformed = output.join(format!("{id}.wav"));
                // The original is already immutable and hashed. Only write new transforms.
                let input = if gain_db == 0. && !normalized {
                    path
                } else {
                    write(&transformed, &samples, pcm.sample_rate, pcm.channels);
                    &transformed
                };
                let transform_write_s = start.elapsed().as_secs_f64();
                let mut meter =
                    EbuR128::new(u32::from(pcm.channels), pcm.sample_rate, Mode::I).unwrap();
                meter.add_frames_f32(&samples).unwrap();
                let lufs = meter.loudness_global().unwrap();
                let measured_lufs = lufs.is_finite().then_some(lufs);
                drop(samples);
                let start = Instant::now();
                let mut results = engine::analyze_tracks_core_with_progress_sync(
                    vec![engine::AnalyzeRequest {
                        id: TrackId(id.clone()),
                        path: input.to_string_lossy().into_owned(),
                    }],
                    |_, _| {},
                )
                .unwrap();
                assert_eq!(results.len(), 1);
                let analysis = results.remove(0);
                let analysis_s = start.elapsed().as_secs_f64();
                report["rows"].as_array_mut().unwrap().push(
                    json!({"id":id,"source_id":entry["id"],
                    "source_sha256":entry["sha256"],"path":input,"sha256":sha(input),
                    "source_gain_db":gain_db,"normalized":normalized,"actual_copy_peak":peak,
                    "measured_input_lufs":measured_lufs,"decode_s":decode_s,
                    "transform_write_s":transform_write_s,"analysis_s":analysis_s,
                    "resolved":resolved(&analysis,pcm.sample_rate,&template),"analysis":analysis}),
                );
                std::fs::write(
                    output.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
                println!("Analyzed {id}");
            }
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
