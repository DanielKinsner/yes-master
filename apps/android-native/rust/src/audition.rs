//! Live mastered audition for Android.
//!
//! One engine = one imported track: the facade's [`LiveStream`] (the SAME
//! `MasteringChain` desktop and the iPhone app audition through — nothing is
//! re-implemented here) plus the Android output stream that pulls from it.
//! The device-output side (`crate::aaudio`) exists only on android targets;
//! everything else is plain Rust so the host lane tests it without a device.
//!
//! ## Threading contract (inherited from the facade's `live_stream`)
//!
//! - [`Pump::fill`] is called ONLY from the single AAudio data-callback
//!   thread (host tests stand in for it). It is the sole caller of
//!   `yes_master_native_live_process`.
//! - Every other method is a UI-side operation. They are serialized by the
//!   `output` mutex, satisfying the facade's single-UI-thread contract even
//!   though Kotlin coroutines hop dispatcher threads.
//! - [`AuditionEngine::measure_landing`] is the documented exception: it
//!   reads only the immutable decoded PCM, is safe concurrent with both
//!   sides, and is slow (it masters a measurement window) — so it
//!   deliberately does NOT hold the UI lock, keeping pause/seek responsive
//!   while a Preview LUFS measurement runs.

use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use native_bridge::live_stream as live;

/// State shared with the audio callback. Boxed separately from
/// [`AuditionEngine`] so its address is stable for the callback's user-data
/// pointer no matter how the outer engine handle moves between JNI calls.
// Several members exist for the android-only `crate::aaudio` callbacks, so
// host builds see them as dead code.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) struct Pump {
    /// The facade live handle. Bound once at create, freed in
    /// [`AuditionEngine`]'s `Drop` after the output stream is torn down —
    /// never rebound, so the callback may read it without synchronization.
    live: *mut live::LiveStream,
    /// UI-facing playback intent. The data callback does not consult it —
    /// on-device, pause is a stream-level operation (callbacks stop).
    playing: AtomicBool,
    /// Set by the AAudio error callback when the device stream dies (route
    /// change, headphones unplugged). The next `start` rebuilds the stream.
    stream_lost: AtomicBool,
    /// Interleaved channel count of the decoded PCM.
    channels: usize,
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
impl Pump {
    /// Pull up to `frames` frames of audio into `out` (interleaved,
    /// `self.channels` wide). Audio-callback thread only. The facade
    /// zero-fills past EOF and contains panics, so this is total.
    pub(crate) fn fill(&self, out: &mut [f32], frames: u32) -> u32 {
        // SAFETY: `live` is valid for the engine's lifetime (see field docs)
        // and `out` is sized by the caller to frames * channels.
        unsafe { live::yes_master_native_live_process(self.live, out.as_mut_ptr(), frames) }
    }

    pub(crate) fn channels(&self) -> usize {
        self.channels
    }

    pub(crate) fn on_stream_error(&self) {
        self.playing.store(false, Ordering::SeqCst);
        self.stream_lost.store(true, Ordering::SeqCst);
    }
}

/// Slot for the android output stream. On host builds it carries no stream;
/// locking it still serializes the UI-side operations.
struct OutputSlot {
    #[cfg(target_os = "android")]
    stream: Option<crate::aaudio::OutputStream>,
}

impl OutputSlot {
    fn new() -> Self {
        Self {
            #[cfg(target_os = "android")]
            stream: None,
        }
    }

    #[cfg(target_os = "android")]
    fn ensure_started(&mut self, pump: &Pump, sample_rate: u32) -> bool {
        if pump.stream_lost.swap(false, Ordering::SeqCst) {
            // The device stream died (route change / unplug). Drop it off
            // the audio path and rebuild below.
            self.stream = None;
        }
        if self.stream.is_none() {
            match crate::aaudio::OutputStream::open(pump, sample_rate) {
                Ok(stream) => self.stream = Some(stream),
                Err(_) => return false,
            }
        }
        self.stream.as_ref().expect("stream ensured above").start()
    }

    #[cfg(not(target_os = "android"))]
    fn ensure_started(&mut self, pump: &Pump, _sample_rate: u32) -> bool {
        // Host stand-in: no device stream; tests drive `Pump::fill` directly.
        let _ = pump;
        true
    }

    fn pause(&mut self) {
        #[cfg(target_os = "android")]
        if let Some(stream) = self.stream.as_ref() {
            stream.pause();
        }
    }

    fn teardown(&mut self) {
        #[cfg(target_os = "android")]
        {
            // OutputStream::drop stops the stream and blocks until the
            // callback thread is out — nothing touches the live handle after.
            self.stream = None;
        }
    }
}

/// Live-audition engine handed to Kotlin as an opaque `jlong`.
pub struct AuditionEngine {
    pump: Box<Pump>,
    /// Serializes UI-side operations (start/pause/seek/params/teardown) —
    /// see the module-level threading contract.
    output: Mutex<OutputSlot>,
    sample_rate: u32,
}

// SAFETY: `pump.live` follows the facade's documented two-thread contract —
// `fill` is audio-callback-only, every other entry point is serialized by the
// `output` mutex, and the cross-thread state inside the facade is atomic.
// The handle is therefore safe to move between and share across JNI threads.
unsafe impl Send for AuditionEngine {}
unsafe impl Sync for AuditionEngine {}

impl AuditionEngine {
    /// Decode `path` and build an engine initialized with the given Simple
    /// controls. `None` on a missing/undecodable file or NUL-poisoned input.
    /// Not real-time safe (decodes the whole file) — call off the UI thread.
    pub fn create(
        path: &str,
        preset: Option<&str>,
        intensity: f32,
        lufs_target: f32,
    ) -> Option<Box<Self>> {
        let path_c = CString::new(path).ok()?;
        let preset_c = preset.and_then(|p| CString::new(p).ok());
        let preset_ptr = preset_c.as_ref().map_or(std::ptr::null(), |p| p.as_ptr());
        // SAFETY: valid NUL-terminated strings (or null preset) for the call.
        let live = unsafe {
            live::yes_master_native_live_create(path_c.as_ptr(), preset_ptr, intensity, lufs_target)
        };
        if live.is_null() {
            return None;
        }
        // SAFETY: `live` is a valid handle from `create`.
        let (channels, sample_rate) = unsafe {
            (
                live::yes_master_native_live_channels(live) as usize,
                live::yes_master_native_live_sample_rate(live) as u32,
            )
        };
        Some(Box::new(Self {
            pump: Box::new(Pump {
                live,
                playing: AtomicBool::new(false),
                stream_lost: AtomicBool::new(false),
                channels,
            }),
            output: Mutex::new(OutputSlot::new()),
            sample_rate,
        }))
    }

    fn ui_lock(&self) -> std::sync::MutexGuard<'_, OutputSlot> {
        // A poisoned lock means a UI-side op panicked; the slot itself is
        // still structurally sound, and refusing to pause/teardown would be
        // strictly worse than continuing.
        self.output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Start (or resume) playback. Snaps the audio-thread smoothers to the
    /// current control targets first — the facade's documented start/resume
    /// semantics — so changes made while paused are heard from frame one
    /// instead of fading in from stale smoother state.
    pub fn start(&self) -> bool {
        let mut slot = self.ui_lock();
        // SAFETY: valid live handle; UI-side call under the lock.
        unsafe { live::yes_master_native_live_snap_controls_to_targets(self.pump.live) };
        if !slot.ensure_started(&self.pump, self.sample_rate) {
            return false;
        }
        self.pump.playing.store(true, Ordering::SeqCst);
        true
    }

    pub fn pause(&self) {
        let mut slot = self.ui_lock();
        slot.pause();
        self.pump.playing.store(false, Ordering::SeqCst);
    }

    pub fn is_playing(&self) -> bool {
        self.pump.playing.load(Ordering::SeqCst)
    }

    /// `true` = play Original (dry passthrough); `false` = play Mastered.
    /// The facade leaves the frame cursor untouched, so the playhead is
    /// preserved exactly across the switch.
    pub fn set_bypass(&self, original: bool) {
        let _ui = self.ui_lock();
        // SAFETY: valid live handle; UI-side call under the lock.
        unsafe { live::yes_master_native_live_set_bypass(self.pump.live, original) };
    }

    /// Update Style/Intensity/Loudness live. Coefficients are recomputed off
    /// the audio thread and crossfaded in by the facade.
    pub fn set_params(&self, preset: Option<&str>, intensity: f32, lufs_target: f32) {
        let _ui = self.ui_lock();
        let preset_c = preset.and_then(|p| CString::new(p).ok());
        let preset_ptr = preset_c.as_ref().map_or(std::ptr::null(), |p| p.as_ptr());
        // SAFETY: valid live handle and C strings; UI-side call under the lock.
        unsafe {
            live::yes_master_native_live_set_params(
                self.pump.live,
                preset_ptr,
                intensity,
                lufs_target,
            )
        };
    }

    /// Audition-only Volume Match gain (linear; `1.0` = unity). Never
    /// reaches the export path.
    pub fn set_volume_match(&self, linear_gain: f32) {
        let _ui = self.ui_lock();
        // SAFETY: valid live handle; UI-side call under the lock.
        unsafe { live::yes_master_native_live_set_volume_match(self.pump.live, linear_gain) };
    }

    /// Loudness landing gain (linear; `1.0` = unity), Mastered path only.
    /// Driven by [`Self::measure_landing`] when Preview LUFS is on.
    pub fn set_landing_gain(&self, linear_gain: f32) {
        let _ui = self.ui_lock();
        // SAFETY: valid live handle; UI-side call under the lock.
        unsafe { live::yes_master_native_live_set_landing_gain(self.pump.live, linear_gain) };
    }

    /// Measure the loudness landing for the given controls. Returns
    /// `(linear_gain, mastered_lufs)`; `mastered_lufs` is `NEG_INFINITY`
    /// when unavailable. Slow — call from a background thread. Deliberately
    /// NOT under the UI lock (see the module-level threading contract).
    pub fn measure_landing(
        &self,
        preset: Option<&str>,
        intensity: f32,
        lufs_target: f32,
    ) -> (f32, f32) {
        let preset_c = preset.and_then(|p| CString::new(p).ok());
        let preset_ptr = preset_c.as_ref().map_or(std::ptr::null(), |p| p.as_ptr());
        let mut mastered_lufs = f32::NEG_INFINITY;
        // SAFETY: valid live handle, valid C strings, valid out pointer. The
        // facade documents this call as safe concurrent with `process`.
        let gain = unsafe {
            live::yes_master_native_live_measure_landing(
                self.pump.live,
                preset_ptr,
                intensity,
                lufs_target,
                &mut mastered_lufs,
            )
        };
        (gain, mastered_lufs)
    }

    /// Move the playhead. Applied by the facade at the top of the next
    /// processed block (i.e. once playback runs); the Kotlin layer owns the
    /// displayed position while paused.
    pub fn seek(&self, position_seconds: f64) {
        let _ui = self.ui_lock();
        // SAFETY: valid live handle; UI-side call under the lock.
        unsafe { live::yes_master_native_live_seek(self.pump.live, position_seconds) };
    }

    pub fn position_seconds(&self) -> f64 {
        // SAFETY: valid live handle; reads an atomic inside the facade.
        unsafe { live::yes_master_native_live_position_seconds(self.pump.live) }
    }

    pub fn duration_seconds(&self) -> f64 {
        // SAFETY: valid live handle; reads immutable stream metadata.
        unsafe { live::yes_master_native_live_duration_seconds(self.pump.live) }
    }

    #[cfg(test)]
    pub(crate) fn fill(&self, out: &mut [f32], frames: u32) -> u32 {
        self.pump.fill(out, frames)
    }

    #[cfg(test)]
    pub(crate) fn channels(&self) -> usize {
        self.pump.channels
    }

    #[cfg(test)]
    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Drop for AuditionEngine {
    fn drop(&mut self) {
        // Tear the device stream down FIRST: it blocks until the callback
        // thread is out, so nothing can touch `live` after it is freed.
        self.ui_lock().teardown();
        // SAFETY: handle from `create`, freed exactly once, after the only
        // other user (the callback) is gone.
        unsafe { live::yes_master_native_live_destroy(self.pump.live) };
    }
}

/// dB-derived linear gain to apply to the side currently being heard so both
/// sides play at the QUIETER side's loudness — the gain is always <= 1, so
/// Volume Match can never boost or clip. Single source for the platforms;
/// mirrors the iPhone's `VolumeMatch.swift` (pinned by a test below).
/// Non-finite inputs (e.g. -inf LUFS for digital silence) fall back to unity.
pub fn volume_match_linear_gain(side_lufs: f64, other_lufs: f64) -> f32 {
    if !side_lufs.is_finite() || !other_lufs.is_finite() {
        return 1.0;
    }
    let reference = side_lufs.min(other_lufs);
    let gain_db = reference - side_lufs;
    10f64.powf(gain_db / 20.0) as f32
}

/// The JSON shape `measureLandingNative` hands Kotlin. `mastered_lufs`
/// serializes as `null` when the measurement was unavailable (serde maps the
/// non-finite sentinel to null), which Gson surfaces as a nullable Double.
pub(crate) fn landing_json(gain_lin: f32, mastered_lufs: f32) -> String {
    serde_json::json!({
        "gain_lin": gain_lin,
        "mastered_lufs": mastered_lufs,
    })
    .to_string()
}

/// JNI externs for `com.yesmaster.app.AuditionBridge`. Same shape as the
/// mastering shims: decode arguments, delegate, encode — never a throw
/// across the boundary. Every engine-touching body runs under
/// [`crate::jni_util::catch_panic`] so a panic can never reach the
/// `extern "system"` boundary (which would abort the whole app). The engine
/// handle travels as a `jlong` (0 = invalid).
mod jni_shims {
    use super::{landing_json, volume_match_linear_gain, AuditionEngine};
    use crate::jni_util::{catch_panic, from_jstring, to_jstring};
    use jni::objects::{JObject, JString};
    use jni::sys::{jboolean, jdouble, jfloat, jlong, jstring, JNI_FALSE, JNI_TRUE};
    use jni::JNIEnv;

    fn engine(handle: jlong) -> Option<&'static AuditionEngine> {
        if handle == 0 {
            return None;
        }
        // SAFETY: `handle` is a Box::into_raw from `createNative`, freed only
        // by `destroyNative`; the Kotlin wrapper nulls its copy on destroy.
        Some(unsafe { &*(handle as *const AuditionEngine) })
    }

    fn optional_string(env: &mut JNIEnv, value: &JString) -> Option<String> {
        if value.is_null() {
            None
        } else {
            from_jstring(env, value)
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_createNative(
        mut env: JNIEnv,
        _this: JObject,
        source_path: JString,
        preset: JString,
        intensity: jfloat,
        lufs_target: jfloat,
    ) -> jlong {
        let Some(path) = from_jstring(&mut env, &source_path) else {
            return 0;
        };
        let preset = optional_string(&mut env, &preset);
        catch_panic(
            || 0,
            || match AuditionEngine::create(&path, preset.as_deref(), intensity, lufs_target) {
                Some(engine) => Box::into_raw(engine) as jlong,
                None => 0,
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_destroyNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
    ) {
        if handle != 0 {
            catch_panic(
                || (),
                // SAFETY: handle from `createNative`, destroyed exactly once.
                || drop(unsafe { Box::from_raw(handle as *mut AuditionEngine) }),
            );
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_startNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
    ) -> jboolean {
        catch_panic(
            || JNI_FALSE,
            || match engine(handle) {
                Some(e) if e.start() => JNI_TRUE,
                _ => JNI_FALSE,
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_pauseNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
    ) {
        catch_panic(
            || (),
            || {
                if let Some(e) = engine(handle) {
                    e.pause();
                }
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_isPlayingNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
    ) -> jboolean {
        catch_panic(
            || JNI_FALSE,
            || match engine(handle) {
                Some(e) if e.is_playing() => JNI_TRUE,
                _ => JNI_FALSE,
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_setBypassNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
        original: jboolean,
    ) {
        catch_panic(
            || (),
            || {
                if let Some(e) = engine(handle) {
                    e.set_bypass(original == JNI_TRUE);
                }
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_setParamsNative(
        mut env: JNIEnv,
        _this: JObject,
        handle: jlong,
        preset: JString,
        intensity: jfloat,
        lufs_target: jfloat,
    ) {
        let preset = optional_string(&mut env, &preset);
        catch_panic(
            || (),
            || {
                if let Some(e) = engine(handle) {
                    e.set_params(preset.as_deref(), intensity, lufs_target);
                }
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_setVolumeMatchNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
        linear_gain: jfloat,
    ) {
        catch_panic(
            || (),
            || {
                if let Some(e) = engine(handle) {
                    e.set_volume_match(linear_gain);
                }
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_setLandingGainNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
        linear_gain: jfloat,
    ) {
        catch_panic(
            || (),
            || {
                if let Some(e) = engine(handle) {
                    e.set_landing_gain(linear_gain);
                }
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_measureLandingNative(
        mut env: JNIEnv,
        _this: JObject,
        handle: jlong,
        preset: JString,
        intensity: jfloat,
        lufs_target: jfloat,
    ) -> jstring {
        let preset = optional_string(&mut env, &preset);
        let result = catch_panic(
            || r#"{"error":"panic during landing measurement"}"#.to_string(),
            || match engine(handle) {
                Some(e) => {
                    let (gain, lufs) = e.measure_landing(preset.as_deref(), intensity, lufs_target);
                    landing_json(gain, lufs)
                }
                None => r#"{"error":"invalid audition handle"}"#.to_string(),
            },
        );
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_seekNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
        position_seconds: jdouble,
    ) {
        catch_panic(
            || (),
            || {
                if let Some(e) = engine(handle) {
                    e.seek(position_seconds);
                }
            },
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_positionSecondsNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
    ) -> jdouble {
        catch_panic(
            || 0.0,
            || engine(handle).map_or(0.0, |e| e.position_seconds()),
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_durationSecondsNative(
        _env: JNIEnv,
        _this: JObject,
        handle: jlong,
    ) -> jdouble {
        catch_panic(
            || 0.0,
            || engine(handle).map_or(0.0, |e| e.duration_seconds()),
        )
    }

    #[no_mangle]
    pub extern "system" fn Java_com_yesmaster_app_AuditionBridge_volumeMatchGainNative(
        _env: JNIEnv,
        _this: JObject,
        side_lufs: jdouble,
        other_lufs: jdouble,
    ) -> jfloat {
        catch_panic(|| 1.0, || volume_match_linear_gain(side_lufs, other_lufs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{write_dense_wav, write_sine_wav};
    use yes_master_lib::dsp::MasteringChain;

    static ADAPTIVE_COMPRESSION_GATE_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct AdaptiveCompressionGateReset(bool);

    impl Drop for AdaptiveCompressionGateReset {
        fn drop(&mut self) {
            yes_master_lib::guardrails::set_adaptive_compression_enabled(self.0);
        }
    }

    fn adaptive_compression_gate_for_test(enabled: bool) -> AdaptiveCompressionGateReset {
        AdaptiveCompressionGateReset(
            yes_master_lib::guardrails::set_adaptive_compression_enabled(enabled),
        )
    }

    fn desktop_adaptive_settings_for_path(
        path: &std::path::Path,
        preset: Option<&str>,
        intensity: f32,
        lufs_target: f32,
    ) -> yes_master_lib::MasteringSettings {
        let request = yes_master_lib::engine::AnalyzeRequest {
            id: yes_master_lib::TrackId::new(),
            path: path.to_string_lossy().into_owned(),
        };
        let analysis =
            futures_executor::block_on(yes_master_lib::engine::analyze_tracks_core(vec![request]))
                .expect("analyze dense fixture")
                .pop()
                .expect("analysis result");
        let band_psr = analysis
            .deep_analysis
            .as_deref()
            .and_then(yes_master_lib::deep_analysis::band_psr_p10_db);
        let stand_down = yes_master_lib::guardrails::classify_already_mastered_stand_down(
            analysis.lufs_integrated,
            analysis.true_peak_dbtp,
            analysis.dynamic_range_lu,
            band_psr,
        );
        let mut settings =
            native_bridge::export_settings_for_options(preset, intensity, lufs_target);
        settings.source_lufs_integrated = Some(analysis.lufs_integrated);
        yes_master_lib::profile_store::apply_resolved_profile(
            &mut settings,
            yes_master_lib::SourceProfile::from_analysis(&analysis),
            false,
        );
        yes_master_lib::profile_store::apply_resolved_confidence(
            &mut settings,
            analysis.deep_analysis.clone(),
            false,
        );
        yes_master_lib::profile_store::apply_resolved_compression_guards(
            &mut settings,
            analysis.deep_analysis.clone(),
            Some(stand_down),
            false,
        );
        settings
    }

    fn make_engine(frames: u32) -> Box<AuditionEngine> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, frames, 2, 48_000);
        AuditionEngine::create(&path.to_string_lossy(), Some("balanced"), 0.7, -11.0)
            .expect("engine from a real wav")
        // `dir` cleans up here; create() already decoded the whole file.
    }

    #[test]
    fn create_rejects_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.wav");
        assert!(AuditionEngine::create(&missing.to_string_lossy(), None, 0.5, -11.0).is_none());
    }

    #[test]
    fn create_reports_the_decoded_track_format() {
        let engine = make_engine(48_000); // 1.0 s stereo @ 48 kHz
        assert_eq!(engine.channels(), 2);
        assert_eq!(engine.sample_rate(), 48_000);
        assert!((engine.duration_seconds() - 1.0).abs() < 0.01);
        assert_eq!(engine.position_seconds(), 0.0);
    }

    /// The facade's snapped-Original test, at engine level: Original chosen
    /// BEFORE start must be dry from the very first frame, proving `start`
    /// wires snap_controls_to_targets through.
    #[test]
    fn start_snaps_pre_play_controls_so_original_is_dry_from_frame_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 2_000, 2, 48_000);
        let pcm = yes_master_lib::decode::decode_full(&path).expect("decode");
        let engine = AuditionEngine::create(&path.to_string_lossy(), Some("balanced"), 0.7, -11.0)
            .expect("engine");

        engine.set_bypass(true);
        assert!(engine.start());
        let frames = 256u32;
        let mut out = vec![0.0f32; frames as usize * engine.channels()];
        engine.fill(&mut out, frames);

        let expected = &pcm.samples[..out.len()];
        let max_diff = expected
            .iter()
            .zip(out.iter())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(
            max_diff < 1e-6,
            "snapped Original must be dry from frame one; max_diff={max_diff}"
        );
    }

    /// The product non-negotiable, at engine level: switching sides must not
    /// move the playhead.
    #[test]
    fn playhead_survives_original_mastered_toggle() {
        let engine = make_engine(10_000);
        assert!(engine.start());
        let mut out = vec![0.0f32; 1_000 * engine.channels()];
        engine.fill(&mut out, 1_000);
        let pos = engine.position_seconds();
        assert!(pos > 0.0);

        engine.set_bypass(true);
        assert_eq!(engine.position_seconds(), pos);
        engine.set_bypass(false);
        assert_eq!(engine.position_seconds(), pos);
    }

    #[test]
    fn start_and_pause_drive_the_playing_state() {
        let engine = make_engine(4_000);
        assert!(!engine.is_playing());
        assert!(engine.start());
        assert!(engine.is_playing());
        engine.pause();
        assert!(!engine.is_playing());
        assert!(engine.start(), "resume after pause must succeed");
        assert!(engine.is_playing());
    }

    #[test]
    fn seek_lands_at_the_next_fill_and_clamps_past_eof() {
        let engine = make_engine(48_000); // 1.0 s
        engine.seek(0.5);
        let mut out = vec![0.0f32; 2 * engine.channels()];
        engine.fill(&mut out, 2);
        let pos = engine.position_seconds();
        assert!((0.5..=0.55).contains(&pos), "expected ~0.5s, got {pos}");

        engine.seek(999.0);
        engine.fill(&mut out, 2);
        assert!(engine.position_seconds() <= 1.0 + 1e-6);
    }

    #[test]
    fn fill_saturates_at_eof_with_silence() {
        let engine = make_engine(1_000);
        assert!(engine.start());
        let frames = 4_096u32;
        let mut out = vec![0.0f32; frames as usize * engine.channels()];
        engine.fill(&mut out, frames);
        // Past-EOF region is zero-filled and the playhead parks at duration.
        let tail = &out[2_000 * engine.channels()..];
        assert!(
            tail.iter().all(|s| s.abs() < 1e-6),
            "EOF tail must be silent"
        );
        assert!((engine.position_seconds() - engine.duration_seconds()).abs() < 1e-6);
    }

    #[test]
    fn set_params_keeps_the_engine_alive_and_audible() {
        let engine = make_engine(8_000);
        assert!(engine.start());
        engine.set_params(Some("warm"), 0.9, -9.0);
        let frames = 1_024u32;
        let mut out = vec![0.0f32; frames as usize * engine.channels()];
        assert_eq!(engine.fill(&mut out, frames), frames);
        assert!(out.iter().all(|s| s.is_finite()));
        let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak > 0.01, "params change must not silence playback");
    }

    /// Landing measurement is wired through to the shared
    /// `engine::preview_landing`, and a louder target never lands quieter —
    /// the facade pins the math; this pins the engine-level wiring.
    #[test]
    fn measure_landing_is_wired_and_tracks_the_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 48_000 * 10, 2, 48_000); // 10 s: full landing window
        let engine = AuditionEngine::create(&path.to_string_lossy(), Some("balanced"), 0.5, -14.0)
            .expect("engine");

        let (gain_quiet, lufs_quiet) = engine.measure_landing(Some("balanced"), 0.5, -14.0);
        let (gain_loud, lufs_loud) = engine.measure_landing(Some("balanced"), 0.5, -9.0);

        assert!(gain_quiet.is_finite() && gain_quiet > 0.0);
        assert!(lufs_quiet.is_finite() && lufs_loud.is_finite());
        assert!(gain_loud >= gain_quiet - 1e-6);
        assert!(lufs_loud >= lufs_quiet - 1e-3);
    }

    #[test]
    fn adaptive_compressor_gate_off_output_matches_desktop_chain_bit_for_bit() {
        let _lock = ADAPTIVE_COMPRESSION_GATE_TEST_LOCK
            .lock()
            .expect("adaptive compression gate test lock");
        let _gate = adaptive_compression_gate_for_test(false);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dense-gate-off.wav");
        write_dense_wav(&path, 48_000 * 4, 2, 48_000);

        let pcm = yes_master_lib::decode::decode_full(&path).expect("decode");
        let channels = pcm.channels as usize;
        let total = pcm.samples.len() / channels;
        let settings = desktop_adaptive_settings_for_path(&path, Some("heavy"), 1.0, -9.0);
        let plan = yes_master_lib::guardrails::compression_plan_for_resolved_settings(&settings);
        assert!(
            !plan.active,
            "gate-off dense fixture must not resolve active adaptive compression guards: {plan:?}"
        );

        let mut reference = pcm.samples.clone();
        let mut chain = MasteringChain::new(pcm.sample_rate, channels, &settings);
        chain.process_interleaved(&mut reference, channels);

        let engine = AuditionEngine::create(&path.to_string_lossy(), Some("heavy"), 1.0, -9.0)
            .expect("engine");
        engine.set_bypass(false);
        assert!(engine.start());
        let mut android = vec![0.0f32; total * channels];
        assert_eq!(engine.fill(&mut android, total as u32), total as u32);

        let max_diff = reference
            .iter()
            .zip(android.iter())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(
            max_diff < 1e-6,
            "Android gate-off adaptive-compressor output must equal the desktop MasteringChain; max_diff={max_diff}"
        );
    }

    #[test]
    fn adaptive_compressor_output_matches_desktop_chain_bit_for_bit() {
        let _lock = ADAPTIVE_COMPRESSION_GATE_TEST_LOCK
            .lock()
            .expect("adaptive compression gate test lock");
        let _gate = adaptive_compression_gate_for_test(true);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dense.wav");
        write_dense_wav(&path, 48_000 * 4, 2, 48_000);

        let pcm = yes_master_lib::decode::decode_full(&path).expect("decode");
        let channels = pcm.channels as usize;
        let total = pcm.samples.len() / channels;
        let settings = desktop_adaptive_settings_for_path(&path, Some("heavy"), 1.0, -9.0);
        let plan = yes_master_lib::guardrails::compression_plan_for_resolved_settings(&settings);
        assert!(
            plan.active,
            "dense fixture must resolve adaptive compression guards: {plan:?}"
        );

        let mut reference = pcm.samples.clone();
        let mut chain = MasteringChain::new(pcm.sample_rate, channels, &settings);
        chain.process_interleaved(&mut reference, channels);

        let engine = AuditionEngine::create(&path.to_string_lossy(), Some("heavy"), 1.0, -9.0)
            .expect("engine");
        engine.set_bypass(false);
        assert!(engine.start());
        let mut android = vec![0.0f32; total * channels];
        assert_eq!(engine.fill(&mut android, total as u32), total as u32);

        let max_diff = reference
            .iter()
            .zip(android.iter())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(
            max_diff < 1e-6,
            "Android adaptive-compressor output must equal the desktop MasteringChain; max_diff={max_diff}"
        );
    }

    /// Pin against the iPhone's VolumeMatch.swift: reference is the QUIETER
    /// side, so the gain never boosts. Same numbers the Swift formula yields.
    #[test]
    fn volume_match_matches_the_iphone_formula() {
        // Heard side louder by 4 dB -> attenuate by 4 dB: 10^(-4/20).
        let g = volume_match_linear_gain(-8.0, -12.0);
        assert!((g - 0.630_957_3).abs() < 1e-5, "got {g}");
        // Heard side already the quieter one -> unity, never boost.
        assert_eq!(volume_match_linear_gain(-12.0, -8.0), 1.0);
        assert_eq!(volume_match_linear_gain(-10.0, -10.0), 1.0);
        // Non-finite loudness (digital silence) -> unity fallback.
        assert_eq!(volume_match_linear_gain(f64::NEG_INFINITY, -10.0), 1.0);
        assert_eq!(volume_match_linear_gain(-10.0, f64::NAN), 1.0);
    }

    /// Wire-key pin for the landing JSON Kotlin decodes, including the
    /// unavailable-measurement shape (`mastered_lufs: null`).
    #[test]
    fn landing_json_carries_every_key_kotlin_decodes() {
        let happy: serde_json::Value =
            serde_json::from_str(&landing_json(0.84, -11.2)).expect("parse");
        assert!(happy["gain_lin"].is_number());
        assert!(happy["mastered_lufs"].is_number());

        let unavailable: serde_json::Value =
            serde_json::from_str(&landing_json(1.0, f32::NEG_INFINITY)).expect("parse");
        assert!(unavailable["gain_lin"].is_number());
        assert!(
            unavailable["mastered_lufs"].is_null(),
            "unavailable measurement must decode as null, got {unavailable}"
        );
    }
}
