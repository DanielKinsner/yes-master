//! Native parity check: master a wav through the browser crate's entry point
//! and print delivered LUFS / true peak. Compare against the desktop render.
//! cargo run --release --example native_check -- in.wav out.wav universal 0.5 -14
use yes_master_web::{master_standard, measure_loudness};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut r = hound::WavReader::open(&a[1]).expect("open");
    let spec = r.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().map(|s| s.unwrap()).collect(),
        hound::SampleFormat::Int => { let m = (1i64 << (spec.bits_per_sample - 1)) as f32; r.samples::<i32>().map(|s| s.unwrap() as f32 / m).collect() }
    };
    let style = a.get(3).map(String::as_str).unwrap_or("universal");
    let intensity: f32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let target: f32 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(-14.0);
    let src = measure_loudness(&samples, spec.channels as u32, spec.sample_rate).unwrap();
    let t = std::time::Instant::now();
    let out = master_standard(&samples, spec.channels as u32, spec.sample_rate, style, intensity, target).unwrap();
    let el = t.elapsed().as_secs_f32();
    let m = measure_loudness(&out, spec.channels as u32, spec.sample_rate).unwrap();
    println!("source {:.2} LUFS / {:.2} dBTP -> {} @{} target {} => {:.2} LUFS / {:.2} dBTP  ({:.1}s audio in {:.2}s)", src[0], src[1], style, intensity, target, m[0], m[1], samples.len() as f32 / spec.channels as f32 / spec.sample_rate as f32, el);
    let mut w = hound::WavWriter::create(&a[2], hound::WavSpec { channels: spec.channels, sample_rate: spec.sample_rate, bits_per_sample: 32, sample_format: hound::SampleFormat::Float }).unwrap();
    for s in out { w.write_sample(s).unwrap(); }
    w.finalize().unwrap();
}
