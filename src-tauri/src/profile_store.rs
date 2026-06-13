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
use std::sync::{Arc, Mutex};

use crate::types::{MasteringSettings, SourceProfile, TrackId};

/// Thread-safe map of `TrackId -> SourceProfile`. Shared (as an `Arc`) between
/// the analysis command (writer), the render/readout commands (readers), and the
/// audio thread (reader for the settings-only `update_chain` live path, which
/// carries no track id and resolves via the currently-loaded track instead).
#[derive(Default)]
pub struct SourceProfileStore {
    by_track: Mutex<HashMap<TrackId, SourceProfile>>,
    /// Backend-internal DeepAnalysis (Tier-2 Phase A), keyed by track. Separate
    /// map from `by_track` so the two never need a combined lock; methods take
    /// at most one lock, and the two-map clear in `prune_failed_profiles` calls
    /// the setters sequentially (never holding both). Arc so readers share
    /// cheaply without cloning the per-window series.
    by_track_deep: Mutex<HashMap<TrackId, Arc<crate::deep_analysis::DeepAnalysis>>>,
    /// Adaptive Compressor already-mastered classification, derived at analysis
    /// time from loudness, true peak, LRA, and per-band PSR. Stored separately so
    /// chain entry points never need to keep whole `AnalysisResult`s alive.
    by_track_stand_down: Mutex<HashMap<TrackId, crate::guardrails::AlreadyMasteredStandDown>>,
}

impl SourceProfileStore {
    /// Insert (or replace) the derived profile for a track. Called by
    /// `analyze_tracks` after `SourceProfile::from_analysis` succeeds.
    pub fn insert(&self, track_id: TrackId, profile: SourceProfile) {
        self.by_track
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(track_id, profile);
    }

    /// Insert when `profile` is `Some`, otherwise clear any prior entry. Used so
    /// a re-analysis that can no longer derive a profile (e.g. a now-too-short
    /// source) doesn't leave a stale one behind.
    pub fn set(&self, track_id: TrackId, profile: Option<SourceProfile>) {
        let mut guard = self
            .by_track
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match profile {
            Some(p) => {
                guard.insert(track_id, p);
            }
            None => {
                guard.remove(&track_id);
            }
        }
    }

    /// The cached profile for a track, if any. `SourceProfile` is `Copy`, so the
    /// lock is held only for the lookup.
    pub fn get(&self, track_id: &TrackId) -> Option<SourceProfile> {
        self.by_track
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(track_id)
            .copied()
    }

    /// Insert when `Some`, otherwise clear any prior entry (mirrors `set` for the
    /// deep map). Locks only `by_track_deep`.
    pub fn insert_deep(
        &self,
        track_id: TrackId,
        deep: Option<Arc<crate::deep_analysis::DeepAnalysis>>,
    ) {
        let mut guard = self
            .by_track_deep
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match deep {
            Some(d) => {
                guard.insert(track_id, d);
            }
            None => {
                guard.remove(&track_id);
            }
        }
    }

    /// The cached DeepAnalysis for a track, if any (clones the `Arc`, cheap).
    pub fn get_deep(&self, track_id: &TrackId) -> Option<Arc<crate::deep_analysis::DeepAnalysis>> {
        self.by_track_deep
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(track_id)
            .cloned()
    }

    pub fn set_stand_down(
        &self,
        track_id: TrackId,
        stand_down: Option<crate::guardrails::AlreadyMasteredStandDown>,
    ) {
        let mut guard = self
            .by_track_stand_down
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match stand_down {
            Some(value) => {
                guard.insert(track_id, value);
            }
            None => {
                guard.remove(&track_id);
            }
        }
    }

    pub fn get_stand_down(
        &self,
        track_id: &TrackId,
    ) -> Option<crate::guardrails::AlreadyMasteredStandDown> {
        self.by_track_stand_down
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(track_id)
            .copied()
    }

    /// Remove both adaptive cache entries for a track. Used when the frontend
    /// removes a track from the session so backend-only DeepAnalysis data does
    /// not linger for the life of the app process.
    pub fn evict(&self, track_id: &TrackId) {
        self.set(track_id.clone(), None);
        self.insert_deep(track_id.clone(), None);
        self.set_stand_down(track_id.clone(), None);
    }
}

#[tauri::command]
pub fn evict_source_profile(
    track_id: TrackId,
    profile_store: tauri::State<'_, Arc<SourceProfileStore>>,
) {
    profile_store.evict(&track_id);
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

/// Tier-2 Phase B companion to [`apply_resolved_profile`]: inject the per-axis
/// confidence derived from the cached `DeepAnalysis` onto
/// `settings.advanced.source_confidence`. Delegates to
/// [`crate::confidence::resolve_source_confidence`], which resolves to `None`
/// (=> full confidence => the chain stays byte-identical to Tier-1) when any of: the
/// owner-calibration gate ([`crate::confidence::is_confidence_gating_enabled`]) is
/// off (the default — the provisional voicing must not reach a render until
/// A/B-validated), the surface is album (non-adaptive), or there is no cached deep
/// read. Confidence is backend-internal — no FE override.
pub fn apply_resolved_confidence(
    settings: &mut MasteringSettings,
    deep: Option<std::sync::Arc<crate::deep_analysis::DeepAnalysis>>,
    album: bool,
) {
    settings.advanced.source_confidence = crate::confidence::resolve_source_confidence(
        deep.as_deref(),
        album,
        crate::confidence::is_confidence_gating_enabled(),
    );
}

/// Adaptive Compressor companion to the profile/confidence injection path.
/// Resolves backend-owned `CompressionGuards` from the same cached DeepAnalysis
/// used for confidence plus the already-mastered stand-down summary populated
/// at analysis time. Gate OFF, album mode, no DeepAnalysis, or no trigger all
/// resolve to `None`, preserving the AC-2 byte-identity contract.
pub fn apply_resolved_compression_guards(
    settings: &mut MasteringSettings,
    deep: Option<std::sync::Arc<crate::deep_analysis::DeepAnalysis>>,
    stand_down: Option<crate::guardrails::AlreadyMasteredStandDown>,
    album: bool,
) {
    let strength = settings
        .advanced
        .adaptive_strength
        .unwrap_or(crate::guardrails::ADAPTIVE_STRENGTH_DEFAULT)
        .clamp(0.0, 1.0);
    let band_psr = deep
        .as_deref()
        .and_then(crate::deep_analysis::band_psr_p10_db);
    let confidence = settings.advanced.source_confidence.unwrap_or_default();
    settings.advanced.compression_guards = crate::guardrails::resolve_compression_guards(
        band_psr,
        &confidence,
        stand_down.unwrap_or_else(crate::guardrails::AlreadyMasteredStandDown::identity),
        strength,
        !album && crate::guardrails::is_adaptive_compression_enabled(),
    );
}

/// Evict cached profiles for any *requested* track that did NOT produce an
/// analysis result. A hard analysis failure (missing / unreadable / decode
/// error) is skipped under `analyze_tracks_core`'s partial-success policy, so
/// the populate path never sees it — and without this an in-session
/// re-analysis that now fails (e.g. a moved or replaced source carried under a
/// persisted project's `TrackId`) would leave the PRIOR profile cached and keep
/// adapting the audition from stale audio. The soft "too short / silent =>
/// `None`" case is already cleared by the normal `set(id, None)` populate path;
/// this closes the hard-failure gap. `succeeded` is the set of `TrackId`s that
/// produced a result this pass.
pub fn prune_failed_profiles(
    store: &SourceProfileStore,
    requested: &[TrackId],
    succeeded: &[TrackId],
) {
    for id in requested {
        if !succeeded.contains(id) {
            store.set(id.clone(), None);
            store.insert_deep(id.clone(), None);
            store.set_stand_down(id.clone(), None);
        }
    }
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
        assert_eq!(
            store.get(&id).map(|p| p.dynamic_range_p95_p10_db),
            Some(7.0)
        );
    }

    #[test]
    fn store_set_some_then_none_clears() {
        let store = SourceProfileStore::default();
        let id = TrackId("t1".to_string());
        store.set(id.clone(), Some(profile(7.0)));
        assert!(store.get(&id).is_some());
        store.set(id.clone(), None);
        assert_eq!(
            store.get(&id),
            None,
            "re-analysis losing a profile must clear"
        );
    }

    #[test]
    fn evict_clears_profile_and_deep_analysis() {
        let store = SourceProfileStore::default();
        let id = TrackId("t1".to_string());
        store.set(id.clone(), Some(profile(7.0)));
        store.insert_deep(id.clone(), Some(std::sync::Arc::new(make_test_deep())));

        store.evict(&id);

        assert_eq!(store.get(&id), None);
        assert!(store.get_deep(&id).is_none());
    }

    #[test]
    fn store_insert_replaces_prior() {
        let store = SourceProfileStore::default();
        let id = TrackId("t1".to_string());
        store.insert(id.clone(), profile(3.0));
        store.insert(id.clone(), profile(9.0));
        assert_eq!(
            store.get(&id).map(|p| p.dynamic_range_p95_p10_db),
            Some(9.0)
        );
    }

    #[test]
    fn prune_failed_profiles_clears_only_the_failed_track() {
        let store = SourceProfileStore::default();
        let kept = TrackId("kept".to_string());
        let failed = TrackId("failed".to_string());
        // Both carry a profile from an earlier successful analysis.
        store.insert(kept.clone(), profile(7.0));
        store.insert(failed.clone(), profile(7.0));
        // Re-analysis pass: only `kept` produced a result; `failed` hard-errored
        // (missing / unreadable source) and was skipped from the results.
        prune_failed_profiles(
            &store,
            &[kept.clone(), failed.clone()],
            std::slice::from_ref(&kept),
        );
        assert!(
            store.get(&kept).is_some(),
            "a track that still analyzes must keep its profile"
        );
        assert_eq!(
            store.get(&failed),
            None,
            "a track that failed re-analysis must have its stale profile evicted"
        );
    }

    #[test]
    fn prune_failed_profiles_clears_all_when_nothing_succeeded() {
        let store = SourceProfileStore::default();
        let a = TrackId("a".to_string());
        let b = TrackId("b".to_string());
        store.insert(a.clone(), profile(1.0));
        store.insert(b.clone(), profile(2.0));
        // Total failure (the analyze_tracks Err path) -> succeeded is empty.
        prune_failed_profiles(&store, &[a.clone(), b.clone()], &[]);
        assert_eq!(store.get(&a), None);
        assert_eq!(store.get(&b), None);
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
            s.advanced
                .source_profile
                .map(|p| p.dynamic_range_p95_p10_db),
            Some(5.0)
        );
    }

    #[test]
    fn apply_keeps_frontend_override_over_cache() {
        let mut s = settings_with(Some(profile(1.0)));
        apply_resolved_profile(&mut s, Some(profile(2.0)), false);
        assert_eq!(
            s.advanced
                .source_profile
                .map(|p| p.dynamic_range_p95_p10_db),
            Some(1.0)
        );
    }

    #[test]
    fn apply_clears_to_none_for_album_even_with_cache() {
        let mut s = settings_with(Some(profile(1.0)));
        apply_resolved_profile(&mut s, Some(profile(2.0)), true);
        assert_eq!(s.advanced.source_profile, None, "album must stay byte-flat");
    }

    // Concrete helper (no stubs): build a real DeepAnalysis from a synthesized sine.
    fn make_test_deep() -> crate::deep_analysis::DeepAnalysis {
        let sr = 48_000_u32;
        let n = sr as usize * 2;
        let omega = 2.0 * std::f32::consts::PI * 1000.0 / sr as f32;
        let samples: Vec<f32> = (0..n).map(|i| 0.3 * (omega * i as f32).sin()).collect();
        let windows = crate::deep_analysis::scan_windows(&samples, sr, 1);
        crate::deep_analysis::DeepAnalysis::from_parts([1.0 / 31.0; 31], windows)
    }

    #[test]
    fn deep_store_insert_get_and_prune() {
        let store = SourceProfileStore::default();
        let id = TrackId("t".into());
        assert!(store.get_deep(&id).is_none());
        store.insert_deep(id.clone(), Some(std::sync::Arc::new(make_test_deep())));
        assert!(store.get_deep(&id).is_some());
        // soft clear
        store.insert_deep(id.clone(), None);
        assert!(store.get_deep(&id).is_none());
        // prune clears deep too
        store.insert_deep(id.clone(), Some(std::sync::Arc::new(make_test_deep())));
        prune_failed_profiles(&store, std::slice::from_ref(&id), &[]);
        assert!(store.get_deep(&id).is_none());
    }

    #[test]
    fn confidence_gating_off_resolves_to_none() {
        // Owner-calibration gate: while confidence gating is off (the default), even a
        // present deep read resolves to None, so the chain stays byte-identical Tier-1.
        // Force the gate OFF under the shared lock (restoring afterwards) so this
        // asserts a concrete contract instead of silently no-opping if the default is
        // ever flipped during calibration.
        let _lock = crate::guardrails::ADAPTIVE_COMPRESSION_GATE_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let prev = crate::confidence::set_confidence_gating_enabled(false);
        let mut s = settings_with(None);
        apply_resolved_confidence(&mut s, Some(std::sync::Arc::new(make_test_deep())), false);
        crate::confidence::set_confidence_gating_enabled(prev);
        assert!(
            s.advanced.source_confidence.is_none(),
            "confidence gating must be inert when the gate is off"
        );
    }

    #[test]
    fn digest_is_compact_and_handles_sentinel_and_mono() {
        // B5 — receipt digest. presence+air = 0.22, sub+low = 0.38.
        let p = SourceProfile {
            spectral_6: SpectralBalance6 {
                sub: 0.2,
                low: 0.18,
                low_mid: 0.2,
                mid: 0.2,
                presence: 0.14,
                air: 0.08,
            },
            dynamic_range_p95_p10_db: 9.0,
            dynamic_range_lu: 7.0,
            stereo_correlation: Some(0.8),
            stereo_width: 1.0,
        };
        let d = p.digest();
        assert!(d.contains("bright 0.22"), "{d}");
        assert!(d.contains("low 0.38"), "{d}");
        assert!(d.contains("DR 9.0dB"), "{d}");
        assert!(d.contains("corr 0.80"), "{d}");

        // The 100 dB "no DR trigger" sentinel reads as n/a; mono reads as mono.
        let sentinel = SourceProfile {
            dynamic_range_p95_p10_db: 100.0,
            stereo_correlation: None,
            ..p
        };
        let d2 = sentinel.digest();
        assert!(d2.contains("DR n/a"), "{d2}");
        assert!(d2.contains("corr mono"), "{d2}");
    }
}
