// Pass 3 (2026-08-19) — modified-from-default marker. A knob that opts in
// with `editedIndicator` shows a small dot beside its label while its value
// differs from `defaultValue`, so a glance at the deck/rail says which
// controls have been touched. Opt-in: Intensity is a creative choice, not an
// edit, and must not carry the dot.

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { Knob } from "./Knob";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.innerHTML = "";
});

async function mount(node: React.ReactElement): Promise<HTMLDivElement> {
  const host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  await act(async () => {
    root!.render(node);
  });
  return host;
}

describe("Knob edited indicator", () => {
  it("shows the dot only when opted in AND the value differs from defaultValue", async () => {
    const a = await mount(
      <Knob label="Low" value={0} min={-12} max={12} step={0.1} defaultValue={0} editedIndicator format={(v) => v.toFixed(1)} onChange={vi.fn()} />,
    );
    expect(a.querySelector(".knob-edited-dot")).toBeNull();
    expect(a.querySelector(".knob")?.classList.contains("is-edited")).toBe(false);

    const b = await mount(
      <Knob label="Low" value={2.5} min={-12} max={12} step={0.1} defaultValue={0} editedIndicator format={(v) => v.toFixed(1)} onChange={vi.fn()} />,
    );
    expect(b.querySelector(".knob-edited-dot")).not.toBeNull();
    expect(b.querySelector(".knob")?.classList.contains("is-edited")).toBe(true);
  });

  it("does not mark a knob that did not opt in (Intensity)", async () => {
    const host = await mount(
      <Knob label="Intensity" value={0.75} min={0} max={1} step={0.01} defaultValue={0.5} format={(v) => v.toFixed(2)} onChange={vi.fn()} />,
    );
    expect(host.querySelector(".knob-edited-dot")).toBeNull();
  });
});
