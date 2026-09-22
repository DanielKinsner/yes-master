# Mac launch verification — September 22, 2026

**Later same-day update:** the owner resumed testing after pulling newer main.
The postponed checks are now complete, and a newly reproduced off-screen
startup defect is fixed. Use the [resumed verification and current status](2026-09-22-mac-resumed-verification.md)
for the final installed package, offline results and main transfer. The account
below records the earlier checkpoint and its then-current limits.

Earlier status: downloaded universal candidate passed the installed Mac journeys below.
The corrected local build passes automated/package checks. After the Mac locked,
the owner explicitly postponed the remaining hands-on checks and authorized
putting the fixes and this status report on remote `main` for another machine.
Public release remains NO-GO until the separate release gates are satisfied.

## Source and artifact identity

- Pulled remote main `b69277b8017b4c625c50ef7202650a0141252121` into local main
  `36543fd`, preserving the three September 6 Mac commits. The original checkout
  is retained on `codex/mac-before-main-sync-2026-09-22`.
- Continued on `codex/mac-launch-readiness-20260922`, including the reviewed
  `origin/codex/launch-sept22` work (`ed492cd`, application revision `e6db1f4`).
  Combined application revision `fb89d63` contains the fixes tested below;
  subsequent checkpoint commits update documentation only. Main integration
  and push are owner-authorized. No tag, release publication or deployment is
  part of this checkpoint; automatic main website deployment remains disabled.
- Actual machine: Apple M4, 16 GiB RAM, macOS 26.6.2 (25G83). The exposed and
  tested output is MacBook Pro Speakers, stereo 48 kHz.
- Downloaded the universal installer from unpublished draft
  `yes-master-v0.9.2-manual-9`. Its SHA-256 is
  `d49fe293210f0207bab141f7718d8e58ea60279ef58e9ff8e9ac6c164b34b71e`.
  Installed in `/Applications/YES Master.app` after preserving the previous app.
  Help identifies clean `e6db1f4 · 2026-09-22 08:37`.
- The DMG and installed app pass strict deep ad hoc signature verification.
  Both executable architectures are present. The installed encoder hash is
  `b3622fcd32e2c1236853c27d334a2788afd9fb996f55ab9193f1d67f1ea29c72`, exactly
  matching the recorded signed universal sidecar. Downloading with GitHub CLI
  does not establish a browser-quarantine/Gatekeeper first-open experience.
- Prior candidate [CI 35700809284](https://github.com/DanielKinsner/yes-master/actions/runs/35700809284)
  passed at `e6db1f4`. This is separate from local combined-source verification.

Raw evidence, private projects and audio remain ignored under
`test-output/mac-launch-2026-09-22/`. No private source or render is committed.
The existing session was backed up before app installation and interaction.

## Native installed journeys

Computer Use exercised the actual downloaded universal app, not a browser mock:

- Launch and restore the existing familiar 48 kHz source; analysis completed.
- Original playback, switch to Mastered with advancing playhead, Volume Match,
  live Clarity selection, keyboard seek, and Return to start while playing.
- Standard WAV export and Advanced AAC/M4A at 256 kbps through native Save As.
  The latter verifies the signed bundled encoder at runtime. Export settings
  preserved Clarity at 50%. The first WAV's test-entered full path became a
  literal colon-separated filename in macOS Save As; the resulting file was
  moved to the evidence directory. Subsequent checks used Go to Folder normally.
- Both saved formats play in macOS Quick Look with advancing time and full
  duration. This is external-player execution, not a by-ear approval.
- Import a second existing track, enter an Album title, move Builder first,
  cancel an active Album render and retry. Successful output contains two
  numbered M4A tracks, a continuous M4A and explained `metadata/manifest.json`.
- Save project, change its title, then reopen. Original title, two-track order
  and processing settings return; analysis refreshes normally. M4A remains
  selected within that session, but the project schema does not persist the
  export encoding (confirmed during the resumed fresh-launch check).
- Import an intentionally invalid WAV: recoverable error, two valid tracks and
  Album settings retained. No source audio is modified.
- Settings lists the actual default speakers and Refresh succeeds. Tab and
  Shift-Tab wrap between the first and last controls, and Escape closes it.
  The Track export receipt also restores keyboard focus to Export on Escape.

### Findings carried into the local combined build

1. The downloaded candidate reproduces the stale Album cancellation receipt
   during retry. The September 6 local fix clears the previous report only when
   a new export actually begins, preserving it if the folder picker is cancelled.
   That fix and its integration regression are retained in the combined source.
   The local Show files action and native title/meter corrections are retained too.
2. The newer Advanced-layout browser probe fails on Mac system-font metrics:
   Adapt strength and its value have only 0.39 px separation at both supported
   window widths. Reduced the gap between the two equal control columns to give
   each label more room, preserving font size, slider alignment and every
   control. Existing assertions remain unchanged.
3. Rust 1.95 strict Clippy rejects a redundant boolean expression in preview
   scheduling. Applied its logically equivalent form; no audio processing,
   defaults, sound policy or calibrated constants change.

## Independently checked delivered files

FFmpeg 9.0.1 decodes and measures all five complete files. FFprobe confirms
encoding, rate, channels and duration. Measurements broadly agree with the app's
qualified estimator; the independent peak estimator is distinct and reports
one decimal place. This is not a claim of numerical equality between estimators.

| File | Format / stereo rate | Duration | Integrated / range / independent peak |
| --- | --- | --- | --- |
| Track Clarity | WAV 24-bit / 44.1 kHz | 206.720 s | -14.0 LUFS / 6.9 LU / -4.7 dBTP |
| Track Clarity | M4A AAC / 48 kHz | 206.720 s | -14.0 LUFS / 6.9 LU / -3.6 dBTP |
| Album 01 Builder | M4A AAC / 48 kHz | 162.000 s | -14.5 LUFS / 6.0 LU / -4.3 dBTP |
| Album 02 Coat | M4A AAC / 48 kHz | 206.720 s | -14.0 LUFS / 6.5 LU / -5.0 dBTP |
| Continuous Album | M4A AAC / 48 kHz | 368.720 s | -14.2 LUFS / 6.4 LU / -4.3 dBTP |

The continuous duration equals both source durations combined. Per-file hashes,
probes and measurement logs are retained with the Album manifest.

## Automated and hardware evidence

- Combined-source frontend: 902 passed, 3 intentionally skipped. No skipped
  test is counted as passed. Dependency installation reports zero npm advisories.
- Desktop Rust: 711 passed, 34 intentionally ignored across 43 result groups,
  including all four real-fixture contracts on the existing owner source.
  The initial decode-surface run failed because the basic Homebrew FFmpeg lacks
  libvorbis; the full FFmpeg test tool restores that prerequisite and the complete
  suite passes without changing the test or decoder contract.
- Explicit actual-encoder matrices: two Album and four Track tests pass,
  covering exact lossless PCM, order/overrides/gaps, cancellation/collision/source
  protection, quality/rate/channel/short/silent cases and exact M4A gapless frames.
- Local ARM encoder built from pinned archives: 40 independent package cases pass.
  Mac signing regressions pass, including ARM, Intel and universal executables.
- Formatting and strict all-target Clippy pass after the equivalent expression
  correction; 15 preview regressions pass on that final Rust expression.
- Native fixed-256-frame, 48 kHz callback probes: 1,206 callbacks during 120 EQ
  changes and 1,387 callbacks during 1,200 combined-gain changes. Both record zero
  deadline misses, device errors, streaming errors and exhausted samples. These
  short muted measurements are distinct from by-ear or long-session evidence.
- Final headless verification passes: the complete landing suite plus 40 app
  scenario/viewport checks. Evidence: `test-output/headless/2026-09-22T14-43-50-388Z/`.
  The earlier failing captures/logs remain retained; no assertion was relaxed.
- A further callback probe uses a locally derived 120-second, 192 kHz stereo
  source, 44.1 kHz delivery and the actual 48 kHz output. It grants 256-frame
  buffers and records zero deadline misses/errors/exhaustion (see JSON evidence).
  This deliberately exercises both conversions; it is not a new musical source.
- Native Mastered lifecycle passes playing/paused 44.1/48/96 kHz delivery edits,
  paused seek/resume, four A/B transitions, first landing preparation, new/cached
  target application and paused cached-target resume. First unlanded playback
  meter arrives in 229 ms. Under this test workload the first whole-file 96 kHz
  preparation applies after 66.62 s; a new target takes 1.92 s and a cached target
  159 ms. Those are distinct from audio callback timing and normal 48 kHz UI
  observations. They do not justify an unconditional instant-preview claim.

## Corrected local package and postponed native recheck

The clean combined code revision is `fb89d63`. `npm run build:mac` produces an
Apple Silicon app and DMG, both locally verified and installed. This is separate
from the downloaded Intel + ARM universal candidate above.

| Local artifact | SHA-256 |
| --- | --- |
| ARM DMG | `e2ca44b9546e727f8a1e18b3f88455b578b72221d621d23ae3d24d58dab83ad5` |
| Installed main executable | `719469ec80cca2d7070847ac66d4e5cc904e2bc96b0854fc8e8d6359c864b117` |
| Final signed encoder | `9b396dccb4a1a52230e290f73d7e061b597fdc9a9d9ebdfdedf8d584e2fd1d7b` |

Strict deep codesign and the exact post-bundle encoder hash gate both pass. The
DMG was mounted read-only and its app copied into `/Applications`. Both previous
applications are preserved in ignored evidence. The original pre-test session is
restored byte-for-byte and the app is left closed.

Computer Use reported: "The Mac is locked and automatic unlock could not unlock
it." No unlock/bypass was attempted. The owner subsequently chose to postpone
the affected hands-on testing. The postponed work is specific: confirm the
corrected build in Help, inspect the corrected labels, repeat Album cancel/retry
and Show files, and verify offline playback/export. These are not passed checks
and do not require repeating the completed native journeys above.
A per-process network-denial profile and a failing curl control are prepared;
no interactive offline result is claimed. The isolated test process was stopped.
The machine's network settings were not changed.

Build tools: the existing Homebrew Node could not load its simdutf dependency,
so commands used the bundled Node 24.19 runtime. The broken Homebrew FFmpeg was
updated and the full FFmpeg variant installed for libvorbis fixture generation;
these developer tools are never bundled into the app. The product encoder was
built separately from the pinned recipe and signed/staged through its verifier.

## Handoff to another machine

The integrated checkpoint `60c526a47203cf3e7e324842539fe9197b5632ef` was pushed
to remote `main`; GitHub's branch readback and report-file API confirm it is
available. The working checkout is on `main` with no uncommitted changes.
The following documentation-only closeout records that completed transfer.

Pull `main` to obtain the integrated fixes, this report and the updated live
quality/release ledgers. The installed Mac app still identifies `fb89d63` because
the subsequent changes are documentation only. A new build on another machine
will identify its own checkout revision.

Private audio, rendered test files, raw local evidence and the local ARM installer
stay on this Mac; they are not repository contents. The report records their
scope and artifact hashes so source availability cannot be mistaken for an
installer transfer. Do not repeat passing suites without a relevant change.

Exact-main [CI 35744259967](https://github.com/DanielKinsner/yes-master/actions/runs/35744259967)
is running at `60c526a` at this handoff. Mac encoder signing stability has passed;
the full run is not yet a passing result. This is separate from both the earlier
candidate's green CI and the completed local tests above. The documentation-only
closeout does not start another duplicate full qualification run.

GitHub's dependency-alert refresh confirms both Vitest alerts are now fixed.
Nine pre-existing Rust alerts remain: five Linux-only `glib` entries and four
`serde_with` entries, including historical audit snapshots. Their applicability
assessment remains in the [launch preparation record](2026-09-21-launch-preparation.md#website-and-repository-follow-through).
No alert was dismissed and no security-test threshold was weakened. The zero npm
advisories above must not be read as zero repository-wide dependency alerts.

## Remaining release boundaries

The installed universal candidate narrows the previously missing real-Mac gate.
A local replacement build is a separate artifact; public downloads must name a
reviewed final revision and repeat the applicable exact-artifact checks.
Preserve the September 9 Windows listening approval. Final candidate listening,
recipient relinking permission, updater-key recovery, authorized public updater
install/relaunch and publication remain separate gates. Do not adopt an old
provisional beta date: the beta ends 56 calendar days after actual publication.

No Intel hardware, external audio interface, device unplug/reconnect, full
screen-reader speech or multi-hour stress result is inferred from this M4 pass.
