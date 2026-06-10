import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { Macros } from "./App";
import { WaveformLoading, WaveformView } from "./components/Waveform";
import type { MasteringSettings, WaveformPeaks } from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT =
  true;

function settings(overrides: Partial<MasteringSettings> = {}): MasteringSettings {
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
    ...overrides,
  };
}

afterEach(() => {
  document.body.innerHTML = "";
});

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

describe("WaveformLoading", () => {
  it("shows the analysis stage label, percent, and a determinate progress bar", async () => {
    const { container, root } = await render(
      <WaveformLoading
        isAnalyzing
        isLoadingWaveform={false}
        analysisProgress={{ label: "Reading tonal balance", progress: 0.32 }}
      />,
    );
    const text = container.textContent ?? "";
    expect(text).toContain("Reading tonal balance");
    expect(text).toContain("32%");
    const bar = container.querySelector('[role="progressbar"]');
    expect(bar).not.toBeNull();
    expect(bar?.getAttribute("aria-valuenow")).toBe("32");
    expect(bar?.classList.contains("is-indeterminate")).toBe(false);
    await act(async () => root.unmount());
  });

  it("shows an indeterminate bar with no percent while the waveform decodes", async () => {
    const { container, root } = await render(
      <WaveformLoading
        isAnalyzing={false}
        isLoadingWaveform
        analysisProgress={null}
      />,
    );
    expect(container.textContent ?? "").toContain("Loading waveform…");
    expect(container.textContent ?? "").not.toContain("%");
    const bar = container.querySelector('[role="progressbar"]');
    expect(bar?.classList.contains("is-indeterminate")).toBe(true);
    expect(bar?.getAttribute("aria-valuenow")).toBeNull();
    await act(async () => root.unmount());
  });

  it("shows the idle empty state with no bar when nothing is in flight", async () => {
    const { container, root } = await render(
      <WaveformLoading
        isAnalyzing={false}
        isLoadingWaveform={false}
        analysisProgress={null}
      />,
    );
    expect(container.textContent ?? "").toContain("No waveform yet.");
    expect(container.querySelector('[role="progressbar"]')).toBeNull();
    await act(async () => root.unmount());
  });
});

describe("WaveformView busy gating", () => {
  const peaks: WaveformPeaks = {
    track_id: "wf-1",
    channels: [[0.1, 0.3, 0.2, 0.4, 0.25]],
    samples_per_pixel: 512,
    total_samples: 2560,
    sample_rate: 44_100,
  };

  function renderView(
    p: WaveformPeaks | undefined,
    flags: { isAnalyzing: boolean; isLoading: boolean },
  ) {
    return render(
      <WaveformView
        peaks={p}
        isLoading={flags.isLoading}
        isAnalyzing={flags.isAnalyzing}
        analysisProgress={{ label: "Analyzing audio", progress: 0.14 }}
        currentTimeSec={0}
        durationSec={180}
        region={null}
        onSeek={vi.fn()}
        onSetRegion={vi.fn()}
        onClearRegion={vi.fn()}
      />,
    );
  }

  it("keeps the waveform on screen when peaks exist even while a different track analyzes", async () => {
    // Regression: importing a 2nd file flips the GLOBAL isAnalyzing flag; the
    // currently-selected, already-decoded track must NOT blank to the
    // progress surface.
    const { container, root } = await renderView(peaks, {
      isAnalyzing: true,
      isLoading: true,
    });
    expect(
      container.querySelector('[role="slider"][aria-label^="Waveform"]'),
    ).not.toBeNull();
    expect(container.querySelector('[role="progressbar"]')).toBeNull();
    expect(container.textContent ?? "").not.toContain("Analyzing audio");
    await act(async () => root.unmount());
  });

  it("shows the analysis progress surface when the selected track has no peaks yet", async () => {
    const { container, root } = await renderView(undefined, {
      isAnalyzing: true,
      isLoading: false,
    });
    expect(
      container.querySelector('[role="slider"][aria-label^="Waveform"]'),
    ).toBeNull();
    expect(container.querySelector('[role="progressbar"]')).not.toBeNull();
    expect(container.textContent ?? "").toContain("Analyzing audio");
    await act(async () => root.unmount());
  });
});

describe("Macros tone reset", () => {
  function renderMacros(s: MasteringSettings, onResetTone = vi.fn()) {
    return render(
      <Macros
        settings={s}
        onIntensity={vi.fn()}
        onEq={vi.fn()}
        onResetTone={onResetTone}
        onLoudnessTargetProfile={vi.fn()}
        spectrumDb={[]}
      />,
    );
  }

  it("fires onResetTone when the reset button is clicked", async () => {
    const onResetTone = vi.fn();
    const { container, root } = await renderMacros(
      settings({ eq_mid_db: 4, intensity: 0.8 }),
      onResetTone,
    );
    const btn = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Reset intensity & EQ to flat"]',
    );
    expect(btn).not.toBeNull();
    expect(btn?.disabled).toBe(false);
    await act(async () => {
      btn?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onResetTone).toHaveBeenCalledTimes(1);
    await act(async () => root.unmount());
  });

  it("disables the reset button when intensity and EQ are already flat", async () => {
    const onResetTone = vi.fn();
    const { container, root } = await renderMacros(settings(), onResetTone);
    const btn = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Reset intensity & EQ to flat"]',
    );
    expect(btn?.disabled).toBe(true);
    await act(async () => {
      btn?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onResetTone).not.toHaveBeenCalled();
    await act(async () => root.unmount());
  });
});
