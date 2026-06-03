//! Backend-owned cache of derived adaptive source profiles, keyed by track.
//!
//! B2 root-cause refactor: the backend — not the frontend — owns deriving the
//! Tier-1 adaptive [`SourceProfile`]. `analyze_tracks` is the SINGLE derivation
//! point (via [`SourceProfile::from_analysis`]); every Track-Master chain entry
//! point (live `play_master` / `update_chain`, offline `render_track_*`, and the
//! `guardrail_readout`) resolves the effective profile from this store. That
//! dissolves the old "frontend forgot to inject at this call site" bug class and
//! the dual TS/Rust mapper drift by construction.
//!
//! Album Master is intentionally non-adaptive (owner decision): album surfaces
//! never read this store, and [`resolve_effective_profile`] hard-returns `None`
//! when `album` is set. The byte-identity invariant is unaffected — this store
//! only fills `settings.advanced.source_profile` at the command layer;
//! `ChainCoeffs::from_settings` still gates all trimming on the profile being
//! present and `strength > 0`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::types::{MasteringSettings, SourceProfile, TrackId};

/// Thread-safe map of `TrackId -> SourceProfile`. Shared (as an `Arc`) between
/// the analysis command (writer), the render/readout commands (readers), and the
/// audio thread (reader for the settings-only `update_chain` live path, which
/// carries no track id and resolves via the currently-loaded track instead).
#[derive(Default)]
pub struct SourceProfileStore {
    by_track: Mutex<HashMap<TrackId, SourceProfile>>,
}

impl SourceProfileStore {
    /// Insert (or replace) the derived profile for a track. Called by
    /// `analyze_tracks` after `SourceProfile::from_analysis` succeeds.
    pub fn insert(&self, track_id: TrackId, profile: SourceProfile) {
        if let Ok(mut guard) = self.by_track.lock() {
            guard.insert(track_id, profile);
        }
    }

    /// Insert when `profile` is `Some`, otherwise clear any prior entry. Used so
    /// a re-analysis that can no longer derive a profile (e.g. a now-too-short
    /// source) doesn't leave a stale one behind.
    pub fn set(&self, track_id: TrackId, profile: Option<SourceProfile>) {
        if let Ok(mut guard) = self.by_track.lock() {
            match profile {
                Some(p) => {
                    guard.insert(track_id, p);
                }
                None => {
                    guard.remove(&track_id);
                }
            }
        }
    }

    /// The cached profile for a track, if any. `SourceProfile` is `Copy`, so the
    /// lock is held only for the lookup.
    pub fn get(&self, track_id: &TrackId) -> Option<SourceProfile> {
        self.by_track
            .lock()
            .ok()
            .and_then(|guard| guard.get(track_id).copied())
    }
}

/// Resolve the effective adaptive source profile for one chain build (B2
/// precedence rules):
///
///   * **Album is never adaptive** — `album = true` hard-returns `None`, so an
///     album surface stays byte-flat regardless of what's cached or supplied.
///   * A **frontend-supplied** profile (`fe_override`) is treated as an explicit
///     override and wins over the cache.
///   * Otherwise the **backend-derived** cached profile is used.
///
/// A `None` result means "guardrails inert" — the caller leaves
/// `settings.advanced.source_profile` unset and the chain is byte-identical to
/// the non-adaptive path.
pub fn resolve_effective_profile(
    fe_override: Option<SourceProfile>,
    cached: Option<SourceProfile>,
    album: bool,
) -> Option<SourceProfile> {
    if album {
        return None;
    }
    fe_override.or(cached)
}

/// Apply the B2-resolved effective profile onto `settings.advanced.source_profile`
/// in place, treating any value already on `settings` as the frontend override
/// and `cached` as the backend-derived fallback. The single chokepoint every
/// chain-building command routes through, so the precedence (and the album
/// gate) are identical across the live, render, and readout paths.
pub fn apply_resolved_profile(
    settings: &mut MasteringSettings,
    cached: Option<SourceProfile>,
    album: bool,
) {
    settings.advanced.source_profile =
        resolve_effective_profile(settings.advanced.source_profile, cached, album);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SourceProfile, SpectralBalance6};

    fn profile(tag: f32) -> SourceProfile {
        SourceProfile {
            spectral_6: SpectralBalance6 {
                sub: 0.1,
                low: 0.2,
                low_mid: 0.2,
                mid: 0.2,
                presence: 0.15,
                air: 0.15,
            },
            dynamic_range_p95_p10_db: tag,
            dynamic_range_lu: 8.0,
            stereo_correlation: Some(0.5),
            stereo_width: 1.0,
        }
    }

    #[test]
    fn store_insert_get_roundtrip() {
        let store = SourceProfileStore::default();
        let id = TrackId("t1".to_string());
        assert_eq!(store.get(&id), None);
        store.insert(id.clone(), profile(7.0));
        assert_eq!(store.get(&id).map(|p| p.dynamic_range_p95_p10_db), Some(7.0));
    }

    #[test]
    fn store_set_some_then_none_clears() {
        let store = SourceProfileStore::default();
        let id = TrackId("t1".to_string());
        store.set(id.clone(), Some(profile(7.0)));
        assert!(store.get(&id).is_some());
        store.set(id.clone(), None);
        assert_eq!(store.get(&id), None, "re-analysis losing a profile must clear");
    }

    #[test]
    fn store_insert_replaces_prior() {
        let store = SourceProfileStore::default();
        let id = TrackId("t1".to_string());
        store.insert(id.clone(), profile(3.0));
        store.insert(id.clone(), profile(9.0));
        assert_eq!(store.get(&id).map(|p| p.dynamic_range_p95_p10_db), Some(9.0));
    }

    #[test]
    fn resolve_album_is_always_inert() {
        assert_eq!(
            resolve_effective_profile(Some(profile(1.0)), Some(profile(2.0)), true),
            None,
            "album must never adapt, even with an override or a cached profile"
        );
    }

    #[test]
    fn resolve_frontend_override_wins() {
        let r = resolve_effective_profile(Some(profile(1.0)), Some(profile(2.0)), false);
        assert_eq!(r.map(|p| p.dynamic_range_p95_p10_db), Some(1.0));
    }

    #[test]
    fn resolve_falls_back_to_cache() {
        let r = resolve_effective_profile(None, Some(profile(2.0)), false);
        assert_eq!(r.map(|p| p.dynamic_range_p95_p10_db), Some(2.0));
    }

    #[test]
    fn resolve_none_when_nothing_available() {
        assert_eq!(resolve_effective_profile(None, None, false), None);
    }

    fn settings_with(profile: Option<SourceProfile>) -> MasteringSettings {
        MasteringSettings {
            preset: crate::types::Preset::Universal,
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
            delivery_profile: crate::types::DeliveryProfile::Custom,
            album: None,
            advanced: crate::types::AdvancedSettings {
                source_profile: profile,
                ..Default::default()
            },
        }
    }

    #[test]
    fn apply_fills_from_cache_when_frontend_omits() {
        let mut s = settings_with(None);
        apply_resolved_profile(&mut s, Some(profile(5.0)), false);
        assert_eq!(
            s.advanced.source_profile.map(|p| p.dynamic_range_p95_p10_db),
            Some(5.0)
        );
    }

    #[test]
    fn apply_keeps_frontend_override_over_cache() {
        let mut s = settings_with(Some(profile(1.0)));
        apply_resolved_profile(&mut s, Some(profile(2.0)), false);
        assert_eq!(
            s.advanced.source_profile.map(|p| p.dynamic_range_p95_p10_db),
            Some(1.0)
        );
    }

    #[test]
    fn apply_clears_to_none_for_album_even_with_cache() {
        let mut s = settings_with(Some(profile(1.0)));
        apply_resolved_profile(&mut s, Some(profile(2.0)), true);
        assert_eq!(s.advanced.source_profile, None, "album must stay byte-flat");
    }
}
