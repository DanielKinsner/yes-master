// src/components/EmptyState.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EmptyState } from "./EmptyState";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("EmptyState welcome hero", () => {
  it("shows the brand, the pitch, and the idling orb", async () => {
    const { container, root } = await render(<EmptyState onAdd={() => {}} />);
    const text = container.textContent ?? "";
    expect(text).toContain("YES Master");
    expect(text).toContain("Drop audio");
    expect(container.querySelector(".empty-hero-orb canvas.wf-orb")).not.toBeNull();
    await act(async () => root.unmount());
  });

  it("routes the import button to onAdd", async () => {
    const onAdd = vi.fn();
    const { container, root } = await render(<EmptyState onAdd={onAdd} />);
    const btn = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((b) => b.textContent?.includes("Import audio"))!;
    await act(async () => { btn.click(); });
    expect(onAdd).toHaveBeenCalledOnce();
    await act(async () => root.unmount());
  });

  it("keeps the supported formats footnote", async () => {
    const { container, root } = await render(<EmptyState onAdd={() => {}} />);
    expect(container.textContent ?? "").toContain("WAV");
    expect(container.textContent ?? "").toContain("Opus");
    await act(async () => root.unmount());
  });
});
