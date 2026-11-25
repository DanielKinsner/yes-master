import { describe, expect, it } from "vitest";

import type { AdvancedSettings, MasteringSettings, SmartMatchSettings } from "../bindings";
import { presetSettingsKeepingSmartMatch } from "./smart-match";

const ADVANCED: AdvancedSettings = {
  lufs_offset_db: null,
  ceiling_dbtp: null,
  width: null,
  warmth: 0.25,
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

const PRESET: MasteringSettings = {
  preset: { kind: "clarity" },
  intensity: 0.7,
  eq_sub_db: 0,
  eq_low_db: 1,
  eq_low_mid_db: 0,
  eq_mid_db: 0,
  eq_high_mid_db: 0,
  eq_high_db: 0,
  eq_sparkle_db: 0,
  volume_match: false,
  source_lufs_integrated: null,
  input_gain_db: 0,
  output_gain_db: 0,
  delivery_profile: "streaming-universal",
  album: null,
  advanced: ADVANCED,
};

const SONG_MATCH: SmartMatchSettings = {
  reference: {
    name: "LANDR-It's a coat-original-test-Warm-High",
    spectral_6: { sub: 0.2, low: 0.25, low_mid: 0.2, mid: 0.15, presence: 0.12, air: 0.08 },
  },
  enabled: true,
  amount: 0.5,
};

describe("applying a saved preset keeps the song's Smart Match", () => {
  it("keeps the current reference and takes everything else from the preset", () => {
    const applied = presetSettingsKeepingSmartMatch(PRESET, SONG_MATCH);
    expect(applied.advanced.smart_match).toEqual(SONG_MATCH);
    expect(applied.preset).toEqual({ kind: "clarity" });
    expect(applied.intensity).toBe(0.7);
    expect(applied.advanced.warmth).toBe(0.25);
  });

  it("never imposes a reference a preset somehow carries", () => {
    const smuggled: MasteringSettings = {
      ...PRESET,
      advanced: { ...ADVANCED, smart_match: { ...SONG_MATCH, amount: 1 } },
    };
    expect(presetSettingsKeepingSmartMatch(smuggled, null).advanced).not.toHaveProperty(
      "smart_match",
    );
    expect(presetSettingsKeepingSmartMatch(smuggled, SONG_MATCH).advanced.smart_match).toEqual(
      SONG_MATCH,
    );
  });

  it("does not mutate the preset it was given", () => {
    const before = JSON.stringify(PRESET);
    presetSettingsKeepingSmartMatch(PRESET, SONG_MATCH);
    expect(JSON.stringify(PRESET)).toBe(before);
  });
});
