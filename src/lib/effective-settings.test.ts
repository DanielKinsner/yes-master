import { describe, expect, it } from "vitest";

import type {
  AdvancedSettings,
  DeliveryProfile,
  MasteringSettings,
} from "../bindings";
import { effectiveLoudnessTarget } from "./effective-settings";

// Frontend mirror tests for the `effective_*` accessors. The Rust
// source-of-truth tests live in `src-tauri/src/types.rs` under
// `effective_settings_tests`; these tests verify the frontend helper
// honors the same shadowing rule so the LoudnessTarget readout never
// lies about what the chain will target.
//
// First Vitest file in the repo (scaffold for future frontend tests).
// `effectiveLoudnessTarget` is a small pure function but a useful
// canary: it depends on `DELIVERY_PROFILE_TARGET_LUFS` from
// bindings.ts, so this test also catches any divergence between the
// generated bindings and the Rust enum.

const DEFAULT_ADVANCED: AdvancedSettings = {
  lufs_offset_db: null,
  ceiling_dbtp: null,
  width: null,
  warmth: null,
  presence_air: null,
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
  bit_depth: null,
  target_sample_rate: null,
};

function makeSettings(
  profile: DeliveryProfile,
  advanced: Partial<AdvancedSettings> = {},
): MasteringSettings {
  return {
    preset: { kind: "universal" },
    intensity: 0.5,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_db: 0,
    volume_match: false,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: profile,
    advanced: { ...DEFAULT_ADVANCED, ...advanced },
  };
}

describe("effectiveLoudnessTarget", () => {
  it("returns the profile's target when delivery_profile is non-Custom", () => {
    // Mirrors Rust:
    // effective_target_lufs_profile_overrides_advanced
    const settings = makeSettings("streaming-universal", {
      lufs_offset_db: -9, // would-be user override
    });
    expect(effectiveLoudnessTarget(settings)).toBe(-14);
  });

  it("falls through to advanced.lufs_offset_db when delivery_profile is Custom", () => {
    // Mirrors Rust:
    // effective_target_lufs_custom_uses_advanced_value
    const settings = makeSettings("custom", { lufs_offset_db: -9 });
    expect(effectiveLoudnessTarget(settings)).toBe(-9);
  });

  it("returns null when delivery_profile is Custom and advanced.lufs_offset_db is null", () => {
    // Mirrors Rust:
    // effective_target_lufs_custom_with_none_advanced_returns_none
    const settings = makeSettings("custom");
    expect(effectiveLoudnessTarget(settings)).toBeNull();
  });

  it("reports the right target for every non-Custom profile", () => {
    // Mirrors Rust:
    // effective_target_lufs_known_for_every_non_custom_profile
    const cases: Array<[DeliveryProfile, number]> = [
      ["streaming-universal", -14],
      ["apple-music", -16],
      ["cd", -14],
      ["vinyl-premaster", -18],
      ["loud-rock", -10.5],
      ["broadcast-eu", -23],
      ["broadcast-us", -24],
    ];
    for (const [profile, expected] of cases) {
      const settings = makeSettings(profile);
      expect(
        effectiveLoudnessTarget(settings),
        `profile ${profile} must report ${expected} LUFS`,
      ).toBe(expected);
    }
  });

  it("ignores volume_match (orthogonal to the landing target)", () => {
    // VM toggle should not change the effective target — same reason
    // the Rust accessor doesn't consult volume_match, and the same
    // reason the live-preview-cache hash strips it.
    const off = makeSettings("custom", { lufs_offset_db: -12 });
    const on: MasteringSettings = { ...off, volume_match: true };
    expect(effectiveLoudnessTarget(off)).toBe(-12);
    expect(effectiveLoudnessTarget(on)).toBe(-12);
  });
});
