//! YES Master in the browser. The Standard chain, verbatim, compiled to wasm.
//! `types.rs` and `dsp.rs` are included by path from the desktop crate so the
//! browser masters exactly what the app masters. Landing math mirrors
//! `engine::ceiling_bounded_landing_delta_db`.
#![allow(dead_code)]

#[path = "../../../../src-tauri/src/types.rs"]
pub mod types;
#[path = "../../../../src-tauri/src/dsp.rs"]
pub mod dsp;
#[path = "../../../../src-tauri/src/export_format.rs"]
pub mod export_format;
mod stubs;
pub use stubs::{confidence, deep_analysis, guardrails, mp3};

use ebur128::{EbuR128, Mode};
use types::*;
use wasm_bindgen::prelude::*;

fn sanitize_lufs(v: f32) -> f32 { if v.is_finite() { v } else { -70.0 } }

fn landing_delta_db(measured_lufs: f32, measured_tp: f32, target_lufs: f32, ceiling_dbtp: f32) -> f32 {
    if !target_lufs.is_finite() || !measured_lufs.is_finite() || measured_lufs <= -70.0 { return 0.0; }
    let delta = target_lufs - measured_lufs;
    let cap = if measured_tp.is_finite() { ceiling_dbtp - measured_tp } else { 0.0 };
    let d = delta.min(cap);
    if d.abs() > 1.0e-4 { d } else { 0.0 }
}

fn measure(samples: &[f32], channels: u32, sample_rate: u32) -> Result<(f32, f32), String> {
    let mut ebu = EbuR128::new(channels, sample_rate, Mode::I | Mode::TRUE_PEAK).map_err(|e| e.to_string())?;
    ebu.add_frames_f32(samples).map_err(|e| e.to_string())?;
    let lufs = sanitize_lufs(ebu.loudness_global().map_err(|e| e.to_string())? as f32);
    let mut peak = 0.0f64;
    for ch in 0..channels { let tp = ebu.true_peak(ch).map_err(|e| e.to_string())?; if tp > peak { peak = tp; } }
    let tp = if peak > 0.0 { (20.0 * peak.log10()) as f32 } else { -60.0 };
    Ok((lufs, tp))
}

fn preset_for(style: &str) -> Preset {
    match style {
        "clarity" => Preset::Clarity,
        "tape" => Preset::Tape,
        "oomph" => Preset::Oomph,
        _ => Preset::Universal,
    }
}

/// Measure integrated LUFS + true peak of interleaved f32 audio. Returns [lufs, tp].
#[wasm_bindgen]
pub fn measure_loudness(samples: &[f32], channels: u32, sample_rate: u32) -> Result<Vec<f32>, JsValue> {
    let (l, t) = measure(samples, channels, sample_rate).map_err(|e| JsValue::from_str(&e))?;
    Ok(vec![l, t])
}

/// Master interleaved f32 audio with a Standard style, intensity 0..1 and a
/// LUFS target (-14 / -11 / -9 in Standard). Returns the mastered samples,
/// same length and layout, already landed on the target under a -1 dBTP ceiling.
#[wasm_bindgen]
pub fn master_standard(samples: &[f32], channels: u32, sample_rate: u32, style: &str, intensity: f32, target_lufs: f32) -> Result<Vec<f32>, JsValue> {
    let ch = channels.max(1) as usize;
    let (source_lufs, _) = measure(samples, channels, sample_rate).map_err(|e| JsValue::from_str(&e))?;
    let settings = MasteringSettings {
        preset: preset_for(style),
        intensity,
        eq_sub_db: 0.0, eq_low_db: 0.0, eq_low_mid_db: 0.0, eq_mid_db: 0.0,
        eq_high_mid_db: 0.0, eq_high_db: 0.0, eq_sparkle_db: 0.0,
        eq_bands: EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: Some(source_lufs),
        input_gain_db: 0.0, output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings { lufs_offset_db: Some(target_lufs), ..AdvancedSettings::default() },
    };
    let mut out = samples.to_vec();
    let mut chain = dsp::MasteringChain::new(sample_rate, ch, &settings);
    let frames = out.len() / ch;
    let mut frame = vec![0f32; ch];
    for i in 0..frames {
        frame.copy_from_slice(&out[i * ch..(i + 1) * ch]);
        chain.process_frame_inplace(&mut frame);
        out[i * ch..(i + 1) * ch].copy_from_slice(&frame);
    }
    chain.flush_render_tail(&mut out, ch);
    let (lufs, tp) = measure(&out, channels, sample_rate).map_err(|e| JsValue::from_str(&e))?;
    let d = landing_delta_db(lufs, tp, target_lufs, settings.effective_ceiling_dbtp());
    if d != 0.0 { let g = 10f32.powf(d / 20.0); for s in out.iter_mut() { *s *= g; } }
    Ok(out)
}

#[wasm_bindgen]
pub fn version() -> String { "yes-master-web/0.1.0 (standard chain)".into() }
