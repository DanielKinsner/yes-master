//! Replayable bounded reads: permits exact deterministic quantization without
//! retaining a second whole-track buffer. Implementations must return the same
//! samples on repeated reads and preserve channel/frame identity.

pub trait PcmSource {
    fn frames(&self) -> usize;
    fn channels(&self) -> usize;
    /// Fill a nonempty or empty in-range span in one channel. Validate finite
    /// values and return errors rather than substituting silence.
    fn read_channel(
        &self,
        channel: usize,
        start: usize,
        out: &mut [f64],
    ) -> Result<(), &'static str>;
}

/// Random-access exact PCM from a privately staged WAV. The caller owns the
/// file and must keep its contents immutable until this reader is dropped.
/// Scratch storage and the read buffer do not grow with programme duration.
pub struct WavPcm {
    reader: std::cell::RefCell<hound::WavReader<std::io::BufReader<std::fs::File>>>,
    frames: usize,
    spec: hound::WavSpec,
}

impl WavPcm {
    pub fn open(path: &std::path::Path) -> Result<Self, &'static str> {
        let file = std::fs::File::open(path).map_err(|_| "cannot open staged PCM")?;
        let reader = hound::WavReader::new(std::io::BufReader::with_capacity(1 << 20, file))
            .map_err(|_| "cannot read staged WAV")?;
        let spec = reader.spec();
        if spec.channels == 0
            || reader.len() % u32::from(spec.channels) != 0
            || !matches!(
                (spec.sample_format, spec.bits_per_sample),
                (hound::SampleFormat::Int, 16 | 24) | (hound::SampleFormat::Float, 32)
            )
        {
            return Err("invalid staged PCM format");
        }
        Ok(Self {
            frames: reader.duration() as usize,
            reader: std::cell::RefCell::new(reader),
            spec,
        })
    }
}

impl PcmSource for WavPcm {
    fn frames(&self) -> usize {
        self.frames
    }
    fn channels(&self) -> usize {
        usize::from(self.spec.channels)
    }
    fn read_channel(
        &self,
        channel: usize,
        start: usize,
        out: &mut [f64],
    ) -> Result<(), &'static str> {
        if channel >= self.channels() || start > self.frames || out.len() > self.frames - start {
            return Err("PCM read out of range");
        }
        if out.is_empty() {
            return Ok(());
        }
        let mut reader = self
            .reader
            .try_borrow_mut()
            .map_err(|_| "PCM reader busy")?;
        reader.seek(start as u32).map_err(|_| "PCM seek failed")?;
        let count = self.channels();
        match self.spec.sample_format {
            hound::SampleFormat::Float => {
                let mut values = reader.samples::<f32>();
                for destination in out {
                    for c in 0..count {
                        let value = values
                            .next()
                            .ok_or("truncated PCM")?
                            .map_err(|_| "invalid PCM sample")?;
                        if !value.is_finite() {
                            return Err("non-finite PCM");
                        }
                        if c == channel {
                            *destination = f64::from(value);
                        }
                    }
                }
            }
            hound::SampleFormat::Int => {
                let divisor = if self.spec.bits_per_sample == 16 {
                    32768.
                } else {
                    8388608.
                };
                let mut values = reader.samples::<i32>();
                for destination in out {
                    for c in 0..count {
                        let value = values
                            .next()
                            .ok_or("truncated PCM")?
                            .map_err(|_| "invalid PCM sample")?;
                        if c == channel {
                            *destination = f64::from(value) / divisor;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct InterleavedPcm<'a> {
    samples: &'a [f32],
    channels: usize,
}

impl<'a> InterleavedPcm<'a> {
    pub fn new(samples: &'a [f32], channels: usize) -> Result<Self, &'static str> {
        if channels == 0 || samples.len() % channels != 0 {
            return Err("invalid PCM layout");
        }
        Ok(Self { samples, channels })
    }
}

impl PcmSource for InterleavedPcm<'_> {
    fn frames(&self) -> usize {
        self.samples.len() / self.channels
    }
    fn channels(&self) -> usize {
        self.channels
    }
    fn read_channel(
        &self,
        channel: usize,
        start: usize,
        out: &mut [f64],
    ) -> Result<(), &'static str> {
        if channel >= self.channels || start > self.frames() || out.len() > self.frames() - start {
            return Err("PCM read out of range");
        }
        for (i, value) in out.iter_mut().enumerate() {
            let sample = self.samples[(start + i) * self.channels + channel];
            if !sample.is_finite() {
                return Err("non-finite PCM");
            }
            *value = f64::from(sample);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_wav_random_reads_match_exact_quantizer_in_every_channel() {
        let dir = tempfile::tempdir().unwrap();
        for channels in [1, 2] {
            let samples: Vec<f32> = (0..8193 * usize::from(channels))
                .map(|i| (i as f32 * 0.29).sin() * 0.9)
                .collect();
            for bits in [16, 24, 32] {
                let path = crate::wav_writer::write_wav(
                    &dir.path().join(format!("{channels}-{bits}.wav")),
                    &samples,
                    48000,
                    channels,
                    bits,
                )
                .unwrap();
                let file = WavPcm::open(&path).unwrap();
                let expected =
                    crate::wav_writer::DeliveryPcm::new(&samples, channels, bits, || false)
                        .unwrap();
                for start in [8192, 0, 4095, 17, 4096] {
                    let n = 129.min(file.frames() - start);
                    for channel in 0..usize::from(channels) {
                        let mut actual = vec![0.; n];
                        let mut reference = vec![0.; n];
                        file.read_channel(channel, start, &mut actual).unwrap();
                        expected
                            .read_channel(channel, start, &mut reference)
                            .unwrap();
                        assert_eq!(
                            actual, reference,
                            "{channels}/{bits} start={start} channel={channel}"
                        );
                    }
                }
                assert!(file.read_channel(0, 8193, &mut []).is_ok());
                assert!(file.read_channel(0, 8193, &mut [0.]).is_err());
                assert!(file
                    .read_channel(usize::from(channels), 0, &mut [0.])
                    .is_err());
            }
        }
    }
}
