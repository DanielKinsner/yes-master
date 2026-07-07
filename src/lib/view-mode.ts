// src/lib/view-mode.ts
//
// Standard vs Advanced, with a versioned, migration-aware default
// persisted in localStorage (design spec §7). Mirrors the tiny
// store-interface pattern in export-location.ts so it stays testable
// without a real DOM.

// ViewMode is the Rust↔TS wire type (persisted per track in ProjectState);
// re-export from bindings so there is a single source of truth.
import type { ViewMode } from "../bindings";
export type { ViewMode };

export interface PersistedViewState {
  migrated: boolean;
  lastView: ViewMode;
}

export interface ViewModeStore {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

const KEY = "yes-master:view-mode";

export function browserViewModeStore(): ViewModeStore | null {
  try {
    const storage = globalThis.localStorage;
    if (storage && typeof storage.getItem === "function" && typeof storage.setItem === "function") {
      return storage;
    }
  } catch {
    /* localStorage can throw in locked-down webviews; treat as absent */
  }
  return null;
}

export function readPersistedViewMode(store: ViewModeStore | null): PersistedViewState | null {
  if (!store) return null;
  const raw = store.getItem(KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<PersistedViewState>;
    if (
      typeof parsed.migrated === "boolean" &&
      (parsed.lastView === "standard" || parsed.lastView === "advanced")
    ) {
      return { migrated: parsed.migrated, lastView: parsed.lastView };
    }
  } catch {
    /* malformed — treat as absent */
  }
  return null;
}

export function writePersistedViewMode(store: ViewModeStore | null, state: PersistedViewState): void {
  if (!store) return;
  try {
    store.setItem(KEY, JSON.stringify(state));
  } catch {
    /* best-effort; a failed write just means we re-resolve next launch */
  }
}

/// Decide the initial view. When already migrated, reopen the last-used
/// view. Otherwise this is the first launch after the update: returning
/// users (a prior session exists) stay in Advanced; brand-new users land
/// in Standard. Either way we return the `persist` state to write back.
export function resolveInitialViewMode(
  persisted: PersistedViewState | null,
  hadPriorSession: boolean,
): { view: ViewMode; persist: PersistedViewState } {
  if (persisted && persisted.migrated) {
    return { view: persisted.lastView, persist: persisted };
  }
  const view: ViewMode = hadPriorSession ? "advanced" : "standard";
  return { view, persist: { migrated: true, lastView: view } };
}
