import { describe, expect, it } from "vitest";
import { formatDuration } from "./time-format";

describe("formatDuration", () => {
  it("returns an empty string for absent or non-finite input", () => {
    expect(formatDuration(null)).toBe("");
    expect(formatDuration(undefined)).toBe("");
    expect(formatDuration(Number.NaN)).toBe("");
    expect(formatDuration(Number.POSITIVE_INFINITY)).toBe("");
    expect(formatDuration(Number.NEGATIVE_INFINITY)).toBe("");
  });

  it("floors seconds and formats m:ss", () => {
    expect(formatDuration(0)).toBe("0:00");
    expect(formatDuration(59.6)).toBe("0:59");
    expect(formatDuration(60)).toBe("1:00");
    expect(formatDuration(3599.9)).toBe("59:59");
  });

  it("clamps negative finite input to zero", () => {
    expect(formatDuration(-4.2)).toBe("0:00");
  });
});
