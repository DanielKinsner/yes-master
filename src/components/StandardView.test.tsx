// src/components/StandardView.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { LoudnessSegmented, StandardView, StyleTiles } from "./StandardView";
import type { useTrackMaster } from "../hooks/useTrackMaster";

type TM = ReturnType<typeof useTrackMaster>;

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("StyleTiles", () => {
  it("renders the four reference-4 tiles with labels", async () => {
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "universal" }} onSelect={() => {}} />,
    );
    const text = container.textContent ?? "";
    for (const label of ["Balanced", "Bright", "Warm", "Heavy"]) {
      expect(text).toContain(label);
    }
    await act(async () => root.unmount());
  });

  it("marks the active tile from the current preset", async () => {
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "tape" }} onSelect={() => {}} />,
    );
    const active = container.querySelector(".std-tile.is-active");
    expect(active?.textContent).toContain("Warm");
    await act(async () => root.unmount());
  });

  it("calls onSelect with the mapped preset when a tile is clicked", async () => {
    const onSelect = vi.fn();
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "universal" }} onSelect={onSelect} />,
    );
    const tiles = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-tile"));
    const heavy = tiles.find((t) => t.textContent?.includes("Heavy"))!;
    await act(async () => { heavy.click(); });
    expect(onSelect).toHaveBeenCalledWith({ kind: "oomph" });
    await act(async () => root.unmount());
  });
});

describe("LoudnessSegmented", () => {
  it("renders Low/Medium/High and marks the active step from the LUFS target", async () => {
    const { container, root } = await render(
      <LoudnessSegmented targetLufs={-11} onSelect={() => {}} />,
    );
    const text = container.textContent ?? "";
    for (const label of ["Low", "Medium", "High"]) expect(text).toContain(label);
    const active = container.querySelector(".std-seg-option.is-active");
    expect(active?.textContent).toContain("Medium");
    await act(async () => root.unmount());
  });

  it("calls onSelect with the mapped LUFS target", async () => {
    const onSelect = vi.fn();
    const { container, root } = await render(
      <LoudnessSegmented targetLufs={-14} onSelect={onSelect} />,
    );
    const opts = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-seg-option"));
    const high = opts.find((o) => o.textContent?.includes("High"))!;
    await act(async () => { high.click(); });
    expect(onSelect).toHaveBeenCalledWith(-9);
    await act(async () => root.unmount());
  });
});

function fakeSettings() {
  return {
    preset: { kind: "tape" }, intensity: 0.5,
    eq_sub_db: 0, eq_low_db: 0, eq_low_mid_db: 0, eq_mid_db: 0,
    eq_high_mid_db: 0, eq_high_db: 0, eq_sparkle_db: 0,
    volume_match: false, source_lufs_integrated: null,
    input_gain_db: 0, output_gain_db: 0, delivery_profile: "custom",
    album: null,
    advanced: {
      lufs_offset_db: -11, ceiling_dbtp: -1, width: null, warmth: null,
      presence_air: null, compression_mode: "preset", compression_density: null,
      compression_low_threshold_db: null, compression_low_ratio: null,
      compression_low_attack_ms: null, compression_low_release_ms: null,
      compression_mid_threshold_db: null, compression_mid_ratio: null,
      compression_mid_attack_ms: null, compression_mid_release_ms: null,
      compression_high_threshold_db: null, compression_high_ratio: null,
      compression_high_attack_ms: null, compression_high_release_ms: null,
      compression_link_stereo: null, bit_depth: 24, target_sample_rate: 44_100,
      adaptive_strength: 0.5,
    },
  };
}

function fakeTm(overrides: Partial<TM> = {}): TM {
  const noop = () => {};
  return {
    tracks: [{ id: "t1", path: "C:/a.wav", display_name: "Song.wav", source_format: "wav", duration_seconds: 100 }],
    selectedTrackId: "t1",
    selectedTrack: { id: "t1", path: "C:/a.wav", display_name: "Song.wav", source_format: "wav", duration_seconds: 100 },
    selectedAnalysis: { track_id: "t1", lufs_integrated: -16, true_peak_dbtp: -1.2, dynamic_range_lu: 9 },
    selectedWaveform: undefined,
    selectedSettings: fakeSettings(),
    isAnalyzing: false, isLoadingWaveform: false, analysisProgress: null,
    isExporting: false, isRendering: false, lastExportReceipt: null,
    transport: { isPlaying: false, currentTimeSec: 0, playbackKind: "master", volumeMatch: false },
    selectedRegion: null,
    setPreset: noop, setIntensity: noop, setLoudnessTarget: noop,
    setPlaybackKind: noop, setVolumeMatch: noop, togglePlay: noop, seek: noop,
    setRegion: noop, clearRegion: noop, openImportDialog: noop, selectTrack: noop,
    exportStandardMaster: noop, clearExportReceipt: noop,
    ...overrides,
  } as unknown as TM;
}

describe("StandardView", () => {
  it("renders style tiles, an intensity control, loudness steps, and the Create Master CTA", async () => {
    const { container, root } = await render(<StandardView tm={fakeTm()} />);
    const text = container.textContent ?? "";
    expect(text).toContain("Balanced");
    expect(text).toContain("Low");
    expect(text).toContain("Create Master");
    await act(async () => root.unmount());
  });

  it("routes a style click to setPreset", async () => {
    const setPreset = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ setPreset })} />);
    const tiles = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-tile"));
    tiles.find((t) => t.textContent?.includes("Bright"))!;
    await act(async () => { tiles.find((t) => t.textContent?.includes("Bright"))!.click(); });
    expect(setPreset).toHaveBeenCalledWith({ kind: "clarity" });
    await act(async () => root.unmount());
  });

  it("Create Master triggers exportStandardMaster", async () => {
    const exportStandardMaster = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ exportStandardMaster })} />);
    const cta = Array.from(container.querySelectorAll<HTMLButtonElement>("button")).find((b) => b.textContent?.includes("Create Master"))!;
    await act(async () => { cta.click(); });
    expect(exportStandardMaster).toHaveBeenCalled();
    await act(async () => root.unmount());
  });
});
