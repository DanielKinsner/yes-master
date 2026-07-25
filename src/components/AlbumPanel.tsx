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
import { sequenceArcHeights, type SequenceRow } from "../lib/album-sequence";
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
  /// U10 — sequence rows for the compact arc. Optional so the panel still
  /// renders (without an arc) before a plan exists.
  sequenceRows?: SequenceRow[];
};

/**
 * Compact sequence arc (U10).
 *
 * Draws the loudness shape the album will actually render, because the heights
 * come from the backend plan's own per-track targets — so it responds to the
 * flow choice, the flow amount, and the track ordering for free.
 *
 * Renders nothing when there is no spread to draw (flow amount 0, a single
 * track, or no plan yet). A flat line implying "no arc" and an absent arc
 * meaning "we don't know yet" are different states, and inventing a curve to
 * fill the space would be exactly the kind of decorative dishonesty the plan
 * forbids.
 */
function SequenceArc({ rows }: { rows: SequenceRow[] }) {
  const heights = sequenceArcHeights(rows);
  if (heights.length === 0) return null;

  const width = 100;
  const height = 22;
  const step = heights.length > 1 ? width / (heights.length - 1) : width;
  const points = heights
    .map((h, i) => `${(i * step).toFixed(2)},${(height - h * height).toFixed(2)}`)
    .join(" ");

  return (
    <div className="album-sequence-arc">
      {/* U11 — ORIENTATION. Changing Album Flow or Flow Amount re-shapes this
          arc, and the redraw was instantaneous and silent: the shape you were
          reading became a different shape with nothing connecting the two, so
          a small flow-amount nudge was easy to make and impossible to notice.

          Keying the svg on the computed points remounts it when — and only
          when — the shape actually changes, which runs the settle animation.
          An unchanged plan re-render does nothing, so this cannot become
          ambient motion. The arc still comes from the backend plan's own
          targets, so this acknowledges a real change rather than implying one:
          if the flow made no difference to the numbers, nothing moves. */}
      <svg
        key={points}
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
        role="img"
        aria-label={`Loudness arc across ${heights.length} tracks, following the selected flow.`}
      >
        <polyline points={points} fill="none" strokeWidth={1.4} vectorEffect="non-scaling-stroke" />
        {heights.map((h, i) => (
          <circle
            key={i}
            cx={i * step}
            cy={height - h * height}
            r={1.6}
            vectorEffect="non-scaling-stroke"
          />
        ))}
      </svg>
    </div>
  );
}

export function AlbumPanel({
  tracks,
  albumArcKind,
  albumIntensity,
  albumTitle,
  onAlbumArc,
  onAlbumIntensity,
  onAlbumTitle,
  sequenceRows = [],
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
        <SequenceArc rows={sequenceRows} />
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
