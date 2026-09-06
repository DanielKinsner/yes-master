//! Opt-in diagnostics over owner-supplied files; never writes source audio.
//! YES_MASTER_BENCH_FILE=<path> cargo test --lib listening_stage_bench -- --ignored --nocapture
use super::*;
use std::time::Instant;

#[test]
#[ignore = "local DSP throughput and bitwise-output diagnostic"]
fn listening_chain_probe() {
    use sha2::{Digest, Sha256};
    let path = std::env::var("YES_MASTER_BENCH_FILE").expect("set existing fixture path");
    let pcm = decode_full(Path::new(&path)).unwrap();
    let samples = crate::engine::volume_match_window(&pcm.samples, pcm.sample_rate, pcm.channels);
    let mut settings = super::tests::settings_with_intensity(0.5);
    settings.advanced = crate::types::AdvancedSettings::default();
    for (label, compression, saturation) in [
        ("full", true, true),
        ("no_compression_probe", false, true),
        ("no_saturation_probe", true, false),
    ] {
        let mut elapsed = 0.0;
        let mut hash = String::new();
        for _ in 0..5 {
            let mut rendered = samples.clone();
            let mut chain =
                crate::dsp::MasteringChain::new(pcm.sample_rate, pcm.channels as usize, &settings);
            if !compression {
                chain.coeffs.compression_active = false;
            }
            if !saturation {
                chain.coeffs.saturation_amount = 0.0;
            }
            let start = Instant::now();
            chain.process_interleaved(&mut rendered, pcm.channels as usize);
            elapsed += start.elapsed().as_secs_f64();
            let mut digest = Sha256::new();
            for sample in rendered {
                digest.update(sample.to_le_bytes());
            }
            hash = format!("{:x}", digest.finalize());
        }
        println!(
            "probe={label} rate={} mean_seconds={:.6} sha256={hash}",
            pcm.sample_rate,
            elapsed / 5.0
        );
    }
}

#[test]
#[ignore = "local fixture throughput diagnostic, not a timing gate"]
fn listening_stage_bench() {
    let path = std::env::var("YES_MASTER_BENCH_FILE").expect("set existing fixture path");
    let time = |name: &str, start: Instant| {
        println!("stage={name} seconds={:.6}", start.elapsed().as_secs_f64())
    };
    println!(
        "fixture={path} profile={}",
        if cfg!(debug_assertions) {
            "dev opt1/dependencies opt3"
        } else {
            "release"
        }
    );
    let start = Instant::now();
    let pcm = decode_full(Path::new(&path)).unwrap();
    time("decode", start);
    println!(
        "rate={} channels={} frames={} pcm_bytes={}",
        pcm.sample_rate,
        pcm.channels,
        pcm.samples.len() / pcm.channels as usize,
        pcm.samples.len() * 4
    );
    let start = Instant::now();
    for _ in 0..3 {
        std::hint::black_box(pcm.clone());
    }
    time("three_pcm_clones", start);
    let shared: PlaybackPcm = pcm.into();
    let start = Instant::now();
    for _ in 0..3 {
        std::hint::black_box(shared.clone());
    }
    time("three_playback_cache_clones", start);
    let pcm = shared;
    let mut settings = super::tests::settings_with_intensity(0.5);
    settings.advanced = crate::types::AdvancedSettings::default();
    settings.delivery_profile = crate::types::DeliveryProfile::StreamingUniversal;
    let start = Instant::now();
    let vm = crate::engine::preview_volume_match_gain(
        &pcm.samples,
        pcm.sample_rate,
        pcm.channels,
        &settings,
    )
    .unwrap();
    time("vm_eight_seconds", start);
    println!("vm_gain={vm}");
    let start = Instant::now();
    let mut rendered = pcm.samples.to_vec();
    time("landing_copy", start);
    let channels = pcm.channels as usize;
    let start = Instant::now();
    let mut chain = crate::dsp::MasteringChain::new(pcm.sample_rate, channels, &settings);
    chain.process_interleaved(&mut rendered, channels);
    time("stateful_chain", start);
    let start = Instant::now();
    chain.flush_render_tail(&mut rendered, channels);
    time("tail_flush", start);
    let rate = settings.effective_sample_rate(pcm.sample_rate);
    let start = Instant::now();
    if rate != pcm.sample_rate {
        rendered =
            crate::sample_rate::convert_interleaved(&rendered, pcm.sample_rate, rate, pcm.channels)
                .unwrap();
    }
    time("sample_rate_conversion", start);
    println!(
        "measurement_rate={rate} frames={}",
        rendered.len() / channels
    );
    for (label, mode) in [
        ("loudness", ebur128::Mode::I),
        (
            "loudness_and_true_peak",
            ebur128::Mode::I | ebur128::Mode::TRUE_PEAK,
        ),
    ] {
        let start = Instant::now();
        let mut meter = ebur128::EbuR128::new(pcm.channels as u32, rate, mode).unwrap();
        meter.add_frames_f32(&rendered).unwrap();
        time(&format!("{label}_filter_and_blocks"), start);
        let start = Instant::now();
        println!("{label}_lufs={}", meter.loudness_global().unwrap());
        time(&format!("{label}_global_gate"), start);
    }
    drop(rendered);
    drop(pcm);
    let start = Instant::now();
    crate::analysis::analyze_one(TrackId("bench".into()), Path::new(&path), true).unwrap();
    time("analysis_including_its_decode", start);
}
