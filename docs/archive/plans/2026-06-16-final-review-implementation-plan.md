> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Final repo-wide review execution map (Waves A-E); all slices complete with SHAs. _(Status: COMPLETE.)_ See docs/CHANGELOG.md for the project ledger.

# YES Master Final Review Implementation Plan - 2026-06-16

This plan turns `docs/reviews/2026-06-16-repo-wide-review-final.html` into an implementation queue for the next agent. The final HTML remains the evidence source; this file is the execution map.

## Current State

- This queue was executed on `main` in small pushed commits after the final review. Do not re-run completed slices below unless new evidence reopens them.
- The final review file is already saved as `docs/reviews/2026-06-16-repo-wide-review-final.html`.
- Pre-existing untracked review artifacts may appear in `git status`; do not stage them unless the user explicitly asks.
- Treat the current code plus the required product docs as live truth. Do not treat older handoffs or phase plans as canonical.

## Execution Status - 2026-06-17

Completed implementation slices:

- A1 #5b Desktop Preview-LUFS drift: `a0fc718`
- A2 #2 Album loudness arc: `e796977`
- A3 #3 preset parse-failure guard: `c6e7fe9`
- A4 #4 non-finite rendered sample rejection: `bdb5a97`
- A5 #6/#7 track-selection meter/ref state: `e65b781`
- B1 #12/#13 private-output ignores: `9e90893`
- B2 #14 album Volume Match tripwire: `642d7e7`
- B3 #15 Original/Mastered playhead tripwire: `ee15e14`
- B4 #18 album measured-LUFS sanitization: `2c4d2f5`
- C1 #19 album project choices persistence: `3e0d41d`
- C2 #20 partial-analysis visibility: `82752e9`
- C3 #9 near-end playback restart threshold: `b9c9d55`
- C4 #10 Standard off-grid loudness snap: `ce4aaf7`
- C5 #11 import metadata track selection: `bc33e53`
- C6 #16 Standard delivery label source of truth: `e6e36a7`
- C7 #17 DeliveryProfile Rust/TS parity pin: `1688da6`
- D1 #1 Android AAudio read-back/validation: `5428662`
- D2 #8 Android landing/release serialization: `800ba13`
- E1 #5 album source-path guard: `fb6d98b`
- E2 decode channel-count reconciliation: `f61195f`
- E3 mechanical docs/CI/process quick wins: `188f242`, `a1c7b36`, `cac7d2c`, `4009cae`, `147ef6d`, `e5e52e7`, `49dc621`

Remaining by design:

- Product-canonical doc refreshes remain owner-gated. Do not rewrite `docs/PRODUCT.md` or broader product canon from this plan without explicit owner approval.
- Manual listening gates remain outside this implementation plan: normal/already-mastered/long-source sweeps, Reference Retune listening notes, and already-mastered matrix listening signoff.
- The "Hold Until After Stabilization" list below remains parked cleanup/refactor work, not unfinished correctness work from this queue.

## Required Reading

Read these before making changes:

1. `AGENTS.md`
2. `docs/reviews/2026-06-16-repo-wide-review-final.html`
3. `docs/PRODUCT.md`
4. `docs/APP_BEHAVIOR.md`
5. `docs/ARCHITECTURE.md`
6. `docs/TESTING.md`
7. `docs/RELEASE_STABILIZATION.md`

## Suggested Skills

- `review` for re-validating a finding before touching code.
- `harden` for persistence, export-integrity, and private-data slices.
- `qa` for adding regression tests and mapping verification lanes.
- `handoff` if the work is paused mid-slice.

## Operating Rules

- Implement one slice at a time. Prefer one branch/commit per slice or tightly related pair.
- For each objective bug, write or identify a failing mechanical test before changing behavior.
- For UI/state fixes, add hook/component tests where practical; otherwise leave a manual verification note.
- Do not edit the final review HTML as part of implementation unless the user asks for report updates.
- Do not bundle Hold/cleanup refactors with correctness fixes.
- Preserve private-audio safety: do not add private audio, rendered masters, or path ledgers to git.
- When shared Rust types or Tauri command signatures change, run the iPhone bridge lane. When shared crate/facade/Android code changes, run the Android lane too.

## Priority Model

The queue is ordered by risk reduction per effort:

1. High-impact untracked desktop fixes that are small and testable.
2. Cheap tripwire tests for non-negotiables.
3. Privacy/gitignore guards.
4. Broader desktop correctness and persistence gaps.
5. Android crash-class items before mobile release.
6. Low-priority hardening, docs, CI, and cleanup.

Desktop v1 stabilization comes before mobile expansion. Android criticals gate Android/mobile release, but they are not desktop-v1 blockers unless the user says mobile is in scope for the current slice.

## Wave A - Start Here

### A1. #5b Desktop Preview-LUFS drift

Status: complete - `a0fc718`
Priority: first desktop fix  
Source: final review #5b  
Files likely involved: `src-tauri/src/audio.rs`, `src-tauri/src/engine.rs`

Problem:
The desktop live Preview-LUFS path measures landing at the source sample rate, while export and the shared `engine::preview_landing` path land at the render sample rate. A 48/96 kHz source exported to 44.1 kHz can sound different in preview than in the delivered master.

Implementation direction:
- Add a regression where live preview coeffs for a source-rate/render-rate mismatch must match `engine::preview_landing`.
- Route the desktop preview worker through the shared `engine::preview_landing` path, or through the same render-rate preview-window helper.
- Remove or reduce duplicated preview-window logic only if it is part of this parity fix.

Suggested failing test shape:
- Build an 8 second stereo 48 kHz source with enough high-frequency content to make resampling affect the landing measurement.
- Use `DeliveryProfile::Custom` with `advanced.target_sample_rate = Some(44_100)`.
- Assert `live_preview_coeffs(...).export_landing_gain_lin` equals `engine::preview_landing(...).gain_lin` within a tight epsilon.

Targeted verification:

```powershell
cd src-tauri
cargo test preview_landing --target-dir target\codex-rc
cargo test live_preview_coeffs --target-dir target\codex-rc
cargo fmt --check
```

### A2. #2 Album loudness arc discarded under default delivery profile

Status: complete - `e796977`
Priority: second desktop fix  
Source: final review #2  
Files likely involved: `src-tauri/src/album_render.rs`, `src-tauri/src/types.rs`

Problem:
`apply_album_shadow` writes the arc target into `advanced.lufs_offset_db`, but `MasteringSettings::effective_target_lufs()` lets a non-Custom `delivery_profile` win. The default album path is `StreamingUniversal`, so non-zero arc offsets can be ignored and tracks land flat.

Implementation direction:
- Preserve the original profile's effective target, ceiling, bit depth, and requested sample rate.
- Force the shadow settings into a Custom-equivalent landing target, or otherwise pass an explicit absolute target into the landing path.
- Keep album adaptive internals stripped as they are today.

Suggested failing test shape:
- In `album_render.rs`, call `apply_album_shadow` with `DeliveryProfile::StreamingUniversal` and `entry.arc_lufs_offset_db = 1.5`.
- Assert `shadowed.effective_target_lufs() == Some(-12.5)`.
- Also assert profile delivery intent is preserved: ceiling `-1.0`, bit depth `24`, sample rate `48_000` for a 96 kHz source.
- Prefer a render-level LUFS-shift test as a follow-up if cheap.

Targeted verification:

```powershell
cd src-tauri
cargo test album_shadow --target-dir target\codex-rc
cargo test album --target-dir target\codex-rc
cargo fmt --check
```

### A3. #3 Preset-file parse failure can wipe all user presets

Status: complete - `c6e7fe9`
Priority: third desktop fix  
Source: final review #3  
Files likely involved: `src-tauri/src/settings.rs`, `src-tauri/tests/contracts.rs`

Problem:
`save_user_preset` and `delete_user_preset` use `read_presets(&path).unwrap_or_default()`. A missing file should mean empty list, but a parse error should not. Today a malformed or schema-drifted preset file can be treated as empty and overwritten by the next save/delete.

Implementation direction:
- Keep missing file behavior as empty list.
- On parse/read error for an existing file, refuse the mutating operation or back up the unreadable file before writing.
- Surface a useful error to the caller. Do not silently replace unreadable data.
- Consider factoring path-level save/delete helpers so tests do not need a Tauri `AppHandle`.

Suggested failing test shape:
- Write invalid JSON or an old-schema preset payload to a temp `user_presets.json`.
- Attempt the path-level save or delete helper.
- Assert it returns `Err`.
- Assert the original bytes on disk are unchanged, or assert a backup exists and the new file is intentionally written only after backup.

Targeted verification:

```powershell
cd src-tauri
cargo test user_presets --target-dir target\codex-rc
cargo fmt --check
```

### A4. #4 Post-render NaN/Inf detection gap

Status: complete - `bdb5a97`
Priority: high, after A1-A3  
Source: final review #4  
Files likely involved: `src-tauri/src/engine.rs`, `src-tauri/src/exports.rs`, `src-tauri/src/wav_writer.rs`

Problem:
Rendered samples are not scanned for non-finite values before WAV writing. LUFS sanitization can convert non-finite measurements to a finite fallback, so the existing corrupt-render critical cannot reliably fire.

Implementation direction:
- Add a finite sample scan after DSP/render and before `write_wav`.
- Return a render failure or post-render critical if any sample is NaN/Inf.
- Keep `sanitize_lufs` behavior separate from sample integrity.

Suggested failing test shape:
- Create a narrow test helper or writer test that attempts to pass a buffer containing `f32::NAN` or `f32::INFINITY`.
- Assert render/export rejects before writing a successful master.

Targeted verification:

```powershell
cd src-tauri
cargo test non_finite --target-dir target\codex-rc
cargo test render --target-dir target\codex-rc
cargo fmt --check
```

### A5. #6 and #7 selectTrack stale UI state

Status: complete - `e65b781`
Priority: medium, small frontend state fix  
Source: final review #6 and #7  
Files likely involved: `src/hooks/useTrackMaster.ts`, hook tests

Problem:
`selectTrack` leaves previous live meters on a newly selected paused track and does not synchronously update `selectedTrackIdRef`, allowing late old-track ticks to repaint the new selection.

Implementation direction:
- In `selectTrack`, reset meter and spectrum sentinels in the same style as import.
- Immediately assign `selectedTrackIdRef.current = id` when selecting a track.
- Add hook tests for meter reset and, if possible, stale tick rejection.

Targeted verification:

```powershell
npm test
npm run build
```

## Wave B - Cheap Safety Nets And Privacy Guards

### B1. #12 and #13 private-data gitignore hardening

Status: complete - `9e90893`
Priority: medium, additive  
Files likely involved: `.gitignore`, fixture/gitignore tests if present

Do:
- Default-deny `test-output/**`, then re-include only intentionally tracked fixtures.
- Ignore render temp files such as `*.tmp` or `*.wav.*.tmp`.
- Verify with `git check-ignore` for:
  - `test-output/run-summary.json`
  - `test-output/myrun/ledger.csv`
  - a representative `something.wav.<uuid>.tmp`

### B2. #14 Album Volume Match export tripwire

Status: complete - `642d7e7`
Priority: medium, additive test  
Files likely involved: `src-tauri/tests/album_render.rs`, `src-tauri/src/album_render.rs`

Do:
- Add an album-render test proving `volume_match: true` and `volume_match: false` produce byte-identical album output.
- This protects the invariant that Volume Match is audition-only and must not change export level.

### B3. #15 Original/Mastered swap playhead tripwire

Status: complete - `ee15e14`
Priority: medium, additive test  
Files likely involved: `src-tauri/src/audio.rs`, audio tests

Do:
- Add backend coverage proving a swap from Original to Mastered with a non-zero start position seeks the new sink instead of restarting from zero.
- If direct sink observation is hard, add the smallest seam/test helper needed without changing runtime behavior.

### B4. #18 Album measured LUFS sanitization

Status: complete - `2c4d2f5`
Priority: low but cheap  
Files likely involved: `src-tauri/src/engine.rs`, `src-tauri/src/album_render.rs`, `src/bindings.ts`

Do:
- Sanitize album-path `measured_lufs` the same way single-track receipts do.
- Add a silent-track manifest test that `measured_lufs` serializes as a number, not JSON `null`.

## Wave C - Desktop Correctness And Persistence

### C1. #19 Save/Open Project drops Album-panel choices

Status: complete - `3e0d41d`
Priority: high  
Files likely involved: `src/hooks/useTrackMaster.ts`, `src/bindings.ts`, `src-tauri/src/types.rs`, project tests

Do:
- Extend project state with album-panel choices: arc kind, album intensity, album title, album sample rate, and album bit depth.
- Provide backwards-compatible defaults for old `.ams.json` files.
- Wire save and restore.
- Add a Save/Open round-trip test that proves reopened album settings match saved choices.

Run mobile bridge tests if shared Rust types change.

### C2. #20 Partial analysis failures are stderr-only

Status: complete - `82752e9`
Priority: medium  
Files likely involved: `src-tauri/src/engine.rs`, `src/hooks/useTrackMaster.ts`, import/open-project tests

Do:
- Return structured partial failures from analysis, or detect requested-but-absent IDs on the frontend.
- Show calm per-track recovery feedback and keep retry/re-analyze obvious.
- Add tests for partial-success import and Open Project recovery.

### C3. #9 togglePlay near-end restart threshold

Status: complete - `b9c9d55`
Priority: medium  
Files likely involved: `src/hooks/useTrackMaster.ts`

Do:
- Replace the fixed `0.5s` near-end restart condition with a tighter EOF epsilon or an explicit backend-ended signal.
- Add hook coverage for pause/play in the last half-second and for sub-0.5s sources.

### C4. #10 Standard export off-grid loudness target

Status: complete - `ce4aaf7`
Priority: medium  
Files likely involved: `src/lib/standard-export.ts`, `src/lib/standard-managed.ts`, frontend tests

Do:
- Ensure Standard cannot inherit Advanced-only loudness/profile settings.
- Either bounce to Advanced, reset profile and loudness on Standard return, or snap to the Standard grid.
- Add tests for Advanced Apple Music or custom target returning to Standard.

### C5. #11 Import metadata track selection

Status: complete - `bc33e53`
Priority: medium  
Files likely involved: `src-tauri/src/files.rs`, `src-tauri/src/decode.rs`

Do:
- Make metadata probing select the same audio track as decode: first non-NULL codec track.
- Add a fixture or unit test with a non-audio/default track before the audio track.

### C6. #16 Standard delivery label source of truth

Status: complete - `e6e36a7`
Priority: medium  
Files likely involved: `src/components/StandardView.tsx`, `src/lib/standard-export.ts`, tests

Do:
- Render the Standard delivery label from `STANDARD_EXPORT_DELIVERY` instead of hardcoded text.
- Add or update a frontend test so changing the canonical recipe changes the label.

### C7. #17 DeliveryProfile Rust/TS parity pin

Status: complete - `1688da6`
Priority: medium  
Files likely involved: `src/bindings.ts`, `src-tauri/src/types.rs`, parity fixture/tests

Do:
- Generate or snapshot Rust `DeliveryProfile` target/ceiling/bit-depth/sample-rate values.
- Assert TS `DELIVERY_PROFILE_*` records match Rust.
- Do not land broad tauri-specta work unless the user explicitly wants that parked refactor revived.

## Wave D - Mobile Gating

These gate mobile release. They do not need to block desktop-only stabilization unless the user changes scope.

### D1. #1 Android AAudio callback channel-count OOB

Status: complete - `5428662`
Priority: critical for Android  
Files likely involved: `apps/android-native/rust/src/aaudio.rs`

Do:
- After stream open, query actual channel count, sample rate, and format.
- Size callback buffers from actual stream values, not source values.
- Down/up-mix safely or refuse to start on mismatch.
- Add host tests where possible and the Android lane.

Verification:

```powershell
cd apps/android-native/rust
cargo test
cargo ndk -t arm64-v8a --platform 29 check
```

### D2. #8 Android measureLanding vs release race

Status: complete - `800ba13`
Priority: medium for Android  
Files likely involved: Android `AuditionController.kt`, Android Rust facade

Do:
- Serialize `destroyNative` against in-flight `measureLanding`.
- Join/cancel-and-await the landing job before destroying the handle, or protect Rust-side handle access with a lock/refcount.
- Add lifecycle/race tests where the existing harness allows.

## Wave E - Verify Before Closing Or Defer

### E1. #5 album source-path hardening

Status: complete - `fb6d98b`
Do:
- Treat as parity hardening, not a prior-render overwrite bug.
- Add canonical `output == source` guard to album explicit-dir path if low-risk.
- Include symlink/TOCTOU considerations only if the implementation remains small.

### E2. Decode/channel-count speculative findings

Status: complete - `f61195f`; no panic/OOB claim was made without a proving fixture.
Do:
- Re-audit `decode.rs` channel-count and allocation assumptions before changing code.
- Do not claim panic/OOB without a proving fixture.
- If confirmed, add fixture-driven tests around malformed/odd containers.

### E3. Docs, CI, and process drift

Status: complete for mechanical quick wins; broader product-canonical refresh remains owner-gated.
Do:
- Enforce or document the Android ARM64 CI gate.
- Add fold-down receipt copy and tests.
- Refresh stale docs only after owner approval when docs scope is product-canonical.
- Add superseded banners to archaeology docs if desired.

## Hold Until After Stabilization

Do not mix these into correctness slices:

- Export writer dedup/refactor.
- Preview-window cleanup beyond the #5b parity fix.
- DeepAnalysis strata refactor.
- Guardrails preamble/comment cleanup.
- Callback allocation wiring cleanup.
- CSS dead selectors and stale class cleanup.
- Large docs canon rewrite beyond targeted owner-approved fixes.

## Verification Lanes

Fast lane for normal desktop changes:

```powershell
npm test
npm run build
npm run build:windows
cd src-tauri
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

iPhone bridge lane when touching shared crate types or Tauri command signatures:

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

Android bridge lane when touching shared crate types, the iPhone facade, or Android code:

```powershell
cd apps/android-native/rust
cargo test
cargo ndk -t arm64-v8a --platform 29 check
```

Slow fixture lane before DSP/export merges:

```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

## Done Means

- The finding was re-read in the final report and falsified before implementation.
- A mechanical regression test failed before the fix, when practical.
- The fix is scoped to the slice.
- Targeted tests pass.
- Required platform lanes pass for touched surfaces.
- No private audio or rendered private masters are added to git.
- The final response names exactly what changed, what was verified, and what remains.
