// src/hooks/useFirstRunGuide.ts
//
// Stateful shell around the pure deriveGuideStep machine. Owns the
// localStorage gate, the "fast user" silent finish, the aha persistence,
// and the send-off display window. Rendering lives in StandardView.

import { useEffect, useRef, useState } from "react";
import {
  FIRST_RUN_GUIDE_RESET_EVENT,
  deriveGuideStep,
  guideAlreadyFinished,
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
    if (!chipWasVisible.current) {
      markGuideFinished(storage, "done");
      setStarted(false);
      return;
    }
    if (!flipped) {
      // The aha happened. Persist immediately — the send-off and Advanced
      // pointer are session-only from here.
      markGuideFinished(storage, "done");
      setFlipped(true);
    }
  }, [args.playbackKind, started, flipped, storage]);

  useEffect(() => {
    if (step !== "sendoff") return;
    const t = setTimeout(() => setSendOffElapsed(true), SEND_OFF_MS);
    return () => clearTimeout(t);
  }, [step]);

  // Settings' "Show first-run tips again" revives a mounted guide
  // immediately — without this, the reset only worked after a relaunch.
  useEffect(() => {
    const revive = () => {
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
