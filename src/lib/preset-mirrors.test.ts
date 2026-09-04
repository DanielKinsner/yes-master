// Display-mirror tripwires (TS half). `src/preset-mirrors.json` is generated
// from the dsp.rs PRESET_* calibration by src-tauri/tests/preset_mirrors.rs;
// these tests assert the compressor display table matches it:
//
//   the compressor table in compressor-auto.ts (via the public readouts
//      at density 0.5, where engagement = 1 and overdrive = 0, so the raw
//      table values pass straight through);
// SignalChain now receives resolved processing state directly from Rust.

import { describe, expect, it } from "vitest";

import mirrors from "../preset-mirrors.json";
import type { AdvancedSettings, MasteringSettings, Preset } from "../bindings";
import { compressorAutoReadouts } from "./compressor-auto";


const KINDS: Array<Preset["kind"]> = [
  "universal",
  "clarity",
  "tape",
  "spatial",
  "oomph",
  "warmth",
  "punch",
  "loud",
  "custom",
];

function presetOf(kind: Preset["kind"]): Preset {
  return kind === "custom" ? { kind, id: "mirror" } : { kind };
}

const NEUTRAL_ADVANCED: AdvancedSettings = {
  lufs_offset_db: null,
  ceiling_dbtp: null,
  width: null,
  warmth: null,
  presence_air: null,
  compression_mode: "preset",
  compression_density: 0.5,
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
  adaptive_strength: null,
  source_profile: null,
};

function settingsFor(kind: Preset["kind"]): MasteringSettings {
  return {
    preset: presetOf(kind),
    intensity: 0.5,
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
    advanced: { ...NEUTRAL_ADVANCED },
  };
}

describe("compressor table mirrors dsp.rs (preset-mirrors.json)", () => {
  for (const kind of KINDS) {
    it(`${kind} matches the engine calibration`, () => {
      // density 0.5 → engagement 1, overdrive 0 → table values pass through.
      const readout = compressorAutoReadouts(settingsFor(kind)).low;
      const engine = mirrors.compressor[kind];
      expect(readout.thresholdDb).toBeCloseTo(engine.threshold_db, 6);
      expect(readout.ratio).toBeCloseTo(engine.ratio, 6);
      expect(readout.attackMs).toBeCloseTo(engine.attack_ms, 6);
      expect(readout.releaseMs).toBeCloseTo(engine.release_ms, 6);
    });
  }
});
