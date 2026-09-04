// src/lib/source-insight.ts
//
// Source Insight — the analysis disclosure under the track title (2026-08-18
// right-rail pass). Two pure concerns live here so they are testable without
// React:
//
//   1. INTERPRETATION: turn an AnalysisResult into structured rows
//      (metric · value · reading · status) plus a one-line headline. This is
//      the same commentary the old inline AnalysisSummary produced, plus the
//      three Source Check thresholds (true peak / loudness / dynamic range)
//      that used to live in the right rail — one owner now.
//
//   2. REVIEW STATE: "REVIEW" means *unacknowledged analysis revision*, not
//      *warning present*. A revision is `track_id + measured_at_iso`: it
//      changes when Re-analyze runs and when a new source produces a new
//      analysis, and it stays put across ordinary mastering edits. The user
//      acknowledges a revision; the badge stays gone for that revision.
//      Persisted in localStorage (same store-interface pattern as
//      view-mode.ts) so reopening the project does not nag again.

import type { AnalysisResult, QualityCheck } from "../bindings";

export type InsightStatus = "ok" | "info" | "caution" | "problem";

export interface InsightRow {
  key: "loudness" | "dynamics" | "spectrum" | "stereo" | "true-peak";
  label: string;
  /// Emphasised numeric/short value ("-14.1 LUFS", "Dark / low-mid").
  value: string;
  /// One-clause interpretation.
  note: string;
  status: InsightStatus;
}

export function sourceInsightRows(analysis: AnalysisResult): InsightRow[] {
  const rows: InsightRow[] = [];

  const lufs = analysis.lufs_integrated;
  {
    let note: string;
    let status: InsightStatus;
    if (lufs > -6) {
      note = "Extremely loud — little headroom for the chain.";
      status = "problem";
    } else if (lufs > -8) {
      note = "Very loud — may sound flat vs streaming references.";
      status = "caution";
    } else if (lufs > -12) {
      note = "Loud — streaming-loud territory.";
      status = "info";
    } else if (lufs > -16) {
      note = "Close to typical streaming targets.";
      status = "ok";
    } else {
      note = "Conservative loudness — room to push.";
      status = "ok";
    }
    rows.push({ key: "loudness", label: "Loudness", value: `${lufs.toFixed(1)} LUFS`, note, status });
  }

  // Wire field retained for saved-project compatibility; the measurement is
  // EBU loudness range, not crest factor or evidence of compression.
  const lra = analysis.dynamic_range_lu;
  rows.push({
    key: "dynamics", label: "Loudness range (LRA)", value: `${lra.toFixed(1)} LU`,
    note: lra < 5 ? "Little longer-term loudness variation." : lra < 8 ? "Moderate longer-term loudness variation." : "Wide longer-term loudness variation.",
    status: "info",
  });
  const high = analysis.spectral_balance.high;
  const low = analysis.spectral_balance.low;
  {
    let value: string;
    let note: string;
    if (high > 0.35) {
      value = "Bright";
      note = "Presence-forward spectrum.";
    } else if (high < 0.18) {
      value = "Dark";
      note = "Low-mid-focused spectrum.";
    } else if (low > 0.45) {
      value = "Low-heavy";
      note = "Weight sits in the lows.";
    } else {
      value = "Balanced";
      note = "Even spectral balance.";
    }
    rows.push({ key: "spectrum", label: "Spectrum", value, note, status: "info" });
  }

  const w = analysis.stereo_width;
  {
    let value: string;
    let note: string;
    if (w > 0.7) {
      value = "Wide";
      note = "Wide stereo image.";
    } else if (w < 0.3) {
      value = "Narrow";
      note = "Mono-leaning stereo image.";
    } else {
      value = "Standard";
      note = "Standard stereo image.";
    }
    rows.push({ key: "stereo", label: "Stereo image", value, note, status: "info" });
  }

  const tp = analysis.true_peak_dbtp;
  {
    let note: string;
    let status: InsightStatus;
    if (tp > -0.1) {
      note = "At or above the digital ceiling — risky.";
      status = "problem";
    } else if (tp > -1.0) {
      note = "Fine digitally; lossy codecs may overshoot.";
      status = "caution";
    } else {
      note = "Comfortable headroom.";
      status = "ok";
    }
    rows.push({ key: "true-peak", label: "True peak", value: `${tp.toFixed(2)} dBTP`, note, status });
  }

  return rows;
}

/// The one sentence the collapsed row shows. Source loudness is the most
/// actionable single fact for a mastering decision, so it leads — same
/// choice the old AnalysisSummary made.
export function insightHeadline(analysis: AnalysisResult): string {
  const lufs = analysis.lufs_integrated;
  if (lufs > -8) {
    return `Source very loud at ${lufs.toFixed(1)} LUFS — may sound flat vs streaming references.`;
  }
  if (lufs > -12) return `Source loud at ${lufs.toFixed(1)} LUFS — streaming-loud territory.`;
  if (lufs > -16) return `Source ${lufs.toFixed(1)} LUFS — close to typical streaming targets.`;
  return `Source ${lufs.toFixed(1)} LUFS — conservative loudness, room to push.`;
}

/// Worst status across the rows — drives the collapsed row's status dot.
export function insightOverallStatus(rows: InsightRow[]): InsightStatus {
  const rank: Record<InsightStatus, number> = { ok: 0, info: 1, caution: 2, problem: 3 };
  return rows.reduce<InsightStatus>((worst, r) => (rank[r.status] > rank[worst] ? r.status : worst), "ok");
}

// ---------------------------------------------------------------------------
// Export checks (post-export) — shown as a second group when present. Friendly
// labels moved here from RightRail's QualityCheckPanel so the wording has one
// owner.
// ---------------------------------------------------------------------------

export function friendlyCheckLabel(c: QualityCheck): string {
  switch (c.code) {
    case "export_ok":
      return "No issues detected";
    case "true_peak_high":
      return "True peak above safe ceiling";
    case "streaming_headroom_low":
      return "Low streaming headroom";
    case "lufs_very_loud":
      return "Very loud master";
    case "dynamic_range_low":
      return "Low loudness range (LRA)";
    case "bit_depth_low":
      return "Bit depth below 16 bits";
    case "sample_rate_mismatch":
      return "Sample rate does not match delivery";
    case "non_finite_metering":
      return "Non-finite loudness measurement";
    case "comp_density_on_compressed_source":
      return "Low source loudness range";
    default:
      return c.message;
  }
}

export function checkStatus(c: QualityCheck): InsightStatus {
  return c.level === "critical" ? "problem" : c.level === "warning" ? "caution" : "ok";
}

// ---------------------------------------------------------------------------
// Review state
// ---------------------------------------------------------------------------

/// Identity of an analysis result. Two analyses of the same file at different
/// times are different revisions; the same revision reopened later is not.
export function analysisRevisionKey(analysis: AnalysisResult): string {
  return `${analysis.track_id}:${analysis.measured_at_iso}`;
}

export interface ReviewStore {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

const KEY = "yes-master:insight-reviewed";
const MAX_ENTRIES = 200;

export function browserReviewStore(): ReviewStore | null {
  try {
    const storage = globalThis.localStorage;
    if (storage && typeof storage.getItem === "function" && typeof storage.setItem === "function") {
      return storage;
    }
  } catch {
    /* locked-down webviews can throw */
  }
  return null;
}

/// track_id → reviewed revision key.
export type ReviewedMap = Record<string, string>;

export function readReviewedMap(store: ReviewStore | null): ReviewedMap {
  if (!store) return {};
  const raw = store.getItem(KEY);
  if (!raw) return {};
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      const out: ReviewedMap = {};
      for (const [k, v] of Object.entries(parsed as Record<string, unknown>)) {
        if (typeof v === "string") out[k] = v;
      }
      return out;
    }
  } catch {
    /* malformed → treat as empty */
  }
  return {};
}

export function writeReviewedMap(store: ReviewStore | null, map: ReviewedMap): void {
  if (!store) return;
  try {
    // Bound the map so a long-lived install cannot grow it forever.
    const entries = Object.entries(map);
    const trimmed = entries.length > MAX_ENTRIES ? entries.slice(entries.length - MAX_ENTRIES) : entries;
    store.setItem(KEY, JSON.stringify(Object.fromEntries(trimmed)));
  } catch {
    /* best-effort */
  }
}

/// True when the analysis on screen has NOT been acknowledged yet.
export function isAnalysisUnreviewed(map: ReviewedMap, analysis: AnalysisResult | undefined): boolean {
  if (!analysis) return false;
  return map[analysis.track_id] !== analysisRevisionKey(analysis);
}

/// Acknowledge the current revision. Returns the next map (pure).
export function acknowledgeAnalysis(map: ReviewedMap, analysis: AnalysisResult): ReviewedMap {
  return { ...map, [analysis.track_id]: analysisRevisionKey(analysis) };
}
