// src/lib/view-mode.test.ts
import { describe, expect, it } from "vitest";

import {
  resolveInitialViewMode,
  readPersistedViewMode,
  writePersistedViewMode,
  type ViewModeStore,
} from "./view-mode";

function memStore(initial: Record<string, string> = {}): ViewModeStore {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (k) => map.get(k) ?? null,
    setItem: (k, v) => { map.set(k, v); },
  };
}

describe("resolveInitialViewMode (migration-aware default)", () => {
  it("new user (not migrated, no prior session) -> standard, and marks migrated", () => {
    const { view, persist } = resolveInitialViewMode(null, false);
    expect(view).toBe("standard");
    expect(persist).toEqual({ migrated: true, lastView: "standard" });
  });

  it("returning user (not migrated, prior session) -> advanced, and marks migrated", () => {
    const { view, persist } = resolveInitialViewMode(null, true);
    expect(view).toBe("advanced");
    expect(persist).toEqual({ migrated: true, lastView: "advanced" });
  });

  it("once migrated, reopens the last-used view regardless of prior session", () => {
    expect(resolveInitialViewMode({ migrated: true, lastView: "standard" }, true).view).toBe("standard");
    expect(resolveInitialViewMode({ migrated: true, lastView: "advanced" }, false).view).toBe("advanced");
  });
});

describe("view-mode persistence", () => {
  it("round-trips through the store", () => {
    const store = memStore();
    writePersistedViewMode(store, { migrated: true, lastView: "advanced" });
    expect(readPersistedViewMode(store)).toEqual({ migrated: true, lastView: "advanced" });
  });

  it("returns null for absent or malformed data", () => {
    expect(readPersistedViewMode(memStore())).toBeNull();
    expect(readPersistedViewMode(memStore({ "yes-master:view-mode": "not json" }))).toBeNull();
    expect(readPersistedViewMode(null)).toBeNull();
  });
});
