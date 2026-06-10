//! Android native bridge.
//!
//! Thin JNI shims over the platform-neutral facade crate (the same
//! analyze/render C ABI the iPhone app drives through Swift FFI). The
//! contract is identical on purpose: paths in, JSON strings out, errors as
//! `{"error": "..."}` payloads — never an exception across the JNI
//! boundary. All real logic lives in `inner`, which is plain Rust so the
//! host test lane covers it without a JVM or a device.
//!
//! Live audition (oboe pulling from the facade's live_stream) lands in A3.

use std::ffi::{c_char, CStr, CString};

/// Plain-Rust core the JNI externs delegate to. Host tests target this
/// module directly.
pub mod inner {
    use super::*;

    fn consume_bridge_string(raw: *mut c_char) -> String {
        if raw.is_null() {
            return r#"{"error":"null bridge response"}"#.to_string();
        }
        // SAFETY: the facade allocated this CString and we free it through
        // the facade's own deallocator immediately after copying.
        let value = unsafe { CStr::from_ptr(raw) }.to_string_lossy().into_owned();
        unsafe { native_bridge::yes_master_native_free_string(raw) };
        value
    }

    fn json_error(message: &str) -> String {
        serde_json::json!({ "error": message }).to_string()
    }

    pub fn bridge_version() -> String {
        let ptr = native_bridge::yes_master_native_bridge_version();
        // SAFETY: static NUL-terminated string owned by the facade.
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
    }

    pub fn supports_import_extension(extension: &str) -> bool {
        let Ok(extension) = CString::new(extension) else {
            return false;
        };
        // SAFETY: valid NUL-terminated pointer for the duration of the call.
        unsafe { native_bridge::yes_master_native_supports_import_extension(extension.as_ptr()) }
    }

    pub fn analyze_file_json(path: &str) -> String {
        let Ok(path) = CString::new(path) else {
            return json_error("path contains an interior NUL byte");
        };
        // SAFETY: valid NUL-terminated pointer for the duration of the call.
        consume_bridge_string(unsafe {
            native_bridge::yes_master_native_analyze_file_json(path.as_ptr())
        })
    }

    pub fn render_master_with_options_json(
        source_path: &str,
        output_dir: &str,
        preset: Option<&str>,
        intensity: f32,
        lufs_target: f32,
    ) -> String {
        let Ok(source) = CString::new(source_path) else {
            return json_error("source path contains an interior NUL byte");
        };
        let Ok(out_dir) = CString::new(output_dir) else {
            return json_error("output dir contains an interior NUL byte");
        };
        let preset_c = preset.and_then(|p| CString::new(p).ok());
        let preset_ptr = preset_c
            .as_ref()
            .map_or(std::ptr::null(), |p| p.as_ptr());
        // SAFETY: all pointers are valid NUL-terminated strings (or null,
        // which the facade treats as "default preset") for the call.
        consume_bridge_string(unsafe {
            native_bridge::yes_master_native_render_master_with_options_json(
                source.as_ptr(),
                out_dir.as_ptr(),
                preset_ptr,
                intensity,
                lufs_target,
            )
        })
    }
}

/// JNI externs for `com.yesmaster.app.NativeBridge`. Deliberately dumb:
/// decode arguments, delegate to `inner`, encode the result. A JNI string
/// failure returns the error-JSON contract rather than throwing.
mod jni_shims {
    use super::inner;
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jfloat, jstring, JNI_FALSE, JNI_TRUE};
    use jni::JNIEnv;

    fn to_jstring(env: &mut JNIEnv, value: String) -> jstring {
        env.new_string(value)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    fn from_jstring(env: &mut JNIEnv, value: &JString) -> Option<String> {
        env.get_string(value).ok().map(|s| s.into())
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_bridgeVersion(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        to_jstring(&mut env, inner::bridge_version())
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_supportsImportExtension(
        mut env: JNIEnv,
        _class: JClass,
        extension: JString,
    ) -> jboolean {
        match from_jstring(&mut env, &extension) {
            Some(ext) if inner::supports_import_extension(&ext) => JNI_TRUE,
            _ => JNI_FALSE,
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_analyzeFileJson(
        mut env: JNIEnv,
        _class: JClass,
        path: JString,
    ) -> jstring {
        let result = match from_jstring(&mut env, &path) {
            Some(path) => inner::analyze_file_json(&path),
            None => r#"{"error":"invalid path string"}"#.to_string(),
        };
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_renderMasterWithOptionsJson(
        mut env: JNIEnv,
        _class: JClass,
        source_path: JString,
        output_dir: JString,
        preset: JString,
        intensity: jfloat,
        lufs_target: jfloat,
    ) -> jstring {
        let source = from_jstring(&mut env, &source_path);
        let out_dir = from_jstring(&mut env, &output_dir);
        // A null Java string arrives as a null JString; treat it as "no
        // preset" (the facade defaults to balanced/Universal).
        let preset = if preset.is_null() {
            None
        } else {
            from_jstring(&mut env, &preset)
        };
        let result = match (source, out_dir) {
            (Some(source), Some(out_dir)) => inner::render_master_with_options_json(
                &source,
                &out_dir,
                preset.as_deref(),
                intensity,
                lufs_target,
            ),
            _ => r#"{"error":"invalid path strings"}"#.to_string(),
        };
        to_jstring(&mut env, result)
    }
}

#[cfg(test)]
mod tests {
    use super::inner;
    use std::path::Path;

    fn write_sine_wav(path: &Path) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).expect("create wav");
        let omega = 2.0 * std::f32::consts::PI * 440.0 / 44_100.0;
        for i in 0..(44_100 * 2) {
            let v = (0.4 * (omega * i as f32).sin() * 32_767.0) as i16;
            writer.write_sample(v).expect("L");
            writer.write_sample(v).expect("R");
        }
        writer.finalize().expect("finalize");
    }

    #[test]
    fn version_links_the_shared_facade() {
        assert!(inner::bridge_version().contains("yes-master"));
    }

    #[test]
    fn extension_filter_matches_shared_decoder_support() {
        for ext in ["wav", "mp3", "m4a", "flac", "ogg"] {
            assert!(inner::supports_import_extension(ext), "{ext} should import");
        }
        assert!(!inner::supports_import_extension("pdf"));
    }

    /// Wire-key pin, Kotlin edition: NativeBridge.kt decodes exactly these
    /// keys from the analyze JSON (the same set Swift's
    /// NativeAnalysisResult pins on iPhone).
    #[test]
    fn analyze_json_carries_every_key_kotlin_decodes() {
        let tmp = tempfile::tempdir().unwrap();
        let wav = tmp.path().join("source.wav");
        write_sine_wav(&wav);
        let payload: serde_json::Value =
            serde_json::from_str(&inner::analyze_file_json(&wav.to_string_lossy())).unwrap();
        assert!(payload["error"].is_null(), "analyze errored: {payload}");
        for key in ["lufs_integrated", "true_peak_dbtp", "dynamic_range_lu"] {
            assert!(payload[key].is_number(), "missing {key}: {payload}");
        }
    }

    /// Render wire-key pin + the no-listening-gate guarantee: output exists
    /// and measurements carry the keys the receipt renders. Sample-level
    /// engine parity is already pinned by the facade crate's
    /// bridge_render_matches_shared_render_path.
    #[test]
    fn render_json_carries_every_key_kotlin_decodes_and_writes_the_wav() {
        let tmp = tempfile::tempdir().unwrap();
        let wav = tmp.path().join("source.wav");
        let out = tmp.path().join("rendered");
        write_sine_wav(&wav);
        let payload: serde_json::Value = serde_json::from_str(
            &inner::render_master_with_options_json(
                &wav.to_string_lossy(),
                &out.to_string_lossy(),
                Some("warm"),
                0.5,
                -11.0,
            ),
        )
        .unwrap();
        assert!(payload["error"].is_null(), "render errored: {payload}");
        let rendered = payload["output_paths"][0].as_str().expect("output path");
        assert!(Path::new(rendered).exists(), "rendered wav missing");
        let measurements = &payload["measurements"];
        for key in ["lufs_integrated", "true_peak_dbtp", "dynamic_range_lu"] {
            assert!(measurements[key].is_number(), "missing {key}: {payload}");
        }
        for key in ["sample_rate", "bit_depth"] {
            assert!(measurements[key].as_u64().is_some(), "missing {key}: {payload}");
        }
    }

    /// Third assert-side of src/standard-mapping-parity.json (desktop TS and
    /// the iPhone facade are the other two): the style ids Android sends map
    /// to the same engine presets.
    #[test]
    fn standard_style_aliases_match_the_shared_parity_fixture() {
        let parity: serde_json::Value =
            serde_json::from_str(include_str!("../../../../src/standard-mapping-parity.json"))
                .expect("parse parity fixture");
        let styles = parity["styles"].as_object().expect("styles map");
        assert!(!styles.is_empty());
        for (style, expected_kind) in styles {
            let settings = native_bridge::export_settings_for_options(Some(style), 0.5, -11.0);
            let kind = serde_json::to_value(&settings.preset).expect("serialize preset")["kind"]
                .as_str()
                .expect("kind tag")
                .to_string();
            assert_eq!(
                &kind,
                expected_kind.as_str().expect("kind string"),
                "Android style \"{style}\" diverged from the shared parity fixture"
            );
        }
    }
}
