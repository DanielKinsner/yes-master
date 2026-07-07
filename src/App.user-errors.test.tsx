import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";

import App from "./App";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const mocks = vi.hoisted(() => {
  const api = {
    importTracks: vi.fn(),
    analyzeTracks: vi.fn(),
    renderTrackPreview: vi.fn(),
    renderTrackMaster: vi.fn(),
    prepareWaveform: vi.fn(),
    listAudioOutputDevices: vi.fn(),
    setAudioOutputDevice: vi.fn(),
    clearDeviceLost: vi.fn(),
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
    planAlbum: vi.fn(),
    renderAlbumPlan: vi.fn(),
  };
  return {
    api,
    onPlaybackTick: vi.fn(),
    onPlaybackDeviceLost: vi.fn(),
    onRenderProgress: vi.fn(),
    onLandingStatus: vi.fn(),
    onAnalysisProgress: vi.fn(),
    onUpdaterAvailable: vi.fn(),
    open: vi.fn(),
    save: vi.fn(),
    onDragDropEvent: vi.fn(),
  };
});

vi.mock("./lib/api", () => ({
  ADAPTIVE_COMPRESSION_GATE_EVENT: "yes-master:adaptive-compression-gate",
  api: mocks.api,
  onPlaybackTick: mocks.onPlaybackTick,
  onPlaybackDeviceLost: mocks.onPlaybackDeviceLost,
  onRenderProgress: mocks.onRenderProgress,
  onLandingStatus: mocks.onLandingStatus,
  onAnalysisProgress: mocks.onAnalysisProgress,
  onUpdaterAvailable: mocks.onUpdaterAvailable,
}));

vi.mock("./lib/tauri-runtime", () => ({
  open: mocks.open,
  save: mocks.save,
  getCurrentWebview: () => ({
    onDragDropEvent: mocks.onDragDropEvent,
  }),
}));

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

async function waitFor(assertion: () => void, timeoutMs = 2000): Promise<void> {
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

async function mountApp(): Promise<{ root: Root; container: HTMLDivElement }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(<App />);
  });
  return { root, container };
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
  mocks.onPlaybackDeviceLost.mockReset();
  mocks.onRenderProgress.mockReset();
  mocks.onLandingStatus.mockReset();
  mocks.onAnalysisProgress.mockReset();
  mocks.onUpdaterAvailable.mockReset();

  mocks.api.listUserPresets.mockResolvedValue([]);
  mocks.api.loadRecentSession.mockResolvedValue(null);
  mocks.api.importTracks.mockResolvedValue([]);
  mocks.api.listAudioOutputDevices.mockResolvedValue([]);
  mocks.api.setAudioOutputDevice.mockResolvedValue(null);
  mocks.api.autosaveSession.mockResolvedValue(null);
  mocks.onPlaybackTick.mockResolvedValue(() => {});
  mocks.onPlaybackDeviceLost.mockResolvedValue(() => {});
  mocks.onRenderProgress.mockResolvedValue(() => {});
  mocks.onLandingStatus.mockResolvedValue(() => {});
  mocks.onAnalysisProgress.mockResolvedValue(() => {});
  mocks.onUpdaterAvailable.mockResolvedValue(() => {});
  mocks.onDragDropEvent.mockResolvedValue(() => {});
}

function buttonByText(container: HTMLElement, text: string): HTMLButtonElement {
  const button = Array.from(
    container.querySelectorAll<HTMLButtonElement>("button"),
  ).find((candidate) => (candidate.textContent ?? "").trim() === text);
  if (!button) throw new Error(`button not found: ${text}`);
  return button;
}

beforeEach(() => {
  installTestLocalStorage();
  localStorage.clear();
  resetApiMocks();
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("App user-facing errors", () => {
  it("renders friendly Decode errors without hiding pending project feedback", async () => {
    mocks.save.mockResolvedValueOnce(null);
    mocks.open.mockResolvedValueOnce(["C:/audio/bad.wav"]);
    mocks.api.importTracks.mockRejectedValueOnce(
      new Error("decode error: no decodable track"),
    );
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(buttonByText(container, "Import audio")).toBeTruthy();
    });

    await act(async () => {
      container
        .querySelector<HTMLButtonElement>('[aria-label="Save project (.ams.json)"]')
        ?.click();
    });
    await waitFor(() => {
      expect(container.textContent).toContain("Save project canceled.");
    });

    await act(async () => {
      buttonByText(container, "Import audio").click();
    });
    await waitFor(() => {
      expect(container.textContent).toContain("Couldn't read bad.wav.");
    });

    const toasts = Array.from(container.querySelectorAll(".toast"));
    expect(toasts).toHaveLength(2);
    expect(container.textContent).toContain("Save project canceled.");
    expect(container.textContent).toContain("Re-import it or use Re-analyze.");
    expect(container.textContent).toContain("decode error: no decodable track");

    await act(async () => {
      root.unmount();
    });
  });

  it("explains that Album Master opens Advanced", async () => {
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(buttonByText(container, "Import audio")).toBeTruthy();
    });

    await act(async () => {
      buttonByText(container, "Album Master").click();
    });

    await waitFor(() => {
      expect(container.textContent).toContain(
        "Opening Album Master in Advanced.",
      );
    });

    await act(async () => {
      root.unmount();
    });
  });

  it("shows a recovery banner when playback device loss is emitted", async () => {
    let deviceLostHandler:
      | ((event: { track_id: string | null; position_sec: number }) => void)
      | undefined;
    mocks.onPlaybackDeviceLost.mockImplementation((handler) => {
      deviceLostHandler = handler;
      return Promise.resolve(() => {});
    });
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(deviceLostHandler).toBeDefined();
    });

    await act(async () => {
      deviceLostHandler?.({ track_id: "lost-track", position_sec: 5.5 });
    });

    expect(container.textContent).toContain("Playback device disconnected.");
    expect(container.textContent).toContain("Choose an output in Settings");

    await act(async () => {
      buttonByText(container, "Choose device").click();
    });
    expect(container.textContent).toContain("Audio Output");
    expect(container.textContent).toContain("Playback device");

    await act(async () => {
      root.unmount();
    });
  });
});
