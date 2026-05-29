import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AdvancedPanel } from "./App";
import type { MasteringSettings } from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const settings: MasteringSettings = {
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
  },
};

function panelProps() {
  return {
    settings,
    onAdvanced: vi.fn(),
    onInputGain: vi.fn(),
    onOutputGain: vi.fn(),
    onLoudnessTarget: vi.fn(),
    onDeliveryProfile: vi.fn(),
    onDeliveryBitDepth: vi.fn(),
    onDeliverySampleRate: vi.fn(),
  };
}

async function renderNode(node: ReactNode): Promise<{
  container: HTMLDivElement;
  root: Root;
}> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("AdvancedPanel delivery-format visibility", () => {
  it("shows the Delivery Format card by default (Track Master)", async () => {
    const { container } = await renderNode(<AdvancedPanel {...panelProps()} />);
    const titles = [...container.querySelectorAll(".panel-title")].map(
      (n) => n.textContent,
    );
    expect(titles).toContain("DELIVERY FORMAT");
  });

  it("hides the Delivery Format card when showDeliveryFormat is false (Album)", async () => {
    const { container } = await renderNode(
      <AdvancedPanel {...panelProps()} showDeliveryFormat={false} />,
    );
    const titles = [...container.querySelectorAll(".panel-title")].map(
      (n) => n.textContent,
    );
    expect(titles).not.toContain("DELIVERY FORMAT");
    // The other rail cards remain.
    expect(titles).toContain("DELIVERY PROFILE");
  });
});
