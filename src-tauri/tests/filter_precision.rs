//! Recursive filter rounding must not amplify tiny source-normalization errors.
use yes_master_lib::{dsp::MasteringChain, types::MasteringSettings};

#[test]
fn normalized_quiet_copies_keep_the_same_processed_response() {
    let settings: MasteringSettings = serde_json::from_value(serde_json::json!({
        "preset":{"kind":"universal"},"intensity":0.75,"volume_match":false,
        "eq_low_db":0.,"eq_mid_db":0.,"eq_high_db":0.,"delivery_profile":"custom",
        "advanced":{"warmth":0.6}
    }))
    .unwrap();
    let mut failures = Vec::new();
    for rate in [44100, 48000, 96000] {
        for channels in [1, 2] {
            let mut random = 19_u32;
            let samples: Vec<f32> = (0..rate * 4)
                .flat_map(|frame| {
                    random = random.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = (random as f64 / u32::MAX as f64 - 0.5) * 0.09;
                    let time = f64::from(frame) / f64::from(rate);
                    let x = ((time * 24. * std::f64::consts::TAU).sin() * 0.65
                        + (time * 997. * std::f64::consts::TAU).sin() * 0.12
                        + noise) as f32;
                    [x, x * -0.7].into_iter().take(channels)
                })
                .collect();
            let normalize = |gain: f32| {
                let peak = samples
                    .iter()
                    .map(|x| f64::from((x * gain).abs()))
                    .fold(0_f64, f64::max);
                samples
                    .iter()
                    .map(|x| (f64::from(x * gain) / peak) as f32)
                    .collect::<Vec<_>>()
            };
            let process = |mut samples: Vec<f32>| {
                let mut chain = MasteringChain::new(rate, channels, &settings);
                chain.process_interleaved(&mut samples, channels);
                chain.flush_render_tail(&mut samples, channels);
                samples
            };
            let original = normalize(1.);
            let reference = process(original.clone());
            for gain in [1e-2, 1e-4] {
                let copy = normalize(gain);
                let maximum = |x: &[f32], y: &[f32]| {
                    x.iter()
                        .zip(y)
                        .map(|(x, y)| (f64::from(*x) - f64::from(*y)).abs())
                        .fold(0_f64, f64::max)
                };
                assert!(
                    maximum(&original, &copy) <= 2e-7,
                    "normalization input bound"
                );
                let actual = process(copy);
                assert!(actual.iter().all(|x| x.is_finite()));
                let delta = maximum(&reference, &actual);
                eprintln!("{rate} Hz, {channels} channels, gain {gain}: max {delta}");
                // Numeric reproducibility budget; not an audibility threshold.
                if delta > 1e-5 {
                    failures.push((rate, channels, gain, delta));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "recursive-state sensitivity: {failures:?}"
    );
}
