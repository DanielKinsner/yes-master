//! C1 offline orchestration. Included by the small native example entrypoint.
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{
    decode, dsp::MasteringChain, engine, output_protection, profile_store, types::*,
};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn lufs(x: &[f32], rate: u32, channels: u16) -> Option<f64> {
    let mut ebu = EbuR128::new(u32::from(channels), rate, Mode::I).unwrap();
    ebu.add_frames_f32(x).unwrap();
    let value = ebu.loudness_global().unwrap();
    value.is_finite().then_some(value)
}

fn write(path: &Path, x: &[f32], rate: u32, channels: u16) {
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
    for &sample in x {
        assert!(sample.is_finite());
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}

fn settings(analysis: &AnalysisResult) -> MasteringSettings {
    let mut settings: MasteringSettings = serde_json::from_value(json!({
        "preset":{"kind":"universal"}, "intensity":0.75,
        "eq_low_db":0.0,"eq_mid_db":0.0,"eq_high_db":0.0,
        "delivery_profile":"custom", "volume_match":false,
        "advanced":{"lufs_offset_db":-14.0,"ceiling_dbtp":-1.0,
            "target_sample_rate":48000,"bit_depth":32,"adaptive_strength":0.5}
    }))
    .unwrap();
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
    settings
}

/// Observe the actual limiter on every frame, including the drained tail. The
/// source-aligned history excludes warmup exactly as the delivered PCM does.
fn render(
    x: &[f32],
    rate: u32,
    channels: u16,
    settings: &MasteringSettings,
    drive: f32,
) -> (Vec<f32>, Value) {
    let start = Instant::now();
    let ch = usize::from(channels);
    let mut chain = MasteringChain::new(rate, ch, settings);
    chain.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
    let coefficients = format!("{:?}", chain.coeffs);
    let delay = chain.latency_frames();
    let mut samples = x.to_vec();
    let mut gains = Vec::with_capacity(x.len() / ch + delay);
    for frame in samples.chunks_exact_mut(ch) {
        chain.process_frame_inplace(frame);
        gains.push(chain.limiter.gain_reduction_db());
    }
    let mut tail = vec![0_f32; delay * ch];
    for frame in tail.chunks_exact_mut(ch) {
        chain.process_frame_inplace(frame);
        gains.push(chain.limiter.gain_reduction_db());
    }
    let length = samples.len();
    if length <= tail.len() {
        samples.copy_from_slice(&tail[tail.len() - length..]);
    } else {
        samples.copy_within(tail.len().., 0);
        samples[length - tail.len()..].copy_from_slice(&tail);
    }
    let gains = &mut gains[delay..];
    assert_eq!(gains.len(), x.len() / ch);
    let active = gains.iter().filter(|&&g| g > 0.01).count();
    gains.sort_unstable_by(f32::total_cmp);
    let percentile = |fraction: f64| gains[((gains.len() - 1) as f64 * fraction).round() as usize];
    let chain_s = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let converted =
        crate::sample_rate::convert_interleaved(&samples, rate, 48000, channels).unwrap();
    let src_s = start.elapsed().as_secs_f64();
    (
        converted,
        json!({"chain_and_observation_s":chain_s,"src_s":src_s,"coefficients":coefficients,
        "limiter":{"active_fraction_above_0_01db":active as f64/gains.len() as f64,
            "p50_reduction_db":percentile(0.5),"p95_reduction_db":percentile(0.95),
            "p99_reduction_db":percentile(0.99),"max_reduction_db":percentile(1.),"frames":gains.len()}}),
    )
}

pub fn run() {
    let session_start = Instant::now();
    let args: Vec<_> = std::env::args().collect();
    assert!(
        (3..=4).contains(&args.len()),
        "job.json fresh-output-directory [retained-report.json]"
    );
    let job_path = Path::new(&args[1]);
    let job: Value = serde_json::from_slice(&std::fs::read(job_path).unwrap()).unwrap();
    let source = Path::new(job["source"].as_str().unwrap());
    let output = Path::new(&args[2]);
    assert!(!output.exists(), "preserve completed evidence");
    assert_eq!(sha(source), job["source_sha256"].as_str().unwrap());
    std::fs::create_dir_all(output).unwrap();
    let start = Instant::now();
    let mut pcm = decode::decode_full(source).unwrap();
    assert!(!pcm.samples.is_empty() && pcm.samples.iter().all(|v| v.is_finite()));
    let decode_s = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let mut analyses = engine::analyze_tracks_core_with_progress_sync(
        vec![engine::AnalyzeRequest {
            id: TrackId("c1-source".into()),
            path: source.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )
    .unwrap();
    let analysis = analyses.remove(0);
    let analysis_s = start.elapsed().as_secs_f64();
    let base = settings(&analysis);
    let start = Instant::now();
    let source_gain = job["source_gain_db"].as_f64().unwrap_or(0.) as f32;
    assert!(
        matches!(source_gain, 0. | -40. | -80.),
        "frozen development gain copies only"
    );
    let copy_gain = 10_f32.powf(source_gain / 20.);
    for sample in &mut pcm.samples {
        *sample *= copy_gain;
    }
    let source_peak = pcm
        .samples
        .iter()
        .map(|x| f64::from(x.abs()))
        .fold(0_f64, f64::max);
    assert!(
        source_peak > 0.,
        "silence has no automatic operating descriptor"
    );
    let normalized: Vec<_> = pcm
        .samples
        .iter()
        .map(|&x| (f64::from(x) / source_peak) as f32)
        .collect();
    let normalized_lufs =
        lufs(&normalized, pcm.sample_rate, pcm.channels).expect("descriptor unavailable");
    let normalization_db = -20. * source_peak.log10();
    let descriptor_s = start.elapsed().as_secs_f64();
    let targets: Vec<f32> = serde_json::from_value(job["targets"].clone()).unwrap();
    let presets: Vec<Value> = serde_json::from_value(job["presets"].clone()).unwrap();
    let offsets: Vec<f32> = if job["candidate_offsets"].is_array() {
        serde_json::from_value(job["candidate_offsets"].clone()).unwrap()
    } else {
        vec![-9., -6., -3., 0., 3., 6., 9., 12.]
    };
    let refinement = job["phase"].as_str() == Some("refinement");
    assert!(!offsets.is_empty() && offsets.len() <= if refinement { 2 } else { 8 });
    assert!(offsets
        .iter()
        .all(|v| v.is_finite() && (-9. ..=12.).contains(v)));
    let mut report = json!({"version":"c1-development-v2","status":"running","source":source,
        "source_sha256":sha(source),"job_sha256":sha(job_path),"analysis":analysis,
        "source_gain_db":source_gain,"candidate_offsets":offsets,"refinement":refinement,
        "decode_s":decode_s,"analysis_s":analysis_s,"descriptor_s":descriptor_s,
        "descriptor":{"normalized_lufs":normalized_lufs,"normalization_db":normalization_db,
            "actual_source_lufs":lufs(&pcm.samples,pcm.sample_rate,pcm.channels)},
        "source_rate":pcm.sample_rate,"channels":pcm.channels,"delivery_rate":48000,
        "delivery_frames":((pcm.samples.len()/usize::from(pcm.channels)) as u128*48000).div_ceil(u128::from(pcm.sample_rate)),
        "peak_version":yes_master_lib::peak_meter::VERSION,
        "analysis_scope":"one fresh original-source analysis; unchanged guards, retained for every policy",
        "rows":[]});
    // Recovery writes a new report and new files only. Completed deliveries in
    // the interrupted run remain immutable, and each retained hash is checked.
    let mut retained = std::collections::HashMap::new();
    if let Some(previous_path) = args.get(3) {
        let previous_path = Path::new(previous_path);
        let previous: Value =
            serde_json::from_slice(&std::fs::read(previous_path).unwrap()).unwrap();
        assert_eq!(previous["version"], report["version"]);
        for key in [
            "job_sha256",
            "source_sha256",
            "source_rate",
            "channels",
            "delivery_rate",
            "delivery_frames",
            "descriptor",
            "peak_version",
        ] {
            assert_eq!(previous[key], report[key], "recovery mismatch: {key}");
        }
        for row in previous["rows"].as_array().unwrap() {
            let path = Path::new(row["path"].as_str().unwrap());
            assert_eq!(sha(path), row["sha256"].as_str().unwrap());
            assert!(retained
                .insert(row["id"].as_str().unwrap().to_owned(), row.clone())
                .is_none());
        }
        report["recovery"] = json!({"previous_report":previous_path,
            "previous_report_sha256":sha(previous_path),"retained_rows":retained.len(),
            "timing_scope":"interrupted run; full session elapsed unavailable; recovery measured separately"});
    }
    let mut rows = Vec::new();
    for preset in presets {
        let mut settings = base.clone();
        settings.preset = serde_json::from_value(preset["preset"].clone()).unwrap();
        settings.intensity = preset["intensity"].as_f64().unwrap() as f32;
        settings.input_gain_db = job["manual_input_db"].as_f64().unwrap_or(0.) as f32;
        let mut without_manual = settings.clone();
        without_manual.input_gain_db = 0.;
        let coeffs =
            MasteringChain::new(pcm.sample_rate, usize::from(pcm.channels), &without_manual).coeffs;
        let d = 1. + coeffs.saturation_amount * 2.;
        let saturation_gain = if coeffs.saturation_amount > 0. {
            20. * (d / d.tanh()).log10()
        } else {
            0.
        };
        let preset_push = 20. * coeffs.input_gain_lin.log10();
        let (control, control_metrics) =
            render(&pcm.samples, pcm.sample_rate, pcm.channels, &settings, 0.);
        let control_lufs = lufs(&control, 48000, pcm.channels);
        for (target_index, &target) in targets.iter().enumerate() {
            settings.advanced.lufs_offset_db = Some(target);
            let mut target_check = settings.clone();
            target_check.input_gain_db = 0.;
            assert_eq!(
                MasteringChain::new(pcm.sample_rate, usize::from(pcm.channels), &target_check)
                    .coeffs,
                coeffs,
                "target changes raw processing; the control cannot be reused"
            );
            let single = target - normalized_lufs as f32 - preset_push - saturation_gain;
            let mut choices = vec![
                ("control", 0_f32, 0_f32),
                (
                    "attenuate",
                    0.,
                    control_lufs.map_or(0., |v| (target - v as f32).min(0.)),
                ),
            ];
            for &offset in &offsets {
                choices.push((
                    if refinement {
                        "refinement"
                    } else if offset == 0. {
                        "single"
                    } else {
                        "bounded"
                    },
                    offset,
                    single + offset,
                ));
            }
            for (number, (policy, offset, drive)) in choices.into_iter().enumerate() {
                let id = format!(
                    "{}-t{}-{:02}-{policy}",
                    preset["id"].as_str().unwrap(),
                    -target as i32,
                    number
                );
                if let Some(row) = retained.remove(&id) {
                    // Restore typed f32 settings before comparing: a JSON
                    // number's parsed f64 can differ by one ULP from the f64
                    // representation used while serializing the original f32.
                    let old_settings: MasteringSettings =
                        serde_json::from_value(row["requested_settings"].clone()).unwrap();
                    assert_eq!(json!(old_settings), json!(settings));
                    assert_eq!(row["operating_drive_db"].as_f64().unwrap() as f32, drive);
                    assert_eq!(row["offset_db"].as_f64().unwrap() as f32, offset);
                    let mut restored =
                        MasteringChain::new(pcm.sample_rate, usize::from(pcm.channels), &settings);
                    restored.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
                    assert_eq!(
                        row["render"]["coefficients"].as_str().unwrap(),
                        format!("{:?}", restored.coeffs)
                    );
                    rows.push(row);
                    continue;
                }
                let (mut raw, metrics) = if policy == "control" {
                    (control.clone(), control_metrics.clone())
                } else {
                    render(
                        if policy == "attenuate" {
                            &pcm.samples
                        } else {
                            &normalized
                        },
                        pcm.sample_rate,
                        pcm.channels,
                        &settings,
                        drive,
                    )
                };
                let raw_lufs = lufs(&raw, 48000, pcm.channels);
                let start = Instant::now();
                let delivery = output_protection::finalize(
                    &mut raw,
                    48000,
                    pcm.channels,
                    32,
                    Some(target),
                    -1.,
                    None,
                )
                .unwrap();
                let verification_s = start.elapsed().as_secs_f64();
                assert!(delivery.true_peak_dbtp <= -1. + 1e-5);
                let path = output.join(format!("{id}.wav"));
                let start = Instant::now();
                write(&path, &raw, 48000, pcm.channels);
                let write_s = start.elapsed().as_secs_f64();
                rows.push(json!({"id":id,"preset_id":preset["id"],"policy":policy,"target":target,"ceiling":-1.,
                    "raw_response_reused":policy=="control" && target_index>0,
                    "attenuate_unavailable_reason":if policy=="attenuate" && control_lufs.is_none() {Some("original control is below the integrated loudness gate; no attenuation inferred")} else {None},
                    "offset_db":offset,"source_gain_db":source_gain,"normalization_db":if matches!(policy,"control"|"attenuate") {0.} else {normalization_db},
                    "operating_drive_db":drive,"preset_push_db":preset_push,"manual_input_db":settings.input_gain_db,
                    "saturation_small_signal_db":saturation_gain,"requested_settings":settings,"raw_lufs":raw_lufs,
                    "render":metrics,"verification_s":verification_s,"write_s":write_s,"path":path,"sha256":sha(&path),
                    "delivery":{"lufs":delivery.lufs.is_finite().then_some(delivery.lufs),"lra":delivery.lra,
                        "gain_lin":delivery.gain_lin,"true_peak_dbtp":delivery.true_peak_dbtp,
                        "interval_db":delivery.interval_db,"fresh_meter":delivery.used_fresh_meter}}));
                report["rows"] = json!(rows);
                std::fs::write(
                    output.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
                println!(
                    "Completed {id}: LUFS={} peak={} verification={verification_s:.3}s",
                    delivery.lufs, delivery.true_peak_dbtp
                );
            }
        }
    }
    assert!(
        retained.is_empty(),
        "unexpected retained cases outside this job"
    );
    report["rows"] = json!(rows);
    report["status"] = json!("complete");
    if report["recovery"].is_object() {
        report["recovery_session_s"] = json!(session_start.elapsed().as_secs_f64());
    } else {
        report["total_session_s"] = json!(session_start.elapsed().as_secs_f64());
    }
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
