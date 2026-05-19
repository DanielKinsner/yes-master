// Phase B Step 4: Album Master panel.
//
// Top-strip control surface for album-mode mastering. Shows:
//   * Arc dropdown (4 named curves)
//   * Album intensity slider
//   * Album title input
//   * Export Album CTA (calls plan_album → render_album_plan via the hook)
//   * Last export report when present
//
// Per-track DSP is still edited via the regular Tone Shape / Macros / Advanced
// controls on whichever track the user has selected from the sidebar. The
// album layer only modulates the per-track LUFS target via arc + character.

import type { AlbumArcKind, ImportedTrack } from "../bindings";
import { ALBUM_ARC_DISPLAY } from "../bindings";
import type { AlbumRenderReport } from "../lib/api";

type AlbumPanelProps = {
  tracks: ImportedTrack[];
  albumArcKind: AlbumArcKind;
  albumIntensity: number;
  albumTitle: string;
  albumRendering: boolean;
  albumExportReport: AlbumRenderReport | null;
  onAlbumArc: (kind: AlbumArcKind) => void;
  onAlbumIntensity: (v: number) => void;
  onAlbumTitle: (v: string) => void;
  onExportAlbum: () => void;
};

function formatAlbumDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const minutes = Math.floor(total / 60);
  const remaining = total % 60;
  return `${minutes}:${remaining.toString().padStart(2, "0")}`;
}

export function AlbumPanel({
  tracks,
  albumArcKind,
  albumIntensity,
  albumTitle,
  albumRendering,
  albumExportReport,
  onAlbumArc,
  onAlbumIntensity,
  onAlbumTitle,
  onExportAlbum,
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
                <strong>{formatAlbumDuration(totalSeconds)}</strong>
              </>
            )}
          </span>
        </div>
        <span className="section-label album-panel-mode">Album Master</span>
        <input
          type="text"
          className="album-title-input"
          value={albumTitle}
          placeholder="Album title…"
          onChange={(e) => onAlbumTitle(e.target.value)}
          maxLength={120}
        />
        <button
          type="button"
          className="primary album-export-btn"
          onClick={onExportAlbum}
          disabled={albumRendering || tracks.length === 0}
        >
          {albumRendering ? "Rendering album…" : "Export Album"}
        </button>
      </header>
      <div className="album-panel-controls">
        <label className="adv-label" htmlFor="album-arc-select">
          Arc
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
          Intensity
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
            {albumExportReport.tracks.length} tracks · manifest:{" "}
            {albumExportReport.manifest_path}
          </span>
        </div>
      )}
    </section>
  );
}
