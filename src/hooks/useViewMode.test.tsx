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

  it("a manual setView before the signal arrives is not clobbered by a late hadPriorSession", async () => {
    // Use a store whose reads stay empty so the only thing carrying the manual
    // choice across the late signal is the in-memory resolvedRef guard -- not a
    // storage re-read. (With real localStorage, setView's migrated:true write is
    // itself a backstop that masks the guard, so this isolates resolvedRef.)
    const writes: string[] = [];
    const realLocalStorage = globalThis.localStorage;
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      value: {
        getItem: () => null,
        setItem: (_k: string, v: string) => { writes.push(v); },
      },
    });
    try {
      // hadPriorSession not yet known + empty storage -> view holds at null.
      const h = await renderWith(null);
      expect(h.current().view).toBeNull();
      // User makes a manual choice before the session signal resolves.
      await act(async () => { h.current().setView("standard"); });
      expect(h.current().view).toBe("standard");
      // The "returning user" signal arrives late. A fresh resolve from storage
      // would yield "advanced" (getItem -> null, hadPriorSession=true); only the
      // resolvedRef guard keeps the manual choice intact.
      await h.rerender(true);
      expect(h.current().view).toBe("standard");
      // The manual choice was persisted as the migrated last-used view.
      expect(JSON.parse(writes.at(-1)!)).toEqual({ migrated: true, lastView: "standard" });
      await act(async () => h.root.unmount());
    } finally {
      Object.defineProperty(globalThis, "localStorage", { configurable: true, value: realLocalStorage });
    }
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
