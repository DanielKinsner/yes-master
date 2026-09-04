//! Rust ↔ TS wire-contract drift gate.
//!
//! `src/bindings.ts` is hand-written; `types.rs` ↔ `bindings.ts` drift has
//! shipped before and was only ever caught by human review (`f7377f0`). This
//! test serializes a canonical sample of each load-bearing wire type into
//! `src/wire-samples.json`; the TypeScript side (`src/bindings-drift.test.ts`)
//! compares the emitted key sets against the bindings interfaces at compile
//! time, so a field add/remove/rename on either side fails `npm run build`
//! instead of shipping silently.
//!
//! After an intentional contract change, regenerate and commit:
//!   $env:YES_MASTER_UPDATE_GOLDEN = "1"; cargo test --test wire_shape
//! then fix bindings.ts until `npm run build` is green again.
//!
//! Values are halves/quarters so the f32→f64 widening through
//! `serde_json::Value` stays clean in the committed JSON.

use std::path::Path;

use yes_master_lib::audio::AudioOutputDevice;
use yes_master_lib::confidence::Confidence;
use yes_master_lib::engine::{AlbumRenderReport, AlbumTrackRenderInput, AlbumTrackRenderRecord};
use yes_master_lib::guardrails::{
    CompressionBandPlan, CompressionPlan, CompressionPlanReason, GuardReason, GuardrailReadout,
};
use yes_master_lib::types::{
    AdvancedSettings, AlbumArc, AlbumArcKind, AlbumCharacter, AlbumPlan, AlbumTrackEntry,
    AnalysisProgress, AnalysisResult, CompressionMode, ExportReport, ImportedTrack,
    InferenceConfidence, JobStatus, LandingStatus, PlaybackTick, Preset, PresetKind, ProjectMode,
    ProjectState, QualityCheck, QualityLevel, RenderJob, RenderKind, RenderedMeasurements,
    SourceProfile, SpectralBalance, SpectralBalance6, TrackCharacter, TrackId, TrackRole,
    TransitionSpec, UserPreset, WaveformPeaks,
};

mod common;
use common::default_master_settings;

fn spectral6_sample() -> SpectralBalance6 {
    SpectralBalance6 {
        sub: 0.125,
        low: 0.25,
        low_mid: 0.1875,
        mid: 0.1875,
        presence: 0.125,
        air: 0.125,
    }
}

fn source_profile_sample() -> SourceProfile {
    SourceProfile {
        spectral_6: spectral6_sample(),
        dynamic_range_p95_p10_db: 9.5,
        dynamic_range_lu: 8.5,
        stereo_correlation: Some(0.5),
        stereo_width: 0.5,
    }
}

/// Every `Option` populated so nested objects materialize in the JSON and the
/// TS side can see the full key set.
fn advanced_sample() -> AdvancedSettings {
    AdvancedSettings {
        lufs_offset_db: Some(-12.5),
        ceiling_dbtp: Some(-1.0),
        width: Some(1.25),
        warmth: Some(0.25),
        presence_air: Some(0.25),
        compression_mode: CompressionMode::Manual,
        compression_density: Some(0.5),
        compression_low_threshold_db: Some(-24.0),
        compression_low_ratio: Some(2.0),
        compression_low_attack_ms: Some(10.0),
        compression_low_release_ms: Some(120.0),
        compression_mid_threshold_db: Some(-22.0),
        compression_mid_ratio: Some(2.0),
        compression_mid_attack_ms: Some(8.0),
        compression_mid_release_ms: Some(110.0),
        compression_high_threshold_db: Some(-20.0),
        compression_high_ratio: Some(2.5),
        compression_high_attack_ms: Some(5.0),
        compression_high_release_ms: Some(90.0),
        compression_link_stereo: Some(true),
        bit_depth: Some(24),
        target_sample_rate: Some(48_000),
        adaptive_strength: Some(0.5),
        source_profile: Some(source_profile_sample()),
        // #[serde(skip)] — backend-internal, never serialized, so it is
        // correctly invisible to this wire gate and absent from bindings.ts.
        source_confidence: None,
        compression_guards: None,
    }
}

fn album_track_entry_sample() -> AlbumTrackEntry {
    AlbumTrackEntry {
        track_id: TrackId("wire-sample".to_string()),
        position: 1,
        role: TrackRole::Opener,
        role_locked: true,
        arc_lufs_offset_db: -1.5,
        intensity_scale: 1.0,
        album_character: Some(AlbumCharacter::HeavyDjent),
    }
}

fn album_plan_sample() -> AlbumPlan {
    AlbumPlan {
        title: "Wire Sample".to_string(),
        arc: AlbumArc::Preset {
            preset: AlbumArcKind::Cinematic,
        },
        tracks: vec![album_track_entry_sample()],
        transitions: vec![TransitionSpec::gap(1.5)],
        intensity: 1.0,
        delivery_sample_rate: Some(48_000),
        delivery_bit_depth: Some(24),
    }
}

fn mastering_settings_sample() -> yes_master_lib::types::MasteringSettings {
    let mut settings = default_master_settings();
    settings.preset = Preset::Universal;
    settings.source_lufs_integrated = Some(-12.5);
    settings.album = Some(album_plan_sample());
    settings.advanced = advanced_sample();
    settings
}

fn analysis_result_sample() -> AnalysisResult {
    AnalysisResult {
        track_id: TrackId("wire-sample".to_string()),
        lufs_integrated: -14.0,
        lufs_short_term_max: -10.0,
        true_peak_dbtp: -1.0,
        dynamic_range_lu: 8.0,
        spectral_balance: SpectralBalance {
            low: 0.25,
            mid: 0.5,
            high: 0.25,
        },
        transient_density: 0.5,
        stereo_width: 0.5,
        recommended_universal: default_master_settings(),
        measured_at_iso: "2026-06-09T00:00:00+00:00".to_string(),
        inferred_role: Some(TrackRole::Opener),
        role_confidence: Some(InferenceConfidence::Moderate),
        inferred_character: Some(TrackCharacter::Balanced),
        character_confidence: Some(InferenceConfidence::Moderate),
        spectral_balance_6band: Some(spectral6_sample()),
        transient_flux: Some(0.5),
        stereo_correlation: Some(0.5),
        dynamic_range_p95_p10_db: Some(9.5),
        lufs_short_term_max_3s: Some(-9.5),
        energy_density_score: Some(0.5),
        // Backend-internal (bridge/profile-store only); `#[serde(skip)]` keeps
        // it off the wire entirely, so the key never appears in the JSON and
        // bindings.ts omits it on purpose. The TS gate allowlists it defensively.
        deep_analysis: None,
    }
}

fn rendered_measurements_sample() -> RenderedMeasurements {
    RenderedMeasurements {
        lufs_integrated: -13.5,
        true_peak_dbtp: -1.25,
        dynamic_range_lu: 8.5,
        sample_rate: 48_000,
        bit_depth: 24,
        effective_adaptive_strength: 0.75,
        source_profile_digest: Some("bass +1.2 | air -0.4".to_string()),
        confidence_digest: Some("bass 0.9 | tilt 0.6".to_string()),
        compression_digest: Some(
            "compression eased low 20% / mid 16% / high 25%; stand-down 0.00; density confidence 1.00"
                .to_string(),
        ),
    }
}

fn compression_plan_sample() -> CompressionPlan {
    CompressionPlan {
        active: true,
        low: CompressionBandPlan {
            threshold_db: -10.5,
            ratio: 1.25,
            density_mult: 0.8,
            threshold_lift_db: 2.0,
            ratio_mult: 0.9,
            adaptive: true,
        },
        mid: CompressionBandPlan {
            threshold_db: -12.5,
            ratio: 1.45,
            density_mult: 1.0,
            threshold_lift_db: 0.0,
            ratio_mult: 1.0,
            adaptive: false,
        },
        high: CompressionBandPlan {
            threshold_db: -9.5,
            ratio: 1.2,
            density_mult: 0.75,
            threshold_lift_db: 2.4,
            ratio_mult: 0.88,
            adaptive: true,
        },
        reasons: vec![CompressionPlanReason {
            code: GuardReason::LowBandDense,
            message: "Low band is already dense - easing compression there.".to_string(),
        }],
        guidance: Some("Low band is already dense - easing compression there.".to_string()),
        digest: Some(
            "compression eased low 20% / mid 0% / high 25%; stand-down 0.00; density confidence 1.00"
                .to_string(),
        ),
    }
}

fn analysis_progress_sample() -> AnalysisProgress {
    AnalysisProgress {
        batch_id: "wire-batch".to_string(),
        fraction: 0.5,
        label: "Reading tonal balance".to_string(),
    }
}

fn landing_status_sample() -> LandingStatus {
    LandingStatus {
        track_id: Some(TrackId("wire-sample".to_string())),
        pending: true,
    }
}

fn render_job_sample() -> RenderJob {
    RenderJob {
        id: "wire-job".to_string(),
        job_id: "wire-job".to_string(),
        kind: RenderKind::Master,
        target_tracks: vec![TrackId("wire-sample".to_string())],
        status: JobStatus::Done,
        progress: 1.0,
        started_at_iso: "2026-06-09T00:00:00+00:00".to_string(),
        output_paths: vec!["out/wire-sample.master.wav".to_string()],
        measurements: Some(rendered_measurements_sample()),
    }
}

fn guardrail_readout_sample() -> GuardrailReadout {
    GuardrailReadout {
        signal_chain: Some(yes_master_lib::guardrails::SignalChainReadout {
            eq_active: true,
            warmth_active: false,
            air_active: false,
            compression_active: true,
            width: 1.11,
            saturation: 0.055,
        }),
        active: true,
        strength: 0.5,
        bright_trim: 0.25,
        low_trim: 0.25,
        density_trim: 0.25,
        width_trim: 0.25,
        brightness_share: 0.25,
        low_share: 0.25,
        dynamic_range_db: 9.5,
        bright_deadband: 0.0625,
        low_deadband: 0.0625,
        width_corr_deadband: 0.25,
        stereo_correlation: Some(0.5),
        // Materialized (Some) so the nested Confidence / AxisConfidence objects
        // appear in the JSON and the TS drift gate can recurse into them. The
        // key set is unchanged vs None (the field is always serialized).
        confidence: Some(Confidence::full()),
        effective_auto_width: Some(1.11),
    }
}

fn playback_tick_sample() -> PlaybackTick {
    PlaybackTick {
        track_id: Some(TrackId("wire-sample".to_string())),
        position_sec: 1.5,
        is_playing: true,
        is_loaded: true,
        device_lost: false,
        peak_dbfs: -6.0,
        peak_left_dbfs: -6.5,
        peak_right_dbfs: -6.25,
        gr_low_db: -1.5,
        gr_mid_db: -1.25,
        gr_high_db: -1.0,
        lufs_momentary: -12.0,
        lufs_integrated: -14.0,
        spectrum_db: vec![-30.0, -24.0],
    }
}

fn project_state_sample() -> ProjectState {
    let track_id = TrackId("wire-sample".to_string());
    let mut track_settings = std::collections::HashMap::new();
    track_settings.insert(track_id.0.clone(), mastering_settings_sample());
    ProjectState {
        schema_version: 1,
        mode: ProjectMode::Track,
        tracks: vec![ImportedTrack {
            id: track_id.clone(),
            path: "C:/music/wire-sample.wav".to_string(),
            display_name: "wire-sample".to_string(),
            source_format: "wav".to_string(),
            duration_seconds: Some(1.0),
            sample_rate: Some(44_100),
            channels: Some(2),
        }],
        track_order: vec![track_id.clone()],
        track_settings,
        album_intent: Some(mastering_settings_sample()),
        album_arc_kind: AlbumArcKind::Cinematic,
        album_intensity: 1.0,
        album_title: "Wire Sample".to_string(),
        album_sample_rate: Some(48_000),
        album_bit_depth: Some(24),
        track_override_album: vec![track_id.clone()],
        selected_track_id: Some(track_id),
        view_by_track_id: std::collections::HashMap::new(),
        last_saved_iso: Some("2026-06-09T00:00:00+00:00".to_string()),
    }
}

fn audio_output_device_sample() -> AudioOutputDevice {
    AudioOutputDevice {
        id: "wire-device".to_string(),
        name: "Wire Output".to_string(),
        is_default: true,
        is_selected: true,
    }
}

fn waveform_peaks_sample() -> WaveformPeaks {
    WaveformPeaks {
        track_id: TrackId("wire-sample".to_string()),
        channels: vec![vec![-0.5, 0.5], vec![-0.25, 0.25]],
        samples_per_pixel: 512,
        total_samples: 44_100,
        sample_rate: 44_100,
    }
}

fn quality_check_sample() -> QualityCheck {
    QualityCheck {
        level: QualityLevel::Warning,
        code: "true_peak_near_ceiling".to_string(),
        message: "True peak is close to the ceiling.".to_string(),
    }
}

fn export_report_sample() -> ExportReport {
    ExportReport {
        track_id: TrackId("wire-sample".to_string()),
        output_path: "out/wire-sample.master.wav".to_string(),
        measured_lufs: -13.5,
        measured_true_peak_dbtp: -1.25,
        measured_dynamic_range_lu: 8.5,
        source_format: "wav".to_string(),
        destination_format: "wav 24-bit 48 kHz".to_string(),
        sample_rate: 48_000,
        bit_depth: 24,
        effective_adaptive_strength: 0.5,
        source_profile_digest: Some("bass +1.2 | air -0.4".to_string()),
        confidence_digest: Some("bass 0.9 | tilt 0.6".to_string()),
        compression_digest: Some("compression eased low 20%".to_string()),
        measurements_are_rendered: true,
        checks: vec![quality_check_sample()],
    }
}

fn user_preset_sample() -> UserPreset {
    UserPreset {
        id: "wire-preset".to_string(),
        name: "Wire Preset".to_string(),
        kind: PresetKind::Track,
        settings: mastering_settings_sample(),
        created_at_iso: "2026-06-09T00:00:00+00:00".to_string(),
    }
}

fn album_track_render_record_sample() -> AlbumTrackRenderRecord {
    AlbumTrackRenderRecord {
        track_id: TrackId("wire-sample".to_string()),
        position: 1,
        output_path: "out/wire-sample.master.wav".to_string(),
        measured_lufs: -13.5,
        source_sample_rate: 44_100,
        rendered_sample_rate: 48_000,
        source_channels: 2,
        rendered_channels: 2,
        override_album: true,
    }
}

fn album_render_report_sample() -> AlbumRenderReport {
    AlbumRenderReport {
        job_id: "wire-album-job".to_string(),
        status: JobStatus::Done,
        album_wav_path: "out/album.wav".to_string(),
        manifest_path: "out/album.manifest.json".to_string(),
        requested_sample_rate: Some(48_000),
        rendered_sample_rate: 48_000,
        source_sample_rates: vec![44_100, 48_000],
        bit_depth: 24,
        rendered_channels: 2,
        source_channels: vec![2, 2],
        tracks: vec![album_track_render_record_sample()],
    }
}

fn album_track_render_input_sample() -> AlbumTrackRenderInput {
    AlbumTrackRenderInput {
        track_id: TrackId("wire-sample".to_string()),
        source_path: "C:/music/wire-sample.wav".to_string(),
        settings: mastering_settings_sample(),
        override_album: true,
    }
}

fn wire_samples() -> serde_json::Value {
    serde_json::json!({
        "_readme": "Generated by src-tauri/tests/wire_shape.rs — do not edit. \
                    Regenerate after an intentional Rust wire-type change with \
                    YES_MASTER_UPDATE_GOLDEN=1 cargo test --test wire_shape, \
                    then make `npm run build` green against bindings.ts.",
        "advanced": advanced_sample(),
        "album_plan": album_plan_sample(),
        "album_render_report": album_render_report_sample(),
        "album_track_entry": album_track_entry_sample(),
        "album_track_render_input": album_track_render_input_sample(),
        "analysis_progress": analysis_progress_sample(),
        "analysis_result": analysis_result_sample(),
        "audio_output_device": audio_output_device_sample(),
        "compression_plan": compression_plan_sample(),
        "export_report": export_report_sample(),
        "guardrail_readout": guardrail_readout_sample(),
        "landing_status": landing_status_sample(),
        "mastering_settings": mastering_settings_sample(),
        "playback_tick": playback_tick_sample(),
        "project_state": project_state_sample(),
        "render_job": render_job_sample(),
        "rendered_measurements": rendered_measurements_sample(),
        "user_preset": user_preset_sample(),
        "waveform_peaks": waveform_peaks_sample(),
    })
}

#[test]
fn wire_samples_json_is_current() {
    let mut actual = serde_json::to_string_pretty(&wire_samples()).expect("serialize samples");
    actual.push('\n');
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/wire-samples.json");
    if std::env::var("YES_MASTER_UPDATE_GOLDEN").as_deref() == Ok("1") {
        std::fs::write(&path, &actual).expect("write wire samples");
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .expect(
            "src/wire-samples.json missing — generate it with \
             YES_MASTER_UPDATE_GOLDEN=1 and commit it",
        )
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "a Rust wire type changed shape vs the committed src/wire-samples.json. \
         If intentional: regenerate with YES_MASTER_UPDATE_GOLDEN=1, commit the \
         diff, and update src/bindings.ts until `npm run build` passes its \
         key-set gate."
    );
}
