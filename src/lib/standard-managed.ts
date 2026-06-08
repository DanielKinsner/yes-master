// src/lib/standard-managed.ts
//
// Standard "owns" {preset, intensity, loudness target, delivery format}.
// Everything else that affects the sound is a "non-managed edit": it can
// only exist because the user went into Advanced. Standard holds a hard
// invariant — its non-managed fields are always at their defaults — which
// these functions detect and enforce (see design spec §2b).

import type { MasteringSettings } from "../bindings";
import { ADAPTIVE_STRENGTH_DEFAULT } from "../bindings";

export function hasNonManagedEdits(s: MasteringSettings): boolean {
  if (
    s.eq_sub_db !== 0 ||
    s.eq_low_db !== 0 ||
    s.eq_low_mid_db !== 0 ||
    s.eq_mid_db !== 0 ||
    s.eq_high_mid_db !== 0 ||
    s.eq_high_db !== 0 ||
    s.eq_sparkle_db !== 0
  ) {
    return true;
  }
  if (s.input_gain_db !== 0 || s.output_gain_db !== 0) return true;

  const a = s.advanced;
  if (a.width !== null || a.warmth !== null || a.presence_air !== null) return true;
  if (a.compression_mode !== undefined && a.compression_mode !== "preset") return true;
  if (a.compression_density !== null) return true;
  if (
    a.compression_low_threshold_db !== null ||
    a.compression_low_ratio !== null ||
    a.compression_low_attack_ms !== null ||
    a.compression_low_release_ms !== null ||
    a.compression_mid_threshold_db !== null ||
    a.compression_mid_ratio !== null ||
    a.compression_mid_attack_ms !== null ||
    a.compression_mid_release_ms !== null ||
    a.compression_high_threshold_db !== null ||
    a.compression_high_ratio !== null ||
    a.compression_high_attack_ms !== null ||
    a.compression_high_release_ms !== null
  ) {
    return true;
  }
  if (a.compression_link_stereo !== null) return true;
  // Deliberate superset of the spec table — keep Standard's adaptive
  // behavior at the validated default. See plan Task 2 scope note.
  if ((a.adaptive_strength ?? ADAPTIVE_STRENGTH_DEFAULT) !== ADAPTIVE_STRENGTH_DEFAULT) {
    return true;
  }
  return false;
}

/// Returns a NEW settings object with every non-managed field reset to its
/// default, preserving preset / intensity / loudness target / delivery
/// format. Broader than `resetToneSettings` (which also resets intensity
/// and only touches EQ) — used by the Advanced->Standard return.
export function resetToStandardManaged(s: MasteringSettings): MasteringSettings {
  return {
    ...s,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    input_gain_db: 0,
    output_gain_db: 0,
    advanced: {
      ...s.advanced,
      width: null,
      warmth: null,
      presence_air: null,
      compression_mode: "preset",
      compression_density: null,
      compression_low_threshold_db: null,
      compression_low_ratio: null,
      compression_low_attack_ms: null,
      compression_low_release_ms: null,
      compression_mid_threshold_db: null,
      compression_mid_ratio: null,
      compression_mid_attack_ms: null,
      compression_mid_release_ms: null,
      compression_high_threshold_db: null,
      compression_high_ratio: null,
      compression_high_attack_ms: null,
      compression_high_release_ms: null,
      compression_link_stereo: null,
      adaptive_strength: ADAPTIVE_STRENGTH_DEFAULT,
    },
  };
}

/// The always-clean-invariant guard (design spec §2a/§2b). Standard must
/// never silently render a track that carries hidden Advanced edits — when
/// it would, the caller shows that track in Advanced instead. Also forces
/// Advanced in Album mode (Album Master is Advanced-only in v1).
export function shouldForceAdvancedOnStandardEntry(args: {
  isAlbum: boolean;
  hasTrack: boolean;
  settings: MasteringSettings;
}): boolean {
  if (args.isAlbum) return true;
  if (args.hasTrack && hasNonManagedEdits(args.settings)) return true;
  return false;
}
