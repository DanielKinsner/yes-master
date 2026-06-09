import { describe, expect, it } from "vitest";
import {
  MIN_PREVIEW_HEIGHT_PX,
  computeRailAlignment,
  type RailAlignmentInput,
} from "./rail-alignment";

// A comfortable 940px-class window: intensity card ends mid-column,
// loudness ends above the rail's content bottom, everything fits.
function baseInput(): RailAlignmentInput {
  return {
    previewTop: 96,
    intensityBottom: 620,
    loudnessBottom: 840,
    railContentBottom: 900,
    deliveryHeight: 120,
    exportHeight: 56,
    railGap: 16,
  };
}

describe("computeRailAlignment", () => {
  it("aligns the seams in the normal fits-without-scrolling case", () => {
    const result = computeRailAlignment(baseInput());
    expect(result).toEqual({
      // Preview bottom flush with intensity bottom: 620 - 96.
      previewHeightPx: 524,
      // Export group lifted flush with loudness bottom: 900 - 840.
      exportMarginBottomPx: 60,
    });
  });

  it("falls back when the preview would collapse below the meter minimum", () => {
    const input = baseInput();
    input.intensityBottom = input.previewTop + MIN_PREVIEW_HEIGHT_PX - 1;
    expect(computeRailAlignment(input)).toBeNull();
  });

  it("keeps the preview at exactly the meter minimum", () => {
    const input = baseInput();
    input.intensityBottom = input.previewTop + MIN_PREVIEW_HEIGHT_PX;
    // Loudness sits low enough that delivery + export still fit below.
    input.loudnessBottom = input.railContentBottom - 4;
    expect(computeRailAlignment(input)).toEqual({
      previewHeightPx: MIN_PREVIEW_HEIGHT_PX,
      exportMarginBottomPx: 4,
    });
  });

  it("falls back when the center column ends below the rail content box", () => {
    const input = baseInput();
    input.loudnessBottom = input.railContentBottom + 10;
    expect(computeRailAlignment(input)).toBeNull();
  });

  it("falls back when the pinned export would overlap the delivery card", () => {
    const input = baseInput();
    // Lift the export group high enough to collide with Delivery Format.
    input.loudnessBottom = input.intensityBottom + input.railGap + 40;
    expect(computeRailAlignment(input)).toBeNull();
  });

  it("falls back on non-finite measurements", () => {
    expect(
      computeRailAlignment({ ...baseInput(), intensityBottom: NaN }),
    ).toBeNull();
    expect(
      computeRailAlignment({ ...baseInput(), railContentBottom: Infinity }),
    ).toBeNull();
  });

  it("allows a zero export lift when loudness ends exactly at the rail bottom", () => {
    const input = baseInput();
    input.loudnessBottom = input.railContentBottom;
    expect(computeRailAlignment(input)).toEqual({
      previewHeightPx: 524,
      exportMarginBottomPx: 0,
    });
  });
});
