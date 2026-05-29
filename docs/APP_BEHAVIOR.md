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
- Original/Mastered audition at the same playhead.
- Region selection and loop playback.
- Presets and intensity.
- Visual EQ/tone shaping.
- Volume Match for audition only.
- Delivery profile selection.
- Advanced controls.
- Explicit compressor modes.
- Per-band compressor detail.
- Delivery format selection.
- Explicit save destination for export.
- Warning-aware export review.
- Post-render export receipt/checks.
- Baseline Settings and contextual Help dialogs.
- Visible Save/Open Project feedback.

Track Master Delivery Profile and Delivery Format are authoritative for Track
Master exports:

- Named profiles set their owned target LUFS, ceiling, bit depth, and sample
  rate together.
- Custom Source keeps the source sample rate.
- Custom format can request 44.1 kHz, 48 kHz, or 96 kHz.
- The rendered WAV, export receipt, and export checks report the effective
  rendered sample rate and bit depth.
- A requested/rendered sample-rate mismatch is treated as a technical integrity
  issue.

Album Master now honors an album-wide Delivery Format (sample rate + bit depth,
chosen on the Album panel; Auto = highest source rate / first-track bit depth). It
resamples each track to that single album rate, so albums built from mixed-rate
sources render one continuous file instead of failing. Channel-count parity
(mono vs stereo tracks) remains a hard error and is still deferred.

Mastered preview readiness timeouts surface recoverable user-facing guidance
instead of silent non-playback.

## Project, Settings, And Help

- Save Project and Open Project use `.ams.json` project files.
- Save/Open success and cancelled dialogs surface calm visible feedback.
- Open Project restores state, selects a track when possible, and reports
  recovery issues that need user action.
- Settings covers current baseline app defaults and app info.
- Help explains current Import/Analyze, Original vs Mastered, Volume Match vs
  Preview LUFS, Delivery Profile/Format, Export Review, and Save/Open behavior.

## Export Checks And Review

The backend checks rendered-output measurements for:

- True peak above critical/safe streaming thresholds.
- Loudness above very-loud territory.
- Dynamic range below the low-dynamic-range threshold.
- Bit depth below 16-bit.
- Non-finite LUFS metering.
- Already-compressed source combined with moderate/heavy preset compression.

Before an export receipt exists, the right rail derives preflight review rows
from current source analysis for true peak, loudness, and dynamic range. These
rows make already-hot or already-compressed sources visible before the user
treats export as a clean path.

The export button is warning-aware:

- No review rows: `Export Master`.
- Warning or critical review rows: `Export With Review`.
- First `Export With Review` click opens an inline review panel.
- `Adjust Settings` closes the panel and does not export.
- `Export Anyway` calls the existing export path.

Quality rows are advisory when the app can write a file. Technical failures
still stop export through the render/save path: invalid paths, cancelled save
dialogs, decode/render/write failures, or corrupt/non-finite render state.

## Compressor Modes

The app has an explicit compressor mode field:

- `Preset`: current preset/density fallback behavior.
- `Manual`: user per-band values replace preset compression.
- `Off`: bypasses creative/preset compression only.

`Off` does not bypass the limiter, ceiling protection, LUFS landing, metering,
or export warnings.

The per-band compressor card labels preset fallback values as `Preset`, not
track-aware `Auto`. If a low-dynamic-range source is loaded while `Preset` mode
is active, the card gives local guidance to lower density or switch Off if
movement collapses.

## Private Fixture And Reference Lanes

Private audio is local-only and ignored by git.

The already-mastered matrix runner measures preset/compressor cases against
private fixtures:

```powershell
cd src-tauri
cargo run --example private_fixture_matrix -- --manifest ..\private-audio-fixtures\manifest.json --output ..\test-output\private-fixture-matrix
```

The private reference tuning runner compares YES Master presets against external
reference masters:

```powershell
cd src-tauri
cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning
```

Both lanes write ignored ledgers and rendered WAVs. Do not commit private audio,
rendered private masters, waveform images from private audio, or fixture-specific
ledgers.

## Reference Retune Snapshot

The 2026-05-26 reference retune preserved `-14 LUFS` delivery landing and
compressor-mode semantics. External references were hotter than YES Master
exports; that is expected for this slice.

Observed aggregate after the retune:

```text
universal  ref -10.53 LUFS, YES -14.00 LUFS, DR gap -0.31 LU, warnings dynamic_range_low|comp_density_on_compressed_source
clarity    ref -12.04 LUFS, YES -14.00 LUFS, DR gap -0.50 LU, warnings dynamic_range_low|comp_density_on_compressed_source
oomph      ref -11.87 LUFS, YES -14.00 LUFS, DR gap -1.10 LU, warnings dynamic_range_low|comp_density_on_compressed_source
tape       ref  -9.91 LUFS, YES -14.00 LUFS, DR gap -0.82 LU, warnings dynamic_range_low|comp_density_on_compressed_source
```

Oomph remains the least-matched preset and needs careful listening before any
further subjective retune.

## Current Gaps

1. Manual listening signoff is still required; automated tests cannot approve
   taste.
2. The full private fixture matrix needs a longer unattended run after the
   representative subset completed.
3. Oomph needs listening notes before another targeted tuning pass.
4. Album Master sample-rate + bit-depth delivery parity has landed; only
   channel-count parity (mono vs stereo tracks) remains deferred.
5. Public signing, notarization, autoupdate, and store-style distribution remain
   deferred.
