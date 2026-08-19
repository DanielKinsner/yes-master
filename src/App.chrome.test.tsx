import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  HelpPanel,
  PresetTiles,
  SettingsPanel,
  TopHeader,
  TrackHeader,
  type AudioOutputSettingsState,
  ShortcutsPanel,
} from "./App";
import type { ImportedTrack } from "./bindings";
import { STANDARD_EXPORT_DELIVERY } from "./lib/standard-export";
import { api } from "./lib/api";
import { save } from "./lib/tauri-runtime";

// HelpPanel's diagnostics flow talks to the save dialog; stub only `save`
// and keep the rest of the runtime module real.
vi.mock("./lib/tauri-runtime", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./lib/tauri-runtime")>();
  return { ...actual, save: vi.fn() };
});

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
    expect(container.textContent).toContain("Audio Output");
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

  it("renders an audio output selector in Settings", async () => {
    const onSelect = vi.fn().mockResolvedValue(undefined);
    const onRefresh = vi.fn().mockResolvedValue(undefined);
    const audioOutput: AudioOutputSettingsState = {
      devices: [
        { id: "Speakers", name: "Speakers", is_default: true, is_selected: false },
        { id: "Studio Monitor", name: "Studio Monitor", is_default: false, is_selected: true },
      ],
      selectedDeviceId: "Studio Monitor",
      isLoading: false,
      message: "Audio output saved.",
      error: null,
      onSelect,
      onRefresh,
    };

    const { container, root } = await renderNode(
      <SettingsPanel audioOutput={audioOutput} onClose={vi.fn()} />,
    );

    const select = container.querySelector("#audio-output-device");
    expect(select).toBeInstanceOf(HTMLSelectElement);
    expect((select as HTMLSelectElement).value).toBe("Studio Monitor");
    expect(container.textContent).toContain("System default (Speakers)");
    expect(container.textContent).toContain("Audio output saved.");

    await act(async () => {
      (select as HTMLSelectElement).value = "Speakers";
      select?.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(onSelect).toHaveBeenCalledWith("Speakers");

    const refresh = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Refresh",
    );
    expect(refresh).toBeInstanceOf(HTMLButtonElement);
    await act(async () => {
      (refresh as HTMLButtonElement).click();
    });
    expect(onRefresh).toHaveBeenCalledTimes(1);

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
    // 2026-07-06 audit: the old copy claimed the WAV lands "next to the
    // source", but Create Master opens a save dialog defaulting to the last
    // export folder — the copy now tells the truth (asks where to save).
    expect(text).toContain("asks where to save");
    expect(text).toContain("never overwrites your source");
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

  // Pass 4 (2026-08-19): the `?` overlay lists exactly the catalogue the
  // key handlers read, so it can never advertise a dead key.
  it("renders the Shortcuts overlay from the shared catalogue", async () => {
    const { SHORTCUTS } = await import("./lib/shortcuts");
    const onClose = vi.fn();
    const { container, root } = await renderNode(<ShortcutsPanel onClose={onClose} />);
    const text = container.textContent ?? "";
    for (const entry of SHORTCUTS) {
      expect(text).toContain(entry.label);
      for (const k of entry.keys) expect(text).toContain(k);
    }
    expect(container.querySelectorAll(".shortcut-row").length).toBe(SHORTCUTS.length);
    expect(container.querySelector(".shortcut-scope")?.textContent).toBe("Advanced");
    await act(async () => {
      buttonByLabel(container, "Close Shortcuts").click();
    });
    expect(onClose).toHaveBeenCalledTimes(1);
    await act(async () => {
      root.unmount();
    });
  });

  it("saves a diagnostics report from Help and confirms the location", async () => {
    vi.mocked(save).mockResolvedValue("C:/reports/diag.txt");
    const report = vi
      .spyOn(api, "saveDiagnosticsReport")
      .mockResolvedValue("C:/reports/diag.txt");
    const { container, root } = await renderNode(<HelpPanel onClose={vi.fn()} />);

    // The privacy promise is part of the panel copy.
    expect(container.textContent).toContain("Nothing is sent anywhere");

    const button = [...container.querySelectorAll("button")].find((b) =>
      b.textContent?.includes("Save diagnostics report"),
    );
    expect(button).toBeInstanceOf(HTMLButtonElement);
    await act(async () => {
      (button as HTMLButtonElement).click();
    });

    expect(report).toHaveBeenCalledWith("C:/reports/diag.txt");
    expect(container.textContent).toContain("Saved to C:/reports/diag.txt");

    report.mockRestore();
    await act(async () => {
      root.unmount();
    });
  });

  it("does nothing when the diagnostics save dialog is cancelled", async () => {
    vi.mocked(save).mockResolvedValue(null);
    const report = vi.spyOn(api, "saveDiagnosticsReport").mockResolvedValue("");
    const { container, root } = await renderNode(<HelpPanel onClose={vi.fn()} />);

    const button = [...container.querySelectorAll("button")].find((b) =>
      b.textContent?.includes("Save diagnostics report"),
    );
    await act(async () => {
      (button as HTMLButtonElement).click();
    });

    expect(report).not.toHaveBeenCalled();
    expect(container.textContent).not.toContain("Saved to");

    report.mockRestore();
    await act(async () => {
      root.unmount();
    });
  });
});

describe("advanced track header", () => {
  it("keeps analysis retry out of the waveform header", async () => {
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
        isAlbum={false}
        isOverriding={false}
        onToggleOverride={vi.fn()}
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
      />,
    );

    expect(container.querySelector<HTMLButtonElement>(".track-reanalyze")).toBeNull();

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

    // U9 (2026-07-24): this used to read `container.textContent`, which cannot
    // distinguish "not visible" from "not in the DOM". The tiles now carry
    // their character blurb as `.sr-only` text so screen-reader users get the
    // description that was previously hover-only — visually nothing changed.
    //
    // The visual assertion below is UNCHANGED in strictness: it still requires
    // that no blurb is visible on a tile. It is only measured correctly now,
    // against visible text rather than raw DOM text. The a11y-tree half is
    // asserted additionally, at the end of this test.
    const visible = container.cloneNode(true) as HTMLElement;
    visible.querySelectorAll(".sr-only").forEach((n) => n.remove());
    const text = visible.textContent ?? "";

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

    // ...and the blurb must nonetheless be reachable by assistive tech.
    expect(container.textContent ?? "").toContain("Safe, well-rounded default");
    // The four Standard tiles lead the Advanced list in the same order, so
    // Oomph sits ahead of Spatial (the unification ordering decision).
    expect(text.indexOf("Oomph")).toBeGreaterThan(-1);
    expect(text.indexOf("Oomph")).toBeLessThan(text.indexOf("Spatial"));
    await act(async () => {
      root.unmount();
    });
  });
});
