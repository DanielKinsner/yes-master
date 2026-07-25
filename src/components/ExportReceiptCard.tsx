import { useEffect, useRef, useState, type CSSProperties } from "react";
import { api } from "../lib/api";
import { formatDuration } from "../lib/time-format";
import { presetCopy } from "../lib/preset-copy";
import {
  buildQualityRows,
  type QualityRowState,
} from "../lib/receipt-quality";
import { PresetIcon, PRESET_ACCENT } from "./PresetIcon";
import { intensityLabel } from "./Knob";
import {
  DELIVERY_PROFILE_DISPLAY,
  DELIVERY_PROFILE_TARGET_LUFS,
} from "../bindings";
import type {
  AnalysisResult,
  DeliveryProfile,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
} from "../bindings";
import type { ExportReceipt } from "../hooks/useTrackMaster";

export function ExportReceiptCard({
  receipt,
  track,
  settings,
  analysis,
  onClose,
}: {
  receipt: ExportReceipt;
  // The source track behind this receipt (selectedTrack at the render site).
  // Supplies the Track section's identity line; the receipt payload itself
  // never carried it. `null` only in the degenerate no-selection case.
  track: ImportedTrack | null;
  // The settings that produced this master (selectedSettings) — supplies the
  // Mastering Style block (preset + intensity) and the delivery-profile target.
  settings: MasteringSettings;
  // The source analysis (selectedAnalysis) — supplies the Quality Check rows'
  // source measurements. `null` if the analysis isn't available.
  analysis: AnalysisResult | null;
  onClose: () => void;
}) {
  const [copiedPath, setCopiedPath] = useState<string | null>(null);
  // Real build stamp for the footer: version · git hash · build time, resolved
  // from the build_info command (same source as the Help build stamp). Absent
  // wherever the command isn't available (e.g. tests) — the footer degrades to
  // the wordmark alone rather than showing a fabricated version.
  const [buildInfo, setBuildInfo] = useState<string | null>(null);
  // Focus moves into the receipt when it opens and Escape closes it, matching
  // ChromeDialog. Without this the dialog role would be a claim the keyboard
  // behaviour did not honour.
  const dialogRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    dialogRef.current?.focus();
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);
  useEffect(() => {
    let cancelled = false;
    Promise.resolve(api.buildInfo?.())
      .then((info) => {
        if (!cancelled && info) setBuildInfo(info);
      })
      .catch(() => {
        /* build stamp is best-effort; omit silently on failure */
      });
    return () => {
      cancelled = true;
    };
  }, []);
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
  const qualityRows = buildQualityRows(receipt.checks, analysis);
  // A Critical quality row means the saved file needs attention — don't present
  // it as an unqualified "complete". The file IS written and valid (criticals are
  // advisory format/loudness flags), so "saved" stays accurate while the header
  // matches the "Needs attention" medallion instead of contradicting it. (F4.)
  const headerTitle =
    quality.tone === "attention"
      ? "Export saved — needs attention"
      : "Export complete";
  const headerSubtitle =
    quality.tone === "clean"
      ? "Your track has been mastered and is ready."
      : `Your master is saved — ${quality.detail}.`;
  // The mock's calm reassurance badge; for a flagged export it turns into the
  // honest warning cue rather than an unconditional "verified".
  const badge =
    quality.tone === "clean"
      ? { title: "File processed and verified", detail: "by the YES Master engine" }
      : { title: quality.label, detail: quality.detail };
  return (
    // U10(a) — the receipt has always been a visual modal (full-screen
    // backdrop over the still-mounted rail) but carried no modal semantics. So
    // the receipt's Quality check rows and the rail's EXPORT CHECK rows — the
    // same export checks, rendered from the same payload — were BOTH in the
    // accessibility tree at once. Sighted users saw one; screen-reader users
    // met the same warning twice with no indication either was behind a
    // dialog. `aria-modal` makes the receipt the single owner while it is up.
    <div className="receipt-backdrop" role="presentation" onClick={onClose}>
      <div
        ref={dialogRef}
        tabIndex={-1}
        className="receipt"
        role="dialog"
        aria-modal="true"
        aria-labelledby="export-receipt-title"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="receipt-header">
          <button type="button" className="toast-close receipt-close" onClick={onClose} aria-label="Close">
            ×
          </button>
          <div className="receipt-header-main">
            <div className="receipt-headline">
              <span className="receipt-wordmark">
                <WordmarkGlyph />
                YES Master
              </span>
              <h2 className="receipt-title" id="export-receipt-title">
                {headerTitle}
              </h2>
              <p className="receipt-subtitle">{headerSubtitle}</p>
            </div>
            <div className={`receipt-verified receipt-verified-${quality.tone}`}>
              <span className="receipt-verified-icon" aria-hidden>
                {quality.tone === "clean" ? <CircleCheckGlyph /> : <CircleAlertGlyph />}
              </span>
              <span className="receipt-verified-text">
                <span className="receipt-verified-title">{badge.title}</span>
                <span className="receipt-verified-detail receipt-summary">
                  {badge.detail}
                </span>
              </span>
            </div>
          </div>
        </header>
        <div className="receipt-body">
          <div className="receipt-col">
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
        <MasteringStyle settings={settings} />
          </div>
          <div className="receipt-col">
            <div className="receipt-panel">
        {measurements && (
          // Delivered-master results. These numbers describe the written file
          // (post-landing), measured and carried on the payload all along.
          <section className="receipt-results" aria-label="Results">
            <div className="receipt-section-title">Results</div>
            <dl className="receipt-result-list">
              <div className="receipt-result-row">
                <dt>Integrated loudness</dt>
                <dd>{measurements.lufs_integrated.toFixed(1)} LUFS</dd>
              </div>
              <div className="receipt-result-row">
                <dt>True peak</dt>
                <dd>{measurements.true_peak_dbtp.toFixed(2)} dBTP</dd>
              </div>
              <div className="receipt-result-row">
                {/* Measured dynamic range — deliberately NOT labelled "LRA":
                    the engine measures dynamic range, not EBU Loudness Range. */}
                <dt>Dynamic range</dt>
                <dd>{measurements.dynamic_range_lu.toFixed(1)} LU</dd>
              </div>
              <div className="receipt-result-row receipt-result-row-target">
                <dt>Mastering target</dt>
                <dd>
                  <span className="receipt-target-chip">
                    {masteringTargetLabel(settings.delivery_profile)}
                  </span>
                </dd>
              </div>
            </dl>
          </section>
        )}
        <section className="receipt-quality" aria-label="Quality check">
          <div className="receipt-section-title">Quality check</div>
          <ul className="receipt-quality-list">
            {qualityRows.map((row) => (
              <li key={row.key} className={`receipt-quality-row is-${row.state}`}>
                <span
                  className="receipt-quality-icon"
                  role="img"
                  aria-label={QUALITY_STATE_LABEL[row.state]}
                >
                  {row.state === "ok" ? <CheckGlyph /> : <AlertGlyph />}
                </span>
                <span className="receipt-quality-text">
                  <span className="receipt-quality-label">{row.label}</span>
                  {row.note && (
                    <span className="receipt-quality-note">{row.note}</span>
                  )}
                </span>
                {row.value && (
                  <span className="receipt-quality-value">{row.value}</span>
                )}
              </li>
            ))}
          </ul>
        </section>
        {measurements && (
          <section className="receipt-format" aria-label="Audio format">
            <div className="receipt-section-title">Audio format</div>
            <dl className="receipt-result-list">
              <div className="receipt-result-row">
                <dt>Bit depth</dt>
                <dd>{formatBitDepth(measurements.bit_depth)}</dd>
              </div>
              <div className="receipt-result-row">
                <dt>Sample rate</dt>
                <dd>{formatSampleRate(measurements.sample_rate)}</dd>
              </div>
              <div className="receipt-result-row">
                <dt>File type</dt>
                <dd>{fileTypeFromPath(paths[0]) || "—"}</dd>
              </div>
            </dl>
          </section>
        )}
            </div>
          </div>
        </div>
        {measurements && (
          // B5 — adaptive-DSP traceability. Kept honest and available but tucked
          // behind a collapsed disclosure so it doesn't clutter the receipt
          // (owner decision 2026-07-08); the "Source ·" labelling from the
          // 2026-06-09 inquiry is preserved inside.
          <details className="receipt-details">
            <summary className="receipt-details-summary">Advanced details</summary>
            <div className="receipt-details-body" aria-label="Adaptive DSP">
              {measurements.source_profile_digest ? (
                <>
                  <span>
                    Adaptive{" "}
                    {Math.round((measurements.effective_adaptive_strength ?? 0) * 100)}%
                  </span>
                  <span title="Source profile that drove adaptation — these describe the SOURCE, not the delivered master">
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
          </details>
        )}
        <footer className="receipt-footer">
          <span className="receipt-footer-stamp" title="Version · git hash · build time">
            YES Master{buildInfo ? ` · ${buildInfo}` : ""}
          </span>
          <span className="receipt-footer-time">
            Exported {formatExportedAt(receipt.job.started_at_iso)}
          </span>
        </footer>
      </div>
    </div>
  );
}

const QUALITY_STATE_LABEL: Record<QualityRowState, string> = {
  ok: "Clean",
  warning: "Warning",
  critical: "Critical",
};

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

// Delivered file type from the output path extension (e.g. "…/x.wav" -> "WAV").
// Empty when the path has no extension rather than inventing a format.
function fileTypeFromPath(path: string | undefined): string {
  if (!path) return "";
  const name = path.split(/[\\/]/).pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toUpperCase() : "";
}

// Footer timestamp. `started_at_iso` is the only time the receipt carries; a
// track master render is near-instant, so it stamps the export. Labelled
// "Exported" (not "Completed") to stay honest about which time this is.
function formatExportedAt(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString(undefined, { dateStyle: "long", timeStyle: "short" });
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

// Small waveform wordmark glyph beside "YES Master" in the header.
function WordmarkGlyph() {
  return (
    <svg
      width={17}
      height={17}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      aria-hidden
    >
      <path d="M3 12h1.5" />
      <path d="M7 7v10" />
      <path d="M11 4v16" />
      <path d="M15 8v8" />
      <path d="M19 10v4" />
      <path d="M21.5 12H21" />
    </svg>
  );
}

// Verified-badge icons: a ringed check for a clean export, a ringed alert for a
// flagged one. The ring + colour come from the badge's tone class.
function CircleCheckGlyph() {
  return (
    <svg width={26} height={26} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="12" cy="12" r="9" />
      <path d="M8.5 12.2l2.4 2.4 4.6-4.8" />
    </svg>
  );
}

function CircleAlertGlyph() {
  return (
    <svg width={26} height={26} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 8v4.5" />
      <path d="M12 16h.01" />
    </svg>
  );
}

// Warning/critical marker for a flagged Quality Check row. Colour is carried by
// the row's is-warning / is-critical class, not the glyph.
function AlertGlyph() {
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
      <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z" />
      <path d="M12 9v4" />
      <path d="M12 17h.01" />
    </svg>
  );
}

// Mastering Style block: the preset's own orb art (same PNGs as the Styles
// tiles) + its real label and one-line blurb, beside the intensity dial. The
// arc + orb glow inherit the preset's accent so it reads as the same identity
// the user picked.
function MasteringStyle({ settings }: { settings: MasteringSettings }) {
  const { label, blurb } = presetCopy(settings.preset);
  const accent = PRESET_ACCENT[settings.preset.kind];
  return (
    <section
      className="receipt-style"
      aria-label="Mastering style"
      style={{ "--preset-accent": accent } as CSSProperties}
    >
      <div className="receipt-style-main">
        <span className="receipt-style-orb">
          <PresetIcon kind={settings.preset.kind} className="receipt-style-orb-img" />
        </span>
        <div className="receipt-style-text">
          <div className="receipt-section-title">Mastering style</div>
          <div className="receipt-style-name">{label}</div>
          <p className="receipt-style-blurb">{blurb}</p>
        </div>
      </div>
      <IntensityDial intensity={settings.intensity} />
    </section>
  );
}

// Radial intensity dial. Shows the REAL intensity percentage and the app's REAL
// intensity label (Subtle/Restrained/Moderate/Driving/Aggressive) — the mock's
// "Aggressive at 50%" is fiction; 50% is "Moderate" here.
function IntensityDial({ intensity }: { intensity: number }) {
  const value = Math.min(1, Math.max(0, intensity));
  const pct = Math.round(value * 100);
  const label = intensityLabel(intensity);
  const radius = 30;
  const circumference = 2 * Math.PI * radius;
  const filled = value * circumference;
  return (
    <div className="receipt-intensity">
      <div className="receipt-section-title">Preset intensity</div>
      <div
        className="receipt-dial"
        role="img"
        aria-label={`Intensity ${pct} percent, ${label}`}
      >
        <svg viewBox="0 0 72 72" className="receipt-dial-svg" aria-hidden>
          <circle className="receipt-dial-track" cx="36" cy="36" r={radius} />
          <circle
            className="receipt-dial-value"
            cx="36"
            cy="36"
            r={radius}
            strokeDasharray={`${filled} ${circumference}`}
            transform="rotate(-90 36 36)"
          />
        </svg>
        <span className="receipt-dial-pct">{pct}%</span>
      </div>
      <div className="receipt-intensity-label">{label}</div>
    </div>
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

// The Mastering Target chip: the real delivery-profile name + its target LUFS.
// `custom` (and any profile without a target) shows the name alone rather than
// inventing a number.
function masteringTargetLabel(profile: DeliveryProfile): string {
  const name = DELIVERY_PROFILE_DISPLAY[profile];
  const target = DELIVERY_PROFILE_TARGET_LUFS[profile];
  return target != null ? `${name} · ${target.toFixed(1)} LUFS` : name;
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
