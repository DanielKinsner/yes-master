// src/lib/first-run-guide.test.ts
import { describe, expect, it } from "vitest";
import {
  FIRST_RUN_GUIDE_KEY,
  deriveGuideStep,
  guideAlreadyFinished,
  markGuideFinished,
  resetGuide,
} from "./first-run-guide";

const base = {
  started: true,
  hasAnalyzedTrack: true,
  flippedToMastered: false,
  sendOffElapsed: false,
  advancedDone: false,
};

describe("deriveGuideStep", () => {
  it("is null before the guide starts or before analysis", () => {
    expect(deriveGuideStep({ ...base, started: false })).toBeNull();
    expect(deriveGuideStep({ ...base, hasAnalyzedTrack: false })).toBeNull();
  });

  it("asks for the flip once a track is analyzed", () => {
    expect(deriveGuideStep(base)).toBe("flip");
  });

  it("moves to the send-off after the flip", () => {
    expect(deriveGuideStep({ ...base, flippedToMastered: true })).toBe("sendoff");
  });

  it("moves to the Advanced pointer after the send-off window", () => {
    expect(
      deriveGuideStep({ ...base, flippedToMastered: true, sendOffElapsed: true }),
    ).toBe("advanced");
  });

  it("ends after the Advanced pointer is done", () => {
    expect(
      deriveGuideStep({
        ...base,
        flippedToMastered: true,
        sendOffElapsed: true,
        advancedDone: true,
      }),
    ).toBeNull();
  });
});

function fakeStorage(): Storage {
  const map = new Map<string, string>();
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    removeItem: (k: string) => void map.delete(k),
  } as Storage;
}

describe("guide storage", () => {
  it("is unfinished on a fresh machine and finished after done or dismissed", () => {
    const s = fakeStorage();
    expect(guideAlreadyFinished(s)).toBe(false);
    markGuideFinished(s, "done");
    expect(guideAlreadyFinished(s)).toBe(true);
    expect(s.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
    markGuideFinished(s, "dismissed");
    expect(s.getItem(FIRST_RUN_GUIDE_KEY)).toBe("dismissed");
    expect(guideAlreadyFinished(s)).toBe(true);
  });

  it("reset clears the flag so the guide can run again", () => {
    const s = fakeStorage();
    markGuideFinished(s, "done");
    resetGuide(s);
    expect(guideAlreadyFinished(s)).toBe(false);
  });

  it("tolerates a missing storage object", () => {
    expect(guideAlreadyFinished(undefined)).toBe(false);
    expect(() => markGuideFinished(undefined, "done")).not.toThrow();
    expect(() => resetGuide(undefined)).not.toThrow();
  });
});
