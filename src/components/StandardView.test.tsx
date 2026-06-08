// src/components/StandardView.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { StyleTiles } from "./StandardView";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("StyleTiles", () => {
  it("renders the four reference-4 tiles with labels", async () => {
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "universal" }} onSelect={() => {}} />,
    );
    const text = container.textContent ?? "";
    for (const label of ["Balanced", "Bright", "Warm", "Heavy"]) {
      expect(text).toContain(label);
    }
    await act(async () => root.unmount());
  });

  it("marks the active tile from the current preset", async () => {
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "tape" }} onSelect={() => {}} />,
    );
    const active = container.querySelector(".std-tile.is-active");
    expect(active?.textContent).toContain("Warm");
    await act(async () => root.unmount());
  });

  it("calls onSelect with the mapped preset when a tile is clicked", async () => {
    const onSelect = vi.fn();
    const { container, root } = await render(
      <StyleTiles preset={{ kind: "universal" }} onSelect={onSelect} />,
    );
    const tiles = Array.from(container.querySelectorAll<HTMLButtonElement>(".std-tile"));
    const heavy = tiles.find((t) => t.textContent?.includes("Heavy"))!;
    await act(async () => { heavy.click(); });
    expect(onSelect).toHaveBeenCalledWith({ kind: "oomph" });
    await act(async () => root.unmount());
  });
});
