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
// lib/source-insight.ts. Opening the disclosure IS the acknowledgement
// (owner 2026-08-19): the pill disappears the moment the panel opens, and
// clicking the pill itself opens the panel. Findings keep their own status.
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

  // Opening = reviewing. Acknowledge on the open transition only, so a
  // fresh revision arriving while the panel is already open still gets its
  // pill (the user hasn't looked at the NEW numbers yet).
  const openAndAcknowledge = useCallback(() => {
    setOpen(true);
    if (unreviewed) onAcknowledge();
  }, [unreviewed, onAcknowledge]);

  return (
    <div ref={rootRef} className={"source-insight" + (open ? " is-open" : "")}>
      <div className="source-insight-row">
        <button
          type="button"
          className="source-insight-toggle"
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => (open ? setOpen(false) : openAndAcknowledge())}
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
            title="New analysis — click to review it"
            aria-label="New analysis to review — open source analysis"
            aria-expanded={open}
            aria-controls={panelId}
            onClick={(e) => {
              e.stopPropagation();
              openAndAcknowledge();
            }}
          >
            New
          </button>
        )}
      </div>

      {open && (
        <div id={panelId} className="source-insight-panel" role="region" aria-label="Source analysis">
          <div className="source-insight-panel-head">
            <span className="source-insight-panel-title">Source analysis</span>
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

/// The row for a track that has no analysis yet (failed decode, cancelled
/// analysis, or a project loaded before analysis ran). Without it the only
/// way to (re)run analysis in Advanced disappeared with the rail's Source
/// Check — and the decode-error toast tells people to "use Re-analyze".
export function SourceInsightEmpty({
  isAnalyzing = false,
  onReanalyze,
}: {
  isAnalyzing?: boolean;
  onReanalyze?: () => void;
}) {
  return (
    <div className="source-insight is-empty">
      <div className="source-insight-row">
        <span className="source-insight-toggle is-static" aria-disabled="true">
          <span className="source-insight-eyebrow">Insight</span>
          <span className="source-insight-dot is-info" aria-hidden="true" />
          <span className="source-insight-headline is-muted">
            {isAnalyzing ? "Analyzing source…" : "Not analyzed yet."}
          </span>
        </span>
        {onReanalyze && !isAnalyzing && (
          <button type="button" className="ghost-btn source-insight-reanalyze" onClick={onReanalyze}>
            Analyze
          </button>
        )}
      </div>
    </div>
  );
}
