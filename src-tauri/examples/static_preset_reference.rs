//! Reference-only buffers; never delivered masters or a production DSP path.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};
use yes_master_lib::{
    decode,
    dsp::{BiquadCoeffs, BiquadState, ChainCoeffs},
    types,
};

fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
fn pcm_sha(samples: &[f32]) -> String {
    let mut hash = Sha256::new();
    for sample in samples {
        hash.update(sample.to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}
fn section(samples: &mut [f32], coeff: &BiquadCoeffs) {
    let mut states = [BiquadState::default(); 2];
    for frame in samples.chunks_exact_mut(2) {
        for ch in 0..2 {
            frame[ch] = states[ch].process(coeff, frame[ch]);
        }
    }
}
fn eqs(c: &ChainCoeffs) -> [BiquadCoeffs; 11] {
    [
        c.sub_highpass,
        c.sub_highpass,
        c.sub,
        c.low,
        c.low_mid,
        c.mid,
        c.high_mid,
        c.high,
        c.sparkle,
        c.warmth,
        c.presence_air,
    ]
}
fn array(c: BiquadCoeffs) -> [f32; 5] {
    [c.b0, c.b1, c.b2, c.a1, c.a2]
}
fn topology(c: &ChainCoeffs) -> Value {
    json!({"eq":eqs(c).map(array),"compression_active":c.compression_active,
        "branches":[[array(c.comp_low_lp),array(c.comp_low_lp)],
        [array(c.comp_mid_hp),array(c.comp_mid_hp),array(c.comp_mid_lp),array(c.comp_mid_lp)],
        [array(c.comp_high_hp),array(c.comp_high_hp)]],
        "makeup":[c.comp_low_makeup_lin,c.comp_mid_makeup_lin,c.comp_high_makeup_lin],
        "width":c.width_side_scale})
}
fn reference(input: &[f32], c: &ChainCoeffs) -> Vec<f32> {
    let mut samples = input.to_vec();
    for coeff in eqs(c) {
        section(&mut samples, &coeff);
    }
    if c.compression_active {
        let mut low = samples.clone();
        let mut mid = samples.clone();
        let mut high = samples.clone();
        for coeff in [c.comp_low_lp, c.comp_low_lp] {
            section(&mut low, &coeff);
        }
        for coeff in [c.comp_mid_hp, c.comp_mid_hp, c.comp_mid_lp, c.comp_mid_lp] {
            section(&mut mid, &coeff);
        }
        for coeff in [c.comp_high_hp, c.comp_high_hp] {
            section(&mut high, &coeff);
        }
        for i in 0..samples.len() {
            samples[i] = low[i] * c.comp_low_makeup_lin
                + mid[i] * c.comp_mid_makeup_lin
                + high[i] * c.comp_high_makeup_lin;
        }
    }
    if (c.width_side_scale - 1.).abs() > 1e-5 {
        for frame in samples.chunks_exact_mut(2) {
            let mid = 0.5 * (frame[0] + frame[1]);
            let side = 0.5 * (frame[0] - frame[1]) * c.width_side_scale;
            frame[0] = mid + side;
            frame[1] = mid - side;
        }
    }
    assert!(samples.iter().all(|v| v.is_finite()));
    samples
}
fn save(path: &Path, samples: &[f32], rate: u32) {
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
    for value in samples {
        writer.write_sample(*value).unwrap();
    }
    writer.finalize().unwrap();
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let job_path = Path::new(&args[1]);
    let directory = Path::new(&args[2]);
    assert!(!directory.exists());
    let job: Value = serde_json::from_slice(&std::fs::read(job_path).unwrap()).unwrap();
    assert_eq!(job["experiment"], "static-preset-reference-v1");
    assert_eq!(
        sha(Path::new(job["specification"].as_str().unwrap())),
        job["specification_sha256"]
    );
    std::fs::create_dir_all(directory).unwrap();
    let mut report = json!({"status":"running","job_sha256":sha(job_path),"rows":[]});
    for case in job["cases"].as_array().unwrap() {
        let source = Path::new(case["source"].as_str().unwrap());
        assert_eq!(sha(source), case["source_sha256"]);
        let pcm = decode::decode_full(source).unwrap();
        assert_eq!(pcm.channels, 2);
        assert_eq!(pcm.sample_rate, case["rate"].as_u64().unwrap() as u32);
        let settings: types::MasteringSettings =
            serde_json::from_value(case["settings"].clone()).unwrap();
        let coeffs = ChainCoeffs::from_settings(pcm.sample_rate, &settings);
        let mut original_coeffs = coeffs;
        original_coeffs.input_gain_lin *=
            10_f32.powf(case["original_drive"].as_f64().unwrap() as f32 / 20.);
        assert_eq!(
            format!("{original_coeffs:?}"),
            case["original_coefficients"]
        );
        let mut inputs = Vec::new();
        for fixture in case["fixtures"].as_array().unwrap() {
            inputs.push((
                fixture["name"].as_str().unwrap().to_owned(),
                Some(fixture.clone()),
                None,
            ));
        }
        for gain in [0_f64, -40., -80.] {
            inputs.push((format!("copy{gain}"), None, Some(gain)));
        }
        let mut original: Option<Vec<f32>> = None;
        for (label, fixture, gain) in inputs {
            let started = Instant::now();
            let input = if let Some(fixture) = &fixture {
                let path = Path::new(fixture["path"].as_str().unwrap());
                assert_eq!(sha(path), fixture["sha256"]);
                let mut reader = hound::WavReader::open(path).unwrap();
                assert_eq!(reader.spec().channels, 2);
                assert_eq!(reader.spec().sample_rate, pcm.sample_rate);
                reader
                    .samples::<f32>()
                    .map(Result::unwrap)
                    .collect::<Vec<_>>()
            } else {
                let gain = 10_f64.powf(gain.unwrap() / 20.);
                let mut copied: Vec<f32> = pcm
                    .samples
                    .iter()
                    .map(|v| (f64::from(*v) * gain) as f32)
                    .collect();
                let peak = copied.iter().map(|v| f64::from(v.abs())).fold(0., f64::max);
                assert!(peak > 0.);
                for value in &mut copied {
                    *value = (f64::from(*value) / peak) as f32;
                }
                copied
            };
            let input_sha = pcm_sha(&input);
            let output = reference(&input, &coeffs);
            let peak = output.iter().map(|v| v.abs()).fold(0_f32, f32::max);
            let difference = original.as_ref().filter(|_| gain.is_some()).map(|prior| {
                let scale = prior.iter().map(|v| v.abs()).fold(1e-12_f32, f32::max);
                prior
                    .iter()
                    .zip(&output)
                    .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs() / f64::from(scale))
                    .fold(0., f64::max)
            });
            if gain == Some(0.) {
                original = Some(output.clone());
            }
            let id = format!("{}-{label}", case["id"].as_str().unwrap());
            let path = directory.join(format!("{id}.wav"));
            save(&path, &output, pcm.sample_rate);
            report["rows"].as_array_mut().unwrap().push(json!({"id":id,"case":case["id"],"fixture":fixture,
                "source_gain_db":gain,"input_pcm_sha256":input_sha,"path":path,"sha256":sha(&path),
                "frames":output.len()/2,"rate":pcm.sample_rate,"channels":2,"peak":peak,
                "topology":topology(&coeffs),"relative_copy_difference":difference,"work_s":started.elapsed().as_secs_f64()}));
            std::fs::write(
                directory.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!("{id}");
        }
    }
    report["status"] = json!("complete");
    std::fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
