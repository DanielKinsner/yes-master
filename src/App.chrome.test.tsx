import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { HelpPanel, PresetTiles, SettingsPanel, TopHeader, TrackHeader } from "./App";
import type { ImportedTrack } from "./bindings";
import { STANDARD_EXPORT_DELIVERY } from "./lib/standard-export";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

async function renderNode(node: ReactNode): Promise<{
  container: HTMLDivElement;
  root: Root;
}> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

function buttonByLabel(container: Element, label: string): HTMLButtonElement {
  const button = container.querySelector(`button[aria-label="${label}"]`);
  if (!(button instanceof HTMLButtonElement)) {
    throw new Error(`Button not found: ${label}`);
  }
  return button;
}

function expectedStandardExportDefaults(): string {
  const sampleRateKhz = STANDARD_EXPORT_DELIVERY.sampleRate / 1_000;
  const sampleRate =
    Number.isInteger(sampleRateKhz) ? `${sampleRateKhz} kHz` : `${sampleRateKhz.toFixed(1)} kHz`;
  const ceiling =
    STANDARD_EXPORT_DELIVERY.ceilingDbtp < 0
      ? `−${Math.abs(STANDARD_EXPORT_DELIVERY.ceilingDbtp)} dBTP`
      : `${STANDARD_EXPORT_DELIVERY.ceilingDbtp} dBTP`;
  return `${sampleRate}, ${STANDARD_EXPORT_DELIVERY.bitDepth}-bit WAV, ${ceiling}`;
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("top chrome", () => {
  it("wires undo and redo header buttons to history state", async () => {
    const onUndo = vi.fn();
    const onRedo = vi.fn();
    const baseProps = {
      mode: "track" as const,
      onModeChange: vi.fn(),
      onSaveProject: vi.fn(),
      onOpenProject: vi.fn(),
      onOpenSettings: vi.fn(),
      onOpenHelp: vi.fn(),
      viewMode: "standard" as const,
      onEnterAdvanced: vi.fn(),
      onBackToStandard: vi.fn(),
      onUndo,
      onRedo,
    };
    const { container, root } = await renderNode(
      <TopHeader {...baseProps} canUndo={false} canRedo={true} />,
    );

    const undo = buttonByLabel(container, "Undo — Ctrl+Z");
    const redo = buttonByLabel(container, "Redo — Ctrl+Y");
    expect(undo.disabled).toBe(true);
    expect(undo.title).toBe("Undo — Ctrl+Z");
    expect(redo.disabled).toBe(false);
    expect(redo.title).toBe("Redo — Ctrl+Y");

    await act(async () => {
      undo.click();
      redo.click();
    });
    expect(onUndo).not.toHaveBeenCalled();
    expect(onRedo).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.render(<TopHeader {...baseProps} canUndo={true} canRedo={false} />);
    });
    expect(buttonByLabel(container, "Undo — Ctrl+Z").disabled).toBe(false);
    expect(buttonByLabel(container, "Redo — Ctrl+Y").disabled).toBe(true);

    await act(async () => {
      buttonByLabel(container, "Undo — Ctrl+Z").click();
      buttonByLabel(container, "Redo — Ctrl+Y").click();
    });
    expect(onUndo).toHaveBeenCalledTimes(1);
    expect(onRedo).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.unmount();
    });
  });

  it("wires Settings and Help as active top-header buttons", async () => {
    const onOpenSettings = vi.fn();
    const onOpenHelp = vi.fn();
    const { container, root } = await renderNode(
      <TopHeader
        mode="track"
        onModeChange={vi.fn()}
        onSaveProject={vi.fn()}
        onOpenProject={vi.fn()}
        onOpenSettings={onOpenSettings}
        onOpenHelp={onOpenHelp}
        viewMode="advanced"
        onEnterAdvanced={vi.fn()}
        onBackToStandard={vi.fn()}
        canUndo={false}
        canRedo={false}
        onUndo={vi.fn()}
        onRedo={vi.fn()}
      />,
    );

    await act(async () => {
      buttonByLabel(container, "Settings").click();
      buttonByLabel(container, "Help").click();
    });

    expect(onOpenSettings).toHaveBeenCalledTimes(1);
    expect(onOpenHelp).toHaveBeenCalledTimes(1);
    await act(async () => {
      root.unmount();
    });
  });

  it("renders baseline Settings content without mastering-setting controls", async () => {
    const onClose = vi.fn();
    const { container, root } = await renderNode(
      <SettingsPanel onClose={onClose} />,
    );

    expect(container.textContent).toContain("Audio Preview");
    expect(container.textContent).toContain("Current defaults");
    expect(container.textContent).toContain("Standard · Create Master");
    expect(container.textContent).toContain(expectedStandardExportDefaults());
    expect(container.textContent).toContain("Advanced · delivery profile");
    expect(container.textContent).toContain("Streaming Universal — 48 kHz, 24-bit WAV");
    expect(container.textContent).toContain(".ams.json Save As / Open");

    await act(async () => {
      buttonByLabel(container, "Close Settings").click();
    });

    expect(onClose).toHaveBeenCalledTimes(1);
    await act(async () => {
      root.unmount();
    });
  });

  it("renders contextual Help sections for current release behavior", async () => {
    const onClose = vi.fn();
    const { container, root } = await renderNode(<HelpPanel onClose={onClose} />);

    const text = container.textContent ?? "";
    expect(text.indexOf("Standard view")).toBeLessThan(text.indexOf("Import / Analyze"));
    expect(text).toContain("Styles");
    expect(text).toContain("Low / Medium / High");
    expect(text).toContain("Create Master");
    expect(text).toContain("next to the source");
    expect(text).toContain("Keyboard shortcuts");
    expect(text).toContain("Space");
    expect(text).toContain("Ctrl/Cmd+Z/Y");
    expect(text).toContain("Shift+drag loop region");
    expect(text).toContain("Advanced only");
    expect(text).toContain("LUFS");
    expect(text).toContain("dBTP");
    expect(text).toContain("dynamic range");
    expect(container.textContent).toContain("Original vs Mastered");
    expect(container.textContent).toContain("Volume Match / Preview LUFS");
    expect(container.textContent).toContain("Delivery Profile / Format");
    expect(container.textContent).toContain("Export Review");

    await act(async () => {
      buttonByLabel(container, "Close Help").click();
    });

    expect(onClose).toHaveBeenCalledTimes(1);
    await act(async () => {
      root.unmount();
    });
  });
});

describe("advanced track header", () => {
  it("wires the manual re-analyze action", async () => {
    const onReanalyze = vi.fn();
    const track: ImportedTrack = {
      id: "track-1",
      path: "C:/audio/track-1.wav",
      display_name: "Track One.wav",
      source_format: "wav",
      duration_seconds: 120,
      sample_rate: 44_100,
      channels: 2,
    };
    const { container, root } = await renderNode(
      <TrackHeader
        track={track}
        analysis={undefined}
        playbackKind="master"
        volumeMatch={false}
        exportLufsPreview={false}
        isAnalyzing={false}
        analysisProgress={null}
        isRendering={false}
        isPlaying={false}
        renderProgress={null}
        onPlaybackKindChange={vi.fn()}
        onVolumeMatchChange={vi.fn()}
        onExportLufsPreviewChange={vi.fn()}
        onReanalyze={onReanalyze}
      />,
    );

    const reanalyze = container.querySelector<HTMLButtonElement>(".track-reanalyze")!;
    expect(reanalyze.textContent).toContain("Re-analyze");
    expect(reanalyze.disabled).toBe(false);

    await act(async () => {
      reanalyze.click();
    });
    expect(onReanalyze).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.render(
        <TrackHeader
          track={track}
          analysis={undefined}
          playbackKind="master"
          volumeMatch={false}
          exportLufsPreview={false}
          isAnalyzing={true}
          analysisProgress={{ label: "Analyzing audio", progress: 0.14 }}
          isRendering={false}
          isPlaying={false}
          renderProgress={null}
          onPlaybackKindChange={vi.fn()}
          onVolumeMatchChange={vi.fn()}
          onExportLufsPreviewChange={vi.fn()}
          onReanalyze={onReanalyze}
        />,
      );
    });
    expect(container.querySelector<HTMLButtonElement>(".track-reanalyze")?.disabled).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });
});

describe("advanced preset tiles", () => {
  it("shows unified preset names — first four mirror Standard, no alias suffix", async () => {
    const { container, root } = await renderNode(
      <PresetTiles
        selected={{ kind: "universal" }}
        onChange={vi.fn()}
        savingPreset={false}
        onSave={vi.fn()}
      />,
    );

    const text = container.textContent ?? "";
    // Both views now use the canonical names — the old "· … in Standard"
    // alias suffix is gone (preset-name unification).
    for (const name of ["Universal", "Clarity", "Tape", "Oomph"]) {
      expect(text).toContain(name);
    }
    expect(text).not.toContain("in Standard");
    expect(text).not.toContain("Safe, well-rounded default");
    expect(text).not.toContain("Vocal/upper-mid definition");
    expect(text).not.toContain("Saturation, glue, softer top");
    expect(text).not.toContain("Low-end weight, punch");
    // The four Standard tiles lead the Advanced list in the same order, so
    // Oomph sits ahead of Spatial (the unification ordering decision).
    expect(text.indexOf("Oomph")).toBeGreaterThan(-1);
    expect(text.indexOf("Oomph")).toBeLessThan(text.indexOf("Spatial"));
    await act(async () => {
      root.unmount();
    });
  });
});
