import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";

import type { MasteringSettings } from "../bindings";
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

async function renderPanel(): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <AdvancedPanel
        settings={DEFAULT_SETTINGS}
        onAdvanced={vi.fn()}
        onInputGain={vi.fn()}
        onOutputGain={vi.fn()}
        onLoudnessTarget={vi.fn()}
        onDeliveryProfile={vi.fn()}
        onDeliveryBitDepth={vi.fn()}
        onDeliverySampleRate={vi.fn()}
      />,
    );
  });
  return { container, root };
}

describe("AdvancedPanel", () => {
  it("states that Track Master delivery exports WAV files", async () => {
    const { container, root } = await renderPanel();

    expect(container.textContent).toContain("Track Master exports WAV files.");
    await act(async () => {
      root.unmount();
    });
  });
});
