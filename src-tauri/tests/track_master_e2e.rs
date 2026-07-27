//! U14 native synthetic end-to-end flow (quality plan 2026-07-24).
//!
//! One synthetic WAV walks the whole Track Master surface in order —
//! import probe, analysis, waveform peaks, render, re-analysis of the
//! rendered output, export receipt + advisory checks, collision-safe
//! second export, project save, and reload — all on clean temp paths.
//! Every stage asserts on what the previous stage actually produced, so
//! a regression anywhere in the chain fails here even if each unit's own
//! test still passes in isolation.
//!
//! The receipt is built from the render's `RenderedMeasurements` with the
//! same field mapping as `evidence_lanes::export_report_for` (pub(crate),
//! so not callable from an integration test): measured_* fields come from
//! the rendered output, never from source analysis, and
//! `measurements_are_rendered` is true.

mod common;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use yes_master_lib::decode::{decode_to_peaks, probe_audio_format};
use yes_master_lib::engine::{
    analyze_tracks_core_with_progress_sync, mastering_render_to_path,
    measure_integrated_lufs_at_path, AnalyzeRequest,
};
use yes_master_lib::exports::export_checks_for_report;
use yes_master_lib::project::{read_session, write_session_atomic};
use yes_master_lib::types::{
    AnalysisResult, ExportReport, ImportedTrack, JobStatus, ProjectMode, ProjectState,
    QualityLevel, RenderKind, RenderedMeasurements, TrackId, ViewMode,
};

const SESSION_SCHEMA_VERSION: u32 = 1;

fn write_sine_wav(path: &Path, sample_rate: u32, seconds: f32, freq: f32, channels: u16) {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).expect("create test wav");
    let frames = (sample_rate as f32 * seconds) as usize;
    for n in 0..frames {
        let s = (0.4
            * (2.0 * std::f32::consts::PI * freq * n as f32 / sample_rate as f32).sin()
            * 32_767.0) as i16;
        for _ in 0..channels {
            w.write_sample(s).expect("write sample");
        }
    }
    w.finalize().expect("finalize test wav");
}

fn analyze_one(id: &str, path: &Path) -> AnalysisResult {
    let mut results = analyze_tracks_core_with_progress_sync(
        vec![AnalyzeRequest {
            id: TrackId(id.to_string()),
            path: path.to_string_lossy().to_string(),
        }],
        |_, _| {},
    )
    .expect("analysis succeeds");
    assert_eq!(results.len(), 1, "one request in, one result out");
    results.remove(0)
}

fn receipt_from(
    track_id: &TrackId,
    output_path: &Path,
    rendered: &RenderedMeasurements,
) -> ExportReport {
    ExportReport {
        track_id: track_id.clone(),
        output_path: output_path.to_string_lossy().to_string(),
        measured_lufs: rendered.lufs_integrated,
        measured_true_peak_dbtp: rendered.true_peak_dbtp,
        measured_dynamic_range_lu: rendered.dynamic_range_lu,
        source_format: "wav".to_string(),
        destination_format: "wav".to_string(),
        sample_rate: rendered.sample_rate,
        bit_depth: rendered.bit_depth,
        effective_adaptive_strength: rendered.effective_adaptive_strength,
        source_profile_digest: rendered.source_profile_digest.clone(),
        confidence_digest: rendered.confidence_digest.clone(),
        compression_digest: rendered.compression_digest.clone(),
        measurements_are_rendered: true,
        checks: Vec::new(),
    }
}

#[test]
fn synthetic_wav_full_flow_import_to_reload() {
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("e2e-source.wav");
    write_sine_wav(&src, 44_100, 2.0, 440.0, 2);

    // ---- Import: probe the source the way the import surface does.
    let probed = probe_audio_format(&src).expect("probe imported wav");
    assert_eq!(probed.sample_rate, 44_100);
    assert_eq!(probed.channels, 2);
    let track_id = TrackId("e2e-track".to_string());
    let imported = ImportedTrack {
        id: track_id.clone(),
        path: src.to_string_lossy().to_string(),
        display_name: "e2e-source".to_string(),
        source_format: "wav".to_string(),
        duration_seconds: Some(2.0),
        sample_rate: Some(probed.sample_rate),
        channels: Some(probed.channels),
    };

    // ---- Analyze: real measurements with a live progress callback.
    let progress_calls = std::cell::Cell::new(0usize);
    let mut results = analyze_tracks_core_with_progress_sync(
        vec![AnalyzeRequest {
            id: track_id.clone(),
            path: imported.path.clone(),
        }],
        |fraction, label| {
            assert!(
                (0.0..=1.0).contains(&fraction),
                "progress fraction out of range: {fraction} ({label})",
            );
            progress_calls.set(progress_calls.get() + 1);
        },
    )
    .expect("source analysis succeeds");
    let analysis = results.remove(0);
    assert!(analysis.lufs_integrated.is_finite());
    assert!(analysis.true_peak_dbtp.is_finite());
    assert!(
        analysis.true_peak_dbtp < 0.0,
        "a 0.4-amplitude sine cannot be at or above full scale, got {} dBTP",
        analysis.true_peak_dbtp,
    );

    // ---- Waveform: peaks for the UI, bounded and non-silent.
    let peaks = decode_to_peaks(&src, 800).expect("waveform peaks");
    assert_eq!(peaks.sample_rate, 44_100);
    assert!(!peaks.channels.is_empty(), "peaks must carry channels");
    for channel in &peaks.channels {
        assert!(!channel.is_empty(), "peak channel must not be empty");
        assert!(
            channel.iter().all(|v| (-1.0..=1.0).contains(v)),
            "peaks must stay in [-1, 1]",
        );
    }
    assert!(
        peaks.channels[0].iter().any(|v| v.abs() > 0.1),
        "a 0.4-amplitude sine must produce visibly non-silent peaks",
    );

    // ---- Render: master to an explicit chosen path.
    let settings = common::default_master_settings();
    let chosen = tmp.path().join("renders").join("e2e-master.wav");
    let job = mastering_render_to_path(
        track_id.clone(),
        &src,
        &settings,
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("render succeeds");
    assert!(matches!(job.status, JobStatus::Done));
    assert_eq!(job.output_paths.len(), 1);
    let rendered_path = PathBuf::from(&job.output_paths[0]);
    assert!(rendered_path.exists(), "rendered output must exist");
    let rendered = job
        .measurements
        .clone()
        .expect("master render must measure its own output");
    assert!(rendered.lufs_integrated.is_finite());
    assert!(rendered.true_peak_dbtp.is_finite());

    // The written file must match what the measurements claim it is.
    let reader = hound::WavReader::open(&rendered_path).expect("rendered wav decodes");
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, rendered.sample_rate);
    assert_eq!(spec.bits_per_sample, rendered.bit_depth);

    // ---- Re-analyze the rendered output: it must survive its own pipeline.
    let re_analysis = analyze_one("e2e-rendered", &rendered_path);
    assert!(re_analysis.lufs_integrated.is_finite());
    let independent_lufs =
        measure_integrated_lufs_at_path(&rendered_path).expect("independent LUFS measurement");
    assert!(
        (independent_lufs - rendered.lufs_integrated).abs() < 0.5,
        "receipt LUFS ({}) must agree with an independent measurement of the \
         written file ({}) within quantization tolerance",
        rendered.lufs_integrated,
        independent_lufs,
    );

    // ---- Export receipt + advisory checks on the real measurements.
    let mut report = receipt_from(&track_id, &rendered_path, &rendered);
    report.checks = export_checks_for_report(&report, Some(&analysis), Some(&settings));
    assert!(report.measurements_are_rendered);
    for check in &report.checks {
        assert!(!check.code.is_empty() && !check.message.is_empty());
        assert!(
            !matches!(check.level, QualityLevel::Critical),
            "a clean synthetic sine master must not produce a Critical check, got {}: {}",
            check.code,
            check.message,
        );
    }

    // ---- Collision-safe second export: same chosen path must divert.
    let first_bytes = std::fs::read(&rendered_path).expect("read first render");
    let job2 = mastering_render_to_path(
        track_id.clone(),
        &src,
        &settings,
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("second render succeeds");
    let second_path = PathBuf::from(&job2.output_paths[0]);
    assert_ne!(
        second_path, rendered_path,
        "second export must not land on the first",
    );
    assert_eq!(
        second_path,
        chosen.parent().unwrap().join("e2e-master__1.wav"),
        "divert follows the __{{n}} convention",
    );
    assert!(second_path.exists());
    assert_eq!(
        std::fs::read(&rendered_path).expect("re-read first render"),
        first_bytes,
        "first render must survive the second byte-for-byte",
    );

    // ---- Project save: full state to a clean path, then reload.
    let mut track_settings = std::collections::HashMap::new();
    track_settings.insert(track_id.0.clone(), settings.clone());
    let mut view_by_track_id = std::collections::HashMap::new();
    view_by_track_id.insert(track_id.0.clone(), ViewMode::Standard);
    let state = ProjectState {
        schema_version: SESSION_SCHEMA_VERSION,
        mode: ProjectMode::Track,
        tracks: vec![imported.clone()],
        track_order: vec![track_id.clone()],
        track_settings,
        album_intent: None,
        album_arc_kind: Default::default(),
        album_intensity: 1.0,
        album_title: String::new(),
        album_sample_rate: None,
        album_bit_depth: None,
        track_override_album: Vec::new(),
        selected_track_id: Some(track_id.clone()),
        view_by_track_id,
        last_saved_iso: Some(yes_master_lib::types::now_iso()),
    };
    let session_path = tmp.path().join("projects").join("e2e.ams.json");
    std::fs::create_dir_all(session_path.parent().unwrap()).expect("create project dir");
    write_session_atomic(&session_path, &state).expect("project saves");

    // ---- Reload: the state round-trips and its track is still usable.
    let reloaded = read_session(&session_path).expect("project reloads");
    assert_eq!(reloaded.schema_version, SESSION_SCHEMA_VERSION);
    assert_eq!(reloaded.tracks.len(), 1);
    assert_eq!(reloaded.tracks[0].id, track_id);
    assert_eq!(reloaded.tracks[0].path, imported.path);
    assert_eq!(reloaded.selected_track_id, Some(track_id.clone()));
    let reloaded_settings = reloaded
        .track_settings
        .get(&track_id.0)
        .expect("settings survive reload");
    assert_eq!(reloaded_settings.preset, settings.preset);
    assert!((reloaded_settings.intensity - settings.intensity).abs() < f32::EPSILON);
    assert_eq!(
        reloaded.view_by_track_id.get(&track_id.0),
        Some(&ViewMode::Standard),
    );

    // The reloaded track's path must still analyze — a session whose track
    // paths are unusable after reload is the "my tracks disappeared" case.
    let post_reload = analyze_one("e2e-reloaded", Path::new(&reloaded.tracks[0].path));
    assert!(
        (post_reload.lufs_integrated - analysis.lufs_integrated).abs() < 0.1,
        "re-analyzing the same source after reload must reproduce the measurement",
    );

    // Flow completed with a live progress path.
    assert!(
        progress_calls.get() > 0,
        "analysis must report progress at least once",
    );
}

#[test]
fn mono_synthetic_wav_flows_end_to_end() {
    // The same flow contract holds for a mono source — the render surface
    // must accept it, measure it, and the collision divert must hold.
    let tmp = TempDir::new().expect("tempdir");
    let src = tmp.path().join("mono-source.wav");
    write_sine_wav(&src, 44_100, 1.0, 330.0, 1);

    let probed = probe_audio_format(&src).expect("probe mono wav");
    assert_eq!(probed.channels, 1);

    let analysis = analyze_one("mono-track", &src);
    assert!(analysis.lufs_integrated.is_finite());

    let settings = common::default_master_settings();
    let chosen = tmp.path().join("mono-master.wav");
    let job = mastering_render_to_path(
        TrackId("mono-track".to_string()),
        &src,
        &settings,
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("mono render succeeds");
    assert!(matches!(job.status, JobStatus::Done));
    let out = PathBuf::from(&job.output_paths[0]);
    assert!(out.exists());
    hound::WavReader::open(&out).expect("mono rendered output decodes");

    let job2 = mastering_render_to_path(
        TrackId("mono-track-2".to_string()),
        &src,
        &settings,
        tmp.path(),
        RenderKind::Master,
        &chosen,
    )
    .expect("second mono render succeeds");
    assert_eq!(
        PathBuf::from(&job2.output_paths[0]),
        tmp.path().join("mono-master__1.wav"),
    );
}
