> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Codex Phase-1 refactor survey; evaluated/consolidated into the refactor backlog. _(Status: SUPERSEDED.)_ See docs/CHANGELOG.md for the project ledger.

# Codex Refactor Survey - 2026-06-10

Phase 1 survey only. No code or test changes are proposed for this commit.

The adaptive DSP engine is treated as locked. Anything that would change rendered
output, byte-identical confidence behavior, fixture expectations, or the iPhone
FFI contract is either scoped as behavior-preserving wrapper work or declined.

## Method

- Required docs read first: `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`,
  `docs/ARCHITECTURE.md`, `docs/TESTING.md`,
  `docs/RELEASE_STABILIZATION.md`.
- Git history mined from the last 200 commits, dated 2026-06-02 through
  2026-06-09.
- Generated `graphify-out/cache/*` files were filtered out of churn rankings.
  They create a raw one-commit co-change burst and are surveyed separately below.
- `docs/archive/reviews/2026-06-10-refactor-survey.md` was excluded per the owner
  instruction to ignore the nearby Claude review.
- `graphify query` was attempted but the command was not on PATH in this shell.
  Graph use therefore came from `graphify-out/GRAPH_REPORT.md` and direct
  inspection of `graphify-out/graph.json`.

## Churn And Co-Change Evidence

Filtered combined ranking, using `churn * log2(pair-incidents + 1)`:

| Rank | File | Churn | Pair incidents | Partners |
|---:|---|---:|---:|---:|
| 1 | `src/App.tsx` | 33 | 72 | 38 |
| 2 | `src/App.css` | 26 | 33 | 13 |
| 3 | `src/hooks/useTrackMaster.ts` | 19 | 64 | 21 |
| 4 | `src-tauri/src/guardrails.rs` | 14 | 36 | 18 |
| 5 | `src/components/StandardView.tsx` | 15 | 28 | 10 |
| 6 | `src-tauri/src/types.rs` | 12 | 57 | 27 |
| 7 | `src-tauri/src/fixture_matrix.rs` | 11 | 57 | 24 |
| 8 | `src-tauri/src/engine.rs` | 11 | 39 | 18 |
| 9 | `src-tauri/src/reference_tuning.rs` | 10 | 53 | 24 |
| 10 | `src/bindings.ts` | 10 | 44 | 18 |
| 11 | `src-tauri/src/profile_store.rs` | 10 | 35 | 17 |
| 12 | `apps/iphone-native/rust/src/lib.rs` | 9 | 21 | 15 |

Top filtered co-change pairs:

| Pair | Commits |
|---|---:|
| `src-tauri/src/fixture_matrix.rs` + `src-tauri/src/reference_tuning.rs` | 10 |
| `src/App.css` + `src/components/StandardView.tsx` | 10 |
| `src/hooks/useTrackMaster.ts` + `src/hooks/useTrackMaster.integration.test.tsx` | 8 |
| `src/App.css` + `src/App.tsx` | 7 |
| `src-tauri/src/engine.rs` + `src-tauri/src/profile_store.rs` | 7 |
| `src-tauri/src/types.rs` + `src/bindings.ts` | 6 |
| `src/App.tsx` + `src/hooks/useTrackMaster.ts` | 6 |
| `src/components/StandardView.tsx` + `src/components/StandardView.test.tsx` | 6 |
| `src-tauri/src/fixture_matrix.rs` + `src-tauri/src/types.rs` | 6 |

## Graph Lead Verdicts

- `Confidence & Deep Analysis` cohesion 0.05: clustering noise for refactor
  purposes. The source modules are cohesive enough, and the behavior is engine
  adjacent and locked while confidence gating is off by default.
- `Reference Tuning & Exports` cohesion 0.07: partially real. The broad community
  mixes runner entry points, export checks, and report shaping, but the repeated
  pain is narrower: fixture/reference evidence lanes duplicate adaptive settings
  resolution.
- `Native Bridge Types` cohesion 0.08: real blast radius, not a broad type-split
  mandate. The bridge is intentionally small at the C ABI, but its Rust JSON
  settings builder is coupled to shared `MasteringSettings`, `SourceProfile`, and
  confidence resolution.
- `AuditionController` god node: decline for now. It is an iPhone app
  coordinator, not a top churn/co-change file in this survey window.
- `default_master_settings()` and `settings_with_intensity()` god nodes: mostly
  test-helper hubs. Do not refactor just because graph degree is high.
- `MasteringSettings` god node: true contract hub. Avoid broad structural
  changes unless a future slice first pins the exact desktop/iPhone contract and
  accepts the FFI blast radius.

## Ranked Findings

### 1. Make Standard/Advanced/Album Navigation An Explicit State Machine

#### Proposed Change

Replace the reactive `view` plus `tm.mode` repair pattern with one explicit
navigation module that owns legal states and transitions, for example
`standard-track`, `advanced-track`, and `advanced-album`. `App.tsx` should call
transition functions such as `enterAlbum`, `enterAdvanced`, and
`returnToStandard`, and those functions should return the next legal state plus
any required reset/confirm action.

#### Evidence

- `src/App.tsx` is the top churn/coupling file: 33 touched commits, 72
  co-change pair incidents, 38 partners.
- Current logic is reactive correction: `App.tsx:80-83` toggles WYSIWYG from
  `view`, `App.tsx:90-100` bounces illegal Standard entries to Advanced, and
  `App.tsx:108-110` separately mutates Album mode before returning to Standard.
- The known silent Album -> Standard trap was fixed on 2026-06-09 in commit
  `2a78f4a fix(standard): Back to Standard now works from Album mode`.
  `src/App.transitions.test.tsx:524-548` describes the regression: setting only
  `view="standard"` let the Album-only Advanced guard bounce the app back.
- The Standard handoff says the invariant is enforced "at every entry" through a
  bounce guard (`docs/HANDOFF_2026-06-08_STANDARD_VIEW_COMPLETE.md:52-55`), and
  the addendum records another review/fix loop around spec-breaking Standard
  chrome and return-door behavior
  (`docs/HANDOFF_2026-06-08_STANDARD_VIEW_COMPLETE.md:124-133`).

#### Payoff

Illegal UI states become unrepresentable instead of being corrected after
render. That concentrates Standard/Advanced/Album rules in one module, makes
future Standard polish less likely to break Album mode, and gives tests a
smaller, behavior-pinned interface to exercise.

#### Risk

Locked engine proximity: low. This is UI state and should not alter
`MasteringSettings` values sent to the chain except where current transition
tests already require reset behavior.

FFI proximity: none.

#### Effort

M

#### Verification Plan

- `npm test -- src/App.transitions.test.tsx src/lib/view-mode.test.ts src/lib/standard-managed.test.ts src/components/StandardView.test.tsx`
- `npm test`
- `npm run build`
- `npm run build:windows`

### 2. Extract A Shared Evidence-Lane Adaptive Settings Resolver

#### Proposed Change

Move the duplicated "source analysis + preset/case -> render settings with
`SourceProfile` and gate-aware confidence" logic behind one Rust helper used by
both `fixture_matrix.rs` and `reference_tuning.rs`. Keep runner CLIs, report
schemas, CSV columns, and rendered audio paths unchanged.

#### Evidence

- `src-tauri/src/fixture_matrix.rs` and `src-tauri/src/reference_tuning.rs` are
  the highest co-change pair in the filtered history: 10 commits changed them
  together.
- The same adaptive setup is duplicated today:
  `src-tauri/src/fixture_matrix.rs:117-138` and
  `src-tauri/src/reference_tuning.rs:305-324` both clone the source's
  recommended settings, set source LUFS, inject `SourceProfile::from_analysis`,
  and call `apply_resolved_confidence`.
- This pair has a bug-ledger history. The 2026-06-02 review found the slow lanes
  did not exercise adaptive guardrails at all
  (`docs/archive/reviews/2026-06-02-adaptive-dsp-tier1-review.md:62-65`). The
  2026-06-04 review found they would validate the wrong chain once confidence
  gating was enabled
  (`docs/reviews/2026-06-04-adaptive-deep-analysis-adversarial-review.md:52-62`).
  The 2026-06-05 triage confirms both were fixed in parallel
  (`docs/archive/reviews/2026-06-05-adaptive-dsp-GLOBAL-review-triage.md:34`,
  `docs/HANDOFF_2026-06-04_ADAPTIVE_DSP_TIER2_PHASE_B_CONFIDENCE.md:111`).

#### Payoff

Future adaptive context changes hit one evidence-lane module instead of two
nearly identical runner files. The private already-mastered matrix and reference
tuning lanes stay representative of the app chain by construction.

#### Risk

Locked engine proximity: medium. The helper feeds render settings used by
evidence lanes, but it must not change DSP math or rendered output. Any diff in
runner JSON/CSV other than ordering from refactor mechanics is a failure.

FFI proximity: none.

#### Effort

M

#### Verification Plan

- `cd src-tauri; cargo test --lib fixture_matrix --target-dir target\codex-rc`
- `cd src-tauri; cargo test --lib reference_tuning --target-dir target\codex-rc`
- `cd src-tauri; $env:YES_MASTER_CONFIDENCE_GATING = "1"; cargo test --lib matrix_case_resolves_confidence_like_the_app --target-dir target\codex-rc; cargo test --lib reference_settings_resolve_confidence_like_the_app --target-dir target\codex-rc; Remove-Item Env:\YES_MASTER_CONFIDENCE_GATING`
- `cd src-tauri; cargo test --target-dir target\codex-rc`
- Before any DSP/export merge that depends on these lanes: `cd src-tauri; $env:AMS_RUN_REAL_FIXTURE = "1"; cargo test --target-dir target\codex-rc; Remove-Item Env:\AMS_RUN_REAL_FIXTURE`

### 3. Deepen `useTrackMaster` By Moving Internal Workflows Behind Its Existing Hook Interface

#### Proposed Change

Keep the public `useTrackMaster()` return shape stable, but move three internal
workflows into private helper modules: live-chain dispatch/WYSIWYG landing,
track export receipt assembly, and project/session persistence. `App.tsx` and
`StandardView` should still consume the same hook interface after the refactor.

#### Evidence

- `src/hooks/useTrackMaster.ts` is the third combined churn/coupling hotspot: 19
  touched commits, 64 co-change pair incidents, 21 partners.
- It is 2,057 lines and owns unrelated workflow clusters: mode/album state
  (`src/hooks/useTrackMaster.ts:245-250`), live-chain dispatch
  (`src/hooks/useTrackMaster.ts:342-430`), session restore/autosave
  (`src/hooks/useTrackMaster.ts:525-611`), album export
  (`src/hooks/useTrackMaster.ts:1207-1272`), track export receipt construction
  (`src/hooks/useTrackMaster.ts:1357-1428`), playback switching
  (`src/hooks/useTrackMaster.ts:1441-1480`), and project open/save
  (`src/hooks/useTrackMaster.ts:1900-1985`).
- The hook returns a very broad interface from `src/hooks/useTrackMaster.ts:2059`
  through `src/hooks/useTrackMaster.ts:2140`, which is why small workflow changes
  frequently co-change with integration tests and `App.tsx`.
- Review history points to caller-order and stale-state defects around this hook:
  preview/export adaptive divergence
  (`docs/archive/reviews/2026-06-02-adaptive-dsp-tier1-review.md:64-65`), readout/input
  contract drift (`docs/archive/reviews/2026-06-03-adaptive-dsp-FINAL-master-review.md:36-37`),
  and backend cache eviction after remove
  (`docs/reviews/2026-06-04-adaptive-deep-analysis-adversarial-review.md:106-114`).

#### Payoff

The hook remains the single interface used by the app, but its implementation
gains locality: live audition responsiveness, export receipt semantics, and
project persistence can be reviewed and tested independently. This also reduces
the chance that Standard View work accidentally changes export or live-chain
behavior.

#### Risk

Locked engine proximity: low to medium. The refactor touches callers of render,
playback, and export commands, so behavior must be preserved at the command
payload level. It must not touch Rust DSP or expected render measurements.

FFI proximity: none.

#### Effort

L

#### Verification Plan

- `npm test -- src/hooks/useTrackMaster.integration.test.tsx src/App.transitions.test.tsx src/App.album-export.test.tsx src/App.compressor-mode.test.tsx`
- `npm test`
- `npm run build`
- `npm run build:windows`

### 4. Keep The iPhone C ABI Stable While Splitting Bridge Internals

#### Proposed Change

Leave `apps/iphone-native/rust/src/lib.rs` as the C ABI facade with the same
`#[no_mangle]` functions, but move native settings construction, preset mapping,
adaptive context resolution, and JSON error/string helpers into private bridge
modules. Do not change the C header, Swift call sites, JSON response shape, or
shared `yes_master_lib` types in this refactor.

#### Evidence

- `apps/iphone-native/rust/src/lib.rs` has 9 touched commits, 21 co-change pair
  incidents, and 15 partners in the last 200 commits.
- The file mixes ABI exports (`apps/iphone-native/rust/src/lib.rs:18-140`) with
  shared engine calls and settings construction
  (`apps/iphone-native/rust/src/lib.rs:157-247`) and Standard/iPhone preset
  aliases (`apps/iphone-native/rust/src/lib.rs:249-261`).
- The graph flags this seam directly: iPhone native reuses shared desktop DSP
  (`graphify-out/GRAPH_REPORT.md:193-195`), `native_adaptive_context_for_path`
  bridges Native Bridge Types to engine/live stream communities
  (`graphify-out/GRAPH_REPORT.md:706-711`), and several iPhone-to-engine edges
  are inferred (`graphify-out/GRAPH_REPORT.md:159-168`), so source verification
  is required.
- The FFI has already changed once: `yes_master_native_render_master_with_options_json`
  added a fifth `lufs_target` arg, requiring Rust header and Swift call site
  sync (`docs/HANDOFF_2026-06-01_IPHONE_NATIVE_PERF_WIRING.md:32-33`).
- Current tests do pin much of the bridge behavior:
  `apps/iphone-native/rust/src/lib.rs:361-386` checks adaptive context injection
  and gate-off confidence, `apps/iphone-native/rust/src/lib.rs:702-761` checks
  bridge render parity with the shared render path, and
  `apps/iphone-native/rust/src/lib.rs:836-857` checks native render settings
  match the desktop contract.

#### Payoff

The minimal ABI stays easy for Swift to call, while Rust-side settings and
adaptive resolution gain a smaller interface and clearer test surface. Future
shared engine fields will be less likely to force edits inside unsafe FFI
functions.

#### Risk

Locked engine proximity: medium. The bridge calls shared analysis/render paths
and constructs `MasteringSettings`, so payload equality matters.

FFI proximity: high. The work is safe only if the exported symbols, header, and
Swift wrapper remain unchanged.

#### Effort

M

#### Verification Plan

- `cd apps/iphone-native/rust; cargo check --all-targets`
- `cd apps/iphone-native/rust; cargo test`
- On a Mac bridge lane: `cd apps/iphone-native; xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test`
- If any shared Rust type is touched despite the intended scope: also run `cd src-tauri; cargo test --target-dir target\codex-rc`

### 5. Pin And Consolidate Display-Only Preset Mirrors

#### Proposed Change

Create a small frontend preset-display contract module for display-only mirrors
that currently live in separate files, then add parity tests for every mirrored
constant. This should cover compressor readouts, `SignalChain` preset width
baselines, and Standard style mapping; do not add a Rust command or codegen in
this pass.

#### Evidence

- The 2026-05-29 review identified Rust/TS compressor calibration duplication as
  the highest-value cleanup in that pass
  (`docs/archive/reviews/2026-05-29-adversarial-review.md:127-133`), and Codex
  validated the drift risk
  (`docs/archive/reviews/2026-05-29-codex-validation-of-master-review.md:54-56`).
- Current code still hand-mirrors the compressor table in
  `src/lib/compressor-auto.ts:23-33`, with engagement math at
  `src/lib/compressor-auto.ts:51-60`.
- Current `SignalChain` still documents manual sync with Rust preset width
  baselines (`src/components/SignalChain.tsx:45-68`), after an earlier stale
  mirror bug was validated
  (`docs/archive/reviews/2026-05-29-codex-validation-of-master-review.md:45-47`).
- Standard desktop/iPhone mapping parity is explicitly load-bearing:
  `docs/HANDOFF_2026-06-08_STANDARD_VIEW_COMPLETE.md:56` says
  `src/lib/standard-mapping.ts` and `apps/iphone-native/rust/src/lib.rs::native_preset`
  must stay in lockstep.

#### Payoff

UI readouts and labels become harder to drift from the validated engine. This
does not make the frontend authoritative over DSP; it only makes display mirrors
easier to review and test as mirrors.

#### Risk

Locked engine proximity: low. Display-only values must not feed render settings
except the existing Standard style mapping, which is already tested.

FFI proximity: medium if Standard/iPhone mapping tests are extended; low if the
pass only moves frontend display mirrors.

#### Effort

S

#### Verification Plan

- `npm test -- src/lib/compressor-auto.test.ts src/components/SignalChain.test.tsx src/lib/standard-mapping.test.ts`
- `npm test`
- If `apps/iphone-native/rust/src/lib.rs::native_preset` is touched: `cd apps/iphone-native/rust; cargo test native_options_map_to_shared_preset_and_intensity`

### 6. Remove Path-Keyed Graphify Cache Churn From The Tracked Signal

#### Proposed Change

Keep portable graph artifacts tracked, but stop tracking path-keyed generated
cache artifacts or regenerate them with repo-relative paths. The likely target
is `graphify-out/manifest.json`, `graphify-out/cache/stat-index.json`,
`graphify-out/cache/ast/*`, and similar machine-local cache files; keep
`graphify-out/GRAPH_REPORT.md`, `graphify-out/graph.json`, and labels if they
are the intended shared artifacts.

#### Evidence

- Raw co-change analysis was dominated by one generated graphify cache commit:
  multiple `graphify-out/cache/*` files showed 158 pair incidents and 158
  partners despite only one touched commit.
- The Standard handoff already records the cleanup decision: shareable artifacts
  are path-clean, while `manifest.json`, `cache/stat-index.json`, and 63
  `cache/ast/*` files embed absolute paths and mtimes and will churn on each
  machine/regeneration
  (`docs/HANDOFF_2026-06-08_STANDARD_VIEW_COMPLETE.md:135-142`).
- Direct reading confirmed `graphify-out/manifest.json` contains absolute
  machine paths.

#### Payoff

Future churn/co-change mining will stop treating generated cache churn as a
first-class architecture signal. Cross-machine graph sharing also becomes less
fragile.

#### Risk

Locked engine proximity: none.

FFI proximity: none.

#### Effort

S

#### Verification Plan

- `git ls-files graphify-out`
- `git diff --check`
- Re-run the churn script from this survey and confirm `graphify-out/cache/*`
  no longer dominates raw co-change.

## Quick Wins

Bundle findings 5 and 6 as the low-risk Phase 2 opening batch:

- Pin and consolidate display-only preset mirrors. This is UI/test-surface work,
  not DSP math.
- Remove path-keyed Graphify cache churn from the tracked signal. This is repo
  hygiene and makes future survey data cleaner.

These should be tiny, independent commits. Neither should change rendered audio,
Tauri command payloads, or the iPhone C ABI.

## Recommended Phase 2 Sequence

1. Graphify cache cleanup. It de-noises the repo before more agents use the graph
   or churn data, and it has no product behavior risk.
2. Display-only mirror parity. It closes known UI drift hazards before any
   Standard/iPhone polish or preset-display edits.
3. Standard/Advanced/Album navigation state machine. Do this before more
   Standard visual/layout work, because recent Standard churn has repeatedly
   touched `App.tsx`, `App.css`, and `StandardView` together.
4. Evidence-lane adaptive settings resolver. Do this before any future
   confidence-gate or private-fixture work, because the private lanes are the
   proof harness for the locked engine.
5. `useTrackMaster` internal deepening. This is larger; take it in tiny commits
   by workflow: live-chain dispatch first, then export receipt assembly, then
   project persistence.
6. iPhone bridge internals. Do this only when a Mac/iPhone bridge lane is
   available, and keep the C ABI facade unchanged.

## Decline List

- Do not refactor DSP math in `src-tauri/src/dsp.rs`. Soft-knee behavior,
  dither amplitude, density cap shape, preset voicing, and guardrail constants
  are output-affecting or owner-ear-gated.
- Do not delete or rewrite legacy `process_sample` as part of this refactor
  pass. It is a known dead/trap path, but touching it changes engine code and
  test expectations; leave it flagged unless the owner explicitly approves a
  no-output engine cleanup slice.
- Do not broadly split `MasteringSettings` or `src-tauri/src/types.rs` now.
  It is a real contract hub for desktop, tests, bindings, project state, and the
  iPhone bridge. Pin any new interface first.
- Do not split `Confidence & Deep Analysis` based only on graph cohesion. The
  gate is off by default, tests are dense, and broad movement is too close to the
  locked adaptive engine.
- Do not broadly restructure `Reference Tuning & Exports`. The real tangle is
  the shared evidence-lane adaptive settings resolver, not export checks as a
  whole.
- Do not refactor `AuditionController` from graph degree alone. It is not a
  demonstrated recent churn/coupling hotspot in this survey window.
- Do not pursue framework, dependency, or codegen changes. They are not needed
  for the demonstrated pain in this pass.
- Do not treat old handoffs as active spec. They were useful for the bug ledger
  only when confirmed against current source and current docs.
