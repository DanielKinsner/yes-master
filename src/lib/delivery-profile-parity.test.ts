// DeliveryProfile parity tripwire (TS half). The Rust test
// src-tauri/tests/delivery_profile_parity.rs generates the fixture from
// DeliveryProfile methods; this asserts bindings.ts still mirrors it.

import { describe, expect, it } from "vitest";

import {
  DELIVERY_PROFILE_BIT_DEPTH,
  DELIVERY_PROFILE_CEILING_DBTP,
  DELIVERY_PROFILE_DISPLAY,
  DELIVERY_PROFILE_SAMPLE_RATE,
  DELIVERY_PROFILE_SHORT,
  DELIVERY_PROFILE_TARGET_LUFS,
  type DeliveryProfile,
} from "../bindings";
import fixture from "../delivery-profile-parity.json";

type DeliveryProfileRow = {
  target_lufs: number | null;
  ceiling_dbtp: number | null;
  bit_depth: number | null;
  sample_rate: number | null;
};

const profiles = (fixture as {
  profiles: Record<DeliveryProfile, DeliveryProfileRow>;
}).profiles;

function valuesFor(key: keyof DeliveryProfileRow): Record<DeliveryProfile, number | null> {
  return Object.fromEntries(
    Object.entries(profiles).map(([profile, row]) => [profile, row[key]]),
  ) as Record<DeliveryProfile, number | null>;
}

describe("DeliveryProfile bindings mirror Rust", () => {
  it("target LUFS values match DeliveryProfile::target_lufs", () => {
    expect(DELIVERY_PROFILE_TARGET_LUFS).toEqual(valuesFor("target_lufs"));
  });

  it("every profile has a display name and a short name (receipt chip, D11b)", () => {
    // Same key set as the Rust-generated fixture, so a profile added on the
    // Rust side without a name fails here rather than rendering
    // "undefined · -14.0 LUFS" on the receipt.
    const expected = Object.keys(profiles).sort();
    expect(Object.keys(DELIVERY_PROFILE_DISPLAY).sort()).toEqual(expected);
    expect(Object.keys(DELIVERY_PROFILE_SHORT).sort()).toEqual(expected);
    for (const profile of expected as DeliveryProfile[]) {
      const short = DELIVERY_PROFILE_SHORT[profile];
      expect(short.trim().length, profile).toBeGreaterThan(0);
      expect(
        short.length,
        `${profile}: the short name must not be longer than the display name`,
      ).toBeLessThanOrEqual(DELIVERY_PROFILE_DISPLAY[profile].length);
    }
  });

  it("ceiling values match DeliveryProfile::ceiling_dbtp", () => {
    expect(DELIVERY_PROFILE_CEILING_DBTP).toEqual(valuesFor("ceiling_dbtp"));
  });

  it("bit-depth values match DeliveryProfile::output_bit_depth", () => {
    expect(DELIVERY_PROFILE_BIT_DEPTH).toEqual(valuesFor("bit_depth"));
  });

  it("sample-rate values match DeliveryProfile::output_sample_rate", () => {
    expect(DELIVERY_PROFILE_SAMPLE_RATE).toEqual(valuesFor("sample_rate"));
  });
});
