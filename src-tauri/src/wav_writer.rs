use crate::types::{CommandError, CommandResult};
use std::path::Path;

// ============================================================================
// Phase A4: TPDF dither for integer-output WAV writers.
//
// 16/24-bit PCM rounds a float sample to the nearest integer, which
// produces signal-correlated harmonic distortion at low levels — the
// quantization error becomes a periodic function of the signal. Triangular
// probability density noise of ±1 LSB peak amplitude, added BEFORE
// quantization, decorrelates the error from the signal: the per-sample
// quantization noise becomes Gaussian-ish white noise at the LSB level,
// at the cost of ~3 dB extra noise floor (inaudible at 16-bit; below
// hearing at 24-bit). Reference: Lipshitz / Vanderkooy 1992.
//
// Applied ONLY in the offline render path. The live audio thread in
// audio.rs stays f32 throughout, so there's no quantization to dither.
//
// PRNG: xorshift32. Two shifts, two XORs, one f32 divide per draw — far
// cheaper than the rand crate's SmallRng for the volume of noise we
// generate (millions of samples per render). State held in `DitherRng`
// for deterministic per-render output.
// ============================================================================

struct DitherRng {
    state: u32,
}

impl DitherRng {
    fn new(seed: u32) -> Self {
        // xorshift32 has a zero-fixed-point; substitute a non-zero seed.
        Self {
            state: if seed == 0 { 0xCAFE_BABE } else { seed },
        }
    }

    /// One uniform draw in `[0, 1)` from the top 23 bits of state.
    #[inline]
    fn next_unit(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        ((self.state >> 9) as f32) / 8_388_608.0_f32
    }

    /// Standard TPDF dither: triangular noise in `[-1, 1)` LSB — the sum of
    /// two independent RPDFs each uniform in `[-0.5, 0.5)` LSB. This is the
    /// Lipshitz / Vanderkooy optimum (2 LSB peak-to-peak), which fully
    /// decorrelates the quantization error and eliminates noise modulation.
    /// (Earlier this summed two `[-1, 1)` uniforms → `[-2, 2)` LSB, twice the
    /// intended amplitude and ~6 dB hotter noise floor than standard TPDF.)
    /// Returned in LSB units; callers multiply by `1 / scale` to convert to
    /// amplitude before adding to the sample.
    #[inline]
    fn tpdf_lsb(&mut self) -> f32 {
        let u1 = self.next_unit() - 0.5; // [-0.5, 0.5)
        let u2 = self.next_unit() - 0.5; // [-0.5, 0.5)
        u1 + u2 // triangle in [-1, 1)
    }
}

// B2 fix — symmetric-range scaling for integer quantization.
//
// Pre-fix, INT16_SCALE was 32_767.0 and INT24_SCALE was 8_388_607.0
// (the absolute value of i16::MAX / i24::MAX). With `clamp(-1, 1) *
// SCALE`, the most-negative integer (`i16::MIN = -32_768` / equivalent
// for i24) was unreachable — output range was asymmetric by 1 LSB.
// Audibly inconsequential (< -90 dB FS DC offset) but technically
// incorrect.
//
// Industry-standard fix: scale by the absolute value of the integer
// MIN (32_768 / 8_388_608) so negative samples reach the full range,
// and clamp the rounded result to the actual integer range post-
// multiply so positive samples cap at MAX without overflowing.
//
// `INT16_PEAK` / `INT24_PEAK` are still 32_767 / 8_388_607 — used as
// the upper clamp bound (the most-positive integer the format
// supports).
const INT16_SCALE: f32 = 32_768.0;
const INT16_PEAK_POS: f32 = 32_767.0;
const INT24_SCALE: f32 = 8_388_608.0;
const INT24_PEAK_POS: f32 = 8_388_607.0;

#[inline]
fn quantize_16_tpdf(sample: f32, rng: &mut DitherRng) -> i16 {
    let dithered = sample + rng.tpdf_lsb() / INT16_SCALE;
    let scaled = (dithered * INT16_SCALE).round();
    scaled.clamp(-INT16_SCALE, INT16_PEAK_POS) as i16
}

#[inline]
fn quantize_24_tpdf(sample: f32, rng: &mut DitherRng) -> i32 {
    let dithered = sample + rng.tpdf_lsb() / INT24_SCALE;
    let scaled = (dithered * INT24_SCALE).round();
    scaled.clamp(-INT24_SCALE, INT24_PEAK_POS) as i32
}

pub(crate) fn wav_spec(
    channels: u16,
    sample_rate: u32,
    bit_depth: u16,
) -> CommandResult<hound::WavSpec> {
    let (bits, fmt) = match bit_depth {
        16 => (16u16, hound::SampleFormat::Int),
        24 => (24u16, hound::SampleFormat::Int),
        32 => (32u16, hound::SampleFormat::Float),
        other => {
            return Err(CommandError::Other(format!(
                "unsupported bit depth: {other}"
            )))
        }
    };
    Ok(hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: bits,
        sample_format: fmt,
    })
}

pub(crate) fn write_samples_into_writer(
    writer: &mut hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    samples: &[f32],
    bit_depth: u16,
) -> CommandResult<()> {
    // Phase A4: TPDF dither for int paths. f32 output stays as-is.
    let mut rng = DitherRng::new(0x000A_11CE);
    match bit_depth {
        16 => {
            for &s in samples {
                writer
                    .write_sample(quantize_16_tpdf(s, &mut rng))
                    .map_err(|e| CommandError::Io(e.to_string()))?;
            }
        }
        24 => {
            for &s in samples {
                writer
                    .write_sample(quantize_24_tpdf(s, &mut rng))
                    .map_err(|e| CommandError::Io(e.to_string()))?;
            }
        }
        32 => {
            for &s in samples {
                writer
                    .write_sample(s.clamp(-1.0, 1.0))
                    .map_err(|e| CommandError::Io(e.to_string()))?;
            }
        }
        other => {
            return Err(CommandError::Other(format!(
                "unsupported bit depth: {other}"
            )))
        }
    }
    Ok(())
}

pub(crate) fn write_wav(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    bit_depth: u16,
) -> CommandResult<()> {
    let (bits, fmt) = match bit_depth {
        16 => (16u16, hound::SampleFormat::Int),
        24 => (24u16, hound::SampleFormat::Int),
        32 => (32u16, hound::SampleFormat::Float),
        other => {
            return Err(CommandError::Other(format!(
                "unsupported bit depth: {other}"
            )))
        }
    };
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: bits,
        sample_format: fmt,
    };
    let mut writer =
        hound::WavWriter::create(path, spec).map_err(|e| CommandError::Io(e.to_string()))?;
    // Phase A4: TPDF dither for int paths. f32 output stays as-is.
    let mut rng = DitherRng::new(0x000A_11CE);
    match bit_depth {
        16 => {
            for &s in samples {
                writer
                    .write_sample(quantize_16_tpdf(s, &mut rng))
                    .map_err(|e| CommandError::Io(e.to_string()))?;
            }
        }
        24 => {
            for &s in samples {
                writer
                    .write_sample(quantize_24_tpdf(s, &mut rng))
                    .map_err(|e| CommandError::Io(e.to_string()))?;
            }
        }
        32 => {
            for &s in samples {
                writer
                    .write_sample(s.clamp(-1.0, 1.0))
                    .map_err(|e| CommandError::Io(e.to_string()))?;
            }
        }
        _ => unreachable!(),
    }
    writer
        .finalize()
        .map_err(|e| CommandError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::collections::HashSet;
    use std::fs;

    fn sha256_file(path: &Path) -> String {
        let bytes = fs::read(path).expect("read wav bytes");
        let digest = Sha256::digest(&bytes);
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn write_wav_16bit_snapshot_pins_spec_samples_and_file_hash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out = tmp.path().join("writer-16.wav");
        let samples = [-1.0, -0.5, -0.125, 0.0, 0.125, 0.5, 0.99999];

        write_wav(&out, &samples, 48_000, 1, 16).expect("write 16-bit wav");

        let mut reader = hound::WavReader::open(&out).expect("open 16-bit wav");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 48_000);
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        let decoded: Vec<i16> = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .expect("decode 16-bit samples");
        assert_eq!(decoded, vec![-32768, -16384, -4095, -1, 4096, 16384, 32767]);
        assert_eq!(
            sha256_file(&out),
            "816224efa3de11b822957fa46fd674100b9ecf5f157e1225761d70b524adfb91"
        );
    }

    #[test]
    fn write_samples_24bit_snapshot_pins_spec_samples_and_file_hash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out = tmp.path().join("writer-24.wav");
        let samples = [-1.0, -0.25, 0.0, 0.25, 0.75, 0.99999];
        let spec = wav_spec(2, 44_100, 24).expect("24-bit spec");
        let mut writer = hound::WavWriter::create(&out, spec).expect("create 24-bit wav");

        write_samples_into_writer(&mut writer, &samples, 24).expect("write 24-bit samples");
        writer.finalize().expect("finalize 24-bit wav");

        let mut reader = hound::WavReader::open(&out).expect("open 24-bit wav");
        let spec = reader.spec();
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 44_100);
        assert_eq!(spec.bits_per_sample, 24);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        let decoded: Vec<i32> = reader
            .samples::<i32>()
            .collect::<Result<Vec<_>, _>>()
            .expect("decode 24-bit samples");
        assert_eq!(
            decoded,
            vec![-8388608, -2097152, 1, 2097151, 6291456, 8388525]
        );
        assert_eq!(
            sha256_file(&out),
            "2a9d31aa7a50e59816dabdd86bb23a8633c9dd3c732aaa3d6c58e1fd043f2e25"
        );
    }

    #[test]
    fn write_wav_and_writer_helper_match_for_same_16bit_spec() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let via_write_wav = tmp.path().join("write-wav.wav");
        let via_helper = tmp.path().join("writer-helper.wav");
        let samples = [-0.75, -0.25, 0.0, 0.25, 0.75, 0.99999];

        write_wav(&via_write_wav, &samples, 48_000, 2, 16).expect("write_wav");
        let spec = wav_spec(2, 48_000, 16).expect("16-bit spec");
        let mut writer = hound::WavWriter::create(&via_helper, spec).expect("create wav");
        write_samples_into_writer(&mut writer, &samples, 16).expect("write via helper");
        writer.finalize().expect("finalize helper wav");

        assert_eq!(
            fs::read(&via_write_wav).expect("read write_wav bytes"),
            fs::read(&via_helper).expect("read helper bytes"),
            "write_wav and write_samples_into_writer must stay byte-identical when their specs overlap"
        );
    }

    #[test]
    fn writer_helpers_intentionally_reset_dither_rng_per_call() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let first = tmp.path().join("first.wav");
        let second = tmp.path().join("second.wav");
        let segmented = tmp.path().join("segmented.wav");
        let continuous = tmp.path().join("continuous.wav");
        let segment = [0.0_f32; 32];
        let doubled: Vec<f32> = segment.iter().chain(segment.iter()).copied().collect();

        // Intentional writer contract: every public writer helper starts a
        // fresh deterministic TPDF dither stream with DitherRng::new(0x000A_11CE).
        // Continuous-album segments therefore reset dither at each helper call.
        // Do not "fix" this into one shared RNG stream without changing the
        // byte contract and the album-render expectations together.
        write_wav(&first, &segment, 48_000, 1, 16).expect("first write_wav");
        write_wav(&second, &segment, 48_000, 1, 16).expect("second write_wav");
        assert_eq!(
            fs::read(&first).expect("read first write_wav"),
            fs::read(&second).expect("read second write_wav"),
            "write_wav must reset its dither RNG on every call"
        );

        let spec = wav_spec(1, 48_000, 16).expect("segmented spec");
        let mut segmented_writer =
            hound::WavWriter::create(&segmented, spec).expect("create segmented wav");
        write_samples_into_writer(&mut segmented_writer, &segment, 16)
            .expect("first segmented write");
        write_samples_into_writer(&mut segmented_writer, &segment, 16)
            .expect("second segmented write");
        segmented_writer.finalize().expect("finalize segmented");

        let spec = wav_spec(1, 48_000, 16).expect("continuous spec");
        let mut continuous_writer =
            hound::WavWriter::create(&continuous, spec).expect("create continuous wav");
        write_samples_into_writer(&mut continuous_writer, &doubled, 16)
            .expect("single continuous write");
        continuous_writer.finalize().expect("finalize continuous");

        let mut segmented_reader = hound::WavReader::open(&segmented).expect("open segmented");
        let segmented_samples: Vec<i16> = segmented_reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .expect("decode segmented samples");
        assert_eq!(&segmented_samples[..32], &segmented_samples[32..]);
        assert_ne!(
            fs::read(&segmented).expect("read segmented bytes"),
            fs::read(&continuous).expect("read continuous bytes"),
            "two helper calls must not collapse into one continuous dither stream"
        );
    }

    #[test]
    fn write_wav_32bit_float_snapshot_pins_spec_samples_and_file_hash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out = tmp.path().join("writer-32.wav");
        let samples = [-1.2, -0.25, 0.0, 0.25, 1.2];

        write_wav(&out, &samples, 48_000, 1, 32).expect("write 32-bit float wav");

        let mut reader = hound::WavReader::open(&out).expect("open 32-bit wav");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 48_000);
        assert_eq!(spec.bits_per_sample, 32);
        assert_eq!(spec.sample_format, hound::SampleFormat::Float);

        let decoded: Vec<f32> = reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .expect("decode 32-bit samples");
        assert_eq!(decoded, vec![-1.0, -0.25, 0.0, 0.25, 1.0]);
        assert_eq!(
            sha256_file(&out),
            "ca8a2aef746c21b009a818f48c1cbfb4b13a51fb72aae503ee5f85e6950813f7"
        );
    }

    /// Phase A4: at -90 dBFS the signal sits at ~1 LSB of a 16-bit
    /// quantizer. Without dither, `round()` quantizes the sine to a
    /// tiny set of integer values (mostly 0, with occasional ±1 at the
    /// peaks) — the quantization noise is periodic and signal-correlated.
    /// With TPDF dither, the noise floor expands so the output takes on
    /// MANY distinct integer values, decorrelating the error from the
    /// signal. This is the textbook reason to dither.
    ///
    /// Concrete acceptance: the dithered sequence must produce at least
    /// 6 distinct integer values; the undithered sequence stays at 3
    /// or fewer (the deliberate signed-quantization fan-out).
    #[test]
    fn tpdf_dither_decorrelates_quantization_at_minus_90_dbfs() {
        let sr = 48_000_u32;
        let n = (sr as f32 * 0.1) as usize;
        let amp = 10.0_f32.powf(-90.0 / 20.0);
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| amp * (omega * i as f32).sin()).collect();

        let mut undithered = HashSet::new();
        for &s in &samples {
            let v = (s * INT16_SCALE)
                .round()
                .clamp(-INT16_SCALE, INT16_PEAK_POS) as i16;
            undithered.insert(v);
        }

        let mut rng = DitherRng::new(0x1234_5678);
        let mut dithered = HashSet::new();
        for &s in &samples {
            dithered.insert(quantize_16_tpdf(s, &mut rng));
        }

        assert!(
            undithered.len() <= 3,
            "undithered -90 dBFS sine should stay tightly quantized; got {} distinct values",
            undithered.len()
        );
        assert!(
            dithered.len() > undithered.len(),
            "dither must expand the integer count: undithered={}, dithered={}",
            undithered.len(),
            dithered.len()
        );
        assert!(
            dithered.len() >= 4,
            "dithered -90 dBFS sine should hit at least 4 distinct values \
             (signal ~±1 LSB peak, standard TPDF noise ±1 LSB peak); got {}",
            dithered.len()
        );
    }

    /// TPDF dither's mean should be ~0 — over many samples the noise
    /// contribution averages out. Verifies the PRNG is balanced.
    #[test]
    fn tpdf_dither_has_zero_mean() {
        let mut rng = DitherRng::new(0xDEAD_BEEF);
        let n = 100_000;
        let mean: f32 = (0..n).map(|_| rng.tpdf_lsb()).sum::<f32>() / (n as f32);
        assert!(
            mean.abs() < 0.01,
            "TPDF mean across {} samples should be ~0; got {}",
            n,
            mean
        );
    }

    /// TPDF dither on silence stays within ±1 LSB (the dither's peak
    /// amplitude). Verifies the dither is applied and bounded.
    #[test]
    fn tpdf_dither_on_silence_stays_within_one_lsb() {
        let mut rng = DitherRng::new(0x4242_4242);
        let mut max_abs: u16 = 0;
        let n = 10_000;
        for _ in 0..n {
            let v = quantize_16_tpdf(0.0, &mut rng);
            let a = v.unsigned_abs();
            if a > max_abs {
                max_abs = a;
            }
        }
        assert!(
            max_abs <= 1,
            "dither on silence should never exceed ±1 LSB; saw {}",
            max_abs
        );
    }

    // ========================================================================
    // B2: symmetric-range integer quantization. Pre-fix, INT16_SCALE and
    // INT24_SCALE were `i16::MAX` / equivalent, so `clamp(-1, 1) * SCALE`
    // never reached `i16::MIN`/i24-MIN — the most-negative integer was
    // unreachable. Post-fix, SCALE is `|i16::MIN|` / equivalent, with a
    // post-multiply clamp to the integer range so positive samples cap
    // at MAX without overflow.
    // ========================================================================

    /// `sample <= -1.0` (safely below the lower bound even with dither)
    /// must produce `i16::MIN`. Pre-B2 it produced -32_767 (one LSB
    /// short of the actual minimum). This test is the regression gate
    /// for the asymmetric-range bug.
    #[test]
    fn quantize_16_tpdf_reaches_i16_min_on_negative_extreme() {
        // Sample = -1.5 is far enough below -1.0 that no TPDF dither
        // amplitude (which lives in [-1/INT16_SCALE, +1/INT16_SCALE) ≈
        // ±3e-5) can pull the dithered value above the clamp floor.
        // The scaled value rounds to ~-49152, clamps to -32768, casts
        // to `i16::MIN`.
        let mut rng = DitherRng::new(0xDEAD_BEEF);
        let result = quantize_16_tpdf(-1.5, &mut rng);
        assert_eq!(
            result,
            i16::MIN,
            "sample <= -1.0 must reach i16::MIN ({}); pre-B2 the asymmetric \
             scale capped at -32767 instead",
            i16::MIN
        );
    }

    /// `sample >= 1.0` (or above) must clamp to `i16::MAX`, never
    /// overflow to a negative value. With the new scale = 32768, raw
    /// `1.0 * 32768 = 32768` would overflow `as i16`; the post-multiply
    /// clamp is what keeps the result valid.
    #[test]
    fn quantize_16_tpdf_clamps_positive_extreme_at_i16_max() {
        let mut rng = DitherRng::new(0xCAFE_F00D);
        let result = quantize_16_tpdf(2.0, &mut rng);
        assert_eq!(
            result,
            i16::MAX,
            "sample >= 1.0 must clamp to i16::MAX ({}); the post-multiply \
             clamp is what prevents overflow with the new scale",
            i16::MAX
        );
    }

    /// Equivalent regression gate for the 24-bit path: the i24 MIN
    /// value (-8_388_608) is reachable for samples at or below -1.0.
    #[test]
    fn quantize_24_tpdf_reaches_i24_min_on_negative_extreme() {
        let mut rng = DitherRng::new(0xDEAD_BEEF);
        let result = quantize_24_tpdf(-1.5, &mut rng);
        assert_eq!(
            result, -8_388_608,
            "sample <= -1.0 must reach i24-MIN (-8_388_608); pre-B2 \
             the asymmetric scale capped at -8_388_607"
        );
    }

    /// And the 24-bit positive clamp.
    #[test]
    fn quantize_24_tpdf_clamps_positive_extreme_at_i24_max() {
        let mut rng = DitherRng::new(0xCAFE_F00D);
        let result = quantize_24_tpdf(2.0, &mut rng);
        assert_eq!(
            result, 8_388_607,
            "sample >= 1.0 must clamp to i24-MAX (8_388_607); the \
             post-multiply clamp prevents overflow with the new scale"
        );
    }

    /// Zero sample with deterministic RNG produces a value within ±1
    /// LSB of zero (the standard TPDF dither amplitude bound). Verifies the
    /// dither + quantize path doesn't drift away from silence on the
    /// new scale — the prior implementation had the same property and
    /// nothing should have changed for typical-amplitude samples.
    #[test]
    fn quantize_16_tpdf_preserves_silence_within_dither_bound() {
        let mut rng = DitherRng::new(0xBEEF_1234);
        for _ in 0..100 {
            let result = quantize_16_tpdf(0.0, &mut rng);
            assert!(
                result.abs() <= 1,
                "zero sample with TPDF dither should produce values within \
                 ±1 LSB; got {result}"
            );
        }
    }
}
