// REVIEW state for Source Insight (2026-08-18). Thin React wrapper over the
// pure revision model in lib/source-insight.ts, backed by localStorage so an
// acknowledged analysis stays acknowledged when the project is reopened.
import { useCallback, useState } from "react";

import type { AnalysisResult } from "../bindings";
import {
  acknowledgeAnalysis,
  browserReviewStore,
  isAnalysisUnreviewed,
  readReviewedMap,
  writeReviewedMap,
  type ReviewedMap,
} from "../lib/source-insight";

export function useInsightReview(): {
  isUnreviewed: (analysis: AnalysisResult | undefined) => boolean;
  acknowledge: (analysis: AnalysisResult) => void;
} {
  const [map, setMap] = useState<ReviewedMap>(() => readReviewedMap(browserReviewStore()));
  const isUnreviewed = useCallback(
    (analysis: AnalysisResult | undefined) => isAnalysisUnreviewed(map, analysis),
    [map],
  );
  const acknowledge = useCallback((analysis: AnalysisResult) => {
    setMap((prev) => {
      const next = acknowledgeAnalysis(prev, analysis);
      writeReviewedMap(browserReviewStore(), next);
      return next;
    });
  }, []);
  return { isUnreviewed, acknowledge };
}
