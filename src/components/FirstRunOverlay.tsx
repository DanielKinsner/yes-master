// src/components/FirstRunOverlay.tsx
//
// L9: the first-run hint is a quiet floating coachmark, not an inline rail
// chip. Rendered once at the App root (a sibling of the drop/toast overlays)
// so it floats over the layout instead of pushing it — and so position:fixed
// resolves against the viewport rather than being trapped by the
// transform-animated .standard-view ancestor. The per-step copy and marker
// classes are unchanged from the chips this replaces; only the placement moved.

import type { ReactNode } from "react";
import { HintChip } from "./HintChip";
import type { GuideStep } from "../lib/first-run-guide";

const STEP_CONTENT: Record<GuideStep, { className: string; body: ReactNode }> = {
  flip: {
    className: "hint-chip-flip",
    body: (
      <>
        Press Play, then flip to <strong>Mastered</strong> to hear the
        difference.
      </>
    ),
  },
  sendoff: {
    className: "hint-chip-sendoff",
    body: (
      <>
        That's the whole idea. Presets and Intensity shape the sound — explore.
      </>
    ),
  },
  advanced: {
    className: "hint-chip-advanced",
    body: (
      <>
        Need more control? Try <strong>Advanced</strong> — top right.
      </>
    ),
  },
};

export function FirstRunOverlay({
  step,
  onDismiss,
}: {
  step: GuideStep | null;
  onDismiss: () => void;
}) {
  if (step === null) return null;
  const { className, body } = STEP_CONTENT[step];
  return (
    // position/z-index inline so the float holds regardless of stylesheet
    // load order; per-step top/right/bottom offsets live in App.css against
    // the first-run-overlay-<step> class.
    <div
      className={`first-run-overlay first-run-overlay-${step}`}
      style={{ position: "fixed", zIndex: 120 }}
    >
      <HintChip className={className} onDismiss={onDismiss}>
        {body}
      </HintChip>
    </div>
  );
}
