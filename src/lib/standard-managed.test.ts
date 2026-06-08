// src/lib/standard-managed.test.ts
import { describe, expect, it } from "vitest";

import type { MasteringSettings } from "../bindings";
import { hasNonManagedEdits, resetToStandardManaged, shouldForceAdvancedOnStandardEntry } from "./standard-managed";

function base(overrides: Partial<MasteringSettings> = {}): MasteringSettings {
  return {
    preset: { kind: "tape" },
    intensity: 0.72,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    volume_match: false,
    source_lufs_integrated: null,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "custom",
    album: null,
    advanced: {
      lufs_offset_db: -11,
      ceiling_dbtp: -1,
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
      bit_depth: 24,
      target_sample_rate: 44_100,
      adaptive_strength: 0.5,
    },
    ...overrides,
  };
}

describe("hasNonManagedEdits", () => {
  it("is false for a clean Standard-shaped settings object", () => {
    expect(hasNonManagedEdits(base())).toBe(false);
  });

  it("ignores managed fields (preset, intensity, loudness, delivery format)", () => {
    expect(
      hasNonManagedEdits(
        base({
          preset: { kind: "oomph" },
          intensity: 0.95,
          advanced: { ...base().advanced, lufs_offset_db: -9, bit_depth: 16, target_sample_rate: 48_000, ceiling_dbtp: -2 },
        }),
      ),
    ).toBe(false);
  });

  it("flags any non-zero EQ band", () => {
    expect(hasNonManagedEdits(base({ eq_low_db: 1.5 }))).toBe(true);
    expect(hasNonManagedEdits(base({ eq_sparkle_db: -0.5 }))).toBe(true);
  });

  it("flags width / warmth / presence_air", () => {
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, width: 0.2 } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, warmth: 0.1 } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, presence_air: 0.3 } }))).toBe(true);
  });

  it("flags compressor mode != preset and any density / per-band override", () => {
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_mode: "manual" } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_mode: "off" } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_density: 0.4 } }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, compression_low_ratio: 3 } }))).toBe(true);
  });

  it("flags input/output gain and a non-default adaptive_strength", () => {
    expect(hasNonManagedEdits(base({ input_gain_db: -1 }))).toBe(true);
    expect(hasNonManagedEdits(base({ output_gain_db: 0.5 }))).toBe(true);
    expect(hasNonManagedEdits(base({ advanced: { ...base().advanced, adaptive_strength: 0 } }))).toBe(true);
  });
});

describe("resetToStandardManaged", () => {
  it("clears all non-managed fields but preserves preset/intensity/loudness/format", () => {
    const dirty = base({
      preset: { kind: "oomph" },
      intensity: 0.83,
      eq_low_db: 3,
      eq_high_db: -2,
      input_gain_db: -1.5,
      output_gain_db: 0.7,
      advanced: {
        ...base().advanced,
        lufs_offset_db: -9,
        bit_depth: 24,
        target_sample_rate: 44_100,
        ceiling_dbtp: -1,
        width: 0.3,
        warmth: 0.2,
        presence_air: 0.1,
        compression_mode: "manual",
        compression_density: 0.6,
        compression_low_ratio: 4,
        compression_link_stereo: false,
        adaptive_strength: 0,
      },
    });
    const clean = resetToStandardManaged(dirty);

    // Managed values preserved:
    expect(clean.preset).toEqual({ kind: "oomph" });
    expect(clean.intensity).toBe(0.83);
    expect(clean.delivery_profile).toBe("custom");
    expect(clean.advanced.lufs_offset_db).toBe(-9);
    expect(clean.advanced.bit_depth).toBe(24);
    expect(clean.advanced.target_sample_rate).toBe(44_100);
    expect(clean.advanced.ceiling_dbtp).toBe(-1);

    // Non-managed cleared:
    expect(clean.eq_low_db).toBe(0);
    expect(clean.eq_high_db).toBe(0);
    expect(clean.input_gain_db).toBe(0);
    expect(clean.output_gain_db).toBe(0);
    expect(clean.advanced.width).toBeNull();
    expect(clean.advanced.warmth).toBeNull();
    expect(clean.advanced.presence_air).toBeNull();
    expect(clean.advanced.compression_mode).toBe("preset");
    expect(clean.advanced.compression_density).toBeNull();
    expect(clean.advanced.compression_low_ratio).toBeNull();
    expect(clean.advanced.compression_link_stereo).toBeNull();
    expect(clean.advanced.adaptive_strength).toBe(0.5);

    // And the result is, by definition, clean:
    expect(hasNonManagedEdits(clean)).toBe(false);
  });

  it("does not mutate its input", () => {
    const input = base({ eq_low_db: 5 });
    resetToStandardManaged(input);
    expect(input.eq_low_db).toBe(5);
  });
});

describe("shouldForceAdvancedOnStandardEntry (always-clean invariant guard)", () => {
  it("is false for a clean track", () => {
    expect(shouldForceAdvancedOnStandardEntry({ isAlbum: false, hasTrack: true, settings: base() })).toBe(false);
  });

  it("is true when the selected track carries hidden non-managed edits", () => {
    expect(shouldForceAdvancedOnStandardEntry({ isAlbum: false, hasTrack: true, settings: base({ eq_low_db: 2 }) })).toBe(true);
  });

  it("is true in album mode (Album Master is Advanced-only in v1)", () => {
    expect(shouldForceAdvancedOnStandardEntry({ isAlbum: true, hasTrack: true, settings: base() })).toBe(true);
  });

  it("is false when there is no selected track (nothing to leak)", () => {
    expect(shouldForceAdvancedOnStandardEntry({ isAlbum: false, hasTrack: false, settings: base({ eq_low_db: 9 }) })).toBe(false);
  });
});
