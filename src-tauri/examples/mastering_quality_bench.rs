//! Local C0 preparation experiment. Whole-file outputs, no shipping policy.
//! cargo run --example mastering_quality_bench -- <source> <fresh-output-dir>
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, path::Path, time::Instant};
use yes_master_lib::{decode, dsp::MasteringChain, engine, profile_store, types::*};

// Exercise the production SRC module without widening its application API.
use yes_master_lib::types;
#[path = "../src/sample_rate.rs"]
mod sample_rate;

fn seconds(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}
fn db(x: f64) -> f64 {
    20.0 * x.max(1e-30).log10()
}
fn measure(x: &[f32], sr: u32, ch: u16) -> (f32, f32) {
    let mut ebu = EbuR128::new(u32::from(ch), sr, Mode::I | Mode::TRUE_PEAK).unwrap();
    ebu.add_frames_f32(x).unwrap();
    let mut peak = x.iter().map(|x| f64::from(x.abs())).fold(0.0, f64::max);
    for c in 0..u32::from(ch) {
        peak = peak.max(ebu.true_peak(c).unwrap());
    }
    (ebu.loudness_global().unwrap() as f32, db(peak) as f32)
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
    for &v in x {
        assert!(v.is_finite());
        writer.write_sample(v).unwrap();
    }
    writer.finalize().unwrap();
}
fn settings(a: &AnalysisResult) -> MasteringSettings {
    let mut s: MasteringSettings = serde_json::from_value(json!({
        "preset":{"kind":"universal"}, "intensity":0.75,
        "eq_low_db":0.0,"eq_mid_db":0.0,"eq_high_db":0.0,
        "delivery_profile":"custom", "volume_match":false,
        "advanced":{"lufs_offset_db":-9.0,"ceiling_dbtp":-1.0,
            "target_sample_rate":48000,"bit_depth":24,"adaptive_strength":0.5}
    }))
    .unwrap();
    s.advanced.source_profile = SourceProfile::from_analysis(a);
    profile_store::apply_resolved_confidence(&mut s, a.deep_analysis.clone(), false);
    let psr = a
        .deep_analysis
        .as_deref()
        .and_then(yes_master_lib::deep_analysis::band_psr_p10_db);
    let stand_down = yes_master_lib::guardrails::classify_already_mastered_stand_down(
        a.lufs_integrated,
        a.true_peak_dbtp,
        a.dynamic_range_lu,
        psr,
    );
    profile_store::apply_resolved_compression_guards(
        &mut s,
        a.deep_analysis.clone(),
        Some(stand_down),
        false,
    );
    s.source_lufs_integrated = Some(a.lufs_integrated);
    s
}
fn render(
    pcm: &decode::DecodedPcm,
    settings: &MasteringSettings,
    extra_drive: f32,
) -> (Vec<f32>, Value) {
    let start = Instant::now();
    let ch = usize::from(pcm.channels);
    let mut x = pcm.samples.clone();
    let mut chain = MasteringChain::new(pcm.sample_rate, ch, settings);
    // Separate experimental automatic gain from the saved/manual input trim.
    chain.coeffs.input_gain_lin *= 10.0_f32.powf(extra_drive / 20.0);
    let coeffs = format!("{:?}", chain.coeffs);
    for block in x.chunks_mut(8192 * ch) {
        chain.process_interleaved(block, ch);
    }
    chain.flush_render_tail(&mut x, ch);
    let chain_s = seconds(start);
    let start = Instant::now();
    let x = sample_rate::convert_interleaved(&x, pcm.sample_rate, 48000, pcm.channels).unwrap();
    let src_s = seconds(start);
    let start = Instant::now();
    let (lufs, peak) = measure(&x, 48000, pcm.channels);
    (
        x,
        json!({"chain_s":chain_s,"src_s":src_s,"meter_s":seconds(start),
        "raw_lufs":lufs,"provisional_peak":peak,"coefficients":coeffs,"extra_drive_db":extra_drive}),
    )
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let timing_only = args.iter().any(|a| a == "--timing-only");
    let candidates_only = args.iter().any(|a| a == "--candidates-only");
    assert!(!(timing_only && candidates_only));
    let trials: usize = args
        .windows(2)
        .find(|a| a[0] == "--trials")
        .map(|a| a[1].parse().unwrap())
        .unwrap_or(3);
    assert!(
        args.len() >= 3,
        "source and fresh output directory required"
    );
    let path = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    assert!(
        !out.exists(),
        "preserve completed evidence: choose a new directory"
    );
    std::fs::create_dir_all(out).unwrap();
    let start = Instant::now();
    let source_hash = format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()));
    let hash_s = seconds(start);
    let start = Instant::now();
    let pcm = decode::decode_full(path).unwrap();
    let decode_s = seconds(start);
    let start = Instant::now();
    let mut analyses = engine::analyze_tracks_core_with_progress_sync(
        vec![engine::AnalyzeRequest {
            id: TrackId("c0-source".into()),
            path: path.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )
    .unwrap();
    let analysis_s = seconds(start);
    let analysis = analyses.remove(0);
    let base = settings(&analysis);
    let start = Instant::now();
    let deep = analysis.deep_analysis.as_deref().unwrap();
    let frames = pcm.samples.len() / usize::from(pcm.channels);
    let window = yes_master_lib::deep_analysis::SHORT_WINDOW;
    let normal_hop = yes_master_lib::deep_analysis::SHORT_HOP;
    let cap = yes_master_lib::deep_analysis::MAX_SCAN_WINDOWS;
    let hop = if frames.saturating_sub(window) / normal_hop + 1 > cap {
        ((frames - window) / (cap - 1)).max(1)
    } else {
        normal_hop
    };
    let passage = |score: fn(&yes_master_lib::deep_analysis::WindowMetrics) -> f32, high: bool| {
        deep.windows
            .iter()
            .enumerate()
            // Descriptive active passage threshold only, never peak coverage.
            .filter(|(_, w)| w.loudness_key.is_finite() && w.loudness_key > -40.0)
            .filter(|(_, w)| score(w).is_finite())
            .max_by(|(_, a), (_, b)| {
                let cmp = score(a).total_cmp(&score(b));
                if high {
                    cmp
                } else {
                    cmp.reverse()
                }
            })
            .map(|(i, _)| i * hop)
    };
    let passages = json!({"loud":passage(|w|w.loudness_key,true),
        "quiet_active":passage(|w|w.loudness_key,false),"attack":passage(|w|w.crest,true),
        "high_frequency":passage(|w|w.harsh_31+w.sibilant_31,true),"start":0,"tail":frames});
    let preparation_s = seconds(start);
    let mut report = json!({"version":"c0-v2","source_sha256":source_hash,
        "source_rate":pcm.sample_rate,"channels":pcm.channels,"frames":frames,
        "pcm_bytes":pcm.samples.len()*4,"hash_s":hash_s,"decode_s":decode_s,"analysis_s":analysis_s,
        "passage_preparation_s":preparation_s,"passages":passages,
        "deep_windows":deep.windows.len(),"deep_bytes":deep.windows.len()*std::mem::size_of::<yes_master_lib::deep_analysis::WindowMetrics>(),
        "source_lufs":analysis.lufs_integrated,"settings":base,
        "timing_peak_scope":"existing estimator; independent FFT/SOXR postchecks required"});
    std::fs::write(
        out.join("preparation.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    // Real application exports preserve On/Off controls and exercise A/B1.
    let mut witnesses = Vec::new();
    for (name, mode) in if candidates_only {
        Vec::new()
    } else {
        vec![
            ("on", CompressionMode::Preset),
            ("off", CompressionMode::Off),
        ]
    } {
        let mut s = base.clone();
        s.advanced.compression_mode = mode;
        let start = Instant::now();
        let job = engine::mastering_render_to_path(
            TrackId(name.into()),
            path,
            &s,
            out,
            RenderKind::Master,
            &out.join(format!("{name}.wav")),
        )
        .unwrap();
        witnesses.push(json!({"name":name,"elapsed_s":seconds(start),"job":job,
            "sha256":format!("{:x}",Sha256::digest(std::fs::read(out.join(format!("{name}.wav"))).unwrap()))}));
    }
    report["production_witnesses"] = json!(witnesses);
    let mut edits = Vec::new();
    edits.push(("initial", base.clone()));
    let mut s = base.clone();
    s.advanced.lufs_offset_db = Some(-14.0);
    edits.push(("target", s));
    let mut s = base.clone();
    s.intensity = 0.5;
    edits.push(("intensity", s));
    let mut s = base.clone();
    s.eq_high_db = 1.0;
    edits.push(("eq", s));
    let mut s = base.clone();
    s.advanced.compression_density = Some(0.2);
    edits.push(("density", s));
    let mut s = base.clone();
    s.advanced.adaptive_strength = Some(1.0);
    edits.push(("adapt", s));
    let mut s = base.clone();
    s.preset = Preset::Loud;
    edits.push(("preset", s));
    edits.push(("return_initial", base.clone()));
    let mut s = base.clone();
    s.advanced.lufs_offset_db = Some(-14.0);
    edits.push(("return_target", s));
    edits.push(("replay", base.clone()));
    let mut sessions = Vec::new();
    for trial in 0..if candidates_only { 0 } else { trials } {
        // First report the actual preview API separately: it does not perform
        // the extra final-file verification imposed on all simulation strategies.
        let mut actual = std::collections::HashMap::new();
        let mut actual_rows = Vec::new();
        let start_session = Instant::now();
        for (name, settings) in &edits {
            let key = serde_json::to_string(settings).unwrap();
            let start = Instant::now();
            let hit = actual.contains_key(&key);
            let landed = actual.entry(key).or_insert_with(|| {
                engine::preview_landing(&pcm.samples, pcm.sample_rate, pcm.channels, settings)
                    .unwrap()
            });
            actual_rows.push(
                json!({"edit":name,"elapsed_s":seconds(start),"scalar_hit":hit,
                "gain":landed.gain_lin,"predicted_lufs":landed.mastered_lufs}),
            );
        }
        sessions.push(
            json!({"trial":trial,"strategy":"actual_preview_api_with_existing_scalar_reuse",
            "total_session_s":seconds(start_session),"edits":actual_rows}),
        );
        let strategies = [
            "existing_scalar_cache_plus_final_check",
            "source_plus_raw_cache",
            "source_plus_initial_raw",
        ];
        for index in 0..3 {
            let strategy = strategies[(trial + index) % 3];
            let session = Instant::now();
            let mut scalar = std::collections::HashMap::new();
            let mut raw: VecDeque<(String, Vec<f32>, Value)> = VecDeque::new();
            let prep = Instant::now();
            if strategy == "source_plus_initial_raw" {
                let (x, m) = render(&pcm, &base, 0.0);
                raw.push_back((m["coefficients"].as_str().unwrap().to_owned(), x, m));
            }
            let settings_prep_s = seconds(prep);
            let mut rows = Vec::new();
            let mut max_cache_bytes = raw.iter().map(|(_, x, _)| x.len() * 4).sum::<usize>();
            for (name, s) in &edits {
                let start = Instant::now();
                let key = serde_json::to_string(s).unwrap();
                if let Some(gain) = scalar.get(&key) {
                    rows.push(json!({"edit":name,"elapsed_s":seconds(start),"scalar_hit":true,"gain":gain}));
                    continue;
                }
                let coeffs = format!(
                    "{:?}",
                    MasteringChain::new(pcm.sample_rate, usize::from(pcm.channels), s).coeffs
                );
                let hit = if strategy == "existing_scalar_cache_plus_final_check" {
                    None
                } else {
                    raw.iter().position(|(k, _, _)| *k == coeffs)
                };
                let (x, m) = if let Some(index) = hit {
                    let (_, x, m) = raw.remove(index).unwrap();
                    (x, m)
                } else {
                    render(&pcm, s, 0.0)
                };
                let lufs = m["raw_lufs"].as_f64().unwrap_or(f64::NEG_INFINITY);
                let peak = m["provisional_peak"].as_f64().unwrap();
                let gain = (f64::from(s.effective_target_lufs().unwrap()) - lufs).min(-1.0 - peak);
                // Final verification still walks the whole song; cache only
                // removes repeated nonlinear processing and raw measurement.
                let verify = Instant::now();
                let gain_lin = 10.0_f32.powf(gain as f32 / 20.0);
                let landed: Vec<_> = x.iter().map(|v| *v * gain_lin).collect();
                let delivered = measure(&landed, 48000, pcm.channels);
                let verify_s = seconds(verify);
                scalar.insert(key, gain);
                if strategy != "existing_scalar_cache_plus_final_check" {
                    raw.push_back((coeffs, x, m.clone()));
                    if raw.len() > 2 {
                        raw.pop_front();
                    }
                }
                max_cache_bytes =
                    max_cache_bytes.max(raw.iter().map(|(_, x, _)| x.len() * 4).sum());
                rows.push(json!({"edit":name,"elapsed_s":seconds(start),"raw_hit":hit.is_some(),
                    "final_verify_s":verify_s,"gain":gain,"delivered_lufs":delivered.0,"provisional_peak":delivered.1,
                    "render":if hit.is_some(){Value::Null}else{m}}));
            }
            sessions.push(json!({"trial":trial,"strategy":strategy,"settings_preparation_s":settings_prep_s,
                "total_session_s":seconds(session),"max_raw_cache_bytes":max_cache_bytes,"edits":rows}));
            report["sessions"] = json!(sessions);
            std::fs::write(
                out.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
        }
    }
    if timing_only {
        return;
    }
    // Save simple-rule/search candidates once. The independent Python analysis
    // selects and verifies actual whole-file output using corrected references.
    let mut candidates = Vec::new();
    let (control, metrics) = render(&pcm, &base, 0.0);
    write(&out.join("raw-control.wav"), &control, 48000, pcm.channels);
    let raw_lufs = metrics["raw_lufs"].as_f64().unwrap() as f32;
    candidates.push(json!({"id":"control","metrics":metrics}));
    let coeffs = MasteringChain::new(pcm.sample_rate, usize::from(pcm.channels), &base).coeffs;
    let d = 1.0 + coeffs.saturation_amount * 2.0;
    let sat_gain = if coeffs.saturation_amount > 0.0 {
        20.0 * (d / d.tanh()).log10()
    } else {
        0.0
    };
    for target in [-14.0, -9.0] {
        let simple =
            target - analysis.lufs_integrated - 20.0 * coeffs.input_gain_lin.log10() - sat_gain;
        for (policy, offsets) in [
            ("single", vec![0.0]),
            ("bounded", vec![-9.0, -6.0, -3.0, 3.0, 6.0, 9.0, 12.0]),
            ("attenuate", vec![0.0]),
        ] {
            for (i, offset) in offsets.iter().enumerate() {
                let drive = if policy == "attenuate" {
                    (target - raw_lufs).min(0.0)
                } else {
                    simple + offset
                };
                let id = format!("t{}-{policy}-{i}", -target as i32);
                let (x, m) = render(&pcm, &base, drive);
                write(&out.join(format!("raw-{id}.wav")), &x, 48000, pcm.channels);
                candidates.push(json!({"id":id,"target":target,"policy":policy,"metrics":m}));
                report["candidates"] = json!(candidates);
                std::fs::write(
                    out.join("report.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
            }
        }
    }
}
