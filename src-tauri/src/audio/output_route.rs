//! Open CPAL explicitly while retaining Rodio's mixer, sink and fallback order.
//! Rodio 0.20's try_from_device_config can silently fall back to another format;
//! retaining its input configuration would misidentify the mixer rate.
use crate::{
    export_format::ExportEncoding,
    quality_source::QualitySource,
    sources::{FadeEnvelope, MasteringSource, MeteredSource},
    types::{DeliveryProfile, MasteringSettings},
};
use rodio::cpal::{
    self,
    traits::{DeviceTrait, StreamTrait},
    Device, SupportedStreamConfig,
};
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};

pub(super) type MasteredRateSource = MeteredSource<QualitySource<QualitySource<MasteringSource>>>;

/// Resolve only the pre-encode PCM contract, using the encoder's own rules.
/// This transient copy never replaces saved settings or simulates codec loss.
pub(super) fn preview_settings(
    settings: &MasteringSettings,
    source_rate: u32,
    encoding: ExportEncoding,
) -> MasteringSettings {
    let requested_rate = settings.effective_sample_rate(source_rate);
    let requested_bits = settings.effective_bit_depth();
    let rate = encoding.delivery_rate(requested_rate);
    let bits = encoding.delivery_bits(requested_bits);
    let mut resolved = settings.clone();
    if rate != requested_rate || bits != requested_bits {
        // A named profile shadows Advanced fields. Preserve its resolved target
        // and ceiling before using a transient Custom PCM delivery description.
        resolved.advanced.lufs_offset_db = settings.effective_target_lufs();
        resolved.advanced.ceiling_dbtp = Some(settings.effective_ceiling_dbtp());
        resolved.advanced.target_sample_rate = Some(rate);
        resolved.advanced.bit_depth = Some(bits);
        resolved.delivery_profile = DeliveryProfile::Custom;
    }
    resolved
}

/// Preserve the export's intermediate rate before conversion to the device.
/// Its antialias filter may remove content the export-derived gain would boost.
/// Meter/fade only the final device signal. No full-file analysis occurs here.
pub(super) fn mastered_source(
    mut source: MasteringSource,
    file_rate: u32,
    device_rate: u32,
    fade: FadeEnvelope,
) -> Result<(MasteredRateSource, Arc<AtomicU8>), String> {
    let slots = source.take_meter_slots();
    let revision = source.revision_slot();
    let file =
        QualitySource::new(source.with_render_alignment(), file_rate)?.with_revision(revision);
    let failure = file.error_slot();
    let revision = file.revision_slot();
    let device = QualitySource::new(file, device_rate)?
        .with_error_slot(failure.clone())
        .with_revision(revision);
    let revision = device.revision_slot();
    Ok((
        MeteredSource::new(device, slots, fade).with_revision(revision),
        failure,
    ))
}

pub(super) struct OutputHandle(Arc<rodio::dynamic_mixer::DynamicMixerController<f32>>);

impl OutputHandle {
    pub fn sink(&self) -> rodio::Sink {
        // Identical queue connection to Sink::try_new / OutputStream::play_raw.
        // The handle and native stream are owned by the same AudioThreadState.
        let (sink, queue) = rodio::Sink::new_idle();
        self.0.add(queue);
        sink
    }
}

pub(super) struct OpenedOutput {
    pub stream: cpal::Stream,
    pub handle: OutputHandle,
    pub config: SupportedStreamConfig,
    pub failed: Arc<AtomicBool>,
}

pub(super) fn open_device(device: &Device) -> Result<OpenedOutput, rodio::StreamError> {
    let config = device
        .default_output_config()
        .map_err(rodio::StreamError::DefaultStreamConfigError)?;
    let output = with_config_fallback(
        config,
        |config| build(device, config),
        || {
            let mut supported: Vec<_> = device
                .supported_output_configs()
                .map_err(rodio::StreamError::SupportedStreamConfigsError)?
                .collect();
            supported.sort_by(|a, b| b.cmp_default_heuristics(a));
            Ok(supported.into_iter().flat_map(|range| {
                let mut configs = vec![range.with_max_sample_rate()];
                let rate = cpal::SampleRate(44100);
                if rate < range.max_sample_rate() && rate > range.min_sample_rate() {
                    configs.push(range.with_sample_rate(rate));
                }
                configs.push(range.with_sample_rate(range.min_sample_rate()));
                configs
            }))
        },
    )?;
    // Rodio only tries other formats on build failure, not on play failure.
    output
        .stream
        .play()
        .map_err(rodio::StreamError::PlayStreamError)?;
    Ok(output)
}

fn with_config_fallback<C, T, I: Iterator<Item = C>>(
    preferred: C,
    mut build: impl FnMut(C) -> Result<T, cpal::BuildStreamError>,
    alternatives: impl FnOnce() -> Result<I, rodio::StreamError>,
) -> Result<T, rodio::StreamError> {
    build(preferred).or_else(|original| {
        alternatives()?
            .find_map(|config| build(config).ok())
            .ok_or(rodio::StreamError::BuildStreamError(original))
    })
}

fn build(
    device: &Device,
    config: SupportedStreamConfig,
) -> Result<OpenedOutput, cpal::BuildStreamError> {
    let (mixer, source) =
        rodio::dynamic_mixer::mixer::<f32>(config.channels(), config.sample_rate().0);
    let failed = Arc::new(AtomicBool::new(false));
    macro_rules! typed {
        ($sample:ty) => {
            build_typed::<$sample>(device, &config, source, failed.clone())?
        };
    }
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => typed!(f32),
        cpal::SampleFormat::F64 => typed!(f64),
        cpal::SampleFormat::I8 => typed!(i8),
        cpal::SampleFormat::I16 => typed!(i16),
        cpal::SampleFormat::I32 => typed!(i32),
        cpal::SampleFormat::I64 => typed!(i64),
        cpal::SampleFormat::U8 => typed!(u8),
        cpal::SampleFormat::U16 => typed!(u16),
        cpal::SampleFormat::U32 => typed!(u32),
        cpal::SampleFormat::U64 => typed!(u64),
        _ => return Err(cpal::BuildStreamError::StreamConfigNotSupported),
    };
    Ok(OpenedOutput {
        stream,
        handle: OutputHandle(mixer),
        config,
        failed,
    })
}

fn build_typed<T: cpal::SizedSample + cpal::FromSample<f32>>(
    device: &Device,
    config: &SupportedStreamConfig,
    mut source: rodio::dynamic_mixer::DynamicMixer<f32>,
    failed: Arc<AtomicBool>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    device.build_output_stream::<T, _, _>(
        &config.config(),
        move |output, _| {
            for sample in output {
                *sample = T::from_sample(source.next().unwrap_or(0.));
            }
        },
        move |_| {
            failed.store(true, Ordering::Release);
        },
        None,
    )
}

pub(super) fn with_fallback<D, T, E, I: Iterator<Item = D>>(
    preferred: &D,
    mut open: impl FnMut(&D) -> Result<T, E>,
    alternatives: impl FnOnce() -> Option<I>,
) -> Result<T, E> {
    open(preferred).or_else(|original| {
        let Some(mut alternatives) = alternatives() else {
            return Err(original);
        };
        alternatives
            .find_map(|device| open(&device).ok())
            .ok_or(original)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_codec_resolution_preserves_processing_and_requested_intent() {
        for encoding in [
            ExportEncoding::Wav,
            ExportEncoding::Mp3 { bitrate_kbps: 320 },
            ExportEncoding::Flac,
            ExportEncoding::M4a { bitrate_kbps: 256 },
            ExportEncoding::Aac { bitrate_kbps: 192 },
            ExportEncoding::Ogg { quality: 6 },
            ExportEncoding::Aiff,
        ] {
            for source_rate in [32000, 44100, 48000, 96000] {
                for profile in [
                    DeliveryProfile::StreamingUniversal,
                    DeliveryProfile::AppleMusic,
                    DeliveryProfile::Cd,
                    DeliveryProfile::VinylPremaster,
                    DeliveryProfile::LoudRock,
                    DeliveryProfile::BroadcastEu,
                    DeliveryProfile::BroadcastUs,
                    DeliveryProfile::Custom,
                ] {
                    for bits in [16, 24, 32] {
                        let mut original = super::super::tests::settings_with_intensity(0.75);
                        original.delivery_profile = profile;
                        original.advanced.target_sample_rate = None;
                        original.advanced.bit_depth = Some(bits);
                        original.advanced.lufs_offset_db = None;
                        let before = serde_json::to_value(&original).unwrap();
                        let resolved = preview_settings(&original, source_rate, encoding);
                        assert_eq!(
                            resolved.effective_sample_rate(source_rate),
                            encoding.delivery_rate(original.effective_sample_rate(source_rate))
                        );
                        assert_eq!(
                            resolved.effective_bit_depth(),
                            encoding.delivery_bits(original.effective_bit_depth())
                        );
                        assert_eq!(
                            resolved.effective_target_lufs(),
                            original.effective_target_lufs()
                        );
                        assert_eq!(
                            resolved.effective_ceiling_dbtp(),
                            original.effective_ceiling_dbtp()
                        );
                        assert_eq!(
                            crate::dsp::ChainCoeffs::from_settings(source_rate, &resolved),
                            crate::dsp::ChainCoeffs::from_settings(source_rate, &original)
                        );
                        assert_eq!(serde_json::to_value(&original).unwrap(), before);
                        assert_eq!(
                            serde_json::to_value(preview_settings(
                                &resolved,
                                source_rate,
                                encoding
                            ))
                            .unwrap(),
                            serde_json::to_value(resolved).unwrap(),
                            "resolution must be idempotent on a rate rebuild"
                        );
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "muted native codec PCM route updates; set YES_MASTER_BENCH_MUTE and YES_MASTER_CODEC_REPORT"]
    fn mastering_quality_codec_native_routes() {
        use super::super::{
            handle_play_master, process_audio_command, AudioCommand, AudioThreadState,
        };
        use std::sync::{atomic::AtomicU64, Mutex, RwLock};
        assert!(std::env::var_os("YES_MASTER_BENCH_MUTE").is_some());
        let report = std::path::PathBuf::from(std::env::var("YES_MASTER_CODEC_REPORT").unwrap());
        assert!(!report.exists());
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("codec-route.wav");
        let mut writer = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: 2,
                sample_rate: 96000,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .unwrap();
        for i in 0..96000 * 4 {
            let sample = 0.05 * (i as f32 * 0.13).sin();
            writer.write_sample(sample).unwrap();
            writer.write_sample(-sample).unwrap();
        }
        writer.finalize().unwrap();
        let selected = Arc::new(RwLock::new(None));
        let prewarm = Arc::new(Mutex::new(None));
        let epoch = Arc::new(AtomicU64::new(1));
        let profiles = Arc::new(crate::profile_store::SourceProfileStore::default());
        let (tx, _) = std::sync::mpsc::channel();
        let mut state: Option<AudioThreadState> = None;
        let mut loop_region = None;
        let mut settings = super::super::tests::settings_with_intensity(0.75);
        settings.delivery_profile = DeliveryProfile::Custom;
        settings.advanced.target_sample_rate = Some(96000);
        settings.advanced.bit_depth = Some(32);
        handle_play_master(
            &mut state,
            &mut loop_region,
            crate::types::TrackId("codec-route".into()),
            &path,
            &settings,
            0.1,
            false,
            false,
            false,
            ExportEncoding::Wav,
            &selected,
            &prewarm,
            &epoch,
            1,
            &tx,
            &profiles,
        )
        .unwrap();
        let mut rows = Vec::new();
        for encoding in [
            ExportEncoding::Mp3 { bitrate_kbps: 320 },
            ExportEncoding::Flac,
            ExportEncoding::M4a { bitrate_kbps: 256 },
            ExportEncoding::Aac { bitrate_kbps: 192 },
            ExportEncoding::Ogg { quality: 6 },
            ExportEncoding::Aiff,
            ExportEncoding::Wav,
        ] {
            let start = std::time::Instant::now();
            assert!(!process_audio_command(
                AudioCommand::UpdateChain {
                    settings: settings.clone(),
                    preview_lufs_landing: false,
                    album: false,
                    encoding
                },
                &mut state,
                &mut loop_region,
                &selected,
                &prewarm,
                &epoch,
                &tx,
                &profiles
            ));
            let state = state.as_ref().unwrap();
            assert_eq!(state.live_file_rate, encoding.delivery_rate(96000));
            assert!(!state.sink.is_paused());
            assert!(state.playback_error.is_none());
            while state.peak_linear.load(Ordering::Acquire) == 0 {
                assert!(start.elapsed() < std::time::Duration::from_secs(2));
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert!(!state.stream_failed.load(Ordering::Acquire));
            assert_eq!(
                state
                    .source_failure
                    .as_ref()
                    .unwrap()
                    .load(Ordering::Acquire),
                0
            );
            rows.push(serde_json::json!({"encoding":encoding,"file_rate":state.live_file_rate,
                "device_rate":state._output_config.sample_rate().0,"observed_s":start.elapsed().as_secs_f64()}));
        }
        state.as_ref().unwrap().sink.stop();
        std::fs::write(report,serde_json::to_vec_pretty(&serde_json::json!({"status":"complete","rows":rows,
            "scope":"actual native play/update handlers with synthetic 96 kHz source, test-only mute; pre-encode PCM rate selection, no codec-decode or listening proof"})).unwrap()).unwrap();
    }

    #[test]
    fn format_fallback_records_the_successful_configuration() {
        let mut attempts = Vec::new();
        let result = with_config_fallback(
            96000,
            |rate| {
                attempts.push(rate);
                if rate == 44100 {
                    Ok(rate)
                } else {
                    Err(cpal::BuildStreamError::StreamConfigNotSupported)
                }
            },
            || Ok([48000, 44100, 32000].into_iter()),
        )
        .unwrap();
        assert_eq!(result, 44100);
        assert_eq!(attempts, [96000, 48000, 44100]);
        let result = with_config_fallback(
            0,
            |_| Err::<(), _>(cpal::BuildStreamError::StreamConfigNotSupported),
            || Err::<std::vec::IntoIter<u32>, _>(rodio::StreamError::NoDevice),
        );
        assert!(matches!(result, Err(rodio::StreamError::NoDevice)));
        let result = with_config_fallback(
            0,
            |_| Err::<(), _>(cpal::BuildStreamError::StreamConfigNotSupported),
            || Ok([1].into_iter()),
        );
        assert!(matches!(
            result,
            Err(rodio::StreamError::BuildStreamError(
                cpal::BuildStreamError::StreamConfigNotSupported
            ))
        ));
    }

    #[test]
    fn successful_default_never_enumerates_or_guesses_another_format() {
        let result = with_fallback(
            &44100,
            |rate| Ok::<_, &str>(*rate),
            || -> Option<std::vec::IntoIter<u32>> {
                panic!("must not enumerate");
            },
        );
        assert_eq!(result, Ok(44100));
    }

    #[test]
    fn fallback_returns_the_actual_success_and_preserves_the_original_error() {
        let mut attempts = Vec::new();
        let result = with_fallback(
            &96000,
            |rate| {
                attempts.push(*rate);
                if *rate == 48000 {
                    Ok(*rate)
                } else {
                    Err("default failure")
                }
            },
            || Some(vec![44100, 48000, 32000].into_iter()),
        );
        assert_eq!(attempts, vec![96000, 44100, 48000]);
        assert_eq!(result, Ok(48000));
        for devices in [None, Some(Vec::new()), Some(vec![1, 2])] {
            let result = with_fallback(
                &0,
                |device| Err::<(), _>(if *device == 0 { "original" } else { "other" }),
                || devices.map(Vec::into_iter),
            );
            assert_eq!(result, Err("original"));
        }
    }

    #[test]
    #[ignore = "opens silent native streams to record default and explicitly selected configurations"]
    fn mastering_quality_opened_output_config() {
        use rodio::cpal::traits::HostTrait;
        let output =
            std::path::PathBuf::from(std::env::var("YES_MASTER_OUTPUT_CONFIG_REPORT").unwrap());
        assert!(!output.exists());
        let default = rodio::cpal::default_host().default_output_device().unwrap();
        let name = default.name().unwrap();
        let mut rows = Vec::new();
        for selected in [None, Some(name.as_str())] {
            let state = super::super::AudioThreadState::open(selected, 44100).unwrap();
            assert!(state.sink.empty());
            state.sink.set_volume(0.);
            state.sink.append(rodio::buffer::SamplesBuffer::new(
                2,
                44100,
                vec![0.1_f32; 44100 * 2 * 4],
            ));
            let start = std::time::Instant::now();
            while state.sink.get_pos() < std::time::Duration::from_millis(40) {
                assert!(start.elapsed() < std::time::Duration::from_secs(2));
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            let first_progress_ms = start.elapsed().as_secs_f64() * 1000.;
            state.sink.pause();
            state
                .sink
                .try_seek(std::time::Duration::from_secs(1))
                .unwrap();
            assert!(state.sink.is_paused());
            state.sink.play();
            let start = std::time::Instant::now();
            while state.sink.get_pos() < std::time::Duration::from_millis(1040) {
                assert!(start.elapsed() < std::time::Duration::from_secs(2));
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            let seek_resume_ms = start.elapsed().as_secs_f64() * 1000.;
            assert!(!state.stream_failed.load(Ordering::Acquire));
            state.sink.stop();
            rows.push(serde_json::json!({"selection":selected,
                "stream_rate":state._output_config.sample_rate().0,
                "stream_channels":state._output_config.channels(),
                "sample_format":format!("{:?}",state._output_config.sample_format()),
                "first_progress_ms":first_progress_ms,"seek_resume_ms":seek_resume_ms,
                "native_error":state.stream_failed.load(Ordering::Acquire)}));
        }
        std::fs::write(output, serde_json::to_vec_pretty(&serde_json::json!({"rows":rows,
                "scope":"muted native opening and Rodio sink progress/pause/seek/resume; actual CPAL build configuration retained; no DAC/listening or new SRC claim"})).unwrap()).unwrap();
    }

    #[test]
    #[ignore = "muted native failure injection; verifies processing errors do not look like normal end-of-file"]
    fn mastering_quality_stream_failure_is_visible() {
        use crate::{
            quality_source::QualitySource,
            sources::{FadeEnvelope, MeteredPcmSource, MeteredSource},
        };
        let mut state = super::super::AudioThreadState::open(None, 44100).unwrap();
        state.sink.set_volume(0.);
        let mut source = MeteredPcmSource::new(
            vec![0.1, 0.1, f32::NAN, 0.1],
            2,
            44100,
            state.peak_linear.clone(),
            state.peak_left_linear.clone(),
            state.peak_right_linear.clone(),
            state.lufs_x100.clone(),
            state.integrated_lufs_x100.clone(),
            state.spectrum_ring.clone(),
        );
        let slots = source.take_meter_slots();
        let source = QualitySource::new(source, state._output_config.sample_rate().0).unwrap();
        state.source_failure = Some(source.error_slot());
        state.play_generation = 17;
        state
            .sink
            .append(MeteredSource::new(source, slots, FadeEnvelope::inactive()));
        let start = std::time::Instant::now();
        while state
            .source_failure
            .as_ref()
            .unwrap()
            .load(Ordering::Acquire)
            == 0
        {
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        state.observe_source_failure();
        let error = state.playback_error.as_ref().unwrap();
        assert_eq!(error.generation, state.preview_work.epoch);
        assert!(!state.preview_work.cancelled.load(Ordering::Relaxed));
        assert!(error.message.contains("Playback stopped"));
        assert!(state.sink.is_paused());
        assert!(!state.device_lost);
        assert!(!state.landing_pending);
        assert_eq!(
            state
                .source_failure
                .as_ref()
                .unwrap()
                .load(Ordering::Acquire),
            1
        );
        assert!(state.sink.try_seek(std::time::Duration::ZERO).is_err() || state.sink.empty());
    }
}
