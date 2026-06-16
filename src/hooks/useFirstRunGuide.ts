// src/hooks/useFirstRunGuide.ts
//
// Stateful shell around the pure deriveGuideStep machine. Owns the
// localStorage gate, the "fast user" silent finish, the aha persistence,
// and the send-off display window. Rendering lives in FirstRunOverlay.

import { useEffect, useRef, useState } from "react";
import {
  FIRST_RUN_GUIDE_RESET_EVENT,
  deriveGuideStep,
  guideAlreadyFinished,
  guideWasReset,
  markGuideFinished,
  type GuideStep,
} from "../lib/first-run-guide";

const SEND_OFF_MS = 6000;

export type FirstRunGuide = {
  step: GuideStep | null;
  /// × on any chip: ends the whole guide, persisted as "dismissed".
  dismiss: () => void;
  /// Any route into Advanced ends the guide silently as "done".
  noteEnteredAdvanced: () => void;
};

export function useFirstRunGuide(args: {
  hasAnalyzedTrack: boolean;
  playbackKind: string;
  isPlaying: boolean;
}): FirstRunGuide {
  const storage = globalThis.localStorage;
  // Snapshot once at mount: a returning user never re-enters the guide.
  const [started, setStarted] = useState(() => !guideAlreadyFinished(storage));
  const [flipped, setFlipped] = useState(false);
  const [sendOffElapsed, setSendOffElapsed] = useState(false);
  const [advancedDone, setAdvancedDone] = useState(false);
  // True once the flip chip has actually been on screen. Distinguishes a
  // guided flip from a fast user who reached Mastered before the guide
  // could appear — those users are never lectured (silent finish).
  const chipWasVisible = useRef(false);
  // An explicit Settings reset is a REQUEST to be shown tips: it bypasses
  // both the silent finish AND the audible-flip requirement so the click
  // always produces visible feedback (flip chip on Original, send-off when
  // already on Mastered).
  const armedByReset = useRef(guideWasReset(storage));

  const rawStep = deriveGuideStep({
    started,
    hasAnalyzedTrack: args.hasAnalyzedTrack,
    flippedToMastered: flipped,
    sendOffElapsed,
    advancedDone,
  });
  // Never render the flip chip while playback is ALREADY on master — that
  // state is either a fast user about to be silently finished by the effect
  // below, or the one-frame gap before `flipped` flows through. Both would
  // otherwise flash the chip.
  const step = rawStep === "flip" && args.playbackKind === "master" ? null : rawStep;
  if (step === "flip") chipWasVisible.current = true;

  useEffect(() => {
    if (args.playbackKind !== "master" || !started) return;
    if (armedByReset.current) {
      // Reset path: acknowledge the flip immediately (even paused) so the
      // Settings action always visibly does something.
      if (!flipped) {
        markGuideFinished(storage, "done");
        setFlipped(true);
      }
      return;
    }
    if (!chipWasVisible.current) {
      markGuideFinished(storage, "done");
      setStarted(false);
      return;
    }
    if (!flipped && args.isPlaying) {
      // The aha happened — and was actually HEARD (a paused flip teaches
      // nothing, so the guide waits for play before completing). Persist
      // immediately; the send-off and Advanced pointer are session-only.
      markGuideFinished(storage, "done");
      setFlipped(true);
    }
  }, [args.playbackKind, args.isPlaying, started, flipped, storage]);

  useEffect(() => {
    if (step !== "sendoff") return;
    const t = setTimeout(() => setSendOffElapsed(true), SEND_OFF_MS);
    return () => clearTimeout(t);
  }, [step]);

  // Settings' "Show first-run tips again" revives a mounted guide
  // immediately — without this, the reset only worked after a relaunch.
  useEffect(() => {
    const revive = () => {
      armedByReset.current = true;
      chipWasVisible.current = false;
      setStarted(true);
      setFlipped(false);
      setSendOffElapsed(false);
      setAdvancedDone(false);
    };
    window.addEventListener(FIRST_RUN_GUIDE_RESET_EVENT, revive);
    return () => window.removeEventListener(FIRST_RUN_GUIDE_RESET_EVENT, revive);
  }, []);

  const dismiss = () => {
    markGuideFinished(storage, "dismissed");
    setStarted(false);
    setAdvancedDone(true);
  };

  const noteEnteredAdvanced = () => {
    if (started) markGuideFinished(storage, "done");
    setStarted(false);
    setAdvancedDone(true);
  };

  return { step, dismiss, noteEnteredAdvanced };
}
