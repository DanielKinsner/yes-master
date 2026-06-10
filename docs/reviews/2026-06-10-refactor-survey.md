# Refactor Survey — Phase 1 (2026-06-10)

Behavior-preserving refactor survey of yes-master. Survey only — no code was
changed. Phase 2 executes only after owner approval of this document.

**Standing constraint:** the adaptive DSP engine is locked. `cargo test`
(src-tauri — `preset_signature`, `tests/contracts.rs`, byte-identical
confidence tests) and `npm test` define current behavior. Every finding below
was screened against that lock; anything that would alter rendered output, a
test expectation, or the iPhone FFI contract is in the
[Flagged findings](#flagged-findings--engine-locked-never-act) section and is
**not** proposed for action.

**Method.** Four lenses, in priority order: (1) churn × co-change over the
last 192 commits of `main`; (2) the bug ledger — every `docs/reviews/*` since
2026-05-29 plus all HANDOFF docs; (3) direct reading of the high-churn files;
(4) the knowledge graph in `graphify-out/` for orientation and blast radius
(INFERRED edges treated as hints). Thirty-two candidate findings were
produced, then each was adversarially re-verified against the working tree at
`c22d71a` (every cited file:line was re-opened; stale line numbers from older
reviews were corrected to current main). 24 survived, 5 were refuted, 3 were
downgraded to hunches. Two pairs of survivors covered the same ground and are
merged below, giving **22 ranked findings**.

**Churn × co-change headline numbers** (last 192 commits, files changed per
commit):

| Signal | Value |
| --- | --- |
| Top churn | `src/App.tsx` 33, `src/App.css` 26, `src/hooks/useTrackMaster.ts` 19, `src/components/StandardView.tsx` 15, `src-tauri/src/guardrails.rs` 14, `src-tauri/src/types.rs` 12 |
| Top Rust co-change pair | `fixture_matrix.rs` ↔ `reference_tuning.rs` — 10 same-commit changes |
| Top FE co-change pair | `App.css` ↔ `StandardView.tsx` — 10; `App.css` ↔ `App.tsx` — 7 |
| Contract chain | `types.rs` ↔ `bindings.ts` — 5–6 same-commit changes (hand-maintained mirror) |
| Sizes | App.tsx 3,014 lines · App.css 5,801 · dsp.rs 5,024 · audio.rs 3,620 · useTrackMaster.ts 2,149 |

---

## Finding 0 (P0) — `cargo test` does not compile on main

This is not a refactor finding; it is the state of the verification baseline
every other finding depends on, so it ranks above everything.

### 1. Proposed Change
Add the missing `measurements_are_rendered: <bool>` field to all 8
`ExportReport { ... }` literals in `src-tauri/tests/contracts.rs` (lines 592,
632, 655, 679, 714, 748, 1785, 1817). Use `true` where the test simulates a
rendered receipt (592, and the no-settings tests 632–748 where
`target_not_reached` cannot fire anyway); use `false` (or verify the advisory
cannot fire) on the two compression-density tests at 1785/1817 so the new
advisory does not contaminate their assertions.

### 2. Evidence
- Reproduced first-hand during this survey: `cargo build --tests
  --target-dir target\codex-rc` from `src-tauri` fails with 8 × E0063
  `missing field 'measurements_are_rendered'`, all in `tests/contracts.rs`.
- Commit `fdbcb34` (export-metrics fix 2) added the field at
  `src-tauri/src/types.rs:851` and gated the advisory at
  `src-tauri/src/exports.rs:72`, updated `fixture_matrix.rs:311` and
  `reference_tuning.rs:362`, but missed the 8 test literals. The commit's
  "cargo lib 304/304" was honest — `--lib` passes; the integration test
  binary was never built.
- Grep confirms `contracts.rs` is the only test file with `ExportReport {`
  literals (8 occurrences, exactly the 8 errors).
- CLAUDE.md's fast lane requires `cargo test --target-dir target\codex-rc`;
  it currently returns a build error, not test results.

### 3. Payoff
Restores the repo's own behavior pin. Until this lands, no Phase 2 step can
run its verification plan, and the slow fixture lane is equally unrunnable.

### 4. Risk
None to the engine: test-only edits to hand-built fake reports. No production
code, no DSP, no FFI surface. `#[serde(default)]` affects deserialization
only, not struct literals, so the fix is purely mechanical.

### 5. Effort
S

### 6. Verification Plan
- `cd src-tauri && cargo build --tests --target-dir target\codex-rc` → clean
- `cd src-tauri && cargo test --target-dir target\codex-rc` → green
- Spot-check the two density tests (1785/1817) still assert what they did.

---

## Ranked findings

### 1. Consolidate view + mode into one explicit state machine (Lens 2: bug ledger; Lens 1: top churn)

**Proposed Change.** Replace the split-brain state — `view` in
`useViewMode`, `mode` inside `useTrackMaster`, plus two cross-cutting
`useEffect` guards in App.tsx — with a single discriminated-union state
`{view, mode, hasTrack, hasNonManagedEdits}` driven by a pure reducer with an
explicit legal-transition table (living next to
`shouldForceAdvancedOnStandardEntry` in `src/lib/standard-managed.ts`). All
transition sites (open project, restore session, track switch, tab switch,
Back to Standard, Reset & continue, Save-as-preset) route through one
`dispatch()`. Keep `tm.setMode` and `setView` as thin dispatch wrappers so
the TopHeader/StandardView prop contracts stay byte-identical.

**Evidence.**
- `src/App.tsx:80-101` — two effect-based guards (WYSIWYG flag at 80–83;
  always-clean/Album-only-in-Advanced re-bounce at 90–101).
- `src/App.tsx:108-111` — `returnToStandard` helper added 2026-06-09 in
  commit `2a78f4a` to patch the silent Album → Standard trap: the return
  door set the view and the entry-guard effect re-bounced it to Advanced in
  the same commit; the button visibly did nothing.
- `docs/HANDOFF_2026-06-08_STANDARD_VIEW_COMPLETE.md:111-133` — a *second*
  illegal-state regression class in the same (view, mode) interaction within
  24 hours (`a7d407d` → fixed across `5a944c3`/`9add9d8`).
- Churn: App.tsx 33/192 (highest in repo); the (view, mode) interaction is
  the hot zone.

**Payoff.** Illegal states (Standard+Album, Standard+non-managed edits,
unresolved view) become unrepresentable instead of corrected by reaction.
Each future transition is a table row, not a new effect+guard pair — the
"silent trap" regression class cannot recur. The reducer is exhaustively
table-testable.

**Risk.** FE-only; zero proximity to the locked engine and the FFI contract.
External setter signatures and prop contracts preserved. The interface is
already pinned: `App.transitions.test.tsx` (9 end-to-end scenarios, scenario
9 is the 2a78f4a regression), `standard-managed.test.ts`,
`useViewMode.test.tsx`.

**Effort.** M (~120 lines of state-machine logic; existing scenarios become
reducer-table assertions plus thin DOM smoke).

**Verification Plan.**
- `npm test` — all suites, especially `App.transitions.test.tsx` scenarios
  1–9 unchanged and green.
- New table-driven reducer tests covering every (view, mode, hasTrack,
  hasNonManagedEdits, action) cell, including the 2a78f4a case (Album +
  Back-to-Standard → Standard+Track).
- `npm run build` (tsc clean). No Rust touched — cargo lanes unaffected.

### 2. Pin the Rust ↔ TS wire contract with a repo-local drift gate (Lens 2 + Lens 1; merges two survivors)

**Proposed Change.** `src/bindings.ts` is hand-written ("Phase 1.2 will
replace this file with auto-generated bindings via tauri-specta" — a promise
that never landed). Add a drift gate with no new dependencies: a focused Rust
test serializes a canonical sample (`serde_json::to_value`) of each
load-bearing wire type (MasteringSettings, AdvancedSettings, AnalysisResult,
RenderJob, RenderedMeasurements, GuardrailReadout, AlbumPlan,
AlbumTrackEntry, PlaybackTick), and a check script compares the emitted key
sets against `bindings.ts`. Any Rust-side field add/remove/rename fails CI
instead of shipping silently. The full tauri-specta migration remains the
right long-term answer but is a **dependency change → owner decision**, out
of scope for this pass; the gate neither blocks nor presupposes it.

**Evidence.**
- `src/bindings.ts:1-2` — hand-written, with the stale specta promise.
- Live drift today: `bindings.ts:41` declares
  `compression_mode?: CompressionMode` while `types.rs:679-680` is a
  non-Option field with `#[serde(default)]` — an explicit `null` from the FE
  would fail deserialization; the TS type lies about acceptable values.
- Commit `f7377f0` "fix(types): close TS↔Rust binding drift" fixed three
  concrete drifts (missing `AlbumCharacter`/`album_character`, the wrong
  `| null`, width max 1.5 vs DSP clamp [0,2]) — each caught by human review,
  not CI.
- `docs/reviews/2026-05-29-adversarial-review.md:97` and
  `2026-05-29-codex-validation-of-master-review.md:121` ("Generate TS
  bindings from Rust types or add a binding drift gate").
- Co-change: `types.rs` ↔ `bindings.ts` 5 same-commit pairs in the window;
  no test anywhere roundtrips Rust JSON against the TS shape (verified —
  `contracts.rs` pins behavior, not wire shape).

**Payoff.** Closes the recurring "TS type drifted from Rust struct" defect
pattern mechanically. Future struct changes either propagate or fail CI.

**Risk.** Zero engine proximity — serde already defines the wire format; the
gate only polices the TS description of it. Net-new test artifacts only.
This is the "pin the interface first" move; it deliberately precedes any
generation step.

**Effort.** M (low-M: one Rust test file + one check script + npm test hook).

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` — new wire-shape
  test passes.
- `npm test` — drift check passes against current bindings.ts.
- Mutate one field name in bindings.ts locally → gate fails loudly → revert.

### 3. Golden ExportReport JSON snapshot test (Lens 2: bug ledger)

**Proposed Change.** Pin the export receipt wire shape with one "golden
receipt" Rust snapshot test: a fixed
RenderedMeasurements + ExportReport + MasteringSettings triple serialized to
JSON and committed as a reviewable fixture. Land it in the same commit as
Finding 0's constructor fixes so the snapshot lands on a compiling test bin.

**Evidence.**
- The receipt accreted `effective_adaptive_strength`,
  `source_profile_digest`, `confidence_digest`, `measurements_are_rendered`,
  `target_not_reached` across the 2026-06-09 export-metrics inquiry
  (`types.rs:790-853`; commits `600f638`/`fdbcb34`/`ca1a3ae`→`ca1ae3a`).
- `docs/reviews/2026-06-09-export-metrics-inquiry.md:73-76` — the inquiry
  was a contract-comprehensibility defect: the FE rendered source data under
  output-describing chips.
- Finding 0 is the proof the current pinning style (positional struct
  literals) catches deletions but not `#[serde(default)]` additions — the
  drift class already shipped.

**Payoff.** Every receipt field change becomes an intentional, reviewable
snapshot diff; renames and accidental drops fail CI.

**Risk.** Pure additive test. No rendered audio, no FFI (`ExportReport` does
not cross the iPhone bridge — verified).

**Effort.** S

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` green; `.snap`/
  JSON fixture committed.

### 4. Extract the shared evidence-lane helpers from fixture_matrix.rs / reference_tuning.rs (Lens 1: top Rust co-change pair)

**Proposed Change.** Move the byte-identical helper copies — `csv_escape`,
`sanitize_path_part`, `preset_slug`, `lexically_normalize`,
`normalized_absolute_path`, `export_report_for` — into one shared private
module (e.g. `src-tauri/src/evidence_lanes.rs`); both runners import from
it. Keep the runners' public APIs (`run_manifest_path`,
`run_fixture_matrix`, `run_reference_tuning_dir`) and their case lists
unchanged — the case lists are real, intentional divergence.

**Evidence.**
- `fixture_matrix.rs` ↔ `reference_tuning.rs` is the #1 Rust co-change pair:
  10 same-commit changes in 192 commits.
- Byte-identical pairs verified line-by-line: `csv_escape`
  (fixture_matrix.rs:343 / reference_tuning.rs:408), `sanitize_path_part`
  (:351/:416), `normalized_absolute_path` + `lexically_normalize`
  (:398,:407/:433,:442), `export_report_for` (:291/:342 — same body, same
  comment, same `measurements_are_rendered: true`).
- `preset_slug` (:364/:328) — semantically equivalent for every preset
  reference_tuning can ever pass (its `default_reference_presets()` at
  :467-477 is hardcoded to four presets); merge on fixture_matrix's full arm
  list.

**Payoff.** Cuts the structural cause of the top Rust tangle: CSV escaping,
path normalization, report-shape, and slug changes stop requiring
remember-to-touch-both edits (the `measurements_are_rendered` rollout is the
live example of this class).

**Risk.** Low. Pure functions off the DSP path; neither file is referenced
from `tests/contracts.rs` or the iPhone bridge (verified by grep). Runner
behavior pinned by 9 named in-module tests (fixture_matrix.rs:480, 553, 638,
660; reference_tuning.rs:467, 480, 524, 554, 763).

**Effort.** M

**Verification Plan.**
- `cd src-tauri && cargo test --lib --target-dir target\codex-rc` and full
  `cargo test` green (after Finding 0).
- Slow lane: `AMS_RUN_REAL_FIXTURE=1; cargo test` — CSV/JSON outputs
  byte-identical.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`.

### 5. Extract the AdvancedPanel cluster out of App.tsx (Lens 1 + Lens 3; merges two survivors)

**Proposed Change.** Move the contiguous Advanced region
(`src/App.tsx:1737-2814`, 1,078 lines, ~36% of the file) — `AdvancedPanel`,
`DeliveryProfileCard`, `AdaptiveReadout`, `AdvancedControlsCard`,
`PerBandCompressorCard`, `CompressionKnobGrid`, `DeliveryFormatCard`, the
compressor helpers, and the field primitives — into
`src/components/AdvancedPanel.tsx`. Keep a named re-export from `./App` so
the three pinning test files don't churn. `PanelResetButton` is also used at
App.tsx:1667 (Standard EQUALIZER block), so either leave it in App.tsx or
lift the field primitives (`PanelResetButton`/`GainField`/`NumberField`/
`SelectField`) into a small shared module. Only `AdvancedPanel` stays a
public symbol; child cards remain module-internal.

**Evidence.**
- App.tsx is 3,014 lines, 33 commits / 192 — the #1 hotspot.
- Sole production call site `App.tsx:181`; interface pinned by three test
  files importing `AdvancedPanel` from `./App`
  (`App.adaptive-strength.test.tsx:5`,
  `App.delivery-format-visibility.test.tsx:6`,
  `App.compressor-mode.test.tsx:6`).
- `src/components/` already houses 14 sibling components with co-located
  tests — the established pattern.

**Payoff.** Cuts App.tsx by a third; future Advanced-rail work stops
re-touching the file everything else lives in.

**Risk.** Zero engine/FFI proximity (App.tsx has no invoke/tauri
references). CSS class names stay in App.css unchanged. Watch-item:
PanelResetButton's second consumer (above).

**Effort.** M (M-high — the shared-helper hoisting is the real work).

**Verification Plan.**
- `npm test` — adaptive-strength, delivery-format-visibility,
  compressor-mode, transitions, album-export all unchanged and green.
- `npm run build` and `npm run build:windows`.
- Visual smoke: Advanced rail renders all four cards; per-band knobs work.

### 6. Extract ExportReceiptCard + formatters out of App.tsx (Lens 3)

**Proposed Change.** Move `ExportReceiptCard`, `exportQualitySummary`,
`pluralize`, `fileNameFromPath`, `formatSampleRate`, `formatBitDepth`,
`CheckRow`, `levelClass` (`src/App.tsx:2816-3011`, contiguous) into
`src/components/ExportReceiptCard.tsx`. Add a thin component-level smoke
test as cheap insurance.

**Evidence.**
- Single feature region App.tsx:2816-3011; only consumer App.tsx:230-235.
- Rendered behavior already pinned by `App.album-export.test.tsx:245-345`
  (medallion tones, journey steps, path names, delivered LUFS/TP/LRA,
  "Source ·" digest prefix, absence cases) — so this is not a
  pin-first situation, contrary to the first-pass assessment.
- Secondary payoff: `formatSampleRate`/`formatBitDepth` can later absorb the
  hardcoded "44.1 kHz · 24-bit" strings in `StandardView.tsx:346` and
  `AlbumPanel.tsx:147-149`.

**Payoff.** Lifts the second-largest contiguous region out of the top-churn
file; the formatters become independently testable.

**Risk.** Zero engine/FFI proximity; only API touch is the UI-side
`api.openOutput`.

**Effort.** M (small-M)

**Verification Plan.**
- New `ExportReceiptCard.test.tsx` mounting smoke (clean/review/needs-
  attention tones).
- `npm test` — `App.album-export.test.tsx` unchanged and green; `npm run
  build`.
- Manual: complete an export, confirm receipt renders identically.

### 7. Single source of truth for the Rust ↔ TS compressor calibration table (Lens 2)

**Proposed Change.** `src/lib/compressor-auto.ts:23-33` hand-mirrors the
eight `PRESET_*` calibrations from `src-tauri/src/dsp.rs:377-592`. Pin it
the same way as Finding 2: a tiny Rust test emits the table
(`preset_calibration()` already exists at dsp.rs:623), and a TS test asserts
`PRESET_COMPRESSOR` matches byte-for-byte. Optional follow-up: generate the
TS map from the emitted JSON at build time (repo-local script, no new
dependency). Do not switch the UI to a runtime fetch — that adds an
async-boot wrinkle for no gain.

**Evidence.**
- `docs/reviews/2026-05-29-master-review.md:84` — P2: "Compressor
  calibration duplicated Rust↔TS … actively-retuned table → next retune
  silently drifts the UI readout. Highest-value cleanup." Independently
  confirmed by the Codex validation review.
- The Rust table was retuned twice in the window (`88b3796`, `436afd1`);
  both times the TS copy was hand-synced in the same commit — the trap is
  latent, held shut by dev memory only.
- TS interface pinned by `compressor-auto.test.ts` (exact label strings) and
  `App.compressor-mode.test.tsx`.

**Payoff.** The next preset retune cannot silently lie to the UI readouts.

**Risk.** Very low. The chain's own constants are untouched
(`PresetCalibration` is internal to dsp.rs, not FFI-exposed, not consumed by
the iPhone bridge — verified). Display-only data source.

**Effort.** S

**Verification Plan.**
- `npm test` — readout strings identical; new drift assertion green.
- `cd src-tauri && cargo test --target-dir target\codex-rc` — engine
  untouched; preset_signature/byte-identity green.

### 8. Consolidate the duplicated MasteringSettings test builder into tests/common (Lens 4 → Lens 3)

**Proposed Change.** Create `src-tauri/tests/common/mod.rs` with one shared
`default_master_settings()` builder; replace the byte-identical copies in
`tests/album_sample_rate.rs:16`, `tests/album_render.rs:26`,
`tests/album_arc_trace.rs:24`, `tests/album_character_bias.rs:28`, and the
same-body `default_settings()` in `tests/contracts.rs:1871`. Leave
`src/album.rs:547` (cfg(test), can't share with integration tests) and
`src/dsp.rs:3175` (intentionally different fixture: `Preset::Custom`,
intensity 0.0) alone. Do **not** unify the per-test analysis builders
(`fake_analysis`/`neutral_analysis`/`analysis_for`) — different signatures,
deliberately scenario-tuned.

**Evidence.**
- Six real duplicates verified body-by-body (five byte-identical + one
  renamed). The graph's "36-edge god node `default_master_settings()`"
  (GRAPH_REPORT.md:148) is this copy-paste, not architecture.
- Demonstrated pain: each non-defaulted `MasteringSettings` field addition
  (e.g. `4b00cde` eq fields, the delivery/album/advanced additions) forced
  edits in every copy.

**Payoff.** ~80 LOC of scaffolding deleted; the next field addition is a
one-place fix; the god-node graph artifact disappears.

**Risk.** Tests-only; zero engine/FFI proximity. Standard `mod common;`
integration-test pattern.

**Effort.** S

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` — all
  integration binaries green (after Finding 0).
- `cargo fmt --check`, clippy clean.

### 9. Pin the four unpinned Swift-consumed wire keys at the bridge (Lens 4: FFI seam)

**Proposed Change.** Add two Rust unit tests in
`apps/iphone-native/rust/src/lib.rs` that call the real FFI entry points
(`yes_master_native_analyze_file_json`,
`yes_master_native_render_master_with_options_json`) on the existing
`write_sine_wav` fixture and assert the JSON contains the exact keys Swift
decodes. Today `true_peak_dbtp`, `dynamic_range_lu`, `sample_rate`, and
`bit_depth` are not pinned on the actual wire at all; `lufs_integrated` has
one loose substring check (lib.rs:496) and `output_paths`/`measurements`
only indexed access.

**Evidence.**
- Swift decodes via `convertFromSnakeCase`
  (`NativeMasteringBridge.swift:95,136`); any `#[serde(rename)]` on
  `AnalysisResult`/`RenderedMeasurements` (types.rs:62-120, 789-810) breaks
  Swift silently at runtime, and no test on either side catches it.
- Existing bridge wire pin covers only 3 keys of one type
  (`fixed_export_settings_json_uses_shared_contract_shape`, lib.rs:391-398).

**Payoff.** Serde renames/drops fail in the existing `verify:iphone` lane
instead of crashing the app at runtime — the cheapest insurance on the
highest-consequence seam.

**Risk.** Zero. Key-presence assertions on serialized output; sample-level
parity already pinned separately by `bridge_render_matches_shared_render_path`
(lib.rs:702).

**Effort.** S

**Verification Plan.**
- `cd apps/iphone-native/rust && cargo test` — new tests green.
- Locally rename one serde field → test fails → revert.

*Coordination note:* overnight Job 3 (`feature/ffi-contract-tests`) covers
this same seam; fold this finding into that branch's review rather than
duplicating work.

### 10. Desktop-side bridge-surface canary test (Lens 4: FFI seam)

**Proposed Change.** Add `src-tauri/tests/iphone_bridge_surface.rs` (~50
LOC) that imports exactly the symbol list the bridge imports
(`apps/iphone-native/rust/src/lib.rs:6-11`) and touches the fields the
bridge reads — a compile-time canary in the fast lane.

**Evidence.**
- CLAUDE.md: the bridge re-uses `yes_master_lib` but no fast-lane builds it;
  "it broke the bridge build once already."
- Honest scope note from verification: most of these symbols are already
  *transitively* pinned by existing desktop tests (contracts.rs,
  profile_store unit tests), so this is consolidation-and-labeling — one
  obvious chokepoint whose failure message names the bridge — not a true
  coverage gap.

**Payoff.** A desktop-only refactor that would break the bridge fails in
`cargo test --target-dir target\codex-rc` with a message that says why,
instead of being discovered when someone manually runs the bridge lane.

**Risk.** Zero. Tests-only, imports already-public symbols.

**Effort.** S

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` green.
- Sanity: locally rename one symbol → canary and bridge fail identically →
  revert. Bridge lane still green.

### 11. Delete the dead activeModifierChips / activeModifierSummary helpers (Lens 3)

**Proposed Change.** Remove `ActiveModifierChip`, `activeModifierChips`,
`activeModifierSummary` (`src/App.tsx:1103-1236`, 134 LOC) and their
self-referential tests (`App.compressor-mode.test.tsx:7-8` imports, test
cases at 147-194). The chip strip they powered was removed from the UI; the
helpers survive only because their own tests assert their own outputs.

**Evidence.** Repo-wide grep: zero references outside the definitions, the
self-tests, and auto-generated graphify artifacts.

**Payoff.** 134 lines off the top-churn file; removes a test that only tests
itself.

**Risk.** None — pure FE dead code; engine/FFI untouched.

**Effort.** S

**Verification Plan.** `npm test -- App.compressor-mode` (remaining 10 tests
green), `npm run build` (no dangling import).

### 12. Delete the legacy `process_sample` path — option (a) only (Lens 2; engine-adjacent, removal-only)

**Proposed Change.** Delete `MasteringChain::process_sample`
(`src-tauri/src/dsp.rs:2260-2406`, ~147 lines) and its two self-pinning
tests (dsp.rs:2663-2680, 4545-4564). Do **not** attempt option (b)
(rewriting it to share the frame path) — the frame path's stereo-linked
lookahead limiter has no per-sample equivalent, and a zero-caller function
doesn't justify engine-math work.

**Evidence.**
- `docs/reviews/2026-05-29-adversarial-review.md` §4 P3 and the Codex
  validation both document it: no production caller, deliberately skips
  low_mid (dsp.rs:2272-2274), soft-clip instead of the lookahead limiter
  (:2399-2404 vs :2055), `powf` where the live path uses `exp` (:2374 vs
  :2202-2203) — "its own passing test lends it false credibility."
- Grep: zero non-test references in src-tauri and the iPhone bridge; all
  production audio flows through `process_interleaved` →
  `process_frame_inplace`.
- Correction recorded during verification: the old review's claim that
  engine.rs:422/441 shims constrain this was a misread — those lines are an
  AppHandle parameter and a progress closure; no constraint exists.

**Payoff.** Removes a documented latent trap (a wrong-EQ, weaker-ceiling
chain one wiring mistake away from production) and 147 lines of divergent
near-engine code.

**Risk.** Flagged engine-adjacent, but removal of an unreachable function
cannot perturb rendered output, preset_signature, or the FFI. The deleted
tests pin only the dead function itself.

**Effort.** S

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` green
  (preset_byte_identity SHAs untouched), clippy `-D warnings` clean.
- Grep confirms no remaining `.process_sample(` references.

### 13. Delete `analyze_tracks_core_lite` — path (a) (Lens 3)

**Proposed Change.** Inline `analyze_tracks_core_impl` into
`analyze_tracks_core`, delete the `_lite` wrapper
(`src-tauri/src/engine.rs:68-77`), its gate test
(`tests/contracts.rs:184-221`) and the ignored bench (:223-257). The iPhone
bridge deliberately uses the deep-capable path (lib.rs:59-60 comment:
render/live settings resolve adaptive profile + confidence from
DeepAnalysis), so repointing it to `_lite` (path b) would *change bridge
behavior* — not an option under the lock.

**Evidence.**
- engine.rs:69-72 docstring admits the bridge doesn't use it; zero non-test
  callers (grep).
- Three-way doc/code drift: `HANDOFF_2026-06-03_…PHASE_A_COMPLETE.md:194,250,
  350-353` still says the bridge was repointed to `_lite`; the bridge was
  reverted without removing the helper.

**Payoff.** One analysis entry point; the doc/code story becomes honest;
removes a drift source on engine.rs (11 commits in window).

**Risk.** Very low. The surviving `analyze_tracks_core` is heavily pinned
(contracts.rs ×9, deep_analysis_integration.rs ×3, the bridge itself). FFI
contract unchanged.

**Effort.** S

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` green.
- `cd apps/iphone-native/rust && cargo check --all-targets && cargo test`
  green (bridge untouched but lane run per CLAUDE.md).

### 14. Hoist the duplicated one-pole envelope + soft-knee math inside dsp.rs (Lens 2; engine-internal, byte-identity-gated — owner may defer)

**Proposed Change.** Extract two private `#[inline]` helpers inside dsp.rs:
the one-pole `alpha*env + (1-alpha)*x` (five inline sites: :1575, :1615-1616,
:2174, :2356) and the soft-knee block duplicated byte-identically at
:2182-2191 (hot frame path) and :2364-2373 (legacy path — which Finding 12
deletes first, reducing this to one knee site). Pure expression extraction;
zero-bit drift required.

**Evidence.**
- `docs/reviews/2026-05-29-adversarial-review.md` §4 P3 ("one-pole envelope
  is implemented three times" — actually five sites on re-count) and §2b P3
  (the soft-knee `.max(0.0)` behavior duplicated in both copies — a future
  one-sided "fix" is the documented trap).
- preset_byte_identity SHA-256 snapshots (dsp.rs:3505-3700+) pin the frame
  path bit-exact per preset — the strongest possible behavior pin for this
  refactor.

**Payoff.** The (taste-gated, owner-ear, currently deferred) soft-knee
correction becomes a one-place change when its day comes; eliminates the
fix-one-copy-miss-the-other trap.

**Risk.** Engine-internal — the closest ranked finding to the lock, which is
why it ranks below every removal/extraction above. Safe by construction
(identical expression, `#[inline]`), and any drift fails the SHA snapshots
immediately. Sequence after Finding 12 so only one knee site remains.
`EnvelopeFollower` (test-only, but `pub`) keeps its API; switch its impl to
the helper rather than deleting it.

**Effort.** S

**Verification Plan.**
- `cd src-tauri && cargo test --target-dir target\codex-rc` — byte-identity
  SHAs, preset_signature, distinctness, loudness all unchanged.
- Slow lane `AMS_RUN_REAL_FIXTURE=1` byte-identical.
- `cd apps/iphone-native/rust && cargo test` green.

### 15. Replace reference_tuning's divergent `now_iso()` with the canonical helper (Lens 3)

**Proposed Change.** Delete the private `now_iso()` at
`reference_tuning.rs:429-431` (forces millisecond precision + `Z` suffix)
and the `use chrono::SecondsFormat;` at :8; point the sole call site (:281)
at the canonical `types::now_iso()` (types.rs:933), adding it to the
explicit import list at :4-7.

**Evidence.** Every other production site (album.rs:532, analysis.rs:190,
album_render.rs:419, engine.rs:847, settings.rs:29, fixture_matrix.rs:268)
uses the canonical helper; the canonical format is pinned by
engine.rs:1139's RFC3339 test; nothing pins the millis variant.

**Payoff.** One timestamp format across every report on disk; one grep hit
for the helper instead of two.

**Risk.** None to the engine — a JSON report timestamp, not a render input.
Cosmetic format shift (`Z` → `+00:00`) noted; both parse as RFC3339.

**Effort.** S

**Verification Plan.** `cargo build --lib` + `cargo test --lib
--target-dir target\codex-rc` green; confirm `SecondsFormat` no longer
imported anywhere.

### 16. Drop the stale `selectedAnalysis` dep from `updatePreview` (Lens 3)

**Proposed Change.** Remove `selectedAnalysis` from the `useCallback` dep
list at `src/hooks/useTrackMaster.ts:1355`. Drop, don't document: git shows
`dc62dab` added the dep for an `injectSourceProfile(...)` call that
`0272e9a` (backend-owned profiles, same day) removed — the dep is a
demonstrated leftover, and a `void` justification would document a fiction.

**Evidence.** Body (:1331-1354) reads only `selectedTrackId`,
`selectedTrack`, `selectedSettings`, `markFresh` — verified by read and
grep; the two commits above are the root cause.

**Payoff.** Removes a spurious callback-identity churn on every analysis
arrival; strictly fewer re-renders.

**Risk.** None. Interface (`() => Promise<void>`) pinned by
`useTrackMaster.integration.test.tsx:1359` and the prop site App.tsx:202.

**Effort.** S

**Verification Plan.** `npm test -- useTrackMaster.integration` green;
manual double-click of Update preview unchanged.

### 17. Retarget the Waveform test import and delete App.tsx's re-export shim (Lens 3)

**Proposed Change.** Update `src/App.progress-and-reset.test.tsx:5` to
import `WaveformView`/`WaveformLoading` from `./components/Waveform`
(keeping `Macros` from `./App`), then delete the one-line re-export at
`src/App.tsx:1337`.

**Evidence.** The re-export exists solely for that one test; App.tsx itself
imports from `./components/Waveform` (line 26) and StandardView.tsx:23
already uses the direct pattern.

**Payoff.** Removes a misdirection — App.tsx no longer pretends to own the
waveform components.

**Risk.** None. The same test continues to pin Waveform behavior.

**Effort.** S

**Verification Plan.** `npm test -- App.progress-and-reset` green;
`npm run build`.

### 18. Lift SettingsPanel/HelpPanel copy into `src/lib/chrome-content.ts` (Lens 3)

**Proposed Change.** `SettingsPanel` (App.tsx:464-502) and `HelpPanel`
(:504-543) are pure presentational components over hardcoded copy. Move the
literal data to `src/lib/chrome-content.ts`; shrink the components to thin
maps. Matches the established extraction pattern (SettingsGroup/ChromeDialog
already live in components/).

**Evidence.** `App.chrome.test.tsx:5` imports both and asserts the exact
copy strings + close-button aria-labels — fully pinned.

**Payoff.** Copy becomes editable without touching React; good warm-up
before the AdvancedPanel move.

**Risk.** None.

**Effort.** S

**Verification Plan.** `npm test -- App.chrome` green; `npm run build`.

### 19. Decide the AdaptiveReadout debug-gate (owner-sanctioned UI change — not strictly behavior-preserving)

**Proposed Change.** App.tsx:1846-1849 carries an owner TODO (2026-06-08):
the per-axis adaptive readout is an iteration aid and must hide behind a
debug/advanced flag before release-stable
(`docs/ADAPTIVE_DSP_NEXT_STEPS.md:63-67` — "hide, don't delete"). Add a
UI-preference flag (no existing diagnostics flag exists — this is net-new
state; localStorage or a UI-state hook, **not** a MasteringSettings field)
and gate the render at App.tsx:2087.

**Evidence.** TODO verbatim at App.tsx:1846-1849; unconditional render at
:2087; ADAPTIVE_DSP_NEXT_STEPS.md:63-67.

**Payoff.** Closes a documented release-blocker without losing the tuning
data.

**Risk.** None to the engine — pixels only. Caveat honestly: this *changes
visible UI behavior* (readout hidden by default), which is why it sits in
this list only because the owner's own TODO mandates it. The existing
positive test (App.adaptive-strength.test.tsx:167-199) must set the flag;
add a negative test for flag-unset.

**Effort.** S

**Verification Plan.** `npm test -- App.adaptive-strength` (updated positive
+ new negative case); toggle flag in dev and confirm readout reappears.

### 20. Refresh the stale jump-fix queue in CLAUDE.md / AGENTS.md (Lens 3: docs)

**Proposed Change.** Replace the "Current Jump-Fix Queue" block (CLAUDE.md
and the byte-identical AGENTS.md copy, lines 27-33) with the genuinely open
work per `docs/RELEASE_STABILIZATION.md`: Manual Listening Gate (:202),
pending Reference Retune listening (:157), Album channel-count parity (:31),
deferred subjective preset retuning (:217).

**Evidence.** All five queue items are closed in RELEASE_STABILIZATION.md:
item 1 (:104 implemented), item 2 (:122 implemented), item 3 (:168-172
manifest completed 2026-05-28), item 4 (:179-189, counters removed in
`58c25d7` — grep confirms no production counter wiring remains), item 5
(:191-199 complete). CLAUDE.md is the first thing every fresh agent reads;
it currently assigns work that's already done.

**Payoff.** Stops agents hunting for non-existent diagnostic counters;
restores CLAUDE.md as a source of truth.

**Risk.** Doc-only.

**Effort.** S

**Verification Plan.** Side-by-side read: every queue bullet maps to an open
status in RELEASE_STABILIZATION.md. Update both files in one commit.

### 21. Sync the conflicting WindowMetrics 3-band doc comments (Lens 3: docs)

**Proposed Change.** Tighten `deep_analysis.rs:51` ("3-band tonal shares …
for temporal brightness clumping" — reads as a live Phase-B input) to match
:370-371 (correct: retained for Phase-A diagnostics and historical tests;
Phase B consumes the 31-band detail). Do **not** delete the fields — the
integration test pins them.

**Evidence.** `Confidence::from_deep` (confidence.rs:183-228) never reads
`.low/.mid/.high`; sole consumers are deep_analysis.rs:209/:662 and
`tests/deep_analysis_integration.rs` — verified by grep.

**Payoff.** The next reader doesn't think the 3-band path feeds Phase B.

**Risk.** Comment-only.

**Effort.** S

**Verification Plan.** `cargo test --lib --target-dir target\codex-rc`
green (nothing else needed).

---

## Flagged findings — engine-locked, never act

Recorded per the standing constraint; each would alter rendered output, a
test expectation, or owner-ear calibration. **None is proposed for Phase 2.**

1. **Mono live LUFS +3 LU inflation** (master-review §2) — touches
   `sources.rs::process_frame` metering math; rendered-metering change.
2. **WAV TPDF dither amplitude ±1 LSB correction** (master-review §2) —
   changes exported PCM byte hashes pinned by snapshot tests.
3. **Soft-knee lower-half correction** (adversarial §2b) — sub-dB voicing
   change; CLAUDE.md requires a listening note first. Finding 14 only
   pre-positions the code so this becomes a one-place change if ever
   approved.
4. **Limiter ISP Lagrange-4 → polyphase FIR** — changes true-peak readings.
5. **LUFS-landing loss budget across axes (B3)** — changes rendered loudness
   on multi-axis-trimmed sources; breaks preset_signature/contracts.
6. **Deadband / default Adapt-strength recalibration** — owner-ear per the
   GLOBAL-review triage.
7. **tauri-specta adoption** — the right long-term bindings answer, but a
   dependency change and therefore an owner decision, not a Phase 2 step.
   Finding 2's gate is the dependency-free 90%.

---

## Graph leads adjudication (Lens 4)

Every lead from `graphify-out/GRAPH_REPORT.md` was confirmed or dismissed
against source.

| Lead | Verdict |
| --- | --- |
| **Confidence & Deep Analysis** (cohesion 0.05) | **Clustering noise.** Three well-named files (deep_analysis / confidence / guardrails) with a clean consume chain, byte-identity tested. No split. |
| **Reference Tuning & Exports** (0.07) | **Community framing is noise** (inflated by two 57-LOC private CLI wrappers) — but the churn lens independently found the real tangle inside it: the fixture_matrix ↔ reference_tuning helper duplication (Finding 4). |
| **Native Bridge Types** (0.08) | **Noise.** The bridge deliberately re-exports shared types so the FFI mirrors desktop; output parity is test-pinned. Splitting risks the locked contract. |
| **AuditionController (53 edges, #1 god node)** | **Dismissed — not even in this crate.** It is a Swift symbol (apps/iphone-native), reaching the graph only via handoff docs. Zero hits in src-tauri. Don't chase it. |
| **default_master_settings() (36 edges)** | **Real, but it's copy-paste, not architecture** — six duplicated test builders. Resolved by Finding 8. |
| **MasteringSettings (31 edges)** | **God node by design** — THE shared payload across desktop, FFI, persistence, and the wire. Any seam breaks the locked contract. No action. |
| **settings_with_intensity() (27 edges)** | **Graph artifact.** A single test-only helper (audio.rs:1930, cfg(test)) with ~30 in-file uses — fan-out inside one file, not coupling. Intentionally bypasses the compressor for live-coeff RMS tests. No action. |
| **Desktop ↔ iPhone FFI seam** | **Surface is near-minimal** (11-symbol import line, lib.rs:6-11; the one indirect import via live_stream is intentional). Bridge-side pinning is strong (sample-level parity, settings-contract, gate-off inertness tests). Desktop-side pinning exists transitively; gaps are the four unpinned wire keys (Finding 9) and the missing labeled chokepoint (Finding 10). The C header as a third hand-maintained copy is a watch-item (hunch), not yet demonstrated pain. |
| **21 "import cycles"** | All are `mod tests { use super::* }` single-file artifacts. Not real. |

---

## Appendix — unverified hunches

Observations without enough evidence to rank. Recorded so they aren't
re-derived from scratch next survey.

- `forceWysiwygRef` + the WYSIWYG effect is the same effect-based-coupling
  shape that produced the Album→Standard trap; candidate for folding into
  Finding 1's reducer.
- `useViewMode`'s `hadPriorSession: boolean | null` tri-state is a smaller
  instance of the same illegal-state class.
- localStorage `yes-master:view-mode` could disagree with in-session view on
  project-open if the entry guard fires after the read — wants a focused
  test, no repro yet.
- App.css (5,801 lines, 26 commits, co-changes 10× with StandardView) likely
  carries stale selectors from the May restyle slices (see "Restructure
  2026-05-14 slice A/B" comments ~:1976/:2170). Needs a coverage tool pass
  (e.g. PurgeCSS dry-run) before any deletion — declined as a manual task.
- useTrackMaster.ts (2,149 lines, 19 commits) is trending toward the App.tsx
  monolith pattern. Plausible seams (album slice ~:1175-1272; history slice
  ~:271-792) are coupled through `restoreSnapshot`; the composed surface is
  pinned but slices aren't. If pain recurs: pin slice contracts first.
- Autosave debounce (useTrackMaster.ts:593-611) may write-amplify during
  slider drags (300 ms history coalesce vs 1.5 s autosave). Not measured.
- audio.rs has ~10 near-identical meter-slot `Arc::new(AtomicU32::new(0))`
  constructions; snapshot/swap ordering subtleties make this look-don't-touch
  until the engine lock loosens.
- `MeterFanout` (dsp.rs:1907-1909) may duplicate per-band GR slots on
  AudioPlayer (audio.rs:721-723) — DSP-adjacent, not traced.
- fixture_matrix.rs:132 and reference_tuning.rs:319 both inline the
  `SourceProfile::from_analysis + apply_resolved_confidence` dance that
  engine.rs:121 owns for production — small win, each lane's tests pin exact
  shapes; revisit alongside Finding 4 if convenient.
- The chain `compute_spectral_balance` → `WindowMetrics.high` →
  `DeepAnalysis.brightness` now exists only for tests/diagnostics; pinned by
  integration tests, so inline/simplify is not worth the expectation risk.
- Bridge transitively depends on the full `DeepAnalysis` shape via
  `Arc<DeepAnalysis>` (lib.rs:154) — shape changes compile but could drift
  confidence behavior; watch.
- live_stream.rs exposes 16 C functions pinned only by the hand-maintained C
  header; a wrong Swift-side cast fails as a render-thread crash, not a
  build error. Highest-consequence drift class with no current bug —
  overnight Job 3 territory.
- The C header (`include/yes_master_native_bridge.h`) is a third
  hand-maintained copy of the FFI contract (downgraded finding — no
  demonstrated pain yet).
- AGENTS.md / CLAUDE.md / README.md triplicate the verification-lane prose;
  the drift already cost the bridge lane once (downgraded — fix by keeping
  them in sync when Finding 20 lands).
- `MasteringSettings.album` always-None on the track path is named
  serialized-contract bloat in two reviews; Finding 2's gate makes the dead
  field visible either way.
- bindings.ts mirrors only the FE-relevant subset of types.rs (good), but
  `CommandError`'s named Rust variants reach the FE as opaque strings —
  probably intentional; owner question, not a finding.
- audio.rs comment at :3185-3189 ("Reproduces the realtime stutter…") is
  historical test context, not leftover instrumentation; a final grep pass
  after Finding 20 lands would close jump-fix item 4 conclusively.

---

## Recommendation layer

### Quick Wins — one greenlightable batch (all S, low risk)

Commit-sized, independently green, in suggested order:

1. **Finding 0** — fix the 8 `contracts.rs` constructors, *with* **Finding
   3**'s golden receipt snapshot in the adjacent commit.
2. **Finding 11** — delete activeModifierChips/Summary (dead FE code).
3. **Finding 17** — Waveform re-export shim.
4. **Finding 16** — stale `selectedAnalysis` dep.
5. **Finding 18** — chrome copy lift.
6. **Finding 15** — `now_iso()` unification.
7. **Finding 13** — delete `analyze_tracks_core_lite`.
8. **Finding 12** — delete legacy `process_sample`.
9. **Finding 8** — tests/common settings builder.
10. **Finding 7** — compressor-table drift pin.
11. **Findings 9 + 10** — bridge wire-key tests + desktop canary (coordinate
    with the Job 3 branch).
12. **Findings 20 + 21** — doc syncs.

Everything in this batch is deletion, test addition, or doc truth-telling —
no behavior moves.

### Recommended Phase 2 sequence

| Step | What | Why this order |
| --- | --- | --- |
| 0 | Finding 0 (+3) | Nothing else can verify until `cargo test` compiles. The snapshot lands while the wound is fresh. |
| 1 | Quick-win batch (above) | Shrinks the noise floor; each commit tiny and green; deleting `process_sample` first also halves Finding 14's knee sites. |
| 2 | Finding 2 (wire-contract drift gate) | Pin contracts **before** moving code. The gate then guards every later step for free. |
| 3 | Finding 4 (evidence-lane helpers) | Top Rust tangle; verified byte-identical by the slow fixture lane, which is runnable again after step 0. |
| 4 | Finding 5 then 6 (App.tsx extractions) | Mechanical moves behind test-pinned boundaries; doing them before step 5 means the state-machine diff lands in a ~1,800-line App.tsx instead of a 3,000-line one. |
| 5 | Finding 1 (view/mode state machine) | The largest behavioral-risk FE change goes last, against a smaller file, with the transition table pinned by the existing 9 scenarios plus new table tests. |
| 6 (optional) | Finding 14 (dsp.rs de-dup) | Engine-internal; only with the byte-identity gate, only if the owner wants the soft-knee pre-positioning now. Skipping it costs nothing until the knee correction is ever approved. |
| owner-timed | Finding 19 (AdaptiveReadout gate) | Owner picks the flag's home; schedule before release-gate, independent of everything above. |

Dependencies: 0 → everything; 12 → 14; 2 de-risks 5/6/1 (any accidental
contract touch fails the gate). Every step is a small, independently green
commit per repo convention; no branches required by this plan beyond what
the owner prefers.

### Decline List

Surveyed and judged not worth doing — one line each so silence is
informative:

- **Split the Confidence & Deep Analysis community** — cohesion 0.05 is a
  clustering artifact; three clean files, byte-identity-tested.
- **Split Reference Tuning & Exports community** — inflated by two 57-LOC
  CLI wrappers; the real tangle is Finding 4.
- **Split Native Bridge Types community** — deliberate FFI mirror design;
  splitting risks the locked contract.
- **Touch AuditionController** — Swift symbol on the iPhone side; not in
  this crate.
- **Carve MasteringSettings into seam-aware types** — god node by design;
  THE shared payload.
- **Refactor settings_with_intensity()** — single test-only helper; graph
  fan-out artifact; preset-scaling math is locked anyway.
- **"Fix" the 21 import cycles** — all `mod tests { use super::* }`
  artifacts.
- **Rewrite useTrackMaster.ts internals / split sub-hooks** — composed
  surface is pinned, slices aren't, and `restoreSnapshot` couples them;
  pin-first if pain recurs (refuted as proposed).
- **Split App.css by feature area** — no test pins CSS; taste-only without a
  coverage signal.
- **Extract Sidebar / TrackMaster from App.tsx** — unpinned exports, no
  co-change signal; would be pin-first busywork.
- **Per-band compressor tab strip → select** — taste; current strip is
  test-pinned.
- **Inline `withSourceLufs` into the chain dispatcher** — would erase a
  deliberately tested pure-helper boundary.
- **Unify 6-band / 31-band spectral balance** — analysis.rs:376 comment says
  the copy is deliberate; both golden-pinned.
- **Consolidate MeterFanout / GR slots** — DSP-adjacent, unpinned, audio-
  thread ordering risk.
- **Derive `recommended_universal` helper** — duplication exists only in
  test fixtures; production derivation already single-sited.
- **Shared case-list abstraction for the evidence lanes** — the case lists
  are intentionally divergent; abstraction for a hypothetical need.
- **Rewrite audio.rs internals** — 3,620 lines but only 6 commits/192; low
  churn pressure, maximal engine risk.
- **Extract the bridge's settings-construction block** — the Default-spread
  defense comment (lib.rs:235-238) is load-bearing context for the 27edcf4
  bug class; extraction hides it, zero churn since.
- **Remove the bridge's Default-spread defense** — it *is* the fix for
  27edcf4's bug class.
- **Rename the `yes_master_lib` package alias** — zero payoff, churns every
  bridge import.
- **Whole-struct MasteringSettings JSON snapshot** — brittle (every
  legitimate field re-blesses it); superseded by Finding 2's key-set gate.
- **Audit live_stream.rs realtime wiring now** — engine-adjacent on the
  audio thread; belongs to the dedicated FFI test-hardening effort (Job 3),
  not this pass.
- **Merge album_render's `sanitize_for_filename` with
  `sanitize_path_part`** — refuted: functionally distinct sanitizers.
- **Delete the unused `AlbumCharacter` TS type** — refuted: deliberate
  forward surface from the drift-fix commit, kept intentionally.
- **Backend-derive source_profile / retire the TS injector** — already
  shipped; settings-transitions.ts:197-208 is the tombstone.
- **Add `evict_source_profile` and wire to removeTrack** — already shipped
  end-to-end with tests on both sides.
- **Welch-average the 6-band tonal FFT** — already fixed
  (analysis.rs:387-457) with a regression test.
- **C-header drift check** — third-copy risk is real but no demonstrated
  pain; recorded as a hunch.
- **De-duplicate the verification-lane prose across AGENTS/CLAUDE/README** —
  process discipline, not a refactor; handle inside Finding 20's commit.
- **Strip large preset PNGs / CSP / a11y sweep** — real hygiene, not
  refactor scope for this pass.

---

*Survey date 2026-06-10, HEAD `c22d71a`. Phase 2 executes only after owner
approval; the engine lock applies to every step above without exception.*
