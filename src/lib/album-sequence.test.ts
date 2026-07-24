// U10 — album sequence overview rows.
//
// The unit's premise is that Album mode already HELD this information and
// simply never showed it. These tests pin the two rules that make the overview
// trustworthy rather than decorative:
//
//   1. roles and arc offsets come from the backend plan, never a client-side
//      re-derivation that could drift from what actually renders;
//   2. an overridden track is a full sound exemption (D9), so it must not be
//      shown an album target or arc offset it will not receive.

import { describe, expect, it } from "vitest";

import {
  buildSequenceRows,
  sequenceArcHeights,
  sequenceSummary,
} from "./album-sequence";
import type { AlbumPlan, AnalysisResult, ImportedTrack } from "../bindings";

function track(id: string, name: string): ImportedTrack {
  return {
    id,
    path: `/music/${id}.wav`,
    display_name: name,
    source_format: "wav",
    duration_seconds: 200,
    sample_rate: 44100,
    channels: 2,
  };
}

function analysis(id: string, lufs: number): AnalysisResult {
  return {
    track_id: id,
    lufs_integrated: lufs,
    lufs_short_term_max: lufs + 3,
    true_peak_dbtp: -1.2,
    dynamic_range_lu: 7,
    spectral_balance: { low: 0.33, mid: 0.34, high: 0.33 },
    transient_density: 0.5,
    stereo_width: 1,
    recommended_universal: null as never,
    measured_at_iso: "2026-01-01T00:00:00Z",
    inferred_role: null,
    role_confidence: null,
    inferred_character: null,
    character_confidence: null,
  } as AnalysisResult;
}

function plan(entries: Array<[string, string, number]>): AlbumPlan {
  return {
    title: "Album",
    arc: { kind: "preset", preset: "cinematic" },
    intensity: 1,
    transitions: [],
    tracks: entries.map(([track_id, role, arc_lufs_offset_db], i) => ({
      track_id,
      position: i + 1,
      role,
      role_locked: false,
      arc_lufs_offset_db,
      intensity_scale: 1,
    })),
  } as unknown as AlbumPlan;
}

const TRACKS = [track("a", "01 - Opener.wav"), track("b", "02 - Middle.wav"), track("c", "03 - Closer.wav")];

describe("buildSequenceRows", () => {
  it("shows source loudness, album target, role, and arc offset from the plan", () => {
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: {
        a: analysis("a", -16.2),
        b: analysis("b", -14.0),
        c: analysis("c", -12.8),
      },
      overrideAlbum: new Set(),
      plan: plan([
        ["a", "opener", -2.1],
        ["b", "album_track", 0],
        ["c", "closer", 1.8],
      ]),
      albumTargetLufs: -14,
    });

    expect(rows).toHaveLength(3);
    expect(rows[0].sourceLufs).toBeCloseTo(-16.2);
    expect(rows[0].roleLabel).toBe("Opener");
    expect(rows[0].arcOffsetLabel).toBe("−2.1 LU");
    expect(rows[0].targetLufs).toBeCloseTo(-16.1);

    // A zero offset must read as an explicit "no offset", not an empty cell
    // that looks like missing data.
    expect(rows[1].arcOffsetLabel).toBe("±0.0 LU");
    expect(rows[2].roleLabel).toBe("Closer");
    expect(rows[2].targetLufs).toBeCloseTo(-12.2);
  });

  it("does not show an overriding track an album target or arc offset", () => {
    // D9: override is a FULL sound exemption — its own settings, its own
    // target, no arc offset. Showing it an album target would contradict the
    // promise the override makes.
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: { a: analysis("a", -16.2) },
      overrideAlbum: new Set(["a"]),
      plan: plan([["a", "opener", -2.1]]),
      albumTargetLufs: -14,
    });

    expect(rows[0].overridesAlbum).toBe(true);
    expect(rows[0].targetLufs).toBeNull();
    expect(rows[0].arcOffsetLabel).toBe("");
    expect(rows[0].roleLabel).toBe("");
  });

  it("marks unanalyzed tracks as pending rather than inventing a loudness", () => {
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: { a: analysis("a", -16.2) },
      overrideAlbum: new Set(),
      plan: null,
      albumTargetLufs: -14,
    });

    expect(rows[0].analysisStatus).toBe("analyzed");
    expect(rows[1].analysisStatus).toBe("pending");
    expect(rows[1].sourceLufs).toBeNull();
    expect(rows[1].hasConcern).toBe(false);
  });

  it("omits roles and offsets entirely when no plan is available", () => {
    // Rather than guessing "first track = opener" client-side, which would be
    // a second implementation of a backend decision and could drift from it.
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: {},
      overrideAlbum: new Set(),
      plan: null,
      albumTargetLufs: -14,
    });

    for (const row of rows) {
      expect(row.roleLabel).toBe("");
      expect(row.arcOffsetLabel).toBe("");
    }
  });

  it("degrades instead of throwing when inputs are missing", () => {
    // A read-only overview must never be able to take the app down. This is
    // not hypothetical: the first wiring threw on an absent analysis map.
    expect(() => buildSequenceRows({ tracks: TRACKS })).not.toThrow();
    const rows = buildSequenceRows({ tracks: TRACKS });
    expect(rows).toHaveLength(3);
    expect(rows[0].sourceLufs).toBeNull();
    expect(rows[0].targetLufs).toBeNull();
  });

  it("preserves album order, so reorder is reflected immediately", () => {
    const reordered = [TRACKS[2], TRACKS[0], TRACKS[1]];
    const rows = buildSequenceRows({ tracks: reordered });
    expect(rows.map((r) => r.trackId)).toEqual(["c", "a", "b"]);
    expect(rows.map((r) => r.position)).toEqual([1, 2, 3]);
  });
});

describe("sequenceArcHeights", () => {
  it("normalizes the plan's own targets into a drawable shape", () => {
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: {},
      overrideAlbum: new Set(),
      plan: plan([
        ["a", "opener", -2],
        ["b", "album_track", 0],
        ["c", "closer", 2],
      ]),
      albumTargetLufs: -14,
    });

    expect(sequenceArcHeights(rows)).toEqual([0, 0.5, 1]);
  });

  it("draws nothing for a flat album rather than inventing a curve", () => {
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: {},
      overrideAlbum: new Set(),
      plan: plan([
        ["a", "opener", 0],
        ["b", "album_track", 0],
        ["c", "closer", 0],
      ]),
      albumTargetLufs: -14,
    });

    expect(sequenceArcHeights(rows)).toEqual([]);
  });

  it("draws nothing when any target is unknown", () => {
    const rows = buildSequenceRows({ tracks: TRACKS });
    expect(sequenceArcHeights(rows)).toEqual([]);
  });
});

describe("sequenceSummary", () => {
  it("counts overrides and pending analyses", () => {
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: { a: analysis("a", -16) },
      overrideAlbum: new Set(["b"]),
      plan: null,
      albumTargetLufs: -14,
    });

    const summary = sequenceSummary(rows);
    expect(summary).toContain("1 overriding");
    expect(summary).toContain("2 awaiting analysis");
  });

  it("is empty when there is nothing worth flagging", () => {
    const rows = buildSequenceRows({
      tracks: TRACKS,
      analysisByTrackId: {
        a: analysis("a", -16),
        b: analysis("b", -14),
        c: analysis("c", -13),
      },
      overrideAlbum: new Set(),
      plan: null,
      albumTargetLufs: -14,
    });

    expect(sequenceSummary(rows)).toBe("");
  });
});
