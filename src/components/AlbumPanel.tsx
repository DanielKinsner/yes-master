// Phase B Step 4: Album Master panel.
//
// Top-strip control surface for album-mode mastering. Shows:
//   * Album flow dropdown (4 named curves)
//   * Flow amount slider
//   * Album title input
//   * Last export report when present
//
// Per-track DSP is still edited via the regular Tone Shape / Macros / Advanced
// controls on whichever track the user has selected from the sidebar. The
// album layer only modulates the per-track LUFS target via arc + character.

import type { AlbumArcKind, ImportedTrack } from "../bindings";
import { ALBUM_ARC_DISPLAY } from "../bindings";
import { formatBitDepth, formatSampleRate } from "./ExportReceiptCard";
import type { AlbumRenderReport } from "../lib/api";
import { formatDuration } from "../lib/time-format";

function formatChannelCount(channels: number): string {
  if (channels === 1) return "mono";
  if (channels === 2) return "stereo";
  return `${channels} ch`;
}

type AlbumPanelProps = {
  tracks: ImportedTrack[];
  albumArcKind: AlbumArcKind;
  albumIntensity: number;
  albumTitle: string;
  albumExportReport: AlbumRenderReport | null;
  onAlbumArc: (kind: AlbumArcKind) => void;
  onAlbumIntensity: (v: number) => void;
  onAlbumTitle: (v: string) => void;
};

export function AlbumPanel({
  tracks,
  albumArcKind,
  albumIntensity,
  albumTitle,
  albumExportReport,
  onAlbumArc,
  onAlbumIntensity,
  onAlbumTitle,
}: AlbumPanelProps) {
  const arcKinds: AlbumArcKind[] = [
    "cinematic",
    "afterhours",
    "club-peak",
    "fever-dream",
  ];
  const totalSeconds = tracks.reduce(
    (acc, t) => acc + (t.duration_seconds ?? 0),
    0,
  );
  const renderedRate =
    albumExportReport && formatSampleRate(albumExportReport.rendered_sample_rate);
  const requestedRate =
    albumExportReport?.requested_sample_rate == null
      ? "Auto"
      : formatSampleRate(albumExportReport.requested_sample_rate);
  const sourceRates = Array.from(
    new Set(albumExportReport?.source_sample_rates ?? []),
  ).sort((a, b) => a - b);
  const upsampledRates =
    albumExportReport == null
      ? []
      : sourceRates.filter((rate) => rate < albumExportReport.rendered_sample_rate);
  const sourceChannels = Array.from(
    new Set(albumExportReport?.source_channels ?? []),
  ).sort((a, b) => a - b);
  const upmixedChannels =
    albumExportReport == null
      ? []
      : sourceChannels.filter(
          (channels) => channels < albumExportReport.rendered_channels,
        );
  const foldedChannels =
    albumExportReport == null
      ? []
      : sourceChannels.filter(
          (channels) => channels > albumExportReport.rendered_channels,
        );
  const requestedMismatch =
    albumExportReport?.requested_sample_rate != null &&
    albumExportReport.requested_sample_rate !== albumExportReport.rendered_sample_rate;
  return (
    <section className="album-panel">
      <header className="album-panel-head">
        <div className="album-panel-summary">
          <span className="section-label">Album</span>
          <span className="album-panel-stat">
            <strong>{tracks.length}</strong> tracks
            {totalSeconds > 0 && (
              <>
                <span className="dim"> · </span>
                <strong>{formatDuration(totalSeconds)}</strong>
              </>
            )}
          </span>
        </div>
        <input
          type="text"
          className="album-title-input"
          value={albumTitle}
          placeholder="Album title…"
          onChange={(e) => onAlbumTitle(e.target.value)}
          maxLength={120}
        />
      </header>
      <div className="album-panel-controls">
        <label className="adv-label" htmlFor="album-arc-select">
          Album flow
        </label>
        <select
          id="album-arc-select"
          className="loudness-profile-select"
          value={albumArcKind}
          onChange={(e) => onAlbumArc(e.target.value as AlbumArcKind)}
        >
          {arcKinds.map((k) => (
            <option key={k} value={k}>
              {ALBUM_ARC_DISPLAY[k]}
            </option>
          ))}
        </select>
        <label className="adv-label" htmlFor="album-intensity-range">
          Flow amount
        </label>
        <input
          id="album-intensity-range"
          type="range"
          min={0}
          max={2}
          step={0.05}
          value={albumIntensity}
          onChange={(e) => onAlbumIntensity(parseFloat(e.target.value))}
          className="album-intensity-range"
        />
        <span className="album-intensity-value">
          ×{albumIntensity.toFixed(2)}
        </span>
      </div>
      {albumExportReport && (
        <div className="album-export-receipt">
          <span className="album-export-receipt-label">Last export:</span>
          <code className="album-export-receipt-path">
            {albumExportReport.album_wav_path}
          </code>
          <span className="album-export-receipt-meta">
            {albumExportReport.tracks.length} tracks · rendered {renderedRate} /{" "}
            {formatBitDepth(albumExportReport.bit_depth)} /{" "}
            {formatChannelCount(albumExportReport.rendered_channels)} ·
            requested {requestedRate}
            {requestedMismatch && `, got ${renderedRate}`} · manifest:{" "}
            {albumExportReport.manifest_path}
          </span>
          {upsampledRates.length > 0 && (
            <span className="album-export-receipt-advisory">
              Upsampled source {upsampledRates.map(formatSampleRate).join(", ")}
            </span>
          )}
          {upmixedChannels.length > 0 && (
            <span className="album-export-receipt-advisory">
              Upmixed source {upmixedChannels.map(formatChannelCount).join(", ")}
            </span>
          )}
          {foldedChannels.length > 0 && (
            <span className="album-export-receipt-advisory">
              Folded source {foldedChannels.map(formatChannelCount).join(", ")} to{" "}
              {formatChannelCount(albumExportReport.rendered_channels)}
            </span>
          )}
        </div>
      )}
    </section>
  );
}
