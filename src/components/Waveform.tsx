import {
  useEffect,
  useId,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { waveformLoadingView } from "../lib/waveform-progress";
import { MORPH_MS } from "../lib/analysis-orb";
import { prefersReducedMotion } from "../lib/motion";
import { AnalysisOrb } from "./AnalysisOrb";
import { WaveformDbScale } from "./WaveformDbScale";
import type { LoopRegion, WaveformPeaks } from "../bindings";

// Progress surface shown in the waveform deck while a track is prepared.
// `analyzing` drives a determinate bar from the staged analysis progress
// (with the current stage label + percent); `loading` shows an indeterminate
// sweeping bar for the waveform decode (no real fraction available); `idle`
// is the plain empty state.
export function WaveformLoading({
  isAnalyzing,
  isLoadingWaveform,
  analysisProgress,
}: {
  isAnalyzing: boolean;
  isLoadingWaveform: boolean;
  analysisProgress: { label: string; progress: number } | null;
}) {
  const view = waveformLoadingView({
    isAnalyzing,
    analysisProgress,
    isLoadingWaveform,
  });
  const labelId = useId();
  const showBar = view.mode !== "idle";
  const determinate = view.mode === "analyzing" && view.percent !== null;
  // The orb is the engaging face of the analysis wait; the label + bar stay
  // the honest, screen-reader-visible source of truth underneath it. It
  // persists through the waveform-decode gap ("loading") so the sequence
  // reads orb → orb → morph instead of flashing a bare "Loading waveform…"
  // between analysis and the morph (owner note 2026-06-11).
  const showOrb =
    (view.mode === "analyzing" || view.mode === "loading") &&
    !prefersReducedMotion();
  return (
    <div
      className={
        `wf-loading wf-loading-${view.mode}` + (showOrb ? " wf-loading-has-orb" : "")
      }
    >
      {showOrb && <AnalysisOrb phase="orb" />}
      <div className="wf-loading-row">
        {/* Only the stage LABEL is a polite live region, so a screen reader
            announces each stage once ("Reading tonal balance") rather than
            re-reading the percent + the bar's value on every ~1.4 s tick. The
            progressbar below (outside any live region) carries the numeric
            value for AT to poll; the visible percent is decorative. */}
        <span
          id={labelId}
          className="wf-loading-text"
          role="status"
          aria-live="polite"
        >
          {view.label}
        </span>
        {view.percent !== null && (
          <span className="wf-loading-pct" aria-hidden="true">
            {view.percent}%
          </span>
        )}
      </div>
      {showBar && (
        <div
          className={`wf-loading-bar${determinate ? "" : " is-indeterminate"}`}
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={determinate ? (view.percent ?? undefined) : undefined}
          aria-labelledby={labelId}
        >
          {determinate ? (
            <span
              className="wf-loading-bar-fill"
              style={{ width: `${view.percent}%` }}
            />
          ) : (
            <span className="wf-loading-bar-sweep" />
          )}
        </div>
      )}
    </div>
  );
}

export function WaveformView({
  peaks,
  isLoading,
  isAnalyzing,
  analysisProgress,
  currentTimeSec,
  durationSec,
  region,
  regionsEnabled = true,
  onSeek,
  onSetRegion,
  onClearRegion,
}: {
  peaks: WaveformPeaks | undefined;
  isLoading: boolean;
  isAnalyzing: boolean;
  analysisProgress: { label: string; progress: number } | null;
  currentTimeSec: number;
  durationSec: number;
  region: LoopRegion | null;
  /// Gates the loop-region GESTURE, not just the hint text: looping is
  /// Advanced-only (owner smoke F3 — the lasso used to work in Standard,
  /// which has no loop UI at all). False turns shift+drag into a plain seek.
  regionsEnabled?: boolean;
  onSeek: (positionSec: number) => void;
  onSetRegion: (region: LoopRegion) => void;
  onClearRegion: () => void;
}) {
  const [dragRegion, setDragRegion] = useState<LoopRegion | null>(null);

  // Morph: when this track's analysis JUST finished and peaks arrived, the
  // orb's particles fly into the real waveform shape for one short window.
  // Presentation only — the parent timer + any interaction cut it; playback
  // and seeking never wait on it. (Hooks live above the !peaks return.)
  const wasAnalyzingNoPeaks = useRef(false);
  const [morphing, setMorphing] = useState(false);
  useEffect(() => {
    if (!peaks) {
      // Real import flow is sequential: analyze (isAnalyzing), THEN waveform
      // decode (isLoading), THEN peaks. Latch during analysis, hold through
      // the decode gap, and clear only when the slot goes fully idle (so a
      // later track switch with ready peaks doesn't morph).
      if (isAnalyzing) wasAnalyzingNoPeaks.current = true;
      else if (!isLoading) wasAnalyzingNoPeaks.current = false;
      return;
    }
    if (wasAnalyzingNoPeaks.current) {
      wasAnalyzingNoPeaks.current = false;
      if (!prefersReducedMotion()) setMorphing(true);
    }
  }, [peaks, isAnalyzing, isLoading]);
  useEffect(() => {
    if (!morphing) return;
    const cut = () => setMorphing(false);
    const t = setTimeout(cut, MORPH_MS);
    window.addEventListener("pointerdown", cut);
    window.addEventListener("keydown", cut);
    return () => {
      clearTimeout(t);
      window.removeEventListener("pointerdown", cut);
      window.removeEventListener("keydown", cut);
    };
  }, [morphing]);

  // Show the waveform whenever the SELECTED track has peaks; otherwise show the
  // prepare-progress surface (staged analysis bar → indeterminate decode bar →
  // idle "no waveform yet"). Gating on `!peaks` alone — not the GLOBAL
  // isAnalyzing/isLoading flags — keeps an already-decoded track's waveform,
  // transport, playhead, and loop region on screen while a DIFFERENT imported
  // track analyzes or decodes in the background. isAnalyzing/isLoading still
  // flow into WaveformLoading to pick the right progress mode for the genuine
  // first-load case (when this track has no peaks yet).
  if (!peaks) {
    return (
      <section className="wf-card">
        <WaveformLoading
          isAnalyzing={isAnalyzing}
          isLoadingWaveform={isLoading}
          analysisProgress={analysisProgress}
        />
      </section>
    );
  }
  const channel = peaks.channels[0] ?? [];
  const W = 1000;
  const H = 240;
  const playheadX =
    durationSec > 0
      ? Math.max(0, Math.min(W, (currentTimeSec / durationSec) * W))
      : 0;

  const timeAtPointer = (e: ReactPointerEvent<SVGSVGElement>): number => {
    const rect = e.currentTarget.getBoundingClientRect();
    if (rect.width <= 0 || durationSec <= 0) return 0;
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    return ratio * durationSec;
  };

  const handlePointerDown = (e: ReactPointerEvent<SVGSVGElement>) => {
    if (durationSec <= 0) return;
    const t = timeAtPointer(e);
    if (e.shiftKey && regionsEnabled) {
      e.preventDefault();
      try {
        e.currentTarget.setPointerCapture(e.pointerId);
      } catch {
        /* setPointerCapture can throw on some platforms; we still track via state */
      }
      setDragRegion({ start_sec: t, end_sec: t });
    } else {
      onSeek(t);
    }
  };

  const handlePointerMove = (e: ReactPointerEvent<SVGSVGElement>) => {
    if (!dragRegion) return;
    const t = timeAtPointer(e);
    setDragRegion({ start_sec: dragRegion.start_sec, end_sec: t });
  };

  const handlePointerUp = (_e: ReactPointerEvent<SVGSVGElement>) => {
    if (!dragRegion) return;
    const start = Math.min(dragRegion.start_sec, dragRegion.end_sec);
    const end = Math.max(dragRegion.start_sec, dragRegion.end_sec);
    const meaningfulDrag =
      durationSec > 0 && end - start > Math.max(0.1, durationSec * 0.005);
    if (meaningfulDrag) {
      onSetRegion({ start_sec: start, end_sec: end });
    } else if (region) {
      onClearRegion();
    }
    setDragRegion(null);
  };

  const displayRegion: LoopRegion | null = dragRegion ?? region;
  const regionRect = displayRegion && durationSec > 0
    ? (() => {
        const startX = Math.max(
          0,
          Math.min(W, (Math.min(displayRegion.start_sec, displayRegion.end_sec) / durationSec) * W),
        );
        const endX = Math.max(
          0,
          Math.min(W, (Math.max(displayRegion.start_sec, displayRegion.end_sec) / durationSec) * W),
        );
        return { startX, endX };
      })()
    : null;

  return (
    <section className="wf-card">
      <div className={"wf-main" + (morphing ? " is-morphing" : "")}>
      {morphing && <AnalysisOrb phase="morph" peaks={channel} />}
      <svg
        className="wf"
        viewBox={`0 0 ${W} ${H}`}
        preserveAspectRatio="none"
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        role="slider"
        aria-label={
          regionsEnabled
            ? "Waveform — click to seek, shift+drag to set a loop region"
            : "Waveform — click to seek"
        }
        aria-valuemin={0}
        aria-valuemax={durationSec}
        aria-valuenow={currentTimeSec}
      >
        <WaveformDefs prefix="wf" />
        {/* The peak bars are a clip mask, drawn once. Two gradient-filled
            sheets are clipped by it: the full unplayed sheet, then the
            played sheet cut at the playhead — so the heard span lights up
            without duplicating the peak DOM or fighting CSS in a <use>
            shadow tree. */}
        <clipPath id="wf-bars-clip">
          {channel.map((v, i) => {
            const x = (i / channel.length) * W;
            const barW = (W / channel.length) * 0.85;
            const barH = v * (H * 0.88);
            const y = (H - barH) / 2;
            return <rect key={i} x={x} y={y} width={barW} height={barH} rx={0.5} />;
          })}
        </clipPath>
        <rect
          className="wf-sheet wf-sheet-unplayed"
          x={0}
          y={0}
          width={W}
          height={H}
          clipPath="url(#wf-bars-clip)"
        />
        <rect
          className="wf-sheet wf-sheet-played"
          x={0}
          y={0}
          width={playheadX}
          height={H}
          clipPath="url(#wf-bars-clip)"
        />
        {regionRect && (
          <rect
            className="wf-region"
            x={regionRect.startX}
            y={0}
            width={Math.max(1, regionRect.endX - regionRect.startX)}
            height={H}
          />
        )}
        <line
          className="wf-playhead"
          x1={playheadX}
          y1={0}
          x2={playheadX}
          y2={H}
        />
      </svg>
      <WaveformDbScale />
      </div>
      <WaveformOverview
        channel={channel}
        currentTimeSec={currentTimeSec}
        durationSec={durationSec}
        region={displayRegion}
        onSeek={onSeek}
      />
      {regionsEnabled && (
        <p className="wf-hint">
          Click to seek. Shift+drag to define a loop region. Shift+click clears it.
        </p>
      )}
    </section>
  );
}

function WaveformOverview({
  channel,
  currentTimeSec,
  durationSec,
  region,
  onSeek,
}: {
  channel: number[];
  currentTimeSec: number;
  durationSec: number;
  region: LoopRegion | null;
  onSeek: (positionSec: number) => void;
}) {
  // Compact 48 px-high overview rendered below the main waveform. Click-to-
  // seek only — no shift-drag region edit here, the main waveform handles
  // that. Adds a "viewport" rectangle showing what's currently in the
  // main waveform's visible window; for v1 the main waveform shows the
  // whole track, so the viewport equals the visible region (or the loop
  // region if set).
  const W = 1000;
  const H = 48;
  const playheadX =
    durationSec > 0
      ? Math.max(0, Math.min(W, (currentTimeSec / durationSec) * W))
      : 0;
  const regionRect = region && durationSec > 0
    ? (() => {
        const startX = Math.max(
          0,
          Math.min(W, (Math.min(region.start_sec, region.end_sec) / durationSec) * W),
        );
        const endX = Math.max(
          0,
          Math.min(W, (Math.max(region.start_sec, region.end_sec) / durationSec) * W),
        );
        return { startX, endX };
      })()
    : null;
  const handlePointer = (e: ReactPointerEvent<SVGSVGElement>) => {
    if (durationSec <= 0) return;
    const rect = e.currentTarget.getBoundingClientRect();
    if (rect.width <= 0) return;
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    onSeek(ratio * durationSec);
  };
  return (
    <svg
      className="wf-overview"
      viewBox={`0 0 ${W} ${H}`}
      preserveAspectRatio="none"
      onPointerDown={handlePointer}
      role="slider"
      aria-label="Waveform overview — click to seek"
      aria-valuemin={0}
      aria-valuemax={durationSec}
      aria-valuenow={currentTimeSec}
    >
      <WaveformDefs prefix="wfo" />
      <clipPath id="wfo-bars-clip">
        {channel.map((v, i) => {
          const x = (i / channel.length) * W;
          const barW = (W / channel.length) * 0.85;
          const barH = v * (H * 0.92);
          const y = (H - barH) / 2;
          return <rect key={i} x={x} y={y} width={barW} height={barH} rx={0.5} />;
        })}
      </clipPath>
      <rect
        className="wf-sheet wf-sheet-unplayed"
        x={0}
        y={0}
        width={W}
        height={H}
        clipPath="url(#wfo-bars-clip)"
      />
      <rect
        className="wf-sheet wf-sheet-played"
        x={0}
        y={0}
        width={playheadX}
        height={H}
        clipPath="url(#wfo-bars-clip)"
      />
      {regionRect && (
        <rect
          className="wf-overview-region"
          x={regionRect.startX}
          y={0}
          width={Math.max(1, regionRect.endX - regionRect.startX)}
          height={H}
        />
      )}
      <line
        className="wf-overview-playhead"
        x1={playheadX}
        y1={0}
        x2={playheadX}
        y2={H}
      />
    </svg>
  );
}

// Shared gradient defs for the main waveform ("wf") and the overview ("wfo").
// Both SVGs are always mounted together, so the ids must differ per prefix.
// Unplayed bars are a dim cool cobalt that brightens toward the centre line;
// the played span is lit — the same hue, pushed toward white at the peaks.
// Both sheets span the full viewBox height, so the default bounding-box
// gradient lands identically on each even though the played sheet is narrower.
function WaveformDefs({ prefix }: { prefix: string }) {
  return (
    <defs>
      <linearGradient id={`${prefix}-fill-unplayed`} x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stopColor="#3760b8" stopOpacity="0.55" />
        <stop offset="50%" stopColor="#5f8fe6" stopOpacity="0.85" />
        <stop offset="100%" stopColor="#3760b8" stopOpacity="0.55" />
      </linearGradient>
      {/* Owner 2026-08-19: ~25% less intense than the first cut — each stop
          is blended a quarter of the way back toward the unplayed colour, so
          the heard span still reads as lit without going to paper-white. */}
      <linearGradient id={`${prefix}-fill-played`} x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stopColor="#6995ed" stopOpacity="0.85" />
        <stop offset="50%" stopColor="#bcd1f9" stopOpacity="0.96" />
        <stop offset="100%" stopColor="#6995ed" stopOpacity="0.85" />
      </linearGradient>
    </defs>
  );
}
