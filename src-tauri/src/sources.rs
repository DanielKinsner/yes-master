//! Rodio `Source` implementations for playback.
//!
//! Two flavors of the same shape:
//!
//! - [`MeteredPcmSource`] streams raw decoded PCM unchanged. Used for
//!   Original playback so A/B metering hits the same peak / LUFS /
//!   spectrum slots as Mastered playback.
//! - [`MasteringSource`] streams PCM through `crate::dsp::MasteringChain`
//!   with hot-swappable coefficients delivered over an mpsc channel.
//!   Coefficient changes trigger a short crossfade between the old and
//!   new chain so filter-state transients don't audibly snap.
//!
//! Both sources push their metering state into shared atomics
//! (`peak_linear`, `lufs_x100`, `integrated_lufs_x100`) and the post-mix
//! mono signal into a `SpectrumRing`; the audio thread's snapshot tick
//! reads from those slots without ever blocking the audio loop.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use crate::spectrum::SpectrumRing;

/// Length of the click-suppressing fade applied on an Original<->Mastered swap
/// (L10). ~22 ms is long enough to mask the zero-crossing discontinuity that
/// made the toggle click, short enough that audition stays responsive.
pub(crate) const SWAP_FADE_MS: usize = 22;

/// `SWAP_FADE_MS` expressed in frames at `sample_rate` (min 1 so the ramp math
/// never divides by zero on a degenerate rate).
pub(crate) fn swap_fade_frames(sample_rate: u32) -> usize {
    ((sample_rate as usize).saturating_mul(SWAP_FADE_MS) / 1000).max(1)
}

/// Per-frame outcome of the swap fade envelope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FrameFade {
    /// Scale the produced frame by this gain (`1.0` = untouched).
    Gain(f32),
    /// A triggered fade-out has fully rolled off — the source must end (return
    /// `None`) so its detached sink drains and self-cleans.
    End,
}

/// Click-free gain envelope for Original<->Mastered swaps, evaluated once per
/// frame *inside* the source (see [`MeteredPcmSource`] / [`MasteringSource`]).
///
/// Why not rodio's `FadeIn`/`FadeOut` combinators: their ramp clock is driven by
/// elapsed playback time, and `Sink::try_seek` sets that clock to the seek
/// target — so wrapping a source in `fade_in` and then seeking to a non-zero
/// playhead skips the fade entirely. Counting *emitted frames* here decouples
/// the fade from the seek, which is exactly the "seek first, then fade in"
/// sequence a playhead-preserving swap needs.
///
/// A swap drives the envelope on two sources, sequentially (never a dual-sink
/// crossfade):
/// - the **outgoing** source is triggered via `fade_out` (a flag the audio
///   thread sets at swap time); it ramps `last_gain -> 0` over `fade_out_frames`
///   then ends, so its detached sink drains.
/// - the **incoming** source stays silent for `lead_in_frames` (so the outgoing
///   fade-out is never audible at the same instant) then ramps `0 -> 1` over
///   `fade_in_frames`.
///
/// A fresh play (no prior source to replace) uses `lead_in_frames == 0` and
/// `fade_in_frames == 0`, so it is identical to ungated playback; it still
/// carries a `fade_out` trigger so a *later* toggle can fade it out.
pub(crate) struct FadeEnvelope {
    lead_in_frames: usize,
    fade_in_frames: usize,
    fade_out_frames: usize,
    fade_out: Arc<AtomicBool>,
    emitted_frames: usize,
    fade_out_remaining: Option<usize>,
    fade_out_start_gain: f32,
    last_gain: f32,
}

impl FadeEnvelope {
    pub(crate) fn new(
        lead_in_frames: usize,
        fade_in_frames: usize,
        fade_out_frames: usize,
        fade_out: Arc<AtomicBool>,
    ) -> Self {
        Self {
            lead_in_frames,
            fade_in_frames,
            fade_out_frames,
            fade_out,
            emitted_frames: 0,
            fade_out_remaining: None,
            fade_out_start_gain: 1.0,
            last_gain: 1.0,
        }
    }

    /// An envelope that never alters the signal — used by non-swap construction
    /// (and tests). Its trigger is private (nothing else clones it), so it can
    /// never fire.
    pub(crate) fn inactive() -> Self {
        Self::new(0, 0, 0, Arc::new(AtomicBool::new(false)))
    }

    /// Advance the envelope by one frame and return the gain to apply (or `End`
    /// once a triggered fade-out has fully rolled off). Call exactly once per
    /// produced frame.
    pub(crate) fn advance_frame(&mut self) -> FrameFade {
        // A swap was requested on this (outgoing) source: latch the fade-out,
        // starting from whatever gain we last emitted so a re-toggle mid-fade
        // never jumps back to full scale (guards rapid toggling).
        if self.fade_out_remaining.is_none() && self.fade_out.load(Ordering::Relaxed) {
            self.fade_out_remaining = Some(self.fade_out_frames);
            self.fade_out_start_gain = self.last_gain;
        }
        if let Some(remaining) = self.fade_out_remaining {
            if remaining == 0 {
                return FrameFade::End;
            }
            // Ramp start_gain -> ~0 across fade_out_frames. With realistic frame
            // counts (~1000 at 48 kHz) the final pre-`End` frame sits near
            // -60 dB, so the cut to silence is inaudible.
            let progress = remaining as f32 / self.fade_out_frames.max(1) as f32;
            let gain = self.fade_out_start_gain * progress;
            self.fade_out_remaining = Some(remaining - 1);
            self.last_gain = gain;
            return FrameFade::Gain(gain);
        }

        let gain = if self.emitted_frames < self.lead_in_frames {
            0.0
        } else if self.emitted_frames < self.lead_in_frames + self.fade_in_frames {
            let k = self.emitted_frames - self.lead_in_frames;
            (k as f32 + 1.0) / self.fade_in_frames.max(1) as f32
        } else {
            1.0
        };
        self.emitted_frames += 1;
        self.last_gain = gain;
        FrameFade::Gain(gain)
    }
}

pub(crate) struct LiveCoeffUpdate {
    pub(crate) generation: u64,
    pub(crate) coeffs: crate::dsp::ChainCoeffs,
}

/// How many frames to process before draining the coefficient channel. At
/// 44.1 kHz this is ~3 ms — well below the perception threshold for parameter
/// changes.
const COEFFS_CHECK_INTERVAL_FRAMES: usize = 128;
/// Crossfade length between old and new chain when coefficients change.
/// 512 frames ≈ 12 ms at 44.1 kHz. Long enough to mask filter-state transients
/// on preset/intensity changes; short enough to feel instantaneous.
const COEFFS_CROSSFADE_FRAMES: usize = 512;

/// Fold one channel sample's magnitude into a shared per-channel peak slot,
/// same bit-encoding/`fetch_max` discipline as the all-channel `peak_linear`.
/// Non-finite samples are skipped so a stray NaN can't poison the meter.
fn fold_channel_peak(slot: &Arc<AtomicU32>, sample: f32) {
    let abs = sample.abs();
    if abs.is_finite() {
        slot.fetch_max(abs.to_bits(), Ordering::Relaxed);
    }
}

/// Channel pair fed to the live BS.1770 LUFS meters. Stereo passes both
/// channels; mono passes `(l, 0.0)` so the meter's channel-energy sum
/// (`l² + r²`) equals true single-channel mono loudness instead of the
/// `+3.01 LU` (`10·log10(2)`) offset that duplicating mono into both
/// channels would add. The peak meters still mirror mono onto both channels
/// for display — only the loudness sum must avoid the double-count.
fn lufs_meter_input(frame: &[f32], channels: usize) -> (f32, f32) {
    let l = frame.first().copied().unwrap_or(0.0);
    if channels >= 2 {
        (l, frame.get(1).copied().unwrap_or(0.0))
    } else {
        (l, 0.0)
    }
}

/// Output slots are independent of the processor/source sample rate. They can
/// be handed to a meter after device conversion without duplicating meter work.
#[derive(Clone)]
pub(crate) struct MeterSlots {
    peak: Arc<AtomicU32>,
    left: Arc<AtomicU32>,
    right: Arc<AtomicU32>,
    momentary: Arc<AtomicI32>,
    integrated: Arc<AtomicI32>,
    spectrum: Arc<SpectrumRing>,
}

struct OutputMeter {
    slots: MeterSlots,
    momentary: crate::dsp::MomentaryLufs,
    integrated: crate::dsp::IntegratedLufs,
}

impl OutputMeter {
    fn new(rate: u32, slots: MeterSlots) -> Self {
        Self {
            slots,
            momentary: crate::dsp::MomentaryLufs::new(rate),
            integrated: crate::dsp::IntegratedLufs::new(rate),
        }
    }

    fn process(&mut self, frame: &[f32]) {
        let mut peak = 0_f32;
        for sample in frame {
            let value = sample.abs();
            if value.is_finite() && value > peak {
                peak = value;
            }
        }
        self.slots.peak.fetch_max(peak.to_bits(), Ordering::Relaxed);
        let l = frame.first().copied().unwrap_or(0.);
        let r = if frame.len() >= 2 { frame[1] } else { l };
        fold_channel_peak(&self.slots.left, l);
        fold_channel_peak(&self.slots.right, r);
        let to_x100 = |value: f32| {
            if value.is_finite() && value > -120. {
                (value * 100.) as i32
            } else {
                i32::MIN
            }
        };
        let (ml, mr) = lufs_meter_input(frame, frame.len());
        self.slots.momentary.store(
            to_x100(self.momentary.process_frame(ml, mr)),
            Ordering::Relaxed,
        );
        self.slots.integrated.store(
            to_x100(self.integrated.process_frame(ml, mr)),
            Ordering::Relaxed,
        );
        let mono = (l + r) * 0.5;
        if mono.is_finite() {
            self.slots.spectrum.push(mono);
        }
    }

    fn reset(&mut self) {
        self.momentary.reset();
        self.integrated.reset();
    }
}

/// Device-rate meter/fade stage. Qualified as a test-only wrapper before the
/// live route is switched; it consumes the converter's actual emitted frames.
#[cfg(test)]
pub(crate) struct MeteredSource<S: rodio::Source<Item = f32>> {
    source: S,
    frame: Vec<f32>,
    position: usize,
    meter: OutputMeter,
    fade: FadeEnvelope,
}

#[cfg(test)]
impl<S: rodio::Source<Item = f32>> MeteredSource<S> {
    pub(crate) fn new(source: S, slots: MeterSlots, fade: FadeEnvelope) -> Self {
        let channels = usize::from(source.channels());
        assert!(channels > 0);
        let meter = OutputMeter::new(source.sample_rate(), slots);
        Self {
            source,
            frame: vec![0.; channels],
            position: channels,
            meter,
            fade,
        }
    }
}

#[cfg(test)]
impl<S: rodio::Source<Item = f32>> Iterator for MeteredSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.position == self.frame.len() {
            let gain = match self.fade.advance_frame() {
                FrameFade::End => return None,
                FrameFade::Gain(value) => value,
            };
            for value in &mut self.frame {
                *value = self.source.next()? * gain;
            }
            self.meter.process(&self.frame);
            self.position = 0;
        }
        let value = self.frame[self.position];
        self.position += 1;
        Some(value)
    }
}

#[cfg(test)]
impl<S: rodio::Source<Item = f32>> rodio::Source for MeteredSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.source.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.source.try_seek(pos)?;
        self.position = self.frame.len();
        self.meter.reset();
        Ok(())
    }
}

/// Pass-through source for Original playback that still feeds the same peak,
/// LUFS, and spectrum meter path as Mastered playback. This keeps A/B metering
/// honest without routing Original through any mastering DSP.
pub(crate) struct MeteredPcmSource {
    samples: Arc<Vec<f32>>,
    position: usize,
    channels: u16,
    sample_rate: u32,
    frame: Vec<f32>,
    frame_out_pos: usize,
    meter: Option<OutputMeter>,
    /// L10 — click-free Original<->Mastered swap fade. Inactive by default; the
    /// live audio path installs a real one via [`Self::with_swap_fade`].
    fade: FadeEnvelope,
}

impl MeteredPcmSource {
    #[cfg(test)]
    pub(crate) fn take_meter_slots(&mut self) -> MeterSlots {
        self.meter.take().expect("meter already moved").slots
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        samples: impl Into<Arc<Vec<f32>>>,
        channels: u16,
        sample_rate: u32,
        peak_linear: Arc<AtomicU32>,
        peak_left_linear: Arc<AtomicU32>,
        peak_right_linear: Arc<AtomicU32>,
        lufs_x100: Arc<AtomicI32>,
        integrated_lufs_x100: Arc<AtomicI32>,
        spectrum_ring: Arc<SpectrumRing>,
    ) -> Self {
        let channels_usize = channels.max(1) as usize;
        Self {
            samples: samples.into(),
            position: 0,
            channels,
            sample_rate,
            frame: vec![0.0; channels_usize],
            frame_out_pos: channels_usize,
            meter: Some(OutputMeter::new(
                sample_rate,
                MeterSlots {
                    peak: peak_linear,
                    left: peak_left_linear,
                    right: peak_right_linear,
                    momentary: lufs_x100,
                    integrated: integrated_lufs_x100,
                    spectrum: spectrum_ring,
                },
            )),
            fade: FadeEnvelope::inactive(),
        }
    }

    /// Install the L10 swap fade envelope. Only the live audio path calls this;
    /// tests and other construction keep the inactive default from [`Self::new`].
    pub(crate) fn with_swap_fade(mut self, fade: FadeEnvelope) -> Self {
        self.fade = fade;
        self
    }
}

impl Iterator for MeteredPcmSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let channels = self.channels.max(1) as usize;
        if self.frame_out_pos >= channels {
            if self.position >= self.samples.len() {
                return None;
            }
            // L10 — advance the swap fade once per frame. `End` means a triggered
            // fade-out finished, so the source ends and its detached sink drains.
            let fade_gain = match self.fade.advance_frame() {
                FrameFade::End => return None,
                FrameFade::Gain(g) => g,
            };

            for i in 0..channels {
                self.frame[i] = if self.position + i < self.samples.len() {
                    self.samples[self.position + i]
                } else {
                    0.0
                };
            }
            self.position += channels;

            // Apply the fade before metering so the meters track what is heard.
            if fade_gain != 1.0 {
                for v in &mut self.frame[..channels] {
                    *v *= fade_gain;
                }
            }

            if let Some(meter) = &mut self.meter {
                meter.process(&self.frame[..channels]);
            }

            self.frame_out_pos = 0;
        }

        let out = self.frame[self.frame_out_pos];
        self.frame_out_pos += 1;
        Some(out)
    }
}

impl rodio::Source for MeteredPcmSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels.max(1)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let total_frames = self.samples.len() / self.channels.max(1) as usize;
        if self.sample_rate == 0 {
            None
        } else {
            Some(Duration::from_secs_f64(
                total_frames as f64 / self.sample_rate as f64,
            ))
        }
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        let channels = self.channels.max(1) as usize;
        let target_frame = (pos.as_secs_f64() * self.sample_rate as f64) as usize;
        let target_sample = target_frame.saturating_mul(channels);
        self.position = target_sample.min(self.samples.len());
        if let Some(meter) = &mut self.meter {
            meter.reset();
        }
        self.frame_out_pos = channels;
        Ok(())
    }
}

/// A rodio Source that streams interleaved PCM through the DSP chain.
/// Coefficient updates flow in via mpsc; samples are picked up at most
/// every `COEFFS_CHECK_INTERVAL_FRAMES` samples (~3 ms at 44.1 kHz). When
/// new coefficients arrive, a `COEFFS_CROSSFADE_FRAMES`-long crossfade
/// between the old and new chain hides filter-state transients.
pub(crate) struct MasteringSource {
    samples: Arc<Vec<f32>>,
    position: usize,
    channels: u16,
    sample_rate: u32,
    chain: crate::dsp::MasteringChain,
    pending_chain: crate::dsp::MasteringChain,
    pending_chain_active: bool,
    crossfade_remaining: usize,
    crossfade_total: usize,
    coeffs_rx: mpsc::Receiver<LiveCoeffUpdate>,
    coeffs_generation: u64,
    frames_since_check: usize,
    // Frame-level scratch buffers; preallocated to avoid heap traffic on the
    // audio thread.
    frame_in: Vec<f32>,
    frame_main: Vec<f32>,
    frame_pending: Vec<f32>,
    frame_out_pos: usize,
    render_aligned: bool,
    prime_samples_left: usize,
    tail_frames_left: usize,
    meter: Option<OutputMeter>,
    fade: FadeEnvelope,
}

impl MasteringSource {
    /// Move meter ownership to a downstream device-rate stage. Called before
    /// playback, so the old rate's meter allocations are dropped off-callback.
    #[cfg(test)]
    pub(crate) fn take_meter_slots(&mut self) -> MeterSlots {
        self.meter.take().expect("meter already moved").slots
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        samples: impl Into<Arc<Vec<f32>>>,
        channels: u16,
        sample_rate: u32,
        chain: crate::dsp::MasteringChain,
        coeffs_rx: mpsc::Receiver<LiveCoeffUpdate>,
        peak_linear: Arc<AtomicU32>,
        peak_left_linear: Arc<AtomicU32>,
        peak_right_linear: Arc<AtomicU32>,
        lufs_x100: Arc<AtomicI32>,
        integrated_lufs_x100: Arc<AtomicI32>,
        spectrum_ring: Arc<SpectrumRing>,
    ) -> Self {
        let channels_usize = channels.max(1) as usize;
        let pending_chain =
            crate::dsp::MasteringChain::with_coeffs_inheriting_state(chain.coeffs, &chain);
        Self {
            samples: samples.into(),
            position: 0,
            channels,
            sample_rate,
            chain,
            pending_chain,
            pending_chain_active: false,
            crossfade_remaining: 0,
            crossfade_total: 0,
            coeffs_rx,
            coeffs_generation: 0,
            frames_since_check: 0,
            frame_in: vec![0.0; channels_usize],
            frame_main: vec![0.0; channels_usize],
            frame_pending: vec![0.0; channels_usize],
            // Setting to `channels_usize` triggers the fetch on the first
            // `next()` call rather than requiring a separate "primed" flag.
            frame_out_pos: channels_usize,
            render_aligned: false,
            prime_samples_left: 0,
            tail_frames_left: 0,
            meter: Some(OutputMeter::new(
                sample_rate,
                MeterSlots {
                    peak: peak_linear,
                    left: peak_left_linear,
                    right: peak_right_linear,
                    momentary: lufs_x100,
                    integrated: integrated_lufs_x100,
                    spectrum: spectrum_ring,
                },
            )),
            fade: FadeEnvelope::inactive(),
        }
    }

    /// Install the L10 swap fade envelope. Only the live audio path calls this;
    /// tests and other construction keep the inactive default from [`Self::new`].
    pub(crate) fn with_swap_fade(mut self, fade: FadeEnvelope) -> Self {
        self.fade = fade;
        self
    }

    /// Qualify finite playback against the file renderer before enabling this
    /// in production. Remove lookahead once and drain the same number of frames.
    #[cfg(test)]
    pub(crate) fn with_render_alignment(mut self) -> Self {
        self.render_aligned = true;
        self.reset_render_alignment();
        self
    }

    fn reset_render_alignment(&mut self) {
        if self.render_aligned {
            self.tail_frames_left = self.chain.latency_frames();
            self.prime_samples_left = self.tail_frames_left * self.channels.max(1) as usize;
        }
    }

    fn next_unaligned(&mut self) -> Option<f32> {
        let channels = self.channels.max(1) as usize;
        if self.frame_out_pos >= channels {
            // Time to fetch + process the next input frame.
            if self.position >= self.samples.len() {
                if self.tail_frames_left == 0 {
                    return None;
                }
                self.tail_frames_left -= 1;
            }
            // L10 — advance the swap fade once per frame. `End` means a triggered
            // fade-out finished, so the source ends and its detached sink drains.
            let fade_gain = if self.prime_samples_left > 0 {
                // Discarded lookahead must not consume the audible swap fade.
                1.0
            } else {
                match self.fade.advance_frame() {
                    FrameFade::End => return None,
                    FrameFade::Gain(g) => g,
                }
            };

            // Pull one frame out of the source PCM. If we're short at the end
            // of the file, zero-pad — keeps the limiter happy.
            for i in 0..channels {
                self.frame_in[i] = if self.position + i < self.samples.len() {
                    self.samples[self.position + i]
                } else {
                    0.0
                };
            }
            self.position += channels;

            // Coefficient check / crossfade arming.
            self.frames_since_check += 1;
            if self.frames_since_check >= COEFFS_CHECK_INTERVAL_FRAMES {
                self.frames_since_check = 0;
                let mut latest: Option<LiveCoeffUpdate> = None;
                while let Ok(update) = self.coeffs_rx.try_recv() {
                    if update.generation >= self.coeffs_generation {
                        match latest {
                            Some(ref current) if current.generation > update.generation => {}
                            _ => latest = Some(update),
                        }
                    }
                }
                if let Some(update) = latest {
                    self.coeffs_generation = update.generation;
                    // If a crossfade is already in progress, promote the
                    // current pending chain to main BEFORE installing the new
                    // pending. Without this, sustained updates (knob sweeps)
                    // re-arm the 512-frame crossfade every check interval,
                    // self.chain stays frozen at the pre-sweep coefficients,
                    // and the source runs 2x DSP for the entire sweep while
                    // the output remains weighted ~75% toward the stale
                    // chain. Promoting first bounds the 2x window to a single
                    // COEFFS_CROSSFADE_FRAMES interval per update and keeps
                    // the audible chain tracking the latest settings.
                    if self.pending_chain_active {
                        std::mem::swap(&mut self.chain, &mut self.pending_chain);
                        self.pending_chain_active = false;
                    }
                    if !self
                        .pending_chain
                        .overwrite_with_coeffs_inheriting_state(update.coeffs, &self.chain)
                    {
                        self.pending_chain =
                            crate::dsp::MasteringChain::with_coeffs_inheriting_state(
                                update.coeffs,
                                &self.chain,
                            );
                    }
                    self.pending_chain_active = true;
                    self.crossfade_remaining = COEFFS_CROSSFADE_FRAMES;
                    self.crossfade_total = COEFFS_CROSSFADE_FRAMES;
                }
            }

            // Process the main chain into frame_main.
            for i in 0..channels {
                self.frame_main[i] = self.frame_in[i];
            }
            self.chain
                .process_frame_inplace(&mut self.frame_main[..channels]);

            // Process pending chain into frame_pending and mix.
            if self.pending_chain_active && self.crossfade_total > 0 {
                for i in 0..channels {
                    self.frame_pending[i] = self.frame_in[i];
                }
                self.pending_chain
                    .process_frame_inplace(&mut self.frame_pending[..channels]);
                let t = 1.0 - (self.crossfade_remaining as f32 / self.crossfade_total as f32);
                let inv_t = 1.0 - t;
                for i in 0..channels {
                    self.frame_main[i] = self.frame_main[i] * inv_t + self.frame_pending[i] * t;
                }
                self.crossfade_remaining = self.crossfade_remaining.saturating_sub(1);
                if self.crossfade_remaining == 0 {
                    std::mem::swap(&mut self.chain, &mut self.pending_chain);
                    self.pending_chain_active = false;
                    self.crossfade_total = 0;
                }
            }

            // L10 — apply the swap fade to the post-chain frame before metering,
            // so the meters track what is actually heard during the fade.
            if fade_gain != 1.0 {
                for v in &mut self.frame_main[..channels] {
                    *v *= fade_gain;
                }
            }

            if self.prime_samples_left == 0 {
                if let Some(meter) = &mut self.meter {
                    meter.process(&self.frame_main[..channels]);
                }
            }

            self.frame_out_pos = 0;
        }

        let out = self.frame_main[self.frame_out_pos];
        self.frame_out_pos += 1;
        Some(out)
    }
}

impl Iterator for MasteringSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        while self.prime_samples_left > 0 {
            self.next_unaligned()?;
            self.prime_samples_left -= 1;
        }
        self.next_unaligned()
    }
}

impl rodio::Source for MasteringSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels.max(1)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let total_frames = self.samples.len() / self.channels.max(1) as usize;
        if self.sample_rate == 0 {
            None
        } else {
            Some(Duration::from_secs_f64(
                total_frames as f64 / self.sample_rate as f64,
            ))
        }
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        let channels = self.channels.max(1) as usize;
        let target_frame = (pos.as_secs_f64() * self.sample_rate as f64) as usize;
        let target_sample = target_frame.saturating_mul(channels);
        self.position = target_sample.min(self.samples.len());
        // A seek landing inside the 512-frame settings crossfade must not
        // revert to the pre-update coefficients: the update was already
        // consumed from coeffs_rx (generation bumped), so deactivating the
        // pending chain here would discard the NEWEST settings with no
        // re-send — the loop wrap's auto-seek made this reachable every
        // region iteration (2026-07-06 audit). Promote first, like the
        // update-arrival path above.
        if self.pending_chain_active {
            std::mem::swap(&mut self.chain, &mut self.pending_chain);
            self.pending_chain_active = false;
        }
        // Drop accumulated biquad/limiter state to avoid clicks across
        // discontinuities. Also force a frame re-fetch on the next yield.
        self.chain.reset_states();
        self.reset_render_alignment();
        self.crossfade_remaining = 0;
        self.crossfade_total = 0;
        // Restart both meters exactly like MeteredPcmSource::try_seek —
        // Original and Mastered must measure the same program span or the
        // O/M integrated-LUFS comparison drifts after every loop wrap.
        if let Some(meter) = &mut self.meter {
            meter.reset();
        }
        self.frame_out_pos = channels;
        Ok(())
    }
}

#[cfg(test)]
mod render_alignment_tests {
    use super::*;
    use crate::{dsp::MasteringChain, types::MasteringSettings};
    use rodio::Source;

    fn settings() -> MasteringSettings {
        serde_json::from_value(serde_json::json!({
            "preset":{"kind":"universal"},"intensity":0.75,"volume_match":false,
            "eq_low_db":0.,"eq_mid_db":0.,"eq_high_db":0.,
            "delivery_profile":"custom","advanced":{"warmth":0.6}
        }))
        .unwrap()
    }

    fn source(samples: Vec<f32>, channels: u16, rate: u32) -> MasteringSource {
        let (_, rx) = mpsc::channel();
        MasteringSource::new(
            samples,
            channels,
            rate,
            MasteringChain::new(rate, usize::from(channels), &settings()),
            rx,
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(SpectrumRing::new()),
        )
        .with_render_alignment()
    }

    #[test]
    fn complete_streaming_dsp_and_device_meter_seek_allocate_nothing() {
        use crate::{dsp, quality_source, sources, spectrum, types};
        use rodio::Source;
        use std::{
            sync::{
                atomic::{AtomicI32, AtomicU32, Ordering},
                mpsc, Arc,
            },
            time::Duration,
        };
        let rate = 44100;
        let settings: types::MasteringSettings = serde_json::from_value(serde_json::json!({
            "preset":{"kind":"universal"},"intensity":0.75,"volume_match":false,
            "eq_low_db":0.,"eq_mid_db":0.,"eq_high_db":0.,"delivery_profile":"custom","advanced":{}
        }))
        .unwrap();
        let input: Vec<f32> = (0..rate * 6)
            .map(|n| 0.3 * (n as f32 * 0.23).sin())
            .collect();
        let (_, receive) = mpsc::channel();
        let peak = Arc::new(AtomicU32::new(0));
        let mut chain = sources::MasteringSource::new(
            input,
            2,
            rate,
            dsp::MasteringChain::new(rate, 2, &settings),
            receive,
            peak.clone(),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(spectrum::SpectrumRing::new()),
        )
        .with_render_alignment();
        let slots = chain.take_meter_slots();
        let converted = quality_source::QualitySource::new(chain, 48000).unwrap();
        let error = converted.error_slot();
        let mut source =
            sources::MeteredSource::new(converted, slots, sources::FadeEnvelope::inactive());
        let (allocations, (first, seek)) = crate::test_allocations::count(|| {
            let first = source.next();
            let seek = source.try_seek(Duration::from_millis(250));
            for sample in source {
                std::hint::black_box(sample);
            }
            (first, seek)
        });
        assert!(first.is_some() && seek.is_ok());
        assert_eq!(error.load(Ordering::Acquire), 0);
        assert!(peak.load(Ordering::Relaxed) > 0);
        assert_eq!(
            allocations, 0,
            "DSP/SRC/output meter allocated during initial fill, seek or drain"
        );
    }

    #[test]
    fn converted_output_meter_matches_emitted_pcm_and_single_channel_loudness() {
        for channels in [1, 2] {
            for frequency in [997., 38400.] {
                let input: Vec<f32> = (0..192000)
                    .flat_map(|n| {
                        let x = 0.6 * (n as f32 * frequency * std::f32::consts::TAU / 96000.).sin();
                        if channels == 1 {
                            vec![x]
                        } else {
                            vec![x, x * 0.5]
                        }
                    })
                    .collect();
                let slots = MeterSlots {
                    peak: Arc::new(AtomicU32::new(0)),
                    left: Arc::new(AtomicU32::new(0)),
                    right: Arc::new(AtomicU32::new(0)),
                    momentary: Arc::new(AtomicI32::new(i32::MIN)),
                    integrated: Arc::new(AtomicI32::new(i32::MIN)),
                    spectrum: Arc::new(SpectrumRing::new()),
                };
                let expected =
                    crate::sample_rate::convert_interleaved(&input, 96000, 48000, channels)
                        .unwrap();
                let mut original = MeteredPcmSource::new(
                    input,
                    channels,
                    96000,
                    slots.peak.clone(),
                    slots.left.clone(),
                    slots.right.clone(),
                    slots.momentary.clone(),
                    slots.integrated.clone(),
                    slots.spectrum.clone(),
                );
                let moved_slots = original.take_meter_slots();
                assert!(original.meter.is_none());
                let converted = crate::quality_source::QualitySource::new(original, 48000).unwrap();
                let mut metered =
                    MeteredSource::new(converted, moved_slots, FadeEnvelope::inactive());
                let actual: Vec<f32> = metered.by_ref().collect();
                assert_eq!(actual, expected);
                let peak = expected.iter().map(|x| x.abs()).fold(0_f32, f32::max);
                assert_eq!(f32::from_bits(slots.peak.load(Ordering::Relaxed)), peak);
                let right = expected
                    .chunks_exact(usize::from(channels))
                    .map(|frame| frame[usize::from(channels) - 1].abs())
                    .fold(0_f32, f32::max);
                assert_eq!(f32::from_bits(slots.right.load(Ordering::Relaxed)), right);
                let mut ebu =
                    ebur128::EbuR128::new(u32::from(channels), 48000, ebur128::Mode::I).unwrap();
                ebu.add_frames_f32(&expected).unwrap();
                let measured = slots.integrated.load(Ordering::Relaxed) as f64 / 100.;
                assert!((measured - ebu.loudness_global().unwrap()).abs() < 0.15);
                metered.try_seek(Duration::from_millis(250)).unwrap();
                assert_eq!(metered.meter.integrated.lufs(), -120.);
            }
        }
    }

    #[test]
    fn finite_source_including_short_tail_matches_file_renderer_exactly() {
        for rate in [32000, 44100, 48000, 96000] {
            for channels in [1, 2] {
                for frames in [0, 1, 17, 127, 128, 129, 257, 2049] {
                    let input: Vec<f32> = (0..frames * usize::from(channels))
                        .map(|i| 0.7 * ((i * 31 % 113) as f32 / 56. - 1.))
                        .collect();
                    let mut expected = input.clone();
                    let mut chain = MasteringChain::new(rate, usize::from(channels), &settings());
                    chain.process_interleaved(&mut expected, usize::from(channels));
                    chain.flush_render_tail(&mut expected, usize::from(channels));
                    let actual: Vec<f32> = source(input, channels, rate).collect();
                    assert_eq!(actual, expected, "{rate}, {channels}ch, {frames}frames");
                }
            }
        }
    }

    #[test]
    fn seek_restarts_alignment_and_priming_does_not_consume_swap_fade() {
        let input = vec![0.2; 48000];
        let mut actual = source(input.clone(), 2, 48000);
        for _ in 0..997 {
            actual.next().unwrap();
        }
        actual.try_seek(Duration::from_millis(250)).unwrap();
        let expected: Vec<f32> = source(input[24000..].to_vec(), 2, 48000).collect();
        assert_eq!(actual.collect::<Vec<_>>(), expected);

        let reference: Vec<f32> = source(input.clone(), 2, 48000).take(5).collect();
        let mut faded = source(input, 2, 48000).with_swap_fade(FadeEnvelope::new(
            2,
            4,
            4,
            Arc::new(AtomicBool::new(false)),
        ));
        assert_eq!(faded.by_ref().take(4).collect::<Vec<_>>(), vec![0.; 4]);
        assert_eq!(faded.fade.emitted_frames, 2);
        assert_eq!(
            faded
                .meter
                .as_ref()
                .unwrap()
                .slots
                .peak
                .load(Ordering::Relaxed),
            0
        );
        assert_eq!(faded.next().unwrap(), reference[4] * 0.25);
        assert_eq!(faded.fade.emitted_frames, 3);
    }
}

#[cfg(test)]
mod meter_input_tests {
    use super::lufs_meter_input;
    use crate::dsp::MomentaryLufs;

    #[test]
    fn mono_feeds_single_channel_not_a_duplicated_pair() {
        // Regression for master review §2: duplicating mono into both
        // channels double-counts energy and inflates loudness by +3.01 LU.
        assert_eq!(lufs_meter_input(&[0.5], 1), (0.5, 0.0));
        assert_eq!(lufs_meter_input(&[0.5, -0.25], 2), (0.5, -0.25));
        assert_eq!(lufs_meter_input(&[], 1), (0.0, 0.0));
    }

    #[test]
    fn mono_meter_input_reads_about_3lu_below_duplicated_stereo() {
        let sr = 48_000u32;
        let measure = |duplicate: bool| {
            let mut meter = MomentaryLufs::new(sr);
            let mut last = -120.0f32;
            // 1 kHz sine, > 400 ms so the momentary window fills.
            for i in 0..(sr as usize / 2) {
                let x = 0.5 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sr as f32).sin();
                let (l, r) = if duplicate { (x, x) } else { (x, 0.0) };
                last = meter.process_frame(l, r);
            }
            last
        };
        let delta = measure(true) - measure(false);
        assert!(
            (delta - 3.0103).abs() < 0.1,
            "duplicated-mono should read ~3.01 LU hotter than single-channel; got {delta}"
        );
    }
}

#[cfg(test)]
mod live_update_allocation_tests {
    use super::*;
    use crate::dsp::{ChainCoeffs, MasteringChain};
    use crate::types::{AdvancedSettings, DeliveryProfile, MasteringSettings, Preset};
    use std::sync::atomic::{AtomicI32, AtomicU32};
    use std::sync::{mpsc, Arc};

    fn settings_with_intensity(intensity: f32) -> MasteringSettings {
        let advanced = AdvancedSettings {
            compression_density: Some(0.0),
            ..Default::default()
        };
        MasteringSettings {
            preset: Preset::Universal,
            intensity,
            eq_sub_db: 0.0,
            eq_low_db: 0.0,
            eq_low_mid_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_mid_db: 0.0,
            eq_high_db: 0.0,
            eq_sparkle_db: 0.0,
            eq_bands: crate::types::EqBandFrequencies::default(),
            volume_match: false,
            source_lufs_integrated: None,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            delivery_profile: DeliveryProfile::Custom,
            album: None,
            advanced,
        }
    }

    fn source_with_updates(
        frames: usize,
    ) -> (MasteringSource, mpsc::Sender<LiveCoeffUpdate>, u32, u16) {
        let sample_rate = 44_100;
        let channels = 2u16;
        let samples = vec![0.05; frames * channels as usize];
        let settings = settings_with_intensity(0.0);
        let chain = MasteringChain::new(sample_rate, channels as usize, &settings);
        let (tx, rx) = mpsc::channel::<LiveCoeffUpdate>();
        let source = MasteringSource::new(
            samples,
            channels,
            sample_rate,
            chain,
            rx,
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(SpectrumRing::new()),
        );
        (source, tx, sample_rate, channels)
    }

    fn drain_frames(source: &mut MasteringSource, frames: usize) {
        let samples = frames * source.channels.max(1) as usize;
        for _ in 0..samples {
            assert!(source.next().is_some(), "source ended before update check");
        }
    }

    #[test]
    fn mastering_source_live_updates_reuse_preallocated_chains() {
        let (mut source, tx, sample_rate, _channels) =
            source_with_updates(COEFFS_CHECK_INTERVAL_FRAMES * 3);
        let initial_main_allocs = source.chain.allocation_fingerprint();
        let initial_pending_allocs = source.pending_chain.allocation_fingerprint();

        let first_settings = settings_with_intensity(0.5);
        tx.send(LiveCoeffUpdate {
            generation: 1,
            coeffs: ChainCoeffs::from_settings(sample_rate, &first_settings),
        })
        .expect("send first update");

        drain_frames(&mut source, COEFFS_CHECK_INTERVAL_FRAMES);

        assert!(source.pending_chain_active);
        assert_eq!(
            source.pending_chain.allocation_fingerprint(),
            initial_pending_allocs,
            "first live update must overwrite the preallocated pending chain"
        );
        assert!(source.crossfade_remaining > 0);

        let second_settings = settings_with_intensity(1.0);
        tx.send(LiveCoeffUpdate {
            generation: 2,
            coeffs: ChainCoeffs::from_settings(sample_rate, &second_settings),
        })
        .expect("send second update");

        drain_frames(&mut source, COEFFS_CHECK_INTERVAL_FRAMES);

        assert!(source.pending_chain_active);
        assert_eq!(
            source.chain.allocation_fingerprint(),
            initial_pending_allocs,
            "active pending chain should be promoted by swapping, not rebuilt"
        );
        assert_eq!(
            source.pending_chain.allocation_fingerprint(),
            initial_main_allocs,
            "follow-up live update should reuse the alternate preallocated chain"
        );
    }

    /// 2026-07-06 audit: a seek landing inside the settings crossfade must
    /// keep the NEWEST coefficients. The update was already consumed from
    /// coeffs_rx (generation bumped), so discarding the pending chain
    /// reverted audition to the pre-update settings until the next knob
    /// touch — and the loop wrap's auto-seek opened that window on every
    /// region iteration.
    #[test]
    fn seek_mid_crossfade_promotes_the_pending_chain() {
        use rodio::Source as _;
        let (mut source, tx, sample_rate, _channels) =
            source_with_updates(COEFFS_CHECK_INTERVAL_FRAMES * 3);
        let initial_pending_allocs = source.pending_chain.allocation_fingerprint();

        tx.send(LiveCoeffUpdate {
            generation: 1,
            coeffs: ChainCoeffs::from_settings(sample_rate, &settings_with_intensity(0.9)),
        })
        .expect("send update");
        drain_frames(&mut source, COEFFS_CHECK_INTERVAL_FRAMES);
        assert!(source.pending_chain_active, "update must arm the crossfade");

        source.try_seek(Duration::from_millis(0)).expect("seek");

        assert!(!source.pending_chain_active);
        assert_eq!(source.crossfade_remaining, 0);
        assert_eq!(
            source.chain.allocation_fingerprint(),
            initial_pending_allocs,
            "seek during the fade must promote the newest coefficients, not discard them"
        );
    }

    /// O/M A-B honesty (2026-07-06 audit): both source types restart their
    /// meters on seek so Original and Mastered integrated LUFS measure the
    /// same program span — MeteredPcmSource already did; MasteringSource
    /// kept accumulating across seeks/loop wraps.
    #[test]
    fn seek_restarts_the_mastered_meters_like_original() {
        use rodio::Source as _;
        let (mut source, _tx, sample_rate, _channels) =
            source_with_updates(COEFFS_CHECK_INTERVAL_FRAMES * 3);
        let fresh = crate::dsp::IntegratedLufs::new(sample_rate).lufs();
        // Accumulate integrated state directly on the source's meter (the
        // harness samples are DC, which K-weighting gates to silence): one
        // second of 1 kHz sine fills the 400 ms gating blocks.
        for i in 0..sample_rate as usize {
            let x =
                0.5 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sample_rate as f32).sin();
            source
                .meter
                .as_mut()
                .unwrap()
                .integrated
                .process_frame(x, x);
        }
        let accumulated = source.meter.as_mut().unwrap().integrated.lufs();
        assert_ne!(
            accumulated, fresh,
            "precondition: the meter must have accumulated integrated state"
        );

        source.try_seek(Duration::from_millis(0)).expect("seek");

        assert_eq!(
            source.meter.as_mut().unwrap().integrated.lufs(),
            fresh,
            "seek must restart the integrated meter exactly like MeteredPcmSource"
        );
        // (The momentary meter is replaced in the same statement; it exposes
        // no state accessor to pin separately.)
    }
}

#[cfg(test)]
mod swap_fade_tests {
    //! L10 — the Original<->Mastered swap fade. The fade lives inside the source
    //! (decoupled from `try_seek`) so the playhead is preserved while the click
    //! is suppressed; the audio command thread never blocks.
    use super::*;
    use rodio::Source as _;
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32};

    fn metered(
        samples: Vec<f32>,
        channels: u16,
        sample_rate: u32,
        fade: FadeEnvelope,
    ) -> MeteredPcmSource {
        MeteredPcmSource::new(
            samples,
            channels,
            sample_rate,
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(AtomicI32::new(i32::MIN)),
            Arc::new(SpectrumRing::new()),
        )
        .with_swap_fade(fade)
    }

    #[test]
    fn fade_in_stays_silent_through_lead_in_then_ramps_to_unity() {
        let trigger = Arc::new(AtomicBool::new(false));
        let mut env = FadeEnvelope::new(2, 4, 4, trigger);
        // Lead-in: silent so the outgoing source's fade-out never overlaps.
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.0));
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.0));
        // Fade-in ramps 0 -> 1 across four frames.
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.25));
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.5));
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.75));
        assert_eq!(env.advance_frame(), FrameFade::Gain(1.0));
        // Steady state thereafter.
        assert_eq!(env.advance_frame(), FrameFade::Gain(1.0));
    }

    #[test]
    fn fade_out_ramps_from_full_then_ends() {
        let trigger = Arc::new(AtomicBool::new(false));
        // No lead-in / fade-in: models a fresh-play source that is later toggled.
        let mut env = FadeEnvelope::new(0, 0, 4, trigger.clone());
        assert_eq!(env.advance_frame(), FrameFade::Gain(1.0));
        // A toggle requests this (outgoing) source fade out and end.
        trigger.store(true, Ordering::Relaxed);
        assert_eq!(env.advance_frame(), FrameFade::Gain(1.0)); // 1.0 * 4/4
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.75));
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.5));
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.25));
        assert_eq!(env.advance_frame(), FrameFade::End);
        // Once ended it stays ended (the source returns None forever).
        assert_eq!(env.advance_frame(), FrameFade::End);
    }

    #[test]
    fn re_toggle_mid_fade_in_fades_out_from_partial_gain_not_full() {
        // Guards rapid re-toggling: the fade-out must start from whatever gain
        // the source last emitted, never jump back to unity (which would click).
        let trigger = Arc::new(AtomicBool::new(false));
        let mut env = FadeEnvelope::new(0, 4, 4, trigger.clone());
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.25)); // partway through fade-in
        trigger.store(true, Ordering::Relaxed);
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.25)); // 0.25 * 4/4
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.1875)); // 0.25 * 3/4
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.125)); // 0.25 * 2/4
        assert_eq!(env.advance_frame(), FrameFade::Gain(0.0625)); // 0.25 * 1/4
        assert_eq!(env.advance_frame(), FrameFade::End);
    }

    #[test]
    fn inactive_envelope_passes_signal_through_unchanged() {
        // Fresh play / tests: gain is always unity and the source never ends early.
        let mut env = FadeEnvelope::inactive();
        for _ in 0..1000 {
            assert_eq!(env.advance_frame(), FrameFade::Gain(1.0));
        }
    }

    #[test]
    fn swap_fade_in_suppresses_startup_click_and_reaches_unity() {
        // A constant full-scale signal: with the fade installed the first audible
        // sample is near silence (no click) and the ramp climbs to full scale.
        let sr = 48_000;
        let frames = 8usize;
        let trigger = Arc::new(AtomicBool::new(false));
        let env = FadeEnvelope::new(0, frames, frames, trigger);
        let mut src = metered(vec![1.0f32; 64], 1, sr, env);

        let first = src.next().expect("sample");
        assert!(first < 0.5, "fade-in must start near silence, got {first}");
        let mut last = first;
        for _ in 1..frames {
            let v = src.next().expect("sample");
            assert!(
                v >= last - 1e-6,
                "fade-in must be monotonic non-decreasing, {v} < {last}"
            );
            last = v;
        }
        assert!(
            (last - 1.0).abs() < 1e-6,
            "fade-in must reach unity, got {last}"
        );
    }

    #[test]
    fn swap_preserves_playhead_under_lead_in_then_fade_in() {
        // Ramp signal (sample value == frame index) so the seeked position is
        // observable in the output. Seek to frame 10, lead-in 3, fade-in 3.
        let sr = 1_000;
        let samples: Vec<f32> = (0..200).map(|i| i as f32).collect();
        let trigger = Arc::new(AtomicBool::new(false));
        let env = FadeEnvelope::new(3, 3, 3, trigger);
        let mut src = metered(samples, 1, sr, env);

        // Playhead preservation: seek to 10 ms == frame 10 at 1 kHz mono.
        src.try_seek(Duration::from_secs_f64(0.010)).expect("seek");

        // Lead-in: three silent frames (consuming source frames 10, 11, 12).
        assert_eq!(src.next(), Some(0.0));
        assert_eq!(src.next(), Some(0.0));
        assert_eq!(src.next(), Some(0.0));
        // First audible frame is source frame 13 (== seek 10 + lead-in 3),
        // scaled by the first fade-in step (1/3). Had the seek been dropped it
        // would be frame 3 -> 1.0; landing on ~4.333 proves the playhead held.
        let first_audible = src.next().expect("sample");
        assert!(
            (first_audible - 13.0 / 3.0).abs() < 1e-3,
            "expected source frame 13 * 1/3 ≈ 4.333 (playhead preserved), got {first_audible}"
        );
    }

    #[test]
    fn swap_keeps_playing_with_fade_installed_on_idle_sink() {
        // Device-free sink (no OutputStream): proves isPlaying stays true and the
        // playhead advances while the swap fade is active.
        let sr = 48_000;
        let channels = 2u16;
        let frames = swap_fade_frames(sr);
        let samples = vec![0.5f32; sr as usize * channels as usize * 2];
        let trigger = Arc::new(AtomicBool::new(false));
        let env = FadeEnvelope::new(frames, frames, frames, trigger);
        let src = metered(samples, channels, sr, env);

        let (sink, mut queue) = rodio::Sink::new_idle();
        sink.append(src);
        sink.play();
        // Pull ~0.2 s so the sink's periodic position tracking advances.
        let pulls = sr as usize * channels as usize / 5;
        let mut produced = 0usize;
        for _ in 0..pulls {
            if queue.next().is_some() {
                produced += 1;
            }
        }
        assert!(produced > 0, "idle sink must produce samples");
        assert!(
            !sink.is_paused() && !sink.empty(),
            "isPlaying must stay true across the swap"
        );
        assert!(
            sink.get_pos() > Duration::ZERO,
            "playhead must advance during playback, got {:?}",
            sink.get_pos()
        );
    }
}
