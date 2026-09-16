//! Signed far-tail polynomial and an explicit geometric remainder.
//! Independently implemented counterpart of sinc_tail_moments.py.
use std::sync::OnceLock;
pub const ORDER: usize = 20;
pub type Polynomial = [f64; ORDER + 1];

fn binomial() -> &'static [[f64; ORDER + 1]; ORDER + 1] {
    static TABLE: OnceLock<[[f64; ORDER + 1]; ORDER + 1]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [[0.; ORDER + 1]; ORDER + 1];
        for n in 0..=ORDER {
            table[n][0] = 1.;
            table[n][n] = 1.;
            for k in 1..n {
                table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
            }
        }
        table
    })
}

struct Node {
    start: usize,
    stop: usize,
    centre: f64,
    radius: f64,
    l1: f64,
    moments: Polynomial,
    children: Vec<Node>,
}

impl Node {
    fn build(
        samples: &[f32],
        channels: usize,
        channel: usize,
        start: usize,
        span: usize,
        cancelled: &impl Fn() -> bool,
    ) -> Result<Self, &'static str> {
        if cancelled() {
            return Err("cancelled");
        }
        let stop = (samples.len() / channels).min(start + span);
        let mut node = Self {
            start,
            stop,
            centre: (start + span / 2) as f64,
            radius: (span / 2) as f64,
            l1: 0.,
            moments: [0.; ORDER + 1],
            children: Vec::new(),
        };
        if span == super::CORE {
            for frame in start..stop {
                let x = f64::from(samples[frame * channels + channel]);
                let y = if frame % 2 == 0 { x } else { -x };
                let v = (frame as f64 - node.centre) / node.radius;
                let mut power = 1.;
                for moment in &mut node.moments {
                    *moment += y * power;
                    power *= v;
                }
                node.l1 += x.abs();
            }
        } else {
            for a in [start, start + span / 2] {
                if a < samples.len() / channels {
                    node.children.push(Self::build(
                        samples,
                        channels,
                        channel,
                        a,
                        span / 2,
                        cancelled,
                    )?);
                }
            }
            for child in &node.children {
                let shift = (child.centre - node.centre) / node.radius;
                let scale = child.radius / node.radius;
                for k in 0..=ORDER {
                    for m in 0..=k {
                        node.moments[k] += binomial()[k][m]
                            * shift.powi((k - m) as i32)
                            * scale.powi(m as i32)
                            * child.moments[m];
                    }
                }
                node.l1 += child.l1;
            }
        }
        Ok(node)
    }

    fn add_far(
        &self,
        a: usize,
        b: usize,
        centre: f64,
        radius: f64,
        polynomial: &mut Polynomial,
    ) -> f64 {
        if self.l1 == 0. || (self.start >= a && self.stop <= b) {
            return 0.;
        }
        let d = centre - self.centre;
        let q = (radius + self.radius) / d.abs();
        if (self.stop <= a || self.start >= b) && q <= 0.5 {
            let mut source_powers = [1.; ORDER + 1];
            for m in 1..=ORDER {
                source_powers[m] = source_powers[m - 1] * self.radius / d;
            }
            let mut target_power = 1.;
            for (j, coefficient) in polynomial.iter_mut().enumerate() {
                let mut sum = 0.;
                for (m, power) in source_powers.iter().enumerate().take(ORDER + 1 - j) {
                    sum += binomial()[m + j][j] * power * self.moments[m];
                }
                *coefficient += target_power / d * sum;
                target_power *= -radius / d;
            }
            return self.l1 / d.abs() * q.powi((ORDER + 1) as i32) / (1. - q);
        }
        assert!(
            !self.children.is_empty(),
            "local span must cover complete source blocks"
        );
        self.children
            .iter()
            .map(|child| child.add_far(a, b, centre, radius, polynomial))
            .sum()
    }

    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.children.iter().map(Self::bytes).sum::<usize>()
    }
}

pub struct TailTree(Node);

impl TailTree {
    pub fn new(
        samples: &[f32],
        channels: usize,
        channel: usize,
        cancelled: &impl Fn() -> bool,
    ) -> Result<Self, &'static str> {
        let frames = samples.len() / channels;
        let span = frames.max(super::CORE).next_power_of_two();
        Ok(Self(Node::build(
            samples, channels, channel, 0, span, cancelled,
        )?))
    }

    pub fn polynomial(&self, a: usize, b: usize, start: i64) -> (Polynomial, f64) {
        let mut coefficients = [0.; ORDER + 1];
        let remainder = self.0.add_far(
            a,
            b,
            start as f64 + super::CORE as f64 / 2.,
            super::CORE as f64 / 2.,
            &mut coefficients,
        );
        (coefficients, remainder)
    }

    pub fn bytes(&self) -> usize {
        self.0.bytes()
    }
}

pub fn evaluate(coefficients: &Polynomial, at: f64) -> f64 {
    coefficients
        .iter()
        .rev()
        .fold(0., |result, c| result * at + c)
}
