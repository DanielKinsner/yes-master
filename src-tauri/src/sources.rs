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

use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use crate::spectrum::SpectrumRing;

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

/// Pass-through source for Original playback that still feeds the same peak,
/// LUFS, and spectrum meter path as Mastered playback. This keeps A/B metering
/// honest without routing Original through any mastering DSP.
pub(crate) struct MeteredPcmSource {
    samples: Vec<f32>,
    position: usize,
    channels: u16,
    sample_rate: u32,
    frame: Vec<f32>,
    frame_out_pos: usize,
    peak_linear: Arc<AtomicU32>,
    peak_left_linear: Arc<AtomicU32>,
    peak_right_linear: Arc<AtomicU32>,
    lufs_meter: crate::dsp::MomentaryLufs,
    lufs_x100: Arc<AtomicI32>,
    integrated_lufs_meter: crate::dsp::IntegratedLufs,
    integrated_lufs_x100: Arc<AtomicI32>,
    spectrum_ring: Arc<SpectrumRing>,
}

impl MeteredPcmSource {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        samples: Vec<f32>,
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
            samples,
            position: 0,
            channels,
            sample_rate,
            frame: vec![0.0; channels_usize],
            frame_out_pos: channels_usize,
            peak_linear,
            peak_left_linear,
            peak_right_linear,
            lufs_meter: crate::dsp::MomentaryLufs::new(sample_rate),
            lufs_x100,
            integrated_lufs_meter: crate::dsp::IntegratedLufs::new(sample_rate),
            integrated_lufs_x100,
            spectrum_ring,
        }
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

            for i in 0..channels {
                self.frame[i] = if self.position + i < self.samples.len() {
                    self.samples[self.position + i]
                } else {
                    0.0
                };
            }
            self.position += channels;

            let mut frame_peak = 0.0f32;
            for v in &self.frame[..channels] {
                let abs = v.abs();
                if abs.is_finite() && abs > frame_peak {
                    frame_peak = abs;
                }
            }
            self.peak_linear
                .fetch_max(frame_peak.to_bits(), Ordering::Relaxed);

            let l = self.frame.first().copied().unwrap_or(0.0);
            let r = if channels >= 2 { self.frame[1] } else { l };
            fold_channel_peak(&self.peak_left_linear, l);
            fold_channel_peak(&self.peak_right_linear, r);
            let to_x100 = |lufs: f32| -> i32 {
                if lufs.is_finite() && lufs > -120.0 {
                    (lufs * 100.0) as i32
                } else {
                    i32::MIN
                }
            };
            // Mono must feed the BS.1770 meters as a single channel, not a
            // duplicated pair, or loudness reads +3.01 LU hot (master review §2).
            let (meter_l, meter_r) = lufs_meter_input(&self.frame, channels);
            let momentary = self.lufs_meter.process_frame(meter_l, meter_r);
            self.lufs_x100.store(to_x100(momentary), Ordering::Relaxed);
            let integrated = self.integrated_lufs_meter.process_frame(meter_l, meter_r);
            self.integrated_lufs_x100
                .store(to_x100(integrated), Ordering::Relaxed);

            let mono = (l + r) * 0.5;
            if mono.is_finite() {
                self.spectrum_ring.push(mono);
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
        self.lufs_meter = crate::dsp::MomentaryLufs::new(self.sample_rate);
        self.integrated_lufs_meter = crate::dsp::IntegratedLufs::new(self.sample_rate);
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
    samples: Vec<f32>,
    position: usize,
    channels: u16,
    sample_rate: u32,
    chain: crate::dsp::MasteringChain,
    pending_chain: Option<crate::dsp::MasteringChain>,
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
    /// Shared post-output-gain peak slot. Per-frame max of |frame_main[i]| is
    /// atomic-max'd into this slot. The audio thread consumes it via swap.
    peak_linear: Arc<AtomicU32>,
    /// Per-channel post-output-gain peak slots for the stereo MASTER OUT meter.
    peak_left_linear: Arc<AtomicU32>,
    peak_right_linear: Arc<AtomicU32>,
    /// Live BS.1770 momentary LUFS meter (K-weighted, 400 ms window).
    lufs_meter: crate::dsp::MomentaryLufs,
    /// Shared atomic slot for the audio thread to read the latest LUFS value.
    /// Stored as LUFS×100 in an i32. `i32::MIN` = silent / pre-prime.
    lufs_x100: Arc<AtomicI32>,
    /// BS.1770-4 integrated LUFS meter — aggregates the whole listen-through
    /// with absolute (-70 LUFS) and relative (-10 LU from ungated mean) gates.
    integrated_lufs_meter: crate::dsp::IntegratedLufs,
    /// Shared atomic slot for the integrated readout. Same storage convention
    /// as `lufs_x100`.
    integrated_lufs_x100: Arc<AtomicI32>,
    /// L4b — lock-free ring of post-chain mono mix samples. The audio
    /// thread pushes one sample per output frame; the snapshot tick
    /// reads it and runs an FFT to produce the EQ panel's live
    /// spectrum.
    spectrum_ring: Arc<SpectrumRing>,
}

impl MasteringSource {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        samples: Vec<f32>,
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
        Self {
            samples,
            position: 0,
            channels,
            sample_rate,
            chain,
            pending_chain: None,
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
            peak_linear,
            peak_left_linear,
            peak_right_linear,
            lufs_meter: crate::dsp::MomentaryLufs::new(sample_rate),
            lufs_x100,
            integrated_lufs_meter: crate::dsp::IntegratedLufs::new(sample_rate),
            integrated_lufs_x100,
            spectrum_ring,
        }
    }
}

impl Iterator for MasteringSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let channels = self.channels.max(1) as usize;
        if self.frame_out_pos >= channels {
            // Time to fetch + process the next input frame.
            if self.position >= self.samples.len() {
                return None;
            }

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
                    if let Some(prev_pending) = self.pending_chain.take() {
                        self.chain = prev_pending;
                    }
                    self.pending_chain =
                        Some(crate::dsp::MasteringChain::with_coeffs_inheriting_state(
                            update.coeffs,
                            &self.chain,
                        ));
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
            if self.pending_chain.is_some() && self.crossfade_total > 0 {
                for i in 0..channels {
                    self.frame_pending[i] = self.frame_in[i];
                }
                let pending = self
                    .pending_chain
                    .as_mut()
                    .expect("pending_chain just checked");
                pending.process_frame_inplace(&mut self.frame_pending[..channels]);
                let t = 1.0 - (self.crossfade_remaining as f32 / self.crossfade_total as f32);
                let inv_t = 1.0 - t;
                for i in 0..channels {
                    self.frame_main[i] = self.frame_main[i] * inv_t + self.frame_pending[i] * t;
                }
                self.crossfade_remaining = self.crossfade_remaining.saturating_sub(1);
                if self.crossfade_remaining == 0 {
                    self.chain = self
                        .pending_chain
                        .take()
                        .expect("pending_chain just checked");
                    self.crossfade_total = 0;
                }
            }

            // Phase 12.2 — fold the post-output-gain frame peak into the shared
            // atomic for the live clipping meter. Per-frame instead of
            // per-sample: cheaper, and the meter only needs ~50 ms resolution
            // (the snapshot loop's tick rate). NaN/inf are filtered so a DSP
            // bug can't poison the atomic with a non-finite value.
            let mut frame_peak = 0.0f32;
            for i in 0..channels {
                let v = self.frame_main[i].abs();
                if v.is_finite() && v > frame_peak {
                    frame_peak = v;
                }
            }
            // Bits comparison is safe here because we only ever store
            // non-negative finite f32, where IEEE 754 ordering matches numeric.
            self.peak_linear
                .fetch_max(frame_peak.to_bits(), Ordering::Relaxed);

            // Live BS.1770 LUFS meters — feed the post-output stereo frame
            // into both the momentary (400 ms K-weighted window) and the
            // integrated (whole-listen-through with BS.1770-4 gating) meters.
            // Mono input gets duplicated so the meters see a stereo pair
            // (matches BS.1770's stereo channel summation).
            let l = self.frame_main.first().copied().unwrap_or(0.0);
            let r = if channels >= 2 { self.frame_main[1] } else { l };
            fold_channel_peak(&self.peak_left_linear, l);
            fold_channel_peak(&self.peak_right_linear, r);
            let to_x100 = |lufs: f32| -> i32 {
                if lufs.is_finite() && lufs > -120.0 {
                    (lufs * 100.0) as i32
                } else {
                    i32::MIN
                }
            };
            // Mono must feed the BS.1770 meters as a single channel, not a
            // duplicated pair, or loudness reads +3.01 LU hot (master review §2).
            let (meter_l, meter_r) = lufs_meter_input(&self.frame_main, channels);
            let momentary = self.lufs_meter.process_frame(meter_l, meter_r);
            self.lufs_x100.store(to_x100(momentary), Ordering::Relaxed);
            let integrated = self.integrated_lufs_meter.process_frame(meter_l, meter_r);
            self.integrated_lufs_x100
                .store(to_x100(integrated), Ordering::Relaxed);

            // L4b — push post-chain mono mix into the spectrum ring.
            // Lock-free atomic store; the snapshot tick FFTs the latest
            // 2048 samples to drive the EQ panel's live bars.
            let mono = (l + r) * 0.5;
            if mono.is_finite() {
                self.spectrum_ring.push(mono);
            }

            self.frame_out_pos = 0;
        }

        let out = self.frame_main[self.frame_out_pos];
        self.frame_out_pos += 1;
        Some(out)
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
        // Drop accumulated biquad/limiter state to avoid clicks across
        // discontinuities. Also force a frame re-fetch on the next yield.
        self.chain.reset_states();
        self.pending_chain = None;
        self.crossfade_remaining = 0;
        self.crossfade_total = 0;
        self.frame_out_pos = channels;
        Ok(())
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
