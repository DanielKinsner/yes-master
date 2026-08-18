//! Diagnostic: realtime throughput of the live mastering chain over a real
//! file, in THIS build profile. If the factor is near (or under) 1.0x, live
//! Mastered audition will underrun — crackle, stutter, or stall.
//!
//! Usage:
//!   cargo run --example chain_bench -- "C:\path\to\file.wav"

use std::time::Instant;
use yes_master_lib::dsp::MasteringChain;
use yes_master_lib::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};

fn main() {
    let path = std::env::args().nth(1).expect("usage: chain_bench <file>");
    let pcm =
        yes_master_lib::decode::decode_full(std::path::Path::new(&path)).expect("decode failed");
    let ch = pcm.channels.max(1) as usize;

    let settings = MasteringSettings {
        preset: Preset::Universal,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: yes_master_lib::EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings::default(),
    };
    let mut chain = MasteringChain::new(pcm.sample_rate, ch, &settings);

    let frames = pcm.samples.len() / ch;
    let mut frame = vec![0f32; ch];
    let start = Instant::now();
    for f in 0..frames {
        frame.copy_from_slice(&pcm.samples[f * ch..(f + 1) * ch]);
        chain.process_frame_inplace(&mut frame);
        std::hint::black_box(&frame);
    }
    let elapsed = start.elapsed().as_secs_f64().max(1e-9);
    let audio_secs = frames as f64 / pcm.sample_rate as f64;
    println!(
        "profile={} | {:.1}s of {}ch @ {} Hz processed in {:.2}s -> {:.2}x realtime",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        audio_secs,
        ch,
        pcm.sample_rate,
        elapsed,
        audio_secs / elapsed
    );
}
