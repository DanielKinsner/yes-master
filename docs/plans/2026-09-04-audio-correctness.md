# Audio correctness and regression protection

Owner authorized implementation on 2026-09-04 following the solo audit of
`8e1dd6c`. Release preparation is parked. Sound-affecting fixes are collected
for **one listening pass toward the end**, not individual approval stops.
No preset retuning or activation of gated adaptive features is included.

## Work ledger

- [x] Regression tests: nonstationary preview, compressor transfer, limiter,
      stereo/sample-rate analysis, delivered-file metering, truthful UI.
- [x] Preview: whole-track landing, existing asynchronous cache, safe pending behavior.
- [x] Compressor: monotonic continuous soft knee.
- [x] Limiter: independent peak checks and smooth lookahead attack.
- [x] Analysis: stereo energy and sample-rate correctness; accurate LRA language.
- [x] Delivery: measurements of delivered PCM, consistent float/integer behavior.
- [x] UI: signal chain truthfulness, Standard minimum-size controls, receipts.
- [x] Album assembly: stream delivered PCM instead of retaining the whole album.
- [ ] Finish profiling long/high-rate work and repair demonstrated bottlenecks.
- [x] Full required local verification and documented remaining platform limits.
- [ ] One combined owner listening pass (pending owner, after mechanical work).

## Regression contract

Tests use deterministic generated audio, independent reference measurements,
and mathematical properties rather than only current-output snapshots. Watch
each new regression fail before its fix. Keep all tests in normal CI lanes;
no private audio or manual opt-in required for these regressions. Existing
sound snapshots are changed only for explained DSP changes, never to conceal
a failure. Preserve source files, export warning semantics, playhead continuity,
and optional audition-only Volume Match.

## Implemented regression coverage

- `src-tauri/tests/audio_invariants.rs`: quiet-middle/loud-ends preview versus
  export; monotonic soft-knee sweeps; physical-frequency analysis at multiple
  sample rates; stereo side-energy preservation; float headroom and 16/24-bit
  readback parity; limiter lookahead continuity and 48 burst/rate/phase cases
  through 192 kHz measured independently with ebur128.
- `src-tauri/tests/album_render.rs`: actual delivered LUFS and sample identity
  between per-track and continuous WAVs at 16/24/32 bits. The new test first
  reproduced a 3.8 LU receipt error on intentionally overdriven 16-bit output.
- Rust unit tests cover safe pending preview gain, low-LRA informational
  semantics, backend-resolved stage activity, and separate 31-band stereo
  polarity invariance. Frontend tests cover delivered
  versus source labels, stale readout removal, and actual factory processing.
- The normal headless lane now checks every Standard style, intensity chip,
  loudness choice, and export control for viewport bounds and pointer reachability
  before scrolling. This first reproduced the 1360x740 clipping failure.
- Removed 18 table-mirror tests with the redundant SignalChain preset tables;
  replaced them with resolved-state behavior/wire-contract coverage.

## Sound reference changes

The compressor knee and limiter detector/envelope corrections intentionally
change output. No preset constants or golden tolerances were relaxed. Regenerated
the affected Oomph, Punch, Loud, and Warmth PCM references and preset fingerprints;
the largest observed fixture changes were Punch/Loud dynamics. Deep-analysis
fixture updates change only the sample-rate-aware three-band diagnostic values.
These are mechanical references, **not listening approval**.

## Local implementation and verification (2026-09-04)

Implementation commits:

- `e4de416`: compressor/limiter math, whole-track preview landing, analysis,
  delivered PCM metering, bounded album assembly, and audio regressions.
- `858383b`: backend-resolved signal chain and stale readout invalidation.
- `0212a7d`: delivered/source receipt values and accurate LRA language.
- `93aafae`: Standard minimum-height layout and browser regression coverage.
- `13d285e`: separate 31-band polarity regression and consistent warning fixtures.

| Lane | Result |
| --- | --- |
| Frontend | 830 tests across 82 files passed; TypeScript/Vite build passed |
| Desktop Rust | 625 passed, 7 explicitly ignored diagnostics; fmt and strict all-target Clippy passed |
| Private fixture | `AMS_RUN_REAL_FIXTURE=1` contracts lane: 54 passed; actual existing fixture import, analysis, render and saved-file analysis ran |
| Headless | 31 app scenario/viewport cases plus landing suite passed, including final preview-fixture consistency rerun |
| iPhone bridge | All-target host check and 46 tests passed; 1 diagnostic ignored |
| Android bridge | 26 host tests and arm64 API-29 cross-check passed; initial dependency-linker failure cleared on bounded-job retry |
| Windows packaging | MSI and NSIS builds passed; no installation or publication |

These are local working-tree checks of the changes, not remote CI or a signed,
installed release candidate. macOS and real-device playback were not exercised.
Landing capture assets were regenerated with the normal digest gate.
Local logs: `test-output/audio-correctness-2026-09-04/`. Final browser evidence:
`test-output/headless/2026-09-04T22-24-20-052Z/`. These ignored local artifacts
are not shipped assets or substitutes for running the lanes on another OS.

The optimized development chain benchmark processed ten seconds of stereo
synthetic audio at roughly 22x realtime (44.1 kHz), 7x (96 kHz), and 2.4x
(192 kHz) while other builds were active. This is throughput evidence, not an
audio callback deadline guarantee. The high-rate margin and full-track preview
cost still deserve dedicated profiling on slower hardware; no claim that all
performance concerns are closed.

## Follow-up implementation (2026-09-04)

- `7c4c8d6`: bounded live integrated-meter history, allocation-free feed/reset,
  and limiter envelope pruning. The long-listen test first reproduced ten
  callback allocations, then passed with zero. Live histogram bins are 0.1 LU;
  export measurement remains exact. Limiter sound references did not change.
- `d2e30af`: Album receipts expose each track's resolved target, delivered LUFS,
  delivered true peak, and ceiling. Target shortfalls stay informational; older
  receipts explicitly label missing measurements. Browser scenarios expand the
  real export receipt and assert all four tracks' results. Fixed the clean-export
  test's race by waiting for its asynchronous receipt before asserting it.
- `ac8859c`: cold Volume Match runs on the existing preview worker; its early
  result need not wait for whole-track landing. The command-thread lookup has
  no PCM/render access. One pending request replaces duplicate controller logic.
  Repeated A/B switches reuse the current PCM worker, and recreated audio states
  receive distinct source epochs so old device-state workers cannot poison caches.
  Both lifecycle regressions failed before their respective fixes.

The fixed-size meter replaces the growing custom block history; this also removes
its repeatedly scanned logic. Preview coefficient publication and measurement
scheduling now have shared paths. These are targeted maintenance changes; file
size alone is not a reason for a broader controller or CSS rewrite.

### Follow-up evidence

- Frontend: 831 tests / 82 files passed; TypeScript and Vite builds passed.
- Rust: full suite 630 passed / 7 ignored, followed by all 448 final library
  tests after the additional device-epoch regression; strict all-target Clippy
  and formatting passed. DSP fingerprints and golden tolerances unchanged.
- Private fixture contracts: 54 passed with `AMS_RUN_REAL_FIXTURE=1`, including
  actual fixture analysis, rendering, and saved-file metering.
- iPhone: all-target check and 46 host tests passed / 1 diagnostic ignored.
  Android: 26 host tests and arm64 API-29 cross-check passed.
- Headless: 31 app scenario/viewport checks plus landing suite passed; the
  expanded Album receipt at 1360x740 was also visually inspected.
- Final Windows MSI and NSIS packaging passed at implementation commit
  `ac8859c`; no installation, push, publication, or deployment. This is local
  evidence, not macOS validation or remote CI.

Logs: `test-output/audio-correctness-followup-2026-09-04/`. Browser evidence:
`test-output/headless/2026-09-04T23-07-10-511Z/`. Neither substitutes for
hardware playback or listening approval.

Optimized-development diagnostics on this machine: standalone 192 kHz limiter
improved from roughly 3.2x realtime to 5.2x–10.3x while limiting, and 37x on quiet
input. The full chain processed ten seconds of synthetic stereo at about 70x
(44.1 kHz), 29x (96 kHz), and 7.4x (192 kHz), compared with the prior pass's
22x/7x/2.4x. These are indicative throughput measurements taken during other
build activity, not guarantees of callback deadlines on slower devices.

### Remaining owner batch and performance limits

The mechanical follow-up queue above is implemented. One combined by-ear and
native playback check remains, including cold Volume Match and rapid A/B on real
hardware. Full-track landing still has a full-track measurement cost, and first
playback may decode synchronously if prewarming has not completed. Slower-device
profiling remains useful; no universal latency guarantee is claimed. Preset
retuning, adaptive-compressor calibration, and other parked feature expansions
remain outside this correction pass. Release activity remains parked.

## Combined listening checklist

Compare baseline and final processing on transient/dense tracks, wide stereo,
quiet breakdowns into loud choruses, already-mastered audio, and high-rate
sources. Sweep intensity and manual compression; toggle Original/Mastered and
Volume Match at a fixed playhead; compare audition with the saved master.
Record one consolidated taste decision after the objective suite is green.
