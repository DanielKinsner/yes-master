// src/hooks/useViewMode.ts
import { useCallback, useEffect, useRef, useState } from "react";

import {
  browserViewModeStore,
  readPersistedViewMode,
  resolveInitialViewMode,
  writePersistedViewMode,
  type ViewMode,
} from "../lib/view-mode";

/// Owns the Standard/Advanced view with a migration-aware default.
/// `hadPriorSession` comes from useTrackMaster's session load:
///   null  = not yet known (still loading)
///   true  = a prior session with tracks was restored (returning user)
///   false = no restorable session (new user)
/// `view` stays null until resolvable, so the caller can avoid flashing.
export function useViewMode(hadPriorSession: boolean | null): {
  view: ViewMode | null;
  setView: (next: ViewMode) => void;
} {
  const storeRef = useRef(browserViewModeStore());
  const resolvedRef = useRef(false);
  const [view, setViewState] = useState<ViewMode | null>(() => {
    // If we've already migrated, resolve synchronously on first render so
    // the steady-state case never flashes.
    const persisted = readPersistedViewMode(storeRef.current);
    if (persisted && persisted.migrated) {
      resolvedRef.current = true;
      return persisted.lastView;
    }
    return null;
  });

  useEffect(() => {
    if (resolvedRef.current) return;
    if (hadPriorSession === null) return; // wait for the signal
    const persisted = readPersistedViewMode(storeRef.current);
    const { view: resolved, persist } = resolveInitialViewMode(persisted, hadPriorSession);
    resolvedRef.current = true;
    writePersistedViewMode(storeRef.current, persist);
    setViewState(resolved);
  }, [hadPriorSession]);

  const setView = useCallback((next: ViewMode) => {
    resolvedRef.current = true;
    writePersistedViewMode(storeRef.current, { migrated: true, lastView: next });
    setViewState(next);
  }, []);

  return { view, setView };
}
