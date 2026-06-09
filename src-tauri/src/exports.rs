use crate::types::*;

#[tauri::command]
pub async fn run_export_checks(
    report: ExportReport,
    source_analysis: Option<AnalysisResult>,
    settings: Option<MasteringSettings>,
) -> CommandResult<Vec<QualityCheck>> {
    Ok(export_checks_for_report(
        &report,
        source_analysis.as_ref(),
        settings.as_ref(),
    ))
}

pub fn export_checks_for_report(
    report: &ExportReport,
    source_analysis: Option<&AnalysisResult>,
    settings: Option<&MasteringSettings>,
) -> Vec<QualityCheck> {
    let mut checks = Vec::new();

    if report.measured_true_peak_dbtp > -0.1 {
        checks.push(QualityCheck {
            level: QualityLevel::Warning,
            code: "true_peak_high".to_string(),
            message: format!(
                "True peak is {:.2} dBTP — near or above full scale, which risks inter-sample clipping. Most delivery platforms want -1.0 dBTP or lower.",
                report.measured_true_peak_dbtp
            ),
        });
    } else if report.measured_true_peak_dbtp > -1.0 {
        // Streaming-headroom advisory for the narrow gray zone between the
        // high-true-peak -0.1 dBTP threshold above (also advisory) and the
        // typical -1.0 dBTP streaming ceiling below. Lossy codecs (AAC, MP3, Opus) can boost
        // decoded peaks by up to ~1 dB relative to the source true peak due
        // to quantization noise added inside the codec's spectral bands, so a
        // master at e.g. -0.5 dBTP can clip after AAC encoding on dense
        // pop/rock material. This is a headroom-based advisory, NOT an actual
        // codec simulation; a real Phase 6.x-bis could add an encode/decode
        // round-trip if codec QC ever needs to be more precise.
        checks.push(QualityCheck {
            level: QualityLevel::Warning,
            code: "streaming_headroom_low".to_string(),
            message: format!(
                "True peak is {:.2} dBTP. Within safe digital range, but lossy delivery (AAC, MP3, Opus) can overshoot by up to ~1 dB after encoding. Consider lowering the ceiling for comfortable streaming masters.",
                report.measured_true_peak_dbtp
            ),
        });
    }

    if report.measured_lufs > -8.0 {
        checks.push(QualityCheck {
            level: QualityLevel::Warning,
            code: "lufs_very_loud".to_string(),
            message: format!(
                "Integrated loudness is {:.1} LUFS. This is louder than typical streaming targets and may sound flat.",
                report.measured_lufs
            ),
        });
    }

    // 2026-06-09 export-metrics inquiry — landed-short-of-target advisory.
    // The LUFS landing is ceiling-bounded (engine.rs::ceiling_bounded_landing_delta_db):
    // upward gain stops when true peak reaches the delivery ceiling, so
    // high-crest material can deliver below the requested target. Surface the
    // shortfall at export time instead of leaving it discoverable only by
    // external measurement. Info-level: the ceiling protecting the master is
    // the design, not a defect. Gated on measurements_are_rendered so the
    // legacy source-analysis fallback (album path) can never compare a SOURCE
    // loudness against the delivery target.
    if report.measurements_are_rendered && report.measured_lufs.is_finite() {
        if let Some(s) = settings {
            if let Some(target) = s.effective_target_lufs() {
                let shortfall = target - report.measured_lufs;
                if shortfall > 0.25 {
                    let ceiling = s.effective_ceiling_dbtp();
                    let reason = if report.measured_true_peak_dbtp >= ceiling - 0.1 {
                        format!(
                            "the {ceiling:.1} dBTP ceiling capped the loudness push (true peak is at the ceiling)"
                        )
                    } else {
                        "the loudness landing could not push further".to_string()
                    };
                    checks.push(QualityCheck {
                        level: QualityLevel::Info,
                        code: "target_not_reached".to_string(),
                        message: format!(
                            "Delivered {:.1} LUFS — {shortfall:.1} LU below the {target:.1} LUFS target; {reason}. Use a lower target or a higher ceiling if this master should be louder.",
                            report.measured_lufs
                        ),
                    });
                }
            }
        }
    }

    if report.measured_dynamic_range_lu < 5.0 {
        checks.push(QualityCheck {
            level: QualityLevel::Warning,
            code: "dynamic_range_low".to_string(),
            message: format!(
                "Dynamic range is {:.1} LU. Highly compressed material; verify by ear before exporting.",
                report.measured_dynamic_range_lu
            ),
        });
    }

    if report.bit_depth < 16 {
        checks.push(QualityCheck {
            level: QualityLevel::Critical,
            code: "bit_depth_low".to_string(),
            message: format!(
                "Bit depth {} is below 16. Not suitable for delivery.",
                report.bit_depth
            ),
        });
    }

    if let Some(expected_sample_rate) =
        settings.and_then(MasteringSettings::requested_delivery_sample_rate)
    {
        if report.sample_rate != expected_sample_rate {
            checks.push(QualityCheck {
                level: QualityLevel::Critical,
                code: "sample_rate_mismatch".to_string(),
                message: format!(
                    "Rendered sample rate {} Hz does not match the requested delivery rate {} Hz.",
                    report.sample_rate, expected_sample_rate
                ),
            });
        }
    }

    if !report.measured_lufs.is_finite() {
        checks.push(QualityCheck {
            level: QualityLevel::Critical,
            code: "non_finite_metering".to_string(),
            message: "LUFS measurement is not finite. Re-analyze before exporting.".to_string(),
        });
    }

    // Phase 12.2 — already-compressed source advisory. Fires when the SOURCE
    // material is dynamically squashed (DR < 6 LU) AND Preset mode is asking
    // for moderate-to-heavy compression density (> 0.3). Manual mode is an
    // explicit user decision; Off mode bypasses the creative compressor.
    // Advisory only — does not block export.
    if let (Some(analysis), Some(s)) = (source_analysis, settings) {
        let default_density = if matches!(s.preset, Preset::Custom { .. }) {
            0.0
        } else {
            0.5
        };
        let density = s.advanced.compression_density.unwrap_or(default_density);
        if analysis.dynamic_range_lu < 6.0
            && density > 0.3
            && matches!(s.advanced.compression_mode, CompressionMode::Preset)
        {
            checks.push(QualityCheck {
                level: QualityLevel::Warning,
                code: "comp_density_on_compressed_source".to_string(),
                message:
                    "Source appears already compressed (DR < 6 LU). Heavy compression may pump."
                        .to_string(),
            });
        }
    }

    if checks.is_empty() {
        checks.push(QualityCheck {
            level: QualityLevel::Info,
            code: "export_ok".to_string(),
            message: "No issues detected in measured values.".to_string(),
        });
    }

    checks
}

#[tauri::command]
pub async fn open_output(output_path: String) -> CommandResult<()> {
    if output_path.is_empty() {
        return Err(CommandError::InvalidPath("empty path".to_string()));
    }
    let path = std::path::Path::new(&output_path);
    if crate::files::has_parent_dir_component(path) {
        return Err(CommandError::InvalidPath(format!(
            "path traversal not allowed: {output_path}"
        )));
    }
    if !path.exists() {
        return Err(CommandError::Io(format!(
            "path does not exist: {output_path}"
        )));
    }

    #[cfg(target_os = "windows")]
    {
        // /select, opens Explorer at the parent folder with the file highlighted.
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&output_path)
            .spawn()
            .map_err(|e| CommandError::Io(format!("failed to open Explorer: {e}")))?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&output_path)
            .spawn()
            .map_err(|e| CommandError::Io(format!("failed to open Finder: {e}")))?;
        Ok(())
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let parent = path.parent().unwrap_or(path);
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| CommandError::Io(format!("failed to open file manager: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(target_lufs: Option<f32>, ceiling_dbtp: Option<f32>) -> MasteringSettings {
        MasteringSettings {
            preset: Preset::Universal,
            intensity: 0.5,
            eq_sub_db: 0.0,
            eq_low_db: 0.0,
            eq_low_mid_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_mid_db: 0.0,
            eq_high_db: 0.0,
            eq_sparkle_db: 0.0,
            volume_match: false,
            source_lufs_integrated: None,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            delivery_profile: DeliveryProfile::Custom,
            album: None,
            advanced: AdvancedSettings {
                lufs_offset_db: target_lufs,
                ceiling_dbtp,
                ..Default::default()
            },
        }
    }

    fn report(measured_lufs: f32, measured_tp: f32, rendered: bool) -> ExportReport {
        ExportReport {
            track_id: TrackId("t1".to_string()),
            output_path: "C:/out/master.wav".to_string(),
            measured_lufs,
            measured_true_peak_dbtp: measured_tp,
            measured_dynamic_range_lu: 10.9,
            source_format: "wav".to_string(),
            destination_format: "wav".to_string(),
            sample_rate: 96_000,
            bit_depth: 32,
            effective_adaptive_strength: 0.5,
            source_profile_digest: None,
            confidence_digest: None,
            measurements_are_rendered: rendered,
            checks: Vec::new(),
        }
    }

    fn codes(checks: &[QualityCheck]) -> Vec<&str> {
        checks.iter().map(|c| c.code.as_str()).collect()
    }

    /// The inquiry scenario: -9 target, -1 ceiling, delivered -10.26 LUFS with
    /// TP parked at the ceiling -> Info note naming the shortfall and reason.
    #[test]
    fn landed_short_at_ceiling_fires_info_note() {
        let s = settings(Some(-9.0), Some(-1.0));
        let checks = export_checks_for_report(&report(-10.26, -0.97, true), None, Some(&s));
        let note = checks
            .iter()
            .find(|c| c.code == "target_not_reached")
            .expect("shortfall note must fire");
        assert!(matches!(note.level, QualityLevel::Info), "{:?}", note.level);
        assert!(note.message.contains("-10.3 LUFS"), "{}", note.message);
        assert!(note.message.contains("1.3 LU below"), "{}", note.message);
        assert!(
            note.message.contains("-9.0 LUFS target"),
            "{}",
            note.message
        );
        assert!(note.message.contains("ceiling capped"), "{}", note.message);
    }

    /// Landing on target (within the 0.25 LU band) stays quiet.
    #[test]
    fn on_target_no_note() {
        let s = settings(Some(-9.0), Some(-1.0));
        let checks = export_checks_for_report(&report(-9.05, -1.2, true), None, Some(&s));
        assert!(
            !codes(&checks).contains(&"target_not_reached"),
            "{checks:?}"
        );
    }

    /// Delivering louder than target (downward landing applied in full) never
    /// reads as a shortfall.
    #[test]
    fn louder_than_target_no_note() {
        let s = settings(Some(-9.0), Some(-1.0));
        let checks = export_checks_for_report(&report(-8.0, -1.4, true), None, Some(&s));
        assert!(
            !codes(&checks).contains(&"target_not_reached"),
            "{checks:?}"
        );
    }

    /// The legacy source-analysis fallback (album path) must never compare a
    /// SOURCE loudness against the delivery target.
    #[test]
    fn source_fallback_never_fires() {
        let s = settings(Some(-9.0), Some(-1.0));
        let checks = export_checks_for_report(&report(-16.3, -3.9, false), None, Some(&s));
        assert!(
            !codes(&checks).contains(&"target_not_reached"),
            "{checks:?}"
        );
    }

    /// No effective target (Custom profile, no advanced offset) -> the landing
    /// never ran, so there is no target to miss.
    #[test]
    fn no_target_no_note() {
        let s = settings(None, Some(-1.0));
        let checks = export_checks_for_report(&report(-16.3, -3.9, true), None, Some(&s));
        assert!(
            !codes(&checks).contains(&"target_not_reached"),
            "{checks:?}"
        );
    }
}
