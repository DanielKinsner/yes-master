# YES Master Agent Work And Review Queue

> **Superseded:** retained for archaeology only. Do not execute this as the
> active work queue; use `AGENTS.md`, current code, and
> `docs/RELEASE_STABILIZATION.md` for the live stabilization state.

Purpose: give Claude or another agent a safe, current, human-out-of-loop task list
for YES Master, plus an adversarial review prompt to run before implementation.

This is not a product spec and it is not permission to tune sound. Use the
current code plus:

- `docs/PRODUCT.md`
- `docs/APP_BEHAVIOR.md`
- `docs/ARCHITECTURE.md`
- `docs/TESTING.md`
- `docs/RELEASE_STABILIZATION.md`
- `docs/ADAPTIVE_DSP_NEXT_STEPS.md`
- `docs/HANDOFF_2026-06-04_ADAPTIVE_DSP_TIER2_PHASE_B_CONFIDENCE.md`

## Current State

- `main` contains Tier-1 adaptive guardrails and Tier-2 Phase B confidence
  machinery.
- Confidence gating is still off by default. Do not turn it on by default.
- Confidence is backend/devtools-only. Do not add an everyday-user confidence UI.
- The 31-band per-window low/harsh/sibilant/air/tilt detail now feeds Phase-B
  confidence. Do not re-implement that work.
- Desktop and iPhone have truthful staged analysis/render progress copy. This is
  timed UI state, not backend per-stage telemetry.
- Owner by-ear calibration is still pending. Merged does not mean validated.
- The full fast verification lane is available as `npm run verify:fast`.

## Prime Directive

Agents may do objective, bounded, self-verifiable work. Agents may not make
taste, calibration, or product-positioning decisions.

Do not touch these without Dan:

- Guardrail constants or deadbands in `src-tauri/src/guardrails.rs`.
- Confidence thresholds, dispersion scales, or adaptive tuning constants in a way
  that changes sound.
- Density-cap semantics.
- Tilt-vs-reference brightness metric.
- Whole-track Welch spectral window swap.
- Activating `stereo_width` as a width co-trigger.
- Total-loudness-loss budget behavior.
- Simple Mode product or UX decisions.
- Subjective preset/DSP retuning.
- Turning confidence gating on by default.
- README release positioning.

If a task discovers an objective bug in one of these areas, document it with a
minimal reproduction and stop. Do not tune around it.

## Agent Operating Rules

- Start with an adversarial review of current `main` before changing code.
- Report only material findings: bugs, wiring drift, false product claims,
  broken contracts, missing verification, or stale docs that could mislead work.
- Make very small commits. One concern per commit.
- Commit directly to `main` only after the slice is verified.
- Push after each successful small commit.
- Do not batch unrelated refactors, tests, and docs into one large commit.
- Do not rename public symbols, move state, or change behavior while doing
  extraction/refactor work unless that is the explicit task.
- If a finding is objective, add a mechanical test.
- If a finding is taste/listening-dependent, capture it as a note and do not
  change calibration.
- Keep private audio, rendered private masters, private fixture ledgers, and
  package/build artifacts out of git.

Recommended commit shape:

```text
review: triage global adaptive findings
test(iphone): lock render parity with desktop
refactor(app): extract track header component
chore(verify): improve fast lane summaries
docs(queue): refresh open agent tasks
```

## Required Verification

For docs-only changes:

```powershell
git diff --check
```

For frontend-only changes:

```powershell
npm test
npm run build
```

For shared Rust, Tauri commands, backend contracts, DSP/export behavior, or the
iPhone bridge:

```powershell
npm run verify:fast
```

For private DSP/export merges before release-stable claims:

```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

Swift/Xcode simulator tests require an Apple environment and are not covered by
the Windows fast lane. Do not pretend they ran on Windows.

## Master Task List

### 1. Adversarial Current-State Review

Type: read-first, then fix only objective defects.

Goal: review recent commits and the current repo for broken wiring, stale claims,
and important objective gaps before taking on feature/refactor work.

Scope:

- Adaptive Phase B confidence wiring.
- 31-band adaptation input.
- Desktop staged progress UI.
- iPhone staged progress UI and Rust bridge parity.
- Verification script coverage.
- Handoff/backlog docs.

Allowed fixes:

- Broken tests or compile errors.
- Incorrect docs that would mislead another agent.
- Missing mechanical tests for already-implemented objective behavior.
- Wiring drift where the intended current behavior is unambiguous.

Not allowed:

- Sound tuning.
- New product surfaces.
- Turning gates on.
- Rewriting the adaptive design.

Oracle:

- A short review markdown or section in a handoff doc.
- Focused tests for objective fixes.
- `npm run verify:fast` if code/shared contracts changed.

### 2. Triage `GLOBAL-review.md`

Type: read-only/doc-only.

File: `docs/reviews/2026-06-03-adaptive-dsp-GLOBAL-review.md`

Task:

- Extract every concrete finding into a clean checklist.
- Tag each item as:
  - `objective-fix`
  - `needs-owner-ear`
  - `already-fixed`
  - `stale`
  - `unclear`
- Include the current code evidence for each tag.

Important correction: this file is tracked now. Older docs may call it
untracked/unread; treat that wording as stale.

Oracle:

- One markdown triage document.
- No code changes.
- `git diff --check`.

### 3. iPhone Render Parity Golden Tests

Type: objective test work.

Why it matters: the iPhone bridge reuses `yes_master_lib`, but bridge
orchestration can drift through parameter mapping, adaptive context resolution,
or export-only settings.

Task:

- Add render-path parity tests in `apps/iphone-native/rust`.
- Compare iPhone bridge render behavior against the desktop/shared Rust render
  path for identical input WAV and identical settings.
- Cover:
  - one representative preset
  - source-profile-present adaptive context
  - confidence gate off behavior
  - export Volume Match invariance, meaning render output should not change when
    a Volume Match-like option is present because exports force it off

Notes:

- iPhone already has some live-chain bit-for-bit tests. Do not duplicate those
  unless the new test covers a real gap.
- Do not tune presets or adaptive constants.

Oracle:

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

Run `npm run verify:fast` before pushing if shared Rust or contracts move.

### 4. Mechanical `App.tsx` Extraction

Type: behavior-preserving refactor.

Why it matters: `src/App.tsx` is large and is a drag on safe future work.

Task:

- Extract JSX/components only.
- Leave state and behavior where they are.
- Use existing product seams:
  - creative/main UI: preset, intensity, EQ, tone, saturation, width,
    compression, limiter, audition controls
  - judgment/delivery UI: quality checks, delivery profile, advanced controls,
    per-band compressor detail, format, export review
- One extraction per commit.

Hard constraints:

- Do not rename props or state.
- Do not change logic.
- Do not change styling/design.
- Do not optimize while extracting.
- If a bug is found, list it separately unless it blocks the extraction.

Oracle:

```powershell
npm test
npm run build
```

Recommended first slices:

- Track/header/status area.
- Waveform/audition controls.
- Center creative controls.
- Left rail navigation/sidebar.
- Keep right rail changes separate because it owns export/review behavior.

### 5. Desktop Staged Progress Tests

Type: objective frontend test work.

Task:

- Add explicit tests for the staged analysis progress introduced in
  `src/hooks/useTrackMaster.ts` and rendered through `src/App.tsx`.
- Assert:
  - the first analysis label/progress appears while analysis is active
  - progress advances while analysis is still active
  - progress clears after analysis completes or fails
  - rendering status remains separate from analysis status

Oracle:

```powershell
npm test
npm run build
```

### 6. Stable Primitive Characterization Tests

Type: objective test coverage.

Safe targets:

- `src-tauri/src/wav_writer.rs` dither and quantization behavior.
- `src-tauri/src/decode.rs` error cases and supported containers.
- `src-tauri/src/sample_rate.rs` conversion invariants.
- Limiter true-peak / intersample behavior in stable DSP primitives.
- Export path uniqueness and no-overwrite behavior.

Hard constraint:

- Do not add tests that freeze provisional guardrail/deadband/adaptive tuning
  constants. Those are not owner-validated yet.

Oracle:

```powershell
cd src-tauri
cargo test --target-dir target\codex-rc
```

Do not require a coverage report unless coverage tooling is added in the same
slice.

### 7. `deep_analysis.rs` Per-Window Allocation Optimization

Type: low-urgency perf cleanup.

Current known point: `src-tauri/src/deep_analysis.rs` has a TODO around the
per-window mono `Vec` allocation used to reuse the 6-band helper.

Task:

- Remove or reduce the per-window allocation only if output remains identical.
- Prefer a borrowed/iterator-based helper over changing analysis semantics.
- Do not change 6-band golden behavior.
- Do not change 31-band confidence behavior.

Oracle:

```powershell
cd src-tauri
cargo test --lib --target-dir target\codex-rc deep_analysis::tests
cargo test --lib --target-dir target\codex-rc confidence::tests
```

If a benchmark is used, record the before/after, but identical output matters
more than speed.

### 8. LRA `Option<f32>` Cleanup

Type: safe-ish contract cleanup; not fire-and-forget.

Why it matters: older reviews identify sentinel/fallback hazards around LRA and
P95-P10 dynamic range. Minimal guards already shipped, but a proper type cleanup
could make unknown values explicit.

Task:

- Convert unknown LRA / missing dynamic range behavior to explicit optional
  representation only if the current contracts can be moved cleanly.
- Preserve current audible behavior unless fixing an objective sentinel bug with
  a test.
- Update TypeScript bindings and iPhone bridge if shared types move.

Hard constraints:

- Do not change density thresholds.
- Do not tune density behavior by ear.
- Do not alter adaptive constants.

Oracle:

```powershell
npm run verify:fast
```

### 9. Verification Script Ergonomics

Type: tooling-only.

Task:

- Improve `scripts/verify-fast.ps1` logging or add smaller wrappers such as:
  - `verify:frontend`
  - `verify:rust`
  - `verify:iphone`
- Keep the existing full lane intact.
- Do not hide failing commands.

Oracle:

- The changed script runs successfully.
- If package scripts are changed, run the affected scripts.

### 10. Handoff/Doc Drift Cleanup

Type: docs-only unless objective code drift is found.

Task:

- Review handoffs and active docs for stale wording that would mislead another
  agent.
- Especially check:
  - claims that 31-band adaptation input is still unimplemented
  - claims that `GLOBAL-review.md` is untracked
  - claims that progress UI is backend stage telemetry
  - claims that confidence is user-facing
  - old verification commands that omit the iPhone Rust bridge

Oracle:

```powershell
git diff --check
```

## Copy-Paste Prompt For Claude: Review First

```text
You are working in `.` on main.

Read these first:
- AGENTS.md
- docs/PRODUCT.md
- docs/APP_BEHAVIOR.md
- docs/ARCHITECTURE.md
- docs/TESTING.md
- docs/RELEASE_STABILIZATION.md
- docs/ADAPTIVE_DSP_NEXT_STEPS.md
- docs/HANDOFF_2026-06-04_ADAPTIVE_DSP_TIER2_PHASE_B_CONFIDENCE.md
- docs/AGENT_WORK_AND_REVIEW_QUEUE_2026-06-05.md

Before implementing anything, do an adversarial current-state review. Focus on
recent adaptive/31-band/progress/iPhone/verification changes and the whole repo
surface where relevant. Report only material findings: objective bugs, broken
wiring, stale or false docs, missing tests for implemented behavior, or contract
drift.

Important boundaries:
- Confidence remains backend/devtools-only.
- Confidence gating remains off by default.
- 31-band confidence input is already implemented; do not re-implement it.
- Do not tune guardrail constants, confidence constants, deadbands, presets, or
  subjective DSP behavior.
- Do not make Simple Mode or README release-positioning decisions.
- If something requires listening or taste, document it and stop.

If you find objective bugs or important objective additions, fix them in very
small commits. One concern per commit. Verify each slice before committing. Push
each commit to main after it is green.

Use these verification lanes:
- Docs-only: git diff --check
- Frontend-only: npm test; npm run build
- Shared Rust/Tauri/iPhone bridge/DSP/export: npm run verify:fast

After the review, pick one safe task from docs/AGENT_WORK_AND_REVIEW_QUEUE_2026-06-05.md
and execute it. Prefer iPhone render parity tests or mechanical App.tsx extraction
unless your review finds a higher-priority objective bug.
```

## Copy-Paste Prompt For Claude: Execute One Task

```text
You are working in `.` on main.

Use docs/AGENT_WORK_AND_REVIEW_QUEUE_2026-06-05.md as the active task queue.
Pick exactly one task unless I specify otherwise. Keep the change small.

Rules:
- One concern per commit.
- No taste, calibration, preset, deadband, confidence-threshold, or Simple Mode
  product changes.
- Do not expose confidence in everyday UI.
- Do not turn confidence gating on by default.
- If you discover a separate bug, document it instead of mixing it into the
  current task.
- Verify before commit.
- Commit and push to main after each green slice.

When done, report:
- commit hash
- files changed
- verification commands run
- anything intentionally left open
```

## Recommended Order

1. Run the adversarial current-state review prompt.
2. Triage `GLOBAL-review.md` if review context is still fuzzy.
3. Add iPhone render parity golden tests.
4. Add desktop staged progress tests.
5. Start mechanical `App.tsx` extraction, one tiny slice at a time.
6. Add stable primitive characterization tests.
7. Only then consider LRA `Option<f32>` cleanup or deep-analysis allocation
   optimization.

Dan should do the by-ear adaptive calibration pass before any further behavior
that stacks on adaptive guardrail taste.
