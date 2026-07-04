//! Workstream E — Preset fingerprint harness (hardening plan, owner fear #1).
//!
//! Turns "do the presets have enough character while staying safe" into
//! numbers. Every factory preset is rendered (at default intensity 0.5,
//! through the same `process_interleaved` + `flush_render_tail` path the
//! export uses) against a deterministic synthetic fixture set:
//!
//!   * `pink bed`   — Kellet pink noise, dual mono, −12 dBFS peak.
//!                    Probes the 6-band tonal tilt (volume-matched).
//!   * `drum loop`  — synthesized kick/snare/hat pattern, −6 dBFS peak.
//!                    Probes density: crest (DR proxy) and PSR.
//!   * `tonal pad`  — detuned sustained chord with real side content.
//!                    Probes the chain's stereo-width contribution.
//!   * `sine probe` — pure 1 kHz tone. Probes the saturation THD proxy.
//!
//! The 6 bands reuse the app's own analysis band edges
//! (`analysis.rs`: 20/80/250/800/2500/6500/Nyquist) so the numbers here
//! speak the same language as the in-app analysis display.
//!
//! Tests in this file:
//!   * safety bounds — no preset may exceed sane tilt/density/width/THD
//!     caps at default intensity (the "never ruin a track" floor);
//!   * pairwise character distance — all presets stay mutually distinct
//!     (min distance floor across every pair, not just hand-picked ones);
//!   * tolerance golden — the full fingerprint table is pinned within
//!     small per-metric tolerances (OS/arch independent, Batch-F pattern)
//!     so any retune shows up as an explicit, reviewable diff:
//!     `YES_MASTER_UPDATE_GOLDEN=1 cargo test --test preset_fingerprint`.
//!
//! Diagnostics (run with `--ignored --nocapture`):
//!   * `dump_fingerprint_table` — prints the table + distance matrix;
//!   * `write_owner_fingerprint_report` — writes the owner-readable
//!     MD/CSV report to `test-output/preset-fingerprints/` (git-ignored)
//!     for Wave-10 listening sittings.

use std::sync::OnceLock;
use yes_master_lib::dsp::MasteringChain;
use yes_master_lib::engine::measure_integrated_lufs;
use yes_master_lib::types::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};

const SR_HZ: u32 = 48_000;
const STEREO: usize = 2;
const TEST_INTENSITY: f32 = 0.5;

const PINK_SECONDS: f32 = 4.0;
const DRUM_SECONDS: f32 = 6.0;
const PAD_SECONDS: f32 = 4.0;
const SINE_SECONDS: f32 = 2.0;

const PINK_PEAK: f32 = 0.251; // −12 dBFS
const DRUM_PEAK: f32 = 0.501; // −6 dBFS
const PAD_PEAK: f32 = 0.251;
const SINE_PEAK: f32 = 0.251;

/// Harmonic energy below this (relative to the fundamental) is numerical
/// floor, not saturation — clamp so cross-platform noise in the deep
/// floor can't jitter distances or goldens.
const THD_FLOOR_DB: f32 = -80.0;

// ---------------------------------------------------------------------------
// Fixtures (deterministic — same LCG family as the other preset contracts)
// ---------------------------------------------------------------------------

struct Lcg(u32);

impl Lcg {
    fn next_white(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (((self.0 >> 16) & 0x7FFF) as f32 / 32_768.0) - 0.5
    }
}

fn normalize_peak(interleaved: &mut [f32], target_peak: f32) {
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
fn synth_pink_stereo(samples_per_channel: usize, target_peak: f32) -> Vec<f32> {
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
fn synth_drum_loop_stereo(samples_per_channel: usize, target_peak: f32) -> Vec<f32> {
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
fn synth_tonal_pad_stereo(samples_per_channel: usize, target_peak: f32) -> Vec<f32> {
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

fn synth_sine_stereo(samples_per_channel: usize, freq_hz: f32, target_peak: f32) -> Vec<f32> {
    let sr = SR_HZ as f32;
    let mut out = Vec::with_capacity(samples_per_channel * STEREO);
    for n in 0..samples_per_channel {
        let s = (2.0 * std::f32::consts::PI * freq_hz * n as f32 / sr).sin() * target_peak;
        out.push(s);
        out.push(s);
    }
    out
}

struct Fixtures {
    pink: Vec<f32>,
    drums: Vec<f32>,
    pad: Vec<f32>,
    sine: Vec<f32>,
}

fn fixtures() -> &'static Fixtures {
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

fn default_settings_for(preset: Preset) -> MasteringSettings {
    MasteringSettings {
        preset,
        intensity: TEST_INTENSITY,
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

fn master(input: &[f32], preset: Preset) -> Vec<f32> {
    let settings = default_settings_for(preset);
    let mut chain = MasteringChain::new(SR_HZ, STEREO, &settings);
    let mut buf = input.to_vec();
    chain.process_interleaved(&mut buf, STEREO);
    chain.flush_render_tail(&mut buf, STEREO);
    buf
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

fn goertzel_mag_db(samples: &[f32], freq_hz: f32) -> f32 {
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
const BANDS: [(&str, [f32; 3]); 6] = [
    ("sub", [30.0, 45.0, 65.0]),
    ("low", [100.0, 150.0, 220.0]),
    ("low-mid", [320.0, 480.0, 700.0]),
    ("mid", [1_000.0, 1_500.0, 2_200.0]),
    ("high-mid", [3_000.0, 4_200.0, 5_800.0]),
    ("air", [8_000.0, 11_000.0, 15_000.0]),
];

fn band_mean_db(samples_mono: &[f32], taps: &[f32; 3]) -> f32 {
    taps.iter()
        .map(|&f| goertzel_mag_db(samples_mono, f))
        .sum::<f32>()
        / taps.len() as f32
}

fn left_channel(interleaved: &[f32]) -> Vec<f32> {
    interleaved.iter().step_by(STEREO).copied().collect()
}

fn sample_peak_dbfs(interleaved: &[f32]) -> f32 {
    let peak = interleaved
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f32, f32::max)
        .max(f32::MIN_POSITIVE);
    20.0 * peak.log10()
}

fn integrated_lufs(interleaved: &[f32]) -> f32 {
    measure_integrated_lufs(interleaved, SR_HZ, STEREO as u16)
        .expect("integrated LUFS on multi-second stereo fixture")
}

/// Max short-term (3 s window) loudness, fed in 100 ms hops the way a
/// meter would see it. Peak − this = PSR.
fn max_short_term_lufs(interleaved: &[f32]) -> f32 {
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
fn side_mid_db(interleaved: &[f32]) -> f32 {
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
fn thd_proxy_db(output_mono: &[f32]) -> f32 {
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
struct Fingerprint {
    /// Volume-matched chain tilt per app band on the pink bed, in dB:
    /// (out − in band energy) − (out − in broadband LUFS). Zero means
    /// "the chain left this band's balance alone".
    band_tilt_db: [f32; 6],
    /// Where the chain lands the pink bed / drum loop with no delivery
    /// target (Custom profile): the preset's inherent loudness push.
    landed_lufs_pink: f32,
    landed_lufs_drums: f32,
    /// Crest (sample peak − integrated LUFS) on the drum loop — DR proxy.
    crest_db_drums: f32,
    /// PSR (sample peak − max short-term LUFS) on the drum loop.
    psr_db_drums: f32,
    /// Chain's stereo-width contribution on the pad, in dB of side/mid
    /// balance change (positive = wider than the source).
    width_delta_db: f32,
    /// Harmonics 2–5 vs fundamental on the 1 kHz probe (≤ 0 dB,
    /// clamped at −80): the "flavor vs distortion" meter.
    thd_proxy_db: f32,
}

fn compute_fingerprint(preset: Preset, fx: &Fixtures) -> Fingerprint {
    let pink_out = master(&fx.pink, preset.clone());
    let drums_out = master(&fx.drums, preset.clone());
    let pad_out = master(&fx.pad, preset.clone());
    let sine_out = master(&fx.sine, preset);

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

const FACTORY_PRESETS: [(&str, Preset); 8] = [
    ("Universal", Preset::Universal),
    ("Clarity", Preset::Clarity),
    ("Tape", Preset::Tape),
    ("Spatial", Preset::Spatial),
    ("Oomph", Preset::Oomph),
    ("Warmth", Preset::Warmth),
    ("Punch", Preset::Punch),
    ("Loud", Preset::Loud),
];

/// Full fingerprint table, computed once per test process (the renders
/// are the expensive part; every test shares this).
fn fingerprint_table() -> &'static Vec<(&'static str, Fingerprint)> {
    static TABLE: OnceLock<Vec<(&'static str, Fingerprint)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let fx = fixtures();
        FACTORY_PRESETS
            .iter()
            .map(|(name, preset)| (*name, compute_fingerprint(preset.clone(), fx)))
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
fn character_distance(a: &Fingerprint, b: &Fingerprint) -> f32 {
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
fn distance_pairs() -> Vec<(&'static str, &'static str, f32)> {
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
// Tolerance golden (Batch-F pattern — OS/arch independent)
// ---------------------------------------------------------------------------

/// dB tolerance for every loudness/tilt/width metric. Cross-platform
/// libm rounding on these multi-second aggregates measures far below
/// 0.01 dB; real voicing changes move tenths. 0.25 dB catches any
/// deliberate retune while never firing on platform noise.
const GOLDEN_DB_TOLERANCE: f32 = 0.25;
/// The THD proxy rides low-level harmonics whose dB values swing more
/// per unit of actual change (and sit near the −80 clamp for clean
/// presets); a real drive change moves it 5+ dB.
const GOLDEN_THD_TOLERANCE: f32 = 2.0;

#[derive(serde::Serialize, serde::Deserialize)]
struct GoldenRow {
    preset: String,
    #[serde(flatten)]
    fingerprint: Fingerprint,
}

fn golden_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/preset_fingerprint.json")
}

#[test]
fn fingerprints_match_tolerance_golden() {
    let table = fingerprint_table();
    let path = golden_path();

    if std::env::var("YES_MASTER_UPDATE_GOLDEN").as_deref() == Ok("1") {
        let rows: Vec<GoldenRow> = table
            .iter()
            .map(|(name, fp)| GoldenRow {
                preset: (*name).to_string(),
                fingerprint: fp.clone(),
            })
            .collect();
        let json = serde_json::to_string_pretty(&rows).expect("serialize golden");
        std::fs::write(&path, json + "\n").expect("write golden");
        return;
    }

    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing fingerprint golden {} ({e}); regenerate with \
             YES_MASTER_UPDATE_GOLDEN=1 cargo test --test preset_fingerprint and commit it",
            path.display(),
        )
    });
    let golden: Vec<GoldenRow> = serde_json::from_str(&json).expect("parse fingerprint golden");
    assert_eq!(
        golden.len(),
        table.len(),
        "preset count changed; regenerate the golden deliberately",
    );

    let mut drifts = Vec::new();
    for (row, (name, fp)) in golden.iter().zip(table.iter()) {
        assert_eq!(
            row.preset, *name,
            "preset order changed; regenerate the golden deliberately",
        );
        let g = &row.fingerprint;
        let mut check = |metric: &str, got: f32, want: f32, tol: f32| {
            if (got - want).abs() > tol || !(got - want).is_finite() {
                drifts.push(format!(
                    "  {name}: {metric} drifted {want:+.3} -> {got:+.3} (tolerance {tol})"
                ));
            }
        };
        for (i, (band, _)) in BANDS.iter().enumerate() {
            check(
                &format!("band_tilt[{band}]"),
                fp.band_tilt_db[i],
                g.band_tilt_db[i],
                GOLDEN_DB_TOLERANCE,
            );
        }
        check(
            "landed_lufs_pink",
            fp.landed_lufs_pink,
            g.landed_lufs_pink,
            GOLDEN_DB_TOLERANCE,
        );
        check(
            "landed_lufs_drums",
            fp.landed_lufs_drums,
            g.landed_lufs_drums,
            GOLDEN_DB_TOLERANCE,
        );
        check(
            "crest_db_drums",
            fp.crest_db_drums,
            g.crest_db_drums,
            GOLDEN_DB_TOLERANCE,
        );
        check(
            "psr_db_drums",
            fp.psr_db_drums,
            g.psr_db_drums,
            GOLDEN_DB_TOLERANCE,
        );
        check(
            "width_delta_db",
            fp.width_delta_db,
            g.width_delta_db,
            GOLDEN_DB_TOLERANCE,
        );
        check(
            "thd_proxy_db",
            fp.thd_proxy_db,
            g.thd_proxy_db,
            GOLDEN_THD_TOLERANCE,
        );
    }
    assert!(
        drifts.is_empty(),
        "preset voicing drifted from the pinned fingerprint golden:\n{}\n\
         If this retune is deliberate (owner-approved), regenerate with \
         YES_MASTER_UPDATE_GOLDEN=1 cargo test --test preset_fingerprint, \
         commit the JSON diff, and add a Spot-Listen Queue entry.",
        drifts.join("\n"),
    );
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

#[test]
#[ignore = "diagnostic dump — run with --ignored --nocapture"]
fn dump_fingerprint_table() {
    println!(
        "{:>9} |   sub    low  lowmid    mid  himid    air | LUFSp LUFSd  crest    psr | width    thd",
        "preset"
    );
    for (name, fp) in fingerprint_table() {
        let b = fp.band_tilt_db;
        println!(
            "{name:>9} | {:+5.2} {:+5.2} {:+6.2} {:+5.2} {:+5.2} {:+6.2} | {:+5.1} {:+5.1} {:+6.2} {:+6.2} | {:+5.2} {:+6.1}",
            b[0], b[1], b[2], b[3], b[4], b[5],
            fp.landed_lufs_pink, fp.landed_lufs_drums,
            fp.crest_db_drums, fp.psr_db_drums,
            fp.width_delta_db, fp.thd_proxy_db,
        );
    }

    let mut pairs = distance_pairs();
    pairs.sort_by(|a, b| a.2.total_cmp(&b.2));
    println!("\npairwise character distances (closest first):");
    for (a, b, d) in &pairs {
        println!("  {a:>9} <-> {b:<9}  {d:6.2}");
    }
}

// ---------------------------------------------------------------------------
// Safety bounds — "users may overcook their own track, but a factory
// preset at default intensity must never do it for them."
// ---------------------------------------------------------------------------

/// No factory preset may re-balance any analysis band by more than this
/// at default intensity (a mastering move, not a remix).
const MAX_ABS_BAND_TILT_DB: f32 = 6.0;
/// A drum loop must keep at least this much crest after mastering —
/// below this the transients are being flattened into a brick.
const MIN_DRUM_CREST_DB: f32 = 5.0;
const MIN_DRUM_PSR_DB: f32 = 4.0;
/// Chain's inherent loudness push must land pink noise inside a sane
/// mastering window at intensity 0.5 (no delivery target engaged).
/// Hottest observed voicing: Loud at −6.7 LUFS (accepted 85% lean).
const LANDED_LUFS_RANGE: (f32, f32) = (-20.0, -5.0);
/// Width changes stay bounded: never collapse the image, never blow it
/// out (mono-compatibility guard). Sized around the accepted voicing —
/// Spatial contributes +3.22 dB, Oomph narrows −1.62 dB — with margin
/// for drift but far from a runaway widener or mono-collapse.
const WIDTH_DELTA_RANGE_DB: (f32, f32) = (-2.5, 4.0);
/// Saturation stays flavor: harmonic energy at least 20 dB below the
/// fundamental for every preset.
const MAX_THD_PROXY_DB: f32 = -20.0;

#[test]
fn every_preset_stays_within_safety_bounds() {
    for (name, fp) in fingerprint_table() {
        for (i, (band, _)) in BANDS.iter().enumerate() {
            assert!(
                fp.band_tilt_db[i].abs() <= MAX_ABS_BAND_TILT_DB,
                "{name}: {band} tilt {:+.2} dB exceeds ±{MAX_ABS_BAND_TILT_DB} dB safety cap",
                fp.band_tilt_db[i],
            );
        }
        assert!(
            fp.crest_db_drums >= MIN_DRUM_CREST_DB,
            "{name}: drum crest {:.2} dB below {MIN_DRUM_CREST_DB} dB floor — preset is crushing transients at default intensity",
            fp.crest_db_drums,
        );
        assert!(
            fp.psr_db_drums >= MIN_DRUM_PSR_DB,
            "{name}: drum PSR {:.2} dB below {MIN_DRUM_PSR_DB} dB floor",
            fp.psr_db_drums,
        );
        for (label, lufs) in [
            ("pink", fp.landed_lufs_pink),
            ("drums", fp.landed_lufs_drums),
        ] {
            assert!(
                (LANDED_LUFS_RANGE.0..=LANDED_LUFS_RANGE.1).contains(&lufs),
                "{name}: landed LUFS ({label}) {lufs:+.1} outside sane window {LANDED_LUFS_RANGE:?}",
            );
        }
        assert!(
            (WIDTH_DELTA_RANGE_DB.0..=WIDTH_DELTA_RANGE_DB.1).contains(&fp.width_delta_db),
            "{name}: width delta {:+.2} dB outside {WIDTH_DELTA_RANGE_DB:?}",
            fp.width_delta_db,
        );
        assert!(
            fp.thd_proxy_db <= MAX_THD_PROXY_DB,
            "{name}: THD proxy {:+.1} dB above {MAX_THD_PROXY_DB} dB — saturation crossed from flavor into distortion",
            fp.thd_proxy_db,
        );
    }
}

// ---------------------------------------------------------------------------
// Distinctness — presets must be creative directions, not tonal cousins
// ---------------------------------------------------------------------------

/// Minimum character distance between ANY two factory presets. Observed
/// closest pairs at the accepted 85%-lean voicing: Universal↔Clarity
/// 1.80, Punch↔Loud 2.06, Universal↔Tape 2.25 (all other pairs ≥ 2.5).
/// The floor sits at ~55% of the closest pair — it fires when a retune
/// collapses two presets toward each other, not on small drift.
const MIN_PAIRWISE_CHARACTER_DISTANCE: f32 = 1.0;

#[test]
fn all_preset_pairs_stay_mutually_distinct() {
    for (a, b, d) in distance_pairs() {
        assert!(
            d >= MIN_PAIRWISE_CHARACTER_DISTANCE,
            "{a} and {b} have character distance {d:.2} — below the \
             {MIN_PAIRWISE_CHARACTER_DISTANCE} floor. Two presets have collapsed \
             into tonal cousins; run dump_fingerprint_table to see which \
             dimensions merged.",
        );
    }
}
