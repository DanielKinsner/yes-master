# Current App Behavior

This document describes what the current program does now, not what older
handoffs planned.

## Application

- Tauri desktop app.
- React/TypeScript UI.
- Rust backend for decode, analysis, playback, DSP, render, export checks, and
  project/file operations.
- Product name shown to users: YES Master.
- Package/repo identity: `yes-master`.

## Track Master

Track Master supports:

- Importing local audio.
- Source analysis.
- Waveform display.
- Original/Mastered audition.
- Region selection and loop playback.
- Presets and intensity.
- Visual EQ/tone shaping.
- Advanced controls.
- Delivery profile selection.
- Explicit save destination for export.
- Post-render export receipt/checks.

## Export Checks

The backend currently checks rendered-output measurements for:

- True peak above critical/safe streaming thresholds.
- Loudness above very-loud territory.
- Dynamic range below the low-dynamic-range threshold.
- Bit depth below 16-bit.
- Non-finite LUFS metering.
- Already-compressed source combined with moderate/heavy compression density.

These checks warn; they do not currently create a pre-export review/confirm
flow in the right rail.

## Known Current Gaps

1. Export warnings exist, but the primary button still reads `Export Master`.
   The UX needs a warning-aware review step.

2. The per-band compressor UI says `Auto`, but the values are preset/density
   fallbacks rather than track-aware auto-analysis.

3. There is no compressor mode field yet. `Preset / Manual / Off` needs schema,
   UI, DSP, migration, and tests.

4. Already-mastered material can be pushed louder and flatter by the default
   chain. The app can warn afterward, but it needs better pre-export review and
   fixture-based regression coverage.

5. Temporary diagnostic counters for the realtime recovery are still wired.
   Remove them after the aggressive playback sweep is verified clean.

6. `cargo fmt --check` has pre-existing formatting drift.

7. Clippy is not currently available in the local Rust toolchain used during
   the migration recon.

## Real-Fixture Recon Snapshot

A targeted private-fixture snapshot was run before this repo foundation branch.
It passed mechanically: import, analyze, waveform, render, re-analyze, and export
checks all completed.

The objective result showed the risk this stabilization pass must address:

- Source: about -14.6 LUFS integrated, -4.0 dBTP true peak, 5.2 LU dynamic
  range.
- Rendered Universal output: about -8.4 LUFS integrated, -0.7 dBTP true peak,
  3.5 LU dynamic range.
- Checks warned for streaming headroom and low dynamic range.

Interpretation: current measurement/warning infrastructure sees the problem,
but the export UX needs to surface review before the user treats the render as a
clean success.
