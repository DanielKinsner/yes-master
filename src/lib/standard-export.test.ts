// src/lib/standard-export.test.ts
import { describe, expect, it } from "vitest";

import type { MasteringSettings, QualityCheck } from "../bindings";
import parity from "../standard-mapping-parity.json";
import { standardExportSettings, standardExportNotes } from "./standard-export";

interface StandardDeliveryFixture {
  sample_rate: number;
  bit_depth: number;
  ceiling_dbtp: number;
  lufs_clamp: number[];
}

function parityDelivery(): StandardDeliveryFixture {
  const delivery = (parity as { delivery?: StandardDeliveryFixture }).delivery;
  expect(delivery, "standard-mapping-parity.json must pin Standard delivery").toBeDefined();
  expect(delivery?.lufs_clamp, "Standard delivery lufs_clamp must be a two-value window").toHaveLength(2);
  return delivery!;
}

function settings(overrides: Partial<MasteringSettings["advanced"]> = {}): MasteringSettings {
  return {
    preset: { kind: "tape" },
    intensity: 0.6,
    eq_sub_db: 0, eq_low_db: 0, eq_low_mid_db: 0, eq_mid_db: 0,
    eq_high_mid_db: 0, eq_high_db: 0, eq_sparkle_db: 0,
    volume_match: false,
    source_lufs_integrated: null,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "streaming-universal",
    album: null,
    advanced: {
      lufs_offset_db: null, ceiling_dbtp: null, width: null, warmth: null,
      presence_air: null, compression_mode: "preset", compression_density: null,
      compression_low_threshold_db: null, compression_low_ratio: null,
      compression_low_attack_ms: null, compression_low_release_ms: null,
      compression_mid_threshold_db: null, compression_mid_ratio: null,
      compression_mid_attack_ms: null, compression_mid_release_ms: null,
      compression_high_threshold_db: null, compression_high_ratio: null,
      compression_high_attack_ms: null, compression_high_release_ms: null,
      compression_link_stereo: null, bit_depth: null, target_sample_rate: null,
      adaptive_strength: 0.5, ...overrides,
    },
  };
}

describe("standardExportSettings", () => {
  it("pins Custom / 44.1k / 24-bit / -1 dBTP and captures the effective loudness", () => {
    // Fresh streaming-universal track: effective target is -14 via the profile.
    const delivery = parityDelivery();
    const out = standardExportSettings(settings());
    expect(out.delivery_profile).toBe("custom");
    expect(out.advanced.lufs_offset_db).toBe(-14);
    expect(out.advanced.target_sample_rate).toBe(delivery.sample_rate);
    expect(out.advanced.bit_depth).toBe(delivery.bit_depth);
    expect(out.advanced.ceiling_dbtp).toBe(delivery.ceiling_dbtp);
  });

  it("pins the shared Standard loudness clamp from the parity fixture", () => {
    const [minLufs, maxLufs] = parityDelivery().lufs_clamp;
    const tooQuiet = settings({ lufs_offset_db: minLufs - 10 });
    tooQuiet.delivery_profile = "custom";
    expect(standardExportSettings(tooQuiet).advanced.lufs_offset_db).toBe(minLufs);

    const tooHot = settings({ lufs_offset_db: maxLufs + 10 });
    tooHot.delivery_profile = "custom";
    expect(standardExportSettings(tooHot).advanced.lufs_offset_db).toBe(maxLufs);
  });

  it("keeps an explicit custom loudness target", () => {
    const out = standardExportSettings(
      settings({ }) // start streaming-universal
    );
    expect(out.advanced.lufs_offset_db).toBe(-14);

    const custom = settings();
    custom.delivery_profile = "custom";
    custom.advanced.lufs_offset_db = -9;
    expect(standardExportSettings(custom).advanced.lufs_offset_db).toBe(-9);
  });

  it("does not mutate its input", () => {
    const input = settings();
    standardExportSettings(input);
    expect(input.delivery_profile).toBe("streaming-universal");
  });
});

describe("standardExportNotes", () => {
  const crit = (code: string): QualityCheck => ({ level: "critical", code, message: `${code} msg` });
  const warn = (code: string): QualityCheck => ({ level: "warning", code, message: `${code} msg` });
  const info = (code: string): QualityCheck => ({ level: "info", code, message: `${code} msg` });

  it("is clean when only cosmetic warnings + info are present", () => {
    const notes = standardExportNotes([info("export_ok"), warn("lufs_very_loud"), warn("dynamic_range_low")]);
    expect(notes.invalid).toBe(false);
    expect(notes.invalidMessage).toBeUndefined();
    expect(notes.integrityNote).toBeUndefined();
  });

  it("surfaces a tiny integrity note for true-peak, without flagging invalid", () => {
    const notes = standardExportNotes([warn("true_peak_high")]);
    expect(notes.invalid).toBe(false);
    expect(notes.integrityNote).toContain("true_peak_high msg");
  });

  it("flags the saved master invalid on any critical hard-stop", () => {
    const notes = standardExportNotes([warn("lufs_very_loud"), crit("non_finite_metering")]);
    expect(notes.invalid).toBe(true);
    expect(notes.invalidMessage).toContain("non_finite_metering msg");
  });
});
