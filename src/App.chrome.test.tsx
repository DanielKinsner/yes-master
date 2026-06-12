import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { HelpPanel, SettingsPanel, TopHeader } from "./App";
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
