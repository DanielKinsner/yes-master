//! YES Master in the browser. The Standard chain, verbatim, compiled to wasm.
//!
//! Every DSP and analysis module is included **by path from the desktop
//! crate**, so the browser masters what the app masters: the same chain
//! (`dsp.rs`), the same whole-track source analysis (`analysis.rs`), the same
//! adaptive guardrails (`guardrails.rs`) and the same profile construction
//! (`types.rs::SourceProfile::from_measurements`). Landing math mirrors
//! `engine::ceiling_bounded_landing_delta_db`.
//!
//! What is deliberately NOT here: the confidence gate and the Adaptive
//! Compressor. Both are owner-gated OFF on the desktop today, so the desktop
//! Track Master path resolves them to `None` — exactly what this crate passes.
//! `scripts/build-tryit-wasm.mjs` stamps the desktop sources this binary was
//! built from; `src/tryit/engine-stamp.test.ts` fails when they drift.
#![allow(dead_code)]

#[path = "../../../../src-tauri/src/types.rs"]
pub mod types;
#[path = "../../../../src-tauri/src/dsp.rs"]
pub mod dsp;
#[path = "../../../../src-tauri/src/export_format.rs"]
pub mod export_format;
#[path = "../../../../src-tauri/src/analysis.rs"]
pub mod analysis;
#[path = "../../../../src-tauri/src/deep_analysis.rs"]
pub mod deep_analysis;
#[path = "../../../../src-tauri/src/confidence.rs"]
pub mod confidence;
#[path = "../../../../src-tauri/src/guardrails.rs"]
pub mod guardrails;
mod stubs;
pub use stubs::{decode, files, mp3};

use ebur128::{EbuR128, Mode};
use types::*;
use wasm_bindgen::prelude::*;

fn js(e: impl ToString) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn sanitize_lufs(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        -70.0
    }
}

/// Mirror of `engine::ceiling_bounded_landing_delta_db`.
fn landing_delta_db(measured_lufs: f32, measured_tp: f32, target_lufs: f32, ceiling_dbtp: f32) -> f32 {
    if !target_lufs.is_finite() || !measured_lufs.is_finite() || measured_lufs <= -70.0 {
        return 0.0;
    }
    let delta = target_lufs - measured_lufs;
    let cap = if measured_tp.is_finite() { ceiling_dbtp - measured_tp } else { 0.0 };
    let d = delta.min(cap);
    if d.abs() > 1.0e-4 {
        d
    } else {
        0.0
    }
}

fn true_peak_dbtp(ebu: &EbuR128, channels: u32) -> Result<f32, String> {
    let mut peak = 0.0f64;
    for ch in 0..channels {
        let tp = ebu.true_peak(ch).map_err(|e| e.to_string())?;
        if tp > peak {
            peak = tp;
        }
    }
    Ok(if peak > 0.0 { (20.0 * peak.log10()) as f32 } else { -60.0 })
}

/// Integrated LUFS + true peak of interleaved f32 audio.
fn measure(samples: &[f32], channels: u32, sample_rate: u32) -> Result<(f32, f32), String> {
    let mut ebu = EbuR128::new(channels, sample_rate, Mode::I | Mode::TRUE_PEAK).map_err(|e| e.to_string())?;
    ebu.add_frames_f32(samples).map_err(|e| e.to_string())?;
    let lufs = sanitize_lufs(ebu.loudness_global().map_err(|e| e.to_string())? as f32);
    Ok((lufs, true_peak_dbtp(&ebu, channels)?))
}

fn preset_for(style: &str) -> Preset {
    match style {
        "clarity" => Preset::Clarity,
        "tape" => Preset::Tape,
        "oomph" => Preset::Oomph,
        _ => Preset::Universal,
    }
}

// ---------------------------------------------------------------------------
// Whole-track analysis — the desktop's `analysis::analyze_one_with_progress`
// stages, exposed one call per stage so the worker can report REAL progress
// between them (the app's analysis bar moves at real phase boundaries too).
// ---------------------------------------------------------------------------

/// Stage 1 — loudness. Returns `[integrated LUFS, true peak dBTP, LRA LU]`.
#[wasm_bindgen]
pub fn analyze_loudness(samples: &[f32], channels: u32, sample_rate: u32) -> Result<Vec<f32>, JsValue> {
    let mut ebu = EbuR128::new(channels, sample_rate, Mode::I | Mode::LRA | Mode::TRUE_PEAK).map_err(js)?;
    ebu.add_frames_f32(samples).map_err(js)?;
    let lufs = sanitize_lufs(ebu.loudness_global().map_err(js)? as f32);
    let lra = ebu.loudness_range().map_err(js)? as f32;
    let tp = true_peak_dbtp(&ebu, channels).map_err(js)?;
    Ok(vec![lufs, tp, if lra.is_finite() { lra } else { 0.0 }])
}

/// Stage 2 — dynamics. P95−P10 of 100 ms RMS blocks in dB; `NaN` when the
/// signal is too short or silent (the desktop's "no DR trigger" case).
#[wasm_bindgen]
pub fn analyze_dynamics(samples: &[f32], channels: u32, sample_rate: u32) -> f32 {
    analysis::compute_dynamic_range_p95_p10(samples, sample_rate, channels as usize).unwrap_or(f32::NAN)
}

/// Stage 3 — stereo field. Returns `[L/R correlation or NaN for mono, side-energy width]`.
#[wasm_bindgen]
pub fn analyze_stereo(samples: &[f32], channels: u32) -> Vec<f32> {
    let ch = channels as usize;
    vec![
        analysis::compute_stereo_correlation(samples, ch).unwrap_or(f32::NAN),
        analysis::compute_stereo_width(samples, ch),
    ]
}

/// Stage 4 — tonal balance. Whole-track 6-band energy shares as JSON, or an
/// empty string when the track is too short for a spectral read.
#[wasm_bindgen]
pub fn analyze_tonal(samples: &[f32], channels: u32, sample_rate: u32) -> Result<String, JsValue> {
    match analysis::compute_spectral_balance_6band(samples, sample_rate, channels as usize) {
        Some(bands) => serde_json::to_string(&bands).map_err(js),
        None => Ok(String::new()),
    }
}

/// Stage 5 — build the mastering context: the desktop's `SourceProfile`, via
/// the same constructor `SourceProfile::from_analysis` uses. Empty string when
/// no profile can be derived (the chain then runs exactly as the app's own
/// "no analysis" path).
#[wasm_bindgen]
pub fn build_profile(tonal_json: &str, dynamics_db: f32, lra_lu: f32, correlation: f32, width: f32) -> Result<String, JsValue> {
    let spectral_6 = if tonal_json.is_empty() {
        None
    } else {
        Some(serde_json::from_str::<SpectralBalance6>(tonal_json).map_err(js)?)
    };
    let profile = SourceProfile::from_measurements(
        spectral_6,
        if dynamics_db.is_finite() { Some(dynamics_db) } else { None },
        if lra_lu.is_finite() { lra_lu } else { 0.0 },
        if correlation.is_finite() { Some(correlation) } else { None },
        width,
    );
    match profile {
        Some(p) => serde_json::to_string(&p).map_err(js),
        None => Ok(String::new()),
    }
}

/// Plain-language digest of a profile for the UI ("bright 0.31 / low 0.28 / …").
#[wasm_bindgen]
pub fn profile_digest(profile_json: &str) -> Result<String, JsValue> {
    if profile_json.is_empty() {
        return Ok(String::new());
    }
    let p: SourceProfile = serde_json::from_str(profile_json).map_err(js)?;
    Ok(p.digest())
}

/// Integrated LUFS + true peak of interleaved f32 audio. Returns `[lufs, tp]`.
#[wasm_bindgen]
pub fn measure_loudness(samples: &[f32], channels: u32, sample_rate: u32) -> Result<Vec<f32>, JsValue> {
    let (l, t) = measure(samples, channels, sample_rate).map_err(js)?;
    Ok(vec![l, t])
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// One mastered excerpt plus the measurement of what was delivered. The
/// landing stage already measured the chain output once; the post-landing
/// numbers are that measurement shifted by the applied gain, so the caller
/// never needs a second loudness pass.
#[wasm_bindgen]
pub struct Render {
    samples: Vec<f32>,
    lufs: f32,
    tp: f32,
}

#[wasm_bindgen]
impl Render {
    #[wasm_bindgen(getter)]
    pub fn lufs(&self) -> f32 {
        self.lufs
    }
    #[wasm_bindgen(getter)]
    pub fn tp(&self) -> f32 {
        self.tp
    }
    /// Take the samples out (consumes the object).
    pub fn samples(self) -> Vec<f32> {
        self.samples
    }
}

/// Master interleaved f32 audio with a Standard style, intensity 0..1 and an
/// absolute LUFS target (−14 / −11 / −9 in Standard), landed under the −1 dBTP
/// ceiling. `source_lufs` is the WHOLE-TRACK integrated loudness from
/// `analyze_loudness` (the desktop injects the same value); `profile_json` is
/// the `build_profile` output, or empty for the app's no-analysis path.
#[wasm_bindgen]
pub fn master_standard(
    samples: &[f32],
    channels: u32,
    sample_rate: u32,
    style: &str,
    intensity: f32,
    target_lufs: f32,
    source_lufs: f32,
    profile_json: &str,
) -> Result<Render, JsValue> {
    let ch = channels.max(1) as usize;
    let source_profile = if profile_json.is_empty() {
        None
    } else {
        Some(serde_json::from_str::<SourceProfile>(profile_json).map_err(js)?)
    };
    let settings = MasteringSettings {
        preset: preset_for(style),
        intensity,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: if source_lufs.is_finite() { Some(source_lufs) } else { None },
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings {
            lufs_offset_db: Some(target_lufs),
            // The app's Track Master sends exactly this (useTrackMaster.ts
            // `adaptive_strength: 0.5`), and resolves confidence and
            // compression guards to None while their owner gates are off.
            adaptive_strength: Some(guardrails::ADAPTIVE_STRENGTH_DEFAULT),
            source_profile,
            source_confidence: None,
            compression_guards: None,
            ..AdvancedSettings::default()
        },
    };
    let mut out = samples.to_vec();
    let mut chain = dsp::MasteringChain::new(sample_rate, ch, &settings);
    // Same entry point the desktop render uses (engine.rs walks the track in
    // 4096-frame chunks only to report progress; the DSP is identical).
    chain.process_interleaved(&mut out, ch);
    chain.flush_render_tail(&mut out, ch);
    let (lufs, tp) = measure(&out, channels, sample_rate).map_err(js)?;
    let d = landing_delta_db(lufs, tp, target_lufs, settings.effective_ceiling_dbtp());
    if d != 0.0 {
        let g = 10f32.powf(d / 20.0);
        for s in out.iter_mut() {
            *s *= g;
        }
    }
    Ok(Render { samples: out, lufs: lufs + d, tp: tp + d })
}

#[wasm_bindgen]
pub fn version() -> String {
    format!("yes-master-web/0.2.0 (standard chain + source analysis; {})", env!("YES_TRYIT_SOURCE_STAMP"))
}
