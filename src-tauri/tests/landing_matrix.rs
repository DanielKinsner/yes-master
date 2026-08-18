//! B2 — Track-master landing accuracy matrix (hardening plan; owner
//! ruin-type #2: "overall loudness being inaccurate").
//!
//! `delivery_profile_render.rs` proves every profile lands a friendly
//! −10 dBFS sine. This matrix stresses the landing where it can actually
//! miss: source loudness extremes and high-crest material, crossed with
//! gain-hungry presets and targets from −10.5 to −23 LUFS.
//!
//! The pinned contract per cell (source × preset × profile):
//!   1. the render never lands HOT: measured LUFS ≤ target + TOL;
//!   2. the true-peak ceiling holds: receipt TP ≤ ceiling + 0.1 dB and
//!      the file's sample peak ≤ ceiling + 0.2 dB;
//!   3. any undershoot is honest: LUFS ≥ target − TOL, OR the peak sits
//!      at the ceiling (the documented ceiling-bounded landing — the
//!      only legitimate reason to come up short);
//!   4. the receipt tells the truth: receipt LUFS within 0.5 LU of an
//!      independent re-measurement of the written file.

use std::path::Path;
use yes_master_lib::engine::{self, measure_integrated_lufs_at_path};
use yes_master_lib::types::{
    AdvancedSettings, DeliveryProfile, MasteringSettings, Preset, RenderKind, TrackId,
};

const SR_HZ: u32 = 48_000;
const SECONDS: f32 = 4.0;
const LUFS_TOL: f32 = 1.0;

// ---------------------------------------------------------------------------
// Synthetic sources — deterministic, spanning the loudness/crest space
// ---------------------------------------------------------------------------

struct Lcg(u32);
impl Lcg {
    fn next_white(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (((self.0 >> 16) & 0x7FFF) as f32 / 32_768.0) - 0.5
    }
}

fn pink_frames(frames: usize, peak: f32) -> Vec<f32> {
    let mut rng = Lcg(0xCAFE_BABE);
    let (mut b0, mut b1, mut b2, mut b3, mut b4, mut b5) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let mut mono = Vec::with_capacity(frames);
    for _ in 0..frames {
        let w = rng.next_white();
        b0 = 0.99886 * b0 + w * 0.0555179;
        b1 = 0.99332 * b1 + w * 0.0750759;
        b2 = 0.96900 * b2 + w * 0.153_852;
        b3 = 0.86650 * b3 + w * 0.3104856;
        b4 = 0.55000 * b4 + w * 0.5329522;
        b5 = -0.7616 * b5 - w * 0.0168980;
        mono.push(b0 + b1 + b2 + b3 + b4 + b5 + w * 0.5362 + w * 0.115926);
    }
    let max = mono
        .iter()
        .map(|s| s.abs())
        .fold(f32::MIN_POSITIVE, f32::max);
    let scale = peak / max;
    mono.iter_mut().for_each(|s| *s *= scale);
    mono
}

/// Kick/snare pattern — high crest so a hot target forces the landing
/// into its ceiling bound.
fn drum_frames(frames: usize, peak: f32) -> Vec<f32> {
    let sr = SR_HZ as f32;
    let mut rng = Lcg(0xDEAD_BEA7);
    let mut mono = vec![0.0_f32; frames];
    let mut beat = 0usize;
    loop {
        let start = (beat as f32 * 0.5 * sr) as usize;
        if start >= frames {
            break;
        }
        let dur = (0.25 * sr) as usize;
        let mut phase = 0.0_f32;
        for n in 0..dur {
            let idx = start + n;
            if idx >= frames {
                break;
            }
            let t = n as f32 / sr;
            let s = if beat % 2 == 0 {
                let freq = 45.0 + 55.0 * (-t / 0.03).exp();
                phase += 2.0 * std::f32::consts::PI * freq / sr;
                phase.sin() * (-t / 0.10).exp()
            } else {
                (rng.next_white() * 1.6 + 0.5 * (2.0 * std::f32::consts::PI * 200.0 * t).sin())
                    * (-t / 0.07).exp()
                    * 0.8
            };
            mono[idx] += s;
        }
        beat += 1;
    }
    let max = mono
        .iter()
        .map(|s| s.abs())
        .fold(f32::MIN_POSITIVE, f32::max);
    let scale = peak / max;
    mono.iter_mut().for_each(|s| *s *= scale);
    mono
}

fn write_stereo_wav(path: &Path, mono: &[f32]) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: SR_HZ,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).expect("create source wav");
    for &s in mono {
        let q = (s.clamp(-1.0, 1.0) * 8_388_607.0) as i32;
        w.write_sample(q).expect("L");
        w.write_sample(q).expect("R");
    }
    w.finalize().expect("finalize source");
}

fn settings_for(preset: Preset, profile: DeliveryProfile) -> MasteringSettings {
    MasteringSettings {
        preset,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        eq_bands: yes_master_lib::EqBandFrequencies::default(),
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: profile,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

fn file_sample_peak_dbfs(path: &Path) -> f32 {
    let mut reader = hound::WavReader::open(path).expect("open rendered wav");
    let spec = reader.spec();
    let peak = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 24) => reader
            .samples::<i32>()
            .map(|s| (s.expect("sample") as f32 / 8_388_608.0).abs())
            .fold(0.0_f32, f32::max),
        (hound::SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|s| (s.expect("sample") as f32 / 32_768.0).abs())
            .fold(0.0_f32, f32::max),
        (hound::SampleFormat::Float, _) => reader
            .samples::<f32>()
            .map(|s| s.expect("sample").abs())
            .fold(0.0_f32, f32::max),
        other => panic!("unexpected rendered format {other:?}"),
    };
    20.0 * peak.max(f32::MIN_POSITIVE).log10()
}

#[test]
fn landing_matrix_holds_targets_and_ceilings_across_sources_and_presets() {
    let frames = (SR_HZ as f32 * SECONDS) as usize;
    let sources: [(&str, Vec<f32>); 3] = [
        // Whisper-quiet: the landing must pull it UP hard.
        ("quiet-pink", pink_frames(frames, 0.02)),
        // Already-hot dense bed: the landing must pull it DOWN.
        ("hot-pink", pink_frames(frames, 0.89)),
        // High-crest drums: a hot target runs into the ceiling bound.
        ("drums", drum_frames(frames, 0.70)),
    ];
    let presets = [
        ("Universal", Preset::Universal),
        ("Loud", Preset::Loud),
        ("Oomph", Preset::Oomph),
    ];
    let profiles = [
        DeliveryProfile::LoudRock,
        DeliveryProfile::StreamingUniversal,
        DeliveryProfile::BroadcastEu,
    ];

    let tmp = tempfile::tempdir().expect("tempdir");
    let mut failures: Vec<String> = Vec::new();

    for (src_name, mono) in &sources {
        let src_path = tmp.path().join(format!("{src_name}.wav"));
        write_stereo_wav(&src_path, mono);

        for (preset_name, preset) in &presets {
            for profile in &profiles {
                let target = profile.target_lufs().expect("profile has target");
                let ceiling = profile.ceiling_dbtp().expect("profile has ceiling");
                let label = format!("{src_name} x {preset_name} x {profile:?}");

                let out_dir = tmp
                    .path()
                    .join(format!("{src_name}-{preset_name}-{profile:?}"));
                std::fs::create_dir_all(&out_dir).expect("out dir");
                let job = engine::mastering_render(
                    TrackId(label.clone()),
                    &src_path,
                    &settings_for(preset.clone(), *profile),
                    &out_dir,
                    RenderKind::Master,
                )
                .unwrap_or_else(|e| panic!("{label}: render failed: {e:?}"));

                let out_path = Path::new(&job.output_paths[0]);
                let measured = measure_integrated_lufs_at_path(out_path)
                    .unwrap_or_else(|e| panic!("{label}: re-measure failed: {e:?}"));
                let m = job.measurements.as_ref().expect("measurements present");

                // 4 — the receipt tells the truth.
                if (m.lufs_integrated - measured).abs() > 0.5 {
                    failures.push(format!(
                        "{label}: receipt LUFS {:.2} vs file {measured:.2}",
                        m.lufs_integrated,
                    ));
                }
                // 1 — never lands hot.
                if measured > target + LUFS_TOL {
                    failures.push(format!(
                        "{label}: landed {measured:.2} LUFS, HOTTER than target {target} + {LUFS_TOL}",
                    ));
                }
                // 2 — ceiling holds (receipt and file).
                if m.true_peak_dbtp > ceiling + 0.1 {
                    failures.push(format!(
                        "{label}: receipt true peak {:.2} above ceiling {ceiling}",
                        m.true_peak_dbtp,
                    ));
                }
                let sample_peak = file_sample_peak_dbfs(out_path);
                if sample_peak > ceiling + 0.2 {
                    failures.push(format!(
                        "{label}: file sample peak {sample_peak:.2} above ceiling {ceiling}",
                    ));
                }
                // 3 — undershoot only if the ceiling stopped it.
                let reached_target = measured >= target - LUFS_TOL;
                let ceiling_bounded = m.true_peak_dbtp >= ceiling - 0.5;
                if !reached_target && !ceiling_bounded {
                    failures.push(format!(
                        "{label}: landed {measured:.2} LUFS, short of {target} ± {LUFS_TOL}, \
                         and the peak ({:.2} dBTP) is NOT at the ceiling ({ceiling}) — \
                         the miss has no legitimate cause",
                        m.true_peak_dbtp,
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "landing matrix violations ({}):\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}
