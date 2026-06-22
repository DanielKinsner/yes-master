# Handoff — 2026-06-02 — Adaptive DSP (Tier-1 defensive guardrails)

> **⚠️ Superseded for current state.** The post-review fixes and the self-contained
> "review-then-code" guide are in **`docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2.md`** —
> read that to start coding. This doc is the original build record; some counts/notes
> below predate the fix stack.

Audience: the owner returning to listen, or the next agent. Pointer doc — the
full design is in `docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md`.

## What shipped

Branch: `feat/adaptive-dsp-guardrails` (not merged; awaiting the listening gate).
Built in small commits: spec → plumbing → trim math + wiring → injection →
behavioral tests → Adapt Strength UI → spec reconcile.

The feature: **presets now fit the track.** When a source already has a quality,
the matching preset move is trimmed toward neutral:

- already-bright → less air/high lift
- already-boomy → less low/sub lift
- already-dense → softer compression
- already-wide → less widening

It is **defensive only** — it can reduce a preset's move toward flat, never add a
move, flip a sign, touch a preset cut, or narrow a source. Per-axis caps (EQ 50%,
compression 60%, width 70%) and a +0.5 dB character floor keep presets
recognizable at any strength.

## Post-review update (2026-06-02)

Two independent reviews (`docs/archive/reviews/2026-06-02-adaptive-dsp-tier1-review.md`,
`docs/archive/reviews/2026-06-03-adaptive-dsp-desktop-review.md`) were verified against
the code by a fan-out workflow, and their real bugs fixed. Fixed: the `LRA=0.0`
density misfire (`f6c7cc8`); the preview + slow-lane wiring gaps that made the
evidence lane render the *old* chain (`dc62dab`, + invariant tests `27f2a4c`);
the bright deadband over-trimming genuinely-neutral masters (raised 0.20 → 0.30,
`d6519b7`); doc drift + default-strength dedup (`44a44ef`). Shipped
(owner-approved before the listening session): the per-axis "what was trimmed"
readout (`f112f1f`) and an honest "adaptive applies to Track Master" album label
(`907b1f4`).

Remaining **taste / new-math / architecture** work — the principled deadband
tilt-metric, density-cap semantics, whole-track Welch window, `stereo_width`
co-trigger, the backend-owned-profile refactor, and the Tier-2 measured-neutral +
PSR closed loop — is planned with options + recommendations in
`docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`. The owner
chose: quick deadband bump (done), build the readout (done), defer the refactor,
keep album unadapted (done).

**The status-tagged backlog / single entry point for what's left (Tier-1 finish,
the deferred refactor, Tier-2, and the parallel Simple Mode view) is
`docs/ADAPTIVE_DSP_NEXT_STEPS.md` — start there.**

## Default behavior

- **On by default at strength 0.6**, the moment a track is analyzed (the source
  profile is injected on the live chain and on export).
- **Strength 0, or no analysis, = exactly the old sound** — byte-identical. The
  `preset_byte_identity` SHA snapshots prove the non-adaptive path is untouched.
- Export applies the same guardrails as audition (what you hear is what you get),
  and never inherits Volume Match.

## How to listen / tune (your gate)

The numbers are deliberately provisional starting points — calibrating them is
the listening job:

1. Load already-mastered / AI / hot sources (the stress class) **and** a few
   neutral mixes.
2. A/B per track: set **Advanced → Adapt strength** to **Off** (preset as
   designed) vs **60%** (adapted). Use **Volume Match** for a level-matched
   compare.
3. Sweep Adapt strength live while auditioning — find the level that fixes the
   over-cooked cases **without flattening** the good ones. On a neutral mix the
   guardrails should do little/nothing (that's the deadband working).
4. When you settle on numbers, they all live in **one file**:
   `src-tauri/src/guardrails.rs` (deadbands, full-trim points, per-axis caps,
   default strength). Edit there, then `cd src-tauri && cargo test --lib`.

## Deferred (not in v1 — deliberate)

- Auto Compressor `Off` at extreme density (v1 caps softening at 60% instead;
  the mode-flip was flagged as surprising by the research fact-check).
- "What was trimmed and why" transparency readout (needs the computed trims
  surfaced to the UI — the natural next increment).
- Per-dimension strength (v1 is one global dial).
- Calibrating the neutral 6-band references from your own reference masters
  (provisional defaults shipped; see the spec's "Calibration plan").
- Tier-2 corrective (push toward a target curve), genre detection, reference-
  track upload.
- Album Master guardrails (Album uses its own per-character bias; out of scope).

## Pre-merge gates remaining (owner-side)

- **Listening signoff** — only a human can clear taste.
- **Slow fixture lane** with your private audio (those tests are *ignored*
  without fixtures on this machine):
  ```powershell
  cd src-tauri
  $env:AMS_RUN_REAL_FIXTURE = "1"
  cargo test
  Remove-Item Env:\AMS_RUN_REAL_FIXTURE
  ```

## Verification done this session

- `npm test` — 158 passed (incl. injection + Adapt Strength control tests).
- `npm run build` — clean (tsc + vite).
- `cargo test --lib` — 224 passed (12 new guardrail unit/behavioral tests;
  all `preset_byte_identity` snapshots green).
- `cargo test` (full integration) — green; only private-fixture tests ignored.
- `npm run build:windows` — green; MSI + NSIS installers produced.
- `cargo clippy --all-targets -- -D warnings` — clean.

## Key files

- `src-tauri/src/guardrails.rs` — trim math + **all tuning constants** (start here
  to retune).
- `src-tauri/src/dsp.rs` → `ChainCoeffs::from_settings` — the three hook points
  (EQ contribution, compression density, preset width).
- `src-tauri/src/types.rs` — `SourceProfile` + `AdvancedSettings.adaptive_strength`.
- `src/lib/settings-transitions.ts` — profile injection (`applyChainDispatchOverrides`
  for live, `injectSourceProfile` for export).
- `src/App.tsx` → `AdvancedControlsCard` — the Adapt Strength control.
- `docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md` — the spec, with
  citations and the honest sourced-vs-inferred caveats.
