//! Album Master sample-rate parity: mixed-source resampling + explicit
//! delivery rate/bit-depth honored end to end.

use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::PathBuf;
use tempfile::TempDir;
use yes_master_lib::album;
use yes_master_lib::album_render::render_album_plan_impl;
use yes_master_lib::engine::{AlbumPlanRenderRequest, AlbumRenderReport, AlbumTrackRenderInput};
use yes_master_lib::types::{
    AdvancedSettings, AlbumArc, AlbumArcKind, AnalysisResult, DeliveryProfile, InferenceConfidence,
    MasteringSettings, Preset, SpectralBalance, TrackCharacter, TrackId, TrackRole,
    ISO_PLACEHOLDER,
};

fn default_master_settings() -> MasteringSettings {
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
        advanced: AdvancedSettings::default(),
    }
}

fn fake_analysis(id: &str) -> AnalysisResult {
    AnalysisResult {
        track_id: TrackId(id.to_string()),
        lufs_integrated: -14.0,
        lufs_short_term_max: -10.0,
        true_peak_dbtp: -1.0,
        dynamic_range_lu: 8.0,
        spectral_balance: SpectralBalance {
            low: 0.33,
            mid: 0.34,
            high: 0.33,
        },
        transient_density: 0.5,
        stereo_width: 0.5,
        recommended_universal: default_master_settings(),
        measured_at_iso: ISO_PLACEHOLDER.to_string(),
        inferred_role: Some(TrackRole::AlbumTrack),
        role_confidence: Some(InferenceConfidence::Moderate),
        inferred_character: Some(TrackCharacter::Balanced),
        character_confidence: Some(InferenceConfidence::Moderate),
        spectral_balance_6band: None,
        transient_flux: Some(0.5),
        stereo_correlation: None,
        dynamic_range_p95_p10_db: None,
        lufs_short_term_max_3s: None,
        energy_density_score: Some(0.5),
    }
}

fn write_sine_mono(path: &PathBuf, sample_rate: u32, seconds: f32) {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut w = WavWriter::create(path, spec).expect("create wav");
    let frames = (sample_rate as f32 * seconds) as usize;
    let omega = 2.0 * std::f32::consts::PI * 440.0 / sample_rate as f32;
    for i in 0..frames {
        let s = 0.3 * (omega * i as f32).sin();
        w.write_sample((s * 32767.0) as i16).expect("write");
    }
    w.finalize().expect("finalize");
}

/// Build a 2-track plan, optionally set explicit delivery, render, return the report.
fn render_two_track_album(
    tmp: &TempDir,
    rate_a: u32,
    rate_b: u32,
    delivery_sample_rate: Option<u32>,
    delivery_bit_depth: Option<u16>,
) -> AlbumRenderReport {
    let a = tmp.path().join("a.wav");
    let b = tmp.path().join("b.wav");
    write_sine_mono(&a, rate_a, 1.0);
    write_sine_mono(&b, rate_b, 1.0);

    let analyses = [fake_analysis("a"), fake_analysis("b")];
    let refs: Vec<&AnalysisResult> = analyses.iter().collect();
    let mut plan = album::build_album_plan(
        "Parity".to_string(),
        &refs,
        &[1.0, 1.0],
        AlbumArc::Preset {
            preset: AlbumArcKind::Cinematic,
        },
        1.0,
    );
    plan.delivery_sample_rate = delivery_sample_rate;
    plan.delivery_bit_depth = delivery_bit_depth;

    let request = AlbumPlanRenderRequest {
        plan,
        tracks: vec![
            AlbumTrackRenderInput {
                track_id: TrackId("a".into()),
                source_path: a.to_string_lossy().into(),
                settings: default_master_settings(),
            },
            AlbumTrackRenderInput {
                track_id: TrackId("b".into()),
                source_path: b.to_string_lossy().into(),
                settings: default_master_settings(),
            },
        ],
    };
    let out_dir = tmp.path().join("out");
    render_album_plan_impl(&request, &out_dir, None).expect("render")
}

#[test]
fn mixed_source_rates_resample_to_common_album_rate() {
    // 44.1 kHz + 48 kHz sources, explicit 48 kHz delivery — must NOT error,
    // and every output WAV must be 48 kHz.
    let tmp = TempDir::new().expect("tempdir");
    let report = render_two_track_album(&tmp, 44_100, 48_000, Some(48_000), None);
    for rec in &report.tracks {
        let spec = hound::WavReader::open(&rec.output_path)
            .expect("open track")
            .spec();
        assert_eq!(spec.sample_rate, 48_000, "per-track WAV must be 48 kHz");
    }
    let album_spec = hound::WavReader::open(&report.album_wav_path)
        .expect("open album")
        .spec();
    assert_eq!(album_spec.sample_rate, 48_000, "album.wav must be 48 kHz");
}

#[test]
fn explicit_cd_delivery_downsamples_48k_sources_to_44100_16bit() {
    let tmp = TempDir::new().expect("tempdir");
    let report = render_two_track_album(&tmp, 48_000, 48_000, Some(44_100), Some(16));
    let album_spec = hound::WavReader::open(&report.album_wav_path)
        .expect("open album")
        .spec();
    assert_eq!(album_spec.sample_rate, 44_100);
    assert_eq!(album_spec.bits_per_sample, 16);
    let manifest = std::fs::read_to_string(&report.manifest_path).expect("manifest");
    let parsed: serde_json::Value = serde_json::from_str(&manifest).expect("json");
    assert_eq!(parsed["sample_rate"], 44_100);
    assert_eq!(parsed["bit_depth"], 16);
}

#[test]
fn auto_delivery_picks_highest_source_rate() {
    // No explicit delivery; sources are 44.1 + 48 → album should be 48 kHz.
    let tmp = TempDir::new().expect("tempdir");
    let report = render_two_track_album(&tmp, 44_100, 48_000, None, None);
    let album_spec = hound::WavReader::open(&report.album_wav_path)
        .expect("open album")
        .spec();
    assert_eq!(album_spec.sample_rate, 48_000);
}
