// src/lib/standard-mapping.test.ts
import { describe, expect, it } from "vitest";

import type { Preset } from "../bindings";
import parity from "../standard-mapping-parity.json";
import {
  STANDARD_STYLES,
  STANDARD_LOUDNESS,
  styleToPreset,
  presetToStyle,
  loudnessToTarget,
  targetToLoudness,
} from "./standard-mapping";

describe("standard style mapping (the reference-4)", () => {
  it("maps the four styles to their internal presets", () => {
    expect(styleToPreset("balanced")).toEqual<Preset>({ kind: "universal" });
    expect(styleToPreset("bright")).toEqual<Preset>({ kind: "clarity" });
    expect(styleToPreset("warm")).toEqual<Preset>({ kind: "tape" });
    expect(styleToPreset("heavy")).toEqual<Preset>({ kind: "oomph" });
  });

  it("round-trips preset back to style for the reference-4", () => {
    expect(presetToStyle({ kind: "universal" })).toBe("balanced");
    expect(presetToStyle({ kind: "clarity" })).toBe("bright");
    expect(presetToStyle({ kind: "tape" })).toBe("warm");
    expect(presetToStyle({ kind: "oomph" })).toBe("heavy");
  });

  it("returns null for presets outside the Standard set", () => {
    expect(presetToStyle({ kind: "spatial" })).toBeNull();
    expect(presetToStyle({ kind: "punch" })).toBeNull();
    expect(presetToStyle({ kind: "custom", id: "x" })).toBeNull();
  });

  it("exposes exactly four ordered tiles with metadata", () => {
    expect(STANDARD_STYLES.map((s) => s.id)).toEqual([
      "balanced",
      "bright",
      "warm",
      "heavy",
    ]);
    expect(STANDARD_STYLES[0]).toMatchObject({ label: "Balanced", subtitle: "Clean balance", tone: "blue" });
    expect(STANDARD_STYLES[1]).toMatchObject({ label: "Bright",   subtitle: "Air & detail",  tone: "cyan" });
    expect(STANDARD_STYLES[2]).toMatchObject({ label: "Warm",     subtitle: "Glue & body",   tone: "gold" });
    // "red" matches Oomph's PRESET_ACCENT + artwork across both views.
    expect(STANDARD_STYLES[3]).toMatchObject({ label: "Heavy",    subtitle: "Sub & weight",  tone: "red" });
  });
});

describe("standard loudness mapping", () => {
  it("maps the three loudness steps to LUFS targets", () => {
    expect(loudnessToTarget("low")).toBe(-14);
    expect(loudnessToTarget("medium")).toBe(-11);
    expect(loudnessToTarget("high")).toBe(-9);
  });

  it("matches an effective LUFS back to its step within tolerance", () => {
    expect(targetToLoudness(-14)).toBe("low");
    expect(targetToLoudness(-11)).toBe("medium");
    expect(targetToLoudness(-9)).toBe("high");
    expect(targetToLoudness(-13.999)).toBe("low");
  });

  it("returns null when the target is off-grid or absent", () => {
    expect(targetToLoudness(null)).toBeNull();
    expect(targetToLoudness(-7)).toBeNull();
  });

  it("exposes exactly three ordered loudness steps", () => {
    expect(STANDARD_LOUDNESS.map((l) => l.id)).toEqual([
      "low",
      "medium",
      "high",
    ]);
  });
});

// Cross-language pin: the same fixture is asserted by the iPhone bridge test
// (`standard_style_aliases_match_the_shared_parity_fixture` in
// apps/iphone-native/rust/src/lib.rs), so the desktop tables and
// native_preset() cannot drift apart without one side failing.
describe("standard-mapping-parity.json cross-pin", () => {
  it("styles match the shared fixture", () => {
    const fromTables = Object.fromEntries(
      STANDARD_STYLES.map((s) => [s.id, s.preset.kind]),
    );
    expect(fromTables).toEqual(parity.styles);
  });

  it("loudness steps match the shared fixture (Swift NativeLoudness mirror)", () => {
    const fromTables = Object.fromEntries(
      STANDARD_LOUDNESS.map((l) => [l.id, l.lufs]),
    );
    expect(fromTables).toEqual(parity.loudness);
  });
});
