import { describe, expect, it } from "vitest";
import {
  IPHONE_SIMPLE_FEATURES,
  buildIphoneSimplePlan,
  iphoneSimpleExportProfileOptions,
  iphoneSimpleLoudnessOptions,
  iphoneSimpleToneOptions,
} from "./simple-mode";

describe("iPhone Simple Mode contract", () => {
  it("exposes the Simple Mode feature list without adaptive or smart analysis", () => {
    expect(IPHONE_SIMPLE_FEATURES).toEqual([
      "single-track-import",
      "tone-presets",
      "export-profile",
      "original-mastered-toggle",
      "volume-match-toggle",
      "lufs-preview-toggle",
      "loudness-choice",
      "export-action",
    ]);
    expect(IPHONE_SIMPLE_FEATURES).not.toContain("adaptive-analysis");
    expect(IPHONE_SIMPLE_FEATURES).not.toContain("smart-analysis");
    expect(IPHONE_SIMPLE_FEATURES).not.toContain("advanced-mode");
  });

  it("maps four iPhone tone choices to the existing desktop DSP presets", () => {
    expect(iphoneSimpleToneOptions.map((option) => option.id)).toEqual([
      "balanced",
      "warm",
      "open",
      "punch",
    ]);

    expect(buildIphoneSimplePlan({ tone: "balanced" }).exportSettings.preset).toEqual({
      kind: "universal",
    });
    expect(buildIphoneSimplePlan({ tone: "warm" }).exportSettings.preset).toEqual({
      kind: "warmth",
    });
    expect(buildIphoneSimplePlan({ tone: "open" }).exportSettings.preset).toEqual({
      kind: "clarity",
    });
    expect(buildIphoneSimplePlan({ tone: "punch" }).exportSettings.preset).toEqual({
      kind: "punch",
    });
  });

  it("keeps loudness separate from the export destination profile", () => {
    expect(iphoneSimpleLoudnessOptions.map((option) => option.targetLufs)).toEqual([
      -16,
      -14,
      -10.5,
    ]);
    expect(iphoneSimpleExportProfileOptions.map((option) => option.id)).toEqual([
      "streaming",
      "cd",
      "custom",
    ]);

    const plan = buildIphoneSimplePlan({
      tone: "open",
      loudness: "high",
      exportProfile: "cd",
    });

    expect(plan.exportSettings.delivery_profile).toBe("custom");
    expect(plan.exportSettings.advanced.lufs_offset_db).toBe(-10.5);
    expect(plan.exportSettings.advanced.bit_depth).toBe(16);
    expect(plan.exportSettings.advanced.target_sample_rate).toBe(44_100);
    expect(plan.exportSettings.advanced.ceiling_dbtp).toBe(-1);
  });

  it("lets Volume Match affect audition while keeping export level clean", () => {
    const plan = buildIphoneSimplePlan({
      volumeMatch: true,
      lufsPreview: true,
    });

    expect(plan.auditionSettings.volume_match).toBe(true);
    expect(plan.exportSettings.volume_match).toBe(false);
    expect(plan.previewLufsLanding).toBe(true);
  });

  it("supports a custom iPhone export destination without enabling smart analysis", () => {
    const plan = buildIphoneSimplePlan({
      exportProfile: "custom",
      customExport: {
        bitDepth: 24,
        ceilingDbtp: -2,
        sampleRate: 96_000,
      },
    });

    expect(plan.exportSettings.advanced.bit_depth).toBe(24);
    expect(plan.exportSettings.advanced.ceiling_dbtp).toBe(-2);
    expect(plan.exportSettings.advanced.target_sample_rate).toBe(96_000);
    expect(plan.usesAdaptiveAnalysis).toBe(false);
  });
});
