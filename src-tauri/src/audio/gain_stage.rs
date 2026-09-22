//! Revision-matched audition scalars after the file/device converters.
use rodio::{source::SeekError, Source};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GainPlan {
    pub revision: u64,
    pub raw_revision: u64,
    pub landing: f32,
    pub volume_match: f32,
}

/// Single control-thread writer, audio-thread reader. Odd/even sequence protects
/// the pair of gains and raw revision as one publication. A busy reader skips a
/// publication attempt instead of spinning or allocating on the callback.
pub(super) struct GainMailbox {
    sequence: AtomicU64,
    raw_revision: AtomicU64,
    gains: AtomicU64,
}

impl GainMailbox {
    pub fn new(plan: GainPlan) -> Arc<Self> {
        let result = Arc::new(Self {
            sequence: AtomicU64::new(0),
            raw_revision: AtomicU64::new(0),
            gains: AtomicU64::new(0),
        });
        result.publish(plan);
        result
    }

    pub fn publish(&self, plan: GainPlan) {
        assert!(plan.revision > 0 && plan.revision < u64::MAX / 2);
        assert!(plan.raw_revision > 0 && plan.landing.is_finite() && plan.landing >= 0.);
        assert!(plan.volume_match.is_finite() && (0. ..=1.).contains(&plan.volume_match));
        self.sequence.store(plan.revision * 2 + 1, Ordering::SeqCst);
        self.raw_revision.store(plan.raw_revision, Ordering::SeqCst);
        self.gains.store(
            u64::from(plan.landing.to_bits()) | (u64::from(plan.volume_match.to_bits()) << 32),
            Ordering::SeqCst,
        );
        self.sequence.store(plan.revision * 2, Ordering::SeqCst);
    }

    fn read(&self) -> Option<GainPlan> {
        let first = self.sequence.load(Ordering::SeqCst);
        if first == 0 || first & 1 != 0 {
            return None;
        }
        let raw_revision = self.raw_revision.load(Ordering::SeqCst);
        let gains = self.gains.load(Ordering::SeqCst);
        if self.sequence.load(Ordering::SeqCst) != first {
            return None;
        }
        Some(GainPlan {
            revision: first / 2,
            raw_revision,
            landing: f32::from_bits(gains as u32),
            volume_match: f32::from_bits((gains >> 32) as u32),
        })
    }
}

pub(super) struct GainSource<S: Source<Item = f32>> {
    source: S,
    mailbox: Arc<GainMailbox>,
    input_revision: Arc<AtomicU64>,
    output_revision: Arc<AtomicU64>,
    plan: GainPlan,
    landing: f32,
    volume_match: f32,
    ramp_from: (f32, f32),
    ramp_to: (f32, f32),
    ramp_left: usize,
    ramp_frames: usize,
    frame: Vec<f32>,
    position: usize,
}

impl<S: Source<Item = f32>> GainSource<S> {
    pub fn new(source: S, input_revision: Arc<AtomicU64>, mailbox: Arc<GainMailbox>) -> Self {
        let plan = mailbox.read().expect("initial gain plan");
        let channels = usize::from(source.channels());
        // Match the existing source's 512-frame/48k settings transition duration.
        let ramp_frames = (source.sample_rate() as usize * 512 / 48000).max(1);
        Self {
            source,
            mailbox,
            input_revision,
            output_revision: Arc::new(AtomicU64::new(0)),
            plan,
            landing: plan.landing,
            volume_match: plan.volume_match,
            ramp_from: (plan.landing, plan.volume_match),
            ramp_to: (plan.landing, plan.volume_match),
            ramp_left: 0,
            ramp_frames,
            frame: vec![0.; channels],
            position: channels,
        }
    }

    pub fn revision_slot(&self) -> Arc<AtomicU64> {
        self.output_revision.clone()
    }

    fn ramp(&mut self, target: (f32, f32)) {
        if self.ramp_to != target {
            self.ramp_from = (self.landing, self.volume_match);
            self.ramp_to = target;
            self.ramp_left = self.ramp_frames;
        }
    }
}

impl<S: Source<Item = f32>> Iterator for GainSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.position == self.frame.len() {
            for value in &mut self.frame {
                *value = self.source.next()?;
            }
            if let Some(plan) = self.mailbox.read() {
                if plan.revision > self.plan.revision {
                    self.plan = plan;
                    // Drop unmeasured boosts during a different raw response.
                    // Existing attenuation survives until its matched plan arrives.
                    if self.input_revision.load(Ordering::Acquire) != plan.raw_revision {
                        self.ramp((
                            self.landing.min(plan.landing).min(1.),
                            self.volume_match.min(plan.volume_match),
                        ));
                    }
                }
            }
            let matched = self.input_revision.load(Ordering::Acquire) == self.plan.raw_revision;
            if matched {
                self.ramp((self.plan.landing, self.plan.volume_match));
            }
            if self.ramp_left > 0 {
                if self.ramp_left == 1 {
                    (self.landing, self.volume_match) = self.ramp_to;
                } else {
                    let t =
                        (self.ramp_frames - self.ramp_left + 1) as f32 / self.ramp_frames as f32;
                    self.landing = self.ramp_from.0 + (self.ramp_to.0 - self.ramp_from.0) * t;
                    // Interpolate the audible total gain. Ramping both factors
                    // independently creates a bump when they compensate each
                    // other. Keep a valid factor pair for interruptions/pending
                    // attenuation, and preserve the settled f32 multiply order.
                    let from = f64::from(self.ramp_from.0) * f64::from(self.ramp_from.1);
                    let to = f64::from(self.ramp_to.0) * f64::from(self.ramp_to.1);
                    let total = from + (to - from) * f64::from(t);
                    self.volume_match = if self.landing > 0. {
                        ((total / f64::from(self.landing)) as f32).clamp(0., 1.)
                    } else {
                        self.ramp_to.1
                    };
                }
                self.ramp_left -= 1;
            }
            for value in &mut self.frame {
                *value *= self.landing;
                *value *= self.volume_match;
            }
            let revision = if matched
                && self.ramp_left == 0
                && self.ramp_to == (self.plan.landing, self.plan.volume_match)
            {
                self.plan.revision
            } else {
                0
            };
            if self.output_revision.load(Ordering::Relaxed) != revision {
                self.output_revision.store(revision, Ordering::Release);
            }
            self.position = 0;
        }
        let value = self.frame[self.position];
        self.position += 1;
        Some(value)
    }
}

impl<S: Source<Item = f32>> Source for GainSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.source.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
    fn try_seek(&mut self, position: Duration) -> Result<(), SeekError> {
        self.source.try_seek(position)?;
        self.position = self.frame.len();
        self.output_revision.store(0, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compensating_device_and_volume_match_edits_preserve_combined_level() {
        let mut rows = Vec::new();
        for rate in [44100, 48000, 96000] {
            for interrupt in [false, true] {
                let mailbox = GainMailbox::new(GainPlan {
                    revision: 1,
                    raw_revision: 1,
                    landing: 0.5,
                    volume_match: 1.,
                });
                let mut source = GainSource::new(
                    rodio::buffer::SamplesBuffer::new(2, rate, vec![0.2; rate as usize]),
                    Arc::new(AtomicU64::new(1)),
                    mailbox.clone(),
                );
                assert_eq!(source.next(), Some(0.1));
                assert_eq!(source.next(), Some(0.1));
                mailbox.publish(GainPlan {
                    revision: 2,
                    raw_revision: 1,
                    landing: 1.,
                    volume_match: 0.5,
                });
                let mut maximum_delta_db = 0_f64;
                for frame in 0..source.ramp_frames * 2 {
                    if interrupt && frame == source.ramp_frames / 4 {
                        mailbox.publish(GainPlan {
                            revision: 3,
                            raw_revision: 1,
                            landing: 0.625,
                            volume_match: 0.8,
                        });
                    }
                    let left = source.next().unwrap();
                    assert_eq!(source.next(), Some(left));
                    let delta = (20. * (f64::from(left) / f64::from(0.1_f32)).log10()).abs();
                    maximum_delta_db = maximum_delta_db.max(delta);
                }
                assert_eq!(
                    source.revision_slot().load(Ordering::Acquire),
                    if interrupt { 3 } else { 2 }
                );
                rows.push((rate, interrupt, maximum_delta_db));
            }
        }
        eprintln!(
            "Equal combined-gain transitions (rate, interrupted, maximum dB delta): {rows:?}"
        );
        assert!(
            rows.iter().all(|row| row.2 <= 0.01),
            "equal endpoints must retain the existing 0.01 dB transition budget: {rows:?}"
        );
    }

    #[test]
    fn gain_ramps_through_zero_remain_finite_monotonic_and_allocation_free() {
        for (from, to) in [
            ((0., 1.), (1., 0.5)),
            ((1., 0.), (0.5, 1.)),
            ((1., 0.5), (0., 1.)),
            ((0.5, 1.), (1., 0.)),
        ] {
            let mailbox = GainMailbox::new(GainPlan {
                revision: 1,
                raw_revision: 1,
                landing: from.0,
                volume_match: from.1,
            });
            let mut source = GainSource::new(
                rodio::buffer::SamplesBuffer::new(2, 48000, vec![0.2; 4096]),
                Arc::new(AtomicU64::new(1)),
                mailbox.clone(),
            );
            let mut previous = source.next().unwrap();
            assert_eq!(source.next(), Some(previous));
            mailbox.publish(GainPlan {
                revision: 2,
                raw_revision: 1,
                landing: to.0,
                volume_match: to.1,
            });
            let final_value = (0.2 * to.0) * to.1;
            let increasing = final_value > previous;
            let (allocations, ()) = crate::test_allocations::count(|| {
                for _ in 0..source.ramp_frames {
                    let value = source.next().unwrap();
                    assert!(value.is_finite());
                    assert_eq!(source.next(), Some(value));
                    if increasing {
                        assert!(value >= previous && value <= final_value);
                    } else {
                        assert!(value <= previous && value >= final_value);
                    }
                    previous = value;
                }
                assert_eq!(previous, final_value);
                assert_eq!(source.revision_slot().load(Ordering::Acquire), 2);
            });
            assert_eq!(allocations, 0);
        }
    }

    #[test]
    fn concurrent_publication_never_pairs_different_gains_or_raw_revisions() {
        let mailbox = GainMailbox::new(GainPlan {
            revision: 1,
            raw_revision: 1,
            landing: 1.,
            volume_match: 0.5,
        });
        let writer = mailbox.clone();
        let done = std::thread::spawn(move || {
            for revision in 2..=100_000 {
                writer.publish(GainPlan {
                    revision,
                    raw_revision: revision,
                    landing: revision as f32,
                    volume_match: 0.5,
                });
            }
        });
        while !done.is_finished() {
            if let Some(plan) = mailbox.read() {
                assert_eq!(plan.revision, plan.raw_revision);
                assert_eq!(plan.landing, plan.revision as f32);
                assert_eq!(plan.volume_match, 0.5);
            }
        }
        done.join().unwrap();
        assert_eq!(mailbox.read().unwrap().revision, 100_000);
    }

    #[test]
    fn gain_updates_and_seek_allocate_nothing_and_never_skip_small_attenuation() {
        let landing = f32::from_bits(1_f32.to_bits() - 1);
        let mailbox = GainMailbox::new(GainPlan {
            revision: 1,
            raw_revision: 1,
            landing,
            volume_match: 1.,
        });
        let input = Arc::new(AtomicU64::new(1));
        let mut source = GainSource::new(
            rodio::buffer::SamplesBuffer::new(2, 48000, vec![0.9; 48000]),
            input,
            mailbox.clone(),
        );
        let (allocations, ()) = crate::test_allocations::count(|| {
            assert_eq!(source.next(), Some(0.9 * landing));
            mailbox.publish(GainPlan {
                revision: 2,
                raw_revision: 1,
                landing: 0.7,
                volume_match: 0.37,
            });
            for _ in 0..4095 {
                source.next();
            }
            source.try_seek(Duration::from_millis(100)).unwrap();
            assert_eq!(source.next(), Some((0.9 * 0.7) * 0.37));
            assert_eq!(source.revision_slot().load(Ordering::Acquire), 2);
        });
        assert_eq!(allocations, 0);
    }

    #[test]
    fn gain_waits_for_matching_raw_revision_and_cached_gain_edits_need_no_new_raw_pcm() {
        let initial = GainPlan {
            revision: 1,
            raw_revision: 1,
            landing: 0.5,
            volume_match: 1.,
        };
        let mailbox = GainMailbox::new(initial);
        let input = Arc::new(AtomicU64::new(1));
        let mut source = GainSource::new(
            rodio::buffer::SamplesBuffer::new(2, 48000, vec![0.2; 48000]),
            input.clone(),
            mailbox.clone(),
        );
        let applied = source.revision_slot();
        assert_eq!(source.next(), Some(0.1));
        source.next();
        assert_eq!(applied.load(Ordering::Acquire), 1);
        mailbox.publish(GainPlan {
            revision: 2,
            raw_revision: 2,
            landing: 3.,
            volume_match: 1.,
        });
        assert!(source.by_ref().take(4096).all(|sample| sample == 0.1));
        assert_eq!(applied.load(Ordering::Acquire), 0);
        input.store(2, Ordering::Release);
        for _ in 0..511 {
            source.next();
            source.next();
            assert_eq!(applied.load(Ordering::Acquire), 0);
        }
        assert_eq!(source.next(), Some(0.2 * 3.));
        source.next();
        assert_eq!(applied.load(Ordering::Acquire), 2);
        mailbox.publish(GainPlan {
            revision: 3,
            raw_revision: 2,
            landing: 1.,
            volume_match: 0.37,
        });
        for _ in 0..512 {
            source.next();
            source.next();
        }
        assert_eq!(applied.load(Ordering::Acquire), 3);
        assert_eq!(source.next(), Some((0.2 * 1.) * 0.37));
    }
}
