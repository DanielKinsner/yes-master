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

fn lowpass_filter() -> &'static reconstruction_fir::ReconstructionFir {
    static FIR: OnceLock<reconstruction_fir::ReconstructionFir> = OnceLock::new();
    FIR.get_or_init(|| {
        let bytes = include_bytes!("soxr16.f64le");
        let kernel: Vec<_> = bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().expect("fixed coefficient width")))
            .collect();
        reconstruction_fir::ReconstructionFir::new(&kernel)
    })
}

/// Bound either reconstruction's change from a maximum per-sample error.
/// For finite sinc: at most two terms at distance k-1/2 from the nearest
/// integer; bound their harmonic sum by its first term plus an integral.
/// For the lowpass response: maximum polyphase L1 norm and the qualified
/// between-grid/numerical envelope. This is amplitude-domain, not a dB margin.
pub fn reconstruction_error_gain(frames: usize) -> f64 {
    if frames == 0 {
        return 0.;
    }
    let sinc = if frames == 1 {
        1.
    } else {
        1. + 2. / PI * (2. + (2. * frames as f64 - 1.).ln())
    };
    sinc.max((lowpass_filter().maximum_l1 + FIR_NUMERICAL_GAIN) / GRID_DENOMINATOR)
        * (1. + 16. * f64::EPSILON)
}

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

/// Preview-only entry: the exact same qualified calculations, with bounded
/// independent work. File-backed export providers retain the serial entry.
pub(crate) fn measure_preview(
    samples: &[f32],
    channels: usize,
    cancelled: impl Fn() -> bool + Sync,
) -> Result<PeakMeasurement, &'static str> {
    let workers =
        std::thread::available_parallelism().map_or(1, |n| n.get().saturating_sub(1).clamp(1, 4));
    measure_preview_source(
        &InterleavedPcm::new(samples, channels)?,
        &cancelled,
        workers,
    )
}

fn measure_preview_source(
    source: &(impl PcmSource + Sync),
    cancelled: &(impl Fn() -> bool + Sync),
    workers: usize,
) -> Result<PeakMeasurement, &'static str> {
    if workers <= 1 || source.frames() < 32768 || source.channels() == 0 || source.channels() > 32 {
        return measure_source(source, cancelled);
    }
    if source.frames() > (i64::MAX as usize / 8).min(usize::MAX / 8) {
        return Err("invalid peak PCM geometry");
    }
    if cancelled() {
        return Err("cancelled");
    }
    // Each lane reads one original channel with exactly the serial arithmetic.
    // Separate providers keep file-backed, non-Sync export readers untouched.
    struct Channel<'a, S>(&'a S, usize);
    impl<S: PcmSource> PcmSource for Channel<'_, S> {
        fn frames(&self) -> usize {
            self.0.frames()
        }
        fn channels(&self) -> usize {
            1
        }
        fn read_channel(
            &self,
            channel: usize,
            start: usize,
            out: &mut [f64],
        ) -> Result<(), &'static str> {
            if channel != 0 {
                return Err("PCM channel out of range");
            }
            self.0.read_channel(self.1, start, out)
        }
    }
    enum Part {
        Finite(finite_peak::ChannelPeak),
        Lowpass(f64),
    }
    let jobs = source.channels() * 2;
    let workers = workers.clamp(1, 4).min(jobs);
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for lane in 0..workers {
            handles.push(
                std::thread::Builder::new()
                    .name(format!("preview-peak-{lane}"))
                    .spawn_scoped(scope, move || {
                        (lane..jobs)
                            .step_by(workers)
                            .map(|job| {
                                let channel = Channel(source, job / 2);
                                let result = if job % 2 == 0 {
                                    finite_peak::measure(&channel, cancelled)
                                        .map(|mut values| Part::Finite(values.remove(0)))
                                } else {
                                    lowpass_filter()
                                        .measure(&channel, cancelled)
                                        .map(|values| Part::Lowpass(values[0]))
                                };
                                (job / 2, result)
                            })
                            .collect::<Vec<_>>()
                    }),
            );
        }
        // Join every worker even on error/cancellation, before returning to the
        // single preview-worker permit or releasing the immutable PCM.
        handles
            .into_iter()
            .map(|handle| match handle {
                Ok(handle) => handle.join().map_err(|_| "preview peak worker failed"),
                Err(_) => Err("preview peak worker could not start"),
            })
            .collect::<Vec<_>>()
    });
    let mut finite: Vec<Option<finite_peak::ChannelPeak>> = std::iter::repeat_with(|| None)
        .take(source.channels())
        .collect();
    let mut lowpass = vec![0.; source.channels()];
    for result in results {
        for (channel, part) in result? {
            match part? {
                Part::Finite(value) => finite[channel] = Some(value),
                Part::Lowpass(value) => lowpass[channel] = value,
            }
        }
    }
    if cancelled() {
        return Err("cancelled");
    }
    combine_measurements(
        finite
            .into_iter()
            .map(|value| value.expect("every channel measured"))
            .collect(),
        lowpass,
    )
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use std::{collections::HashSet, sync::Mutex};

    #[test]
    fn parallel_preview_preserves_edges_silence_channels_and_failures() {
        for channels in [1, 2, 6] {
            for silent in [false, true] {
                let mut samples = vec![0.; 32771 * channels];
                if !silent {
                    for channel in 0..channels {
                        samples[channel] = (channel + 1) as f32 * 0.13;
                        samples[32770 * channels + channel] = -0.87;
                    }
                }
                let serial = measure(&samples, channels, || false).unwrap();
                let parallel = measure_preview_source(
                    &InterleavedPcm::new(&samples, channels).unwrap(),
                    &|| false,
                    4,
                )
                .unwrap();
                assert_eq!(
                    serde_json::to_value(serial).unwrap(),
                    serde_json::to_value(parallel).unwrap()
                );
            }
        }
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut samples = vec![0.; 32771 * 2];
            samples[32770 * 2 + 1] = invalid;
            assert!(measure_preview(&samples, 2, || false).is_err());
        }
        use std::sync::atomic::{AtomicUsize, Ordering};
        let checks = AtomicUsize::new(0);
        let samples = vec![0.1; 96001 * 2];
        let pcm = InterleavedPcm::new(&samples, 2).unwrap();
        assert_eq!(
            measure_preview_source(&pcm, &|| checks.fetch_add(1, Ordering::Relaxed) > 6, 4)
                .unwrap_err(),
            "cancelled"
        );
        // A cancelled run leaves no poisoned shared state or surviving workers.
        let serial = measure(&samples, 2, || false).unwrap();
        let fresh = measure_preview_source(&pcm, &|| false, 4).unwrap();
        assert_eq!(
            serde_json::to_value(serial).unwrap(),
            serde_json::to_value(fresh).unwrap()
        );
    }

    #[test]
    fn preview_parallel_work_preserves_every_peak_fact_with_a_fixed_thread_budget() {
        struct Observed<'a> {
            pcm: InterleavedPcm<'a>,
            threads: Mutex<HashSet<std::thread::ThreadId>>,
        }
        impl PcmSource for Observed<'_> {
            fn frames(&self) -> usize {
                self.pcm.frames()
            }
            fn channels(&self) -> usize {
                self.pcm.channels()
            }
            fn read_channel(
                &self,
                channel: usize,
                start: usize,
                out: &mut [f64],
            ) -> Result<(), &'static str> {
                self.threads
                    .lock()
                    .unwrap()
                    .insert(std::thread::current().id());
                self.pcm.read_channel(channel, start, out)
            }
        }
        let samples: Vec<f32> = (0..96001 * 2)
            .map(|i| {
                if i % 997 == 0 {
                    1.2
                } else {
                    (i as f32 * 0.17).sin() * 0.31
                }
            })
            .collect();
        let reference = measure(&samples, 2, || false).unwrap();
        for budget in [2, 4] {
            let observed = Observed {
                pcm: InterleavedPcm::new(&samples, 2).unwrap(),
                threads: Mutex::new(HashSet::new()),
            };
            let actual = measure_preview_source(&observed, &|| false, budget).unwrap();
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(&reference).unwrap()
            );
            let workers = observed.threads.lock().unwrap().len();
            assert!(
                workers > 1 && workers <= budget,
                "independent peak work must use the bounded worker budget, observed {workers}"
            );
        }
    }
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
    let lowpass = lowpass_filter().measure(source, &cancelled)?;
    combine_measurements(finite, lowpass)
}

fn combine_measurements(
    finite: Vec<finite_peak::ChannelPeak>,
    lowpass: Vec<f64>,
) -> Result<PeakMeasurement, &'static str> {
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
