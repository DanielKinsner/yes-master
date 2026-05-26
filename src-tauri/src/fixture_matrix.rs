use crate::analysis::analyze_one;
use crate::engine::mastering_render_to_path;
use crate::exports::export_checks_for_report;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateFixtureManifest {
    pub version: u32,
    pub notes: Option<String>,
    pub fixtures: Vec<PrivateFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateFixture {
    pub id: String,
    pub path: Option<String>,
    pub purpose: Option<String>,
    #[serde(default)]
    pub mode: Vec<String>,
    #[serde(default)]
    pub quick_test: Option<bool>,
    #[serde(default)]
    pub slow_test: Option<bool>,
    #[serde(default)]
    pub listening_focus: Vec<String>,
    #[serde(default)]
    pub known_issues: Vec<String>,
}

impl PrivateFixture {
    pub fn resolved_path(&self, manifest_dir: &Path) -> PathBuf {
        let path = PathBuf::from(self.path.as_deref().unwrap_or_default());
        if path.is_absolute() {
            path
        } else {
            manifest_dir.join(path)
        }
    }

    fn is_track_fixture(&self) -> bool {
        self.path.is_some()
            && (self.mode.is_empty()
                || self
                    .mode
                    .iter()
                    .any(|mode| mode.eq_ignore_ascii_case("track")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatrixCase {
    pub name: String,
    pub preset: Preset,
    pub compression_mode: CompressionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixLedgerRow {
    pub fixture_id: String,
    pub source_path: String,
    pub case_name: String,
    pub preset: Preset,
    pub compression_mode: CompressionMode,
    pub source_lufs: f32,
    pub source_true_peak_dbtp: f32,
    pub source_dynamic_range_lu: f32,
    pub rendered_lufs: f32,
    pub rendered_true_peak_dbtp: f32,
    pub rendered_dynamic_range_lu: f32,
    pub lufs_delta_db: f32,
    pub dynamic_range_delta_lu: f32,
    pub warning_codes: Vec<String>,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateFixtureMatrixReport {
    pub generated_at_iso: String,
    pub manifest_path: String,
    pub output_dir: String,
    pub cases: Vec<MatrixCase>,
    pub rows: Vec<MatrixLedgerRow>,
}

#[derive(Debug, Clone)]
pub struct MatrixRunResult {
    pub report_path: PathBuf,
    pub csv_path: PathBuf,
    pub render_dir: PathBuf,
    pub rows: Vec<MatrixLedgerRow>,
}

pub fn default_already_mastered_matrix() -> Vec<MatrixCase> {
    [Preset::Universal, Preset::Loud, Preset::Clarity]
        .into_iter()
        .flat_map(|preset| {
            let preset_name = preset_slug(&preset);
            [
                MatrixCase {
                    name: format!("{preset_name}-preset"),
                    preset: preset.clone(),
                    compression_mode: CompressionMode::Preset,
                },
                MatrixCase {
                    name: format!("{preset_name}-compressor-off"),
                    preset,
                    compression_mode: CompressionMode::Off,
                },
            ]
        })
        .collect()
}

pub fn settings_for_matrix_case(
    source_analysis: &AnalysisResult,
    case: &MatrixCase,
) -> MasteringSettings {
    let mut settings = source_analysis.recommended_universal.clone();
    settings.preset = case.preset.clone();
    settings.volume_match = false;
    settings.source_lufs_integrated = Some(source_analysis.lufs_integrated);
    settings.advanced.compression_mode = case.compression_mode;
    settings
}

pub fn ledger_row_for(
    fixture_id: &str,
    source_path: &Path,
    case: &MatrixCase,
    source_analysis: &AnalysisResult,
    rendered: &RenderedMeasurements,
    output_path: &Path,
    checks: &[QualityCheck],
) -> MatrixLedgerRow {
    MatrixLedgerRow {
        fixture_id: fixture_id.to_string(),
        source_path: source_path.to_string_lossy().to_string(),
        case_name: case.name.clone(),
        preset: case.preset.clone(),
        compression_mode: case.compression_mode,
        source_lufs: source_analysis.lufs_integrated,
        source_true_peak_dbtp: source_analysis.true_peak_dbtp,
        source_dynamic_range_lu: source_analysis.dynamic_range_lu,
        rendered_lufs: rendered.lufs_integrated,
        rendered_true_peak_dbtp: rendered.true_peak_dbtp,
        rendered_dynamic_range_lu: rendered.dynamic_range_lu,
        lufs_delta_db: rendered.lufs_integrated - source_analysis.lufs_integrated,
        dynamic_range_delta_lu: rendered.dynamic_range_lu - source_analysis.dynamic_range_lu,
        warning_codes: checks
            .iter()
            .filter(|check| !matches!(check.level, QualityLevel::Info))
            .map(|check| check.code.clone())
            .collect(),
        output_path: output_path.to_string_lossy().to_string(),
    }
}

pub fn run_manifest_path(
    manifest_path: &Path,
    output_dir: &Path,
) -> CommandResult<MatrixRunResult> {
    let cwd = std::env::current_dir().map_err(|e| CommandError::Io(format!("current dir: {e}")))?;
    let manifest_dir = normalized_manifest_dir(&cwd, manifest_path)?;
    let manifest_file_name = manifest_path.file_name().ok_or_else(|| {
        CommandError::InvalidPath(format!(
            "manifest path must include a file name: {}",
            manifest_path.display()
        ))
    })?;
    let manifest_path = manifest_dir.join(manifest_file_name);
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|e| CommandError::Io(format!("read manifest: {e}")))?;
    let manifest: PrivateFixtureManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| CommandError::Other(format!("parse manifest: {e}")))?;
    run_fixture_matrix(&manifest, &manifest_path, output_dir)
}

pub fn run_fixture_matrix(
    manifest: &PrivateFixtureManifest,
    manifest_path: &Path,
    output_dir: &Path,
) -> CommandResult<MatrixRunResult> {
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let cases = default_already_mastered_matrix();
    let render_dir = output_dir.join("renders");
    fs::create_dir_all(&render_dir)
        .map_err(|e| CommandError::Io(format!("create renders: {e}")))?;

    let mut rows = Vec::new();
    for fixture in manifest
        .fixtures
        .iter()
        .filter(|fixture| fixture.is_track_fixture())
    {
        let source_path = fixture.resolved_path(manifest_dir);
        let source_analysis = analyze_one(TrackId(fixture.id.clone()), &source_path)?;
        let fixture_dir = render_dir.join(sanitize_path_part(&fixture.id));
        fs::create_dir_all(&fixture_dir)
            .map_err(|e| CommandError::Io(format!("create fixture render dir: {e}")))?;

        for case in &cases {
            let settings = settings_for_matrix_case(&source_analysis, case);
            let output_path = fixture_dir.join(format!("{}.wav", sanitize_path_part(&case.name)));
            let job = mastering_render_to_path(
                TrackId(format!("{}-{}", fixture.id, case.name)),
                &source_path,
                &settings,
                &fixture_dir,
                RenderKind::Master,
                &output_path,
            )?;
            let rendered = job.measurements.ok_or_else(|| {
                CommandError::Render("matrix render did not return measurements".to_string())
            })?;
            let export_report = export_report_for(
                &source_analysis.track_id,
                &output_path,
                &rendered,
                source_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("audio"),
            );
            let checks =
                export_checks_for_report(&export_report, Some(&source_analysis), Some(&settings));
            rows.push(ledger_row_for(
                &fixture.id,
                &source_path,
                case,
                &source_analysis,
                &rendered,
                &output_path,
                &checks,
            ));
        }
    }

    if rows.is_empty() {
        return Err(CommandError::Other(
            "manifest contains no track fixtures with a path".to_string(),
        ));
    }

    fs::create_dir_all(output_dir)
        .map_err(|e| CommandError::Io(format!("create matrix output dir: {e}")))?;
    let report = PrivateFixtureMatrixReport {
        generated_at_iso: now_iso(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        cases,
        rows: rows.clone(),
    };
    let report_path = output_dir.join("already-mastered-matrix.json");
    let csv_path = output_dir.join("already-mastered-matrix.csv");
    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|e| CommandError::Other(format!("serialize report: {e}")))?;
    fs::write(&report_path, report_json)
        .map_err(|e| CommandError::Io(format!("write report: {e}")))?;
    fs::write(&csv_path, ledger_csv(&rows))
        .map_err(|e| CommandError::Io(format!("write csv: {e}")))?;

    Ok(MatrixRunResult {
        report_path,
        csv_path,
        render_dir,
        rows,
    })
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

fn ledger_csv(rows: &[MatrixLedgerRow]) -> String {
    let mut out = String::from(
        "fixture_id,case_name,preset,compression_mode,source_lufs,rendered_lufs,lufs_delta_db,source_true_peak_dbtp,rendered_true_peak_dbtp,source_dynamic_range_lu,rendered_dynamic_range_lu,dynamic_range_delta_lu,warning_codes,source_path,output_path\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{},{}\n",
            csv_escape(&row.fixture_id),
            csv_escape(&row.case_name),
            csv_escape(preset_slug(&row.preset)),
            csv_escape(compression_mode_slug(row.compression_mode)),
            row.source_lufs,
            row.rendered_lufs,
            row.lufs_delta_db,
            row.source_true_peak_dbtp,
            row.rendered_true_peak_dbtp,
            row.source_dynamic_range_lu,
            row.rendered_dynamic_range_lu,
            row.dynamic_range_delta_lu,
            csv_escape(&row.warning_codes.join("|")),
            csv_escape(&row.source_path),
            csv_escape(&row.output_path),
        ));
    }
    out
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

fn preset_slug(preset: &Preset) -> &'static str {
    match preset {
        Preset::Universal => "universal",
        Preset::Clarity => "clarity",
        Preset::Tape => "tape",
        Preset::Spatial => "spatial",
        Preset::Oomph => "oomph",
        Preset::Warmth => "warmth",
        Preset::Punch => "punch",
        Preset::Loud => "loud",
        Preset::Custom { .. } => "custom",
    }
}

fn compression_mode_slug(mode: CompressionMode) -> &'static str {
    match mode {
        CompressionMode::Preset => "preset",
        CompressionMode::Manual => "manual",
        CompressionMode::Off => "off",
    }
}

fn normalized_manifest_dir(cwd: &Path, manifest_path: &Path) -> CommandResult<PathBuf> {
    normalized_absolute_path(cwd, manifest_path)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            CommandError::InvalidPath(format!(
                "manifest path has no parent: {}",
                manifest_path.display()
            ))
        })
}

fn normalized_absolute_path(cwd: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    lexically_normalize(&absolute)
}

fn lexically_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    fn source_analysis() -> AnalysisResult {
        AnalysisResult {
            track_id: TrackId("source-a".to_string()),
            lufs_integrated: -12.3,
            lufs_short_term_max: -9.1,
            true_peak_dbtp: -0.4,
            dynamic_range_lu: 4.2,
            spectral_balance: SpectralBalance {
                low: 0.3,
                mid: 0.4,
                high: 0.3,
            },
            transient_density: 0.5,
            stereo_width: 0.5,
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
    fn default_matrix_covers_release_stabilization_presets_and_compressor_off() {
        let cases = default_already_mastered_matrix();
        let names: Vec<&str> = cases.iter().map(|case| case.name.as_str()).collect();

        assert_eq!(
            names,
            vec![
                "universal-preset",
                "universal-compressor-off",
                "loud-preset",
                "loud-compressor-off",
                "clarity-preset",
                "clarity-compressor-off",
            ]
        );
    }

    #[test]
    fn matrix_case_settings_set_preset_mode_and_source_lufs() {
        let analysis = source_analysis();
        let case = MatrixCase {
            name: "loud-compressor-off".to_string(),
            preset: Preset::Loud,
            compression_mode: CompressionMode::Off,
        };

        let settings = settings_for_matrix_case(&analysis, &case);

        assert_eq!(settings.preset, Preset::Loud);
        assert_eq!(settings.advanced.compression_mode, CompressionMode::Off);
        assert_eq!(settings.source_lufs_integrated, Some(-12.3));
        assert_eq!(settings.volume_match, false);
    }

    #[test]
    fn ledger_row_records_deltas_and_warning_codes() {
        let analysis = source_analysis();
        let case = MatrixCase {
            name: "universal-preset".to_string(),
            preset: Preset::Universal,
            compression_mode: CompressionMode::Preset,
        };
        let render = RenderedMeasurements {
            lufs_integrated: -10.7,
            true_peak_dbtp: -0.2,
            dynamic_range_lu: 3.1,
            sample_rate: 44_100,
            bit_depth: 24,
        };
        let checks = vec![
            QualityCheck {
                level: QualityLevel::Warning,
                code: "dynamic_range_low".to_string(),
                message: "Dynamic range is low.".to_string(),
            },
            QualityCheck {
                level: QualityLevel::Warning,
                code: "comp_density_on_compressed_source".to_string(),
                message: "Source appears already compressed.".to_string(),
            },
        ];

        let row = ledger_row_for(
            "coat-test",
            &PathBuf::from("C:/private/coat.wav"),
            &case,
            &analysis,
            &render,
            &PathBuf::from("C:/private/out.wav"),
            &checks,
        );

        assert_eq!(row.fixture_id, "coat-test");
        assert_eq!(row.case_name, "universal-preset");
        assert_eq!(
            row.warning_codes,
            vec!["dynamic_range_low", "comp_density_on_compressed_source"]
        );
        assert!((row.lufs_delta_db - 1.6).abs() < 0.01);
        assert!((row.dynamic_range_delta_lu - -1.1).abs() < 0.01);
    }

    #[test]
    fn fixture_paths_resolve_relative_to_manifest_directory() {
        let manifest = PrivateFixtureManifest {
            version: 1,
            notes: None,
            fixtures: vec![PrivateFixture {
                id: "local-track".to_string(),
                path: Some("masters/local-track.wav".to_string()),
                purpose: None,
                mode: vec!["track".to_string()],
                quick_test: Some(true),
                slow_test: Some(true),
                listening_focus: Vec::new(),
                known_issues: Vec::new(),
            }],
        };

        let path = manifest.fixtures[0].resolved_path(PathBuf::from("D:/fixtures").as_path());

        assert_eq!(path, PathBuf::from("D:/fixtures/masters/local-track.wav"));
    }

    #[test]
    fn manifest_base_dir_removes_parent_components_from_relative_manifest_paths() {
        let cwd = PathBuf::from("D:/repo/src-tauri");
        let manifest = PathBuf::from("../private-audio-fixtures/manifest.json");

        let dir = normalized_manifest_dir(&cwd, &manifest).expect("manifest dir");

        assert_eq!(dir, PathBuf::from("D:/repo/private-audio-fixtures"));
    }
}
