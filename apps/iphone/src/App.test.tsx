import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import App from "./App";
import type { IphoneBackend } from "./iphone-api";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((innerResolve) => {
    resolve = innerResolve;
  });
  return { promise, resolve };
}

function makeBackend(): IphoneBackend {
  return {
    importTrack: vi.fn().mockResolvedValue(importedTrack()),
    analyzeTrack: vi.fn().mockResolvedValue({
      track_id: "track-1",
      lufs_integrated: -15,
      true_peak_dbtp: -1.4,
      dynamic_range_lu: 8,
    }),
    renderMaster: vi.fn().mockResolvedValue({
      output_paths: ["/private/new-master__master.wav"],
      measurements: {
        lufs_integrated: -14,
        true_peak_dbtp: -1,
        dynamic_range_lu: 8,
        sample_rate: 48_000,
        bit_depth: 24,
      },
    }),
    prepareMasterPreview: vi.fn().mockResolvedValue({
      output_paths: ["/private/preview/track-1-mastered-preview.wav"],
      measurements: {
        lufs_integrated: -14,
        true_peak_dbtp: -1,
        dynamic_range_lu: 8,
        sample_rate: 48_000,
        bit_depth: 24,
      },
    }),
    runExportChecks: vi.fn().mockResolvedValue([
      {
        level: "info",
        code: "export_ok",
        message: "No issues detected in measured values.",
      },
    ]),
  } as unknown as IphoneBackend;
}

function importedTrack(
  overrides: Partial<{
    id: string;
    path: string;
    display_name: string;
    source_format: string;
    duration_seconds: number;
    sample_rate: number;
    channels: number;
  }> = {},
) {
  return {
    id: "track-1",
    path: "/private/new-master.wav",
    display_name: "new-master",
    source_format: "wav",
    duration_seconds: 181,
    sample_rate: 44_100,
    channels: 2,
    ...overrides,
  };
}

function renderApp({
  backend = makeBackend(),
  pickAudioPath = vi.fn().mockResolvedValue("/private/new-master.wav"),
  pickOutputPath = vi.fn().mockResolvedValue("/private/new-master__master.wav"),
  toAudioUrl = (path: string) => `https://audio.local${path}`,
}: {
  backend?: IphoneBackend;
  pickAudioPath?: () => Promise<string | null>;
  pickOutputPath?: () => Promise<string | null>;
  toAudioUrl?: (path: string) => string;
} = {}) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <App
        backend={backend}
        pickAudioPath={pickAudioPath}
        pickOutputPath={pickOutputPath}
        toAudioUrl={toAudioUrl}
      />,
    );
  });

  return { backend, container, pickAudioPath, pickOutputPath, root };
}

async function click(container: HTMLElement, selector: string) {
  const element = container.querySelector<HTMLElement>(selector);
  if (!element) throw new Error(`Missing element ${selector}`);
  await act(async () => {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

async function scrub(container: HTMLElement, selector: string, value: string) {
  const element = container.querySelector<HTMLInputElement>(selector);
  if (!element) throw new Error(`Missing element ${selector}`);
  const setValue = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    "value",
  )?.set;
  await act(async () => {
    setValue?.call(element, value);
    element.dispatchEvent(new Event("input", { bubbles: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

async function choose(container: HTMLElement, selector: string, value: string) {
  const element = container.querySelector<HTMLSelectElement>(selector);
  if (!element) throw new Error(`Missing element ${selector}`);
  await act(async () => {
    element.value = value;
    element.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

describe("iPhone app shell", () => {
  it("opens as a Simple-only phone app without desktop advanced controls", () => {
    const { container, root } = renderApp();

    expect(container.textContent).toContain("YES Master");
    expect(container.textContent).toContain("Simple");
    expect(container.textContent).toContain("Import Track");
    expect(container.textContent).not.toContain("Advanced");
    expect(container.textContent).not.toContain("Album Master");

    act(() => root.unmount());
  });

  it("imports and analyzes a track through the iPhone backend", async () => {
    const { backend, container, pickAudioPath, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");

    expect(pickAudioPath).toHaveBeenCalled();
    expect(backend.importTrack).toHaveBeenCalledWith("/private/new-master.wav");
    expect(backend.analyzeTrack).toHaveBeenCalledWith(
      "track-1",
      "/private/new-master.wav",
    );
    expect(container.textContent).toContain("new-master");
    expect(container.textContent).toContain("Ready");

    act(() => root.unmount());
  });

  it("does not start duplicate imports while import is already running", async () => {
    const selectedPath = deferred<string | null>();
    const pickAudioPath = vi.fn().mockReturnValue(selectedPath.promise);
    const { container, root } = renderApp({ pickAudioPath });

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='iphone-import']");

    expect(pickAudioPath).toHaveBeenCalledTimes(1);
    expect(container.textContent).toContain("Importing...");

    await act(async () => {
      selectedPath.resolve(null);
      await selectedPath.promise;
    });
    act(() => root.unmount());
  });

  it("lets the user replace a loaded track", async () => {
    const backend = makeBackend();
    vi.mocked(backend.importTrack)
      .mockResolvedValueOnce(importedTrack())
      .mockResolvedValueOnce(
        importedTrack({
          id: "track-2",
          path: "/private/second-song.wav",
          display_name: "second-song",
        }),
      );
    const pickAudioPath = vi
      .fn()
      .mockResolvedValueOnce("/private/new-master.wav")
      .mockResolvedValueOnce("/private/second-song.wav");
    const { container, root } = renderApp({ backend, pickAudioPath });

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='iphone-change-track']");

    expect(pickAudioPath).toHaveBeenCalledTimes(2);
    expect(backend.importTrack).toHaveBeenLastCalledWith(
      "/private/second-song.wav",
    );
    expect(container.textContent).toContain("second-song");
    expect(container.textContent).not.toContain("new-master");

    act(() => root.unmount());
  });

  it("lets the user pick tone, loudness, profile, audition mode, and export", async () => {
    const { container, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='tone-warm']");
    await click(container, "[data-testid='loudness-high']");
    await click(container, "[data-testid='profile-cd']");
    await click(container, "[data-testid='playback-mastered']");
    await click(container, "[data-testid='volume-match']");
    await click(container, "[data-testid='lufs-preview']");

    expect(container.textContent).toContain("Warm");
    expect(container.textContent).toContain("-10.5 LUFS");
    expect(container.textContent).toContain("44.1 kHz");
    expect(container.textContent).toContain("16-bit");
    expect(container.textContent).toContain("Mastered");
    expect(container.textContent).toContain("Export Master");

    act(() => root.unmount());
  });

  it("keeps the visible playhead when switching Original and Mastered", async () => {
    const { container, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    await scrub(container, "[data-testid='iphone-playhead']", "42");
    await click(container, "[data-testid='playback-mastered']");

    const playhead = container.querySelector<HTMLInputElement>(
      "[data-testid='iphone-playhead']",
    );
    expect(playhead?.value).toBe("42");
    expect(container.textContent).toContain("0:42");

    act(() => root.unmount());
  });

  it("prepares a mastered preview before switching to Mastered", async () => {
    const { backend, container, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='volume-match']");
    await click(container, "[data-testid='playback-mastered']");

    expect(backend.prepareMasterPreview).toHaveBeenCalledWith(
      expect.objectContaining({
        trackId: "track-1",
        trackPath: "/private/new-master.wav",
        settings: expect.objectContaining({
          volume_match: true,
        }),
      }),
    );
    expect(container.textContent).toContain("Mastered");

    act(() => root.unmount());
  });

  it("only applies LUFS landing to mastered preview when LUFS Preview is on", async () => {
    const { backend, container, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='playback-mastered']");

    expect(backend.prepareMasterPreview).toHaveBeenLastCalledWith(
      expect.objectContaining({
        settings: expect.objectContaining({
          advanced: expect.objectContaining({
            lufs_offset_db: null,
          }),
        }),
      }),
    );

    await click(container, "[data-testid='playback-original']");
    await click(container, "[data-testid='lufs-preview']");
    await click(container, "[data-testid='playback-mastered']");

    expect(backend.prepareMasterPreview).toHaveBeenLastCalledWith(
      expect.objectContaining({
        settings: expect.objectContaining({
          advanced: expect.objectContaining({
            lufs_offset_db: -14,
          }),
        }),
      }),
    );

    act(() => root.unmount());
  });

  it("swaps the audition audio source from Original to Mastered preview", async () => {
    const { container, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    const audio = container.querySelector<HTMLAudioElement>(
      "[data-testid='iphone-audio-preview']",
    );
    expect(audio?.src).toBe("https://audio.local/private/new-master.wav");

    await click(container, "[data-testid='playback-mastered']");

    expect(audio?.src).toBe(
      "https://audio.local/private/preview/track-1-mastered-preview.wav",
    );

    act(() => root.unmount());
  });

  it("exports with the selected Custom profile settings", async () => {
    const { backend, container, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='profile-custom']");
    await choose(container, "[data-testid='custom-sample-rate']", "96000");
    await choose(container, "[data-testid='custom-bit-depth']", "16");
    await choose(container, "[data-testid='custom-ceiling']", "-2");
    await click(container, "[data-testid='iphone-export']");

    expect(backend.renderMaster).toHaveBeenCalledWith(
      expect.objectContaining({
        settings: expect.objectContaining({
          advanced: expect.objectContaining({
            target_sample_rate: 96_000,
            bit_depth: 16,
            ceiling_dbtp: -2,
          }),
        }),
      }),
    );
    expect(container.textContent).toContain("96 kHz");
    expect(container.textContent).toContain("16-bit");

    act(() => root.unmount());
  });

  it("exports through the iPhone render path with export settings", async () => {
    const { backend, container, pickOutputPath, root } = renderApp();

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='volume-match']");
    await click(container, "[data-testid='iphone-export']");

    expect(pickOutputPath).toHaveBeenCalled();
    expect(backend.renderMaster).toHaveBeenCalledWith(
      expect.objectContaining({
        trackId: "track-1",
        trackPath: "/private/new-master.wav",
        outputPath: "/private/new-master__master.wav",
        settings: expect.objectContaining({
          volume_match: false,
        }),
      }),
    );
    expect(backend.runExportChecks).toHaveBeenCalled();
    expect(container.textContent).toContain("Exported");

    act(() => root.unmount());
  });

  it("does not start duplicate exports while export is already running", async () => {
    const selectedOutput = deferred<string | null>();
    const pickOutputPath = vi.fn().mockReturnValue(selectedOutput.promise);
    const { container, pickOutputPath: pickOutput, root } = renderApp({
      pickOutputPath,
    });

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='iphone-export']");
    await click(container, "[data-testid='iphone-export']");

    expect(pickOutput).toHaveBeenCalledTimes(1);
    expect(container.textContent).toContain("Exporting...");

    await act(async () => {
      selectedOutput.resolve(null);
      await selectedOutput.promise;
    });
    act(() => root.unmount());
  });

  it("shows advisory export warnings without blocking export", async () => {
    const backend = makeBackend();
    vi.mocked(backend.runExportChecks).mockResolvedValue([
      {
        level: "warning",
        code: "true_peak_high",
        message: "True peak is high. Consider lowering the ceiling.",
      },
    ]);
    const { container, root } = renderApp({ backend });

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='iphone-export']");

    expect(container.textContent).toContain("Exported with 1 warning");
    expect(container.textContent).toContain("True peak is high");

    act(() => root.unmount());
  });

  it("clears export warnings when a different track is imported", async () => {
    const backend = makeBackend();
    vi.mocked(backend.importTrack)
      .mockResolvedValueOnce(importedTrack())
      .mockResolvedValueOnce(
        importedTrack({
          id: "track-2",
          path: "/private/second-song.wav",
          display_name: "second-song",
        }),
      );
    vi.mocked(backend.runExportChecks).mockResolvedValue([
      {
        level: "warning",
        code: "true_peak_high",
        message: "True peak is high. Consider lowering the ceiling.",
      },
    ]);
    const pickAudioPath = vi
      .fn()
      .mockResolvedValueOnce("/private/new-master.wav")
      .mockResolvedValueOnce("/private/second-song.wav");
    const { container, root } = renderApp({ backend, pickAudioPath });

    await click(container, "[data-testid='iphone-import']");
    await click(container, "[data-testid='iphone-export']");
    expect(container.textContent).toContain("True peak is high");

    await click(container, "[data-testid='iphone-change-track']");

    expect(container.textContent).toContain("second-song");
    expect(container.textContent).not.toContain("True peak is high");

    act(() => root.unmount());
  });
});
