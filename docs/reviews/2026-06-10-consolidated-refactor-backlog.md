# Consolidated Refactor Backlog — 2026-06-10

Single actionable record consolidating the four 2026-06-10 review documents.
This file **supersedes the recommendation/sequence sections** of all four; the
source docs remain the evidence record:

1. `2026-06-10-refactor-survey.md` — Claude survey (Finding 0 + 21 ranked)
2. `codex-2026-06-10-refactor-survey-.md` — Codex survey (6 findings)
3. `codex-2026-06-10-refactor-survey-review-review.md` — Codex review of (1)
4. `2026-06-10-codex-survey-evaluation.md` — evaluation of (2) + adjudication
   of (3)'s disputes

Every disagreement between the documents was already adjudicated in doc (4);
this file applies those verdicts. Nothing here re-litigates.

**Standing constraint (unchanged):** the adaptive DSP engine is locked.
Nothing in Batches 0–5 alters rendered output, a test expectation, or the
iPhone FFI contract. Engine-adjacent work lives only in the owner-gated
parking lot.

**Freshness checks at consolidation time (HEAD `e63eebf`):**

- P0 still open — `measurements_are_rendered` has zero occurrences in
  `src-tauri/tests/contracts.rs`; `cargo test` still fails to compile.
- `feature/ffi-contract-tests` does **not** exist on origin — the Claude
  survey's "coordinate with overnight Job 3" note is stale. Bridge wire-key
  tests (B2.3) are unowned work; just do them.
- `buildProjectState` duplication re-verified byte-identical at
  `useTrackMaster.ts:596-605` and `:1917-1926`.

---

## Dedup map — where every finding landed

28 findings across both surveys → **19 work items + 4 parked decisions**.
Merges and dispositions:

| Source finding | Disposition |
| --- | --- |
| Claude F0 / reproduced by Codex review-review | **B0.1** |
| Claude F3 | **B0.2** |
| Claude F11, F17, F16, F20, F21, F8, F15, F18 | **B1.1–B1.8** |
| Claude F2 (evidence corrected) | **B2.1** |
| Claude F7 + Codex F5 (merged; SignalChain width is Codex's catch) | **B2.2** |
| Claude F9 | **B2.3** |
| Claude F10 | **B2.4** (optional) |
| Claude F4 + Codex F2 (merged) + cross-lane test (new, from doc 4) | **B3.1** |
| Claude F6 | **B4.1** |
| Claude F5 | **B4.2** |
| Codex F3 re-scoped to pure parts + buildProjectState dedup (new) | **B4.3** |
| Claude F1 + Codex F1 (merged, spec corrected by doc 4) | **B5.1** |
| Codex F6 (reframed) | **Parked P1** |
| Claude F12, F13, F14 (engine-adjacent, per Codex objection — conceded) | **Parked P2** |
| Claude F19 (owner-sanctioned UI change, both docs agree) | **Parked P3** |
| tauri-specta (flagged in both) | **Parked P4** |
| Codex F3 at L scope | **Declined** (B4.3 is the kept slice) |
| Codex F4 bridge module split | **Declined** (B2.3/B2.4 target the real pain) |
| Codex F5 "display-contract module" half | **Deferred** (tripwires only) |
| Both surveys' agreed decline lists | **Declined** (see end) |

Corrections folded in from the adjudication: the `compression_mode` "live
drift" sentence is withdrawn (benign optionality drift; the `| null` bug was
fixed in `f7377f0`); commit typo `ca1ae3a`; Codex's state set was undercounted
(B5.1 spec below).

---

## Batch 0 — Baseline repair (blocks everything)

### B0.1 Fix the 8 `ExportReport` literals in contracts.rs — **S**
Add `measurements_are_rendered: <bool>` to the 8 struct literals at
`src-tauri/tests/contracts.rs:592, 632, 655, 679, 714, 748, 1785, 1817`.
`true` for 592 and the no-settings tests 632–748; `false` (or verify the
advisory cannot fire) on the density tests 1785/1817.
**Verify:** `cargo build --tests --target-dir target\codex-rc` clean, then
full `cargo test --target-dir target\codex-rc` green.

### B0.2 Golden `ExportReport` JSON snapshot — **S** (adjacent commit)
Fixed RenderedMeasurements + ExportReport + MasteringSettings triple
serialized to a committed JSON fixture. Every future receipt-field change
becomes a reviewable snapshot diff; `#[serde(default)]` additions can no
longer ship silently (B0.1's root cause).

## Batch 1 — Quick wins (all S; deletion, tests, docs only)

| # | Item | Anchor |
| --- | --- | --- |
| B1.1 | Delete dead `activeModifierChips`/`activeModifierSummary` + self-tests (134 LOC) | `App.tsx:1103-1236`; `App.compressor-mode.test.tsx:7-8,147-194` |
| B1.2 | Retarget Waveform test import; delete App.tsx re-export shim | `App.progress-and-reset.test.tsx:5`; `App.tsx:1337` |
| B1.3 | Drop stale `selectedAnalysis` dep from `updatePreview` | `useTrackMaster.ts:1355` (leftover from `dc62dab`→`0272e9a`) |
| B1.4 | Refresh stale jump-fix queue in CLAUDE.md + AGENTS.md from RELEASE_STABILIZATION.md | all 5 queue items are closed there |
| B1.5 | Sync `WindowMetrics` 3-band doc comment | `deep_analysis.rs:51` vs `:370-371` |
| B1.6 | Consolidate 6 duplicated `MasteringSettings` test builders into `tests/common/mod.rs` (needs B0.1) | `album_sample_rate.rs:16`, `album_render.rs:26`, `album_arc_trace.rs:24`, `album_character_bias.rs:28`, `contracts.rs:1871` |
| B1.7 | Rider: replace reference_tuning's divergent `now_iso()` with `types::now_iso()` | `reference_tuning.rs:429-431` → `types.rs:933` |
| B1.8 | Rider: lift SettingsPanel/HelpPanel copy into `src/lib/chrome-content.ts` | `App.tsx:464-543`; pinned by `App.chrome.test.tsx` |

**Verify per commit:** `npm test`, `npm run build`; Rust items add the cargo
fast lane. B1.4 ships CLAUDE.md and AGENTS.md in one commit.

## Batch 2 — Contract pins before movement

### B2.1 Rust ↔ TS bindings drift gate — **M(low)**
`src/bindings.ts` is hand-written; `types.rs` ↔ `bindings.ts` co-changed 5–6×,
drift caught by humans only (`f7377f0`). Rust test serializes a canonical
sample of each load-bearing wire type; a check script compares emitted key
sets against bindings.ts; any field add/remove/rename fails CI. No new
dependencies (tauri-specta stays parked, P4). Evidence note: current
`compression_mode` state is benign optionality drift — do not cite it as live
breakage.

### B2.2 Display-mirror tripwires — **S**
Three parity tests, no module consolidation, no Rust command/codegen:
1. Compressor shadow-table test pinning `src/lib/compressor-auto.ts:23-33`
   against `dsp.rs:377-592` (`preset_calibration()` at dsp.rs:623 emits it).
2. **SignalChain width parity** — `SignalChain.tsx:45-69` hand-mirrors 8
   per-preset `stereo_width` baselines; current test pins zero values and the
   mirror has drifted before (2026-05-29 validation :46). Codex's catch.
3. One cross-test linking `standard-mapping.ts` to its Rust counterpart.
These are tripwires (drift becomes loud), not single-source-of-truth fixes.

### B2.3 Bridge wire-key pins — **S**
Two Rust tests in `apps/iphone-native/rust/src/lib.rs` calling the real FFI
entry points on the `write_sine_wav` fixture, asserting the exact JSON keys
Swift decodes. Unpinned today: `true_peak_dbtp`, `dynamic_range_lu`,
`sample_rate`, `bit_depth`. Swift's `convertFromSnakeCase` means a serde
rename breaks at runtime with no test on either side. Unowned (no
ffi-contract-tests branch exists) — land directly.

### B2.4 Desktop bridge-surface canary — **S, optional**
`src-tauri/tests/iphone_bridge_surface.rs` importing the bridge's 11-symbol
list (`iphone lib.rs:6-11`) so a desktop refactor that breaks the bridge
fails in the fast lane with a message naming the bridge.

**Verify:** cargo fast lane + `cd apps/iphone-native/rust && cargo check
--all-targets && cargo test`. Mutate-one-field smoke on each new gate.

## Batch 3 — Evidence-lanes shared module

### B3.1 `evidence_lanes` module: mechanical helpers + adaptive resolver — **M**
One module, one PR, three parts:
1. Six byte-identical helpers from `fixture_matrix.rs`/`reference_tuning.rs`:
   `csv_escape`, `sanitize_path_part`, `preset_slug`, `lexically_normalize`,
   `normalized_absolute_path`, `export_report_for` (top Rust co-change pair,
   10×/192).
2. The near-identical adaptive settings resolver
   (`fixture_matrix.rs:117-138` vs `reference_tuning.rs:305-326`): clone
   `recommended_universal`, `volume_match=false`, source LUFS,
   `SourceProfile::from_analysis`, `apply_resolved_confidence(.., false)`.
   Higher-stakes class: silent adaptive-chain drift between evidence lanes
   and the live app (the 2026-06-02 defect). Parameterize the 2-line
   preset/compression divergence.
3. **New cross-lane equivalence test** (neither survey had it): both lanes
   produce identical `source_profile`/`source_confidence` for the same
   source — catches future inter-lane drift even when each lane's own pin
   stays green.
Runner public APIs and case lists unchanged; case lists are intentional
divergence. Locked resolver (`apply_resolved_confidence`) untouched.
**Verify:** full fast lane; gated confidence tests
(`YES_MASTER_CONFIDENCE_GATING=1` on `matrix_case_resolves_confidence_like_the_app`
+ `reference_settings_resolve_confidence_like_the_app`); slow lane
(`AMS_RUN_REAL_FIXTURE=1`) with byte-identical CSV/JSON outputs.

## Batch 4 — App.tsx reductions + pure-parts slice

### B4.1 Extract `ExportReceiptCard` + formatters — **M(small)**
`App.tsx:2816-3011` → `src/components/ExportReceiptCard.tsx`; add a component
smoke test. Behavior already pinned by `App.album-export.test.tsx:245-345`.

### B4.2 Extract the `AdvancedPanel` cluster — **M(high)**
`App.tsx:1737-2814` (1,078 lines, ~36% of file) →
`src/components/AdvancedPanel.tsx`, named re-export from `./App` so the three
pinning test files don't churn. Decide the `PanelResetButton` home first
(second consumer at App.tsx:1667) — lift the field primitives
(`PanelResetButton`/`GainField`/`NumberField`/`SelectField`) into a shared
module. Only `AdvancedPanel` stays public.

### B4.3 useTrackMaster pure-parts slice — **S** (re-scoped Codex F3)
1. Export-report constructor (~22 lines, `useTrackMaster.ts:1391-1413`) →
   `src/lib/export-receipt.ts` with a Vitest pinning
   `measurements_are_rendered = m != null` (gates the `target_not_reached`
   advisory downstream — preserve exactly).
2. `buildProjectState()` helper replacing the byte-identical `ProjectState`
   literals at `:596-605` (autosave) and `:1917-1926` (saveProjectAs) — a
   real autosave/save-as drift hazard, verified again at consolidation.
The L-scope hook split stays declined.

**Verify:** `npm test` (album-export, adaptive-strength,
delivery-format-visibility, compressor-mode, transitions, integration all
unchanged), `npm run build`, `npm run build:windows`, visual smoke of the
Advanced rail and one export receipt.

## Batch 5 — View/mode state machine (capstone)

### B5.1 One explicit navigation state machine — **M/L**
Replace the split-brain `view` (useViewMode) + `mode` (useTrackMaster) + two
cross-cutting App.tsx effect-guards with a single discriminated-union state +
pure reducer + legal-transition table. Merged spec, all corrections applied:

- State set: `{unresolved, standard-track, advanced-track, advanced-album,
  return-confirm-pending}` — models the `view: null` resolution window
  (loading-flicker guard, App.tsx:81) and absorbs the `returnConfirm`
  useState (App.tsx:74,115-121) with cancel semantics.
- All transition sites route through one `dispatch()`: open project, restore
  session, track switch, tab switch, Back to Standard, Reset & continue,
  Save-as-preset.
- Transitions route persistence through `writePersistedViewMode`
  (`src/lib/view-mode.ts`) or localStorage desyncs.
- `tm.setMode`/`setView` stay as thin dispatch wrappers — TopHeader/
  StandardView prop contracts byte-identical.
- Lands **after** B4 so the diff hits a ~1,800-line App.tsx, not 3,014.

Why it's last and biggest: it kills the regression class that produced the
silent Album→Standard trap (`2a78f4a`, scenario 9) and the
HANDOFF-2026-06-08 illegal-state loop — both surveys' #1 structural finding,
reached blind.

**Verify:** `App.transitions.test.tsx` scenarios 1–9 unchanged; new
table-driven reducer tests covering every (state × action) cell including the
2a78f4a case; `view-mode.test.ts`, `standard-managed.test.ts`,
`StandardView.test.tsx`, **plus `App.album-export.test.tsx` and
`App.compressor-mode.test.tsx`** (both assert against view+mode coupling);
`npm run build`, `npm run build:windows`.

---

## Parking lot — owner decisions (one line each unlocks)

| # | Decision | Options |
| --- | --- | --- |
| P1 | Graphify cache portability: tracked path-keyed cache (234 abs-path hits in manifest.json) cannot warm caches on other machines; tracking it was a deliberate 2026-06-09 choice (`c22d71a`) | **(a)** untrack `manifest.json` + `cache/` (~155 files, S) and accept cold rebuilds; **(b)** patch graphify to write repo-relative paths (M/L) |
| P2 | Engine-adjacent cleanup slice — plans fully written in the Claude survey (F12/F13/F14), byte-identity verification specified; deletes only dead code + its self-tests, but touches engine files and test expectations, so it runs only on explicit approval | **approve** (one line) / **defer** — if approved: F12 `process_sample` delete → F13 `analyze_tracks_core_lite` delete → F14 one-pole/soft-knee hoist (after F12 halves the knee sites) |
| P3 | AdaptiveReadout debug gate (owner's own TODO, App.tsx:1846-1849; "hide, don't delete" per ADAPTIVE_DSP_NEXT_STEPS.md:63-67) — changes visible UI, so it rides outside refactor batches | pick the flag home: **localStorage UI pref** (suggested) vs UI-state hook; schedule before release gate |
| P4 | tauri-specta adoption — the long-term bindings answer; a dependency change | **adopt later** / **stay with B2.1 gate** (the dependency-free 90%) |

## Declined (consensus across all docs — do not re-derive)

Community splits (Confidence/DeepAnalysis, ReferenceTuning&Exports,
NativeBridgeTypes); MasteringSettings seam-carving; AuditionController (Swift,
not in crate); App.css split (no coverage signal); useTrackMaster sub-hook
split at L scope (`restoreSnapshot` closure knot); Codex F4 bridge module
split (contract pins instead); consolidated display-contract module
(tripwires only); settings_with_intensity / import-cycle / god-node graph
artifacts; dependency/codegen changes this pass; everything on the Claude
survey's 30-item decline list. The engine-locked never-act list (mono LUFS,
dither, soft-knee correction, limiter ISP, loss budget, deadband recal)
stands.

---

## Execution plan

Order is dependency-driven; every commit independently green per repo
convention. Engine lock applies throughout.

| Session | Work | Gate to proceed |
| --- | --- | --- |
| 1 | B0.1 → B0.2, then B1.1–B1.8 (~10 small commits) | full fast lane green at each commit — first time `cargo test` compiles since `fdbcb34` |
| 2 | B2.1–B2.4 (4 commits) | each gate smoke-tested by mutating one field locally; bridge lane green |
| 3 | B3.1 (1–2 commits) | slow fixture lane byte-identical before merge |
| 4 | B4.1 → B4.2 → B4.3 (3 commits) | npm suites unchanged; visual smoke |
| 5 | B5.1 (own branch, several commits) | scenarios 1–9 + reducer table green; manual pass of all nav flows |
| owner-timed | P1–P4 as decided | per-item plans already written |

Sequencing rationale: pins before movement (B2 guards B3–B5 for free), small
moves before the big one (B4 shrinks the B5 diff), engine-adjacent only
behind the P2 gate. Estimated total: ~19 commits + parked items.

*Consolidated at HEAD `e63eebf`, 2026-06-09 (doc series dated 2026-06-10).
Supersedes the recommendation sections of all four source documents.*
