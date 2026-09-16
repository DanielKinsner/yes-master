//! Isolated, generated full-chain arithmetic comparison; no production switch.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{decode, engine, guardrails, output_protection, types};
#[path = "dsp_narrow.rs"]
#[allow(dead_code)]
mod dsp;
#[allow(dead_code)]
mod dsp_wide;
#[allow(dead_code)]
mod sample_rate;

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn diff(x: &[f32], y: &[f32]) -> Value {
    assert_eq!(x.len(), y.len());
    let (mut maximum, mut sum) = (0_f64, 0_f64);
    for (&x, &y) in x.iter().zip(y) {
        assert!(x.is_finite() && y.is_finite());
        let delta = f64::from(x) - f64::from(y);
        maximum = maximum.max(delta.abs());
        sum += delta * delta;
    }
    json!({"maximum":maximum,"rms":(sum/x.len() as f64).sqrt()})
}

fn normalized(x: &[f32], gain_db: f32) -> Vec<f32> {
    let gain = 10_f32.powf(gain_db / 20.);
    let peak = x
        .iter()
        .map(|x| f64::from((x * gain).abs()))
        .fold(0_f64, f64::max);
    x.iter()
        .map(|x| (f64::from(x * gain) / peak) as f32)
        .collect()
}

fn render(
    x: &[f32],
    rate: u32,
    channels: u16,
    settings: &types::MasteringSettings,
    drive: f32,
    wide: bool,
    expected_coeffs: &str,
) -> (Vec<f32>, f64) {
    let mut samples = x.to_vec();
    let start = Instant::now();
    macro_rules! process {
        ($module:ident) => {{
            let mut chain = $module::MasteringChain::new(rate, usize::from(channels), settings);
            chain.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
            assert_eq!(format!("{:?}", chain.coeffs), expected_coeffs);
            chain.process_interleaved(&mut samples, usize::from(channels));
            chain.flush_render_tail(&mut samples, usize::from(channels));
        }};
    }
    if wide {
        process!(dsp_wide);
    } else {
        process!(dsp);
    }
    let chain_s = start.elapsed().as_secs_f64();
    (
        sample_rate::convert_interleaved(&samples, rate, 48000, channels).unwrap(),
        chain_s,
    )
}

fn write(path: &Path, x: &[f32], channels: u16) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate: 48000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for &x in x {
        assert!(x.is_finite());
        writer.write_sample(x).unwrap();
    }
    writer.finalize().unwrap();
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "original run report, candidate id, fresh output directory"
    );
    let output = Path::new(&args[3]);
    assert!(!output.exists());
    std::fs::create_dir_all(output).unwrap();
    let report: Value = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let row = report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == args[2])
        .unwrap();
    assert_eq!(row["source_gain_db"], 0.);
    let source = Path::new(report["source"].as_str().unwrap());
    assert_eq!(sha(source), report["source_sha256"].as_str().unwrap());
    let pcm = decode::decode_full(source).unwrap();
    let settings: types::MasteringSettings =
        serde_json::from_value(row["requested_settings"].clone()).unwrap();
    let drive = row["operating_drive_db"].as_f64().unwrap() as f32;
    let coeffs = row["render"]["coefficients"].as_str().unwrap();
    let mut result = json!({"status":"running","source":source,"source_sha256":sha(source),
        "original_report_sha256":sha(Path::new(&args[1])),"candidate":args[2],
        "requested_settings":settings,"operating_drive_db":drive,"coefficients":coeffs,
        "scope":"full source-normalized chain at identical coefficients, f32 boundary outputs, qualified whole-file float delivery; development experiment, no production policy",
        "arithmetic":[],"gain_copies":[],"timing_trials":[]});
    let original = normalized(&pcm.samples, 0.);
    let mut delivered_original: Vec<Vec<f32>> = Vec::new();
    let mut raw_original: Vec<Vec<f32>> = Vec::new();
    for gain_db in [0_f32, -40., -80.] {
        let input = normalized(&pcm.samples, gain_db);
        let mut raw_pair = Vec::new();
        let mut delivered_pair = Vec::new();
        for wide in [false, true] {
            let (raw, chain_s) = render(
                &input,
                pcm.sample_rate,
                pcm.channels,
                &settings,
                drive,
                wide,
                coeffs,
            );
            if !wide && gain_db == 0. {
                let retained = Path::new(row["path"].as_str().unwrap());
                assert_eq!(sha(retained), row["sha256"].as_str().unwrap());
                let retained = decode::decode_full(retained).unwrap();
                let old_gain = row["delivery"]["gain_lin"].as_f64().unwrap() as f32;
                assert!(
                    raw.iter()
                        .zip(&retained.samples)
                        .all(|(x, y)| x * old_gain == *y),
                    "narrow replay must reproduce frozen candidate PCM"
                );
                assert_eq!(raw.len(), retained.samples.len());
            }
            let raw_lufs = engine::measure_integrated_lufs(&raw, 48000, pcm.channels).unwrap();
            let mut delivered = raw.clone();
            let started = Instant::now();
            let protection = output_protection::finalize(
                &mut delivered,
                48000,
                pcm.channels,
                32,
                settings.effective_target_lufs(),
                settings.effective_ceiling_dbtp(),
                None,
            )
            .unwrap();
            let finalize_s = started.elapsed().as_secs_f64();
            let path = output.join(format!(
                "{}-{}.wav",
                if wide { "wide" } else { "narrow" },
                -gain_db as i32
            ));
            write(&path, &delivered, pcm.channels);
            let mut record = json!({"wide":wide,"source_gain_db":gain_db,"chain_s":chain_s,"finalize_s":finalize_s,
                "raw_lufs":raw_lufs,"delivered_lufs":protection.lufs,"gain_lin":protection.gain_lin,
                "true_peak_dbtp":protection.true_peak_dbtp,"path":path,"sha256":sha(&path)});
            if gain_db != 0. {
                let index = usize::from(wide);
                record["raw_vs_original"] = diff(&raw_original[index], &raw);
                record["delivered_vs_original"] = diff(&delivered_original[index], &delivered);
            }
            result["gain_copies"].as_array_mut().unwrap().push(record);
            raw_pair.push(raw);
            delivered_pair.push(delivered);
        }
        result["arithmetic"].as_array_mut().unwrap().push(json!({"source_gain_db":gain_db,
            "normalized_input_vs_original":diff(&original,&input),"raw_wide_vs_narrow":diff(&raw_pair[0],&raw_pair[1]),
            "delivered_wide_vs_narrow":diff(&delivered_pair[0],&delivered_pair[1])}));
        if gain_db == 0. {
            raw_original = raw_pair;
            delivered_original = delivered_pair;
        }
        std::fs::write(
            output.join("report.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        println!("Completed {} gain {}", args[2], gain_db);
    }
    // Alternating AB/BA whole-song trials; processing excludes copy/SRC/finalizer.
    // Concurrent load must be recorded separately; timings are not a CI gate.
    for trial in 0..3 {
        for wide in if trial % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let (samples, seconds) = render(
                &original,
                pcm.sample_rate,
                pcm.channels,
                &settings,
                drive,
                wide,
                coeffs,
            );
            assert_eq!(
                samples,
                raw_original[usize::from(wide)],
                "repeat processing must be deterministic"
            );
            result["timing_trials"]
                .as_array_mut()
                .unwrap()
                .push(json!({"trial":trial,"wide":wide,"chain_s":seconds}));
        }
    }
    result["status"] = json!("complete");
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
