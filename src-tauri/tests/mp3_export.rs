#![cfg(feature = "app-runner")]
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use yes_master_lib::{engine::*, mp3, *};
mod common;

fn source(path: &Path, rate: u32, frequency: f32) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 2,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for frame in 0..rate * 2 {
        let sample = (frame as f32 * frequency * std::f32::consts::TAU / rate as f32).sin() * 0.12;
        writer.write_sample(sample).unwrap();
        writer.write_sample(sample * 0.7).unwrap();
    }
    writer.finalize().unwrap();
}

#[test]
fn mp3_master_encodes_the_same_processed_float_pcm_and_measures_delivered_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("source.wav");
    source(&input, 96_000, 997.0);
    let original = std::fs::read(&input).unwrap();
    let mut settings = common::default_master_settings();
    settings.advanced.bit_depth = Some(32);
    settings.advanced.target_sample_rate = Some(48_000);
    settings.eq_high_db = 3.0;
    settings.intensity = 0.8;
    settings.volume_match = true;
    let wav = mastering_render_with_cancel(
        TrackId("a".into()),
        &input,
        &settings,
        tmp.path(),
        RenderKind::Master,
        RenderJobOptions::default(),
    )
    .unwrap();
    let mp3_path = tmp.path().join("master.mp3");
    let job = mastering_render_encoded_with_cancel(
        TrackId("a".into()),
        &input,
        &settings,
        tmp.path(),
        RenderKind::Master,
        RenderJobOptions {
            output_path: Some(&mp3_path),
            ..Default::default()
        },
        Some(320),
    )
    .unwrap();
    let pcm = decode::decode_full(Path::new(&wav.output_paths[0])).unwrap();
    let reference = mp3::write(
        &tmp.path().join("reference.mp3"),
        pcm.samples.into_iter().map(Ok),
        pcm.sample_rate,
        pcm.channels,
        320,
        None,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(&mp3_path).unwrap(),
        std::fs::read(reference).unwrap(),
        "MP3 must encode the current mastered PCM, including EQ and intensity"
    );
    let delivered = mp3::measure(&mp3_path, None).unwrap();
    let m = job.measurements.unwrap();
    assert_eq!(m.mp3_bitrate_kbps, Some(320));
    assert_eq!(m.bit_depth, 0);
    assert_eq!(
        (m.lufs_integrated, m.true_peak_dbtp, m.dynamic_range_lu),
        delivered
    );
    assert_eq!(std::fs::read(&input).unwrap(), original);
    assert!(mastering_render_encoded_with_cancel(
        TrackId("a".into()),
        &input,
        &settings,
        tmp.path(),
        RenderKind::Master,
        RenderJobOptions {
            output_path: Some(&input),
            ..Default::default()
        },
        Some(320)
    )
    .is_err());
}

#[test]
fn incremental_analysis_delivers_each_track_before_starting_the_next() {
    let tmp = tempfile::tempdir().unwrap();
    let tracks: Vec<_> = (0..2)
        .map(|i| {
            let path = tmp.path().join(format!("{i}.wav"));
            source(&path, 48_000, 440.0 + i as f32 * 200.0);
            AnalyzeRequest {
                id: TrackId(i.to_string()),
                path: path.to_string_lossy().into_owned(),
            }
        })
        .collect();
    let events = std::cell::RefCell::new(Vec::new());
    let results = analyze_tracks_incremental_sync(
        tracks,
        |index, _, _, _, _| events.borrow_mut().push(format!("progress-{index}")),
        |result| {
            events
                .borrow_mut()
                .push(format!("ready-{}", result.track_id.as_str()))
        },
    )
    .unwrap();
    assert_eq!(results.len(), 2);
    let events = events.into_inner();
    assert!(
        events.iter().position(|e| e == "ready-0").unwrap()
            < events.iter().position(|e| e == "progress-1").unwrap()
    );
    assert_eq!(events.last().unwrap(), "ready-1");
}

#[test]
fn album_mp3_preserves_sequence_overrides_gaps_and_uses_one_encode_for_continuous_audio() {
    let tmp = tempfile::tempdir().unwrap();
    let inputs: Vec<_> = (0..2)
        .map(|i| {
            let path = tmp.path().join(format!("Song-{i}.wav"));
            source(&path, 48_000, 440.0 + i as f32 * 440.0);
            AnalyzeRequest {
                id: TrackId(i.to_string()),
                path: path.to_string_lossy().into_owned(),
            }
        })
        .collect();
    let analyses = analyze_tracks_core_with_progress_sync(inputs.clone(), |_, _| {}).unwrap();
    let mut plan = album::build_album_plan(
        "MP3 Album".into(),
        &analyses.iter().rev().collect::<Vec<_>>(),
        &[2.0, 2.0],
        AlbumArc::default(),
        0.5,
    );
    plan.delivery_sample_rate = Some(48_000);
    plan.delivery_bit_depth = Some(32);
    plan.transitions[0] = TransitionSpec {
        kind: TransitionKind::Gap,
        duration_seconds: 0.5,
    };
    let request = AlbumPlanRenderRequest {
        plan,
        tracks: inputs
            .iter()
            .map(|t| AlbumTrackRenderInput {
                track_id: t.id.clone(),
                source_path: t.path.clone(),
                settings: common::default_master_settings(),
                override_album: t.id.as_str() == "1",
            })
            .collect(),
    };
    let output = tmp.path().join("exports");
    let report =
        mp3::render_album(&request, &output, None, None, Some("mp3-album"), Some(256)).unwrap();
    assert_eq!(
        report
            .tracks
            .iter()
            .map(|t| t.track_id.as_str())
            .collect::<Vec<_>>(),
        ["1", "0"]
    );
    assert!(report.tracks[0].override_album);
    assert!(report.tracks[0]
        .output_path
        .ends_with("01-Song-1_mastered.mp3"));
    assert_eq!(report.mp3_bitrate_kbps, Some(256));
    let wav = album_render::render_album_plan_impl(&request, &tmp.path().join("reference"), None)
        .unwrap();
    let pcm = decode::decode_full(Path::new(&wav.album_wav_path)).unwrap();
    assert_eq!(pcm.samples.len() / 2, 48_000 * 9 / 2);
    let expected = mp3::write(
        &tmp.path().join("expected.mp3"),
        pcm.samples.into_iter().map(Ok),
        pcm.sample_rate,
        pcm.channels,
        256,
        None,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(expected).unwrap(),
        std::fs::read(&report.album_wav_path).unwrap()
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report.manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["mp3_bitrate_kbps"], 256);
    assert!(manifest["album_measurements"]["true_peak_dbtp"].is_number());
    for track in &report.tracks {
        let m = mp3::measure(Path::new(&track.output_path), None).unwrap();
        assert_eq!(track.measured_lufs, m.0);
        assert_eq!(track.true_peak_dbtp, m.1);
    }
    let album_dir = Path::new(&report.album_wav_path).parent().unwrap();
    assert!(!std::fs::read_dir(album_dir).unwrap().any(|p| p
        .unwrap()
        .path()
        .extension()
        .is_some_and(|e| e == "wav")));
    // Cancel during delivery after staging succeeded; a prior export survives.
    let cancel = AtomicBool::new(false);
    let progress = |fraction: f32| {
        if fraction > 0.8 {
            cancel.store(true, Ordering::Relaxed);
        }
    };
    let cancelled = mp3::render_album(
        &request,
        &output,
        Some(&progress),
        Some(&cancel),
        Some("cancel-album"),
        Some(128),
    )
    .unwrap();
    assert!(matches!(cancelled.status, JobStatus::Cancelled));
    assert_eq!(std::fs::read_dir(&output).unwrap().count(), 1);
    assert!(Path::new(&report.album_wav_path).exists());
}
