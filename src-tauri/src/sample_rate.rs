use crate::types::{CommandError, CommandResult};

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

const SRC_CHUNK_FRAMES: usize = 2048;

pub(crate) fn convert_interleaved(
    samples: &[f32],
    source_sample_rate: u32,
    target_sample_rate: u32,
    channels: u16,
) -> CommandResult<Vec<f32>> {
    if source_sample_rate == target_sample_rate {
        return Ok(samples.to_vec());
    }
    if source_sample_rate == 0 || target_sample_rate == 0 {
        return Err(CommandError::Render(format!(
            "invalid sample-rate conversion: {source_sample_rate} Hz to {target_sample_rate} Hz"
        )));
    }
    let channel_count = usize::from(channels.max(1));
    if samples.len() % channel_count != 0 {
        return Err(CommandError::Render(format!(
            "interleaved sample count {} is not divisible by channel count {}",
            samples.len(),
            channel_count
        )));
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
    let output_capacity_frames = resampler.process_all_needed_output_len(input_frames);
    let mut output_samples = vec![0.0_f32; output_capacity_frames * channel_count];
    let mut output =
        InterleavedSlice::new_mut(&mut output_samples, channel_count, output_capacity_frames)
            .map_err(|e| CommandError::Render(format!("SRC output adapter: {e}")))?;
    let (_input_used, output_frames) = resampler
        .process_all_into_buffer(&input, &mut output, input_frames, None)
        .map_err(|e| CommandError::Render(format!("SRC process: {e}")))?;

    output_samples.truncate(output_frames * channel_count);
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
}
