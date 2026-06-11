// src/lib/first-run-guide.ts
//
// First-run guide: pure step machine for the Standard-view hint chips.
// The guide exists to produce one aha-moment — flipping Original→Mastered
// on the user's own track — then a short send-off and a single Advanced
// pointer. Storage gates whether the guide ever STARTS; every step derives
// from live app state (never a step counter), so the guide cannot desync.

export const FIRST_RUN_GUIDE_KEY = "yes-master:first-run-guide";

export type GuideStep = "flip" | "sendoff" | "advanced";

export type GuideInputs = {
  /// False when storage said done/dismissed at mount (or the user
  /// pre-flipped before the guide could ever appear).
  started: boolean;
  hasAnalyzedTrack: boolean;
  /// playbackKind has been "master" at some point since the guide started.
  flippedToMastered: boolean;
  /// The send-off chip finished its display window.
  sendOffElapsed: boolean;
  /// The Advanced pointer was dismissed or Advanced was entered.
  advancedDone: boolean;
};

export function deriveGuideStep(i: GuideInputs): GuideStep | null {
  if (!i.started) return null;
  if (!i.hasAnalyzedTrack) return null;
  if (!i.flippedToMastered) return "flip";
  if (!i.sendOffElapsed) return "sendoff";
  if (!i.advancedDone) return "advanced";
  return null;
}

type GuideStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

export function guideAlreadyFinished(storage: GuideStorage | undefined): boolean {
  const v = storage?.getItem(FIRST_RUN_GUIDE_KEY);
  return v === "done" || v === "dismissed";
}

export function markGuideFinished(
  storage: GuideStorage | undefined,
  how: "done" | "dismissed",
): void {
  storage?.setItem(FIRST_RUN_GUIDE_KEY, how);
}

export function resetGuide(storage: GuideStorage | undefined): void {
  storage?.removeItem(FIRST_RUN_GUIDE_KEY);
}

/// Fired on window when the user asks to see the tips again, so a mounted
/// guide revives immediately instead of waiting for the next app launch.
export const FIRST_RUN_GUIDE_RESET_EVENT = "yes-master:first-run-guide-reset";

export function requestGuideReset(storage: GuideStorage | undefined): void {
  resetGuide(storage);
  globalThis.dispatchEvent?.(new Event(FIRST_RUN_GUIDE_RESET_EVENT));
}
