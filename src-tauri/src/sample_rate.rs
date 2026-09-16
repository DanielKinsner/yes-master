use crate::types::{CommandError, CommandResult};

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};

const SRC_CHUNK_FRAMES: usize = 2048;

pub(crate) fn convert_interleaved(
    samples: &[f32],
    source_sample_rate: u32,
    target_sample_rate: u32,
    channels: u16,
) -> CommandResult<Vec<f32>> {
    if source_sample_rate == 0 || target_sample_rate == 0 {
        return Err(CommandError::Render(format!(
            "invalid sample-rate conversion: {source_sample_rate} Hz to {target_sample_rate} Hz"
        )));
    }
    if channels == 0 {
        return Err(CommandError::Render(
            "SRC requires at least one channel".into(),
        ));
    }
    let channel_count = usize::from(channels);
    if samples.len() % channel_count != 0 {
        return Err(CommandError::Render(format!(
            "interleaved sample count {} is not divisible by channel count {}",
            samples.len(),
            channel_count
        )));
    }

    // Validate identity conversions too. All production callers use decoded,
    // nonzero rates/channels; malformed layouts must not succeed only at unity.
    if source_sample_rate == target_sample_rate {
        return Ok(samples.to_vec());
    }

    let input_frames = samples.len() / channel_count;
    if input_frames == 0 {
        return Ok(Vec::new());
    }

    let input = InterleavedSlice::new(samples, channel_count, input_frames)
        .map_err(|e| CommandError::Render(format!("SRC input adapter: {e}")))?;
    let mut resampler = Fft::<f32>::new(
        source_sample_rate as usize,
        target_sample_rate as usize,
        SRC_CHUNK_FRAMES,
        1,
        channel_count,
        FixedSync::Both,
    )
    .map_err(|e| CommandError::Render(format!("SRC setup: {e}")))?;
    let overflow = || CommandError::Render("SRC output size overflow".into());
    let expected_frames = usize::try_from(
        (input_frames as u128 * u128::from(target_sample_rate))
            .div_ceil(u128::from(source_sample_rate)),
    )
    .map_err(|_| overflow())?;
    let delay = resampler.output_delay();
    let needed = expected_frames.checked_add(delay).ok_or_else(overflow)?;
    let output_capacity_frames = needed
        .checked_add(resampler.output_frames_max())
        .ok_or_else(overflow)?;
    let capacity = output_capacity_frames
        .checked_mul(channel_count)
        .ok_or_else(overflow)?;
    let mut output_samples = Vec::new();
    output_samples
        .try_reserve_exact(capacity)
        .map_err(|e| CommandError::Render(format!("SRC allocation: {e}")))?;
    output_samples.resize(capacity, 0.0_f32);
    let mut output =
        InterleavedSlice::new_mut(&mut output_samples, channel_count, output_capacity_frames)
            .map_err(|e| CommandError::Render(format!("SRC output adapter: {e}")))?;
    let mut consumed = 0;
    let mut written = 0;
    // Rubato 1.0.1's convenience loop copies the wrong suffix for odd output
    // blocks and may stop before a short input's useful output emerges. Keep
    // its FFT filter, explicitly zero-pad/drain, then remove the delay ONCE.
    while written < needed {
        let want = resampler.input_frames_next();
        let take = (input_frames - consumed).min(want);
        let indexing = Indexing {
            input_offset: consumed,
            output_offset: written,
            partial_len: (take < want).then_some(take),
            active_channels_mask: None,
        };
        let (used, produced) = resampler
            .process_into_buffer(&input, &mut output, Some(&indexing))
            .map_err(|e| CommandError::Render(format!("SRC process: {e}")))?;
        // Fft reports the full processed block, including implicit padding.
        if want == 0 || used != want || produced == 0 {
            return Err(CommandError::Render("SRC made invalid progress".into()));
        }
        consumed += take;
        written = written.checked_add(produced).ok_or_else(overflow)?;
    }
    if consumed != input_frames {
        return Err(CommandError::Render(
            "SRC did not consume the complete input".into(),
        ));
    }

    // The integer delay leaves the existing half-output-sample residual for
    // 48/96 -> 44.1 kHz. Fractional-delay compensation would change the filter.
    output_samples.copy_within(delay * channel_count..needed * channel_count, 0);
    output_samples.truncate(expected_frames * channel_count);
    Ok(output_samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(sample_rate: u32, seconds: f32, channels: u16) -> Vec<f32> {
        let frames = (sample_rate as f32 * seconds) as usize;
        let mut out = Vec::with_capacity(frames * usize::from(channels));
        for i in 0..frames {
            let t = i as f32 / sample_rate as f32;
            let sample = 0.25 * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
            for _ in 0..channels {
                out.push(sample);
            }
        }
        out
    }

    #[test]
    fn convert_interleaved_returns_copy_when_rate_matches() {
        let input = sine(48_000, 0.01, 2);
        let converted = convert_interleaved(&input, 48_000, 48_000, 2).expect("same-rate copy");
        assert_eq!(converted, input);
    }

    #[test]
    fn convert_interleaved_converts_44100_to_48000() {
        let input = sine(44_100, 0.1, 2);
        let converted = convert_interleaved(&input, 44_100, 48_000, 2).expect("SRC");
        let output_frames = converted.len() / 2;
        assert_eq!(
            output_frames, 4_800,
            "0.1 s at 48 kHz should render exactly 4800 stereo frames"
        );
        assert!(
            converted.iter().any(|sample| sample.abs() > 0.01),
            "converted sine should retain audible signal energy"
        );
    }

    #[test]
    fn identity_path_rejects_invalid_metadata_and_layout() {
        for (samples, rate, channels) in [
            (&[0.25][..], 0, 1),
            (&[0.25][..], 48_000, 0),
            (&[0.25][..], 48_000, 2),
        ] {
            assert!(convert_interleaved(samples, rate, rate, channels).is_err());
        }
    }

    #[test]
    fn exact_frame_and_channel_matrix() {
        // Custom/source-rate delivery accepts arbitrary positive rates. Cover
        // both common PCM families and the MPEG/AAC low-rate delivery families.
        let rates = [
            8_000, 11_025, 12_000, 16_000, 22_050, 24_000, 32_000, 44_100, 48_000, 88_200, 96_000,
            176_400, 192_000,
        ];
        for from in rates {
            for to in rates {
                let block = if from == to {
                    SRC_CHUNK_FRAMES
                } else {
                    Fft::<f32>::new(
                        from as usize,
                        to as usize,
                        SRC_CHUNK_FRAMES,
                        1,
                        2,
                        FixedSync::Both,
                    )
                    .unwrap()
                    .input_frames_next()
                };
                for frames in [
                    0,
                    1,
                    2,
                    17,
                    block - 1,
                    block,
                    block + 1,
                    2 * block - 1,
                    2 * block,
                    2 * block + 1,
                ] {
                    let mut input = vec![0.0; frames * 2];
                    if frames > 0 {
                        input[0] = 0.75;
                        input[2 * frames - 2] = 0.5;
                    }
                    let out = convert_interleaved(&input, from, to, 2).unwrap();
                    // Independent quotient/remainder oracle (no float or ceil).
                    let numerator = frames as u64 * u64::from(to);
                    let expected =
                        numerator / u64::from(from) + u64::from(numerator % u64::from(from) != 0);
                    assert_eq!(out.len() as u64, expected * 2, "{from}->{to}, {frames}");
                    assert!(out.iter().all(|x| x.is_finite()));
                    assert!(out.chunks_exact(2).all(|f| f[1] == 0.0), "channel leakage");
                    if frames > 0 {
                        assert!(
                            out.iter().any(|x| x.abs() > 1e-5),
                            "lost impulse {from}->{to}, {frames}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn finite_tail_matches_explicit_zero_extension() {
        for (from, to) in [
            (44_100, 48_000),
            (48_000, 44_100),
            (96_000, 44_100),
            (44_100, 96_000),
            (48_000, 96_000),
            (96_000, 48_000),
        ] {
            for frames in [1, 2, 31, 2047, 2048, 2049, 4097] {
                for dc in [false, true] {
                    let mut input = vec![if dc { 0.25 } else { 0.0 }; frames * 2];
                    input[2 * frames - 1] = -0.7;
                    let out = convert_interleaved(&input, from, to, 2).unwrap();
                    input.resize(input.len() + 16_384, 0.0);
                    let extended = convert_interleaved(&input, from, to, 2).unwrap();
                    assert_eq!(
                        out,
                        extended[..out.len()],
                        "finite edge {from}->{to}, {frames}"
                    );
                    assert!(out.chunks_exact(2).rev().take(8).any(|f| f[1].abs() > 1e-5));
                }
            }
        }
    }

    #[test]
    fn odd_output_blocks_have_no_stale_frame_and_retain_fractional_delay() {
        for (from, stale) in [(48_000, 955), (96_000, 514)] {
            let input: Vec<f32> = (0..from * 2)
                .map(|i| {
                    (0.8 * (std::f64::consts::TAU * 100.0 * f64::from(i) / f64::from(from)).sin())
                        as f32
                })
                .collect();
            let out = convert_interleaved(&input, from, 44_100, 1).unwrap();
            let expected = |i: usize| {
                (0.8 * (std::f64::consts::TAU * 100.0 * (i as f64 - 0.5) / 44_100.0).sin()) as f32
            };
            assert!(
                (out[stale] - expected(stale)).abs() < 1e-5,
                "stale frame {from}"
            );
            let max_error = (1000..out.len() - 1000)
                .map(|i| (out[i] - expected(i)).abs())
                .fold(0.0_f32, f32::max);
            assert!(
                max_error < 1e-5,
                "fixed-amplitude/phase residual {from}: {max_error}"
            );
        }
    }

    #[test]
    fn convert_interleaved_rejects_zero_source_rate() {
        let input = sine(44_100, 0.01, 2);
        let result = convert_interleaved(&input, 0, 48_000, 2);
        assert!(matches!(result, Err(CommandError::Render(_))));
    }

    #[test]
    fn convert_interleaved_rejects_zero_target_rate() {
        let input = sine(44_100, 0.01, 2);
        let result = convert_interleaved(&input, 44_100, 0, 2);
        assert!(matches!(result, Err(CommandError::Render(_))));
    }

    #[test]
    fn convert_interleaved_rejects_non_divisible_length() {
        // 3 samples cannot split evenly across 2 channels.
        let result = convert_interleaved(&[0.0, 0.0, 0.0], 44_100, 48_000, 2);
        assert!(matches!(result, Err(CommandError::Render(_))));
    }

    #[test]
    fn convert_interleaved_empty_input_is_empty_ok() {
        let converted = convert_interleaved(&[], 44_100, 48_000, 2).expect("empty SRC");
        assert!(converted.is_empty());
    }

    #[test]
    fn convert_interleaved_mono_stays_finite_and_lands_near_target_length() {
        let input = sine(44_100, 0.1, 1);
        let converted = convert_interleaved(&input, 44_100, 48_000, 1).expect("mono SRC");
        assert!(
            converted.iter().all(|s| s.is_finite()),
            "conversion must not introduce NaN/Inf"
        );
        assert_eq!(converted.len(), 4800);
    }
}
