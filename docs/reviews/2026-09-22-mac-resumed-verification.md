# Resumed Mac verification — September 22, 2026

The postponed hands-on Mac checks are complete. The app passes the tested M4
playback, project, offline and export journeys. A newly reproduced startup bug
is fixed: the window now opens fully on-screen. This is a tested Mac development
checkpoint; public release remains NO-GO for the separate gates below.

## What changed

- Pulled main from `b17aa228` to `aec5f6ea` before resuming, including the newer
  modal-shortcut, render-efficiency and multi-core peak-measurement changes.
- `76122a4`: removed competing startup centering. macOS had already fitted the
  requested 1920px window to the 1512px display, but a queued centering action
  subsequently shifted it 204px off the left edge. Centering now happens in the
  display-fit routine only when it chooses normal window placement. Existing
  maximize, layout-floor and zoom behavior is retained.
- The defect repeated across clean launches. Native geometry changed from
  physical `(0,66)` to `(-408,66)` after setup; the corrected probe remains at
  `(0,66)` with a 3024×1716 frame at 2× scale. The regression fails with the old
  config and passes with the fix. Temporary instrumentation was removed.
- `9379feb`: corrected a test helper so a 96 kHz callback probe also performs
  its background preparation at 96 kHz. Previously that worker stayed at
  48 kHz. Reports now include the worker rate. This changes test accuracy,
  not application sound. Historical Windows callback results retain their
  actual scope; the investigation and testing guide are corrected.
- No preset voicing, calibrated constants, export defaults or source audio changed.

## Actual installed-app checks

Machine: Apple M4, 16 GiB, 10 logical cores, macOS 26.6.2 (25G83), built-in
MacBook Pro Speakers at 48 kHz. Computer Use operated the installed native app.
The initial clean package identified `aec5f6e` in Help; final package identity
and cold-launch results are recorded below.

- Help opens with the correct version/build. Space, A, arrows, question mark
  and Undo no longer act behind the open dialog. Escape closes it with playback
  paused at the original position and settings retained.
- Advanced control labels/readouts fit. Playback stays responsive while
  Clarity at 85%, -11 LUFS, 96 kHz delivery prepares; changing to Tape while
  playing completes another preparation with the playhead continuing to advance.
- Export a complete 96 kHz/24-bit WAV while playing. The finished receipt and
  Finder's Show file target agree with the actual file.
- Open the existing two-track QA project: title, order, processing settings
  and analysis restoration work. The schema does **not** save export encoding;
  the earlier report's M4A restoration observation was a retained in-session
  selection, not persistent project codec storage. That earlier wording is fixed.
- Album: cancel an active export, cancel the retry folder picker (the previous
  receipt remains), then retry for real (the stale receipt clears). Export
  finishes with two M4A tracks, continuous M4A and metadata. Show files reveals
  the correct folder. macOS Quick Look plays the continuous output with
  advancing time. This is external playback, not subjective listening approval.

All playback, project and export steps above after the Help check ran with
network access denied to the app and its child processes using `sandbox-exec`.
A curl negative control under the same profile could not connect. The updater
reported the expected network failure while local work continued. System
network settings were unchanged. This establishes the isolated app's offline
behavior, not a browser-download/Gatekeeper first-open experience.

## Independent complete-output checks

Independent FFmpeg 9.0.1 decoded and measured every complete file below.
The M4A decoded frame counts exactly match the sources, including the combined
17,698,560-frame Album. No duration/padding discrepancy was hidden by rounding.
The independent peak meter and the app's qualified estimator are distinct;
their values are not claimed to be numerically identical.

| Output | Encoding / stereo rate | Duration | Independent LUFS / LRA / true peak |
| --- | --- | ---: | --- |
| Tape Track | WAV 24-bit / 96 kHz | 206.720 s | -11.0 / 5.3 LU / -3.6 dBTP |
| Album 01 Builder | AAC/M4A 256 kbps / 48 kHz | 162.000 s | -14.5 / 6.0 LU / -4.3 dBTP |
| Album 02 Coat | AAC/M4A 256 kbps / 48 kHz | 206.720 s | -14.0 / 6.5 LU / -5.0 dBTP |
| Continuous Album | AAC/M4A 256 kbps / 48 kHz | 368.720 s | -14.2 / 6.4 LU / -4.3 dBTP |

The private Mac Coat source is 206.72 s, SHA-256
`80b6974095a9d0ff838854498dd63a69866b06c1d02a27a10e6f30c86b3e49ca`.
It differs from the 252.36 s Windows investigation source. No exact reproduction
of that owner's reported click-to-audible timing is claimed.

## Automated and actual-device evidence

- Frontend: **905 passed, 3 intentionally skipped**; production typecheck/build
  pass. The previous other-machine wording of “908 tests” included the skips.
  All 36 release-document contract tests pass after this report/ledger update.
- Headless: complete landing checks and **40 app scenario/viewport checks**
  pass (`test-output/headless/2026-09-22T21-47-08-611Z/`). This is browser evidence.
- Full desktop Rust with the existing private real fixture: **716 passed,
  34 intentionally ignored**, 43 result groups, all four fixture contracts run.
  Final startup regression suite separately passes **12 tests**, including the
  newly added regression. Formatting and strict all-target Clippy pass.
- Explicit staged-encoder matrices: **four Track and two Album tests pass**,
  including lossless PCM equality, delivery quality/rate/channel cases, source
  protection, cancellation, collisions and independent exact M4A gapless frames.
- Six muted, alternating single-worker/budgeted callback probes: **7,244 actual
  callbacks; zero deadline misses, device errors or exhausted samples**. Both
  requested and granted buffers are 256 frames at 48 kHz, with live 96 kHz
  delivery conversion and verified 96 kHz concurrent preparation. Worst callback
  used 16.3–16.5% of its budget single-worker and 16.4–24.0% with the bounded
  worker pool. Two background preparations complete per budgeted run (2.70–2.83 s);
  single-worker runs are cancelled by the six-second probe before finishing.
- Separate actual Mastered lifecycle: playing/paused 44.1/48/96 kHz edits,
  paused seek/resume, A/B, full cold preparation, new/cached target application
  and paused cached-target resume pass. First unlanded meter: 276 ms. First
  whole-file landing settles at 17.70 s; emitted-output revision follows at
  17.87 s. New/cached target application: 1.96 s / 183 ms. These are application
  observations, not DAC latency or an audible-hiccup verdict.

Audio timing ran without simultaneous builds. Detailed scope and the corrected
historical benchmark interpretation are in the
[preset investigation](2026-09-22-preset-preview-investigation.md#mac-follow-up-corrected-benchmark-scope).
No passing test tolerance was weakened. Shared bridge behavior is unchanged by
these two fixes; the pulled revision's complete platform CI is green.

## Final installer and transfer

Clean build `9379feb · 2026-09-22 15:12` was packaged with `npm run build:mac`,
installed from the read-only mounted DMG and verified in Help. Strict deep
ad hoc codesign and the exact post-bundle signed-encoder hash check pass.
This local installer is Apple Silicon; no new notarization or universal package
is claimed. Two fresh installed launches, including one under network denial,
open fully on-screen without manual movement or zooming. Standard titles and
live-meter labels are readable, Advanced labels fit, and the second launch
restores Standard view. Offline Mastered playback advances through preparation
completion and remains responsive. The app is left closed and the owner's
original session is restored byte-for-byte.

| Final local artifact | SHA-256 |
| --- | --- |
| ARM DMG | `bf841afa2c75fe7bcd6f2ecf9b0d60852f955ed50f243f314a914d2a19aad0e5` |
| Installed main executable | `0c8336abfaa0b1b3ead4b3bf1f6a3d4e3e063a937aabf2a629d4c6a93be26631` |
| Final signed encoder | `9b396dccb4a1a52230e290f73d7e061b597fdc9a9d9ebdfdedf8d584e2fd1d7b` |

The earlier main CI [35744259967](https://github.com/DanielKinsner/yes-master/actions/runs/35744259967)
at `60c526a` and pulled-main CI
[35769648800](https://github.com/DanielKinsner/yes-master/actions/runs/35769648800)
at `aec5f6e` both completed successfully, all 11 jobs. The new code checkpoint
`9379feb6f4d8c0d7010ce8e00e2b51cb2eb75f96` is pushed to main; its
[CI 35791097861](https://github.com/DanielKinsner/yes-master/actions/runs/35791097861)
is still running at this documentation checkpoint. Keep that exact-revision CI distinct from the local and prior-CI results.
The report/ledger closeout changes documentation only and does not create a
duplicate full qualification run.

Local raw evidence, private projects, audio, output files and preserved prior
apps remain ignored under `test-output/mac-resume-2026-09-22/`. They are not
uploaded into the repository. Pull main on another machine for the fixes and
this report; the local installer is not transferred by a source push.

## Where the application stands

The previously postponed native Mac/offline checks are closed for this M4 and
the recorded journeys. The startup defect is fixed. There is no newly observed
unresolved blocker in those tested flows. Public launch still requires the
[live release gate](../plans/beta-go-no-go.md): final candidate listening,
recipient relinking permission, permanent updater-key recovery and the
authorized public updater/install/relaunch/publication transaction. Preserve
the existing September Windows listening approval; this pass does not replace it.

Still unproven: the reported completion-boundary audible hiccup, physical Intel
Mac behavior, external/Focusrite interface behavior and hot-unplug, full
screen-reader speech, multi-hour stress and browser-quarantined first launch.
Built-in 256-frame callback success does not certify other audio hardware.
No release, public installer, tag, website deployment or sonic-policy change
was activated by this task.
