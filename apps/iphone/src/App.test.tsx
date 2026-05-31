import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import App from "./App";
import type { IphoneBackend } from "./iphone-api";

function makeBackend(): IphoneBackend {
  return {
    importTrack: vi.fn().mockResolvedValue({
      id: "track-1",
      path: "/private/new-master.wav",
      display_name: "new-master",
      source_format: "wav",
      duration_seconds: 181,
      sample_rate: 44_100,
      channels: 2,
    }),
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
    runExportChecks: vi.fn().mockResolvedValue([
      {
        level: "info",
        code: "export_ok",
        message: "No issues detected in measured values.",
      },
    ]),
  } as unknown as IphoneBackend;
}

function renderApp({
  backend = makeBackend(),
  pickAudioPath = vi.fn().mockResolvedValue("/private/new-master.wav"),
  pickOutputPath = vi.fn().mockResolvedValue("/private/new-master__master.wav"),
}: {
  backend?: IphoneBackend;
  pickAudioPath?: () => Promise<string | null>;
  pickOutputPath?: () => Promise<string | null>;
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
});
