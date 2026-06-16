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
- Live Master Out meters with Standard's selected target shown separately from
  the live readouts: `Live Now` is the short loudness window, `Play Avg` is the
  current playback run, and `Peak` is dBFS/peak behavior. Advanced keeps the
  exact technical labels/tooltips.
- A first-run guide in Standard view: subtle floating coachmarks that walk a
  new user to the Original→Mastered flip, then a send-off and one Advanced
  pointer. Dismissible, never blocking, persisted under
  `yes-master:first-run-guide`, resettable from Settings ("Show first-run
  tips again"). Entering Advanced or pre-flipping to Mastered ends it
  silently. Spec: `docs/superpowers/specs/2026-06-11-first-run-guide-design.md`.
- A first-launch welcome hero: when no track is loaded, the empty state
  shows the orb idling as the brand visual with the product promise and
  import CTA. Never a wall — import and drag-drop always work; reduced
  motion gets a static glyph.
- An analysis orb: a particle point-cloud animates in the waveform slot
  while analysis runs and persists through the waveform decode, then
  morphs into the track's real waveform when peaks arrive. Presentation
  only — playback readiness never waits on it; any interaction cuts it;
  `prefers-reduced-motion` disables it. Spec:
  `docs/superpowers/specs/2026-06-11-analysis-orb-design.md`.
- Real analysis progress: the backend emits `analysis:progress` events at
  the actual analyzer phase boundaries (decode → dynamics → stereo field →
  tonal balance → deep scan), batch-rescaled across multi-track imports.
  The UI's stage labels and bar track real work; the old paced timer
  remains only as a pre-first-event fallback.
- A clean Standard `Create Master` confirms itself: success card with file
  name, landed LUFS, and a Show file action. Invalid renders keep the
  prominent re-render alert; album receipts stay in their own flow.

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
chosen on the Album panel; Auto = highest source rate / first-track bit depth).
It resamples each track to that single album rate and resolves one album-wide
channel count. Mixed mono/stereo albums render stereo, with mono tracks upmixed,
and sources above stereo fold down to stereo delivery. Mixed-rate and mixed
channel-count albums therefore render one continuous file instead of failing.

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

Both lanes honor `YES_MASTER_CONFIDENCE_GATING`. Leave it unset for the default
release path; set it to `1` only when collecting Phase B confidence-gate evidence
against the same private fixtures/references.

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
4. Public signing, notarization, autoupdate, and store-style distribution remain
   deferred.
