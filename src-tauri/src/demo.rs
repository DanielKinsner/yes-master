//! Demo track (Pass 4, 2026-08-19): a short, synthesised musical loop the
//! empty state can offer as "Try a demo track", so a first-run user with
//! nothing to drop still reaches the Original → Mastered moment.
//!
//! Synthesised on first use (no bundled asset, no licence question) and
//! written once to app-data `demo/yes-master-demo.wav`; later calls return
//! the existing file. Deliberately MIXED QUIET (peaks ≈ −8 dBFS, integrated
//! ≈ −20 LUFS) with a soft top so the master has room to move: louder,
//! brighter, tighter — an audible A/B, not a subtle one.
//!
//! If the owner later supplies a real CC track, replace `render_demo_samples`
//! with a bundled resource read; `prepare_demo_track`'s contract (returns a
//! path the importer accepts) stays the same.

use std::path::{Path, PathBuf};

use crate::types::{CommandError, CommandResult};

pub const DEMO_SAMPLE_RATE: u32 = 48_000;
pub const DEMO_SECONDS: f32 = 24.0;
pub const DEMO_FILE_NAME: &str = "yes-master-demo.wav";
const BPM: f32 = 96.0;

/// Interleaved stereo f32 at `DEMO_SAMPLE_RATE`, `DEMO_SECONDS` long.
/// Deterministic (no RNG state shared across calls): the same bytes every time.
pub fn render_demo_samples() -> Vec<f32> {
    let sr = DEMO_SAMPLE_RATE as f32;
    let n = (sr * DEMO_SECONDS) as usize;
    let beat = 60.0 / BPM; // seconds per beat
    let bar = beat * 4.0;
    // Am – F – C – G (roots in Hz) — two bars each over the 8-bar loop.
    let chords: [[f32; 3]; 4] = [
        [220.00, 261.63, 329.63], // A minor: A3 C4 E4
        [174.61, 220.00, 261.63], // F major: F3 A3 C4
        [261.63, 329.63, 392.00], // C major: C4 E4 G4
        [196.00, 246.94, 293.66], // G major: G3 B3 D4
    ];
    let roots = [110.0f32, 87.31, 130.81, 98.0]; // A2 F2 C3 G2

    let mut out = vec![0.0f32; n * 2];
    // One-pole low-pass state per channel for the pad (soft top on purpose).
    let mut lp_l = 0.0f32;
    let mut lp_r = 0.0f32;
    let lp_a = 1.0 - (-2.0 * std::f32::consts::PI * 2_400.0 / sr).exp();
    // Deterministic noise for snare/hat.
    let mut seed: u32 = 0x1234_5678;
    let mut noise = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    };

    for i in 0..n {
        let t = i as f32 / sr;
        let bar_idx = ((t / bar).floor() as usize) % 8;
        let chord = &chords[(bar_idx / 2) % 4];
        let root = roots[(bar_idx / 2) % 4];
        let t_in_beat = t % beat;
        let beat_idx = ((t / beat).floor() as usize) % 4;

        // Pad: three detuned voices per note, gentle attack at each chord.
        let chord_t = t % (bar * 2.0);
        let pad_env =
            (chord_t / 0.35).min(1.0) * (1.0 - ((chord_t - bar * 2.0 + 0.4) / 0.4).clamp(0.0, 1.0));
        let mut pad_l = 0.0f32;
        let mut pad_r = 0.0f32;
        for (k, f) in chord.iter().enumerate() {
            let w = 2.0 * std::f32::consts::PI * f;
            let det = 1.0 + 0.0025 * (k as f32 + 1.0);
            let a = (w * t).sin() + 0.5 * (w * t * 2.0).sin() * 0.6;
            let b = (w * det * t).sin();
            let c = (w / det * t).sin();
            pad_l += a * 0.5 + b * 0.35;
            pad_r += a * 0.5 + c * 0.35;
        }
        pad_l *= 0.055 * pad_env;
        pad_r *= 0.055 * pad_env;
        lp_l += lp_a * (pad_l - lp_l);
        lp_r += lp_a * (pad_r - lp_r);

        // Bass: sine on the root, plucked each beat.
        let bass_env = (-t_in_beat * 4.0).exp();
        let bass = (2.0 * std::f32::consts::PI * root * t).sin() * 0.16 * bass_env;

        // Kick on 1 and 3: pitch-dropping sine.
        let kick = if beat_idx % 2 == 0 {
            let k_t = t_in_beat;
            let f = 40.0 + 80.0 * (-k_t * 30.0).exp();
            (2.0 * std::f32::consts::PI * f * k_t).sin() * (-k_t * 9.0).exp() * 0.5
        } else {
            0.0
        };
        // Snare on 2 and 4: noise burst + body.
        let snare = if beat_idx % 2 == 1 {
            let s_t = t_in_beat;
            let body = (2.0 * std::f32::consts::PI * 180.0 * s_t).sin() * (-s_t * 25.0).exp();
            let n0 = noise() * (-s_t * 18.0).exp();
            (body * 0.35 + n0 * 0.22) * 0.6
        } else {
            0.0
        };
        // Hats: short noise ticks on eighths.
        let eighth_t = t % (beat / 2.0);
        let hat = noise() * (-eighth_t * 90.0).exp() * 0.07;

        let mono_drums = kick + snare;
        let l = lp_l + bass * 0.9 + mono_drums + hat * 0.9;
        let r = lp_r + bass * 0.9 + mono_drums + hat * 1.1;
        out[i * 2] = l;
        out[i * 2 + 1] = r;
    }

    // Normalise to a deliberately conservative peak so the master has room.
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs())).max(1e-6);
    let target = 10f32.powf(-8.0 / 20.0); // −8 dBFS
    let g = target / peak;
    for s in &mut out {
        *s *= g;
    }
    out
}

/// Ensure the demo WAV exists under `app_data_dir/demo/` and return its path.
pub fn prepare_demo_track_in(app_data_dir: &Path) -> CommandResult<PathBuf> {
    let dir = app_data_dir.join("demo");
    std::fs::create_dir_all(&dir).map_err(|e| CommandError::Other(format!("demo dir: {e}")))?;
    let path = dir.join(DEMO_FILE_NAME);
    if path.is_file() {
        return Ok(path);
    }
    let samples = render_demo_samples();
    crate::wav_writer::write_wav(&path, &samples, DEMO_SAMPLE_RATE, 2, 24)
}

#[tauri::command]
pub async fn prepare_demo_track(app: tauri::AppHandle) -> CommandResult<String> {
    use tauri::Manager;
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| CommandError::Other(format!("app data dir unavailable: {e}")))?;
    let path = tauri::async_runtime::spawn_blocking(move || prepare_demo_track_in(&app_data))
        .await
        .map_err(|e| CommandError::Other(format!("demo task: {e}")))??;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_is_24s_stereo_quiet_and_not_silent() {
        let s = render_demo_samples();
        assert_eq!(
            s.len(),
            (DEMO_SAMPLE_RATE as f32 * DEMO_SECONDS) as usize * 2
        );
        let peak = s.iter().fold(0.0f32, |m, x| m.max(x.abs()));
        let peak_db = 20.0 * peak.log10();
        assert!((-8.6..=-7.4).contains(&peak_db), "peak {peak_db} dBFS");
        let rms = (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt();
        assert!(rms > 0.01, "not silent: rms {rms}");
        assert!(s.iter().all(|x| x.is_finite()));
        // Deterministic.
        assert_eq!(render_demo_samples()[12_345], s[12_345]);
    }

    #[test]
    fn prepare_writes_once_and_reuses() {
        let dir = std::env::temp_dir().join(format!("yes-master-demo-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p1 = prepare_demo_track_in(&dir).expect("write");
        assert!(p1.is_file());
        let m1 = std::fs::metadata(&p1).unwrap().modified().unwrap();
        let p2 = prepare_demo_track_in(&dir).expect("reuse");
        assert_eq!(p1, p2);
        assert_eq!(std::fs::metadata(&p2).unwrap().modified().unwrap(), m1);
        // Decodes back as 48k stereo.
        let reader = hound::WavReader::open(&p1).unwrap();
        assert_eq!(reader.spec().sample_rate, DEMO_SAMPLE_RATE);
        assert_eq!(reader.spec().channels, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
