// Right-rail master-out / quality panels. MasterOutPanel is live transport
// telemetry only; QualityCheckPanel owns source/export analysis.

import { useEffect, useState, type ReactNode } from "react";
import type { AnalysisResult, QualityCheck } from "../bindings";

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
  canExport: boolean;
  isExporting: boolean;
  isRendering: boolean;
  onExport: () => void;
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

const LUFS_SCALE_MIN = -36;
const LUFS_SCALE_MAX = -6;

export function RightRail({
  analysis,
  lastChecks,
  advancedSlot,
  canExport,
  isExporting,
  isRendering,
  onExport,
  previewStale,
  canRenderPreview,
  onUpdatePreview,
}: RightRailProps) {
  const qualityRows = qualityRowsFor(lastChecks, analysis);
  const needsReview = canExport && hasReviewRows(qualityRows);
  const [reviewOpen, setReviewOpen] = useState(false);

  useEffect(() => {
    setReviewOpen(false);
  }, [analysis?.track_id, lastChecks]);

  const exportLabel = isExporting
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
    setReviewOpen(false);
    onExport();
  };

  return (
    <aside className="right-rail">
      {/* UI_LAYOUT_REVISION_1600x940 L3 — rail order per spec:
          Quality Check → advancedSlot (Delivery / Advanced / Per-Band /
          Bit+SR cards, App.tsx composes) → sticky Export Master at
          bottom. Levels moved to the waveform deck's meters column;
          MasterOutPanel moved in L2. */}
      <QualityCheckPanel checks={lastChecks} analysis={analysis} />
      {advancedSlot}
      <div className="right-rail-export-group">
        <details className="right-rail-tools">
          <summary>Tools</summary>
          <button
            type="button"
            className="ghost-btn right-rail-audit"
            onClick={onUpdatePreview}
            disabled={!canRenderPreview || isRendering || isExporting}
            title={
              isExporting
                ? "Disabled while an export is in progress — they share render state."
                : !canRenderPreview
                ? "Import a track first."
                : "Render a temporary WAV with the current settings so you can audit it in another player or DAW. Not required for live audition — the Original/Mastered toggle plays through the chain in real time."
            }
          >
            {previewStale ? "Render audit WAV" : "Re-render audit WAV"}
          </button>
        </details>
        <button
          type="button"
          className="primary right-rail-export"
          onClick={handlePrimaryExport}
          disabled={!canExport || isExporting || isRendering}
          title={
            isRendering && !isExporting
              ? "Disabled while a render-audit WAV is in progress — they share render state."
              : !canExport
              ? "Analyze a track first."
              : undefined
          }
        >
          {exportLabel}
        </button>
        {reviewOpen && needsReview && (
          <section className="export-review-panel" aria-label="Export review">
            <header className="export-review-head">
              <span className="export-review-title">Review before export</span>
              <span className="quality-badge badge-warn">REVIEW</span>
            </header>
            <ul className="export-review-list">
              {qualityRows
                .filter((row) => row.warn || row.crit)
                .map((row) => (
                  <li
                    key={row.key}
                    className={
                      "export-review-row " + (row.crit ? "is-crit" : "is-warn")
                    }
                    title={row.detail}
                  >
                    <span className="quality-check-glyph" aria-hidden>
                      {row.crit ? "✗" : "△"}
                    </span>
                    <span>{row.label}</span>
                  </li>
                ))}
            </ul>
            <div className="export-review-actions">
              <button
                type="button"
                className="ghost-btn"
                onClick={() => setReviewOpen(false)}
              >
                Adjust Settings
              </button>
              <button
                type="button"
                className="primary"
                onClick={handleExportAnyway}
              >
                Export Anyway
              </button>
            </div>
          </section>
        )}
      </div>
    </aside>
  );
}

export function MasterOutPanel({
  isAnalyzing,
  peakDbfs,
  isPlaying,
  lufsMomentary,
  lufsIntegrated,
}: {
  isAnalyzing: boolean;
  peakDbfs: number;
  isPlaying: boolean;
  lufsMomentary: number;
  lufsIntegrated: number;
}) {
  // This panel is a live output meter. Source/export analysis lives in
  // QualityCheckPanel and export receipts; mixing those fallback values into
  // this meter made the transport look hot while stopped.
  const liveMomentary = isPlaying && lufsMomentary > -120 ? lufsMomentary : undefined;
  const liveIntegrated =
    isPlaying && lufsIntegrated > -120 ? lufsIntegrated : undefined;
  const liveTp = isPlaying && peakDbfs > -120 ? peakDbfs : undefined;

  const momentaryDisplay =
    liveMomentary !== undefined ? liveMomentary.toFixed(1) : "—";
  const integratedDisplay =
    liveIntegrated !== undefined ? liveIntegrated.toFixed(1) : "—";
  const peakDisplay = liveTp !== undefined ? liveTp.toFixed(1) : "—";

  return (
    <section className={`panel master-out ${isPlaying ? "is-live" : "is-idle"}`}>
      <header className="panel-head">
        <span className="panel-title">MASTER OUT</span>
        {isPlaying ? (
          <span className="panel-live-pill" title="Momentary bars + live integrated readout are metering the playback in real time.">
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
          <LufsBar
            value={liveMomentary}
            peakHold={liveIntegrated}
            label="M"
            title="Momentary loudness (BS.1770). Tick marks the integrated average."
          />
          <LufsBar
            value={liveIntegrated}
            label="I"
            title="Integrated loudness since playback started."
          />
        </div>
        <LufsScale />
        <PeakBar value={liveTp} />
      </div>
      <dl className="master-readouts">
        <Readout
          label="Momentary"
          value={momentaryDisplay}
          unit="LUFS"
        />
        <Readout
          label="Since Play"
          value={integratedDisplay}
          unit="LUFS"
        />
        <Readout
          label="Live Peak"
          value={peakDisplay}
          unit="dBFS"
        />
      </dl>
    </section>
  );
}

function LufsScale() {
  // Drawing the dB ticks alongside the meter bars. Matches the reference's
  // descending scale from -6 (top) down to -36 (bottom).
  const ticks = [-6, -12, -18, -24, -30, -36];
  return (
    <div className="lufs-scale">
      {ticks.map((db) => (
        <span key={db} className="lufs-tick">{db}</span>
      ))}
    </div>
  );
}

function LufsBar({
  value,
  peakHold,
  label,
  title,
}: {
  value: number | undefined;
  peakHold?: number | undefined;
  label: string;
  title?: string;
}) {
  // Map a dBFS value into 0..1 fill against the -36..-6 scale.
  const ratio = (db: number): number => {
    if (!Number.isFinite(db)) return 0;
    const clamped = Math.max(LUFS_SCALE_MIN, Math.min(LUFS_SCALE_MAX, db));
    return (clamped - LUFS_SCALE_MIN) / (LUFS_SCALE_MAX - LUFS_SCALE_MIN);
  };
  const fill = value !== undefined ? ratio(value) : 0;
  const peakRatio = peakHold !== undefined ? ratio(peakHold) : null;
  return (
    <div className="lufs-bar" title={title}>
      <div className="lufs-bar-track" />
      <div className="lufs-bar-fill" style={{ height: `${fill * 100}%` }} />
      {peakRatio !== null && peakRatio > 0 && (
        <div
          className="lufs-peak-hold"
          style={{ bottom: `calc(${peakRatio * 100}% - 1px)` }}
          title="Integrated average"
        />
      )}
      <span className="lufs-bar-label">{label}</span>
    </div>
  );
}

function PeakBar({ value }: { value: number | undefined }) {
  // Live peak gets its own narrow bar on a 0..-36 dBFS scale (0 at top means
  // clipping). Export/source true peak is shown in checks, not in this live
  // meter, because this value comes from the playback tick's sample peak.
  const TP_MIN = -36;
  const TP_MAX = 0;
  let fill = 0;
  let tone: "ok" | "warn" | "hot" = "ok";
  if (value !== undefined && Number.isFinite(value)) {
    const clamped = Math.max(TP_MIN, Math.min(TP_MAX, value));
    fill = (clamped - TP_MIN) / (TP_MAX - TP_MIN);
    if (value > -0.1) tone = "hot";
    else if (value > -1.0) tone = "warn";
  }
  return (
    <div className={`tp-bar tp-${tone}`}>
      <div className="tp-bar-track" />
      <div className="tp-bar-fill" style={{ height: `${fill * 100}%` }} />
      <div className="tp-clip-line" title="-1 dBFS live warning line" />
      <span className="tp-bar-label">PK</span>
    </div>
  );
}

function Readout({
  label,
  value,
  unit,
}: {
  label: string;
  value: string;
  unit: string;
}) {
  return (
    <div className="readout">
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
}: {
  checks: QualityCheck[] | undefined;
  analysis: AnalysisResult | undefined;
}) {
  const hasExportChecks = !!checks && checks.length > 0;
  const rows = qualityRowsFor(checks, analysis);

  const overallSafe = rows.every((r) => r.ok);
  return (
    <section className={`panel quality-check ${overallSafe ? "is-safe" : "has-issues"}`}>
      <header className="panel-head">
        <span className="panel-title">{hasExportChecks ? "EXPORT CHECK" : "SOURCE CHECK"}</span>
        <span className={`quality-badge ${overallSafe ? "badge-safe" : "badge-warn"}`}>
          {overallSafe ? "SAFE" : "REVIEW"}
        </span>
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
