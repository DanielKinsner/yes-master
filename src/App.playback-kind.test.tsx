// Alive pass 1 (2026-08-19) — the .app root tells CSS what the user is
// hearing. Every "you are hearing the master" effect keys on these two
// attributes, so they are pinned here rather than inferred from a class.

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const mocks = vi.hoisted(() => ({
  tm: null as Record<string, unknown> | null,
}));

vi.mock("./hooks/useTrackMaster", () => ({
  useTrackMaster: () => {
    if (!mocks.tm) throw new Error("mock tm not configured");
    return mocks.tm;
  },
}));

function trackState(transport: {
  playbackKind: "source" | "master";
  isPlaying: boolean;
}): Record<string, unknown> {
  return {
    mode: "track",
    setMode: vi.fn(),
    saveProjectAs: vi.fn(),
    openProjectFromDisk: vi.fn(),
    tracks: [],
    selectedTrackId: null,
    selectedTrack: null,
    selectedAnalysis: undefined,
    selectedWaveform: undefined,
    selectedSettings: undefined,
    selectedRegion: null,
    selectTrack: vi.fn(),
    removeTrack: vi.fn(),
    openImportDialog: vi.fn(),
    isAnalyzing: false,
    isLoadingWaveform: false,
    isDragOver: false,
    isExporting: false,
    isRendering: false,
    previewStale: false,
    updatePreview: vi.fn(),
    exportMaster: vi.fn(),
    error: null,
    clearError: vi.fn(),
    lastExportReceipt: null,
    clearExportReceipt: vi.fn(),
    reorderTracks: vi.fn(),
    overrideAlbum: new Set(),
    albumArcKind: "cinematic",
    albumIntensity: 1,
    albumTitle: "",
    albumRendering: false,
    albumExportReport: null,
    albumSampleRate: null,
    albumBitDepth: null,
    setAlbumArc: vi.fn(),
    setAlbumIntensity: vi.fn(),
    setAlbumTitle: vi.fn(),
    setAlbumSampleRate: vi.fn(),
    setAlbumBitDepth: vi.fn(),
    exportAlbumPlan: vi.fn(),
    transport: {
      isPlaying: transport.isPlaying,
      currentTimeSec: 0,
      playbackKind: transport.playbackKind,
      loop: false,
      volumeMatch: false,
      exportLufsPreview: false,
      peakDbfs: -120,
      compressionGr: { low: -120, mid: -120, high: -120 },
      lufsMomentary: -120,
      lufsIntegrated: -120,
      spectrumDb: [],
    },
    renderProgress: null,
    undo: vi.fn(),
    redo: vi.fn(),
    canUndo: false,
    canRedo: false,
    setPreset: vi.fn(),
    setIntensity: vi.fn(),
    setEqBand: vi.fn(),
    setAdvanced: vi.fn(),
    setInputGain: vi.fn(),
    setOutputGain: vi.fn(),
    setDeliveryProfile: vi.fn(),
    setLoudnessTarget: vi.fn(),
    setDeliveryBitDepth: vi.fn(),
    setDeliverySampleRate: vi.fn(),
    togglePlay: vi.fn(),
    seek: vi.fn(),
    setPlaybackKind: vi.fn(),
    toggleLoop: vi.fn(),
    setVolumeMatch: vi.fn(),
    setExportLufsPreview: vi.fn(),
    setRegion: vi.fn(),
    clearRegion: vi.fn(),
    albumIntent: null,
    selectedIsOverriding: false,
    followingAlbumIntent: false,
    toggleOverrideAlbum: vi.fn(),
    userPresets: [],
    savingPreset: false,
    saveCurrentPreset: vi.fn(),
    deleteUserPresetById: vi.fn(),
    hadPriorSession: true,
    setForceWysiwyg: vi.fn(),
    resetToStandardManaged: vi.fn(),
    exportStandardMaster: vi.fn(),
    saveUserPreset: vi.fn(),
  };
}

let root: Root | null = null;

async function mount(): Promise<HTMLDivElement> {
  const host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  await act(async () => {
    root!.render(<App />);
  });
  return host;
}

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  mocks.tm = null;
  document.body.innerHTML = "";
});

describe("app root exposes what the user is hearing", () => {
  it("stamps data-playback-kind=source / data-playing=false while idle on Original", async () => {
    mocks.tm = trackState({ playbackKind: "source", isPlaying: false });
    const host = await mount();
    const app = host.querySelector(".app");
    expect(app).not.toBeNull();
    expect(app!.getAttribute("data-playback-kind")).toBe("source");
    expect(app!.getAttribute("data-playing")).toBe("false");
  });

  it("stamps data-playback-kind=master / data-playing=true while the master plays", async () => {
    mocks.tm = trackState({ playbackKind: "master", isPlaying: true });
    const host = await mount();
    const app = host.querySelector(".app");
    expect(app!.getAttribute("data-playback-kind")).toBe("master");
    expect(app!.getAttribute("data-playing")).toBe("true");
  });
});
