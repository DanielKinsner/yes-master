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

import type { MasteringSettings } from "../bindings";
import { DELIVERY_PROFILE_TARGET_LUFS } from "../bindings";

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
