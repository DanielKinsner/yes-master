//! Diagnostic: realtime throughput of the live mastering chain over a real
//! file, in THIS build profile. If the factor is near (or under) 1.0x, live
//! Mastered audition will underrun — crackle, stutter, or stall.
//!
//! Usage:
//!   cargo run --example chain_bench -- "C:\path\to\file.wav" [--smart-match]
//!
//! `--smart-match` measures the file through the app's meter, resolves that
//! as the track profile, and sets a reference 3 dB darker in sub and 3 dB
//! brighter in air, at the default 50% amount, so the Smart Match corrective
//! path is active (Smart Match plan, gate G8). Without it the run is exactly
//! the baseline configuration. `--with-shelves` turns on the Advanced Warmth
//! and Presence/Air shelves, the one setup where Smart Match's corrective
//! filter needs a stage of its own; compare it with and without
//! `--smart-match`.

use std::time::Instant;
use yes_master_lib::dsp::MasteringChain;
use yes_master_lib::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: chain_bench <file> [--smart-match]");
    let smart_match = std::env::args().skip(2).any(|a| a == "--smart-match");
    let with_shelves = std::env::args().skip(2).any(|a| a == "--with-shelves");
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
    let mut settings = settings;
    if with_shelves {
        settings.advanced.warmth = Some(0.5);
        settings.advanced.presence_air = Some(0.25);
    }
    if smart_match {
        use yes_master_lib::smart_match::{measure_shares, shares_array, shares_to_db};
        use yes_master_lib::{SmartMatchReference, SmartMatchSettings, SourceProfile};
        let track = measure_shares(&pcm.samples, pcm.sample_rate, ch).expect("measurable file");
        let mut levels = shares_to_db(&shares_array(&track));
        levels[0] -= 3.0;
        levels[5] += 3.0;
        let lin = levels.map(|d| 10f64.powf(d / 10.0));
        let total: f64 = lin.iter().sum();
        let r = lin.map(|v| (v / total) as f32);
        settings.advanced.source_profile = Some(SourceProfile {
            spectral_6: track,
            dynamic_range_p95_p10_db: 100.0,
            dynamic_range_lu: 0.0,
            stereo_correlation: None,
            stereo_width: 0.0,
        });
        // Isolate Smart Match's cost: no guardrail trims in this run.
        settings.advanced.adaptive_strength = Some(0.0);
        settings.advanced.smart_match = Some(SmartMatchSettings {
            reference: Some(SmartMatchReference {
                name: "bench".into(),
                spectral_6: yes_master_lib::SpectralBalance6 {
                    sub: r[0],
                    low: r[1],
                    low_mid: r[2],
                    mid: r[3],
                    presence: r[4],
                    air: r[5],
                },
            }),
            ..SmartMatchSettings::default()
        });
    }
    let mut chain = MasteringChain::new(pcm.sample_rate, ch, &settings);
    if smart_match {
        let d = yes_master_lib::smart_match::decide_for_settings(&settings);
        assert!(
            d.changes_anything(),
            "the Smart Match correction must be active for this run: {d:?}"
        );
        println!("smart match active: corrections {:?} dB", d.correction_db);
    }

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
