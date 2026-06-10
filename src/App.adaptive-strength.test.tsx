import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AdvancedPanel } from "./App";
import { ADAPTIVE_STRENGTH_DEFAULT } from "./bindings";
import { setAdaptiveReadoutEnabled } from "./lib/debug-flags";
import type {
  AdvancedSettings,
  GuardrailReadout,
  MasteringSettings,
} from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT =
  true;

const ADVANCED: AdvancedSettings = {
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

function settings(adaptiveStrength: number | null): MasteringSettings {
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
    advanced: { ...ADVANCED, adaptive_strength: adaptiveStrength },
  };
}

async function renderPanel(
  s: MasteringSettings,
  onAdvanced = vi.fn(),
  readout: GuardrailReadout | null = null,
) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <AdvancedPanel
        settings={s}
        onAdvanced={onAdvanced}
        onInputGain={vi.fn()}
        onOutputGain={vi.fn()}
        onLoudnessTarget={vi.fn()}
        onDeliveryProfile={vi.fn()}
        onDeliveryBitDepth={vi.fn()}
        onDeliverySampleRate={vi.fn()}
        adaptiveReadout={readout}
      />,
    );
  });
  return { container, root };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("AdvancedPanel adaptive strength control", () => {
  it("renders the Adapt strength control reflecting the current level", async () => {
    const { container, root } = await renderPanel(settings(0.6));
    const text = container.textContent ?? "";
    expect(text).toContain("Adapt strength");
    expect(text).toContain("60%");
    await act(async () => {
      root.unmount();
    });
  });

  it("shows the default (on, 50%) when strength is unset", async () => {
    const { container, root } = await renderPanel(settings(null));
    expect(container.textContent ?? "").toContain("50%");
    await act(async () => {
      root.unmount();
    });
  });

  it("shows Off when strength is 0 (preserves the explicit zero)", async () => {
    const { container, root } = await renderPanel(settings(0));
    expect(container.textContent ?? "").toContain("Off");
    await act(async () => {
      root.unmount();
    });
  });

  it("reports the new strength through onAdvanced when the slider moves", async () => {
    const onAdvanced = vi.fn();
    const { container, root } = await renderPanel(settings(0.6), onAdvanced);
    const slider = container.querySelector(
      'input[type="range"][aria-label="Adapt strength"]',
    );
    if (!(slider instanceof HTMLInputElement)) {
      throw new Error("Adapt strength slider not found");
    }
    await act(async () => {
      // React controls the value, so use the native setter before firing the
      // input event — the standard way to simulate user input in jsdom.
      const setter = Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set;
      setter?.call(slider, "0");
      slider.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(onAdvanced).toHaveBeenCalled();
    const arg = onAdvanced.mock.calls.at(-1)?.[0] as AdvancedSettings;
    expect(arg.adaptive_strength).toBe(0);
    await act(async () => {
      root.unmount();
    });
  });

  it("resets Adapt strength to the explicit default (0.5) on Reset", async () => {
    // Regression: Reset used to skip adaptive_strength entirely, so a user who
    // dragged it to 0/100% could never get back to default via Reset. Reset now
    // writes the explicit default (not null), like every other Advanced control.
    const onAdvanced = vi.fn();
    const { container, root } = await renderPanel(settings(0), onAdvanced);
    const resetBtn = container.querySelector(
      'button[aria-label="Reset advanced controls"]',
    );
    if (!(resetBtn instanceof HTMLButtonElement)) {
      throw new Error("Advanced reset button not found");
    }
    await act(async () => {
      resetBtn.click();
    });
    expect(onAdvanced).toHaveBeenCalled();
    const arg = onAdvanced.mock.calls.at(-1)?.[0] as AdvancedSettings;
    expect(arg.adaptive_strength).toBe(ADAPTIVE_STRENGTH_DEFAULT);
    await act(async () => {
      root.unmount();
    });
  });

  it("renders the per-axis adaptive trim readout when active and debug-enabled", async () => {
    const readout: GuardrailReadout = {
      active: true,
      strength: 0.6,
      bright_trim: 0.5,
      low_trim: 0,
      density_trim: 0.2,
      width_trim: 0.15,
      brightness_share: 0.34,
      low_share: 0.3,
      dynamic_range_db: 4,
      bright_deadband: 0.3,
      low_deadband: 0.42,
      width_corr_deadband: 0.5,
      stereo_correlation: 0.3,
    };
    // P3: the readout is a debug-gated iteration aid — enable the flag the
    // way a calibration session would.
    setAdaptiveReadoutEnabled(true);
    const { container, root } = await renderPanel(settings(0.6), vi.fn(), readout);
    setAdaptiveReadoutEnabled(false);
    const text = container.textContent ?? "";
    expect(text).toContain("Adaptive trims");
    expect(text).toContain("Highs -50%");
    expect(text).toContain("Comp -20%");
    expect(text).toContain("Width -15%");
    // Source context vs deadband is shown so a -0% axis is legible, not "broken".
    expect(text).toContain("presence+air 0.34 / 0.30");
    expect(text).toContain("sub+low 0.30 / 0.42");
    expect(text).toContain("DR 4.0 dB");
    expect(text).toContain("corr 0.30 / 0.50");
    // Lows trimmed 0% AND source below its deadband -> flagged "in range".
    expect(text).toContain("· in range");
    await act(async () => {
      root.unmount();
    });
  });

  it("hides the readout by default even when active (debug flag unset)", async () => {
    // P3 negative case: release builds must not show the iteration aid —
    // active guardrails alone are not enough without the localStorage flag.
    const readout: GuardrailReadout = {
      active: true,
      strength: 0.6,
      bright_trim: 0.5,
      low_trim: 0,
      density_trim: 0.2,
      width_trim: 0.15,
      brightness_share: 0.34,
      low_share: 0.3,
      dynamic_range_db: 4,
      bright_deadband: 0.3,
      low_deadband: 0.42,
      width_corr_deadband: 0.5,
      stereo_correlation: 0.3,
    };
    const { container, root } = await renderPanel(settings(0.6), vi.fn(), readout);
    expect(container.textContent ?? "").not.toContain("Adaptive trims");
    await act(async () => {
      root.unmount();
    });
  });

  it("hides the readout when inactive", async () => {
    const readout: GuardrailReadout = {
      active: false,
      strength: 0,
      bright_trim: 0,
      low_trim: 0,
      density_trim: 0,
      width_trim: 0,
      brightness_share: 0,
      low_share: 0,
      dynamic_range_db: 0,
      stereo_correlation: null,
    };
    const { container, root } = await renderPanel(settings(0.6), vi.fn(), readout);
    expect(container.textContent ?? "").not.toContain("Adaptive trims");
    await act(async () => {
      root.unmount();
    });
  });
});
