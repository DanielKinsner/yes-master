// src/components/StandardView.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  LoudnessSegmented,
  StandardView,
  StyleTiles,
  sourceLufsCopy,
} from "./StandardView";
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
    const { container, root } = await render(<StandardView tm={fakeTm()} onEnterAdvanced={() => {}} />);
    const text = container.textContent ?? "";
    expect(text).toContain("Balanced");
    expect(text).toContain("Low");
    expect(text).toContain("Create Master");
    await act(async () => root.unmount());
  });

  it("routes a style click to setPreset", async () => {
    const setPreset = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ setPreset })} onEnterAdvanced={() => {}} />);
    const tiles = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-tile"));
    tiles.find((t) => t.textContent?.includes("Bright"))!;
    await act(async () => { tiles.find((t) => t.textContent?.includes("Bright"))!.click(); });
    expect(setPreset).toHaveBeenCalledWith({ kind: "clarity" });
    await act(async () => root.unmount());
  });

  it("Create Master triggers exportStandardMaster", async () => {
    const exportStandardMaster = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ exportStandardMaster })} onEnterAdvanced={() => {}} />);
    const cta = Array.from(container.querySelectorAll<HTMLButtonElement>("button")).find((b) => b.textContent?.includes("Create Master"))!;
    await act(async () => { cta.click(); });
    expect(exportStandardMaster).toHaveBeenCalled();
    await act(async () => root.unmount());
  });
});

describe("intensity zone chips", () => {
  it("renders the five intensityLabel zones with the current zone active", async () => {
    // fakeTm intensity 0.5 -> intensityLabel "Moderate"
    const { container, root } = await render(
      <StandardView tm={fakeTm()} onEnterAdvanced={() => {}} />,
    );
    const chips = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-zone-chip"));
    expect(chips.map((c) => c.textContent)).toEqual([
      "Subtle",
      "Restrained",
      "Moderate",
      "Driving",
      "Aggressive",
    ]);
    const active = chips.filter((c) => c.classList.contains("is-active"));
    expect(active).toHaveLength(1);
    expect(active[0].textContent).toBe("Moderate");
    await act(async () => root.unmount());
  });

  it("routes a zone click to setIntensity with a value inside that zone", async () => {
    const setIntensity = vi.fn();
    const { container, root } = await render(
      <StandardView tm={fakeTm({ setIntensity })} onEnterAdvanced={() => {}} />,
    );
    const chips = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-zone-chip"));
    await act(async () => { chips.find((c) => c.textContent === "Driving")!.click(); });
    expect(setIntensity).toHaveBeenCalledWith(0.8);
    await act(async () => root.unmount());
  });
});

describe("sourceLufsCopy", () => {
  it("reads 'close to' within ±1.5 LU of the target (band edges inclusive)", () => {
    expect(sourceLufsCopy(-12.5, -14)).toBe("Source -12.5 LUFS · close to your -14 LUFS target");
    expect(sourceLufsCopy(-14, -14)).toBe("Source -14.0 LUFS · close to your -14 LUFS target");
  });

  it("reads 'quieter than' below the band and 'louder than' above it", () => {
    expect(sourceLufsCopy(-22, -14)).toBe("Source -22.0 LUFS · quieter than your -14 LUFS target");
    expect(sourceLufsCopy(-8, -11)).toBe("Source -8.0 LUFS · louder than your -11 LUFS target");
  });

  it("shows the bare measurement when there is no target", () => {
    expect(sourceLufsCopy(-16.3, null)).toBe("Source -16.3 LUFS");
  });
});

const SECOND_TRACK = {
  id: "t2",
  path: "C:/b.wav",
  display_name: "B-side.wav",
  source_format: "wav",
  duration_seconds: 61,
};

function twoTrackTm(overrides: Partial<TM> = {}): TM {
  const base = fakeTm(overrides);
  return { ...base, tracks: [...base.tracks, SECOND_TRACK] } as TM;
}

describe("TracksRail", () => {
  it("lists every track with index and m:ss duration, marking the selected row", async () => {
    const { container, root } = await render(<StandardView tm={twoTrackTm()} onEnterAdvanced={() => {}} />);
    const rows = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-track-row"));
    expect(rows).toHaveLength(2);
    expect(rows[0].textContent).toContain("01");
    expect(rows[0].textContent).toContain("Song.wav");
    expect(rows[0].textContent).toContain("1:40");
    expect(rows[1].textContent).toContain("02");
    expect(rows[1].textContent).toContain("B-side.wav");
    expect(rows[1].textContent).toContain("1:01");
    expect(rows[0].classList.contains("is-active")).toBe(true);
    expect(rows[1].classList.contains("is-active")).toBe(false);
    await act(async () => root.unmount());
  });

  it("routes a row click to selectTrack with that track's id", async () => {
    const selectTrack = vi.fn();
    const { container, root } = await render(<StandardView tm={twoTrackTm({ selectTrack })} onEnterAdvanced={() => {}} />);
    const rows = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-track-row"));
    await act(async () => { rows[1].click(); });
    expect(selectTrack).toHaveBeenCalledWith("t2");
    await act(async () => root.unmount());
  });

  it("'+ Add Tracks' opens the import dialog", async () => {
    const openImportDialog = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ openImportDialog })} onEnterAdvanced={() => {}} />);
    const add = container.querySelector<HTMLButtonElement>(".std-add-track")!;
    await act(async () => { add.click(); });
    expect(openImportDialog).toHaveBeenCalled();
    await act(async () => root.unmount());
  });

  it("status chip is truthful: Analyzing… / Analyzed / Not analyzed (idle dot)", async () => {
    const analyzing = await render(<StandardView tm={fakeTm({ isAnalyzing: true })} onEnterAdvanced={() => {}} />);
    expect(analyzing.container.querySelector(".std-tracks-status")?.textContent).toContain("Analyzing…");
    await act(async () => analyzing.root.unmount());

    const analyzed = await render(<StandardView tm={fakeTm()} onEnterAdvanced={() => {}} />);
    const okChip = analyzed.container.querySelector(".std-tracks-status")!;
    expect(okChip.textContent).toContain("Analyzed");
    expect(okChip.classList.contains("is-idle")).toBe(false);
    await act(async () => analyzed.root.unmount());

    const idle = await render(<StandardView tm={fakeTm({ selectedAnalysis: undefined })} onEnterAdvanced={() => {}} />);
    const idleChip = idle.container.querySelector(".std-tracks-status")!;
    expect(idleChip.textContent).toContain("Not analyzed");
    expect(idleChip.classList.contains("is-idle")).toBe(true);
    await act(async () => idle.root.unmount());
  });
});

describe("StandardRightRail", () => {
  it("A/B reflects playbackKind via aria-pressed and routes clicks to setPlaybackKind", async () => {
    const setPlaybackKind = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ setPlaybackKind })} onEnterAdvanced={() => {}} />);
    const group = container.querySelector(".std-rail-ab")!;
    const buttons = Array.from(group.querySelectorAll<HTMLButtonElement>("button"));
    const original = buttons.find((b) => b.textContent === "Original")!;
    const mastered = buttons.find((b) => b.textContent === "Mastered")!;
    expect(mastered.getAttribute("aria-pressed")).toBe("true");
    expect(original.getAttribute("aria-pressed")).toBe("false");
    await act(async () => { original.click(); });
    expect(setPlaybackKind).toHaveBeenCalledWith("source");
    await act(async () => root.unmount());
  });

  it("Volume Match toggle routes to setVolumeMatch with the flipped value", async () => {
    const setVolumeMatch = vi.fn();
    const { container, root } = await render(<StandardView tm={fakeTm({ setVolumeMatch })} onEnterAdvanced={() => {}} />);
    const toggle = container.querySelector<HTMLButtonElement>(".std-volume-match")!;
    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    await act(async () => { toggle.click(); });
    expect(setVolumeMatch).toHaveBeenCalledWith(true);
    await act(async () => root.unmount());
  });

  it("Delivery 'Change' routes to onEnterAdvanced", async () => {
    const onEnterAdvanced = vi.fn();
    const { container, root } = await render(
      <StandardView tm={fakeTm()} onEnterAdvanced={onEnterAdvanced} />,
    );
    const change = container.querySelector<HTMLButtonElement>(".std-delivery-change")!;
    await act(async () => { change.click(); });
    expect(onEnterAdvanced).toHaveBeenCalled();
    await act(async () => root.unmount());
  });

  it("hero subtitle derives the LUFS qualifier from measurement vs target", async () => {
    // fakeTm: source -16 LUFS, delivery_profile "custom" -> target -11 => quieter.
    const { container, root } = await render(<StandardView tm={fakeTm()} onEnterAdvanced={() => {}} />);
    expect(container.querySelector(".std-source")?.textContent).toBe(
      "Source -16.0 LUFS · quieter than your -11 LUFS target",
    );
    await act(async () => root.unmount());
  });

  it("hero subtitle says 'not analyzed yet' instead of claiming analysis is running", async () => {
    const { container, root } = await render(
      <StandardView tm={fakeTm({ selectedAnalysis: undefined })} onEnterAdvanced={() => {}} />,
    );
    expect(container.querySelector(".std-source")?.textContent).toBe(
      "Source not analyzed yet",
    );
    await act(async () => root.unmount());
  });
});
