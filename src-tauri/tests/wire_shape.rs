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

use yes_master_lib::guardrails::GuardrailReadout;
use yes_master_lib::types::{
    AdvancedSettings, AlbumArc, AlbumArcKind, AlbumCharacter, AlbumPlan, AlbumTrackEntry,
    AnalysisResult, CompressionMode, InferenceConfidence, JobStatus, PlaybackTick, Preset,
    RenderJob, RenderKind, RenderedMeasurements, SourceProfile, SpectralBalance,
    SpectralBalance6, TrackCharacter, TrackId, TrackRole, TransitionSpec,
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
        // Backend-internal (bridge/profile-store only); bindings.ts omits it
        // on purpose and the TS gate allowlists it. The key still serializes
        // as null because nothing uses skip_serializing_if.
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
    }
}

fn render_job_sample() -> RenderJob {
    RenderJob {
        id: "wire-job".to_string(),
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
        confidence: None,
    }
}

fn playback_tick_sample() -> PlaybackTick {
    PlaybackTick {
        track_id: Some(TrackId("wire-sample".to_string())),
        position_sec: 1.5,
        is_playing: true,
        is_loaded: true,
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

fn wire_samples() -> serde_json::Value {
    serde_json::json!({
        "_readme": "Generated by src-tauri/tests/wire_shape.rs — do not edit. \
                    Regenerate after an intentional Rust wire-type change with \
                    YES_MASTER_UPDATE_GOLDEN=1 cargo test --test wire_shape, \
                    then make `npm run build` green against bindings.ts.",
        "advanced": advanced_sample(),
        "album_plan": album_plan_sample(),
        "album_track_entry": album_track_entry_sample(),
        "analysis_result": analysis_result_sample(),
        "guardrail_readout": guardrail_readout_sample(),
        "mastering_settings": mastering_settings_sample(),
        "playback_tick": playback_tick_sample(),
        "render_job": render_job_sample(),
        "rendered_measurements": rendered_measurements_sample(),
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
