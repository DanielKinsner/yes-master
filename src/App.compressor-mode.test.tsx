import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AdvancedPanel } from "./App";
import type { AdvancedSettings, MasteringSettings } from "./bindings";

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
): MasteringSettings {
  return {
    preset: { kind: "universal" },
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
  settings: MasteringSettings;
  onAdvanced?: (advanced: AdvancedSettings) => void;
}): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <AdvancedPanel
        settings={props.settings}
        onAdvanced={props.onAdvanced ?? vi.fn()}
        onInputGain={vi.fn()}
        onOutputGain={vi.fn()}
        onDeliveryProfile={vi.fn()}
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

afterEach(() => {
  document.body.innerHTML = "";
});

describe("AdvancedPanel compressor mode", () => {
  it("keeps per-band compressor controls inactive until Manual is selected", async () => {
    const preset = await renderAdvancedPanel({
      settings: makeSettings({ compression_mode: "preset" }),
    });
    expect(preset.container.textContent).toContain(
      "Preset values from Universal.",
    );
    expect(preset.container.textContent).toContain("Preset · -16.0 dB");
    expect(preset.container.textContent).toContain("Preset · 1.8:1");
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
        compression_low_threshold_db: -16,
        compression_low_ratio: 1.8,
        compression_low_attack_ms: 15,
        compression_low_release_ms: 250,
        compression_mid_threshold_db: -16,
        compression_mid_ratio: 1.8,
        compression_mid_attack_ms: 15,
        compression_mid_release_ms: 250,
        compression_high_threshold_db: -16,
        compression_high_ratio: 1.8,
        compression_high_attack_ms: 15,
        compression_high_release_ms: 250,
      }),
    );

    await act(async () => {
      root.unmount();
    });
  });
});
