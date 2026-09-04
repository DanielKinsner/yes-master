# Architecture

YES Master is split between a web UI shell and a native Rust audio backend.

## Frontend

- `src/App.tsx` composes the main workspace.
- `src/components/RightRail.tsx` owns the right-rail quality/export area.
- `src/hooks/useTrackMaster.ts` owns Track Master state, playback calls, render
  calls, export receipt state, and user-setting transitions.
- `src/lib/*` holds pure helpers with co-located Vitest coverage.

Important frontend helpers:

- `src/lib/settings-transitions.ts` injects source LUFS and handles profile /
  loudness-setting transitions.
- `src/lib/compressor-auto.ts` computes preset/density compressor readouts for
  the `Preset` compressor UI. The filename is historical; the user-facing
  behavior is not track-aware auto-analysis.
- `src/lib/export-location.ts` tracks last-used export folders and path helpers.
- `src/lib/shortcuts.ts` is the one keyboard-shortcut catalogue (handlers,
  the `?` overlay and Help all read it).

## Rust Backend

- `src-tauri/src/audio.rs` handles playback, live chain updates, realtime
  metering, and preview LUFS landing workers.
- `src-tauri/src/dsp.rs` defines preset calibration, chain coefficients,
  filters, compressor behavior, saturation, width, limiter, and metering.
- `src-tauri/src/engine.rs` handles analysis, rendering, LUFS landing, output
  measurements, and album render entry points.
- `src-tauri/src/exports.rs` runs export quality checks.
- `src-tauri/src/demo.rs` synthesises the empty state's demo track once into
  app-data (`prepare_demo_track`); swap the generator for a bundled file
  without changing the command's contract.
- `src-tauri/src/types.rs` defines shared command/data contracts.
- `src-tauri/src/album_render.rs` is the active album render path
  (`render_album_plan_impl`), invoked by the `render_album_plan` command in
  `engine.rs`. `album.rs` is the separate album *planner*; the two are
  complementary, not duplicate.

## Preview and measurement lifecycle

Preview LUFS measures the whole processed track off the audio thread. Cold
Volume Match uses the shared preview worker and returns its representative
8-second measurement first. A cache-only command path applies gains; one active
worker plus the latest pending edit avoids duplicate jobs on the same source.
Source epochs reject old results after track/device changes, independently of
playback coefficient generations. Same-source A/B switching reuses in-flight work.

Live integrated metering uses bounded 0.1 LU histogram history and fixed batches;
feed/reset allocate nothing. Export receipts measure delivered PCM with exact
history. Album assembly streams delivered tracks into the continuous WAV.
Signal-chain indicators use backend-resolved stage activity rather than copied
frontend preset tables.

## Signal-Chain Direction

The mastering chain, as implemented in `MasteringChain::process_frame_inplace`
(`src-tauri/src/dsp.rs`), is:

1. Decode/source PCM.
2. Input gain.
3. Preset/tone EQ (7 bands).
4. Creative/preset multiband compression.
5. Transient shaping.
6. Stereo width (M/S) — applied before saturation so the non-linear stage
   doesn't smear the chosen image back toward mono.
7. Saturation/warmth.
8. Limiter/ceiling (true-peak lookahead).
9. Audition-only Volume Match when enabled.
10. User output trim, then export LUFS landing where applicable.
11. Output measurement/check/report.

Compressor Off bypasses step 4 only.

## Export Flow Today

1. Right rail derives preflight review rows from current source analysis until
   an export receipt exists.
2. Clean path shows `Export Master`.
3. Warning/critical review rows show `Export With Review`.
4. First review click opens an inline review panel instead of rendering.
5. `Adjust Settings` closes review; `Export Anyway` calls the normal export
   path.
6. User chooses an explicit save path.
7. Frontend calls `renderTrackMaster`.
8. Backend renders and returns output measurements.
9. Frontend builds an `ExportReport`.
10. Frontend calls `runExportChecks`.
11. Receipt is stored and rendered.

Quality review rows are advisory. Technical failures still stop export in the
render/save path.

## Historical Docs

Old phase plans and handoffs are intentionally not the active architecture
source. Use git history or the historical repo only when recovering context.
