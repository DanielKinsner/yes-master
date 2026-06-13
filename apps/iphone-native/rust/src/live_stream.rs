//! Live mastered audition stream for the native iPhone app.
//!
//! Swift owns the iOS output graph (`AVAudioEngine` + one `AVAudioSourceNode`)
//! and calls [`yes_master_native_live_process`] from the CoreAudio render thread
//! to pull mastered PCM. This module owns the persistent live state: one decoded
//! interleaved PCM buffer, one frame cursor, and one `MasteringChain` — the SAME
//! chain the desktop and Create Master use, via `yes_master_lib::dsp`. It never
//! owns an output stream, so no `rodio`/`cpal` is involved in playback.
//!
//! ## Threading contract (the soundness basis for the raw-pointer FFI)
//!
//! - [`yes_master_native_live_process`] is called ONLY from the single CoreAudio
//!   render thread. It is the sole accessor of [`AudioCore`], reached through an
//!   `UnsafeCell`.
//! - The setters/getters (`set_params`, `set_bypass`, `set_volume_match`,
//!   `set_landing_gain`, `seek`, `position_seconds`, `duration_seconds`) are
//!   called ONLY from non-audio (UI) threads. They touch the coefficient
//!   `Sender` and the [`Shared`] atomics — never [`AudioCore`].
//! - The two threads therefore touch DISJOINT state. The mpsc channel and the
//!   atomics are each internally synchronized, so the handle is safe to share
//!   between the threads via its raw pointer. Every entry point forms a shared
//!   `&LiveStream` (never `&mut`), so there is no aliasing; `process` mutates
//!   `AudioCore` through the `UnsafeCell`.
//!
//! ## Real-time discipline in `process`
//!
//! No locks, no logging, no JSON, no blocking. The only heap allocation is the
//! `MasteringChain` state clone done by `with_coeffs_inheriting_state` when a
//! parameter change arrives — bounded to the coefficient-check interval, never
//! in steady-state playback. This is ported verbatim from the desktop
//! `MasteringSource`. Replacing that clone with a preallocated double-buffer is
//! the first planned RT-hardening follow-up. `process` is wrapped in
//! `catch_unwind` at the FFI boundary so a panic can never unwind into C.

use std::cell::UnsafeCell;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use yes_master_lib::dsp::{ChainCoeffs, MasteringChain};

use crate::{
    export_settings_for_options_with_context, ffi_string, native_adaptive_context_for_path,
    NativeAdaptiveContext,
};

/// Crossfade length between the old and new chain when coefficients change.
/// 512 frames ≈ 10.7 ms at 48 kHz — long enough to mask filter-state
/// transients, short enough to feel instantaneous.
const COEFFS_CROSSFADE_FRAMES: usize = 512;
/// Sentinel for "no pending seek" in [`Shared::seek_target_frame`].
const NO_SEEK: u64 = u64::MAX;

/// A coefficient update handed from the UI thread to the audio thread. Carries
/// a monotonic generation so the drain can keep only the newest during a knob
/// sweep. `ChainCoeffs` is `Copy`, so this transfers cleanly by value.
struct LiveCoeffUpdate {
    generation: u64,
    coeffs: ChainCoeffs,
}

/// Cross-thread state. Every field is atomic so the UI and audio threads can
/// touch it without locks.
struct Shared {
    /// `true` = play Original (dry passthrough); `false` = play Mastered.
    bypass_original: AtomicBool,
    /// Audition-only Volume Match gain (linear), stored as `f32` bits. `1.0` =
    /// unity. Never reaches the export path.
    volume_match_gain: AtomicU32,
    /// Loudness landing gain (linear), stored as `f32` bits. `1.0` = unity until
    /// the windowed measurement lands a value (Task 7). Mastered path only.
    landing_gain: AtomicU32,
    /// One-shot request from the UI thread to copy current control targets into
    /// the audio-thread smoothers before the next block renders. Used when
    /// playback starts/resumes so paused/pre-play changes do not fade in from an
    /// old smoother state.
    snap_controls_to_targets: AtomicBool,
    /// Pending seek target in frames; `NO_SEEK` when none. Applied at the top of
    /// the next `process` block.
    seek_target_frame: AtomicU64,
    /// Current playback position in frames. `process` writes it; the UI reads it
    /// for the playhead.
    cursor_frame: AtomicU64,
    /// Monotonic coefficient generation, bumped by `set_params` (UI thread).
    generation: AtomicU64,
}

impl Shared {
    fn new() -> Self {
        Self {
            bypass_original: AtomicBool::new(false),
            volume_match_gain: AtomicU32::new(1.0f32.to_bits()),
            landing_gain: AtomicU32::new(1.0f32.to_bits()),
            snap_controls_to_targets: AtomicBool::new(false),
            seek_target_frame: AtomicU64::new(NO_SEEK),
            cursor_frame: AtomicU64::new(0),
            generation: AtomicU64::new(0),
        }
    }
}

#[inline]
fn load_gain(slot: &AtomicU32) -> f32 {
    f32::from_bits(slot.load(Ordering::Relaxed))
}

/// Sanitize a caller-supplied linear gain: finite and non-negative, else unity.
#[inline]
fn sanitize_gain(value: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        1.0
    }
}

/// Audio-thread-only playback state. Reached exclusively from `process` via the
/// handle's `UnsafeCell`.
struct AudioCore {
    samples: Arc<[f32]>,
    channels: usize,
    total_frames: usize,
    chain: MasteringChain,
    pending_chain: Option<MasteringChain>,
    crossfade_remaining: usize,
    crossfade_total: usize,
    coeffs_rx: Receiver<LiveCoeffUpdate>,
    coeffs_generation: u64,
    // Preallocated per-frame scratch — never grows on the audio thread.
    frame_in: Vec<f32>,
    frame_main: Vec<f32>,
    frame_pending: Vec<f32>,
    // Per-sample one-pole smoothers toward the lock-free targets, so control
    // changes ramp (~12 ms) instead of stepping — no clicks on Volume Match,
    // Loudness, or Original/Mastered. `smoothed_bypass` is the dry/wet mix
    // (0 = Mastered, 1 = Original); the chain runs every frame so flipping sides
    // crossfades and the chain stays warm (no cold-start transient).
    smoothed_bypass: f32,
    smoothed_vol_match: f32,
    smoothed_landing: f32,
    gain_smooth_coeff: f32,
}

impl AudioCore {
    /// Fill `out` (interleaved, `self.channels` wide) with up to `frames` frames
    /// of audio at the current cursor under the current params. Returns the
    /// number of frames actually written (short at end-of-file; the remainder of
    /// `out` is zero-filled). Real-time safe: see the module docs.
    fn process(&mut self, out: &mut [f32], frames: usize, shared: &Shared) {
        let channels = self.channels;

        // 1. Drain the coefficient channel ONCE per block, before anything else
        //    and regardless of cursor position. Draining at the block top (vs.
        //    the desktop source's per-128-frame in-loop drain) is the right fit
        //    for a block-pulled `AVAudioSourceNode`: it keeps the unbounded mpsc
        //    bounded even while parked past EOF (where the per-frame loop only
        //    emits silence) and keeps the chain tracking the latest settings.
        //    Equivalent to desktop's check-while-draining: generations are
        //    unique + monotonic on one FIFO channel, so keeping the
        //    max-generation update and applying it once selects the same newest
        //    coefficients.
        let mut latest: Option<LiveCoeffUpdate> = None;
        while let Ok(update) = self.coeffs_rx.try_recv() {
            match latest {
                Some(ref current) if current.generation >= update.generation => {}
                _ => latest = Some(update),
            }
        }
        if let Some(update) = latest {
            if update.generation >= self.coeffs_generation {
                self.coeffs_generation = update.generation;
                // Promote an in-flight pending chain to main before arming the
                // next so a sustained sweep bounds the 2x-DSP window to one
                // crossfade interval and keeps tracking the latest settings.
                if let Some(prev_pending) = self.pending_chain.take() {
                    self.chain = prev_pending;
                }
                self.pending_chain = Some(MasteringChain::with_coeffs_inheriting_state(
                    update.coeffs,
                    &self.chain,
                ));
                self.crossfade_remaining = COEFFS_CROSSFADE_FRAMES;
                self.crossfade_total = COEFFS_CROSSFADE_FRAMES;
            }
        }

        // 2. Apply a pending seek AFTER the drain, so the seek lands on the
        //    newest coefficients. A seek is a discontinuity: promote any in-flight
        //    pending chain to main first (otherwise a seek mid-crossfade would
        //    discard the newer settings), then drop filter/limiter state so the
        //    new position starts clean instead of ringing the old state.
        let seek = shared.seek_target_frame.swap(NO_SEEK, Ordering::Acquire);
        if seek != NO_SEEK {
            if let Some(pending) = self.pending_chain.take() {
                self.chain = pending;
            }
            self.chain.reset_states();
            self.crossfade_remaining = 0;
            self.crossfade_total = 0;
            let target = (seek as usize).min(self.total_frames);
            shared.cursor_frame.store(target as u64, Ordering::Relaxed);
        }

        let should_snap = shared
            .snap_controls_to_targets
            .swap(false, Ordering::Acquire);
        let target_bypass = if shared.bypass_original.load(Ordering::Relaxed) {
            1.0
        } else {
            0.0
        };
        let target_vol_match = load_gain(&shared.volume_match_gain);
        let target_landing = load_gain(&shared.landing_gain);
        if should_snap {
            self.smoothed_bypass = target_bypass;
            self.smoothed_vol_match = target_vol_match;
            self.smoothed_landing = target_landing;
        }
        let smooth = self.gain_smooth_coeff;

        let mut cursor = shared.cursor_frame.load(Ordering::Relaxed) as usize;

        for f in 0..frames {
            let out_base = f * channels;

            // Ramp toward the control targets every frame (one-pole, ~12 ms) so
            // Volume Match, Loudness, and Original/Mastered fade in instead of
            // stepping. Advanced even past EOF so the values stay current.
            self.smoothed_bypass += smooth * (target_bypass - self.smoothed_bypass);
            self.smoothed_vol_match += smooth * (target_vol_match - self.smoothed_vol_match);
            self.smoothed_landing += smooth * (target_landing - self.smoothed_landing);

            if cursor >= self.total_frames {
                // Past EOF: emit silence. The cursor intentionally does NOT
                // advance past total_frames, so position_seconds saturates at
                // duration and a forever-pulled source node just gets silence.
                for ch in 0..channels {
                    out[out_base + ch] = 0.0;
                }
                continue;
            }

            // Pull one input frame, sanitizing non-finite samples so a corrupt
            // input can never get stuck in the IIR biquad feedback: a single NaN
            // would otherwise poison the channel until the next seek (the limiter
            // peak scan uses `>` and lets NaN pass through).
            let in_base = cursor * channels;
            for ch in 0..channels {
                let s = self.samples.get(in_base + ch).copied().unwrap_or(0.0);
                self.frame_in[ch] = if s.is_finite() { s } else { 0.0 };
            }

            // Always run the chain so it stays warm even while Original is
            // selected — switching to Mastered then has no cold-state transient.
            for ch in 0..channels {
                self.frame_main[ch] = self.frame_in[ch];
            }
            self.chain
                .process_frame_inplace(&mut self.frame_main[..channels]);

            if self.pending_chain.is_some() && self.crossfade_total > 0 {
                for ch in 0..channels {
                    self.frame_pending[ch] = self.frame_in[ch];
                }
                let pending = self
                    .pending_chain
                    .as_mut()
                    .expect("pending_chain just checked is_some");
                pending.process_frame_inplace(&mut self.frame_pending[..channels]);
                let t = 1.0 - (self.crossfade_remaining as f32 / self.crossfade_total as f32);
                let inv_t = 1.0 - t;
                for ch in 0..channels {
                    self.frame_main[ch] = self.frame_main[ch] * inv_t + self.frame_pending[ch] * t;
                }
                self.crossfade_remaining = self.crossfade_remaining.saturating_sub(1);
                if self.crossfade_remaining == 0 {
                    self.chain = self
                        .pending_chain
                        .take()
                        .expect("pending_chain just checked is_some");
                    self.crossfade_total = 0;
                }
            }

            // Crossfade dry (Original, no landing) <-> wet (Mastered, landed) by
            // the smoothed bypass mix, then apply the smoothed Volume Match to the
            // side being heard. All three transitions are click-free.
            let bypass_mix = self.smoothed_bypass;
            for ch in 0..channels {
                let dry = self.frame_in[ch];
                let wet = self.frame_main[ch] * self.smoothed_landing;
                out[out_base + ch] =
                    (dry * bypass_mix + wet * (1.0 - bypass_mix)) * self.smoothed_vol_match;
            }

            cursor += 1;
        }

        shared.cursor_frame.store(cursor as u64, Ordering::Relaxed);
    }
}

/// Opaque live-audition handle. Created on the UI thread, shared with the audio
/// thread via its raw pointer. See the module-level threading contract.
pub struct LiveStream {
    core: UnsafeCell<AudioCore>,
    coeffs_tx: Sender<LiveCoeffUpdate>,
    shared: Shared,
    sample_rate: u32,
    channels: usize,
    total_frames: usize,
    /// Immutable decoded PCM, shared with `AudioCore`. Read-only after create,
    /// so the UI thread can measure loudness from it (see
    /// `yes_master_native_live_measure_landing`) while the audio thread reads its
    /// own `Arc` clone in `process` — concurrent immutable reads, no race.
    samples: Arc<[f32]>,
    /// Desktop-equivalent adaptive source context derived once at stream create
    /// time. Immutable and reused for live coefficient updates / landing probes.
    adaptive_context: Option<NativeAdaptiveContext>,
}

// SAFETY: see the module-level threading contract. `core` (behind the
// `UnsafeCell`) is touched only by `process` on the single audio thread;
// `coeffs_tx` only by the setters on the UI thread; `shared` is all atomics.
// The two threads never touch the same field, so the handle is safe to send to
// and share with the audio thread through its raw pointer.
unsafe impl Send for LiveStream {}
unsafe impl Sync for LiveStream {}

impl LiveStream {
    fn create(path: &Path, preset: Option<&str>, intensity: f32, lufs_target: f32) -> Option<Self> {
        let pcm = yes_master_lib::decode::decode_full(path).ok()?;
        let channels = (pcm.channels.max(1)) as usize;
        let sample_rate = pcm.sample_rate.max(1);
        let samples: Arc<[f32]> = Arc::from(pcm.samples);
        let total_frames = samples.len() / channels;

        // Reuse the exact (preset, intensity, lufs) -> MasteringSettings mapping
        // the export path uses, so preview and Create Master share one source of
        // truth. The chain runs at the decoded source rate; its coefficients are
        // computed for that rate inside `MasteringChain::new`.
        let adaptive_context = native_adaptive_context_for_path(path);
        let settings = export_settings_for_options_with_context(
            preset,
            intensity,
            lufs_target,
            adaptive_context.as_ref(),
        );
        let chain = MasteringChain::new(sample_rate, channels, &settings);

        let (coeffs_tx, coeffs_rx) = mpsc::channel();
        let core = AudioCore {
            samples: samples.clone(),
            channels,
            total_frames,
            chain,
            pending_chain: None,
            crossfade_remaining: 0,
            crossfade_total: 0,
            coeffs_rx,
            coeffs_generation: 0,
            frame_in: vec![0.0; channels],
            frame_main: vec![0.0; channels],
            frame_pending: vec![0.0; channels],
            // Start at the default targets (Mastered, unity gains) so a stream
            // with no control changes is bit-identical to the bare chain.
            smoothed_bypass: 0.0,
            smoothed_vol_match: 1.0,
            smoothed_landing: 1.0,
            // ~12 ms one-pole time constant: a = 1 - exp(-1 / (tau * sr)).
            gain_smooth_coeff: 1.0 - (-1.0 / (0.012 * sample_rate as f32)).exp(),
        };

        Some(Self {
            core: UnsafeCell::new(core),
            coeffs_tx,
            shared: Shared::new(),
            sample_rate,
            channels,
            total_frames,
            samples,
            adaptive_context,
        })
    }

    /// Recompute coefficients off the audio thread and hand them over. Builds the
    /// full `MasteringSettings` from (preset, intensity, lufs) — the chain reads
    /// the whole struct, so passing only the three Simple-mode controls would be
    /// lossy if the chain ever grew to read more fields.
    fn set_params(&self, preset: Option<&str>, intensity: f32, lufs_target: f32) {
        let settings = export_settings_for_options_with_context(
            preset,
            intensity,
            lufs_target,
            self.adaptive_context.as_ref(),
        );
        let coeffs = ChainCoeffs::from_settings(self.sample_rate, &settings);
        let generation = self.shared.generation.fetch_add(1, Ordering::Relaxed) + 1;
        // Best-effort: a closed channel only happens if the handle is being torn
        // down, in which case dropping the update is correct.
        let _ = self.coeffs_tx.send(LiveCoeffUpdate { generation, coeffs });
    }
}

// ---------------------------------------------------------------------------
// C ABI
// ---------------------------------------------------------------------------

/// Decode `source_path` and build a live-audition handle initialized with the
/// given Simple-mode controls. Returns `NULL` on failure (missing/undecodable
/// file). NOT real-time safe — call off the audio thread. Free with
/// [`yes_master_native_live_destroy`].
///
/// # Safety
/// `source_path`/`preset` must be valid C strings or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_create(
    source_path: *const c_char,
    preset: *const c_char,
    intensity: f32,
    lufs_target: f32,
) -> *mut LiveStream {
    let Some(path) = ffi_string(source_path) else {
        return std::ptr::null_mut();
    };
    if path.trim().is_empty() {
        return std::ptr::null_mut();
    }
    let preset = ffi_string(preset);
    match LiveStream::create(Path::new(&path), preset.as_deref(), intensity, lufs_target) {
        Some(stream) => Box::into_raw(Box::new(stream)),
        None => std::ptr::null_mut(),
    }
}

/// Fill `out_interleaved` with up to `frames` frames (interleaved, channel count
/// from [`yes_master_native_live_channels`]). Returns frames written (short at
/// EOF; remainder zero-filled). Real-time safe; call from the render callback.
///
/// # Safety
/// `handle` must be a live handle from `create`; `out_interleaved` must point to
/// at least `frames * channels` writable floats.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_process(
    handle: *mut LiveStream,
    out_interleaved: *mut f32,
    frames: u32,
) -> u32 {
    if handle.is_null() || out_interleaved.is_null() {
        return 0;
    }
    let stream = &*handle;
    let channels = stream.channels;
    let frames = frames as usize;
    let len = frames.saturating_mul(channels);
    let out = std::slice::from_raw_parts_mut(out_interleaved, len);

    // A panic must never unwind across the C ABI (UB). On panic, emit silence.
    let core = &mut *stream.core.get();
    let result = catch_unwind(AssertUnwindSafe(|| {
        core.process(out, frames, &stream.shared)
    }));
    if result.is_err() {
        for s in out.iter_mut() {
            *s = 0.0;
        }
        return 0;
    }
    frames as u32
}

/// Switch Original (`original = true`, dry passthrough) vs Mastered. The frame
/// cursor is untouched, so the playhead is preserved exactly across the switch.
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_set_bypass(
    handle: *mut LiveStream,
    original: bool,
) {
    if handle.is_null() {
        return;
    }
    (*handle)
        .shared
        .bypass_original
        .store(original, Ordering::Relaxed);
}

/// Update Style/Intensity/Loudness live. Coefficients are recomputed here (off
/// the audio thread) and crossfaded in by `process` — no full-song re-render.
///
/// # Safety
/// `handle` must be a live handle from `create` or null; `preset` a C string or
/// null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_set_params(
    handle: *mut LiveStream,
    preset: *const c_char,
    intensity: f32,
    lufs_target: f32,
) {
    if handle.is_null() {
        return;
    }
    let preset = ffi_string(preset);
    (*handle).set_params(preset.as_deref(), intensity, lufs_target);
}

/// Set the audition-only Volume Match gain (linear; `1.0` = unity). Applied to
/// the selected side for fair A/B; never reaches export.
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_set_volume_match(
    handle: *mut LiveStream,
    linear_gain: f32,
) {
    if handle.is_null() {
        return;
    }
    (*handle)
        .shared
        .volume_match_gain
        .store(sanitize_gain(linear_gain).to_bits(), Ordering::Relaxed);
}

/// Set the loudness landing gain (linear; `1.0` = unity) for the Mastered path.
/// Driven by the windowed loudness measurement (Task 7).
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_set_landing_gain(
    handle: *mut LiveStream,
    linear_gain: f32,
) {
    if handle.is_null() {
        return;
    }
    (*handle)
        .shared
        .landing_gain
        .store(sanitize_gain(linear_gain).to_bits(), Ordering::Relaxed);
}

/// Snap the audio-thread smoothers to the current control targets at the next
/// render block. This is for playback start/resume only; live UI changes still
/// ramp smoothly through `process`.
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_snap_controls_to_targets(handle: *mut LiveStream) {
    if handle.is_null() {
        return;
    }
    (*handle)
        .shared
        .snap_controls_to_targets
        .store(true, Ordering::Release);
}

/// Move the shared cursor to `position_seconds`. Applied at the top of the next
/// `process` block, which also resets chain state for a clean landing.
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_seek(
    handle: *mut LiveStream,
    position_seconds: f64,
) {
    if handle.is_null() {
        return;
    }
    let stream = &*handle;
    let frame = if position_seconds.is_finite() && position_seconds > 0.0 {
        (position_seconds * stream.sample_rate as f64) as u64
    } else {
        0
    };
    let frame = frame.min(stream.total_frames as u64);
    stream
        .shared
        .seek_target_frame
        .store(frame, Ordering::Release);
}

/// Current playhead in seconds.
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_position_seconds(handle: *const LiveStream) -> f64 {
    if handle.is_null() {
        return 0.0;
    }
    let stream = &*handle;
    let frame = stream.shared.cursor_frame.load(Ordering::Relaxed);
    frame as f64 / stream.sample_rate.max(1) as f64
}

/// Total duration in seconds.
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_duration_seconds(handle: *const LiveStream) -> f64 {
    if handle.is_null() {
        return 0.0;
    }
    let stream = &*handle;
    stream.total_frames as f64 / stream.sample_rate.max(1) as f64
}

/// Channel count of the decoded PCM (so Swift can match the source-node format).
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_channels(handle: *const LiveStream) -> u32 {
    if handle.is_null() {
        return 0;
    }
    (*handle).channels as u32
}

/// Decoded source sample rate in Hz (so Swift can match the source-node format
/// and let the engine resample to the hardware rate).
///
/// # Safety
/// `handle` must be a live handle from `create` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_sample_rate(handle: *const LiveStream) -> f64 {
    if handle.is_null() {
        return 0.0;
    }
    (*handle).sample_rate as f64
}

/// Measure the live loudness landing for the given Simple controls: returns the
/// linear landing gain and writes the resulting mastered integrated LUFS into
/// `out_mastered_lufs` (`f32::NEG_INFINITY` when unavailable). Routes through the
/// shared `engine::preview_landing` so the live preview lands at the same level
/// the full render will. NOT real-time safe — call off the audio thread (it
/// processes a representative window through a throwaway chain). Reads only the
/// immutable shared PCM, so it is safe to call while `process` runs.
///
/// # Safety
/// `handle` must be a live handle from `create` or null; `preset` a C string or
/// null; `out_mastered_lufs` a writable `f32` or null.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_measure_landing(
    handle: *const LiveStream,
    preset: *const c_char,
    intensity: f32,
    lufs_target: f32,
    out_mastered_lufs: *mut f32,
) -> f32 {
    if handle.is_null() {
        return 1.0;
    }
    let stream = &*handle;
    let preset = ffi_string(preset);
    let settings = export_settings_for_options_with_context(
        preset.as_deref(),
        intensity,
        lufs_target,
        stream.adaptive_context.as_ref(),
    );
    match yes_master_lib::engine::preview_landing(
        &stream.samples,
        stream.sample_rate,
        stream.channels as u16,
        &settings,
    ) {
        Ok(landing) => {
            if !out_mastered_lufs.is_null() {
                *out_mastered_lufs = landing.mastered_lufs;
            }
            landing.gain_lin
        }
        Err(_) => {
            if !out_mastered_lufs.is_null() {
                *out_mastered_lufs = f32::NEG_INFINITY;
            }
            1.0
        }
    }
}

/// Free a handle from [`yes_master_native_live_create`].
///
/// # Safety
/// `handle` must be a handle from `create` (or null) and not used afterwards.
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_live_destroy(handle: *mut LiveStream) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::sync::Mutex;

    static ADAPTIVE_COMPRESSION_GATE_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn write_sine_wav(path: &std::path::Path, frames: u32, channels: u16) {
        let spec = hound::WavSpec {
            channels,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for n in 0..frames {
            let t = n as f32 / 48_000.0;
            let sample = (t * 440.0 * std::f32::consts::TAU).sin() * 0.3;
            let value = (sample * i16::MAX as f32) as i16;
            for _ in 0..channels {
                writer.write_sample(value).unwrap();
            }
        }
        writer.finalize().unwrap();
    }

    fn write_dense_wav(path: &std::path::Path, frames: u32, channels: u16) {
        let spec = hound::WavSpec {
            channels,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let tones = [95.0, 180.0, 430.0, 1_200.0, 3_600.0, 8_400.0];
        for n in 0..frames {
            let t = n as f32 / 48_000.0;
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
                writer.write_sample(value).unwrap();
            }
        }
        writer.finalize().unwrap();
    }

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

    /// Build a stream straight from a temp WAV via the real create path.
    fn make_stream(frames: u32, channels: u16) -> *mut LiveStream {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, frames, channels);
        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("balanced").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 0.7, -11.0) };
        // Keep the tempdir alive until after create has decoded the file.
        drop(dir);
        handle
    }

    #[test]
    fn create_rejects_missing_file() {
        let missing = CString::new("/tmp/yes-master-live-missing.wav").unwrap();
        let preset = CString::new("balanced").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(missing.as_ptr(), preset.as_ptr(), 0.5, -11.0) };
        assert!(handle.is_null());
    }

    #[test]
    fn create_rejects_null_and_empty_path() {
        let preset = CString::new("balanced").unwrap();
        let null_handle =
            unsafe { yes_master_native_live_create(std::ptr::null(), preset.as_ptr(), 0.5, -11.0) };
        assert!(null_handle.is_null());
        let empty = CString::new("").unwrap();
        let empty_handle =
            unsafe { yes_master_native_live_create(empty.as_ptr(), preset.as_ptr(), 0.5, -11.0) };
        assert!(empty_handle.is_null());
    }

    #[test]
    fn create_derives_adaptive_context_for_live_chain_updates() {
        let handle = make_stream(48_000, 2);
        assert!(!handle.is_null());
        unsafe {
            let stream = &*handle;
            let context = stream
                .adaptive_context
                .as_ref()
                .expect("live stream adaptive context");
            assert!(
                context.source_profile.is_some(),
                "live audition should use the same backend-derived source profile as export"
            );
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn bypass_returns_source_samples_unchanged() {
        let handle = make_stream(2_000, 2);
        assert!(!handle.is_null());
        unsafe {
            yes_master_native_live_set_bypass(handle, true);
            let frames = 512u32;
            let mut out = vec![0.0f32; frames as usize * 2];
            let written = yes_master_native_live_process(handle, out.as_mut_ptr(), frames);
            assert_eq!(written, frames);
            // Original passthrough at unity Volume Match == the decoded signal.
            // It is non-silent (a 0.3-amplitude sine).
            let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
            assert!(
                peak > 0.05,
                "bypassed output should carry the source signal"
            );
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn mastered_differs_from_original() {
        let handle = make_stream(4_000, 2);
        assert!(!handle.is_null());
        unsafe {
            let frames = 1_024u32;
            let n = frames as usize * 2;

            yes_master_native_live_set_bypass(handle, true);
            let mut original = vec![0.0f32; n];
            yes_master_native_live_process(handle, original.as_mut_ptr(), frames);

            yes_master_native_live_seek(handle, 0.0);
            yes_master_native_live_set_bypass(handle, false);
            let mut mastered = vec![0.0f32; n];
            yes_master_native_live_process(handle, mastered.as_mut_ptr(), frames);

            let diff: f32 = original
                .iter()
                .zip(mastered.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            assert!(
                diff > 1.0,
                "mastered output should differ from original (diff={diff})"
            );
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn toggling_bypass_preserves_cursor() {
        let handle = make_stream(10_000, 2);
        assert!(!handle.is_null());
        unsafe {
            let frames = 1_000u32;
            let mut out = vec![0.0f32; frames as usize * 2];
            yes_master_native_live_process(handle, out.as_mut_ptr(), frames);
            let pos_before = yes_master_native_live_position_seconds(handle);
            assert!(pos_before > 0.0);

            // Flipping sides must not move the playhead.
            yes_master_native_live_set_bypass(handle, true);
            let pos_after_toggle = yes_master_native_live_position_seconds(handle);
            assert_eq!(pos_before, pos_after_toggle);
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn seek_changes_position_and_clamps() {
        let handle = make_stream(48_000, 2); // 1.0 s @ 48 kHz
        assert!(!handle.is_null());
        unsafe {
            assert!((yes_master_native_live_duration_seconds(handle) - 1.0).abs() < 0.01);
            yes_master_native_live_seek(handle, 0.5);
            // Seek is applied at the next process block.
            let mut out = vec![0.0f32; 2 * 2];
            yes_master_native_live_process(handle, out.as_mut_ptr(), 2);
            let pos = yes_master_native_live_position_seconds(handle);
            assert!((0.5..=0.55).contains(&pos), "expected ~0.5s, got {pos}");

            // Seeking past the end clamps to duration.
            yes_master_native_live_seek(handle, 999.0);
            yes_master_native_live_process(handle, out.as_mut_ptr(), 2);
            let pos = yes_master_native_live_position_seconds(handle);
            assert!(pos <= 1.0 + 1e-6, "seek past EOF should clamp, got {pos}");
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn set_params_does_not_recreate_handle() {
        let handle = make_stream(4_000, 2);
        assert!(!handle.is_null());
        unsafe {
            let warm = CString::new("warm").unwrap();
            // Same handle, new params — must not crash or require recreation.
            yes_master_native_live_set_params(handle, warm.as_ptr(), 0.9, -9.0);
            let frames = 1_024u32;
            let mut out = vec![0.0f32; frames as usize * 2];
            let written = yes_master_native_live_process(handle, out.as_mut_ptr(), frames);
            assert_eq!(written, frames);
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn output_is_block_size_independent() {
        // With no parameter changes the chain is deterministic and stateful, so
        // processing in one big block must equal processing in small blocks.
        // This guards the cursor/block-boundary port of the desktop source.
        let total_frames = 5_000u32;
        let chans = 2usize;

        let one_shot = make_stream(total_frames, 2);
        let chunked = make_stream(total_frames, 2);
        assert!(!one_shot.is_null() && !chunked.is_null());

        unsafe {
            let mut a = vec![0.0f32; total_frames as usize * chans];
            yes_master_native_live_process(one_shot, a.as_mut_ptr(), total_frames);

            let mut b = vec![0.0f32; total_frames as usize * chans];
            let chunk = 137u32; // deliberately not aligned to the 128 check interval
            let mut done = 0u32;
            while done < total_frames {
                let n = chunk.min(total_frames - done);
                let offset = done as usize * chans;
                yes_master_native_live_process(chunked, b[offset..].as_mut_ptr(), n);
                done += n;
            }

            let max_diff = a
                .iter()
                .zip(b.iter())
                .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()));
            assert!(
                max_diff < 1e-6,
                "block-size must not change output; max_diff={max_diff}"
            );

            yes_master_native_live_destroy(one_shot);
            yes_master_native_live_destroy(chunked);
        }
    }

    #[test]
    fn params_changed_at_eof_survive_seek_back() {
        // Regression: a coefficient change made while parked past EOF must still
        // be drained (the unbounded mpsc must not grow forever) AND survive a
        // seek back into the file. Before the fix the per-frame drain was
        // skipped at EOF and a seek discarded the pending chain, so the change
        // was silently lost.
        unsafe {
            // Stream A: play to EOF, switch to a strong "warm" preset while
            // parked, then seek home and listen Mastered.
            let a = make_stream(2_000, 2);
            assert!(!a.is_null());
            let mut scratch = vec![0.0f32; 4_096 * 2];
            yes_master_native_live_process(a, scratch.as_mut_ptr(), 4_096); // parks past EOF
            let warm = CString::new("warm").unwrap();
            yes_master_native_live_set_params(a, warm.as_ptr(), 1.0, -9.0);
            yes_master_native_live_process(a, scratch.as_mut_ptr(), 256); // must drain at EOF
            yes_master_native_live_seek(a, 0.0);
            yes_master_native_live_set_bypass(a, false);
            let mut warmed = vec![0.0f32; 2_000 * 2];
            yes_master_native_live_process(a, warmed.as_mut_ptr(), 2_000);

            // Stream B: same source, left at the default "balanced" preset.
            let b = make_stream(2_000, 2);
            assert!(!b.is_null());
            yes_master_native_live_set_bypass(b, false);
            let mut balanced = vec![0.0f32; 2_000 * 2];
            yes_master_native_live_process(b, balanced.as_mut_ptr(), 2_000);

            assert!(warmed.iter().all(|v| v.is_finite()));
            let peak = warmed.iter().fold(0.0f32, |m, v| m.max(v.abs()));
            assert!(peak > 0.01, "warmed master should be audible, peak={peak}");
            let diff: f32 = warmed
                .iter()
                .zip(balanced.iter())
                .map(|(x, y)| (x - y).abs())
                .sum();
            assert!(
                diff > 1.0,
                "preset change at EOF should survive the seek (diff={diff})"
            );

            yes_master_native_live_destroy(a);
            yes_master_native_live_destroy(b);
        }
    }

    #[test]
    fn live_output_matches_desktop_chain_bit_for_bit() {
        // The whole premise of the feature: the iPhone live path must run the
        // EXACT desktop MasteringChain. With no param changes and unity audition
        // gains, the streamed output equals MasteringChain::process_interleaved
        // over the same decoded PCM + settings.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 5_000, 2);

        let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
        let channels = pcm.channels as usize;
        let total = pcm.samples.len() / channels;
        let settings = crate::export_settings_for_options(Some("warm"), 0.7, -11.0);

        // Desktop reference: build the chain and process the whole buffer.
        let mut reference = pcm.samples.clone();
        let mut chain = MasteringChain::new(pcm.sample_rate, channels, &settings);
        chain.process_interleaved(&mut reference, channels);

        // iPhone live path: same file + settings, Mastered, no param changes.
        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("warm").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 0.7, -11.0) };
        assert!(!handle.is_null());
        let mut live = vec![0.0f32; total * channels];
        unsafe {
            yes_master_native_live_process(handle, live.as_mut_ptr(), total as u32);
            yes_master_native_live_destroy(handle);
        }

        let max_diff = reference
            .iter()
            .zip(live.iter())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(
            max_diff < 1e-6,
            "iPhone live output must equal the desktop MasteringChain; max_diff={max_diff}"
        );
    }

    #[test]
    fn adaptive_compressor_gate_off_live_output_matches_desktop_chain_bit_for_bit() {
        let _lock = ADAPTIVE_COMPRESSION_GATE_TEST_LOCK
            .lock()
            .expect("adaptive compression gate test lock");
        let _gate = adaptive_compression_gate_for_test(false);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dense-gate-off.wav");
        write_dense_wav(&path, 48_000 * 4, 2);

        let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
        let channels = pcm.channels as usize;
        let total = pcm.samples.len() / channels;
        let context = crate::native_adaptive_context_for_path(&path).expect("adaptive context");
        let settings = crate::export_settings_for_options_with_context(
            Some("heavy"),
            1.0,
            -9.0,
            Some(&context),
        );
        let plan = yes_master_lib::guardrails::compression_plan_for_resolved_settings(&settings);
        assert!(
            !plan.active,
            "gate-off dense fixture must not resolve active adaptive compression guards: {plan:?}"
        );

        let mut reference = pcm.samples.clone();
        let mut chain = MasteringChain::new(pcm.sample_rate, channels, &settings);
        chain.process_interleaved(&mut reference, channels);

        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("heavy").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 1.0, -9.0) };
        assert!(!handle.is_null());
        let mut live = vec![0.0f32; total * channels];
        unsafe {
            yes_master_native_live_process(handle, live.as_mut_ptr(), total as u32);
            yes_master_native_live_destroy(handle);
        }

        let max_diff = reference
            .iter()
            .zip(live.iter())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(
            max_diff < 1e-6,
            "iPhone gate-off adaptive-compressor output must equal the desktop MasteringChain; max_diff={max_diff}"
        );
    }

    #[test]
    fn adaptive_compressor_live_output_matches_desktop_chain_bit_for_bit() {
        let _lock = ADAPTIVE_COMPRESSION_GATE_TEST_LOCK
            .lock()
            .expect("adaptive compression gate test lock");
        let _gate = adaptive_compression_gate_for_test(true);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dense.wav");
        write_dense_wav(&path, 48_000 * 4, 2);

        let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
        let channels = pcm.channels as usize;
        let total = pcm.samples.len() / channels;
        let context = crate::native_adaptive_context_for_path(&path).expect("adaptive context");
        let settings = crate::export_settings_for_options_with_context(
            Some("heavy"),
            1.0,
            -9.0,
            Some(&context),
        );
        let plan = yes_master_lib::guardrails::compression_plan_for_resolved_settings(&settings);
        assert!(
            plan.active,
            "dense fixture must resolve adaptive compression guards: {plan:?}"
        );

        let mut reference = pcm.samples.clone();
        let mut chain = MasteringChain::new(pcm.sample_rate, channels, &settings);
        chain.process_interleaved(&mut reference, channels);

        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("heavy").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 1.0, -9.0) };
        assert!(!handle.is_null());
        let mut live = vec![0.0f32; total * channels];
        unsafe {
            yes_master_native_live_process(handle, live.as_mut_ptr(), total as u32);
            yes_master_native_live_destroy(handle);
        }

        let max_diff = reference
            .iter()
            .zip(live.iter())
            .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(
            max_diff < 1e-6,
            "iPhone adaptive-compressor output must equal the desktop MasteringChain; max_diff={max_diff}"
        );
    }

    #[test]
    fn intensity_is_wired_and_matches_desktop_curve() {
        // Intensity must (a) actually change the live output and (b) match the
        // desktop preset_scale curve at each setting. Intensity 0 is NOT bypass:
        // desktop maps preset_scale = 0.4 + 1.2*intensity, so 0.0 -> 0.4x preset
        // (subtle) and 1.0 -> 1.6x (pushed). They differ, but neither is "off".
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 6_000, 2);
        let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
        let channels = pcm.channels as usize;
        let total = pcm.samples.len() / channels;

        let render_live = |intensity: f32| -> Vec<f32> {
            let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let preset = CString::new("punch").unwrap();
            let handle = unsafe {
                yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), intensity, -11.0)
            };
            assert!(!handle.is_null());
            let mut out = vec![0.0f32; total * channels];
            unsafe {
                yes_master_native_live_process(handle, out.as_mut_ptr(), total as u32);
                yes_master_native_live_destroy(handle);
            }
            out
        };

        let low = render_live(0.0);
        let high = render_live(1.0);
        let diff: f32 = low
            .iter()
            .zip(high.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            diff > 1.0,
            "intensity must change the live output (diff={diff})"
        );

        for intensity in [0.0f32, 1.0] {
            let settings = crate::export_settings_for_options(Some("punch"), intensity, -11.0);
            let mut reference = pcm.samples.clone();
            let mut chain = MasteringChain::new(pcm.sample_rate, channels, &settings);
            chain.process_interleaved(&mut reference, channels);
            let live = render_live(intensity);
            let max_diff = reference
                .iter()
                .zip(live.iter())
                .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
            assert!(
                max_diff < 1e-6,
                "live must match desktop chain at intensity {intensity}; max_diff={max_diff}"
            );
        }
    }

    #[test]
    fn set_params_intensity_changes_live_output() {
        // The slider path: set_params with a new intensity must change the audible
        // output. Two streams start identical at intensity 0; one is swept to 1.0
        // via set_params. After the crossfade their outputs must diverge.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 8_000, 2);
        let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
        let channels = pcm.channels as usize;

        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("punch").unwrap();
        let steady =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 0.0, -11.0) };
        let swept =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 0.0, -11.0) };
        assert!(!steady.is_null() && !swept.is_null());

        unsafe { yes_master_native_live_set_params(swept, preset.as_ptr(), 1.0, -11.0) };

        let frames = 4_000u32;
        let mut a = vec![0.0f32; frames as usize * channels];
        let mut b = vec![0.0f32; frames as usize * channels];
        unsafe {
            yes_master_native_live_process(steady, a.as_mut_ptr(), frames);
            yes_master_native_live_process(swept, b.as_mut_ptr(), frames);
        }
        let diff: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
        assert!(
            diff > 1.0,
            "set_params(intensity) must change the live output (diff={diff})"
        );
        unsafe {
            yes_master_native_live_destroy(steady);
            yes_master_native_live_destroy(swept);
        }
    }

    #[test]
    fn measure_landing_is_wired_and_responds_to_target() {
        // 10 s source so the 8 s landing window is full. Proves the measurement
        // FFI wires through to engine::preview_landing and responds to the target:
        // a louder target never yields a smaller gain or a quieter result.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 48_000 * 10, 2);
        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("balanced").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 0.5, -14.0) };
        assert!(!handle.is_null());

        let mut lufs_quiet = f32::NEG_INFINITY;
        let gain_quiet = unsafe {
            yes_master_native_live_measure_landing(
                handle,
                preset.as_ptr(),
                0.5,
                -14.0,
                &mut lufs_quiet,
            )
        };
        let mut lufs_loud = f32::NEG_INFINITY;
        let gain_loud = unsafe {
            yes_master_native_live_measure_landing(
                handle,
                preset.as_ptr(),
                0.5,
                -9.0,
                &mut lufs_loud,
            )
        };

        assert!(
            gain_quiet.is_finite() && gain_quiet > 0.0,
            "gain must be finite positive"
        );
        assert!(
            lufs_quiet.is_finite() && lufs_loud.is_finite(),
            "mastered LUFS must be finite"
        );
        assert!(
            gain_loud >= gain_quiet - 1e-6,
            "a louder target must not yield a smaller gain (quiet={gain_quiet}, loud={gain_loud})"
        );
        assert!(
            lufs_loud >= lufs_quiet - 1e-3,
            "a louder target must not land quieter (quiet={lufs_quiet}, loud={lufs_loud})"
        );
        unsafe { yes_master_native_live_destroy(handle) };
    }

    #[test]
    fn gain_changes_are_smoothed_not_stepped() {
        // The fix for click-on-Volume-Match: a gain change must RAMP, not step.
        // Drop Volume Match to 0 — with smoothing the frames right after the change
        // are still audible and the tail fades to ~silence; an instantaneous step
        // would silence everything immediately (head_peak ~ 0), failing this test.
        let handle = make_stream(48_000, 2); // 1 s @ 48k — well past the ~12 ms ramp
        assert!(!handle.is_null());
        unsafe {
            // Settle the smoother at unity first.
            let mut warm = vec![0.0f32; 1_024 * 2];
            yes_master_native_live_process(handle, warm.as_mut_ptr(), 1_024);

            yes_master_native_live_set_volume_match(handle, 0.0);
            let frames = 4_096usize;
            let mut out = vec![0.0f32; frames * 2];
            yes_master_native_live_process(handle, out.as_mut_ptr(), frames as u32);

            let head_peak = out[..64 * 2].iter().fold(0.0f32, |m, v| m.max(v.abs()));
            let tail_peak = out[(frames - 64) * 2..]
                .iter()
                .fold(0.0f32, |m, v| m.max(v.abs()));
            assert!(
                head_peak > 0.01,
                "gain must ramp (head still audible), head={head_peak}"
            );
            assert!(
                tail_peak < head_peak * 0.1,
                "gain must reach ~0 after the ramp (tail={tail_peak}, head={head_peak})"
            );
            yes_master_native_live_destroy(handle);
        }
    }

    #[test]
    fn snapped_original_starts_dry_from_first_frame() {
        // The Swift app sets Original before playback starts. The first render
        // must honor that target immediately, not fade from the default Mastered
        // smoother state into Original over the first few milliseconds.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.wav");
        write_sine_wav(&path, 2_000, 2);
        let pcm = yes_master_lib::decode::decode_full(&path).unwrap();
        let channels = pcm.channels as usize;

        let c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let preset = CString::new("balanced").unwrap();
        let handle =
            unsafe { yes_master_native_live_create(c.as_ptr(), preset.as_ptr(), 0.7, -11.0) };
        assert!(!handle.is_null());

        unsafe {
            yes_master_native_live_set_bypass(handle, true);
            yes_master_native_live_snap_controls_to_targets(handle);
            let frames = 256u32;
            let mut out = vec![0.0f32; frames as usize * channels];
            yes_master_native_live_process(handle, out.as_mut_ptr(), frames);
            yes_master_native_live_destroy(handle);

            let expected = &pcm.samples[..out.len()];
            let max_diff = expected
                .iter()
                .zip(out.iter())
                .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
            assert!(
                max_diff < 1e-6,
                "snapped Original should be dry from the first frame; max_diff={max_diff}"
            );
        }
    }
}
