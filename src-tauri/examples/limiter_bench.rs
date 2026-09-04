//! Deterministic limiter throughput diagnostic; not a wall-clock CI assertion.
use std::time::Instant;
use yes_master_lib::dsp::Limiter;

fn main() {
    for sr in [44100, 96000, 192000] {
        for amp in [0.1, 1.5, 4.0] {
            let input: Vec<f32> = (0..sr * 10)
                .map(|i| amp * (std::f32::consts::TAU * 997.0 * i as f32 / sr as f32).sin())
                .collect();
            let mut limiter = Limiter::new(sr, 2, -1.0, 3.0, 50.0);
            let start = Instant::now();
            for sample in input {
                let mut frame = [sample, sample * 0.8];
                limiter.process_frame_inplace(&mut frame);
                std::hint::black_box(frame);
            }
            println!(
                "{sr} Hz amplitude {amp}: {:.2}x realtime",
                10.0 / start.elapsed().as_secs_f64()
            );
        }
    }
}
