# Release Stabilization

This is the active jump-fix queue for the new YES Master repo. Keep it aligned
with `docs/PRODUCT.md` and `docs/APP_BEHAVIOR.md`.

## Implemented Stabilization Slices

### Track Master Delivery Format

Status: implemented for Track Master.

Current behavior:

- Delivery Profile owns target LUFS, ceiling, bit depth, and sample rate.
- Custom Source preserves the source sample rate.
- Custom delivery format can explicitly request 44.1 kHz, 48 kHz, or 96 kHz.
- Track Master renders the selected effective sample rate and reports it in the
  receipt.
- Requested/rendered sample-rate mismatch is a technical export-check issue.

Verification coverage:

- `src/lib/effective-settings.test.ts`
- `src/lib/settings-transitions.test.ts`
- `src-tauri/src/types.rs` unit tests
- `src-tauri/tests/delivery_profile_render.rs`
- `src-tauri/tests/contracts.rs`

Album Master sample-rate + bit-depth delivery parity has landed (album-wide
Delivery Format, mixed-source resampling; see
`src-tauri/tests/album_sample_rate.rs`). Channel-count parity (mono vs stereo
tracks) remains deferred.

### Project Chrome And Help

Status: implemented.

Current behavior:

- Settings and Help open as real in-app dialogs for current behavior.
- Save/Open project flows surface success, cancellation, and recovery feedback.
- Settings/Help do not mutate mastering settings or interrupt selection.

Verification coverage:

- `src/App.chrome.test.tsx`
- `src/hooks/useTrackMaster.integration.test.tsx`

### Loudness Target Semantics

Status: implemented.

Current behavior:

- Center loudness quick-select and right-rail LUFS edits use one shared settings
  transition.
- Explicit target edits switch delivery profile to Custom and keep the effective
  target truthful.
- Named delivery profiles restore their owned target, ceiling, bit depth, and
  sample rate together.

Verification coverage:

- `src/lib/settings-transitions.test.ts`
- `src/App.loudness-target.test.tsx`
- `src/hooks/useTrackMaster.integration.test.tsx`

### Long-Track Preview Timeout Feedback

Status: bounded RC hardening implemented.

Current behavior:

- Mastered preview readiness timeouts are surfaced as recoverable user-facing
  feedback instead of silent transport failure.
- A full 25-minute manual playback reproduction is still a listening/signoff
  item, not completed evidence.

Verification coverage:

- `src-tauri/src/audio.rs` timeout path
- `src/hooks/useTrackMaster.integration.test.tsx`

### Live-Chain And Track Master Chrome

Status: implemented.

Current behavior:

- Settings edits, user presets, undo/redo, and album intent edits share the
  same "Mastered chain is loaded" predicate.
- The old mostly empty undo/redo/readiness strip is gone.
- Undo/redo are compact header tools; analysis/readiness lives with track
  metadata.
- The accepted centered Track Master / Album Master header layout remains.

Verification coverage:

- `src/hooks/useTrackMaster.integration.test.tsx`
- `src/App.layout-css.test.ts`
- Local ignored screenshots summarized in
  `docs/RELEASE_EVIDENCE_2026-05-28.md`.

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

Latest aggregate evidence is recorded in
`docs/RELEASE_EVIDENCE_2026-05-28.md`.

## Active Gates

### Reference Retune Validation

Status: aggregate runner completed on 2026-05-28; listening still pending.

- Re-run the private reference tuning runner after DSP/preset changes.
- Use listening notes before any further subjective preset tuning.
- Oomph remains the least-matched preset in the current private reference
  snapshot and needs careful listening before more changes.
- Do not change export LUFS landing or compressor mode semantics in this gate.

### Already-Mastered Input Matrix

Status: full local manifest completed on 2026-05-28 (18 cases, `--release`
example build). No silent regression; aggregate in
`docs/RELEASE_EVIDENCE_2026-05-28.md`. The earlier timeout was a debug-build cost
only. Listening signoff is still pending.

- Re-run the private fixture matrix for DSP/export changes.
- Capture source/render LUFS, true peak, dynamic range, and warning codes.
- Include Universal, Loud, Clarity, and compressor Off cases.
- Treat the goal as evidence and review visibility, not banning bold masters.

### Realtime Sweep Confirmation

Status: responsive sweep accepted, diagnostic counters removed in
`58c25d7 chore: remove realtime diagnostic counters`.

- Aggressively sweep Intensity, EQ, output gain, compressor threshold, and
  density while audio plays.
- Repeat with Preview LUFS off and on.
- Verify no stutter, stuck 2x DSP, stale LUFS cache, or track-switch poison.
- If future regressions appear, add temporary instrumentation behind a dev-only
  path instead of restoring production diagnostic API wiring.

### Tooling Gate Cleanup

Status: complete on 2026-05-28.

- Rust formatting was applied in a dedicated mechanical commit.
- Clippy was installed locally and passes with `-D warnings`.
- `npm test`, `npm run build`, `cargo test --lib`, `cargo test`, and
  `npm run build:windows` are green. See
  `docs/RELEASE_EVIDENCE_2026-05-28.md`.

### Manual Listening Gate

Status: pending.

- Verify normal, already-mastered/compressed, and long edge-case sources by ear.
- Sweep Intensity, EQ/tone, output gain, compressor controls, Preview LUFS, and
  Volume Match while audio plays.
- Seek across a long source in Mastered mode with Preview LUFS enabled.
- Export a clean case and a warning case, then open and compare output by ear.

## Deferred

- Public signing/notarization.
- Autoupdate.
- Reference-track UX.
- Album dashboard/report expansion.
- Subjective preset retuning without fresh listening notes.
