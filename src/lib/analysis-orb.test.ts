// src/lib/analysis-orb.test.ts
import { describe, expect, it } from "vitest";
import {
  createOrbParticles,
  easeInOutQuad,
  mulberry32,
  orbDisplacement,
  waveformTargetY,
} from "./analysis-orb";

describe("mulberry32", () => {
  it("is deterministic for a fixed seed and stays in [0,1)", () => {
    const a = mulberry32(7), b = mulberry32(7);
    for (let i = 0; i < 100; i++) {
      const v = a();
      expect(v).toBe(b());
      expect(v).toBeGreaterThanOrEqual(0);
      expect(v).toBeLessThan(1);
    }
  });
});

describe("createOrbParticles", () => {
  it("creates N particles on the unit sphere with waveform slots", () => {
    const ps = createOrbParticles(500, mulberry32(1));
    expect(ps).toHaveLength(500);
    for (const p of ps.slice(0, 20)) {
      expect(Math.hypot(p.x, p.y, p.z)).toBeCloseTo(1, 6);
      expect(p.u).toBeGreaterThanOrEqual(0);
      expect(p.u).toBeLessThan(1);
      expect(Math.abs(p.tyUnit)).toBeLessThanOrEqual(1);
    }
  });

  it("is deterministic for the same random source", () => {
    const a = createOrbParticles(10, mulberry32(2));
    const b = createOrbParticles(10, mulberry32(2));
    expect(a).toEqual(b);
  });
});

describe("waveformTargetY", () => {
  it("scales the particle's vertical slot by the peak amplitude at u", () => {
    const peaks = [0, 0.5, 1, 0.5];
    expect(waveformTargetY(1, 0.5, peaks)).toBeCloseTo(1, 6);
    expect(waveformTargetY(-1, 0.5, peaks)).toBeCloseTo(-1, 6);
    expect(waveformTargetY(1, 0.26, peaks)).toBeCloseTo(0.5, 6);
  });

  it("is 0 for empty peaks", () => {
    expect(waveformTargetY(1, 0.3, [])).toBe(0);
  });
});

describe("easeInOutQuad", () => {
  it("clamps and eases 0..1", () => {
    expect(easeInOutQuad(-1)).toBe(0);
    expect(easeInOutQuad(0)).toBe(0);
    expect(easeInOutQuad(0.5)).toBeCloseTo(0.5, 6);
    expect(easeInOutQuad(1)).toBe(1);
    expect(easeInOutQuad(2)).toBe(1);
  });
});

describe("orbDisplacement", () => {
  it("stays within the wispy band around the sphere surface", () => {
    const ps = createOrbParticles(50, mulberry32(3));
    for (const p of ps) {
      for (const t of [0, 1.7, 12.3]) {
        const n = orbDisplacement(p, t);
        expect(n).toBeGreaterThan(0.7);
        expect(n).toBeLessThan(1.3);
      }
    }
  });
});
