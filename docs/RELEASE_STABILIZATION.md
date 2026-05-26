# Release Stabilization

This is the active jump-fix queue for the new YES Master repo.

## P0 - Export Review

Problem:

- Export checks warn after render, but the primary action still looks like a
  normal clean export path.

Required behavior:

- No warnings: `Export Master`.
- Warnings present or expected from current/last checks: `Export With Review`.
- Review panel lists warning rows plainly.
- User actions: `Adjust Settings` or `Export Anyway`.
- Technical failures still block export.

Verification:

- Frontend tests for button label and review panel.
- Export receipt tests still pass.
- Manual UI pass in the desktop app.

## P0 - Compressor Mode

Problem:

- UI says `Auto` for compressor values that are not track-aware. They are
  preset/density defaults.

Required behavior:

- Rename current readout concept to `Preset`.
- Add mode: `Preset / Manual / Off`.
- `Manual` engages user overrides.
- `Off` bypasses creative/preset compressor only.
- Limiter, ceiling, LUFS landing, metering, and export warnings remain active.

Verification:

- Type/schema migration test.
- Frontend interaction test.
- DSP test proving Off removes compressor gain reduction while limiter still
  catches peaks.
- Export-check test proving Off still warns on unsafe output.

## P0 - Already-Mastered Input Matrix

Problem:

- Current real-fixture recon shows already-processed material can become much
  louder and flatter after default render.

Required behavior:

- Add private-fixture metrics protocol.
- Capture source/render LUFS, true peak, dynamic range, and warning codes.
- Run across at least Universal, Loud, Clarity, and compressor Off.

Verification:

- Slow lane prints a clear ledger.
- No private audio or rendered private masters are committed.

## P1 - Realtime Sweep Confirmation

Problem:

- Realtime stutter fixes landed, but manual playback verification is still the
  honest gate.

Required behavior:

- Aggressively sweep Intensity, EQ, output gain, compressor threshold, and
  density while audio plays.
- Repeat with Preview LUFS off and on.
- Verify no stutter, stuck 2x DSP, stale LUFS cache, or track-switch poison.

After clean verification:

- Remove `get_diag_counters`.
- Remove temporary diagnostic atomics/types/API wiring.

## P1 - Tooling Gate Cleanup

- Decide whether to format Rust now or in a dedicated mechanical commit.
- Install Clippy and make `cargo clippy --all-targets -- -D warnings` runnable.
- Keep `npm test`, `npm run build`, `cargo test --lib`, and `cargo test` green.

## Deferred

- Public signing/notarization.
- Autoupdate.
- Reference-track UX.
- Album dashboard/report expansion.
- Subjective preset retuning without fresh listening notes.
