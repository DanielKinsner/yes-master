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
import {
  ALBUM_FLOW_DESCRIPTION,
  SINGLE_TRACK_ALBUM_NOTE,
  flowAmountDescription,
  flowAmountValueText,
  trackCountLabel,
} from "../lib/album-copy";

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
            // U10: "Name this album" clipped to "Name this alb..." in the
            // sidebar. A placeholder that does not fit its field is worse than
            // a shorter one that does.
            placeholder="Album title"
            onChange={(e) => onAlbumTitle(e.target.value)}
            maxLength={120}
            aria-label="Album title"
          />
        </div>
        <div className="album-panel-chips">
          <span className="meta-chip">{trackCountLabel(tracks.length)}</span>
          {totalSeconds > 0 && (
            <span className="meta-chip">{formatDuration(totalSeconds)}</span>
          )}
        </div>
      </header>
      {/* U10 — a one-track album is not broken, but the sequence half of the
          feature has nothing to act on. Saying so is better than showing a flow
          control that cannot change anything. */}
      {tracks.length === 1 && (
        <p className="album-single-track-note">{SINGLE_TRACK_ALBUM_NOTE}</p>
      )}
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
        {/* U10 — the flow name alone ("Cinematic") told the user nothing about
            what it does to their record, and clipped to "Cinem..." in the rail
            besides. The description is now on screen. */}
        <p className="album-flow-description" id="album-flow-description">
          {ALBUM_FLOW_DESCRIPTION[albumArcKind]}
        </p>
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
            // U10 — `×1.00` was a number with no stated meaning anywhere in the
            // product. A control nobody can interpret is not a feature.
            aria-valuetext={flowAmountValueText(albumIntensity)}
            aria-describedby="album-flow-amount-description"
          />
          <span className="album-intensity-value">
            ×{albumIntensity.toFixed(2)}
          </span>
        </div>
        <p
          className="album-flow-description"
          id="album-flow-amount-description"
        >
          {flowAmountDescription(albumIntensity)}
        </p>
      </div>
    </section>
  );
}
