//! New-format Album delivery shares the existing bounded PCM assembler.
use crate::engine::{AlbumPlanRenderRequest, AlbumRenderReport};
use crate::export_format::ExportEncoding;
use crate::types::{CommandError, CommandResult, JobStatus};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

fn error(e: impl std::fmt::Display) -> CommandError {
    CommandError::Render(format!("Album export: {e}"))
}
fn check_cancel(cancel: Option<&AtomicBool>) -> CommandResult<()> {
    if crate::engine::render_cancelled(cancel) {
        Err(error("cancelled"))
    } else {
        Ok(())
    }
}

pub(crate) fn requested_bit_depth(request: &AlbumPlanRenderRequest) -> u16 {
    request.plan.delivery_bit_depth.unwrap_or_else(|| {
        request
            .plan
            .tracks
            .first()
            .and_then(|entry| {
                request
                    .tracks
                    .iter()
                    .find(|track| track.track_id == entry.track_id)
            })
            .map(|track| track.settings.effective_bit_depth())
            .unwrap_or(24)
    })
}

pub fn render_album(
    request: &AlbumPlanRenderRequest,
    out_dir: &Path,
    progress: Option<&dyn Fn(f32)>,
    cancel: Option<&AtomicBool>,
    job_id: Option<&str>,
    encoding: ExportEncoding,
) -> CommandResult<AlbumRenderReport> {
    encoding.validate()?;
    if !encoding.needs_sidecar() {
        return crate::mp3::render_album(
            request,
            out_dir,
            progress,
            cancel,
            job_id,
            match encoding {
                ExportEncoding::Mp3 { bitrate_kbps } => Some(bitrate_kbps),
                _ => None,
            },
        );
    }
    let encoder = crate::export_encoding::Encoder::packaged()?;
    crate::engine::validate_album_source_paths(&request.tracks)?;
    if crate::files::has_parent_dir_component(out_dir) {
        return Err(error("Invalid output directory"));
    }
    let requested_rate = match request.plan.delivery_sample_rate {
        Some(rate) => rate,
        None => request
            .tracks
            .iter()
            .map(|track| crate::decode::probe_sample_rate(Path::new(&track.source_path)))
            .collect::<CommandResult<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(48_000),
    };
    let requested_bits = requested_bit_depth(request);
    if !matches!(requested_bits, 16 | 24 | 32) {
        return Err(error("Unsupported requested precision"));
    }
    let mut staged_request = request.clone();
    staged_request.plan.delivery_sample_rate = Some(encoding.delivery_rate(requested_rate));
    staged_request.plan.delivery_bit_depth = Some(encoding.delivery_bits(requested_bits));
    let scratch = tempfile::tempdir().map_err(error)?;
    let staged_progress = |fraction: f32| {
        if let Some(cb) = progress {
            cb(fraction * 0.8);
        }
    };
    let mut report = crate::album_render::render_album_plan_impl_with_cancel(
        &staged_request,
        scratch.path(),
        Some(&staged_progress),
        cancel,
        job_id,
    )?;
    if matches!(report.status, JobStatus::Cancelled) {
        return Ok(report);
    }
    std::fs::create_dir_all(out_dir).map_err(error)?;
    let destination = crate::album_render::unique_export_subdir(out_dir, &request.plan.title)?;
    let metadata = destination.join("metadata");
    let mut written = Vec::<PathBuf>::new();
    let result = (|| {
        std::fs::create_dir(&metadata).map_err(error)?;
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report.manifest_path).map_err(error)?)
                .map_err(error)?;
        let total = report.tracks.len() + 1;
        for (index, track) in report.tracks.iter_mut().enumerate() {
            check_cancel(cancel)?;
            let input = Path::new(&track.output_path);
            let output = destination
                .join(
                    input
                        .file_name()
                        .ok_or_else(|| error("Missing track filename"))?,
                )
                .with_extension(encoding.extension());
            let delivered =
                crate::export_encoding::deliver_staged(&encoder, input, &output, encoding, cancel)?;
            written.push(delivered.path.clone());
            track.output_path = delivered.path.to_string_lossy().into_owned();
            track.measured_lufs = delivered.measurements.0;
            track.true_peak_dbtp = delivered.measurements.1;
            if let Some(cb) = progress {
                cb(0.8 + 0.2 * (index + 1) as f32 / total as f32);
            }
        }
        check_cancel(cancel)?;
        // One encode of the assembled programme, never concatenated/re-encoded
        // lossy tracks. Integer staging keeps the assembler's quantization once.
        let delivered = crate::export_encoding::deliver_staged(
            &encoder,
            Path::new(&report.album_wav_path),
            &destination
                .join("album")
                .with_extension(encoding.extension()),
            encoding,
            cancel,
        )?;
        written.push(delivered.path.clone());
        report.album_wav_path = delivered.path.to_string_lossy().into_owned();
        report.delivered_format = Some(encoding.delivered(
            report.rendered_sample_rate,
            report.rendered_channels,
            report.bit_depth,
            request.plan.delivery_sample_rate,
            requested_bits,
        ));
        report.requested_sample_rate = request.plan.delivery_sample_rate;
        report.bit_depth = if encoding.is_lossy() {
            0
        } else {
            report.bit_depth
        };
        report.manifest_path = metadata
            .join("manifest.json")
            .to_string_lossy()
            .into_owned();
        manifest["format"] = serde_json::json!(encoding.extension());
        manifest["delivered_format"] =
            serde_json::to_value(&report.delivered_format).map_err(error)?;
        manifest["bit_depth"] = if encoding.is_lossy() {
            serde_json::Value::Null
        } else {
            serde_json::json!(report.bit_depth)
        };
        manifest["plan"] = serde_json::to_value(&request.plan).map_err(error)?;
        manifest["album_wav_path"] = serde_json::json!(report.album_wav_path);
        manifest["album_measurements"] = serde_json::json!({"lufs_integrated": delivered.measurements.0,
            "true_peak_dbtp": delivered.measurements.1, "dynamic_range_lu": delivered.measurements.2});
        manifest["tracks"] = serde_json::to_value(&report.tracks).map_err(error)?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&report.manifest_path)
            .map_err(error)?;
        written.push(PathBuf::from(&report.manifest_path));
        file.write_all(&serde_json::to_vec_pretty(&manifest).map_err(error)?)
            .map_err(error)?;
        file.sync_all().map_err(error)?;
        check_cancel(cancel)?;
        if let Some(cb) = progress {
            cb(1.0);
        }
        check_cancel(cancel)?;
        Ok(())
    })();
    if let Err(err) = result {
        for path in written {
            let _ = std::fs::remove_file(path);
        }
        let _ = std::fs::remove_dir(metadata);
        let _ = std::fs::remove_dir(destination);
        if crate::engine::render_cancelled(cancel) {
            report.status = JobStatus::Cancelled;
            report.album_wav_path.clear();
            report.manifest_path.clear();
            report.tracks.clear();
            report.delivered_format = None;
            return Ok(report);
        }
        return Err(err);
    }
    Ok(report)
}
