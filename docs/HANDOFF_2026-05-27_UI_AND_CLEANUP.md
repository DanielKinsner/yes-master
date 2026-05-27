# Handoff - Track Header UI And Cleanup

Date: 2026-05-27
Repo: `C:\Users\SM - Dan\Documents\GitHub\yes-master`
Branch at handoff: `main`

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

Primary implementation files:

- `src/App.tsx`
- `src/App.css`
- `src/App.layout-css.test.ts`

The key structural choice is that `DeckPreviewOptions` is rendered inside
`TrackHeader`, not inside `.wf-deck`. The waveform deck should not regain a
floating center toolbar unless the product direction changes.

## Verification Already Run

Commands run from the repo root:

```powershell
npm test
npm run build
```

Observed result:

- `npm test`: 15 test files passed, 124 tests passed.
- `npm run build`: TypeScript build and Vite production build passed.

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

The following docs/hook changes are part of the loose-end cleanup state and are
expected to be committed with this handoff:

- `docs/APP_BEHAVIOR.md`
- `docs/ARCHITECTURE.md`
- `docs/PRIVATE_AUDIO_FIXTURES.md`
- `docs/RELEASE_STABILIZATION.md`
- `docs/TESTING.md`
- `src/hooks/useTrackMaster.ts`
- `docs/CODEX_WIRING_REVIEW_2026-05-27.md`

`docs/CODEX_WIRING_REVIEW_2026-05-27.md` is the deeper wiring and cleanup review.
Do not duplicate it here; use it as the queue for the next cleanup pass.

Important highlights from that review:

- Safe-delete candidates: unused `LevelsPanel`, `StereoWidthGauge`, orphaned
  advanced state, unused album intent updater, and old album render command.
- `get_diag_counters` should be removed only after the realtime sweep is
  verified clean.
- The loudness-target controls have divergent side effects and need a product
  decision before changing calibration behavior.
- A focused test is recommended around live-chain updates when toggling
  Original/Mastered, undoing, and editing while paused/playing.

## Recommended Next Steps

1. Re-open the app visually and confirm the accepted header layout still feels
   right with the user's real viewport and the `It's a coat` fixture.
2. If continuing cleanup, start with the pure dead-code deletes in
   `docs/CODEX_WIRING_REVIEW_2026-05-27.md`.
3. Run `npm test` and `npm run build` after frontend deletes.
4. Run Rust tests if backend command cleanup is touched.
5. Do not remove diagnostic counters until the realtime playback sweep is
   explicitly verified clean.
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
