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

1. Private-fixture and private-reference slow lanes must be rerun for DSP/export
   changes; automated tests cannot approve taste.
2. Oomph needs listening notes before another targeted tuning pass.
3. `cargo fmt --check` has pre-existing formatting drift.
4. Clippy is not currently available in the local Rust toolchain used during
   the migration recon.
5. Windows packaging still needs local verification before release-candidate
   status.
