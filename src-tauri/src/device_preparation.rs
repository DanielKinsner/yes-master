//! Worker-only whole-file facts for final device-rate audition.
//! The production player must apply this gain AFTER both SRC stages and match
//! its raw processing revision; file-rate/scalar commutation is not assumed.
use crate::{output_protection, sample_rate, types::*};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct PreparedDevicePcm {
    samples: Vec<f32>,
    measurements: output_protection::PreparedMeasurements,
    pub rate: u32,
    pub channels: u16,
}

pub struct DeviceDelivery {
    /// Exact gain-applied device PCM, owned by the worker. No quantization or
    /// lossy decoding is simulated here. Volume Match remains separate.
    pub samples: Vec<f32>,
    pub protection: output_protection::ProtectedPcm,
}

impl PreparedDevicePcm {
    pub fn new(
        file_rate_raw: &[f32],
        file_rate: u32,
        channels: u16,
        device_rate: u32,
        cancel: Option<&AtomicBool>,
    ) -> CommandResult<Self> {
        check_cancel(cancel)?;
        let samples = sample_rate::convert_interleaved_cancellable(
            file_rate_raw,
            file_rate,
            device_rate,
            channels,
            || cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)),
        )?;
        check_cancel(cancel)?;
        let measurements = output_protection::prepare(&samples, device_rate, channels, cancel)?;
        Ok(Self {
            samples,
            measurements,
            rate: device_rate,
            channels,
        })
    }

    pub fn pcm_bytes(&self) -> usize {
        self.samples.capacity() * std::mem::size_of::<f32>()
    }

    pub fn finish(
        &self,
        desired_file_gain: f32,
        ceiling_dbtp: f32,
        cancel: Option<&AtomicBool>,
    ) -> CommandResult<DeviceDelivery> {
        check_cancel(cancel)?;
        let mut samples = self.samples.clone();
        let protection = output_protection::finalize_prepared_device_gain(
            &mut samples,
            &self.measurements,
            desired_file_gain,
            ceiling_dbtp,
            cancel,
        )?;
        Ok(DeviceDelivery {
            samples,
            protection,
        })
    }

    pub fn finish_owned(
        mut self,
        desired_file_gain: f32,
        ceiling_dbtp: f32,
        cancel: Option<&AtomicBool>,
    ) -> CommandResult<DeviceDelivery> {
        check_cancel(cancel)?;
        let protection = output_protection::finalize_prepared_device_gain(
            &mut self.samples,
            &self.measurements,
            desired_file_gain,
            ceiling_dbtp,
            cancel,
        )?;
        Ok(DeviceDelivery {
            samples: self.samples,
            protection,
        })
    }
}

fn check_cancel(cancel: Option<&AtomicBool>) -> CommandResult<()> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        Err(CommandError::Render("device preparation cancelled".into()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_plan_reuses_response_but_verifies_every_gain_and_keeps_zero_available() {
        for rate in [44100, 48000, 96000] {
            for channels in [1, 2] {
                let frames = 8193;
                let input: Vec<f32> = (0..frames * usize::from(channels))
                    .map(|n| 1.4 * (n as f32 * 1.171).sin())
                    .collect();
                let prepared = PreparedDevicePcm::new(&input, 48000, channels, rate, None).unwrap();
                for gain in [0., 1e-4, 0.25, 1., 3., 1.] {
                    let result = prepared.finish(gain, -1., None).unwrap();
                    assert!(result.protection.gain_lin <= gain);
                    assert_eq!(
                        result.samples.len() / usize::from(channels),
                        (frames * rate as usize).div_ceil(48000)
                    );
                    let actual =
                        crate::peak_meter::measure(&result.samples, usize::from(channels), || {
                            false
                        })
                        .unwrap();
                    assert!(actual.upper_dbtp().is_none_or(|peak| peak <= -1. + 1e-5));
                    // Check the exact second f32 product used by optional VM.
                    for vm in [1., f32::from_bits(1_f32.to_bits() - 1), 0.37] {
                        let matched: Vec<_> =
                            result.samples.iter().map(|sample| sample * vm).collect();
                        let actual =
                            crate::peak_meter::measure(&matched, usize::from(channels), || false)
                                .unwrap();
                        assert!(actual.upper_dbtp().is_none_or(|peak| peak <= -1. + 1e-5));
                    }
                    if gain == 0. {
                        assert!(result.samples.iter().all(|sample| *sample == 0.));
                    }
                }
                let again = prepared.finish(1., -1., None).unwrap();
                assert_eq!(
                    again.samples,
                    prepared.finish(1., -1., None).unwrap().samples
                );
            }
        }
    }

    #[test]
    fn invalid_and_cancelled_device_plans_cannot_return_success() {
        let cancel = AtomicBool::new(true);
        assert!(PreparedDevicePcm::new(&[0.1], 48000, 1, 44100, Some(&cancel)).is_err());
        assert!(PreparedDevicePcm::new(&[f32::NAN], 48000, 1, 44100, None).is_err());
        let prepared = PreparedDevicePcm::new(&[0.1], 48000, 1, 44100, None).unwrap();
        for gain in [f32::NAN, f32::INFINITY, -1.] {
            assert!(prepared.finish(gain, -1., None).is_err());
        }
        assert!(prepared.finish(1., -1., Some(&cancel)).is_err());
    }
}
