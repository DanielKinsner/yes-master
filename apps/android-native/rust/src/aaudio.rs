//! Minimal AAudio output shim (android targets only).
//!
//! The build spec's sanctioned alternative to oboe-rs: AAudio is a stable
//! plain C API (NDK, API 26+; minSdk here is 29), so ~100 lines of FFI
//! declarations replace a cmake/C++ build of liboboe. The data callback
//! pulls mastered frames straight from [`crate::audition::Pump`] in-process
//! — no per-buffer JNI hop, exactly the shape the spec drew.
//!
//! Sample-rate handling mirrors the iPhone (`AVAudioEngine` resamples after
//! the source node): the stream requests the SOURCE rate, because the
//! mastering chain's coefficients are computed for that rate (parity!).
//! AAudio satisfies an off-rate request via its AudioTrack-backed path
//! (resampled, slightly higher latency) when the low-latency MMAP path
//! can't run at that rate — audition keeps working either way.
//!
//! Constants are transcribed from the installed NDK r27.2 header
//! (`sysroot/usr/include/aaudio/AAudio.h`), not from memory.

use std::os::raw::c_void;

use crate::aaudio_config::{
    callback_buffer_samples, validate_actual_stream, AaudioResult, ActualStreamConfig,
    AAUDIO_FORMAT_PCM_FLOAT,
};
use crate::audition::Pump;

// --- FFI surface (only the symbols this module uses) ---------------------

#[repr(C)]
struct AAudioStreamBuilder {
    _opaque: [u8; 0],
}
#[repr(C)]
struct AAudioStreamHandle {
    _opaque: [u8; 0],
}

const AAUDIO_OK: AaudioResult = 0;
const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;
const AAUDIO_USAGE_MEDIA: i32 = 1;
const AAUDIO_CALLBACK_RESULT_CONTINUE: i32 = 0;
const AAUDIO_STREAM_STATE_STOPPING: i32 = 9;

type DataCallback = unsafe extern "C" fn(
    stream: *mut AAudioStreamHandle,
    user_data: *mut c_void,
    audio_data: *mut c_void,
    num_frames: i32,
) -> i32;
type ErrorCallback = unsafe extern "C" fn(
    stream: *mut AAudioStreamHandle,
    user_data: *mut c_void,
    error: AaudioResult,
);

#[link(name = "aaudio")]
extern "C" {
    fn AAudio_createStreamBuilder(builder: *mut *mut AAudioStreamBuilder) -> AaudioResult;
    fn AAudioStreamBuilder_setFormat(builder: *mut AAudioStreamBuilder, format: i32);
    fn AAudioStreamBuilder_setChannelCount(builder: *mut AAudioStreamBuilder, count: i32);
    fn AAudioStreamBuilder_setSampleRate(builder: *mut AAudioStreamBuilder, rate: i32);
    fn AAudioStreamBuilder_setPerformanceMode(builder: *mut AAudioStreamBuilder, mode: i32);
    fn AAudioStreamBuilder_setUsage(builder: *mut AAudioStreamBuilder, usage: i32);
    fn AAudioStreamBuilder_setDataCallback(
        builder: *mut AAudioStreamBuilder,
        callback: DataCallback,
        user_data: *mut c_void,
    );
    fn AAudioStreamBuilder_setErrorCallback(
        builder: *mut AAudioStreamBuilder,
        callback: ErrorCallback,
        user_data: *mut c_void,
    );
    fn AAudioStreamBuilder_openStream(
        builder: *mut AAudioStreamBuilder,
        stream: *mut *mut AAudioStreamHandle,
    ) -> AaudioResult;
    fn AAudioStreamBuilder_delete(builder: *mut AAudioStreamBuilder) -> AaudioResult;
    fn AAudioStream_getChannelCount(stream: *mut AAudioStreamHandle) -> i32;
    fn AAudioStream_getFormat(stream: *mut AAudioStreamHandle) -> i32;
    fn AAudioStream_getSampleRate(stream: *mut AAudioStreamHandle) -> i32;
    fn AAudioStream_requestStart(stream: *mut AAudioStreamHandle) -> AaudioResult;
    fn AAudioStream_requestPause(stream: *mut AAudioStreamHandle) -> AaudioResult;
    fn AAudioStream_requestStop(stream: *mut AAudioStreamHandle) -> AaudioResult;
    fn AAudioStream_waitForStateChange(
        stream: *mut AAudioStreamHandle,
        input_state: i32,
        next_state: *mut i32,
        timeout_nanos: i64,
    ) -> AaudioResult;
    fn AAudioStream_close(stream: *mut AAudioStreamHandle) -> AaudioResult;
}

// --- Callbacks ------------------------------------------------------------

struct CallbackState {
    pump: *const Pump,
    channels: usize,
}

/// Audio-thread entry point. Real-time discipline is inherited: `Pump::fill`
/// is the facade's `live_process` (no locks, no allocation in steady state,
/// panic-contained, zero-fills past EOF), so the only work added here is
/// slice bookkeeping. Always CONTINUE — "ended" is a UI-side notion (the
/// playhead parks at duration and the stream keeps emitting silence, exactly
/// like the iPhone's forever-pulled source node).
unsafe extern "C" fn data_callback(
    _stream: *mut AAudioStreamHandle,
    user_data: *mut c_void,
    audio_data: *mut c_void,
    num_frames: i32,
) -> i32 {
    if user_data.is_null() || audio_data.is_null() {
        return AAUDIO_CALLBACK_RESULT_CONTINUE;
    }
    let state = &*(user_data as *const CallbackState);
    let frames = num_frames.max(0) as u32;
    let out = std::slice::from_raw_parts_mut(
        audio_data as *mut f32,
        callback_buffer_samples(num_frames, state.channels),
    );
    let pump = &*state.pump;
    pump.fill(out, frames);
    AAUDIO_CALLBACK_RESULT_CONTINUE
}

/// Called by AAudio when the stream becomes unusable (device unplugged,
/// route change). Closing the stream from inside the callback is forbidden;
/// flag it and let the next UI-side `start` rebuild.
unsafe extern "C" fn error_callback(
    _stream: *mut AAudioStreamHandle,
    user_data: *mut c_void,
    _error: AaudioResult,
) {
    if user_data.is_null() {
        return;
    }
    let state = &*(user_data as *const CallbackState);
    (*state.pump).on_stream_error();
}

// --- Safe wrapper ----------------------------------------------------------

/// One open AAudio output stream bound to a [`Pump`]. The pump pointer
/// handed to the callbacks stays valid because [`crate::audition`] tears
/// this stream down (blocking) before the pump or live handle is freed.
pub(crate) struct OutputStream {
    stream: *mut AAudioStreamHandle,
    _callback_state: Box<CallbackState>,
}

// SAFETY: the raw stream handle is only touched from UI-side calls, which
// the audition engine serializes behind its mutex.
unsafe impl Send for OutputStream {}

impl OutputStream {
    pub(crate) fn open(pump: &Pump, sample_rate: u32) -> Result<Self, AaudioResult> {
        let mut builder: *mut AAudioStreamBuilder = std::ptr::null_mut();
        // SAFETY: out-pointer to a local; checked before use.
        let rc = unsafe { AAudio_createStreamBuilder(&mut builder) };
        if rc != AAUDIO_OK || builder.is_null() {
            return Err(rc);
        }
        let mut callback_state = Box::new(CallbackState {
            pump: pump as *const Pump,
            channels: pump.channels(),
        });
        let callback_ptr = callback_state.as_mut() as *mut CallbackState as *mut c_void;
        let mut stream: *mut AAudioStreamHandle = std::ptr::null_mut();
        // SAFETY: builder is valid; setters are plain stores; openStream
        // writes the out-pointer which is checked below.
        let rc = unsafe {
            AAudioStreamBuilder_setFormat(builder, AAUDIO_FORMAT_PCM_FLOAT);
            AAudioStreamBuilder_setChannelCount(builder, pump.channels() as i32);
            AAudioStreamBuilder_setSampleRate(builder, sample_rate.max(1) as i32);
            AAudioStreamBuilder_setPerformanceMode(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);
            AAudioStreamBuilder_setUsage(builder, AAUDIO_USAGE_MEDIA);
            AAudioStreamBuilder_setDataCallback(builder, data_callback, callback_ptr);
            AAudioStreamBuilder_setErrorCallback(builder, error_callback, callback_ptr);
            let rc = AAudioStreamBuilder_openStream(builder, &mut stream);
            AAudioStreamBuilder_delete(builder);
            rc
        };
        if rc != AAUDIO_OK || stream.is_null() {
            return Err(rc);
        }
        let actual = unsafe { actual_stream_config(stream) };
        let actual = match actual
            .and_then(|actual| validate_actual_stream(actual, pump.channels()).map(|()| actual))
        {
            Ok(actual) => actual,
            Err(err) => {
                unsafe { AAudioStream_close(stream) };
                return Err(err);
            }
        };
        callback_state.channels = actual.channel_count;
        Ok(Self {
            stream,
            _callback_state: callback_state,
        })
    }

    pub(crate) fn start(&self) -> bool {
        // SAFETY: open stream handle owned by self.
        unsafe { AAudioStream_requestStart(self.stream) == AAUDIO_OK }
    }

    pub(crate) fn pause(&self) {
        // Asynchronous by design — a few more callbacks may run while the
        // OS ramps the stream down, which is the correct click-free fade.
        // SAFETY: open stream handle owned by self.
        unsafe { AAudioStream_requestPause(self.stream) };
    }
}

unsafe fn actual_stream_config(
    stream: *mut AAudioStreamHandle,
) -> Result<ActualStreamConfig, AaudioResult> {
    ActualStreamConfig::from_raw(
        AAudioStream_getChannelCount(stream),
        AAudioStream_getSampleRate(stream),
        AAudioStream_getFormat(stream),
    )
}

impl Drop for OutputStream {
    fn drop(&mut self) {
        // SAFETY: open stream handle, freed exactly once. Stop first and
        // wait out the STOPPING transition so the callback thread is gone
        // before close — the audition engine relies on this barrier to free
        // the live handle safely afterwards.
        unsafe {
            AAudioStream_requestStop(self.stream);
            let mut state: i32 = 0;
            AAudioStream_waitForStateChange(
                self.stream,
                AAUDIO_STREAM_STATE_STOPPING,
                &mut state,
                500_000_000, // 500 ms cap
            );
            AAudioStream_close(self.stream);
        }
    }
}
