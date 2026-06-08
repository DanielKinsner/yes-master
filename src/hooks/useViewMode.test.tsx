// src/hooks/useViewMode.test.tsx
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useViewMode } from "./useViewMode";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

function Harness({ hadPriorSession, onRender }: { hadPriorSession: boolean | null; onRender: (v: ReturnType<typeof useViewMode>) => void }) {
  onRender(useViewMode(hadPriorSession));
  return null;
}

async function renderWith(hadPriorSession: boolean | null): Promise<{ current: () => ReturnType<typeof useViewMode>; root: Root; rerender: (next: boolean | null) => Promise<void> }> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  let current: ReturnType<typeof useViewMode> | null = null;
  const root = createRoot(container);
  const onRender = (v: ReturnType<typeof useViewMode>) => { current = v; };
  await act(async () => { root.render(<Harness hadPriorSession={hadPriorSession} onRender={onRender} />); });
  return {
    current: () => { if (!current) throw new Error("not rendered"); return current; },
    root,
    rerender: async (next) => { await act(async () => { root.render(<Harness hadPriorSession={next} onRender={onRender} />); }); },
  };
}

beforeEach(() => { localStorage.clear(); });
afterEach(() => { document.body.innerHTML = ""; });

describe("useViewMode", () => {
  it("new user (no flag, no prior session) resolves to standard and persists migration", async () => {
    const h = await renderWith(false);
    expect(h.current().view).toBe("standard");
    expect(JSON.parse(localStorage.getItem("yes-master:view-mode")!)).toEqual({ migrated: true, lastView: "standard" });
    await act(async () => h.root.unmount());
  });

  it("returning user (no flag, prior session) resolves to advanced", async () => {
    const h = await renderWith(true);
    expect(h.current().view).toBe("advanced");
    await act(async () => h.root.unmount());
  });

  it("holds view=null until hadPriorSession is known, then resolves", async () => {
    const h = await renderWith(null);
    expect(h.current().view).toBeNull();
    await h.rerender(true);
    expect(h.current().view).toBe("advanced");
    await act(async () => h.root.unmount());
  });

  it("once migrated, reopens last-used view immediately and ignores prior session", async () => {
    localStorage.setItem("yes-master:view-mode", JSON.stringify({ migrated: true, lastView: "standard" }));
    const h = await renderWith(true);
    expect(h.current().view).toBe("standard");
    await act(async () => h.root.unmount());
  });

  it("setView persists the new view", async () => {
    const h = await renderWith(false);
    await act(async () => { h.current().setView("advanced"); });
    expect(h.current().view).toBe("advanced");
    expect(JSON.parse(localStorage.getItem("yes-master:view-mode")!)).toEqual({ migrated: true, lastView: "advanced" });
    await act(async () => h.root.unmount());
  });
});
