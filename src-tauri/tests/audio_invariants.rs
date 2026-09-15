//! Independent behavioral regressions from the September 2026 audio audit.
//! Synthetic signals only; runs in the normal cargo test lane on every OS.
mod common;

use std::path::Path;
use yes_master_lib::dsp::{Limiter, MasteringChain};
use yes_master_lib::engine::{
    analyze_tracks_core_with_progress_sync, mastering_render, preview_landing, AnalyzeRequest,
};
use yes_master_lib::*;

fn sine(sr: u32, seconds: usize, hz: f64, amplitude: f64) -> Vec<f32> {
    (0..sr as usize * seconds)
        .flat_map(|i| {
            let x = (amplitude * (std::f64::consts::TAU * hz * i as f64 / sr as f64).sin()) as f32;
            [x, x]
        })
        .collect()
}

fn write(path: &Path, sr: u32, samples: &[f32]) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: sr,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for sample in samples {
        writer.write_sample(*sample).unwrap();
    }
    writer.finalize().unwrap();
}

fn measure(samples: &[f32], sr: u32, channels: u32) -> (f64, f64) {
    let mut ebu =
        ebur128::EbuR128::new(channels, sr, ebur128::Mode::I | ebur128::Mode::TRUE_PEAK).unwrap();
    ebu.add_frames_f32(samples).unwrap();
    let peak = (0..channels)
        .map(|ch| ebu.true_peak(ch).unwrap())
        .fold(0.0_f64, f64::max);
    (ebu.loudness_global().unwrap(), 20.0 * peak.log10())
}

fn analyze(path: &Path) -> AnalysisResult {
    analyze_tracks_core_with_progress_sync(
        vec![AnalyzeRequest {
            id: TrackId("invariant".into()),
            path: path.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )
    .unwrap()
    .remove(0)
}

#[test]
fn short_and_no_target_exports_still_apply_known_peak_protection() {
    let dir = tempfile::tempdir().unwrap();
    for duration_ms in [50, 1000] {
        let path = dir.path().join(format!("short-{duration_ms}.wav"));
        let samples: Vec<f32> = (0..48 * duration_ms)
            .flat_map(|i| {
                let x =
                    (0.85 * (std::f64::consts::TAU * 997.0 * f64::from(i) / 48000.0).cos()) as f32;
                [x, -x * 0.8]
            })
            .collect();
        write(&path, 48_000, &samples);
        for profile in [DeliveryProfile::Custom, DeliveryProfile::StreamingUniversal] {
            for bits in [16, 24, 32] {
                let mut settings = common::default_master_settings();
                settings.delivery_profile = profile;
                settings.advanced.ceiling_dbtp = Some(-3.0);
                settings.advanced.bit_depth = Some(bits);
                settings.output_gain_db = 12.0;
                let landing = preview_landing(&samples, 48_000, 2, &settings).unwrap();
                assert!(landing.gain_lin < 1.0, "short/no-target peak bypass");
                if duration_ms == 50 {
                    assert!(!landing.mastered_lufs.is_finite());
                }
                let job = mastering_render(
                    TrackId("peak-bypass".into()),
                    &path,
                    &settings,
                    dir.path(),
                    RenderKind::Master,
                )
                .unwrap();
                let output = decode::decode_full(Path::new(&job.output_paths[0])).unwrap();
                assert_eq!(output.samples.len(), samples.len());
                let (lufs, peak) = measure(&output.samples, 48_000, 2);
                // B1 uses the existing estimator. Independent estimator and
                // delivered-PCM numeric qualification are B2/B3, separately.
                assert!(
                    peak <= f64::from(settings.effective_ceiling_dbtp()) + 0.002,
                    "B1 measured peak {peak}, bits {bits}"
                );
                assert!(output.samples.iter().all(|x| x.abs() < 1.0));
                let receipt = job.measurements.unwrap();
                assert!((f64::from(receipt.true_peak_dbtp) - peak).abs() < 0.002);
                if lufs.is_finite() {
                    assert!((f64::from(receipt.lufs_integrated) - lufs).abs() < 0.002);
                }
            }
        }
    }
}

#[test]
fn hot_start_cd_export_retains_exact_frame_count_and_tail() {
    let dir = tempfile::tempdir().unwrap();
    for rate in [48_000, 96_000] {
        let path = dir.path().join(format!("hot-start-{rate}.wav"));
        let samples: Vec<f32> = (0..rate * 2 + 1)
            .flat_map(|i| {
                let x = (0.3
                    * (std::f64::consts::TAU * 100.0 * f64::from(i) / f64::from(rate)).cos())
                    as f32;
                [x, 0.0]
            })
            .collect();
        write(&path, rate, &samples);
        let mut settings = common::default_master_settings();
        settings.delivery_profile = DeliveryProfile::Cd;
        settings.advanced.adaptive_strength = Some(0.0);
        settings.advanced.width = Some(1.0);
        let job = mastering_render(
            TrackId("cd-src".into()),
            &path,
            &settings,
            dir.path(),
            RenderKind::Master,
        )
        .unwrap();
        let out = decode::decode_full(Path::new(&job.output_paths[0])).unwrap();
        assert_eq!(out.sample_rate, 44_100);
        assert_eq!(out.samples.len(), 88_201 * 2);
        assert!(out.samples[..20].iter().any(|x| x.abs() > 0.01));
        assert!(out.samples[out.samples.len() - 20..]
            .iter()
            .any(|x| x.abs() > 0.01));
        // Stereo-linked dynamics cannot invent right-channel programme.
        assert!(out
            .samples
            .chunks_exact(2)
            .all(|f| f[1].abs() <= 1.0 / 32768.0));
    }
}

#[test]
fn preview_landing_covers_loud_sections_outside_the_middle_window() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("breakdown.wav");
    let sr = 44100;
    let mut samples = sine(sr, 24, 1000.0, 0.4);
    for frame in samples
        .chunks_mut(2)
        .skip(8 * sr as usize)
        .take(8 * sr as usize)
    {
        for sample in frame {
            *sample *= 0.1;
        }
    }
    write(&path, sr, &samples);
    let mut settings = common::default_master_settings();
    settings.delivery_profile = DeliveryProfile::StreamingUniversal;
    settings.advanced.source_profile = SourceProfile::from_analysis(&analyze(&path));
    let landing = preview_landing(&samples, sr, 2, &settings).unwrap();
    let mut chain = MasteringChain::new(sr, 2, &settings);
    chain.coeffs.export_landing_gain_lin = landing.gain_lin;
    chain.process_interleaved(&mut samples, 2);
    chain.flush_render_tail(&mut samples, 2);
    let preview = measure(&samples, sr, 2);
    assert!(
        preview.1 <= -0.95,
        "preview exceeds -1 dBTP ceiling: {}",
        preview.1
    );
    let job = mastering_render(
        TrackId("breakdown".into()),
        &path,
        &settings,
        dir.path(),
        RenderKind::Master,
    )
    .unwrap();
    let output = decode::decode_full(Path::new(&job.output_paths[0])).unwrap();
    let exported = measure(&output.samples, output.sample_rate, 2);
    assert!(
        (preview.0 - exported.0).abs() < 0.2,
        "preview {} versus export {} LUFS",
        preview.0,
        exported.0
    );
}

#[test]
fn compressor_output_is_monotonic_across_the_soft_knee() {
    for ratio in [2.0, 4.0, 10.0] {
        let mut previous = f32::NEG_INFINITY;
        for step in 0..=24 {
            let input_db = -24.0 + step as f32 * 0.5;
            let mut settings = common::default_master_settings();
            settings.preset = Preset::Custom {
                id: "neutral".into(),
            };
            settings.input_gain_db = -1.5;
            settings.advanced.compression_mode = CompressionMode::Manual;
            settings.advanced.compression_low_threshold_db = Some(-20.0);
            settings.advanced.compression_low_ratio = Some(ratio);
            settings.advanced.compression_low_attack_ms = Some(1.0);
            settings.advanced.compression_low_release_ms = Some(50.0);
            settings.advanced.compression_mid_ratio = Some(1.0);
            settings.advanced.compression_high_ratio = Some(1.0);
            let mut chain = MasteringChain::new(44100, 1, &settings);
            let mut output = 0.0;
            for i in 0..22050 {
                let mut frame = [10.0_f32.powf(input_db / 20.0)];
                chain.process_frame_inplace(&mut frame);
                if i >= 21050 {
                    output += frame[0] / 1000.0;
                }
            }
            assert!(
                output >= previous - 1e-6,
                "ratio {ratio}, input {input_db}: {output} < {previous}"
            );
            previous = output;
        }
    }
}

#[test]
fn spectral_balance_is_sample_rate_invariant() {
    let dir = tempfile::tempdir().unwrap();
    let mut highs = Vec::new();
    for sr in [44100, 48000, 96000] {
        let path = dir.path().join(format!("{sr}.wav"));
        write(&path, sr, &sine(sr, 2, 4000.0, 0.25));
        highs.push(analyze(&path).spectral_balance.high);
    }
    for high in &highs {
        assert!(
            (high - highs[0]).abs() < 0.025,
            "same spectrum at different rates: {highs:?}"
        );
    }
}

#[test]
fn adaptive_spectrum_preserves_stereo_side_energy() {
    let dir = tempfile::tempdir().unwrap();
    for polarity in [1.0, -1.0] {
        let samples: Vec<f32> = (0..88200)
            .flat_map(|i| {
                let t = i as f64 / 44100.0;
                let low = 0.08 * (std::f64::consts::TAU * 200.0 * t).sin();
                let high = 0.4 * (std::f64::consts::TAU * 8000.0 * t).sin();
                [(low + high) as f32, (low + polarity * high) as f32]
            })
            .collect();
        let path = dir.path().join(format!("side-{polarity}.wav"));
        write(&path, 44100, &samples);
        let spectrum = analyze(&path).spectral_balance_6band.unwrap();
        // Power ratio is 0.4² / (0.4² + 0.08²), independent of channel polarity.
        assert!(
            (spectrum.air - 0.961538).abs() < 0.002,
            "side energy disappeared: {spectrum:?}"
        );
    }
}

#[test]
fn no_target_float_export_uses_scalar_protection_and_receipt_matches_saved_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("float.wav");
    write(&path, 44100, &sine(44100, 2, 1000.0, 0.5));
    let mut settings = common::default_master_settings();
    settings.preset = Preset::Custom {
        id: "neutral".into(),
    };
    settings.input_gain_db = -1.5;
    settings.output_gain_db = 12.0;
    settings.advanced.compression_mode = CompressionMode::Off;
    settings.advanced.bit_depth = Some(32);
    let job = mastering_render(
        TrackId("float".into()),
        &path,
        &settings,
        dir.path(),
        RenderKind::Master,
    )
    .unwrap();
    let output = decode::decode_full(Path::new(&job.output_paths[0])).unwrap();
    // B1 intentionally removes the no-target bypass, including float exports.
    // Verify uniform scaling of the above-full-scale chain, not sample clipping.
    // Direct writer float-headroom regressions remain in wav_writer.rs.
    let mut raw = sine(44100, 2, 1000.0, 0.5);
    let mut chain = MasteringChain::new(44100, 2, &settings);
    chain.process_interleaved(&mut raw, 2);
    chain.flush_render_tail(&mut raw, 2);
    assert!(raw.iter().any(|x| x.abs() > 1.5));
    let (index, peak) = raw
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.abs().total_cmp(&b.abs()))
        .unwrap();
    let gain = output.samples[index] / peak;
    assert!(gain > 0.0 && gain < 1.0);
    assert!(raw
        .iter()
        .zip(&output.samples)
        .all(|(before, after)| (*before * gain - after).abs() < 2e-7));
    let actual = measure(&output.samples, output.sample_rate, 2);
    let receipt = job.measurements.unwrap();
    assert!(actual.1 <= f64::from(settings.effective_ceiling_dbtp()) + 0.002);
    assert!((actual.0 - receipt.lufs_integrated as f64).abs() < 0.02);
    assert!((actual.1 - receipt.true_peak_dbtp as f64).abs() < 0.02);
}

#[test]
fn limiter_bounds_quarter_rate_intersample_peak() {
    let sr = 44100;
    let mut limiter = Limiter::new(sr, 2, -1.0, 3.0, 50.0);
    let mut samples: Vec<f32> = (0..sr)
        .flat_map(|i| {
            let x =
                (std::f64::consts::FRAC_PI_4 + std::f64::consts::FRAC_PI_2 * i as f64).sin() as f32;
            [x, x]
        })
        .collect();
    for frame in samples.chunks_mut(2) {
        limiter.process_frame_inplace(frame);
    }
    let peak = measure(&samples, sr, 2).1;
    assert!(peak <= -0.95, "limiter missed true peak: {peak} dBTP");
}

#[test]
fn integer_export_receipt_measures_the_quantized_delivered_signal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("integer.wav");
    write(&path, 44100, &sine(44100, 2, 1000.0, 0.5));
    for bits in [16, 24] {
        let mut settings = common::default_master_settings();
        settings.preset = Preset::Custom {
            id: "neutral".into(),
        };
        settings.input_gain_db = -1.5;
        settings.output_gain_db = 12.0;
        settings.advanced.compression_mode = CompressionMode::Off;
        settings.advanced.bit_depth = Some(bits);
        let job = mastering_render(
            TrackId(format!("int-{bits}")),
            &path,
            &settings,
            dir.path(),
            RenderKind::Master,
        )
        .unwrap();
        let decoded = decode::decode_full(Path::new(&job.output_paths[0])).unwrap();
        let actual = measure(&decoded.samples, decoded.sample_rate, 2);
        let reported = job.measurements.unwrap();
        assert!(
            (actual.0 - reported.lufs_integrated as f64).abs() < 0.02,
            "{bits}-bit LUFS: saved {}, reported {}",
            actual.0,
            reported.lufs_integrated
        );
        assert!(
            (actual.1 - reported.true_peak_dbtp as f64).abs() < 0.02,
            "{bits}-bit true peak mismatch"
        );
    }
}

#[test]
fn limiter_peak_bound_survives_rate_phase_and_burst_changes() {
    for sr in [44100, 48000, 96000, 192000] {
        for fraction in [0.013, 0.125, 0.25, 0.43] {
            for phase in [0.0, 0.37, std::f64::consts::FRAC_PI_4] {
                let mut limiter = Limiter::new(sr, 2, -1.0, 3.0, 50.0);
                let mut samples: Vec<f32> = (0..sr / 2)
                    .flat_map(|i| {
                        let burst = if (i / (sr / 100)) % 5 == 2 { 3.0 } else { 0.2 };
                        let x = (std::f64::consts::TAU * fraction * i as f64 + phase).sin() as f32;
                        [burst * x, -0.7 * burst * x]
                    })
                    .collect();
                // Include the delayed tail and both hard burst edges.
                samples.resize(samples.len() + sr as usize / 50, 0.0);
                for frame in samples.chunks_mut(2) {
                    limiter.process_frame_inplace(frame);
                }
                let peak = measure(&samples, sr, 2).1;
                assert!(
                    peak <= -0.95,
                    "{sr} Hz, frequency fraction {fraction}, phase {phase}: {peak} dBTP"
                );
            }
        }
    }
}

#[test]
fn limiter_uses_lookahead_to_avoid_a_gain_step_on_a_future_peak() {
    let mut limiter = Limiter::new(44100, 1, -1.0, 3.0, 50.0);
    for _ in 0..1000 {
        limiter.process_frame_inplace(&mut [0.2]);
    }
    let mut frame = [4.0];
    limiter.process_frame_inplace(&mut frame);
    let jump = 20.0 * (frame[0] / 0.2).log10();
    assert!(
        jump > -0.5,
        "lookahead starts with an abrupt {jump} dB gain step"
    );
}
