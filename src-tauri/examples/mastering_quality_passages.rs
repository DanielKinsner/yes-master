//! Compare contextual excerpts with retained continuous C0 renders. Research only.
//! Args: source.wav candidate-directory fresh-output-directory
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;
use yes_master_lib::types;
use yes_master_lib::{decode, dsp::MasteringChain, types::*};
#[path = "../src/sample_rate.rs"]
mod sample_rate;

fn measure(x: &[f32], channels: u16) -> (f64, f64) {
    let mut m = EbuR128::new(channels.into(), 48_000, Mode::I | Mode::TRUE_PEAK).unwrap();
    m.add_frames_f32(x).unwrap();
    let mut peak = x.iter().map(|v| f64::from(v.abs())).fold(0.0, f64::max);
    for c in 0..u32::from(channels) {
        peak = peak.max(m.true_peak(c).unwrap());
    }
    (m.loudness_global().unwrap(), 20.0 * peak.max(1e-30).log10())
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4);
    let input = Path::new(&args[1]);
    let run = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    assert!(!out.exists());
    std::fs::create_dir_all(out).unwrap();
    let pcm = decode::decode_full(input).unwrap();
    let ch = usize::from(pcm.channels);
    let frames = pcm.samples.len() / ch;
    let rate = pcm.sample_rate as usize;
    let source: Value =
        serde_json::from_slice(&std::fs::read(run.join("report.json")).unwrap()).unwrap();
    let settings: MasteringSettings = serde_json::from_value(source["settings"].clone()).unwrap();
    // Version 1's quiet minimum could select an inactive tail. Use a fresh map
    // from the existing native analyzer, then its ordered windows only.
    let analysis = yes_master_lib::engine::analyze_tracks_core_with_progress_sync(
        vec![yes_master_lib::engine::AnalyzeRequest {
            id: TrackId("passage".into()),
            path: input.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )
    .unwrap()
    .remove(0);
    let deep = analysis.deep_analysis.unwrap();
    use yes_master_lib::deep_analysis::{MAX_SCAN_WINDOWS, SHORT_HOP, SHORT_WINDOW};
    let hop = if frames.saturating_sub(SHORT_WINDOW) / SHORT_HOP + 1 > MAX_SCAN_WINDOWS {
        (frames - SHORT_WINDOW) / (MAX_SCAN_WINDOWS - 1)
    } else {
        SHORT_HOP
    };
    let pick = |score: fn(&yes_master_lib::deep_analysis::WindowMetrics) -> f32, high: bool| {
        deep.windows
            .iter()
            .enumerate()
            .filter(|(_, w)| {
                w.loudness_key.is_finite() && w.loudness_key > -40.0 && score(w).is_finite()
            })
            .max_by(|(_, a), (_, b)| {
                if high {
                    score(a).total_cmp(&score(b))
                } else {
                    score(b).total_cmp(&score(a))
                }
            })
            .map(|(i, _)| i * hop)
            .unwrap_or(0)
    };
    let windows = [
        ("loud", pick(|w| w.loudness_key, true)),
        ("quiet_active", pick(|w| w.loudness_key, false)),
        ("attack", pick(|w| w.crest, true)),
        ("high_frequency", pick(|w| w.harsh_31 + w.sibilant_31, true)),
        ("start", 0),
        ("tail", frames.saturating_sub(3 * rate)),
    ];
    let mut a = rate;
    let mut b = 48000;
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    let quantum = rate / a;
    let mut rows = Vec::new();
    for id in ["control", "t14-single-0", "t9-single-0"] {
        let candidate = source["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap();
        let drive = candidate["metrics"]["extra_drive_db"].as_f64().unwrap() as f32;
        let full = decode::decode_full(&run.join(format!("raw-{id}.wav"))).unwrap();
        for (label, start) in windows {
            let stop = (start + 3 * rate).min(frames);
            let full_a = start * 48000 / rate;
            let full_b = stop * 48000 / rate;
            let reference = &full.samples[full_a * ch..full_b * ch];
            let measured_reference = measure(reference, pcm.channels);
            for seconds in [0, 2, 10] {
                // Align SRC phase to the full-file rational sample grid.
                let left = start.saturating_sub(seconds * rate) / quantum * quantum;
                let right = (stop + rate).min(frames);
                let before = Instant::now();
                let mut x = pcm.samples[left * ch..right * ch].to_vec();
                let mut chain = MasteringChain::new(pcm.sample_rate, ch, &settings);
                chain.coeffs.input_gain_lin *= 10.0_f32.powf(drive / 20.0);
                for block in x.chunks_mut(8192 * ch) {
                    chain.process_interleaved(block, ch);
                }
                chain.flush_render_tail(&mut x, ch);
                let x = sample_rate::convert_interleaved(&x, pcm.sample_rate, 48000, pcm.channels)
                    .unwrap();
                let local_a = (start - left) * 48000 / rate;
                let local_b = (stop - left) * 48000 / rate;
                let segment = &x[local_a * ch..local_b * ch];
                assert_eq!(segment.len(), reference.len());
                let m = measure(segment, pcm.channels);
                let error = segment
                    .iter()
                    .zip(reference)
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0, f32::max);
                rows.push(json!({"id":id,"passage":label,"start_frame":start,"stop_frame":stop,
                    "requested_context_seconds":seconds,"actual_context_frames":start-left,"rendered_source_frames":right-left,
                    "seconds":before.elapsed().as_secs_f64(),"max_sample_error":error,
                    "continuous_lufs":measured_reference.0,"excerpt_lufs":m.0,"lufs_difference":m.0-measured_reference.0,
                    "continuous_provisional_peak":measured_reference.1,"excerpt_provisional_peak":m.1,
                    "peak_difference":m.1-measured_reference.1}));
                std::fs::write(out.join("comparison.json"),serde_json::to_vec_pretty(&json!({"source_sha256":source["source_sha256"],
                    "scope":"context/state and rational SRC phase comparison only; not final peak protection or full-song ranking proof","rows":rows})).unwrap()).unwrap();
            }
        }
    }
}
