//! Shared integration-test fixtures. Declare `mod common;` in a test binary
//! to use them; binaries that don't declare it don't compile this file.

use yes_master_lib::types::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};

/// The canonical neutral settings used across integration tests: Universal
/// preset at intensity 0.5, flat EQ, Custom delivery, no album plan, default
/// advanced block. When `MasteringSettings` gains a field, this is the one
/// test builder to update (the per-preset builders in the preset_* binaries
/// are intentionally scenario-tuned and stay local).
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
