import { describe, expect, it } from "vitest";

import { rememberView, rememberedView, type ViewByTrack } from "./view-by-track";

describe("per-track view memory (F6)", () => {
  it("records an explicit choice and reads it back", () => {
    const m = rememberView({}, "a", "advanced");
    expect(rememberedView(m, "a")).toBe("advanced");
  });

  it("returns null for a track with no explicit choice", () => {
    expect(rememberedView({ a: "advanced" }, "b")).toBeNull();
    expect(rememberedView({}, "a")).toBeNull();
  });

  it("ignores a null track id (nothing selected) and never writes for it", () => {
    const m: ViewByTrack = { a: "standard" };
    expect(rememberView(m, null, "advanced")).toBe(m); // same ref, unchanged
    expect(rememberedView(m, null)).toBeNull();
  });

  it("keeps other tracks' choices when one changes — no global clobber", () => {
    const twoTracks = rememberView(rememberView({}, "a", "advanced"), "b", "standard");
    const flipped = rememberView(twoTracks, "b", "advanced"); // b flips, a untouched
    expect(rememberedView(flipped, "a")).toBe("advanced");
    expect(rememberedView(flipped, "b")).toBe("advanced");
  });

  it("returns the same reference when the value is unchanged (stable state)", () => {
    const m: ViewByTrack = { a: "advanced" };
    expect(rememberView(m, "a", "advanced")).toBe(m);
  });
});
