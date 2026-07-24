//! Fingerprint kit for the adaptive preset characterization (U13).
//!
//! This is a deliberate, guarded COPY of the fixture generators, metrics, and
//! character-distance function in `tests/preset_fingerprint.rs`.
//!
//! Why copy rather than refactor: `preset_fingerprint.rs` owns a committed
//! tolerance golden (`tests/golden/preset_fingerprint.json`) that gates every
//! preset retune. Rewiring that file to import from here would put the repo's
//! most safety-critical pinned artifact on the end of a refactor, for the sake
//! of a test-only helper. Not a trade worth making.
//!
//! The duplication is prevented from drifting mechanically:
//! `preset_adaptive_characterization.rs::kit_reproduces_the_committed_golden`
//! asserts this kit reproduces the committed golden within its own tolerances.
//! If either copy moves, that test fails.
//!
//! Lives in its own `tests/fingerprint_kit/` directory rather than in the
//! pre-existing `tests/common/` module, which is shared by six other test
//! files and has nothing to do with fingerprints.
//!
//! What this kit ADDS over the original: `master_with_profile` /
//! `compute_fingerprint_with_profile`, which inject a `SourceProfile` so the
//! ADAPTIVE chain can be characterized. The original only ever renders the
//! non-adaptive path.

#![allow(dead_code)]

use std::sync::OnceLock;
use yes_master_lib::dsp::MasteringChain;
use yes_master_lib::engine::measure_integrated_lufs;
use yes_master_lib::types::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};

pub(crate) const SR_HZ: u32 = 48_000;
pub(crate) const STEREO: usize = 2;
pub(crate) const TEST_INTENSITY: f32 = 0.5;

pub(crate) const PINK_SECONDS: f32 = 4.0;
pub(crate) const DRUM_SECONDS: f32 = 6.0;
pub(crate) const PAD_SECONDS: f32 = 4.0;
pub(crate) const SINE_SECONDS: f32 = 2.0;

pub(crate) const PINK_PEAK: f32 = 0.251; // −12 dBFS
pub(crate) const DRUM_PEAK: f32 = 0.501; // −6 dBFS
pub(crate) const PAD_PEAK: f32 = 0.251;
pub(crate) const SINE_PEAK: f32 = 0.251;

/// Harmonic energy below this (relative to the fundamental) is numerical
/// floor, not saturation — clamp so cross-platform noise in the deep
/// floor can't jitter distances or goldens.
pub(crate) const THD_FLOOR_DB: f32 = -80.0;

// ---------------------------------------------------------------------------
// Fixtures (deterministic — same LCG family as the other preset contracts)
// ---------------------------------------------------------------------------

pub(crate) struct Lcg(pub(crate) u32);

impl Lcg {
    fn next_white(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (((self.0 >> 16) & 0x7FFF) as f32 / 32_768.0) - 0.5
    }
}

pub(crate) fn normalize_peak(interleaved: &mut [f32], target_peak: f32) {
    let peak = interleaved
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f32, f32::max)
        .max(f32::MIN_POSITIVE);
    let scale = target_peak / peak;
    for s in interleaved {
        *s *= scale;
    }
}

/// Paul Kellet six-stage pinking IIR — identical generator/seed to
/// `preset_distinctness.rs` so both harnesses probe the same spectrum.
pub(crate) fn synth_pink_stereo(samples_per_channel: usize, target_peak: f32) -> Vec<f32> {
    let mut rng = Lcg(0xCAFE_BABE);
    let (mut b0, mut b1, mut b2, mut b3, mut b4, mut b5);
    b0 = 0.0_f32;
    b1 = 0.0_f32;
    b2 = 0.0_f32;
    b3 = 0.0_f32;
    b4 = 0.0_f32;
    b5 = 0.0_f32;

    let mut out = Vec::with_capacity(samples_per_channel * STEREO);
    for _ in 0..samples_per_channel {
        let w = rng.next_white();
        b0 = 0.99886 * b0 + w * 0.0555179;
        b1 = 0.99332 * b1 + w * 0.0750759;
        b2 = 0.96900 * b2 + w * 0.153_852;
        b3 = 0.86650 * b3 + w * 0.3104856;
        b4 = 0.55000 * b4 + w * 0.5329522;
        b5 = -0.7616 * b5 - w * 0.0168980;
        let p = b0 + b1 + b2 + b3 + b4 + b5 + w * 0.5362 + w * 0.115926;
        out.push(p);
        out.push(p);
    }
    normalize_peak(&mut out, target_peak);
    out
}

/// Drum-ish transient loop: kick on the beat (pitch-dropping sine),
/// snare on 2 and 4 (enveloped noise + 200 Hz body), closed hats every
/// eighth (short first-difference noise ticks, panned slightly right;
/// snare slightly left). 120 BPM. Everything is synthesized from the
/// deterministic LCG — no audio files, nothing private.
pub(crate) fn synth_drum_loop_stereo(samples_per_channel: usize, target_peak: f32) -> Vec<f32> {
    let sr = SR_HZ as f32;
    let beat = 0.5_f32; // 120 BPM
    let mut rng = Lcg(0xDEAD_BEA7);

    let mut left = vec![0.0_f32; samples_per_channel];
    let mut right = vec![0.0_f32; samples_per_channel];

    let add = |start_sec: f32,
               samples: &[f32],
               pan_l: f32,
               pan_r: f32,
               left: &mut [f32],
               right: &mut [f32]| {
        let start = (start_sec * sr) as usize;
        for (i, &s) in samples.iter().enumerate() {
            let idx = start + i;
            if idx >= left.len() {
                break;
            }
            left[idx] += s * pan_l;
            right[idx] += s * pan_r;
        }
    };

    // One-shot generators -------------------------------------------------
    let kick: Vec<f32> = {
        let dur = (0.30 * sr) as usize;
        let mut phase = 0.0_f32;
        (0..dur)
            .map(|n| {
                let t = n as f32 / sr;
                let freq = 45.0 + 55.0 * (-t / 0.03).exp();
                phase += 2.0 * std::f32::consts::PI * freq / sr;
                (phase.sin()) * (-t / 0.12).exp()
            })
            .collect()
    };
    let snare: Vec<f32> = {
        let dur = (0.20 * sr) as usize;
        (0..dur)
            .map(|n| {
                let t = n as f32 / sr;
                let noise = rng.next_white() * 1.6;
                let body = 0.5 * (2.0 * std::f32::consts::PI * 200.0 * t).sin();
                (noise + body) * (-t / 0.08).exp() * 0.8
            })
            .collect()
    };
    let hat: Vec<f32> = {
        let dur = (0.06 * sr) as usize;
        let mut prev = 0.0_f32;
        (0..dur)
            .map(|n| {
                let t = n as f32 / sr;
                let w = rng.next_white();
                // First difference ≈ crude high-pass: keeps hats out of
                // the kick/snare bands.
                let hp = w - prev;
                prev = w;
                hp * (-t / 0.025).exp() * 0.5
            })
            .collect()
    };

    let total_sec = samples_per_channel as f32 / sr;
    let mut beat_idx = 0;
    loop {
        let t0 = beat_idx as f32 * beat;
        if t0 >= total_sec {
            break;
        }
        if beat_idx % 2 == 0 {
            add(t0, &kick, 1.0, 1.0, &mut left, &mut right);
        } else {
            add(t0, &snare, 1.0, 0.7, &mut left, &mut right);
        }
        add(t0, &hat, 0.6, 1.0, &mut left, &mut right);
        add(t0 + beat * 0.5, &hat, 0.6, 1.0, &mut left, &mut right);
        beat_idx += 1;
    }

    let mut out = Vec::with_capacity(samples_per_channel * STEREO);
    for i in 0..samples_per_channel {
        out.push(left[i]);
        out.push(right[i]);
    }
    normalize_peak(&mut out, target_peak);
    out
}

/// Sustained A-major-ish pad (110/164.81/220/277.18 Hz) with ±0.15%
/// L/R detune so the fixture carries real side content for the width
/// probe. 0.25 s linear fade-in/out avoids transient clicks.
pub(crate) fn synth_tonal_pad_stereo(samples_per_channel: usize, target_peak: f32) -> Vec<f32> {
    let sr = SR_HZ as f32;
    let freqs = [110.0_f32, 164.81, 220.0, 277.18];
    let detune = 0.0015_f32;
    let fade = (0.25 * sr) as usize;

    let mut out = Vec::with_capacity(samples_per_channel * STEREO);
    for n in 0..samples_per_channel {
        let t = n as f32 / sr;
        let env = if n < fade {
            n as f32 / fade as f32
        } else if n >= samples_per_channel - fade {
            (samples_per_channel - n) as f32 / fade as f32
        } else {
            1.0
        };
        let mut l = 0.0_f32;
        let mut r = 0.0_f32;
        for &f in &freqs {
            l += (2.0 * std::f32::consts::PI * f * (1.0 - detune) * t).sin();
            r += (2.0 * std::f32::consts::PI * f * (1.0 + detune) * t).sin();
        }
        out.push(l * env * 0.25);
        out.push(r * env * 0.25);
    }
    normalize_peak(&mut out, target_peak);
    out
}

pub(crate) fn synth_sine_stereo(
    samples_per_channel: usize,
    freq_hz: f32,
    target_peak: f32,
) -> Vec<f32> {
    let sr = SR_HZ as f32;
    let mut out = Vec::with_capacity(samples_per_channel * STEREO);
    for n in 0..samples_per_channel {
        let s = (2.0 * std::f32::consts::PI * freq_hz * n as f32 / sr).sin() * target_peak;
        out.push(s);
        out.push(s);
    }
    out
}

pub(crate) struct Fixtures {
    pub(crate) pink: Vec<f32>,
    pub(crate) drums: Vec<f32>,
    pub(crate) pad: Vec<f32>,
    pub(crate) sine: Vec<f32>,
}

pub(crate) fn fixtures() -> &'static Fixtures {
    static FIXTURES: OnceLock<Fixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| Fixtures {
        pink: synth_pink_stereo((SR_HZ as f32 * PINK_SECONDS) as usize, PINK_PEAK),
        drums: synth_drum_loop_stereo((SR_HZ as f32 * DRUM_SECONDS) as usize, DRUM_PEAK),
        pad: synth_tonal_pad_stereo((SR_HZ as f32 * PAD_SECONDS) as usize, PAD_PEAK),
        sine: synth_sine_stereo((SR_HZ as f32 * SINE_SECONDS) as usize, 1_000.0, SINE_PEAK),
    })
}

// ---------------------------------------------------------------------------
// Chain plumbing (mirrors the export path: process + flush)
// ---------------------------------------------------------------------------

pub(crate) fn default_settings_for(preset: Preset, intensity: f32) -> MasteringSettings {
    MasteringSettings {
        preset,
        intensity,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

pub(crate) fn master(input: &[f32], preset: Preset, intensity: f32) -> Vec<f32> {
    let settings = default_settings_for(preset, intensity);
    let mut chain = MasteringChain::new(SR_HZ, STEREO, &settings);
    let mut buf = input.to_vec();
    chain.process_interleaved(&mut buf, STEREO);
    chain.flush_render_tail(&mut buf, STEREO);
    buf
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

pub(crate) fn goertzel_mag_db(samples: &[f32], freq_hz: f32) -> f32 {
    let omega = 2.0 * std::f32::consts::PI * freq_hz / SR_HZ as f32;
    let coeff = 2.0 * omega.cos();
    let mut q1 = 0.0_f32;
    let mut q2 = 0.0_f32;
    for &s in samples {
        let q0 = coeff * q1 - q2 + s;
        q2 = q1;
        q1 = q0;
    }
    let mag = (q1 * q1 + q2 * q2 - coeff * q1 * q2).max(1e-30).sqrt();
    20.0 * (mag / samples.len() as f32).log10()
}

/// Band names + Goertzel taps, one row per app analysis band
/// (`analysis.rs` edges: 20/80/250/800/2500/6500/Nyquist).
pub(crate) const BANDS: [(&str, [f32; 3]); 6] = [
    ("sub", [30.0, 45.0, 65.0]),
    ("low", [100.0, 150.0, 220.0]),
    ("low-mid", [320.0, 480.0, 700.0]),
    ("mid", [1_000.0, 1_500.0, 2_200.0]),
    ("high-mid", [3_000.0, 4_200.0, 5_800.0]),
    ("air", [8_000.0, 11_000.0, 15_000.0]),
];

pub(crate) fn band_mean_db(samples_mono: &[f32], taps: &[f32; 3]) -> f32 {
    taps.iter()
        .map(|&f| goertzel_mag_db(samples_mono, f))
        .sum::<f32>()
        / taps.len() as f32
}

pub(crate) fn left_channel(interleaved: &[f32]) -> Vec<f32> {
    interleaved.iter().step_by(STEREO).copied().collect()
}

pub(crate) fn sample_peak_dbfs(interleaved: &[f32]) -> f32 {
    let peak = interleaved
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f32, f32::max)
        .max(f32::MIN_POSITIVE);
    20.0 * peak.log10()
}

pub(crate) fn integrated_lufs(interleaved: &[f32]) -> f32 {
    measure_integrated_lufs(interleaved, SR_HZ, STEREO as u16)
        .expect("integrated LUFS on multi-second stereo fixture")
}

/// Max short-term (3 s window) loudness, fed in 100 ms hops the way a
/// meter would see it. Peak − this = PSR.
pub(crate) fn max_short_term_lufs(interleaved: &[f32]) -> f32 {
    let mut ebu = ebur128::EbuR128::new(STEREO as u32, SR_HZ, ebur128::Mode::S)
        .expect("ebur128 short-term init");
    let hop_frames = (SR_HZ / 10) as usize;
    let mut max_st = f32::NEG_INFINITY;
    for chunk in interleaved.chunks(hop_frames * STEREO) {
        ebu.add_frames_f32(chunk).expect("ebur128 feed");
        if let Ok(st) = ebu.loudness_shortterm() {
            let st = st as f32;
            if st.is_finite() {
                max_st = max_st.max(st);
            }
        }
    }
    assert!(
        max_st.is_finite(),
        "short-term loudness never became finite — fixture too short?"
    );
    max_st
}

/// Side-vs-mid balance in dB: 20·log10(RMS(side)/RMS(mid)).
pub(crate) fn side_mid_db(interleaved: &[f32]) -> f32 {
    let mut mid_sq = 0.0_f64;
    let mut side_sq = 0.0_f64;
    let mut n = 0_usize;
    for frame in interleaved.chunks_exact(STEREO) {
        let mid = (frame[0] + frame[1]) * 0.5;
        let side = (frame[0] - frame[1]) * 0.5;
        mid_sq += (mid as f64) * (mid as f64);
        side_sq += (side as f64) * (side as f64);
        n += 1;
    }
    let mid_rms = (mid_sq / n as f64).sqrt().max(1e-15);
    let side_rms = (side_sq / n as f64).sqrt().max(1e-15);
    (20.0 * (side_rms / mid_rms).log10()) as f32
}

/// Saturation proxy: total energy at harmonics 2–5 of the 1 kHz probe,
/// relative to the fundamental, clamped at `THD_FLOOR_DB`.
pub(crate) fn thd_proxy_db(output_mono: &[f32]) -> f32 {
    let fundamental = goertzel_mag_db(output_mono, 1_000.0);
    let harmonic_energy: f32 = [2_000.0_f32, 3_000.0, 4_000.0, 5_000.0]
        .iter()
        .map(|&f| {
            let db = goertzel_mag_db(output_mono, f);
            10.0_f32.powf(db / 10.0)
        })
        .sum();
    let harmonics_db = 10.0 * harmonic_energy.max(1e-30).log10();
    (harmonics_db - fundamental).max(THD_FLOOR_DB)
}

// ---------------------------------------------------------------------------
// The fingerprint
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Fingerprint {
    /// Volume-matched chain tilt per app band on the pink bed, in dB:
    /// (out − in band energy) − (out − in broadband LUFS). Zero means
    /// "the chain left this band's balance alone". Caveat (adversarial
    /// review, 2026-07-03): the broadband term is K-weighted, so a boost
    /// in a K-suppressed band (sub) reads at nearly full magnitude while
    /// the same boost in a K-heavy band (mid/air) reads partially
    /// absorbed into the LUFS term — the safety cap is effectively
    /// stricter for sub-heavy voicings than air-heavy ones. Fine for
    /// pinning/distance (both terms move deterministically), just don't
    /// read tilts as exact EQ curves across bands.
    pub(crate) band_tilt_db: [f32; 6],
    /// Where the chain lands the pink bed / drum loop with no delivery
    /// target (Custom profile): the preset's inherent loudness push.
    pub(crate) landed_lufs_pink: f32,
    pub(crate) landed_lufs_drums: f32,
    /// Crest (sample peak − integrated LUFS) on the drum loop — DR proxy.
    pub(crate) crest_db_drums: f32,
    /// PSR (sample peak − max short-term LUFS) on the drum loop.
    pub(crate) psr_db_drums: f32,
    /// Chain's stereo-width contribution on the pad, in dB of side/mid
    /// balance change (positive = wider than the source).
    pub(crate) width_delta_db: f32,
    /// Harmonics 2–5 vs fundamental on the 1 kHz probe (≤ 0 dB,
    /// clamped at −80): the "flavor vs distortion" meter.
    pub(crate) thd_proxy_db: f32,
}

pub(crate) fn compute_fingerprint(preset: Preset, fx: &Fixtures, intensity: f32) -> Fingerprint {
    let pink_out = master(&fx.pink, preset.clone(), intensity);
    let drums_out = master(&fx.drums, preset.clone(), intensity);
    let pad_out = master(&fx.pad, preset.clone(), intensity);
    let sine_out = master(&fx.sine, preset, intensity);

    let pink_in_mono = left_channel(&fx.pink);
    let pink_out_mono = left_channel(&pink_out);
    let broadband_gain = integrated_lufs(&pink_out) - integrated_lufs(&fx.pink);
    let mut band_tilt_db = [0.0_f32; 6];
    for (i, (_, taps)) in BANDS.iter().enumerate() {
        let chain_gain = band_mean_db(&pink_out_mono, taps) - band_mean_db(&pink_in_mono, taps);
        band_tilt_db[i] = chain_gain - broadband_gain;
    }

    let drums_lufs = integrated_lufs(&drums_out);
    let drums_peak = sample_peak_dbfs(&drums_out);

    Fingerprint {
        band_tilt_db,
        landed_lufs_pink: integrated_lufs(&pink_out),
        landed_lufs_drums: drums_lufs,
        crest_db_drums: drums_peak - drums_lufs,
        psr_db_drums: drums_peak - max_short_term_lufs(&drums_out),
        width_delta_db: side_mid_db(&pad_out) - side_mid_db(&fx.pad),
        thd_proxy_db: thd_proxy_db(&left_channel(&sine_out)),
    }
}

pub(crate) const FACTORY_PRESETS: [(&str, Preset); 8] = [
    ("Universal", Preset::Universal),
    ("Clarity", Preset::Clarity),
    ("Tape", Preset::Tape),
    ("Spatial", Preset::Spatial),
    ("Oomph", Preset::Oomph),
    ("Warmth", Preset::Warmth),
    ("Punch", Preset::Punch),
    ("Loud", Preset::Loud),
];

/// Full fingerprint table at default intensity 0.5, computed once per
/// test process (the renders are the expensive part; every test shares
/// this). The golden, distance floor, and report all read this table.
pub(crate) fn fingerprint_table() -> &'static Vec<(&'static str, Fingerprint)> {
    static TABLE: OnceLock<Vec<(&'static str, Fingerprint)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let fx = fixtures();
        FACTORY_PRESETS
            .iter()
            .map(|(name, preset)| {
                (
                    *name,
                    compute_fingerprint(preset.clone(), fx, TEST_INTENSITY),
                )
            })
            .collect()
    })
}

/// Fingerprints at the intensity slider's maximum (1.0), where
/// `preset_scale` multiplies every preset move by 1.6× relative to the
/// 0.5 default. Only the max-intensity safety test pays for this table.
pub(crate) fn fingerprint_table_max_intensity() -> &'static Vec<(&'static str, Fingerprint)> {
    static TABLE: OnceLock<Vec<(&'static str, Fingerprint)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let fx = fixtures();
        FACTORY_PRESETS
            .iter()
            .map(|(name, preset)| (*name, compute_fingerprint(preset.clone(), fx, 1.0)))
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Character distance
// ---------------------------------------------------------------------------

/// Perceptual-dB-space L2 distance between two fingerprints. Components:
/// the six band tilts, drum crest, and width contribution enter at full
/// weight (1 dB of difference ≈ 1 unit); the inherent loudness push is
/// half-weighted so Loud/Punch's gain character counts without drowning
/// the tonal dimensions; the THD proxy is quarter-weighted because dB
/// swings on low-level harmonics are large relative to their audibility.
pub(crate) fn character_distance(a: &Fingerprint, b: &Fingerprint) -> f32 {
    let mut sum = 0.0_f32;
    for i in 0..BANDS.len() {
        sum += (a.band_tilt_db[i] - b.band_tilt_db[i]).powi(2);
    }
    sum += (a.crest_db_drums - b.crest_db_drums).powi(2);
    sum += (a.width_delta_db - b.width_delta_db).powi(2);
    sum += (0.5 * (a.landed_lufs_pink - b.landed_lufs_pink)).powi(2);
    sum += (0.25 * (a.thd_proxy_db - b.thd_proxy_db)).powi(2);
    sum.sqrt()
}

/// Every ordered pair (i < j) with its distance, for the floor assertion
/// and the owner report.
pub(crate) fn distance_pairs() -> Vec<(&'static str, &'static str, f32)> {
    let table = fingerprint_table();
    let mut pairs = Vec::new();
    for i in 0..table.len() {
        for j in (i + 1)..table.len() {
            pairs.push((
                table[i].0,
                table[j].0,
                character_distance(&table[i].1, &table[j].1),
            ));
        }
    }
    pairs
}

// ---------------------------------------------------------------------------
// Adaptive-path additions (U13)
//
// The original harness only renders the NON-adaptive chain
// (`AdvancedSettings::default()` carries no source profile). Everything below
// injects a `SourceProfile` so the guardrail/adaptive path is what gets
// measured.
// ---------------------------------------------------------------------------

use yes_master_lib::types::SourceProfile;

pub(crate) fn settings_with_profile(
    preset: Preset,
    intensity: f32,
    profile: Option<SourceProfile>,
) -> MasteringSettings {
    let mut settings = default_settings_for(preset, intensity);
    settings.advanced.source_profile = profile;
    settings
}

pub(crate) fn master_with_profile(
    input: &[f32],
    preset: Preset,
    intensity: f32,
    profile: Option<SourceProfile>,
) -> Vec<f32> {
    let settings = settings_with_profile(preset, intensity, profile);
    let mut chain = MasteringChain::new(SR_HZ, STEREO, &settings);
    let mut buf = input.to_vec();
    chain.process_interleaved(&mut buf, STEREO);
    chain.flush_render_tail(&mut buf, STEREO);
    buf
}

pub(crate) fn compute_fingerprint_with_profile(
    preset: Preset,
    fx: &Fixtures,
    intensity: f32,
    profile: Option<SourceProfile>,
) -> Fingerprint {
    let pink_out = master_with_profile(&fx.pink, preset.clone(), intensity, profile);
    let drums_out = master_with_profile(&fx.drums, preset.clone(), intensity, profile);
    let pad_out = master_with_profile(&fx.pad, preset.clone(), intensity, profile);
    let sine_out = master_with_profile(&fx.sine, preset, intensity, profile);

    let pink_in_mono = left_channel(&fx.pink);
    let pink_out_mono = left_channel(&pink_out);
    let broadband_gain = integrated_lufs(&pink_out) - integrated_lufs(&fx.pink);
    let mut band_tilt_db = [0.0_f32; 6];
    for (i, (_, taps)) in BANDS.iter().enumerate() {
        let chain_gain = band_mean_db(&pink_out_mono, taps) - band_mean_db(&pink_in_mono, taps);
        band_tilt_db[i] = chain_gain - broadband_gain;
    }

    let drums_lufs = integrated_lufs(&drums_out);
    let drums_peak = sample_peak_dbfs(&drums_out);

    Fingerprint {
        band_tilt_db,
        landed_lufs_pink: integrated_lufs(&pink_out),
        landed_lufs_drums: drums_lufs,
        crest_db_drums: drums_peak - drums_lufs,
        psr_db_drums: drums_peak - max_short_term_lufs(&drums_out),
        width_delta_db: side_mid_db(&pad_out) - side_mid_db(&fx.pad),
        thd_proxy_db: thd_proxy_db(&left_channel(&sine_out)),
    }
}
