// src/lib/view-by-track.ts
//
// Per-track view memory (F6 / Q24). Standard vs Advanced is remembered PER
// TRACK so switching tracks restores the view the user last explicitly chose
// for each — instead of a single global last-view that a force-bounce to
// Advanced could clobber. Only EXPLICIT user choices land here: the
// force-bounce for a dirty track still renders Advanced but must never call
// `rememberView`, so a bounce can't overwrite a remembered choice. Pure and
// tiny so the rule is unit-testable without React.

import type { TrackId, ViewMode } from "../bindings";

export type ViewByTrack = Record<TrackId, ViewMode>;

/// Record an EXPLICIT user view choice for a track. No-ops for a null track
/// (nothing selected) and returns the SAME reference when unchanged, so it is
/// safe to feed straight into a React state setter without spurious renders.
export function rememberView(
  map: ViewByTrack,
  trackId: TrackId | null,
  view: ViewMode,
): ViewByTrack {
  if (!trackId) return map;
  if (map[trackId] === view) return map;
  return { ...map, [trackId]: view };
}

/// The view the user last explicitly chose for a track, or null if the track
/// has no remembered choice (caller then keeps the current/default view).
export function rememberedView(map: ViewByTrack, trackId: TrackId | null): ViewMode | null {
  if (!trackId) return null;
  return map[trackId] ?? null;
}
