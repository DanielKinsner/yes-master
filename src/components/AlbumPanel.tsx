// Album identity block (Slice 13b): lives at the TOP of the left sidebar in
// Album Master. It carries the album's identity + shaping controls:
//   * in-place-editable album title (heading, scaled to sidebar width)
//   * track/duration metadata chips
//   * the compact flow cluster (Album flow curve + Flow amount), stacked
//
// Moving this out of the center column is what lets Album Master's waveform/
// meters/timeline sit at the exact same height as Track Master's — the album
// bands no longer push the center column down (Slice 13b done-criterion).
//
// The post-export receipt now renders separately (AlbumExportReceipt) near the
// sidebar bottom. Per-track adaptation (Follows/Override) moved inline into the
// track header badge row. The character system (inferred labels → LUFS/EQ
// biases) stays gated OFF by default — see `album.rs::ALBUM_CHARACTER`.

import type { AlbumArcKind, ImportedTrack } from "../bindings";
import { ALBUM_ARC_DISPLAY } from "../bindings";
import { formatDuration } from "../lib/time-format";

type AlbumPanelProps = {
  tracks: ImportedTrack[];
  albumArcKind: AlbumArcKind;
  albumIntensity: number;
  albumTitle: string;
  onAlbumArc: (kind: AlbumArcKind) => void;
  onAlbumIntensity: (v: number) => void;
  onAlbumTitle: (v: string) => void;
};

export function AlbumPanel({
  tracks,
  albumArcKind,
  albumIntensity,
  albumTitle,
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
  return (
    <section className="album-panel">
      <header className="album-panel-head">
        <div className="album-panel-summary">
          <span className="section-label">Album</span>
          <input
            type="text"
            className="album-title-input"
            value={albumTitle}
            placeholder="Name this album"
            onChange={(e) => onAlbumTitle(e.target.value)}
            maxLength={120}
            aria-label="Album title"
          />
        </div>
        <div className="album-panel-chips">
          <span className="meta-chip">
            {tracks.length === 1 ? "1 track" : `${tracks.length} tracks`}
          </span>
          {totalSeconds > 0 && (
            <span className="meta-chip">{formatDuration(totalSeconds)}</span>
          )}
        </div>
      </header>
      <div className="album-panel-controls">
        <div className="album-control-row">
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
        </div>
        <div className="album-control-row">
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
      </div>
    </section>
  );
}
