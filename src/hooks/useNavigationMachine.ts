// src/hooks/useNavigationMachine.ts
//
// Wires lib/navigation-machine (the pure legal-state table) to the app:
// resolves the initial view through lib/view-mode's migration logic, keeps
// the machine's context in sync with useTrackMaster, persists every view
// change, and performs the one sanctioned side effect (leaving Album mode
// when an EXPLICIT return to Standard lands) at the dispatch site —
// synchronously, so no observer can ever see Standard+Album in between
// (the 2a78f4a regression class).

import { useCallback, useEffect, useReducer, useRef } from "react";

import {
  isReturnConfirmPending,
  navReduce,
  navView,
  UNRESOLVED,
  type NavAction,
  type NavContext,
  type NavState,
} from "../lib/navigation-machine";
import {
  browserViewModeStore,
  readPersistedViewMode,
  resolveInitialViewMode,
  writePersistedViewMode,
  type ViewMode,
} from "../lib/view-mode";

/// Only the explicit doors may leave Album mode. `context-changed` can also
/// LAND in standard (it never does today — album context forces Advanced),
/// and a restored album project transiently renders as standard+album for
/// one frame; flipping the mode there would silently un-album a restored
/// project, so drift actions never trigger the side effect.
function isExplicitReturn(action: NavAction): boolean {
  return (
    action.type === "request-back-to-standard" ||
    action.type === "complete-return" ||
    (action.type === "set-view" && action.view === "standard")
  );
}

export function useNavigationMachine(args: {
  hadPriorSession: boolean | null;
  isAlbum: boolean;
  hasTrack: boolean;
  hasNonManagedEdits: boolean;
  leaveAlbumMode: () => void;
}): {
  view: ViewMode | null;
  setView: (next: ViewMode) => void;
  returnConfirmOpen: boolean;
  requestBackToStandard: () => void;
  cancelReturn: () => void;
  completeReturn: () => void;
} {
  const storeRef = useRef(browserViewModeStore());

  // The machine reads context at dispatch time through a ref so `dispatch`
  // stays referentially stable. navReduce is pure per call, so StrictMode's
  // reducer double-invocation is safe.
  const ctx: NavContext = {
    isAlbum: args.isAlbum,
    hasTrack: args.hasTrack,
    hasNonManagedEdits: args.hasNonManagedEdits,
  };
  const ctxRef = useRef(ctx);
  ctxRef.current = ctx;

  const [state, dispatch] = useReducer(
    (prev: NavState, action: NavAction) => navReduce(prev, action, ctxRef.current),
    undefined,
    (): NavState => {
      // Steady state resolves synchronously on first render so the migrated
      // case never flashes (same contract useViewMode had). Context at mount
      // is empty (no tracks restored yet), so a persisted "standard" cannot
      // be force-bounced here; the context-changed effect re-checks once the
      // session restore lands.
      const persisted = readPersistedViewMode(storeRef.current);
      if (persisted && persisted.migrated) {
        return navReduce(
          UNRESOLVED,
          { type: "resolve", view: persisted.lastView },
          { isAlbum: false, hasTrack: false, hasNonManagedEdits: false },
        );
      }
      return UNRESOLVED;
    },
  );

  const stateRef = useRef(state);
  stateRef.current = state;
  const leaveAlbumModeRef = useRef(args.leaveAlbumMode);
  leaveAlbumModeRef.current = args.leaveAlbumMode;

  // First-launch migration: wait for the session signal, then resolve.
  useEffect(() => {
    if (state.kind !== "unresolved") return;
    if (args.hadPriorSession === null) return;
    const persisted = readPersistedViewMode(storeRef.current);
    const { view, persist } = resolveInitialViewMode(persisted, args.hadPriorSession);
    writePersistedViewMode(storeRef.current, persist);
    dispatch({ type: "resolve", view });
  }, [state.kind, args.hadPriorSession]);

  // Legality re-check whenever the machine's inputs move (mode change, track
  // switch, settings edit, session restore). Replaces the App entry-guard
  // effect: the decision now lives in the reducer's table.
  useEffect(() => {
    dispatch({ type: "context-changed" });
  }, [args.isAlbum, args.hasTrack, args.hasNonManagedEdits]);

  // Persist the resolved view on every change (the old setView wrote only on
  // explicit navigation; bounced/derived transitions persist too, so
  // localStorage can never disagree with the in-session view).
  const view = navView(state);
  const lastPersistedRef = useRef<ViewMode | null>(null);
  useEffect(() => {
    if (view === null || lastPersistedRef.current === view) return;
    lastPersistedRef.current = view;
    writePersistedViewMode(storeRef.current, { migrated: true, lastView: view });
  }, [view]);

  /// Dispatch + the sanctioned side effect, computed against the SAME
  /// transition the reducer will apply (UI actions are discrete, so the
  /// rendered state in stateRef is the reducer's current state).
  const dispatchNav = useCallback((action: NavAction) => {
    if (isExplicitReturn(action) && ctxRef.current.isAlbum) {
      const next = navReduce(stateRef.current, action, ctxRef.current);
      if (next.kind === "standard-track") {
        leaveAlbumModeRef.current();
      }
    }
    dispatch(action);
  }, []);

  const setView = useCallback(
    (next: ViewMode) => dispatchNav({ type: "set-view", view: next }),
    [dispatchNav],
  );
  const requestBackToStandard = useCallback(
    () => dispatchNav({ type: "request-back-to-standard" }),
    [dispatchNav],
  );
  const cancelReturn = useCallback(
    () => dispatchNav({ type: "cancel-return" }),
    [dispatchNav],
  );
  const completeReturn = useCallback(
    () => dispatchNav({ type: "complete-return" }),
    [dispatchNav],
  );

  return {
    view,
    setView,
    returnConfirmOpen: isReturnConfirmPending(state),
    requestBackToStandard,
    cancelReturn,
    completeReturn,
  };
}
