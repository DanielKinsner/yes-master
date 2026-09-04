// Album export receipt — the post-export delivery summary + advisories.
//
import { formatBitDepth, formatSampleRate } from "./ExportReceiptCard";
import type { AlbumRenderReport, AlbumTrackRenderRecord } from "../lib/api";
import { trackCountLabel } from "../lib/album-copy";

function formatChannelCount(channels: number): string {
  if (channels === 1) return "mono";
  if (channels === 2) return "stereo";
  return `${channels} ch`;
}

function TrackResult({ track }: { track: AlbumTrackRenderRecord }) {
  const filename = track.output_path.split(/[\\/]/).pop() || `Track ${track.position}`;
  const target = track.target_lufs;
  const peak = track.true_peak_dbtp;
  const ceiling = track.ceiling_dbtp;
  const hasLoudness = Number.isFinite(track.measured_lufs) && track.measured_lufs > -70;
  const hasTarget = target != null && Number.isFinite(target);
  const hasPeak = peak != null && Number.isFinite(peak);
  const hasCeiling = ceiling != null && Number.isFinite(ceiling);
  const difference = hasTarget && hasLoudness ? track.measured_lufs - target : 0;
  const aboveCeiling = hasPeak && hasCeiling && peak > ceiling + 0.05;
  return (
    <li className="album-track-result">
      <strong title={track.output_path}>{filename}</strong>
      {track.override_album && <span className="album-track-result-note">Own settings</span>}
      <dl>
        <div>
          <dt>Loudness</dt>
          <dd>{hasLoudness ? `${track.measured_lufs.toFixed(1)} LUFS` : "Below metering range"}</dd>
        </div>
        <div>
          <dt>True peak</dt>
          <dd>{hasPeak ? `${peak.toFixed(2)} dBTP` : "Not recorded"}</dd>
        </div>
      </dl>
      <span className="album-track-result-note">
        {hasTarget ? `Target ${target.toFixed(1)} LUFS` : target === null ? "No loudness target" : "Target not recorded"}
        {hasCeiling && ` · ceiling ${ceiling.toFixed(1)} dBTP`}
      </span>
      {Math.abs(difference) > 0.5 && (
        <span className="album-track-result-note">
          {Math.abs(difference).toFixed(1)} LU {difference < 0 ? "below" : "above"} target.
        </span>
      )}
      {aboveCeiling && (
        <span className="album-export-receipt-advisory">
          Above ceiling by {(peak - ceiling).toFixed(2)} dB. Review before sharing.
        </span>
      )}
    </li>
  );
}

export function AlbumExportReceipt({ report }: { report: AlbumRenderReport }) {
  const renderedRate = formatSampleRate(report.rendered_sample_rate);
  const exportCancelled = report.status.status === "cancelled";
  const requestedRate =
    report.requested_sample_rate == null
      ? "Auto"
      : formatSampleRate(report.requested_sample_rate);
  const sourceRates = Array.from(new Set(report.source_sample_rates ?? [])).sort(
    (a, b) => a - b,
  );
  const upsampledRates = sourceRates.filter(
    (rate) => rate < report.rendered_sample_rate,
  );
  const sourceChannels = Array.from(new Set(report.source_channels ?? [])).sort(
    (a, b) => a - b,
  );
  const upmixedChannels = sourceChannels.filter(
    (channels) => channels < report.rendered_channels,
  );
  const foldedChannels = sourceChannels.filter(
    (channels) => channels > report.rendered_channels,
  );
  const requestedMismatch =
    report.requested_sample_rate != null &&
    report.requested_sample_rate !== report.rendered_sample_rate;
  const overriddenPositions = (report.tracks ?? [])
    .filter((track) => track.override_album)
    .map((track) => track.position)
    .sort((a, b) => a - b);

  if (exportCancelled) {
    return (
      <div className="album-export-receipt is-cancelled" role="status">
        <span className="album-export-receipt-label">Export cancelled:</span>
        <span className="album-export-receipt-meta">
          No album files were written.
        </span>
      </div>
    );
  }

  return (
    <div className="album-export-receipt">
      <span className="album-export-receipt-label">Last export:</span>
      <code className="album-export-receipt-path">{report.album_wav_path}</code>
      <span className="album-export-receipt-meta">
        {trackCountLabel(report.tracks.length)} · rendered {renderedRate} /{" "}
        {formatBitDepth(report.bit_depth)} /{" "}
        {formatChannelCount(report.rendered_channels)} · requested {requestedRate}
        {requestedMismatch && `, got ${renderedRate}`} · manifest:{" "}
        {report.manifest_path}
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
          {formatChannelCount(report.rendered_channels)}
        </span>
      )}
      {overriddenPositions.length > 0 && (
        <span className="album-export-receipt-advisory">
          Override: track {overriddenPositions.join(", ")} rendered with its own
          settings
        </span>
      )}
      {report.tracks.length > 0 && (
        <details className="album-receipt-tracks">
          <summary>Track results ({report.tracks.length})</summary>
          <ol aria-label="Delivered track measurements">
            {report.tracks.map((track) => (
              <TrackResult key={track.track_id} track={track} />
            ))}
          </ol>
        </details>
      )}
    </div>
  );
}
