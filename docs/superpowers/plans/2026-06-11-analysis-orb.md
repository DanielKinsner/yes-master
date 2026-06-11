# Analysis Orb Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A particle orb animates in the waveform region while analysis runs, then dissolves into the track's real waveform when the preview is ready (v1: orb + morph, timer-paced stage labels unchanged).

**Architecture:** Pure geometry/easing helpers in `src/lib/analysis-orb.ts` (mechanically tested) + one canvas component `src/components/AnalysisOrb.tsx` (rAF loop, DPR-aware, draws only) + two seams in `src/components/Waveform.tsx`: `WaveformLoading` hosts the orb during `analyzing` mode, `WaveformView` hosts a short morph overlay when analysis just finished and peaks arrived. The PARENT owns the morph lifetime (timer + interaction cut); the child only draws. StandardView renders `WaveformView` unconditionally so both views share the same seams. No Rust changes, no new dependencies.

**Tech Stack:** Canvas 2D, React 19, vitest + jsdom (note: jsdom's `canvas.getContext` returns null — the component must no-op-draw and tests assert structure, not pixels).

**Spec:** `docs/superpowers/specs/2026-06-11-analysis-orb-design.md`

Key codebase facts (verified 2026-06-11):

- `WaveformLoading` (Waveform.tsx:11) renders the staged label + progress bar; mode comes from `waveformLoadingView` ("analyzing" | "loading" | "idle").
- `WaveformView` (Waveform.tsx:73) internally renders the `wf-card` + `WaveformLoading` when `!peaks` — and it already receives `isAnalyzing`. In Advanced (App.tsx:729) it stays mounted across the analyzing→peaks transition.
- StandardView currently ternaries `tm.selectedWaveform ? WaveformView : WaveformLoading` (StandardView.tsx:~470). `.std-wave .wf-card` is already styled (App.css:5425), so always rendering WaveformView is layout-safe.
- Peaks shape: `peaks.channels[0]` is `number[]` of 0..1 amplitudes; the SVG draws bar height `v * 0.88H` centered vertically.
- Reduced motion: `globalThis.matchMedia` may be absent in jsdom — guard with optional chaining (absent → motion allowed).

---

### Task 1: Pure helpers

**Files:**
- Create: `src/lib/analysis-orb.ts`
- Test: `src/lib/analysis-orb.test.ts`

- [ ] **Step 1: Write failing tests**

```ts
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
```

- [ ] **Step 2: Run, expect resolve failure** — `npx vitest run src/lib/analysis-orb.test.ts`

- [ ] **Step 3: Implement**

```ts
// src/lib/analysis-orb.ts
//
// Pure geometry/easing for the analysis orb: a point-cloud sphere that
// breathes while analysis runs, then dissolves into the track's real
// waveform. Everything here is deterministic and unit-tested; the canvas
// component only iterates and draws.

export type OrbParticle = {
  /// Unit-sphere position.
  x: number; y: number; z: number;
  /// Per-particle phase so the noise field doesn't move in lockstep.
  jitter: number;
  /// Waveform slot: horizontal position 0..1 …
  u: number;
  /// … and vertical position inside the bar, -1..1 (scaled by peak height).
  tyUnit: number;
};

/// Small deterministic PRNG so the loading orb and the morph overlay can
/// regenerate the SAME cloud from a fixed seed (visual continuity across
/// the WaveformLoading → WaveformView remount).
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function gauss(random: () => number): number {
  let u = 0, v = 0;
  while (!u) u = random();
  while (!v) v = random();
  return Math.sqrt(-2 * Math.log(u)) * Math.cos(6.283185307179586 * v);
}

export function createOrbParticles(
  count: number,
  random: () => number,
): OrbParticle[] {
  const out: OrbParticle[] = [];
  for (let i = 0; i < count; i++) {
    let x = gauss(random), y = gauss(random), z = gauss(random);
    const m = Math.hypot(x, y, z) || 1;
    x /= m; y /= m; z /= m;
    out.push({ x, y, z, jitter: random() * 6.283, u: random() * 0.999999, tyUnit: random() * 2 - 1 });
  }
  return out;
}

/// Wispy radial displacement (the "ethereal" look): a sum of slow sines in
/// particle-space and time. Bounded well inside (0.7, 1.3) so the cloud
/// never collapses or explodes.
export function orbDisplacement(p: OrbParticle, t: number): number {
  return (
    1 +
    0.17 * Math.sin(3 * p.x + t * 1.1 + p.jitter) * Math.sin(2.3 * p.y - t * 0.8) +
    0.06 * Math.sin(5 * p.z + t * 1.7)
  );
}

/// Vertical morph target in -1..1, scaled by the peak amplitude at the
/// particle's horizontal slot. Mirrors the SVG's centered-bar geometry.
export function waveformTargetY(
  tyUnit: number,
  u: number,
  peaks: readonly number[],
): number {
  if (peaks.length === 0) return 0;
  const idx = Math.min(peaks.length - 1, Math.floor(u * peaks.length));
  return tyUnit * peaks[idx];
}

export function easeInOutQuad(t: number): number {
  const c = Math.max(0, Math.min(1, t));
  return c < 0.5 ? 2 * c * c : 1 - Math.pow(-2 * c + 2, 2) / 2;
}

export const ORB_PARTICLE_COUNT = 1800;
export const ORB_SEED = 92;
export const MORPH_MS = 1400;
```

- [ ] **Step 4: Run, expect pass** — `npx vitest run src/lib/analysis-orb.test.ts`

- [ ] **Step 5: Commit** — `feat(orb): pure particle/morph geometry helpers`

---

### Task 2: AnalysisOrb canvas component

**Files:**
- Create: `src/components/AnalysisOrb.tsx`
- Modify: `src/App.css` (append)
- Test: `src/components/AnalysisOrb.test.tsx`

- [ ] **Step 1: Failing test**

```tsx
// src/components/AnalysisOrb.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import { AnalysisOrb } from "./AnalysisOrb";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => { document.body.innerHTML = ""; });

describe("AnalysisOrb", () => {
  it("renders a presentation canvas and survives jsdom's null 2d context", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => { root.render(<AnalysisOrb phase="orb" />); });
    const canvas = container.querySelector("canvas.wf-orb");
    expect(canvas).not.toBeNull();
    expect(canvas?.getAttribute("aria-hidden")).toBe("true");
    await act(async () => root.unmount());
  });

  it("renders the morph variant with peaks without crashing", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => {
      root.render(<AnalysisOrb phase="morph" peaks={[0.2, 0.9, 0.4]} />);
    });
    expect(container.querySelector("canvas.wf-orb.is-morph")).not.toBeNull();
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run, expect failure** — `npx vitest run src/components/AnalysisOrb.test.tsx`

- [ ] **Step 3: Implement**

```tsx
// src/components/AnalysisOrb.tsx
//
// The analysis orb: ~2k particles on a breathing point-cloud sphere while
// analysis runs ("orb"), or flying into the track's waveform shape
// ("morph"). Draw-only: the PARENT owns when this mounts/unmounts (incl.
// the morph window timer and interaction cuts). jsdom returns a null 2d
// context — in that case the component renders the canvas and does nothing.

import { useEffect, useRef } from "react";
import {
  MORPH_MS,
  ORB_PARTICLE_COUNT,
  ORB_SEED,
  createOrbParticles,
  easeInOutQuad,
  mulberry32,
  orbDisplacement,
  waveformTargetY,
} from "../lib/analysis-orb";

export function AnalysisOrb({
  phase,
  peaks = null,
}: {
  phase: "orb" | "morph";
  peaks?: readonly number[] | null;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const dpr = globalThis.devicePixelRatio || 1;
    const particles = createOrbParticles(ORB_PARTICLE_COUNT, mulberry32(ORB_SEED));
    const start = performance.now();
    let frame = 0;

    const draw = (now: number) => {
      frame = requestAnimationFrame(draw);
      if (document.visibilityState === "hidden") return;
      const rect = canvas.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;
      const W = Math.round(rect.width * dpr);
      const H = Math.round(rect.height * dpr);
      if (canvas.width !== W || canvas.height !== H) {
        canvas.width = W;
        canvas.height = H;
      }
      ctx.clearRect(0, 0, W, H);

      const t = (now - start) / 1000;
      const mix = phase === "morph" ? easeInOutQuad((now - start) / MORPH_MS) : 0;
      const cx = W / 2, cy = H / 2;
      const R = Math.min(W, H) * 0.36;
      const ry = t * 0.4, rx = Math.sin(t * 0.23) * 0.35;
      const cosY = Math.cos(ry), sinY = Math.sin(ry);
      const cosX = Math.cos(rx), sinX = Math.sin(rx);

      for (const p of particles) {
        const x1 = p.x * cosY + p.z * sinY;
        const z1 = -p.x * sinY + p.z * cosY;
        const y1 = p.y * cosX - z1 * sinX;
        const z2 = p.y * sinX + z1 * cosX;
        const n = orbDisplacement(p, t);
        const persp = 1.7 / (1.7 + z2 * 0.9);
        const ox = cx + x1 * n * R * persp;
        const oy = cy + y1 * n * R * persp;

        let X = ox, Y = oy;
        if (mix > 0) {
          const wx = p.u * W;
          const wy = cy + waveformTargetY(p.tyUnit, p.u, peaks ?? []) * H * 0.44;
          X = ox + (wx - ox) * mix;
          Y = oy + (wy - oy) * mix;
        }
        const depth = (1 - z2) / 2;
        // Morph ending: particles thin out as the real waveform fades in.
        const alpha = (0.18 + 0.5 * depth) * (1 - 0.85 * mix * mix);
        ctx.fillStyle = `rgba(140, 176, 255, ${alpha.toFixed(3)})`;
        const s = (0.8 + 0.8 * depth) * dpr;
        ctx.fillRect(X, Y, s, s);
      }
    };
    frame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frame);
  }, [phase, peaks]);

  return (
    <canvas
      ref={canvasRef}
      className={"wf-orb" + (phase === "morph" ? " is-morph" : "")}
      aria-hidden="true"
    />
  );
}
```

- [ ] **Step 4: Append CSS to `src/App.css`**

```css
/* Analysis orb: particle canvas during analysis + the morph overlay. */
.wf-orb {
  display: block;
  width: 100%;
  flex: 1 1 auto;
  min-height: 0;
}
.wf-orb.is-morph {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
.wf-loading-has-orb {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.wf-main.is-morphing .wf {
  animation: wf-morph-fade-in 1400ms ease-in both;
}
@keyframes wf-morph-fade-in {
  0% { opacity: 0; }
  55% { opacity: 0.12; }
  100% { opacity: 1; }
}
@media (prefers-reduced-motion: reduce) {
  .wf-main.is-morphing .wf { animation: none; }
}
```

- [ ] **Step 5: Run, expect pass; commit** — `feat(orb): AnalysisOrb canvas component`

---

### Task 3: Orb inside WaveformLoading (both views)

**Files:**
- Modify: `src/components/Waveform.tsx` (`WaveformLoading`)
- Test: `src/App.progress-and-reset.test.tsx` (extend `WaveformLoading` describe)

- [ ] **Step 1: Failing tests** — append to the `WaveformLoading` describe:

```tsx
it("hosts the analysis orb while analyzing", async () => {
  const { container, root } = await render(
    <WaveformLoading isAnalyzing={true} isLoadingWaveform={false}
      analysisProgress={{ label: "Checking dynamics", progress: 0.5 }} />,
  );
  expect(container.querySelector("canvas.wf-orb")).not.toBeNull();
  await act(async () => root.unmount());
});

it("does not host the orb when idle or merely decoding", async () => {
  const { container, root } = await render(
    <WaveformLoading isAnalyzing={false} isLoadingWaveform={true} analysisProgress={null} />,
  );
  expect(container.querySelector("canvas.wf-orb")).toBeNull();
  await act(async () => root.unmount());
});
```

(match that file's existing render helper; add one if absent.)

- [ ] **Step 2: Run, expect failure.**

- [ ] **Step 3: Implement** — in `WaveformLoading`, compute reduced motion and render the orb above the label row when analyzing:

```tsx
const reducedMotion =
  globalThis.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false;
const showOrb = view.mode === "analyzing" && !reducedMotion;
```

Root div gains `wf-loading-has-orb` when `showOrb`, and `{showOrb && <AnalysisOrb phase="orb" />}` renders before `.wf-loading-row`. Import `AnalysisOrb`.

- [ ] **Step 4: Run file + full suite, expect pass; commit** — `feat(orb): particle orb in the analyzing waveform slot`

---

### Task 4: Morph overlay in WaveformView + Standard seam

**Files:**
- Modify: `src/components/Waveform.tsx` (`WaveformView`)
- Modify: `src/components/StandardView.tsx` (drop the ternary; always `WaveformView`)
- Test: `src/App.progress-and-reset.test.tsx`

- [ ] **Step 1: Failing tests**

```tsx
describe("WaveformView morph", () => {
  const peaks = { channels: [[0.2, 0.8, 0.5, 0.9]], length: 4 };
  function view(p: { peaks?: typeof peaks; isAnalyzing: boolean }) {
    return (
      <WaveformView peaks={p.peaks} isLoading={false} isAnalyzing={p.isAnalyzing}
        analysisProgress={null} currentTimeSec={0} durationSec={10} region={null}
        onSeek={() => {}} onSetRegion={() => {}} onClearRegion={() => {}} />
    );
  }

  it("plays the morph when analysis just finished and peaks arrive, then clears", async () => {
    vi.useFakeTimers();
    try {
      const { container, root } = await render(view({ isAnalyzing: true }));
      await act(async () => { root.render(view({ peaks, isAnalyzing: false })); });
      expect(container.querySelector(".wf-orb.is-morph")).not.toBeNull();
      expect(container.querySelector(".wf-main.is-morphing")).not.toBeNull();
      await act(async () => { vi.advanceTimersByTime(1400); });
      expect(container.querySelector(".wf-orb.is-morph")).toBeNull();
      await act(async () => root.unmount());
    } finally { vi.useRealTimers(); }
  });

  it("cuts the morph on user interaction", async () => {
    vi.useFakeTimers();
    try {
      const { container, root } = await render(view({ isAnalyzing: true }));
      await act(async () => { root.render(view({ peaks, isAnalyzing: false })); });
      expect(container.querySelector(".wf-orb.is-morph")).not.toBeNull();
      await act(async () => {
        window.dispatchEvent(new Event("pointerdown"));
      });
      expect(container.querySelector(".wf-orb.is-morph")).toBeNull();
      await act(async () => root.unmount());
    } finally { vi.useRealTimers(); }
  });

  it("shows no morph without a preceding analysis", async () => {
    const { container, root } = await render(view({ peaks, isAnalyzing: false }));
    expect(container.querySelector(".wf-orb.is-morph")).toBeNull();
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run, expect failure.**

- [ ] **Step 3: Implement** — in `WaveformView` (now a stateful transition watcher):

```tsx
const wasAnalyzingNoPeaks = useRef(false);
const [morphing, setMorphing] = useState(false);
const reducedMotion =
  globalThis.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false;

useEffect(() => {
  if (!peaks) {
    wasAnalyzingNoPeaks.current = isAnalyzing;
    return;
  }
  if (wasAnalyzingNoPeaks.current) {
    wasAnalyzingNoPeaks.current = false;
    if (!reducedMotion) setMorphing(true);
  }
}, [peaks, isAnalyzing, reducedMotion]);

useEffect(() => {
  if (!morphing) return;
  const cut = () => setMorphing(false);
  const t = setTimeout(cut, MORPH_MS);
  window.addEventListener("pointerdown", cut);
  window.addEventListener("keydown", cut);
  return () => {
    clearTimeout(t);
    window.removeEventListener("pointerdown", cut);
    window.removeEventListener("keydown", cut);
  };
}, [morphing]);
```

`.wf-main` gains `is-morphing` class and hosts `{morphing && <AnalysisOrb phase="morph" peaks={channel} />}` (channel is already computed). Ensure `.wf-main` has `position: relative` in CSS (add if missing).

In `StandardView.tsx`: replace the `tm.selectedWaveform ? <WaveformView/> : <WaveformLoading/>` ternary with a single `<WaveformView peaks={tm.selectedWaveform} ... />`; remove the now-unused `WaveformLoading` import.

- [ ] **Step 4: Run full suite, expect pass; commit** — `feat(orb): morph the orb into the real waveform on analysis completion`

---

### Task 5: Browser-preview realism + visual verification

**Files:**
- Modify: `src/lib/preview-mock.ts` (analysis delay so the orb is visible in browser preview)

- [ ] **Step 1:** In the preview mock's analyze command, add a ~4 s artificial delay (browser-preview only code; the real Tauri path never imports this). Find the analyze handler and prepend `await new Promise((r) => setTimeout(r, 4000));` with a comment explaining it exists so the analysis orb/staged progress are visible in browser preview.

- [ ] **Step 2:** Visual verification via preview server: trigger a (mock) import/re-analyze, confirm the orb renders during analysis, the morph plays into the waveform, console has no errors. Screenshot for the owner — final look-and-feel signoff is the owner's (taste), per spec.

- [ ] **Step 3: Commit** — `feat(orb): preview-mock analysis delay for browser verification`

---

### Task 6: Full verification + push

- [ ] `npm test` and `npm run build` clean.
- [ ] `git push`.

## Out of scope (per spec)

- Real `analysis:progress` events from Rust (future slice).
- Audio-seeded particle motion (future slice).
- Album batch analysis visuals; render/export visuals.
