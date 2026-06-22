> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Repo-wide confidence/archaeology audit; consolidated into docs/reviews/2026-06-16-repo-wide-review.html rev 3. _(Status: SUPERSEDED.)_ See docs/CHANGELOG.md for the project ledger.

# YES Master Global Confidence Audit - Agent Findings - 2026-06-16

> **Merged 2026-06-16:** this audit's findings have been verified against current
> code and consolidated (deduped) into the single audit of record,
> `docs/reviews/2026-06-16-repo-wide-review.html` (rev 3). All P1/P2 items here
> appear there — the Album Save/Open P1 as #19, the Preview-LUFS P1 as #5b, the
> partial-analysis P2 as #20, and the docs/CI/archaeology items under
> "Process, CI & docs drift." This file is retained for provenance. One
> correction: this audit listed `.gitignore` as a non-finding, but the
> consolidated report (#12) shows `test-output/*.csv|*.json` + subfolders are
> trackable.

Read this as an execution brief for follow-up agents. It is not a request to fix
everything at once. Validate each item against current code before changing it,
because this repo moves quickly.

## Scope And Rules

- Repo: `C:\Users\SM - Dan\Documents\GitHub\yes-master`
- Audit type: repo-wide confidence and archaeology audit.
- Source of truth: current code plus `docs/PRODUCT.md`,
  `docs/APP_BEHAVIOR.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md`, and
  `docs/RELEASE_STABILIZATION.md`.
- Historical handoffs, old plans, and old reviews are evidence, not active spec.
- Do not change sound/preset calibration from stale prose. Taste-dependent work
  needs an owner listening note.
- If a finding is objective, add a mechanical test with the fix.

## Recommended Work Order

1. Fix Album Save/Open persistence for Album-panel choices.
2. Fix desktop live Preview LUFS to measure at rendered sample rate.
3. Surface partial analysis failures in import/open-project recovery.
4. Add or explicitly document the Android ARM64 native verification gate.
5. Add AlbumPanel fold-down receipt visibility.
6. Clean docs drift that misroutes agents: README, APP_BEHAVIOR gap, iPhone
   handoff pointer, Android spec status, adaptive next-steps status.
7. Rebaseline CSS selector inventory and delete verified-dead blocks.

## Material Findings

### P1 - Album Save/Open Drops Album Panel Choices

- Bucket: Objective Bug; Current Contract Drift; Test Gap
- Evidence:
  - `src/hooks/useTrackMaster.ts:62` builds `ProjectState` with mode, tracks,
    track settings, `album_intent`, overrides, and timestamp only.
  - `src/bindings.ts:456` and `src-tauri/src/types.rs:874` have no project
    fields for album arc, album title, album intensity, album sample rate, or
    album bit depth.
  - `src/hooks/useTrackMaster.ts:1447` stores `albumArcKind`, `albumIntensity`,
    `albumTitle`, `albumSampleRate`, and `albumBitDepth` as live React state.
  - `src/hooks/useTrackMaster.ts:1501` uses those live values for album export.
  - `docs/APP_BEHAVIOR.md:91` says Open Project restores state.
- Current code reality: An Album project can reopen without the Album-panel
  choices that control the next export.
- Why it matters: A user can reopen an album session and unknowingly export a
  different arc/title/delivery format from the one they saved.
- Next action:
  - Extend project schema with Album-panel choices.
  - Preserve backwards compatibility for old `.ams.json` files.
  - Add frontend Save/Open round-trip coverage and Rust schema/default tests if
    shared types change.
- Verification:
  - `npm test`
  - `cd src-tauri; cargo test --target-dir target\codex-rc`
  - If shared Rust project types change, run iPhone and Android bridge lanes from
    `docs/TESTING.md`.

### P1 - Desktop Live Preview LUFS Measures At Source Rate

- Bucket: Objective Bug; Current Contract Drift
- Evidence:
  - Shared helper in `src-tauri/src/engine.rs:396` renders the preview landing
    window and measures at `rendered_sample_rate`.
  - Test at `src-tauri/src/engine.rs:1217` pins rendered-rate preview landing.
  - Offline render resamples before EBU measurement at `src-tauri/src/engine.rs:832`.
  - Desktop live path `src-tauri/src/audio.rs:1059` uses
    `export_landing_gain_lin_for_preview`, which processes the window at source
    rate and initializes EBU with `sample_rate` at `src-tauri/src/audio.rs:1101`.
  - iPhone live audition routes through shared `engine::preview_landing` at
    `apps/iphone-native/rust/src/live_stream.rs:727`.
- Current code reality: Desktop live WYSIWYG audition can disagree with export
  on non-44.1 kHz sources.
- Why it matters: The audition/export trust contract is weakest exactly where
  sample-rate conversion is involved.
- Next action:
  - Route desktop live Preview LUFS through the shared helper or duplicate its
    SRC behavior exactly.
  - Add a desktop audio-worker parity test for 48 kHz or 96 kHz source landing
    to a 44.1 kHz render target.
- Verification:
  - `cd src-tauri; cargo test --lib --target-dir target\codex-rc`
  - Run slow fixture lane before merging if DSP/export behavior changes.

### P2 - Partial Analysis Failures Are Stderr-Only

- Bucket: Current Contract Drift; Test Gap
- Evidence:
  - Backend partial-success policy returns successes and logs failures only with
    `eprintln!` in `src-tauri/src/engine.rs:130`.
  - Import appends all imported tracks before analysis returns at
    `src/hooks/useTrackMaster.ts:1135`.
  - Open Project only flags recovery when `api.analyzeTracks` rejects at
    `src/hooks/useTrackMaster.ts:2275`.
  - Existing recovery test covers whole-call rejection at
    `src/hooks/useTrackMaster.integration.test.tsx:980`, not partial success
    with missing result IDs.
  - `docs/APP_BEHAVIOR.md:91` says Open Project reports recovery issues needing
    user action.
- Current code reality: One bad/moved source in a multi-track batch can leave an
  imported/restored track unanalyzed without visible feedback.
- Why it matters: The session can look clean while important analysis state is
  missing.
- Next action:
  - Return structured partial failures from the Tauri command, or detect
    requested IDs absent from returned results on the frontend.
  - Show calm per-track recovery feedback and keep retry/reanalyze paths clear.
- Verification:
  - Add hook integration tests for import and Open Project partial-success cases.
  - Add Rust command/core coverage if the backend return type changes.

### P2 - Android ARM64 Native Gate Is Not CI-Enforced

- Bucket: Test / Verification Gap
- Evidence:
  - `docs/TESTING.md:47` requires `cargo ndk -t arm64-v8a --platform 29 check`.
  - `scripts/verify-fast.ps1:127` runs the ARM64 check locally.
  - CI Android job only runs `./gradlew test` at `.github/workflows/ci.yml:232`.
  - Windows CI runs Android host Rust tests but not ARM64 `cargo ndk`.
  - Native alignment is hooked to JniLib merge tasks in
    `apps/android-native/app/build.gradle.kts:202`, not plain JVM tests.
- Current code reality: Android JNI link/package/alignment regressions can pass
  CI.
- Why it matters: Android now shares real adaptive logic and depends on native
  bridge correctness.
- Next action:
  - Add a CI job for Android Rust host tests plus `cargo ndk --platform 29 check`,
    or explicitly label the ARM64/package check as a required local-only gate.
- Verification:
  - Confirm CI actually exercises the same target as `docs/TESTING.md`.

### P2 - Above-Stereo Fold-Down Is Missing From Frontend Receipt

- Bucket: UI Verification Gap
- Evidence:
  - Backend fold-down test is `src-tauri/tests/album_sample_rate.rs:195`.
  - AlbumPanel derives only `upmixedChannels` where source channels are less than
    rendered channels in `src/components/AlbumPanel.tsx:82`.
  - Receipt renders "Upmixed source" at `src/components/AlbumPanel.tsx:215`.
  - Frontend test covers upsample/upmix only in `src/components/AlbumPanel.test.tsx:86`.
  - `docs/APP_BEHAVIOR.md:77` says above-stereo sources fold down to stereo.
- Current code reality: Backend report contains the information, but the visible
  receipt does not call out fold-down.
- Why it matters: The user review surface hides an important delivery conversion.
- Next action:
  - Add fold-down advisory copy.
  - Add a frontend receipt test with `source_channels: [6, 2]` and
    `rendered_channels: 2`.

### P2 - Current Docs Disagree On Private Fixture Matrix Status

- Bucket: Docs Drift; Current Contract Drift
- Evidence:
  - `docs/APP_BEHAVIOR.md:191` says the full private fixture matrix still needs a
    longer unattended run.
  - `docs/RELEASE_STABILIZATION.md:205` says the full local manifest completed on
    2026-05-28.
  - `docs/archive/RELEASE_EVIDENCE_2026-05-28.md:92` records the full 18-row completion.
- Current reality: Runner evidence is complete; listening signoff remains open.
- Why it matters: Agents may rerun the wrong gate.
- Next action:
  - Update APP_BEHAVIOR gap wording to say listening signoff remains, not runner
    completion.

### P2 - README Uses Old Standard Style Names

- Bucket: Docs Drift
- Evidence:
  - `README.md:19` says Balanced / Bright / Warm / Heavy.
  - `docs/PRODUCT.md:39` says Universal / Clarity / Tape / Oomph.
  - `src/lib/standard-mapping.ts:18` explains old IDs are internal only.
  - `src/lib/standard-mapping.test.ts:37` pins canonical user-facing labels.
- Current reality: User-facing Standard labels are Universal, Clarity, Tape, and
  Oomph.
- Why it matters: README is release-facing and can mislead reviewers/users.
- Next action:
  - Refresh README Standard wording.

### P2 - Album Delivery Comments Preserve Pre-Wave-9 Limitations

- Bucket: Stale Code / Comment Debt; Docs Drift
- Evidence:
  - `src-tauri/src/engine.rs:599` says mixed sample-rate/channel albums fail and
    resampling is deferred.
  - `src-tauri/src/types.rs:242` says Album Master requires matching source
    rates for the release-candidate pass.
  - `src-tauri/src/album_render.rs:584` resamples to album delivery rate.
  - `src-tauri/tests/album_sample_rate.rs:134` tests mixed-rate rendering.
  - `docs/APP_BEHAVIOR.md:79` and `docs/RELEASE_STABILIZATION.md:29` say mixed
    rates/channels now render.
- Current reality: Behavior is correct; comments are stale.
- Why it matters: Misleading comments sit near core rendering contracts.
- Next action:
  - Update or delete stale comments.

### P3 - `verify:fast` Script Name Does Not Match Testing Docs

- Bucket: Verification Docs Drift
- Evidence:
  - `package.json:15` maps `verify:fast` to `scripts/verify-fast.ps1`.
  - `scripts/verify-fast.ps1:3` defaults `Lane = "all"`.
  - `scripts/verify-fast.ps1:142` runs frontend, Rust, iPhone, and Android lanes.
  - `docs/TESTING.md:3` defines Fast Lane as desktop-normal-change checks, with
    mobile lanes as conditional extras.
- Current reality: NPM "fast" means all lanes, including Android prerequisites.
- Why it matters: Agents can run the wrong command or misread an Android
  prerequisite failure.
- Next action:
  - Rename the script, or clarify docs and package script labels.

## Historical Archaeology Candidates

- `docs/archive/AGENT_WORK_AND_REVIEW_QUEUE_2026-06-05.md` still presents itself as an
  active task queue at line 444. Classification: intentionally superseded.
- `docs/ANDROID_NATIVE_SPEC.md` still says A4 is ongoing and process-death
  restore is absent. Classification: intentionally superseded by Wave 9.
- `docs/IPHONE_APP.md:5` points readers to `apps/iphone-native/HANDOFF.md`, but
  that handoff says it is historical and contains stale controls. Classification:
  docs drift / owner cleanup.
- `docs/ADAPTIVE_DSP_NEXT_STEPS.md` still frames Tier-1 finish as remaining and
  says `stereo_width` is currently unused. Classification: unclear / owner
  decision needed for how much of this doc remains live.
- Manual Listening Gate, Reference Retune listening, Oomph listening, and
  already-mastered matrix listening signoff are open by design. Classification:
  current verification backlog, not bugs.
- One-pole/soft-knee hoist and tauri-specta remain parked. Classification:
  intentionally deferred.

## Stale Code / CSS / Comment Debt

- `src/App.css` is large and high-churn. Confirmed orphaned or historical
  selector families include:
  - `.empty-state-glyph` at `src/App.css:679`
  - `.analysis-summary-icon` / `.analysis-summary-status` around
    `src/App.css:1022`
  - `.user-preset-empty` at `src/App.css:1845`
  - `.quality-summary-*` around `src/App.css:2965`
  - `.workspace-section-label` at `src/App.css:3651`
  - `.advanced-panel-slot` / `.advanced-slot-body` around `src/App.css:3781`
- Existing `src/App.layout-css.test.ts` pins layout expectations but does not do
  a full dead-selector inventory.
- `graphify-out/graph.json`, labels, and report are tracked intentionally per
  `.gitignore`; treat them as owner-approved artifacts unless separately asked
  to reduce repo size.

## Non-Findings Checked

- Export overwrite: normal render paths reject source overwrite and uniquify
  prior renders. Tests exist around explicit output and album path uniqueness.
- Album mixed-rate/mixed-channel backend: shipped and covered. The issue is UI
  receipt copy and stale comments, not backend rendering.
- Standard fixed export: code/test reality matches current product docs.
- Original/Mastered switching: playhead preservation is wired through
  `playWithKind(..., positionSec)` and current transport time.
- Analysis progress: batch identity exists and stale progress is filtered. The
  remaining issue is partial failure visibility.
- Private audio/renders: `.gitignore` covers private fixtures and generated
  renders; the tracked test-output exception is intentional.

## Suggested Slice Boundaries

- Slice A: Album project persistence only.
- Slice B: Desktop Preview LUFS rendered-rate parity only.
- Slice C: Partial analysis failure feedback only.
- Slice D: Android verification docs/CI only.
- Slice E: Album fold-down receipt UI only.
- Slice F: Docs drift cleanup only.
- Slice G: CSS inventory plus verified dead CSS deletion only.
