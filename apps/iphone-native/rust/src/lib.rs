use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Arc;

use yes_master_lib::{
    engine::{analyze_tracks_core, mastering_render, AnalyzeRequest},
    profile_store::{apply_resolved_confidence, apply_resolved_profile},
    AdvancedSettings, AnalysisResult, CompressionMode, DeliveryProfile, MasteringSettings, Preset,
    RenderKind, SourceProfile, TrackId,
};

// `pub` so the Android bridge crate (which depends on this facade as an
// rlib) can drive the same live-audition state via Rust paths instead of
// duplicating it. Visibility-only: the C ABI and iPhone build are unchanged
// (same precedent as `export_settings_for_options`).
pub mod live_stream;

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
pub unsafe extern "C" fn yes_master_native_analyze_file_json(path: *const c_char) -> *mut c_char {
    let Some(path) = ffi_string(path) else {
        return error_to_ffi("missing path");
    };

    if path.trim().is_empty() {
        return error_to_ffi("missing path");
    }

    let request = AnalyzeRequest {
        id: TrackId::new(),
        path,
    };

    // The native bridge has no Tauri runtime, so it calls the State-free core
    // (the `analyze_tracks` command takes a `tauri::State<SourceProfileStore>`
    // that can't exist here). Use the same deep-capable analysis entry as
    // desktop; `DeepAnalysis` stays Rust-internal via serde(skip), but render/live
    // settings can resolve adaptive profile + confidence from it below.
    match futures_executor::block_on(analyze_tracks_core(vec![request])) {
        Ok(mut results) => {
            if let Some(result) = results.pop() {
                string_to_ffi(serde_json::to_string(&result).unwrap_or_else(|error| {
                    format!(r#"{{"error":"analysis serialization failed: {error}"}}"#)
                }))
            } else {
                error_to_ffi("analysis returned no result")
            }
        }
        Err(error) => error_to_ffi(&error.to_string()),
    }
}

#[no_mangle]
pub unsafe extern "C" fn yes_master_native_render_master_json(
    source_path: *const c_char,
    output_dir: *const c_char,
) -> *mut c_char {
    yes_master_native_render_master_with_options_json(
        source_path,
        output_dir,
        std::ptr::null(),
        0.5,
        -11.0,
    )
}

#[no_mangle]
pub unsafe extern "C" fn yes_master_native_render_master_with_options_json(
    source_path: *const c_char,
    output_dir: *const c_char,
    preset: *const c_char,
    intensity: f32,
    lufs_target: f32,
) -> *mut c_char {
    let Some(source_path) = ffi_string(source_path) else {
        return error_to_ffi("missing source path");
    };
    let Some(output_dir) = ffi_string(output_dir) else {
        return error_to_ffi("missing output directory");
    };

    if source_path.trim().is_empty() {
        return error_to_ffi("missing source path");
    }
    if output_dir.trim().is_empty() {
        return error_to_ffi("missing output directory");
    }

    let source_path = Path::new(&source_path);
    let output_dir = Path::new(&output_dir);
    let adaptive_context = native_adaptive_context_for_path(source_path);
    let settings = export_settings_for_options_with_context(
        unsafe { ffi_string(preset) }.as_deref(),
        intensity,
        lufs_target,
        adaptive_context.as_ref(),
    );

    if let Err(error) = std::fs::create_dir_all(output_dir) {
        return error_to_ffi(&error.to_string());
    }

    match mastering_render(
        TrackId::new(),
        source_path,
        &settings,
        output_dir,
        RenderKind::Master,
    ) {
        Ok(job) => string_to_ffi(serde_json::to_string(&job).unwrap_or_else(|error| {
            format!(r#"{{"error":"render serialization failed: {error}"}}"#)
        })),
        Err(error) => error_to_ffi(&error.to_string()),
    }
}

#[no_mangle]
pub unsafe extern "C" fn yes_master_native_free_string(value: *mut c_char) {
    if !value.is_null() {
        let _ = CString::from_raw(value);
    }
}

fn fixed_export_settings() -> MasteringSettings {
    export_settings_for_options(Some("balanced"), 0.5, -11.0)
}

#[derive(Clone)]
pub(crate) struct NativeAdaptiveContext {
    source_lufs_integrated: f32,
    source_profile: Option<SourceProfile>,
    deep_analysis: Option<Arc<yes_master_lib::deep_analysis::DeepAnalysis>>,
}

fn native_adaptive_context_from_analysis(analysis: &AnalysisResult) -> NativeAdaptiveContext {
    NativeAdaptiveContext {
        source_lufs_integrated: analysis.lufs_integrated,
        source_profile: SourceProfile::from_analysis(analysis),
        deep_analysis: analysis.deep_analysis.clone(),
    }
}

pub(crate) fn native_adaptive_context_for_path(path: &Path) -> Option<NativeAdaptiveContext> {
    let request = AnalyzeRequest {
        id: TrackId::new(),
        path: path.to_string_lossy().into_owned(),
    };
    futures_executor::block_on(analyze_tracks_core(vec![request]))
        .ok()
        .and_then(|mut results| results.pop())
        .map(|analysis| native_adaptive_context_from_analysis(&analysis))
}

/// `pub` (not just crate-visible) since 2026-06-10: the Android bridge crate
/// consumes this facade as an rlib and asserts the same style→preset parity
/// fixture against it. Visibility-only change; behavior untouched.
pub fn export_settings_for_options(
    preset: Option<&str>,
    intensity: f32,
    lufs_target: f32,
) -> MasteringSettings {
    export_settings_for_options_with_context(preset, intensity, lufs_target, None)
}

pub(crate) fn export_settings_for_options_with_context(
    preset: Option<&str>,
    intensity: f32,
    lufs_target: f32,
    adaptive_context: Option<&NativeAdaptiveContext>,
) -> MasteringSettings {
    // The native bridge has no Tauri `run()` startup hook. Mirror desktop/headless
    // startup here so YES_MASTER_CONFIDENCE_GATING=1 can exercise Phase B.
    yes_master_lib::confidence::init_confidence_gating_from_env();
    let mut settings = MasteringSettings {
        preset: native_preset(preset),
        intensity: intensity.clamp(0.0, 1.0),
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
            // Under DeliveryProfile::Custom this is read back as the absolute LUFS
            // target (see MasteringSettings::effective_target_lufs), not a relative
            // offset despite the field name. We pass the loudness target here.
            lufs_offset_db: Some(lufs_target.clamp(-24.0, -6.0)),
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
            // Keep future desktop fields compiling through the bridge. Adaptive
            // profile/confidence are resolved from `adaptive_context` immediately
            // below, matching desktop's backend-owned command-layer injection.
            ..Default::default()
        },
    };
    if let Some(context) = adaptive_context {
        settings.source_lufs_integrated = Some(context.source_lufs_integrated);
        apply_resolved_profile(&mut settings, context.source_profile, false);
        apply_resolved_confidence(&mut settings, context.deep_analysis.clone(), false);
    }
    settings
}

fn native_preset(preset: Option<&str>) -> Preset {
    match preset
        .unwrap_or("balanced")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "balanced" | "universal" => Preset::Universal,
        "bright" | "clarity" | "open" => Preset::Clarity,
        "warm" | "tape" => Preset::Tape,
        "heavy" | "oomph" => Preset::Oomph,
        // Back-compat aliases for older builds / saved payloads.
        "warmth" => Preset::Warmth,
        "punch" => Preset::Punch,
        _ => Preset::Universal,
    }
}

unsafe fn normalize_extension(extension: *const c_char) -> Option<String> {
    let raw = ffi_string(extension)?;
    let normalized = raw.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub(crate) unsafe fn ffi_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    Some(CStr::from_ptr(value).to_str().ok()?.to_owned())
}

fn string_to_ffi(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("{}").expect("static JSON has no nul bytes"))
        .into_raw()
}

fn error_to_ffi(message: &str) -> *mut c_char {
    string_to_ffi(serde_json::json!({ "error": message }).to_string())
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
    fn native_options_map_loudness_target() {
        let low = export_settings_for_options(Some("balanced"), 0.5, -14.0);
        assert_eq!(low.effective_target_lufs(), Some(-14.0));

        let high = export_settings_for_options(Some("balanced"), 0.5, -9.0);
        assert_eq!(high.effective_target_lufs(), Some(-9.0));

        // out-of-range is clamped to a safe mastering window
        let clamped = export_settings_for_options(Some("balanced"), 0.5, 5.0);
        assert_eq!(clamped.effective_target_lufs(), Some(-6.0));
    }

    #[test]
    fn native_options_map_to_shared_preset_and_intensity() {
        let balanced = export_settings_for_options(Some("balanced"), 0.5, -11.0);
        assert_eq!(balanced.preset, Preset::Universal);

        let bright = export_settings_for_options(Some("bright"), 0.5, -11.0);
        assert_eq!(bright.preset, Preset::Clarity);

        let warm = export_settings_for_options(Some("warm"), 0.8, -11.0);
        assert_eq!(warm.preset, Preset::Tape);
        assert_eq!(warm.intensity, 0.8);

        let heavy = export_settings_for_options(Some("heavy"), 1.0, -11.0);
        assert_eq!(heavy.preset, Preset::Oomph);

        // Back-compat aliases still resolve.
        assert_eq!(
            export_settings_for_options(Some("open"), 0.5, -11.0).preset,
            Preset::Clarity
        );
        assert_eq!(
            export_settings_for_options(Some("warmth"), 0.5, -11.0).preset,
            Preset::Warmth
        );
        assert_eq!(
            export_settings_for_options(Some("punch"), 0.5, -11.0).preset,
            Preset::Punch
        );

        let fallback = export_settings_for_options(Some("unknown"), -1.0, -11.0);
        assert_eq!(fallback.preset, Preset::Universal);
        assert_eq!(fallback.intensity, 0.0);
    }

    /// Cross-language pin: src/standard-mapping-parity.json is the canonical
    /// style→preset contract, asserted here AND by
    /// src/lib/standard-mapping.test.ts on the desktop side, so this
    /// `native_preset` alias map and the Standard view's tables cannot drift
    /// apart without one side failing. (The loudness trio in the fixture is
    /// mapped by Swift's NativeLoudness into the `lufs_target` FFI argument;
    /// its bridge-side behavior is pinned by the loudness-target test above.)
    #[test]
    fn standard_style_aliases_match_the_shared_parity_fixture() {
        let parity: serde_json::Value =
            serde_json::from_str(include_str!("../../../../src/standard-mapping-parity.json"))
                .expect("parse standard-mapping-parity.json");
        let styles = parity["styles"]
            .as_object()
            .expect("styles map in parity fixture");
        assert!(!styles.is_empty());
        for (style, expected_kind) in styles {
            let preset = native_preset(Some(style));
            let kind = serde_json::to_value(&preset).expect("serialize preset")["kind"]
                .as_str()
                .expect("preset kind tag")
                .to_string();
            assert_eq!(
                &kind,
                expected_kind.as_str().expect("kind string"),
                "native_preset(\"{style}\") diverged from the shared parity fixture"
            );
        }
    }

    #[test]
    fn native_adaptive_context_injects_desktop_profile_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);

        let context = native_adaptive_context_for_path(&input).expect("adaptive context");
        let settings =
            export_settings_for_options_with_context(Some("balanced"), 0.5, -11.0, Some(&context));

        assert!(
            settings.advanced.source_profile.is_some(),
            "native bridge must inject the same backend-derived profile desktop uses"
        );
        assert_eq!(
            settings.source_lufs_integrated,
            Some(context.source_lufs_integrated)
        );
        if yes_master_lib::confidence::is_confidence_gating_enabled() {
            assert!(
                settings.advanced.source_confidence.is_some(),
                "when Phase B is enabled, native settings must carry resolved confidence"
            );
        } else {
            assert!(
                settings.advanced.source_confidence.is_none(),
                "owner gate off keeps confidence inert, matching desktop"
            );
        }
    }

    #[test]
    fn fixed_export_settings_json_uses_shared_contract_shape() {
        let json = serde_json::to_string(&fixed_export_settings()).unwrap();

        assert!(json.contains(r#""delivery_profile":"custom""#));
        assert!(json.contains(r#""lufs_offset_db":-11.0"#));
        assert!(json.contains(r#""target_sample_rate":44100"#));
    }

    #[test]
    fn analyze_file_json_returns_error_for_missing_file() {
        let missing_path = CString::new("/tmp/yes-master-native-missing.wav").unwrap();

        let pointer = unsafe { yes_master_native_analyze_file_json(missing_path.as_ptr()) };
        assert!(!pointer.is_null());

        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };

        assert!(json.contains(r#""error""#), "got {json}");
        assert!(json.contains("source file not found"), "got {json}");
    }

    #[test]
    fn render_master_json_writes_fixed_target_wav() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        let output_dir = tmp.path().join("rendered");
        write_sine_wav(&input);

        let input = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let output_dir_c = CString::new(output_dir.to_string_lossy().as_bytes()).unwrap();
        let pointer =
            unsafe { yes_master_native_render_master_json(input.as_ptr(), output_dir_c.as_ptr()) };
        assert!(!pointer.is_null());

        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();
        let output = std::path::PathBuf::from(
            payload["output_paths"][0]
                .as_str()
                .expect("render output path"),
        );

        assert!(json.contains(r#""kind":"master""#), "got {json}");
        assert!(output.exists(), "rendered WAV was not written");

        let reader = hound::WavReader::open(&output).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, 44_100);
        assert_eq!(spec.bits_per_sample, 24);
    }

    #[test]
    fn render_master_json_records_adaptive_source_digest() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        let output_dir = tmp.path().join("rendered");
        write_sine_wav(&input);

        let input = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let output_dir_c = CString::new(output_dir.to_string_lossy().as_bytes()).unwrap();
        let pointer =
            unsafe { yes_master_native_render_master_json(input.as_ptr(), output_dir_c.as_ptr()) };
        assert!(!pointer.is_null());

        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(payload["measurements"]["effective_adaptive_strength"], 0.5);
        assert!(
            payload["measurements"]["source_profile_digest"].is_string(),
            "native render should be traceable to its source profile: {json}"
        );
    }

    #[test]
    fn rendered_wav_can_be_analyzed_again() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        let output_dir = tmp.path().join("rendered");
        write_sine_wav(&input);

        let rendered = render_master_for_test(&input, &output_dir);
        let rendered = CString::new(rendered.to_string_lossy().as_bytes()).unwrap();
        let pointer = unsafe { yes_master_native_analyze_file_json(rendered.as_ptr()) };
        assert!(!pointer.is_null());

        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };

        assert!(!json.contains(r#""error""#), "got {json}");
        assert!(json.contains("lufs_integrated"), "got {json}");
    }

    /// Wire-key pin: `NativeAnalysisResult` (NativeMasteringBridge.swift)
    /// decodes exactly these keys via `convertFromSnakeCase`. A
    /// `#[serde(rename)]` or field rename on `AnalysisResult` compiles fine
    /// on both sides and then crashes the app at runtime — this makes it
    /// fail here instead, on the real FFI wire.
    #[test]
    fn analyze_json_carries_every_key_swift_decodes() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);

        let input_c = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let pointer = unsafe { yes_master_native_analyze_file_json(input_c.as_ptr()) };
        assert!(!pointer.is_null());
        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();

        for key in ["lufs_integrated", "true_peak_dbtp", "dynamic_range_lu"] {
            assert!(
                payload[key].is_number(),
                "analyze JSON lost Swift-decoded key `{key}`: {json}"
            );
        }
    }

    /// Wire-key pin for the render side: `NativeRenderJob` decodes
    /// `output_paths` + `measurements`, and `NativeRenderedMeasurements`
    /// decodes the five keys below. Same runtime-crash class as the analyze
    /// pin above.
    #[test]
    fn render_json_carries_every_key_swift_decodes() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        let output_dir = tmp.path().join("rendered");
        write_sine_wav(&input);

        let input_c = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let output_dir_c = CString::new(output_dir.to_string_lossy().as_bytes()).unwrap();
        let pointer = unsafe {
            yes_master_native_render_master_json(input_c.as_ptr(), output_dir_c.as_ptr())
        };
        assert!(!pointer.is_null());
        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(
            payload["output_paths"][0].is_string(),
            "render JSON lost Swift-decoded key `output_paths`: {json}"
        );
        let measurements = &payload["measurements"];
        assert!(
            measurements.is_object(),
            "render JSON lost Swift-decoded key `measurements`: {json}"
        );
        for key in ["lufs_integrated", "true_peak_dbtp", "dynamic_range_lu"] {
            assert!(
                measurements[key].is_number(),
                "measurements lost Swift-decoded key `{key}`: {json}"
            );
        }
        for key in ["sample_rate", "bit_depth"] {
            assert!(
                measurements[key].as_u64().is_some(),
                "measurements lost Swift-decoded integer key `{key}`: {json}"
            );
        }
    }

    #[test]
    fn render_master_json_creates_unique_outputs_in_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        let output_dir = tmp.path().join("rendered");
        write_sine_wav(&input);

        let first = render_master_for_test(&input, &output_dir);
        let second = render_master_for_test(&input, &output_dir);

        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());
    }

    #[test]
    fn render_master_json_never_overwrites_source_when_rendering_beside_it() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);
        let source_before = std::fs::read(&input).unwrap();

        let output = render_master_for_test(&input, tmp.path());
        let source_after = std::fs::read(&input).unwrap();

        assert_ne!(output, input);
        assert!(output.exists(), "rendered WAV was not written");
        assert_eq!(
            source_after, source_before,
            "source WAV changed during render"
        );
    }

    #[test]
    fn render_master_json_keeps_outputs_unique_when_source_name_repeats() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("My Mix 01.wav");
        let output_dir = tmp.path().join("Masters");
        write_sine_wav(&input);

        let first = render_master_for_test(&input, &output_dir);
        let second = render_master_for_test(&input, &output_dir);

        assert_ne!(first.file_name(), second.file_name());
        assert_eq!(
            first.extension().and_then(|value| value.to_str()),
            Some("wav")
        );
        assert_eq!(
            second.extension().and_then(|value| value.to_str()),
            Some("wav")
        );
    }

    #[test]
    fn render_master_with_options_writes_wav() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        let output_dir = tmp.path().join("rendered");
        write_sine_wav(&input);

        let input = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let output_dir = CString::new(output_dir.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("punch").unwrap();
        let pointer = unsafe {
            yes_master_native_render_master_with_options_json(
                input.as_ptr(),
                output_dir.as_ptr(),
                preset.as_ptr(),
                0.9,
                -9.0,
            )
        };
        assert!(!pointer.is_null());

        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();
        let output = std::path::PathBuf::from(
            payload["output_paths"][0]
                .as_str()
                .expect("render output path"),
        );

        assert!(output.exists(), "rendered WAV was not written");
    }

    fn render_master_for_test(
        input: &std::path::Path,
        output_dir: &std::path::Path,
    ) -> std::path::PathBuf {
        let input = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let output_dir = CString::new(output_dir.to_string_lossy().as_bytes()).unwrap();
        let pointer =
            unsafe { yes_master_native_render_master_json(input.as_ptr(), output_dir.as_ptr()) };
        assert!(!pointer.is_null());

        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();
        std::path::PathBuf::from(
            payload["output_paths"][0]
                .as_str()
                .expect("render output path"),
        )
    }

    #[test]
    #[ignore = "timing proxy; run with --release -- --ignored --nocapture"]
    fn timing_proxy_analyze_and_render() {
        use std::time::Instant;
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("timing.wav");
        let output_dir = tmp.path().join("out");
        write_long_sine_wav(&input, 30); // 30 seconds, 44.1k stereo

        let path = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let t0 = Instant::now();
        let analysis = unsafe { yes_master_native_analyze_file_json(path.as_ptr()) };
        let analyze_ms = t0.elapsed().as_millis();
        unsafe { yes_master_native_free_string(analysis) };

        let out = render_master_for_test(&input, &output_dir);
        let render_ms = t0.elapsed().as_millis() - analyze_ms;
        assert!(out.exists());
        eprintln!("TIMING analyze={analyze_ms}ms render={render_ms}ms (30s source)");
    }

    fn write_long_sine_wav(path: &std::path::Path, seconds: u32) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let frames = 44_100 * seconds;
        for n in 0..frames {
            let t = n as f32 / 44_100.0;
            let sample = (t * 220.0 * std::f32::consts::TAU).sin() * 0.2;
            let value = (sample * i16::MAX as f32) as i16;
            writer.write_sample(value).unwrap();
            writer.write_sample(value).unwrap();
        }
        writer.finalize().unwrap();
    }

    fn write_sine_wav(path: &std::path::Path) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for n in 0..22_050 {
            let t = n as f32 / 44_100.0;
            let sample = (t * 440.0 * std::f32::consts::TAU).sin() * 0.2;
            let value = (sample * i16::MAX as f32) as i16;
            writer.write_sample(value).unwrap();
            writer.write_sample(value).unwrap();
        }
        writer.finalize().unwrap();
    }

    fn read_wav_samples(path: &std::path::Path) -> (hound::WavSpec, Vec<i32>) {
        let mut reader = hound::WavReader::open(path).expect("open rendered wav");
        let spec = reader.spec();
        let samples = reader
            .samples::<i32>()
            .map(|s| s.expect("read sample"))
            .collect::<Vec<_>>();
        (spec, samples)
    }

    fn max_normalized_diff(a: &(hound::WavSpec, Vec<i32>), b: &(hound::WavSpec, Vec<i32>)) -> f64 {
        assert_eq!(a.0.sample_rate, b.0.sample_rate, "sample-rate mismatch");
        assert_eq!(a.0.channels, b.0.channels, "channel-count mismatch");
        assert_eq!(
            a.0.bits_per_sample, b.0.bits_per_sample,
            "bit-depth mismatch"
        );
        assert_eq!(a.1.len(), b.1.len(), "sample-count mismatch");
        let scale = (1i64 << (a.0.bits_per_sample - 1)) as f64;
        a.1.iter()
            .zip(&b.1)
            .map(|(x, y)| (*x as f64 - *y as f64).abs() / scale)
            .fold(0.0, f64::max)
    }

    // Task #3: the bridge's offline render (FFI -> mastering_render) must match
    // the shared render path sample-for-sample for an identical WAV + settings,
    // with a source-present adaptive context. Guards against drift in the FFI
    // parameter mapping or adaptive-context resolution that the existing
    // structural render tests would not catch. (Gate-off confidence inertness is
    // pinned deterministically in `confidence_resolver_is_inert_when_gate_off`.)
    #[test]
    fn bridge_render_matches_shared_render_path() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);

        // Source-profile-present adaptive context (the interesting case).
        let context = native_adaptive_context_for_path(&input);
        assert!(context.is_some(), "expected a resolved adaptive context");
        let settings =
            export_settings_for_options_with_context(Some("warm"), 0.5, -11.0, context.as_ref());
        assert!(
            settings.advanced.source_profile.is_some(),
            "render-path parity should exercise an injected source profile"
        );

        // Reference: render directly through the shared lib with those settings.
        let ref_dir = tmp.path().join("reference");
        std::fs::create_dir_all(&ref_dir).unwrap();
        let reference = mastering_render(
            TrackId::new(),
            &input,
            &settings,
            &ref_dir,
            RenderKind::Master,
        )
        .expect("reference render");
        let reference = read_wav_samples(&std::path::PathBuf::from(&reference.output_paths[0]));

        // Bridge: render the same input + matching args through the FFI.
        let bridge_dir = tmp.path().join("bridge");
        let input_c = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let bridge_dir_c = CString::new(bridge_dir.to_string_lossy().as_bytes()).unwrap();
        let preset_c = CString::new("warm").unwrap();
        let pointer = unsafe {
            yes_master_native_render_master_with_options_json(
                input_c.as_ptr(),
                bridge_dir_c.as_ptr(),
                preset_c.as_ptr(),
                0.5,
                -11.0,
            )
        };
        assert!(!pointer.is_null());
        let json = unsafe {
            let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            yes_master_native_free_string(pointer);
            value
        };
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();
        let bridge_out = std::path::PathBuf::from(
            payload["output_paths"][0]
                .as_str()
                .expect("bridge render output path"),
        );
        let bridge = read_wav_samples(&bridge_out);

        let diff = max_normalized_diff(&reference, &bridge);
        assert!(
            diff < 1e-6,
            "bridge render diverged from the shared render path by {diff} (normalized)"
        );
    }

    // Export forces Volume Match off (engine-side), so a VM-like option present
    // on the settings must not change the rendered output.
    #[test]
    fn export_render_is_invariant_to_volume_match() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);
        let context = native_adaptive_context_for_path(&input);

        let off = export_settings_for_options_with_context(
            Some("balanced"),
            0.5,
            -11.0,
            context.as_ref(),
        );
        let mut on = export_settings_for_options_with_context(
            Some("balanced"),
            0.5,
            -11.0,
            context.as_ref(),
        );
        on.volume_match = true;
        on.source_lufs_integrated = Some(-20.0);

        let off_dir = tmp.path().join("off");
        let on_dir = tmp.path().join("on");
        std::fs::create_dir_all(&off_dir).unwrap();
        std::fs::create_dir_all(&on_dir).unwrap();
        let off_job =
            mastering_render(TrackId::new(), &input, &off, &off_dir, RenderKind::Master).unwrap();
        let on_job =
            mastering_render(TrackId::new(), &input, &on, &on_dir, RenderKind::Master).unwrap();

        let off_wav = read_wav_samples(&std::path::PathBuf::from(&off_job.output_paths[0]));
        let on_wav = read_wav_samples(&std::path::PathBuf::from(&on_job.output_paths[0]));
        let diff = max_normalized_diff(&off_wav, &on_wav);
        assert!(
            diff < 1e-6,
            "render changed when a Volume Match-like option was present: {diff}"
        );
    }

    // Deterministic gate-off contract for the resolver the bridge depends on.
    // The pure resolver takes the gate (and album) as explicit params, so this
    // pins inertness without racing the process-wide CONFIDENCE_GATING atomic or
    // depending on the ambient YES_MASTER_CONFIDENCE_GATING env.
    #[test]
    fn confidence_resolver_is_inert_when_gate_off() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);
        let context = native_adaptive_context_for_path(&input).expect("adaptive context");
        let deep = context.deep_analysis.as_deref();

        // Gate off => no confidence, even with a real source DeepAnalysis present.
        assert!(
            yes_master_lib::confidence::resolve_source_confidence(deep, false, false).is_none(),
            "gate off must resolve no confidence"
        );
        // Album is non-adaptive even with the gate on.
        assert!(
            yes_master_lib::confidence::resolve_source_confidence(deep, true, true).is_none(),
            "album must resolve no confidence"
        );
    }

    // Documents the desktop-equivalent settings contract the native render builds
    // for a non-default preset/intensity with a source-present context, asserted
    // before render so a helper drift is caught directly (not only via the
    // render-output parity test).
    #[test]
    fn native_render_settings_match_desktop_contract() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source.wav");
        write_sine_wav(&input);
        let context = native_adaptive_context_for_path(&input).expect("adaptive context");

        let settings =
            export_settings_for_options_with_context(Some("warm"), 0.8, -9.0, Some(&context));

        // Preset / intensity / loudness mapping.
        assert_eq!(settings.preset, Preset::Tape);
        assert_eq!(settings.intensity, 0.8);
        assert_eq!(settings.effective_target_lufs(), Some(-9.0));
        // Fixed delivery contract for the phone target.
        assert_eq!(settings.delivery_profile, DeliveryProfile::Custom);
        assert_eq!(settings.effective_ceiling_dbtp(), -1.0);
        assert_eq!(settings.effective_bit_depth(), 24);
        assert_eq!(settings.effective_sample_rate(48_000), 44_100);
        // Export forces Volume Match off.
        assert!(!settings.volume_match);
        // Source-present adaptive context is injected like desktop's command layer.
        assert!(settings.advanced.source_profile.is_some());
        assert_eq!(
            settings.source_lufs_integrated,
            Some(context.source_lufs_integrated)
        );
    }
}
