# Release Stabilization

This is the active jump-fix queue for the new YES Master repo. Keep it aligned
with `docs/PRODUCT.md` and `docs/APP_BEHAVIOR.md`.

## Implemented Stabilization Slices

### Export Review

Status: implemented.

Current behavior:

- No review rows: `Export Master`.
- Warning/critical review rows: `Export With Review`.
- Review panel lists warning rows plainly.
- User actions: `Adjust Settings` or `Export Anyway`.
- Technical failures still stop export through the render/save path.

Verification coverage:

- `src/components/RightRail.test.tsx`
- `src/App.album-export.test.tsx`
- Export receipt/backend contract tests.

### Compressor Mode

Status: implemented.

Current behavior:

- The UI uses `Preset / Manual / Off`.
- `Preset` displays preset/density fallback values.
- `Manual` engages user per-band overrides.
- `Off` bypasses creative/preset compression only.
- Limiter, ceiling, LUFS landing, metering, and export warnings remain active.

Verification coverage:

- `src/App.compressor-mode.test.tsx`
- `src/lib/compressor-auto.test.ts`
- `src-tauri/src/dsp.rs` unit tests.
- `src-tauri/tests/contracts.rs`

### Private Fixture And Reference Harnesses

Status: implemented as local-only slow lanes.

Current behavior:

- Already-mastered matrix runner writes ignored JSON/CSV/render outputs.
- Private reference tuning runner writes ignored JSON/CSV/render outputs.
- Private source audio, rendered private masters, and private ledgers must not
  be committed.

## Active Gates

### Reference Retune Validation

- Re-run the private reference tuning runner after DSP/preset changes.
- Use listening notes before any further subjective preset tuning.
- Oomph remains the least-matched preset in the current private reference
  snapshot and needs careful listening before more changes.
- Do not change export LUFS landing or compressor mode semantics in this gate.

### Already-Mastered Input Matrix

- Re-run the private fixture matrix for DSP/export changes.
- Capture source/render LUFS, true peak, dynamic range, and warning codes.
- Include Universal, Loud, Clarity, and compressor Off cases.
- Treat the goal as evidence and review visibility, not banning bold masters.

### Realtime Sweep Confirmation

- Aggressively sweep Intensity, EQ, output gain, compressor threshold, and
  density while audio plays.
- Repeat with Preview LUFS off and on.
- Verify no stutter, stuck 2x DSP, stale LUFS cache, or track-switch poison.

After clean verification:

- Remove `get_diag_counters`.
- Remove temporary diagnostic atomics/types/API wiring.

### Tooling Gate Cleanup

- Decide whether to format Rust now or in a dedicated mechanical commit.
- Install Clippy and make `cargo clippy --all-targets -- -D warnings` runnable.
- Keep `npm test`, `npm run build`, `cargo test --lib`, and `cargo test` green.

## Deferred

- Public signing/notarization.
- Autoupdate.
- Reference-track UX.
- Album dashboard/report expansion.
- Subjective preset retuning without fresh listening notes.
