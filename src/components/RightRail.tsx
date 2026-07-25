// Right-rail master-out / quality panels. MasterOutPanel is live transport
// telemetry only; QualityCheckPanel owns source/export analysis.

import { useEffect, useRef, useState, type ReactNode } from "react";
import type { AnalysisResult, QualityCheck } from "../bindings";
import type { RenderFeedback, RenderProgressState } from "../hooks/useTrackMaster";
import { DisabledReason } from "./fields";

type RightRailProps = {
  /// QualityCheckPanel uses this for the preflight checks when no
  /// export receipt has been generated yet.
  analysis: AnalysisResult | undefined;
  lastChecks: QualityCheck[] | undefined;
  /// Slot for the advanced rail cards (Delivery Profile, Advanced
  /// Controls, Per-Band Compressor, Delivery Format). App.tsx composes
  /// them as a fragment so the rail just renders the slot in the right
  /// place, between Quality Check and the sticky Export group.
  advancedSlot?: ReactNode;
  // Export action — promoted from the workspace into the right rail to
  // match the reference layout. Disabled until analysis exists and while
  // any render/export is in flight.
  exportMode?: "track" | "album";
  canExport: boolean;
  isExporting: boolean;
  isRendering: boolean;
  renderProgress?: RenderProgressState | null;
  renderFeedback?: RenderFeedback | null;
  cancelRenderPending?: boolean;
  isAnalyzing?: boolean;
  onExport: () => void;
  onCancelRender?: () => void;
  onReanalyze?: () => void;
  // UI restyle 2026-05-14: the secondary "Render audit WAV" action used
  // to live in the main StaleBar. Moved here so the playback strip can
  // become a quiet status indicator, while audit-WAV stays one click
  // away from Export Master — its natural neighbor.
  previewStale: boolean;
  canRenderPreview: boolean;
  onUpdatePreview: () => void;
};

type QualityRow = {
  key: string;
  ok: boolean;
  warn: boolean;
  crit: boolean;
  label: string;
  detail: string;
};

// MASTER OUT stereo peak meter scale, in dBFS. 0 at top = clipping, -36 floor.
const PEAK_SCALE_MIN = -36;
const PEAK_SCALE_MAX = 0;
// Streaming-safe ceiling line drawn across each bar.
const PEAK_CEIL_DBFS = -1;

export function RightRail({
  analysis,
  lastChecks,
  advancedSlot,
  exportMode = "track",
  canExport,
  isExporting,
  isRendering,
  renderProgress,
  renderFeedback,
  cancelRenderPending = false,
  isAnalyzing = false,
  onExport,
  onCancelRender,
  onReanalyze,
  previewStale,
  canRenderPreview,
  onUpdatePreview,
}: RightRailProps) {
  const qualityRows = qualityRowsFor(lastChecks, analysis);

  // U10 — disabled-action reasons computed once, then used for BOTH the
  // tooltip and the accessible description. Keeping them as one expression is
  // what stops the two drifting into saying different things.
  const auditDisabled = !canRenderPreview || isRendering || isExporting;
  const auditDisabledReason = isExporting
    ? "Unavailable while an export is in progress — they share render state."
    : isRendering
      ? "Unavailable while a render is in progress — they share render state."
      : !canRenderPreview
        ? "Analyze a track first."
        : undefined;
  const auditTitle =
    auditDisabledReason ??
    "Render a temporary WAV with the current settings so you can audit it in another player or DAW. Not required for live audition — the Original/Mastered toggle plays through the chain in real time.";

  const exportDisabled = !canExport || isExporting || isRendering;
  const exportDisabledReason = isExporting
    ? "An export is already running — it finishes or fails before the next one starts."
    : isRendering
      ? "Unavailable while a render-audit WAV is in progress — they share render state."
      : !canExport
        ? exportMode === "album"
          ? "Import album tracks first."
          : "Analyze a track first."
        : undefined;
  const needsReview =
    exportMode === "track" && canExport && hasReviewRows(qualityRows);
  const [reviewOpen, setReviewOpen] = useState(false);
  const activeRenderProgress =
    (isExporting || isRendering) && renderProgress
      ? {
          percent: Math.round(Math.max(0, Math.min(1, renderProgress.fraction)) * 100),
          label: `${renderProgress.kind[0].toUpperCase()}${renderProgress.kind.slice(
            1,
          )} render`,
        }
      : null;
  const visibleRenderFeedback =
    renderFeedback &&
    (exportMode === "album"
      ? renderFeedback.kind === "album"
      : renderFeedback.kind !== "album")
      ? renderFeedback
      : null;

  useEffect(() => {
    setReviewOpen(false);
  }, [analysis?.track_id, lastChecks]);

  const exportLabel =
    exportMode === "album"
      ? isExporting
        ? "Rendering Album..."
        : "Export Album"
      : isExporting
        ? "Exporting..."
        : needsReview
          ? "Export With Review"
          : "Export Master";

  const handlePrimaryExport = () => {
    if (!canExport || isExporting || isRendering) return;
    if (needsReview) {
      setReviewOpen(true);
      return;
    }
    onExport();
  };

  const handleExportAnyway = () => {
    // Honor the same in-flight guard the primary button enforces — otherwise a
    // second click while an export/render is running double-fires onExport (§6).
    if (!canExport || isExporting || isRendering) return;
    setReviewOpen(false);
    onExport();
  };

  const gateOpen = reviewOpen && needsReview;

  return (
    <aside className="right-rail">
      {/* UI_LAYOUT_REVISION_1600x940 L3 — rail order per spec:
          Quality Check → advancedSlot (Delivery / Advanced / Per-Band /
          Bit+SR cards, App.tsx composes) → sticky Export Master at
          bottom. Levels moved to the waveform deck's meters column;
          MasterOutPanel moved in L2.

          U10(a): while the review gate is up it is the single owner of those
          warnings, so the standing rail goes `inert` — out of the
          accessibility tree and out of tab order. A scrim alone is not enough:
          without this, the same warning is still readable behind it by a
          screen reader and still reachable by Tab, which is the duplicate
          wearing a disguise. */}
      <div className="right-rail-standing" inert={gateOpen}>
        <QualityCheckPanel
          checks={lastChecks}
          analysis={analysis}
          isAnalyzing={isAnalyzing}
          onReanalyze={onReanalyze}
        />
        {advancedSlot}
        <div className="right-rail-export-group">
          <details className="right-rail-tools">
            <summary>Tools</summary>
            <button
              type="button"
              className="ghost-btn right-rail-audit"
              onClick={onUpdatePreview}
              disabled={auditDisabled}
              title={auditTitle}
              // U10: the reason existed but only on hover. A keyboard user met a
              // dead button with no stated cause.
              aria-describedby={auditDisabledReason ? "audit-disabled-reason" : undefined}
            >
              {previewStale ? "Render audit WAV" : "Re-render audit WAV"}
            </button>
            <DisabledReason id="audit-disabled-reason" reason={auditDisabledReason} />
          </details>
          <button
            type="button"
            className="primary right-rail-export"
            onClick={handlePrimaryExport}
            disabled={exportDisabled}
            title={exportDisabledReason}
            aria-describedby={exportDisabledReason ? "export-disabled-reason" : undefined}
          >
            {exportLabel}
          </button>
          <DisabledReason id="export-disabled-reason" reason={exportDisabledReason} />
          {activeRenderProgress && (
            <div className="right-rail-render-progress" role="status" aria-live="polite">
              <div className="right-rail-render-progress-head">
                <span>
                  {activeRenderProgress.label} {activeRenderProgress.percent}%
                </span>
                <button
                  type="button"
                  className="ghost-btn right-rail-cancel-render"
                  onClick={onCancelRender}
                  disabled={cancelRenderPending || !onCancelRender}
                >
                  {cancelRenderPending ? "Cancelling..." : "Cancel"}
                </button>
              </div>
              <div
                className="right-rail-render-progress-track"
                role="progressbar"
                aria-label={activeRenderProgress.label}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={activeRenderProgress.percent}
              >
                <span
                  className="right-rail-render-progress-fill"
                  style={{ width: `${activeRenderProgress.percent}%` }}
                />
              </div>
            </div>
          )}
          {visibleRenderFeedback && (
            <p className="right-rail-render-feedback" role="status">
              {visibleRenderFeedback.message}
            </p>
          )}
        </div>
      </div>
      {gateOpen && (
        <ExportReviewDialog
          rows={qualityRows.filter((row) => row.warn || row.crit)}
          onAdjust={() => setReviewOpen(false)}
          onExportAnyway={handleExportAnyway}
        />
      )}
    </aside>
  );
}

// U10(a) — the pre-export review gate.
//
// This used to render inline inside the export group, which meant every
// warning was on screen TWICE at once: the QUALITY CHECK panel at the top of
// the rail listed it, and this gate listed it again a few hundred pixels
// below, REVIEW badge and all. Two owners, one warning.
//
// The gate is a decision surface, not a second report, so it is now a real
// modal. It keeps the detail because at 1360x740 the standing panel is usually
// scrolled out of view by the time you reach Export — a gate that made you
// scroll back up to learn what you were confirming would be worse than the
// duplicate it replaces. The standing rail is marked `inert` behind it, so
// exactly one copy is presented to anyone, by any means.
//
// Semantics match the app's existing dialogs (ChromeDialog,
// BackToStandardConfirm): role=dialog + aria-modal + labelled title + Escape +
// focus moved in.
function ExportReviewDialog({
  rows,
  onAdjust,
  onExportAnyway,
}: {
  rows: QualityRow[];
  onAdjust: () => void;
  onExportAnyway: () => void;
}) {
  const dialogRef = useRef<HTMLElement | null>(null);
  useEffect(() => {
    dialogRef.current?.focus();
    const onKeyDown = (e: KeyboardEvent) => {
      // Escape is "Adjust Settings", never "Export Anyway" — dismissing a
      // warning gate must not become a route to the flagged export.
      if (e.key === "Escape") onAdjust();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onAdjust]);
  return (
    <div className="export-review-backdrop" role="presentation" onClick={onAdjust}>
      <section
        ref={dialogRef}
        tabIndex={-1}
        className="export-review-panel"
        role="dialog"
        aria-modal="true"
        aria-labelledby="export-review-title"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="export-review-head">
          <span className="export-review-title" id="export-review-title">
            Review before export
          </span>
          <span className="quality-badge badge-warn">REVIEW</span>
        </header>
        <ul className="export-review-list">
          {rows.map((row) => (
            <li
              key={row.key}
              className={"export-review-row " + (row.crit ? "is-crit" : "is-warn")}
              title={row.detail}
            >
              <span className="quality-check-glyph" aria-hidden>
                {row.crit ? "✗" : "△"}
              </span>
              <span>{row.label}</span>
              {/* U9: the severity was carried only by a colour class and
                  a glyph marked aria-hidden, and the explanation only by
                  a hover tooltip. A screen-reader user heard the label
                  and nothing else — on the panel whose entire job is to
                  make you review a warning before exporting. */}
              <span className="sr-only">
                {row.crit ? "Critical. " : "Warning. "}
                {row.detail}
              </span>
            </li>
          ))}
        </ul>
        <div className="export-review-actions">
          <button type="button" className="ghost-btn" onClick={onAdjust}>
            Adjust Settings
          </button>
          <button type="button" className="primary" onClick={onExportAnyway}>
            Export Anyway
          </button>
        </div>
      </section>
    </div>
  );
}

export function MasterOutPanel({
  isAnalyzing,
  peakDbfs,
  peakLeftDbfs,
  peakRightDbfs,
  isPlaying,
  lufsMomentary,
  lufsIntegrated,
  landingPending = false,
  meterMode = "advanced",
}: {
  isAnalyzing: boolean;
  peakDbfs: number;
  peakLeftDbfs: number;
  peakRightDbfs: number;
  isPlaying: boolean;
  lufsMomentary: number;
  lufsIntegrated: number;
  /// True while Mastered audition plays hotter than the loudness target —
  /// the corrective landing gain is still being measured in the background.
  landingPending?: boolean;
  /// Standard uses plain language; Advanced keeps the exact technical terms.
  meterMode?: "standard" | "advanced";
}) {
  // This panel is a live output meter. Source/export analysis lives in
  // QualityCheckPanel and export receipts; mixing those fallback values into
  // this meter made the transport look hot while stopped.
  const live = (v: number): number | undefined =>
    isPlaying && v > -120 ? v : undefined;
  const liveL = live(peakLeftDbfs);
  const liveR = live(peakRightDbfs);
  const liveMomentary = live(lufsMomentary);
  const liveIntegrated = live(lufsIntegrated);
  // Live Peak readout = the louder of the two channels.
  const livePeak = live(Math.max(peakLeftDbfs, peakRightDbfs, peakDbfs));

  const momentaryDisplay =
    liveMomentary !== undefined ? liveMomentary.toFixed(1) : "—";
  const integratedDisplay =
    liveIntegrated !== undefined ? liveIntegrated.toFixed(1) : "—";
  const peakDisplay = livePeak !== undefined ? livePeak.toFixed(1) : "—";
  const readouts =
    meterMode === "standard"
      ? [
          {
            label: "Loudness",
            value: momentaryDisplay,
            unit: "LUFS",
            title:
              "Short live loudness window. This is not the selected target.",
          },
          {
            label: "Since Play",
            value: integratedDisplay,
            unit: "LUFS",
            title:
              "Average loudness since playback started. It changes with the section you are hearing.",
          },
          {
            label: "Peak",
            value: peakDisplay,
            unit: "dBFS",
            title: "Live digital peak level. Peaks use dBFS, not LUFS.",
          },
        ]
      : [
          {
            label: "Momentary LUFS",
            value: momentaryDisplay,
            unit: "",
            title:
              "Short-window live loudness. This is not the selected delivery target.",
          },
          {
            label: "Since-play LUFS",
            value: integratedDisplay,
            unit: "",
            title:
              "Integrated loudness over the current playback run, not the full export measurement.",
          },
          {
            label: "Live peak dBFS",
            value: peakDisplay,
            unit: "",
            title:
              "Live digital peak level. Use export receipts for rendered true-peak checks.",
          },
        ];

  return (
    <section className={`panel master-out ${isPlaying ? "is-live" : "is-idle"}`}>
      <header className="panel-head">
        <span className="panel-title">MASTER OUT</span>
        {isPlaying ? (
          <span className="panel-live-pill" title="Live stereo peak (L/R) plus loudness readouts, metering playback in real time.">
            <span className="panel-live-dot" aria-hidden /> LIVE
          </span>
        ) : isAnalyzing ? (
          <span className="panel-hint">analyzing…</span>
        ) : (
          <span className="panel-hint">idle</span>
        )}
      </header>
      <div className="lufs-meter">
        <div className="lufs-bars">
          <PeakMeterBar value={liveL} label="L" title="Left channel peak (dBFS)" />
          <PeakMeterBar value={liveR} label="R" title="Right channel peak (dBFS)" />
        </div>
        <PeakScale />
      </div>
      <dl className="master-readouts">
        {readouts.map((readout) => (
          <Readout key={readout.label} {...readout} />
        ))}
      </dl>
      {landingPending && (
        <p className="landing-note" role="status">
          Landing loudness… level settles in a moment.
        </p>
      )}
    </section>
  );
}

function PeakScale() {
  // dBFS ticks for the stereo peak meter: 0 (clipping) at top down to -36.
  const ticks = [0, -6, -12, -18, -24, -30, -36];
  return (
    <div className="lufs-scale">
      {ticks.map((db) => (
        <span key={db} className="lufs-tick">{db}</span>
      ))}
    </div>
  );
}

function PeakMeterBar({
  value,
  label,
  title,
}: {
  value: number | undefined;
  label: string;
  title?: string;
}) {
  // Map a dBFS peak into 0..1 fill against the -36..0 scale (0 = clip).
  const ratio = (db: number): number => {
    if (!Number.isFinite(db)) return 0;
    const clamped = Math.max(PEAK_SCALE_MIN, Math.min(PEAK_SCALE_MAX, db));
    return (clamped - PEAK_SCALE_MIN) / (PEAK_SCALE_MAX - PEAK_SCALE_MIN);
  };
  const fill = value !== undefined ? ratio(value) : 0;
  const clipping = value !== undefined && value > -0.1;
  const ceilRatio = ratio(PEAK_CEIL_DBFS);
  return (
    <div className={`lufs-bar${clipping ? " is-clipping" : ""}`} title={title}>
      <div className="lufs-bar-track" />
      <div className="lufs-bar-fill" style={{ height: `${fill * 100}%` }} />
      <div
        className="peak-ceil-line"
        style={{ bottom: `${ceilRatio * 100}%` }}
        title="-1 dBFS ceiling"
      />
      <span className="lufs-bar-label">{label}</span>
    </div>
  );
}

function Readout({
  label,
  value,
  unit,
  title,
}: {
  label: string;
  value: string;
  unit: string;
  title: string;
}) {
  return (
    <div className="readout" title={title}>
      <dt className="readout-label">{label}</dt>
      <dd className="readout-value">
        <span className="readout-number">{value}</span>
        {unit && <span className="readout-unit">{unit}</span>}
      </dd>
    </div>
  );
}

function qualityRowsFor(
  checks: QualityCheck[] | undefined,
  analysis: AnalysisResult | undefined,
): QualityRow[] {
  return checks && checks.length > 0
    ? checks.map((c, i) => ({
        key: `${c.code}-${i}`,
        ok: c.level === "info",
        warn: c.level === "warning",
        crit: c.level === "critical",
        label: friendlyCheckLabel(c),
        detail: c.message,
      }))
    : derivePreflightChecks(analysis);
}

function hasReviewRows(rows: QualityRow[]): boolean {
  return rows.some((row) => row.warn || row.crit);
}

function QualityCheckPanel({
  checks,
  analysis,
  isAnalyzing,
  onReanalyze,
}: {
  checks: QualityCheck[] | undefined;
  analysis: AnalysisResult | undefined;
  isAnalyzing: boolean;
  onReanalyze?: () => void;
}) {
  const hasExportChecks = !!checks && checks.length > 0;
  const rows = qualityRowsFor(checks, analysis);
  const showReanalyze = !hasExportChecks && !!onReanalyze;

  const overallSafe = rows.every((r) => r.ok);
  return (
    <section className={`panel quality-check ${overallSafe ? "is-safe" : "has-issues"}`}>
      <header className="panel-head quality-check-head">
        <span className="panel-title">{hasExportChecks ? "EXPORT CHECK" : "SOURCE CHECK"}</span>
        <div className="quality-check-head-actions">
          {showReanalyze && (
            <button
              type="button"
              className="ghost-btn quality-reanalyze"
              disabled={isAnalyzing}
              onClick={onReanalyze}
              title={
                isAnalyzing
                  ? "Analysis is already running."
                  : "Refresh this track's source analysis and waveform."
              }
            >
              Re-analyze
            </button>
          )}
          <span className={`quality-badge ${overallSafe ? "badge-safe" : "badge-warn"}`}>
            {overallSafe ? "SAFE" : "REVIEW"}
          </span>
        </div>
      </header>
      <ul className="quality-check-list">
        {rows.map((r) => (
          <li
            key={r.key}
            className={
              "quality-check-row " +
              (r.crit ? "is-crit" : r.warn ? "is-warn" : "is-ok")
            }
            title={r.detail}
          >
            <span className="quality-check-glyph" aria-hidden>
              {r.crit ? "✗" : r.warn ? "△" : "✓"}
            </span>
            <span className="quality-check-text">{r.label}</span>
            {/* U9: same as the export-review list — pass/warn/critical was
                conveyed by an aria-hidden glyph plus colour, and the detail by
                a tooltip. Both now exist as text. */}
            <span className="sr-only">
              {r.crit ? "Critical. " : r.warn ? "Warning. " : "Pass. "}
              {r.detail}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function friendlyCheckLabel(c: QualityCheck): string {
  // The export checks come in as short technical codes. The reference UI
  // uses plain-language one-liners; surface those when we can recognize the
  // code, fall back to the raw message otherwise.
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
      return "Heavy compression detected";
    case "bit_depth_low":
      return "Bit depth below 16 bits";
    case "sample_rate_mismatch":
      return "Sample rate does not match delivery";
    case "non_finite_metering":
      return "Non-finite loudness measurement";
    case "comp_density_on_compressed_source":
      return "Already-compressed source";
    default:
      return c.message;
  }
}

function derivePreflightChecks(analysis: AnalysisResult | undefined): QualityRow[] {
  if (!analysis) {
    return [
      {
        key: "pre-no-analysis",
        ok: false,
        warn: true,
        crit: false,
        label: "Awaiting analysis",
        detail: "Run Analyze to populate quality checks.",
      },
    ];
  }
  const tp = analysis.true_peak_dbtp;
  const lufs = analysis.lufs_integrated;
  const dr = analysis.dynamic_range_lu;
  return [
    {
      key: "tp",
      ok: tp <= -1.0,
      warn: tp > -1.0 && tp <= -0.1,
      crit: tp > -0.1,
      label: `Source true peak ${tp.toFixed(1)} dBTP`,
      detail: `Analyzed source true peak at ${tp.toFixed(2)} dBTP.`,
    },
    {
      key: "lufs",
      ok: lufs <= -8.0,
      warn: lufs > -8.0 && lufs <= -6.0,
      crit: lufs > -6.0,
      label: `Source loudness ${lufs.toFixed(1)} LUFS`,
      detail: `Analyzed source integrated loudness at ${lufs.toFixed(2)} LUFS.`,
    },
    {
      key: "dr",
      ok: dr >= 6.0,
      warn: dr >= 4.0 && dr < 6.0,
      crit: dr < 4.0,
      label: `Source dynamic range ${dr.toFixed(1)} LU`,
      detail: `Analyzed source dynamic range at ${dr.toFixed(2)} LU.`,
    },
  ];
}
