# YES Master Agent Instructions

This is the active YES Master repo. Do not treat old handoff files or prior
phase plans as active spec. Use the current code plus the docs listed below.

## Required Reading

1. `docs/PRODUCT.md`
2. `docs/APP_BEHAVIOR.md`
3. `docs/ARCHITECTURE.md`
4. `docs/TESTING.md`
5. `docs/RELEASE_STABILIZATION.md`

## Non-Negotiables

- Local desktop app for Mac and Windows. Linux remains deferred.
- Track Master stabilization comes before new feature expansion.
- Real-time or near-real-time audition must stay responsive.
- Original/Mastered switching must preserve playhead.
- Volume Match is optional, off by default, and must not change export level.
- Exports never overwrite source files or prior renders by default.
- Export warnings are advisory unless the export is technically invalid.
- Users may overcook their own track, but the app must show clear metering,
  warnings, and review states.
- Private audio and rendered private masters never belong in git.

## Current Jump-Fix Queue

1. Warning-aware export review flow.
2. Compressor mode: Preset / Manual / Off.
3. Already-mastered input regression matrix using private fixtures.
4. Realtime sweep verification, then removal of temporary diagnostic counters.
5. Release-gate cleanup: Rust formatting decision and Clippy install/gate.

## Verification

Use the fast lane for normal changes:

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

When you touch shared crate types or `#[tauri::command]` signatures, also build
AND test the iPhone native bridge — it re-uses `yes_master_lib` but none of the
lanes above build it, so a struct-field or signature change drifts there silently
(it broke the bridge build once already). The bridge now carries real adaptive
logic (it resolves the source profile + confidence like desktop), so `cargo check`
alone is not enough — run its tests too:

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

Use the slow fixture lane before DSP/export merges:

```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

## Working Style

- Prefer current code reality over historical prose.
- Keep changes scoped and testable.
- If a finding is objective, add a mechanical test.
- If a finding is taste/listening-dependent, capture the listening note before
  changing preset calibration.
- Do not call a slice complete because the UI resembles the goal; verify the
  behavior.
