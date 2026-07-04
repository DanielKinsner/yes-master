// src/components/FirstRunOverlay.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FirstRunOverlay } from "./FirstRunOverlay";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("FirstRunOverlay", () => {
  it("renders nothing when the step is null", async () => {
    const { container, root } = await render(
      <FirstRunOverlay step={null} onDismiss={() => {}} />,
    );
    expect(container.querySelector(".first-run-overlay")).toBeNull();
    expect(container.querySelector(".hint-chip")).toBeNull();
    await act(async () => root.unmount());
  });

  // The copy must match the inline chips it replaces — each step keeps its
  // hint-chip-<step> marker class so existing selectors/styling still apply.
  it.each([
    ["flip", "Mastered", "hint-chip-flip"],
    ["sendoff", "Presets and Intensity", "hint-chip-sendoff"],
    ["advanced", "Advanced", "hint-chip-advanced"],
  ] as const)(
    "renders the %s hint copy inside a reused HintChip",
    async (step, copy, chipClass) => {
      const { container, root } = await render(
        <FirstRunOverlay step={step} onDismiss={() => {}} />,
      );
      const chip = container.querySelector(`.${chipClass}`);
      expect(chip).not.toBeNull();
      expect(chip?.classList.contains("hint-chip")).toBe(true);
      expect(chip?.getAttribute("role")).toBe("status");
      expect(container.textContent).toContain(copy);
      await act(async () => root.unmount());
    },
  );

  it("floats fixed at the overlay z-index with a per-step placement hook", async () => {
    const { container, root } = await render(
      <FirstRunOverlay step="flip" onDismiss={() => {}} />,
    );
    const overlay = container.querySelector<HTMLElement>(".first-run-overlay")!;
    // Inline so the float/stacking holds regardless of stylesheet load order;
    // per-step offsets (top/right/bottom) live in App.css keyed off the
    // first-run-overlay-<step> class below.
    expect(overlay.style.position).toBe("fixed");
    // Shared overlay tier (--z-overlay) with the literal as fallback, so
    // this layer can't drift from .chrome-dialog-backdrop's.
    expect(overlay.style.zIndex).toBe("var(--z-overlay, 120)");
    expect(overlay.classList.contains("first-run-overlay-flip")).toBe(true);
    await act(async () => root.unmount());
  });

  it("routes the dismiss × to onDismiss", async () => {
    const onDismiss = vi.fn();
    const { container, root } = await render(
      <FirstRunOverlay step="sendoff" onDismiss={onDismiss} />,
    );
    const x = container.querySelector<HTMLButtonElement>(".hint-chip-x")!;
    await act(async () => { x.click(); });
    expect(onDismiss).toHaveBeenCalledOnce();
    await act(async () => root.unmount());
  });
});
