//! Polyphase comparison with the qualified SOXR impulse response.
//! This is a measurement filter, never a processing/limiter filter. The kernel
//! is embedded with its generator/version/hash provenance alongside this module.
use super::{allocated, pcm::PcmSource};
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use std::sync::Arc;

const CORE: usize = 4096;
const SUPPORT: usize = 512;
const FACTOR: usize = 16;
const FFT_SIZE: usize = (CORE + 4 * SUPPORT).next_power_of_two();

pub struct ReconstructionFir {
    forward: Arc<dyn RealToComplex<f64>>,
    inverse: Arc<dyn ComplexToReal<f64>>,
    filters: Vec<Vec<Complex<f64>>>,
}

impl ReconstructionFir {
    pub fn new(kernel: &[f64]) -> Self {
        assert_eq!(kernel.len(), 2 * SUPPORT * FACTOR + 1);
        assert!(kernel.iter().all(|v| v.is_finite()));
        let mut planner = RealFftPlanner::new();
        let forward = planner.plan_fft_forward(FFT_SIZE);
        let inverse = planner.plan_fft_inverse(FFT_SIZE);
        let mut filters = Vec::new();
        for phase in 0..FACTOR {
            let mut filter = forward.make_input_vec();
            for j in -(SUPPORT as i64)..=SUPPORT as i64 {
                let index = ((j + SUPPORT as i64) * FACTOR as i64 + phase as i64) as usize;
                if let Some(value) = kernel.get(index) {
                    filter[j.rem_euclid(FFT_SIZE as i64) as usize] = *value;
                }
            }
            let mut spectrum = forward.make_output_vec();
            forward
                .process(&mut filter, &mut spectrum)
                .expect("fixed kernel geometry");
            filters.push(spectrum);
        }
        Self {
            forward,
            inverse,
            filters,
        }
    }

    pub fn measure(
        &self,
        source: &(impl PcmSource + ?Sized),
        cancelled: impl Fn() -> bool,
    ) -> Result<Vec<f64>, &'static str> {
        let frames = source.frames();
        let mut input = allocated(FFT_SIZE)?;
        let mut spectrum = allocated(FFT_SIZE / 2 + 1)?;
        let mut product = allocated(FFT_SIZE / 2 + 1)?;
        let mut output = allocated(FFT_SIZE)?;
        let mut scratch = allocated(
            self.forward
                .get_scratch_len()
                .max(self.inverse.get_scratch_len()),
        )?;
        let mut peaks: Vec<f64> = allocated(source.channels())?;
        for (channel, peak) in peaks.iter_mut().enumerate() {
            for start in (-(CORE as i64)..(frames + SUPPORT).div_ceil(CORE) as i64 * CORE as i64)
                .step_by(CORE)
            {
                if cancelled() {
                    return Err("cancelled");
                }
                let lo = start - SUPPORT as i64;
                let a = lo.clamp(0, frames as i64) as usize;
                let b = (start + (CORE + SUPPORT) as i64).clamp(0, frames as i64) as usize;
                if b == a {
                    continue;
                }
                input.fill(0.0);
                let offset = (a as i64 - lo) as usize;
                source.read_channel(channel, a, &mut input[offset..offset + b - a])?;
                self.forward
                    .process_with_scratch(&mut input, &mut spectrum, &mut scratch)
                    .map_err(|_| "FFT input transform failed")?;
                for filter in &self.filters {
                    if cancelled() {
                        return Err("cancelled");
                    }
                    for ((out, x), h) in product.iter_mut().zip(&spectrum).zip(filter) {
                        *out = *x * *h;
                    }
                    self.inverse
                        .process_with_scratch(&mut product, &mut output, &mut scratch)
                        .map_err(|_| "FFT output transform failed")?;
                    *peak = output[SUPPORT..SUPPORT + CORE]
                        .iter()
                        .fold(*peak, |m, v| m.max(v.abs() / FFT_SIZE as f64));
                }
            }
        }
        Ok(peaks)
    }
}
