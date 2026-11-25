//! Smart Match reference measurement (Smart Match plan, Slice 5; contract
//! rule 12).
//!
//! A chosen reference file is decoded and measured once, through the same
//! decoder and the same meter call the track analysis uses
//! (`analysis::analyze_one_with_progress`: `decode_full`, then
//! `compute_spectral_balance_6band` on the whole file at its own rate). The
//! result is the reference's display name and six band shares, which is all a
//! project keeps (plan M9), or a refusal with one of the locked reasons.
//!
//! A reference is refused only when it cannot be measured (plan M10). A mono
//! file can be measured, so it is accepted. The command runs off the audio
//! thread; playback continues while it works. Which pick wins when several are
//! in flight, and keeping the previous reference after a refusal, belong to
//! the caller that stores the result (the desktop hook).

use crate::smart_match::{REFERENCE_MIN_DURATION_SEC, REFERENCE_MIN_SAMPLE_RATE_HZ};
use crate::types::SmartMatchReference;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A file whose sample peak never reaches this level is treated as silent.
pub const REFERENCE_SILENCE_PEAK_DBFS: f64 = -60.0;

/// Why a file cannot be a reference. Each maps to one locked reason.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmartMatchRefusal {
    TooShort,
    Silent,
    Unreadable,
    LowSampleRate,
}

impl SmartMatchRefusal {
    /// The locked reason, as it follows
    /// `That file can't be used as a reference: `.
    pub fn reason(self) -> &'static str {
        match self {
            Self::TooShort => "it is shorter than 30 seconds",
            Self::Silent => "it is silent",
            Self::Unreadable => "it could not be read",
            Self::LowSampleRate => "its sample rate is too low",
        }
    }
}

/// The outcome of measuring one chosen file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SmartMatchReferenceMeasurement {
    Measured { reference: SmartMatchReference },
    Refused { reason: SmartMatchRefusal },
}

/// The reference's display name: the file name without its extension, as
/// plain text. Nothing ever uses it as a path.
pub fn display_name(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "Reference".to_string())
}

/// Measure decoded audio as a reference. The checks run in a fixed order, so
/// a file with several problems always reports the same one: too short, then
/// sample rate, then silence.
pub fn measure_reference_samples(
    name: String,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> SmartMatchReferenceMeasurement {
    let refused = |reason| SmartMatchReferenceMeasurement::Refused { reason };
    let channel_count = usize::from(channels.max(1));
    if sample_rate == 0 || samples.is_empty() {
        return refused(SmartMatchRefusal::Unreadable);
    }
    let duration_sec = (samples.len() / channel_count) as f64 / f64::from(sample_rate);
    if duration_sec < REFERENCE_MIN_DURATION_SEC {
        return refused(SmartMatchRefusal::TooShort);
    }
    if sample_rate < REFERENCE_MIN_SAMPLE_RATE_HZ {
        return refused(SmartMatchRefusal::LowSampleRate);
    }
    let peak = samples
        .iter()
        .filter(|s| s.is_finite())
        .fold(0.0_f32, |acc, s| acc.max(s.abs()));
    if peak <= 0.0 || 20.0 * f64::from(peak).log10() < REFERENCE_SILENCE_PEAK_DBFS {
        return refused(SmartMatchRefusal::Silent);
    }
    // The track analysis passes `pcm.channels as usize`; so does this.
    match crate::smart_match::measure_shares(samples, sample_rate, usize::from(channels)) {
        Some(spectral_6)
            if crate::smart_match::shares_array(&spectral_6)
                .iter()
                .all(|s| s.is_finite() && *s >= 0.0) =>
        {
            SmartMatchReferenceMeasurement::Measured {
                reference: SmartMatchReference { name, spectral_6 },
            }
        }
        _ => refused(SmartMatchRefusal::Silent),
    }
}

/// Decode and measure a file as a reference. Any read or decode failure is
/// the "could not be read" refusal; nothing here panics on a bad file.
pub fn measure_reference_file(path: &Path) -> SmartMatchReferenceMeasurement {
    match crate::decode::decode_full(path) {
        Ok(pcm) => measure_reference_samples(
            display_name(path),
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels,
        ),
        Err(_) => SmartMatchReferenceMeasurement::Refused {
            reason: SmartMatchRefusal::Unreadable,
        },
    }
}

/// Tauri command: measure a chosen file as a Smart Match reference. Runs on a
/// blocking worker, never on the audio thread.
#[cfg(not(target_arch = "wasm32"))]
#[tauri::command]
pub async fn measure_smart_match_reference(
    path: String,
) -> crate::types::CommandResult<SmartMatchReferenceMeasurement> {
    if path.is_empty() || crate::files::has_parent_dir_component(Path::new(&path)) {
        return Ok(SmartMatchReferenceMeasurement::Refused {
            reason: SmartMatchRefusal::Unreadable,
        });
    }
    tauri::async_runtime::spawn_blocking(move || measure_reference_file(Path::new(&path)))
        .await
        .map_err(|e| crate::types::CommandError::Other(format!("reference measurement: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(sample_rate: u32, channels: u16, seconds: f64, amp: f32) -> Vec<f32> {
        let frames = (f64::from(sample_rate) * seconds) as usize;
        let mut out = Vec::with_capacity(frames * usize::from(channels));
        for n in 0..frames {
            let t = n as f32 / sample_rate as f32;
            let v = amp
                * ((std::f32::consts::TAU * 110.0 * t).sin()
                    + 0.5 * (std::f32::consts::TAU * 2_000.0 * t).sin()
                    + 0.2 * (std::f32::consts::TAU * 9_000.0 * t).sin());
            for _ in 0..channels {
                out.push(v);
            }
        }
        out
    }

    #[test]
    fn a_normal_stereo_file_is_measured_with_its_name() {
        let m = measure_reference_samples("Song".into(), &tone(44_100, 2, 31.0, 0.3), 44_100, 2);
        let SmartMatchReferenceMeasurement::Measured { reference } = m else {
            panic!("expected a measurement, got {m:?}");
        };
        assert_eq!(reference.name, "Song");
        let total: f64 = crate::smart_match::shares_array(&reference.spectral_6)
            .iter()
            .sum();
        assert!((total - 1.0).abs() < 1e-3);
    }

    #[test]
    fn mono_is_measurable_so_it_is_accepted() {
        let m = measure_reference_samples("Mono".into(), &tone(48_000, 1, 31.0, 0.3), 48_000, 1);
        assert!(
            matches!(m, SmartMatchReferenceMeasurement::Measured { .. }),
            "{m:?}"
        );
    }

    #[test]
    fn each_refusal_has_its_locked_reason() {
        let cases = [
            (
                tone(44_100, 2, 10.0, 0.3),
                44_100,
                SmartMatchRefusal::TooShort,
            ),
            (
                tone(22_050, 2, 31.0, 0.3),
                22_050,
                SmartMatchRefusal::LowSampleRate,
            ),
            (
                tone(44_100, 2, 31.0, 0.0),
                44_100,
                SmartMatchRefusal::Silent,
            ),
            (
                tone(44_100, 2, 31.0, 0.0001),
                44_100,
                SmartMatchRefusal::Silent,
            ),
            (Vec::new(), 44_100, SmartMatchRefusal::Unreadable),
        ];
        for (samples, rate, expected) in cases {
            let m = measure_reference_samples("x".into(), &samples, rate, 2);
            assert_eq!(
                m,
                SmartMatchReferenceMeasurement::Refused { reason: expected }
            );
        }
        assert_eq!(
            SmartMatchRefusal::TooShort.reason(),
            "it is shorter than 30 seconds"
        );
        assert_eq!(SmartMatchRefusal::Silent.reason(), "it is silent");
        assert_eq!(
            SmartMatchRefusal::Unreadable.reason(),
            "it could not be read"
        );
        assert_eq!(
            SmartMatchRefusal::LowSampleRate.reason(),
            "its sample rate is too low"
        );
    }

    #[test]
    fn a_missing_or_broken_file_could_not_be_read() {
        let dir = std::env::temp_dir().join(format!("ym-ref-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let broken = dir.join("broken.wav");
        std::fs::write(&broken, b"not audio at all").unwrap();
        for path in [dir.join("missing.wav"), broken] {
            assert_eq!(
                measure_reference_file(&path),
                SmartMatchReferenceMeasurement::Refused {
                    reason: SmartMatchRefusal::Unreadable
                }
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_display_name_is_the_file_stem_never_a_path() {
        assert_eq!(
            display_name(Path::new("C:/music/LANDR-It’s a coat-Warm-High.wav")),
            "LANDR-It’s a coat-Warm-High"
        );
        assert_eq!(display_name(Path::new("/x/.wav")), ".wav");
        assert_eq!(display_name(Path::new("")), "Reference");
    }
}
