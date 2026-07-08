import { useState } from "react";
import { api } from "../lib/api";
import { formatDuration } from "../lib/time-format";
import type { ImportedTrack, QualityCheck, QualityLevel } from "../bindings";
import type { ExportReceipt } from "../hooks/useTrackMaster";

export function ExportReceiptCard({
  receipt,
  track,
  onClose,
}: {
  receipt: ExportReceipt;
  // The source track behind this receipt (selectedTrack at the render site).
  // Supplies the Track section's identity line; the receipt payload itself
  // never carried it. `null` only in the degenerate no-selection case.
  track: ImportedTrack | null;
  onClose: () => void;
}) {
  const [copiedPath, setCopiedPath] = useState<string | null>(null);
  const reveal = async (path: string) => {
    if (!path) return;
    try {
      await api.openOutput(path);
    } catch (err) {
      console.error("openOutput failed", err);
    }
  };
  // Copy the full path to the clipboard. Reveal (open in file manager) stays
  // the row's primary action; copy is a distinct sibling button so the two
  // affordances never nest. The check state self-clears after a beat.
  const copy = async (path: string) => {
    const clip = navigator.clipboard;
    if (!path || !clip) return;
    try {
      await clip.writeText(path);
      setCopiedPath(path);
      window.setTimeout(
        () => setCopiedPath((current) => (current === path ? null : current)),
        1600,
      );
    } catch (err) {
      console.error("copy path failed", err);
    }
  };
  const paths = receipt.job.output_paths;
  const measurements = receipt.job.measurements ?? null;
  const quality = exportQualitySummary(receipt.checks);
  // A Critical quality row means the saved file needs attention — don't present
  // it as an unqualified "complete". The file IS written and valid (criticals are
  // advisory format/loudness flags), so "saved" stays accurate while the header
  // matches the "Needs attention" medallion instead of contradicting it. (F4.)
  const headerTitle =
    quality.tone === "attention"
      ? "Export saved — needs attention"
      : "Export complete";
  const journeySteps = ["Analyze", "Master", "Quality", "Saved"];
  return (
    <div className="receipt-backdrop" onClick={onClose}>
      <div className="receipt" onClick={(e) => e.stopPropagation()}>
        <header className="receipt-header">
          <div className="receipt-title-group">
            <span className="receipt-eyebrow">
              Track master
            </span>
            <h2>{headerTitle}</h2>
          </div>
          <div className={`receipt-medallion receipt-medallion-${quality.tone}`}>
            <span className="receipt-medallion-label">{quality.label}</span>
            <span className="receipt-medallion-detail receipt-summary">
              {quality.detail}
            </span>
          </div>
          <button type="button" className="toast-close" onClick={onClose} aria-label="Close">
            ×
          </button>
        </header>
        <ol className="receipt-journey" aria-label="Export steps">
          {journeySteps.map((step) => (
            <li key={step} className="receipt-journey-step is-complete">
              <span className="receipt-journey-dot" aria-hidden />
              <span>{step}</span>
            </li>
          ))}
        </ol>
        {track && (
          <section className="receipt-track" aria-label="Track">
            <div className="receipt-section-title">Track</div>
            <h3 className="receipt-track-name">{track.display_name}</h3>
            <span className="track-meta-line">{trackMetaLine(track)}</span>
          </section>
        )}
        <div className="receipt-section-title">
          {paths.length === 1 ? "File saved" : "Files saved"}
        </div>
        <div className="receipt-paths">
          {paths.map((path, i) => (
            <div key={path + i} className="receipt-file">
              <button
                type="button"
                className="receipt-file-open"
                onClick={() => reveal(path)}
                title="Reveal in file manager"
              >
                <span className="receipt-path-name">
                  {fileNameFromPath(path)}
                </span>
                <span className="receipt-path-full">
                  {path}
                </span>
              </button>
              <button
                type="button"
                className={
                  "receipt-file-copy" +
                  (copiedPath === path ? " is-copied" : "")
                }
                onClick={() => copy(path)}
                aria-label={
                  copiedPath === path ? "Path copied" : "Copy file path"
                }
                title="Copy path"
              >
                {copiedPath === path ? <CheckGlyph /> : <CopyGlyph />}
              </button>
            </div>
          ))}
        </div>
        {measurements && (
          <div className="receipt-render-meta" aria-label="Rendered format">
            <span>{formatSampleRate(measurements.sample_rate)}</span>
            <span>{formatBitDepth(measurements.bit_depth)}</span>
          </div>
        )}
        {measurements && (
          // 2026-06-09 export-metrics inquiry: these delivered-master numbers
          // were measured and carried on the payload all along but never
          // rendered — the only loudness on the receipt was the
          // source-describing adaptive digest below. Values describe the
          // written file (post-landing).
          <div className="receipt-render-meta" aria-label="Delivered master measurements">
            <span>Master {measurements.lufs_integrated.toFixed(1)} LUFS</span>
            <span>TP {measurements.true_peak_dbtp.toFixed(2)} dBTP</span>
            <span>LRA {measurements.dynamic_range_lu.toFixed(1)} LU</span>
          </div>
        )}
        {measurements && (
          // B5 — adaptive-DSP traceability: a delivered master records what
          // adaptation produced it (digest present = guardrails were active).
          <div className="receipt-render-meta" aria-label="Adaptive DSP">
            {measurements.source_profile_digest ? (
              <>
                <span>
                  Adaptive{" "}
                  {Math.round((measurements.effective_adaptive_strength ?? 0) * 100)}%
                </span>
                <span title="Source profile that drove adaptation — these describe the SOURCE, not the delivered master">
                  {/* Visible "Source" prefix (export-metrics inquiry 2026-06-09):
                      a hover-only tooltip let these source stats read as master
                      measurements right under the output chips. */}
                  Source · {measurements.source_profile_digest}
                </span>
              </>
            ) : (
              <span>Adaptive: off</span>
            )}
            {measurements.compression_digest && (
              <span title="Backend-resolved adaptive compressor guard summary">
                Compression · {measurements.compression_digest}
              </span>
            )}
          </div>
        )}
        {receipt.checks.length > 0 && (
          <div className="receipt-checks">
            <div className="receipt-section-title">Quality notes</div>
            {receipt.checks.map((c, i) => (
              <CheckRow key={i} check={c} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function exportQualitySummary(checks: QualityCheck[]): {
  tone: "clean" | "review" | "attention";
  label: string;
  detail: string;
} {
  const critical = checks.filter((c) => c.level === "critical").length;
  const warning = checks.filter((c) => c.level === "warning").length;
  if (critical > 0) {
    return {
      tone: "attention",
      label: "Needs attention",
      detail: pluralize(critical, "critical item"),
    };
  }
  if (warning > 0) {
    return {
      tone: "review",
      label: "Review",
      // Pluralize only the noun, not the whole phrase — pluralize() appends "s"
      // to its argument, so "item to review" became "items to reviews".
      detail: `${pluralize(warning, "item")} to review`,
    };
  }
  return {
    tone: "clean",
    label: "Clean",
    detail: checks.length > 0 ? "No issues detected" : "Saved",
  };
}

function pluralize(count: number, singular: string): string {
  return `${count} ${singular}${count === 1 ? "" : "s"}`;
}

function fileNameFromPath(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

// Copy affordance glyphs — inline SVG so they inherit currentColor and need no
// asset import. CheckGlyph is the transient confirmation after a copy.
function CopyGlyph() {
  return (
    <svg
      width={15}
      height={15}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

function CheckGlyph() {
  return (
    <svg
      width={15}
      height={15}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2.4}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      <path d="M20 6 9 17l-5-5" />
    </svg>
  );
}

// The Track section's quiet identity line — the 13c metadata-diet dialect
// (`.track-meta-line`): each fact stated once, in spec order
// duration · format · sample rate · channels. Every part is guarded so a
// null field simply drops out rather than rendering an empty segment.
function trackMetaLine(track: ImportedTrack): string {
  const parts: string[] = [];
  const duration = formatDuration(track.duration_seconds);
  if (duration) parts.push(duration);
  if (track.source_format) parts.push(track.source_format.toUpperCase());
  if (track.sample_rate) parts.push(formatSampleRate(track.sample_rate));
  if (track.channels) parts.push(channelLabel(track.channels));
  return parts.join(" · ");
}

// Matches the app's existing channel wording (App.tsx track chips).
function channelLabel(channels: number): string {
  return channels === 1 ? "Mono" : channels === 2 ? "Stereo" : `${channels}ch`;
}

// Exported for reuse: StandardView.tsx and AlbumPanel.tsx still hardcode
// "44.1 kHz · 24-bit"-style strings these could absorb.
export function formatSampleRate(sampleRate: number): string {
  return sampleRate >= 1000
    ? `${(sampleRate / 1000).toFixed(sampleRate % 1000 === 0 ? 0 : 1)} kHz`
    : `${sampleRate} Hz`;
}

export function formatBitDepth(bitDepth: number): string {
  return bitDepth === 32 ? "32-bit float" : `${bitDepth}-bit`;
}

function CheckRow({ check }: { check: QualityCheck }) {
  return (
    <div className={"check-row level-" + levelClass(check.level)}>
      <span className="check-level">{check.level}</span>
      <span className="check-msg">{check.message}</span>
    </div>
  );
}

function levelClass(level: QualityLevel): string {
  return level;
}
