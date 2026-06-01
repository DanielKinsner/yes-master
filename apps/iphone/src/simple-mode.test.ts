import { describe, expect, it } from "vitest";
import {
  IPHONE_SIMPLE_FEATURES,
  buildIphoneSimplePlan,
  iphoneSimpleLoudnessOptions,
  iphoneSimpleToneOptions,
} from "./simple-mode";

describe("iPhone Simple Mode contract", () => {
  it("exposes the Simple Mode feature list without adaptive or smart analysis", () => {
    expect(IPHONE_SIMPLE_FEATURES).toEqual([
      "single-track-import",
      "tone-presets",
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

  it("uses one fixed iPhone WAV destination with a default -11 LUFS medium target", () => {
    expect(iphoneSimpleLoudnessOptions.map((option) => option.targetLufs)).toEqual([
      -14,
      -11,
      -9,
    ]);

    const plan = buildIphoneSimplePlan({
      tone: "open",
    });

    expect(plan.exportSettings.delivery_profile).toBe("custom");
    expect(plan.exportSettings.advanced.lufs_offset_db).toBe(-11);
    expect(plan.exportSettings.advanced.bit_depth).toBe(24);
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

  it("keeps the fixed iPhone WAV destination when loudness changes", () => {
    const plan = buildIphoneSimplePlan({ loudness: "high" });

    expect(plan.exportSettings.advanced.bit_depth).toBe(24);
    expect(plan.exportSettings.advanced.ceiling_dbtp).toBe(-1);
    expect(plan.exportSettings.advanced.target_sample_rate).toBe(44_100);
    expect(plan.exportSettings.advanced.lufs_offset_db).toBe(-9);
    expect(plan.usesAdaptiveAnalysis).toBe(false);
  });
});
