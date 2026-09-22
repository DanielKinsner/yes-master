// Frontend mirrors of the `MasteringSettings::effective_*` accessors in
// `src-tauri/src/types.rs`. The Rust accessors are the source of truth
// (tested in `types.rs::effective_settings_tests`); these helpers exist
// so the UI displays don't lie about what the chain is targeting when
// a non-Custom DeliveryProfile is shadowing advanced fields.
//
// Why a separate module: the LoudnessTarget readout (App.tsx) was
// reading raw `advanced.lufs_offset_db` for both its display number
// and the dropdown's selected value. When a non-Custom profile was
// active, the chain WAS targeting the profile's value but the readout
// showed "—" — same trust-failure pattern as VM-in-export (B3) and
// B7's write-direction auto-flip-to-Custom, this time in the read
// direction. Extracting the rule lets it live alongside its tests.

import type { DeliveryProfile, MasteringSettings } from "../bindings";
import {
  DELIVERY_PROFILE_BIT_DEPTH,
  DELIVERY_PROFILE_CEILING_DBTP,
  DELIVERY_PROFILE_SAMPLE_RATE,
  DELIVERY_PROFILE_SHORT,
  DELIVERY_PROFILE_TARGET_LUFS,
} from "../bindings";

export function formatLoudnessTarget(lufs: number | null): string {
  return lufs === null ? "Off / Natural" : `${lufs.toFixed(1)} LUFS`;
}

function deliveryTargetOption(profile: DeliveryProfile, id = profile as string) {
  const lufs = DELIVERY_PROFILE_TARGET_LUFS[profile];
  return { id, lufs, label: `${DELIVERY_PROFILE_SHORT[profile]} · ${formatLoudnessTarget(lufs)}` };
}

/// The same targets and names as Delivery Profile, plus the established loud,
/// hot and natural choices. IDs of existing quick choices remain stable.
export const LOUDNESS_PROFILES: ReadonlyArray<{
  id: string;
  label: string;
  lufs: number | null;
}> = [
  deliveryTargetOption("streaming-universal", "streaming"),
  deliveryTargetOption("apple-music"),
  deliveryTargetOption("vinyl-premaster"),
  { id: "loud-streaming", label: `Loud · ${formatLoudnessTarget(-11)}`, lufs: -11 },
  deliveryTargetOption("loud-rock"),
  { id: "cd-master", label: `Hot master · ${formatLoudnessTarget(-9)}`, lufs: -9 },
  deliveryTargetOption("broadcast-eu"),
  deliveryTargetOption("broadcast-us"),
  { id: "off", label: "Off / Natural", lufs: null },
];

/// Effective target LUFS that the chain will actually apply.
///
/// Mirror of `MasteringSettings::effective_target_lufs`. When
/// `delivery_profile !== "custom"`, the profile's target wins —
/// `advanced.lufs_offset_db` is shadowed. When `delivery_profile ===
/// "custom"`, the helper falls through to `advanced.lufs_offset_db`.
/// `null` means "no target" — the chain skips its landing block.
export function effectiveLoudnessTarget(
  settings: MasteringSettings,
): number | null {
  const profileTarget = DELIVERY_PROFILE_TARGET_LUFS[settings.delivery_profile];
  if (profileTarget !== null && profileTarget !== undefined) {
    return profileTarget;
  }
  return settings.advanced.lufs_offset_db ?? null;
}

export function effectiveCeilingDbtp(settings: MasteringSettings): number | null {
  const profileCeiling = DELIVERY_PROFILE_CEILING_DBTP[settings.delivery_profile];
  if (profileCeiling !== null && profileCeiling !== undefined) {
    return profileCeiling;
  }
  return settings.advanced.ceiling_dbtp ?? null;
}

export function effectiveBitDepth(settings: MasteringSettings): number | null {
  const profileBitDepth = DELIVERY_PROFILE_BIT_DEPTH[settings.delivery_profile];
  if (profileBitDepth !== null && profileBitDepth !== undefined) {
    return profileBitDepth;
  }
  return settings.advanced.bit_depth ?? null;
}

export function effectiveSampleRate(settings: MasteringSettings): number | null {
  const profileSampleRate = DELIVERY_PROFILE_SAMPLE_RATE[settings.delivery_profile];
  if (profileSampleRate !== null && profileSampleRate !== undefined) {
    return profileSampleRate;
  }
  return settings.advanced.target_sample_rate ?? null;
}

/// Transient live settings. Keep export format out of preparation identity,
/// while preserving audible profile values. Rust resolves the actual source
/// rate after decoding; null here means source, and 32 means unquantized float.
export function livePreviewSettings(settings: MasteringSettings): MasteringSettings {
  return {
    ...settings,
    delivery_profile: "custom",
    advanced: {
      ...settings.advanced,
      lufs_offset_db: effectiveLoudnessTarget(settings),
      ceiling_dbtp: effectiveCeilingDbtp(settings) ?? -1,
      target_sample_rate: null,
      bit_depth: 32,
    },
  };
}

/// Match a LUFS value to a quick-select dropdown option id. Returns
/// the canonical id
/// when the value matches a `LOUDNESS_PROFILES` entry within ±1e-3 LU,
/// otherwise returns "custom" to indicate the value falls outside the
/// quick-select set. Null maps to "off / natural."
export function profileIdForLufs(lufs: number | null): string {
  if (lufs === null) return "off";
  for (const p of LOUDNESS_PROFILES) {
    if (p.lufs !== null && Math.abs(p.lufs - lufs) < 1e-3) return p.id;
  }
  return "custom";
}

/// Aggregate display state for the LoudnessTarget UI block. Computes:
///
///   * `current` — the effective target the chain will use, mirror
///     of `effectiveLoudnessTarget`.
///   * `profileId` — the dropdown's selected value, derived from
///     `current` via `profileIdForLufs`.
///   * `displayText` — the formatted readout above the dropdown
///     ("-14.0", "—" for no target).
///
/// Single pure entry-point so the LoudnessTarget component can pass
/// `settings` in and get everything it needs out, without inline
/// formatting / matching logic.
export interface LoudnessTargetDisplay {
  current: number | null;
  profileId: string;
  displayText: string;
}

export function loudnessTargetDisplay(
  settings: MasteringSettings,
): LoudnessTargetDisplay {
  const current = effectiveLoudnessTarget(settings);
  return {
    current,
    profileId: profileIdForLufs(current),
    displayText: current !== null ? current.toFixed(1) : "—",
  };
}
