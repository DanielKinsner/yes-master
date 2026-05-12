use album_mastering_studio_lib::*;

#[tokio::test]
async fn analyze_tracks_returns_one_result_per_input() {
    let ids = vec![
        TrackId("track-a".to_string()),
        TrackId("track-b".to_string()),
    ];
    let results = engine::analyze_tracks(ids.clone()).await.expect("analyze ok");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].track_id, ids[0]);
    assert_eq!(results[1].track_id, ids[1]);
    for r in &results {
        assert!(r.lufs_integrated.is_finite());
        assert!(r.true_peak_dbtp.is_finite());
        assert!(r.dynamic_range_lu.is_finite());
        assert!(r.spectral_balance.low.is_finite());
        assert!(r.spectral_balance.mid.is_finite());
        assert!(r.spectral_balance.high.is_finite());
        assert_eq!(r.recommended_universal.preset, Preset::Universal);
    }
}

#[tokio::test]
async fn prepare_waveform_returns_stereo_peaks() {
    let result = audio::prepare_waveform(TrackId("track-a".to_string()), 256)
        .await
        .expect("waveform ok");
    assert_eq!(result.channels.len(), 2, "expected stereo");
    assert!(!result.channels[0].is_empty());
    assert_eq!(result.channels[0].len(), result.channels[1].len());
    assert_eq!(result.samples_per_pixel, 256);
    assert_eq!(result.sample_rate, 44_100);
    for peak in &result.channels[0] {
        assert!(peak.is_finite() && (0.0..=1.0).contains(peak));
    }
}

#[tokio::test]
async fn import_tracks_rejects_traversal_paths() {
    let err = files::import_tracks(vec!["../../etc/passwd".to_string()])
        .await
        .expect_err("expected rejection");
    match err {
        CommandError::InvalidPath(_) => {}
        other => panic!("expected InvalidPath, got {other:?}"),
    }
}

#[tokio::test]
async fn import_tracks_extracts_display_name_and_format() {
    let tracks = files::import_tracks(vec!["C:/music/Song Title.flac".to_string()])
        .await
        .expect("import ok");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].display_name, "Song Title");
    assert_eq!(tracks[0].source_format, "flac");
}

#[tokio::test]
async fn run_export_checks_warns_on_high_true_peak() {
    let report = ExportReport {
        track_id: TrackId("t".to_string()),
        output_path: "out.wav".to_string(),
        measured_lufs: -14.0,
        measured_true_peak_dbtp: 0.5,
        measured_dynamic_range_lu: 8.0,
        source_format: "wav".to_string(),
        destination_format: "wav".to_string(),
        sample_rate: 44_100,
        bit_depth: 24,
        checks: Vec::new(),
    };
    let checks = exports::run_export_checks(report).await.expect("checks ok");
    assert!(checks.iter().any(|c| c.code == "true_peak_high"));
}

#[tokio::test]
async fn run_export_checks_passes_silently_when_clean() {
    let report = ExportReport {
        track_id: TrackId("t".to_string()),
        output_path: "out.wav".to_string(),
        measured_lufs: -14.0,
        measured_true_peak_dbtp: -1.2,
        measured_dynamic_range_lu: 9.0,
        source_format: "wav".to_string(),
        destination_format: "wav".to_string(),
        sample_rate: 44_100,
        bit_depth: 24,
        checks: Vec::new(),
    };
    let checks = exports::run_export_checks(report).await.expect("checks ok");
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].code, "export_ok");
}

#[tokio::test]
async fn render_track_master_returns_done_with_output_path() {
    let settings = default_settings();
    let job = engine::render_track_master(TrackId("t".to_string()), settings)
        .await
        .expect("render ok");
    assert!(matches!(job.status, JobStatus::Done));
    assert_eq!(job.progress, 1.0);
    assert!(!job.output_paths.is_empty());
    assert!(matches!(job.kind, RenderKind::Master));
}

#[tokio::test]
async fn save_user_preset_rejects_empty_name() {
    let err = settings::save_user_preset(
        "  ".to_string(),
        PresetKind::Track,
        default_settings(),
    )
    .await
    .expect_err("expected rejection");
    match err {
        CommandError::Other(_) => {}
        other => panic!("expected Other, got {other:?}"),
    }
}

fn default_settings() -> MasteringSettings {
    MasteringSettings {
        preset: Preset::Universal,
        intensity: 0.5,
        eq_low_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_db: 0.0,
        volume_match: false,
        advanced: AdvancedSettings::default(),
    }
}
