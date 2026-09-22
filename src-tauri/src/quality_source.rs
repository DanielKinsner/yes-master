//! Streaming counterpart of the qualified offline Rubato frame/drain loop.
//! Construction allocates; steady iteration and seek reuse fixed buffers.
//! Original playback uses it at the actual device rate. Mastered integration
//! additionally needs matching file-rate/gain plans.
use audioadapter_buffers::direct::InterleavedSlice;
use rodio::{source::SeekError, Source};
use rubato::{Fft, FixedSync, Resampler};
use std::{
    sync::{
        atomic::{AtomicU64, AtomicU8, Ordering},
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
    input_revision: Option<Arc<AtomicU64>>,
    output_revision: Arc<AtomicU64>,
    previous_input_revision: u64,
    buffered_revision: u64,
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
            input_revision: None,
            output_revision: Arc::new(AtomicU64::new(0)),
            previous_input_revision: 0,
            buffered_revision: 0,
        })
    }

    /// Retain this on the control thread before handing the source to a sink.
    /// Runtime failure publishes a code without formatting/allocating in audio.
    pub(crate) fn error_slot(&self) -> Arc<AtomicU8> {
        self.error_slot.clone()
    }

    /// Cascaded conversion stages share one failure latch. An inner failure
    /// must not become an apparently normal end-of-file in the outer stage.
    pub(crate) fn with_error_slot(mut self, slot: Arc<AtomicU8>) -> Self {
        self.error_slot = slot;
        self
    }

    /// Track the revision of actually emitted PCM, independently of worker
    /// completion. Zero means mixed/unknown. The upstream revision must only
    /// change from next()/seek(), with nonzero revisions increasing on updates.
    pub(crate) fn with_revision(mut self, upstream: Arc<AtomicU64>) -> Self {
        self.input_revision = Some(upstream);
        self
    }

    pub(crate) fn revision_slot(&self) -> Arc<AtomicU64> {
        self.output_revision.clone()
    }

    fn upstream_revision(&self) -> u64 {
        self.input_revision
            .as_ref()
            .map_or(0, |r| r.load(Ordering::Acquire))
    }

    fn fail(&mut self, error: StreamFailure) {
        self.failed = true;
        self.error_slot.store(error as u8, Ordering::Release);
        self.output_revision.store(0, Ordering::Release);
    }

    fn refill(&mut self) -> Result<bool, StreamFailure> {
        let revision_before = self.upstream_revision();
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
            // Identity consumes exactly one frame and has no filter history.
            self.buffered_revision = self.upstream_revision();
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
        let after = self.upstream_revision();
        let whole_input_revision = if revision_before == after { after } else { 0 };
        // Locked Rubato 1.0.1 FixedSync::Both with one subchunk produces each
        // FFT block from this block plus the preceding block's overlap only
        // (synchro.rs::FftResampler::resample_unit). Require both complete
        // input blocks to carry the same revision. Publish only on emission,
        // never when a nested converter merely reads ahead into its buffers.
        self.buffered_revision = if whole_input_revision == self.previous_input_revision {
            whole_input_revision
        } else {
            0
        };
        self.previous_input_revision = whole_input_revision;
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
        if self.output_revision.load(Ordering::Relaxed) != self.buffered_revision {
            self.output_revision
                .store(self.buffered_revision, Ordering::Release);
        }
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
        self.previous_input_revision = 0;
        self.buffered_revision = 0;
        self.output_revision.store(0, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RevisionStep {
        position: usize,
        rate: u32,
        revision: Arc<AtomicU64>,
    }
    impl Iterator for RevisionStep {
        type Item = f32;
        fn next(&mut self) -> Option<f32> {
            if self.position == self.rate as usize * 2 {
                return None;
            }
            let revision = if self.position < 3101 {
                1
            } else if self.position < 4507 {
                2
            } else {
                3
            };
            self.revision.store(revision, Ordering::Release);
            self.position += 1;
            // Any nonzero sample after revision 3 is stale buffered/history PCM.
            Some(if revision < 3 { 0.5 } else { 0.0 })
        }
    }
    impl Source for RevisionStep {
        fn current_frame_len(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> u16 {
            1
        }
        fn sample_rate(&self) -> u32 {
            self.rate
        }
        fn total_duration(&self) -> Option<Duration> {
            Some(Duration::from_secs(2))
        }
        fn try_seek(&mut self, _: Duration) -> Result<(), SeekError> {
            self.position = 0;
            self.revision.store(0, Ordering::Release);
            Ok(())
        }
    }

    #[test]
    fn revisions_follow_emitted_pcm_through_two_converter_buffers_and_seek() {
        for source_rate in [32000, 44100, 48000, 96000] {
            for file_rate in [44100, 48000, 96000] {
                for device_rate in [44100, 48000, 96000] {
                    let input_revision = Arc::new(AtomicU64::new(0));
                    let input = RevisionStep {
                        position: 0,
                        rate: source_rate,
                        revision: input_revision.clone(),
                    };
                    let file = QualitySource::new(input, file_rate)
                        .unwrap()
                        .with_revision(input_revision.clone());
                    let file_revision = file.revision_slot();
                    let mut output = QualitySource::new(file, device_rate)
                        .unwrap()
                        .with_revision(file_revision);
                    let revision = output.revision_slot();
                    for replay in 0..2 {
                        assert_eq!(revision.load(Ordering::Acquire), 0);
                        let mut first_input = None;
                        let mut first_output = None;
                        let mut count = 0;
                        for sample in output.by_ref() {
                            if input_revision.load(Ordering::Acquire) == 3 {
                                first_input.get_or_insert(count);
                            }
                            if revision.load(Ordering::Acquire) == 3 {
                                first_output.get_or_insert(count);
                                assert_eq!(sample, 0., "stale PCM marked applied: {source_rate}/{file_rate}/{device_rate}, replay {replay}");
                            } else {
                                assert!(
                                    first_output.is_none(),
                                    "revision must not regress after stable output"
                                );
                            }
                            count += 1;
                        }
                        assert_eq!(
                            count,
                            (source_rate as usize * 2 * file_rate as usize)
                                .div_ceil(source_rate as usize)
                                .saturating_mul(device_rate as usize)
                                .div_ceil(file_rate as usize)
                        );
                        let gap = first_output.unwrap() - first_input.unwrap();
                        if source_rate != file_rate || file_rate != device_rate {
                            assert!(gap > 0);
                        }
                        output.try_seek(Duration::ZERO).unwrap();
                    }
                }
            }
        }
    }

    #[test]
    fn cascade_retains_inner_processing_failure() {
        let inner = QualitySource::new(
            rodio::buffer::SamplesBuffer::new(1, 96000, vec![0.1, f32::NAN, 0.2]),
            44100,
        )
        .unwrap();
        let failure = inner.error_slot();
        let mut outer = QualitySource::new(inner, 48000)
            .unwrap()
            .with_error_slot(failure.clone());
        assert!(outer.next().is_none());
        assert_eq!(
            failure.load(Ordering::Acquire),
            StreamFailure::NonFinite as u8
        );
        // An outer-stage seek must not revive a failed inner converter.
        assert!(outer.try_seek(Duration::ZERO).is_err());
    }
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
