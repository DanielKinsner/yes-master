//! Bounded reuse of a post-chain response and its full-file measurements.
//! The existing single preview-worker permit owns expensive work; this cache
//! never runs on an audio callback or copies PCM under a lock.
use super::{settings_landing_values_are_finite, PlaybackPcm};
use crate::{
    device_preparation::PreparedDevicePcm,
    dsp::ChainCoeffs,
    engine::{PreparedPreviewAudio, PreviewLanding},
    types::*,
};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock, Weak,
    },
    time::Instant,
};

const MAX_ENTRIES: usize = 2;
const PCM_BUDGET: usize = 192 * 1024 * 1024;
// Source-rate raw PCM is now retained alongside changed-file-rate PCM so an
// actual gain/SRC edit can be verified without rerunning nonlinear DSP.
const FILE_PCM_BUDGET: usize = 288 * 1024 * 1024;

pub(super) fn same_response(rate: u32, a: &MasteringSettings, b: &MasteringSettings) -> bool {
    if !settings_landing_values_are_finite(a) || !settings_landing_values_are_finite(b) {
        return false;
    }
    let mut a = a.clone();
    let mut b = b.clone();
    a.volume_match = false;
    b.volume_match = false;
    a.effective_sample_rate(rate) == b.effective_sample_rate(rate)
        && ChainCoeffs::from_settings(rate, &a) == ChainCoeffs::from_settings(rate, &b)
}

#[derive(Clone)]
struct Key {
    source: Weak<Vec<f32>>,
    source_rate: u32,
    channels: u16,
    delivery_rate: u32,
    device_rate: Option<u32>,
    coeffs: ChainCoeffs,
}

impl Key {
    fn new(pcm: &PlaybackPcm, settings: &MasteringSettings) -> Option<Self> {
        if !settings_landing_values_are_finite(settings) {
            return None;
        }
        let mut raw_settings = settings.clone();
        raw_settings.volume_match = false;
        Some(Self {
            source: Arc::downgrade(&pcm.samples),
            source_rate: pcm.sample_rate,
            channels: pcm.channels,
            delivery_rate: settings.effective_sample_rate(pcm.sample_rate),
            device_rate: None,
            coeffs: ChainCoeffs::from_settings(pcm.sample_rate, &raw_settings),
        })
    }

    fn matches(&self, other: &Self) -> bool {
        self.source.ptr_eq(&other.source)
            && self.source_rate == other.source_rate
            && self.channels == other.channels
            && self.delivery_rate == other.delivery_rate
            && self.device_rate == other.device_rate
            && self.coeffs == other.coeffs
    }
}

trait PcmBytes {
    fn pcm_bytes(&self) -> usize;
}
impl PcmBytes for PreparedPreviewAudio {
    fn pcm_bytes(&self) -> usize {
        self.pcm_bytes()
    }
}
impl PcmBytes for PreparedDevicePcm {
    fn pcm_bytes(&self) -> usize {
        self.pcm_bytes()
    }
}

struct Entry<T> {
    key: Key,
    prepared: Arc<T>,
}
struct Cache<T = PreparedPreviewAudio> {
    entries: VecDeque<Entry<T>>,
    bytes: usize,
    budget: usize,
}

impl<T: PcmBytes> Cache<T> {
    fn new(budget: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            bytes: 0,
            budget,
        }
    }
    fn prune(&mut self) {
        self.entries
            .retain(|entry| entry.key.source.strong_count() > 0);
        self.bytes = self
            .entries
            .iter()
            .map(|entry| entry.prepared.pcm_bytes())
            .sum();
    }
    fn get(&mut self, key: &Key) -> Option<Arc<T>> {
        self.prune();
        let index = self
            .entries
            .iter()
            .position(|entry| entry.key.matches(key))?;
        let entry = self.entries.remove(index)?;
        let result = Arc::clone(&entry.prepared);
        self.entries.push_back(entry);
        Some(result)
    }
    fn insert(&mut self, key: Key, prepared: Arc<T>) {
        self.prune();
        let size = prepared.pcm_bytes();
        if size > self.budget {
            return;
        }
        while self.entries.len() >= MAX_ENTRIES || self.bytes > self.budget - size {
            if let Some(entry) = self.entries.pop_front() {
                self.bytes -= entry.prepared.pcm_bytes();
            }
        }
        self.bytes += size;
        self.entries.push_back(Entry { key, prepared });
    }
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Cache::new(FILE_PCM_BUDGET)))
}

pub(super) fn measure(
    pcm: &PlaybackPcm,
    settings: &MasteringSettings,
    cancel: &AtomicBool,
) -> CommandResult<PreviewLanding> {
    measure_with_cache(cache(), pcm, settings, cancel)
}

pub(super) struct DeviceLanding {
    pub file_gain: f32,
    pub device_gain: f32,
}

/// Match the existing DSP landing shortcut exactly; the post-device correction
/// also recovers a requested attenuation that this shortcut would skip.
pub(super) fn applied_source_landing(gain: f32) -> f32 {
    if (gain - 1.).abs() > 1e-4 {
        gain
    } else {
        1.
    }
}

pub(super) fn measure_for_device(
    pcm: &PlaybackPcm,
    settings: &MasteringSettings,
    device_rate: u32,
    cancel: &AtomicBool,
) -> CommandResult<DeviceLanding> {
    static DEVICE_CACHE: OnceLock<Mutex<Cache<PreparedDevicePcm>>> = OnceLock::new();
    measure_device_with_cache(
        cache(),
        DEVICE_CACHE.get_or_init(|| Mutex::new(Cache::new(PCM_BUDGET))),
        pcm,
        settings,
        device_rate,
        cancel,
    )
}

fn measure_device_with_cache(
    file_cache: &Mutex<Cache>,
    device_cache: &Mutex<Cache<PreparedDevicePcm>>,
    pcm: &PlaybackPcm,
    settings: &MasteringSettings,
    device_rate: u32,
    cancel: &AtomicBool,
) -> CommandResult<DeviceLanding> {
    if cancel.load(Ordering::Relaxed) {
        return Err(CommandError::Render("device preparation cancelled".into()));
    }
    let key = Key::new(pcm, settings)
        .ok_or_else(|| CommandError::Render("invalid device preparation settings".into()))?;
    let mut device_key = key.clone();
    device_key.device_rate = Some(device_rate);
    let started = Instant::now();
    let (file, file_budget) = {
        let mut cache = file_cache
            .lock()
            .map_err(|e| CommandError::Other(e.to_string()))?;
        (cache.get(&key), cache.budget)
    };
    let file_hit = file.is_some();
    let file = match file {
        Some(file) => file,
        None => Arc::new(crate::engine::prepare_preview_audio(
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels,
            settings,
            Some(cancel),
        )?),
    };
    let file_prepare_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = Instant::now();
    let (device, device_budget) = {
        let mut cache = device_cache
            .lock()
            .map_err(|e| CommandError::Other(e.to_string()))?;
        (cache.get(&device_key), cache.budget)
    };
    let device_hit = device.is_some();
    let device = match device {
        Some(device) => device,
        None => Arc::new(PreparedDevicePcm::new(
            file.raw_pcm(),
            key.delivery_rate,
            pcm.channels,
            device_rate,
            Some(cancel),
        )?),
    };
    let device_prepare_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = Instant::now();
    // Keep source PCM alive until the actual landed/SRC response is built.
    // Delivery's temporary file copy is released before conversion begins.
    let file_result = file.finish(settings, Some(cancel))?;
    let file_verify_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = Instant::now();
    let source_gain = applied_source_landing(file_result.gain_lin);
    let desired_correction = if source_gain == 0. {
        1.
    } else {
        file_result.gain_lin / source_gain
    };
    let device_result = if source_gain == 1. {
        device.finish(
            desired_correction,
            settings.effective_ceiling_dbtp(),
            Some(cancel),
        )?
    } else {
        let actual = file.device_pcm_with_landing(source_gain, device_rate, Some(cancel))?;
        device.finish_converted_gain(
            actual,
            source_gain,
            desired_correction,
            settings.effective_ceiling_dbtp(),
            Some(cancel),
        )?
    };
    let retain_file = (file.pcm_bytes() <= file_budget).then_some(file);
    let retain_device = (device.pcm_bytes() <= device_budget).then_some(device);
    let device_verify_ms = started.elapsed().as_secs_f64() * 1000.;
    if cancel.load(Ordering::Relaxed) {
        return Err(CommandError::Render("device preparation cancelled".into()));
    }
    let result = DeviceLanding {
        file_gain: file_result.gain_lin,
        device_gain: device_result.protection.gain_lin,
    };
    drop(device_result);
    let file_bytes = {
        let mut cache = file_cache
            .lock()
            .map_err(|e| CommandError::Other(e.to_string()))?;
        if let Some(file) = retain_file.filter(|_| !file_hit) {
            cache.insert(key, file);
        }
        cache.bytes
    };
    let device_bytes = {
        let mut cache = device_cache
            .lock()
            .map_err(|e| CommandError::Other(e.to_string()))?;
        if let Some(device) = retain_device.filter(|_| !device_hit) {
            cache.insert(device_key, device);
        }
        cache.bytes
    };
    crate::diagnostics::info(format!("Device preparation: file_hit={file_hit} device_hit={device_hit} file_pcm_bytes={file_bytes} device_pcm_bytes={device_bytes} file_prepare_ms={file_prepare_ms:.3} device_prepare_ms={device_prepare_ms:.3} file_verify_ms={file_verify_ms:.3} device_verify_ms={device_verify_ms:.3} file_gain={} device_gain={}", result.file_gain, result.device_gain));
    Ok(result)
}

fn measure_with_cache(
    cache: &Mutex<Cache>,
    pcm: &PlaybackPcm,
    settings: &MasteringSettings,
    cancel: &AtomicBool,
) -> CommandResult<PreviewLanding> {
    if cancel.load(Ordering::Relaxed) {
        return Err(CommandError::Render("preview preparation cancelled".into()));
    }
    let Some(key) = Key::new(pcm, settings) else {
        return crate::engine::preview_landing_with_cancel(
            &pcm.samples,
            pcm.sample_rate,
            pcm.channels,
            settings,
            Some(cancel),
        );
    };
    let started = Instant::now();
    let cached = cache
        .lock()
        .map_err(|e| CommandError::Other(e.to_string()))?
        .get(&key);
    let hit = cached.is_some();
    let prepared = match cached {
        Some(prepared) => prepared,
        None => {
            let prepared = crate::engine::prepare_preview_audio(
                &pcm.samples,
                pcm.sample_rate,
                pcm.channels,
                settings,
                Some(cancel),
            )?;
            let budget = cache
                .lock()
                .map_err(|e| CommandError::Other(e.to_string()))?
                .budget;
            if prepared.pcm_bytes() > budget {
                // Oversized responses are delivered once in place; bypassing
                // retention must not add another whole-file copy either.
                let preparation_ms = started.elapsed().as_secs_f64() * 1000.;
                let verification_start = Instant::now();
                let result = prepared.finish_owned(settings, Some(cancel))?;
                if cancel.load(Ordering::Relaxed) {
                    return Err(CommandError::Render("preview preparation cancelled".into()));
                }
                crate::diagnostics::info(format!("Preview preparation: cache_bypass=byte_budget prepare_ms={preparation_ms:.3} whole_file_verify_ms={:.3}", verification_start.elapsed().as_secs_f64() * 1000.));
                return Ok(result);
            }
            Arc::new(prepared)
        }
    };
    let preparation_ms = started.elapsed().as_secs_f64() * 1000.;
    let verification_start = Instant::now();
    let result = prepared.finish(settings, Some(cancel))?;
    if cancel.load(Ordering::Relaxed) {
        return Err(CommandError::Render("preview preparation cancelled".into()));
    }
    let verification_ms = verification_start.elapsed().as_secs_f64() * 1000.;
    let bytes = {
        let mut cache = cache
            .lock()
            .map_err(|e| CommandError::Other(e.to_string()))?;
        if !hit {
            cache.insert(key, prepared);
        }
        cache.bytes
    };
    crate::diagnostics::info(format!("Preview preparation: cache_hit={hit} cache_pcm_bytes={bytes} prepare_ms={preparation_ms:.3} whole_file_verify_ms={verification_ms:.3} estimator={}", crate::peak_meter::VERSION));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DeliveryProfile, Preset};
    fn pcm() -> PlaybackPcm {
        PlaybackPcm {
            samples: Arc::new((0..24001).map(|i| (i as f32 * 0.31).sin() * 0.4).collect()),
            sample_rate: 48000,
            channels: 1,
        }
    }
    fn settings() -> MasteringSettings {
        let mut settings = super::super::tests::settings_with_intensity(0.7);
        settings.delivery_profile = DeliveryProfile::Custom;
        settings.advanced.lufs_offset_db = Some(-14.);
        settings.advanced.bit_depth = Some(24);
        settings
    }

    #[test]
    fn device_reuse_preserves_delivery_and_separates_rates_source_lifetime_and_byte_limits() {
        let pcm = pcm();
        let cancel = AtomicBool::new(false);
        let files = Mutex::new(Cache::new(PCM_BUDGET));
        let devices = Mutex::new(Cache::new(PCM_BUDGET));
        let mut settings = settings();
        settings.advanced.target_sample_rate = Some(96000);
        let mut file_pointer = None;
        let mut device_pointer = None;
        for (rate, target, bits) in [
            (44100, -14., 24),
            (44100, -23., 16),
            (44100, -9., 32),
            (48000, -14., 24),
        ] {
            settings.advanced.lufs_offset_db = Some(target);
            settings.advanced.bit_depth = Some(bits);
            let actual =
                measure_device_with_cache(&files, &devices, &pcm, &settings, rate, &cancel)
                    .unwrap();
            let raw = crate::engine::prepare_preview_audio(
                &pcm.samples,
                pcm.sample_rate,
                pcm.channels,
                &settings,
                None,
            )
            .unwrap();
            let file = raw.finish(&settings, None).unwrap();
            let applied = applied_source_landing(file.gain_lin);
            let desired = if applied == 0. {
                1.
            } else {
                file.gain_lin / applied
            };
            let mut direct_pcm = raw.device_pcm_with_landing(applied, rate, None).unwrap();
            let fresh =
                crate::output_protection::prepare(&direct_pcm, rate, pcm.channels, None).unwrap();
            let direct = crate::output_protection::finalize_prepared_device_gain(
                &mut direct_pcm,
                &fresh,
                desired,
                settings.effective_ceiling_dbtp(),
                None,
            )
            .unwrap();
            assert_eq!(actual.file_gain.to_bits(), file.gain_lin.to_bits());
            assert!(actual.device_gain <= direct.gain_lin);
            assert!(
                20. * (direct.gain_lin / actual.device_gain).log10() <= 0.0011,
                "reuse must not add more attenuation than its 0.001 dB residual reporting limit"
            );
            let file_cache = files.lock().unwrap();
            assert_eq!(file_cache.entries.len(), 1);
            let pointer = Arc::as_ptr(&file_cache.entries[0].prepared);
            assert_eq!(*file_pointer.get_or_insert(pointer), pointer);
            drop(file_cache);
            if rate == 44100 {
                let cache = devices.lock().unwrap();
                assert_eq!(cache.entries.len(), 1);
                let pointer = Arc::as_ptr(&cache.entries[0].prepared);
                assert_eq!(*device_pointer.get_or_insert(pointer), pointer);
            }
            let bypass_files = Mutex::new(Cache::new(1));
            let bypass_devices = Mutex::new(Cache::new(1));
            let bypass = measure_device_with_cache(
                &bypass_files,
                &bypass_devices,
                &pcm,
                &settings,
                rate,
                &cancel,
            )
            .unwrap();
            assert_eq!(bypass.file_gain, actual.file_gain);
            assert_eq!(bypass.device_gain, actual.device_gain);
            assert!(bypass_files.lock().unwrap().entries.is_empty());
            assert!(bypass_devices.lock().unwrap().entries.is_empty());
        }
        assert_eq!(devices.lock().unwrap().entries.len(), 2);
        let other = PlaybackPcm {
            samples: Arc::new((*pcm.samples).clone()),
            ..pcm.clone()
        };
        measure_device_with_cache(&files, &devices, &other, &settings, 48000, &cancel).unwrap();
        assert_eq!(files.lock().unwrap().entries.len(), 2);
        cancel.store(true, Ordering::Relaxed);
        assert!(
            measure_device_with_cache(&files, &devices, &pcm, &settings, 44100, &cancel).is_err()
        );
        drop(pcm);
        drop(other);
        files.lock().unwrap().prune();
        devices.lock().unwrap().prune();
        assert_eq!(files.lock().unwrap().bytes, 0);
        assert_eq!(devices.lock().unwrap().bytes, 0);
    }

    #[test]
    fn codec_pcm_changes_invalidate_rates_but_reuse_identical_raw_responses() {
        use super::super::output_route::preview_settings;
        use crate::export_format::ExportEncoding;
        let mut pcm = pcm();
        pcm.sample_rate = 96000;
        let mut requested = settings();
        requested.advanced.bit_depth = Some(32);
        requested.advanced.target_sample_rate = None;
        let wav = preview_settings(&requested, pcm.sample_rate, ExportEncoding::Wav);
        let mp3 = preview_settings(
            &requested,
            pcm.sample_rate,
            ExportEncoding::Mp3 { bitrate_kbps: 320 },
        );
        let aac = preview_settings(
            &requested,
            pcm.sample_rate,
            ExportEncoding::M4a { bitrate_kbps: 128 },
        );
        let flac = preview_settings(&requested, pcm.sample_rate, ExportEncoding::Flac);
        assert!(!Key::new(&pcm, &wav)
            .unwrap()
            .matches(&Key::new(&pcm, &mp3).unwrap()));
        assert!(Key::new(&pcm, &mp3)
            .unwrap()
            .matches(&Key::new(&pcm, &aac).unwrap()));
        assert!(
            Key::new(&pcm, &wav)
                .unwrap()
                .matches(&Key::new(&pcm, &flac).unwrap()),
            "precision-only edits reuse raw PCM but finalize their own precision"
        );
        let cache = Mutex::new(Cache::new(PCM_BUDGET));
        let cancel = AtomicBool::new(false);
        for settings in [&mp3, &aac, &wav, &flac] {
            let cached = measure_with_cache(&cache, &pcm, settings, &cancel).unwrap();
            let direct = crate::engine::preview_landing(
                &pcm.samples,
                pcm.sample_rate,
                pcm.channels,
                settings,
            )
            .unwrap();
            assert_eq!(cached.gain_lin, direct.gain_lin);
            assert_eq!(cached.mastered_lufs, direct.mastered_lufs);
        }
        assert_eq!(cache.lock().unwrap().entries.len(), 2);
    }

    #[test]
    fn reuse_keeps_full_delivery_identical_and_invalidates_processing_and_source() {
        let pcm = pcm();
        let cache = Mutex::new(Cache::new(PCM_BUDGET));
        let cancel = AtomicBool::new(false);
        let mut settings = settings();
        for (bits, target) in [16, 24, 32]
            .into_iter()
            .flat_map(|bits| [-14., -23., -9.].map(|target| (bits, target)))
        {
            settings.advanced.bit_depth = Some(bits);
            settings.advanced.lufs_offset_db = Some(target);
            let cached = measure_with_cache(&cache, &pcm, &settings, &cancel).unwrap();
            let uncached = crate::engine::preview_landing(
                &pcm.samples,
                pcm.sample_rate,
                pcm.channels,
                &settings,
            )
            .unwrap();
            assert_eq!(cached.gain_lin.to_bits(), uncached.gain_lin.to_bits());
            assert_eq!(
                cached.mastered_lufs.to_bits(),
                uncached.mastered_lufs.to_bits()
            );
            assert_eq!(cache.lock().unwrap().entries.len(), 1);
        }
        let original = Key::new(&pcm, &settings).unwrap();
        let mut changed = settings.clone();
        changed.advanced.bit_depth = Some(16);
        assert!(original.matches(&Key::new(&pcm, &changed).unwrap()));
        let mut changed = settings.clone();
        changed.volume_match = true;
        assert!(original.matches(&Key::new(&pcm, &changed).unwrap()));
        let mut changed = settings.clone();
        changed.eq_mid_db += 0.1;
        assert!(!original.matches(&Key::new(&pcm, &changed).unwrap()));
        let mut changed = settings.clone();
        changed.preset = Preset::Punch;
        assert!(!original.matches(&Key::new(&pcm, &changed).unwrap()));
        let mut changed = settings.clone();
        changed.advanced.compression_density = Some(0.6);
        assert!(!original.matches(&Key::new(&pcm, &changed).unwrap()));
        let mut changed = settings.clone();
        changed.advanced.source_profile = Some(SourceProfile {
            spectral_6: SpectralBalance6 {
                sub: 0.08,
                low: 0.22,
                low_mid: 0.2,
                mid: 0.2,
                presence: 0.25,
                air: 0.20,
            },
            dynamic_range_p95_p10_db: 2.,
            dynamic_range_lu: 2.,
            stereo_correlation: Some(0.1),
            stereo_width: 1.,
        });
        assert!(!original.matches(&Key::new(&pcm, &changed).unwrap()));
        let adapted = Key::new(&pcm, &changed).unwrap();
        changed.advanced.adaptive_strength = Some(0.);
        assert!(!adapted.matches(&Key::new(&pcm, &changed).unwrap()));
        let mut changed = settings.clone();
        changed.advanced.target_sample_rate = Some(44100);
        assert!(!original.matches(&Key::new(&pcm, &changed).unwrap()));
        let other_pcm = PlaybackPcm {
            samples: Arc::new((*pcm.samples).clone()),
            ..pcm.clone()
        };
        assert!(!original.matches(&Key::new(&other_pcm, &settings).unwrap()));
        let invalid = {
            let mut s = settings.clone();
            s.eq_mid_db = f32::NAN;
            s
        };
        assert!(Key::new(&pcm, &invalid).is_none());
    }

    #[test]
    fn cache_budget_source_lifetime_and_cancelled_work_are_bounded() {
        let pcm = pcm();
        let settings = settings();
        let cancel = AtomicBool::new(false);
        let cache = Mutex::new(Cache::new(1));
        let bypass = measure_with_cache(&cache, &pcm, &settings, &cancel).unwrap();
        let direct =
            crate::engine::preview_landing(&pcm.samples, pcm.sample_rate, pcm.channels, &settings)
                .unwrap();
        assert_eq!(bypass.gain_lin.to_bits(), direct.gain_lin.to_bits());
        assert_eq!(
            bypass.mastered_lufs.to_bits(),
            direct.mastered_lufs.to_bits()
        );
        assert!(cache.lock().unwrap().entries.is_empty());
        let cache = Mutex::new(Cache::new(PCM_BUDGET));
        for eq in [0., 1., 2.] {
            let mut settings = settings.clone();
            settings.eq_mid_db = eq;
            measure_with_cache(&cache, &pcm, &settings, &cancel).unwrap();
        }
        assert_eq!(cache.lock().unwrap().entries.len(), 2);
        assert!(cache.lock().unwrap().bytes <= PCM_BUDGET);
        cancel.store(true, Ordering::Relaxed);
        assert!(measure_with_cache(&cache, &pcm, &settings, &cancel).is_err());
        drop(pcm);
        let mut cache = cache.lock().unwrap();
        cache.prune();
        assert_eq!(cache.bytes, 0);
        assert!(cache.entries.is_empty());
    }

    #[test]
    #[ignore = "controlled whole-song preview preparation timing over restored fixtures"]
    fn mastering_quality_preparation_cost_bench() {
        use serde_json::json;
        use sha2::{Digest, Sha256};
        let config: serde_json::Value = serde_json::from_slice(
            &std::fs::read(std::env::var("YES_MASTER_PREPARATION_INPUTS").unwrap()).unwrap(),
        )
        .unwrap();
        let output =
            std::path::PathBuf::from(std::env::var("YES_MASTER_PREPARATION_REPORT").unwrap());
        assert!(!output.exists());
        let mut rows = Vec::new();
        for source in config.as_array().unwrap() {
            let path = std::path::Path::new(source["path"].as_str().unwrap());
            let saved: serde_json::Value = serde_json::from_slice(
                &std::fs::read(source["preparation"].as_str().unwrap()).unwrap(),
            )
            .unwrap();
            let sha = format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()));
            assert_eq!(saved["source_sha256"].as_str().unwrap(), sha);
            let start = Instant::now();
            let pcm: PlaybackPcm = crate::decode::decode_full(path).unwrap().into();
            let decode_s = start.elapsed().as_secs_f64();
            let settings: MasteringSettings =
                serde_json::from_value(saved["settings"].clone()).unwrap();
            let mut sequence = vec![("first", settings.clone())];
            for (label, target) in [("target_lower", -18.), ("target_higher", -11.)] {
                let mut changed = settings.clone();
                changed.advanced.lufs_offset_db = Some(target);
                sequence.push((label, changed));
            }
            let mut changed = settings.clone();
            changed.eq_mid_db += 1.;
            sequence.push(("eq", changed));
            sequence.push(("return", settings.clone()));
            for trial in 0..3 {
                let mut reference = std::collections::HashMap::new();
                for strategy in if trial % 2 == 0 {
                    ["existing", "reuse"]
                } else {
                    ["reuse", "existing"]
                } {
                    let started_unix_s = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    let cache = Mutex::new(Cache::new(PCM_BUDGET));
                    let cancel = AtomicBool::new(false);
                    let mut scalar = std::collections::HashMap::<u64, (f32, f32)>::new();
                    let mut events = Vec::new();
                    let session = Instant::now();
                    for (label, settings) in &sequence {
                        let super::super::LandingSettingsHash::Stable(key) =
                            super::super::settings_landing_hash(settings)
                        else {
                            panic!("invalid settings");
                        };
                        let start = Instant::now();
                        let scalar_hit = scalar.contains_key(&key);
                        let raw_hit = cache
                            .lock()
                            .unwrap()
                            .get(&Key::new(&pcm, settings).unwrap())
                            .is_some();
                        let (gain, lufs) = match scalar.get(&key).copied() {
                            Some(result) => result,
                            None => {
                                let result = if strategy == "reuse" {
                                    measure_with_cache(&cache, &pcm, settings, &cancel).unwrap()
                                } else {
                                    crate::engine::preview_landing(
                                        &pcm.samples,
                                        pcm.sample_rate,
                                        pcm.channels,
                                        settings,
                                    )
                                    .unwrap()
                                };
                                scalar.insert(key, (result.gain_lin, result.mastered_lufs));
                                (result.gain_lin, result.mastered_lufs)
                            }
                        };
                        let seconds = start.elapsed().as_secs_f64();
                        if let Some(previous) =
                            reference.insert(*label, (gain.to_bits(), lufs.to_bits()))
                        {
                            assert_eq!(
                                previous,
                                (gain.to_bits(), lufs.to_bits()),
                                "cache changed delivery for {label}"
                            );
                        }
                        events.push(json!({"event":label,"seconds":seconds,"scalar_hit":scalar_hit,"raw_hit":raw_hit,"gain":gain,"lufs":lufs,"cache_bytes":cache.lock().unwrap().bytes}));
                    }
                    rows.push(json!({"source":source["id"],"source_sha256":sha,"decode_s":decode_s,"trial":trial,"strategy":strategy,"started_unix_s":started_unix_s,"ended_unix_s":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),"session_s":session.elapsed().as_secs_f64(),"events":events}));
                    std::fs::write(&output, serde_json::to_vec_pretty(&json!({"status":"in_progress","estimator":crate::peak_meter::VERSION,"cache_budget":PCM_BUDGET,"rows":rows})).unwrap()).unwrap();
                    println!(
                        "preparation cost: {} trial={trial} strategy={strategy}",
                        source["id"]
                    );
                }
            }
        }
        std::fs::write(&output, serde_json::to_vec_pretty(&json!({"status":"complete","estimator":crate::peak_meter::VERSION,"cache_budget":PCM_BUDGET,"rows":rows})).unwrap()).unwrap();
    }
}
