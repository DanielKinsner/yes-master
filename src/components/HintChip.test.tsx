// src/components/HintChip.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { HintChip } from "./HintChip";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("HintChip", () => {
  it("renders its copy as a polite status", async () => {
    const { container, root } = await render(
      <HintChip onDismiss={() => {}}>Flip to Mastered</HintChip>,
    );
    const chip = container.querySelector(".hint-chip");
    expect(chip?.getAttribute("role")).toBe("status");
    expect(chip?.textContent).toContain("Flip to Mastered");
    await act(async () => root.unmount());
  });

  it("routes the × to onDismiss", async () => {
    const onDismiss = vi.fn();
    const { container, root } = await render(
      <HintChip onDismiss={onDismiss}>hi</HintChip>,
    );
    const x = container.querySelector<HTMLButtonElement>(".hint-chip-x")!;
    await act(async () => { x.click(); });
    expect(onDismiss).toHaveBeenCalledOnce();
    await act(async () => root.unmount());
  });
});
