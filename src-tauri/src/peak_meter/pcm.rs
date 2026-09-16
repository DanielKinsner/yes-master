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
