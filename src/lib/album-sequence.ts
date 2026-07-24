// Album sequence overview rows (U10).
//
// The 2026-07-24 headless review found Album mode holding twelve tracks
// correctly while showing almost none of that work: filenames clipped to
// fragments, `Flow Amount ×1.00` with no stated meaning, and no scan-level view
// of loudness arc, role, target, or per-track status. The mechanics were real;
// the presentation hid them.
//
// This module turns data the app ALREADY HAS into one row per track, so a user
// can read the whole record without opening each track.
//
// Two rules it follows strictly:
//
//   1. **Nothing is invented.** Roles and arc offsets come from the backend's
//      own `plan_album` — the same planner the export uses — not from a
//      client-side re-derivation that could drift from what actually renders.
//      When no plan is available the fields are simply absent.
//   2. **Warnings are not re-computed.** The per-track concern flag comes from
//      `receipt-quality.ts`, the existing single owner of quality state. U10
//      requires each warning to have one owner and one location; adding a
//      second implementation here would violate exactly what it asks for.
//
// Pure and React-free so it can be tested directly.

import type { AlbumPlan, AnalysisResult, ImportedTrack, QualityCheck } from "../bindings";
import { arcOffsetLabel, sequenceRoleLabel } from "./album-copy";
import { buildQualityRows, hasQualityConcern } from "./receipt-quality";

export type SequenceAnalysisStatus = "analyzed" | "pending";

export interface SequenceRow {
  trackId: string;
  /** 1-based position in the album order. */
  position: number;
  displayName: string;
  durationSeconds: number | null;
  /** Measured source loudness, when the track has been analyzed. */
  sourceLufs: number | null;
  /**
   * Where this track is aimed, including its arc offset. Null when the album
   * target is unknown or the track has no plan entry yet.
   */
  targetLufs: number | null;
  /** "Opener" / "Album track" / "Closer" — empty when no plan is available. */
  roleLabel: string;
  /** Signed arc offset relative to the album target, e.g. "−2.1 LU". */
  arcOffsetLabel: string;
  /** True when this track opts out of the album settings (D9 full exemption). */
  overridesAlbum: boolean;
  analysisStatus: SequenceAnalysisStatus;
  /** True when the shared quality owner reports a concern for this track. */
  hasConcern: boolean;
}

export interface SequenceInput {
  tracks: ImportedTrack[];
  analysisByTrackId?: Record<string, AnalysisResult> | null;
  overrideAlbum?: Set<string> | null;
  /** Backend plan for the current order/flow/amount. Null until one exists. */
  plan?: AlbumPlan | null;
  /** Effective album loudness target, when one is resolvable. */
  albumTargetLufs?: number | null;
  /** Per-track export checks, when any have been run. */
  checksByTrackId?: Record<string, QualityCheck[]> | null;
}

/**
 * Build the display rows.
 *
 * Every input beyond `tracks` is optional and every one degrades to "show
 * less". This is a read-only overview: a missing analysis map or an
 * unavailable plan must produce a thinner row, never a thrown error that takes
 * the whole app down. It is not worth crashing a mastering session to avoid
 * omitting a loudness number.
 */
export function buildSequenceRows({
  tracks,
  analysisByTrackId,
  overrideAlbum,
  plan,
  albumTargetLufs = null,
  checksByTrackId,
}: SequenceInput): SequenceRow[] {
  const analyses = analysisByTrackId ?? {};
  const overrides = overrideAlbum ?? new Set<string>();
  const checks = checksByTrackId ?? {};
  const planByTrackId = new Map(
    (plan?.tracks ?? []).map((entry) => [entry.track_id, entry]),
  );

  return tracks.map((track, index) => {
    const analysis = analyses[track.id] ?? null;
    const planEntry = planByTrackId.get(track.id);
    const overridesAlbum = overrides.has(track.id);

    // An overridden track is a full sound exemption (D9): it renders with its
    // own settings and its OWN target, with no arc offset applied. Showing it
    // an arc offset would contradict the promise the override makes.
    const arcOffset =
      planEntry && !overridesAlbum ? planEntry.arc_lufs_offset_db : null;

    const targetLufs =
      albumTargetLufs != null && !overridesAlbum
        ? albumTargetLufs + (arcOffset ?? 0)
        : null;

    const rows = buildQualityRows(checks[track.id] ?? [], analysis);

    return {
      trackId: track.id,
      position: index + 1,
      displayName: track.display_name,
      durationSeconds: track.duration_seconds ?? null,
      sourceLufs: analysis ? analysis.lufs_integrated : null,
      targetLufs,
      roleLabel: overridesAlbum ? "" : sequenceRoleLabel(planEntry?.role),
      arcOffsetLabel: arcOffsetLabel(arcOffset),
      overridesAlbum,
      analysisStatus: analysis ? "analyzed" : "pending",
      hasConcern: analysis ? hasQualityConcern(rows) : false,
    };
  });
}

/**
 * Normalized arc heights (0..1) for a compact sequence sparkline.
 *
 * Derived ONLY from the plan's own offsets, so the drawn shape is the shape
 * that will actually render — it responds to flow choice, flow amount, and
 * ordering because the plan does. Returns an empty array when there is no plan
 * or no spread to draw, rather than inventing a curve.
 */
export function sequenceArcHeights(rows: SequenceRow[]): number[] {
  const offsets = rows.map((row) => {
    if (row.targetLufs == null) return null;
    return row.targetLufs;
  });
  if (offsets.some((o) => o == null)) return [];
  const values = offsets as number[];
  if (values.length < 2) return [];

  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min;
  // A flat album (flow amount 0, or a flow with no spread) has no arc to draw.
  if (span < 0.05) return [];
  return values.map((v) => (v - min) / span);
}

/** Summary line for the whole sequence, e.g. "12 tracks · 2 overriding". */
export function sequenceSummary(rows: SequenceRow[]): string {
  const overriding = rows.filter((r) => r.overridesAlbum).length;
  const pending = rows.filter((r) => r.analysisStatus === "pending").length;
  const concerns = rows.filter((r) => r.hasConcern).length;

  const parts: string[] = [];
  if (overriding > 0) parts.push(`${overriding} overriding`);
  if (pending > 0) parts.push(`${pending} awaiting analysis`);
  if (concerns > 0) parts.push(`${concerns} to review`);
  return parts.join(" · ");
}
