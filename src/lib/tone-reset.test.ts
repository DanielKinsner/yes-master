import { describe, expect, it } from "vitest";

import type { MasteringSettings } from "../bindings";
import { isToneFlat, resetToneSettings, TONE_DEFAULT_INTENSITY } from "./tone-reset";

// The "fast reset" for the Visual EQ + Intensity area. Pure logic so the
// reset's exact scope (intensity + the seven EQ bands, and ONLY those) is
// nailed down by tests rather than re-derived from React click-throughs.

function settings(overrides: Partial<MasteringSettings> = {}): MasteringSettings {
  return {
    preset: { kind: "warm" },
    intensity: 0.82,
    eq_sub_db: -3,
    eq_low_db: 2.5,
    eq_low_mid_db: -1,
    eq_mid_db: 4,
    eq_high_mid_db: -2.5,
    eq_high_db: 1.5,
    eq_sparkle_db: 3,
    volume_match: true,
    input_gain_db: -1.5,
    output_gain_db: 0.5,
    delivery_profile: "club",
    advanced: {
      lufs_offset_db: -1,
      ceiling_dbtp: -0.8,
      width: 1.2,
      warmth: 0.4,
      presence_air: 0.3,
      compression_mode: "manual",
      compression_density: 0.7,
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
      target_sample_rate: 48000,
      adaptive_strength: 0.7,
    },
    ...overrides,
  };
}

describe("resetToneSettings", () => {
  it("flattens intensity to the 50% default and every EQ band to 0 dB", () => {
    const out = resetToneSettings(settings());
    expect(out.intensity).toBe(TONE_DEFAULT_INTENSITY);
    expect(out.eq_sub_db).toBe(0);
    expect(out.eq_low_db).toBe(0);
    expect(out.eq_low_mid_db).toBe(0);
    expect(out.eq_mid_db).toBe(0);
    expect(out.eq_high_mid_db).toBe(0);
    expect(out.eq_high_db).toBe(0);
    expect(out.eq_sparkle_db).toBe(0);
  });

  it("leaves preset, gains, delivery, volume-match, and advanced untouched", () => {
    const input = settings();
    const out = resetToneSettings(input);
    expect(out.preset).toEqual(input.preset);
    expect(out.input_gain_db).toBe(input.input_gain_db);
    expect(out.output_gain_db).toBe(input.output_gain_db);
    expect(out.delivery_profile).toBe(input.delivery_profile);
    expect(out.volume_match).toBe(input.volume_match);
    expect(out.advanced).toEqual(input.advanced);
  });

  it("does not mutate the input settings object", () => {
    const input = settings();
    const snapshot = JSON.parse(JSON.stringify(input));
    resetToneSettings(input);
    expect(input).toEqual(snapshot);
  });
});

describe("isToneFlat", () => {
  it("is true when intensity is the default and all EQ bands are 0", () => {
    expect(isToneFlat(resetToneSettings(settings()))).toBe(true);
  });

  it("is false when any EQ band is non-zero", () => {
    expect(isToneFlat(settings({ eq_mid_db: 0.1 } as Partial<MasteringSettings>))).toBe(
      false,
    );
  });

  it("is false when intensity differs from the default", () => {
    const flat = resetToneSettings(settings());
    expect(isToneFlat({ ...flat, intensity: 0.51 })).toBe(false);
  });
});
