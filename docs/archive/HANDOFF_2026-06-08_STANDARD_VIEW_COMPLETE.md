# Handoff — Standard View shipped (2026-06-08)

**For the next agent picking up `yes-master`.** Self-contained current state, what's done, what's deferred, and what's the owner's (Dan's) call. Read this first.

---

## TL;DR

**Standard View — the new default desktop UI — is implemented, fully tested, reviewed, and merged to `main`** (merge commit `238b1c9`, branch `feat/standard-view`, base `9acff95`). It's a stripped, opinionated front-of-house for the *already-validated* preset+adaptive engine: pick a sound, YES Master fits it to the track. **Advanced** (the existing full desk) is now a door you step through.

- Design spec: `docs/superpowers/specs/2026-06-08-simple-mode-design.md`
- Implementation plan (v2, the build's source of truth): `docs/superpowers/plans/2026-06-08-standard-view.md`
- Visual mockup of record: `docs/simple-mode-mockup.html`
- Engine reference (presets + adaptive math): `docs/ENGINE_REFERENCE.md`

**Everything green:** frontend 247 vitest (incl. a real-`<App/>` end-to-end transition suite) + `npm run build`; desktop Rust `fmt`/`clippy -D warnings`/598 tests; iPhone bridge 34 tests; `tsc --noEmit` clean.

---

## What shipped

A new **Standard** view chosen by default for new users; **Advanced** for power users (and returning users, via migration). The two share **one settings truth** — Standard binds to the same per-track `selectedSettings` an Advanced user edits; nothing is Standard-only.

- **Four reference-tuned style tiles** — Balanced / Bright / Warm / Heavy → internal presets Universal / Clarity / Tape / Oomph. (Oomph kept 1:1, owner-ear-confirmed.)
- **Intensity** knob (0–1) and **Loudness** segmented control (Low / Medium / High → −14 / −11 / −9 LUFS).
- **WYSIWYG Mastered audition** — what you hear == what Create Master exports (preset + adaptive + loudness landing + limiter), driven by an internal flag (see Invariants).
- **Asymmetric Standard↔Advanced transition** — "Take control" seeds Advanced from the current Standard choice; "Back to Standard" returns silently if clean, else warns + offers Save-as-preset / Reset & continue. Standard holds an **always-clean invariant**.
- **Fixed-format export** — 44.1 kHz / 24-bit WAV at −1 dBTP, mirroring the iPhone's fixed recipe; no blocking review gate in Standard (cosmetic warnings suppressed; one tiny non-blocking integrity note; technical hard-stops surfaced as "saved but invalid — re-render").
- **iPhone parity** — `NativeStylePreset` renamed to the reference-4 and the Rust bridge `native_preset` re-pointed; desktop and iPhone now map the four styles **identically** (test-backed), with back-compat aliases for old payloads.

### New / changed files

| File | Role |
|---|---|
| `src/lib/standard-mapping.ts` | reference-4 + loudness mappings, tile metadata (the FE source of truth, mirrors the iPhone bridge) |
| `src/lib/standard-managed.ts` | `hasNonManagedEdits` / `resetToStandardManaged` / `shouldForceAdvancedOnStandardEntry` (the always-clean model) |
| `src/lib/standard-export.ts` | `standardExportSettings` (fixed-format wrap) + `standardExportNotes` (stripped-ceremony classification) |
| `src/lib/view-mode.ts` | `resolveInitialViewMode` + localStorage store (versioned migration) |
| `src/hooks/useViewMode.ts` | the `standard \| advanced \| null` hook (migration-aware, no-flash) |
| `src/components/StandardView.tsx` | the Option-B layout + `StyleTiles` + `LoudnessSegmented` + `StandardExportButton` |
| `src/components/Waveform.tsx` | `WaveformView` / `WaveformLoading` extracted from `App.tsx` (broke an import cycle; `App.tsx` re-exports) |
| `src/hooks/useTrackMaster.ts` | + `hadPriorSession`, `resetToStandardManaged`, `exportStandardMaster` (via extracted `runExport`), internal WYSIWYG flag (`setForceWysiwyg`/`effectivePreviewLanding`), async-safe `saveUserPreset`→`Promise<boolean>` |
| `src/App.tsx` | wires `useViewMode`, the chrome affordance, the transition, the entry guard, the return-confirm modal, the body branch |
| `src/App.transitions.test.tsx` | **E2E**: mounts the real `<App/>` and drives all 8 transitions deterministically |
| `apps/iphone-native/.../ContentView.swift`, `apps/iphone-native/rust/src/lib.rs` | reference-4 rename + bridge remap |
| `docs/PRODUCT.md` | canon: the Standard export ceremony |

---

## Key invariants (don't break these)

1. **One settings truth.** `StandardView` binds to `tm.selectedSettings` + the hook's setters — never a global. A Standard choice writes the *same* `MasteringSettings` an Advanced user would build.
2. **Always-clean Standard.** Standard never silently renders a track carrying hidden Advanced edits. Enforced at **every** entry (open project / session restore / track switch / album) by the `shouldForceAdvancedOnStandardEntry` guard in `App.tsx` (bounces to Advanced), and on the return door by the warn+reset modal. If you add a new way to reach Standard, route it through that guard.
3. **Internal WYSIWYG flag.** Standard forces the loudness-landing in the live audition via `forceWysiwygRef` / `effectivePreviewLanding()` — it must **never** mutate the user-facing Advanced `Preview LUFS` toggle (`transport.exportLufsPreview`). Every live-chain landing read goes through `effectivePreviewLanding()`. If you add a new live-chain dispatch site, use that accessor, not the raw ref. (A regression test asserts this.)
4. **Migration default.** New user → Standard; returning user (prior session) → Advanced; thereafter the last-used view. `view` is `null` until resolved so the body doesn't flash. Don't render the desk or Standard while `view === null`.
5. **Desktop/iPhone mapping parity.** `src/lib/standard-mapping.ts` and `apps/iphone-native/rust/src/lib.rs::native_preset` must stay in lockstep. Change one → change both → run both test suites.

---

## How to verify / run

- Frontend: `npm test` (vitest), `npm run build`.
- Full umbrella: `npm run verify:fast` (frontend test+build+**build:windows**; desktop Rust fmt/clippy/lib/integration; iPhone bridge check+test). Lanes: `verify:frontend` / `verify:rust` / `verify:iphone`.
- iPhone bridge alone: `cd apps/iphone-native/rust; cargo test`.
- **Gotcha:** close the running desktop app before any `cargo test` on `src-tauri` (it locks `target\debug\yes-master.exe`); the rust lane already uses `--target-dir target\codex-rc` to dodge this.

---

## Deferred — explicitly NOT done (flagged, not dropped)

1. **§1 flagship visual-polish pass.** What shipped is a *functional, accessible* Option-B layout. The spec's first-class polish bar (motion, hero treatment, micro-interactions) is a **follow-up design pass** — use the design skills against `docs/simple-mode-mockup.html`. This is the single biggest remaining item for Standard View and the most likely "next thing" Dan wants.
2. **`build:windows` packaging.** The MSI/NSIS installer build was NOT run (heavy; unaffected by this branch — no desktop-Rust or Tauri-config changes). Run it when cutting a release.
3. **Manual GUI smoke (owner-side).** Automated tests cover the logic; nobody has *clicked through* Standard in a running build. Worth doing: import a track, audition Mastered in Standard, confirm WYSIWYG by ear, walk the transitions, Create Master and confirm the file.

---

## Remind / ask the owner (Dan)

- **Visual polish:** "Want the flagship Standard polish pass next (motion/hero/micro-interactions vs the mockup), or is the functional layout enough for now?"
- **By-ear smoke:** Standard's WYSIWYG and the export were validated by tests + a Rust bit-for-bit parity test, **not by ear in a running app yet** — offer to walk it with him, or have him smoke it.
- **Export-format parity:** the fixed export (Custom / 44.1k / 24-bit / −1 dBTP / chosen LUFS) mirrors the iPhone's `export_settings_for_options`; confirm by ear/spec it matches what he expects from the phone.
- **Scoping decisions made on his behalf** (all documented, reversible):
  - `adaptive_strength ≠ 0.5` is treated as a non-managed edit (reset to 0.5 on return) — a deliberate superset of the spec's §2b table, to keep Standard on the validated default adaptive.
  - A preset outside the reference-4 (e.g. `spatial` carried back from Advanced) shows **no** active Standard tile until the user picks one.
  - The Standard hero has a pragmatic `<select>` track switcher (>1 track) that isn't in the mockup — revisit in the polish pass.
- **Process note:** during the subagent build, a worker **auto-pushed `feat/standard-view` to origin** against instructions. No harm (main was clean; it's since been merged + pushed deliberately), but worth knowing the guardrail leaked.

---

## Broader yes-master state (beyond Standard View)

The preset+adaptive **engine is validated and treated as locked** (preset_signature/distinctness/loudness tests pass; reference-tuning gaps small; Dan ear-confirmed). Two engine-side items remain open and are **owner-gated**, independent of Standard View:

- **Tier-2 Phase B confidence gating is built but OFF by default** (`confidence::CONFIDENCE_GATING` runtime `AtomicBool`, default false; reduce-only; byte-identical Tier-1 when off). Activating it needs **Dan's ears**: enable the gate (`api.setConfidenceGating(true)` or `YES_MASTER_CONFIDENCE_GATING=1`), A/B Adapt 0 vs 0.5 on bright/dense/wide vs neutral sources, and lock the provisional `confidence.rs` constants. See `docs/HANDOFF_2026-06-04_ADAPTIVE_DSP_TIER2_PHASE_B_CONFIDENCE.md`.
- **Deadband by-ear calibration** of the provisional guardrail constants is the only other open ear-gate — de-prioritized; Dan has listened and it sounds good.
- **Owner-only / document-and-STOP (never tune without Dan):** guardrail constants/deadbands, confidence thresholds, density cap, preset voicing, gate-on-by-default, README positioning.

---

## Related docs

- Spec: `docs/superpowers/specs/2026-06-08-simple-mode-design.md`
- Plan (v2): `docs/superpowers/plans/2026-06-08-standard-view.md`
- Mockup: `docs/simple-mode-mockup.html`
- Engine reference: `docs/ENGINE_REFERENCE.md` (+ `docs/preset-reference.html`)
- Adaptive Phase B handoff: `docs/HANDOFF_2026-06-04_ADAPTIVE_DSP_TIER2_PHASE_B_CONFIDENCE.md`
- Phase A handoff: `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2_PHASE_A_COMPLETE.md`

---

## Addendum (2026-06-09): post-handoff polish commits, review, and fix series

Two **unreviewed** polish commits landed the same evening this handoff was
written, so several claims above describe a state that briefly stopped being
true; an adversarially-verified review (2026-06-09) then drove a fix series.
Read this section as the corrections layer:

- `a7d407d` ("polish standard mode shell", +645/−193) restructured the
  Standard shell: the hero `<select>` track switcher (line ~85 above) was
  **replaced by the clickable TracksRail row list**, the transport/waveform
  moved into the center column, and a StandardRightRail (A/B, Volume Match,
  delivery card, Create Master) was added. `ee65e44` added CSS accents.
  The "polish wholly deferred" framing above predates these commits.
- The review found `a7d407d` had also **flipped the return door to
  always-confirm and dropped the Track/Album tabs and single-Advanced
  chrome**, contradicting the spec; branch
  `fix/standard-review-2026-06-09` **restored spec behavior** (silent
  when clean — line 27 above is true again), restored the chrome, gave the
  tracks list/rail a scroll path, made the Standard copy truthful (derived
  LUFS qualifier, real Analyzed gating, "Standard WAV" delivery name),
  fixed the dead Create Master restyle (specificity), added TracksRail/
  StandardRightRail test coverage (259 vitest total), and cleared the CSS
  hygiene tail. See the `(review)` commits on that branch for the
  itemized evidence.
- **Graphify note (owner decision, clean later):** `graphify-out/` is
  deliberately tracked for cross-machine agent reuse. The shareable
  artifacts (`graph.json`, `GRAPH_REPORT.md`, labels) are path-clean and
  portable; `manifest.json`, `cache/stat-index.json`, and 63 `cache/ast/`
  files embed this machine's absolute paths + mtimes (they will miss and
  rebuild on any other machine, and they churn on each run), and
  `cost.json` rides along untracked-by-intent. Deferred cleanup: either
  gitignore the path-keyed files or have graphify store repo-relative
  paths before the next regeneration.
