//! Retained synthetic engine outputs for independent external decoder checks.
use std::path::PathBuf;
use yes_master_lib::{demo, engine::*, export_format::ExportEncoding, *};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: export_formats_probe OUTPUT_DIRECTORY")?,
    );
    std::fs::create_dir(&out)?;
    let input = demo::prepare_demo_track_in(&out.join("source"))?;
    let analysis = analyze_tracks_core_with_progress_sync(
        vec![AnalyzeRequest {
            id: TrackId("export-probe".into()),
            path: input.to_string_lossy().into_owned(),
        }],
        |_, _| {},
    )?
    .remove(0);
    let mut settings = analysis.recommended_universal.clone();
    settings.advanced.source_profile = SourceProfile::from_analysis(&analysis);
    settings.advanced.bit_depth = Some(24);
    settings.advanced.target_sample_rate = Some(48_000);
    settings.eq_high_db = 2.0;
    settings.intensity = 0.8;
    let encodings = [
        ExportEncoding::Wav,
        ExportEncoding::Mp3 { bitrate_kbps: 320 },
        ExportEncoding::Flac,
        ExportEncoding::M4a { bitrate_kbps: 256 },
        ExportEncoding::Aac { bitrate_kbps: 256 },
        ExportEncoding::Ogg { quality: 6 },
        ExportEncoding::Aiff,
    ];
    let mut jobs = Vec::new();
    for encoding in encodings {
        let path = out.join("master").with_extension(encoding.extension());
        let job = mastering_render_format_with_cancel(
            analysis.track_id.clone(),
            &input,
            &settings,
            &out,
            RenderKind::Master,
            RenderJobOptions {
                output_path: Some(&path),
                ..Default::default()
            },
            encoding,
        )?;
        jobs.push(job);
    }
    std::fs::write(
        out.join("reports.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "build_stamp": env!("YES_BUILD_STAMP"), "source": input, "jobs": jobs,
        }))?,
    )?;
    println!(
        "Saved seven engine exports and reports in {}",
        out.display()
    );
    Ok(())
}
