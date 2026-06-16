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

The previous five queue items all shipped — see "Implemented Stabilization
Slices" in `docs/RELEASE_STABILIZATION.md`. Genuinely open:

1. Manual Listening Gate — normal / already-mastered / long-source sweeps and
   a clean-vs-warning export comparison, by ear (owner signoff).
2. Reference Retune listening — aggregate runner completed 2026-05-28;
   listening notes pending. Oomph is the least-matched preset; listen before
   changing it.
3. Already-mastered matrix listening signoff (runner evidence complete).

Album mixed mono/stereo channel-count parity shipped 2026-06-16 (see the
"Wave 9 Mechanical Reconciliation" section in
`docs/RELEASE_STABILIZATION.md`); it is no longer a deferred slice.

The refactor backlog (former item 5) was executed in full on 2026-06-09 —
see the execution record at the end of
`docs/reviews/2026-06-10-consolidated-refactor-backlog.md`. Only P2's
one-pole/soft-knee hoist (owner-deferred) and P4 tauri-specta stay parked.

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

The Android bridge (`apps/android-native/rust`) re-uses the same shared crate
AND the iPhone facade crate as an rlib. When you touch shared crate types,
the facade, or the android crate, run its lane too (host tests + arm64
cross-check; needs the toolchain from docs/ANDROID_NATIVE_SPEC.md "Build
prerequisites" — JDK 17 on JAVA_HOME, SDK+NDK r27.2 via local.properties or
ANDROID_HOME, rust android targets, cargo-ndk ≥ 3.5.6):

```powershell
cd apps/android-native/rust
cargo test
cargo ndk -t arm64-v8a --platform 29 check
```

`--platform 29` matches minSdk; cargo-ndk's default API 21 sysroot predates
libaaudio, so the audition link fails without it.

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
