#![cfg(feature = "app-runner")]
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use yes_master_lib::{engine::*, export_format::ExportEncoding, *};
mod common;

fn request(dir: &Path) -> AlbumPlanRenderRequest {
    let inputs: Vec<_> = (0..3)
        .map(|index| {
            let rate = if index == 1 { 96_000 } else { 48_000 };
            let channels = if index == 0 {
                1
            } else if index == 2 {
                4
            } else {
                2
            };
            let path = dir.join(format!("Song-{index}.wav"));
            let mut writer = hound::WavWriter::create(
                &path,
                hound::WavSpec {
                    sample_rate: rate,
                    channels,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .unwrap();
            for n in 0..rate {
                for channel in 0..channels {
                    writer
                        .write_sample(
                            (n as f32 * (431.0 + index as f32 * 173.0) * std::f32::consts::TAU
                                / rate as f32)
                                .sin()
                                * 0.1
                                / (1.0 + channel as f32),
                        )
                        .unwrap();
                }
            }
            writer.finalize().unwrap();
            AnalyzeRequest {
                id: TrackId(index.to_string()),
                path: path.to_string_lossy().into_owned(),
            }
        })
        .collect();
    let analyses = analyze_tracks_core_with_progress_sync(inputs.clone(), |_, _| {}).unwrap();
    let mut plan = album::build_album_plan(
        "New formats".into(),
        &analyses.iter().rev().collect::<Vec<_>>(),
        &[1.0; 3],
        AlbumArc::default(),
        0.6,
    );
    plan.delivery_sample_rate = Some(96_000);
    plan.delivery_bit_depth = Some(32);
    plan.transitions[0] = TransitionSpec {
        kind: TransitionKind::Gap,
        duration_seconds: 0.25,
    };
    plan.transitions[1] = TransitionSpec {
        kind: TransitionKind::Gap,
        duration_seconds: 0.5,
    };
    AlbumPlanRenderRequest {
        plan,
        tracks: inputs
            .iter()
            .map(|input| {
                let mut settings = common::default_master_settings();
                settings.eq_high_db = 3.0;
                AlbumTrackRenderInput {
                    track_id: input.id.clone(),
                    source_path: input.path.clone(),
                    settings,
                    override_album: input.id.as_str() == "1",
                }
            })
            .collect(),
    }
}

#[test]
#[ignore = "requires the staged qualified encoder package"]
fn album_matrix_preserves_order_overrides_gaps_and_one_continuous_encode() {
    let temp = tempfile::tempdir().unwrap();
    let request = request(temp.path());
    let encoder = export_encoding::Encoder::packaged().unwrap();
    let out = temp.path().join("exports");
    for encoding in [
        ExportEncoding::Flac,
        ExportEncoding::Aiff,
        ExportEncoding::M4a { bitrate_kbps: 256 },
        ExportEncoding::Aac { bitrate_kbps: 256 },
        ExportEncoding::Ogg { quality: 6 },
    ] {
        let report =
            album_encoding::render_album(&request, &out, None, None, Some("matrix"), encoding)
                .unwrap();
        assert_eq!(
            report
                .tracks
                .iter()
                .map(|track| track.track_id.as_str())
                .collect::<Vec<_>>(),
            ["2", "1", "0"]
        );
        assert!(report.tracks[1].override_album);
        assert_eq!(report.rendered_channels, 2);
        assert_eq!(report.source_channels, vec![4, 2, 1]);
        assert!(report.tracks[0]
            .output_path
            .ends_with(&format!("01-Song-2_mastered.{}", encoding.extension())));
        let mut staged_request = request.clone();
        staged_request.plan.delivery_sample_rate = Some(encoding.delivery_rate(96_000));
        staged_request.plan.delivery_bit_depth = Some(encoding.delivery_bits(32));
        let reference = album_render::render_album_plan_impl(
            &staged_request,
            &temp.path().join("reference"),
            None,
        )
        .unwrap();
        let reference_pcm = decode::decode_full(Path::new(&reference.album_wav_path)).unwrap();
        assert_eq!(
            reference_pcm.samples.len() / 2,
            encoding.delivery_rate(96_000) as usize * 15 / 4
        );
        for (delivered, staged) in report
            .tracks
            .iter()
            .map(|track| &track.output_path)
            .chain([&report.album_wav_path])
            .zip(
                reference
                    .tracks
                    .iter()
                    .map(|track| &track.output_path)
                    .chain([&reference.album_wav_path]),
            )
        {
            let expected = export_encoding::deliver_staged(
                &encoder,
                Path::new(staged),
                &temp
                    .path()
                    .join("expected")
                    .with_extension(encoding.extension()),
                encoding,
                None,
            )
            .unwrap();
            assert_eq!(
                decode::decode_full(Path::new(delivered)).unwrap().samples,
                decode::decode_full(&expected.path).unwrap().samples
            );
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report.manifest_path).unwrap()).unwrap();
        assert_eq!(manifest["format"], encoding.extension());
        assert_eq!(manifest["album_wav_path"], report.album_wav_path);
        assert_eq!(
            manifest["delivered_format"],
            serde_json::to_value(&report.delivered_format).unwrap()
        );
        for track in &report.tracks {
            let m =
                export_encoding::measure(&encoder, Path::new(&track.output_path), None).unwrap();
            assert_eq!(
                (track.measured_lufs, track.true_peak_dbtp),
                (m.measurements.0, m.measurements.1)
            );
        }
        let original = std::fs::read(&report.album_wav_path).unwrap();
        let directories_before = std::fs::read_dir(&out).unwrap().count();
        let cancel = AtomicBool::new(false);
        let progress = |value: f32| {
            if value > 0.8 {
                cancel.store(true, Ordering::Relaxed)
            }
        };
        let cancelled = album_encoding::render_album(
            &request,
            &out,
            Some(&progress),
            Some(&cancel),
            None,
            encoding,
        )
        .unwrap();
        assert!(matches!(cancelled.status, JobStatus::Cancelled));
        assert!(cancelled.tracks.is_empty());
        assert_eq!(std::fs::read_dir(&out).unwrap().count(), directories_before);
        assert_eq!(std::fs::read(&report.album_wav_path).unwrap(), original);
    }
}

#[test]
#[ignore = "requires the staged qualified encoder package"]
fn failed_manifest_keeps_foreign_file_and_removes_only_owned_audio() {
    let temp = tempfile::tempdir().unwrap();
    let request = request(temp.path());
    let out = temp.path().join("exports");
    let foreign = std::cell::RefCell::new(None);
    let progress = |value: f32| {
        if value > 0.94 && value < 1.0 && foreign.borrow().is_none() {
            let destination = std::fs::read_dir(&out)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            let path = destination.join("metadata/manifest.json");
            std::fs::write(&path, b"belongs to another writer").unwrap();
            *foreign.borrow_mut() = Some(path);
        }
    };
    assert!(album_encoding::render_album(
        &request,
        &out,
        Some(&progress),
        None,
        None,
        ExportEncoding::Flac
    )
    .is_err());
    let path = foreign.into_inner().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"belongs to another writer");
    assert_eq!(
        std::fs::read_dir(path.parent().unwrap().parent().unwrap())
            .unwrap()
            .count(),
        1
    );
}
