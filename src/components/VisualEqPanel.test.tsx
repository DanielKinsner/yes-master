import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

import type { MasteringSettings } from "../bindings";
import { EQ_BAND_DEFAULTS, EQ_BAND_RANGES } from "../bindings";
import { VisualEqPanel, formatHz, resolveBandHz } from "./VisualEqPanel";

// 2026-08-18 — the EQ nodes became draggable in FREQUENCY, not just gain.
// The engine reads per-band Hz from `settings.eq_bands`; these tests pin the
// UI half of that contract: a horizontal drag reports a clamped Hz for the
// right band, a double-click puts gain AND frequency back, and settings that
// predate the field still render at the chain's defaults.

function settings(overrides: Partial<MasteringSettings> = {}): MasteringSettings {
  return {
    preset: { kind: "universal" },
    intensity: 0.5,
    eq_sub_db: 0,
    eq_low_db: 0,
    eq_low_mid_db: 0,
    eq_mid_db: 0,
    eq_high_mid_db: 0,
    eq_high_db: 0,
    eq_sparkle_db: 0,
    eq_bands: { ...EQ_BAND_DEFAULTS },
    volume_match: false,
    input_gain_db: 0,
    output_gain_db: 0,
    delivery_profile: "streaming-universal",
    advanced: {
      lufs_offset_db: null,
      ceiling_dbtp: null,
      width: null,
      warmth: null,
      presence_air: null,
      compression_mode: "preset",
      compression_density: null,
      compression_low_threshold_db: null,
      compression_low_ratio: null,
      compression_low_attack_ms: null,
      compression_low_release_ms: null,
      compression_mid_threshold_db: null,
      compression_mid_ratio: null,
      compression_mid_attack_ms: null,
      compression_mid_release_ms: null,
      compression_high_threshold_db: null,
      compression_high_ratio: null,
      compression_high_attack_ms: null,
      compression_high_release_ms: null,
      compression_link_stereo: null,
      bit_depth: null,
      target_sample_rate: null,
      adaptive_strength: null,
    },
    ...overrides,
  };
}

// The compact panel's viewBox geometry (mirrors the constants in the
// component). jsdom has no layout, so the test stubs the SVG's screen CTM
// to identity — client coordinates then ARE viewBox coordinates.
const PAD_LEFT = 28;
const PAD_RIGHT = 8;
const PAD_TOP = 8;
const PAD_BOTTOM = 18;
const VBW = 420;
const VBH = 180;
const PLOT_W = VBW - PAD_LEFT - PAD_RIGHT;
const PLOT_H = VBH - PAD_TOP - PAD_BOTTOM;
const LOG_MIN = Math.log10(20);
const LOG_SPAN = Math.log10(20_000) - LOG_MIN;
const xForHz = (hz: number) => PAD_LEFT + ((Math.log10(hz) - LOG_MIN) / LOG_SPAN) * PLOT_W;
const yForDb = (db: number) => PAD_TOP + ((12 - db) / 24) * PLOT_H;

beforeAll(() => {
  const proto = SVGSVGElement.prototype as unknown as Record<string, unknown>;
  proto.createSVGPoint = () => {
    const pt = { x: 0, y: 0, matrixTransform: () => ({ x: pt.x, y: pt.y }) };
    return pt;
  };
  proto.getScreenCTM = () => ({ inverse: () => ({}) });
  const el = Element.prototype as unknown as Record<string, unknown>;
  el.setPointerCapture = () => {};
  el.hasPointerCapture = () => true;
  el.releasePointerCapture = () => {};
});

let root: Root | null = null;
let host: HTMLDivElement | null = null;

function mount(props: Parameters<typeof VisualEqPanel>[0]) {
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  act(() => root!.render(<VisualEqPanel {...props} />));
  return host;
}

afterEach(() => {
  act(() => root?.unmount());
  host?.remove();
  root = null;
  host = null;
});

function pointer(type: string, target: Element, x: number, y: number) {
  const ev = new MouseEvent(type, { bubbles: true, clientX: x, clientY: y });
  Object.defineProperty(ev, "pointerId", { value: 1 });
  target.dispatchEvent(ev);
}

// Hit targets render in BANDS order: sub, low, low-mid, mid, high-mid, high, sparkle.
const MID_INDEX = 3;

describe("VisualEqPanel — frequency drag (2026-08-18)", () => {
  it("reports gain AND frequency in ONE call per pointer move (they must not race)", () => {
    const onEq = vi.fn();
    const onEqPoint = vi.fn();
    const el = mount({ settings: settings(), onEq, onEqPoint, compact: true });
    const hits = el.querySelectorAll(".eq-node-hit");
    expect(hits).toHaveLength(7);
    const mid = hits[MID_INDEX];
    act(() => pointer("pointerdown", mid, xForHz(1500), yForDb(0)));
    act(() => pointer("pointermove", mid, xForHz(2200), yForDb(3)));
    expect(onEqPoint).toHaveBeenLastCalledWith("mid", 3, 2200);
    // The single-mutation path replaces the gain-only setter; calling both
    // from one event is exactly the race this test exists to forbid.
    expect(onEq).not.toHaveBeenCalled();
    act(() => pointer("pointerup", mid, xForHz(2200), yForDb(3)));
  });

  it("clamps the frequency to the band's own window so neighbours cannot cross", () => {
    const onEqPoint = vi.fn();
    const el = mount({ settings: settings(), onEq: vi.fn(), onEqPoint, compact: true });
    const mid = el.querySelectorAll(".eq-node-hit")[MID_INDEX];
    const [lo, hi] = EQ_BAND_RANGES.mid_hz;
    act(() => pointer("pointerdown", mid, xForHz(1500), yForDb(0)));
    act(() => pointer("pointermove", mid, xForHz(20), yForDb(0)));
    expect(onEqPoint).toHaveBeenLastCalledWith("mid", 0, lo);
    act(() => pointer("pointermove", mid, xForHz(19_000), yForDb(0)));
    expect(onEqPoint).toHaveBeenLastCalledWith("mid", 0, hi);
  });

  it("double-click returns BOTH gain and frequency to the band's defaults", () => {
    const onEq = vi.fn();
    const onEqPoint = vi.fn();
    const el = mount({
      settings: settings({ eq_mid_db: 4, eq_bands: { ...EQ_BAND_DEFAULTS, mid_hz: 2600 } }),
      onEq,
      onEqPoint,
      compact: true,
    });
    const mid = el.querySelectorAll(".eq-node-hit")[MID_INDEX];
    act(() => mid.dispatchEvent(new MouseEvent("dblclick", { bubbles: true })));
    expect(onEqPoint).toHaveBeenCalledWith("mid", 0, EQ_BAND_DEFAULTS.mid_hz);
    expect(onEq).not.toHaveBeenCalled();
  });

  it("without onEqPoint the legacy gain-only path still drags vertically", () => {
    const onEq = vi.fn();
    const el = mount({ settings: settings(), onEq, compact: true });
    const mid = el.querySelectorAll(".eq-node-hit")[MID_INDEX];
    act(() => pointer("pointerdown", mid, xForHz(1500), yForDb(0)));
    act(() => pointer("pointermove", mid, xForHz(2200), yForDb(-4)));
    expect(onEq).toHaveBeenLastCalledWith("mid", -4);
  });

  it("draws the node at the band's live frequency and stays vertical-only without onEqPoint", () => {
    const onEq = vi.fn();
    const el = mount({
      settings: settings({ eq_bands: { ...EQ_BAND_DEFAULTS, mid_hz: 2600 } }),
      onEq,
      compact: true,
    });
    const nodes = el.querySelectorAll(".eq-node");
    const cx = Number(nodes[MID_INDEX].getAttribute("cx"));
    expect(cx).toBeCloseTo(xForHz(2600), 3);
    // Without an onEqPoint handler the hit target advertises vertical drag only.
    const hit = el.querySelectorAll(".eq-node-hit")[MID_INDEX] as SVGElement;
    expect(hit.style.cursor).toBe("ns-resize");
  });
});

describe("resolveBandHz / formatHz", () => {
  it("falls back to the chain defaults for settings that predate eq_bands", () => {
    const legacy = settings();
    delete (legacy as Partial<MasteringSettings>).eq_bands;
    const hz = resolveBandHz(legacy);
    expect(hz.sub).toBe(80);
    expect(hz.mid).toBe(1500);
    expect(hz.sparkle).toBe(12_000);
  });

  it("uses per-key defaults for a partial or corrupt eq_bands", () => {
    const partial = settings({
      eq_bands: { ...EQ_BAND_DEFAULTS, mid_hz: Number.NaN, low_hz: 160 },
    });
    const hz = resolveBandHz(partial);
    expect(hz.mid).toBe(1500);
    expect(hz.low).toBe(160);
  });

  it("formats Hz the way the axis does", () => {
    expect(formatHz(80)).toBe("80");
    expect(formatHz(1500)).toBe("1.5k");
    expect(formatHz(12_000)).toBe("12k");
    expect(formatHz(2200)).toBe("2.2k");
  });
});
