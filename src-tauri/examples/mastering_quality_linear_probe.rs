//! Test peak reuse under uniform gain plus an exact whole-file residual scan.
//! This is an offline experiment; production landing is not changed here.
//! Args: inputs.json fresh-output-dir. Entries may select gains/bits.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::f64::consts::PI;
use std::path::Path;
use std::time::Instant;
use yes_master_lib::peak_meter::{self, pcm::PcmSource};
use yes_master_lib::wav_writer::DeliveryPcm;

fn residual_gain(frames: usize) -> f64 {
    // At most two samples at distance >= k-1/2 from t, with sinc <=1
    // for the nearest sample. Bound the harmonic sum by its first term
    // plus an integral. This holds also outside the finite input interval.
    let sinc = if frames == 0 {
        0.
    } else if frames == 1 {
        1.
    } else {
        1. + 2. / PI * (2. + (2. * frames as f64 - 1.).ln())
    };
    let bytes = include_bytes!("../src/peak_meter/soxr16.f64le");
    let kernel: Vec<_> = bytes
        .chunks_exact(8)
        .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
        .collect();
    let fir = (0..16)
        .map(|phase| {
            kernel
                .iter()
                .skip(phase)
                .step_by(16)
                .map(|v| v.abs())
                .sum::<f64>()
        })
        .fold(0_f64, f64::max);
    // The same qualified finite lowpass reconstruction and numerical envelope.
    sinc.max((fir + 1e-10) / (1. - PI * PI / (8. * 16. * 16.))) * (1. + 16. * f64::EPSILON)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let entries: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let out = Path::new(&args[2]);
    assert!(!out.exists(), "preserve completed evidence");
    std::fs::create_dir_all(out).unwrap();
    let mut rows = Vec::new();
    for entry in entries {
        let path = Path::new(entry["path"].as_str().unwrap());
        let pcm = yes_master_lib::decode::decode_full(path).unwrap();
        let channels = usize::from(pcm.channels);
        let frames = pcm.samples.len() / channels;
        let start = Instant::now();
        let prepared = peak_meter::measure(&pcm.samples, channels, || false).unwrap();
        let prepare_s = start.elapsed().as_secs_f64();
        let norm = residual_gain(frames);
        let gains: Vec<f64> = entry
            .get("gains")
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .unwrap_or(vec![-12., 0., 6.]);
        let bits: Vec<u16> = entry
            .get("bits")
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .unwrap_or(vec![16, 24, 32]);
        for db in gains {
            let gain = 10_f32.powf(db as f32 / 20.);
            let scaled: Vec<_> = pcm.samples.iter().map(|v| *v * gain).collect();
            for &bit_depth in &bits {
                let start = Instant::now();
                let source = DeliveryPcm::new(&scaled, pcm.channels, bit_depth, || false).unwrap();
                let mut residual = vec![0_f64; channels];
                let mut scratch = vec![0.; 4096];
                for (c, error) in residual.iter_mut().enumerate() {
                    for a in (0..frames).step_by(4096) {
                        let n = 4096.min(frames - a);
                        source.read_channel(c, a, &mut scratch[..n]).unwrap();
                        for (i, &value) in scratch[..n].iter().enumerate() {
                            // Product of two f32 values is exact in f64. This
                            // scans ACTUAL delivery, including gain rounding,
                            // deterministic dither, quantization and clipping.
                            let ideal =
                                f64::from(pcm.samples[(a + i) * channels + c]) * f64::from(gain);
                            *error = error.max((value - ideal).abs());
                        }
                    }
                }
                let lower: Vec<_> = prepared
                    .channels
                    .iter()
                    .zip(&residual)
                    .map(|(c, e)| (c.lower * f64::from(gain) - norm * e).max(0.))
                    .collect();
                let upper: Vec<_> = prepared
                    .channels
                    .iter()
                    .zip(&residual)
                    .map(|(c, e)| c.upper * f64::from(gain) + norm * e)
                    .collect();
                let reuse_s = start.elapsed().as_secs_f64();
                let start = Instant::now();
                let fresh = peak_meter::measure_source(&source, || false).unwrap();
                let fresh_s = start.elapsed().as_secs_f64();
                let intersects = fresh
                    .channels
                    .iter()
                    .enumerate()
                    .all(|(c, p)| upper[c] >= p.lower && lower[c] <= p.upper);
                // Optional real output enables independent reconstruction checks.
                let output_path = if entry["write"].as_bool() == Some(true) {
                    let file = out.join(format!(
                        "{}-g{db}-b{bit_depth}.wav",
                        entry["id"].as_str().unwrap()
                    ));
                    let spec = hound::WavSpec {
                        channels: pcm.channels,
                        sample_rate: pcm.sample_rate,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };
                    let mut writer = hound::WavWriter::create(&file, spec).unwrap();
                    let mut planar = vec![vec![0.; 4096]; channels];
                    for a in (0..frames).step_by(4096) {
                        let n = 4096.min(frames - a);
                        for (c, data) in planar.iter_mut().enumerate() {
                            source.read_channel(c, a, &mut data[..n]).unwrap();
                        }
                        for i in 0..n {
                            for channel in &planar {
                                writer.write_sample(channel[i] as f32).unwrap();
                            }
                        }
                    }
                    writer.finalize().unwrap();
                    Some(file.to_string_lossy().into_owned())
                } else {
                    None
                };
                rows.push(json!({"id":entry["id"],"source_sha256":format!("{:x}",Sha256::digest(std::fs::read(path).unwrap())),
                    "rate":pcm.sample_rate,"frames":frames,"channels":channels,"gain_db":db,"gain":gain,"bits":bit_depth,
                    "prepare_s":prepare_s,"reuse_s":reuse_s,"fresh_s":fresh_s,"checkpoint_bytes":source.checkpoint_bytes(),
                    "residual_gain":norm,"maximum_residual":residual,"reused_lower":lower,"reused_upper":upper,
                    "fresh":fresh,"intervals_intersect":intersects,"path":output_path}));
                std::fs::write(
                    out.join("comparison.json"),
                    serde_json::to_vec_pretty(
                        &json!({"version":"linear-residual-reuse-1","rows":rows}),
                    )
                    .unwrap(),
                )
                .unwrap();
                assert!(intersects, "{} gain={db} bits={bit_depth}", entry["id"]);
            }
        }
        println!("{} complete", entry["id"]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn residual_norm_contains_direct_finite_cardinal_sums() {
        for n in [1, 2, 3, 17, 63, 257, 1025] {
            let bound = residual_gain(n);
            for phase in 0..64 {
                for integer in [
                    -(n as i64),
                    -1,
                    0,
                    1,
                    n as i64 / 2,
                    n as i64 - 1,
                    n as i64,
                    2 * n as i64,
                ] {
                    let t = integer as f64 + f64::from(phase) / 64.;
                    let exact = (0..n)
                        .map(|i| {
                            let d = PI * (t - i as f64);
                            if d.abs() < 1e-14 {
                                1.
                            } else {
                                (d.sin() / d).abs()
                            }
                        })
                        .sum::<f64>();
                    assert!(exact <= bound, "n={n} t={t}: {exact} > {bound}");
                }
            }
        }
    }
}
