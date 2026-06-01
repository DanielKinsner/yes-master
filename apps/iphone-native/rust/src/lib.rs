use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use yes_master_lib::{
    AdvancedSettings, CompressionMode, DeliveryProfile, MasteringSettings, Preset,
};

const VERSION: &[u8] = b"yes-master-iphone-native-bridge/0.1.0\0";
const SUPPORTED_IMPORT_EXTENSIONS: &[&str] = &["wav", "mp3", "m4a", "aac", "flac", "ogg"];

#[no_mangle]
pub extern "C" fn yes_master_native_bridge_version() -> *const c_char {
    VERSION.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn yes_master_native_supports_import_extension(
    extension: *const c_char,
) -> bool {
    let Some(extension) = normalize_extension(extension) else {
        return false;
    };
    SUPPORTED_IMPORT_EXTENSIONS.contains(&extension.as_str())
}

#[no_mangle]
pub extern "C" fn yes_master_native_fixed_export_settings_json() -> *mut c_char {
    string_to_ffi(
        serde_json::to_string(&fixed_export_settings())
            .unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#)),
    )
}

#[no_mangle]
pub unsafe extern "C" fn yes_master_native_free_string(value: *mut c_char) {
    if !value.is_null() {
        let _ = CString::from_raw(value);
    }
}

fn fixed_export_settings() -> MasteringSettings {
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
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings {
            lufs_offset_db: Some(-11.0),
            ceiling_dbtp: Some(-1.0),
            width: None,
            warmth: None,
            presence_air: None,
            compression_mode: CompressionMode::Preset,
            compression_density: None,
            compression_low_threshold_db: None,
            compression_low_ratio: None,
            compression_low_attack_ms: None,
            compression_low_release_ms: None,
            compression_mid_threshold_db: None,
            compression_mid_ratio: None,
            compression_mid_attack_ms: None,
            compression_mid_release_ms: None,
            compression_high_threshold_db: None,
            compression_high_ratio: None,
            compression_high_attack_ms: None,
            compression_high_release_ms: None,
            compression_link_stereo: None,
            bit_depth: Some(24),
            target_sample_rate: Some(44_100),
        },
    }
}

unsafe fn normalize_extension(extension: *const c_char) -> Option<String> {
    if extension.is_null() {
        return None;
    }
    let raw = CStr::from_ptr(extension).to_str().ok()?;
    let normalized = raw.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn string_to_ffi(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("{}").expect("static JSON has no nul bytes"))
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_native_import_filter_aligned_to_shared_decoder_support() {
        for extension in ["wav", "mp3", "m4a", "aac", "flac", "ogg"] {
            let extension = CString::new(extension).unwrap();
            assert!(unsafe { yes_master_native_supports_import_extension(extension.as_ptr()) });
        }

        for extension in ["aiff", "aif", "opus", "caf", "alac"] {
            let extension = CString::new(extension).unwrap();
            assert!(!unsafe { yes_master_native_supports_import_extension(extension.as_ptr()) });
        }
    }

    #[test]
    fn fixed_export_settings_match_simple_iphone_target() {
        let settings = fixed_export_settings();

        assert_eq!(settings.effective_target_lufs(), Some(-11.0));
        assert_eq!(settings.effective_ceiling_dbtp(), -1.0);
        assert_eq!(settings.effective_bit_depth(), 24);
        assert_eq!(settings.effective_sample_rate(48_000), 44_100);
        assert!(!settings.volume_match);
    }

    #[test]
    fn fixed_export_settings_json_uses_shared_contract_shape() {
        let json = serde_json::to_string(&fixed_export_settings()).unwrap();

        assert!(json.contains(r#""delivery_profile":"custom""#));
        assert!(json.contains(r#""lufs_offset_db":-11.0"#));
        assert!(json.contains(r#""target_sample_rate":44100"#));
    }
}
