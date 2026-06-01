use std::f32::consts::TAU;

use hound::{SampleFormat, WavSpec, WavWriter};
use serde_json::{json, Value};
use tempfile::tempdir;
use yes_master_iphone_lib::{iphone_render_master_to_path, normalize_iphone_file_path};
use yes_master_lib::{
    AdvancedSettings, CompressionMode, DeliveryProfile, JobStatus, MasteringSettings, Preset,
    RenderKind,
};

#[test]
fn normalize_iphone_file_path_decodes_copied_document_urls() {
    let normalized = normalize_iphone_file_path(
        "file:///private/var/mobile/Containers/Data/It%E2%80%99s%20a%20Coat.mp3",
    );

    assert_eq!(
        normalized,
        "/private/var/mobile/Containers/Data/It’s a Coat.mp3"
    );
}

#[test]
fn normalize_iphone_file_path_decodes_plain_picker_paths() {
    let normalized = normalize_iphone_file_path("/tmp/YES%20Master/rough%20mix.wav");

    assert_eq!(normalized, "/tmp/YES Master/rough mix.wav");
}

#[test]
fn normalize_iphone_file_path_keeps_invalid_percent_text() {
    let normalized = normalize_iphone_file_path("/tmp/100% legit%ZZ.wav");

    assert_eq!(normalized, "/tmp/100% legit%ZZ.wav");
}

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

#[test]
fn simple_mode_settings_json_still_deserializes_to_rust_shape() {
    let value = simple_mode_settings_json();

    let settings: MasteringSettings =
        serde_json::from_value(value.clone()).expect("bindings.ts simple-mode shape should parse");

    assert!(matches!(settings.preset, Preset::Warmth));
    assert!(matches!(settings.delivery_profile, DeliveryProfile::Custom));
    assert!(matches!(
        settings.advanced.compression_mode,
        CompressionMode::Preset
    ));
    assert_eq!(settings.intensity, 0.5);
    assert!(settings.volume_match);
    assert_eq!(settings.eq_sub_db, 0.0);
    assert_eq!(settings.eq_low_db, 0.0);
    assert_eq!(settings.eq_low_mid_db, 0.0);
    assert_eq!(settings.eq_mid_db, 0.0);
    assert_eq!(settings.eq_high_mid_db, 0.0);
    assert_eq!(settings.eq_high_db, 0.0);
    assert_eq!(settings.eq_sparkle_db, 0.0);
    assert_eq!(settings.input_gain_db, 0.0);
    assert_eq!(settings.output_gain_db, 0.0);
    assert_eq!(settings.advanced.lufs_offset_db, Some(-14.0));
    assert_eq!(settings.advanced.ceiling_dbtp, Some(-1.0));
    assert_eq!(settings.advanced.width, None);
    assert_eq!(settings.advanced.warmth, None);
    assert_eq!(settings.advanced.presence_air, None);
    assert_eq!(settings.advanced.compression_density, None);
    assert_eq!(settings.advanced.compression_low_threshold_db, None);
    assert_eq!(settings.advanced.compression_low_ratio, None);
    assert_eq!(settings.advanced.compression_low_attack_ms, None);
    assert_eq!(settings.advanced.compression_low_release_ms, None);
    assert_eq!(settings.advanced.compression_mid_threshold_db, None);
    assert_eq!(settings.advanced.compression_mid_ratio, None);
    assert_eq!(settings.advanced.compression_mid_attack_ms, None);
    assert_eq!(settings.advanced.compression_mid_release_ms, None);
    assert_eq!(settings.advanced.compression_high_threshold_db, None);
    assert_eq!(settings.advanced.compression_high_ratio, None);
    assert_eq!(settings.advanced.compression_high_attack_ms, None);
    assert_eq!(settings.advanced.compression_high_release_ms, None);
    assert_eq!(settings.advanced.compression_link_stereo, None);
    assert_eq!(settings.advanced.bit_depth, Some(24));
    assert_eq!(settings.advanced.target_sample_rate, Some(48_000));
    assert_eq!(settings.source_lufs_integrated, Some(-15.0));

    let round_tripped = serde_json::to_value(&settings).expect("serialize settings");
    assert_eq!(sorted_json_keys(&round_tripped), sorted_json_keys(&value));
    assert_eq!(
        sorted_json_keys(&round_tripped["advanced"]),
        sorted_json_keys(&value["advanced"])
    );
    assert_eq!(round_tripped, value);
}

fn simple_mode_settings_json() -> Value {
    json!({
        "preset": { "kind": "warmth" },
        "intensity": 0.5,
        "eq_sub_db": 0.0,
        "eq_low_db": 0.0,
        "eq_low_mid_db": 0.0,
        "eq_mid_db": 0.0,
        "eq_high_mid_db": 0.0,
        "eq_high_db": 0.0,
        "eq_sparkle_db": 0.0,
        "volume_match": true,
        "source_lufs_integrated": -15.0,
        "input_gain_db": 0.0,
        "output_gain_db": 0.0,
        "delivery_profile": "custom",
        "album": null,
        "advanced": {
            "lufs_offset_db": -14.0,
            "ceiling_dbtp": -1.0,
            "width": null,
            "warmth": null,
            "presence_air": null,
            "compression_mode": "preset",
            "compression_density": null,
            "compression_low_threshold_db": null,
            "compression_low_ratio": null,
            "compression_low_attack_ms": null,
            "compression_low_release_ms": null,
            "compression_mid_threshold_db": null,
            "compression_mid_ratio": null,
            "compression_mid_attack_ms": null,
            "compression_mid_release_ms": null,
            "compression_high_threshold_db": null,
            "compression_high_ratio": null,
            "compression_high_attack_ms": null,
            "compression_high_release_ms": null,
            "compression_link_stereo": null,
            "bit_depth": 24,
            "target_sample_rate": 48000
        }
    })
}

fn sorted_json_keys(value: &Value) -> Vec<String> {
    let mut keys = value
        .as_object()
        .expect("expected JSON object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys
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

/// The exact regression behind the device "flat placeholder" waveform: a
/// percent-encoded `file://` URL from the iOS document picker must round-trip
/// through `normalize_iphone_file_path` and decode to non-empty peaks. If this
/// passes, the decode/normalize path is sound and any device failure is
/// environmental (missing file / codec), not a logic bug here.
#[test]
fn prepare_waveform_decodes_normalized_file_url() {
    let temp = tempdir().expect("tempdir");
    // Spaced filename exercises percent-decoding, mirroring the copied
    // document-picker path (e.g. ".../My Song.wav").
    let source = temp.path().join("My Song.wav");
    write_test_wav(&source);

    let url = format!("file://{}", source.to_string_lossy().replace(' ', "%20"));
    let normalized = normalize_iphone_file_path(&url);
    assert_eq!(normalized, source.to_string_lossy());

    let peaks = block_on(yes_master_lib::audio::prepare_waveform(
        yes_master_lib::TrackId("wave-track".to_string()),
        normalized,
        Some(140),
    ))
    .expect("waveform should decode from a normalized file:// URL");

    assert!(!peaks.channels.is_empty(), "expected at least one channel");
    let first = &peaks.channels[0];
    assert!(!first.is_empty(), "expected non-empty peak buckets");
    assert!(
        first.iter().any(|&value| value > 0.01),
        "expected real signal in the decoded waveform peaks"
    );
}

/// Minimal single-poll executor. `prepare_waveform`'s async body contains no
/// `.await` points (it delegates to the synchronous `decode_to_peaks`), so one
/// poll drives it to completion — no async runtime needed in the test crate.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::pin::pin;
    use std::ptr;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    unsafe fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(ptr::null(), &VTABLE)
    }
    unsafe fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);

    let waker = unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &VTABLE)) };
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("prepare_waveform unexpectedly awaited; test needs a real runtime"),
    }
}
