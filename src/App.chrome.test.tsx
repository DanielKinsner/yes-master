import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { HelpPanel, SettingsPanel, TopHeader } from "./App";

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
    expect(container.textContent).toContain("48 kHz, 24-bit WAV");
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
