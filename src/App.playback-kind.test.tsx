// Alive pass 1 (2026-08-19) — the .app root tells CSS what the user is
// hearing. Every "you are hearing the master" effect keys on these two
// attributes, so they are pinned here rather than inferred from a class.

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import type { MasteringSettings } from "./bindings";

const SETTINGS: MasteringSettings = {
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

function trackState(
  transport: {
    playbackKind: "source" | "master";
    isPlaying: boolean;
  },
  overrides: Record<string, unknown> = {},
): Record<string, unknown> {
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
    isAnalyzing: false,
    analysisProgress: null,
    rememberTrackView: vi.fn(),
    rememberedTrackView: vi.fn(() => null),
    disarmLoop: vi.fn(),
    ...overrides,
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

// Pass 2 (2026-08-19): analysis progress used to print FOUR times on one
// Advanced screen (header pill, waveform slot, sidebar footer, bottom status
// bar). Two owners remain: the header SessionStatus pill (coarse state) and
// the waveform slot (rich progress). Pinned by counting the label.
describe("analysis progress has two owners, not four", () => {
  it("prints the stage label at most twice in Advanced while analyzing", async () => {
    const track = {
      id: "t1",
      path: "/audio/t1.wav",
      display_name: "t1.wav",
      source_format: "wav",
      duration_seconds: 120,
      sample_rate: 44_100,
      channels: 2,
    };
    mocks.tm = trackState(
      { playbackKind: "source", isPlaying: false },
      {
        tracks: [track],
        selectedTrackId: "t1",
        selectedTrack: track,
        selectedSettings: SETTINGS,
        isAnalyzing: true,
        analysisProgress: { label: "Checking dynamics", progress: 0.5 },
      },
    );
    window.localStorage.setItem(
      "yes-master:view-mode",
      JSON.stringify({ migrated: true, lastView: "advanced" }),
    );
    const host = await mount();
    const text = host.textContent ?? "";
    const count = text.split("Checking dynamics").length - 1;
    expect(count).toBeGreaterThanOrEqual(1);
    expect(count).toBeLessThanOrEqual(2);
    expect(host.querySelector(".sidebar-status")).toBeNull();
    expect(host.querySelector(".status-processing-label")).toBeNull();
  });
});
