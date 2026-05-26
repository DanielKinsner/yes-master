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
- `src/lib/compressor-auto.ts` currently computes preset/density compressor
  readouts. This should be renamed conceptually to Preset compressor behavior.
- `src/lib/export-location.ts` tracks last-used export folders and path helpers.

## Rust Backend

- `src-tauri/src/audio.rs` handles playback, live chain updates, realtime
  metering, and preview LUFS landing workers.
- `src-tauri/src/dsp.rs` defines preset calibration, chain coefficients,
  filters, compressor behavior, saturation, width, limiter, and metering.
- `src-tauri/src/engine.rs` handles analysis, rendering, LUFS landing, output
  measurements, and album render entry points.
- `src-tauri/src/exports.rs` runs export quality checks.
- `src-tauri/src/types.rs` defines shared command/data contracts.
- `src-tauri/src/album_render.rs` retains the simple album render path used by
  backend command/test/back-compat surfaces.

## Signal-Chain Direction

The intended mastering chain is:

1. Decode/source PCM.
2. Input gain.
3. Preset/tone EQ.
4. Creative/preset compression.
5. Saturation/warmth/width.
6. Limiter/ceiling.
7. Audition-only Volume Match when enabled.
8. Export LUFS landing where applicable.
9. Output measurement/check/report.

Compressor Off should bypass step 4 only.

## Export Flow Today

1. User chooses an explicit save path.
2. Frontend calls `renderTrackMaster`.
3. Backend renders and returns output measurements.
4. Frontend builds an `ExportReport`.
5. Frontend calls `runExportChecks`.
6. Receipt is stored and rendered.

Missing flow:

- Warning-aware pre-confirm/review state before final user acceptance.

## Historical Docs

Old phase plans and handoffs are intentionally not the active architecture
source. Use git history or the historical repo only when recovering context.
