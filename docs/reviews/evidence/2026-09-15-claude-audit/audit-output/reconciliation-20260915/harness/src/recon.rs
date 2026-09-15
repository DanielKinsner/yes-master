//! Reconciliation additions: SRC probes and stage statistics.
//!
//! `prod_convert` is a VERBATIM copy of the production
//! `sample_rate::convert_interleaved` (pub(crate), so not callable from an
//! external crate). Its equivalence to the production export path is proven
//! separately by `src-file` against a 32-bit-float Custom/no-target export
//! (chain -> SRC only), which must be bit-identical.
//!
//! `recon_convert` is an independently written corrected loop (same rubato
//! FFT resampler, explicit delay removal, integer length accounting).
//! `codex_src_fixed::convert` is Codex's prototype, copied unchanged for
//! comparison (SHA-256 recorded in the evidence manifest).

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};
use std::path::{Path, PathBuf};

const SRC_CHUNK_FRAMES: usize = 2048;

/// VERBATIM production loop (src-tauri/src/sample_rate.rs, a4fb621).
pub fn prod_convert(samples: &[f32], source_sample_rate: u32, target_sample_rate: u32, channels: u16) -> Vec<f32> {
    if source_sample_rate == target_sample_rate {
        return samples.to_vec();
    }
    let channel_count = usize::from(channels.max(1));
    let input_frames = samples.len() / channel_count;
    if input_frames == 0 {
        return Vec::new();
    }
    let input = InterleavedSlice::new(samples, channel_count, input_frames).unwrap();
    let mut resampler = Fft::<f32>::new(
        source_sample_rate as usize,
        target_sample_rate as usize,
        SRC_CHUNK_FRAMES,
        1,
        channel_count,
        FixedSync::Both,
    )
    .unwrap();
    let output_capacity_frames = resampler.process_all_needed_output_len(input_frames);
    let mut output_samples = vec![0.0_f32; output_capacity_frames * channel_count];
    let mut output =
        InterleavedSlice::new_mut(&mut output_samples, channel_count, output_capacity_frames).unwrap();
    let (_input_used, output_frames) = resampler
        .process_all_into_buffer(&input, &mut output, input_frames, None)
        .unwrap();
    output_samples.truncate(output_frames * channel_count);
    output_samples
}

/// Expected output length: ceil(frames * to / from) in exact integer arithmetic.
pub fn expected_frames(frames: usize, from: u32, to: u32) -> usize {
    let n = frames as u128 * to as u128;
    let d = from as u128;
    ((n + d - 1) / d) as usize
}

/// Independently written corrected loop. Same resampler construction as
/// production; drives `process_into_buffer` chunk by chunk with explicit
/// indexing, pumps zeros until `delay + expected` frames exist, then removes
/// exactly `delay` frames of start-up and keeps exactly `expected` frames.
pub fn recon_convert(samples: &[f32], from: u32, to: u32, channels: u16) -> Vec<f32> {
    if from == to {
        return samples.to_vec();
    }
    let ch = usize::from(channels.max(1));
    let frames = samples.len() / ch;
    if frames == 0 {
        return Vec::new();
    }
    let input = InterleavedSlice::new(samples, ch, frames).unwrap();
    let mut rs = Fft::<f32>::new(from as usize, to as usize, SRC_CHUNK_FRAMES, 1, ch, FixedSync::Both).unwrap();
    let delay = rs.output_delay();
    let expected = expected_frames(frames, from, to);
    let needed = delay + expected;
    let capacity = needed + rs.output_frames_max() + 1;
    let mut buf = vec![0.0f32; capacity * ch];
    let mut consumed = 0usize;
    let mut written = 0usize;
    {
        let mut out = InterleavedSlice::new_mut(&mut buf, ch, capacity).unwrap();
        while written < needed {
            let want = rs.input_frames_next();
            let available = frames - consumed;
            let take = available.min(want);
            let idx = Indexing {
                input_offset: consumed,
                output_offset: written,
                active_channels_mask: None,
                partial_len: if take < want { Some(take) } else { None },
            };
            let (used, produced) = rs.process_into_buffer(&input, &mut out, Some(&idx)).unwrap();
            consumed += used.min(take);
            written += produced;
        }
    }
    buf[delay * ch..(delay + expected) * ch].to_vec()
}

pub fn convert_mode(mode: &str, x: &[f32], from: u32, to: u32, ch: u16) -> Vec<f32> {
    match mode {
        "prod" => prod_convert(x, from, to, ch),
        "recon" => recon_convert(x, from, to, ch),
        "codex" => crate::codex_src_fixed::convert(x, from, to, usize::from(ch)),
        other => panic!("unknown SRC mode {other}"),
    }
}

fn write_f32(path: &Path, x: &[f32], sr: u32, ch: u16) {
    let spec = hound::WavSpec { channels: ch, sample_rate: sr, bits_per_sample: 32, sample_format: hound::SampleFormat::Float };
    let mut w = hound::WavWriter::create(path, spec).expect("create");
    for s in x {
        w.write_sample(*s).expect("write");
    }
    w.finalize().expect("finalize");
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter().position(|a| a == key).and_then(|i| args.get(i + 1).cloned())
}

/// src-probe <out_dir>
/// Own sine generator (f64 phase, 0.5 peak), integer frame counts chosen to
/// straddle rubato chunk boundaries, three converters per case.
pub fn cmd_src_probe(args: &[String]) {
    let out = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&out).unwrap();
    let pairs: [(u32, u32); 6] = [(44100, 48000), (48000, 44100), (96000, 48000), (48000, 96000), (44100, 96000), (96000, 44100)];
    let mut params = Vec::new();
    let mut rows = Vec::new();
    for (from, to) in pairs {
        let mut rs = Fft::<f32>::new(from as usize, to as usize, SRC_CHUNK_FRAMES, 1, 2, FixedSync::Both).unwrap();
        let p = serde_json::json!({
            "from": from, "to": to,
            "input_frames_next": rs.input_frames_next(),
            "output_frames_next": rs.output_frames_next(),
            "output_frames_max": rs.output_frames_max(),
            "output_delay": rs.output_delay(),
            "resample_ratio": rs.resample_ratio(),
        });
        println!("{p}");
        let chunk_in = rs.input_frames_next();
        params.push(p);
        let ms = |m: f64| (from as f64 * m / 1000.0).round() as usize;
        let mut lengths: Vec<usize> = vec![1, 16, ms(5.0), ms(10.0), ms(30.0), ms(100.0), ms(500.0), ms(2000.0)];
        for k in [1usize, 2, 3] {
            lengths.push(k * chunk_in - 1);
            lengths.push(k * chunk_in);
            lengths.push(k * chunk_in + 1);
        }
        lengths.sort_unstable();
        lengths.dedup();
        let freqs: Vec<f64> = [100.0, 997.0, 5000.0, 19000.0].into_iter().filter(|f| *f < from.min(to) as f64 * 0.45).collect();
        for frames in lengths {
            for hz in &freqs {
                let x: Vec<f32> = (0..frames)
                    .flat_map(|i| {
                        let y = (0.5 * (std::f64::consts::TAU * hz * i as f64 / from as f64).sin()) as f32;
                        [y, y]
                    })
                    .collect();
                for mode in ["prod", "recon", "codex"] {
                    let y = convert_mode(mode, &x, from, to, 2);
                    let file = format!("{from}_{to}_{frames}_{hz}_{mode}.wav");
                    write_f32(&out.join(&file), &y, to, 2);
                    rows.push(serde_json::json!({
                        "from": from, "to": to, "frames_in": frames, "hz": hz, "mode": mode,
                        "file": file, "frames_out": y.len() / 2,
                        "expected_frames": expected_frames(frames, from, to),
                        "delay": rs.output_delay(),
                    }));
                }
            }
        }
    }
    std::fs::write(out.join("probe.json"), serde_json::to_string_pretty(&serde_json::json!({"params": params, "cases": rows})).unwrap()).unwrap();
    println!("wrote {} cases", rows.len());
}

/// src-file <in.wav> <out.wav> --to <rate> --mode prod|recon|codex
pub fn cmd_src_file(args: &[String]) {
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let to: u32 = arg_value(args, "--to").expect("--to").parse().unwrap();
    let mode = arg_value(args, "--mode").unwrap_or_else(|| "prod".to_string());
    let pcm = yes_master_lib::decode::decode_full(&input).expect("decode");
    let t0 = std::time::Instant::now();
    let y = convert_mode(&mode, &pcm.samples, pcm.sample_rate, to, pcm.channels);
    let el = t0.elapsed().as_secs_f64();
    write_f32(&output, &y, to, pcm.channels);
    println!("{}", serde_json::json!({
        "input": input.to_string_lossy(), "output": output.to_string_lossy(), "mode": mode,
        "from": pcm.sample_rate, "to": to, "frames_in": pcm.samples.len() / pcm.channels as usize,
        "frames_out": y.len() / pcm.channels as usize,
        "expected_frames": expected_frames(pcm.samples.len() / pcm.channels as usize, pcm.sample_rate, to),
        "elapsed_s": el,
    }));
}

/// src-impulse <out_dir>: unit impulse (and a step) through the production
/// loop for each rate pair, for frequency-response characterisation.
pub fn cmd_src_impulse(args: &[String]) {
    let out = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&out).unwrap();
    for (from, to) in [(44100u32, 48000u32), (48000, 44100), (96000, 48000)] {
        let frames = from as usize; // 1 s
        let mut x = vec![0.0f32; frames * 2];
        let at = frames / 2;
        x[at * 2] = 1.0;
        x[at * 2 + 1] = 1.0;
        let y = prod_convert(&x, from, to, 2);
        write_f32(&out.join(format!("impulse_{from}_{to}.wav")), &y, to, 2);
        let mut s = vec![0.0f32; frames * 2];
        for i in at..frames {
            s[i * 2] = 0.5;
            s[i * 2 + 1] = 0.5;
        }
        let y = prod_convert(&s, from, to, 2);
        write_f32(&out.join(format!("step_{from}_{to}.wav")), &y, to, 2);
        println!("{from}->{to}: impulse at input frame {at}");
    }
}

/// Limiter gain-reduction statistics from the bypass ratio. The limiter is a
/// linked per-frame linear gain applied to the delayed signal, and the
/// bypass keeps the same delay line, so g[n] = |full[n]| / |nolim[n]| on the
/// channel with the larger unlimited magnitude.
pub fn limiter_stats(full: &[f32], nolim: &[f32], ch: usize) -> serde_json::Value {
    let frames = full.len() / ch;
    let mut max_gr = 0.0f64;
    let mut sum_gr = 0.0f64;
    let mut active = 0usize;
    let mut counted = 0usize;
    for f in 0..frames {
        let mut best = 0usize;
        let mut bmag = 0.0f32;
        for c in 0..ch {
            let m = nolim[f * ch + c].abs();
            if m > bmag {
                bmag = m;
                best = c;
            }
        }
        if bmag < 1.0e-3 {
            continue;
        }
        let g = (full[f * ch + best].abs() / bmag) as f64;
        let gr = -20.0 * g.max(1.0e-9).log10();
        let gr = gr.max(0.0);
        counted += 1;
        sum_gr += gr;
        if gr > max_gr {
            max_gr = gr;
        }
        if gr > 0.01 {
            active += 1;
        }
    }
    serde_json::json!({
        "max_gr_db": max_gr,
        "mean_gr_db_over_signal_frames": if counted > 0 { sum_gr / counted as f64 } else { 0.0 },
        "mean_gr_db_over_all_frames": if frames > 0 { sum_gr / frames as f64 } else { 0.0 },
        "active_fraction": if counted > 0 { active as f64 / counted as f64 } else { 0.0 },
        "frames": frames,
        "signal_frames": counted,
    })
}

pub fn level_stats(a: &[f32], b: &[f32], ch: usize) -> serde_json::Value {
    let peak = |x: &[f32]| x.iter().fold(0.0f32, |m, v| m.max(v.abs())) as f64;
    let pw = |x: &[f32]| x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / (x.len().max(1) as f64);
    let _ = ch;
    serde_json::json!({
        "in_peak_dbfs": 20.0 * peak(a).max(1e-12).log10(),
        "out_peak_dbfs": 20.0 * peak(b).max(1e-12).log10(),
        "in_rms_dbfs": 10.0 * pw(a).max(1e-24).log10(),
        "out_rms_dbfs": 10.0 * pw(b).max(1e-24).log10(),
    })
}
