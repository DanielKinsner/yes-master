// UI restyle slice 4b — Visual EQ panel v1.
//
// Renders a log-frequency / linear-dB grid with seven draggable EQ nodes
// (Sub / Low / Low-Mid / Mid / High-Mid / High / Sparkle) pinned to the
// frequencies the Rust chain actually uses (see `ChainCoeffs::from_settings`
// in src-tauri/src/dsp.rs:
// 80 Hz peak Q=0.8, 200 Hz low shelf, 400 Hz peak Q=0.9, 1500 Hz peak
// Q=0.8, 3500 Hz peak Q=0.9, 6000 Hz high shelf, 12000 Hz high shelf).
// Vertical drag changes a band's gain; double-click resets the
// node to 0 dB. The response curve is an APPROXIMATION of the chain's
// filter cascade — Gaussian peaks + sigmoid shelves in log-frequency
// space — chosen to give the user a fast visual feedback loop, not
// numerically-exact dB-vs-frequency response (the actual Rust chain
// does the audible work; the curve is a "shape preview").
//
// 2026-08-18 — bands are movable in FREQUENCY as well as gain: the engine
// now reads per-band Hz from `settings.eq_bands` (`EqBandFrequencies` in
// Rust, defaults = the old fixed constants). Drag = gain (vertical) AND
// frequency (horizontal, clamped to the band's `EQ_BAND_RANGES` window so
// neighbours cannot cross); double-click puts BOTH back to default. Q and
// shelf slope stay fixed — the UI must not promise what the engine can't
// honor.
//
// Still intentionally OMITTED:
//   * Warmth + Presence/Air nodes (different units — 0..1 saturation
//     drive vs dB EQ — would need separate scaling and don't fit the
//     same plot cleanly).
// Live FFT spectrum can render as an underlay when `spectrumDb` is supplied.

import { useCallback, useRef, useState } from "react";
import type { EqBandFrequencies, MasteringSettings } from "../bindings";
import { EQ_BAND_DEFAULTS, EQ_BAND_RANGES } from "../bindings";

type BandId = "sub" | "low" | "low-mid" | "mid" | "high-mid" | "high" | "sparkle";

/// Band id → the `eq_bands` key that holds its frequency.
const BAND_HZ_KEY: Record<BandId, keyof EqBandFrequencies> = {
  sub: "sub_hz",
  low: "low_hz",
  "low-mid": "low_mid_hz",
  mid: "mid_hz",
  "high-mid": "high_mid_hz",
  high: "high_hz",
  sparkle: "sparkle_hz",
};

/// Resolve every band's current frequency: settings first, defaults for any
/// missing key (older saved state predates `eq_bands`).
export function resolveBandHz(settings: MasteringSettings): Record<BandId, number> {
  const b = settings.eq_bands;
  const out = {} as Record<BandId, number>;
  for (const id of Object.keys(BAND_HZ_KEY) as BandId[]) {
    const key = BAND_HZ_KEY[id];
    const v = b?.[key];
    out[id] = typeof v === "number" && Number.isFinite(v) ? v : EQ_BAND_DEFAULTS[key];
  }
  return out;
}

/// Human frequency label: 80, 400, 1.5k, 12k.
export function formatHz(hz: number): string {
  if (hz >= 1000) {
    const k = hz / 1000;
    return `${Number.isInteger(k) ? k : k.toFixed(1)}k`;
  }
  return `${Math.round(hz)}`;
}
type BandKind = "shelf-low" | "peak" | "shelf-high";
type BandTier = "primary" | "secondary";

interface Band {
  id: BandId;
  label: string;
  /// Default centre/corner frequency; the live value comes from settings.
  hz: number;
  color: string;
  kind: BandKind;
  tier: BandTier;
  /// Q-equivalent in octaves; only used for peak bands' Gaussian width.
  qOctaves: number;
}

// Match the Rust chain's actual filter frequencies — `dsp.rs` ChainCoeffs:
//   BiquadCoeffs::peaking(sr, 80.0, 0.8, ...)
//   BiquadCoeffs::low_shelf(sr, 200.0, ...)
//   BiquadCoeffs::peaking(sr, 400.0, 0.9, ...)
//   BiquadCoeffs::peaking(sr, 1500.0, 0.8, ...)
//   BiquadCoeffs::peaking(sr, 3500.0, 0.9, ...)
//   BiquadCoeffs::high_shelf(sr, 6000.0, ...)
//   BiquadCoeffs::high_shelf(sr, 12000.0, ...)
// Muted, cohesive palette — a subtle low→high hue journey (slate → teal →
// periwinkle → cool blue → ice) at low saturation so the nodes read as a
// precise instrument rather than candy. Band identity stays legible without
// the rainbow.
const BANDS: readonly Band[] = [
  { id: "sub", label: "SUB", hz: 80, color: "#6f86ad", kind: "peak", tier: "secondary", qOctaves: 1.2 },
  { id: "low", label: "LOW", hz: 200, color: "#6d93b3", kind: "shelf-low", tier: "primary", qOctaves: 0 },
  { id: "low-mid", label: "LOW-MID", hz: 400, color: "#6f9d9c", kind: "peak", tier: "secondary", qOctaves: 1.0 },
  { id: "mid", label: "MID", hz: 1500, color: "#8f8fb5", kind: "peak", tier: "primary", qOctaves: 1.2 },
  { id: "high-mid", label: "HIGH-MID", hz: 3500, color: "#a89bb0", kind: "peak", tier: "secondary", qOctaves: 1.0 },
  { id: "high", label: "HIGH", hz: 6000, color: "#8aa6c6", kind: "shelf-high", tier: "primary", qOctaves: 0 },
  { id: "sparkle", label: "SPARKLE", hz: 12_000, color: "#a3b6d2", kind: "shelf-high", tier: "secondary", qOctaves: 0 },
];

const F_MIN = 20;
const F_MAX = 20_000;
const DB_MIN = -12;
const DB_MAX = 12;
const GRID_FREQS = [20, 50, 100, 200, 500, 1_000, 2_000, 5_000, 10_000, 20_000];
const GRID_DBS = [-12, -6, 0, 6, 12];

const LOG_F_MIN = Math.log10(F_MIN);
const LOG_F_MAX = Math.log10(F_MAX);
const LOG_F_SPAN = LOG_F_MAX - LOG_F_MIN;

// L4b — separate y mapping for the live spectrum so very-quiet signal
// (e.g. -50 dBFS) still parks the bar near the bottom rather than way
// below the EQ panel's -12 dB floor. Range tuned to match the Rust
// SPECTRUM_FLOOR_DB / SPECTRUM_CEIL_DB published from the audio thread.
const SPECTRUM_FLOOR_DB = -60;
const SPECTRUM_CEIL_DB = 6;
const SPECTRUM_DB_SPAN = SPECTRUM_CEIL_DB - SPECTRUM_FLOOR_DB;

function freqToX(hz: number, width: number): number {
  return ((Math.log10(hz) - LOG_F_MIN) / LOG_F_SPAN) * width;
}

function dbToY(db: number, height: number): number {
  // 0 dB at the vertical center; +DB_MAX at top, -DB_MIN at bottom.
  return ((DB_MAX - db) / (DB_MAX - DB_MIN)) * height;
}

function yToDb(y: number, height: number): number {
  const raw = DB_MAX - (y / height) * (DB_MAX - DB_MIN);
  return Math.max(DB_MIN, Math.min(DB_MAX, raw));
}

/// Approximate the chain's per-band magnitude response at `hz` for a band
/// configured with `gainDb`. Peaks use a Gaussian whose width tracks the
/// declared Q-octaves; shelves use a logistic sigmoid centered at the
/// shelf frequency. Sum across all bands at each plot point to draw the
/// composite curve.
function bandResponseDb(hz: number, band: Band, gainDb: number, bandHz: number): number {
  if (gainDb === 0) return 0;
  const distOctaves = Math.log2(hz / bandHz);
  switch (band.kind) {
    case "peak": {
      // Gaussian whose FWHM ≈ 1 octave at Q=1 (qOctaves ≈ 1).
      const sigma = band.qOctaves * 0.5 / 2.355;
      const safeSigma = sigma > 0 ? sigma : 0.5;
      return gainDb * Math.exp(-0.5 * (distOctaves / safeSigma) ** 2);
    }
    case "shelf-low":
      // Below fc → full gain; above → smoothly returns to 0.
      return gainDb * (1 - 1 / (1 + Math.exp(-1.8 * distOctaves)));
    case "shelf-high":
      // Above fc → full gain; below → smoothly returns to 0.
      return gainDb * (1 / (1 + Math.exp(-1.8 * distOctaves)));
  }
}

function totalResponseDb(
  hz: number,
  gains: Record<BandId, number>,
  bandHz: Record<BandId, number>,
): number {
  let total = 0;
  for (const band of BANDS) {
    total += bandResponseDb(hz, band, gains[band.id], bandHz[band.id]);
  }
  return total;
}

interface VisualEqPanelProps {
  settings: MasteringSettings;
  onEq: (band: BandId, db: number) => void;
  /** 2026-08-18 — when provided, a node drag reports gain AND frequency in
   * ONE call (the host stores both in one mutation; two separate setters
   * would race — see useTrackMaster.setEqBandPoint). Optional so read-only
   * hosts and older callers keep working: without it the nodes drag
   * vertically only through `onEq`. */
  onEqPoint?: (band: BandId, db: number, hz: number) => void;
  /** `true` renders the dense embedded variant used inside the
   * mastering-deck Tone Shape cell — no card chrome, no
   * outer header, no per-node value labels, smaller viewBox
   * so it sits beside the L/M/H knobs without dominating. */
  compact?: boolean;
  /** L4b — live FFT spectrum bins from the audio thread. Length
   * should be `SPECTRUM_N_BINS` (32) on the Rust side; we accept
   * any non-empty array and map proportionally. Empty array =
   * draw no spectrum (idle / Original playback). */
  spectrumDb?: number[];
}

export function VisualEqPanel({
  settings,
  onEq,
  onEqPoint,
  compact = false,
  spectrumDb,
}: VisualEqPanelProps) {
  const svgRef = useRef<SVGSVGElement | null>(null);
  // Drag state: which band is being dragged.
  const [dragging, setDragging] = useState<BandId | null>(null);
  const bandHz = resolveBandHz(settings);

  const gains: Record<BandId, number> = {
    "sub": settings.eq_sub_db,
    "low": settings.eq_low_db,
    "low-mid": settings.eq_low_mid_db,
    "mid": settings.eq_mid_db,
    "high-mid": settings.eq_high_mid_db,
    "high": settings.eq_high_db,
    "sparkle": settings.eq_sparkle_db,
  };

  // SVG drawing constants. Viewport is `width × height` (logical units);
  // CSS sizes the visible panel. Padding leaves room for axis labels.
  const PAD_LEFT = compact ? 28 : 40;
  const PAD_RIGHT = compact ? 8 : 12;
  const PAD_TOP = compact ? 8 : 14;
  // Bottom padding holds two text rows: the frequency axis (50/100/1k/…)
  // on the first line, then the colored band labels (LOW / LOW-MID /
  // MID / HIGH) on the second so they don't overlap the axis ticks.
  // Compact mode drops the band-label row entirely — node colors carry
  // the identity.
  const PAD_BOTTOM = compact ? 18 : 34;
  const AXIS_LABEL_Y_OFFSET = compact ? 12 : 14;
  const BAND_LABEL_Y_OFFSET = 28;
  const VBW = compact ? 420 : 720;
  const VBH = compact ? 180 : 272;
  const plotW = VBW - PAD_LEFT - PAD_RIGHT;
  const plotH = VBH - PAD_TOP - PAD_BOTTOM;

  const localFreqToX = (hz: number) => PAD_LEFT + freqToX(hz, plotW);
  const localDbToY = (db: number) => PAD_TOP + dbToY(db, plotH);
  const yToDbInPlot = (y: number) => yToDb(y - PAD_TOP, plotH);
  const xToHzInPlot = (x: number) => {
    const t = Math.max(0, Math.min(1, (x - PAD_LEFT) / plotW));
    return Math.pow(10, LOG_F_MIN + t * LOG_F_SPAN);
  };

  // Pre-compute the composite response curve as an SVG path. 180 sample
  // points across the log-frequency range gives smooth visuals without
  // being expensive (this re-runs every settings change).
  const N_SAMPLES = 180;
  const curvePoints: { x: number; y: number }[] = [];
  for (let i = 0; i <= N_SAMPLES; i++) {
    const t = i / N_SAMPLES;
    const logHz = LOG_F_MIN + t * LOG_F_SPAN;
    const hz = Math.pow(10, logHz);
    const db = totalResponseDb(hz, gains, bandHz);
    curvePoints.push({ x: PAD_LEFT + t * plotW, y: localDbToY(db) });
  }
  const curvePath = curvePoints
    .map((p, i) => (i === 0 ? `M ${p.x.toFixed(2)} ${p.y.toFixed(2)}` : `L ${p.x.toFixed(2)} ${p.y.toFixed(2)}`))
    .join(" ");
  // Filled area under the curve, clipped at the 0-dB line so the fill
  // reads as "lift above zero" / "cut below zero" rather than a giant
  // blob across the panel.
  const zeroY = localDbToY(0);
  const fillPath =
    `M ${PAD_LEFT} ${zeroY.toFixed(2)} ` +
    curvePoints.map((p) => `L ${p.x.toFixed(2)} ${p.y.toFixed(2)}`).join(" ") +
    ` L ${(PAD_LEFT + plotW).toFixed(2)} ${zeroY.toFixed(2)} Z`;
  // Boost vs cut are drawn in two colours (owner mockup 2026-08-18): the
  // response line + fill are rendered twice, once clipped to the half of the
  // plot ABOVE the 0 dB line (cool blue = lift) and once clipped to the half
  // BELOW it (amber = cut). Both clips + the two fade gradients live in
  // <defs>; ids are suffixed by variant so a compact and a full panel can
  // coexist without colliding.
  const idSuffix = compact ? "c" : "f";
  const clipAboveId = `eq-clip-above-${idSuffix}`;
  const clipBelowId = `eq-clip-below-${idSuffix}`;
  const gradBoostId = `eq-grad-boost-${idSuffix}`;
  const gradCutId = `eq-grad-cut-${idSuffix}`;
  const plotBottom = PAD_TOP + plotH;
  /// Signed colour role for a band's node: lifted, cut, or resting at 0 dB.
  const nodeRole = (db: number): "boost" | "cut" | "flat" =>
    db > 0.05 ? "boost" : db < -0.05 ? "cut" : "flat";

  // Pointer → viewBox coordinates via the SVG's CTM, independent of CSS
  // scaling or window size.
  const toLocal = useCallback((event: { clientX: number; clientY: number }) => {
    const svg = svgRef.current;
    if (!svg) return null;
    const pt = svg.createSVGPoint();
    pt.x = event.clientX;
    pt.y = event.clientY;
    const ctm = svg.getScreenCTM();
    if (!ctm) return null;
    return pt.matrixTransform(ctm.inverse());
  }, []);

  // Which band a pointer position "means": the NEAREST node within the hit
  // radius, decided from geometry rather than element stacking. With movable
  // bands two nodes can share a spot (LOW parked at its 400 Hz ceiling on
  // top of LOW-MID's 400 Hz default); per-node hit circles would let the
  // top-most steal every press and the one underneath could never be grabbed
  // or double-clicked back. Ties go to the band already being dragged.
  const HIT_RADIUS = 18;
  const bandAt = useCallback(
    (local: { x: number; y: number }): BandId | null => {
      let best: BandId | null = null;
      let bestD = Infinity;
      for (const band of BANDS) {
        const dx = localFreqToX(bandHz[band.id]) - local.x;
        const dy = localDbToY(gains[band.id]) - local.y;
        const d = Math.hypot(dx, dy);
        // Strictly nearer wins; an exact tie goes to the band already being
        // dragged, else to the later (top-drawn) band — what the eye sees.
        if (d <= HIT_RADIUS && (d < bestD || (d === bestD && (band.id === dragging || dragging === null)))) {
          best = band.id;
          bestD = d;
        }
      }
      return best;
    },
    [bandHz, gains, dragging, localFreqToX, localDbToY],
  );

  const handlePointerDown = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      const local = toLocal(event);
      if (!local) return;
      const band = bandAt(local);
      if (!band) return;
      event.preventDefault();
      event.currentTarget.setPointerCapture(event.pointerId);
      setDragging(band);
    },
    [toLocal, bandAt],
  );

  const handlePointerMove = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      if (!dragging) return;
      const local = toLocal(event);
      if (!local) return;
      const band = dragging;
      const newDb = Math.round(yToDbInPlot(local.y) * 10) / 10;
      if (onEqPoint) {
        const [lo, hi] = EQ_BAND_RANGES[BAND_HZ_KEY[band]];
        const rawHz = xToHzInPlot(local.x);
        onEqPoint(band, newDb, Math.round(Math.max(lo, Math.min(hi, rawHz))));
      } else {
        onEq(band, newDb);
      }
    },
    [dragging, toLocal, onEq, onEqPoint, yToDbInPlot, xToHzInPlot],
  );

  const handlePointerUp = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      if (!dragging) return;
      const target = event.currentTarget;
      if (target.hasPointerCapture(event.pointerId)) {
        target.releasePointerCapture(event.pointerId);
      }
      setDragging(null);
    },
    [dragging],
  );

  // Double-click: gain back to 0 dB AND the band back on its default
  // frequency (one gesture = "this band as shipped").
  const handleDoubleClick = useCallback(
    (event: React.MouseEvent<SVGSVGElement>) => {
      const local = toLocal(event);
      if (!local) return;
      const band = bandAt(local);
      if (!band) return;
      if (onEqPoint) onEqPoint(band, 0, EQ_BAND_DEFAULTS[BAND_HZ_KEY[band]]);
      else onEq(band, 0);
    },
    [toLocal, bandAt, onEq, onEqPoint],
  );

  return (
    <section
      className={`visual-eq-panel ${compact ? "visual-eq-panel-compact" : ""}`}
      aria-label="Visual EQ"
    >
      {!compact && (
        <header className="visual-eq-head">
          <span className="section-label">TONE CURVE</span>
          <span className="visual-eq-hint">Drag a node up or down · double-click to reset</span>
        </header>
      )}
      <svg
        ref={svgRef}
        className="eq-overlay"
        viewBox={`0 0 ${VBW} ${VBH}`}
        preserveAspectRatio="none"
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        onDoubleClick={handleDoubleClick}
        style={{ touchAction: "none" }}
      >
        <defs>
          <clipPath id={clipAboveId}>
            <rect x={PAD_LEFT} y={PAD_TOP} width={plotW} height={Math.max(0, zeroY - PAD_TOP)} />
          </clipPath>
          <clipPath id={clipBelowId}>
            <rect x={PAD_LEFT} y={zeroY} width={plotW} height={Math.max(0, plotBottom - zeroY)} />
          </clipPath>
          {/* userSpaceOnUse so the fade is anchored to the 0 dB line, not to
              the bounding box of whatever the curve happens to be. */}
          <linearGradient id={gradBoostId} gradientUnits="userSpaceOnUse" x1="0" y1={PAD_TOP} x2="0" y2={zeroY}>
            <stop offset="0%" className="eq-grad-boost-hi" />
            <stop offset="100%" className="eq-grad-boost-lo" />
          </linearGradient>
          <linearGradient id={gradCutId} gradientUnits="userSpaceOnUse" x1="0" y1={zeroY} x2="0" y2={plotBottom}>
            <stop offset="0%" className="eq-grad-cut-lo" />
            <stop offset="100%" className="eq-grad-cut-hi" />
          </linearGradient>
        </defs>
        {/* Grid: major frequency lines + minor sub-octave ticks. */}
        {GRID_FREQS.map((hz) => (
          <line
            key={`gx-${hz}`}
            className="eq-grid-major"
            x1={localFreqToX(hz)}
            x2={localFreqToX(hz)}
            y1={PAD_TOP}
            y2={PAD_TOP + plotH}
          />
        ))}
        {/* Horizontal dB grid. */}
        {GRID_DBS.map((db) => (
          <line
            key={`gy-${db}`}
            className={db === 0 ? "eq-zero-line" : "eq-grid-major"}
            x1={PAD_LEFT}
            x2={PAD_LEFT + plotW}
            y1={localDbToY(db)}
            y2={localDbToY(db)}
          />
        ))}
        {/* Frequency axis labels along the bottom (first row). */}
        {GRID_FREQS.map((hz) => (
          <text
            key={`fx-${hz}`}
            className="eq-label"
            x={localFreqToX(hz)}
            y={PAD_TOP + plotH + AXIS_LABEL_Y_OFFSET}
            textAnchor="middle"
          >
            {hz >= 1000 ? `${hz / 1000}k` : `${hz}`}
          </text>
        ))}
        {/* dB axis labels along the left edge. */}
        {GRID_DBS.map((db) => (
          <text
            key={`fy-${db}`}
            className="eq-label"
            x={PAD_LEFT - 8}
            y={localDbToY(db) + 4}
            textAnchor="end"
          >
            {db > 0 ? `+${db}` : `${db}`}
          </text>
        ))}
        {/* L4b — live FFT spectrum, drawn BEFORE the response curve so
            the curve renders on top. Each bin contributes a stepped
            segment from its left x to its right x at the bin's dB y.
            Spectrum uses its own y range (-60..+6 dBFS) so quiet signal
            parks near the bottom of the plot, independent of the EQ
            gain axis on top. */}
        {spectrumDb && spectrumDb.length > 1 && (() => {
          const n = spectrumDb.length;
          const bottom = PAD_TOP + plotH;
          const spectrumDbToY = (db: number) =>
            PAD_TOP +
            plotH *
              (1 - (Math.max(SPECTRUM_FLOOR_DB, Math.min(SPECTRUM_CEIL_DB, db)) - SPECTRUM_FLOOR_DB) /
                SPECTRUM_DB_SPAN);
          // Bin frequency edges match the Rust side: log-spaced between
          // SPECTRUM_F_MIN_HZ (20) and SPECTRUM_F_MAX_HZ (20k).
          const binX = (i: number) => {
            const t = i / n;
            const logHz = LOG_F_MIN + t * LOG_F_SPAN;
            const hz = Math.pow(10, logHz);
            return PAD_LEFT + freqToX(hz, plotW);
          };
          // Build a stepped area path: start at bottom-left of bin 0,
          // step up to bin 0's dB y, then walk along each bin's top
          // edge, then close to bottom-right.
          let d = `M ${binX(0).toFixed(2)} ${bottom.toFixed(2)} `;
          for (let i = 0; i < n; i++) {
            const xL = binX(i);
            const xR = binX(i + 1);
            const y = spectrumDbToY(spectrumDb[i]);
            d += `L ${xL.toFixed(2)} ${y.toFixed(2)} `;
            d += `L ${xR.toFixed(2)} ${y.toFixed(2)} `;
          }
          d += `L ${binX(n).toFixed(2)} ${bottom.toFixed(2)} Z`;
          return <path className="eq-spectrum-fill" d={d} />;
        })()}
        {/* Response curve: fill under, then line on top — each drawn twice,
            clipped to the boost half (blue) and the cut half (amber). */}
        <g clipPath={`url(#${clipAboveId})`}>
          <path className="eq-response-fill eq-response-fill-boost" d={fillPath} style={{ fill: `url(#${gradBoostId})` }} />
          <path className="eq-response-line eq-response-line-boost" d={curvePath} />
        </g>
        <g clipPath={`url(#${clipBelowId})`}>
          <path className="eq-response-fill eq-response-fill-cut" d={fillPath} style={{ fill: `url(#${gradCutId})` }} />
          <path className="eq-response-line eq-response-line-cut" d={curvePath} />
        </g>
        {/* Per-band nodes. Each renders a colored dot + label; the
            invisible hit-target above (eq-node-hit) is twice the size
            so dragging is forgiving. */}
        {BANDS.map((band) => {
          const hzNow = bandHz[band.id];
          const x = localFreqToX(hzNow);
          const y = localDbToY(gains[band.id]);
          const isDragging = dragging === band.id;
          const isPrimary = band.tier === "primary";
          const role = nodeRole(gains[band.id]);
          // Hollow ring nodes: the ring colour says boost (blue) / cut
          // (amber) / flat (dim), the dark centre keeps the curve legible
          // through the node. One size for every band (owner 2026-08-19):
          // tier still shows in the label weight, not the dot.
          const nodeRadius = 4.6;
          const renderedRadius = isDragging ? nodeRadius + 1 : nodeRadius;
          const labelOpacity = isPrimary ? 0.92 : 0.7;
          return (
            <g key={band.id} className={`eq-band eq-band-${role}`}>
              {isDragging && (
                <circle
                  className="eq-node-halo"
                  cx={x}
                  cy={y}
                  r={nodeRadius + 5}
                />
              )}
              <circle
                className={`eq-node eq-node-${band.tier} ${isDragging ? "is-dragging" : ""}`}
                cx={x}
                cy={y}
                r={renderedRadius}
              />
              {/* Hit area is only a cursor hint now — the SVG-level handlers
                  below decide the band by nearest node (see bandAt). */}
              <circle
                className="eq-node-hit"
                cx={x}
                cy={y}
                r={HIT_RADIUS}
                style={{ cursor: onEqPoint ? "move" : "ns-resize", touchAction: "none" }}
              />
              {isDragging && (
                <text
                  className="eq-node-readout"
                  x={x}
                  y={Math.max(PAD_TOP + 10, y - 11)}
                  textAnchor="middle"
                >
                  {`${gains[band.id] > 0 ? "+" : ""}${gains[band.id].toFixed(1)} dB · ${formatHz(hzNow)}`}
                </text>
              )}
              {!compact && (
                <text
                  className="eq-node-label"
                  x={x}
                  y={PAD_TOP + plotH + BAND_LABEL_Y_OFFSET}
                  textAnchor="middle"
                  fill={band.color}
                  opacity={labelOpacity}
                >
                  {band.label}
                </text>
              )}
              {!compact && (
                <text
                  className="eq-node-value"
                  x={x}
                  y={Math.max(PAD_TOP + 12, y - 12)}
                  textAnchor="middle"
                  fill={band.color}
                  opacity={labelOpacity}
                >
                  {gains[band.id] > 0 ? `+${gains[band.id].toFixed(1)}` : gains[band.id].toFixed(1)}
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </section>
  );
}
