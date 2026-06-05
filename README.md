# YES Master

YES Master is a local desktop mastering app for finished tracks and album
drafts. It runs as a Tauri + React + TypeScript frontend with a Rust audio/DSP
backend.

The product goal is private-solid first: a musician or producer should be able
to trust it on real material before any public-release discussion.

## Current Product Shape

- Track Master is the primary workflow.
- Album Master exists and should remain intact, but Track Master stabilization
  is the next release gate.
- Main UI is for creative shaping: preset, intensity, EQ, tone, saturation,
  width, compression, and limiter choices.
- Right rail is for judgment and delivery: quality checks, delivery profile,
  advanced controls, per-band compressor detail, format, and export review.
- Export warnings are advisory when technically possible. The user can decide
  to export a risky master, but the app must make that risk visible.
- Source files are never destructively edited.
- Private audio fixtures may be used locally and must never be committed.

## Setup

```powershell
npm install
```

## Run

```powershell
npm run tauri dev
```

## Fast Verification

From the repo root:

```powershell
npm test
npm run build
npm run build:windows
```

From `src-tauri`:

```powershell
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

## iPhone Native Bridge Lane

When changing shared Rust types, `yes_master_lib` behavior, adaptive/profile
resolution, or `#[tauri::command]` signatures that the phone bridge may depend
on, also run:

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

The desktop lanes do not compile this bridge. A shared struct or signature can
drift here while all desktop checks stay green.

## Slow Fixture Lane

Use this only when local private fixtures exist under
`private-audio-fixtures/`.

```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

The fixture lane is required before merging work that touches DSP, render,
LUFS landing, WAV writing, export checks, or source/master parity.

## Documentation

Read these first:

- `docs/PRODUCT.md`
- `docs/APP_BEHAVIOR.md`
- `docs/ARCHITECTURE.md`
- `docs/TESTING.md`
- `docs/RELEASE_STABILIZATION.md`
- `docs/RELEASE_EVIDENCE_2026-05-28.md`

Historical handoffs, old phase plans, and prior-session notes live in git
history and in the archived source repo. They are not active product spec.
