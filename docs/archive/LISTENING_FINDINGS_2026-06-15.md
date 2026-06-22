> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Owner listening/UX triage (L1-L15); fixes shipped on mechanical-tail. _(Status: COMPLETE.)_ See docs/CHANGELOG.md for the project ledger.

# Listening / UX Findings — 2026-06-15

Working triage list from the owner's 2026-06-15 pass. Status: **catalog + plan
only, nothing implemented.** IDs (L1…) are stable handles for later commits.

## Operating constraints (read first — these reframe the whole plan)

The owner is a solo dev; this is a side project, not a day job. Practical limits:

- **No extended / multi-session listening.** The pass below was a single window.
  Calibration must lean on **objective measurement + mechanical tests**, with at
  most a short ear spot-check per change — not a listening marathon.
- **Limited test material**: 44.1 kHz / 16-bit WAVs only, barely a DAW available.
  No purpose-mixed low-DR exports on hand. Ears are a sanity check, not ground truth.
- Therefore: **the math should suffice.** Prefer measurable fingerprints + tests
  to ear-based iteration wherever possible.

## Catalog

### Bugs / correctness
- **L1 — Can't close the app during analysis** (no X, no taskbar→close). 🔴 Critical.
  Prime suspect: the S6.8 close-guard (the only window-close handler in the app,
  present in the tested build). Either it blocks close on a stalled async handler,
  or it exposed an analysis-thread event-loop block. Revert + retest is the
  diagnostic.
- **L2 — "Landing loudness… settles in a moment" never clears** until a setting
  changes; may never actually settle to target. Standard + Advanced, long & short
  tracks. 🟠 High. Likely the root cause of L3.
- **L3 — Meters disagree with target**: at −14 the readouts "tell a different
  story"; at −9 live readouts hit near 0 dB often; a quiet high-DR track read
  hotter while an already-mastered track pinned −1.5…−3. 🟠 High.
  Preliminary read: **peak differences by material are expected** (dynamic =
  high peaks/low density; limited = consistent density). The real concern is the
  **integrated ("since play") loudness sitting hot vs the selected target**, which
  ties to L2. Needs a definitive written explanation + fix.
- **L4 — Fresh import shows an active play button + live meters** with nothing
  playing. 🟡 Medium. Stale transport/meter state on import.
- **L5 — Import doesn't auto-select the new track** (selection stays on the
  previous track). Should jump to the imported track + show its analyzing
  animation. 🟡 Medium.
- **L6 — Advanced→Standard "settings will 100% reset" warning, but Standard then
  shows the *last* Standard setting**, not a reset. 🟡 Medium. Bug-or-feature —
  needs a decision then likely a fix.
- **L14 — Receipt header reads "2 ITEM TO REVIEWS"** (should be "2 items to
  review"). 🟢 Low copy bug.
- **L15 — Advanced export via the Windows save dialog can overwrite a prior
  render** if the user accepts "replace." 🟠 High. Violates "never overwrite a
  prior render by default." The Standard "Create Master" path auto-uniquifies
  (safe); this gap is the explicit save-dialog path. (Codex catch — corrects the
  "no overwrite" item below.)

### Standard-view gaps
- **L7 — No export progress indicator in Standard.** 🟡 Medium. S6.3 was meant to
  add Standard render progress; verify it covers export, not just render.
- **L8 — Standard shows the small receipt; the full receipt only appears in
  Advanced.** 🟢 Low/info. Likely by design (Standard = simpler) — confirm intent.
- **L13 — Receipt shows "ADAPTIVE 50%" + source profile** (BRIGHT/LOW/DR/LRA/CORR).
  🟢 Low/info. Confirm whether these are shipped or a debug-flag readout; ties to
  the mode-pill label decision.

### UX / polish
- **L9 — First-run tips are embedded in the rails; should be an overlay.** 🟡 Medium.
- **L10 — Modest click on Original↔Mastered toggle; slight lag on fast toggling.**
  🟢 Low. Owner notes it's not a dealbreaker; candidate for a short crossfade.

### Product decision (owner-decided)
- **L11 — Unify preset names: Standard uses the same names as Advanced** —
  Universal / Clarity / Tape / Oomph (drop Balanced/Bright/Warm/Heavy and the
  "in Standard" subtitle). The 4 Standard presets become the **first 4** in
  Advanced. Scope: labeling + ordering + parity fixture/tests, **end-to-end**.
  Expected to be **no DSP change** (Standard already drives those same 4 presets
  under alias names) — to be confirmed.

### DSP / preset voicing
- **L12 — Presets feel characterless and indistinct;** intensity sweep
  underwhelming (Universal/Warm/Clarity = mostly loudness at 100%); Oomph has the
  most character but a narrow intensity range and still under-delivers at the top.
  Owner's read: *presets had more character previously; likely just need a bit more
  tonal curve per preset.* 🟠 High (see retuning approach below).

### Confirmed working / expected
- No *source* overwrite — Standard "Create Master" appends `__master` /
  uniquifies to `__master2`. ✅ (But see **L15**: the Advanced save-dialog path
  can still overwrite a prior *render* if the user accepts "replace".)
- 30-min track at 44.1/16 **and** 96/32-float both import, analyze, and play. ✅
- EQ knobs + visual EQ work mid-play. ✅
- The two quality warnings on the already-mastered LANDR track fire correctly. ✅

## Plan (sequenced; nothing executed)

- **Phase 0 — L1 first.** Revert/quarantine S6.8, rebuild, retest close during
  analysis. Redesign the guard to be fail-safe (never block close on a stalled
  handler; only attach during an actual export), or escalate to off-thread
  analysis if the block is pre-existing.
- **Phase 1 — Metering & landing truth (L2 + L3).** Likely one root cause.
  Establish what each readout *should* show (dynamic vs already-mastered), fix the
  not-settling integrated loudness, and write the explanation down. Trust-critical.
- **Phase 2 — Import/transport state (L4, L5).** Frontend, testable.
- **Phase 3 — Standard parity (L6, L7, L8, L14).**
- **Phase 4 — Preset-name unification (L11).** Clean, owner-decided.
- **Phase 5 — UX polish (L9, L10).**
- **Phase 6 — Preset voicing (L12).** Measurement-driven approach below.
- Existing tail (S6.6 relink, S6.7 cancel) drops below Phases 0–3.

## Measurement-driven preset retuning (adapted to the constraints)

Principle: **objective fingerprints + tests are the gate; ears are a short
spot-check.** No multi-session listening required.

1. **Quantify current character.** A measurement harness renders each preset over
   fixed signals (pink noise + the available 44/16 WAVs) and reports per-band tonal
   delta vs source, dynamics delta, width, and saturation — at intensity **0.5 and
   1.0**. This is each preset's objective "fingerprint."
2. **Define target fingerprints.** Numeric per-preset targets (LF weight for Oomph,
   HF air for Clarity, etc.), informed by measured tilts of reference masters. This
   spec becomes the test assertion.
3. **Diagnose the three levers (no ears needed):**
   - **Base tonal magnitude** — are the static EQ moves just small? (owner's
     hypothesis)
   - **Intensity scaling** — does 0.5→1.0 grow the *tonal* delta, or only loudness?
     (explains "100% felt like only loudness")
   - **Adaptive / guardrail trim** — does the source-match trim flatten the delta on
     near-target sources? (could explain "characterless" + "had more character
     previously" if trimming increased over time)
   - **Git archaeology** — diff preset tonal tables across history to test the
     "had more character previously" memory directly.
4. **Tune to hit the target fingerprint**, re-measure, and assert the new
   fingerprint via a mechanical test (the test is the gate).
5. **One short ear spot-check per change** to confirm math ≈ perception.
6. **Lock** with a snapshot regen + `LOCKED-BY-LISTENING`/`LOCKED-BY-MEASUREMENT`
   marker, one commit.

Note: this preset-voicing work (Tier-1) is distinct from the AC-5 adaptive-compressor
gate A/B; the measurement harness from step 1 serves both.

## Cross-reference
- **Canonical living plan:** `docs/plans/2026-06-15-track-master-stabilization-and-preset-listening.md`
  (Codex `/grill-me` draft + the "Reviewer Reconciliation (Claude, 2026-06-15)"
  section that merges these findings, the locked decisions, and the
  measurement-first retune gate). This findings doc is the raw catalog feeding it.

## Execution record (2026-06-15)

Shipped on `mechanical-tail`. Verified: vitest 440/440, frontend production
build, `cargo fmt --check` + `clippy -D warnings` + full `cargo test` + the slow
fixture lane (`AMS_RUN_REAL_FIXTURE=1`), plus an iPhone-bridge check.

- `ffea79e` — **L1** close-trap fixed (S6.8 guard removed); owner-confirmed.
- `759fb4f` — **L2/L3** stuck `landing_pending` flag now clears when a cached
  pending measurement is drained at the live generation (stuck-flag only — DSP
  and landing semantics unchanged; convergence verified sound, so the "hot"
  meters were the stuck flag, not a real level error). **L15** Advanced export
  can no longer overwrite a prior render: `explicit_output_path` uniquifies to a
  `__{n}` sibling and the receipt shows the real saved path.
- `81617f9` — **L4/L5** fresh import auto-selects the new track and resets stale
  transport + meter state. **L6** Back-to-Standard copy corrected (the reset
  logic already preserved style/intensity/loudness). **L7** Standard
  export-progress proven by test (was wired, unproven). **L8** "View full
  report" affordance from the lean Standard receipt into Advanced. **L11** preset
  names unified (Universal/Clarity/Tape/Oomph in both views; first four Advanced
  tiles reordered to lead with them; labels-only, internal ids stable so the
  iPhone parity fixture and any persistence are untouched). **L14** receipt
  pluralization fixed.

Confirmed correct, no change needed:
- **L13** adaptive readout — shipped/intentional, Advanced-only (not a debug flag).
- **L3 meter labels** — already distinguish target / momentary / since-play /
  live-peak (dBFS) with tooltips.
- **No source overwrite** and the **lean Standard receipt** were already correct.

Deferred follow-ups (not regressions):
- **L9** first-run tips → overlay/coachmarks (self-contained UI re-architecture).
- **L10** Original↔Mastered crossfade — touches the real-time audio path and
  needs ear validation; Wave 10.
- **L12** preset voicing/character — parked on `preset-bold-experiment`, gated by
  the measurement harness per the plan; Wave 10.

### Adversarial review pass (2026-06-15)

Six independent skeptics tried to break each committed fix. Four survived clean
(L15, L11, L7/L8, L6/L14 — including an exhaustive search confirming the
`StandardStyleId` ids are never persisted). Two real issues were found and fixed:

- `70ca1ca` — **L2 gain promotion.** The drain-branch flag-clear was incomplete:
  it cleared `landing_pending` without promoting the cached gain to the live
  chain, so on the "wiggle away then back while the worker is still in flight"
  path the UI said "landing complete" while the audio stayed on the last-known
  scalar until the next `UpdateChain`. The drain branch now promotes the cached
  gain (via a `LiveCoeffUpdate` crossfade) exactly like the worker-completion
  branch. (Resolves the watch item.)
- `676c567` — **L4/L5 stale-tick race.** `importFiles` set `selectedTrackId`, but
  the tick guard's ref (`selectedTrackIdRef`) only synced on the next render, so
  a late tick for the previous track could repaint the old playing state before
  React committed. The ref is now synced synchronously on import; a regression
  assertion covers the guard rejecting a post-import old-track tick.

All lanes re-run green after both fixes (vitest 440, cargo full + slow fixture,
iPhone bridge check).
