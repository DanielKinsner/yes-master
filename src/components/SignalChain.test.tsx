import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";

import type { MasteringSettings } from "../bindings";
import { SignalChain, STAGE_JUMP_TARGETS } from "./SignalChain";

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

async function renderSignalChain(): Promise<{
  container: HTMLDivElement;
  root: Root;
}> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(<SignalChain settings={DEFAULT_SETTINGS} />);
  });
  return { container, root };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("SignalChain", () => {
  it("renders as a static chain bar without an expand dropdown affordance", async () => {
    const { container, root } = await renderSignalChain();

    expect(container.querySelector(".signal-chain")).toBeTruthy();
    expect(container.querySelector(".signal-chain-toggle")).toBeNull();
    expect(container.querySelector("[aria-expanded]")).toBeNull();
    expect(
      container.querySelector('[aria-label*="signal chain detail" i]'),
    ).toBeNull();

    await act(async () => {
      root.unmount();
    });
  });

  // Pass 2 (2026-08-19): every stage that has an editable home is a button
  // that jumps to it — the strip stops reading as decoration. SOURCE has no
  // control, so it stays a plain node.
  it("stages with a home are buttons that scroll their target into view", async () => {
    // Stand in the targets so the jump has somewhere to land.
    const eq = document.createElement("section");
    eq.id = STAGE_JUMP_TARGETS.eq;
    const scrolled: string[] = [];
    (eq as unknown as { scrollIntoView: () => void }).scrollIntoView = () => scrolled.push(eq.id);
    document.body.appendChild(eq);

    const { container, root } = await renderSignalChain();
    const buttons = container.querySelectorAll("button.chain-stage");
    // in/source is not a button; the other seven are.
    expect(buttons.length).toBe(7);
    expect(container.querySelector(".chain-stage:not(button)")).not.toBeNull();

    const eqBtn = [...buttons].find((b) => b.textContent?.includes("EQ"))!;
    expect(eqBtn.getAttribute("title")).toMatch(/jump|open|go to/i);
    await act(async () => {
      eqBtn.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(scrolled).toEqual([STAGE_JUMP_TARGETS.eq]);
    expect(eq.classList.contains("is-jump-target")).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });
});
