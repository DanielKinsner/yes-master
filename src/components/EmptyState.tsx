// src/components/EmptyState.tsx
//
// The app's idle face and first-launch welcome: the orb idles as the brand
// visual, with the product promise underneath. Dropping a track flows the
// same orb into the analysis ceremony and then into the real waveform —
// one continuous visual thread. Never a wall: import/drag-drop always
// works; this is just what the empty slot looks like.

import { prefersReducedMotion } from "../lib/motion";
import { SUPPORTED_FORMATS_COPY } from "../lib/supported-formats";
import { AnalysisOrb } from "./AnalysisOrb";

export function EmptyState({
  onAdd,
  onDemo,
}: {
  onAdd: () => void;
  /// Pass 4 (2026-08-19): "Try a demo track" — the engine synthesises a
  /// short loop into app-data and imports it, so a first-run user with
  /// nothing to drop still reaches the Original → Mastered moment.
  onDemo?: () => void;
}) {
  const reducedMotion = prefersReducedMotion();
  return (
    <div className="empty-state empty-hero">
      <div className="empty-hero-orb" aria-hidden>
        {reducedMotion ? <StaticGlyph /> : <AnalysisOrb phase="orb" />}
      </div>
      <p className="empty-hero-brand">YES Master</p>
      <h1>Drop audio. Hear it mastered.</h1>
      <p>
        Audition Original vs Mastered on your own track, shape the sound, and
        export a technically checked master — without ever risking the source
        file.
      </p>
      <div className="empty-hero-actions">
        <button type="button" className="primary" onClick={onAdd}>
          Import audio
        </button>
        {onDemo && (
          <button type="button" className="ghost-btn empty-hero-demo" onClick={onDemo}>
            Try a demo track
          </button>
        )}
      </div>
      <p className="empty-foot">
        Supports {SUPPORTED_FORMATS_COPY}.
      </p>
    </div>
  );
}

/// Reduced-motion fallback: the original static waveform glyph.
function StaticGlyph() {
  return (
    <svg width="64" height="64" viewBox="0 0 64 64" fill="none">
      <defs>
        <linearGradient id="emptyglow" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#6fa3ff" />
          <stop offset="1" stopColor="#2a6bf2" />
        </linearGradient>
      </defs>
      <circle cx="32" cy="32" r="28" stroke="url(#emptyglow)" strokeWidth="2" opacity="0.5" />
      <path
        d="M14 32h4l2-12 4 24 4-18 4 14 4-10 4 8 4-6 4 4h4"
        stroke="url(#emptyglow)"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
    </svg>
  );
}
