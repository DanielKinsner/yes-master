use crate::types::*;
use std::f32::consts::PI;

// ============================================================================
// Biquad — direct form II transposed
// Coefficients computed per RBJ Audio EQ Cookbook
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCoeffs {
    pub fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    pub fn low_shelf(sample_rate: f32, freq_hz: f32, gain_db: f32, slope: f32) -> Self {
        if gain_db.abs() < 1.0e-4 {
            return Self::identity();
        }
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * freq_hz / sample_rate;
        let cos_o = omega.cos();
        let sin_o = omega.sin();
        let alpha = sin_o / 2.0 * ((a + 1.0 / a) * (1.0 / slope - 1.0) + 2.0).sqrt();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_o + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_o);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_o - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_o + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_o);
        let a2 = (a + 1.0) + (a - 1.0) * cos_o - two_sqrt_a_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub fn high_shelf(sample_rate: f32, freq_hz: f32, gain_db: f32, slope: f32) -> Self {
        if gain_db.abs() < 1.0e-4 {
            return Self::identity();
        }
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * freq_hz / sample_rate;
        let cos_o = omega.cos();
        let sin_o = omega.sin();
        let alpha = sin_o / 2.0 * ((a + 1.0 / a) * (1.0 / slope - 1.0) + 2.0).sqrt();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_o + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_o);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_o - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_o + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_o);
        let a2 = (a + 1.0) - (a - 1.0) * cos_o - two_sqrt_a_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub fn peaking(sample_rate: f32, freq_hz: f32, q: f32, gain_db: f32) -> Self {
        if gain_db.abs() < 1.0e-4 {
            return Self::identity();
        }
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * freq_hz / sample_rate;
        let cos_o = omega.cos();
        let alpha = omega.sin() / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_o;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_o;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadState {
    z1: f32,
    z2: f32,
}

impl BiquadState {
    pub fn process(&mut self, c: &BiquadCoeffs, x: f32) -> f32 {
        let y = c.b0 * x + self.z1;
        self.z1 = c.b1 * x - c.a1 * y + self.z2;
        self.z2 = c.b2 * x - c.a2 * y;
        y
    }
}

// ============================================================================
// MasteringChain — gain → 3-band EQ → optional saturation → soft-clip ceiling
// Phase 4.1 only: no real compressor, no lookahead limiter, no true-peak.
// Those land in Phase 11 (DSP audit).
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct ChainCoeffs {
    pub low: BiquadCoeffs,
    pub mid: BiquadCoeffs,
    pub high: BiquadCoeffs,
    pub input_gain_lin: f32,
    pub saturation_amount: f32,
    pub ceiling_lin: f32,
    /// Phase 12.1 — user-controllable output trim, applied AFTER the limiter
    /// and the volume-match scalar. 1.0 = no change. Boosting above 1.0 can
    /// reintroduce peaks above the ceiling (intentional — user choice; the
    /// export check will flag it).
    pub user_output_gain_lin: f32,
    /// Post-chain output gain used to align mastered playback loudness with
    /// the unprocessed source. 1.0 = no adjustment (Volume Match off).
    /// When on, set to the inverse of the input gain stage so the master
    /// comes back down to roughly the source's level for fair A/B. Approximate
    /// — doesn't account for EQ/saturation contributions, but close enough
    /// for tone judgment. Tooltip in the UI is honest about this.
    pub volume_match_gain_lin: f32,
    /// Phase 12.2 — stereo width via M/S processing. Scales the side
    /// component between EQ and saturation; 0 = mono (collapse side to zero),
    /// 1 = neutral (no-op), > 1 widens. Only consulted when the chain is
    /// running on a stereo frame. Clamped to [0.0, 2.0] in `from_settings`
    /// so an out-of-range user value can't flip phase or destabilize gain.
    pub width_side_scale: f32,
}

impl ChainCoeffs {
    pub fn from_settings(sample_rate: u32, settings: &MasteringSettings) -> Self {
        let sr = sample_rate as f32;
        let intensity = settings.intensity.clamp(0.0, 1.0);

        // Per-PRODUCT.md, Intensity is a macro that "should change how hard the
        // preset works across multiple parameters" — not a volume knob. So each
        // preset gets a baseline EQ curve, saturation amount, and gain push,
        // and Intensity scales the whole preset character. At intensity = 0.5
        // (the default), the preset is at full character; below 0.5 it
        // softens toward neutral, above 0.5 it pushes harder.
        //
        //   preset_scale(intensity) = 0.4 + 1.2 * intensity
        //     intensity 0.0  -> 0.40 (preset audible but subtle)
        //     intensity 0.5  -> 1.00 (full preset character — the default)
        //     intensity 1.0  -> 1.60 (preset pushed past full)
        //
        // The user's manual Low/Mid/High EQ adds ON TOP of the preset
        // baseline, so dialing in a custom tweak doesn't erase the preset's
        // signature sound. Numbers below were tuned conservatively for a
        // first pass — Phase 12.1 listening on real material will calibrate.
        let preset_scale = 0.4 + 1.2 * intensity;

        // (low_db, mid_db, high_db, gain_db, saturation_amount)
        // EQ values are the preset's signature curve before user EQ adds on.
        // Gain in dB is the preset's loudness push before Intensity scaling.
        // Saturation is a unitless drive parameter consumed by the tanh stage.
        let (preset_low_db, preset_mid_db, preset_high_db, preset_gain_db, preset_sat) =
            match settings.preset {
                // Universal: well-rounded, mostly transparent, gentle air on top.
                Preset::Universal => (0.0, 0.0, 0.5, 1.5, 0.0),
                // Clarity: cut low mud, lift presence + air for vocal/detail.
                Preset::Clarity => (-0.5, 1.0, 2.5, 1.5, 0.0),
                // Tape: low-mid body, softened top, audible saturation glue.
                Preset::Tape => (1.5, 0.0, -1.5, 1.0, 0.45),
                // Spatial: cut mids, lift highs — open, V-ish for width feel.
                Preset::Spatial => (0.0, -1.0, 1.5, 1.5, 0.0),
                // Oomph: heavy low boost for bass-forward material.
                Preset::Oomph => (2.5, -0.5, 0.0, 2.0, 0.15),
                // Warmth: fuller body, softer top, moderate saturation.
                Preset::Warmth => (1.5, 0.5, -2.0, 1.0, 0.30),
                // Punch: mid emphasis for transient impact + presence.
                Preset::Punch => (1.0, 2.0, 1.0, 2.0, 0.20),
                // Loud: broadband density + gain push for streaming targets.
                Preset::Loud => (0.5, 0.5, 0.5, 3.5, 0.10),
                // Custom: neutral baseline — user controls drive everything.
                Preset::Custom { .. } => (0.0, 0.0, 0.0, 1.5, 0.0),
            };

        // Effective EQ = scaled preset EQ + user EQ.
        let effective_low_db = preset_low_db * preset_scale + settings.eq_low_db;
        let effective_mid_db = preset_mid_db * preset_scale + settings.eq_mid_db;
        let effective_high_db = preset_high_db * preset_scale + settings.eq_high_db;

        let low = BiquadCoeffs::low_shelf(sr, 200.0, effective_low_db, 0.7);
        let mid = BiquadCoeffs::peaking(sr, 1500.0, 0.8, effective_mid_db);
        let high = BiquadCoeffs::high_shelf(sr, 6000.0, effective_high_db, 0.7);

        // Input gain = scaled preset gain push + user input gain. User input
        // gain is the standard mastering "back off the source" knob — useful
        // when an already-mastered track would otherwise clip after the
        // preset's baseline gain push lands on top of it.
        let user_input_gain_db = settings.input_gain_db.clamp(-24.0, 24.0);
        let input_gain_db = preset_gain_db * preset_scale + user_input_gain_db;
        let input_gain_lin = 10.0_f32.powf(input_gain_db / 20.0);

        let saturation_amount = preset_sat * preset_scale;

        let ceiling_db = settings
            .advanced
            .ceiling_dbtp
            .unwrap_or(-1.0)
            .clamp(-6.0, 0.0);
        let ceiling_lin = 10.0_f32.powf(ceiling_db / 20.0);

        // Post-limiter user-trim. Clamped to the same ±24 dB range as input
        // gain for symmetric extremes; default 0 dB.
        let user_output_gain_db = settings.output_gain_db.clamp(-24.0, 24.0);
        let user_output_gain_lin = 10.0_f32.powf(user_output_gain_db / 20.0);

        let volume_match_gain_lin = if settings.volume_match {
            // Undo the input-gain boost so mastered playback meets the source
            // at roughly equal loudness. Limiter has already shaped the peaks
            // to the ceiling; we just trim level here.
            if input_gain_lin > 0.0 {
                1.0 / input_gain_lin
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Width: None means "neutral" (1.0 = leave the stereo image alone).
        // Clamp to [0, 2] so a stray slider value can't invert phase or push
        // the side past 2× — typical mastering plugins cap M/S widening here.
        let width_side_scale = settings
            .advanced
            .width
            .unwrap_or(1.0)
            .clamp(0.0, 2.0);

        Self {
            low,
            mid,
            high,
            input_gain_lin,
            saturation_amount,
            ceiling_lin,
            user_output_gain_lin,
            volume_match_gain_lin,
            width_side_scale,
        }
    }
}

/// Apply an in-place M/S width transform to a stereo frame. `frame` must be
/// at least length 2; channels beyond index 1 are untouched. `side_scale`
/// scales the L-R component (0 collapses to mono, 1 is a no-op, > 1 widens).
///
/// Energy budgeting: this is the textbook lossless M/S decode/encode pair, so
/// `side_scale = 1` is exactly identity, and other scales redistribute energy
/// between mid and side without introducing gain on the post-transform signal
/// when summed across both channels. The limiter downstream catches any peak
/// excursions that widening introduces on individual channels.
#[inline]
pub(crate) fn apply_width_stereo(frame: &mut [f32], side_scale: f32) {
    if frame.len() < 2 {
        return;
    }
    let l = frame[0];
    let r = frame[1];
    let mid = 0.5 * (l + r);
    let side = 0.5 * (l - r) * side_scale;
    frame[0] = mid + side;
    frame[1] = mid - side;
}

#[derive(Debug, Clone, Default)]
pub struct ChannelState {
    low: BiquadState,
    mid: BiquadState,
    high: BiquadState,
}

// ============================================================================
// Limiter — linked-stereo brick-wall limiter with lookahead.
// Phase 11.2.a: sample-peak detection, instant attack, exponential release.
// Phase 11.2.b: 2× upsample inter-sample peak via Lagrange-4 midpoint (x=0.5).
// Phase 11.2.c: 4× upsample by also evaluating x=0.25 and x=0.75. The three
//   coefficient triplets below are the 4-point Lagrange basis polynomials
//   evaluated at fractional positions 0.25, 0.5, and 0.75 between samples
//   `b` and `c`, with neighbors `a` and `d` providing curvature. ITU-R
//   BS.1770 recommends ≥ 4× oversampling for true-peak; this estimator is a
//   close approximation that avoids the cost of a polyphase FIR.
// ============================================================================

/// 4-point Lagrange interpolator coefficients at three fractional positions
/// inside a (b, c) sample pair. For samples (a, b, c, d) at indices
/// (-1, 0, 1, 2), each row gives the basis weights at one of (x = 0.25, 0.5,
/// 0.75) so that `L(x) = w[0]·a + w[1]·b + w[2]·c + w[3]·d`.
///
/// Coefficients verified by hand-computing the 4-point Lagrange polynomial at
/// each fractional position. Each row sums to 1.0 (interpolation invariant).
const LAGRANGE_INTERSAMPLE_COEFFS: [[f32; 4]; 3] = [
    [-0.0546875, 0.8203125, 0.2734375, -0.0390625], // x = 0.25
    [-0.0625, 0.5625, 0.5625, -0.0625],             // x = 0.5
    [-0.0390625, 0.2734375, 0.8203125, -0.0546875], // x = 0.75 (mirror of 0.25)
];

#[derive(Debug, Clone)]
pub struct Limiter {
    channels: usize,
    ceiling_lin: f32,
    lookahead_frames: usize,
    release_coef: f32,
    /// Ring buffer of interleaved samples sized `lookahead_frames * channels`.
    buffer: Vec<f32>,
    /// Index of the next frame to overwrite (also the oldest frame in the buffer).
    head_frame: usize,
    /// How many frames have been written so far (capped at `lookahead_frames`).
    filled_frames: usize,
    /// Current linear gain reduction (1.0 = no reduction).
    gain: f32,
    /// Reusable scratch buffer for the oldest-frame read; preallocated to avoid
    /// per-frame allocations on the audio thread.
    oldest_frame_buf: Vec<f32>,
}

impl Limiter {
    pub fn new(
        sample_rate: u32,
        channels: usize,
        ceiling_dbfs: f32,
        lookahead_ms: f32,
        release_ms: f32,
    ) -> Self {
        let ch = channels.max(1);
        let lookahead_frames =
            (((lookahead_ms / 1000.0) * sample_rate as f32).round() as usize).max(1);
        let release_coef = if release_ms > 0.0 {
            (-1.0_f32 / (release_ms / 1000.0 * sample_rate as f32)).exp()
        } else {
            0.0
        };
        let ceiling_lin = 10.0_f32.powf(ceiling_dbfs / 20.0);
        Self {
            channels: ch,
            ceiling_lin,
            lookahead_frames,
            release_coef,
            buffer: vec![0.0; lookahead_frames * ch],
            head_frame: 0,
            filled_frames: 0,
            gain: 1.0,
            oldest_frame_buf: vec![0.0; ch],
        }
    }

    /// Process one frame in place. `frame.len()` must equal `self.channels`
    /// (smaller frames are zero-padded internally).
    pub fn process_frame_inplace(&mut self, frame: &mut [f32]) {
        let ch = self.channels;
        if ch == 0 {
            return;
        }
        let head_base = self.head_frame * ch;
        // Read the OLDEST frame from the ring before overwriting.
        for i in 0..ch {
            self.oldest_frame_buf[i] = self.buffer[head_base + i];
        }
        // Write the new frame into the ring.
        for i in 0..ch {
            let s = if i < frame.len() { frame[i] } else { 0.0 };
            self.buffer[head_base + i] = s;
        }
        self.head_frame = (self.head_frame + 1) % self.lookahead_frames;
        if self.filled_frames < self.lookahead_frames {
            self.filled_frames += 1;
        }

        // Scan the buffer for the peak. Two passes:
        //   1) Raw sample peaks (linked stereo, single max across channels).
        //   2) Lagrange-4 inter-sample peaks at x ∈ {0.25, 0.5, 0.75} between
        //      every adjacent frame pair — catches the true peak across the
        //      sub-sample positions that a 4× upsample would expose. Phase
        //      11.2.b checked x=0.5 only; sign-asymmetric patterns can place
        //      the true peak near x=0.25 or x=0.75 with a relatively small
        //      x=0.5 estimate, which is what this loop now covers.
        let mut peak: f32 = 0.0;
        for &s in &self.buffer {
            let a = s.abs();
            if a > peak {
                peak = a;
            }
        }
        let frames = self.filled_frames;
        if frames >= 4 {
            for f in 1..(frames - 2) {
                for c in 0..ch {
                    let prev = self.frame_sample(f - 1, c);
                    let a = self.frame_sample(f, c);
                    let b = self.frame_sample(f + 1, c);
                    let nxt = self.frame_sample(f + 2, c);
                    for w in &LAGRANGE_INTERSAMPLE_COEFFS {
                        let mid = w[0] * prev + w[1] * a + w[2] * b + w[3] * nxt;
                        let abs_mid = mid.abs();
                        if abs_mid > peak {
                            peak = abs_mid;
                        }
                    }
                }
            }
        }

        let required = if peak > self.ceiling_lin {
            self.ceiling_lin / peak.max(1.0e-9)
        } else {
            1.0
        };

        if required < self.gain {
            // Instant attack — the lookahead gives us time to ramp the OUTPUT
            // down before the peak hits the read pointer, so an instantaneous
            // gain change here translates to a smooth dip in the audible output.
            self.gain = required;
        } else {
            // Exponential release toward `required` (which is 1.0 when no
            // reduction is currently needed).
            self.gain = required - (required - self.gain) * self.release_coef;
        }

        // Output the OLDEST frame * current gain.
        for i in 0..frame.len().min(ch) {
            frame[i] = self.oldest_frame_buf[i] * self.gain;
        }
    }

    pub fn reset(&mut self) {
        for s in self.buffer.iter_mut() {
            *s = 0.0;
        }
        self.head_frame = 0;
        self.filled_frames = 0;
        self.gain = 1.0;
    }

    /// Read the channel sample at logical frame offset `f` (0 = oldest sample
    /// still in the buffer, `filled_frames - 1` = most recently written).
    fn frame_sample(&self, f: usize, c: usize) -> f32 {
        let actual_frame = (self.head_frame + f) % self.lookahead_frames;
        self.buffer[actual_frame * self.channels + c]
    }
}

pub struct MasteringChain {
    pub coeffs: ChainCoeffs,
    pub states: Vec<ChannelState>,
    pub limiter: Limiter,
}

const LIMITER_LOOKAHEAD_MS: f32 = 3.0;
const LIMITER_RELEASE_MS: f32 = 50.0;

impl MasteringChain {
    pub fn new(sample_rate: u32, channels: usize, settings: &MasteringSettings) -> Self {
        let coeffs = ChainCoeffs::from_settings(sample_rate, settings);
        let states = (0..channels).map(|_| ChannelState::default()).collect();
        let ceiling_dbfs = settings
            .advanced
            .ceiling_dbtp
            .unwrap_or(-1.0)
            .clamp(-6.0, 0.0);
        let limiter = Limiter::new(
            sample_rate,
            channels,
            ceiling_dbfs,
            LIMITER_LOOKAHEAD_MS,
            LIMITER_RELEASE_MS,
        );
        Self {
            coeffs,
            states,
            limiter,
        }
    }

    /// Build a sibling chain that inherits the current filter + limiter state
    /// but uses fresh coefficients. Used by `MasteringSource` to crossfade
    /// between old and new coefficients without re-ringing the filters or
    /// dropping the limiter's gain envelope from zero state.
    pub fn with_coeffs_inheriting_state(coeffs: ChainCoeffs, prior: &Self) -> Self {
        Self {
            coeffs,
            states: prior.states.clone(),
            limiter: prior.limiter.clone(),
        }
    }

    /// Process one interleaved frame in place. Runs gain → 3-band EQ →
    /// (optional stereo width) → saturation per channel, then the
    /// linked-stereo lookahead limiter across the frame. Width is inserted
    /// between EQ and saturation so the M/S decode sees the equalized signal
    /// but isn't fed through the non-linear stage twice; widening then
    /// saturating preserves the chosen stereo image instead of having the
    /// non-linearity smear it back toward mono.
    pub fn process_frame_inplace(&mut self, frame: &mut [f32]) {
        let channels = frame.len().min(self.states.len());
        if channels == 0 {
            return;
        }
        // Pass 1: per-channel input gain + 3-band EQ.
        for ch in 0..channels {
            let state = &mut self.states[ch];
            let mut y = frame[ch] * self.coeffs.input_gain_lin;
            y = state.low.process(&self.coeffs.low, y);
            y = state.mid.process(&self.coeffs.mid, y);
            y = state.high.process(&self.coeffs.high, y);
            frame[ch] = y;
        }
        // Width: only meaningful for stereo. The `≈ 1` guard skips the M/S
        // dance when the user hasn't touched the slider, keeping the
        // mono-summed signal mathematically identical to the pre-Phase-12.2
        // chain output for backward compatibility with existing tests.
        if channels == 2 && (self.coeffs.width_side_scale - 1.0).abs() > 1.0e-5 {
            apply_width_stereo(frame, self.coeffs.width_side_scale);
        }
        // Pass 2: per-channel saturation. Pulled out of pass 1 so width can
        // sit between EQ and the non-linear stage.
        if self.coeffs.saturation_amount > 0.0 {
            let drive = 1.0 + self.coeffs.saturation_amount * 2.0;
            let denom = drive.tanh().max(1.0e-3);
            for ch in 0..channels {
                frame[ch] = (frame[ch] * drive).tanh() / denom;
            }
        }
        self.limiter.process_frame_inplace(frame);
        // Volume Match: applied AFTER the limiter so the limiter still sees
        // the full post-gain peaks (and bounds them to the ceiling). The VM
        // scalar then attenuates the limited output down to source-matched
        // level for fair A/B comparison.
        if (self.coeffs.volume_match_gain_lin - 1.0).abs() > 1.0e-4 {
            for s in frame.iter_mut() {
                *s *= self.coeffs.volume_match_gain_lin;
            }
        }
        // User output gain — final trim. Applied last so it scales the
        // already-processed, already-limited signal. Boosting here CAN push
        // peaks above the ceiling (the user is asking for that level); the
        // export receipt's true-peak check catches it.
        if (self.coeffs.user_output_gain_lin - 1.0).abs() > 1.0e-4 {
            for s in frame.iter_mut() {
                *s *= self.coeffs.user_output_gain_lin;
            }
        }
    }

    pub fn process_interleaved(&mut self, samples: &mut [f32], channels: usize) {
        if channels == 0 || self.states.is_empty() {
            return;
        }
        for frame in samples.chunks_mut(channels) {
            self.process_frame_inplace(frame);
        }
    }

    /// Per-sample API. Bypasses the linked-stereo limiter (which needs a full
    /// frame to compute peaks). Used only as a legacy path for callers that
    /// haven't been migrated to `process_frame_inplace`; the soft-clip ceiling
    /// stays in place as a degraded fallback.
    pub fn process_sample(&mut self, sample: f32, channel: usize) -> f32 {
        let idx = if self.states.is_empty() {
            return sample;
        } else {
            channel.min(self.states.len() - 1)
        };
        let state = &mut self.states[idx];
        let mut y = sample * self.coeffs.input_gain_lin;
        y = state.low.process(&self.coeffs.low, y);
        y = state.mid.process(&self.coeffs.mid, y);
        y = state.high.process(&self.coeffs.high, y);
        if self.coeffs.saturation_amount > 0.0 {
            let drive = 1.0 + self.coeffs.saturation_amount * 2.0;
            y = (y * drive).tanh() / drive.tanh().max(1.0e-3);
        }
        let ceiling = self.coeffs.ceiling_lin;
        if y.abs() > ceiling {
            let over = y.abs() - ceiling;
            let shaped = ceiling + over.tanh() * 0.05;
            y = y.signum() * shaped;
        }
        y
    }

    pub fn reset_states(&mut self) {
        for state in self.states.iter_mut() {
            *state = ChannelState::default();
        }
        self.limiter.reset();
    }
}

// ============================================================================
// Tests — Phase 12.2 stereo width / M-S processing.
// `apply_width_stereo` is tested directly so the M/S math is pinned without
// having to drive samples through the full limiter lookahead. A separate
// integration test exercises the wiring inside `process_frame_inplace`.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    /// Width 0 collapses the stereo image to mono. For an L = sine, R = -sine
    /// input (pure-side signal), this means both channels go to zero.
    #[test]
    fn apply_width_stereo_zero_collapses_to_mono() {
        let mut frame = [0.5f32, -0.5];
        apply_width_stereo(&mut frame, 0.0);
        assert!(
            approx_eq(frame[0], frame[1], 1e-6),
            "width=0 must produce L == R, got L={} R={}",
            frame[0],
            frame[1],
        );
        // Mid of (0.5, -0.5) is 0, so the mono signal is zero.
        assert!(
            frame[0].abs() < 1e-6,
            "L=-R input + width=0 should be silence, got {}",
            frame[0]
        );
    }

    /// Width 1.0 is exactly identity for any stereo input.
    #[test]
    fn apply_width_stereo_one_is_identity() {
        let mut frame = [0.3f32, -0.7];
        apply_width_stereo(&mut frame, 1.0);
        assert!(approx_eq(frame[0], 0.3, 1e-6), "L drift at width=1: got {}", frame[0]);
        assert!(approx_eq(frame[1], -0.7, 1e-6), "R drift at width=1: got {}", frame[1]);
    }

    /// Width 1.5 amplifies the side component. Hand-computed expected values
    /// pinned: L=0.3, R=-0.7 → mid=-0.2, side=0.5; after 1.5× → side=0.75 →
    /// L=0.55, R=-0.95.
    #[test]
    fn apply_width_stereo_one_point_five_amplifies_side() {
        let mut frame = [0.3f32, -0.7];
        apply_width_stereo(&mut frame, 1.5);
        assert!(
            approx_eq(frame[0], 0.55, 1e-6),
            "L at width=1.5: expected 0.55, got {}",
            frame[0]
        );
        assert!(
            approx_eq(frame[1], -0.95, 1e-6),
            "R at width=1.5: expected -0.95, got {}",
            frame[1]
        );
    }

    /// Mid-only signal (L = R) must be unchanged regardless of width.
    /// Confirms width adjusts only the side component.
    #[test]
    fn apply_width_stereo_does_not_touch_pure_mid_signal() {
        for &w in &[0.0_f32, 0.5, 1.0, 1.5, 2.0] {
            let mut frame = [0.42f32, 0.42];
            apply_width_stereo(&mut frame, w);
            assert!(
                approx_eq(frame[0], 0.42, 1e-6) && approx_eq(frame[1], 0.42, 1e-6),
                "pure-mid signal changed under width={}, got L={} R={}",
                w,
                frame[0],
                frame[1],
            );
        }
    }

    /// Mono frame (length 1) is a no-op — guard prevents indexing past the
    /// frame's length.
    #[test]
    fn apply_width_stereo_no_op_on_mono_frame() {
        let mut frame = [0.7f32];
        apply_width_stereo(&mut frame, 0.0);
        assert!(approx_eq(frame[0], 0.7, 1e-6));
    }

    /// ChainCoeffs maps `Advanced.width = None` to a neutral side scale of 1.0
    /// — the slider being untouched must never alter the stereo image.
    #[test]
    fn chain_coeffs_default_width_is_neutral() {
        let settings = MasteringSettings {
            preset: Preset::Custom { id: "t".to_string() },
            intensity: 0.0,
            eq_low_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_db: 0.0,
            volume_match: false,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            advanced: AdvancedSettings::default(),
        };
        let c = ChainCoeffs::from_settings(44_100, &settings);
        assert!(
            approx_eq(c.width_side_scale, 1.0, 1e-6),
            "untouched Advanced.width should map to 1.0, got {}",
            c.width_side_scale
        );
    }

    /// Out-of-range user width values are clamped, not honored — a 5.0 user
    /// value can't push the side past 2× (which is already the wide end of
    /// what mastering plugins typically expose).
    #[test]
    fn chain_coeffs_clamps_width_into_safe_range() {
        let base = MasteringSettings {
            preset: Preset::Custom { id: "t".to_string() },
            intensity: 0.0,
            eq_low_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_db: 0.0,
            volume_match: false,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            advanced: AdvancedSettings {
                width: Some(5.0),
                ..AdvancedSettings::default()
            },
        };
        let c = ChainCoeffs::from_settings(44_100, &base);
        assert!(
            approx_eq(c.width_side_scale, 2.0, 1e-6),
            "user width=5.0 should clamp to 2.0, got {}",
            c.width_side_scale
        );

        let mut neg = base.clone();
        neg.advanced.width = Some(-1.0);
        let c_neg = ChainCoeffs::from_settings(44_100, &neg);
        assert!(
            approx_eq(c_neg.width_side_scale, 0.0, 1e-6),
            "user width=-1.0 should clamp to 0.0, got {}",
            c_neg.width_side_scale
        );
    }

    /// End-to-end: drive a stereo (L=sine, R=-sine) signal through the chain
    /// with width=0, neutral preset, neutral EQ, no saturation. After the
    /// limiter lookahead, the output should be silent because the M/S
    /// transform collapsed the pure-side signal to mono and mid is zero.
    #[test]
    fn process_frame_applies_width_inside_full_chain() {
        let settings = MasteringSettings {
            preset: Preset::Custom { id: "t".to_string() },
            intensity: 0.0,
            eq_low_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_db: 0.0,
            volume_match: false,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            advanced: AdvancedSettings {
                width: Some(0.0),
                ..AdvancedSettings::default()
            },
        };
        let mut chain = MasteringChain::new(44_100, 2, &settings);
        let mut last = [0.0f32, 0.0];
        for n in 0..2_048 {
            // Pure-side signal: L = +sine, R = -sine.
            let s = 0.4 * (n as f32 * 2.0 * std::f32::consts::PI * 1000.0 / 44_100.0).sin();
            let mut frame = [s, -s];
            chain.process_frame_inplace(&mut frame);
            last = [frame[0], frame[1]];
        }
        // After the limiter's 3 ms lookahead (~132 frames) the chain's output
        // should reflect the M/S collapse: both channels at silence.
        assert!(
            last[0].abs() < 1e-3,
            "width=0 inside chain should silence pure-side signal, got L={}",
            last[0]
        );
        assert!(
            last[1].abs() < 1e-3,
            "width=0 inside chain should silence pure-side signal, got R={}",
            last[1]
        );
    }

    /// Companion: with width=1.0 the same pure-side signal must NOT be
    /// silenced — proves the silence in the prior test is from width=0, not
    /// some upstream bug.
    #[test]
    fn process_frame_with_neutral_width_preserves_side_signal() {
        let settings = MasteringSettings {
            preset: Preset::Custom { id: "t".to_string() },
            intensity: 0.0,
            eq_low_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_db: 0.0,
            volume_match: false,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            advanced: AdvancedSettings {
                width: Some(1.0),
                ..AdvancedSettings::default()
            },
        };
        let mut chain = MasteringChain::new(44_100, 2, &settings);
        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        for n in 0..2_048 {
            let s = 0.4 * (n as f32 * 2.0 * std::f32::consts::PI * 1000.0 / 44_100.0).sin();
            let mut frame = [s, -s];
            chain.process_frame_inplace(&mut frame);
            if frame[0].abs() > peak_l {
                peak_l = frame[0].abs();
            }
            if frame[1].abs() > peak_r {
                peak_r = frame[1].abs();
            }
        }
        assert!(
            peak_l > 0.1 && peak_r > 0.1,
            "width=1 must pass the side signal through, got peak L={} R={}",
            peak_l,
            peak_r
        );
    }
}
