//! Streaming counterpart of the qualified offline Rubato frame/drain loop.
//! Construction allocates; steady iteration and seek reuse fixed buffers.
//! Original playback uses it at the actual device rate. Mastered integration
//! additionally needs matching file-rate/gain plans.
use audioadapter_buffers::direct::InterleavedSlice;
use rodio::{source::SeekError, Source};
use rubato::{Fft, FixedSync, Resampler};
use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum StreamFailure {
    NonFinite = 1,
    PartialFrame = 2,
    LengthOverflow = 3,
    Configuration = 4,
    Processing = 5,
    NoProgress = 6,
}

/// Control-thread diagnostic; never formats or allocates in the audio callback.
pub(crate) fn failure_description(code: u8) -> &'static str {
    match code {
        1 => "nonfinite input",
        2 => "incomplete channel frame",
        3 => "frame-count overflow",
        4 => "invalid converter configuration",
        5 => "resampler processing error",
        6 => "resampler made no progress",
        _ => "unknown streaming conversion failure",
    }
}

pub(crate) struct QualitySource<S: Source<Item = f32>> {
    source: S,
    rate: u32,
    channels: usize,
    resampler: Option<Fft<f32>>,
    input: Vec<f32>,
    output: Vec<f32>,
    output_position: usize,
    output_length: usize,
    delay_left: usize,
    input_frames: usize,
    emitted_samples: usize,
    expected_samples: Option<usize>,
    ended: bool,
    failed: bool,
    error_slot: Arc<AtomicU8>,
}

impl<S: Source<Item = f32>> QualitySource<S> {
    pub(crate) fn new(source: S, rate: u32) -> Result<Self, String> {
        let channels = usize::from(source.channels());
        let source_rate = source.sample_rate();
        if rate == 0 || source_rate == 0 || channels == 0 {
            return Err("invalid streaming SRC format".into());
        }
        let resampler = if rate == source_rate {
            None
        } else {
            Some(
                Fft::<f32>::new(
                    source_rate as usize,
                    rate as usize,
                    2048,
                    1,
                    channels,
                    FixedSync::Both,
                )
                .map_err(|e| e.to_string())?,
            )
        };
        let delay_left = resampler.as_ref().map_or(0, Resampler::output_delay);
        let input = vec![0.; resampler.as_ref().map_or(0, Resampler::input_frames_max) * channels];
        let output =
            vec![0.; resampler.as_ref().map_or(1, Resampler::output_frames_max) * channels];
        Ok(Self {
            source,
            rate,
            channels,
            resampler,
            input,
            output,
            output_position: 0,
            output_length: 0,
            delay_left,
            input_frames: 0,
            emitted_samples: 0,
            expected_samples: None,
            ended: false,
            failed: false,
            error_slot: Arc::new(AtomicU8::new(0)),
        })
    }

    /// Retain this on the control thread before handing the source to a sink.
    /// Runtime failure publishes a code without formatting/allocating in audio.
    pub(crate) fn error_slot(&self) -> Arc<AtomicU8> {
        self.error_slot.clone()
    }

    fn fail(&mut self, error: StreamFailure) {
        self.failed = true;
        self.error_slot.store(error as u8, Ordering::Release);
    }

    fn refill(&mut self) -> Result<bool, StreamFailure> {
        if self.resampler.is_none() {
            // Pull complete frames even at an unchanged rate. A sample-wise
            // passthrough leaves some Source implementations mid-channel when
            // seek is requested, changing the channel layout of the suffix.
            for index in 0..self.channels {
                match self.source.next() {
                    Some(value) if value.is_finite() => self.output[index] = value,
                    Some(_) => return Err(StreamFailure::NonFinite),
                    None if index == 0 => return Ok(false),
                    None => return Err(StreamFailure::PartialFrame),
                }
            }
            self.output_position = 0;
            self.output_length = self.channels;
            return Ok(true);
        }
        if self
            .expected_samples
            .is_some_and(|n| self.emitted_samples >= n)
        {
            return Ok(false);
        }
        let resampler = self
            .resampler
            .as_mut()
            .ok_or(StreamFailure::Configuration)?;
        let want = resampler.input_frames_next();
        self.input.fill(0.);
        let mut read = 0;
        if !self.ended {
            for index in 0..want * self.channels {
                match self.source.next() {
                    Some(value) if value.is_finite() => {
                        self.input[index] = value;
                        read += 1;
                    }
                    Some(_) => return Err(StreamFailure::NonFinite),
                    None => {
                        self.ended = true;
                        break;
                    }
                }
            }
            if read % self.channels != 0 {
                return Err(StreamFailure::PartialFrame);
            }
            self.input_frames += read / self.channels;
            if self.ended {
                let frames = (self.input_frames as u128 * u128::from(self.rate))
                    .div_ceil(u128::from(self.source.sample_rate()));
                self.expected_samples = Some(
                    usize::try_from(frames)
                        .map_err(|_| StreamFailure::LengthOverflow)?
                        .checked_mul(self.channels)
                        .ok_or(StreamFailure::LengthOverflow)?,
                );
                if self.expected_samples == Some(self.emitted_samples) {
                    return Ok(false);
                }
            }
        }
        let input = InterleavedSlice::new(&self.input, self.channels, want)
            .map_err(|_| StreamFailure::Configuration)?;
        let mut output = InterleavedSlice::new_mut(
            &mut self.output,
            self.channels,
            resampler.output_frames_max(),
        )
        .map_err(|_| StreamFailure::Configuration)?;
        let (used, produced) = resampler
            .process_into_buffer(&input, &mut output, None)
            .map_err(|_| StreamFailure::Processing)?;
        if used != want || produced == 0 {
            return Err(StreamFailure::NoProgress);
        }
        let skip = self.delay_left.min(produced);
        self.delay_left -= skip;
        self.output_position = skip * self.channels;
        let available = (produced - skip) * self.channels;
        self.output_length = self.output_position
            + self.expected_samples.map_or(available, |expected| {
                available.min(expected - self.emitted_samples)
            });
        Ok(true)
    }

    #[cfg(test)]
    fn allocations(&self) -> (usize, usize, usize, usize) {
        (
            self.input.as_ptr() as usize,
            self.input.capacity(),
            self.output.as_ptr() as usize,
            self.output.capacity(),
        )
    }
}

impl<S: Source<Item = f32>> Iterator for QualitySource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.failed {
            return None;
        }
        while self.output_position == self.output_length {
            match self.refill() {
                Ok(true) => {}
                Ok(false) => return None,
                Err(error) => {
                    self.fail(error);
                    return None;
                }
            }
        }
        let sample = self.output[self.output_position];
        self.output_position += 1;
        self.emitted_samples += 1;
        Some(sample)
    }
}

impl<S: Source<Item = f32>> Source for QualitySource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels as u16
    }
    fn sample_rate(&self) -> u32 {
        self.rate
    }
    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
    fn try_seek(&mut self, position: Duration) -> Result<(), SeekError> {
        if self.failed {
            // Malformed input can also leave the underlying Source mid-frame.
            // Keep the error visible and require reconstruction after failure.
            return Err(SeekError::NotSupported {
                underlying_source: "failed streaming source",
            });
        }
        self.source.try_seek(position)?;
        if let Some(resampler) = &mut self.resampler {
            resampler.reset();
            self.delay_left = resampler.output_delay();
        }
        self.output_position = 0;
        self.output_length = 0;
        self.input_frames = 0;
        self.emitted_samples = 0;
        self.expected_samples = None;
        self.ended = false;
        self.failed = false;
        self.error_slot.store(0, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    #[test]
    fn conversion_and_identity_failures_are_visible_outside_the_source() {
        for rate in [44100, 48000] {
            for (input, error, description) in [
                (
                    vec![0.1, f32::NAN],
                    StreamFailure::NonFinite,
                    "nonfinite input",
                ),
                (
                    vec![0.1, 0.2, 0.3],
                    StreamFailure::PartialFrame,
                    "incomplete channel frame",
                ),
            ] {
                let mut source =
                    QualitySource::new(SamplesBuffer::new(2, 44100, input), rate).unwrap();
                let slot = source.error_slot();
                for sample in source.by_ref() {
                    assert!(sample.is_finite());
                }
                assert_eq!(slot.load(Ordering::Acquire), error as u8);
                assert_eq!(
                    failure_description(slot.load(Ordering::Acquire)),
                    description
                );
                assert!(source.next().is_none());
                assert!(source.try_seek(Duration::ZERO).is_err());
                assert_eq!(slot.load(Ordering::Acquire), error as u8);
            }
        }
    }

    #[test]
    fn streaming_matches_whole_file_at_edges_and_preserves_buffers() {
        for from in [32000, 44100, 48000, 96000] {
            for to in [44100, 48000, 96000] {
                for channels in [1, 2] {
                    for frames in [0, 1, 17, 2047, 2048, 2049, 11027] {
                        let mut input = vec![0.; frames * channels];
                        for (i, value) in input.iter_mut().enumerate() {
                            *value = 0.4 * ((i * 17 % 97) as f32 / 48. - 1.);
                        }
                        let expected = crate::sample_rate::convert_interleaved(
                            &input,
                            from,
                            to,
                            channels as u16,
                        )
                        .unwrap();
                        let mut source = QualitySource::new(
                            SamplesBuffer::new(channels as u16, from, input),
                            to,
                        )
                        .unwrap();
                        let pointers = source.allocations();
                        let actual: Vec<f32> = source.by_ref().collect();
                        assert!(!source.failed);
                        assert_eq!(source.allocations(), pointers);
                        assert_eq!(
                            actual, expected,
                            "{from}->{to}, {channels}ch, {frames}frames"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn seek_discards_old_filter_state_and_keeps_duration() {
        let input: Vec<_> = (0..96000).map(|i| 0.3 * (i as f32 * 0.11).sin()).collect();
        let mut source =
            QualitySource::new(SamplesBuffer::new(2, 48000, input.clone()), 44100).unwrap();
        assert_eq!(source.total_duration(), Some(Duration::from_secs(1)));
        for _ in 0..997 {
            source.next();
        }
        source.try_seek(Duration::from_millis(250)).unwrap();
        let actual: Vec<_> = source.collect();
        let expected =
            crate::sample_rate::convert_interleaved(&input[24000..], 48000, 44100, 2).unwrap();
        assert_eq!(actual, expected);
    }
}
