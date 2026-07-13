# Windows End-to-End Test Report — 2026-07-13

## Result

**PASS for the mechanically testable Windows Track Master workflow.** No new
objective product defect was reproduced on `main` at `0a16d7a`.

The current `0.9.0` NSIS build was compiled from that exact revision, installed
over the older local build, launched through the installed binary, exercised
through the real Windows UI, and closed cleanly. The rebuilt export receipt is
present in the installed app and stamps the tested revision (`0a16d7a+`).

This report does not close the owner-ear acceptance item for the residual
Original/Mastered volume dip. UI automation can prove continuity, state, and
error behavior; it cannot decide whether a fast loudness dip is audible.

## Install proof

- `git pull --ff-only` fast-forwarded `main` from `a4d6570` to `0a16d7a`.
- `npm run build:windows` completed and produced both the `0.9.0` MSI and NSIS
  bundles.
- Silent NSIS install exited `0`.
- Windows uninstall registration reports:
  - product: `YES Master`
  - version: `0.9.0`
  - publisher: `Daniel Kinsner`
  - install root: `%LOCALAPPDATA%\YES Master`
- The installed `yes-master.exe` launched, exposed a targetable `YES Master`
  window, and remained healthy through the walkthrough.

## Computer-use walkthrough

### Launch, restore, and chrome

- The prior five-track session restored and completed its analysis pass.
- Settings opened and exposed current defaults, Audio Output, export defaults,
  app information, and the first-run-tip reset.
- Help opened with the current import, Original/Mastered, Volume Match,
  delivery, export, and diagnostics guidance.
- Album Master restored its title, five-track order, flow controls, follow vs
  override state, and album-wide delivery controls.

### Import and project persistence

- A project copy was saved under ignored `test-output/` storage and reopened.
- The local synthetic `test-output/smoke-sine.wav` fixture imported through the
  native file picker, analyzed, reached `READY`, and exposed source measurements.
- Reopening the saved project restored the original five-track session, so the
  synthetic fixture was not left in the user's working session.

### Audition

- Original playback started with live waveform motion and live peak/LUFS meters.
- Switching to Mastered kept playback active.
- Volume Match was enabled and six rapid Original/Mastered flips were issued.
- Playback and meters remained live; no timeout, stale-request error, or error
  toast appeared. The log likewise contains no warning/error for the run.
- This mechanically supports `7fc58f6` and `0f6ab86`. It does **not** constitute
  the owner-ear signoff for the residual near-zero dip described in the beta
  execution plan.

### Standard export

- Selected Tape, Driving intensity, and Medium (`-11 LUFS`).
- `Create Master` opened the native save picker and disabled itself with the
  documented in-progress explanation while rendering.
- Render completed with the compact Standard success card and `Show file`.
- External WAV-header verification: stereo PCM, 44.1 kHz, 24-bit.
- Receipt landing: `-11.0 LUFS`.

### Advanced review and export receipt

- Returning to Advanced displayed the new full export receipt for the Standard
  render with truthful Standard format and exact build stamp.
- The warning-bearing source exposed `Export With Review`.
- `Adjust Settings` closed the review without rendering.
- `Export Anyway` opened the native save picker.
- Collision-safe naming proposed the next `-2` filename rather than a replace
  prompt or silent overwrite.
- Render completed and the receipt reported the selected Advanced delivery
  format, mastering style/intensity, output measurements, source-quality rows,
  saved-file identity, timestamp, and build traceability.
- External WAV-header verification: stereo PCM float, 48 kHz, 32-bit float.

## Mechanical verification

- `npm test`: **60 files, 546 tests passed**.
- `npm run verify:rust`: format, Clippy with warnings denied, 417 non-ignored
  library tests, and the complete Rust integration/doc-test lane passed.
- `npm run build:windows`: production frontend build, optimized Rust build, MSI,
  and NSIS all passed.
- Latest non-doc `main` revision `0f6ab86` has a green GitHub Actions CI run.
  `0a16d7a` is docs-only and intentionally `[skip ci]`.
- Installed-app log: both GUI renders completed successfully; no `WARN`,
  `ERROR`, panic, or timeout line was emitted during this walkthrough.

## Findings

### New objective defects

None reproduced.

### Existing acceptance item — owner-ear A/B confirmation

The residual near-zero Original/Mastered dip remains open exactly as documented
in `docs/plans/2026-07-07-beta-execution-plan.md` Slice 14b. Automation verified
that the Volume Match stall and stale rapid-toggle error do not reproduce as
functional failures on this build. Only the owner can decide whether the
remaining transition is audibly acceptable.

### Coverage boundary — not a defect

The installed Album Master UI and restored album state were smoke-tested, but a
second full GUI album render was not created from the private five-track session.
That would duplicate hundreds of megabytes of private render output without
adding proportionate evidence. Album planning, render, sample-rate/channel
reconciliation, override, manifest, hostile-I/O, and collision behavior all ran
green in the Rust integration lane.

### Expected pre-release updater state — not a defect

Startup logged `update check skipped: Could not fetch a valid release JSON from
the remote` at INFO level. This is expected until the release workflow publishes
the real signed updater manifest and does not affect local mastering.

## Conclusion

There is no evidence-backed code patch to make from this run. Do not change DSP,
crossfade behavior, or gated owner-calibration constants on the strength of this
automation pass. Follow the accompanying plan for the remaining owner-ear gate
and conditional escalation path.
