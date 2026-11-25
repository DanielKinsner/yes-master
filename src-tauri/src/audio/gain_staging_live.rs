//! Auto gain staging during audition (plan Slice 5).
//!
//! The gains are resolved off the audio thread, by the export's own
//! resolution ([`crate::gain_staging::resolve_preview`]), on the existing
//! preview worker or on `auto_gain_staging_readout`'s background task. Both
//! share the one preview-worker permit and this cache, so a song's settings
//! are resolved once and the line, the live chain and an export agree.
//!
//! Contract rule 8: while a resolution is in flight, level in applies at once
//! (from the track analysis), the limiter drive holds or falls and never
//! rises on an unverified value, and the state is `Measuring`, never
//! settled. The audio command thread only ever `try_lock`s this cache, so it
//! never waits on a worker; the audio callback never touches it.
use super::{settings_landing_hash, LandingSettingsHash, PlaybackPcm};
use crate::{
    analysis::SourceLevel,
    gain_staging::Resolution,
    types::{AutoGainStagingReadout, CommandResult, MasteringSettings, ResolvedGainStaging},
};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Mutex, OnceLock},
    time::SystemTime,
};

/// Resolutions kept: one per distinct settings edit per song.
const MAX_ENTRIES: usize = 64;
/// Source levels kept: one per song.
const MAX_LEVELS: usize = 16;

/// One song (file and modification time) with one set of settings, without
/// any resolved gains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Key {
    path: PathBuf,
    mtime: Option<SystemTime>,
    settings: u64,
}

impl Key {
    /// `None` for settings that cannot be keyed (non-finite values), which
    /// keeps the feature inert for them.
    pub(super) fn new(
        path: &Path,
        mtime: Option<SystemTime>,
        settings: &MasteringSettings,
    ) -> Option<Self> {
        let mut unresolved = settings.clone();
        unresolved.advanced.gain_staging_resolved = None;
        match settings_landing_hash(&unresolved) {
            LandingSettingsHash::Stable(settings) => Some(Self {
                path: path.to_path_buf(),
                mtime,
                settings,
            }),
            LandingSettingsHash::AlwaysMiss => None,
        }
    }

    fn same_source(&self, path: &Path, mtime: Option<SystemTime>) -> bool {
        self.path == path && self.mtime == mtime
    }
}

type Entries = VecDeque<(Key, Resolution)>;

fn cache() -> &'static Mutex<Entries> {
    static CACHE: OnceLock<Mutex<Entries>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn levels() -> &'static Mutex<VecDeque<(PathBuf, Option<SystemTime>, SourceLevel)>> {
    type Levels = Mutex<VecDeque<(PathBuf, Option<SystemTime>, SourceLevel)>>;
    static LEVELS: OnceLock<Levels> = OnceLock::new();
    LEVELS.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn find(entries: &Entries, key: &Key) -> Option<Resolution> {
    entries.iter().find(|(k, _)| k == key).map(|(_, r)| *r)
}

/// For the audio command thread: never waits. A busy lock reads as a miss,
/// which only means the song stays pending until the 50 ms retry.
pub(super) fn try_lookup(key: &Key) -> Option<Resolution> {
    find(&*cache().try_lock().ok()?, key)
}

/// For workers and commands.
pub(super) fn lookup(key: &Key) -> Option<Resolution> {
    find(
        &cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        key,
    )
}

fn insert(key: Key, resolution: Resolution) {
    let mut entries = cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    entries.retain(|(k, _)| *k != key);
    while entries.len() >= MAX_ENTRIES {
        entries.pop_front();
    }
    entries.push_back((key, resolution));
}

/// The source's level, measured once per song with the export's own
/// measurement (`measure_source_level` over the decoded file), so audition's
/// level in is exactly the export's.
fn source_level(key: &Key, pcm: &PlaybackPcm) -> CommandResult<SourceLevel> {
    if let Some(level) = levels()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .find(|(path, mtime, _)| key.same_source(path, *mtime))
        .map(|(_, _, level)| *level)
    {
        return Ok(level);
    }
    let level = crate::analysis::measure_source_level(&pcm.samples, pcm.sample_rate, pcm.channels)?;
    let mut entries = levels()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    while entries.len() >= MAX_LEVELS {
        entries.pop_front();
    }
    entries.push_back((key.path.clone(), key.mtime, level));
    Ok(level)
}

/// Resolve one song's settings, or reuse a resolution already made. Run only
/// while holding the preview-worker permit. A cancelled resolution is never
/// stored. `Ok(None)`: the feature is inert for these settings.
pub(super) fn resolve_cached(
    key: &Key,
    pcm: &PlaybackPcm,
    settings: &MasteringSettings,
    cancel: &AtomicBool,
) -> CommandResult<Option<Resolution>> {
    if let Some(hit) = lookup(key) {
        return Ok(Some(hit));
    }
    let level = source_level(key, pcm)?;
    let mut unresolved = settings.clone();
    unresolved.advanced.gain_staging_resolved = None;
    let resolution = crate::gain_staging::resolve_preview(
        &pcm.samples,
        pcm.sample_rate,
        pcm.channels,
        &unresolved,
        Some(&level),
        Some(cancel),
    )?;
    if let Some(resolution) = resolution {
        if !cancel.load(std::sync::atomic::Ordering::Relaxed) {
            insert(key.clone(), resolution);
        }
    }
    Ok(resolution)
}

/// What the live chain does with one song's settings.
#[cfg(test)]
#[path = "gain_staging_live_tests.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum LivePlan {
    /// Off, or inert: the chain is exactly today's. The readout is `None`
    /// while the switch is off.
    Inert(Option<AutoGainStagingReadout>),
    /// A verified resolution: apply it.
    Settled(Resolution),
    /// No verified resolution yet. Apply `pending` (level in from analysis,
    /// the drive held or lowered) and resolve `key` on the worker.
    Pending {
        key: Key,
        pending: ResolvedGainStaging,
    },
}

impl LivePlan {
    /// The resolution the worker must make, if any.
    pub(super) fn key(&self) -> Option<Key> {
        match self {
            LivePlan::Pending { key, .. } => Some(key.clone()),
            _ => None,
        }
    }
}

/// Decide the live gains for `settings` (contract rules 6 to 8).
///
/// - `album`: the live surface is Album.
/// - `analysis`: the track analysis's own level, when it has arrived.
/// - `source`: the loaded file and its modification time.
/// - `held`: the gains the chain applies now for this same source, if any.
pub(super) fn plan(
    settings: &MasteringSettings,
    album: bool,
    analysis: Option<SourceLevel>,
    source: Option<(&Path, Option<SystemTime>)>,
    held: Option<ResolvedGainStaging>,
) -> LivePlan {
    if !settings.auto_gain_staging_enabled() {
        return LivePlan::Inert(None);
    }
    if let Some(readout) = crate::gain_staging::inert_readout(settings, album) {
        return LivePlan::Inert(Some(readout));
    }
    let level_in = match analysis {
        Some(level) => match crate::gain_staging::level_in_for(&level) {
            Some(level_in) => Some(level_in),
            None => return LivePlan::Inert(Some(AutoGainStagingReadout::Unusable)),
        },
        None => None,
    };
    let Some(key) = source.and_then(|(path, mtime)| Key::new(path, mtime, settings)) else {
        return LivePlan::Inert(Some(AutoGainStagingReadout::Unusable));
    };
    if let Some(hit) = try_lookup(&key) {
        return LivePlan::Settled(hit);
    }
    let held = held.unwrap_or_default();
    LivePlan::Pending {
        key,
        pending: ResolvedGainStaging {
            // Level in applies at once when the analysis is in; until then
            // the chain keeps what it applies now.
            level_in_db: level_in.unwrap_or(held.level_in_db),
            // Hold, never raise, on an unverified value.
            limiter_drive_db: held.limiter_drive_db,
            limiter_load_db: 0.0,
            stopped_at_clean_limit: false,
            stopped_by_peak_check: false,
        },
    }
}

/// Write a plan into the settings the chain is built from, and return the
/// readout for it.
pub(super) fn apply(
    plan: &LivePlan,
    settings: &mut MasteringSettings,
) -> Option<AutoGainStagingReadout> {
    match plan {
        LivePlan::Inert(readout) => {
            settings.advanced.gain_staging_resolved = None;
            *readout
        }
        LivePlan::Settled(resolution) => {
            settings.advanced.gain_staging_resolved = Some(resolution.gains);
            Some(resolution.readout())
        }
        LivePlan::Pending { pending, .. } => {
            settings.advanced.gain_staging_resolved = Some(*pending);
            Some(AutoGainStagingReadout::Measuring)
        }
    }
}
