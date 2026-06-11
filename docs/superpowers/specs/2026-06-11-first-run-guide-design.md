# First-Run Guide — Design

Date: 2026-06-11
Status: Approved direction, pending owner spec review.

## Goal

Get a brand-new user to the aha-moment — hearing their own track flip between
Original and Mastered — as fast as possible, with the lightest guidance that
can do the job. No tour, no modal, no wall.

## Product decisions (owner-confirmed)

- Aha-moment first: the guide exists to produce the Original/Mastered flip,
  not to explain the UI.
- The user's own track is the demo. No bundled demo audio.
- Form: action-driven hint chips that advance only when the user performs the
  real step. Dismissible at any point.
- The guided path ends at the A-B flip, followed by a short send-off and one
  final pointer to Advanced.

## Behavior

The guide is a sequence of at most three quiet moments in Standard view:

1. **No track loaded** — no chip. The existing EmptyState ("Drop audio,
   analyze, export" + Import button) already is step one. Do not duplicate it.
2. **Analysis complete, playback still on Original, user has never played
   Mastered** — one chip anchored to the Original/Mastered A-B group in the
   Preview rail card: "Press Play, then flip to **Mastered** to hear the
   difference." A subtle pulse highlight on the Mastered button.
3. **User flips to Mastered** — the chip swaps to a send-off near the preset
   area: "That's the whole idea. Presets and Intensity shape the sound —
   explore." Auto-fades after a few seconds.
4. **After the send-off fades** — one final quiet chip anchored to the
   Standard/Advanced switch: "Need more control? Try Advanced." Shown once;
   dismisses on any interaction with it or with the chip.

After step 4 (or any dismissal), the guide is permanently done.

## Rules

- An **×** on any chip dismisses the entire guide, permanently.
- Entering Advanced view dismisses the guide silently.
- If the user plays Mastered before the step-2 chip ever appears, the guide
  marks itself done and never shows. Fast users are never lectured.
- The guide never blocks input, never dims the screen, never traps focus.
- Standard view, Track Master only. Album view never shows the guide.

## Persistence

- One localStorage key following the existing naming pattern:
  `yes-master:first-run-guide`, values `"done"` / `"dismissed"`.
- Settings gains a "Show first-run tips again" action that clears the key.

## Implementation shape

- `src/lib/first-run-guide.ts` — a pure function deriving the current step
  from inputs: `hasAnalyzedTrack`, `playbackKind`, `hasEverPlayedMastered`,
  `sendOffElapsed`, `dismissed/done`. No timers or storage inside the pure
  logic; the hook layer owns those.
- A small `HintChip` component: one chip, repositioned per step using the
  existing `seamRefs`-style ref pattern in `StandardView.tsx`.
- Wiring lives in `StandardView.tsx` only. No Rust changes. No new
  dependencies.

The step derives from app state, never a step counter, so the guide can never
desync from reality (close mid-guide, reopen a project — the right step
recomputes for free). This mirrors how `useNavigationMachine` reconciles
localStorage against in-session truth.

## Testing

- Unit tests on the step-derivation function: every transition, the
  fast-user case, the dismissed case, the entered-Advanced case.
- A `StandardView` component test confirming the chip renders at step 2,
  advances on the flip, and never reappears once done — same style as the
  existing `StandardView.test.tsx`.

## Out of scope

- Album Master onboarding.
- Bundled demo audio.
- Any export-flow guidance (export already has its own review ceremony).
- The analysis orb (separate spec: `2026-06-11-analysis-orb-design.md`).
