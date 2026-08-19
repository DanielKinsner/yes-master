import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";

import type { GuardrailReadout, MasteringSettings } from "../bindings";
import { AdvancedPanel } from "./AdvancedPanel";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const DEFAULT_SETTINGS: MasteringSettings = {
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
  advanced: {
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
    adaptive_strength: 0.5,
  },
};

async function renderPanel(opts?: {
  settings?: MasteringSettings;
  adaptiveReadout?: GuardrailReadout | null;
  onAdvanced?: (adv: MasteringSettings["advanced"]) => void;
  onResetAll?: () => void;
  canResetAll?: boolean;
}): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <AdvancedPanel
        settings={opts?.settings ?? DEFAULT_SETTINGS}
        onAdvanced={opts?.onAdvanced ?? vi.fn()}
        onInputGain={vi.fn()}
        onOutputGain={vi.fn()}
        onLoudnessTarget={vi.fn()}
        onDeliveryProfile={vi.fn()}
        onDeliveryBitDepth={vi.fn()}
        onDeliverySampleRate={vi.fn()}
        adaptiveReadout={opts?.adaptiveReadout}
        onResetAll={opts?.onResetAll}
        canResetAll={opts?.canResetAll}
      />,
    );
  });
  return { container, root };
}

function readoutWithAutoWidth(effectiveAutoWidth: number): GuardrailReadout {
  return {
    active: true,
    strength: 0.5,
    bright_trim: 0,
    low_trim: 0,
    density_trim: 0,
    width_trim: 0,
    brightness_share: 0,
    low_share: 0,
    dynamic_range_db: 9,
    effective_auto_width: effectiveAutoWidth,
  };
}

describe("AdvancedPanel", () => {
  it("renders no rail header without a global reset; with one, Reset all is disabled until something is edited (owner 2026-08-19)", async () => {
    const bare = await renderPanel();
    expect(bare.container.querySelector(".rail-header")).toBeNull();
    await act(async () => {
      bare.root.unmount();
    });

    const onResetAll = vi.fn();
    const clean = await renderPanel({ onResetAll, canResetAll: false });
    const btn = clean.container.querySelector<HTMLButtonElement>("button.rail-reset-all");
    expect(btn).not.toBeNull();
    expect(btn?.disabled).toBe(true);
    await act(async () => {
      clean.root.unmount();
    });

    const dirty = await renderPanel({ onResetAll, canResetAll: true });
    const live = dirty.container.querySelector<HTMLButtonElement>("button.rail-reset-all");
    expect(live?.disabled).toBe(false);
    await act(async () => {
      live?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onResetAll).toHaveBeenCalledTimes(1);
    await act(async () => {
      dirty.root.unmount();
    });
  });

  it("states that Track Master delivery exports WAV files", async () => {
    const { container, root } = await renderPanel();

    expect(container.textContent).toContain("Track Master exports WAV files.");
    await act(async () => {
      root.unmount();
    });
  });

  it("parks the Width slider at the resolved Auto value and prints it (owner smoke F10)", async () => {
    // With the thumb previously at min (0) on Auto, dragging to 0.05 looked
    // like a tiny increase when it replaced the ~1.11 preset baseline with
    // near-mono. The thumb and readout must tell the truth.
    const { container, root } = await renderPanel({
      adaptiveReadout: readoutWithAutoWidth(1.11),
    });

    const width = container.querySelector<HTMLInputElement>(
      'input[type="range"][aria-label="Width"]',
    );
    expect(width).not.toBeNull();
    expect(width?.value).toBe("1.11");
    expect(container.textContent).toContain("Auto · 1.11");
    await act(async () => {
      root.unmount();
    });
  });

  it("offers a visible reset-to-Auto on an engaged Width and writes null (owner smoke F10)", async () => {
    // Sliding back to 0 is NOT Auto (0 = full mono); the only ways back were
    // an invisible double-click, clearing the number input, or undo.
    const onAdvanced = vi.fn();
    const engaged: MasteringSettings = {
      ...DEFAULT_SETTINGS,
      advanced: { ...DEFAULT_SETTINGS.advanced, width: 0.05 },
    };
    const { container, root } = await renderPanel({
      settings: engaged,
      adaptiveReadout: readoutWithAutoWidth(1.11),
      onAdvanced,
    });

    const reset = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Reset Width to Auto"]',
    );
    expect(reset).not.toBeNull();
    await act(async () => {
      reset?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onAdvanced).toHaveBeenCalledTimes(1);
    expect(onAdvanced.mock.calls[0][0].width).toBeNull();
    await act(async () => {
      root.unmount();
    });
  });
});
