import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  AdvancedPanel,
  activeModifierChips,
  activeModifierSummary,
} from "./App";
import type {
  AdvancedSettings,
  AnalysisResult,
  MasteringSettings,
  Preset,
} from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const DEFAULT_ADVANCED: AdvancedSettings = {
  lufs_offset_db: null,
  ceiling_dbtp: null,
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
  bit_depth: null,
  target_sample_rate: null,
};

function makeSettings(
  advanced: Partial<AdvancedSettings> = {},
  preset: Preset = { kind: "universal" },
): MasteringSettings {
  return {
    preset,
    intensity: 0.5,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    volume_match: false,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "streaming-universal",
    advanced: { ...DEFAULT_ADVANCED, ...advanced },
  };
}

async function renderAdvancedPanel(props: {
  analysis?: AnalysisResult;
  settings: MasteringSettings;
  onAdvanced?: (advanced: AdvancedSettings) => void;
  onInputGain?: (db: number) => void;
  onOutputGain?: (db: number) => void;
  onDeliveryProfile?: (profile: MasteringSettings["delivery_profile"]) => void;
}): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <AdvancedPanel
        analysis={props.analysis}
        settings={props.settings}
        onAdvanced={props.onAdvanced ?? vi.fn()}
        onInputGain={props.onInputGain ?? vi.fn()}
        onOutputGain={props.onOutputGain ?? vi.fn()}
        onDeliveryProfile={props.onDeliveryProfile ?? vi.fn()}
      />,
    );
  });
  return { container, root };
}

function compressionInputs(container: Element): HTMLInputElement[] {
  return Array.from(
    container.querySelectorAll(".per-band-active-body input"),
  ).filter((input): input is HTMLInputElement => input instanceof HTMLInputElement);
}

function buttonNamed(container: Element, name: string): HTMLButtonElement {
  const button = Array.from(container.querySelectorAll("button")).find(
    (candidate) => candidate.textContent?.trim() === name,
  );
  if (!(button instanceof HTMLButtonElement)) {
    throw new Error(`${name} button not found`);
  }
  return button;
}

function makeAnalysis(dynamicRangeLu: number): AnalysisResult {
  const settings = makeSettings();
  return {
    track_id: "track-1",
    lufs_integrated: -14,
    lufs_short_term_max: -12,
    true_peak_dbtp: -1.2,
    dynamic_range_lu: dynamicRangeLu,
    spectral_balance: { low: 0.33, mid: 0.34, high: 0.33 },
    transient_density: 0.5,
    stereo_width: 0.5,
    recommended_universal: settings,
    measured_at_iso: "2026-05-26T00:00:00Z",
    inferred_role: null,
    role_confidence: null,
    inferred_character: null,
    character_confidence: null,
    spectral_balance_6band: null,
    transient_flux: null,
    stereo_correlation: null,
    dynamic_range_p95_p10_db: null,
    lufs_short_term_max_3s: null,
    energy_density_score: null,
  };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("AdvancedPanel compressor mode", () => {
  it("summarizes active modifiers without listing neutral defaults", () => {
    const chips = activeModifierChips(
      {
        ...makeSettings({
          lufs_offset_db: -20,
          width: 0.95,
          compression_density: 0.7,
          compression_mode: "manual",
        }),
        output_gain_db: -6.5,
        delivery_profile: "custom",
      },
      false,
      true,
    );

    expect(chips.map((chip) => chip.label)).toEqual([
      "Output -6.5 dB",
      "Target -20 LUFS",
      "Width 0.95",
      "Compressor Manual",
      "Preview LUFS On",
    ]);
  });

  it("shows no active modifier chips for neutral settings and inactive preview toggles", () => {
    expect(activeModifierChips(makeSettings(), false, false)).toEqual([]);
  });

  it("collapses active modifiers to one compact summary for the header", () => {
    const summary = activeModifierSummary(
      activeModifierChips(
        {
          ...makeSettings({ lufs_offset_db: -20, compression_mode: "manual" }),
          output_gain_db: -6.5,
          delivery_profile: "custom",
        },
        false,
        true,
      ),
    );

    expect(summary?.label).toBe("Active 4");
    expect(summary?.title).toContain("Output -6.5 dB");
    expect(summary?.title).toContain("Target -20 LUFS");
    expect(summary?.title).toContain("Compressor Manual");
    expect(summary?.title).toContain("Preview LUFS On");
  });

  it("does not show a reset button on delivery profile", async () => {
    const { container, root } = await renderAdvancedPanel({
      settings: makeSettings(),
    });

    expect(
      container.querySelector('button[aria-label="Reset delivery profile"]'),
    ).toBeNull();

    await act(async () => {
      root.unmount();
    });
  });

  it("resets the advanced controls section without touching compressor mode", async () => {
    const onAdvanced = vi.fn();
    const onInputGain = vi.fn();
    const onOutputGain = vi.fn();
    const settings = {
      ...makeSettings({
        lufs_offset_db: -20,
        ceiling_dbtp: -0.5,
        width: 0.95,
        warmth: 0.55,
        presence_air: 0.65,
        compression_density: 0.7,
        compression_mode: "manual",
      }),
      input_gain_db: 2,
      output_gain_db: -6.5,
      delivery_profile: "custom" as const,
    };
    const { container, root } = await renderAdvancedPanel({
      settings,
      onAdvanced,
      onInputGain,
      onOutputGain,
    });

    await act(async () => {
      const button = container.querySelector(
        'button[aria-label="Reset advanced controls"]',
      );
      if (!(button instanceof HTMLButtonElement)) {
        throw new Error("Reset advanced controls button not found");
      }
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onInputGain).toHaveBeenCalledWith(0);
    expect(onOutputGain).toHaveBeenCalledWith(0);
    expect(onAdvanced).toHaveBeenCalledWith({
      ...settings.advanced,
      lufs_offset_db: null,
      ceiling_dbtp: null,
      width: null,
      warmth: null,
      presence_air: null,
      compression_density: null,
    });
    await act(async () => {
      root.unmount();
    });
  });

  it("resets the per-band compressor values without changing the selected mode", async () => {
    const onAdvanced = vi.fn();
    const settings = makeSettings({
      compression_mode: "manual",
      compression_density: 0.8,
      compression_link_stereo: false,
      compression_low_threshold_db: -18,
      compression_low_ratio: 3,
      compression_low_attack_ms: 15,
      compression_low_release_ms: 250,
      compression_mid_threshold_db: -20,
      compression_mid_ratio: 2,
      compression_mid_attack_ms: 20,
      compression_mid_release_ms: 300,
      compression_high_threshold_db: -22,
      compression_high_ratio: 1.8,
      compression_high_attack_ms: 5,
      compression_high_release_ms: 120,
    });
    const { container, root } = await renderAdvancedPanel({
      settings,
      onAdvanced,
    });

    await act(async () => {
      const button = container.querySelector(
        'button[aria-label="Reset per-band compressor"]',
      );
      if (!(button instanceof HTMLButtonElement)) {
        throw new Error("Reset per-band compressor button not found");
      }
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onAdvanced).toHaveBeenCalledWith({
      ...settings.advanced,
      compression_mode: "manual",
      compression_density: null,
      compression_link_stereo: null,
      compression_low_threshold_db: -12.5,
      compression_low_ratio: 1.45,
      compression_low_attack_ms: 15,
      compression_low_release_ms: 250,
      compression_mid_threshold_db: -12.5,
      compression_mid_ratio: 1.45,
      compression_mid_attack_ms: 15,
      compression_mid_release_ms: 250,
      compression_high_threshold_db: -12.5,
      compression_high_ratio: 1.45,
      compression_high_attack_ms: 15,
      compression_high_release_ms: 250,
    });
    await act(async () => {
      root.unmount();
    });
  });

  it("keeps per-band compressor controls inactive until Manual is selected", async () => {
    const preset = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "preset" }),
    });
    expect(preset.container.textContent).toContain(
      "Preset values from Universal.",
    );
    expect(preset.container.textContent).toContain(
      "Preset compressor: -12.5 dB · 1.4:1 · 15 ms · 250 ms",
    );
    expect(preset.container.textContent).not.toContain("LOWMIDHIGH");
    expect(compressionInputs(preset.container).every((input) => input.disabled)).toBe(
      true,
    );
    await act(async () => {
      preset.root.unmount();
    });

    const manual = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "manual" }),
    });
    expect(manual.container.textContent).toContain(
      "Manual values replace preset compression.",
    );
    expect(compressionInputs(manual.container).some((input) => input.disabled)).toBe(
      false,
    );
    await act(async () => {
      manual.root.unmount();
    });

    const off = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "off" }),
    });
    expect(off.container.textContent).toContain("Creative compressor bypassed");
    expect(off.container.textContent).toContain("Off");
    expect(compressionInputs(off.container).every((input) => input.disabled)).toBe(
      true,
    );
    await act(async () => {
      off.root.unmount();
    });
  });

  it("uses compact compressor knobs in Manual mode", async () => {
    const onAdvanced = vi.fn();
    const { container, root } = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "manual" }),
      onAdvanced,
    });

    expect(container.querySelector(".compressor-knob-grid")).not.toBeNull();
    expect(container.textContent).toContain("Threshold");
    expect(container.textContent).toContain("Ratio");
    expect(container.textContent).toContain("Attack");
    expect(container.textContent).toContain("Release");
    expect(container.querySelectorAll(".compressor-knob-grid .knob")).toHaveLength(4);
    expect(container.querySelector(".compression-band-column")).toBeNull();

    const threshold = container.querySelector('input[aria-label="Threshold"]');
    expect(threshold).toBeInstanceOf(HTMLInputElement);
    expect((threshold as HTMLInputElement).value).toBe("-12.5");
    await act(async () => {
      root.unmount();
    });
  });

  it("shows preset compressor readouts instead of stale manual values", async () => {
    const universal = await renderAdvancedPanel({
      settings: makeSettings({
        compression_mode: "preset",
        compression_low_threshold_db: -22,
        compression_low_ratio: 2.6,
        compression_low_attack_ms: 25,
        compression_low_release_ms: 280,
      }),
    });
    expect(universal.container.textContent).toContain(
      "Preset values from Universal.",
    );
    expect(universal.container.textContent).toContain(
      "Preset compressor: -12.5 dB · 1.4:1 · 15 ms · 250 ms",
    );
    expect(universal.container.textContent).not.toContain("-22.0 dB");
    expect(universal.container.textContent).not.toContain("2.6:1");
    await act(async () => {
      universal.root.unmount();
    });

    const tape = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "preset" }, { kind: "tape" }),
    });
    expect(tape.container.textContent).toContain("Preset values from Tape.");
    expect(tape.container.textContent).toContain(
      "Preset compressor: -16.0 dB · 1.6:1 · 30 ms · 400 ms",
    );
    await act(async () => {
      tape.root.unmount();
    });
  });

  it("materializes preset compressor values when the user chooses Manual", async () => {
    const onAdvanced = vi.fn();
    const { container, root } = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "preset" }),
      onAdvanced,
    });

    await act(async () => {
      buttonNamed(container, "Manual").dispatchEvent(
        new MouseEvent("click", { bubbles: true }),
      );
    });

    expect(onAdvanced).toHaveBeenCalledWith(
      expect.objectContaining({
        compression_mode: "manual",
        compression_low_threshold_db: -12.5,
        compression_low_ratio: 1.45,
        compression_low_attack_ms: 15,
        compression_low_release_ms: 250,
        compression_mid_threshold_db: -12.5,
        compression_mid_ratio: 1.45,
        compression_mid_attack_ms: 15,
        compression_mid_release_ms: 250,
        compression_high_threshold_db: -12.5,
        compression_high_ratio: 1.45,
        compression_high_attack_ms: 15,
        compression_high_release_ms: 250,
      }),
    );

    await act(async () => {
      root.unmount();
    });
  });

  it("uses preset fallback labels and source guidance instead of Auto copy", async () => {
    const manual = await renderAdvancedPanel({
      analysis: makeAnalysis(4.8),
      settings: makeSettings({ compression_mode: "manual" }),
    });

    expect(manual.container.textContent).toContain("-12.5 dB");
    expect(manual.container.textContent).not.toContain("Auto · -12.5 dB");

    await act(async () => {
      manual.root.unmount();
    });

    const preset = await renderAdvancedPanel({
      analysis: makeAnalysis(4.8),
      settings: makeSettings({ compression_mode: "preset" }),
    });
    expect(preset.container.textContent).toContain("Source dynamic range is low");

    await act(async () => {
      preset.root.unmount();
    });
  });
});
