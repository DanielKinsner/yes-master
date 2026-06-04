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

mod live_stream;

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

pub(crate) fn export_settings_for_options(
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
        "warm" | "warmth" => Preset::Warmth,
        "open" | "clarity" => Preset::Clarity,
        "punch" => Preset::Punch,
        "balanced" | "universal" => Preset::Universal,
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
        let warm = export_settings_for_options(Some("warm"), 0.8, -11.0);
        assert_eq!(warm.preset, Preset::Warmth);
        assert_eq!(warm.intensity, 0.8);

        let open = export_settings_for_options(Some("open"), 2.0, -11.0);
        assert_eq!(open.preset, Preset::Clarity);
        assert_eq!(open.intensity, 1.0);

        let fallback = export_settings_for_options(Some("unknown"), -1.0, -11.0);
        assert_eq!(fallback.preset, Preset::Universal);
        assert_eq!(fallback.intensity, 0.0);
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
        if yes_master_lib::confidence::CONFIDENCE_GATING_ENABLED {
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
}
