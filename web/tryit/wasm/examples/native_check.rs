//! Native parity check: master a wav through the browser crate's entry point
//! (whole-track analysis + profile + chain + landing, exactly as the worker
//! drives it) and print delivered LUFS / true peak. Compare against the
//! desktop render of the same file.
//! cargo run --release --example native_check -- in.wav out.wav universal 0.5 -14
use yes_master_web::{analyze_dynamics, analyze_loudness, analyze_stereo, analyze_tonal, build_profile, master_standard, profile_digest};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut r = hound::WavReader::open(&a[1]).expect("open");
    let spec = r.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().map(|s| s.unwrap()).collect(),
        hound::SampleFormat::Int => { let m = (1i64 << (spec.bits_per_sample - 1)) as f32; r.samples::<i32>().map(|s| s.unwrap() as f32 / m).collect() }
    };
    let ch = spec.channels as u32;
    let style = a.get(3).map(String::as_str).unwrap_or("universal");
    let intensity: f32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let target: f32 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(-14.0);
    let t = std::time::Instant::now();
    let loud = analyze_loudness(&samples, ch, spec.sample_rate).unwrap();
    let dr = analyze_dynamics(&samples, ch, spec.sample_rate);
    let st = analyze_stereo(&samples, ch);
    let tonal = analyze_tonal(&samples, ch, spec.sample_rate).unwrap();
    let profile = build_profile(&tonal, dr, loud[2], st[0], st[1]).unwrap();
    let analysis_s = t.elapsed().as_secs_f32();
    println!("analysis {:.2}s: {} LUFS / {:.2} dBTP / LRA {:.1} LU; profile: {}", analysis_s, loud[0], loud[1], loud[2], profile_digest(&profile).unwrap());
    let t = std::time::Instant::now();
    let out = master_standard(&samples, ch, spec.sample_rate, style, intensity, target, loud[0], &profile).unwrap();
    let el = t.elapsed().as_secs_f32();
    println!("{} @{} target {} => {:.2} LUFS / {:.2} dBTP  ({:.1}s audio rendered in {:.2}s)", style, intensity, target, out.lufs(), out.tp(), samples.len() as f32 / ch as f32 / spec.sample_rate as f32, el);
    let mut w = hound::WavWriter::create(&a[2], hound::WavSpec { channels: spec.channels, sample_rate: spec.sample_rate, bits_per_sample: 32, sample_format: hound::SampleFormat::Float }).unwrap();
    for s in out.samples() { w.write_sample(s).unwrap(); }
    w.finalize().unwrap();
}
