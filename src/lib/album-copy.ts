// Album and count vocabulary (U10).
//
// Two jobs, both about the app saying true things in plain words:
//
//   1. Counts. "1 tracks" shipped in the sidebar header and the album export
//      receipt. Small, but it is the kind of thing that makes a careful user
//      wonder what else was not checked.
//   2. Album sequence language. `Flow Amount ×1.00` was on screen with nothing
//      to tell a user what a flow is, what the multiplier scales, or what ×1.00
//      means. A control nobody can interpret is not a feature.
//
// Pure functions with no React and no DSP. Nothing here changes a rendered
// value — this module only decides how existing numbers are described.

import type { AlbumArcKind } from "../bindings";

/**
 * Pluralize a count with its noun. `pluralize(1, "track")` → `"1 track"`.
 *
 * Deliberately explicit about the plural form rather than appending "s", so
 * an irregular noun cannot quietly produce "1 tracks"-class nonsense in the
 * other direction.
 */
export function pluralize(count: number, singular: string, plural?: string): string {
  const noun = count === 1 ? singular : (plural ?? `${singular}s`);
  return `${count} ${noun}`;
}

/** `"12 tracks"` / `"1 track"`. */
export function trackCountLabel(count: number): string {
  return pluralize(count, "track");
}

// ---------------------------------------------------------------------------
// Album flow vocabulary
// ---------------------------------------------------------------------------

/**
 * Plain-language description of what an album flow does.
 *
 * The promise (owner-defined 2026-07-03) is "one coherent record"; the flow is
 * the *expressive bonus* on top of that, not part of the promise. These
 * descriptions are written to match that framing — they describe a shape, and
 * none of them claims to improve the music.
 */
export const ALBUM_FLOW_DESCRIPTION: Record<AlbumArcKind, string> = {
  cinematic:
    "Starts restrained and builds — early tracks sit a little below the album target, later ones a little above.",
  afterhours:
    "Opens forward, then settles — the back half eases down for a late-night close.",
  "club-peak":
    "Builds to a peak in the middle of the record, then comes back down.",
  "fever-dream":
    "Moves unevenly on purpose — loudness rises and falls track to track rather than following one curve.",
};

/**
 * Plain-language meaning of the flow-amount multiplier.
 *
 * `Flow Amount ×1.00` told the user a number and nothing else. The multiplier
 * scales how far the flow is allowed to move each track away from the album
 * loudness target: ×0 is a flat record, ×1 is the flow as designed, above ×1
 * exaggerates it.
 */
export function flowAmountDescription(amount: number): string {
  if (!Number.isFinite(amount)) return "Flow strength is unavailable.";
  if (amount <= 0.001) {
    return "Off — every track sits at the same album loudness target.";
  }
  if (amount < 0.95) {
    return `Gentle — track-to-track loudness moves ${Math.round(amount * 100)}% as far as the flow's full shape.`;
  }
  if (amount <= 1.05) {
    return "Full — the flow's shape as designed.";
  }
  return `Exaggerated — track-to-track loudness moves ${Math.round(amount * 100)}% of the flow's full shape.`;
}

/** Short label for the flow-amount value, for a control's accessible value. */
export function flowAmountValueText(amount: number): string {
  if (!Number.isFinite(amount)) return "unavailable";
  if (amount <= 0.001) return "off, flat album";
  if (amount <= 1.05 && amount >= 0.95) return "full flow shape";
  return `${Math.round(amount * 100)} percent of the full flow shape`;
}

// ---------------------------------------------------------------------------
// Per-track sequence status
// ---------------------------------------------------------------------------

export type SequenceRole = "opener" | "album_track" | "closer";

/** Human label for a backend sequence role. */
export function sequenceRoleLabel(role: string | null | undefined): string {
  switch (role) {
    case "opener":
      return "Opener";
    case "closer":
      return "Closer";
    case "album_track":
      return "Album track";
    default:
      return "";
  }
}

/**
 * Signed loudness offset relative to the album target, as the user should read
 * it. `+0.0` is deliberately rendered as `±0.0` so "no offset" is visually
 * distinct from a rounding artefact.
 */
export function arcOffsetLabel(offsetDb: number | null | undefined): string {
  if (offsetDb == null || !Number.isFinite(offsetDb)) return "";
  if (Math.abs(offsetDb) < 0.05) return "±0.0 LU";
  const sign = offsetDb > 0 ? "+" : "−";
  return `${sign}${Math.abs(offsetDb).toFixed(1)} LU`;
}

/**
 * One-line explanation of why a single-track album is a degenerate case.
 *
 * Album mode with one track is not broken — it renders, and the delivery format
 * and receipt still apply — but the sequence half of the feature has nothing to
 * act on. Saying so is better than showing a flow control that cannot do
 * anything.
 */
export const SINGLE_TRACK_ALBUM_NOTE =
  "One track: album delivery format and the per-track receipt still apply, but a sequence needs at least two tracks before flow can shape anything.";
