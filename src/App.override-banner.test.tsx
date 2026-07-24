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

/** Visible chip text — sr-only content removed, the way a sighted user reads it. */
function visibleText(el: Element | null): string {
  if (!el) return "";
  const clone = el.cloneNode(true) as HTMLElement;
  clone.querySelectorAll(".sr-only").forEach((n) => n.remove());
  return (clone.textContent ?? "").trim();
}

describe("OverrideBanner (Slice 13 chip + U10 segmented choice)", () => {
  // U9/U10 (2026-07-24) CHANGED THIS CONTRACT DELIBERATELY.
  //
  // This control used to put `disabled` on whichever side was ACTIVE, and
  // these tests pinned that. It is backwards twice over: a disabled control
  // leaves the tab order, so a keyboard user cannot focus their current state
  // to find out what it is, and a screen reader announces the SELECTED option
  // as "unavailable". The plan requires two-state choices to be represented as
  // selected state, not as absence of availability.
  //
  // Both buttons are now enabled and `aria-pressed` carries the choice.
  // Re-selecting the active side is a no-op. The assertions below pin strictly
  // MORE behavior than the old ones did: state, availability, that the active
  // side does not fire, and that the inactive side does.

  it("shows a Follows-album chip and makes both sides focusable", async () => {
    const onToggle = vi.fn();
    const { container } = await render(
      <OverrideBanner isOverriding={false} onToggle={onToggle} />,
    );
    const chip = container.querySelector<HTMLElement>(".override-status");
    expect(visibleText(chip)).toBe("Follows album");
    expect(chip?.classList.contains("status-pill")).toBe(true);
    expect(chip?.classList.contains("is-overriding")).toBe(false);
    expect(chip?.getAttribute("title")).toContain("follows album intent");
    // The explanation must also exist as text, not only as a tooltip.
    expect(chip?.textContent).toContain("follows album intent");

    const follow = buttonByText(container, "Follow album");
    const override = buttonByText(container, "Override");
    expect(follow.getAttribute("aria-pressed")).toBe("true");
    expect(override.getAttribute("aria-pressed")).toBe("false");
    // Neither side may be disabled — that is the defect this replaces.
    expect(follow.disabled).toBe(false);
    expect(override.disabled).toBe(false);

    // Re-selecting the ACTIVE side changes nothing.
    await act(async () => {
      follow.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onToggle).not.toHaveBeenCalled();

    // Selecting the other side switches.
    await act(async () => {
      override.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("shows an Overrides-album chip and flips which side is pressed", async () => {
    const onToggle = vi.fn();
    const { container } = await render(
      <OverrideBanner isOverriding={true} onToggle={onToggle} />,
    );
    const chip = container.querySelector<HTMLElement>(".override-status");
    expect(visibleText(chip)).toBe("Overrides album");
    expect(chip?.classList.contains("is-overriding")).toBe(true);
    expect(chip?.getAttribute("title")).toContain("overrides album intent");
    expect(chip?.textContent).toContain("overrides album intent");

    const follow = buttonByText(container, "Follow album");
    const override = buttonByText(container, "Override");
    expect(override.getAttribute("aria-pressed")).toBe("true");
    expect(follow.getAttribute("aria-pressed")).toBe("false");
    expect(follow.disabled).toBe(false);
    expect(override.disabled).toBe(false);

    await act(async () => {
      override.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onToggle).not.toHaveBeenCalled();

    await act(async () => {
      follow.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("groups the two sides as one labelled choice", async () => {
    const { container } = await render(
      <OverrideBanner isOverriding={false} onToggle={vi.fn()} />,
    );
    const group = container.querySelector(".override-toggle");
    expect(group?.getAttribute("role")).toBe("group");
    expect(group?.getAttribute("aria-label")).toBeTruthy();
    // Exactly one side pressed at any time.
    const pressed = Array.from(
      container.querySelectorAll(".override-toggle button"),
    ).filter((b) => b.getAttribute("aria-pressed") === "true");
    expect(pressed).toHaveLength(1);
  });
});
