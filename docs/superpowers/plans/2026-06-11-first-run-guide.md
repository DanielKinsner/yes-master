# First-Run Guide Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Action-driven first-run hint chips in Standard view that walk a new user to the Original→Mastered aha-moment, then a send-off and one Advanced pointer.

**Architecture:** A pure step-derivation function (`src/lib/first-run-guide.ts`) + a small stateful hook (`src/hooks/useFirstRunGuide.ts`) + one presentational `HintChip` component, wired only into `StandardView.tsx`. One localStorage key gates whether the guide ever starts; everything else derives from live app state. No Rust changes, no new dependencies.

**Tech Stack:** React 19, TypeScript, vitest + jsdom (existing harness patterns from `StandardView.test.tsx`).

**Spec:** `docs/superpowers/specs/2026-06-11-first-run-guide-design.md`

Key codebase facts (verified 2026-06-11):

- `useTrackMaster` transport defaults `playbackKind: "source"` (`useTrackMaster.ts:239`).
- The A-B Original/Mastered buttons live in `StandardRightRail` inside `StandardView.tsx` (`.std-rail-ab`).
- Standard's only Advanced affordances: the top-chrome `top-advanced` button (`App.tsx:364`, handler at `App.tsx:93`) and the rail's Delivery "Change" button (`onEnterAdvanced` prop).
- Settings dialog: `SettingsPanel` in `App.tsx:426`.
- Existing localStorage key pattern: `yes-master:view-mode`, `yes-master:last-track-export-dir`.
- Test harness: manual `createRoot` + `act`, `fakeTm(overrides)` factory in `StandardView.test.tsx`. NOTE: `fakeTm` defaults `transport.playbackKind: "master"`, which means the guide silently self-completes in all existing tests (pre-flipped rule) — existing tests stay green by design.

---

### Task 1: Pure step derivation + storage helpers

**Files:**
- Create: `src/lib/first-run-guide.ts`
- Test: `src/lib/first-run-guide.test.ts`

- [ ] **Step 1: Write the failing tests**

```ts
// src/lib/first-run-guide.test.ts
import { describe, expect, it } from "vitest";
import {
  FIRST_RUN_GUIDE_KEY,
  deriveGuideStep,
  guideAlreadyFinished,
  markGuideFinished,
  resetGuide,
} from "./first-run-guide";

const base = {
  started: true,
  hasAnalyzedTrack: true,
  flippedToMastered: false,
  sendOffElapsed: false,
  advancedDone: false,
};

describe("deriveGuideStep", () => {
  it("is null before the guide starts or before analysis", () => {
    expect(deriveGuideStep({ ...base, started: false })).toBeNull();
    expect(deriveGuideStep({ ...base, hasAnalyzedTrack: false })).toBeNull();
  });

  it("asks for the flip once a track is analyzed", () => {
    expect(deriveGuideStep(base)).toBe("flip");
  });

  it("moves to the send-off after the flip", () => {
    expect(deriveGuideStep({ ...base, flippedToMastered: true })).toBe("sendoff");
  });

  it("moves to the Advanced pointer after the send-off window", () => {
    expect(
      deriveGuideStep({ ...base, flippedToMastered: true, sendOffElapsed: true }),
    ).toBe("advanced");
  });

  it("ends after the Advanced pointer is done", () => {
    expect(
      deriveGuideStep({
        ...base,
        flippedToMastered: true,
        sendOffElapsed: true,
        advancedDone: true,
      }),
    ).toBeNull();
  });
});

function fakeStorage(): Storage {
  const map = new Map<string, string>();
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    removeItem: (k: string) => void map.delete(k),
  } as Storage;
}

describe("guide storage", () => {
  it("is unfinished on a fresh machine and finished after done or dismissed", () => {
    const s = fakeStorage();
    expect(guideAlreadyFinished(s)).toBe(false);
    markGuideFinished(s, "done");
    expect(guideAlreadyFinished(s)).toBe(true);
    expect(s.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
    markGuideFinished(s, "dismissed");
    expect(s.getItem(FIRST_RUN_GUIDE_KEY)).toBe("dismissed");
    expect(guideAlreadyFinished(s)).toBe(true);
  });

  it("reset clears the flag so the guide can run again", () => {
    const s = fakeStorage();
    markGuideFinished(s, "done");
    resetGuide(s);
    expect(guideAlreadyFinished(s)).toBe(false);
  });

  it("tolerates a missing storage object", () => {
    expect(guideAlreadyFinished(undefined)).toBe(false);
    expect(() => markGuideFinished(undefined, "done")).not.toThrow();
    expect(() => resetGuide(undefined)).not.toThrow();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/lib/first-run-guide.test.ts`
Expected: FAIL — cannot resolve `./first-run-guide`.

- [ ] **Step 3: Write the implementation**

```ts
// src/lib/first-run-guide.ts
//
// First-run guide: pure step machine for the Standard-view hint chips.
// The guide exists to produce one aha-moment — flipping Original→Mastered
// on the user's own track — then a short send-off and a single Advanced
// pointer. Storage gates whether the guide ever STARTS; every step derives
// from live app state (never a step counter), so the guide cannot desync.

export const FIRST_RUN_GUIDE_KEY = "yes-master:first-run-guide";

export type GuideStep = "flip" | "sendoff" | "advanced";

export type GuideInputs = {
  /// False when storage said done/dismissed at mount (or the user
  /// pre-flipped before the guide could ever appear).
  started: boolean;
  hasAnalyzedTrack: boolean;
  /// playbackKind has been "master" at some point since the guide started.
  flippedToMastered: boolean;
  /// The send-off chip finished its display window.
  sendOffElapsed: boolean;
  /// The Advanced pointer was dismissed or Advanced was entered.
  advancedDone: boolean;
};

export function deriveGuideStep(i: GuideInputs): GuideStep | null {
  if (!i.started) return null;
  if (!i.hasAnalyzedTrack) return null;
  if (!i.flippedToMastered) return "flip";
  if (!i.sendOffElapsed) return "sendoff";
  if (!i.advancedDone) return "advanced";
  return null;
}

type GuideStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

export function guideAlreadyFinished(storage: GuideStorage | undefined): boolean {
  const v = storage?.getItem(FIRST_RUN_GUIDE_KEY);
  return v === "done" || v === "dismissed";
}

export function markGuideFinished(
  storage: GuideStorage | undefined,
  how: "done" | "dismissed",
): void {
  storage?.setItem(FIRST_RUN_GUIDE_KEY, how);
}

export function resetGuide(storage: GuideStorage | undefined): void {
  storage?.removeItem(FIRST_RUN_GUIDE_KEY);
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run src/lib/first-run-guide.test.ts`
Expected: PASS (8 tests).

- [ ] **Step 5: Commit**

```bash
git add src/lib/first-run-guide.ts src/lib/first-run-guide.test.ts
git commit -m "feat(guide): pure first-run step derivation + storage helpers"
```

---

### Task 2: HintChip component + styles

**Files:**
- Create: `src/components/HintChip.tsx`
- Modify: `src/App.css` (append)
- Test: `src/components/HintChip.test.tsx`

- [ ] **Step 1: Write the failing test**

```tsx
// src/components/HintChip.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { HintChip } from "./HintChip";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => { document.body.innerHTML = ""; });

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("HintChip", () => {
  it("renders its copy as a polite status", async () => {
    const { container, root } = await render(
      <HintChip onDismiss={() => {}}>Flip to Mastered</HintChip>,
    );
    const chip = container.querySelector(".hint-chip");
    expect(chip?.getAttribute("role")).toBe("status");
    expect(chip?.textContent).toContain("Flip to Mastered");
    await act(async () => root.unmount());
  });

  it("routes the × to onDismiss", async () => {
    const onDismiss = vi.fn();
    const { container, root } = await render(
      <HintChip onDismiss={onDismiss}>hi</HintChip>,
    );
    const x = container.querySelector<HTMLButtonElement>(".hint-chip-x")!;
    await act(async () => { x.click(); });
    expect(onDismiss).toHaveBeenCalledOnce();
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run src/components/HintChip.test.tsx`
Expected: FAIL — cannot resolve `./HintChip`.

- [ ] **Step 3: Write the component**

```tsx
// src/components/HintChip.tsx
//
// One quiet first-run hint. Never blocks input, never dims the screen,
// never traps focus — a small floating note with a dismiss ×.

import type { ReactNode } from "react";

export function HintChip({
  children,
  onDismiss,
  className = "",
}: {
  children: ReactNode;
  onDismiss: () => void;
  className?: string;
}) {
  return (
    <div className={`hint-chip ${className}`.trim()} role="status">
      <span className="hint-chip-text">{children}</span>
      <button
        type="button"
        className="hint-chip-x"
        aria-label="Dismiss first-run tips"
        onClick={onDismiss}
      >
        ×
      </button>
    </div>
  );
}
```

- [ ] **Step 4: Append chip + pulse styles to `src/App.css`**

```css
/* First-run guide: quiet hint chips + the Mastered-button pulse. */
.hint-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  padding: 7px 10px;
  border: 1px solid rgba(111, 163, 255, 0.35);
  border-radius: 8px;
  background: rgba(42, 107, 242, 0.12);
  color: #b9cdf2;
  font-size: 12px;
  line-height: 1.4;
  animation: hint-chip-in 240ms ease-out;
}
.hint-chip strong { color: #e3edff; font-weight: 600; }
.hint-chip-text { flex: 1; }
.hint-chip-x {
  flex: none;
  border: 0;
  background: transparent;
  color: #7d8aa3;
  font-size: 14px;
  line-height: 1;
  padding: 2px 4px;
  cursor: pointer;
}
.hint-chip-x:hover { color: #d7e2f5; }
@keyframes hint-chip-in {
  from { opacity: 0; transform: translateY(3px); }
  to { opacity: 1; transform: translateY(0); }
}
.std-rail-ab button.guide-pulse {
  animation: guide-pulse 1.6s ease-in-out infinite;
}
@keyframes guide-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(111, 163, 255, 0); }
  50% { box-shadow: 0 0 0 4px rgba(111, 163, 255, 0.25); }
}
@media (prefers-reduced-motion: reduce) {
  .hint-chip { animation: none; }
  .std-rail-ab button.guide-pulse { animation: none; }
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npx vitest run src/components/HintChip.test.tsx`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add src/components/HintChip.tsx src/components/HintChip.test.tsx src/App.css
git commit -m "feat(guide): HintChip component + chip/pulse styles"
```

---

### Task 3: useFirstRunGuide hook + flip step wiring in StandardView

**Files:**
- Create: `src/hooks/useFirstRunGuide.ts`
- Modify: `src/components/StandardView.tsx`
- Test: `src/components/StandardView.test.tsx` (new describe block)

- [ ] **Step 1: Write the failing tests** (append to `StandardView.test.tsx`; reuse its `render`/`fakeTm` helpers; import `FIRST_RUN_GUIDE_KEY` from `../lib/first-run-guide`)

```tsx
describe("first-run guide", () => {
  afterEach(() => { globalThis.localStorage?.removeItem(FIRST_RUN_GUIDE_KEY); });

  function freshTm(overrides: Partial<TM> = {}): TM {
    return fakeTm({
      transport: { isPlaying: false, currentTimeSec: 0, playbackKind: "source", volumeMatch: false },
      ...overrides,
    } as Partial<TM>);
  }

  it("shows the flip chip and pulses Mastered once a track is analyzed on Original", async () => {
    const { container, root } = await render(
      <StandardView tm={freshTm()} onEnterAdvanced={() => {}} />,
    );
    expect(container.querySelector(".hint-chip-flip")?.textContent).toContain("Mastered");
    const mastered = Array.from(
      container.querySelectorAll<HTMLButtonElement>(".std-rail-ab button"),
    ).find((b) => b.textContent === "Mastered")!;
    expect(mastered.classList.contains("guide-pulse")).toBe(true);
    await act(async () => root.unmount());
  });

  it("never shows when storage says the guide already finished", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const { container, root } = await render(
      <StandardView tm={freshTm()} onEnterAdvanced={() => {}} />,
    );
    expect(container.querySelector(".hint-chip")).toBeNull();
    await act(async () => root.unmount());
  });

  it("never shows for a fast user already on Mastered (and self-finishes)", async () => {
    const { container, root } = await render(
      <StandardView tm={fakeTm()} onEnterAdvanced={() => {}} />,
    );
    expect(container.querySelector(".hint-chip")).toBeNull();
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
    await act(async () => root.unmount());
  });

  it("× dismisses the guide permanently", async () => {
    const { container, root } = await render(
      <StandardView tm={freshTm()} onEnterAdvanced={() => {}} />,
    );
    const x = container.querySelector<HTMLButtonElement>(".hint-chip-x")!;
    await act(async () => { x.click(); });
    expect(container.querySelector(".hint-chip")).toBeNull();
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("dismissed");
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/components/StandardView.test.tsx`
Expected: the four new tests FAIL (no `.hint-chip-flip` rendered); all pre-existing tests still PASS.

- [ ] **Step 3: Write the hook**

```ts
// src/hooks/useFirstRunGuide.ts
//
// Stateful shell around the pure deriveGuideStep machine. Owns the
// localStorage gate, the "fast user" silent finish, the aha persistence,
// and the send-off display window. Rendering lives in StandardView.

import { useEffect, useRef, useState } from "react";
import {
  deriveGuideStep,
  guideAlreadyFinished,
  markGuideFinished,
  type GuideStep,
} from "../lib/first-run-guide";

const SEND_OFF_MS = 6000;

export type FirstRunGuide = {
  step: GuideStep | null;
  /// × on any chip: ends the whole guide, persisted as "dismissed".
  dismiss: () => void;
  /// Any route into Advanced ends the guide silently as "done".
  noteEnteredAdvanced: () => void;
};

export function useFirstRunGuide(args: {
  hasAnalyzedTrack: boolean;
  playbackKind: string;
}): FirstRunGuide {
  const storage = globalThis.localStorage;
  // Snapshot once at mount: a returning user never re-enters the guide.
  const [started, setStarted] = useState(() => !guideAlreadyFinished(storage));
  const [flipped, setFlipped] = useState(false);
  const [sendOffElapsed, setSendOffElapsed] = useState(false);
  const [advancedDone, setAdvancedDone] = useState(false);
  // True once the flip chip has actually been on screen. Distinguishes a
  // guided flip from a fast user who reached Mastered before the guide
  // could appear — those users are never lectured (silent finish).
  const chipWasVisible = useRef(false);

  const rawStep = deriveGuideStep({
    started,
    hasAnalyzedTrack: args.hasAnalyzedTrack,
    flippedToMastered: flipped,
    sendOffElapsed,
    advancedDone,
  });
  // Never render the flip chip while playback is ALREADY on master — that
  // state is either a fast user about to be silently finished by the effect
  // below, or the one-frame gap before `flipped` flows through. Both would
  // otherwise flash the chip.
  const step = rawStep === "flip" && args.playbackKind === "master" ? null : rawStep;
  if (step === "flip") chipWasVisible.current = true;

  useEffect(() => {
    if (args.playbackKind !== "master" || !started) return;
    if (!chipWasVisible.current) {
      markGuideFinished(storage, "done");
      setStarted(false);
      return;
    }
    if (!flipped) {
      // The aha happened. Persist immediately — the send-off and Advanced
      // pointer are session-only from here.
      markGuideFinished(storage, "done");
      setFlipped(true);
    }
  }, [args.playbackKind, started, flipped, storage]);

  useEffect(() => {
    if (step !== "sendoff") return;
    const t = setTimeout(() => setSendOffElapsed(true), SEND_OFF_MS);
    return () => clearTimeout(t);
  }, [step]);

  const dismiss = () => {
    markGuideFinished(storage, "dismissed");
    setStarted(false);
    setAdvancedDone(true);
  };

  const noteEnteredAdvanced = () => {
    if (started) markGuideFinished(storage, "done");
    setStarted(false);
    setAdvancedDone(true);
  };

  return { step, dismiss, noteEnteredAdvanced };
}
```

- [ ] **Step 4: Wire the flip step into StandardView**

In `src/components/StandardView.tsx`:

1. Add imports:

```ts
import { HintChip } from "./HintChip";
import { useFirstRunGuide, type FirstRunGuide } from "../hooks/useFirstRunGuide";
```

2. In `StandardView`, after `const s = tm.selectedSettings;`:

```ts
const guide = useFirstRunGuide({
  hasAnalyzedTrack: tm.selectedAnalysis != null,
  playbackKind: tm.transport.playbackKind,
});
```

3. Thread `guide` into the rail: change `StandardRightRail`'s props to accept `guide: FirstRunGuide` and pass it at the call site (`<StandardRightRail tm={tm} guide={guide} ... />`).

4. In `StandardRightRail`, give the Mastered button the pulse class and render the chip after the `.std-rail-ab` group:

```tsx
<button
  type="button"
  aria-pressed={tm.transport.playbackKind === "master"}
  className={
    (tm.transport.playbackKind === "master" ? "on" : "") +
    (guide.step === "flip" ? " guide-pulse" : "")
  }
  onClick={() => tm.setPlaybackKind("master")}
>
  Mastered
</button>
```

```tsx
{guide.step === "flip" && (
  <HintChip className="hint-chip-flip" onDismiss={guide.dismiss}>
    Press Play, then flip to <strong>Mastered</strong> to hear the difference.
  </HintChip>
)}
```

(placed immediately after the closing `</div>` of `.std-rail-ab`.)

- [ ] **Step 5: Run tests to verify they pass**

Run: `npx vitest run src/components/StandardView.test.tsx`
Expected: PASS, including all pre-existing tests.

- [ ] **Step 6: Commit**

```bash
git add src/hooks/useFirstRunGuide.ts src/components/StandardView.tsx src/components/StandardView.test.tsx
git commit -m "feat(guide): flip-to-Mastered hint chip + pulse in Standard view"
```

---

### Task 4: Send-off + Advanced pointer steps

**Files:**
- Modify: `src/components/StandardView.tsx`
- Modify: `src/App.tsx` (top-chrome Advanced handler marks the guide)
- Test: `src/components/StandardView.test.tsx`

- [ ] **Step 1: Write the failing tests** (append inside the `first-run guide` describe)

```tsx
it("flip → send-off chip; after the window, the Advanced pointer", async () => {
  vi.useFakeTimers();
  try {
    const { container, root } = await render(
      <StandardView tm={freshTm()} onEnterAdvanced={() => {}} />,
    );
    expect(container.querySelector(".hint-chip-flip")).not.toBeNull();
    await act(async () => {
      root.render(
        <StandardView
          tm={freshTm({
            transport: { isPlaying: true, currentTimeSec: 3, playbackKind: "master", volumeMatch: false },
          } as Partial<TM>)}
          onEnterAdvanced={() => {}}
        />,
      );
    });
    expect(container.querySelector(".hint-chip-sendoff")?.textContent).toContain("Presets and Intensity");
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
    await act(async () => { vi.advanceTimersByTime(6000); });
    expect(container.querySelector(".hint-chip-sendoff")).toBeNull();
    expect(container.querySelector(".hint-chip-advanced")?.textContent).toContain("Advanced");
    await act(async () => root.unmount());
  } finally {
    vi.useRealTimers();
  }
});

it("the rail Change button still enters Advanced and ends the guide", async () => {
  const onEnterAdvanced = vi.fn();
  const { container, root } = await render(
    <StandardView tm={freshTm()} onEnterAdvanced={onEnterAdvanced} />,
  );
  const change = container.querySelector<HTMLButtonElement>(".std-delivery-change")!;
  await act(async () => { change.click(); });
  expect(onEnterAdvanced).toHaveBeenCalledOnce();
  expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
  await act(async () => root.unmount());
});
```

NOTE for the first test: re-rendering the same root with the same component type keeps the hook instance, so the in-session flip state carries across the rerender. This mirrors how `App.transitions.test.tsx` re-renders.

- [ ] **Step 2: Run tests to verify they fail**

Run: `npx vitest run src/components/StandardView.test.tsx`
Expected: the two new tests FAIL.

- [ ] **Step 3: Wire send-off + Advanced chips and the enter-Advanced hooks**

In `StandardView` (center column):

1. Send-off chip — insert immediately BEFORE `<div className="std-steps">`:

```tsx
{guide.step === "sendoff" && (
  <HintChip className="hint-chip-sendoff" onDismiss={guide.dismiss}>
    That's the whole idea. Presets and Intensity shape the sound — explore.
  </HintChip>
)}
```

2. Advanced pointer chip — insert immediately AFTER the `.std-hero-head` div:

```tsx
{guide.step === "advanced" && (
  <HintChip className="hint-chip-advanced" onDismiss={guide.dismiss}>
    Need more control? Try <strong>Advanced</strong> — top right.
  </HintChip>
)}
```

3. Wrap Advanced entry inside StandardView so the rail's Change button ends the guide:

```ts
const enterAdvanced = () => {
  guide.noteEnteredAdvanced();
  onEnterAdvanced();
};
```

and pass `onEnterAdvanced={enterAdvanced}` to `StandardRightRail` instead of the raw prop.

In `src/App.tsx`, the top-chrome handler (line ~93, `onEnterAdvanced={() => setView("advanced")}`) — entering Advanced from the chrome must also end the guide because StandardView unmounts without notice:

```ts
import { markGuideFinished } from "./lib/first-run-guide";
```

```ts
onEnterAdvanced={() => {
  markGuideFinished(globalThis.localStorage, "done");
  setView("advanced");
}}
```

(Only the handler used while `view === "standard"`; Advanced/Album chrome paths don't need it but are harmless if shared.)

- [ ] **Step 4: Run the full frontend suite**

Run: `npx vitest run`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/StandardView.tsx src/App.tsx src/components/StandardView.test.tsx
git commit -m "feat(guide): send-off + Advanced pointer; entering Advanced ends the guide"
```

---

### Task 5: Settings reset action

**Files:**
- Modify: `src/App.tsx` (`SettingsPanel`, line ~426)
- Test: `src/App.transitions.test.tsx` style is heavyweight; test `SettingsPanel` directly in a new file `src/App.settings-reset.test.tsx`

- [ ] **Step 1: Write the failing test**

```tsx
// src/App.settings-reset.test.tsx
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import { SettingsPanel } from "./App";
import { FIRST_RUN_GUIDE_KEY } from "./lib/first-run-guide";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
afterEach(() => {
  document.body.innerHTML = "";
  globalThis.localStorage?.removeItem(FIRST_RUN_GUIDE_KEY);
});

describe("SettingsPanel first-run tips reset", () => {
  it("clears the guide flag", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => { root.render(<SettingsPanel onClose={() => {}} />); });
    const btn = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((b) => b.textContent?.includes("Show first-run tips again"))!;
    await act(async () => { btn.click(); });
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBeNull();
    await act(async () => root.unmount());
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run src/App.settings-reset.test.tsx`
Expected: FAIL — no such button.

- [ ] **Step 3: Add the reset row to SettingsPanel**

In `App.tsx`, import `resetGuide` from `./lib/first-run-guide` (merge with the Task 4 import) and add inside the `settings-grid` div, after the mapped groups:

```tsx
<div className="settings-actions">
  <button
    type="button"
    className="ghost-btn"
    onClick={() => resetGuide(globalThis.localStorage)}
  >
    Show first-run tips again
  </button>
</div>
```

Append to `src/App.css`:

```css
.settings-actions { margin-top: 12px; }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npx vitest run src/App.settings-reset.test.tsx`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx src/App.css src/App.settings-reset.test.tsx
git commit -m "feat(guide): Settings action to show first-run tips again"
```

---

### Task 6: Full verification + push

- [ ] **Step 1: Run the frontend verify lane**

Run: `npm test` then `npm run build`
Expected: all tests pass; tsc + vite build clean.

- [ ] **Step 2: Push**

```bash
git push
```

---

## Out of scope (per spec)

- Album view, Advanced view chips, bundled demo audio, export-flow guidance.
- The analysis orb (separate plan).
