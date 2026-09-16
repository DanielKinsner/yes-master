//! B2 offline candidate/library comparison. No production detector replacement.
//! Args: list.json fresh-output.json. List entries: {"id":...,"path":...}.
#[path = "research/finite_peak.rs"]
mod finite_peak;
#[path = "research/reconstruction_fir.rs"]
mod reconstruction_fir;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() == 3 || args.len() == 4);
    let setup = Instant::now();
    let fir = args.get(3).map(|path| {
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() % 8 == 0);
        let kernel: Vec<_> = bytes
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
            .collect();
        reconstruction_fir::ReconstructionFir::new(&kernel)
    });
    let fir_preparation_s = setup.elapsed().as_secs_f64();
    let entries: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let out = Path::new(&args[2]);
    assert!(!out.exists(), "preserve completed evidence");
    let mut rows = Vec::new();
    for entry in entries {
        let path = Path::new(entry["path"].as_str().unwrap());
        let pcm = yes_master_lib::decode::decode_full(path).unwrap();
        let start = Instant::now();
        let measured =
            finite_peak::measure(&pcm.samples, usize::from(pcm.channels), || false).unwrap();
        let seconds = start.elapsed().as_secs_f64();
        let start = Instant::now();
        let fir_peaks = fir.as_ref().map(|f| {
            f.measure(&pcm.samples, usize::from(pcm.channels), || false)
                .unwrap()
        });
        let fir_seconds = start.elapsed().as_secs_f64();
        let mut library = Vec::new();
        for chunk_frames in [1, 257, 4096] {
            let mut ebu = ebur128::EbuR128::new(
                u32::from(pcm.channels),
                pcm.sample_rate,
                ebur128::Mode::TRUE_PEAK,
            )
            .unwrap();
            let start = Instant::now();
            for chunk in pcm.samples.chunks(chunk_frames * usize::from(pcm.channels)) {
                ebu.add_frames_f32(chunk).unwrap();
            }
            let before: Vec<_> = (0..u32::from(pcm.channels))
                .map(|c| ebu.true_peak(c).unwrap())
                .collect();
            ebu.add_frames_f32(&vec![0.; 256 * usize::from(pcm.channels)])
                .unwrap();
            let flushed: Vec<_> = (0..u32::from(pcm.channels))
                .map(|c| ebu.true_peak(c).unwrap())
                .collect();
            library.push(json!({"chunk_frames":chunk_frames,"unflushed":before,"flushed":flushed,"seconds":start.elapsed().as_secs_f64()}));
        }
        rows.push(json!({"id":entry["id"],"source_sha256":format!("{:x}",Sha256::digest(std::fs::read(path).unwrap())),
            "rate":pcm.sample_rate,"frames":pcm.samples.len()/usize::from(pcm.channels),"channels":pcm.channels,
            "candidate":measured,"candidate_seconds":seconds,"fir_grid":fir_peaks,
            "fir_seconds":fir_seconds,"fir_preparation_s":fir_preparation_s,"ebur128_0_1_10":library}));
        std::fs::write(
            out,
            serde_json::to_vec_pretty(&json!({"version":finite_peak::VERSION,"rows":rows}))
                .unwrap(),
        )
        .unwrap();
        println!("{}: {seconds:.3} s", entry["id"]);
    }
}

#[cfg(test)]
mod tests {
    use super::finite_peak;
    use std::f64::consts::PI;

    fn sinc_at(x: &[f32], t: f64) -> f64 {
        x.iter()
            .enumerate()
            .map(|(i, x)| {
                let phase = PI * (t - i as f64);
                f64::from(*x)
                    * if phase.abs() < 1e-14 {
                        1.0
                    } else {
                        phase.sin() / phase
                    }
            })
            .sum::<f64>()
            .abs()
    }

    #[test]
    fn finite_edges_and_interior_are_contained_by_independent_direct_sums() {
        for n in [1, 2, 17, 4095, 4096, 4097, 10001] {
            for kind in 0..4 {
                let mut state = 19_u32;
                let x: Vec<_> = (0..n)
                    .map(|i| match kind {
                        0 => 0.5,
                        1 => {
                            if i % 2 == 0 {
                                0.5
                            } else {
                                -0.5
                            }
                        }
                        2 => {
                            if i == n - 1 {
                                1.4
                            } else {
                                0.0
                            }
                        }
                        _ => {
                            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                            (state as f64 / u32::MAX as f64 * 2.8 - 1.4) as f32
                        }
                    })
                    .collect();
                let got = finite_peak::measure(&x, 1, || false).unwrap();
                let got = &got[0];
                assert!(got.continuous_upper >= got.sample_peak);
                // Includes exterior ringout, phases BETWEEN the meter's grid,
                // and both sides of its internal core boundary.
                for centre in [0, n / 2, n - 1, 4096.min(n)] {
                    for step in -64..=64 {
                        let t = centre as f64 + f64::from(step) / 32.0;
                        let expected = sinc_at(&x, t);
                        assert!(
                            expected <= got.continuous_upper + 1e-10,
                            "n={n} kind={kind} t={t}: {expected} > {}",
                            got.continuous_upper
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn channels_silence_validation_and_cancellation_are_independent() {
        let x = [0., 0.5, 0., -0.75, 0., 0.25];
        let got = finite_peak::measure(&x, 2, || false).unwrap();
        assert_eq!(got[0].continuous_upper, 0.);
        assert!(got[1].continuous_upper >= 0.75);
        let mono = finite_peak::measure(&[0.5, -0.75, 0.25], 1, || false).unwrap();
        assert_eq!(got[1].continuous_upper, mono[0].continuous_upper);
        assert_eq!(
            finite_peak::measure(&[], 1, || false).unwrap()[0].continuous_upper,
            0.
        );
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(finite_peak::measure(&[bad], 1, || false).is_err());
        }
        assert!(finite_peak::measure(&[0.], 0, || false).is_err());
        assert!(finite_peak::measure(&[0.], 2, || false).is_err());
        assert_eq!(
            finite_peak::measure(&[0.], 1, || true).unwrap_err(),
            "cancelled"
        );
    }

    #[test]
    fn coherent_tail_refinement_is_tight_and_zero_prefix_invariant() {
        let x: Vec<_> = (0..262_144)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let peak = finite_peak::measure(&x, 1, || false).unwrap().remove(0);
        assert!(peak.refined_blocks > 0);
        assert!(20. * (peak.continuous_upper / peak.grid_lower).log10() < 0.05);
        // Independent direct sum from the retained 1024x near-edge probe.
        assert!(peak.grid_lower <= 2.3135816417269344);
        assert!(peak.continuous_upper >= 2.3135816417269344);
        let mut padded = vec![0.; 257];
        padded.extend_from_slice(&x);
        padded.extend_from_slice(&[0.; 193]);
        let shifted = finite_peak::measure(&padded, 1, || false)
            .unwrap()
            .remove(0);
        assert!(shifted.grid_lower <= 2.3135816417269344);
        assert!(shifted.continuous_upper >= 2.3135816417269344);
        assert!((20. * (shifted.continuous_upper / peak.continuous_upper).log10()).abs() < 0.002);
    }

    #[test]
    fn cancellation_interrupts_preparation_and_interpolation() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let x = vec![0.2; 40_000];
        for limit in [2, 14, 35] {
            let calls = AtomicUsize::new(0);
            assert_eq!(
                finite_peak::measure(&x, 1, || calls.fetch_add(1, Ordering::Relaxed) >= limit)
                    .unwrap_err(),
                "cancelled"
            );
            assert_eq!(calls.load(Ordering::Relaxed), limit + 1);
        }
    }
}
