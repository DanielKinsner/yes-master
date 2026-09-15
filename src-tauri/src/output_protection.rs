//! Final-rate PCM protection and whole-file delivery verification.
//!
//! Expensive reconstruction is prepared once. A uniform gain preserves that
//! result up to explicitly measured rounding/dither residuals. Every delivered
//! sample is still read and loudness is remeasured after absolute gating.
use crate::peak_meter::{self, pcm::PcmSource};
use crate::types::{CommandError, CommandResult};
use crate::wav_writer::DeliveryPcm;
use ebur128::{EbuR128, Mode};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ProtectedPcm {
    pub gain_lin: f32,
    pub lufs: f32,
    pub lra: f32,
    /// Conservative qualified reconstruction reading for exact delivery PCM.
    pub true_peak_dbtp: f32,
    /// Full precision bound for subsequent programme-level gain planning.
    pub peak_upper: f64,
    pub interval_db: f64,
    pub used_fresh_meter: bool,
}

fn error(message: impl ToString) -> CommandError {
    CommandError::Render(message.to_string())
}

fn check_cancel(cancel: Option<&AtomicBool>) -> CommandResult<()> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        Err(error("output preparation cancelled"))
    } else {
        Ok(())
    }
}

fn finite_loudness(meter: &EbuR128) -> CommandResult<f32> {
    let value = meter.loudness_global().map_err(error)? as f32;
    if value.is_nan() || value == f32::INFINITY {
        return Err(error("invalid integrated loudness"));
    }
    Ok(value)
}

/// No file is written here. On error/cancellation callers discard this working
/// buffer. Source files and any previous renders are never modified.
pub fn finalize(
    samples: &mut [f32],
    rate: u32,
    channels: u16,
    bits: u16,
    target_lufs: Option<f32>,
    ceiling_dbtp: f32,
    cancel: Option<&AtomicBool>,
) -> CommandResult<ProtectedPcm> {
    let count = usize::from(channels);
    if count == 0 || rate == 0 || samples.len() % count != 0 || !matches!(bits, 16 | 24 | 32) {
        return Err(error("invalid delivery PCM format"));
    }
    let ceiling = 10_f64.powf(f64::from(ceiling_dbtp) / 20.);
    if !ceiling.is_finite() || ceiling <= 0. || !ceiling_dbtp.is_finite() {
        return Err(error("invalid peak ceiling"));
    }
    let cancelled = || cancel.is_some_and(|flag| flag.load(Ordering::Relaxed));
    check_cancel(cancel)?;
    let prepared = peak_meter::measure(samples, count, cancelled).map_err(error)?;
    let mut raw_meter = EbuR128::new(u32::from(channels), rate, Mode::I).map_err(error)?;
    for chunk in samples.chunks(4096 * count) {
        check_cancel(cancel)?;
        raw_meter.add_frames_f32(chunk).map_err(error)?;
    }
    let raw_lufs = finite_loudness(&raw_meter)?;
    let delta = crate::engine::ceiling_bounded_landing_delta_db(
        raw_lufs.is_finite().then_some(raw_lufs),
        prepared.upper_dbtp().map(|v| v as f32),
        target_lufs,
        ceiling_dbtp,
    );
    let desired_gain = if delta == 0. {
        1.
    } else {
        10_f32.powf(delta / 20.)
    };
    let frames = samples.len() / count;
    let norm = peak_meter::reconstruction_error_gain(frames);
    let lsb = match bits {
        16 => 1. / 32768.,
        24 => 1. / 8388608.,
        _ => 0.,
    };
    let maximum_sample = prepared
        .channels
        .iter()
        .map(|c| c.finite.sample_peak)
        .fold(0_f64, f64::max);
    let sample_limit = if bits == 32 {
        ceiling.min(f64::from(f32::MAX))
    } else {
        ceiling.min(1.)
    };
    // TPDF <=1 LSB, rounding <=0.5 LSB; each f32 addition/multiplication
    // receives a full-relative-epsilon allowance. Actual errors are measured
    // below. This reserve is format-, length- and reconstruction-specific.
    let round_reserve = f64::from(f32::EPSILON) * sample_limit;
    let quant_reserve = if bits == 32 {
        0.
    } else {
        1.5 * lsb + f64::from(f32::EPSILON) * (sample_limit + lsb)
    };
    let reserve = norm * (round_reserve + quant_reserve);
    if frames > 0 && reserve >= ceiling {
        return Err(error(
            "requested ceiling is below this PCM format's verified noise bound",
        ));
    }
    let mut cap = if prepared.upper() > 0. {
        (ceiling - reserve) / prepared.upper()
    } else {
        1.
    };
    if bits != 32 && maximum_sample > 0. {
        cap = cap.min((1. - 1.5 * lsb - f64::from(f32::EPSILON)) / maximum_sample);
    }
    let candidate = f64::from(desired_gain).min(cap);
    if !candidate.is_finite() || candidate < 0. {
        return Err(error("invalid landing gain"));
    }
    let mut gain = candidate as f32;
    // Round the protection cap down. The loudness helper's tiny-delta shortcut
    // must never waive a real peak attenuation, however small.
    if f64::from(gain) > candidate && gain > 0. {
        gain = f32::from_bits(gain.to_bits() - 1);
    }
    let mut gain_errors = vec![0_f64; count];
    for chunk in samples.chunks_mut(4096 * count) {
        check_cancel(cancel)?;
        for (i, value) in chunk.iter_mut().enumerate() {
            let exact = f64::from(*value) * f64::from(gain);
            *value *= gain;
            gain_errors[i % count] = gain_errors[i % count].max((f64::from(*value) - exact).abs());
        }
    }
    let delivery = DeliveryPcm::new(samples, channels, bits, cancelled)?;
    let mut meter = EbuR128::new(u32::from(channels), rate, Mode::I | Mode::LRA).map_err(error)?;
    let mut scratch = vec![0_f64; 4096];
    let mut interleaved = vec![0_f32; 4096 * count];
    let mut quant_errors = vec![0_f64; count];
    for start in (0..frames).step_by(4096) {
        check_cancel(cancel)?;
        let n = 4096.min(frames - start);
        for c in 0..count {
            delivery
                .read_channel(c, start, &mut scratch[..n])
                .map_err(error)?;
            for (i, &value) in scratch[..n].iter().enumerate() {
                interleaved[i * count + c] = value as f32;
                quant_errors[c] = quant_errors[c]
                    .max((value - f64::from(samples[(start + i) * count + c])).abs());
            }
        }
        meter
            .add_frames_f32(&interleaved[..n * count])
            .map_err(error)?;
    }
    let mut upper = 0_f64;
    let mut lower = 0_f64;
    for (c, original) in prepared.channels.iter().enumerate() {
        let residual = norm * (gain_errors[c] + quant_errors[c]);
        upper = upper.max(original.upper * f64::from(gain) + residual);
        lower = lower.max((original.lower * f64::from(gain) - residual).max(0.));
    }
    let width = |lo: f64, hi: f64| {
        if hi == 0. {
            0.
        } else {
            20. * (hi / lo).log10()
        }
    };
    let used_fresh_meter = width(lower, upper) > 0.0501;
    if used_fresh_meter {
        // Quiet/silent integer PCM may be dominated by dither. Reuse would be
        // safe but too imprecise; measure that delivered signal afresh.
        let fresh = peak_meter::measure_source(&delivery, cancelled).map_err(error)?;
        upper = fresh.upper();
        lower = fresh.channels.iter().map(|c| c.lower).fold(0_f64, f64::max);
    }
    check_cancel(cancel)?;
    let interval_db = width(lower, upper);
    if !upper.is_finite() || interval_db > 0.0501 {
        return Err(error("delivered peak measurement remains too uncertain"));
    }
    if upper > ceiling * (1. + 1e-10) {
        return Err(error("delivered PCM exceeds the requested peak ceiling"));
    }
    let lufs = finite_loudness(&meter)?;
    let lra = meter.loudness_range().map_err(error)? as f32;
    crate::diagnostics::info(format!(
        "PCM verified: estimator={} frames={frames} rate={rate} channels={count} bits={bits} interval_db={interval_db:.6} fresh_meter={used_fresh_meter}",
        peak_meter::VERSION));
    Ok(ProtectedPcm {
        gain_lin: gain,
        lufs,
        lra,
        true_peak_dbtp: if upper > 0. {
            (20. * upper.log10()) as f32
        } else {
            -60.
        },
        interval_db,
        used_fresh_meter,
        peak_upper: upper,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_target_short_silent_and_targeted_pcm_are_verified_after_quantization() {
        for bits in [16, 24, 32] {
            for target in [None, Some(-23.), Some(-8.)] {
                for kind in 0..4 {
                    let frames = if kind == 0 { 17 } else { 24_001 };
                    let source: Vec<f32> = (0..frames * 2)
                        .map(|i| match kind {
                            0 => {
                                if i % 2 == 0 {
                                    1.4
                                } else {
                                    -1.2
                                }
                            }
                            1 => 0.,
                            2 => ((i / 2) as f32 * std::f32::consts::TAU * 0.49).sin() * 0.95,
                            _ => {
                                ((i / 2) as f32 * std::f32::consts::TAU * 997. / 48000.).sin()
                                    * 0.08
                            }
                        })
                        .collect();
                    let mut samples = source;
                    let protected =
                        finalize(&mut samples, 48_000, 2, bits, target, -1., None).unwrap();
                    assert!(protected.true_peak_dbtp <= -1. + 1e-5);
                    assert!(protected.interval_db <= 0.0501);
                    assert!(samples.iter().all(|v| v.is_finite()));
                    let delivered = DeliveryPcm::new(&samples, 2, bits, || false).unwrap();
                    let direct = peak_meter::measure_source(&delivered, || false).unwrap();
                    assert!(
                        direct.upper() <= 10_f64.powf(-1. / 20.) * (1. + 1e-5),
                        "bits={bits} kind={kind}"
                    );
                    assert!(
                        direct
                            .channels
                            .iter()
                            .map(|c| c.lower)
                            .fold(0_f64, f64::max)
                            <= 10_f64.powf(f64::from(protected.true_peak_dbtp) / 20.) * (1. + 1e-6)
                    );
                    if kind == 0 {
                        assert!(protected.gain_lin < 1.);
                    }
                    if kind == 1 && bits != 32 {
                        assert!(protected.used_fresh_meter);
                    }
                }
            }
        }
    }

    #[test]
    fn cancellation_and_invalid_measurements_cannot_produce_a_delivery_result() {
        let cancel = AtomicBool::new(true);
        assert!(finalize(&mut [0.5], 48000, 1, 24, None, -1., Some(&cancel)).is_err());
        for sample in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(finalize(&mut [sample], 48000, 1, 24, None, -1., None).is_err());
        }
        for ceiling in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(finalize(&mut [0.], 48000, 1, 24, None, ceiling, None).is_err());
        }
    }
}
