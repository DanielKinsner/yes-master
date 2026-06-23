//! Android native bridge.
//!
//! Thin JNI shims over the platform-neutral facade crate (the same
//! analyze/render C ABI the iPhone app drives through Swift FFI). The
//! contract is identical on purpose: paths in, JSON strings out, errors as
//! `{"error": "..."}` payloads — never an exception across the JNI
//! boundary. All real logic lives in `inner`, which is plain Rust so the
//! host test lane covers it without a JVM or a device.
//!
//! Live audition (A3): `audition` drives the facade's `live_stream` (same
//! chain as desktop/iPhone) and `aaudio` owns the device output stream.

use std::ffi::{c_char, CStr, CString};

#[cfg(target_os = "android")]
mod aaudio;
#[cfg(any(target_os = "android", test))]
mod aaudio_config;
pub mod audition;

/// JNI string helpers shared by the mastering and audition shim modules.
pub(crate) mod jni_util {
    use jni::objects::JString;
    use jni::sys::jstring;
    use jni::JNIEnv;

    pub(crate) fn to_jstring(env: &mut JNIEnv, value: String) -> jstring {
        match env.new_string(&value) {
            Ok(s) => s.into_raw(),
            Err(_) => {
                // Result-string allocation failed (realistically: JVM OOM).
                // Clear the pending Java exception so it cannot propagate
                // across the boundary — the contract is JSON-or-null, never
                // a throw — then try the small static error payload. If even
                // that fails we return null, which the Kotlin wrapper maps
                // back into the error-JSON contract.
                env.exception_clear().ok();
                env.new_string(r#"{"error":"jni string allocation failed"}"#)
                    .map(|s| s.into_raw())
                    .unwrap_or(std::ptr::null_mut())
            }
        }
    }

    pub(crate) fn from_jstring(env: &mut JNIEnv, value: &JString) -> Option<String> {
        match env.get_string(value) {
            Ok(s) => Some(s.into()),
            Err(_) => {
                // Mirror to_jstring's output-side fix: a failed decode can
                // leave a Java exception pending, and calling further JNI
                // functions with one pending is illegal — clear it so the
                // never-throw contract holds on the input side too.
                env.exception_clear().ok();
                None
            }
        }
    }

    /// Run `f`, mapping a panic to `default()`. A panic must never reach the
    /// `extern "system"` boundary — Rust aborts the process on unwind there,
    /// so a corrupt-file panic deep in the engine would kill the app instead
    /// of surfacing the error-JSON contract (adversarial-review finding).
    pub(crate) fn catch_panic<T>(default: impl FnOnce() -> T, f: impl FnOnce() -> T) -> T {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| default())
    }
}

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
        let value = unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned();
        unsafe { native_bridge::yes_master_native_free_string(raw) };
        value
    }

    fn json_error(message: &str) -> String {
        serde_json::json!({ "error": message }).to_string()
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
        let preset_c = match preset {
            Some(p) => match CString::new(p) {
                Ok(preset) => Some(preset),
                Err(_) => return json_error("preset contains an interior NUL byte"),
            },
            None => None,
        };
        let preset_ptr = preset_c.as_ref().map_or(std::ptr::null(), |p| p.as_ptr());
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
    use crate::jni_util::{catch_panic, from_jstring, to_jstring};
    use jni::objects::{JObject, JString};
    use jni::sys::{jboolean, jfloat, jstring, JNI_FALSE, JNI_TRUE};
    use jni::JNIEnv;

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_supportsImportExtension(
        mut env: JNIEnv,
        _this: JObject,
        extension: JString,
    ) -> jboolean {
        let ext = from_jstring(&mut env, &extension);
        catch_panic(
            || JNI_FALSE,
            || match ext {
                Some(ext) if inner::supports_import_extension(&ext) => JNI_TRUE,
                _ => JNI_FALSE,
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_analyzeFileJsonNative(
        mut env: JNIEnv,
        _this: JObject,
        path: JString,
    ) -> jstring {
        let path = from_jstring(&mut env, &path);
        let result = catch_panic(
            || r#"{"error":"panic during analysis"}"#.to_string(),
            || match path {
                Some(path) => inner::analyze_file_json(&path),
                None => r#"{"error":"invalid path string"}"#.to_string(),
            },
        );
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_NativeBridge_renderMasterWithOptionsJsonNative(
        mut env: JNIEnv,
        _this: JObject,
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
            Ok(None)
        } else {
            from_jstring(&mut env, &preset)
                .map(Some)
                .ok_or_else(|| r#"{"error":"invalid preset string"}"#.to_string())
        };
        let result = catch_panic(
            || r#"{"error":"panic during render"}"#.to_string(),
            || match (source, out_dir, preset) {
                (Some(source), Some(out_dir), Ok(preset)) => {
                    inner::render_master_with_options_json(
                        &source,
                        &out_dir,
                        preset.as_deref(),
                        intensity,
                        lufs_target,
                    )
                }
                (_, _, Err(error)) => error,
                _ => r#"{"error":"invalid path strings"}"#.to_string(),
            },
        );
        to_jstring(&mut env, result)
    }
}

/// Shared by this file's tests and the audition module's tests.
#[cfg(test)]
pub(crate) mod test_util {
    use std::path::Path;

    pub(crate) fn write_sine_wav(path: &Path, frames: u32, channels: u16, sample_rate: u32) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).expect("create wav");
        let omega = 2.0 * std::f32::consts::PI * 440.0 / sample_rate as f32;
        for i in 0..frames {
            let v = (0.4 * (omega * i as f32).sin() * 32_767.0) as i16;
            for _ in 0..channels {
                writer.write_sample(v).expect("sample");
            }
        }
        writer.finalize().expect("finalize");
    }

    pub(crate) fn write_dense_wav(path: &Path, frames: u32, channels: u16, sample_rate: u32) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).expect("create wav");
        let tones = [95.0, 180.0, 430.0, 1_200.0, 3_600.0, 8_400.0];
        for i in 0..frames {
            let t = i as f32 / sample_rate as f32;
            let raw = tones
                .iter()
                .enumerate()
                .map(|(index, hz)| {
                    let phase = (index as f32 * 0.17).fract();
                    ((t * hz + phase) * std::f32::consts::TAU).sin()
                })
                .sum::<f32>()
                / tones.len() as f32;
            let sample = (raw * 4.0).tanh() * 0.72;
            let value = (sample.clamp(-0.95, 0.95) * i16::MAX as f32) as i16;
            for _ in 0..channels {
                writer.write_sample(value).expect("sample");
            }
        }
        writer.finalize().expect("finalize");
    }
}

#[cfg(test)]
mod tests {
    use super::inner;
    use crate::test_util::write_sine_wav as write_sine_wav_with;
    use std::path::Path;

    fn standard_parity_fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("../../../../src/standard-mapping-parity.json"))
            .expect("parse standard-mapping-parity.json")
    }

    fn parity_delivery(parity: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
        parity["delivery"]
            .as_object()
            .expect("delivery object in standard-mapping-parity.json")
    }

    fn delivery_u32(delivery: &serde_json::Map<String, serde_json::Value>, key: &str) -> u32 {
        delivery[key].as_u64().expect(key) as u32
    }

    fn delivery_u16(delivery: &serde_json::Map<String, serde_json::Value>, key: &str) -> u16 {
        delivery[key].as_u64().expect(key) as u16
    }

    fn delivery_f32(delivery: &serde_json::Map<String, serde_json::Value>, key: &str) -> f32 {
        delivery[key].as_f64().expect(key) as f32
    }

    fn delivery_lufs_clamp(delivery: &serde_json::Map<String, serde_json::Value>) -> [f32; 2] {
        let values = delivery["lufs_clamp"].as_array().expect("lufs_clamp");
        [
            values[0].as_f64().expect("lufs min") as f32,
            values[1].as_f64().expect("lufs max") as f32,
        ]
    }

    fn write_sine_wav(path: &Path) {
        // 2 s stereo @ 44.1 kHz — what these wire/parity tests always used.
        write_sine_wav_with(path, 88_200, 2, 44_100);
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
        let payload: serde_json::Value =
            serde_json::from_str(&inner::render_master_with_options_json(
                &wav.to_string_lossy(),
                &out.to_string_lossy(),
                Some("warm"),
                0.5,
                -11.0,
            ))
            .unwrap();
        assert!(payload["error"].is_null(), "render errored: {payload}");
        let rendered = payload["output_paths"][0].as_str().expect("output path");
        assert!(Path::new(rendered).exists(), "rendered wav missing");
        let measurements = &payload["measurements"];
        for key in ["lufs_integrated", "true_peak_dbtp", "dynamic_range_lu"] {
            assert!(measurements[key].is_number(), "missing {key}: {payload}");
        }
        for key in ["sample_rate", "bit_depth"] {
            assert!(
                measurements[key].as_u64().is_some(),
                "missing {key}: {payload}"
            );
        }
    }

    #[test]
    fn render_rejects_nul_poisoned_preset() {
        let tmp = tempfile::tempdir().unwrap();
        let wav = tmp.path().join("source.wav");
        let out = tmp.path().join("rendered");
        write_sine_wav(&wav);

        let payload: serde_json::Value =
            serde_json::from_str(&inner::render_master_with_options_json(
                &wav.to_string_lossy(),
                &out.to_string_lossy(),
                Some("warm\0balanced"),
                0.5,
                -11.0,
            ))
            .unwrap();

        assert_eq!(
            payload["error"], "preset contains an interior NUL byte",
            "render should reject an invalid preset instead of defaulting: {payload}"
        );
    }

    /// Third assert-side of src/standard-mapping-parity.json (desktop TS and
    /// the iPhone facade are the other two): the style ids Android sends map
    /// to the same engine presets.
    #[test]
    fn standard_style_aliases_match_the_shared_parity_fixture() {
        let parity = standard_parity_fixture();
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

    #[test]
    fn fixed_delivery_recipe_matches_the_shared_parity_fixture() {
        let parity = standard_parity_fixture();
        let delivery = parity_delivery(&parity);
        let settings = native_bridge::export_settings_for_options(Some("balanced"), 0.5, -11.0);

        assert_eq!(
            settings.effective_ceiling_dbtp(),
            delivery_f32(delivery, "ceiling_dbtp")
        );
        assert_eq!(
            settings.effective_bit_depth(),
            delivery_u16(delivery, "bit_depth")
        );
        assert_eq!(
            settings.effective_sample_rate(48_000),
            delivery_u32(delivery, "sample_rate")
        );

        let [min_lufs, max_lufs] = delivery_lufs_clamp(delivery);
        let clamped_low =
            native_bridge::export_settings_for_options(Some("balanced"), 0.5, min_lufs - 10.0);
        assert_eq!(clamped_low.effective_target_lufs(), Some(min_lufs));
        let clamped_high =
            native_bridge::export_settings_for_options(Some("balanced"), 0.5, max_lufs + 10.0);
        assert_eq!(clamped_high.effective_target_lufs(), Some(max_lufs));
    }
}
