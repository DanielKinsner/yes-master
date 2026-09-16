//! Whole-file numerical diagnosis using retained C1 settings; no policy changes.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{
    decode,
    dsp::{BiquadCoeffs, BiquadState, MasteringChain},
    types,
};
#[path = "../src/sample_rate.rs"]
mod sample_rate;

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn difference(x: &[f32], y: &[f32]) -> Value {
    assert_eq!(x.len(), y.len());
    let mut maximum = 0_f64;
    let mut squared = 0_f64;
    for (&a, &b) in x.iter().zip(y) {
        assert!(a.is_finite() && b.is_finite());
        let d = f64::from(a) - f64::from(b);
        maximum = maximum.max(d.abs());
        squared += d * d;
    }
    json!({"maximum":maximum,"rms":(squared/x.len() as f64).sqrt()})
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

fn chain(
    settings: &types::MasteringSettings,
    rate: u32,
    channels: u16,
    drive: f32,
) -> MasteringChain {
    let mut chain = MasteringChain::new(rate, usize::from(channels), settings);
    chain.coeffs.input_gain_lin *= 10_f32.powf(drive / 20.);
    chain
}

fn render(x: &[f32], mut chain: MasteringChain, rate: u32, channels: u16) -> Vec<f32> {
    let mut output = x.to_vec();
    chain.process_interleaved(&mut output, usize::from(channels));
    chain.flush_render_tail(&mut output, usize::from(channels));
    sample_rate::convert_interleaved(&output, rate, 48000, channels).unwrap()
}

// Identical f32 coefficients, f64 arithmetic/state; output stays f32 at each
// filter boundary. This isolates arithmetic precision from filter design.
fn filter(x: &mut [f32], c: &BiquadCoeffs, channels: usize, wide: bool) {
    let mut narrow = vec![BiquadState::default(); channels];
    let mut state = vec![(0_f64, 0_f64); channels];
    for frame in x.chunks_exact_mut(channels) {
        for (ch, sample) in frame.iter_mut().enumerate() {
            if wide {
                let x = f64::from(*sample);
                let (z1, z2) = &mut state[ch];
                let y = f64::from(c.b0) * x + *z1;
                *z1 = f64::from(c.b1) * x - f64::from(c.a1) * y + *z2;
                *z2 = f64::from(c.b2) * x - f64::from(c.a2) * y;
                *sample = y as f32;
            } else {
                *sample = narrow[ch].process(c, *sample);
            }
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        5,
        "base report, copy report, candidate id, fresh output JSON"
    );
    let start = Instant::now();
    let output = Path::new(&args[4]);
    assert!(!output.exists());
    let reports: Vec<Value> = args[1..3]
        .iter()
        .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
        .collect();
    assert_eq!(reports[0]["source_sha256"], reports[1]["source_sha256"]);
    assert_eq!(reports[0]["source_gain_db"], 0.);
    let source = Path::new(reports[0]["source"].as_str().unwrap());
    assert_eq!(sha(source), reports[0]["source_sha256"].as_str().unwrap());
    let pcm = decode::decode_full(source).unwrap();
    let rows: Vec<_> = reports
        .iter()
        .map(|r| {
            r["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == args[3])
                .unwrap()
        })
        .collect();
    assert_eq!(
        rows[0]["render"]["coefficients"],
        rows[1]["render"]["coefficients"]
    );
    assert!(matches!(
        rows[0]["policy"].as_str().unwrap(),
        "single" | "bounded"
    ));
    let settings: types::MasteringSettings =
        serde_json::from_value(rows[0]["requested_settings"].clone()).unwrap();
    let drive = rows[0]["operating_drive_db"].as_f64().unwrap() as f32;
    let make = || chain(&settings, pcm.sample_rate, pcm.channels, drive);
    assert_eq!(
        format!("{:?}", make().coeffs),
        rows[0]["render"]["coefficients"].as_str().unwrap()
    );
    let original = normalized(&pcm.samples, 0.);
    let copy = normalized(
        &pcm.samples,
        reports[1]["source_gain_db"].as_f64().unwrap() as f32,
    );
    let mut result = json!({"status":"running","source_sha256":sha(source),"base_report_sha256":sha(Path::new(&args[1])),
        "copy_report_sha256":sha(Path::new(&args[2])),"candidate":args[3],"normalized_input":difference(&original,&copy),
        "scope":"retained whole-file settings; numerical stage isolation, no sonic adoption or audibility claim",
        "filter_stages":[],"ablations":[]});
    let c = make().coeffs;
    let mut filters = Vec::new();
    for wide in [false, true] {
        let mut x: Vec<f32> = original.iter().map(|x| x * c.input_gain_lin).collect();
        let mut y: Vec<f32> = copy.iter().map(|x| x * c.input_gain_lin).collect();
        for (name, coeffs) in [
            ("hp1", c.sub_highpass),
            ("hp2", c.sub_highpass),
            ("sub", c.sub),
            ("low", c.low),
            ("low_mid", c.low_mid),
            ("mid", c.mid),
            ("high_mid", c.high_mid),
            ("high", c.high),
            ("sparkle", c.sparkle),
            ("warmth", c.warmth),
            ("presence_air", c.presence_air),
        ] {
            filter(&mut x, &coeffs, usize::from(pcm.channels), wide);
            filter(&mut y, &coeffs, usize::from(pcm.channels), wide);
            filters.push(json!({"stage":name,"wide_state":wide,"difference":difference(&x,&y)}));
        }
    }
    result["filter_stages"] = json!(filters);
    for mode in ["production", "highpass_identity"] {
        let modified = || {
            let mut chain = make();
            if mode == "highpass_identity" {
                chain.coeffs.sub_highpass = BiquadCoeffs::identity();
            }
            chain
        };
        let x = render(&original, modified(), pcm.sample_rate, pcm.channels);
        let y = render(&copy, modified(), pcm.sample_rate, pcm.channels);
        let mut row = json!({"mode":mode,"raw_response":difference(&x,&y)});
        if mode == "production" {
            let mut delivered = Vec::new();
            for (index, raw) in [&x, &y].into_iter().enumerate() {
                let gain = rows[index]["delivery"]["gain_lin"].as_f64().unwrap() as f32;
                let scaled: Vec<f32> = raw.iter().map(|x| x * gain).collect();
                let path = Path::new(rows[index]["path"].as_str().unwrap());
                assert_eq!(sha(path), rows[index]["sha256"].as_str().unwrap());
                let retained = decode::decode_full(path).unwrap();
                assert_eq!(
                    scaled, retained.samples,
                    "replay must reproduce the original candidate delivery"
                );
                delivered.push(scaled);
            }
            row["delivered_response"] = difference(&delivered[0], &delivered[1]);
            let shared_gain: Vec<f32> = y
                .iter()
                .map(|x| x * rows[0]["delivery"]["gain_lin"].as_f64().unwrap() as f32)
                .collect();
            row["same_final_gain"] = difference(&delivered[0], &shared_gain);
        }
        result["ablations"].as_array_mut().unwrap().push(row);
        std::fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
    result["status"] = json!("complete");
    result["elapsed_s"] = json!(start.elapsed().as_secs_f64());
    std::fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
