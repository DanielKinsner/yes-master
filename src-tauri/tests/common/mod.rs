//! Shared integration-test fixtures. Declare `mod common;` in a test binary
//! to use them; binaries that don't declare it don't compile this file.
//!
//! Each test binary compiles this whole module but uses its own subset of
//! fixtures, so the per-binary dead_code lint would otherwise fail clippy's
//! `-D warnings` for every binary that skips one helper.
#![allow(dead_code)]

use yes_master_lib::types::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};

/// The canonical neutral settings used across integration tests: Universal
/// preset at intensity 0.5, flat EQ, Custom delivery, no album plan, default
/// advanced block. When `MasteringSettings` gains a field, this is the one
/// test builder to update (the per-preset builders in the preset_* binaries
/// are intentionally scenario-tuned and stay local).
/// Hand-rolled RIFF/WAVE with attacker-controlled fmt fields and data.
/// Shared by the hostile-decode corpus (decode_hostile.rs) and the command
/// contract tests (contracts.rs) so both untrusted-parse entry points are
/// probed with the same crafted bytes.
pub fn crafted_wav(
    channels: u16,
    sample_rate: u32,
    bits: u16,
    data: &[u8],
    data_size_lie: Option<u32>,
) -> Vec<u8> {
    let block_align = channels.saturating_mul(bits / 8).max(1);
    let byte_rate = sample_rate.saturating_mul(block_align as u32);
    let data_size = data_size_lie.unwrap_or(data.len() as u32);
    let riff_size = 36u32.saturating_add(data_size);
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    out.extend_from_slice(data);
    out
}

pub fn default_master_settings() -> MasteringSettings {
    MasteringSettings {
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
    }
}
