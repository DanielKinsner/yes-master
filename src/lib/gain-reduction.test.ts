import { describe, expect, it } from "vitest";
import { GR_SCALE_DB, grLabel, grToFill } from "./gain-reduction";

describe("grToFill", () => {
  it("maps 0 dB and the silence sentinel to an empty bar", () => {
    expect(grToFill(0)).toBe(0);
    expect(grToFill(-120)).toBe(0);
    expect(grToFill(undefined)).toBe(0);
  });
  it("maps -6 dB to half of a 12 dB scale", () => {
    expect(GR_SCALE_DB).toBe(12);
    expect(grToFill(-6)).toBeCloseTo(0.5, 6);
  });
  it("clamps beyond the scale", () => {
    expect(grToFill(-40)).toBe(1);
    expect(grToFill(3)).toBe(0);
  });
});

describe("grLabel", () => {
  it("shows a dash for nothing and one decimal otherwise", () => {
    expect(grLabel(undefined)).toBe("—");
    expect(grLabel(-120)).toBe("—");
    expect(grLabel(0)).toBe("0.0 dB");
    expect(grLabel(-3.24)).toBe("-3.2 dB");
  });
});
