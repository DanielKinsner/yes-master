# Handoff - Track Header UI And Cleanup

Date: 2026-05-27
Repo: `C:\Users\SM - Dan\Documents\GitHub\yes-master`
Branch at handoff: `main`
Base pushed head before this addendum: `02613a3 Polish track header controls and add handoff`

## Purpose

This document lets a fresh agent continue from the current YES Master UI and
cleanup state without needing the prior chat. It intentionally lives in `docs/`
even though the handoff skill normally writes to a temp folder, because the user
expects to continue from another machine.

## Current Outcome

The accepted UI direction is the header-control layout shown in the user's final
mockup preference:

- `Track Master / Album Master` remains centered in the app top header.
- Track title and file metadata remain left-aligned in the track header.
- `ANALYZED` sits above the right-side track controls.
- `Original / Mastered`, `Volume Match`, and `Preview LUFS` live in the right
  side of the track header.
- The insight row is now a quiet divider below the title/metadata/control row.
- The waveform deck is clean: left play dock, waveform, and existing dB meters.
- The dB meter colors must stay as-is.

Do not reapply the left-aligned top-tab experiment from
`8ac2c0d fix: align mode tabs with workspace grid`. The user rejected that
layout, and `6a028bf` reverted it. The centered top app mode switch is currently
intentional; the track-level A/B switch is the one that belongs with the track
header controls.

Primary implementation files:

- `src/App.tsx`
- `src/App.css`
- `src/App.layout-css.test.ts`

The key structural choice is that `DeckPreviewOptions` is rendered inside
`TrackHeader`, not inside `.wf-deck`. The waveform deck should not regain a
floating center toolbar unless the product direction changes.

## Work Completed In This Slice

Use this as the commit map when a fresh agent needs to understand why the app
looks and behaves this way:

- Export review flow: `d110c1c`, `dd7f153`, `d27e62f`, `b9346ee`.
  Warning/critical rows change the button to `Export With Review` and require an
  explicit review panel path before export-anyway.
- Audition loudness modes: `217b2f2` makes `Volume Match` and `Preview LUFS`
  mutually exclusive. `Preview LUFS` is intentionally slower under heavy load;
  it should not block normal live edits when off.
- Compressor modes: `784a5ae`, `436afd1`, `254bc9a`, `15cc9c6`,
  `fa4c699`, `6b2dfc0`. `Preset` uses preset/density fallback compression,
  `Manual` replaces preset compression with user values, and `Off` bypasses
  creative/preset compression while leaving limiter, ceiling, LUFS landing,
  metering, and export warnings active.
- Private fixture/reference lanes: `0105efb`, `a5acd7b`, path-normalization
  fixes, `.gitignore` updates, and `88b3796`. The runners exist so agents can
  compare real private fixtures and external reference masters without
  committing audio, rendered masters, or private ledgers.
- Realtime responsiveness: `f38895a` avoids blocking play-master on LUFS
  preview work, `2848bc4` restores Preview-LUFS-off loudness, `f0c9365` keeps
  Original/Mastered switching from starting playback while paused, and
  `58c25d7` removes the temporary realtime diagnostic counters after the
  user-confirmed sweep was clean.
- Header polish: `e986c33`, `bcfee2a`, `86a7a06`, `7b0b32a`, `02613a3`.
  Several versions were tried; the current structure is the accepted direction.

The user manually confirmed export works, the current sound/presets feel good,
and the latest realtime responsiveness pass was "responsive as all hell" with
no reproduced timeout. Remaining annoyance: Original/Mastered can still have a
small audible/visual stutter, and live LUFS readout can take several seconds to
kick in under heavier load. Do not treat those as release blockers unless the
user re-prioritizes them.

## Verification Already Run

Commands run from the repo root:

```powershell
npm test
npm run build
cargo test
```

Observed result:

- `npm test`: 15 test files passed, 124 tests passed.
- `npm run build`: TypeScript build and Vite production build passed.
- `cargo test` from `src-tauri`: all non-ignored Rust tests passed
  (258 passed, 2 intentionally ignored dump tests).

Visual verification was also done with a local Vite dev server and headless
Chrome at `1600x940`. The temporary screenshots were inspected and then deleted
from `test-output/`, so they are not durable evidence in the repo. To reproduce:

```powershell
$port = 5173
$listener = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue
if (-not $listener) {
  $npm = (Get-Command npm.cmd).Source
  Start-Process -FilePath $npm -ArgumentList @('run','dev','--','--host','127.0.0.1','--port',"$port") -WorkingDirectory (Get-Location) -WindowStyle Hidden
  Start-Sleep -Seconds 4
}
$chrome = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
$outDir = Join-Path (Get-Location) 'test-output'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$out = Join-Path $outDir 'codex-ui-header-controls.png'
& $chrome --headless=new --disable-gpu --hide-scrollbars --window-size=1600,940 --virtual-time-budget=6000 --screenshot="$out" 'http://127.0.0.1:5173/' | Out-Null
Write-Output $out
```

## Conversation Context To Preserve

The user disliked several intermediate versions because `Original / Mastered`
felt visually off even after mathematical centering. The final insight was that
the row itself was wrong: the controls should be part of the track header, not a
floating control over the waveform. Do not keep solving this as a pixel-centering
problem.

Generated image mockups were used as brainstorming only. The accepted mockup was
the one where the controls sit on the right side of the track header and the
insight line divides the header from the waveform. Generated image files live
under the local Codex generated-images directory and are not committed.

## Other Cleanup State

The docs/hook cleanup and review artifacts were committed in
`02613a3 Polish track header controls and add handoff`:

- `docs/APP_BEHAVIOR.md`
- `docs/ARCHITECTURE.md`
- `docs/PRIVATE_AUDIO_FIXTURES.md`
- `docs/RELEASE_STABILIZATION.md`
- `docs/TESTING.md`
- `src/hooks/useTrackMaster.ts`
- `docs/CODEX_WIRING_REVIEW_2026-05-27.md`

`docs/CODEX_WIRING_REVIEW_2026-05-27.md` is the deeper wiring and cleanup review.
Do not duplicate it here; use it as evidence, but note its new post-review
status section before treating any item as open.

Important highlights from that review:

- Safe-delete candidates and diagnostic counters were already removed in
  `65381d5` and `58c25d7`.
- The loudness-target controls still have divergent side effects and need a
  product decision before changing calibration behavior.
- A focused test is recommended around live-chain updates when toggling
  Original/Mastered, undoing, and editing while paused/playing.

## Private Evidence To Know About

The repo intentionally does not contain the private audio or private ledgers.
On Dan's current machine, the local-only inputs were observed under:

- `private-audio-fixtures/` with three full-track WAVs plus `manifest.json`.
- `tests for presets/` with one source WAV and four external reference-master
  WAVs for Universal, Clarity, Oomph, and Tape.

Fresh agents on another machine should not assume these files exist. If they do
exist, use the documented slow lanes:

```powershell
cd src-tauri
cargo run --example private_fixture_matrix -- --manifest ..\private-audio-fixtures\manifest.json --output ..\test-output\private-fixture-matrix
cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning
```

The durable aggregate from the last reference retune is in
`docs/APP_BEHAVIOR.md` under "Reference Retune Snapshot"; do not commit the
ignored CSV/JSON ledgers or rendered WAVs.

## Recommended Next Steps

1. Re-open the app visually and confirm the accepted header layout still feels
   right with the user's real viewport and the `It's a coat` fixture.
2. If continuing cleanup, start with the still-open wiring risks in
   `docs/CODEX_WIRING_REVIEW_2026-05-27.md`, not the already-resolved safe
   deletes.
3. Decide the two loudness-target surfaces: right-rail LUFS target vs. center
   loudness dropdown. They currently write the same field with different
   profile/custom semantics.
4. Add the focused live-chain test around Original/Mastered, undo, and edits
   while paused/playing before touching that predicate.
5. Run `npm test` and `npm run build` after frontend changes; run `cargo test`
   after backend or DSP changes.
6. Keep private audio and generated private fixture outputs uncommitted.

## Suggested Skills

- `frontend-design`: Use for any further track-header or console polish.
- `handoff`: Use again before switching machines or agents.
- `audit` or code-review stance: Use before dead-code deletions or backend
  command cleanup.
- `diagnose`: Use for any live-chain playback, Original/Mastered, or LUFS
  preview behavior issues.

## Git Hygiene Notes

- Do not commit local generated screenshots under `test-output/`.
- Do not commit private audio, rendered private masters, or private ledgers.
- The current target is one cleanup/UI handoff commit on `main`, followed by a
  push to `origin/main`.
