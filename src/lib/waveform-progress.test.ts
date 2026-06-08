import { describe, expect, it } from "vitest";

import { waveformLoadingView } from "./waveform-progress";

// Decides what the waveform deck shows while the track is being prepared:
// the staged analysis progress (determinate bar + percent), the waveform
// decode (indeterminate bar), or the idle empty state. Pure so the
// label/percent/mode contract is tested without driving the async import.

describe("waveformLoadingView", () => {
  it("reports the analysis stage label and percent while analyzing", () => {
    const view = waveformLoadingView({
      isAnalyzing: true,
      analysisProgress: { label: "Reading tonal balance", progress: 0.32 },
      isLoadingWaveform: false,
    });
    expect(view.mode).toBe("analyzing");
    expect(view.label).toBe("Reading tonal balance");
    expect(view.percent).toBe(32);
  });

  it("falls back to a generic analyzing label when no stage is set yet", () => {
    const view = waveformLoadingView({
      isAnalyzing: true,
      analysisProgress: null,
      isLoadingWaveform: false,
    });
    expect(view.mode).toBe("analyzing");
    expect(view.label).toBe("Analyzing…");
    expect(view.percent).toBe(0);
  });

  it("clamps the progress fraction into 0..100", () => {
    expect(
      waveformLoadingView({
        isAnalyzing: true,
        analysisProgress: { label: "x", progress: 1.4 },
        isLoadingWaveform: false,
      }).percent,
    ).toBe(100);
    expect(
      waveformLoadingView({
        isAnalyzing: true,
        analysisProgress: { label: "x", progress: -0.2 },
        isLoadingWaveform: false,
      }).percent,
    ).toBe(0);
  });

  it("shows an indeterminate loading state while the waveform decodes", () => {
    const view = waveformLoadingView({
      isAnalyzing: false,
      analysisProgress: null,
      isLoadingWaveform: true,
    });
    expect(view.mode).toBe("loading");
    expect(view.label).toBe("Loading waveform…");
    expect(view.percent).toBeNull();
  });

  it("prefers the analysis view when both flags are set (analysis runs first)", () => {
    const view = waveformLoadingView({
      isAnalyzing: true,
      analysisProgress: { label: "Checking dynamics", progress: 0.5 },
      isLoadingWaveform: true,
    });
    expect(view.mode).toBe("analyzing");
    expect(view.label).toBe("Checking dynamics");
  });

  it("returns the idle empty state when nothing is in flight", () => {
    const view = waveformLoadingView({
      isAnalyzing: false,
      analysisProgress: null,
      isLoadingWaveform: false,
    });
    expect(view.mode).toBe("idle");
    expect(view.label).toBe("No waveform yet.");
    expect(view.percent).toBeNull();
  });
});
