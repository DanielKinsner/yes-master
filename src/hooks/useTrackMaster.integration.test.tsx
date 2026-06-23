import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  vi,
  type Mock,
} from "vitest";

import type {
  AnalysisResult,
  AlbumPlan,
  CompressionPlan,
  GuardrailReadout,
  ImportedTrack,
  MasteringSettings,
  ProjectState,
  RenderJob,
  TrackId,
  WaveformPeaks,
} from "../bindings";
import { lastExportDirectory } from "../lib/export-location";
import { SUPPORTED_FORMATS_COPY } from "../lib/supported-formats";
import {
  playbackErrorMessage,
  shouldPushLiveChainForSettingsEdit,
  useTrackMaster,
} from "./useTrackMaster";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const mocks = vi.hoisted(() => {
  const api = {
    importTracks: vi.fn(),
    analyzeTracks: vi.fn(),
    renderTrackPreview: vi.fn(),
    renderTrackMaster: vi.fn(),
    prepareWaveform: vi.fn(),
    runExportChecks: vi.fn(),
    openOutput: vi.fn(),
    saveProject: vi.fn(),
    autosaveSession: vi.fn(),
    loadRecentSession: vi.fn(),
    loadProject: vi.fn(),
    saveUserPreset: vi.fn(),
    listUserPresets: vi.fn(),
    deleteUserPreset: vi.fn(),
    evictSourceProfile: vi.fn(),
    playTrack: vi.fn(),
    playMaster: vi.fn(),
    updateChain: vi.fn(),
    prewarmDecode: vi.fn(),
    pausePlayback: vi.fn(),
    resumePlayback: vi.fn(),
    stopPlayback: vi.fn(),
    seekPlayback: vi.fn(),
    setLoopRegion: vi.fn(),
    guardrailReadout: vi.fn(),
    resolveCompressionPlan: vi.fn(),
    adaptiveCompressionEnabled: vi.fn(),
    planAlbum: vi.fn(),
    renderAlbumPlan: vi.fn(),
  };
  return {
    api,
    onPlaybackTick: vi.fn(),
    onRenderProgress: vi.fn(),
    onLandingStatus: vi.fn(),
    onAnalysisProgress: vi.fn(),
    open: vi.fn(),
    save: vi.fn(),
    onDragDropEvent: vi.fn(),
  };
});

vi.mock("../lib/api", () => ({
  ADAPTIVE_COMPRESSION_GATE_EVENT: "yes-master:adaptive-compression-gate",
  api: mocks.api,
  onPlaybackTick: mocks.onPlaybackTick,
  onRenderProgress: mocks.onRenderProgress,
  onLandingStatus: mocks.onLandingStatus,
  onAnalysisProgress: mocks.onAnalysisProgress,
}));

vi.mock("../lib/tauri-runtime", () => ({
  open: mocks.open,
  save: mocks.save,
  getCurrentWebview: () => ({
    onDragDropEvent: mocks.onDragDropEvent,
  }),
}));

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

function makeTrack(id: string, path: string): ImportedTrack {
  return {
    id,
    path,
    display_name: `${id}.wav`,
    source_format: "wav",
    duration_seconds: 10,
    sample_rate: 44_100,
    channels: 2,
  };
}

function makeProjectState(track: ImportedTrack): ProjectState {
  return {
    schema_version: 1,
    mode: "track",
    tracks: [track],
    track_order: [track.id],
    track_settings: { [track.id]: DEFAULT_SETTINGS },
    album_intent: DEFAULT_SETTINGS,
    track_override_album: [],
    last_saved_iso: "2026-05-17T00:00:00.000Z",
  };
}

function makeWaveform(trackId: string): WaveformPeaks {
  return {
    track_id: trackId,
    channels: [[], []],
    samples_per_pixel: 512,
    total_samples: 0,
    sample_rate: 44_100,
  };
}

function makeAnalysis(trackId: string): AnalysisResult {
  return {
    track_id: trackId,
    lufs_integrated: -14,
    lufs_short_term_max: -10,
    true_peak_dbtp: -1,
    dynamic_range_lu: 8,
    spectral_balance: { low: 0.33, mid: 0.34, high: 0.33 },
    transient_density: 0.5,
    stereo_width: 0.5,
    recommended_universal: DEFAULT_SETTINGS,
    measured_at_iso: "2026-05-17T00:00:00.000Z",
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
}

function makeRenderJob(
  path: string,
  measurementOverrides: Partial<NonNullable<RenderJob["measurements"]>> = {},
): RenderJob {
  return {
    id: "render-1",
    kind: "master",
    target_tracks: ["export-1"],
    status: { status: "done" },
    progress: 1,
    started_at_iso: "2026-05-17T00:00:00.000Z",
    output_paths: [path],
    measurements: {
      lufs_integrated: -14,
      true_peak_dbtp: -1,
      dynamic_range_lu: 8,
      sample_rate: 44_100,
      bit_depth: 24,
      ...measurementOverrides,
    },
  };
}

function makeAlbumPlan(trackIds: string[]): AlbumPlan {
  return {
    title: "Desk Check",
    arc: { kind: "preset", preset: "cinematic" },
    tracks: trackIds.map((trackId, index) => ({
      track_id: trackId,
      position: index,
      role: index === 0 ? "opener" : "closer",
      role_locked: false,
      arc_lufs_offset_db: 0,
      intensity_scale: 1,
    })),
    transitions: trackIds.map(() => ({
      kind: "direct",
      duration_seconds: 0,
    })),
    intensity: 1,
  };
}

function makeCompressionPlan(active: boolean, digest: string): CompressionPlan {
  const band = {
    threshold_db: -10,
    ratio: 1.2,
    density_mult: 0.9,
    threshold_lift_db: active ? 1 : 0,
    ratio_mult: active ? 0.9 : 1,
    adaptive: active,
  };
  return {
    active,
    low: band,
    mid: band,
    high: band,
    reasons: active
      ? [{ code: "low_band_dense", message: "Low band is already dense." }]
      : [],
    guidance: active ? "Low band is already dense." : null,
    digest,
  };
}

function makeGuardrailReadout(active: boolean): GuardrailReadout {
  return {
    active,
    strength: 0.5,
    bright_trim: active ? 0.2 : 0,
    low_trim: 0,
    density_trim: 0,
    width_trim: 0,
    brightness_share: active ? 0.42 : 0.2,
    low_share: 0.3,
    dynamic_range_db: 8,
    bright_deadband: 0.3,
    low_deadband: 0.42,
    width_corr_deadband: 0.5,
    stereo_correlation: 0.8,
    confidence: null,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((innerResolve) => {
    resolve = innerResolve;
  });
  return { promise, resolve };
}

function HookHarness({
  onRender,
}: {
  onRender: (value: ReturnType<typeof useTrackMaster>) => void;
}) {
  onRender(useTrackMaster());
  return null;
}

async function renderHookHarness(): Promise<{
  current: () => ReturnType<typeof useTrackMaster>;
  root: Root;
  container: HTMLDivElement;
}> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  let current: ReturnType<typeof useTrackMaster> | null = null;
  const root = createRoot(container);
  await act(async () => {
    root.render(<HookHarness onRender={(value) => { current = value; }} />);
  });
  return {
    current: () => {
      if (current === null) throw new Error("hook has not rendered");
      return current;
    },
    root,
    container,
  };
}

async function waitFor(
  assertion: () => void,
  timeoutMs = 1500,
): Promise<void> {
  const startedAt = Date.now();
  let lastError: unknown;
  while (Date.now() - startedAt < timeoutMs) {
    try {
      assertion();
      return;
    } catch (error) {
      lastError = error;
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 10));
      });
    }
  }
  throw lastError;
}

function resetApiMocks() {
  for (const fn of Object.values(mocks.api)) {
    (fn as Mock).mockReset();
    (fn as Mock).mockResolvedValue(null);
  }
  mocks.open.mockReset();
  mocks.save.mockReset();
  mocks.onDragDropEvent.mockReset();
  mocks.onPlaybackTick.mockReset();
  mocks.onRenderProgress.mockReset();
  mocks.onLandingStatus.mockReset();
  mocks.onAnalysisProgress.mockReset();

  mocks.api.listUserPresets.mockResolvedValue([]);
  mocks.api.loadRecentSession.mockResolvedValue(null);
  mocks.api.importTracks.mockResolvedValue([]);
  mocks.api.analyzeTracks.mockImplementation(
    (tracks: Array<{ id: TrackId }>) =>
      Promise.resolve((tracks ?? []).map((track) => makeAnalysis(track.id))),
  );
  mocks.api.prepareWaveform.mockImplementation((trackId: string) =>
    Promise.resolve(makeWaveform(trackId)),
  );
  mocks.api.prewarmDecode.mockResolvedValue(null);
  mocks.api.setLoopRegion.mockResolvedValue(null);
  mocks.api.stopPlayback.mockResolvedValue(null);
  mocks.api.playMaster.mockResolvedValue(null);
  mocks.api.updateChain.mockResolvedValue(null);
  mocks.api.guardrailReadout.mockResolvedValue(null);
  mocks.api.resolveCompressionPlan.mockResolvedValue(null);
  mocks.api.adaptiveCompressionEnabled.mockResolvedValue(false);
  mocks.onPlaybackTick.mockResolvedValue(() => {});
  mocks.onRenderProgress.mockResolvedValue(() => {});
  mocks.onLandingStatus.mockResolvedValue(() => {});
  mocks.onAnalysisProgress.mockResolvedValue(() => {});
  mocks.onDragDropEvent.mockResolvedValue(() => {});
}

function installTestLocalStorage() {
  const values = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => {
        values.set(key, String(value));
      },
      removeItem: (key: string) => {
        values.delete(key);
      },
      clear: () => {
        values.clear();
      },
    },
  });
}

beforeEach(() => {
  installTestLocalStorage();
  localStorage.clear();
  resetApiMocks();
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("live-chain push predicate", () => {
  it("pushes direct track edits only when the selected track is loaded as Mastered", () => {
    const trackId = "track-a" as TrackId;

    expect(
      shouldPushLiveChainForSettingsEdit({
        trackId,
        editingAlbumIntent: false,
        loadedTrackId: null,
        loadedKindByTrack: { [trackId]: "master" },
        overrideAlbum: new Set(),
      }),
    ).toBe(true);
    expect(
      shouldPushLiveChainForSettingsEdit({
        trackId,
        editingAlbumIntent: false,
        loadedTrackId: trackId,
        loadedKindByTrack: { [trackId]: "source" },
        overrideAlbum: new Set(),
      }),
    ).toBe(false);
    expect(
      shouldPushLiveChainForSettingsEdit({
        trackId,
        editingAlbumIntent: false,
        loadedTrackId: trackId,
        loadedKindByTrack: {},
        overrideAlbum: new Set(),
      }),
    ).toBe(true);
  });

  it("pushes album-intent edits for loaded following tracks but skips overrides", () => {
    const first = "album-a" as TrackId;
    const second = "album-b" as TrackId;

    expect(
      shouldPushLiveChainForSettingsEdit({
        trackId: first,
        editingAlbumIntent: true,
        loadedTrackId: first,
        loadedKindByTrack: { [first]: "master", [second]: "source" },
        overrideAlbum: new Set([second]),
      }),
    ).toBe(true);
    expect(
      shouldPushLiveChainForSettingsEdit({
        trackId: second,
        editingAlbumIntent: true,
        loadedTrackId: second,
        loadedKindByTrack: { [second]: "master" },
        overrideAlbum: new Set([second]),
      }),
    ).toBe(false);
  });
});

describe("playback error messages", () => {
  it("turns Mastered preview timeouts into a recoverable user message", () => {
    expect(playbackErrorMessage("audio thread reply timeout", "master")).toBe(
      "Mastered preview is still preparing for this file. Wait a moment and try Mastered again, or export the master directly.",
    );
    expect(
      playbackErrorMessage(
        "Mastered preview did not become ready within 15 seconds; the file may still be decoding",
        "master",
      ),
    ).toBe(
      "Mastered preview is still preparing for this file. Wait a moment and try Mastered again, or export the master directly.",
    );
  });

  it("leaves source playback errors untouched", () => {
    expect(playbackErrorMessage("audio thread reply timeout", "source")).toBe(
      "audio thread reply timeout",
    );
  });
});

describe("useTrackMaster integration dispatches", () => {
  it("defaults export LUFS preview off so live settings edits stay responsive", async () => {
    const harness = await renderHookHarness();

    expect(harness.current().transport.exportLufsPreview).toBe(false);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("defaults a fresh track's Volume Match OFF (non-negotiable: VM off by default)", async () => {
    const harness = await renderHookHarness();

    // With no track edited, selectedSettings falls back to the production
    // DEFAULT_SETTINGS literal; this pins volume_match:false there so a flip of
    // the default would be caught.
    expect(harness.current().selectedSettings.volume_match).toBe(false);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("keeps Volume Match and Preview LUFS mutually exclusive", async () => {
    const harness = await renderHookHarness();

    await act(async () => {
      harness.current().setVolumeMatch(true);
    });
    expect(harness.current().transport.volumeMatch).toBe(true);
    expect(harness.current().transport.exportLufsPreview).toBe(false);

    await act(async () => {
      harness.current().setExportLufsPreview(true);
    });
    expect(harness.current().transport.volumeMatch).toBe(false);
    expect(harness.current().transport.exportLufsPreview).toBe(true);

    await act(async () => {
      harness.current().setVolumeMatch(true);
    });
    expect(harness.current().transport.volumeMatch).toBe(true);
    expect(harness.current().transport.exportLufsPreview).toBe(false);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("prewarms the auto-selected track when restoring the recent session", async () => {
    const track = makeTrack("restored-1", "C:/audio/restored.wav");
    mocks.api.loadRecentSession.mockResolvedValue(makeProjectState(track));

    const harness = await renderHookHarness();

    await waitFor(() => {
      expect(mocks.api.prewarmDecode).toHaveBeenCalledWith(track.path);
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("prewarms the first imported track when import auto-selects it", async () => {
    const track = makeTrack("imported-1", "C:/audio/imported.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });

    expect(mocks.api.prewarmDecode).toHaveBeenCalledWith(track.path);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("auto-selects a fresh import and resets stale playing/meter state from the prior track", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const first = makeTrack("import-reset-first", "C:/audio/first.wav");
    const second = makeTrack("import-reset-second", "C:/audio/second.wav");
    mocks.api.importTracks.mockResolvedValueOnce([first]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(first.id);
      expect(playbackHandler).toBeDefined();
    });

    // Establish "playing" transport plus live (non-sentinel) meter readings on
    // the first track, as a real playback tick would.
    await act(async () => {
      playbackHandler?.({
        track_id: first.id,
        position_sec: 4,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -6,
        gr_low_db: -3,
        gr_mid_db: -4,
        gr_high_db: -2,
        lufs_momentary: -11,
        lufs_integrated: -12,
        spectrum_db: [-20, -18, -16],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.isPlaying).toBe(true);
      expect(harness.current().transport.peakDbfs).toBe(-6);
    });

    // Import a second track. It must jump to the new import AND clear the stale
    // playing/meter state so no phantom transport or moving meters leak.
    mocks.api.importTracks.mockResolvedValueOnce([second]);
    await act(async () => {
      await harness.current().importFiles([second.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(second.id);
    });

    const transport = harness.current().transport;
    expect(transport.isPlaying).toBe(false);
    expect(transport.currentTimeSec).toBe(0);
    expect(transport.loop).toBe(false);
    expect(transport.peakDbfs).toBe(-120);
    expect(transport.peakLeftDbfs).toBe(-120);
    expect(transport.peakRightDbfs).toBe(-120);
    expect(transport.compressionGr).toEqual({ low: -120, mid: -120, high: -120 });
    expect(transport.lufsMomentary).toBe(-120);
    expect(transport.lufsIntegrated).toBe(-120);
    expect(transport.spectrumDb).toEqual([]);

    // A late tick for the PREVIOUS track (the backend's 50ms loop can emit one
    // right after import) must be rejected by the selection guard — not
    // re-paint the old track's playing state. importFiles syncs
    // selectedTrackIdRef synchronously so the guard sees the new selection even
    // before React commits.
    await act(async () => {
      playbackHandler?.({
        track_id: first.id,
        position_sec: 9,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -4,
        gr_low_db: -5,
        gr_mid_db: -6,
        gr_high_db: -3,
        lufs_momentary: -9,
        lufs_integrated: -10,
        spectrum_db: [-12, -10, -8],
      });
    });
    const afterStaleTick = harness.current().transport;
    expect(afterStaleTick.isPlaying).toBe(false);
    expect(afterStaleTick.peakDbfs).toBe(-120);
    expect(afterStaleTick.lufsIntegrated).toBe(-120);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("selectTrack resets stale playing and meter state from the prior track", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const first = makeTrack("select-reset-first", "C:/audio/select-first.wav");
    const second = makeTrack("select-reset-second", "C:/audio/select-second.wav");
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(first.id);
      expect(playbackHandler).toBeDefined();
    });

    await act(async () => {
      playbackHandler?.({
        track_id: first.id,
        position_sec: 5,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -5,
        gr_low_db: -3,
        gr_mid_db: -4,
        gr_high_db: -2,
        lufs_momentary: -10,
        lufs_integrated: -11,
        spectrum_db: [-24, -20, -16],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.peakDbfs).toBe(-5);
    });

    await act(async () => {
      harness.current().selectTrack(second.id);
    });

    expect(harness.current().selectedTrackId).toBe(second.id);
    const transport = harness.current().transport;
    expect(transport.isPlaying).toBe(false);
    expect(transport.currentTimeSec).toBe(0);
    expect(transport.loop).toBe(false);
    expect(transport.peakDbfs).toBe(-120);
    expect(transport.peakLeftDbfs).toBe(-120);
    expect(transport.peakRightDbfs).toBe(-120);
    expect(transport.compressionGr).toEqual({ low: -120, mid: -120, high: -120 });
    expect(transport.lufsMomentary).toBe(-120);
    expect(transport.lufsIntegrated).toBe(-120);
    expect(transport.spectrumDb).toEqual([]);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("selectTrack rejects stale old-track ticks before React commits selection", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const first = makeTrack("select-race-first", "C:/audio/race-first.wav");
    const second = makeTrack("select-race-second", "C:/audio/race-second.wav");
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(first.id);
      expect(playbackHandler).toBeDefined();
    });

    await act(async () => {
      harness.current().selectTrack(second.id);
      playbackHandler?.({
        track_id: first.id,
        position_sec: 9,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -4,
        gr_low_db: -5,
        gr_mid_db: -6,
        gr_high_db: -3,
        lufs_momentary: -9,
        lufs_integrated: -10,
        spectrum_db: [-12, -10, -8],
      });
    });

    expect(harness.current().selectedTrackId).toBe(second.id);
    const transport = harness.current().transport;
    expect(transport.isPlaying).toBe(false);
    expect(transport.currentTimeSec).toBe(0);
    expect(transport.peakDbfs).toBe(-120);
    expect(transport.lufsIntegrated).toBe(-120);
    expect(transport.spectrumDb).toEqual([]);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("surfaces unsupported drop feedback when every dropped file is rejected", async () => {
    let dragDropHandler:
      | ((event: { payload: { type: "drop"; paths: string[] } }) => void)
      | undefined;
    mocks.onDragDropEvent.mockImplementation((handler) => {
      dragDropHandler = handler;
      return Promise.resolve(() => {});
    });
    const harness = await renderHookHarness();
    await waitFor(() => {
      expect(dragDropHandler).toBeDefined();
    });

    await act(async () => {
      dragDropHandler?.({
        payload: { type: "drop", paths: ["C:/video/a.mp4"] },
      });
    });

    expect(mocks.api.importTracks).not.toHaveBeenCalled();
    expect(harness.current().error).toContain("MP4");
    expect(harness.current().error).toContain(SUPPORTED_FORMATS_COPY);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("evicts backend adaptive profile state when removing a track", async () => {
    const track = makeTrack("remove-1", "C:/audio/remove.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      harness.current().removeTrack(track.id);
    });

    expect(mocks.api.evictSourceProfile).toHaveBeenCalledWith(track.id);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("stops playback when removing the loaded track", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const track = makeTrack("remove-playing-1", "C:/audio/playing.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
      expect(playbackHandler).toBeDefined();
    });

    await act(async () => {
      playbackHandler?.({
        track_id: track.id,
        position_sec: 3,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -12,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -14,
        lufs_integrated: -14,
        spectrum_db: [],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.isPlaying).toBe(true);
    });

    mocks.api.stopPlayback.mockClear();
    await act(async () => {
      harness.current().removeTrack(track.id);
    });

    expect(mocks.api.stopPlayback).toHaveBeenCalledTimes(1);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("does not stop playback when removing a non-loaded track", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const loadedTrack = makeTrack("remove-loaded-1", "C:/audio/loaded.wav");
    const otherTrack = makeTrack("remove-other-1", "C:/audio/other.wav");
    mocks.api.importTracks.mockResolvedValue([loadedTrack, otherTrack]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([loadedTrack.path, otherTrack.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
      expect(playbackHandler).toBeDefined();
    });

    await act(async () => {
      playbackHandler?.({
        track_id: loadedTrack.id,
        position_sec: 3,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -12,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -14,
        lufs_integrated: -14,
        spectrum_db: [],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.isPlaying).toBe(true);
    });

    mocks.api.stopPlayback.mockClear();
    await act(async () => {
      harness.current().removeTrack(otherTrack.id);
    });

    expect(mocks.api.stopPlayback).not.toHaveBeenCalled();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("bakes delivery profile defaults into editable Advanced fields and lets Custom inherit them", async () => {
    const track = makeTrack("profile-1", "C:/audio/profile.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      harness.current().setDeliveryProfile("loud-rock");
    });
    expect(harness.current().selectedSettings.delivery_profile).toBe("loud-rock");
    expect(harness.current().selectedSettings.advanced.lufs_offset_db).toBe(-10.5);
    expect(harness.current().selectedSettings.advanced.ceiling_dbtp).toBe(-1);
    expect(harness.current().selectedSettings.advanced.bit_depth).toBe(24);

    await act(async () => {
      harness.current().setDeliveryProfile("cd");
    });
    expect(harness.current().selectedSettings.delivery_profile).toBe("cd");
    expect(harness.current().selectedSettings.advanced.lufs_offset_db).toBe(-14);
    expect(harness.current().selectedSettings.advanced.ceiling_dbtp).toBe(-1);
    expect(harness.current().selectedSettings.advanced.bit_depth).toBe(16);

    await act(async () => {
      harness.current().setDeliveryProfile("custom");
    });
    expect(harness.current().selectedSettings.delivery_profile).toBe("custom");
    expect(harness.current().selectedSettings.advanced.lufs_offset_db).toBe(-14);
    expect(harness.current().selectedSettings.advanced.ceiling_dbtp).toBe(-1);
    expect(harness.current().selectedSettings.advanced.bit_depth).toBe(16);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("routes center and right-rail loudness target edits through one Custom transition", async () => {
    const track = makeTrack("loudness-1", "C:/audio/loudness.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      harness.current().setLoudnessTargetProfile("off");
    });
    expect(harness.current().selectedSettings.delivery_profile).toBe("custom");
    expect(harness.current().selectedSettings.advanced.lufs_offset_db).toBeNull();

    await act(async () => {
      harness.current().setDeliveryProfile("streaming-universal");
    });
    expect(harness.current().selectedSettings.delivery_profile).toBe(
      "streaming-universal",
    );

    await act(async () => {
      harness.current().setLoudnessTarget(-12);
    });
    expect(harness.current().selectedSettings.delivery_profile).toBe("custom");
    expect(harness.current().selectedSettings.advanced.lufs_offset_db).toBe(-12);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("prewarms the first track when opening a project from disk", async () => {
    const track = makeTrack("project-1", "C:/audio/project.wav");
    mocks.open.mockResolvedValue("C:/projects/test.ams.json");
    mocks.api.loadProject.mockResolvedValue(makeProjectState(track));
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().openProjectFromDisk();
    });

    expect(mocks.api.prewarmDecode).toHaveBeenCalledWith(track.path);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("reports project save success and cancellation without using the error channel", async () => {
    mocks.save.mockResolvedValue("C:/projects/release.ams.json");
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().saveProjectAs();
    });

    expect(mocks.api.saveProject).toHaveBeenCalledWith(
      "C:/projects/release.ams.json",
      expect.objectContaining({ schema_version: 1 }),
    );
    expect(harness.current().projectFeedback).toEqual({
      tone: "ok",
      message: "Project saved to release.ams.json.",
    });
    expect(harness.current().error).toBeNull();

    mocks.save.mockResolvedValue(null);
    mocks.api.saveProject.mockClear();
    await act(async () => {
      await harness.current().saveProjectAs();
    });

    expect(mocks.api.saveProject).not.toHaveBeenCalled();
    expect(harness.current().projectFeedback).toEqual({
      tone: "info",
      message: "Save project canceled.",
    });
    expect(harness.current().error).toBeNull();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("round-trips album panel choices through Save and Open Project", async () => {
    const track = makeTrack("album-project", "C:/audio/album-project.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.save.mockResolvedValue("C:/projects/album.ams.json");
    const firstHarness = await renderHookHarness();

    await act(async () => {
      await firstHarness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(firstHarness.current().selectedTrackId).toBe(track.id);
    });
    await act(async () => {
      firstHarness.current().setMode("album");
      firstHarness.current().setAlbumArc("club-peak");
      firstHarness.current().setAlbumIntensity(1.65);
      firstHarness.current().setAlbumTitle("Late Night Sequence");
      firstHarness.current().setAlbumSampleRate(96_000);
      firstHarness.current().setAlbumBitDepth(16);
    });

    await act(async () => {
      await firstHarness.current().saveProjectAs();
    });
    const savedState = mocks.api.saveProject.mock.calls.at(-1)?.[1] as ProjectState;
    expect(savedState).toMatchObject({
      mode: "album",
      album_arc_kind: "club-peak",
      album_intensity: 1.65,
      album_title: "Late Night Sequence",
      album_sample_rate: 96_000,
      album_bit_depth: 16,
    });
    await act(async () => {
      firstHarness.root.unmount();
    });

    mocks.open.mockResolvedValue("C:/projects/album.ams.json");
    mocks.api.loadProject.mockResolvedValue(savedState);
    const secondHarness = await renderHookHarness();

    await act(async () => {
      await secondHarness.current().openProjectFromDisk();
    });

    expect(secondHarness.current().mode).toBe("album");
    expect(secondHarness.current().albumArcKind).toBe("club-peak");
    expect(secondHarness.current().albumIntensity).toBe(1.65);
    expect(secondHarness.current().albumTitle).toBe("Late Night Sequence");
    expect(secondHarness.current().albumSampleRate).toBe(96_000);
    expect(secondHarness.current().albumBitDepth).toBe(16);

    await act(async () => {
      secondHarness.root.unmount();
    });
  });

  it("reports open cancellation without loading or mutating project state", async () => {
    mocks.open.mockResolvedValue(null);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().openProjectFromDisk();
    });

    expect(mocks.api.loadProject).not.toHaveBeenCalled();
    expect(harness.current().projectFeedback).toEqual({
      tone: "info",
      message: "Open project canceled.",
    });
    expect(harness.current().error).toBeNull();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("surfaces open-project recovery failures as project feedback", async () => {
    const track = makeTrack("project-2", "C:/audio/moved.wav");
    mocks.open.mockResolvedValue("C:/projects/moved.ams.json");
    mocks.api.loadProject.mockResolvedValue(makeProjectState(track));
    mocks.api.analyzeTracks.mockRejectedValue(new Error("missing source"));
    mocks.api.prepareWaveform.mockRejectedValue(new Error("missing source"));
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const harness = await renderHookHarness();

    try {
      await act(async () => {
        await harness.current().openProjectFromDisk();
      });

      expect(harness.current().projectFeedback).toEqual({
        tone: "warn",
        message:
          "Project opened from moved.ams.json; analysis could not be refreshed; 1 waveform could not be rebuilt.",
      });
      expect(harness.current().error).toBeNull();
    } finally {
      warn.mockRestore();
    }
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("surfaces partial-success open-project analysis gaps as project feedback", async () => {
    const first = makeTrack("project-partial-ok", "C:/audio/project-partial-ok.wav");
    const second = makeTrack("project-partial-missing", "C:/audio/project-partial-missing.wav");
    const state = {
      ...makeProjectState(first),
      tracks: [first, second],
      track_order: [first.id, second.id],
      track_settings: {
        [first.id]: DEFAULT_SETTINGS,
        [second.id]: DEFAULT_SETTINGS,
      },
    };
    mocks.open.mockResolvedValue("C:/projects/partial.ams.json");
    mocks.api.loadProject.mockResolvedValue(state);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(first.id)]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().openProjectFromDisk();
    });

    expect(harness.current().selectedAnalysis?.track_id).toBe(first.id);
    await act(async () => {
      harness.current().selectTrack(second.id);
    });
    expect(harness.current().selectedAnalysis).toBeUndefined();
    expect(harness.current().projectFeedback).toEqual({
      tone: "warn",
      message:
        "Project opened from partial.ams.json; 1 track still needs analysis: project-partial-missing.wav.",
    });
    expect(harness.current().error).toBeNull();

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("reanalyzes a failed-restore track and merges the recovered analysis", async () => {
    const track = makeTrack("project-retry", "C:/audio/moved-retry.wav");
    mocks.open.mockResolvedValue("C:/projects/moved-retry.ams.json");
    mocks.api.loadProject.mockResolvedValue(makeProjectState(track));
    mocks.api.analyzeTracks.mockRejectedValueOnce(new Error("missing source"));
    mocks.api.prepareWaveform.mockRejectedValueOnce(new Error("missing source"));
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const harness = await renderHookHarness();

    try {
      await act(async () => {
        await harness.current().openProjectFromDisk();
      });
      expect(harness.current().selectedTrackId).toBe(track.id);
      expect(harness.current().selectedAnalysis).toBeUndefined();

      mocks.api.analyzeTracks.mockClear();
      mocks.api.prepareWaveform.mockClear();
      mocks.api.analyzeTracks.mockResolvedValueOnce([makeAnalysis(track.id)]);
      mocks.api.prepareWaveform.mockResolvedValueOnce(makeWaveform(track.id));

      await act(async () => {
        await harness.current().reanalyzeTrack(track.id);
      });

      expect(mocks.api.analyzeTracks).toHaveBeenCalledWith(
        [{ id: track.id, path: track.path }],
        expect.stringMatching(/^analysis-\d+-\d+$/),
      );
      expect(mocks.api.prepareWaveform).toHaveBeenCalledWith(track.id, track.path, 1200);
      expect(harness.current().selectedAnalysis?.track_id).toBe(track.id);
      expect(harness.current().isAnalyzing).toBe(false);
    } finally {
      warn.mockRestore();
    }
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("keeps moved-project recovery feedback visible when reanalysis still cannot find the source", async () => {
    const track = makeTrack("project-retry-missing", "C:/audio/still-missing.wav");
    mocks.open.mockResolvedValue("C:/projects/still-missing.ams.json");
    mocks.api.loadProject.mockResolvedValue(makeProjectState(track));
    mocks.api.analyzeTracks.mockRejectedValue(new Error("missing source"));
    mocks.api.prepareWaveform.mockRejectedValue(new Error("missing source"));
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const harness = await renderHookHarness();

    try {
      await act(async () => {
        await harness.current().openProjectFromDisk();
      });
      const recoveryFeedback = harness.current().projectFeedback;
      expect(recoveryFeedback).toEqual({
        tone: "warn",
        message:
          "Project opened from still-missing.ams.json; analysis could not be refreshed; 1 waveform could not be rebuilt.",
      });

      await act(async () => {
        await harness.current().reanalyzeTrack(track.id);
      });

      expect(harness.current().projectFeedback).toEqual(recoveryFeedback);
      expect(harness.current().error).toContain("missing source");
      expect(harness.current().selectedAnalysis).toBeUndefined();
    } finally {
      warn.mockRestore();
    }
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("marks a recovered preview stale when reanalysis loses the source again", async () => {
    const track = makeTrack("reanalyze-stale", "C:/audio/reanalyze-stale.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.api.renderTrackPreview.mockResolvedValue(makeRenderJob("C:/out/reanalyze-stale.wav"));
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });
    await act(async () => {
      await harness.current().updatePreview();
    });
    expect(harness.current().previewStale).toBe(false);

    mocks.api.analyzeTracks.mockRejectedValueOnce(new Error("missing source"));
    await act(async () => {
      await harness.current().reanalyzeTrack(track.id);
    });

    expect(harness.current().error).toContain("missing source");
    expect(harness.current().previewStale).toBe(true);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("surfaces partial-success import analysis gaps in the error channel", async () => {
    const first = makeTrack("import-partial-ok", "C:/audio/import-partial-ok.wav");
    const second = makeTrack("import-partial-missing", "C:/audio/import-partial-missing.wav");
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(first.id)]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });

    expect(harness.current().selectedAnalysis?.track_id).toBe(first.id);
    await act(async () => {
      harness.current().selectTrack(second.id);
    });
    expect(harness.current().selectedAnalysis).toBeUndefined();
    expect(harness.current().error).toBe(
      "1 track still needs analysis: import-partial-missing.wav.",
    );

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("keeps the current compression plan visible while refetching for the same track", async () => {
    const track = makeTrack("compression-plan-refetch", "C:/audio/plan-refetch.wav");
    const firstPlan = makeCompressionPlan(true, "first plan");
    const nextPlan = makeCompressionPlan(true, "next plan");
    const pendingPlan = deferred<CompressionPlan>();
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.api.resolveCompressionPlan.mockResolvedValue(firstPlan);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().compressionPlan).toEqual(firstPlan);
    });

    const callsBefore = mocks.api.resolveCompressionPlan.mock.calls.length;
    mocks.api.resolveCompressionPlan.mockReturnValue(pendingPlan.promise);
    await act(async () => {
      harness.current().setIntensity(0.73);
    });
    await waitFor(() => {
      expect(mocks.api.resolveCompressionPlan.mock.calls.length).toBeGreaterThan(callsBefore);
    });

    expect(harness.current().compressionPlan).toEqual(firstPlan);

    await act(async () => {
      pendingPlan.resolve(nextPlan);
      await pendingPlan.promise;
    });
    await waitFor(() => {
      expect(harness.current().compressionPlan).toEqual(nextPlan);
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("fetches adaptive readouts when the adaptive compression gate reader is unavailable", async () => {
    const track = makeTrack("adaptive-gate-unavailable", "C:/audio/gate-unavailable.wav");
    const readout = makeGuardrailReadout(true);
    const plan = makeCompressionPlan(false, "fallback gate");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.api.adaptiveCompressionEnabled.mockResolvedValue(null);
    mocks.api.guardrailReadout.mockResolvedValue(readout);
    mocks.api.resolveCompressionPlan.mockResolvedValue(plan);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });

    await waitFor(() => {
      expect(mocks.api.guardrailReadout).toHaveBeenCalledWith(
        expect.any(Object),
        track.id,
        false,
      );
      expect(mocks.api.resolveCompressionPlan).toHaveBeenCalledWith(
        expect.any(Object),
        track.id,
        false,
      );
    });
    await waitFor(() => {
      expect(harness.current().guardrailReadout).toEqual(readout);
      expect(harness.current().compressionPlan).toEqual(plan);
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("clears stale guardrail readout when the latest backend read returns null", async () => {
    const track = makeTrack("guardrail-null", "C:/audio/guardrail-null.wav");
    const readout = makeGuardrailReadout(true);
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.api.guardrailReadout.mockResolvedValue(readout);
    mocks.api.resolveCompressionPlan.mockResolvedValue(null);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().guardrailReadout).toEqual(readout);
    });

    const callsBefore = mocks.api.guardrailReadout.mock.calls.length;
    mocks.api.guardrailReadout.mockResolvedValue(null);
    await act(async () => {
      harness.current().setIntensity(0.66);
    });

    await waitFor(() => {
      expect(mocks.api.guardrailReadout.mock.calls.length).toBeGreaterThan(callsBefore);
      expect(harness.current().guardrailReadout).toBeNull();
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("refetches the compression plan when the adaptive compression gate changes", async () => {
    const track = makeTrack("compression-plan-gate", "C:/audio/plan-gate.wav");
    const gateOffPlan = makeCompressionPlan(false, "gate off");
    const gateOnPlan = makeCompressionPlan(true, "gate on");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.api.resolveCompressionPlan.mockResolvedValue(gateOffPlan);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().compressionPlan).toEqual(gateOffPlan);
    });

    const callsBefore = mocks.api.resolveCompressionPlan.mock.calls.length;
    mocks.api.resolveCompressionPlan.mockResolvedValue(gateOnPlan);
    await act(async () => {
      window.dispatchEvent(
        new CustomEvent("yes-master:adaptive-compression-gate", { detail: true }),
      );
    });

    await waitFor(() => {
      expect(mocks.api.resolveCompressionPlan.mock.calls.length).toBeGreaterThan(callsBefore);
    });
    await waitFor(() => {
      expect(harness.current().compressionPlan).toEqual(gateOnPlan);
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("uses the error channel for unsupported project schemas", async () => {
    const track = makeTrack("project-3", "C:/audio/project.wav");
    mocks.open.mockResolvedValue("C:/projects/old.ams.json");
    mocks.api.loadProject.mockResolvedValue({
      ...makeProjectState(track),
      schema_version: 99,
    });
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().openProjectFromDisk();
    });

    expect(harness.current().error).toBe("Unsupported project schema: v99");
    expect(harness.current().projectFeedback).toBeNull();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("dispatches updateChain with the current export-LUFS preview flag", async () => {
    const track = makeTrack("mastered-1", "C:/audio/mastered.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await waitFor(() => {
      expect(harness.current().transport.playbackKind).toBe("master");
    });

    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setExportLufsPreview(false);
    });

    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalledWith(
        expect.objectContaining({ volume_match: false }),
        false,
        false, // album flag — Track mode
      );
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("requires analysis before Mastered audition or audit render, but still allows Original playback", async () => {
    const track = makeTrack("unanalyzed-1", "C:/audio/unanalyzed.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });
    expect(harness.current().selectedAnalysis).toBeUndefined();

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    expect(harness.current().transport.playbackKind).toBe("source");
    expect(harness.current().error).toBe(
      "Analyze this track before using Mastered playback.",
    );
    expect(mocks.api.playMaster).not.toHaveBeenCalled();

    await act(async () => {
      await harness.current().updatePreview();
    });
    expect(mocks.api.renderTrackPreview).not.toHaveBeenCalled();
    expect(harness.current().error).toBe(
      "Analyze this track before using Mastered playback.",
    );

    await act(async () => {
      await harness.current().togglePlay();
    });
    expect(mocks.api.playTrack).toHaveBeenCalledWith(track.id, track.path, 0);
    expect(mocks.api.playMaster).not.toHaveBeenCalled();

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("routes the export-LUFS preview live push through the EFFECTIVE landing (Standard WYSIWYG survives toggling the user flag off)", async () => {
    // Guard: in Standard (forceWysiwyg on, Volume Match off), toggling the
    // user-facing Preview LUFS flag OFF must NOT drop the live landing — the
    // forced-WYSIWYG flag still demands landing=true. setExportLufsPreview
    // must push effectivePreviewLanding() (= true here), not the raw `on`
    // (= false). Pushing raw `on` would silently un-land Standard playback.
    const track = makeTrack("eff-landing-1", "C:/audio/eff-landing.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await waitFor(() => {
      expect(harness.current().transport.playbackKind).toBe("master");
    });

    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    // Enter Standard: forceWysiwyg on (Volume Match stays off). This itself
    // fires a re-land push; let it fully drain before clearing so the only
    // push we assert on below is the one from setExportLufsPreview (both
    // sites carry identical settings, so we discriminate by timing, not args).
    await act(async () => {
      harness.current().setForceWysiwyg(true);
    });
    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalled();
    });

    mocks.api.updateChain.mockClear();
    // Drive the USER toggle OFF. Effective landing is still true (forced).
    await act(async () => {
      harness.current().setExportLufsPreview(false);
    });

    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalled();
    });
    // The push from setExportLufsPreview must carry the EFFECTIVE landing
    // (true — forced WYSIWYG), NOT the raw `on` (false). Assert on the most
    // recent call so a stray coalesced earlier push can't mask a regression.
    expect(mocks.api.updateChain).toHaveBeenLastCalledWith(
      expect.objectContaining({ volume_match: false }),
      true, // EFFECTIVE landing — forced WYSIWYG, not the raw `on` (false)
      false, // album flag — Track mode
    );
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("reflects a Track<->Album mode switch mid-Mastered-audition in the next updateChain (F2)", async () => {
    // F2 regression: switching mode is a bare state change that does NOT
    // re-prime playback, and update_chain previously reused the album flag
    // cached at the last playMaster. A live edit after the switch must carry
    // the CURRENT mode so the backend resolves album audition as non-adaptive
    // (album=true) and Track audition as adaptive again on the way back.
    const track = makeTrack("mode-switch-1", "C:/audio/mode-switch.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    // Load + play Mastered in Track mode (backend caches album=false).
    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    // Switch to Album mode WITHOUT re-priming, then make a live edit.
    await act(async () => {
      harness.current().setMode("album");
    });
    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setIntensity(0.42);
    });
    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalledWith(
        expect.objectContaining({ intensity: 0.42 }),
        expect.any(Boolean),
        true, // album flag follows the CURRENT mode, not the stale play cache
      );
    });

    // Switch back to Track mode and edit again: adaptive context restored.
    await act(async () => {
      harness.current().setMode("track");
    });
    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setIntensity(0.43);
    });
    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalledWith(
        expect.objectContaining({ intensity: 0.43 }),
        expect.any(Boolean),
        false,
      );
    });

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("pushes export-LUFS preview to the live album master when a following track is selected", async () => {
    // Regression for the hardcoded editingAlbumIntent:false in
    // setExportLufsPreview. In album mode, track A plays as the album master
    // while a *different* following track (B) is selected. Toggling export-LUFS
    // preview must still reach the live chain via the album-intent branch —
    // the per-track branch can't see B (it isn't the loaded track).
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const trackA = makeTrack("album-master-a", "C:/audio/a.wav");
    const trackB = makeTrack("album-follow-b", "C:/audio/b.wav");
    mocks.api.importTracks.mockResolvedValue([trackA, trackB]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([trackA.path, trackB.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(trackA.id);
    });

    await act(async () => {
      harness.current().setMode("album");
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    // Backend reports track A loaded and playing as the album master.
    await act(async () => {
      playbackHandler?.({
        track_id: trackA.id,
        position_sec: 3,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -12,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -14,
        lufs_integrated: -14,
        spectrum_db: [],
      });
    });

    // Select the following track B (not overriding the album intent).
    await act(async () => {
      harness.current().selectTrack(trackB.id);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(trackB.id);
    });

    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setExportLufsPreview(true);
    });

    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalled();
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("does not dispatch updateChain for direct edits while Original playback is loaded", async () => {
    const track = makeTrack("source-live-1", "C:/audio/source-live.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playTrack).toHaveBeenCalled();
    });

    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setIntensity(0.62);
      await new Promise((resolve) => setTimeout(resolve, 30));
    });

    expect(mocks.api.updateChain).not.toHaveBeenCalled();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("surfaces Mastered preview timeout as recoverable playback guidance", async () => {
    const track = makeTrack("long-master-1", "C:/audio/long-master.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.playMaster.mockRejectedValue(new Error("audio thread reply timeout"));
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });

    expect(harness.current().error).toBe(
      "Mastered preview is still preparing for this file. Wait a moment and try Mastered again, or export the master directly.",
    );
    expect(mocks.api.playMaster).toHaveBeenCalled();
    expect(harness.current().transport.isPlaying).toBe(false);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("estimates the live playhead when switching original/mastered", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const nowSpy = vi.spyOn(Date, "now");
    const track = makeTrack("switch-1", "C:/audio/switch.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    nowSpy.mockReturnValue(1_000);
    await act(async () => {
      playbackHandler?.({
        track_id: track.id,
        position_sec: 6,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -12,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -14,
        lufs_integrated: -14,
        spectrum_db: [],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.currentTimeSec).toBe(6);
    });

    mocks.api.playMaster.mockClear();
    nowSpy.mockReturnValue(1_250);
    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });

    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });
    expect(mocks.api.playMaster.mock.calls.at(-1)?.[3]).toBeCloseTo(6.25, 3);

    nowSpy.mockRestore();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("does not autoplay on a paused kind switch and resumes at the same playhead", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const track = {
      ...makeTrack("paused-switch-1", "C:/audio/paused switch.wav"),
      duration_seconds: 120,
    };
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      playbackHandler?.({
        track_id: track.id,
        position_sec: 42,
        is_playing: false,
        is_loaded: true,
        peak_dbfs: -120,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -120,
        lufs_integrated: -120,
        spectrum_db: [],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.currentTimeSec).toBe(42);
    });

    mocks.api.playMaster.mockClear();
    mocks.api.playTrack.mockClear();
    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });

    expect(harness.current().transport.playbackKind).toBe("master");
    expect(harness.current().transport.isPlaying).toBe(false);
    expect(mocks.api.playMaster).not.toHaveBeenCalled();
    expect(mocks.api.playTrack).not.toHaveBeenCalled();

    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });
    expect(mocks.api.playMaster.mock.calls.at(-1)?.[3]).toBeCloseTo(42, 1);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("resumes instead of restarting when paused in the final half-second", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const track = {
      ...makeTrack("paused-switch-end", "C:/audio/paused switch end.wav"),
      duration_seconds: 120,
    };
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      playbackHandler?.({
        track_id: track.id,
        position_sec: 119.75,
        is_playing: false,
        is_loaded: true,
        peak_dbfs: -120,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -120,
        lufs_integrated: -120,
        spectrum_db: [],
      });
    });
    await waitFor(() => {
      expect(harness.current().transport.currentTimeSec).toBe(119.75);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });

    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });
    expect(mocks.api.playMaster.mock.calls.at(-1)?.[3]).toBeCloseTo(119.75, 3);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("resumes a loaded sub-half-second source from zero instead of reloading it", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const track = {
      ...makeTrack("short-loaded", "C:/audio/short-loaded.wav"),
      duration_seconds: 0.4,
    };
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });
    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playTrack).toHaveBeenCalledWith(track.id, track.path, 0);
    });
    await act(async () => {
      playbackHandler?.({
        track_id: track.id,
        position_sec: 0,
        is_playing: false,
        is_loaded: true,
        peak_dbfs: -120,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -120,
        lufs_integrated: -120,
        spectrum_db: [],
      });
    });

    mocks.api.playTrack.mockClear();
    await act(async () => {
      await harness.current().togglePlay();
    });

    expect(mocks.api.playTrack).not.toHaveBeenCalled();
    expect(mocks.api.resumePlayback).toHaveBeenCalled();

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("honors enabled LUFS preview on subsequent live settings edits", async () => {
    const track = makeTrack("live-preview-1", "C:/audio/live preview.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    await act(async () => {
      harness.current().setExportLufsPreview(true);
    });

    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setIntensity(0.72);
    });
    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalledWith(
        expect.objectContaining({ intensity: 0.72 }),
        true,
        false, // album flag — Track mode
      );
    });

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("coalesces rapid live-edit updateChain calls into a single latest-wins IPC", async () => {
    // Fix B: sendUpdateChain is rAF-gated single-in-flight. A burst of
    // synchronous setIntensity calls within one frame must collapse to
    // exactly one api.updateChain call carrying the LAST intensity —
    // not three calls, not the first value.
    const track = makeTrack("coalesce-1", "C:/audio/coalesce.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().setIntensity(0.11);
      harness.current().setIntensity(0.55);
      harness.current().setIntensity(0.93);
    });

    // Wait for the rAF + microtask flush to actually call updateChain.
    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalled();
    });
    // Exactly one IPC for the burst — latest wins.
    expect(mocks.api.updateChain).toHaveBeenCalledTimes(1);
    expect(mocks.api.updateChain).toHaveBeenLastCalledWith(
      expect.objectContaining({ intensity: 0.93 }),
      expect.any(Boolean),
      expect.any(Boolean), // album flag
    );

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("asks where to save a track master and passes that path to render", async () => {
    const track = makeTrack("export-1", "C:/audio/export source.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue("/Users/daniel/Desktop/exported-master");
    mocks.api.renderTrackMaster.mockResolvedValue(
      makeRenderJob("/Users/daniel/Desktop/exported-master.wav"),
    );
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().exportMaster();
    });

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "export-1__master.wav",
      filters: [{ name: "WAV audio", extensions: ["wav"] }],
    });
    expect(mocks.api.renderTrackMaster).toHaveBeenCalledWith(
      track.id,
      track.path,
      DEFAULT_SETTINGS,
      "/Users/daniel/Desktop/exported-master.wav",
    );
    expect(harness.current().lastExportReceipt?.outputPath).toBe(
      "/Users/daniel/Desktop/exported-master.wav",
    );
    expect(lastExportDirectory(localStorage, "track")).toBe("/Users/daniel/Desktop");
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("accepts an existing track master path returned by the save dialog", async () => {
    const track = makeTrack("export-overwrite", "C:/audio/export overwrite.wav");
    const outputPath = "/Users/daniel/Desktop/existing-master.wav";
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue(outputPath);
    mocks.api.renderTrackMaster.mockResolvedValue(makeRenderJob(outputPath));
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().exportMaster();
    });

    expect(mocks.api.renderTrackMaster).toHaveBeenCalledWith(
      track.id,
      track.path,
      DEFAULT_SETTINGS,
      outputPath,
    );
    expect(harness.current().lastExportReceipt?.outputPath).toBe(outputPath);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("sends identical raw settings to preview and export — the backend owns profile injection (B2 WYSIWYG parity)", async () => {
    // B2: the FE no longer injects source_profile. render_track_preview and
    // render_track_master each resolve + inject the backend-derived profile
    // (keyed by track id) via the SAME apply_resolved_profile helper, so WYSIWYG
    // parity is now guaranteed server-side. At the FE boundary we assert the two
    // payloads are identical and carry no FE-injected profile.
    const track = makeTrack("export-adaptive", "C:/audio/adaptive.wav");
    const outputPath = "/Users/daniel/Desktop/adaptive-master.wav";
    const analysis: AnalysisResult = {
      ...makeAnalysis(track.id),
      spectral_balance_6band: {
        sub: 0.1,
        low: 0.25,
        low_mid: 0.2,
        mid: 0.2,
        presence: 0.15,
        air: 0.1,
      },
      dynamic_range_p95_p10_db: 6,
    };
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([analysis]);
    mocks.save.mockResolvedValue(outputPath);
    mocks.api.renderTrackPreview.mockResolvedValue(makeRenderJob(outputPath));
    mocks.api.renderTrackMaster.mockResolvedValue(makeRenderJob(outputPath));
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().updatePreview();
    });
    await act(async () => {
      await harness.current().exportMaster();
    });

    const previewSettings = mocks.api.renderTrackPreview.mock.calls.at(-1)?.[2];
    const exportSettings = mocks.api.renderTrackMaster.mock.calls.at(-1)?.[2];
    // FE no longer injects the profile — that is the backend's job now.
    expect(previewSettings?.advanced?.source_profile ?? null).toBeNull();
    expect(exportSettings?.advanced?.source_profile ?? null).toBeNull();
    // Same settings object reaches both render paths → backend derives the same
    // profile for each (WYSIWYG parity).
    expect(previewSettings).toEqual(exportSettings);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("passes backend adaptive traceability through the export report contract", async () => {
    const track = makeTrack("export-confidence", "C:/audio/confidence.wav");
    const outputPath = "/Users/daniel/Desktop/confidence-master.wav";
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue(outputPath);
    mocks.api.renderTrackMaster.mockResolvedValue(
      makeRenderJob(outputPath, {
        effective_adaptive_strength: 0.5,
        source_profile_digest: "bright 0.50 / low 0.20 / density 0.10 / width 0.00",
        confidence_digest: "bright 0.80 / low 0.10 / density 0.40 / width 0.90",
      }),
    );
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().exportMaster();
    });

    expect(mocks.api.runExportChecks).toHaveBeenCalledWith(
      expect.objectContaining({
        effective_adaptive_strength: 0.5,
        source_profile_digest: "bright 0.50 / low 0.20 / density 0.10 / width 0.00",
        confidence_digest: "bright 0.80 / low 0.10 / density 0.40 / width 0.90",
      }),
      expect.any(Object),
      expect.any(Object),
    );

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("passes Windows-style picker paths through to track rendering unchanged", async () => {
    const track = makeTrack("export-windows", "C:\\audio\\export windows.wav");
    const outputPath = "C:\\Users\\Dan\\Desktop\\existing-master.wav";
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue(outputPath);
    mocks.api.renderTrackMaster.mockResolvedValue(makeRenderJob(outputPath));
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().exportMaster();
    });

    expect(mocks.api.renderTrackMaster).toHaveBeenCalledWith(
      track.id,
      track.path,
      DEFAULT_SETTINGS,
      outputPath,
    );
    expect(lastExportDirectory(localStorage, "track")).toBe("C:\\Users\\Dan\\Desktop");
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("does not render when the export save dialog is cancelled", async () => {
    const track = makeTrack("export-cancel", "C:/audio/export cancel.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue(null);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().exportMaster();
    });

    expect(mocks.save).toHaveBeenCalled();
    expect(mocks.api.renderTrackMaster).not.toHaveBeenCalled();
    expect(harness.current().isExporting).toBe(false);
    expect(harness.current().lastExportReceipt).toBeNull();
    expect(lastExportDirectory(localStorage, "track")).toBeNull();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("uses the last track export folder as the next save default", async () => {
    const track = makeTrack("export-repeat", "C:/audio/export repeat.wav");
    localStorage.setItem("yes-master:last-track-export-dir", "/Users/daniel/Desktop");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue("/Users/daniel/Desktop/repeated-master.wav");
    mocks.api.renderTrackMaster.mockResolvedValue(
      makeRenderJob("/Users/daniel/Desktop/repeated-master.wav"),
    );
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().exportMaster();
    });

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "/Users/daniel/Desktop/export-repeat__master.wav",
      filters: [{ name: "WAV audio", extensions: ["wav"] }],
    });
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("asks where to save an album plan export and passes that folder to render", async () => {
    const first = makeTrack("album-1", "C:/audio/album one.wav");
    const second = makeTrack("album-2", "C:/audio/album two.wav");
    const plan = makeAlbumPlan([first.id, second.id]);
    const outputDir = "/Users/daniel/Desktop/Album Masters";
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    mocks.open.mockResolvedValue(outputDir);
    mocks.api.planAlbum.mockResolvedValue(plan);
    mocks.api.renderAlbumPlan.mockResolvedValue({
      album_wav_path: `${outputDir}/album_continuous_1.wav`,
      manifest_path: `${outputDir}/manifest.json`,
      tracks: [
        {
          track_id: first.id,
          position: 0,
          output_path: `${outputDir}/album-1__master.wav`,
          measured_lufs: -14,
        },
        {
          track_id: second.id,
          position: 1,
          output_path: `${outputDir}/album-2__master.wav`,
          measured_lufs: -14,
        },
      ],
    });
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
    });

    await act(async () => {
      await harness.current().exportAlbumPlan();
    });

    expect(mocks.open).toHaveBeenCalledWith({
      directory: true,
      defaultPath: undefined,
      multiple: false,
      title: "Choose album export folder",
    });
    expect(mocks.api.renderAlbumPlan).toHaveBeenCalledWith(
      plan,
      expect.arrayContaining([
        expect.objectContaining({
          track_id: first.id,
          source_path: first.path,
        }),
        expect.objectContaining({
          track_id: second.id,
          source_path: second.path,
        }),
      ]),
      outputDir,
    );
    expect(harness.current().albumExportReport?.album_wav_path).toBe(
      `${outputDir}/album_continuous_1.wav`,
    );
    expect(lastExportDirectory(localStorage, "album")).toBe(outputDir);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("renders album-following tracks with album intent rather than stale per-track settings", async () => {
    const first = makeTrack("album-intent-1", "C:/audio/album intent one.wav");
    const second = makeTrack("album-intent-2", "C:/audio/album intent two.wav");
    const plan = makeAlbumPlan([first.id, second.id]);
    const outputDir = "/Users/daniel/Desktop/Album Masters";
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    mocks.open.mockResolvedValue(outputDir);
    mocks.api.planAlbum.mockResolvedValue(plan);
    mocks.api.renderAlbumPlan.mockResolvedValue({
      album_wav_path: `${outputDir}/album_continuous_1.wav`,
      manifest_path: `${outputDir}/manifest.json`,
      tracks: [],
    });
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
    });

    await act(async () => {
      harness.current().setMode("album");
    });
    await act(async () => {
      harness.current().setDeliveryProfile("cd");
    });
    expect(harness.current().followingAlbumIntent).toBe(true);
    expect(harness.current().selectedSettings.delivery_profile).toBe("cd");

    await act(async () => {
      await harness.current().exportAlbumPlan();
    });

    const renderTracks = mocks.api.renderAlbumPlan.mock.calls.at(-1)?.[1] as
      | import("../lib/api").AlbumTrackRenderInput[]
      | undefined;
    expect(renderTracks).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          track_id: first.id,
          settings: expect.objectContaining({
            delivery_profile: "cd",
            advanced: expect.objectContaining({
              lufs_offset_db: -14,
              ceiling_dbtp: -1,
              bit_depth: 16,
            }),
          }),
        }),
        expect.objectContaining({
          track_id: second.id,
          settings: expect.objectContaining({
            delivery_profile: "cd",
            advanced: expect.objectContaining({
              lufs_offset_db: -14,
              ceiling_dbtp: -1,
              bit_depth: 16,
            }),
          }),
        }),
      ]),
    );

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("uses fresh album intent after an album override is toggled back off", async () => {
    const first = makeTrack("album-fresh-1", "C:/audio/album fresh one.wav");
    const second = makeTrack("album-fresh-2", "C:/audio/album fresh two.wav");
    const plan = makeAlbumPlan([first.id, second.id]);
    const outputDir = "/Users/daniel/Desktop/Album Masters";
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    mocks.open.mockResolvedValue(outputDir);
    mocks.api.planAlbum.mockResolvedValue(plan);
    mocks.api.renderAlbumPlan.mockResolvedValue({
      album_wav_path: `${outputDir}/album_continuous_1.wav`,
      manifest_path: `${outputDir}/manifest.json`,
      tracks: [],
    });
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
    });

    await act(async () => {
      harness.current().setMode("album");
    });
    await act(async () => {
      harness.current().setDeliveryProfile("cd");
    });
    await act(async () => {
      harness.current().selectTrack(second.id);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(second.id);
    });

    await act(async () => {
      harness.current().toggleOverrideAlbum(second.id);
    });
    expect(harness.current().selectedIsOverriding).toBe(true);
    await act(async () => {
      harness.current().setPreset({ kind: "oomph" });
    });
    expect(harness.current().selectedSettings.preset).toEqual({ kind: "oomph" });
    await act(async () => {
      harness.current().toggleOverrideAlbum(second.id);
    });
    expect(harness.current().followingAlbumIntent).toBe(true);

    const albumIntent = harness.current().albumIntent;
    const divergentSettings: MasteringSettings = {
      ...albumIntent,
      preset: { kind: "oomph" },
    };

    await act(async () => {
      await harness.current().exportAlbumPlan();
    });

    const renderTracks =
      (mocks.api.renderAlbumPlan.mock.calls.at(-1)?.[1] as
        | import("../lib/api").AlbumTrackRenderInput[]
        | undefined) ?? [];
    const secondRenderInput = renderTracks.find(
      (track) => track.track_id === second.id,
    );
    expect(secondRenderInput?.settings).toEqual(albumIntent);
    expect(secondRenderInput?.settings).not.toEqual(divergentSettings);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("accepts an existing album export folder selected by the folder picker", async () => {
    const first = makeTrack("album-existing-1", "C:/audio/album existing one.wav");
    const second = makeTrack("album-existing-2", "C:/audio/album existing two.wav");
    const plan = makeAlbumPlan([first.id, second.id]);
    const outputDir = "/Users/daniel/Desktop/Album Masters";
    localStorage.setItem("yes-master:last-album-export-dir", outputDir);
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    mocks.open.mockResolvedValue(outputDir);
    mocks.api.planAlbum.mockResolvedValue(plan);
    mocks.api.renderAlbumPlan.mockResolvedValue({
      album_wav_path: `${outputDir}/album_continuous_1.wav`,
      manifest_path: `${outputDir}/manifest.json`,
      tracks: [],
    });
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
    });

    await act(async () => {
      await harness.current().exportAlbumPlan();
    });

    expect(mocks.open).toHaveBeenCalledWith({
      directory: true,
      defaultPath: outputDir,
      multiple: false,
      title: "Choose album export folder",
    });
    expect(mocks.api.renderAlbumPlan).toHaveBeenCalledWith(
      plan,
      expect.any(Array),
      outputDir,
    );
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("passes Windows-style album folders through to album rendering unchanged", async () => {
    const first = makeTrack("album-windows-1", "C:\\audio\\album one.wav");
    const second = makeTrack("album-windows-2", "C:\\audio\\album two.wav");
    const plan = makeAlbumPlan([first.id, second.id]);
    const outputDir = "C:\\Users\\Dan\\Desktop\\Album Masters";
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    mocks.open.mockResolvedValue(outputDir);
    mocks.api.planAlbum.mockResolvedValue(plan);
    mocks.api.renderAlbumPlan.mockResolvedValue({
      album_wav_path: `${outputDir}\\album_continuous_1.wav`,
      manifest_path: `${outputDir}\\manifest.json`,
      tracks: [],
    });
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
    });

    await act(async () => {
      await harness.current().exportAlbumPlan();
    });

    expect(mocks.api.renderAlbumPlan).toHaveBeenCalledWith(
      plan,
      expect.any(Array),
      outputDir,
    );
    expect(lastExportDirectory(localStorage, "album")).toBe(outputDir);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("does not render an album plan when the folder picker is cancelled", async () => {
    const first = makeTrack("album-cancel-1", "C:/audio/album cancel one.wav");
    const second = makeTrack("album-cancel-2", "C:/audio/album cancel two.wav");
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    mocks.open.mockResolvedValue(null);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await waitFor(() => {
      expect(harness.current().tracks).toHaveLength(2);
    });

    await act(async () => {
      await harness.current().exportAlbumPlan();
    });

    expect(mocks.open).toHaveBeenCalled();
    expect(mocks.api.planAlbum).not.toHaveBeenCalled();
    expect(mocks.api.renderAlbumPlan).not.toHaveBeenCalled();
    expect(harness.current().albumRendering).toBe(false);
    expect(harness.current().albumExportReport).toBeNull();
    expect(lastExportDirectory(localStorage, "album")).toBeNull();
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("does not expose the legacy album export hook action", async () => {
    const harness = await renderHookHarness();

    expect("exportAlbum" in harness.current()).toBe(false);
    expect("isExportingAlbum" in harness.current()).toBe(false);
    await act(async () => {
      harness.root.unmount();
    });
  });

  it("exposes hadPriorSession=false when there is no restorable session", async () => {
    mocks.api.loadRecentSession.mockResolvedValue(null);
    const harness = await renderHookHarness();
    await waitFor(() => {
      expect(harness.current().hadPriorSession).toBe(false);
    });
    await act(async () => harness.root.unmount());
  });

  it("resetToStandardManaged clears manual EQ but keeps preset/intensity", async () => {
    const track = makeTrack("reset-managed-1", "C:/audio/r.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();
    await act(async () => { await harness.current().importFiles([track.path]); });
    await waitFor(() => { expect(harness.current().selectedTrackId).toBe(track.id); });

    // Separate act blocks: updateSettings reads settingsMap from the
    // current-render closure (by design — see the hook), so batching these
    // into one act would have all three compute from the same stale base and
    // clobber each other. The established suite style (see resetToneControls)
    // is one edit per act.
    await act(async () => { harness.current().setPreset({ kind: "oomph" }); });
    await act(async () => { harness.current().setIntensity(0.8); });
    await act(async () => { harness.current().setEqBand("low", 4); });
    await waitFor(() => { expect(harness.current().selectedSettings.eq_low_db).toBe(4); });

    await act(async () => { harness.current().resetToStandardManaged(); });
    await waitFor(() => {
      const s = harness.current().selectedSettings;
      expect(s.eq_low_db).toBe(0);
      expect(s.preset).toEqual({ kind: "oomph" });
      expect(s.intensity).toBe(0.8);
    });
    await act(async () => harness.root.unmount());
  });

  it("exportStandardMaster renders with the fixed 44.1k/24-bit/-1 dBTP wrap", async () => {
    const track = makeTrack("std-export-1", "C:/audio/e.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue("C:/out/e.wav");
    mocks.api.renderTrackMaster.mockResolvedValue({
      output_paths: ["C:/out/e.wav"],
      measurements: null,
    });
    mocks.api.runExportChecks.mockResolvedValue([]);
    const harness = await renderHookHarness();
    await act(async () => { await harness.current().importFiles([track.path]); });
    await waitFor(() => { expect(harness.current().selectedTrackId).toBe(track.id); });

    await act(async () => { await harness.current().exportStandardMaster(); });
    await waitFor(() => { expect(mocks.api.renderTrackMaster).toHaveBeenCalled(); });

    const sent = mocks.api.renderTrackMaster.mock.calls[0][2] as { delivery_profile: string; advanced: { target_sample_rate: number; bit_depth: number; ceiling_dbtp: number } };
    expect(sent.delivery_profile).toBe("custom");
    expect(sent.advanced.target_sample_rate).toBe(44_100);
    expect(sent.advanced.bit_depth).toBe(24);
    expect(sent.advanced.ceiling_dbtp).toBe(-1);
    await act(async () => harness.root.unmount());
  });

  it("setForceWysiwyg never mutates the user-facing Preview LUFS toggle", async () => {
    const harness = await renderHookHarness();
    expect(harness.current().transport.exportLufsPreview).toBe(false);
    await act(async () => { harness.current().setForceWysiwyg(true); });
    // The internal flag drives landing; the visible Advanced toggle is untouched.
    expect(harness.current().transport.exportLufsPreview).toBe(false);
    await act(async () => { harness.current().setForceWysiwyg(false); });
    expect(harness.current().transport.exportLufsPreview).toBe(false);
    await act(async () => harness.root.unmount());
  });

  it("saveUserPreset resolves true on success and false on failure", async () => {
    const track = makeTrack("save-preset-1", "C:/audio/p.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();
    await act(async () => { await harness.current().importFiles([track.path]); });
    await waitFor(() => { expect(harness.current().selectedTrackId).toBe(track.id); });

    mocks.api.saveUserPreset.mockResolvedValueOnce({ id: "p1", name: "Mine", kind: "track", settings: {} });
    let ok: boolean | undefined;
    await act(async () => { ok = await harness.current().saveUserPreset("Mine"); });
    expect(ok).toBe(true);

    mocks.api.saveUserPreset.mockRejectedValueOnce(new Error("disk full"));
    let ok2: boolean | undefined;
    await act(async () => { ok2 = await harness.current().saveUserPreset("Other"); });
    expect(ok2).toBe(false);
    await act(async () => harness.root.unmount());
  });
});

describe("tone reset", () => {
  it("flattens intensity and every EQ band via resetToneControls", async () => {
    const track = makeTrack("reset-1", "C:/audio/reset.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    // Dirty the tone area: a couple of EQ bands + a non-default intensity.
    await act(async () => {
      harness.current().setEqBand("mid", 4);
    });
    await act(async () => {
      harness.current().setEqBand("sparkle", -2.5);
    });
    await act(async () => {
      harness.current().setIntensity(0.82);
    });
    await waitFor(() => {
      const s = harness.current().selectedSettings;
      expect(s.eq_mid_db).toBe(4);
      expect(s.eq_sparkle_db).toBe(-2.5);
      expect(s.intensity).toBe(0.82);
    });

    await act(async () => {
      harness.current().resetToneControls();
    });

    await waitFor(() => {
      const s = harness.current().selectedSettings;
      expect(s.intensity).toBe(0.5);
      expect(s.eq_sub_db).toBe(0);
      expect(s.eq_low_db).toBe(0);
      expect(s.eq_low_mid_db).toBe(0);
      expect(s.eq_mid_db).toBe(0);
      expect(s.eq_high_mid_db).toBe(0);
      expect(s.eq_high_db).toBe(0);
      expect(s.eq_sparkle_db).toBe(0);
    });

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("pushes the flattened tone to the live chain while a master is playing", async () => {
    const track = makeTrack("reset-live-1", "C:/audio/reset-live.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([track.path]);
    });
    await waitFor(() => {
      expect(harness.current().selectedTrackId).toBe(track.id);
    });

    await act(async () => {
      await harness.current().setPlaybackKind("master");
    });
    await act(async () => {
      await harness.current().togglePlay();
    });
    await waitFor(() => {
      expect(mocks.api.playMaster).toHaveBeenCalled();
    });

    await act(async () => {
      harness.current().setEqBand("mid", 6);
    });

    mocks.api.updateChain.mockClear();
    await act(async () => {
      harness.current().resetToneControls();
    });
    await waitFor(() => {
      expect(mocks.api.updateChain).toHaveBeenCalledWith(
        expect.objectContaining({ intensity: 0.5, eq_mid_db: 0 }),
        expect.any(Boolean),
        false, // Track mode → album flag false
      );
    });

    await act(async () => {
      harness.root.unmount();
    });
  });
});

describe("staged analysis progress", () => {
  it("keeps overlapping imports analyzing and ignores stale batch progress", async () => {
    let emitAnalysisProgress:
      | ((event: { batch_id: string; fraction: number; label: string }) => void)
      | undefined;
    mocks.onAnalysisProgress.mockImplementation((handler) => {
      emitAnalysisProgress = handler;
      return Promise.resolve(() => {});
    });

    const first = makeTrack("batch-a", "C:/audio/batch-a.wav");
    const second = makeTrack("batch-b", "C:/audio/batch-b.wav");
    mocks.api.importTracks.mockImplementation(async (paths: string[]) =>
      paths[0] === first.path ? [first] : [second],
    );
    const analyzeResolvers: Array<(value: AnalysisResult[]) => void> = [];
    mocks.api.analyzeTracks.mockImplementation(
      () =>
        new Promise<AnalysisResult[]>((resolve) => {
          analyzeResolvers.push(resolve);
        }),
    );

    const harness = await renderHookHarness();
    let firstImport!: Promise<void>;
    await act(async () => {
      firstImport = harness.current().importFiles([first.path]);
      await Promise.resolve();
    });
    await waitFor(() => {
      expect(harness.current().isAnalyzing).toBe(true);
      expect(analyzeResolvers).toHaveLength(1);
    });

    let secondImport!: Promise<void>;
    await act(async () => {
      secondImport = harness.current().importFiles([second.path]);
      await Promise.resolve();
    });
    await waitFor(() => {
      expect(harness.current().isAnalyzing).toBe(true);
      expect(analyzeResolvers).toHaveLength(2);
      expect(emitAnalysisProgress).toBeDefined();
    });

    const firstBatch = mocks.api.analyzeTracks.mock.calls[0]?.[1] as string;
    const secondBatch = mocks.api.analyzeTracks.mock.calls[1]?.[1] as string;
    expect(firstBatch).toBeTruthy();
    expect(secondBatch).toBeTruthy();
    expect(firstBatch).not.toBe(secondBatch);

    await act(async () => {
      emitAnalysisProgress?.({
        batch_id: secondBatch,
        fraction: 0.25,
        label: "Import B",
      });
    });
    expect(harness.current().analysisProgress).toEqual({
      label: "Import B",
      progress: 0.25,
    });

    await act(async () => {
      analyzeResolvers[0]([makeAnalysis(first.id)]);
      await firstImport;
    });
    expect(harness.current().isAnalyzing).toBe(true);

    await act(async () => {
      emitAnalysisProgress?.({
        batch_id: firstBatch,
        fraction: 0.95,
        label: "Late import A",
      });
    });
    expect(harness.current().analysisProgress).toEqual({
      label: "Import B",
      progress: 0.25,
    });

    await act(async () => {
      analyzeResolvers[1]([makeAnalysis(second.id)]);
      await secondImport;
    });
    await waitFor(() => {
      expect(harness.current().isAnalyzing).toBe(false);
    });

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("merges a late session-restore analysis without evicting imported analysis", async () => {
    const restored = makeTrack("restore-late", "C:/audio/restore-late.wav");
    const imported = makeTrack("import-during-restore", "C:/audio/import-during-restore.wav");
    mocks.api.loadRecentSession.mockResolvedValue(makeProjectState(restored));
    mocks.api.importTracks.mockResolvedValue([imported]);

    const analyzeResolvers: Array<(value: AnalysisResult[]) => void> = [];
    mocks.api.analyzeTracks.mockImplementation(
      () =>
        new Promise<AnalysisResult[]>((resolve) => {
          analyzeResolvers.push(resolve);
        }),
    );

    const harness = await renderHookHarness();
    await waitFor(() => {
      expect(harness.current().tracks.map((track) => track.id)).toContain(restored.id);
      expect(analyzeResolvers).toHaveLength(1);
    });

    let importDone!: Promise<void>;
    await act(async () => {
      importDone = harness.current().importFiles([imported.path]);
      await Promise.resolve();
    });
    await waitFor(() => {
      expect(analyzeResolvers).toHaveLength(2);
    });

    await act(async () => {
      analyzeResolvers[1]([makeAnalysis(imported.id)]);
      await importDone;
    });
    await act(async () => {
      harness.current().selectTrack(imported.id);
    });
    expect(harness.current().selectedAnalysis?.track_id).toBe(imported.id);

    await act(async () => {
      analyzeResolvers[0]([makeAnalysis(restored.id)]);
      await Promise.resolve();
    });
    await waitFor(() => {
      expect(mocks.api.prepareWaveform).toHaveBeenCalledWith(restored.id, restored.path, 1200);
    });
    await act(async () => {
      harness.current().selectTrack(imported.id);
    });
    expect(harness.current().selectedAnalysis?.track_id).toBe(imported.id);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("shows analyzing state while session restore analysis is pending", async () => {
    const restored = makeTrack("restore-pending", "C:/audio/restore-pending.wav");
    mocks.api.loadRecentSession.mockResolvedValue(makeProjectState(restored));
    let resolveRestore!: (value: AnalysisResult[]) => void;
    mocks.api.analyzeTracks.mockReturnValue(
      new Promise<AnalysisResult[]>((resolve) => {
        resolveRestore = resolve;
      }),
    );

    const harness = await renderHookHarness();
    await waitFor(() => {
      expect(harness.current().isAnalyzing).toBe(true);
      expect(harness.current().analysisProgress?.label).toBe("Analyzing audio");
    });

    await act(async () => {
      resolveRestore([makeAnalysis(restored.id)]);
      await Promise.resolve();
    });
    await waitFor(() => {
      expect(harness.current().isAnalyzing).toBe(false);
    });

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("ignores playback ticks for a non-selected track", async () => {
    let playbackHandler:
      | ((tick: {
          track_id: string | null;
          position_sec: number;
          is_playing: boolean;
          is_loaded: boolean;
          peak_dbfs: number;
          gr_low_db: number;
          gr_mid_db: number;
          gr_high_db: number;
          lufs_momentary: number;
          lufs_integrated: number;
          spectrum_db: number[];
        }) => void)
      | undefined;
    mocks.onPlaybackTick.mockImplementation((handler) => {
      playbackHandler = handler;
      return Promise.resolve(() => {});
    });
    const first = makeTrack("tick-a", "C:/audio/tick-a.wav");
    const second = makeTrack("tick-b", "C:/audio/tick-b.wav");
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await act(async () => {
      harness.current().selectTrack(second.id);
    });
    expect(harness.current().transport.currentTimeSec).toBe(0);

    await act(async () => {
      playbackHandler?.({
        track_id: first.id,
        position_sec: 17,
        is_playing: true,
        is_loaded: true,
        peak_dbfs: -6,
        gr_low_db: -120,
        gr_mid_db: -120,
        gr_high_db: -120,
        lufs_momentary: -12,
        lufs_integrated: -14,
        spectrum_db: [-30],
      });
    });
    expect(harness.current().transport.currentTimeSec).toBe(0);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("ignores landing status events for a non-selected track", async () => {
    let landingHandler:
      | ((
          pending: boolean,
          event: { track_id: string | null; pending: boolean },
        ) => void)
      | undefined;
    mocks.onLandingStatus.mockImplementation((handler) => {
      landingHandler = handler;
      return Promise.resolve(() => {});
    });
    const first = makeTrack("landing-a", "C:/audio/landing-a.wav");
    const second = makeTrack("landing-b", "C:/audio/landing-b.wav");
    mocks.api.importTracks.mockResolvedValue([first, second]);
    mocks.api.analyzeTracks.mockResolvedValue([
      makeAnalysis(first.id),
      makeAnalysis(second.id),
    ]);
    const harness = await renderHookHarness();

    await act(async () => {
      await harness.current().importFiles([first.path, second.path]);
    });
    await act(async () => {
      harness.current().selectTrack(second.id);
    });

    await act(async () => {
      landingHandler?.(true, { track_id: first.id, pending: true });
    });
    expect(harness.current().landingPending).toBe(false);

    await act(async () => {
      landingHandler?.(true, { track_id: second.id, pending: true });
    });
    expect(harness.current().landingPending).toBe(true);

    await act(async () => {
      harness.root.unmount();
    });
  });

  it("advances stages while analyzing, clamps at the last, then clears on completion", async () => {
    const track = makeTrack("track-a", "/in/a.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    // Hold analysis pending so isAnalyzing stays true while we drive the
    // staged-progress interval by hand.
    let resolveAnalyze!: (value: AnalysisResult[]) => void;
    mocks.api.analyzeTracks.mockReturnValue(
      new Promise<AnalysisResult[]>((resolve) => {
        resolveAnalyze = resolve;
      }),
    );

    const harness = await renderHookHarness();
    // Fake only the interval the staged progress uses; setTimeout, Date,
    // microtasks, and act's own scheduling stay real so mount/import settle.
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    try {
      let importDone!: Promise<void>;
      await act(async () => {
        importDone = harness.current().importFiles([track.path]);
      });

      // Analysis is active: the first stage shows immediately, and the
      // render channel is independent (still empty).
      expect(harness.current().isAnalyzing).toBe(true);
      expect(harness.current().analysisProgress?.label).toBe("Analyzing audio");
      expect(harness.current().renderProgress).toBeNull();

      // Each 1400ms tick advances to the next stage, in order.
      const laterStages = [
        "Reading tonal balance",
        "Checking dynamics",
        "Evaluating stereo field",
        "Building mastering context",
        "Preparing preview",
      ];
      for (const label of laterStages) {
        await act(async () => {
          vi.advanceTimersByTime(1400);
        });
        expect(harness.current().analysisProgress?.label).toBe(label);
      }

      // Clamp: extra ticks do not advance past the final stage.
      await act(async () => {
        vi.advanceTimersByTime(1400 * 3);
      });
      expect(harness.current().analysisProgress?.label).toBe("Preparing preview");

      // Completing analysis clears the staged progress.
      await act(async () => {
        resolveAnalyze([makeAnalysis(track.id)]);
        await importDone;
      });
      expect(harness.current().isAnalyzing).toBe(false);
      expect(harness.current().analysisProgress).toBeNull();
    } finally {
      vi.useRealTimers();
      await act(async () => {
        harness.root.unmount();
      });
    }
  });

  it("clears staged progress when analysis fails", async () => {
    const track = makeTrack("track-a", "/in/a.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    let rejectAnalyze!: (reason?: unknown) => void;
    mocks.api.analyzeTracks.mockReturnValue(
      new Promise<AnalysisResult[]>((_, reject) => {
        rejectAnalyze = reject;
      }),
    );

    const harness = await renderHookHarness();
    try {
      let importDone!: Promise<void>;
      await act(async () => {
        importDone = harness.current().importFiles([track.path]);
      });
      expect(harness.current().isAnalyzing).toBe(true);
      expect(harness.current().analysisProgress?.label).toBe("Analyzing audio");

      await act(async () => {
        rejectAnalyze(new Error("analyze boom"));
        await importDone;
      });
      expect(harness.current().isAnalyzing).toBe(false);
      expect(harness.current().analysisProgress).toBeNull();
    } finally {
      await act(async () => {
        harness.root.unmount();
      });
    }
  });
});

describe("render progress timer", () => {
  function captureRenderProgress() {
    let emitRenderProgress!: (evt: {
      fraction: number;
      kind: "preview" | "master" | "album";
    }) => void;
    mocks.onRenderProgress.mockImplementation((cb) => {
      emitRenderProgress = cb;
      return Promise.resolve(() => {});
    });
    return () => emitRenderProgress;
  }

  it("a previous render's completion clear does not wipe a newly-started render", async () => {
    const getEmitRenderProgress = captureRenderProgress();

    const harness = await renderHookHarness();
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    try {
      // Render A completes -> schedules the 600ms "clear the bar" timer.
      await act(async () => {
        getEmitRenderProgress()({ fraction: 1.0, kind: "master" });
      });
      // Render B starts <600ms later (a fresh, non-completion event).
      await act(async () => {
        vi.advanceTimersByTime(100);
        getEmitRenderProgress()({ fraction: 0.1, kind: "master" });
      });
      // Past A's 600ms window: its stale clear must NOT have fired.
      await act(async () => {
        vi.advanceTimersByTime(600);
      });
      expect(harness.current().renderProgress).toEqual({
        fraction: 0.1,
        kind: "master",
      });
    } finally {
      vi.useRealTimers();
      await act(async () => {
        harness.root.unmount();
      });
    }
  });

  it("clears incomplete preview progress when preview rendering fails", async () => {
    const getEmitRenderProgress = captureRenderProgress();
    const track = makeTrack("preview-error", "C:/audio/preview-error.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.api.renderTrackPreview.mockImplementation(async () => {
      getEmitRenderProgress()({ fraction: 0.42, kind: "preview" });
      throw new Error("preview render failed");
    });
    const harness = await renderHookHarness();

    try {
      await act(async () => {
        await harness.current().importFiles([track.path]);
      });
      await waitFor(() => {
        expect(harness.current().selectedTrackId).toBe(track.id);
      });

      await act(async () => {
        await harness.current().updatePreview();
      });

      expect(harness.current().error).toContain("preview render failed");
      expect(harness.current().isRendering).toBe(false);
      expect(harness.current().renderProgress).toBeNull();
    } finally {
      await act(async () => {
        harness.root.unmount();
      });
    }
  });

  it("clears incomplete master progress when master rendering fails", async () => {
    const getEmitRenderProgress = captureRenderProgress();
    const track = makeTrack("master-error", "C:/audio/master-error.wav");
    const outputPath = "/Users/daniel/Desktop/master-error.wav";
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue(outputPath);
    mocks.api.renderTrackMaster.mockImplementation(async () => {
      getEmitRenderProgress()({ fraction: 0.53, kind: "master" });
      throw new Error("master render failed");
    });
    const harness = await renderHookHarness();

    try {
      await act(async () => {
        await harness.current().importFiles([track.path]);
      });
      await waitFor(() => {
        expect(harness.current().selectedTrackId).toBe(track.id);
      });

      await act(async () => {
        await harness.current().exportMaster();
      });

      expect(harness.current().error).toContain("master render failed");
      expect(harness.current().isExporting).toBe(false);
      expect(harness.current().renderProgress).toBeNull();
    } finally {
      await act(async () => {
        harness.root.unmount();
      });
    }
  });

  it("clears stale master progress when export is cancelled before rendering", async () => {
    const getEmitRenderProgress = captureRenderProgress();
    const track = makeTrack("master-cancel", "C:/audio/master-cancel.wav");
    mocks.api.importTracks.mockResolvedValue([track]);
    mocks.api.analyzeTracks.mockResolvedValue([makeAnalysis(track.id)]);
    mocks.save.mockResolvedValue(null);
    const harness = await renderHookHarness();

    try {
      await act(async () => {
        await harness.current().importFiles([track.path]);
      });
      await waitFor(() => {
        expect(harness.current().selectedTrackId).toBe(track.id);
      });
      await act(async () => {
        getEmitRenderProgress()({ fraction: 0.31, kind: "master" });
      });
      expect(harness.current().renderProgress).toEqual({
        fraction: 0.31,
        kind: "master",
      });

      await act(async () => {
        await harness.current().exportMaster();
      });

      expect(mocks.api.renderTrackMaster).not.toHaveBeenCalled();
      expect(harness.current().renderProgress).toBeNull();
    } finally {
      await act(async () => {
        harness.root.unmount();
      });
    }
  });
});
