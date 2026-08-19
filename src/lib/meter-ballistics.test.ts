import { describe, expect, it } from "vitest";
import { DEFAULT_METER_CONFIG, METER_SILENT, stepMeter } from "./meter-ballistics";

const cfg = { fallDbPerSec: 24, holdMs: 1000, holdFallDbPerSec: 12 };

describe("stepMeter", () => {
  it("seeds display and hold at the first input", () => {
    const s = stepMeter(null, -10, 0, cfg);
    expect(s.display).toBe(-10);
    expect(s.hold).toBe(-10);
    expect(s.holdUntilMs).toBe(1000);
    expect(s.lastMs).toBe(0);
  });

  it("rises instantly", () => {
    const a = stepMeter(null, -20, 0, cfg);
    const b = stepMeter(a, -6, 50, cfg);
    expect(b.display).toBe(-6);
    expect(b.hold).toBe(-6);
  });

  it("falls at the configured rate, not instantly", () => {
    const a = stepMeter(null, -6, 0, cfg);
    const b = stepMeter(a, -30, 500, cfg); // 0.5 s * 24 dB/s = 12 dB of fall allowed
    expect(b.display).toBeCloseTo(-18, 5);
  });

  it("never falls below the input", () => {
    const a = stepMeter(null, -6, 0, cfg);
    const b = stepMeter(a, -8, 5000, cfg);
    expect(b.display).toBe(-8);
  });

  it("holds the peak for holdMs then decays at holdFallDbPerSec", () => {
    const a = stepMeter(null, -6, 0, cfg);
    const held = stepMeter(a, -30, 900, cfg);
    expect(held.hold).toBe(-6);
    const after = stepMeter(held, -30, 1500, cfg); // 0.5 s past holdUntil → 6 dB decay
    expect(after.hold).toBeCloseTo(-12, 5);
  });

  it("a new, higher peak re-arms the hold", () => {
    const a = stepMeter(null, -6, 0, cfg);
    const b = stepMeter(a, -3, 700, cfg);
    expect(b.hold).toBe(-3);
    expect(b.holdUntilMs).toBe(1700);
  });

  it("silence sentinel lets both values decay and never rises", () => {
    const a = stepMeter(null, -6, 0, cfg);
    const b = stepMeter(a, METER_SILENT, 250, cfg);
    expect(b.display).toBeCloseTo(-12, 5);
    expect(b.hold).toBe(-6);
  });

  it("display and hold floor at the silence sentinel", () => {
    const a = stepMeter(null, -6, 0, cfg);
    const b = stepMeter(a, METER_SILENT, 60_000, cfg);
    expect(b.display).toBe(METER_SILENT);
    expect(b.hold).toBe(METER_SILENT);
  });

  it("default config is meter-like", () => {
    expect(DEFAULT_METER_CONFIG.fallDbPerSec).toBeGreaterThan(0);
    expect(DEFAULT_METER_CONFIG.holdMs).toBeGreaterThanOrEqual(500);
  });
});
