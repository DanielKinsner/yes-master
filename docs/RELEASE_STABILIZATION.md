# Release Stabilization

This is the active jump-fix queue for the new YES Master repo. Keep it aligned
with `docs/PRODUCT.md` and `docs/APP_BEHAVIOR.md`.

## Implemented Stabilization Slices

### Final Repo-Wide Review Implementation

Status: mechanical queue implemented on 2026-06-17.

The 2026-06-16 final repo-wide review implementation queue was executed on
`main` in small pushed commits. The durable execution ledger is
`docs/archive/plans/2026-06-16-final-review-implementation-plan.md`; it marks the
A1-E3 slices complete and names the remaining owner-gated listening/product-doc
work plus the parked cleanup/refactor backlog.

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

Album Master delivery-format parity has landed (album-wide sample rate,
bit-depth, mixed-source resampling, mono/stereo channel-count resolution, and
above-stereo source fold-down to stereo delivery; see
`src-tauri/tests/album_sample_rate.rs`).

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
- Undo/redo now have compact header buttons plus keyboard shortcuts;
  analysis/readiness lives with track metadata.
- The accepted centered Track Master / Album Master header layout remains.

Verification coverage:

- `src/hooks/useTrackMaster.integration.test.tsx`
- `src/App.chrome.test.tsx`
- `src/App.layout-css.test.ts`
- Local ignored screenshots summarized in
  `docs/archive/RELEASE_EVIDENCE_2026-05-28.md`.

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
`docs/archive/RELEASE_EVIDENCE_2026-05-28.md`.

### Wave 9 Mechanical Reconciliation

Status: implemented on 2026-06-16; taste/listening calibration remains Wave 10.

Current behavior:

- Android A4 catch-up is complete: fail-fast import support checks, import-cache
  pruning, Ready-state process-death restore, Done-screen share/play actions,
  monochrome launcher icon, and bridge/build tidiness riders.
- iPhone live audition measures landing at the rendered sample rate and no
  longer allocates a replacement mastering chain inside the render callback.
- Album export reports rendered delivery format, per-track source/rendered
  sample rates and channel counts, and frontend upsample/upmix receipt copy.
- `stereo_width` is retained as an active analysis/planning metric; new taste
  wiring waits for Wave 10.
- Stale Vera experiment branches were removed from `origin`.

Verification coverage:

- Android JVM tests, Android Rust host tests, and the Gradle native-alignment
  task.
- iPhone Rust bridge check/tests.
- Focused Rust album sample-rate tests and AlbumPanel frontend test.

## Active Gates

### Tier-1 Adaptive Voicing Listening

Status: owner-listened and accepted, 2026-06-11. The owner ran a live
auditioning session (including 96 kHz sources; the dev-profile underrun fix
`201e746` was made during it, which is the verification artifact). Tier-1
guardrail constants are not to be reopened without a new listening note.
Phase-B confidence gating remains OFF by default; its calibration is
scheduled into the adaptive-compressor calibration sitting (see
`docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md` §5).

### Reference Retune Validation

Status: aggregate runner completed on 2026-05-28; listening/taste work moved
to Wave 10 on 2026-06-16 so the mechanical shippability list can close first.

- Re-run the private reference tuning runner after DSP/preset changes.
- Use listening notes before any further subjective preset tuning.
- Oomph remains the least-matched preset in the current private reference
  snapshot and needs careful listening before more changes.
- Do not change export LUFS landing or compressor mode semantics in this gate.

### Already-Mastered Input Matrix

Status: full local manifest completed on 2026-05-28 (18 cases, `--release`
example build). No silent regression; aggregate in
`docs/archive/RELEASE_EVIDENCE_2026-05-28.md`. The earlier timeout was a debug-build cost
only. Listening signoff moved to Wave 10 on 2026-06-16.

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
  `docs/archive/RELEASE_EVIDENCE_2026-05-28.md`.

### Manual Listening Gate

Status: deferred to Wave 10 on 2026-06-16.

- Verify normal, already-mastered/compressed, and long edge-case sources by ear.
- Sweep Intensity, EQ/tone, output gain, compressor controls, Preview LUFS, and
  Volume Match while audio plays.
- Seek across a long source in Mastered mode with Preview LUFS enabled.
- Export a clean case and a warning case, then open and compare output by ear.

## Deferred

- Reference-track UX.
- Album dashboard/report expansion.
- Subjective preset retuning / listening calibration (Wave 10).

Paid Apple signing/notarization and Windows Authenticode are post-beta trust
upgrades under D16 (2026-07-20), not $0-beta blockers. The autoupdater remains
in beta scope: its permanent key and signed artifacts are required, and the
update path still needs one real-machine end-to-end proof.
