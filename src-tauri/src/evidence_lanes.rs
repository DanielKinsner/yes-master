//! Shared plumbing for the two private evidence lanes (`fixture_matrix`,
//! `reference_tuning`).
//!
//! The lanes were the repo's #1 Rust co-change pair (10 same-commit changes
//! in 192): every CSV/path/report/slug tweak and — far more dangerously —
//! every adaptive-context change had to be hand-mirrored into both files.
//! The 2026-06-02 review caught exactly that class live: the lanes were
//! validating a different chain than the app ships. Everything here exists
//! so the lanes stay representative of the app **by construction**.
//!
//! The lane-specific case lists, report schemas, and CSV layouts stay in
//! their own files on purpose — that divergence is intentional.

use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::types::{
    AnalysisResult, CommandError, CommandResult, CompressionMode, ExportReport, MasteringSettings,
    Preset, RenderedMeasurements, TrackId,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceGateState {
    pub confidence_gate_enabled: bool,
    pub adaptive_compression_enabled: bool,
}

pub(crate) fn current_gate_state() -> EvidenceGateState {
    EvidenceGateState {
        confidence_gate_enabled: crate::confidence::is_confidence_gating_enabled(),
        adaptive_compression_enabled: crate::guardrails::is_adaptive_compression_enabled(),
    }
}

/// Resolve render settings for an evidence-lane case the SAME way the app's
/// chain entry does: clone the source's recommended settings, pin the case's
/// preset + compression mode, disable volume match, inject source LUFS, then
/// resolve the backend `SourceProfile` and the gate-aware confidence. The
/// source must be analyzed with `deep = true` so Phase B confidence CAN
/// resolve; while confidence gating is off it resolves to `None`
/// (byte-identical Tier-1), and once the gate flips on the lanes track the
/// app automatically. Adaptive-compressor guards follow the same app path:
/// gate off is inert, gate on consumes the deep per-band PSR and the
/// already-mastered stand-down classifier.
pub(crate) fn resolve_adaptive_render_settings(
    source_analysis: &AnalysisResult,
    preset: Preset,
    compression_mode: CompressionMode,
) -> MasteringSettings {
    let mut settings = source_analysis.recommended_universal.clone();
    settings.preset = preset;
    settings.volume_match = false;
    settings.source_lufs_integrated = Some(source_analysis.lufs_integrated);
    settings.advanced.compression_mode = compression_mode;
    settings.advanced.source_profile = crate::types::SourceProfile::from_analysis(source_analysis);
    crate::profile_store::apply_resolved_confidence(
        &mut settings,
        source_analysis.deep_analysis.clone(),
        false,
    );
    let band_psr = source_analysis
        .deep_analysis
        .as_deref()
        .and_then(crate::deep_analysis::band_psr_p10_db);
    let stand_down = crate::guardrails::classify_already_mastered_stand_down(
        source_analysis.lufs_integrated,
        source_analysis.true_peak_dbtp,
        source_analysis.dynamic_range_lu,
        band_psr,
    );
    crate::profile_store::apply_resolved_compression_guards(
        &mut settings,
        source_analysis.deep_analysis.clone(),
        Some(stand_down),
        false,
    );
    settings
}

/// Receipt built directly from `RenderedMeasurements` — always rendered
/// output, so `measurements_are_rendered` is unconditionally true.
pub(crate) fn export_report_for(
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
        effective_adaptive_strength: rendered.effective_adaptive_strength,
        source_profile_digest: rendered.source_profile_digest.clone(),
        confidence_digest: rendered.confidence_digest.clone(),
        compression_digest: rendered.compression_digest.clone(),
        measurements_are_rendered: true,
        checks: Vec::new(),
    }
}

pub(crate) fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(crate) fn sanitize_path_part(value: &str) -> String {
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

pub(crate) fn preset_slug(preset: &Preset) -> &'static str {
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

pub(crate) fn normalized_absolute_path(cwd: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    lexically_normalize(&absolute)
}

/// Resolve, validate, and create an evidence-runner output directory.
///
/// Shared by the private fixture matrix and the reference-tuning runner so the
/// two lanes cannot disagree about what a safe destination is. Before U12 only
/// `reference_tuning` normalized its output at all, and `fixture_matrix` passed
/// the caller's raw relative path straight through to the render layer — where
/// the desktop traversal guard rejected every documented invocation, because
/// `--output ../test-output/...` leaves a `..` in each render path.
///
/// **The order is load-bearing:** normalize → validate → create → re-verify.
/// Creating first and validating after would mean a rejected destination had
/// already been written to disk, which for these runners can mean rendered
/// private masters sitting inside a private source folder.
///
/// `source_roots` are directories holding private input audio. The output may
/// be neither one of them nor inside one.
pub(crate) fn prepare_evidence_output_dir(
    cwd: &Path,
    requested: &Path,
    source_roots: &[&Path],
) -> CommandResult<PathBuf> {
    if requested.as_os_str().is_empty() {
        return Err(CommandError::InvalidPath(
            "evidence output dir is empty".to_string(),
        ));
    }

    // 1. Normalize. `..` is EXPECTED here — the documented commands are
    //    relative and climb out of `src-tauri` — so it is resolved, not
    //    rejected. Rejection is the render layer's job, on the absolute path
    //    this produces.
    let resolved = normalized_absolute_path(cwd, requested);

    // 2. Validate. A `..` that survives normalization climbed past the root
    //    and there is nothing sane left to point at.
    if resolved
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CommandError::InvalidPath(format!(
            "evidence output dir escapes the filesystem root: {}",
            requested.display()
        )));
    }
    if resolved.file_name().is_none() && resolved.parent().is_some() {
        return Err(CommandError::InvalidPath(format!(
            "evidence output dir has no name: {}",
            requested.display()
        )));
    }
    for root in source_roots {
        let root_abs = comparable_dir(&normalized_absolute_path(cwd, root));
        if comparable_dir(&resolved).starts_with(&root_abs) {
            return Err(CommandError::InvalidPath(format!(
                "evidence output dir is inside the private source dir {} — \
                 rendered masters must not be written next to private audio: {}",
                root.display(),
                requested.display()
            )));
        }
    }

    // 3. Create ONLY the destination that just passed validation.
    fs::create_dir_all(&resolved)
        .map_err(|e| CommandError::Io(format!("create evidence output dir: {e}")))?;

    // 4. Re-verify the destination that now actually exists. Between the check
    //    above and this line the path could resolve somewhere else — a symlink
    //    or a Windows junction planted at the last component would land the
    //    renders in the source dir after all.
    let after_create = comparable_dir(&resolved);
    for root in source_roots {
        let root_abs = comparable_dir(&normalized_absolute_path(cwd, root));
        if after_create.starts_with(&root_abs) {
            return Err(CommandError::InvalidPath(format!(
                "evidence output dir resolves into the private source dir {} after creation \
                 (symlink or junction?): {}",
                root.display(),
                requested.display()
            )));
        }
    }

    // Return the plain absolute path, NOT the canonical one. On Windows
    // `canonicalize` yields a verbatim `\\?\C:\...` path, which would then be
    // stamped into every report string and every render destination (see
    // tests/portability_paths.rs for why that shape causes trouble).
    Ok(resolved)
}

/// Canonical form for containment comparisons only — never returned to a
/// caller, because on Windows it is a verbatim `\\?\C:\...` path.
///
/// The output directory usually does NOT exist yet, so a plain `canonicalize`
/// fails on it while succeeding on the source root — and then the two sides are
/// in different path SHAPES and `starts_with` silently returns false. On
/// Windows that is not theoretical: a temp path canonicalizes to the long form
/// (`C:\Users\SM - Dan\...`) while the unresolved side is still the 8.3 short
/// form (`C:\Users\SM-DAN~1\...`), so a containment check comparing them would
/// pass every unsafe destination. This walks up to the nearest ancestor that
/// does exist, canonicalizes THAT, and re-appends the missing tail, so both
/// sides always end up in the same shape.
fn comparable_dir(path: &Path) -> PathBuf {
    if let Ok(canonical) = fs::canonicalize(path) {
        return canonical;
    }

    let mut tail: Vec<&std::ffi::OsStr> = Vec::new();
    let mut cursor = path;
    while let Some(parent) = cursor.parent() {
        let Some(name) = cursor.file_name() else {
            break;
        };
        tail.push(name);
        if let Ok(canonical) = fs::canonicalize(parent) {
            let mut resolved = canonical;
            for name in tail.iter().rev() {
                resolved.push(name);
            }
            return resolved;
        }
        cursor = parent;
    }
    path.to_path_buf()
}

pub(crate) fn lexically_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // Only a Normal component may be cancelled by a `..`. A bare
                // `pop()` here also popped a previously PUSHED `..`, so at the
                // filesystem root excess `..` components cancelled in PAIRS —
                // an even number of them vanished, the escapes-the-root
                // validation saw nothing, and the runner went on to create
                // `/evidence` at the root (rejected as read-only on macOS CI,
                // silently creatable elsewhere). Whether the traversal test
                // caught it depended on the temp dir's depth parity.
                match normalized.components().next_back() {
                    Some(Component::Normal(_)) => {
                        normalized.pop();
                    }
                    _ => normalized.push(component.as_os_str()),
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Test fixtures shared by both lanes' unit tests.
#[cfg(test)]
pub(crate) mod test_support {
    pub(crate) fn make_deep_for_test() -> crate::deep_analysis::DeepAnalysis {
        let sr = 48_000_u32;
        let n = sr as usize * 2;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let windows = crate::deep_analysis::scan_windows(&samples, sr, 1);
        crate::deep_analysis::DeepAnalysis::from_parts([1.0 / 31.0; 31], windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture_matrix::{settings_for_matrix_case, MatrixCase};
    use crate::reference_tuning::settings_for_reference_preset;
    use crate::types::{
        AdvancedSettings, DeliveryProfile, SpectralBalance, SpectralBalance6, TrackId,
        ISO_PLACEHOLDER,
    };
    use std::sync::Arc;

    struct AdaptiveCompressionGateReset(bool);

    impl AdaptiveCompressionGateReset {
        fn set(enabled: bool) -> Self {
            Self(crate::guardrails::set_adaptive_compression_enabled(enabled))
        }
    }

    impl Drop for AdaptiveCompressionGateReset {
        fn drop(&mut self) {
            crate::guardrails::set_adaptive_compression_enabled(self.0);
        }
    }

    #[test]
    fn excess_parent_dirs_survive_normalization_regardless_of_parity() {
        // Regression for the 2026-07-27 macOS CI failure: a bare `pop()` in
        // lexically_normalize also popped a previously pushed `..`, so at the
        // root excess `..` components cancelled in PAIRS. An EVEN excess
        // vanished entirely, the escapes-the-root validation saw a clean
        // absolute path, and the runner tried to create `/evidence` at the
        // filesystem root. Whether the tempdir-based integration test caught
        // this depended on the temp dir's depth parity — these fixed-depth
        // cases pin both parities on every OS.
        for (cwd, requested) in [
            // depth 2, four `..` → 2 excess (EVEN — the case that vanished)
            ("/a/b", "../../../../evidence"),
            // depth 2, five `..` → 3 excess (odd)
            ("/a/b", "../../../../../evidence"),
        ] {
            let resolved = normalized_absolute_path(Path::new(cwd), Path::new(requested));
            assert!(
                resolved
                    .components()
                    .any(|c| matches!(c, Component::ParentDir)),
                "excess `..` must stay visible after normalization, got {} from cwd={cwd} requested={requested}",
                resolved.display(),
            );
            let error = prepare_evidence_output_dir(Path::new(cwd), Path::new(requested), &[])
                .expect_err("a path that climbs past the root must be rejected");
            let message = format!("{error}");
            assert!(
                message.contains("escapes"),
                "rejection must name the escape, got: {message}",
            );
        }
    }

    #[test]
    fn documented_relative_output_still_normalizes_and_passes_validation() {
        // The documented commands run from src-tauri with `--output
        // ../test-output/...` — one `..` that cancels against a real
        // directory. The parity fix must not break the legitimate case.
        let resolved = normalized_absolute_path(
            Path::new("/repo/src-tauri"),
            Path::new("../test-output/evidence"),
        );
        assert_eq!(resolved, Path::new("/repo/test-output/evidence"));
        assert!(!resolved
            .components()
            .any(|c| matches!(c, Component::ParentDir)));
    }

    #[test]
    fn evidence_gate_state_serializes_both_gate_flags() {
        // The evidence reports embed EvidenceGateState so a future diff can tell
        // which gates were live for a run. Lock its wire shape with concrete
        // values, independent of the live gate, so a field rename / serde(skip)
        // is caught.
        let json = serde_json::to_value(EvidenceGateState {
            confidence_gate_enabled: true,
            adaptive_compression_enabled: false,
        })
        .expect("serialize gate state");
        assert_eq!(
            json.get("confidence_gate_enabled")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            json.get("adaptive_compression_enabled")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    fn synthetic_analysis() -> AnalysisResult {
        AnalysisResult {
            track_id: TrackId("cross-lane".to_string()),
            lufs_integrated: -9.5,
            lufs_short_term_max: -7.5,
            true_peak_dbtp: -0.5,
            dynamic_range_lu: 6.0,
            spectral_balance: SpectralBalance {
                low: 0.25,
                mid: 0.5,
                high: 0.25,
            },
            transient_density: 0.5,
            stereo_width: 0.5,
            recommended_universal: MasteringSettings {
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
                advanced: AdvancedSettings::default(),
            },
            measured_at_iso: ISO_PLACEHOLDER.to_string(),
            inferred_role: None,
            role_confidence: None,
            inferred_character: None,
            character_confidence: None,
            spectral_balance_6band: Some(SpectralBalance6 {
                sub: 0.125,
                low: 0.25,
                low_mid: 0.1875,
                mid: 0.1875,
                presence: 0.125,
                air: 0.125,
            }),
            transient_flux: Some(0.5),
            stereo_correlation: Some(0.5),
            dynamic_range_p95_p10_db: Some(9.5),
            lufs_short_term_max_3s: Some(-7.0),
            energy_density_score: Some(0.5),
            deep_analysis: Some(Arc::new(test_support::make_deep_for_test())),
        }
    }

    fn dense_deep_for_test() -> crate::deep_analysis::DeepAnalysis {
        let window = crate::deep_analysis::WindowMetrics {
            loudness_key: -6.0,
            sample_peak: 1.0,
            crest: 1.4,
            stereo_width: 0.4,
            stereo_correlation: 0.4,
            low: 0.34,
            mid: 0.33,
            high: 0.33,
            comp_low_31: 0.34,
            comp_mid_31: 0.33,
            comp_high_31: 0.33,
            low_31: 0.34,
            harsh_31: 0.12,
            sibilant_31: 0.12,
            air_31: 0.09,
            tilt_31: 0.0,
        };
        crate::deep_analysis::DeepAnalysis::from_parts([1.0 / 31.0; 31], vec![window; 8])
    }

    fn app_resolved_settings(
        source_analysis: &AnalysisResult,
        preset: Preset,
        compression_mode: CompressionMode,
    ) -> MasteringSettings {
        let mut settings = source_analysis.recommended_universal.clone();
        settings.preset = preset;
        settings.volume_match = false;
        settings.source_lufs_integrated = Some(source_analysis.lufs_integrated);
        settings.advanced.compression_mode = compression_mode;
        settings.advanced.source_profile =
            crate::types::SourceProfile::from_analysis(source_analysis);
        crate::profile_store::apply_resolved_confidence(
            &mut settings,
            source_analysis.deep_analysis.clone(),
            false,
        );
        let band_psr = source_analysis
            .deep_analysis
            .as_deref()
            .and_then(crate::deep_analysis::band_psr_p10_db);
        crate::profile_store::apply_resolved_compression_guards(
            &mut settings,
            source_analysis.deep_analysis.clone(),
            Some(crate::guardrails::classify_already_mastered_stand_down(
                source_analysis.lufs_integrated,
                source_analysis.true_peak_dbtp,
                source_analysis.dynamic_range_lu,
                band_psr,
            )),
            false,
        );
        settings
    }

    /// The drift class each lane's own pin cannot see: both lanes staying
    /// individually green while resolving DIFFERENT adaptive contexts for
    /// the same source. Identical inputs through both public wrappers must
    /// produce identical settings — wire-identical (covers the profile and
    /// every plain field) and identical `source_confidence` (serde(skip), so
    /// it needs its own assert; under the gated lane run this compares real
    /// resolved confidences, gate-off it compares None == None).
    #[test]
    fn both_lanes_resolve_identical_adaptive_context_for_the_same_source() {
        let analysis = synthetic_analysis();
        let preset = Preset::Tape;

        let from_matrix = settings_for_matrix_case(
            &analysis,
            &MatrixCase {
                name: "tape-preset".to_string(),
                preset: preset.clone(),
                compression_mode: CompressionMode::Preset,
            },
        );
        let from_reference = settings_for_reference_preset(&analysis, preset.clone());
        let from_shared =
            resolve_adaptive_render_settings(&analysis, preset, CompressionMode::Preset);

        let wire = |s: &MasteringSettings| serde_json::to_value(s).expect("serialize settings");
        assert_eq!(
            wire(&from_matrix),
            wire(&from_reference),
            "fixture-matrix and reference-tuning resolved different render settings"
        );
        assert_eq!(
            wire(&from_matrix),
            wire(&from_shared),
            "a lane wrapper diverged from the shared resolver"
        );

        assert!(
            from_matrix.advanced.source_profile.is_some(),
            "synthetic analysis should resolve a source profile"
        );
        let confidence = |s: &MasteringSettings| format!("{:?}", s.advanced.source_confidence);
        assert_eq!(confidence(&from_matrix), confidence(&from_reference));
        assert_eq!(confidence(&from_matrix), confidence(&from_shared));
    }

    #[test]
    fn evidence_lanes_apply_the_same_compression_guards_as_the_app() {
        let _lock = crate::guardrails::ADAPTIVE_COMPRESSION_GATE_TEST_LOCK
            .lock()
            .expect("adaptive compression gate test lock");
        let _gate = AdaptiveCompressionGateReset::set(true);
        let mut analysis = synthetic_analysis();
        analysis.lufs_integrated = -9.0;
        analysis.true_peak_dbtp = -0.4;
        analysis.dynamic_range_lu = 4.0;
        analysis.deep_analysis = Some(Arc::new(dense_deep_for_test()));
        let preset = Preset::Universal;

        let from_matrix = settings_for_matrix_case(
            &analysis,
            &MatrixCase {
                name: "universal-preset".to_string(),
                preset: preset.clone(),
                compression_mode: CompressionMode::Preset,
            },
        );
        let from_reference = settings_for_reference_preset(&analysis, preset.clone());
        let from_shared =
            resolve_adaptive_render_settings(&analysis, preset.clone(), CompressionMode::Preset);
        let expected = app_resolved_settings(&analysis, preset, CompressionMode::Preset);

        assert!(
            expected.advanced.compression_guards.is_some(),
            "synthetic dense/already-mastered source should trigger adaptive compression guards"
        );
        assert_eq!(
            from_matrix.advanced.compression_guards, expected.advanced.compression_guards,
            "fixture matrix must render with the app-resolved compression guards"
        );
        assert_eq!(
            from_reference.advanced.compression_guards, expected.advanced.compression_guards,
            "reference tuning must render with the app-resolved compression guards"
        );
        assert_eq!(
            from_shared.advanced.compression_guards, expected.advanced.compression_guards,
            "shared evidence resolver must inject app-resolved compression guards"
        );
    }
}
