// U10 — count and album-flow vocabulary.

import { describe, expect, it } from "vitest";

import {
  ALBUM_FLOW_DESCRIPTION,
  SINGLE_TRACK_ALBUM_NOTE,
  arcOffsetLabel,
  flowAmountDescription,
  flowAmountValueText,
  pluralize,
  sequenceRoleLabel,
  trackCountLabel,
} from "./album-copy";
import { ALBUM_ARC_DISPLAY } from "../bindings";

describe("pluralization", () => {
  it("says '1 track', not '1 tracks'", () => {
    // The shipped defect, in the sidebar header and the album export receipt.
    expect(trackCountLabel(1)).toBe("1 track");
  });

  it("pluralizes every other count", () => {
    expect(trackCountLabel(0)).toBe("0 tracks");
    expect(trackCountLabel(2)).toBe("2 tracks");
    expect(trackCountLabel(12)).toBe("12 tracks");
  });

  it("supports an explicit irregular plural", () => {
    expect(pluralize(1, "analysis", "analyses")).toBe("1 analysis");
    expect(pluralize(3, "analysis", "analyses")).toBe("3 analyses");
  });
});

describe("album flow vocabulary", () => {
  it("describes every flow the picker can select", () => {
    // A flow with a name and no explanation is what shipped. If a new arc kind
    // is added without a description, this fails rather than shipping a
    // nameless behavior.
    for (const kind of Object.keys(ALBUM_ARC_DISPLAY) as Array<
      keyof typeof ALBUM_ARC_DISPLAY
    >) {
      const description = ALBUM_FLOW_DESCRIPTION[kind];
      expect(description, `no description for flow "${kind}"`).toBeTruthy();
      expect(description.length).toBeGreaterThan(20);
    }
  });

  it("never claims a flow improves the music", () => {
    // The album promise is "one coherent record"; the flow is an expressive
    // bonus on top of it (owner decision 2026-07-03). Copy must describe a
    // shape, not assert a quality outcome.
    const banned = /\bbetter\b|\bimprove|\bprofessional\b|\bperfect\b/i;
    for (const description of Object.values(ALBUM_FLOW_DESCRIPTION)) {
      expect(banned.test(description)).toBe(false);
    }
  });
});

describe("flow amount", () => {
  it("explains what the multiplier scales instead of only printing a number", () => {
    // `Flow Amount ×1.00` was on screen with nothing anywhere to say what it
    // meant.
    expect(flowAmountDescription(1)).toContain("Full");
    expect(flowAmountDescription(0)).toContain("same album loudness target");
    expect(flowAmountDescription(0.5)).toContain("50%");
    expect(flowAmountDescription(1.6)).toContain("160%");
  });

  it("gives a spoken value for the slider", () => {
    expect(flowAmountValueText(0)).toContain("off");
    expect(flowAmountValueText(1)).toContain("full");
    expect(flowAmountValueText(1.5)).toContain("150 percent");
  });

  it("stays sane on a non-finite value", () => {
    expect(flowAmountDescription(Number.NaN)).toContain("unavailable");
    expect(flowAmountValueText(Number.NaN)).toBe("unavailable");
  });
});

describe("arc offset labels", () => {
  it("signs the offset and marks a true zero explicitly", () => {
    expect(arcOffsetLabel(-2.1)).toBe("−2.1 LU");
    expect(arcOffsetLabel(1.8)).toBe("+1.8 LU");
    // "±0.0" so no-offset is distinguishable from a missing value.
    expect(arcOffsetLabel(0)).toBe("±0.0 LU");
    expect(arcOffsetLabel(0.01)).toBe("±0.0 LU");
  });

  it("returns nothing for an absent offset", () => {
    expect(arcOffsetLabel(null)).toBe("");
    expect(arcOffsetLabel(undefined)).toBe("");
    expect(arcOffsetLabel(Number.NaN)).toBe("");
  });
});

describe("sequence roles", () => {
  it("labels the backend's role vocabulary", () => {
    expect(sequenceRoleLabel("opener")).toBe("Opener");
    expect(sequenceRoleLabel("album_track")).toBe("Album track");
    expect(sequenceRoleLabel("closer")).toBe("Closer");
  });

  it("stays empty for an unknown or absent role", () => {
    expect(sequenceRoleLabel(null)).toBe("");
    expect(sequenceRoleLabel("something_new")).toBe("");
  });
});

describe("single-track album", () => {
  it("explains the degenerate case rather than leaving a dead control", () => {
    expect(SINGLE_TRACK_ALBUM_NOTE).toContain("at least two tracks");
    // It must not read as an error — a one-track album still renders.
    expect(SINGLE_TRACK_ALBUM_NOTE).toContain("still apply");
  });
});
