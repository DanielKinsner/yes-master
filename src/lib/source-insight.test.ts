import { describe, expect, it } from "vitest";

import type { AnalysisResult } from "../bindings";
import {
  acknowledgeAnalysis,
  analysisRevisionKey,
  insightHeadline,
  insightOverallStatus,
  isAnalysisUnreviewed,
  readReviewedMap,
  sourceInsightRows,
  writeReviewedMap,
  type ReviewStore,
} from "./source-insight";

// Source Insight (2026-08-18): the analysis disclosure under the track title
// took over the right rail's SOURCE CHECK. Two behaviours are pinned here:
// the structured interpretation of an analysis, and the REVIEW model —
// "unacknowledged analysis revision", never "warning present".

function analysis(overrides: Partial<AnalysisResult> = {}): AnalysisResult {
  return {
    track_id: "t1",
    lufs_integrated: -14.1,
    lufs_short_term_max: -10,
    true_peak_dbtp: -2.33,
    dynamic_range_lu: 4.6,
    spectral_balance: { low: 0.4, mid: 0.45, high: 0.15 },
    transient_density: 0.5,
    stereo_width: 0.25,
    recommended_universal: undefined as unknown as AnalysisResult["recommended_universal"],
    measured_at_iso: "2026-08-18T10:00:00Z",
    ...overrides,
  } as AnalysisResult;
}

describe("sourceInsightRows", () => {
  it("produces the five structured rows with values and readings", () => {
    const rows = sourceInsightRows(analysis());
    expect(rows.map((r) => r.key)).toEqual(["loudness", "dynamics", "spectrum", "stereo", "true-peak"]);
    const byKey = Object.fromEntries(rows.map((r) => [r.key, r]));
    expect(byKey.loudness.value).toBe("-14.1 LUFS");
    expect(byKey.loudness.note).toMatch(/streaming targets/);
    expect(byKey.loudness.status).toBe("ok");
    expect(byKey.dynamics.value).toBe("4.6 LU");
    expect(byKey.dynamics.status).toBe("caution");
    expect(byKey.spectrum.value).toBe("Dark");
    expect(byKey.stereo.value).toBe("Narrow");
    expect(byKey["true-peak"].value).toBe("-2.33 dBTP");
    expect(byKey["true-peak"].status).toBe("ok");
  });

  it("keeps the Source Check severities: hot true peak and crushed dynamics are problems", () => {
    const rows = sourceInsightRows(analysis({ true_peak_dbtp: 0.2, dynamic_range_lu: 3.3, lufs_integrated: -5 }));
    const byKey = Object.fromEntries(rows.map((r) => [r.key, r]));
    expect(byKey["true-peak"].status).toBe("problem");
    expect(byKey.dynamics.status).toBe("problem");
    expect(byKey.loudness.status).toBe("problem");
    expect(insightOverallStatus(rows)).toBe("problem");
  });

  it("headline is the loudness sentence the old inline summary used", () => {
    expect(insightHeadline(analysis())).toBe("Source -14.1 LUFS — close to typical streaming targets.");
    expect(insightHeadline(analysis({ lufs_integrated: -7 }))).toMatch(/very loud/);
  });
});

describe("REVIEW = unacknowledged analysis revision", () => {
  function memStore(): ReviewStore & { data: Record<string, string> } {
    const data: Record<string, string> = {};
    return {
      data,
      getItem: (k) => data[k] ?? null,
      setItem: (k, v) => {
        data[k] = v;
      },
    };
  }

  it("a fresh analysis is unreviewed until acknowledged, then stays reviewed for that revision", () => {
    const a = analysis();
    let map = {};
    expect(isAnalysisUnreviewed(map, a)).toBe(true);
    map = acknowledgeAnalysis(map, a);
    expect(isAnalysisUnreviewed(map, a)).toBe(false);
    // Opening/closing the panel or editing mastering settings never touches
    // the analysis, so nothing here can bring REVIEW back.
    expect(isAnalysisUnreviewed(map, { ...a })).toBe(false);
  });

  it("re-analyze (new measured_at) and a new source (new track_id) are new revisions", () => {
    const a = analysis();
    const map = acknowledgeAnalysis({}, a);
    const reanalyzed = analysis({ measured_at_iso: "2026-08-18T11:00:00Z" });
    expect(analysisRevisionKey(reanalyzed)).not.toBe(analysisRevisionKey(a));
    expect(isAnalysisUnreviewed(map, reanalyzed)).toBe(true);
    const newSource = analysis({ track_id: "t2" });
    expect(isAnalysisUnreviewed(map, newSource)).toBe(true);
    // ...and acknowledging one track never acknowledges another.
    const map2 = acknowledgeAnalysis(map, newSource);
    expect(isAnalysisUnreviewed(map2, reanalyzed)).toBe(true);
  });

  it("no analysis → nothing to review", () => {
    expect(isAnalysisUnreviewed({}, undefined)).toBe(false);
  });

  it("round-trips through the store and survives garbage", () => {
    const store = memStore();
    writeReviewedMap(store, acknowledgeAnalysis({}, analysis()));
    expect(readReviewedMap(store)).toEqual({ t1: "t1:2026-08-18T10:00:00Z" });
    store.data["yes-master:insight-reviewed"] = "{not json";
    expect(readReviewedMap(store)).toEqual({});
    store.data["yes-master:insight-reviewed"] = JSON.stringify({ t1: 5, t2: "ok" });
    expect(readReviewedMap(store)).toEqual({ t2: "ok" });
    expect(readReviewedMap(null)).toEqual({});
  });
});
