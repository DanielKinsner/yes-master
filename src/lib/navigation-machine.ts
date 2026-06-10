// src/lib/navigation-machine.ts
//
// The single legal-state machine for Standard/Advanced/Album navigation
// (consolidated backlog B5.1; merges the two surveys' #1 finding).
//
// Before this, navigation was split-brain: `view` lived in useViewMode,
// `mode` inside useTrackMaster, `returnConfirm` in App, and legality was
// enforced REACTIVELY by two App.tsx effects that corrected illegal states
// after render. That shape produced two regression classes in one week —
// the silent Album → Standard trap (2a78f4a: the return door set the view,
// the entry guard bounced it back, the button visibly did nothing) and the
// HANDOFF-2026-06-08 illegal-state loop. Here, every transition is a row in
// one pure table; illegal states are unrepresentable rather than repaired.
//
// The machine is pure and synchronous. Ownership stays where it was —
// useTrackMaster still owns `mode`, view persistence still goes through
// lib/view-mode — but every DECISION routes through `navReduce`, and the
// wiring hook (hooks/useNavigationMachine.ts) keeps the outside world in
// sync with the machine instead of the other way around.

import { forceAdvancedOnStandardEntry } from "./standard-managed";

export type NavStateKind =
  | "unresolved"
  | "standard-track"
  | "advanced-track"
  | "advanced-album"
  | "return-confirm-pending";

export interface NavState {
  kind: NavStateKind;
}

/// Everything a transition decision may read. The wiring hook derives this
/// from useTrackMaster on every render; actions never carry stale copies.
export interface NavContext {
  isAlbum: boolean;
  hasTrack: boolean;
  hasNonManagedEdits: boolean;
}

export type NavAction =
  /// `hadPriorSession` resolved — leave `unresolved` for the given view
  /// (already decided by lib/view-mode's migration logic).
  | { type: "resolve"; view: "standard" | "advanced" }
  /// Explicit tab/door navigation (TopHeader "Advanced", StandardView's
  /// "Go to Advanced", and any future direct setView caller).
  | { type: "set-view"; view: "standard" | "advanced" }
  /// tm.mode / selected track / settings changed — re-check legality of the
  /// current state (the "enforced at EVERY entry" invariant, now also
  /// enforced mid-stay by the same table row).
  | { type: "context-changed" }
  /// The Back-to-Standard door. Clean track (or no track) passes straight
  /// through; non-managed edits open the confirm dialog.
  | { type: "request-back-to-standard" }
  /// Confirm dialog: cancel keeps the user in Advanced.
  | { type: "cancel-return" }
  /// Confirm dialog: Reset-&-continue or a successful Save-as-preset. The
  /// caller performs those side effects BEFORE dispatching.
  | { type: "complete-return" };

export const UNRESOLVED: NavState = { kind: "unresolved" };

function advancedFor(ctx: NavContext): NavState {
  return { kind: ctx.isAlbum ? "advanced-album" : "advanced-track" };
}

/// Standard entry always passes the always-clean / Album-only-in-Advanced
/// gate; an illegal entry lands in the matching Advanced state instead.
function standardEntry(ctx: NavContext): NavState {
  return forceAdvancedOnStandardEntry(ctx) ? advancedFor(ctx) : { kind: "standard-track" };
}

export function navReduce(state: NavState, action: NavAction, ctx: NavContext): NavState {
  switch (state.kind) {
    case "unresolved":
      // Nothing is navigable until the session signal resolves; every other
      // action is a no-op by design (the loading-flicker guard).
      if (action.type === "resolve") {
        return action.view === "standard" ? standardEntry(ctx) : advancedFor(ctx);
      }
      return state;

    case "standard-track":
      switch (action.type) {
        case "set-view":
          return action.view === "standard" ? standardEntry(ctx) : advancedFor(ctx);
        case "context-changed":
          // Album switched on, a dirty track restored/selected, etc. — the
          // invariant that produced the entry-guard effect, as a table row.
          return standardEntry(ctx);
        case "request-back-to-standard":
          return state; // already there
        default:
          return state;
      }

    case "advanced-track":
    case "advanced-album":
      switch (action.type) {
        case "set-view":
          return action.view === "standard" ? standardEntry(ctx) : advancedFor(ctx);
        case "context-changed":
          return advancedFor(ctx); // keep the album/track shape truthful
        case "request-back-to-standard":
          // Spec §2a: asymmetric door — silent when clean, confirm when
          // non-managed edits would be reset. NOTE the decision ignores
          // isAlbum: leaving Album is allowed and the caller flips
          // tm.setMode("track") alongside (2a78f4a class, fixed by
          // construction: there is no separate guard left to re-bounce).
          if (!ctx.hasTrack || !ctx.hasNonManagedEdits) {
            return { kind: "standard-track" };
          }
          return { kind: "return-confirm-pending" };
        default:
          return state;
      }

    case "return-confirm-pending":
      switch (action.type) {
        case "cancel-return":
          return advancedFor(ctx);
        case "complete-return":
          // Reset / save-as-preset already happened caller-side, so the
          // track is clean again and Standard entry is legal.
          return { kind: "standard-track" };
        case "set-view":
          // Unreachable from the UI (modal covers the chrome) but the
          // machine stays total: explicit navigation cancels the pending
          // confirm rather than leaving a ghost dialog.
          return action.view === "standard" ? standardEntry(ctx) : advancedFor(ctx);
        case "context-changed":
          return state; // the dialog owns the screen while the user decides
        default:
          return state;
      }
  }
}

/// The view the chrome should render. `null` = still resolving (callers keep
/// their loading guards). `return-confirm-pending` renders Advanced under
/// the dialog, exactly like the old `returnConfirm` boolean did.
export function navView(state: NavState): "standard" | "advanced" | null {
  switch (state.kind) {
    case "unresolved":
      return null;
    case "standard-track":
      return "standard";
    default:
      return "advanced";
  }
}

export function isReturnConfirmPending(state: NavState): boolean {
  return state.kind === "return-confirm-pending";
}
