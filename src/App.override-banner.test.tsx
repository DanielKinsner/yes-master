import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { OverrideBanner } from "./App";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

// OverrideBanner is a presentational export from App.tsx; these mocks only keep
// importing "./App" side-effect-free (nothing here mounts the full app).
vi.mock("./lib/api", () => ({
  ADAPTIVE_COMPRESSION_GATE_EVENT: "yes-master:adaptive-compression-gate",
  api: {},
  onPlaybackTick: vi.fn(),
  onPlaybackDeviceLost: vi.fn(),
  onRenderProgress: vi.fn(),
  onLandingStatus: vi.fn(),
  onAnalysisProgress: vi.fn(),
  onUpdaterAvailable: vi.fn(),
}));

vi.mock("./lib/tauri-runtime", () => ({
  open: vi.fn(),
  save: vi.fn(),
  getCurrentWebview: () => ({ onDragDropEvent: vi.fn() }),
}));

async function render(node: ReactNode): Promise<{
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

function buttonByText(container: HTMLElement, text: string): HTMLButtonElement {
  const button = Array.from(
    container.querySelectorAll<HTMLButtonElement>(".override-toggle button"),
  ).find((b) => (b.textContent ?? "").trim() === text);
  if (!button) throw new Error(`button not found: ${text}`);
  return button;
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("OverrideBanner (Slice 13 — status chip + compact toggle)", () => {
  it("shows a Follows-album chip with the explanation in its tooltip", async () => {
    const onToggle = vi.fn();
    const { container } = await render(
      <OverrideBanner isOverriding={false} onToggle={onToggle} />,
    );
    const chip = container.querySelector<HTMLElement>(".override-status");
    expect(chip?.textContent).toBe("Follows album");
    expect(chip?.classList.contains("status-pill")).toBe(true);
    expect(chip?.classList.contains("is-overriding")).toBe(false);
    expect(chip?.getAttribute("title")).toContain("follows album intent");

    // Follow is the active/disabled state; Override is the actionable button.
    const follow = buttonByText(container, "Follow album");
    const override = buttonByText(container, "Override");
    expect(follow.getAttribute("aria-pressed")).toBe("true");
    expect(follow.disabled).toBe(true);
    expect(override.getAttribute("aria-pressed")).toBe("false");
    expect(override.disabled).toBe(false);

    await act(async () => {
      override.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("shows an Overrides-album chip and flips the toggle's actionable side", async () => {
    const onToggle = vi.fn();
    const { container } = await render(
      <OverrideBanner isOverriding={true} onToggle={onToggle} />,
    );
    const chip = container.querySelector<HTMLElement>(".override-status");
    expect(chip?.textContent).toBe("Overrides album");
    expect(chip?.classList.contains("is-overriding")).toBe(true);
    expect(chip?.getAttribute("title")).toContain("overrides album intent");

    const follow = buttonByText(container, "Follow album");
    const override = buttonByText(container, "Override");
    expect(override.getAttribute("aria-pressed")).toBe("true");
    expect(override.disabled).toBe(true);
    expect(follow.getAttribute("aria-pressed")).toBe("false");
    expect(follow.disabled).toBe(false);

    await act(async () => {
      follow.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onToggle).toHaveBeenCalledTimes(1);
  });
});
