//! Auto gain staging research core (auto gain staging plan, Phase A, Slice 1).
//!
//! Offline only. Nothing here is called by the product, and nothing here
//! changes product behavior. It renders songs through the production chain
//! three ways and measures them:
//!
//! - **T, today.** The production chain unchanged: the Standard export.
//! - **S, turn-down first.** Where T overshoots the target and the landing
//!   turns it down, that reduction moves in front of the chain, repeated until
//!   the overshoot is within tolerance. It never raises the level.
//! - **G, two gains.** "Level in" in front of the Style stages, and a "limiter
//!   drive" just before the limiter, with priority Clean.
//!
//! **How the limiter drive is rendered without touching product code.** The
//! chain's stages before the limiter do not depend on the limiter. So a render
//! is split in two:
//!
//! 1. a *stage pass*: the real `MasteringChain` with its limiter swapped for a
//!    pass-through limiter (same lookahead, a ceiling no music reaches), which
//!    yields the exact signal that reaches the limiter;
//! 2. a *limiter pass*: a real `Limiter` built exactly as the chain builds it,
//!    fed that signal (times the drive, when there is one).
//!
//! [`decomposition_matches_chain`] proves the split is bit-identical to the
//! production chain with no drive. The runner checks it on every song and
//! Style before any number counts.
//!
//! The including crate root must declare `use yes_master_lib::types;` and
//! `#[path = ".../src/sample_rate.rs"] mod sample_rate;` (the app's resampler
//! is crate-private; `tests/realtime_meter.rs` includes it the same way).
//!
//! Metric definitions follow September's fixed protocol
//! (`docs/reviews/2026-09-15-drive-prototype-protocol.md`,
//! `scripts/research/mastering-quality-implementation-20260915/drive_metrics.py`).

#![allow(dead_code)]

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::{Deserialize, Serialize};
use yes_master_lib::dsp::{Limiter, MasteringChain};
use yes_master_lib::guardrails::ADAPTIVE_STRENGTH_DEFAULT;
use yes_master_lib::{
    AdvancedSettings, DeliveryProfile, EqBandFrequencies, MasteringSettings, Preset, SourceProfile,
};

// ---------------------------------------------------------------------------
// Parameters (plan "Parameters"; frozen at the plan's start values)
// ---------------------------------------------------------------------------

/// Most the limiter may take off the loudest peak automatically (A5).
pub const CLEAN_LIMIT_DB: f32 = 6.0;
/// Target tolerance when resolving, and "within tolerance" for R1/R2.
pub const TARGET_TOLERANCE_LU: f32 = 0.2;
/// Maximum resolution passes (limiter passes for G, full passes for S).
/// Development runs 1 and 2 used 4; frozen at 6 after Checkpoint A (inside
/// the plan's 2–6 bound) so R4 holds. `--max-passes` overrides it.
pub const MAX_PASSES: usize = 6;
/// Level in is limited to the existing ±24 dB input range (contract rule 2).
pub const LEVEL_IN_RANGE_DB: f32 = 24.0;

/// The Standard loudness steps (Low / Medium / High).
pub const STANDARD_TARGETS: [f32; 3] = [-14.0, -11.0, -9.0];
/// Standard's fixed export recipe (`src/standard-mapping-parity.json`).
pub const STANDARD_CEILING_DBTP: f32 = -1.0;
pub const STANDARD_RATE: u32 = 44_100;
pub const STANDARD_BITS: u16 = 24;

/// The four Standard Styles, in display order.
pub fn standard_styles() -> [(&'static str, Preset); 4] {
    [
        ("Universal", Preset::Universal),
        ("Clarity", Preset::Clarity),
        ("Tape", Preset::Tape),
        ("Oomph", Preset::Oomph),
    ]
}

/// Mirrors `dsp.rs`'s private `LIMITER_LOOKAHEAD_MS` / `LIMITER_RELEASE_MS`.
/// [`decomposition_matches_chain`] fails if they ever drift.
const LIMITER_LOOKAHEAD_MS: f32 = 3.0;
const LIMITER_RELEASE_MS: f32 = 50.0;
/// A ceiling no music reaches: the stage pass's pass-through limiter.
const PASS_THROUGH_CEILING_DBFS: f32 = 200.0;
/// Mirrors the chain's ceiling clamp (`clamp_finite_or(x, -1, -6, 0)`).
pub fn chain_ceiling_db(ceiling_dbtp: f32) -> f32 {
    if ceiling_dbtp.is_finite() {
        ceiling_dbtp.clamp(-6.0, 0.0)
    } else {
        -1.0
    }
}

pub fn db_to_lin(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

pub fn lin_to_db(lin: f32) -> f32 {
    20.0 * lin.max(f32::MIN_POSITIVE).log10()
}

// ---------------------------------------------------------------------------
// Settings: what the Standard export renders with
// ---------------------------------------------------------------------------

/// The settings a Standard export sends for a Style at default Intensity and
/// a Standard loudness: the frontend's defaults (`useTrackMaster.ts`
/// `DEFAULT_SETTINGS`), the backend-resolved profile (B2), and
/// `standardExportSettings` (Custom profile, target, −1 dBTP, 24-bit,
/// 44.1 kHz). Volume Match is off for export. Confidence and compressor guards
/// resolve to none while their gates are off (A15).
pub fn standard_export_settings(
    preset: Preset,
    target_lufs: f32,
    profile: Option<SourceProfile>,
    source_lufs: Option<f32>,
) -> MasteringSettings {
    MasteringSettings {
        preset,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: source_lufs,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings {
            lufs_offset_db: Some(target_lufs),
            ceiling_dbtp: Some(STANDARD_CEILING_DBTP),
            bit_depth: Some(STANDARD_BITS),
            target_sample_rate: Some(STANDARD_RATE),
            adaptive_strength: Some(ADAPTIVE_STRENGTH_DEFAULT),
            source_profile: profile,
            ..Default::default()
        },
    }
}

// ---------------------------------------------------------------------------
// The level-in rule (contract rule 2)
// ---------------------------------------------------------------------------

/// The reference level, anchored to the owner's reference song (A6).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Nominal {
    pub lufs: f32,
    pub peak_dbtp: f32,
}

/// `level_in = min(NOMINAL_LUFS − source loudness, NOMINAL_PEAK − source
/// peak)`, limited to ±24 dB. `None` (inert, A7) without a usable source
/// measurement.
pub fn level_in_db(nominal: Nominal, source_lufs: f32, source_peak_dbtp: f32) -> Option<f32> {
    if !source_lufs.is_finite()
        || source_lufs <= -70.0
        || !source_peak_dbtp.is_finite()
        || !nominal.lufs.is_finite()
        || !nominal.peak_dbtp.is_finite()
    {
        return None;
    }
    let raw = (nominal.lufs - source_lufs).min(nominal.peak_dbtp - source_peak_dbtp);
    Some(raw.clamp(-LEVEL_IN_RANGE_DB, LEVEL_IN_RANGE_DB))
}

// ---------------------------------------------------------------------------
// The render decomposition
// ---------------------------------------------------------------------------

/// The exact signal that reaches the limiter: `frames + lookahead` frames,
/// aligned with the input. The extra `lookahead` frames are the stages'
/// ring-out on silence, which the production render's tail flush also feeds
/// the limiter.
pub struct StagePass {
    pub pre: Vec<f32>,
    pub frames: usize,
    pub lookahead: usize,
    pub channels: usize,
    pub rate: u32,
}

/// Run the chain's stages before the limiter. `pre_gain_db` multiplies the
/// chain's input gain (the Style push plus the user's input gain), which is
/// where level in (G) and the turn-down (S) act.
pub fn stage_pass(
    samples: &[f32],
    rate: u32,
    channels: usize,
    settings: &MasteringSettings,
    pre_gain_db: f32,
) -> StagePass {
    let ch = channels.max(1);
    let frames = samples.len() / ch;
    let mut chain = MasteringChain::new(rate, ch, settings);
    let c = &chain.coeffs;
    // Stages after the limiter must be idle, or the split is not exact.
    assert!((c.volume_match_gain_lin - 1.0).abs() <= 1.0e-4);
    assert!((c.user_output_gain_lin - 1.0).abs() <= 1.0e-4);
    assert!((c.export_landing_gain_lin - 1.0).abs() <= 1.0e-4);
    if pre_gain_db != 0.0 {
        chain.coeffs.input_gain_lin *= db_to_lin(pre_gain_db);
    }
    let production_lookahead = chain.latency_frames();
    chain.limiter = Limiter::new(
        rate,
        ch,
        PASS_THROUGH_CEILING_DBFS,
        LIMITER_LOOKAHEAD_MS,
        LIMITER_RELEASE_MS,
    );
    let lookahead = chain.latency_frames();
    assert_eq!(lookahead, production_lookahead, "limiter lookahead mirror");
    let mut buf = Vec::with_capacity((frames + 2 * lookahead) * ch);
    buf.extend_from_slice(&samples[..frames * ch]);
    buf.resize((frames + 2 * lookahead) * ch, 0.0);
    chain.process_interleaved(&mut buf, ch);
    buf.drain(..lookahead * ch);
    StagePass {
        pre: buf,
        frames,
        lookahead,
        channels: ch,
        rate,
    }
}

/// One limiter pass: what the chain delivers for a drive (before sample-rate
/// conversion and the final loudness landing).
pub struct LimitedRender {
    /// `frames` frames, aligned with the source.
    pub samples: Vec<f32>,
    /// The limiter's load: its largest gain reduction over the track, in dB.
    pub max_gain_reduction_db: f32,
}

/// Feed the stage pass, times the drive, to a real limiter built as the chain
/// builds it. A drive of exactly 0 dB applies no multiply (the idle stage).
pub fn limiter_pass(stage: &StagePass, ceiling_dbtp: f32, drive_db: f32) -> LimitedRender {
    let ch = stage.channels;
    let l = stage.lookahead;
    let mut limiter = Limiter::new(
        stage.rate,
        ch,
        chain_ceiling_db(ceiling_dbtp),
        LIMITER_LOOKAHEAD_MS,
        LIMITER_RELEASE_MS,
    );
    let gain = (drive_db != 0.0).then(|| db_to_lin(drive_db));
    let mut out = vec![0.0_f32; stage.frames * ch];
    let mut frame = vec![0.0_f32; ch];
    let mut max_gr = 0.0_f32;
    for i in 0..stage.frames + l {
        frame.copy_from_slice(&stage.pre[i * ch..(i + 1) * ch]);
        if let Some(g) = gain {
            for s in frame.iter_mut() {
                *s *= g;
            }
        }
        limiter.process_frame_inplace(&mut frame);
        max_gr = max_gr.max(limiter.gain_reduction_db());
        if i >= l {
            out[(i - l) * ch..(i - l + 1) * ch].copy_from_slice(&frame);
        }
    }
    LimitedRender {
        samples: out,
        max_gain_reduction_db: max_gr,
    }
}

/// The production render (as `engine::mastering_render_format_with_cancel`
/// runs the chain): process, then the tail flush.
pub fn production_chain_render(
    samples: &[f32],
    rate: u32,
    channels: usize,
    settings: &MasteringSettings,
) -> Vec<f32> {
    let ch = channels.max(1);
    let mut chain = MasteringChain::new(rate, ch, settings);
    let mut out = samples[..(samples.len() / ch) * ch].to_vec();
    chain.process_interleaved(&mut out, ch);
    chain.flush_render_tail(&mut out, ch);
    out
}

/// The self-check: stage pass + limiter pass with no drive is bit-identical
/// to the production render.
pub fn decomposition_matches_chain(
    samples: &[f32],
    rate: u32,
    channels: usize,
    settings: &MasteringSettings,
) -> bool {
    let real = production_chain_render(samples, rate, channels, settings);
    let stage = stage_pass(samples, rate, channels, settings, 0.0);
    let split = limiter_pass(&stage, settings.effective_ceiling_dbtp(), 0.0);
    real.len() == split.samples.len()
        && real
            .iter()
            .zip(&split.samples)
            .all(|(a, b)| a.to_bits() == b.to_bits())
}

// ---------------------------------------------------------------------------
// The limiter's own peak detector, replicated
// ---------------------------------------------------------------------------

const TRUE_PEAK_TAPS: usize = 12;

fn true_peak_filter() -> [[f32; TRUE_PEAK_TAPS]; 4] {
    std::array::from_fn(|phase| {
        std::array::from_fn(|tap| {
            let j = (tap * 4 + phase) as f64;
            let window = 0.5 * (1.0 - (std::f64::consts::TAU * j / 48.0).cos());
            let x = std::f64::consts::PI * (j - 24.0) / 4.0;
            (window * if x.abs() < 1e-9 { 1.0 } else { x.sin() / x }) as f32
        })
    })
}

/// The level the limiter reacts to, linear: the largest of every sample and
/// 1.012 × every four-phase reconstructed peak, exactly as `Limiter` computes
/// them. The limiter's load at drive `d` is then
/// `max(0, d + 20·log10(peak) − ceiling)`; a test pins that against the
/// limiter's own gain reduction.
pub fn limiter_detector_peak(pre: &[f32], channels: usize) -> f32 {
    let ch = channels.max(1);
    let filter = true_peak_filter();
    let frames = pre.len() / ch;
    let mut level = 0.0_f32;
    for i in 0..frames {
        for c in 0..ch {
            level = level.max(pre[i * ch + c].abs());
        }
        let mut peak = 0.0_f32;
        for c in 0..ch {
            for weights in &filter {
                let mut value = 0.0_f32;
                for (tap, weight) in weights.iter().enumerate() {
                    let s = if i >= tap {
                        pre[(i - tap) * ch + c]
                    } else {
                        0.0
                    };
                    value += s * weight;
                }
                peak = peak.max(value.abs());
            }
        }
        level = level.max(peak * 1.012);
    }
    level
}

// ---------------------------------------------------------------------------
// Resolvers
// ---------------------------------------------------------------------------

/// Integrated loudness, as the app measures it.
pub fn loudness(samples: &[f32], rate: u32, channels: usize) -> f32 {
    yes_master_lib::engine::measure_integrated_lufs(samples, rate, channels.max(1) as u16)
        .unwrap_or(f32::NEG_INFINITY)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriveResolution {
    pub drive_db: f32,
    /// Loudness of the stage pass with an idle limiter (drive 0).
    pub pre_limiter_lufs: f32,
    /// The limiter detector's level of the stage pass, dBFS.
    pub pre_limiter_peak_db: f32,
    /// Largest drive that keeps the limiter idle.
    pub idle_max_drive_db: f32,
    /// Limiter passes run (the last one is the delivered render).
    pub limiter_passes: usize,
    pub limiter_idle: bool,
    pub stopped_at_clean_limit: bool,
    /// Reached the target within tolerance before delivery.
    pub resolved: bool,
    pub inert: bool,
    /// (drive, loudness) per limiter pass, for the ledger.
    pub trace: Vec<(f32, f32)>,
}

/// Contract rule 3 and 4 for G (priority Clean, or Reach target when
/// `clean_limit_db` is `None`):
/// - target reachable with the limiter idle: drive = the difference, exactly;
/// - otherwise the drive rises until the target is reached, or, with Clean,
///   until the limiter's load reaches the clean limit.
///
/// Each step assumes one dB of drive gives at most one dB of loudness
/// (the limiter only removes level), so a step never lands above the target
/// (contract rule 5).
pub fn resolve_drive(
    stage: &StagePass,
    target_lufs: f32,
    ceiling_dbtp: f32,
    clean_limit_db: Option<f32>,
    tolerance_lu: f32,
    max_passes: usize,
) -> (DriveResolution, LimitedRender) {
    let ch = stage.channels;
    let n = stage.frames;
    let l0 = loudness(&stage.pre[..n * ch], stage.rate, ch);
    let peak = limiter_detector_peak(&stage.pre, ch);
    let peak_db = lin_to_db(peak);
    let c_db = chain_ceiling_db(ceiling_dbtp);
    let mut r = DriveResolution {
        pre_limiter_lufs: l0,
        pre_limiter_peak_db: peak_db,
        idle_max_drive_db: c_db - peak_db,
        ..Default::default()
    };
    if !l0.is_finite() || l0 <= -70.0 || !target_lufs.is_finite() || peak <= 0.0 {
        r.inert = true;
        let render = limiter_pass(stage, ceiling_dbtp, 0.0);
        r.limiter_passes = 1;
        r.limiter_idle = render.max_gain_reduction_db == 0.0;
        return (r, render);
    }
    let d0 = target_lufs - l0;
    if d0 <= r.idle_max_drive_db {
        let render = limiter_pass(stage, ceiling_dbtp, d0);
        r.drive_db = d0;
        r.limiter_passes = 1;
        r.limiter_idle = render.max_gain_reduction_db == 0.0;
        r.resolved = true;
        r.trace.push((d0, l0 + d0));
        return (r, render);
    }
    let cap = clean_limit_db.map_or(f32::INFINITY, |limit| c_db + limit - peak_db);
    let mut d = d0.min(cap);
    loop {
        let render = limiter_pass(stage, ceiling_dbtp, d);
        r.limiter_passes += 1;
        let l = loudness(&render.samples, stage.rate, ch);
        r.trace.push((d, l));
        r.drive_db = d;
        if l >= target_lufs - tolerance_lu {
            r.resolved = true;
            return (r, render);
        }
        if d >= cap {
            r.stopped_at_clean_limit = true;
            return (r, render);
        }
        if r.limiter_passes >= max_passes || !l.is_finite() {
            return (r, render);
        }
        d = (d + (target_lufs - l)).min(cap);
    }
}

/// G resolved as contract rule 3 words it: the drive is chosen so the
/// **delivered** loudness equals the target. Every pass renders the limiter,
/// converts to 44.1 kHz and runs the export's qualified finalizer, so the
/// final peak protection is inside the loop. (`resolve_drive` resolves on
/// the chain's output before delivery; the first development run used it.)
///
/// The first drive is the idle-limiter estimate, capped at the clean limit.
/// Each further step adds the remaining delivered shortfall, still capped:
/// a dB of drive adds at most a dB before the landing, and the landing never
/// leaves the delivered loudness above the target, so no step overshoots.
pub fn resolve_drive_delivered(
    stage: &StagePass,
    target_lufs: f32,
    clean_limit_db: Option<f32>,
    tolerance_lu: f32,
    max_passes: usize,
) -> Result<(DriveResolution, f32, Vec<f32>, DeliveryFacts), String> {
    let ch = stage.channels;
    let n = stage.frames;
    let l0 = loudness(&stage.pre[..n * ch], stage.rate, ch);
    let peak = limiter_detector_peak(&stage.pre, ch);
    let peak_db = lin_to_db(peak);
    let c_db = chain_ceiling_db(STANDARD_CEILING_DBTP);
    let mut r = DriveResolution {
        pre_limiter_lufs: l0,
        pre_limiter_peak_db: peak_db,
        idle_max_drive_db: c_db - peak_db,
        ..Default::default()
    };
    if !l0.is_finite() || l0 <= -70.0 || !target_lufs.is_finite() || peak <= 0.0 {
        r.inert = true;
        let render = limiter_pass(stage, STANDARD_CEILING_DBTP, 0.0);
        let load = render.max_gain_reduction_db;
        let (samples, facts) = deliver(render.samples, stage.rate, ch, target_lufs)?;
        r.limiter_passes = 1;
        r.limiter_idle = load == 0.0;
        return Ok((r, load, samples, facts));
    }
    let cap = clean_limit_db.map_or(f32::INFINITY, |limit| c_db + limit - peak_db);
    let mut d = (target_lufs - l0).min(cap);
    loop {
        let render = limiter_pass(stage, STANDARD_CEILING_DBTP, d);
        let load = render.max_gain_reduction_db;
        let (samples, facts) = deliver(render.samples, stage.rate, ch, target_lufs)?;
        r.limiter_passes += 1;
        r.drive_db = d;
        r.limiter_idle = load == 0.0;
        r.trace.push((d, facts.lufs));
        if facts.lufs >= target_lufs - tolerance_lu {
            r.resolved = true;
            return Ok((r, load, samples, facts));
        }
        if d >= cap {
            r.stopped_at_clean_limit = true;
            return Ok((r, load, samples, facts));
        }
        if r.limiter_passes >= max_passes || !facts.lufs.is_finite() {
            return Ok((r, load, samples, facts));
        }
        d = (d + (target_lufs - facts.lufs)).min(cap);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TurnDownResolution {
    /// The reduction moved in front of the chain (≤ 0).
    pub pre_gain_db: f32,
    /// Full passes (stage + limiter), including T's own render.
    pub passes: usize,
    pub resolved: bool,
    pub trace: Vec<(f32, f32)>,
}

/// S: starting from T's render (loudness `t_lufs`), while the chain's output
/// overshoots the target by more than the tolerance, move the overshoot in
/// front of the chain and render again. Never raises the level. Returns
/// `None` for the render when S equals T (no overshoot to move).
#[allow(clippy::too_many_arguments)]
pub fn resolve_turn_down(
    samples: &[f32],
    rate: u32,
    channels: usize,
    settings: &MasteringSettings,
    t_lufs: f32,
    target_lufs: f32,
    tolerance_lu: f32,
    max_passes: usize,
) -> (TurnDownResolution, Option<LimitedRender>) {
    let mut r = TurnDownResolution {
        passes: 1,
        trace: vec![(0.0, t_lufs)],
        ..Default::default()
    };
    let mut lufs = t_lufs;
    let mut render = None;
    while lufs.is_finite() && lufs - target_lufs > tolerance_lu && r.passes < max_passes {
        r.pre_gain_db = (r.pre_gain_db + (target_lufs - lufs)).min(0.0);
        let stage = stage_pass(samples, rate, channels, settings, r.pre_gain_db);
        let limited = limiter_pass(&stage, settings.effective_ceiling_dbtp(), 0.0);
        lufs = loudness(&limited.samples, rate, channels);
        r.passes += 1;
        r.trace.push((r.pre_gain_db, lufs));
        render = Some(limited);
    }
    r.resolved = lufs.is_finite() && lufs - target_lufs <= tolerance_lu;
    (r, render)
}

// ---------------------------------------------------------------------------
// Delivery: Standard's export after the chain
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryFacts {
    pub lufs: f32,
    pub true_peak_dbtp: f32,
    pub lra: f32,
    /// The final landing, dB (negative = turned down).
    pub landing_db: f32,
    pub all_finite: bool,
    /// The qualified peak check passed (finalize returned) and the peak is
    /// at or under the ceiling.
    pub under_ceiling: bool,
}

/// Sample-rate conversion to Standard's 44.1 kHz, then the export's
/// qualified finalizer (landing, peak protection, 24-bit delivery), exactly
/// as `mastering_render_format_with_cancel` runs them.
pub fn deliver(
    render: Vec<f32>,
    rate: u32,
    channels: usize,
    target_lufs: f32,
) -> Result<(Vec<f32>, DeliveryFacts), String> {
    let ch = channels.max(1) as u16;
    let mut samples = if rate != STANDARD_RATE {
        crate::sample_rate::convert_interleaved(&render, rate, STANDARD_RATE, ch)
            .map_err(|e| e.to_string())?
    } else {
        render
    };
    let p = yes_master_lib::output_protection::finalize(
        &mut samples,
        STANDARD_RATE,
        ch,
        STANDARD_BITS,
        Some(target_lufs),
        STANDARD_CEILING_DBTP,
        None,
    )
    .map_err(|e| e.to_string())?;
    let all_finite = samples.iter().all(|s| s.is_finite());
    let facts = DeliveryFacts {
        lufs: p.lufs,
        true_peak_dbtp: p.true_peak_dbtp,
        lra: p.lra,
        landing_db: lin_to_db(p.gain_lin),
        all_finite,
        under_ceiling: p.true_peak_dbtp <= STANDARD_CEILING_DBTP,
    };
    Ok((samples, facts))
}

// ---------------------------------------------------------------------------
// September's shape metrics (fixed C1 protocol), at 48 kHz
// ---------------------------------------------------------------------------

pub const ANALYSIS_RATE: u32 = 48_000;
/// September's nine fixed tone bands.
pub const TONE_BANDS: [(f64, f64); 9] = [
    (20.0, 60.0),
    (60.0, 120.0),
    (120.0, 250.0),
    (250.0, 500.0),
    (500.0, 1000.0),
    (1000.0, 2000.0),
    (2000.0, 4000.0),
    (4000.0, 8000.0),
    (8000.0, 16000.0),
];
/// The app's six analysis bands (`analysis.rs` edges 20/80/250/800/2500/6500),
/// the top band closed at 20 kHz so it does not depend on the sample rate.
/// Used for the preset-fingerprint band tilt.
pub const APP_BANDS: [(f64, f64); 6] = [
    (20.0, 80.0),
    (80.0, 250.0),
    (250.0, 800.0),
    (800.0, 2500.0),
    (2500.0, 6500.0),
    (6500.0, 20000.0),
];
const WELCH_SEGMENT: usize = 8192;
const WELCH_HOP: usize = 4096;
const SECTION_SECONDS: usize = 10;
const SECTION_ELIGIBLE_DB: f64 = -40.0;
const ENVELOPE_BLOCK: usize = 480;
const TRANSIENT_DISTANCE_BLOCKS: usize = 20;
const TRANSIENT_ANCHORS: usize = 40;

fn db_power(p: f64) -> f64 {
    10.0 * p.max(1e-30).log10()
}

/// Resample to the analysis rate (the app's own converter).
pub fn to_analysis_rate(samples: &[f32], rate: u32, channels: usize) -> Vec<f32> {
    if rate == ANALYSIS_RATE {
        return samples.to_vec();
    }
    crate::sample_rate::convert_interleaved(samples, rate, ANALYSIS_RATE, channels.max(1) as u16)
        .expect("analysis-rate conversion")
}

/// Source anchors, fixed once and never reselected on a render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchors {
    /// (quietest, loudest) eligible 10-second section.
    pub section_pair: Option<(usize, usize)>,
    pub transient_times: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub rms_db: f64,
    pub sample_peak_db: f64,
    pub sections_db: Vec<f64>,
    pub attack_body_db: Vec<f64>,
    pub tone_bands_db: [f64; 9],
    pub app_bands_db: [f64; 6],
    /// 10·log10(side power / mid power); `None` for mono.
    pub side_mid_db: Option<f64>,
    pub correlation: Option<f64>,
}

/// Welch power spectral density (scipy defaults: periodic Hann, constant
/// detrend, density scaling, one-sided), averaged over channels.
pub fn welch_psd(x: &[f32], channels: usize, rate: u32) -> Vec<f64> {
    let ch = channels.max(1);
    let frames = x.len() / ch;
    let n = WELCH_SEGMENT;
    let bins = n / 2 + 1;
    let mut acc = vec![0.0_f64; bins];
    if frames < n {
        return acc;
    }
    let window: Vec<f64> = (0..n)
        .map(|i| 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / n as f64).cos())
        .collect();
    let win_power: f64 = window.iter().map(|w| w * w).sum();
    let scale = 1.0 / (f64::from(rate) * win_power);
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);
    let mut buf = vec![Complex::new(0.0, 0.0); n];
    let segments = (frames - n) / WELCH_HOP + 1;
    for seg in 0..segments {
        let start = seg * WELCH_HOP;
        for c in 0..ch {
            let mean: f64 = (0..n)
                .map(|i| f64::from(x[(start + i) * ch + c]))
                .sum::<f64>()
                / n as f64;
            for (i, b) in buf.iter_mut().enumerate() {
                *b = Complex::new((f64::from(x[(start + i) * ch + c]) - mean) * window[i], 0.0);
            }
            fft.process(&mut buf);
            for (k, a) in acc.iter_mut().enumerate() {
                let mut p = buf[k].norm_sqr() * scale;
                if k != 0 && k != n / 2 {
                    p *= 2.0;
                }
                *a += p;
            }
        }
    }
    let denom = (segments * ch) as f64;
    for a in acc.iter_mut() {
        *a /= denom;
    }
    acc
}

fn band_levels<const B: usize>(psd: &[f64], rate: u32, bands: &[(f64, f64); B]) -> [f64; B] {
    let df = f64::from(rate) / WELCH_SEGMENT as f64;
    std::array::from_fn(|b| {
        let (lo, hi) = bands[b];
        let sum: f64 = psd
            .iter()
            .enumerate()
            .filter(|(k, _)| {
                let f = *k as f64 * df;
                f >= lo && f < hi
            })
            .map(|(_, p)| p)
            .sum();
        db_power(sum * df)
    })
}

fn sections_db(x: &[f32], ch: usize) -> Vec<f64> {
    let len = ANALYSIS_RATE as usize * SECTION_SECONDS;
    let frames = x.len() / ch;
    (0..frames / len)
        .map(|s| {
            let slice = &x[s * len * ch..(s + 1) * len * ch];
            db_power(slice.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / slice.len() as f64)
        })
        .collect()
}

/// scipy's `find_peaks(x, distance)`: local maxima (plateaus at their
/// middle), then the highest first, removing neighbours closer than
/// `distance`.
pub fn find_peaks(x: &[f64], distance: usize) -> Vec<usize> {
    let mut peaks = Vec::new();
    if x.len() >= 3 {
        let mut i = 1;
        let i_max = x.len() - 1;
        while i < i_max {
            if x[i - 1] < x[i] {
                let mut ahead = i + 1;
                while ahead < i_max && x[ahead] == x[i] {
                    ahead += 1;
                }
                if x[ahead] < x[i] {
                    peaks.push((i + ahead - 1) / 2);
                    i = ahead;
                }
            }
            i += 1;
        }
    }
    if distance <= 1 || peaks.len() < 2 {
        return peaks;
    }
    let mut keep = vec![true; peaks.len()];
    let mut order: Vec<usize> = (0..peaks.len()).collect();
    order.sort_by(|a, b| x[peaks[*a]].total_cmp(&x[peaks[*b]]));
    for &j in order.iter().rev() {
        if !keep[j] {
            continue;
        }
        let mut k = j;
        while k > 0 && peaks[j] - peaks[k - 1] < distance {
            keep[k - 1] = false;
            k -= 1;
        }
        let mut k = j + 1;
        while k < peaks.len() && peaks[k] - peaks[j] < distance {
            keep[k] = false;
            k += 1;
        }
    }
    peaks
        .into_iter()
        .zip(keep)
        .filter_map(|(p, k)| k.then_some(p))
        .collect()
}

/// The source's anchors: its quietest and loudest eligible 10-second
/// sections, and up to 40 attack times from 10 ms RMS novelty.
pub fn source_anchors(x48: &[f32], channels: usize) -> Anchors {
    let ch = channels.max(1);
    let sections = sections_db(x48, ch);
    let eligible: Vec<usize> = (0..sections.len())
        .filter(|i| sections[*i] > SECTION_ELIGIBLE_DB)
        .collect();
    let section_pair = (eligible.len() >= 2).then(|| {
        let lo = *eligible
            .iter()
            .min_by(|a, b| sections[**a].total_cmp(&sections[**b]))
            .expect("eligible");
        let hi = *eligible
            .iter()
            .max_by(|a, b| sections[**a].total_cmp(&sections[**b]))
            .expect("eligible");
        (lo, hi)
    });
    let frames = x48.len() / ch;
    let blocks = frames / ENVELOPE_BLOCK;
    let env: Vec<f64> = (0..blocks)
        .map(|b| {
            let s = &x48[b * ENVELOPE_BLOCK * ch..(b + 1) * ENVELOPE_BLOCK * ch];
            (s.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / s.len() as f64).sqrt()
        })
        .collect();
    let novelty: Vec<f64> = (0..env.len())
        .map(|i| {
            if i == 0 {
                0.0
            } else {
                (env[i] - env[i - 1]).max(0.0)
            }
        })
        .collect();
    let mut peaks = find_peaks(&novelty, TRANSIENT_DISTANCE_BLOCKS);
    // Python's stable sort by novelty, descending, then the first 40.
    peaks.sort_by(|a, b| novelty[*b].total_cmp(&novelty[*a]));
    peaks.truncate(TRANSIENT_ANCHORS);
    let duration = frames as f64 / f64::from(ANALYSIS_RATE);
    let mut times: Vec<f64> = peaks
        .into_iter()
        .map(|k| k as f64 * 0.01)
        .filter(|t| *t > 0.05 && *t < duration - 0.2)
        .collect();
    times.sort_by(f64::total_cmp);
    Anchors {
        section_pair,
        transient_times: times,
    }
}

fn attack_body_db(x48: &[f32], ch: usize, times: &[f64]) -> Vec<f64> {
    let frames = x48.len() / ch;
    let rate = f64::from(ANALYSIS_RATE);
    let idx = |t: f64| ((t * rate) as usize).min(frames);
    times
        .iter()
        .map(|t| {
            let a = &x48[idx(t - 0.02) * ch..idx(t + 0.03) * ch];
            let b = &x48[idx(t + 0.03) * ch..idx(t + 0.15) * ch];
            let peak = a.iter().map(|v| f64::from(v.abs())).fold(0.0, f64::max);
            let body = if b.is_empty() {
                0.0
            } else {
                b.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / b.len() as f64
            };
            db_power(peak * peak / body.max(1e-30))
        })
        .collect()
}

/// Measure a 48 kHz signal against the source's anchors.
pub fn shape(x48: &[f32], channels: usize, anchors: &Anchors) -> Shape {
    let ch = channels.max(1);
    let n = x48.len() as f64;
    let rms_db = db_power(x48.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / n.max(1.0));
    let sample_peak_db = db_power(
        x48.iter()
            .map(|v| f64::from(v.abs()))
            .fold(0.0, f64::max)
            .powi(2),
    );
    let psd = welch_psd(x48, ch, ANALYSIS_RATE);
    let (side_mid_db, correlation) = if ch == 2 {
        let (mut mm, mut ss, mut ll, mut rr, mut lr, mut sl, mut sr) = (
            0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64,
        );
        let frames = x48.len() / 2;
        for f in x48.chunks_exact(2) {
            let (l, r) = (f64::from(f[0]), f64::from(f[1]));
            let mid = 0.5 * (l + r);
            let side = 0.5 * (l - r);
            mm += mid * mid;
            ss += side * side;
            ll += l * l;
            rr += r * r;
            lr += l * r;
            sl += l;
            sr += r;
        }
        let nf = frames as f64;
        let cov = lr / nf - (sl / nf) * (sr / nf);
        let vl = ll / nf - (sl / nf).powi(2);
        let vr = rr / nf - (sr / nf).powi(2);
        let corr = cov / (vl * vr).sqrt();
        (
            Some(db_power(ss / mm.max(1e-30))),
            corr.is_finite().then_some(corr),
        )
    } else {
        (None, None)
    };
    Shape {
        rms_db,
        sample_peak_db,
        sections_db: sections_db(x48, ch),
        attack_body_db: attack_body_db(x48, ch, &anchors.transient_times),
        tone_bands_db: band_levels(&psd, ANALYSIS_RATE, &TONE_BANDS),
        app_bands_db: band_levels(&psd, ANALYSIS_RATE, &APP_BANDS),
        side_mid_db,
        correlation,
    }
}

/// numpy's default (linear) percentile.
pub fn percentile(values: &[f64], q: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let pos = q / 100.0 * (v.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    Some(v[lo] + (v[hi] - v[lo]) * (pos - lo as f64))
}

pub fn median(values: &[f64]) -> Option<f64> {
    percentile(values, 50.0)
}

/// A render compared with its source (September's `compare`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeChange {
    /// Change of loud-minus-quiet section contrast, dB (negative = lost).
    pub section_contrast_delta_db: Option<f64>,
    /// Paired attack change, median and 10th percentile, dB.
    pub attack_median_delta_db: Option<f64>,
    pub attack_p10_delta_db: Option<f64>,
    /// Each tone band's change after removing the common RMS gain.
    pub tone_band_delta_db: [f64; 9],
    pub side_mid_delta_db: Option<f64>,
    pub correlation_delta: Option<f64>,
}

pub fn compare(source: &Shape, anchors: &Anchors, render: &Shape) -> ShapeChange {
    let gain = render.rms_db - source.rms_db;
    let section_contrast_delta_db = anchors.section_pair.and_then(|(lo, hi)| {
        let (r_lo, r_hi) = (render.sections_db.get(lo)?, render.sections_db.get(hi)?);
        Some((r_hi - r_lo) - (source.sections_db[hi] - source.sections_db[lo]))
    });
    let attacks: Vec<f64> = render
        .attack_body_db
        .iter()
        .zip(&source.attack_body_db)
        .map(|(r, s)| r - s)
        .collect();
    ShapeChange {
        section_contrast_delta_db,
        attack_median_delta_db: median(&attacks),
        attack_p10_delta_db: percentile(&attacks, 10.0),
        tone_band_delta_db: std::array::from_fn(|b| {
            render.tone_bands_db[b] - source.tone_bands_db[b] - gain
        }),
        side_mid_delta_db: render
            .side_mid_db
            .zip(source.side_mid_db)
            .map(|(r, s)| r - s),
        correlation_delta: render
            .correlation
            .zip(source.correlation)
            .map(|(r, s)| r - s),
    }
}

// ---------------------------------------------------------------------------
// Style distinctness: the preset-fingerprint distance, on real songs
// ---------------------------------------------------------------------------

/// `tests/preset_fingerprint.rs`'s fingerprint, measured on a delivered
/// master of a real song instead of its synthetic fixtures:
/// - band tilt per app band: (output − source band level) − (output − source
///   loudness);
/// - crest: sample peak − integrated loudness;
/// - width: change of side/mid balance;
/// - landed loudness (half weight in the distance).
///
/// The THD term needs the harness's 1 kHz probe tone, which music has not,
/// so it is left out.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Fingerprint {
    pub band_tilt_db: [f64; 6],
    pub crest_db: f64,
    pub width_delta_db: f64,
    pub landed_lufs: f64,
}

pub fn fingerprint(
    source: &Shape,
    source_lufs: f64,
    render: &Shape,
    render_lufs: f64,
    render_sample_peak_db: f64,
) -> Fingerprint {
    let broadband = render_lufs - source_lufs;
    Fingerprint {
        band_tilt_db: std::array::from_fn(|b| {
            render.app_bands_db[b] - source.app_bands_db[b] - broadband
        }),
        crest_db: render_sample_peak_db - render_lufs,
        width_delta_db: render
            .side_mid_db
            .zip(source.side_mid_db)
            .map_or(0.0, |(r, s)| r - s),
        landed_lufs: render_lufs,
    }
}

/// `character_distance` from `tests/preset_fingerprint.rs`, without the THD
/// term.
pub fn character_distance(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let mut sum: f64 = a
        .band_tilt_db
        .iter()
        .zip(&b.band_tilt_db)
        .map(|(x, y)| (x - y).powi(2))
        .sum();
    sum += (a.crest_db - b.crest_db).powi(2);
    sum += (a.width_delta_db - b.width_delta_db).powi(2);
    sum += (0.5 * (a.landed_lufs - b.landed_lufs)).powi(2);
    sum.sqrt()
}

/// One song at one loudness under one design: the mean of the six pairwise
/// distances between the four Styles, and the closest pair.
pub fn distinctness(prints: &[Fingerprint]) -> (f64, f64) {
    let mut d = Vec::new();
    for i in 0..prints.len() {
        for j in (i + 1)..prints.len() {
            d.push(character_distance(&prints[i], &prints[j]));
        }
    }
    let mean = d.iter().sum::<f64>() / d.len().max(1) as f64;
    let min = d.iter().copied().fold(f64::INFINITY, f64::min);
    (mean, min)
}

// ---------------------------------------------------------------------------
// The decision rule (plan "Decision rule, fixed in advance")
// ---------------------------------------------------------------------------

/// The numbers the rule reads for one design on one case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignOutcome {
    pub lufs: f32,
    pub true_peak_dbtp: f32,
    pub plr_db: f32,
    pub landing_db: f32,
    pub all_finite: bool,
    pub under_ceiling: bool,
    pub stopped_at_clean_limit: bool,
    pub tone_band_delta_db: [f64; 9],
    /// Section contrast change and paired-attack median against the source
    /// (R1c, frozen at Checkpoint A).
    #[serde(default)]
    pub section_contrast_delta_db: Option<f64>,
    #[serde(default)]
    pub attack_median_delta_db: Option<f64>,
}

impl DesignOutcome {
    pub fn within(&self, target: f32) -> bool {
        (self.lufs - target).abs() <= TARGET_TOLERANCE_LU
    }
}

/// One (song, Style, loudness) case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCase {
    pub song: String,
    pub reference_song: bool,
    pub source_lufs: f32,
    pub style: String,
    pub target: f32,
    pub level_in_db: Option<f32>,
    pub t: Option<DesignOutcome>,
    pub s: Option<DesignOutcome>,
    pub g: Option<DesignOutcome>,
}

/// Style distinctness for one (song, loudness).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistinctCase {
    pub song: String,
    pub source_lufs: f32,
    pub target: f32,
    pub t: Option<f64>,
    pub s: Option<f64>,
    pub g: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub pass: bool,
    pub cases: usize,
    pub detail: String,
}

/// Which design reads as "the candidate" in R1–R7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Design {
    S,
    G,
}

fn pick(case: &RuleCase, d: Design) -> Option<&DesignOutcome> {
    match d {
        Design::S => case.s.as_ref(),
        Design::G => case.g.as_ref(),
    }
}

/// R1–R7 for one design, exactly as the plan words them.
pub fn evaluate_rule(cases: &[RuleCase], distinct: &[DistinctCase], d: Design) -> Vec<Condition> {
    let mut out = Vec::new();
    let name = match d {
        Design::S => "S",
        Design::G => "G",
    };

    // R1: where T and the design both land within tolerance, its PLR is never
    // more than 0.1 dB below T's.
    let r1: Vec<f64> = cases
        .iter()
        .filter_map(|c| {
            let (t, x) = (c.t.as_ref()?, pick(c, d)?);
            (t.within(c.target) && x.within(c.target)).then(|| f64::from(x.plr_db - t.plr_db))
        })
        .collect();
    let r1_worst = r1.iter().copied().fold(f64::INFINITY, f64::min);
    let r1_fail = r1.iter().filter(|v| **v < -0.1).count();
    out.push(Condition {
        id: "R1".into(),
        pass: !r1.is_empty() && r1_fail == 0,
        cases: r1.len(),
        detail: format!(
            "{name} PLR − T PLR where both within ±{TARGET_TOLERANCE_LU} LU: worst {r1_worst:+.3} dB, {r1_fail} below −0.1 dB, median {:+.3} dB",
            median(&r1).unwrap_or(f64::NAN)
        ),
    });

    // R2: lands within tolerance in at least as many cases as T.
    let t_hits = cases
        .iter()
        .filter(|c| c.t.as_ref().is_some_and(|t| t.within(c.target)))
        .count();
    let x_hits = cases
        .iter()
        .filter(|c| pick(c, d).is_some_and(|x| x.within(c.target)))
        .count();
    out.push(Condition {
        id: "R2".into(),
        pass: x_hits >= t_hits,
        cases: cases.len(),
        detail: format!("within ±{TARGET_TOLERANCE_LU} LU: {name} {x_hits}, T {t_hits}"),
    });

    // R3: where T's landing turns the result down by 1 dB or more, the
    // design's median PLR gain is at least 1.0 dB.
    let r3: Vec<f64> = cases
        .iter()
        .filter_map(|c| {
            let (t, x) = (c.t.as_ref()?, pick(c, d)?);
            (t.landing_db <= -1.0).then(|| f64::from(x.plr_db - t.plr_db))
        })
        .collect();
    let r3_median = median(&r3);
    out.push(Condition {
        id: "R3".into(),
        pass: r3_median.is_some_and(|m| m >= 1.0),
        cases: r3.len(),
        detail: format!(
            "T turned down ≥ 1 dB: median {name} PLR gain {:+.3} dB (p10 {:+.3}, p90 {:+.3})",
            r3_median.unwrap_or(f64::NAN),
            percentile(&r3, 10.0).unwrap_or(f64::NAN),
            percentile(&r3, 90.0).unwrap_or(f64::NAN)
        ),
    });

    // R4: where T misses by more than 0.5 LU, the design reaches the target
    // or stops at the clean limit, in every case.
    let r4: Vec<&RuleCase> = cases
        .iter()
        .filter(|c| {
            c.t.as_ref()
                .is_some_and(|t| (t.lufs - c.target).abs() > 0.5)
        })
        .collect();
    let r4_reached = r4
        .iter()
        .filter(|c| pick(c, d).is_some_and(|x| x.within(c.target)))
        .count();
    let r4_stopped = r4
        .iter()
        .filter(|c| pick(c, d).is_some_and(|x| !x.within(c.target) && x.stopped_at_clean_limit))
        .count();
    let r4_other = r4.len() - r4_reached - r4_stopped;
    out.push(Condition {
        id: "R4".into(),
        pass: r4_other == 0,
        cases: r4.len(),
        detail: format!(
            "T missed by > 0.5 LU: {name} reached {r4_reached}, stopped at the clean limit {r4_stopped}, neither {r4_other}"
        ),
    });

    // R5: on songs louder than −14 LUFS at the source, median Style
    // distinctness under the design is at least that under T.
    let loud: Vec<&DistinctCase> = distinct.iter().filter(|c| c.source_lufs > -14.0).collect();
    let t_vals: Vec<f64> = loud.iter().filter_map(|c| c.t).collect();
    let x_vals: Vec<f64> = loud
        .iter()
        .filter_map(|c| match d {
            Design::S => c.s,
            Design::G => c.g,
        })
        .collect();
    let (tm, xm) = (median(&t_vals), median(&x_vals));
    out.push(Condition {
        id: "R5".into(),
        pass: matches!((tm, xm), (Some(t), Some(x)) if x >= t),
        cases: loud.len(),
        detail: format!(
            "source louder than −14 LUFS: median distinctness {name} {:.3}, T {:.3}",
            xm.unwrap_or(f64::NAN),
            tm.unwrap_or(f64::NAN)
        ),
    });

    // R6: on the reference song, level in is 0.0 dB, and at each loudness
    // the design's tone bands are within 0.5 dB of T's.
    let reference: Vec<&RuleCase> = cases.iter().filter(|c| c.reference_song).collect();
    let level_in_zero = reference
        .iter()
        .all(|c| d == Design::S || c.level_in_db == Some(0.0));
    let worst_band = reference
        .iter()
        .filter_map(|c| {
            let (t, x) = (c.t.as_ref()?, pick(c, d)?);
            Some(
                (0..9)
                    .map(|b| (x.tone_band_delta_db[b] - t.tone_band_delta_db[b]).abs())
                    .fold(0.0, f64::max),
            )
        })
        .fold(0.0, f64::max);
    out.push(Condition {
        id: "R6".into(),
        pass: !reference.is_empty() && level_in_zero && worst_band <= 0.5,
        cases: reference.len(),
        detail: format!(
            "reference song: level in exactly 0.0 dB {}; largest tone-band difference from T {worst_band:.3} dB",
            if d == Design::S {
                "(n/a for S)".to_string()
            } else if level_in_zero {
                "yes".to_string()
            } else {
                "NO".to_string()
            }
        ),
    });

    // R7: every render finite and under the ceiling by the qualified check.
    let renders: Vec<&DesignOutcome> = cases.iter().filter_map(|c| pick(c, d)).collect();
    let missing = cases.len() - renders.len();
    let bad = renders
        .iter()
        .filter(|x| !(x.all_finite && x.under_ceiling))
        .count();
    out.push(Condition {
        id: "R7".into(),
        pass: missing == 0 && bad == 0,
        cases: cases.len(),
        detail: format!(
            "{} renders: {bad} non-finite or over the ceiling, {missing} failed to deliver",
            renders.len()
        ),
    });
    out
}

/// R1c on one measure: over cases where T and G both land within
/// tolerance, G may be worse than T by more than 0.5 dB in at most 5% of
/// cases, and by more than 1.0 dB in none.
fn r1c(
    cases: &[RuleCase],
    id: &str,
    what: &str,
    pick_measure: impl Fn(&DesignOutcome) -> Option<f64>,
) -> Condition {
    let both = |c: &&RuleCase| {
        c.t.as_ref().is_some_and(|t| t.within(c.target))
            && c.g.as_ref().is_some_and(|g| g.within(c.target))
    };
    let compared = cases.iter().filter(both).count();
    let diffs: Vec<(f64, &RuleCase)> = cases
        .iter()
        .filter(both)
        .filter_map(|c| {
            let (t, g) = (c.t.as_ref()?, c.g.as_ref()?);
            Some((pick_measure(g)? - pick_measure(t)?, c))
        })
        .collect();
    let missing = compared - diffs.len();
    let worse_05 = diffs.iter().filter(|(d, _)| *d < -0.5).count();
    let worse_10 = diffs.iter().filter(|(d, _)| *d < -1.0).count();
    let share = worse_05 as f64 / diffs.len().max(1) as f64;
    let worst = diffs
        .iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(d, c)| format!("{d:+.3} dB ({}, {}, {})", c.song, c.style, c.target))
        .unwrap_or_else(|| "n/a".into());
    Condition {
        id: id.into(),
        pass: !diffs.is_empty() && missing == 0 && share <= 0.05 && worse_10 == 0,
        cases: diffs.len(),
        detail: format!(
            "{what}, G against T where both within \u{b1}{TARGET_TOLERANCE_LU} LU: worse by more than 0.5 dB in {worse_05} ({:.2}%), by more than 1.0 dB in {worse_10}; worst {worst}; {missing} without the measure",
            100.0 * share
        ),
    }
}

/// The rule as frozen at Checkpoint A (planning session, 2026-11-24): R1b,
/// R1c on section contrast and on the paired-attack median, then R2 to R7
/// unchanged. G only; S is dropped.
pub fn evaluate_frozen_rule(cases: &[RuleCase], distinct: &[DistinctCase]) -> Vec<Condition> {
    let r1b: Vec<(f64, &RuleCase)> = cases
        .iter()
        .filter_map(|c| {
            let (t, g) = (c.t.as_ref()?, c.g.as_ref()?);
            (t.within(c.target) && g.within(c.target)).then(|| (f64::from(g.plr_db - t.plr_db), c))
        })
        .collect();
    let below = r1b.iter().filter(|(d, _)| *d < -0.5).count();
    let worst = r1b
        .iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(d, c)| format!("{d:+.3} dB ({}, {}, {})", c.song, c.style, c.target))
        .unwrap_or_else(|| "n/a".into());
    let mut out = vec![
        Condition {
            id: "R1b".into(),
            pass: !r1b.is_empty() && below == 0,
            cases: r1b.len(),
            detail: format!(
                "G PLR \u{2212} T PLR where both within \u{b1}{TARGET_TOLERANCE_LU} LU: {below} more than 0.5 dB below; worst {worst}"
            ),
        },
        r1c(cases, "R1c-section", "section contrast change", |o| {
            o.section_contrast_delta_db
        }),
        r1c(cases, "R1c-attack", "paired-attack median", |o| {
            o.attack_median_delta_db
        }),
    ];
    out.extend(
        evaluate_rule(cases, distinct, Design::G)
            .into_iter()
            .filter(|c| c.id != "R1"),
    );
    out
}

/// The gains and flags in the export receipt's locked line, e.g.
/// `Auto gain staging: level in +2.1 dB · limiter drive +4.5 dB · limiting
/// 6.0 dB · stopped at the clean limit`. `None` for `Auto gain staging: off`.
pub fn parse_receipt_digest(line: &str) -> Option<(f32, f32, f32, bool)> {
    let rest = line.strip_prefix("Auto gain staging: level in ")?;
    let number = |text: &str| -> Option<f32> {
        let t = text.trim().strip_suffix(" dB")?.replace('\u{2212}', "-");
        t.parse().ok()
    };
    let mut parts = rest.split(" \u{b7} ");
    let level_in = number(parts.next()?)?;
    let drive = number(parts.next()?.strip_prefix("limiter drive ")?)?;
    let load = number(parts.next()?.strip_prefix("limiting ")?)?;
    let stopped = parts.next() == Some("stopped at the clean limit");
    Some((level_in, drive, load, stopped))
}

/// The plan's selection: G if R1–R7 all hold; else S if G fails R1, R6 or
/// R7 and S satisfies R1, R3 and R7; else neither.
pub fn select_design(g: &[Condition], s: &[Condition]) -> &'static str {
    let pass = |set: &[Condition], id: &str| set.iter().any(|c| c.id == id && c.pass);
    if g.iter().all(|c| c.pass) {
        return "G";
    }
    let g_failed_trigger = ["R1", "R6", "R7"].iter().any(|id| !pass(g, id));
    let s_qualifies = ["R1", "R3", "R7"].iter().all(|id| pass(s, id));
    if g_failed_trigger && s_qualifies {
        "S"
    } else {
        "neither"
    }
}
