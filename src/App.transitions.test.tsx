// src/App.transitions.test.tsx
//
// End-to-end integration tests that mount the REAL <App/> and exercise the
// Standard<->Advanced view-mode wiring landed in Task 10. These tests drive the
// view entirely through the two deterministic inputs the wiring reads:
//
//   * localStorage key `yes-master:view-mode` -> seeds useViewMode's initial
//     resolution (a `{migrated:true, lastView:…}` value resolves synchronously
//     on first render; its absence makes useViewMode wait for the session
//     signal before resolving).
//   * the `api.loadRecentSession` mock -> drives `hadPriorSession` (returning
//     vs new user) and, when it returns a track whose settings carry a
//     non-default field (`eq_low_db: 3`), makes `hasNonManagedEdits` true so
//     the always-clean entry guard forces Advanced.
//
// No DOM-driving of knobs is needed to set up the initial view. The only DOM
// interactions are the affordance buttons ("Advanced" / "‹ Back to Standard")
// and the confirm-modal action ("Reset & continue") — which is the actual
// surface a user touches. We assert on rendered DOM (affordance button text,
// presence/absence of the desk Sidebar, the confirm dialog) rather than on
// mock internals.
//
// Mounting harness + api-mock shape are copied from
// useTrackMaster.integration.test.tsx so <App/> mounts without throwing.

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";

import App from "./App";
import type {
  ImportedTrack,
  MasteringSettings,
  ProjectMode,
  ProjectState,
  WaveformPeaks,
} from "./bindings";

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
    installUpdate: vi.fn(),
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

function makeWaveform(trackId: string): WaveformPeaks {
  return {
    track_id: trackId,
    channels: [[], []],
    samples_per_pixel: 512,
    total_samples: 0,
    sample_rate: 44_100,
  };
}

/// A restorable session whose single track carries `settings`. `mode` defaults
/// to "track"; pass "album" to seed Album mode (which forces Advanced).
function makeSession(
  settings: MasteringSettings,
  mode: ProjectMode = "track",
): ProjectState {
  const track = makeTrack("restored-1", "C:/audio/restored.wav");
  return {
    schema_version: 1,
    mode,
    tracks: [track],
    track_order: [track.id],
    track_settings: { [track.id]: settings },
    album_intent: null,
    track_override_album: [],
    last_saved_iso: null,
  };
}

const CLEAN_SETTINGS: MasteringSettings = DEFAULT_SETTINGS;
const DIRTY_SETTINGS: MasteringSettings = {
  ...DEFAULT_SETTINGS,
  // A non-default Tone Shape band -> hasNonManagedEdits() === true, so the
  // entry guard forces Advanced and the return door routes through confirm.
  eq_low_db: 3,
};

function seedViewMode(lastView: "standard" | "advanced") {
  localStorage.setItem(
    "yes-master:view-mode",
    JSON.stringify({ migrated: true, lastView }),
  );
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

  mocks.api.listUserPresets.mockResolvedValue([]);
  mocks.api.loadRecentSession.mockResolvedValue(null);
  mocks.api.importTracks.mockResolvedValue([]);
  mocks.api.analyzeTracks.mockResolvedValue([]);
  mocks.api.prepareWaveform.mockImplementation((trackId: string) =>
    Promise.resolve(makeWaveform(trackId)),
  );
  mocks.api.listAudioOutputDevices.mockResolvedValue([]);
  mocks.api.setAudioOutputDevice.mockResolvedValue(null);
  mocks.api.prewarmDecode.mockResolvedValue(null);
  mocks.api.autosaveSession.mockResolvedValue(null);
  mocks.api.stopPlayback.mockResolvedValue(null);
  mocks.onPlaybackTick.mockResolvedValue(() => {});
  mocks.onPlaybackDeviceLost.mockResolvedValue(() => {});
  mocks.onRenderProgress.mockResolvedValue(() => {});
  mocks.onLandingStatus.mockResolvedValue(() => {});
  mocks.onAnalysisProgress.mockResolvedValue(() => {});
  mocks.onUpdaterAvailable.mockResolvedValue(() => {});
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

async function mountApp(): Promise<{ root: Root; container: HTMLDivElement }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(<App />);
  });
  return { root, container };
}

// ---- DOM queries (stable selectors only) -----------------------------------

/// The header affordance button. Its text discriminates the view:
/// "Advanced" in Standard, "‹ Back to Standard" in Advanced.
function affordance(container: HTMLElement): HTMLButtonElement | null {
  const buttons = Array.from(
    container.querySelectorAll<HTMLButtonElement>("button.top-advanced"),
  );
  return buttons[0] ?? null;
}

function affordanceText(container: HTMLElement): string {
  return (affordance(container)?.textContent ?? "").trim();
}

/// The desk Sidebar (<aside class="sidebar">) renders only in Advanced.
function hasSidebar(container: HTMLElement): boolean {
  return container.querySelector("aside.sidebar") !== null;
}

/// The Back-to-Standard confirm modal: role="dialog" aria-label="Back to Standard".
function confirmDialog(container: HTMLElement): HTMLElement | null {
  return container.querySelector<HTMLElement>(
    '[role="dialog"][aria-label="Back to Standard"]',
  );
}

function findButtonByText(
  container: HTMLElement,
  text: string,
): HTMLButtonElement {
  const button = Array.from(
    container.querySelectorAll<HTMLButtonElement>("button"),
  ).find((b) => (b.textContent ?? "").trim() === text);
  if (!button) throw new Error(`button with text "${text}" not found`);
  return button;
}

async function click(el: HTMLElement): Promise<void> {
  await act(async () => {
    el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

beforeEach(() => {
  installTestLocalStorage();
  localStorage.clear();
  resetApiMocks();
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("App Standard<->Advanced view transitions", () => {
  it("1) new user with no prior session lands in Standard (no Sidebar, Advanced affordance)", async () => {
    // No view-mode key, no restorable session -> resolveInitialViewMode picks
    // Standard for a brand-new user.
    mocks.api.loadRecentSession.mockResolvedValue(null);
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("1a) new user's FIRST committed frame does not flash the Advanced desk", async () => {
    // Regression guard for the unresolved-view flash (adversarial review): for a
    // brand-new user, `view` is null until the async loadRecentSession probe
    // settles AFTER first paint. The desk gate must NOT paint during that null
    // frame (null === "advanced" is false), so the first committed frame shows
    // chrome only — no Sidebar — then resolves to Standard.
    const sessionProbe = deferred<ProjectState | null>();
    mocks.api.loadRecentSession.mockReturnValue(sessionProbe.promise);

    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    // Synchronous render: flushes the synchronous render + initial effects but
    // NOT the async session probe (which resolves a microtask/tick later).
    // Deliberately NOT awaited and NOT followed by waitFor here.
    act(() => {
      root.render(<App />);
    });

    // First frame: view is still null, so the desk must be absent.
    expect(container.querySelector("aside.sidebar")).toBeNull();

    // Now let the probe settle inside act and confirm we resolve to Standard.
    await act(async () => {
      sessionProbe.resolve(null);
      await sessionProbe.promise;
    });
    await waitFor(() => {
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("2) returning user with a clean prior session lands in Advanced (Sidebar + Back affordance)", async () => {
    // No view-mode key but a restorable session with >=1 track ->
    // hadPriorSession=true -> Advanced.
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(CLEAN_SETTINGS));
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");

    await act(async () => {
      root.unmount();
    });
  });

  it("3) migrated-last-Standard with a clean track stays Standard (no bounce)", async () => {
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(CLEAN_SETTINGS));
    const { root, container } = await mountApp();

    // Let the session restore settle so the entry guard has the real track +
    // (clean) settings to evaluate; a clean track must NOT bounce to Advanced.
    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    expect(affordanceText(container)).toBe("Advanced");
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("4) take control: clicking Advanced from Standard reveals the desk", async () => {
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(CLEAN_SETTINGS));
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    expect(hasSidebar(container)).toBe(false);

    await click(affordance(container)!);

    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");

    await act(async () => {
      root.unmount();
    });
  });

  it("5) back to Standard with a clean track returns silently (no confirm modal)", async () => {
    // Reach Advanced via take-control on a clean track (scenario 4 state), then
    // return: spec §2a — with no non-managed edits the door switches straight
    // to Standard; the confirm modal is reserved for dirty returns (scenario 7).
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(CLEAN_SETTINGS));
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    await click(affordance(container)!);
    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });

    // Return door — clean settings => no confirm.
    await click(affordance(container)!);

    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    expect(confirmDialog(container)).toBeNull();
    expect(affordanceText(container)).toBe("Advanced");
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("5a) Standard entry disarms the loop (F3): entering Standard sends setLoopRegion(null)", async () => {
    // The F3 invariant — an Advanced-armed loop must not survive into
    // Standard, which has no loop UI to see or disarm it — is enforced by a
    // single App-level effect (view === "standard" → tm.disarmLoop()). Pin
    // the WIRING here: reaching Standard must hit the backend with
    // setLoopRegion(null). The disarm semantics (transport.loop cleared,
    // per-track region memory kept) are pinned at the hook level in
    // useTrackMaster.integration.test.tsx; the shift-drag gesture gate has
    // its own pins. Without this test, deleting the effect passed the whole
    // suite while resurrecting the owner-smoke hidden-loop bug (2026-07-06
    // audit).
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(CLEAN_SETTINGS));
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    await click(affordance(container)!);
    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });

    // Re-enter Standard through the return door; only calls made by THIS
    // transition count.
    mocks.api.setLoopRegion.mockClear();
    await click(affordance(container)!);
    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    expect(mocks.api.setLoopRegion).toHaveBeenCalledWith(null);

    await act(async () => {
      root.unmount();
    });
  });

  it("6) dirty entry bounce: last-Standard but a dirty track forces Advanced", async () => {
    // localStorage seeds Standard, so useViewMode resolves "standard"
    // synchronously — but the entry guard sees the restored track's dirty
    // settings (eq_low_db !== 0) and forces Advanced.
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(DIRTY_SETTINGS));
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");
    // Standard surface must NOT be on screen.
    expect(container.querySelector(".std-tiles")).toBeNull();

    await act(async () => {
      root.unmount();
    });
  });

  it("7) back to Standard with dirty edits opens the confirm modal; Reset & continue returns to Standard", async () => {
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(makeSession(DIRTY_SETTINGS));
    const { root, container } = await mountApp();

    // Forced into Advanced by the dirty entry guard.
    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");

    // Return door with dirty edits -> confirm modal appears.
    await click(affordance(container)!);
    await waitFor(() => {
      expect(confirmDialog(container)).not.toBeNull();
    });

    // Reset & continue -> modal closes, view returns to Standard.
    await click(findButtonByText(container, "Reset & continue"));

    await waitFor(() => {
      expect(confirmDialog(container)).toBeNull();
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("8) album-mode session forces Advanced even when last-Standard", async () => {
    // Album Master is Advanced-only in v1: even with last-Standard seeded and a
    // clean track, an album-mode session must force Advanced.
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(
      makeSession(CLEAN_SETTINGS, "album"),
    );
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");
    expect(container.querySelector(".std-tiles")).toBeNull();

    await act(async () => {
      root.unmount();
    });
  });

  it("9) back to Standard from Album mode leaves Album and lands in Standard", async () => {
    // Regression: the return door used to only setView("standard") — the
    // Album-only-in-Advanced entry guard re-bounced to Advanced in the same
    // commit and the button visibly did nothing. Returning to Standard must
    // also return the project mode to Track.
    seedViewMode("standard");
    mocks.api.loadRecentSession.mockResolvedValue(
      makeSession(CLEAN_SETTINGS, "album"),
    );
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");

    await click(affordance(container)!);

    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    expect(hasSidebar(container)).toBe(false);
    expect(affordanceText(container)).toBe("Advanced");
    // The mode tabs must reflect the implied Album -> Track switch.
    const activeTab = container.querySelector("button.top-tab.is-active");
    expect(activeTab?.textContent).toBe("Track Master");

    await act(async () => {
      root.unmount();
    });
  });
});

describe("welcome hero import funnel", () => {
  it("importing from the hero in an empty Advanced session lands in Standard", async () => {
    // Owner repro 2026-06-12: last session ended in Advanced, app boots
    // empty showing the welcome hero inside the Advanced shell; the hero
    // CTA must funnel into Standard (the default face), not keep Advanced.
    seedViewMode("advanced");
    mocks.api.loadRecentSession.mockResolvedValue(null);
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(affordanceText(container)).toBe("‹ Back to Standard");
    });
    expect(hasSidebar(container)).toBe(true);

    await click(findButtonByText(container, "Import audio"));

    await waitFor(() => {
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(hasSidebar(container)).toBe(false);
    expect(mocks.open).toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });
});

describe("App F6 per-track view memory", () => {
  function makeTwoTrackSession(args: {
    aSettings: MasteringSettings;
    bSettings: MasteringSettings;
    viewByTrackId?: Record<string, "standard" | "advanced">;
    selected?: string;
  }): ProjectState {
    const a = makeTrack("track-a", "C:/audio/a.wav");
    const b = makeTrack("track-b", "C:/audio/b.wav");
    return {
      schema_version: 1,
      mode: "track",
      tracks: [a, b],
      track_order: [a.id, b.id],
      track_settings: { [a.id]: args.aSettings, [b.id]: args.bSettings },
      album_intent: null,
      track_override_album: [],
      view_by_track_id: args.viewByTrackId,
      selected_track_id: args.selected ?? a.id,
      last_saved_iso: null,
    };
  }

  function trackPick(container: HTMLElement, name: string): HTMLButtonElement {
    const btn = Array.from(
      container.querySelectorAll<HTMLButtonElement>("button.track-pick"),
    ).find((b) => (b.querySelector(".track-name")?.textContent ?? "").includes(name));
    if (!btn) throw new Error(`track "${name}" not found in the sidebar`);
    return btn;
  }

  it("restores a track's remembered Standard view when switched to in the sidebar", async () => {
    // Both tracks clean; only B remembers Standard. Land in Advanced (returning
    // session), then switch to B — without per-track memory a sidebar switch
    // between clean tracks keeps Advanced, so a flip to Standard proves restore.
    mocks.api.loadRecentSession.mockResolvedValue(
      makeTwoTrackSession({
        aSettings: CLEAN_SETTINGS,
        bSettings: CLEAN_SETTINGS,
        viewByTrackId: { "track-b": "standard" },
        selected: "track-a",
      }),
    );
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });

    await click(trackPick(container, "track-b.wav"));
    await waitFor(() => {
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("applies a loaded session's remembered view, overriding the global default", async () => {
    // Global default would be Advanced (seeded last-view + returning session),
    // but the loaded session remembers Standard for the selected track, so the
    // per-track memory must win — proving it round-trips through save/open.
    seedViewMode("advanced");
    mocks.api.loadRecentSession.mockResolvedValue(
      makeTwoTrackSession({
        aSettings: CLEAN_SETTINGS,
        bSettings: CLEAN_SETTINGS,
        viewByTrackId: { "track-a": "standard" },
        selected: "track-a",
      }),
    );
    const { root, container } = await mountApp();

    await waitFor(() => {
      expect(container.querySelector(".std-tiles")).not.toBeNull();
    });
    expect(affordanceText(container)).toBe("Advanced");
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("a dirty track's forced Advanced does not disturb a sibling's remembered Standard", async () => {
    // B is dirty and selected, so it is force-bounced to Advanced on load. A is
    // clean and remembers Standard. Switching to A restores Standard — proving
    // the dirty bounce never wrote to (clobbered) the remembered-view map.
    mocks.api.loadRecentSession.mockResolvedValue(
      makeTwoTrackSession({
        aSettings: CLEAN_SETTINGS,
        bSettings: DIRTY_SETTINGS,
        viewByTrackId: { "track-a": "standard" },
        selected: "track-b",
      }),
    );
    const { root, container } = await mountApp();

    // Dirty B is force-bounced to Advanced on load (desk visible).
    await waitFor(() => {
      expect(hasSidebar(container)).toBe(true);
    });
    expect(affordanceText(container)).toBe("‹ Back to Standard");

    // A's remembered Standard survived the bounce — switching restores it.
    await click(trackPick(container, "track-a.wav"));
    await waitFor(() => {
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(hasSidebar(container)).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });
});

describe("App updater toast (Slice 7b)", () => {
  function captureUpdaterFire(): { current: (v: string) => void } {
    const ref: { current: (v: string) => void } = { current: () => {} };
    mocks.onUpdaterAvailable.mockImplementation((handler: (v: string) => void) => {
      ref.current = handler;
      return Promise.resolve(() => {});
    });
    return ref;
  }

  it("shows a one-click install toast on updater:available and installs on click", async () => {
    const fire = captureUpdaterFire();
    const { root, container } = await mountApp();

    // No update toast until the backend emits the event.
    await waitFor(() => {
      expect(affordanceText(container)).toBe("Advanced");
    });
    expect(container.querySelector(".toast-action")).toBeNull();

    await act(async () => {
      fire.current("1.2.3");
    });
    await waitFor(() => {
      expect(container.querySelector(".toast-action")).not.toBeNull();
    });
    const toast = container.querySelector(".toast");
    expect(toast?.textContent).toContain("Update available");
    expect(toast?.textContent).toContain("v1.2.3");
    const action = container.querySelector<HTMLButtonElement>(".toast-action");
    expect(action?.textContent).toBe("Restart to update");
    expect(action?.disabled).toBe(false); // idle -> enabled

    await click(action!);
    expect(mocks.api.installUpdate).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.unmount();
    });
  });

  it("dismisses the update toast on close", async () => {
    const fire = captureUpdaterFire();
    const { root, container } = await mountApp();
    await act(async () => {
      fire.current("2.0.0");
    });
    await waitFor(() => {
      expect(container.querySelector(".toast-action")).not.toBeNull();
    });

    const close = container.querySelector<HTMLButtonElement>(".toast .toast-close");
    await click(close!);
    await waitFor(() => {
      expect(container.querySelector(".toast-action")).toBeNull();
    });

    await act(async () => {
      root.unmount();
    });
  });
});
