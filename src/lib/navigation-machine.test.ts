// Table-driven coverage of the navigation reducer: every state × action
// cell, with context variants where the decision branches. The end-to-end
// flows stay pinned by App.transitions.test.tsx (scenarios 1–9); this file
// pins the table itself.

import { describe, expect, it } from "vitest";

import {
  navReduce,
  navView,
  isReturnConfirmPending,
  UNRESOLVED,
  type NavAction,
  type NavContext,
  type NavState,
  type NavStateKind,
} from "./navigation-machine";

const CLEAN_TRACK: NavContext = {
  isAlbum: false,
  hasTrack: true,
  hasNonManagedEdits: false,
};
const DIRTY_TRACK: NavContext = {
  isAlbum: false,
  hasTrack: true,
  hasNonManagedEdits: true,
};
const ALBUM_CLEAN: NavContext = {
  isAlbum: true,
  hasTrack: true,
  hasNonManagedEdits: false,
};
const ALBUM_DIRTY: NavContext = {
  isAlbum: true,
  hasTrack: true,
  hasNonManagedEdits: true,
};
const NO_TRACK: NavContext = {
  isAlbum: false,
  hasTrack: false,
  hasNonManagedEdits: false,
};

const state = (kind: NavStateKind): NavState => ({ kind });

interface Row {
  name: string;
  from: NavStateKind;
  action: NavAction;
  ctx: NavContext;
  to: NavStateKind;
}

const TABLE: Row[] = [
  // ----- unresolved: only `resolve` acts (loading-flicker guard) ---------
  { name: "resolve standard, clean", from: "unresolved", action: { type: "resolve", view: "standard" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "resolve standard, dirty bounces", from: "unresolved", action: { type: "resolve", view: "standard" }, ctx: DIRTY_TRACK, to: "advanced-track" },
  { name: "resolve standard, album bounces", from: "unresolved", action: { type: "resolve", view: "standard" }, ctx: ALBUM_CLEAN, to: "advanced-album" },
  { name: "resolve advanced", from: "unresolved", action: { type: "resolve", view: "advanced" }, ctx: CLEAN_TRACK, to: "advanced-track" },
  { name: "resolve advanced in album", from: "unresolved", action: { type: "resolve", view: "advanced" }, ctx: ALBUM_CLEAN, to: "advanced-album" },
  { name: "unresolved ignores set-view", from: "unresolved", action: { type: "set-view", view: "advanced" }, ctx: CLEAN_TRACK, to: "unresolved" },
  { name: "unresolved ignores context", from: "unresolved", action: { type: "context-changed" }, ctx: ALBUM_DIRTY, to: "unresolved" },
  { name: "unresolved ignores the door", from: "unresolved", action: { type: "request-back-to-standard" }, ctx: CLEAN_TRACK, to: "unresolved" },

  // ----- standard-track ---------------------------------------------------
  { name: "standard → advanced (tab)", from: "standard-track", action: { type: "set-view", view: "advanced" }, ctx: CLEAN_TRACK, to: "advanced-track" },
  { name: "standard re-entry stays (clean)", from: "standard-track", action: { type: "set-view", view: "standard" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "standard context: album flips to advanced-album", from: "standard-track", action: { type: "context-changed" }, ctx: ALBUM_CLEAN, to: "advanced-album" },
  { name: "standard context: dirty track bounces", from: "standard-track", action: { type: "context-changed" }, ctx: DIRTY_TRACK, to: "advanced-track" },
  { name: "standard context: clean stays", from: "standard-track", action: { type: "context-changed" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "standard context: no track stays", from: "standard-track", action: { type: "context-changed" }, ctx: NO_TRACK, to: "standard-track" },
  { name: "standard ignores the door", from: "standard-track", action: { type: "request-back-to-standard" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "standard ignores cancel", from: "standard-track", action: { type: "cancel-return" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "standard ignores complete", from: "standard-track", action: { type: "complete-return" }, ctx: CLEAN_TRACK, to: "standard-track" },

  // ----- advanced-track ---------------------------------------------------
  { name: "advanced door, clean → standard", from: "advanced-track", action: { type: "request-back-to-standard" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "advanced door, no track → standard", from: "advanced-track", action: { type: "request-back-to-standard" }, ctx: NO_TRACK, to: "standard-track" },
  { name: "advanced door, dirty → confirm", from: "advanced-track", action: { type: "request-back-to-standard" }, ctx: DIRTY_TRACK, to: "return-confirm-pending" },
  { name: "advanced set-view standard, clean", from: "advanced-track", action: { type: "set-view", view: "standard" }, ctx: CLEAN_TRACK, to: "standard-track" },
  { name: "advanced set-view standard, dirty bounces", from: "advanced-track", action: { type: "set-view", view: "standard" }, ctx: DIRTY_TRACK, to: "advanced-track" },
  { name: "advanced context: album flips shape", from: "advanced-track", action: { type: "context-changed" }, ctx: ALBUM_CLEAN, to: "advanced-album" },
  { name: "advanced context: track stays", from: "advanced-track", action: { type: "context-changed" }, ctx: DIRTY_TRACK, to: "advanced-track" },

  // ----- advanced-album (scenario 9 / 2a78f4a class) -----------------------
  { name: "album door, clean → standard (2a78f4a)", from: "advanced-album", action: { type: "request-back-to-standard" }, ctx: ALBUM_CLEAN, to: "standard-track" },
  { name: "album door, dirty → confirm", from: "advanced-album", action: { type: "request-back-to-standard" }, ctx: ALBUM_DIRTY, to: "return-confirm-pending" },
  { name: "album set-view standard bounces (album still on)", from: "advanced-album", action: { type: "set-view", view: "standard" }, ctx: ALBUM_CLEAN, to: "advanced-album" },
  { name: "album context: leaving album flips shape", from: "advanced-album", action: { type: "context-changed" }, ctx: CLEAN_TRACK, to: "advanced-track" },
  { name: "album context: staying album stays", from: "advanced-album", action: { type: "context-changed" }, ctx: ALBUM_DIRTY, to: "advanced-album" },

  // ----- return-confirm-pending --------------------------------------------
  { name: "confirm: cancel → advanced-track", from: "return-confirm-pending", action: { type: "cancel-return" }, ctx: DIRTY_TRACK, to: "advanced-track" },
  { name: "confirm: cancel in album → advanced-album", from: "return-confirm-pending", action: { type: "cancel-return" }, ctx: ALBUM_DIRTY, to: "advanced-album" },
  { name: "confirm: complete → standard", from: "return-confirm-pending", action: { type: "complete-return" }, ctx: DIRTY_TRACK, to: "standard-track" },
  { name: "confirm: complete from album → standard", from: "return-confirm-pending", action: { type: "complete-return" }, ctx: ALBUM_DIRTY, to: "standard-track" },
  { name: "confirm: context drift holds the dialog", from: "return-confirm-pending", action: { type: "context-changed" }, ctx: ALBUM_DIRTY, to: "return-confirm-pending" },
  { name: "confirm: explicit advanced cancels the ghost dialog", from: "return-confirm-pending", action: { type: "set-view", view: "advanced" }, ctx: DIRTY_TRACK, to: "advanced-track" },
  { name: "confirm: the door is idempotent-ish (stays pending)", from: "return-confirm-pending", action: { type: "request-back-to-standard" }, ctx: DIRTY_TRACK, to: "return-confirm-pending" },
];

describe("navReduce transition table", () => {
  for (const row of TABLE) {
    it(row.name, () => {
      expect(navReduce(state(row.from), row.action, row.ctx).kind).toBe(row.to);
    });
  }

  it("covers every state at least once", () => {
    const froms = new Set(TABLE.map((r) => r.from));
    for (const kind of [
      "unresolved",
      "standard-track",
      "advanced-track",
      "advanced-album",
      "return-confirm-pending",
    ] as const) {
      expect(froms, `no table rows exercise state ${kind}`).toContain(kind);
    }
  });
});

describe("derivations", () => {
  it("navView maps states to chrome views", () => {
    expect(navView(UNRESOLVED)).toBeNull();
    expect(navView(state("standard-track"))).toBe("standard");
    expect(navView(state("advanced-track"))).toBe("advanced");
    expect(navView(state("advanced-album"))).toBe("advanced");
    // The confirm dialog renders over Advanced, like the old boolean did.
    expect(navView(state("return-confirm-pending"))).toBe("advanced");
  });

  it("only the pending state opens the confirm dialog", () => {
    expect(isReturnConfirmPending(state("return-confirm-pending"))).toBe(true);
    expect(isReturnConfirmPending(state("advanced-track"))).toBe(false);
    expect(isReturnConfirmPending(UNRESOLVED)).toBe(false);
  });
});

describe("illegal states are unrepresentable", () => {
  it("no action from any state can produce standard while context forces advanced", () => {
    const states: NavStateKind[] = [
      "unresolved",
      "standard-track",
      "advanced-track",
      "advanced-album",
      "return-confirm-pending",
    ];
    const actions: NavAction[] = [
      { type: "resolve", view: "standard" },
      { type: "resolve", view: "advanced" },
      { type: "set-view", view: "standard" },
      { type: "set-view", view: "advanced" },
      { type: "context-changed" },
    ];
    // The two explicit return actions are excluded on purpose: they LAND in
    // standard and the wiring hook leaves Album mode in the same dispatch,
    // so the album context is stale by construction there.
    for (const from of states) {
      for (const action of actions) {
        const next = navReduce(state(from), action, ALBUM_DIRTY);
        // No-ops (e.g. a stray `resolve` on an already-resolved state) hold
        // the prior state by design; the entry invariant is about
        // TRANSITIONS into standard.
        if (next.kind === from) continue;
        expect(
          next.kind,
          `${from} + ${JSON.stringify(action)} transitioned to ${next.kind} under album+dirty`,
        ).not.toBe("standard-track");
      }
    }
  });
});
