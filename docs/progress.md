# Progress Log

## 2026-05-11 — Phase 0: Workspace scaffold and architecture ADR

Goal:

Take the Claude-build repo from zero-state to a buildable Tauri 2.x shell with the architecture lock recorded in `docs/adr/0001-tauri-rust-stack.md`.

What changed:

- Rewrote `docs/IMPLEMENTATION_PLAN.md` for the Claude build. Preserved the phase wisdom from the Codex-authored version; reframed Phase 0 as "workspace scaffold + ADR" instead of "canon and repo baseline"; replaced Codex verification commands with Claude-build-native commands; updated module list and typed command list to match the zero-state assumption.
- Wrote ADR 0001 locking Tauri 2.x shell + React/TS frontend + Rust audio engine, with JUCE and Rust-native UI documented as reversible fallbacks.
- Scaffolded the workspace:
  - Root: `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`.
  - Frontend in `src/`: `main.tsx`, `App.tsx`, `App.css`, `vite-env.d.ts`.
  - Tauri app in `src-tauri/`: `Cargo.toml`, `tauri.conf.json`, `build.rs`, `src/main.rs`, `src/lib.rs` with empty module declarations for `audio`, `engine`, `exports`, `files`, `jobs`, `project`, `settings`.
  - `src-tauri/capabilities/default.json` with the minimum permission set.

Verification:

- `npm install` — 74 packages, 0 vulnerabilities. `@types/node ^25.7.0` was added during the run because `vite.config.ts` references `process.env.TAURI_DEV_HOST` and TypeScript needed Node types.
- `npm run build` — frontend builds cleanly via `tsc -b && vite build`. `dist/` produced (~196 KB pre-gzip).
- `cargo check` (from `src-tauri/`) — clean after a placeholder `icon.ico` was added; `tauri-build` requires it for the Windows resource file even with `bundle.active = false`. The placeholder icon is a 64×64 dark "A" generated via .NET `System.Drawing`; it should be replaced with a real icon in Phase 14.
- `cargo test` (from `src-tauri/`) — 0 tests, all 0 pass. Contract/unit tests start in Phase 1.

Real-audio fixture used: none. Phase 0 has no audio path yet.

What failed or remains partial:

- `npm run tauri dev` requires an interactive window and is not verified in this automated pass; deferred to manual verification on the dev machine.
- `tauri build` is intentionally not attempted; signing and proper icons belong to Phase 14.
- No DSP, no audio I/O, no playback. Phase 1 starts the typed command layer; Phase 3 brings real audio.
- The placeholder `src-tauri/icons/icon.ico` is committed (2.6 KB) so the workspace builds out of the box; replace in Phase 14.

Next recommended slice:

Phase 1 — typed Rust app foundation. Define the command list with stub implementations and contract tests. Start with `analyze_tracks`, `prepare_waveform`, and `prepare_source_playback` since those unblock Phase 2's frontend skeleton.

## 2026-05-11 — Phase 1: Rust/Tauri typed app foundation

Goal:

Make the backend speak product concepts via typed `#[tauri::command]` handlers, with realistic mock shapes so frontend phases can develop against the contract before DSP exists.

What changed:

- Added `uuid` and `thiserror` to `src-tauri/Cargo.toml`; `tokio` (dev-only) for async contract tests.
- `src-tauri/src/types.rs`: shared product types — `TrackId`, `ImportedTrack`, `AnalysisResult` (LUFS, true peak, DR, spectral balance, transient density, stereo width, recommended Universal settings), `MasteringSettings` (preset, intensity, EQ, volume_match, advanced), `WaveformPeaks`, `PlaybackHandle`, `AbPreview`, `RenderJob` + `JobStatus`, `ExportReport` + `QualityCheck`, `ProjectState`, `UserPreset`, `CommandError` (serialized as string for the IPC boundary).
- 16 typed commands registered in `lib.rs`:
  - `files`: `import_tracks` (rejects `..` traversal, extracts display name + extension).
  - `engine`: `analyze_tracks`, `render_track_preview`, `render_track_master`, `render_album_master` (all return realistic mocks).
  - `audio`: `prepare_source_playback`, `prepare_master_playback`, `prepare_ab_preview`, `prepare_waveform` (sine-shaped mock peak envelope).
  - `exports`: `run_export_checks` (real logic: true-peak / LUFS / DR / bit-depth / non-finite guards with plain-language messages), `open_output`.
  - `project`: `save_project`, `autosave_session`, `load_recent_session`.
  - `settings`: `save_user_preset`, `list_user_presets`.
- `src-tauri/tests/contracts.rs`: 8 contract tests covering analyze, waveform stereo shape, path-traversal rejection, display-name parsing, export-check warnings (high true peak), export-check silent-when-clean, render-job done-shape, preset name validation.
- `src/bindings.ts`: hand-written TS types mirroring Rust types. Phase 1.2 should replace with `tauri-specta` codegen.
- `src/lib/api.ts`: typed wrapper around `@tauri-apps/api/core`'s `invoke`, with snake_case param keys matching Rust function signatures.
- `src/App.tsx`: IPC proof — two buttons (Import mock track, Analyze) that round-trip through the backend and render shape-valid results. Replaced in Phase 2 by the real Track Master surface.

Verification:

- `npm run build`: clean. Bundle 198 KB (62 KB gzip).
- `cargo test` (from `src-tauri/`): 8/8 contract tests pass. New-deps compile ~30s.
- `npm run tauri dev`: deferred (interactive).

Real-audio fixture used: none. Phase 1 is stub-only; real audio paths begin in Phase 3.

What failed or remains partial:

- TS bindings are hand-written; will drift from Rust types if changed without care. Phase 1.2 should swap to `tauri-specta` codegen.
- No live frontend smoke; the IPC proof in `App.tsx` only exercises 2 of 16 commands and is verified by the manual `npm run tauri dev` Dan runs locally.
- No runtime IPC test from Rust; contract tests call command functions directly.

Next recommended slice:

Phase 2 — Track Master frontend skeleton: left-rail song list, main waveform area (placeholder canvas), transport (play/pause/seek/loop), Original/Mastered toggle, Volume Match toggle (off), preset tile row, Intensity, 3-band EQ, preview-stale indicator, Export Master button, collapsed advanced section. UI shell only, wired to the Phase 1 stubs. Alternative: Phase 1.2 (specta bindings) first if binding drift bites — but Dan's "knock so much out" preference probably favors moving to Phase 2.
