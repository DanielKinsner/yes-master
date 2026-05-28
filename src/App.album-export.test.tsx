import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import type {
  AnalysisResult,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "./bindings";

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

const track: ImportedTrack = {
  id: "album-track-1",
  path: "/audio/album-track-1.wav",
  display_name: "album-track-1.wav",
  source_format: "wav",
  duration_seconds: 120,
  sample_rate: 44_100,
  channels: 2,
};

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

const hotAnalysis: AnalysisResult = {
  track_id: track.id,
  lufs_integrated: -10.5,
  lufs_short_term_max: -8.8,
  true_peak_dbtp: 0.2,
  dynamic_range_lu: 3.3,
  spectral_balance: { low: 0.3, mid: 0.4, high: 0.3 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: settings,
  measured_at_iso: "2026-05-26T00:00:00.000Z",
  inferred_role: null,
  role_confidence: null,
  inferred_character: null,
  character_confidence: null,
  spectral_balance_6band: null,
  transient_flux: null,
  stereo_correlation: null,
  dynamic_range_p95_p10_db: null,
  lufs_short_term_max_3s: null,
  energy_density_score: null,
};

const cleanCheck: QualityCheck = {
  level: "info",
  code: "export_ok",
  message: "No export issues detected.",
};

const warningCheck: QualityCheck = {
  level: "warning",
  code: "streaming_headroom_low",
  message: "Streaming headroom is tight.",
};

function renderJob(outputPaths: string[]): RenderJob {
  return {
    id: "render-job-1",
    kind: "master",
    target_tracks: [track.id],
    status: { status: "done" },
    progress: 1,
    started_at_iso: "2026-05-19T00:00:00Z",
    output_paths: outputPaths,
  };
}

function baseTrackMasterState(): Record<string, unknown> {
  return {
    mode: "album",
    setMode: vi.fn(),
    saveProjectAs: vi.fn(),
    openProjectFromDisk: vi.fn(),
    tracks: [track],
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
    setAlbumArc: vi.fn(),
    setAlbumIntensity: vi.fn(),
    setAlbumTitle: vi.fn(),
    exportAlbumPlan: vi.fn(),
    transport: {
      isPlaying: false,
      currentTimeSec: 0,
      playbackKind: "source",
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
  };
}

async function renderApp(): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(<App />);
  });
  return { container, root };
}

afterEach(() => {
  document.body.innerHTML = "";
  mocks.tm = null;
});

describe("album export actions", () => {
  it("shows a single Album Export button in album mode", async () => {
    mocks.tm = baseTrackMasterState();

    const { container, root } = await renderApp();

    const exportButtons = Array.from(container.querySelectorAll("button")).filter(
      (button) => button.textContent?.trim() === "Export Album",
    );
    expect(exportButtons).toHaveLength(1);
    await act(async () => {
      root.unmount();
    });
  });

  it("shows a quiet export journey and clean result on completed exports", async () => {
    mocks.tm = {
      ...baseTrackMasterState(),
      lastExportReceipt: {
        trackId: track.id,
        outputPath: "/Users/daniel/Masters/album-track-1__master.wav",
        checks: [cleanCheck],
        job: renderJob(["/Users/daniel/Masters/album-track-1__master.wav"]),
        kind: "track",
      },
    };

    const { container, root } = await renderApp();

    expect(container.querySelector(".receipt-medallion-clean")?.textContent).toContain(
      "Clean",
    );
    const steps = Array.from(container.querySelectorAll(".receipt-journey-step"));
    expect(steps.map((step) => step.textContent?.trim())).toEqual([
      "Analyze",
      "Master",
      "Quality",
      "Saved",
    ]);
    expect(container.querySelector(".receipt-path-name")?.textContent).toBe(
      "album-track-1__master.wav",
    );

    await act(async () => {
      root.unmount();
    });
  });

  it("marks completed exports for review when quality checks warn", async () => {
    mocks.tm = {
      ...baseTrackMasterState(),
      lastExportReceipt: {
        trackId: track.id,
        outputPath: "/Users/daniel/Masters/album-track-1__master.wav",
        checks: [warningCheck],
        job: renderJob(["/Users/daniel/Masters/album-track-1__master.wav"]),
        kind: "track",
      },
    };

    const { container, root } = await renderApp();

    expect(container.querySelector(".receipt-medallion-review")?.textContent).toContain(
      "Review",
    );
    expect(container.querySelector(".receipt-summary")?.textContent).toContain(
      "1 item to review",
    );

    await act(async () => {
      root.unmount();
    });
  });

  it("surfaces Export With Review from the app shell when selected source checks warn", async () => {
    mocks.tm = {
      ...baseTrackMasterState(),
      mode: "track",
      selectedTrackId: track.id,
      selectedTrack: track,
      selectedAnalysis: hotAnalysis,
      selectedSettings: settings,
    };

    const { container, root } = await renderApp();

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    );

    expect(exportButton).toBeTruthy();

    await act(async () => {
      root.unmount();
    });
  });
});
