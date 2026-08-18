// Source Insight — the analysis disclosure directly under the track title.
//
// 2026-08-18 right-rail pass: this took over everything the right rail's
// SOURCE CHECK used to say (and its Re-analyze action), so the rail can be
// pure configuration → processing → delivery → export.
//
//   Collapsed:  INSIGHT  [REVIEW]  Source -14.1 LUFS — close to … ▾
//   Expanded:   an anchored panel with five metric rows (value · reading ·
//               status), an optional "Last export" group, and the actions.
//
// REVIEW here means "this analysis revision hasn't been acknowledged" — see
// lib/source-insight.ts. Clicking the badge acknowledges; so does "Mark
// reviewed" in the panel. Findings keep their own status after that.
import { useCallback, useEffect, useId, useRef, useState } from "react";

import type { AnalysisResult, QualityCheck } from "../bindings";
import {
  checkStatus,
  friendlyCheckLabel,
  insightHeadline,
  insightOverallStatus,
  sourceInsightRows,
  type InsightStatus,
} from "../lib/source-insight";

export function SourceInsight({
  analysis,
  lastChecks,
  unreviewed,
  isAnalyzing = false,
  onAcknowledge,
  onReanalyze,
}: {
  analysis: AnalysisResult;
  /// Quality checks from the most recent export, if any.
  lastChecks?: QualityCheck[];
  unreviewed: boolean;
  isAnalyzing?: boolean;
  onAcknowledge: () => void;
  onReanalyze?: () => void;
}) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const panelId = useId();
  const rows = sourceInsightRows(analysis);
  const overall = insightOverallStatus(rows);
  const headline = insightHeadline(analysis);
  const exportRows = lastChecks && lastChecks.length > 0 ? lastChecks : null;

  // Close on outside click / Escape — it's a disclosure, not a mode.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("pointerdown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const acknowledge = useCallback(() => onAcknowledge(), [onAcknowledge]);

  return (
    <div ref={rootRef} className={"source-insight" + (open ? " is-open" : "")}>
      <div className="source-insight-row">
        <button
          type="button"
          className="source-insight-toggle"
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => setOpen((v) => !v)}
        >
          <span className="source-insight-eyebrow">Insight</span>
          <span className={`source-insight-dot is-${overall}`} aria-hidden="true" />
          <span className="source-insight-headline">{headline}</span>
          <span className="source-insight-chevron" aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 12 12" focusable="false">
              <path d="M2.5 4.5 L6 8 L9.5 4.5" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </span>
        </button>
        {unreviewed && (
          <button
            type="button"
            className="source-insight-review"
            title="New analysis — click to mark it reviewed"
            aria-label="New analysis to review — mark reviewed"
            onClick={(e) => {
              e.stopPropagation();
              acknowledge();
            }}
          >
            Review
          </button>
        )}
      </div>

      {open && (
        <div id={panelId} className="source-insight-panel" role="region" aria-label="Source analysis">
          <div className="source-insight-panel-head">
            <span className="source-insight-panel-title">Source analysis</span>
            {unreviewed && <span className="source-insight-unread">Unreviewed</span>}
          </div>
          <dl className="source-insight-list">
            {rows.map((r) => (
              <div key={r.key} className={`source-insight-item is-${r.status}`}>
                <dt>
                  <StatusGlyph status={r.status} />
                  <span className="source-insight-label">{r.label}</span>
                </dt>
                <dd>
                  <span className="source-insight-value">{r.value}</span>
                  <span className="source-insight-note">{r.note}</span>
                  <span className="sr-only">{statusWord(r.status)}</span>
                </dd>
              </div>
            ))}
          </dl>
          {exportRows && (
            <>
              <div className="source-insight-panel-head is-secondary">
                <span className="source-insight-panel-title">Last export</span>
              </div>
              <ul className="source-insight-checks">
                {exportRows.map((c, i) => {
                  const st = checkStatus(c);
                  return (
                    <li key={`${c.code}-${i}`} className={`source-insight-check is-${st}`} title={c.message}>
                      <StatusGlyph status={st} />
                      <span>{friendlyCheckLabel(c)}</span>
                      <span className="sr-only">
                        {statusWord(st)}. {c.message}
                      </span>
                    </li>
                  );
                })}
              </ul>
            </>
          )}
          <div className="source-insight-actions">
            {onReanalyze && (
              <button
                type="button"
                className="ghost-btn source-insight-reanalyze"
                disabled={isAnalyzing}
                onClick={onReanalyze}
                title={
                  isAnalyzing
                    ? "Analysis is already running."
                    : "Refresh this track's source analysis and waveform."
                }
              >
                {isAnalyzing ? "Analyzing…" : "Re-analyze"}
              </button>
            )}
            {unreviewed ? (
              <button type="button" className="ghost-btn source-insight-ack" onClick={acknowledge}>
                Mark reviewed
              </button>
            ) : (
              <span className="source-insight-reviewed" aria-live="polite">
                Reviewed ✓
              </span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function statusWord(s: InsightStatus): string {
  return s === "problem" ? "Problem" : s === "caution" ? "Caution" : s === "info" ? "Note" : "OK";
}

function StatusGlyph({ status }: { status: InsightStatus }) {
  return (
    <span className={`source-insight-glyph is-${status}`} aria-hidden="true">
      {status === "problem" ? "✗" : status === "caution" ? "△" : status === "info" ? "·" : "✓"}
    </span>
  );
}
