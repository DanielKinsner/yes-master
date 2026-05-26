use crate::analysis::analyze_one;
use crate::engine::mastering_render_to_path;
use crate::exports::export_checks_for_report;
use crate::types::{
    AnalysisResult, CommandError, CommandResult, CompressionMode, ExportReport, MasteringSettings,
    Preset, QualityCheck, QualityLevel, RenderKind, RenderedMeasurements, TrackId,
};
use chrono::SecondsFormat;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceTarget {
    pub preset: Preset,
    pub reference_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceSuite {
    pub track_label: String,
    pub source_path: PathBuf,
    pub references: Vec<ReferenceTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLedgerRow {
    pub track_label: String,
    pub preset: Preset,
    pub source_lufs: f32,
    pub reference_lufs: f32,
    pub yes_lufs: f32,
    pub reference_lufs_delta_db: f32,
    pub yes_lufs_delta_db: f32,
    pub lufs_gap_db: f32,
    pub source_dynamic_range_lu: f32,
    pub reference_dynamic_range_lu: f32,
    pub yes_dynamic_range_lu: f32,
    pub reference_dynamic_range_delta_lu: f32,
    pub yes_dynamic_range_delta_lu: f32,
    pub dynamic_range_gap_lu: f32,
    pub spectral_low_gap: f32,
    pub spectral_mid_gap: f32,
    pub spectral_high_gap: f32,
    pub transient_density_gap: f32,
    pub stereo_width_gap: f32,
    pub energy_density_gap: Option<f32>,
    pub transient_flux_gap: Option<f32>,
    pub stereo_correlation_gap: Option<f32>,
    pub warning_codes: Vec<String>,
    pub reference_path: String,
    pub yes_output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceTuningReport {
    pub generated_at_iso: String,
    pub reference_dir: String,
    pub output_dir: String,
    pub track_label: String,
    pub source_path: String,
    pub rows: Vec<ReferenceLedgerRow>,
}

#[derive(Debug, Clone)]
pub struct ReferenceTuningRunResult {
    pub report_path: PathBuf,
    pub csv_path: PathBuf,
    pub render_dir: PathBuf,
    pub rows: Vec<ReferenceLedgerRow>,
}

pub fn default_reference_presets() -> Vec<Preset> {
    vec![
        Preset::Universal,
        Preset::Clarity,
        Preset::Oomph,
        Preset::Tape,
    ]
}

pub fn discover_reference_suite(reference_dir: &Path) -> CommandResult<ReferenceSuite> {
    let reference_dir = fs::canonicalize(reference_dir)
        .map_err(|e| CommandError::Io(format!("canonicalize reference dir: {e}")))?;
    let mut source: Option<(String, PathBuf)> = None;
    for entry in
        fs::read_dir(&reference_dir).map_err(|e| CommandError::Io(format!("read dir: {e}")))?
    {
        let entry = entry.map_err(|e| CommandError::Io(format!("read dir entry: {e}")))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(track_label) = file_name.strip_suffix("-original-test.wav") else {
            continue;
        };
        if source.is_some() {
            return Err(CommandError::Other(format!(
                "multiple original reference sources found in {}",
                reference_dir.display()
            )));
        }
        source = Some((track_label.to_string(), path));
    }

    let (track_label, source_path) = source.ok_or_else(|| {
        CommandError::Other(format!(
            "no *-original-test.wav source found in {}",
                reference_dir.display()
        ))
    })?;
    let references = default_reference_presets()
        .into_iter()
        .map(|preset| {
            let reference_path =
                reference_dir.join(format!("{}-{}-test.wav", track_label, preset_slug(&preset)));
            if !reference_path.is_file() {
                return Err(CommandError::InvalidPath(format!(
                    "missing reference master for {}: {}",
                    preset_slug(&preset),
                    reference_path.display()
                )));
            }
            Ok(ReferenceTarget {
                preset,
                reference_path,
            })
        })
        .collect::<CommandResult<Vec<_>>>()?;

    Ok(ReferenceSuite {
        track_label,
        source_path,
        references,
    })
}

pub fn comparison_row_for(
    track_label: &str,
    preset: Preset,
    source: &AnalysisResult,
    reference: &AnalysisResult,
    yes_render: &AnalysisResult,
    reference_path: &Path,
    yes_output_path: &Path,
    checks: &[QualityCheck],
) -> ReferenceLedgerRow {
    ReferenceLedgerRow {
        track_label: track_label.to_string(),
        preset,
        source_lufs: source.lufs_integrated,
        reference_lufs: reference.lufs_integrated,
        yes_lufs: yes_render.lufs_integrated,
        reference_lufs_delta_db: reference.lufs_integrated - source.lufs_integrated,
        yes_lufs_delta_db: yes_render.lufs_integrated - source.lufs_integrated,
        lufs_gap_db: yes_render.lufs_integrated - reference.lufs_integrated,
        source_dynamic_range_lu: source.dynamic_range_lu,
        reference_dynamic_range_lu: reference.dynamic_range_lu,
        yes_dynamic_range_lu: yes_render.dynamic_range_lu,
        reference_dynamic_range_delta_lu: reference.dynamic_range_lu - source.dynamic_range_lu,
        yes_dynamic_range_delta_lu: yes_render.dynamic_range_lu - source.dynamic_range_lu,
        dynamic_range_gap_lu: yes_render.dynamic_range_lu - reference.dynamic_range_lu,
        spectral_low_gap: yes_render.spectral_balance.low - reference.spectral_balance.low,
        spectral_mid_gap: yes_render.spectral_balance.mid - reference.spectral_balance.mid,
        spectral_high_gap: yes_render.spectral_balance.high - reference.spectral_balance.high,
        transient_density_gap: yes_render.transient_density - reference.transient_density,
        stereo_width_gap: yes_render.stereo_width - reference.stereo_width,
        energy_density_gap: option_gap(
            yes_render.energy_density_score,
            reference.energy_density_score,
        ),
        transient_flux_gap: option_gap(yes_render.transient_flux, reference.transient_flux),
        stereo_correlation_gap: option_gap(
            yes_render.stereo_correlation,
            reference.stereo_correlation,
        ),
        warning_codes: checks
            .iter()
            .filter(|check| !matches!(check.level, QualityLevel::Info))
            .map(|check| check.code.clone())
            .collect(),
        reference_path: reference_path.to_string_lossy().to_string(),
        yes_output_path: yes_output_path.to_string_lossy().to_string(),
    }
}

pub fn run_reference_tuning_dir(
    reference_dir: &Path,
    output_dir: &Path,
) -> CommandResult<ReferenceTuningRunResult> {
    let suite = discover_reference_suite(reference_dir)?;
    let source_analysis = analyze_one(
        TrackId(format!("{}-source", sanitize_path_part(&suite.track_label))),
        &suite.source_path,
    )?;
    let render_dir = output_dir.join("renders");
    fs::create_dir_all(&render_dir)
        .map_err(|e| CommandError::Io(format!("create reference tuning renders: {e}")))?;

    let mut rows = Vec::new();
    for target in &suite.references {
        let slug = preset_slug(&target.preset);
        let reference_analysis = analyze_one(
            TrackId(format!(
                "{}-{}-reference",
                sanitize_path_part(&suite.track_label),
                slug
            )),
            &target.reference_path,
        )?;
        let settings = settings_for_reference_preset(&source_analysis, target.preset.clone());
        let yes_output_path = render_dir.join(format!(
            "{}-{}-yes-master.wav",
            sanitize_path_part(&suite.track_label),
            slug
        ));
        let job = mastering_render_to_path(
            TrackId(format!("{}-{}-yes", suite.track_label, slug)),
            &suite.source_path,
            &settings,
            &render_dir,
            RenderKind::Master,
            &yes_output_path,
        )?;
        let rendered = job.measurements.ok_or_else(|| {
            CommandError::Render("reference tuning render did not return measurements".to_string())
        })?;
        let yes_analysis = analyze_one(
            TrackId(format!(
                "{}-{}-yes-analysis",
                sanitize_path_part(&suite.track_label),
                slug
            )),
            &yes_output_path,
        )?;
        let export_report = export_report_for(
            &source_analysis.track_id,
            &yes_output_path,
            &rendered,
            suite
                .source_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("audio"),
        );
        let checks =
            export_checks_for_report(&export_report, Some(&source_analysis), Some(&settings));
        rows.push(comparison_row_for(
            &suite.track_label,
            target.preset.clone(),
            &source_analysis,
            &reference_analysis,
            &yes_analysis,
            &target.reference_path,
            &yes_output_path,
            &checks,
        ));
    }

    fs::create_dir_all(output_dir)
        .map_err(|e| CommandError::Io(format!("create reference tuning output: {e}")))?;
    let report = ReferenceTuningReport {
        generated_at_iso: now_iso(),
        reference_dir: reference_dir.to_string_lossy().to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        track_label: suite.track_label.clone(),
        source_path: suite.source_path.to_string_lossy().to_string(),
        rows: rows.clone(),
    };
    let report_path = output_dir.join("reference-tuning-report.json");
    let csv_path = output_dir.join("reference-tuning-report.csv");
    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|e| CommandError::Other(format!("serialize reference report: {e}")))?;
    fs::write(&report_path, report_json)
        .map_err(|e| CommandError::Io(format!("write reference report: {e}")))?;
    fs::write(&csv_path, ledger_csv(&rows))
        .map_err(|e| CommandError::Io(format!("write reference csv: {e}")))?;

    Ok(ReferenceTuningRunResult {
        report_path,
        csv_path,
        render_dir,
        rows,
    })
}

pub fn settings_for_reference_preset(
    source_analysis: &AnalysisResult,
    preset: Preset,
) -> MasteringSettings {
    let mut settings = source_analysis.recommended_universal.clone();
    settings.preset = preset;
    settings.volume_match = false;
    settings.source_lufs_integrated = Some(source_analysis.lufs_integrated);
    settings.advanced.compression_mode = CompressionMode::Preset;
    settings
}

fn preset_slug(preset: &Preset) -> &'static str {
    match preset {
        Preset::Universal => "universal",
        Preset::Clarity => "clarity",
        Preset::Tape => "tape",
        Preset::Oomph => "oomph",
        _ => "unsupported",
    }
}

fn option_gap(yes_value: Option<f32>, reference_value: Option<f32>) -> Option<f32> {
    Some(yes_value? - reference_value?)
}

fn export_report_for(
    track_id: &TrackId,
    output_path: &Path,
    rendered: &RenderedMeasurements,
    source_format: &str,
) -> ExportReport {
    ExportReport {
        track_id: track_id.clone(),
        output_path: output_path.to_string_lossy().to_string(),
        measured_lufs: rendered.lufs_integrated,
        measured_true_peak_dbtp: rendered.true_peak_dbtp,
        measured_dynamic_range_lu: rendered.dynamic_range_lu,
        source_format: source_format.to_string(),
        destination_format: "wav".to_string(),
        sample_rate: rendered.sample_rate,
        bit_depth: rendered.bit_depth,
        checks: Vec::new(),
    }
}

fn ledger_csv(rows: &[ReferenceLedgerRow]) -> String {
    let mut out = String::from(
        "track_label,preset,source_lufs,reference_lufs,yes_lufs,reference_lufs_delta_db,yes_lufs_delta_db,lufs_gap_db,source_dynamic_range_lu,reference_dynamic_range_lu,yes_dynamic_range_lu,reference_dynamic_range_delta_lu,yes_dynamic_range_delta_lu,dynamic_range_gap_lu,spectral_low_gap,spectral_mid_gap,spectral_high_gap,transient_density_gap,stereo_width_gap,energy_density_gap,transient_flux_gap,stereo_correlation_gap,warning_codes,reference_path,yes_output_path\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{},{},{}\n",
            csv_escape(&row.track_label),
            csv_escape(preset_slug(&row.preset)),
            row.source_lufs,
            row.reference_lufs,
            row.yes_lufs,
            row.reference_lufs_delta_db,
            row.yes_lufs_delta_db,
            row.lufs_gap_db,
            row.source_dynamic_range_lu,
            row.reference_dynamic_range_lu,
            row.yes_dynamic_range_lu,
            row.reference_dynamic_range_delta_lu,
            row.yes_dynamic_range_delta_lu,
            row.dynamic_range_gap_lu,
            row.spectral_low_gap,
            row.spectral_mid_gap,
            row.spectral_high_gap,
            row.transient_density_gap,
            row.stereo_width_gap,
            csv_option(row.energy_density_gap),
            csv_option(row.transient_flux_gap),
            csv_option(row.stereo_correlation_gap),
            csv_escape(&row.warning_codes.join("|")),
            csv_escape(&row.reference_path),
            csv_escape(&row.yes_output_path),
        ));
    }
    out
}

fn csv_option(value: Option<f32>) -> String {
    value.map(|value| format!("{value:.4}")).unwrap_or_default()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn sanitize_path_part(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AdvancedSettings, DeliveryProfile, MasteringSettings, QualityCheck, QualityLevel,
        SpectralBalance, TrackId, ISO_PLACEHOLDER,
    };

    #[test]
    fn default_reference_presets_cover_private_site_reference_set() {
        assert_eq!(
            default_reference_presets(),
            vec![
                Preset::Universal,
                Preset::Clarity,
                Preset::Oomph,
                Preset::Tape,
            ]
        );
    }

    #[test]
    fn discovers_original_and_named_reference_masters_from_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        for name in [
            "Coat-original-test.wav",
            "Coat-universal-test.wav",
            "Coat-clarity-test.wav",
            "Coat-oomph-test.wav",
            "Coat-tape-test.wav",
        ] {
            std::fs::write(dir.path().join(name), []).expect("fixture marker");
        }

        let suite = discover_reference_suite(dir.path()).expect("reference suite");

        assert_eq!(suite.track_label, "Coat");
        assert_eq!(
            suite.source_path.file_name().and_then(|name| name.to_str()),
            Some("Coat-original-test.wav")
        );
        assert!(suite.source_path.is_absolute());
        assert!(!suite.source_path.to_string_lossy().contains(".."));
        assert_eq!(
            suite
                .references
                .iter()
                .map(|target| target.preset.clone())
                .collect::<Vec<_>>(),
            vec![
                Preset::Universal,
                Preset::Clarity,
                Preset::Oomph,
                Preset::Tape,
            ]
        );
        assert_eq!(
            suite.references[2]
                .reference_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some("Coat-oomph-test.wav")
        );
    }

    #[test]
    fn discovery_normalizes_relative_parent_components_in_reference_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let refs = dir.path().join("refs");
        std::fs::create_dir(&refs).expect("refs dir");
        for name in [
            "Coat-original-test.wav",
            "Coat-universal-test.wav",
            "Coat-clarity-test.wav",
            "Coat-oomph-test.wav",
            "Coat-tape-test.wav",
        ] {
            std::fs::write(refs.join(name), []).expect("fixture marker");
        }
        let traversed_refs = dir.path().join("refs").join("..").join("refs");

        let suite = discover_reference_suite(&traversed_refs).expect("reference suite");

        assert_eq!(
            suite.source_path.file_name().and_then(|name| name.to_str()),
            Some("Coat-original-test.wav")
        );
        assert!(suite.source_path.is_absolute());
        assert!(!suite.source_path.to_string_lossy().contains(".."));
        assert!(!suite.references[0]
            .reference_path
            .to_string_lossy()
            .contains(".."));
    }

    fn base_settings() -> MasteringSettings {
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
            delivery_profile: DeliveryProfile::StreamingUniversal,
            album: None,
            advanced: AdvancedSettings::default(),
        }
    }

    fn analysis(
        id: &str,
        lufs: f32,
        dynamic_range: f32,
        spectral: SpectralBalance,
        transient_density: f32,
        stereo_width: f32,
    ) -> AnalysisResult {
        AnalysisResult {
            track_id: TrackId(id.to_string()),
            lufs_integrated: lufs,
            lufs_short_term_max: lufs + 2.0,
            true_peak_dbtp: -1.0,
            dynamic_range_lu: dynamic_range,
            spectral_balance: spectral,
            transient_density,
            stereo_width,
            recommended_universal: base_settings(),
            measured_at_iso: ISO_PLACEHOLDER.to_string(),
            inferred_role: None,
            role_confidence: None,
            inferred_character: None,
            character_confidence: None,
            spectral_balance_6band: None,
            transient_flux: None,
            stereo_correlation: None,
            dynamic_range_p95_p10_db: None,
            lufs_short_term_max_3s: None,
            energy_density_score: None,
        }
    }

    #[test]
    fn comparison_row_records_reference_yes_deltas_and_gaps() {
        let source = analysis(
            "source",
            -16.0,
            7.0,
            SpectralBalance {
                low: 0.30,
                mid: 0.50,
                high: 0.20,
            },
            0.40,
            0.50,
        );
        let reference = analysis(
            "reference",
            -14.0,
            5.5,
            SpectralBalance {
                low: 0.35,
                mid: 0.42,
                high: 0.23,
            },
            0.55,
            0.62,
        );
        let yes = analysis(
            "yes",
            -14.5,
            6.2,
            SpectralBalance {
                low: 0.32,
                mid: 0.46,
                high: 0.22,
            },
            0.47,
            0.56,
        );
        let checks = vec![
            QualityCheck {
                level: QualityLevel::Info,
                code: "ok".to_string(),
                message: "OK".to_string(),
            },
            QualityCheck {
                level: QualityLevel::Warning,
                code: "dynamic_range_changed".to_string(),
                message: "Dynamics changed.".to_string(),
            },
        ];

        let row = comparison_row_for(
            "Coat",
            Preset::Clarity,
            &source,
            &reference,
            &yes,
            &PathBuf::from("C:/private/Coat-clarity-test.wav"),
            &PathBuf::from("C:/private/renders/clarity.wav"),
            &checks,
        );

        assert_eq!(row.track_label, "Coat");
        assert_eq!(row.preset, Preset::Clarity);
        assert!((row.reference_lufs_delta_db - 2.0).abs() < 0.01);
        assert!((row.yes_lufs_delta_db - 1.5).abs() < 0.01);
        assert!((row.lufs_gap_db - -0.5).abs() < 0.01);
        assert!((row.dynamic_range_gap_lu - 0.7).abs() < 0.01);
        assert!((row.spectral_low_gap - -0.03).abs() < 0.01);
        assert!((row.transient_density_gap - -0.08).abs() < 0.01);
        assert!((row.stereo_width_gap - -0.06).abs() < 0.01);
        assert_eq!(row.warning_codes, vec!["dynamic_range_changed"]);
    }

    #[test]
    fn reference_render_settings_use_preset_compressor_and_source_lufs() {
        let source = analysis(
            "source",
            -12.3,
            5.0,
            SpectralBalance {
                low: 0.3,
                mid: 0.4,
                high: 0.3,
            },
            0.5,
            0.5,
        );

        let settings = settings_for_reference_preset(&source, Preset::Tape);

        assert_eq!(settings.preset, Preset::Tape);
        assert_eq!(
            settings.advanced.compression_mode,
            crate::types::CompressionMode::Preset
        );
        assert_eq!(settings.source_lufs_integrated, Some(-12.3));
        assert_eq!(settings.volume_match, false);
    }
}
