# UI Polish — Pass 1 "Make it feel alive" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the desktop app the motion its headline promises — the Original→Mastered flip reads as an event, the meters behave like real meters (peak hold, smoothed fall, live gain-reduction), view switches transition instead of cutting, and export completion feels like a completion.

**Architecture:** Every effect is presentation-only: CSS transitions/keyframes driven by a data attribute or a class, plus two small pure TypeScript helpers (meter ballistics, GR normalisation) that are unit-tested. No audio-path, playhead, render, or export semantics change. Every new keyframe is registered with the existing U11 motion-discipline tests (`src/App.delight.test.tsx`): it gets a reduced-motion opt-out, animates only compositor-safe properties, and names its purpose in a CSS comment.

**Tech Stack:** React 19 + TypeScript, single stylesheet `src/App.css`, Vitest + jsdom for tests, Tauri `playback:tick` events (20 Hz) as the live data source.

## Global Constraints

- **Presentation only.** No change to `src-tauri/`, to `useTrackMaster` playback/seek/export logic, or to when `sendUpdateChain` fires. The audio crossfade on A/B already exists (`src-tauri/src/audio.rs:2445-2468`, L10) — do not touch it.
- **Every new keyframe/transition** gets: (a) a `/* PURPOSE: … */` comment at its definition, (b) a selector entry in the reduced-motion block list in `src/App.delight.test.tsx` `every U11 effect has a reduced-motion opt-out`, (c) an entry in the `FORBIDDEN` keyframe scan in the same file. Compositor-safe properties only: `opacity`, `transform`, `box-shadow`, `filter`, `stroke-dashoffset`, colour.
- **No layout shift.** Nothing new changes an element's box size.
- **Sentinels:** playback tick dB values use `-120` = silence / no signal. Treat `<= -119` as "no value".
- **Commit per task**, small; commit message prefixes follow the repo convention (`design(...)`, `feat(...)`, `test:`).
- **Landing captures** (`src/assets/landing/desktop-*.png`) are bound to the app's visual state by `src/lib/landing-assets.test.ts`; recapture ONCE at the end of the pass with `npm run capture:landing` (Task 7), not per task.
- After the pass: `npm test`, `npm run build`, and `npm run verify:headless` must be green before the final commit.

---

## File Structure

| File | Responsibility |
|---|---|
| `src/App.tsx` (modify ~`:393`) | Stamp `data-playback-kind` + `data-playing` on the `.app` root so CSS can key on "what am I hearing". |
| `src/App.css` (modify) | All new motion rules; grouped in one new section at the end: `/* ===== ALIVE PASS 1 (2026-08-19) ===== */`. |
| `src/lib/meter-ballistics.ts` (create) | Pure `stepMeter()` — display value with fall rate, peak-hold with hold time + decay. No React. |
| `src/lib/meter-ballistics.test.ts` (create) | Unit tests for `stepMeter`. |
| `src/hooks/useMeterBallistics.ts` (create) | Tiny hook that runs `stepMeter` per render with `performance.now()`. |
| `src/components/RightRail.tsx` (modify `MasterOutPanel`, `PeakMeterBar`) | Peak-hold pip + ballistic fill. |
| `src/lib/gain-reduction.ts` (create) | Pure `grToFill()` (negative dB / sentinel → 0..1) + `grLabel()`. |
| `src/lib/gain-reduction.test.ts` (create) | Unit tests. |
| `src/components/AdvancedPanel.tsx` (modify `PerBandCompressorCard`) | Live per-band GR bars fed from the transport tick. |
| `src/components/ExportReceiptCard.tsx` (modify) | Class hooks for the verdict-icon draw. |
| `src/App.delight.test.tsx` (modify) | Register every new effect. |

---

### Task 1: Playback-kind state on the app root

The whole pass keys off "are we listening to the master right now". React already knows (`tm.transport.playbackKind`, `tm.transport.isPlaying`); CSS does not.

**Files:**
- Modify: `src/App.tsx:393`
- Test: `src/App.playback-kind.test.tsx` (create)

**Interfaces:**
- Produces: the `.app` root carries `data-playback-kind="source" | "master"` and `data-playing="true" | "false"`. Later tasks key CSS on `.app[data-playback-kind="master"]` and `.app[data-playing="true"]`.

- [ ] **Step 1: Write the failing test**

Look at how `src/App.warning-ownership.test.tsx` mounts `<App />` with the preview mock runtime (it imports `./App` and renders under `createRoot`; copy its mock setup verbatim — the preview runtime in `src/lib/preview-mock.ts` drives a fake transport). Create `src/App.playback-kind.test.tsx`:

```tsx
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import App from "./App";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.innerHTML = "";
});

describe("app root exposes what the user is hearing", () => {
  it("stamps data-playback-kind and data-playing on the .app root", async () => {
    const host = document.createElement("div");
    document.body.appendChild(host);
    root = createRoot(host);
    await act(async () => {
      root!.render(<App />);
    });
    const app = host.querySelector(".app");
    expect(app).not.toBeNull();
    expect(app!.getAttribute("data-playback-kind")).toMatch(/^(source|master)$/);
    expect(app!.getAttribute("data-playing")).toMatch(/^(true|false)$/);
  });
});
```

If `App.warning-ownership.test.tsx` needs extra setup (e.g. `vi.mock` of `./lib/tauri-runtime`), copy that block above the `describe`.

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run src/App.playback-kind.test.tsx`
Expected: FAIL — `expected null to match /^(source|master)$/`.

- [ ] **Step 3: Stamp the attributes**

In `src/App.tsx`, change line 393:

```tsx
    <div
      className={"app" + (view === "standard" ? " app-standard" : "")}
      data-playback-kind={tm.transport.playbackKind === "master" ? "master" : "source"}
      data-playing={tm.transport.isPlaying ? "true" : "false"}
    >
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npx vitest run src/App.playback-kind.test.tsx`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx src/App.playback-kind.test.tsx
git commit -m "feat(app): root carries data-playback-kind / data-playing so CSS can react to what the user hears"
```

---

### Task 2: The A/B flip moment

When Mastered is selected the toggle button "lands" (a 360 ms scale/glow bloom) and the waveform's played span lights warmer — you *see* the chain switch in.

**Files:**
- Modify: `src/App.css` (append new section)
- Modify: `src/App.delight.test.tsx:172-184` (reduced-motion list) and `:245-250` (FORBIDDEN keyframe list)

**Interfaces:**
- Consumes: `.app[data-playback-kind="master"]` from Task 1; `.ab-toggle button.on` (`App.css:1649`), `.std-rail-ab button.on`, `.wf-sheet-played` (`Waveform.tsx:290, 423`).

- [ ] **Step 1: Extend the delight tests (failing first)**

In `src/App.delight.test.tsx`, in the `every U11 effect has a reduced-motion opt-out` list add:

```ts
      ".ab-toggle button.on",
      ".wf-sheet-played",
```

In `no U11 effect animates a property that can reflow the page` add `"ab-land"` to the `for (const name of [...])` list.

- [ ] **Step 2: Run to verify it fails**

Run: `npx vitest run src/App.delight.test.tsx`
Expected: FAIL — `keyframes ab-land missing` and the reduced-motion `toContain(".ab-toggle button.on")`.

- [ ] **Step 3: Add the CSS**

Append to `src/App.css`:

```css
/* =====================================================================
   ALIVE PASS 1 (2026-08-19) — motion that says "you are hearing the master"
   Every rule here is presentation-only and keys on the .app root's
   data-playback-kind / data-playing attributes (App.tsx). Each effect
   names its PURPOSE (U11 rule) and opts out under reduced motion.
   ===================================================================== */

/* PURPOSE: ORIENTATION — the selected side of the A/B lands with a short
   bloom so the flip reads as an event, not a colour change. Fires when
   `.on` is (re)applied, i.e. exactly on a flip. */
@keyframes ab-land {
  0%   { transform: scale(0.98); box-shadow: inset 0 1px 0 rgba(255,255,255,0.16), 0 0 0 rgba(79,134,247,0); }
  45%  { transform: scale(1.035); box-shadow: inset 0 1px 0 rgba(255,255,255,0.2), 0 0 22px rgba(122,166,255,0.55); }
  100% { transform: scale(1); box-shadow: inset 0 1px 0 rgba(255,255,255,0.16), var(--accent-glow-soft); }
}
.ab-toggle button.on,
.std-rail-ab button.on {
  animation: ab-land 360ms var(--ease-out) both;
}

/* PURPOSE: COMPREHENSION — while the MASTER is what you hear, the played
   span of the waveform runs warmer/brighter than the cobalt used for the
   original. Filter only: no reflow, gradient stops stay as authored. */
.wf-sheet-played {
  transition: filter 320ms var(--ease-out);
}
.app[data-playback-kind="master"] .wf-sheet-played {
  filter: saturate(1.18) brightness(1.12);
}

@media (prefers-reduced-motion: reduce) {
  .ab-toggle button.on,
  .std-rail-ab button.on { animation: none; }
  .wf-sheet-played { transition: none; }
}
```

- [ ] **Step 4: Run tests**

Run: `npx vitest run src/App.delight.test.tsx src/App.layout-css.test.ts`
Expected: PASS.

- [ ] **Step 5: Eyeball it in the preview**

Start the dev preview (`.claude/launch.json` `yes-master-dev` → `http://localhost:<port>/app`), press Play, flip Original→Mastered. Expected: the Mastered pill blooms once; the lit played span brightens over ~⅓ s; flipping back relaxes it. Take a screenshot mid-play on Mastered for the commit.

- [ ] **Step 6: Commit**

```bash
git add src/App.css src/App.delight.test.tsx
git commit -m "design(ab): the flip lands — selected side blooms once, played span runs brighter while Mastered is audible"
```

---

### Task 3: Meter ballistics — pure helper

Real meters fall at a controlled rate and hold their peak. Build the maths as a pure function first.

**Files:**
- Create: `src/lib/meter-ballistics.ts`
- Create: `src/lib/meter-ballistics.test.ts`

**Interfaces:**
- Produces:
  ```ts
  export type MeterState = { display: number; hold: number; holdUntilMs: number };
  export const METER_SILENT = -120;
  export type MeterConfig = { fallDbPerSec: number; holdMs: number; holdFallDbPerSec: number };
  export const DEFAULT_METER_CONFIG: MeterConfig;
  export function stepMeter(prev: MeterState | null, inputDb: number, nowMs: number, cfg?: MeterConfig): MeterState;
  ```
  Semantics: `display` rises instantly to `inputDb`, falls at `fallDbPerSec`; `hold` is the max seen, kept for `holdMs` after it was set, then falls at `holdFallDbPerSec`. `inputDb <= -119` means silence: display falls, hold decays, nothing rises. `prev === null` seeds at the input.

- [ ] **Step 1: Write the failing tests**

`src/lib/meter-ballistics.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { DEFAULT_METER_CONFIG, METER_SILENT, stepMeter } from "./meter-ballistics";

const cfg = { fallDbPerSec: 24, holdMs: 1000, holdFallDbPerSec: 12 };

describe("stepMeter", () => {
  it("seeds display and hold at the first input", () => {
    const s = stepMeter(null, -10, 0, cfg);
    expect(s.display).toBe(-10);
    expect(s.hold).toBe(-10);
    expect(s.holdUntilMs).toBe(1000);
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
```

- [ ] **Step 2: Run to verify failure**

Run: `npx vitest run src/lib/meter-ballistics.test.ts`
Expected: FAIL — cannot resolve `./meter-ballistics`.

- [ ] **Step 3: Implement**

`src/lib/meter-ballistics.ts` (the state carries `lastMs` so the step can compute `dt` without a clock of its own):

```ts
// Meter ballistics for the live MASTER OUT meters and per-band GR bars.
// Pure: no React, no timers. Callers pass `performance.now()`.
//
// A real meter rises instantly and FALLS at a controlled rate; a peak-hold
// pip sits at the highest value seen, waits `holdMs`, then sinks. Values are
// dB; `METER_SILENT` (-120) is the backend's silence sentinel — treated as
// "no input": everything decays, nothing rises.

export const METER_SILENT = -120;
const SILENT_THRESHOLD = -119;

export type MeterState = {
  display: number;
  hold: number;
  holdUntilMs: number;
  lastMs: number;
};

export type MeterConfig = {
  fallDbPerSec: number;
  holdMs: number;
  holdFallDbPerSec: number;
};

export const DEFAULT_METER_CONFIG: MeterConfig = {
  fallDbPerSec: 24,
  holdMs: 1000,
  holdFallDbPerSec: 12,
};

export function stepMeter(
  prev: MeterState | null,
  inputDb: number,
  nowMs: number,
  cfg: MeterConfig = DEFAULT_METER_CONFIG,
): MeterState {
  const input = Number.isFinite(inputDb) && inputDb > SILENT_THRESHOLD ? inputDb : METER_SILENT;
  if (!prev) {
    return { display: input, hold: input, holdUntilMs: nowMs + cfg.holdMs, lastMs: nowMs };
  }
  const dtSec = Math.max(0, (nowMs - prev.lastMs) / 1000);

  // Display: instant rise, rate-limited fall, never below the input.
  const fallen = prev.display - cfg.fallDbPerSec * dtSec;
  const display = Math.max(input, fallen, METER_SILENT);

  // Hold: re-arm on a new high; otherwise wait out holdMs, then sink.
  let hold = prev.hold;
  let holdUntilMs = prev.holdUntilMs;
  if (input >= prev.hold) {
    hold = input;
    holdUntilMs = nowMs + cfg.holdMs;
  } else if (nowMs > prev.holdUntilMs) {
    const overshootSec = (nowMs - Math.max(prev.holdUntilMs, prev.lastMs)) / 1000;
    hold = Math.max(input, prev.hold - cfg.holdFallDbPerSec * overshootSec, METER_SILENT);
  }
  return { display, hold, holdUntilMs, lastMs: nowMs };
}
```

Update the test's first case to also `expect(s.lastMs).toBe(0)`.

- [ ] **Step 4: Run tests**

Run: `npx vitest run src/lib/meter-ballistics.test.ts`
Expected: PASS (9 tests). If "holds the peak… then decays" is off by the 100 ms between `holdUntilMs` and `lastMs`, re-check: at `nowMs=1500`, `holdUntilMs=1000`, `lastMs=900` → overshoot from `max(1000,900)=1000` → 0.5 s → 6 dB → `-12`. Correct.

- [ ] **Step 5: Commit**

```bash
git add src/lib/meter-ballistics.ts src/lib/meter-ballistics.test.ts
git commit -m "feat(meters): pure meter ballistics — rate-limited fall + peak hold with decay"
```

---

### Task 4: Peak-hold pips + ballistic fill on MASTER OUT

**Files:**
- Create: `src/hooks/useMeterBallistics.ts`
- Modify: `src/components/RightRail.tsx:339-500` (`MasterOutPanel`, `PeakMeterBar`)
- Modify: `src/App.css` (ALIVE section)
- Test: `src/components/RightRail.test.tsx` (add cases under `describe("MasterOutPanel")`)

**Interfaces:**
- Consumes: `stepMeter`, `MeterState`, `METER_SILENT` from Task 3.
- Produces: `useMeterBallistics(valueDb: number | undefined, active: boolean): { display: number | undefined; hold: number | undefined }` — `undefined` when idle or below the sentinel.
- `PeakMeterBar` gains optional `hold?: number` and renders `<div class="lufs-bar-hold">` positioned at the held level; adds class `is-hold-over` when hold > `PEAK_CEIL_DBFS` (−1).

- [ ] **Step 1: Write the failing component tests**

In `src/components/RightRail.test.tsx`, inside `describe("MasterOutPanel", …)` (the file already has `renderNode`/`flush` helpers and a `MasterOutPanel` mount at `:128` — follow that shape):

```tsx
  it("renders a peak-hold pip per channel while playing", async () => {
    const { container } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-6}
        peakLeftDbfs={-6}
        peakRightDbfs={-9}
        isPlaying
        lufsMomentary={-14}
        lufsIntegrated={-14.5}
      />,
    );
    const pips = container.querySelectorAll(".lufs-bar-hold");
    expect(pips.length).toBe(2);
    // -6 dB on a -36..0 scale sits at 83.33% from the bottom.
    expect((pips[0] as HTMLElement).style.bottom).toBe("83.33333333333334%");
  });

  it("hides the peak-hold pip while idle", async () => {
    const { container } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-6}
        peakLeftDbfs={-6}
        peakRightDbfs={-6}
        isPlaying={false}
        lufsMomentary={-14}
        lufsIntegrated={-14.5}
      />,
    );
    expect(container.querySelectorAll(".lufs-bar-hold").length).toBe(0);
  });

  it("flags a hold above the -1 dBFS ceiling", async () => {
    const { container } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-0.3}
        peakLeftDbfs={-0.3}
        peakRightDbfs={-12}
        isPlaying
        lufsMomentary={-10}
        lufsIntegrated={-10}
      />,
    );
    const pips = container.querySelectorAll(".lufs-bar-hold");
    expect(pips[0].classList.contains("is-hold-over")).toBe(true);
    expect(pips[1].classList.contains("is-hold-over")).toBe(false);
  });
```

- [ ] **Step 2: Run to verify failure**

Run: `npx vitest run src/components/RightRail.test.tsx`
Expected: the three new cases FAIL (`.lufs-bar-hold` count 0).

- [ ] **Step 3: Create the hook**

`src/hooks/useMeterBallistics.ts`:

```ts
import { useRef } from "react";
import {
  DEFAULT_METER_CONFIG,
  METER_SILENT,
  stepMeter,
  type MeterConfig,
  type MeterState,
} from "../lib/meter-ballistics";

/// Runs the pure meter ballistics once per render against `valueDb`.
/// `active=false` (idle/stopped) resets the state and reports nothing, so a
/// stopped meter never shows a stale hold. The tick cadence (~20 Hz) is the
/// clock; no timers are started here.
export function useMeterBallistics(
  valueDb: number | undefined,
  active: boolean,
  cfg: MeterConfig = DEFAULT_METER_CONFIG,
): { display: number | undefined; hold: number | undefined } {
  const stateRef = useRef<MeterState | null>(null);
  if (!active) {
    stateRef.current = null;
    return { display: undefined, hold: undefined };
  }
  const now = typeof performance !== "undefined" ? performance.now() : Date.now();
  const next = stepMeter(stateRef.current, valueDb ?? METER_SILENT, now, cfg);
  stateRef.current = next;
  return {
    display: next.display > -119 ? next.display : undefined,
    hold: next.hold > -119 ? next.hold : undefined,
  };
}
```

- [ ] **Step 4: Wire it into MasterOutPanel / PeakMeterBar**

In `src/components/RightRail.tsx`:

Add import at the top: `import { useMeterBallistics } from "../hooks/useMeterBallistics";`

Inside `MasterOutPanel`, after `const liveR = live(peakRightDbfs);` add:

```tsx
  const ballL = useMeterBallistics(liveL, isPlaying);
  const ballR = useMeterBallistics(liveR, isPlaying);
```

Replace the two `PeakMeterBar` usages with:

```tsx
          <PeakMeterBar value={ballL.display} hold={ballL.hold} label="L" title="Left channel peak (dBFS)" />
          <PeakMeterBar value={ballR.display} hold={ballR.hold} label="R" title="Right channel peak (dBFS)" />
```

Change `PeakMeterBar`'s signature and body:

```tsx
function PeakMeterBar({
  value,
  hold,
  label,
  title,
}: {
  value: number | undefined;
  hold?: number;
  label: string;
  title?: string;
}) {
  const ratio = (db: number): number => {
    if (!Number.isFinite(db)) return 0;
    const clamped = Math.max(PEAK_SCALE_MIN, Math.min(PEAK_SCALE_MAX, db));
    return (clamped - PEAK_SCALE_MIN) / (PEAK_SCALE_MAX - PEAK_SCALE_MIN);
  };
  const fill = value !== undefined ? ratio(value) : 0;
  const clipping = value !== undefined && value > -0.1;
  const ceilRatio = ratio(PEAK_CEIL_DBFS);
  const holdOver = hold !== undefined && hold > PEAK_CEIL_DBFS;
  return (
    <div className={`lufs-bar${clipping ? " is-clipping" : ""}`} title={title}>
      <div className="lufs-bar-track" />
      <div className="lufs-bar-fill" style={{ height: `${fill * 100}%` }} />
      {hold !== undefined && (
        <div
          className={`lufs-bar-hold${holdOver ? " is-hold-over" : ""}`}
          style={{ bottom: `${ratio(hold) * 100}%` }}
          aria-hidden
        />
      )}
      <div
        className="peak-ceil-line"
        style={{ bottom: `${ceilRatio * 100}%` }}
        title="-1 dBFS ceiling"
      />
      <span className="lufs-bar-label">{label}</span>
    </div>
  );
}
```

- [ ] **Step 5: CSS for the pip**

Append inside the ALIVE section of `src/App.css`:

```css
/* PURPOSE: COMPREHENSION — peak-hold pip: the highest recent peak stays
   visible for a second so a transient you heard is a line you can read.
   Positioned by `bottom` (set inline, not animated); only opacity/colour
   transition. */
.lufs-bar-hold {
  position: absolute;
  left: 0;
  right: 0;
  height: 2px;
  margin-bottom: -1px;
  background: #e8efff;
  box-shadow: 0 0 6px rgba(122, 166, 255, 0.7);
  opacity: 0.95;
  pointer-events: none;
  transition: background-color 120ms linear, box-shadow 120ms linear;
}
.lufs-bar-hold.is-hold-over {
  background: #ff5d5d;
  box-shadow: 0 0 8px rgba(255, 93, 93, 0.8);
}
@media (prefers-reduced-motion: reduce) {
  .lufs-bar-hold { transition: none; }
}
```

Also register `".lufs-bar-hold"` in the delight test's reduced-motion list.

- [ ] **Step 6: Run tests**

Run: `npx vitest run src/components/RightRail.test.tsx src/App.delight.test.tsx`
Expected: PASS.

- [ ] **Step 7: Eyeball in the preview**

Play in the browser preview (mock runtime feeds a fake tick). Expected: L/R fills now sink rather than snap; a thin bright pip rides above each bar and drops after ~1 s; it turns red above −1 dBFS. Stop → pips vanish.

- [ ] **Step 8: Commit**

```bash
git add src/hooks/useMeterBallistics.ts src/components/RightRail.tsx src/components/RightRail.test.tsx src/App.css src/App.delight.test.tsx
git commit -m "design(meters): MASTER OUT gets peak-hold pips and rate-limited fall — meters behave like meters"
```

---

### Task 5: Live per-band gain reduction in the compressor card

`compressionGr` arrives on every tick (`useTrackMaster.ts:705-709`) and is displayed **nowhere**. Show it where the compressor lives.

**Files:**
- Create: `src/lib/gain-reduction.ts`, `src/lib/gain-reduction.test.ts`
- Modify: `src/components/AdvancedPanel.tsx:33` (props), `:523` (`PerBandCompressorCard`)
- Modify: `src/App.tsx:450-475` (pass `liveGr`)
- Modify: `src/App.css` (ALIVE section)
- Test: `src/components/AdvancedPanel.test.tsx` (add)

**Interfaces:**
- Produces:
  ```ts
  // src/lib/gain-reduction.ts
  export const GR_SCALE_DB = 12; // full bar = 12 dB of reduction
  export function grToFill(grDb: number | undefined): number; // 0..1; sentinel/undefined/positive → 0
  export function grLabel(grDb: number | undefined): string;   // "—" or "-3.2 dB"
  ```
- `AdvancedPanel` gains prop `liveGr?: { low: number; mid: number; high: number } | null` and `isPlayingMaster?: boolean`; passes both to `PerBandCompressorCard`.
- `PerBandCompressorCard` renders `<div class="gr-meters">` with three `<div class="gr-meter">` rows (label + track + fill with inline `transform: scaleX(fill)`) only when `compressorMode !== "off"`; fills are 0 when not `isPlayingMaster`.

- [ ] **Step 1: Failing unit tests for the helper**

`src/lib/gain-reduction.test.ts`:

```ts
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
```

- [ ] **Step 2: Run to verify failure**

Run: `npx vitest run src/lib/gain-reduction.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement the helper**

`src/lib/gain-reduction.ts`:

```ts
// Per-band compressor gain reduction, as the backend reports it on every
// playback tick: dB, NEGATIVE means reduction, -120 is the "no reduction /
// no signal" sentinel (audio.rs PlaybackTick gr_*_db).

export const GR_SCALE_DB = 12;
const SILENT_THRESHOLD = -119;

export function grToFill(grDb: number | undefined): number {
  if (grDb === undefined || !Number.isFinite(grDb) || grDb <= SILENT_THRESHOLD) return 0;
  if (grDb >= 0) return 0;
  return Math.min(1, -grDb / GR_SCALE_DB);
}

export function grLabel(grDb: number | undefined): string {
  if (grDb === undefined || !Number.isFinite(grDb) || grDb <= SILENT_THRESHOLD) return "—";
  return `${grDb.toFixed(1)} dB`;
}
```

- [ ] **Step 4: Run helper tests**

Run: `npx vitest run src/lib/gain-reduction.test.ts`
Expected: PASS.

- [ ] **Step 5: Failing component test**

In `src/components/AdvancedPanel.test.tsx` (follow its existing mount helper; it renders `<AdvancedPanel …>` with a `SETTINGS` fixture — reuse that fixture and prop spread), add:

```tsx
  it("shows live per-band gain reduction while the master plays", async () => {
    const { container } = await renderPanel({
      liveGr: { low: -6, mid: -3, high: -120 },
      isPlayingMaster: true,
    });
    const fills = container.querySelectorAll(".gr-meter-fill");
    expect(fills.length).toBe(3);
    expect((fills[0] as HTMLElement).style.transform).toBe("scaleX(0.5)");
    expect((fills[1] as HTMLElement).style.transform).toBe("scaleX(0.25)");
    expect((fills[2] as HTMLElement).style.transform).toBe("scaleX(0)");
    expect(container.querySelector(".gr-meters")!.getAttribute("data-live")).toBe("true");
  });

  it("keeps GR bars empty when not auditioning the master", async () => {
    const { container } = await renderPanel({
      liveGr: { low: -6, mid: -3, high: -1 },
      isPlayingMaster: false,
    });
    for (const el of container.querySelectorAll(".gr-meter-fill")) {
      expect((el as HTMLElement).style.transform).toBe("scaleX(0)");
    }
  });
```

Where `renderPanel(extraProps)` is whatever the file's existing helper is called — if it doesn't take overrides, add a second argument that spreads onto `<AdvancedPanel>`.

- [ ] **Step 6: Run to verify failure**

Run: `npx vitest run src/components/AdvancedPanel.test.tsx`
Expected: new cases FAIL (no `.gr-meter-fill`).

- [ ] **Step 7: Wire the props and render the bars**

In `src/components/AdvancedPanel.tsx`:

Import: `import { grLabel, grToFill } from "../lib/gain-reduction";`

Add to `AdvancedPanel`'s props type and destructure: `liveGr?: { low: number; mid: number; high: number } | null; isPlayingMaster?: boolean;` and pass `liveGr={liveGr ?? null} isPlayingMaster={!!isPlayingMaster}` to `<PerBandCompressorCard … />`.

In `PerBandCompressorCard` props add `liveGr: { low: number; mid: number; high: number } | null; isPlayingMaster: boolean;`. Directly after the `compressor-mode-note` div, render:

```tsx
      {compressorMode !== "off" && (
        <div
          className="gr-meters"
          data-live={isPlayingMaster ? "true" : "false"}
          aria-label="Live gain reduction per band"
        >
          {(["low", "mid", "high"] as Band[]).map((band) => {
            const db = isPlayingMaster && liveGr ? liveGr[band] : undefined;
            const fill = grToFill(db);
            return (
              <div key={band} className="gr-meter" title={`${bandLabel(band)} gain reduction`}>
                <span className="gr-meter-label">{bandLabel(band)}</span>
                <span className="gr-meter-track" aria-hidden>
                  <span className="gr-meter-fill" style={{ transform: `scaleX(${fill})` }} />
                </span>
                <span className="gr-meter-value">{grLabel(db)}</span>
              </div>
            );
          })}
        </div>
      )}
```

In `src/App.tsx` at the `<AdvancedPanel` call (`:450`), add:

```tsx
              liveGr={tm.transport.compressionGr}
              isPlayingMaster={tm.transport.isPlaying && tm.transport.playbackKind === "master"}
```

- [ ] **Step 8: CSS**

Append to the ALIVE section:

```css
/* PURPOSE: COMPREHENSION — the compressor card shows what the compressor
   is DOING, per band, while the master plays. Bars grow from the right
   (reduction pulls level down); transform-only so the rail never reflows. */
.gr-meters {
  display: grid;
  gap: 0.22rem;
  margin: 0.1rem 0 0.35rem;
}
.gr-meter {
  display: grid;
  grid-template-columns: 2.4rem 1fr 3.6rem;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.66rem;
  color: var(--text-2);
}
.gr-meter-label { letter-spacing: 0.08em; text-transform: uppercase; }
.gr-meter-value { text-align: right; font-variant-numeric: tabular-nums; color: var(--text-1); }
.gr-meter-track {
  position: relative;
  height: 4px;
  border-radius: 2px;
  background: var(--surface-inset);
  box-shadow: inset 0 1px 1px rgba(0, 0, 0, 0.45);
  overflow: hidden;
}
.gr-meter-fill {
  position: absolute;
  inset: 0;
  transform-origin: right center;
  background: linear-gradient(90deg, rgba(122, 166, 255, 0.35), #7aa6ff);
  transition: transform 70ms linear;
}
.gr-meters[data-live="false"] { opacity: 0.55; }
@media (prefers-reduced-motion: reduce) {
  .gr-meter-fill { transition: none; }
}
```

Register `".gr-meter-fill"` in the delight reduced-motion list.

- [ ] **Step 9: Run tests**

Run: `npx vitest run src/components/AdvancedPanel.test.tsx src/App.delight.test.tsx src/App.warning-ownership.test.tsx`
Expected: PASS. (The warning-ownership test pins the one-owner table; GR bars are a new live readout, not a warning — it should not trip. If it does, the table needs a row: "Live per-band GR | PerBandCompressorCard | Advanced rail".)

- [ ] **Step 10: Eyeball**

Preview → Advanced → Play → Mastered. Expected: three thin bars under the mode tabs breathe with the music; on Original or stopped they sit empty and dim. Screenshot.

- [ ] **Step 11: Commit**

```bash
git add src/lib/gain-reduction.ts src/lib/gain-reduction.test.ts src/components/AdvancedPanel.tsx src/components/AdvancedPanel.test.tsx src/App.tsx src/App.css src/App.delight.test.tsx
git commit -m "feat(compressor): live per-band gain-reduction bars — the tick data was on the wire, now it's on screen"
```

---

### Task 6: View-switch and export-completion motion

Two small "moments": the Advanced rail + sidebar slide in when the console mounts (the deck already rises), and a clean export draws its check.

**Files:**
- Modify: `src/App.css` (ALIVE section)
- Modify: `src/components/ExportReceiptCard.tsx:155-158` (class hook)
- Modify: `src/App.delight.test.tsx`

**Interfaces:**
- Consumes: `.right-rail` (`App.css:3194`), `.sidebar` (the Sidebar root class — confirm with `grep -n 'className="sidebar' src/App.tsx`), `.receipt-verified-clean .receipt-verified-icon` (`ExportReceiptCard.tsx:155`), `.std-export-done-check` (`StandardView.tsx:531`).

- [ ] **Step 1: Extend delight tests (failing)**

Reduced-motion list: add `".right-rail"`, `".sidebar"`, `".receipt-verified-icon svg path"`, `".std-export-done-check"`. FORBIDDEN keyframe list: add `"rail-in"`, `"sidebar-in"`, `"check-draw"`, `"done-pop"`.

- [ ] **Step 2: Run to verify failure**

Run: `npx vitest run src/App.delight.test.tsx`
Expected: FAIL on the missing keyframes.

- [ ] **Step 3: Mark the SVG path for drawing**

In `src/components/ExportReceiptCard.tsx`, find `CircleCheckGlyph` (grep `function CircleCheckGlyph`). Give its check `<path>` a `className="receipt-check-path"` and a `pathLength={1}` attribute so CSS can draw it with unit dash values regardless of geometry:

```tsx
      <path className="receipt-check-path" pathLength={1} d="…existing d…" />
```

(Only the check stroke, not the circle.)

- [ ] **Step 4: CSS**

Append to the ALIVE section:

```css
/* PURPOSE: ORIENTATION — entering Advanced, the rail and sidebar arrive
   from their own edges while the deck rises (console-rise). One
   choreography, three surfaces; opacity + transform only. */
@keyframes rail-in    { from { opacity: 0; transform: translateX(14px); }  to { opacity: 1; transform: none; } }
@keyframes sidebar-in { from { opacity: 0; transform: translateX(-12px); } to { opacity: 1; transform: none; } }
.app:not(.app-standard) .right-rail { animation: rail-in 420ms var(--ease-out) both; animation-delay: 80ms; }
.app:not(.app-standard) .sidebar    { animation: sidebar-in 420ms var(--ease-out) both; }

/* PURPOSE: ORIENTATION — a clean export draws its check after the card has
   landed (overlay-surface-in), so "done" is a gesture, not a static glyph. */
@keyframes check-draw { from { stroke-dashoffset: 1; } to { stroke-dashoffset: 0; } }
.receipt-verified-clean .receipt-check-path {
  stroke-dasharray: 1;
  stroke-dashoffset: 1;
  animation: check-draw 420ms var(--ease-out) 220ms forwards;
}
@keyframes done-pop { 0% { transform: scale(0.6); opacity: 0; } 60% { transform: scale(1.15); opacity: 1; } 100% { transform: scale(1); opacity: 1; } }
.std-export-done-check { display: inline-block; animation: done-pop 360ms var(--ease-out) both; }

@media (prefers-reduced-motion: reduce) {
  .app:not(.app-standard) .right-rail,
  .app:not(.app-standard) .sidebar,
  .std-export-done-check { animation: none; }
  .receipt-verified-clean .receipt-check-path { animation: none; stroke-dashoffset: 0; }
  .receipt-verified-icon svg path { animation: none; }
}
```

If the Sidebar root uses a different class than `.sidebar`, use the real one in both CSS and the test list.

- [ ] **Step 5: Run tests**

Run: `npx vitest run src/App.delight.test.tsx src/components/ExportReceiptCard.test.tsx src/App.layout-css.test.ts`
Expected: PASS.

- [ ] **Step 6: Eyeball**

Preview: Standard → Advanced (rail slides in from the right, sidebar from the left, deck rises); Export a clean master (the check draws ~0.2 s after the card lands); Standard Create Master (the ✓ pops). Screenshots.

- [ ] **Step 7: Commit**

```bash
git add src/App.css src/App.delight.test.tsx src/components/ExportReceiptCard.tsx
git commit -m "design(motion): console surfaces arrive together; a clean export draws its check"
```

---

### Task 7: Pass close-out — full lanes, recapture, docs

**Files:**
- Modify: `docs/CHANGELOG.md` (new dated entry at top), `docs/APP_BEHAVIOR.md` (Track Master bullets: meters + GR + motion), `src/assets/landing/desktop-*.png` (via capture script).

- [ ] **Step 1: Full frontend + headless lanes**

Run: `npm test` → expected all green. Run: `npm run build` → green. Run: `npm run verify:headless` → green (if the landing-assets test fails with "Landing asset verification FAILED", that is the freshness gate — proceed to Step 2 then re-run).

- [ ] **Step 2: Recapture landing proof**

Run: `npm run capture:landing` then `npm run verify:landing-assets`. Expected: PASS.

- [ ] **Step 3: Docs**

`docs/CHANGELOG.md` — prepend:

```markdown
## 2026-08-19 — Alive pass 1: A/B flip moment, meter ballistics, live GR, console motion

- **A/B flip lands**: the selected side of Original/Mastered blooms once on
  flip; while Mastered is audible the waveform's played span runs brighter
  (`.app[data-playback-kind]`, presentation only — the L10 audio crossfade
  is unchanged).
- **MASTER OUT ballistics**: rate-limited fall (24 dB/s) + peak-hold pips
  (1 s hold, 12 dB/s decay; red above −1 dBFS). Pure helper
  `lib/meter-ballistics.ts`, unit-tested.
- **Live per-band gain reduction** in the Per-band Compressor card while the
  master plays (`lib/gain-reduction.ts`); the tick already carried it.
- **Console motion**: rail + sidebar arrive with the deck on entering
  Advanced; a clean export draws its check; Standard's ✓ pops. All effects
  registered in `App.delight.test.tsx` (reduced-motion opt-out, no reflow).
```

`docs/APP_BEHAVIOR.md` — in the Track Master bullet list, extend the "Live Master Out meters…" bullet with: "Meters have real ballistics: peaks fall at a controlled rate and a peak-hold pip marks the highest recent peak (red above −1 dBFS). The Advanced per-band compressor card shows live gain reduction per band while the master plays."

- [ ] **Step 4: Commit**

```bash
git add docs/CHANGELOG.md docs/APP_BEHAVIOR.md src/assets/landing
git commit -m "docs+assets: alive pass 1 changelog/app behavior; recapture landing proof"
```

---

## Self-review notes (done at plan time)

- Spec coverage vs the agreed Pass 1 list: A/B moment ✔ (T2), knob feedback — **already shipped** (wheel/Shift/double-click/hover+drag glow exist in `Knob.tsx:13-15`, `App.css:4631-4656`), so dropped deliberately; meter ballistics ✔ (T3/T4) + live GR ✔ (T5); section reveals ✔ (T6); export ceremony ✔ (T6; the "RENDERING 100%" pill already self-clears 600 ms after 1.0 — `useTrackMaster.ts:759-763` — no change needed); reduced-motion ✔ (every task).
- Type consistency: `MeterState` carries `lastMs`; `useMeterBallistics` returns `{display, hold}`; `PeakMeterBar` takes `hold?: number`; `grToFill/grLabel` names match across T5 steps; `liveGr` / `isPlayingMaster` prop names match App.tsx ↔ AdvancedPanel ↔ PerBandCompressorCard.

---

## Appendix — Passes 2–4 roadmap (agreed 2026-08-19; task-ify after Pass 1 lands)

**Pass 2 — Hierarchy, density, laptop size.** *(Executed 2026-08-19 on main — see CHANGELOG. Done: status duplication, flow-layout header, chain-strip jump links, rail advisories → hairline notes, compact compressor rows, T4 bump, neutral Overrides chip. Deliberately NOT done: sidebar track-row facts (U10 per-track facts are a deliberate design; leave), Tools/Format collapsed-by-default (Tools already collapses; Format stays visible).)* Album header: collapse the FOLLOWS ALBUM chip / Follow-Override segmented / status pill into one row at ≤1360 wide. "Checking dynamics" shown 4× on one screen → waveform + status bar only. Advanced rail: compressor band rows as a compact table, advisory box → hairline note, Tools/Format collapsed by default. Signal-chain strip: light active stages + make them scroll-to links, or remove. Sidebar track rows: one metric per row, rest to hover/Insight. Bump the T4 (9–10 px) type rung one step and cut label count.

**Pass 3 — Colour & state semantics.** *(Executed 2026-08-19 on main — see CHANGELOG. Amber kept for EQ cut (recent owner decision, token-pinned); warnings de-ambered on console chrome instead; edited dots + "edited" tile tag; REVIEW → New. Cobalt-for-live already satisfied.)* Amber = EQ cut *and* warning → separate them (warnings to ochre/red family or cut to a cool secondary). "Overrides" red → neutral chip. Modified-from-preset dots on touched controls + "edited" chip on the Styles tile. Cobalt reserved for *live* — Styles selected / Standard loudness selected move to a neutral-raised state. REVIEW pill → quiet neutral "New" dot.

**Pass 4 — Moments & onboarding.** *(Executed 2026-08-19 on main — see CHANGELOG. Demo track is synthesised by the engine (no asset/licence); shortcuts + ? overlay; Standard per-card ↺; receipt actions + mock measurements; Settings/Help + toasts verified already consistent.)* "Try with a demo track" beside Import in the empty state (bundled short CC track). Export card: verify delivered Results (LUFS/TP/DR) + visible Show file / Done / Master another actions in the real app. Standard per-card ↺. `?` keyboard-shortcut overlay (pairs with engineering: arrow seek, A/B key, L loop). Settings + Help panels consistency pass. One toast style.
