//! Finite cardinal-sinc peak interval. Offline/background workers only.
//!
//! Matches qualify_sinc_bound.py using realfft/rustfft instead of SciPy's real
//! FFT. Local finite sinc convolution, Abel bounds on omitted alternating
//! prefix sums, and a Bernstein between-grid bound. See the B2 research record.
use super::allocated;
use super::pcm::PcmSource;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use serde::Serialize;
use std::f64::consts::PI;
use std::sync::{Arc, OnceLock};
#[path = "tail_moments.rs"]
mod tail_moments;

const CORE: usize = 4096;
const GUARD: usize = 4096;
const SIZE: usize = CORE + 2 * GUARD;
const FFT_SIZE: usize = (2 * SIZE - 1).next_power_of_two();
const FACTOR: usize = 16;

struct Kernels {
    forward: Arc<dyn RealToComplex<f64>>,
    inverse: Arc<dyn ComplexToReal<f64>>,
    spectra: Vec<Vec<Complex<f64>>>,
}

struct Workspace<'a, S: PcmSource + ?Sized> {
    source: &'a S,
    bank: &'static Kernels,
    input: Vec<f64>,
    output: Vec<f64>,
    spectrum: Vec<Complex<f64>>,
    product: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
}

impl<S: PcmSource + ?Sized> Workspace<'_, S> {
    fn grid(
        &mut self,
        channel: usize,
        start: i64,
        polynomial: Option<&tail_moments::Polynomial>,
        cancelled: &impl Fn() -> bool,
    ) -> Result<(f64, usize, usize), &'static str> {
        let frames = self.source.frames();
        let lo = start - GUARD as i64;
        let a = lo.clamp(0, frames as i64) as usize;
        let b = (start + (CORE + GUARD) as i64).clamp(0, frames as i64) as usize;
        self.input.fill(0.0);
        if b > a {
            let offset = (a as i64 - lo) as usize;
            self.source
                .read_channel(channel, a, &mut self.input[offset..offset + b - a])?;
        }
        let mut local = self.input[GUARD..GUARD + CORE]
            .iter()
            .fold(0.0_f64, |m, v| m.max(v.abs()));
        if b == a && polynomial.is_none() {
            return Ok((local, a, b));
        }
        self.bank
            .forward
            .process_with_scratch(&mut self.input, &mut self.spectrum, &mut self.scratch)
            .map_err(|_| "FFT input transform failed")?;
        for (phase, filter) in self.bank.spectra.iter().enumerate() {
            if cancelled() {
                return Err("cancelled");
            }
            for ((out, input), kernel) in self.product.iter_mut().zip(&self.spectrum).zip(filter) {
                *out = *input * *kernel;
            }
            self.bank
                .inverse
                .process_with_scratch(&mut self.product, &mut self.output, &mut self.scratch)
                .map_err(|_| "FFT output transform failed")?;
            let fraction = (phase + 1) as f64 / FACTOR as f64;
            let sin_phase = (PI * fraction).sin() / PI;
            for (i, v) in self.output[GUARD..GUARD + CORE].iter().enumerate() {
                let mut value = v / FFT_SIZE as f64;
                if let Some(poly) = polynomial {
                    let at = (i as f64 + fraction - CORE as f64 / 2.) / (CORE as f64 / 2.);
                    let sign = if (start + i as i64) % 2 == 0 { 1. } else { -1. };
                    value += sign * sin_phase * tail_moments::evaluate(poly, at);
                }
                local = local.max(value.abs());
            }
        }
        Ok((local, a, b))
    }
}

fn kernels() -> &'static Kernels {
    static KERNELS: OnceLock<Kernels> = OnceLock::new();
    KERNELS.get_or_init(|| {
        let mut planner = RealFftPlanner::new();
        let forward = planner.plan_fft_forward(FFT_SIZE);
        let inverse = planner.plan_fft_inverse(FFT_SIZE);
        let mut spectra = Vec::new();
        for phase in 1..FACTOR {
            let p = phase as f64 / FACTOR as f64;
            let mut kernel = forward.make_input_vec();
            for j in -(SIZE as i64 - 1)..SIZE as i64 {
                let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
                kernel[j.rem_euclid(FFT_SIZE as i64) as usize] =
                    sign * (PI * p).sin() / (PI * (j as f64 + p));
            }
            let mut spectrum = forward.make_output_vec();
            forward
                .process(&mut kernel, &mut spectrum)
                .expect("fixed kernel geometry");
            spectra.push(spectrum);
        }
        Kernels {
            forward,
            inverse,
            spectra,
        }
    })
}

#[derive(Clone)]
struct PrefixBlock {
    minimum: f64,
    maximum: f64,
    start: f64,
    end: f64,
}

struct Summary {
    blocks: Vec<PrefixBlock>,
    frames: usize,
    l1: f64,
    sample_peak: f64,
}

impl Summary {
    fn new(
        source: &(impl PcmSource + ?Sized),
        channel: usize,
        cancelled: &impl Fn() -> bool,
    ) -> Result<Self, &'static str> {
        let frames = source.frames();
        let mut blocks = Vec::new();
        blocks
            .try_reserve_exact(frames.div_ceil(CORE))
            .map_err(|_| "peak summary allocation failed")?;
        let mut block = allocated(CORE)?;
        let (mut offset, mut compensation, mut l1, mut sample_peak) = (0.0, 0.0, 0.0, 0.0_f64);
        for start in (0..frames).step_by(CORE) {
            if cancelled() {
                return Err("cancelled");
            }
            let (mut sum, mut minimum, mut maximum) = (0.0_f64, 0.0_f64, 0.0_f64);
            let count = CORE.min(frames - start);
            source.read_channel(channel, start, &mut block[..count])?;
            for (i, &x) in block[..count].iter().enumerate() {
                let frame = start + i;
                if !x.is_finite() || x.abs() > f64::from(f32::MAX) {
                    return Err("non-finite or out-of-range PCM");
                }
                l1 += x.abs();
                sample_peak = sample_peak.max(x.abs());
                sum += if frame % 2 == 0 { x } else { -x };
                minimum = minimum.min(sum);
                maximum = maximum.max(sum);
            }
            let increment = sum - compensation;
            let next = offset + increment;
            compensation = (next - offset) - increment;
            blocks.push(PrefixBlock {
                minimum: offset + minimum,
                maximum: offset + maximum,
                start: offset,
                end: next,
            });
            offset = next;
        }
        Ok(Self {
            blocks,
            frames,
            l1,
            sample_peak,
        })
    }

    fn bound(&self, a: usize, b: usize, nearest: f64, left: bool) -> f64 {
        if a == b {
            return 0.0;
        }
        assert!(a % CORE == 0 && (b % CORE == 0 || b == self.frames));
        let span = &self.blocks[a / CORE..b.div_ceil(CORE)];
        let anchor = if left {
            span.last().unwrap().end
        } else {
            span[0].start
        };
        let amplitude = span.iter().fold(0.0_f64, |m, p| {
            m.max((p.minimum - anchor).abs())
                .max((p.maximum - anchor).abs())
        });
        let distance = if left {
            nearest - (b - 1) as f64
        } else {
            a as f64 - nearest
        };
        assert!(distance > 0.0);
        amplitude / distance / PI
    }

    fn tail(&self, mut a: usize, mut b: usize, nearest: f64, left: bool) -> f64 {
        let whole = self.bound(a, b, nearest, left);
        let (mut parts, mut width) = (0.0, GUARD);
        while b > a {
            if left {
                let c = a.max(b.saturating_sub(width) / CORE * CORE);
                parts += self.bound(c, b, nearest, true);
                b = c;
            } else {
                let c = b.min(a.saturating_add(width) / CORE * CORE);
                parts += self.bound(a, c, nearest, false);
                a = c;
            }
            width = width.saturating_mul(2);
        }
        whole.min(parts)
    }
}

#[derive(Debug, Serialize)]
pub struct ChannelPeak {
    pub sample_peak: f64,
    pub local_grid_estimate: f64,
    pub grid_lower: f64,
    pub continuous_upper: f64,
    pub numerical_allowance: f64,
    pub examined_blocks: usize,
    pub summary_bytes: usize,
    pub refined_blocks: usize,
    pub moment_tree_bytes: usize,
}

fn new_workspace<S: PcmSource + ?Sized>(source: &S) -> Result<Workspace<'_, S>, &'static str> {
    let bank = kernels();
    let scratch_len = bank
        .forward
        .get_scratch_len()
        .max(bank.inverse.get_scratch_len());
    Ok(Workspace {
        source,
        bank,
        input: allocated(FFT_SIZE)?,
        output: allocated(FFT_SIZE)?,
        spectrum: allocated(FFT_SIZE / 2 + 1)?,
        product: allocated(FFT_SIZE / 2 + 1)?,
        scratch: allocated(scratch_len)?,
    })
}

/// One block's grid estimate and omitted-tail uncertainty, in file order.
type Record = (i64, f64, f64, usize, usize);

pub fn measure(
    source: &(impl PcmSource + ?Sized),
    workers: super::Workers,
    cancelled: impl Fn() -> bool + Sync,
) -> Result<Vec<ChannelPeak>, &'static str> {
    if cancelled() {
        return Err("cancelled");
    }
    let mut workspace = new_workspace(source)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(source.channels())
        .map_err(|_| "peak result allocation failed")?;
    for channel in 0..source.channels() {
        let summary = Summary::new(source, channel, &cancelled)?;
        let mut peak = ChannelPeak {
            sample_peak: summary.sample_peak,
            local_grid_estimate: summary.sample_peak,
            grid_lower: summary.sample_peak,
            continuous_upper: summary.sample_peak,
            numerical_allowance: 0.0,
            examined_blocks: 0,
            summary_bytes: summary.blocks.len() * std::mem::size_of::<PrefixBlock>(),
            refined_blocks: 0,
            moment_tree_bytes: 0,
        };
        if summary.sample_peak == 0.0 {
            result.push(peak);
            continue;
        }
        // Explicit conservative floating-point allowance, proportional to actual
        // signal L1 and operation depth. Qualification still checks independent
        // f64 references; this is not a formally rounded interval implementation.
        peak.numerical_allowance =
            f64::EPSILON * (16 * CORE + 512 * FFT_SIZE.ilog2() as usize) as f64 * summary.l1;
        let extent = (summary.l1 / (PI * summary.sample_peak)).ceil() as usize + 1;
        let first = -((extent.div_ceil(CORE) * CORE) as i64);
        let last = (summary.frames + extent).div_ceil(CORE) * CORE;
        let blocks = (last as i64 - first) as usize / CORE;
        let allowance = peak.numerical_allowance;
        // Blocks are independent; worker threads may take contiguous ranges.
        let parts = super::for_block_ranges(source, blocks, workers, |reader, range| {
            let mut workspace = new_workspace(reader)?;
            let mut records: Vec<Record> = Vec::new();
            records
                .try_reserve_exact(range.len())
                .map_err(|_| "peak interval allocation failed")?;
            for index in range {
                if cancelled() {
                    return Err("cancelled");
                }
                let start = first + (index * CORE) as i64;
                let stop = start + CORE as i64;
                let (local, a, b) = workspace.grid(channel, start, None, &cancelled)?;
                let left = if a > 0 {
                    summary.tail(0, a, start as f64, true)
                } else {
                    0.0
                };
                let right = if b < summary.frames {
                    summary.tail(b, summary.frames, stop as f64 - 1.0 / FACTOR as f64, false)
                } else {
                    0.0
                };
                records.push((start, local, left + right + allowance, a, b));
            }
            Ok(records)
        })?;
        let mut records = Vec::new();
        records
            .try_reserve_exact(blocks)
            .map_err(|_| "peak interval allocation failed")?;
        for (start, local, uncertainty, a, b) in parts.into_iter().flatten() {
            peak.local_grid_estimate = peak.local_grid_estimate.max(local);
            peak.grid_lower = peak.grid_lower.max((local - uncertainty).max(0.0));
            peak.continuous_upper = peak.continuous_upper.max(local + uncertainty);
            peak.examined_blocks += 1;
            records.push((start, local, uncertainty, a, b));
        }
        let grid_denominator = 1.0 - PI * PI / (8.0 * (FACTOR * FACTOR) as f64);
        // Refine only intervals that can prevent the 0.05 dB research-width
        // criterion. This threshold changes work, not the protection bound.
        let width_ratio = 10.0_f64.powf(0.05 / 20.0);
        if peak.continuous_upper / grid_denominator > peak.grid_lower * width_ratio {
            let tree = tail_moments::TailTree::new(source, channel, &cancelled)?;
            peak.moment_tree_bytes = tree.bytes();
            peak.continuous_upper = peak.sample_peak;
            for (start, mut value, mut uncertainty, a, b) in records {
                if (value + uncertainty) / grid_denominator > peak.grid_lower * width_ratio {
                    let (poly, remainder) = tree.polynomial(a, b, start);
                    value = workspace.grid(channel, start, Some(&poly), &cancelled)?.0;
                    uncertainty = remainder / PI + peak.numerical_allowance;
                    peak.grid_lower = peak.grid_lower.max((value - uncertainty).max(0.));
                    peak.refined_blocks += 1;
                }
                peak.continuous_upper = peak.continuous_upper.max(value + uncertainty);
            }
        }
        peak.continuous_upper /= grid_denominator;
        result.push(peak);
    }
    Ok(result)
}
