use std::f32::consts::TAU;

use hound::{SampleFormat, WavSpec, WavWriter};
use tempfile::tempdir;
use yes_master_iphone_lib::iphone_render_master_to_path;
use yes_master_lib::{
    AdvancedSettings, CompressionMode, DeliveryProfile, JobStatus, MasteringSettings, Preset,
    RenderKind,
};

#[test]
fn iphone_render_master_to_path_uses_shared_dsp_engine() {
    let temp = tempdir().expect("tempdir");
    let source = temp.path().join("source.wav");
    let output = temp.path().join("iphone-master.wav");
    write_test_wav(&source);

    let job = iphone_render_master_to_path(
        "iphone-track".to_string(),
        &source,
        &default_iphone_settings(),
        &output,
    )
    .expect("render should succeed");

    assert!(output.exists());
    assert!(matches!(job.kind, RenderKind::Master));
    assert!(matches!(job.status, JobStatus::Done));
    assert_eq!(job.output_paths, vec![output.to_string_lossy().to_string()]);
    let measurements = job.measurements.expect("render measurements");
    assert_eq!(measurements.sample_rate, 48_000);
    assert_eq!(measurements.bit_depth, 24);
    assert!(measurements.lufs_integrated.is_finite());
}

fn default_iphone_settings() -> MasteringSettings {
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
            lufs_offset_db: Some(-14.0),
            ceiling_dbtp: Some(-1.0),
            width: None,
            warmth: None,
            presence_air: None,
            compression_mode: CompressionMode::Preset,
            compression_density: None,
            compression_low_threshold_db: None,
            compression_low_ratio: None,
            compression_low_attack_ms: None,
            compression_low_release_ms: None,
            compression_mid_threshold_db: None,
            compression_mid_ratio: None,
            compression_mid_attack_ms: None,
            compression_mid_release_ms: None,
            compression_high_threshold_db: None,
            compression_high_ratio: None,
            compression_high_attack_ms: None,
            compression_high_release_ms: None,
            compression_link_stereo: None,
            bit_depth: Some(24),
            target_sample_rate: Some(48_000),
        },
    }
}

fn write_test_wav(path: &std::path::Path) {
    let spec = WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).expect("create wav");
    for index in 0..44_100 {
        let phase = index as f32 * 440.0 * TAU / 44_100.0;
        let sample = (phase.sin() * i16::MAX as f32 * 0.2) as i16;
        writer.write_sample(sample).expect("left");
        writer.write_sample(sample).expect("right");
    }
    writer.finalize().expect("finalize wav");
}
