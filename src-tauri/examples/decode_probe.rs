//! Diagnostic: run a file through the app's REAL decode path and report
//! what playback would actually receive. Useful when a user file
//! misbehaves (odd bit depths, sample rates, float WAVs, etc).
//!
//! Usage:
//!   cargo run --example decode_probe -- "C:\path\to\file.wav"

fn main() {
    let path = std::env::args().nth(1).expect("usage: decode_probe <file>");
    let p = std::path::Path::new(&path);

    match yes_master_lib::decode::probe_sample_rate(p) {
        Ok(sr) => println!("header probe: sample_rate={sr}"),
        Err(e) => println!("header probe error: {e:?}"),
    }

    match yes_master_lib::decode::decode_full(p) {
        Ok(pcm) => {
            let ch = pcm.channels.max(1) as usize;
            let frames = pcm.samples.len() / ch;
            let secs = frames as f64 / pcm.sample_rate as f64;
            let non_finite = pcm.samples.iter().filter(|s| !s.is_finite()).count();
            let peak = pcm
                .samples
                .iter()
                .filter(|s| s.is_finite())
                .fold(0f32, |a, &s| a.max(s.abs()));
            let over_unity = pcm
                .samples
                .iter()
                .filter(|s| s.is_finite() && s.abs() > 1.0)
                .count();
            println!(
                "decode ok: sample_rate={} channels={} frames={} duration={:.2}s",
                pcm.sample_rate, pcm.channels, frames, secs
            );
            println!(
                "signal: peak={:.4} ({:+.2} dBFS), samples_over_1.0={} ({:.3}%), non_finite={}",
                peak,
                20.0 * peak.max(1e-9).log10(),
                over_unity,
                100.0 * over_unity as f64 / pcm.samples.len().max(1) as f64,
                non_finite
            );
        }
        Err(e) => println!("decode error: {e:?}"),
    }
}
