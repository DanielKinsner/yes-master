//! Offline finite-signal peak measurement. Never call this from an audio callback.
//!
//! Two explicitly qualified reconstructions are measured: finite cardinal sinc
//! and the recorded lowpass response. All samples, channel maxima and exterior
//! ringout participate. Results are upper bounds with recorded uncertainty, not
//! a limiter algorithm or a promise about every possible DAC reconstruction.
//! Qualification: docs/reviews/2026-09-15-peak-reconstruction-qualification.md.
mod finite_peak;
pub mod pcm;
mod reconstruction_fir;

use pcm::{InterleavedPcm, PcmSource};
use serde::Serialize;
use std::f64::consts::PI;
use std::sync::OnceLock;

pub const VERSION: &str = "finite-sinc20-soxr16-4591255f-realfft-1";
const GRID_DENOMINATOR: f64 = 1. - PI * PI / (8. * 16. * 16.);
// Tested filter residual (<2e-13 per unit source peak) plus the fixed-size
// f64 FFT error envelope. This is amplitude-relative, not a fixed dB margin.
const FIR_NUMERICAL_GAIN: f64 = 1e-10;

fn allocated<T: Clone + Default>(len: usize) -> Result<Vec<T>, &'static str> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(len)
        .map_err(|_| "peak workspace allocation failed")?;
    result.resize(len, T::default());
    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct ChannelPeak {
    pub finite: finite_peak::ChannelPeak,
    pub lowpass_grid: f64,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Serialize)]
pub struct PeakMeasurement {
    pub version: &'static str,
    pub channels: Vec<ChannelPeak>,
}

impl PeakMeasurement {
    pub fn upper(&self) -> f64 {
        self.channels.iter().fold(0_f64, |m, c| m.max(c.upper))
    }

    /// Silence has no finite dB peak. Keep it distinct from measurement failure.
    pub fn upper_dbtp(&self) -> Option<f64> {
        let peak = self.upper();
        (peak > 0.).then(|| 20. * peak.log10())
    }

    pub fn widest_interval_db(&self) -> f64 {
        self.channels.iter().fold(0_f64, |m, c| {
            m.max(if c.upper == 0. {
                0.
            } else {
                20. * (c.upper / c.lower).log10()
            })
        })
    }
}

pub fn measure(
    samples: &[f32],
    channels: usize,
    cancelled: impl Fn() -> bool,
) -> Result<PeakMeasurement, &'static str> {
    measure_source(&InterleavedPcm::new(samples, channels)?, cancelled)
}

/// The provider can replay deterministic delivery PCM from bounded scratch.
/// State is local to this call; only immutable filter plans are shared. Source
/// facts/results are not cached here, so a second source cannot inherit a peak.
pub fn measure_source(
    source: &(impl PcmSource + ?Sized),
    cancelled: impl Fn() -> bool,
) -> Result<PeakMeasurement, &'static str> {
    if source.channels() == 0
        || source.channels() > 32
        || source.frames() > (i64::MAX as usize / 8).min(usize::MAX / 8)
    {
        return Err("invalid peak PCM geometry");
    }
    if cancelled() {
        return Err("cancelled");
    }
    let finite = finite_peak::measure(source, &cancelled)?;
    if cancelled() {
        return Err("cancelled");
    }
    static FIR: OnceLock<reconstruction_fir::ReconstructionFir> = OnceLock::new();
    let fir = FIR.get_or_init(|| {
        let bytes = include_bytes!("soxr16.f64le");
        let kernel: Vec<_> = bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().expect("fixed coefficient width")))
            .collect();
        reconstruction_fir::ReconstructionFir::new(&kernel)
    });
    let lowpass = fir.measure(source, &cancelled)?;
    let mut channels = Vec::new();
    channels
        .try_reserve_exact(finite.len())
        .map_err(|_| "peak result allocation failed")?;
    for (finite, lowpass_grid) in finite.into_iter().zip(lowpass) {
        let reserve = FIR_NUMERICAL_GAIN * finite.sample_peak;
        let lower = finite.grid_lower.max((lowpass_grid - reserve).max(0.));
        let upper = finite
            .continuous_upper
            .max((lowpass_grid + reserve) / GRID_DENOMINATOR);
        if !lower.is_finite() || !upper.is_finite() {
            return Err("non-finite peak result");
        }
        channels.push(ChannelPeak {
            finite,
            lowpass_grid,
            lower,
            upper,
        });
    }
    Ok(PeakMeasurement {
        version: VERSION,
        channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct SplitReads<'a>(InterleavedPcm<'a>);
    impl PcmSource for SplitReads<'_> {
        fn frames(&self) -> usize {
            self.0.frames()
        }
        fn channels(&self) -> usize {
            self.0.channels()
        }
        fn read_channel(
            &self,
            channel: usize,
            start: usize,
            out: &mut [f64],
        ) -> Result<(), &'static str> {
            for (i, chunk) in out.chunks_mut(257).enumerate() {
                self.0.read_channel(channel, start + 257 * i, chunk)?;
            }
            Ok(())
        }
    }

    #[test]
    fn replayable_reads_preserve_channels_finite_edges_and_reset() {
        let mut stereo = vec![0.; 2 * 8193];
        for frame in 0..8193 {
            stereo[2 * frame + 1] = if frame % 2 == 0 { 0.6 } else { -0.6 };
        }
        stereo[2 * 8192] = 1.4;
        let direct = measure(&stereo, 2, || false).unwrap();
        let source = SplitReads(InterleavedPcm::new(&stereo, 2).unwrap());
        let replay = measure_source(&source, || false).unwrap();
        assert_eq!(
            serde_json::to_value(&direct).unwrap(),
            serde_json::to_value(&replay).unwrap()
        );
        assert!(direct.channels[0].upper >= 1.4_f32 as f64);
        assert!(direct.channels[1].upper > 1.4);
        let silent = measure(&[0.; 34], 2, || false).unwrap();
        assert_eq!(silent.upper_dbtp(), None);
        assert_eq!(silent.widest_interval_db(), 0.);
    }

    #[test]
    fn invalid_or_cancelled_measurements_never_become_silence() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut x = vec![0.; 8193];
            x[8192] = bad;
            assert!(measure(&x, 1, || false).is_err());
        }
        assert!(measure(&[0.], 2, || false).is_err());
        assert!(measure(&[0.], 0, || false).is_err());
        let source = vec![0.2; 40_000];
        for limit in [0, 2, 17, 50] {
            let calls = AtomicUsize::new(0);
            assert_eq!(
                measure(&source, 1, || calls.fetch_add(1, Ordering::Relaxed)
                    >= limit)
                .unwrap_err(),
                "cancelled"
            );
            assert_eq!(calls.load(Ordering::Relaxed), limit + 1);
        }
    }
}
