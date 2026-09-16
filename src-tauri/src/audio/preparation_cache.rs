//! Bounded reuse of a post-chain response and its full-file measurements.
//! The existing single preview-worker permit owns expensive work; this cache
//! never runs on an audio callback or copies PCM under a lock.
use super::{settings_landing_values_are_finite, PlaybackPcm};
use crate::{
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

struct Key {
    source: Weak<Vec<f32>>,
    source_rate: u32,
    channels: u16,
    delivery_rate: u32,
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
            coeffs: ChainCoeffs::from_settings(pcm.sample_rate, &raw_settings),
        })
    }

    fn matches(&self, other: &Self) -> bool {
        self.source.ptr_eq(&other.source)
            && self.source_rate == other.source_rate
            && self.channels == other.channels
            && self.delivery_rate == other.delivery_rate
            && self.coeffs == other.coeffs
    }
}

struct Entry {
    key: Key,
    prepared: Arc<PreparedPreviewAudio>,
}
struct Cache {
    entries: VecDeque<Entry>,
    bytes: usize,
    budget: usize,
}

impl Cache {
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
    fn get(&mut self, key: &Key) -> Option<Arc<PreparedPreviewAudio>> {
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
    fn insert(&mut self, key: Key, prepared: Arc<PreparedPreviewAudio>) {
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
    CACHE.get_or_init(|| Mutex::new(Cache::new(PCM_BUDGET)))
}

pub(super) fn measure(
    pcm: &PlaybackPcm,
    settings: &MasteringSettings,
    cancel: &AtomicBool,
) -> CommandResult<PreviewLanding> {
    measure_with_cache(cache(), pcm, settings, cancel)
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
