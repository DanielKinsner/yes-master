//! Task 10 — the empirical proof of Phase A's two core theses, exercised
//! end-to-end through the REAL analyze pipeline (`analyze_tracks_core` →
//! `DeepAnalysis`) on synthesized audio:
//!
//!   A. **Loudness stratification** actually separates the loud-section
//!      character from the whole-track average: a LOUD+BRIGHT section is not
//!      washed out by a QUIET+DARK section when we look at the `loud` stratum
//!      (top 15% by momentary loudness) vs the `body` (central 25–75 pctl).
//!
//!   B. **Retaining the ordered per-window series** captures sustained-vs-
//!      scattered temporal structure that the strata aggregates alone discard:
//!      two tracks with the SAME multiset of windows but DIFFERENT time order
//!      produce ~identical strata yet provably-different ordered series.
//!
//! Both tests build their own multi-segment fixtures (the `write_sine_wav`
//! helper in `contracts.rs` only emits a single constant tone, which can't
//! encode loud/dark or sustained/scattered structure), then route them through
//! the same `engine::analyze_tracks_core` the desktop app uses.

use std::f32::consts::PI;
use std::path::Path;

use yes_master_lib::deep_analysis::DeepAnalysis;
use yes_master_lib::*;

/// One contiguous tonal block: a sine at `freq` Hz / `amplitude` linear that
/// lasts `duration_sec` seconds.
#[derive(Clone, Copy)]
struct Segment {
    freq: f32,
    amplitude: f32,
    duration_sec: f32,
}

/// Write a stereo 16-bit WAV that concatenates `segments` back-to-back. This is
/// the fixture primitive both tests share: Test A concatenates a loud-bright +
/// quiet-dark block; Test B concatenates many equal-length bright/dark blocks in
/// two different orders.
fn write_segmented_wav(path: &Path, sample_rate: u32, segments: &[Segment]) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("wav create");
    // Continuous phase across segment joins avoids a click/transient at each
    // boundary that would smear the per-window spectral read.
    let mut phase = 0.0_f32;
    for seg in segments {
        let n = (sample_rate as f32 * seg.duration_sec) as u32;
        let dphase = 2.0 * PI * seg.freq / sample_rate as f32;
        let amp_i16 = (seg.amplitude * i16::MAX as f32) as i16;
        for _ in 0..n {
            let s = (phase.sin() * amp_i16 as f32) as i16;
            writer.write_sample(s).expect("write L");
            writer.write_sample(s).expect("write R");
            phase += dphase;
            if phase > 2.0 * PI {
                phase -= 2.0 * PI;
            }
        }
    }
    writer.finalize().expect("wav finalize");
}

const SR: u32 = 48_000;
// Tonal palette. With `compute_spectral_balance`'s mid/high split near ~3 kHz,
// an 8 kHz tone lands almost entirely in the `high` band (bright) and a 100 Hz
// tone almost entirely in `low` (dark).
const BRIGHT_HZ: f32 = 8_000.0;
const DARK_HZ: f32 = 100.0;

// ---------------------------------------------------------------------------
// Test A — loudness stratification: the loud stratum is brighter than the body.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deep_analysis_loud_stratum_is_brighter_than_body() {
    // 6 s LOUD+BRIGHT (0.8 @ 8 kHz) then 6 s QUIET+DARK (0.1 @ 100 Hz). Each
    // half is ~36000 frames >> SHORT_WINDOW (16384), so each contributes
    // several windows. K-weighting makes the bright-loud half far louder than
    // the dark-quiet half, so the top-15% `loud` stratum is drawn from the
    // bright windows while the central `body` percentile band straddles the
    // boundary and mixes in the dark windows. → loud `high`-share > body.
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("loud_then_quiet.wav");
    write_segmented_wav(
        &path,
        SR,
        &[
            Segment {
                freq: BRIGHT_HZ,
                amplitude: 0.8,
                duration_sec: 6.0,
            },
            Segment {
                freq: DARK_HZ,
                amplitude: 0.1,
                duration_sec: 6.0,
            },
        ],
    );

    let results = engine::analyze_tracks_core(vec![engine::AnalyzeRequest {
        id: TrackId("strata-loud-vs-body".to_string()),
        path: path.to_string_lossy().to_string(),
    }])
    .await
    .expect("analyze");

    let da = results[0]
        .deep_analysis
        .as_ref()
        .expect("normal-length track yields DeepAnalysis");

    eprintln!(
        "Test A brightness strata: loud={:.4} body={:.4} whole={:.4} (dispersion={:.4})",
        da.brightness.loud, da.brightness.body, da.brightness.whole, da.brightness.dispersion
    );

    // The central claim: the loud section's brightness is NOT washed out by the
    // quiet-dark section. The loud stratum is the bright windows; the body
    // percentile band mixes in the dark ones.
    assert!(
        da.brightness.loud > da.brightness.body,
        "loud stratum should be brighter than the body: loud={} body={}",
        da.brightness.loud,
        da.brightness.body
    );
    // And meaningfully brighter than the whole-track average too — stratification
    // recovers loud-section character the mean discards.
    assert!(
        da.brightness.loud > da.brightness.whole,
        "loud stratum should exceed the whole-track mean: loud={} whole={}",
        da.brightness.loud,
        da.brightness.whole
    );
    // Sanity: the contrast is real, not float noise (the bright half is ~all
    // high-band energy; the dark half ~none).
    assert!(
        da.brightness.loud - da.brightness.body > 0.10,
        "expected a substantial loud-vs-body brightness gap, got {}",
        da.brightness.loud - da.brightness.body
    );
}

// ---------------------------------------------------------------------------
// Test B — temporal: sustained vs scattered, same window multiset.
// ---------------------------------------------------------------------------

/// Equal-duration bright/dark blocks at CONSTANT amplitude (so per-window
/// loudness keys are ~equal across all windows and the loudness strata come out
/// ~equal). Brightness varies only via frequency, so the two arrangements share
/// the same multiset of window brightness values — only the time order differs.
const BLOCK_SEC: f32 = 2.0; // ~96000 frames/block >> SHORT_WINDOW; many windows
const BLOCK_AMP: f32 = 0.4;

fn bright_block() -> Segment {
    Segment {
        freq: BRIGHT_HZ,
        amplitude: BLOCK_AMP,
        duration_sec: BLOCK_SEC,
    }
}
fn dark_block() -> Segment {
    Segment {
        freq: DARK_HZ,
        amplitude: BLOCK_AMP,
        duration_sec: BLOCK_SEC,
    }
}

#[tokio::test]
async fn deep_analysis_ordered_series_distinguishes_sustained_from_scattered() {
    let tmp = tempfile::tempdir().expect("tempdir");

    // "Sustained": all four bright blocks contiguous, then all four dark.
    let sustained_path = tmp.path().join("sustained.wav");
    write_segmented_wav(
        &sustained_path,
        SR,
        &[
            bright_block(),
            bright_block(),
            bright_block(),
            bright_block(),
            dark_block(),
            dark_block(),
            dark_block(),
            dark_block(),
        ],
    );

    // "Scattered": the SAME four bright + four dark blocks, interleaved.
    let scattered_path = tmp.path().join("scattered.wav");
    write_segmented_wav(
        &scattered_path,
        SR,
        &[
            bright_block(),
            dark_block(),
            bright_block(),
            dark_block(),
            bright_block(),
            dark_block(),
            bright_block(),
            dark_block(),
        ],
    );

    let results = engine::analyze_tracks_core(vec![
        engine::AnalyzeRequest {
            id: TrackId("temporal-sustained".to_string()),
            path: sustained_path.to_string_lossy().to_string(),
        },
        engine::AnalyzeRequest {
            id: TrackId("temporal-scattered".to_string()),
            path: scattered_path.to_string_lossy().to_string(),
        },
    ])
    .await
    .expect("analyze");

    let sustained_da = results[0]
        .deep_analysis
        .as_ref()
        .expect("sustained yields DeepAnalysis");
    let scattered_da = results[1]
        .deep_analysis
        .as_ref()
        .expect("scattered yields DeepAnalysis");

    let high_seq = |da: &DeepAnalysis| da.windows.iter().map(|w| w.high).collect::<Vec<f32>>();
    let s_sustained = high_seq(sustained_da);
    let s_scattered = high_seq(scattered_da);

    eprintln!(
        "Test B windows: sustained={} scattered={}",
        s_sustained.len(),
        s_scattered.len()
    );
    eprintln!(
        "Test B brightness.whole: sustained={:.4} scattered={:.4} (|delta|={:.4})",
        sustained_da.brightness.whole,
        scattered_da.brightness.whole,
        (sustained_da.brightness.whole - scattered_da.brightness.whole).abs()
    );
    eprintln!(
        "Test B loudness.whole: sustained={:.4} scattered={:.4} (|delta|={:.4})",
        sustained_da.loudness.whole,
        scattered_da.loudness.whole,
        (sustained_da.loudness.whole - scattered_da.loudness.whole).abs()
    );

    // KEY CLAIM 1: the retained ORDERED series distinguishes the two
    // arrangements — temporal structure the aggregates cannot see.
    assert_ne!(
        s_sustained, s_scattered,
        "retained ordered series must distinguish sustained vs scattered brightness"
    );

    // KEY CLAIM 2: the strata aggregates alone CANNOT tell them apart — the
    // whole-track brightness mean is ~equal because the window multiset matches.
    assert!(
        (sustained_da.brightness.whole - scattered_da.brightness.whole).abs() < 0.05,
        "aggregate brightness should be ~equal across arrangements: sustained={} scattered={}",
        sustained_da.brightness.whole,
        scattered_da.brightness.whole
    );
    // Constant amplitude ⇒ loudness strata are ~equal too: the ONLY thing that
    // differs is the time order, which only the ordered series captures.
    assert!(
        (sustained_da.loudness.whole - scattered_da.loudness.whole).abs() < 0.5,
        "loudness envelope should be ~equal across arrangements: sustained={} scattered={}",
        sustained_da.loudness.whole,
        scattered_da.loudness.whole
    );
}

// ---------------------------------------------------------------------------
// Test C — tiny clip: both DeepAnalysis and SourceProfile absent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tiny_clip_yields_neither_deep_analysis_nor_source_profile() {
    // A clip shorter than one SHORT_WINDOW (16384 frames) is below the deep
    // scan's minimum, AND too short for the 6-band FFT that backs
    // SourceProfile. Existing tests cover deep_analysis.is_none() for a 9600-
    // frame clip; this pins the STRONGER joint claim — at <1024 frames the
    // source profile is also absent — so the adaptive guardrails have nothing
    // to act on rather than acting on garbage.
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("tiny.wav");
    // ~512 frames @ 48k stereo (0.0107 s) — far below SHORT_WINDOW and the FFT
    // minimum, but still a decodable file.
    write_segmented_wav(
        &path,
        SR,
        &[Segment {
            freq: 440.0,
            amplitude: 0.5,
            duration_sec: 512.0 / SR as f32,
        }],
    );

    let results = engine::analyze_tracks_core(vec![engine::AnalyzeRequest {
        id: TrackId("tiny-clip".to_string()),
        path: path.to_string_lossy().to_string(),
    }])
    .await
    .expect("analyze");

    let r = &results[0];
    assert!(
        r.deep_analysis.is_none(),
        "tiny clip (< SHORT_WINDOW frames) must not produce a DeepAnalysis"
    );
    assert!(
        SourceProfile::from_analysis(r).is_none(),
        "tiny clip must not produce a SourceProfile (6-band balance unavailable)"
    );
    // The base AnalysisResult is still well-formed (nothing regresses).
    assert!(r.lufs_integrated.is_finite());
}
