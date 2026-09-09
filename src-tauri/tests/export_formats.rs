#![cfg(feature = "app-runner")]
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use yes_master_lib::{engine::*, export_encoding, export_format::ExportEncoding, *};
mod common;

fn source(path: &Path, rate: u32, channels: u16, frames: u32) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for n in 0..frames {
        for ch in 0..channels {
            writer
                .write_sample(
                    (n as f32 * (431.0 + 173.0 * ch as f32) * std::f32::consts::TAU / rate as f32)
                        .sin()
                        * 0.12,
                )
                .unwrap();
        }
    }
    writer.finalize().unwrap();
}

fn encodings() -> [ExportEncoding; 5] {
    [
        ExportEncoding::Flac,
        ExportEncoding::Aiff,
        ExportEncoding::M4a { bitrate_kbps: 256 },
        ExportEncoding::Aac { bitrate_kbps: 256 },
        ExportEncoding::Ogg { quality: 6 },
    ]
}

#[cfg(windows)]
#[test]
#[ignore = "requires the staged qualified encoder package"]
fn new_formats_preserve_windows_long_destination_paths() {
    let temp = tempfile::tempdir().unwrap();
    let encoder = export_encoding::Encoder::packaged().unwrap();
    let mut deep = temp.path().to_path_buf();
    while deep.as_os_str().len() < 280 {
        deep.push("long-export-folder-with-spaces-0123456789");
    }
    std::fs::create_dir_all(&deep).unwrap();
    let samples: Vec<_> = (0..44100).map(|n| (n as f32 * 0.1).sin() * 0.2).collect();
    for encoding in encodings() {
        let path = deep.join("master").with_extension(encoding.extension());
        let result =
            export_encoding::write(&encoder, &path, &samples, 44100, 1, 24, encoding, None)
                .unwrap();
        assert!(result.path.exists());
        let first = std::fs::read(&result.path).unwrap();
        let repeat =
            export_encoding::write(&encoder, &path, &samples, 44100, 1, 24, encoding, None)
                .unwrap();
        assert_ne!(result.path, repeat.path);
        assert_eq!(std::fs::read(&result.path).unwrap(), first);
    }
}

#[test]
#[ignore = "requires the staged qualified encoder package"]
fn track_matrix_preserves_mastering_quantization_and_measures_delivered_files() {
    let temp = tempfile::tempdir().unwrap();
    let encoder = export_encoding::Encoder::packaged().unwrap();
    let input = temp.path().join("源 studio.wav");
    source(&input, 96_000, 2, 96_137);
    let original = std::fs::read(&input).unwrap();
    for requested_bits in [16, 24, 32] {
        for encoding in encodings() {
            let mut settings = common::default_master_settings();
            settings.eq_high_db = 3.0;
            settings.intensity = 0.8;
            settings.advanced.bit_depth = Some(requested_bits);
            settings.advanced.target_sample_rate = Some(96_000);
            let output = temp
                .path()
                .join(format!("master-{requested_bits}.{}", encoding.extension()));
            let job = mastering_render_format_with_cancel(
                TrackId("track".into()),
                &input,
                &settings,
                temp.path(),
                RenderKind::Master,
                RenderJobOptions {
                    output_path: Some(&output),
                    ..Default::default()
                },
                encoding,
            )
            .unwrap();
            let facts = job.delivered_format.as_ref().unwrap();
            assert_eq!(facts.encoding, encoding);
            assert_eq!(facts.requested_bit_depth, requested_bits);
            assert_eq!(
                facts.bit_depth,
                if encoding.is_lossy() {
                    None
                } else {
                    Some(requested_bits.min(24))
                }
            );
            let mut reference_settings = settings.clone();
            reference_settings.advanced.target_sample_rate = Some(encoding.delivery_rate(96_000));
            reference_settings.advanced.bit_depth = Some(encoding.delivery_bits(requested_bits));
            let wav = mastering_render_with_cancel(
                TrackId("reference".into()),
                &input,
                &reference_settings,
                temp.path(),
                RenderKind::Master,
                RenderJobOptions::default(),
            )
            .unwrap();
            let reference = export_encoding::deliver_staged(
                &encoder,
                Path::new(&wav.output_paths[0]),
                &temp.path().join(format!(
                    "reference-{requested_bits}.{}",
                    encoding.extension()
                )),
                encoding,
                None,
            )
            .unwrap();
            let actual_pcm = decode::decode_full(&output).unwrap();
            let expected_pcm = decode::decode_full(&reference.path).unwrap();
            assert_eq!(
                actual_pcm.samples, expected_pcm.samples,
                "Mastered PCM differs for {encoding:?}/{requested_bits}"
            );
            let measured = export_encoding::measure(&encoder, &output, None)
                .unwrap()
                .measurements;
            let receipt = job.measurements.unwrap();
            assert_eq!(
                (
                    receipt.lufs_integrated,
                    receipt.true_peak_dbtp,
                    receipt.dynamic_range_lu
                ),
                measured
            );
            settings.volume_match = true;
            let second = mastering_render_format_with_cancel(
                TrackId("track".into()),
                &input,
                &settings,
                temp.path(),
                RenderKind::Master,
                RenderJobOptions {
                    output_path: Some(&output),
                    ..Default::default()
                },
                encoding,
            )
            .unwrap();
            assert_ne!(second.output_paths[0], job.output_paths[0]);
            assert_eq!(
                decode::decode_full(Path::new(&second.output_paths[0]))
                    .unwrap()
                    .samples,
                actual_pcm.samples,
                "Volume Match changed export"
            );
            assert_eq!(
                decode::decode_full(&output).unwrap().samples,
                actual_pcm.samples,
                "Prior render overwritten"
            );
        }
    }
    assert_eq!(std::fs::read(&input).unwrap(), original);
    assert!(!std::fs::read_dir(temp.path()).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".yes-export-")));
}

#[test]
#[ignore = "requires the staged qualified encoder package"]
fn cancelled_and_invalid_exports_leave_source_and_existing_destination_intact() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("source.wav");
    source(&input, 44_100, 1, 257);
    let original = std::fs::read(&input).unwrap();
    let settings = common::default_master_settings();
    for encoding in encodings() {
        let cancel = AtomicBool::new(true);
        let output = temp
            .path()
            .join("prior")
            .with_extension(encoding.extension());
        std::fs::write(&output, b"prior output").unwrap();
        let job = mastering_render_format_with_cancel(
            TrackId("track".into()),
            &input,
            &settings,
            temp.path(),
            RenderKind::Master,
            RenderJobOptions {
                output_path: Some(&output),
                cancel_flag: Some(&cancel),
                ..Default::default()
            },
            encoding,
        )
        .unwrap();
        assert!(matches!(job.status, JobStatus::Cancelled));
        assert!(job.output_paths.is_empty());
        assert_eq!(std::fs::read(&output).unwrap(), b"prior output");
        cancel.store(false, Ordering::Relaxed);
        assert!(mastering_render_format_with_cancel(
            TrackId("track".into()),
            &input,
            &settings,
            temp.path(),
            RenderKind::Master,
            RenderJobOptions {
                output_path: Some(&input),
                ..Default::default()
            },
            encoding
        )
        .is_err());
    }
    assert_eq!(std::fs::read(&input).unwrap(), original);
}

#[test]
#[ignore = "requires the staged qualified encoder package"]
fn quality_rate_channel_short_and_silent_delivery_matrix() {
    let temp = tempfile::tempdir().unwrap();
    let encoder = export_encoding::Encoder::packaged().unwrap();
    let lossy: Vec<_> = [128, 192, 256, 320]
        .into_iter()
        .flat_map(|bitrate_kbps| {
            [
                ExportEncoding::M4a { bitrate_kbps },
                ExportEncoding::Aac { bitrate_kbps },
            ]
        })
        .chain(
            [4, 6, 8]
                .into_iter()
                .map(|quality| ExportEncoding::Ogg { quality }),
        )
        .collect();
    for rate in [44_100, 48_000] {
        for channels in [1, 2] {
            for silence in [false, true] {
                let samples: Vec<f32> = (0..257)
                    .flat_map(|n| {
                        (0..channels).map(move |ch| {
                            if silence {
                                0.0
                            } else {
                                (n as f32 * (431.0 + ch as f32 * 173.0) * std::f32::consts::TAU
                                    / rate as f32)
                                    .sin()
                                    * 0.2
                            }
                        })
                    })
                    .collect();
                for encoding in &lossy {
                    let result = export_encoding::write(
                        &encoder,
                        &temp
                            .path()
                            .join("short")
                            .with_extension(encoding.extension()),
                        &samples,
                        rate,
                        channels,
                        32,
                        *encoding,
                        None,
                    )
                    .unwrap();
                    assert!(result.measurements.0.is_finite() && result.measurements.1.is_finite());
                }
            }
        }
    }
    for rate in [44_100, 48_000, 96_000, 192_000, 384_000] {
        for bits in [16, 24] {
            for encoding in [ExportEncoding::Flac, ExportEncoding::Aiff] {
                let samples: Vec<_> = (0..257).map(|n| (n as f32 * 0.1).sin() * 0.2).collect();
                let result = export_encoding::write(
                    &encoder,
                    &temp
                        .path()
                        .join("integer")
                        .with_extension(encoding.extension()),
                    &samples,
                    rate,
                    1,
                    bits,
                    encoding,
                    None,
                )
                .unwrap();
                let pcm = decode::decode_full(&result.path).unwrap();
                assert_eq!(pcm.samples.len(), 257);
                assert_eq!(pcm.sample_rate, rate);
            }
        }
    }
}
