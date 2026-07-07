// Album export receipt — the post-export delivery summary + advisories.
//
// Slice 13b moved this out of AlbumPanel (which is now the sidebar identity
// block) so the receipt can live at the sidebar bottom, near the export rail,
// while the album title/chips/flow controls sit at the sidebar top. Logic is
// unchanged from its previous AlbumPanel home.

import { formatBitDepth, formatSampleRate } from "./ExportReceiptCard";
import type { AlbumRenderReport } from "../lib/api";

function formatChannelCount(channels: number): string {
  if (channels === 1) return "mono";
  if (channels === 2) return "stereo";
  return `${channels} ch`;
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
        {report.tracks.length} tracks · rendered {renderedRate} /{" "}
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
    </div>
  );
}
