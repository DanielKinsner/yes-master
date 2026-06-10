# Handoff — Consolidated Refactor Backlog Executed (2026-06-09)

The full backlog from `docs/reviews/2026-06-10-consolidated-refactor-backlog.md`
(the four-survey consolidation) executed in one pass: 23 commits, every one
independently green, engine lock respected throughout. Owner decisions taken
before execution: untrack the graphify cache (P1a), engine-adjacent deletions
only (P2 minus the dsp hoist), AdaptiveReadout behind a localStorage flag
(P3). Per-item commit hashes are recorded in the backlog doc's Execution
record table.

## What changed, by layer

### Verification baseline (was broken)
- `cargo test` compiles and runs again — first time since `fdbcb34` (8
  `ExportReport` literals in contracts.rs were missing
  `measurements_are_rendered`).
- New golden snapshot pins the receipt triple
  (`src-tauri/tests/golden/export_report.json`); defaulted-field additions
  now fail CI as a reviewable JSON diff.

### New contract gates (all smoke-tested by mutation)
- **Rust↔TS bindings drift gate** — `src-tauri/tests/wire_shape.rs` emits
  canonical samples of 9 wire types into `src/wire-samples.json`;
  `src/bindings-drift.test.ts` compares key sets against bindings.ts at
  compile time (`npm run build` fails with the drifted key named).
  Regenerate after intentional changes: `YES_MASTER_UPDATE_GOLDEN=1`.
- **Display-mirror tripwires** — `src/preset-mirrors.json` (generated from
  the dsp.rs calibration) pins the compressor table AND the SignalChain
  stereo-width baselines (previously zero pinned values);
  `src/standard-mapping-parity.json` pins the style→preset contract from
  both language sides (desktop tables + bridge `native_preset`).
- **Bridge wire-key pins** — two tests call the real FFI entry points and
  assert every key Swift decodes (`true_peak_dbtp`, `dynamic_range_lu`,
  `sample_rate`, `bit_depth` were unpinned; a serde rename used to crash the
  app at runtime).
- **Bridge-surface canary** — `src-tauri/tests/iphone_bridge_surface.rs`
  fails the desktop fast lane with a message naming the bridge when an
  imported symbol changes.

### Structure
- **`src-tauri/src/evidence_lanes.rs`** — the six byte-identical helpers and
  the adaptive settings resolver shared by fixture_matrix/reference_tuning
  (the #1 Rust co-change pair). New cross-lane equivalence test pins that
  both lanes resolve identical `source_profile`/`source_confidence` for the
  same source, gate-on and gate-off. Slow fixture lane ran green with the
  real private fixture present.
- **App.tsx 3,014 → ~1,430 lines** (at survey HEAD → now):
  `components/ExportReceiptCard.tsx` (+ smoke test, exported formatters),
  `components/AdvancedPanel.tsx`, `components/fields.tsx` (shared field
  primitives; second consumer is the Standard EQUALIZER block),
  `lib/chrome-content.ts` (Settings/Help copy). The three AdvancedPanel test
  files keep their `./App` import via a named re-export.
- **`lib/navigation-machine.ts` + `hooks/useNavigationMachine.ts`** — the
  capstone. One pure reducer over `{unresolved, standard-track,
  advanced-track, advanced-album, return-confirm-pending}` replaces
  useViewMode + the returnConfirm useState + both reactive App guards.
  Leaving Album on an explicit return happens at the dispatch site, so the
  2a78f4a silent-trap class is structurally impossible. 40 table-driven
  tests including an exhaustive no-transition-into-standard-under-album
  sweep; all 9 pre-existing transition scenarios unchanged.
- **`hooks/useTrackMaster.ts`** — `buildExportReport` extracted to
  `lib/export-receipt.ts` (tests pin `measurements_are_rendered = m != null`
  — it gates the backend's `target_not_reached` advisory);
  `buildProjectState()` dedupes the byte-identical autosave/Save-As
  literals.

### Deletions (owner-approved engine-adjacent included)
- Legacy `MasteringChain::process_sample` (147 lines, documented wrong-EQ
  trap, zero callers) — byte-identity SHAs unchanged; the frame-path half of
  its low_mid divergence test survives as `frame_path_applies_low_mid`.
- `analyze_tracks_core_lite` + gate test + bench (never-shipped mobile
  battery path; the bridge uses the deep-capable entry).
- Dead `activeModifierChips`/`activeModifierSummary` + self-tests (134 LOC).
- The App.tsx Waveform re-export shim; the stale `selectedAnalysis` dep.
- `useViewMode` + its self-test (superseded by the machine; the persisted
  store stays pinned by `view-mode.test.ts`).
- 156 machine-local graphify files untracked (cache/, manifest.json,
  cost.json); graph.json / GRAPH_REPORT.md / labels stay shared.

### Behavior changes (the only two, both sanctioned)
- **AdaptiveReadout hidden by default** (owner TODO closed). Re-enable for
  calibration: `localStorage.setItem("yes-master:debug:adaptive-readout", "1")`.
- `reference_tuning` report timestamps now use the canonical `now_iso`
  format (`+00:00` instead of millis+`Z`) — tooling artifact, not audio.

## Verification record (final sweep, HEAD `7fdae8b`)
- `npm test` 336/336 (35 files) · `npm run build` · `npm run build:windows`
  (NSIS bundle produced)
- `cargo fmt --check` · `cargo clippy --all-targets -D warnings` ·
  `cargo test --lib` 304/304 · `cargo test` full, exit 0
- Bridge lane: `cargo check --all-targets` + 37/37 tests
- Slow lane (`AMS_RUN_REAL_FIXTURE=1`) ran green during B3.1 with the real
  fixture present
- Engine lock: preset_byte_identity SHAs, preset_signature, distinctness,
  loudness balance all unchanged in every run

## Open / waiting on owner
- **Listening signoff** (owner ear, per the refreshed CLAUDE.md queue):
  manual listening gate, reference-retune listening (Oomph least-matched),
  already-mastered matrix signoff. Nothing in this pass should change
  rendered audio — the byte-identity gates agree — but the navigation
  machine and Standard/Advanced flows deserve a hands-on pass: open project,
  restore session, Album → Back to Standard (clean + dirty), Reset &
  continue, Save-as-preset, cancel.
- **Parked**: F14 one-pole/soft-knee hoist (deferred; plans written in the
  Claude survey), P4 tauri-specta (B2.1 gate is the dependency-free answer).
- `churn_raw.txt` at repo root is untracked survey scratch — delete at will.
