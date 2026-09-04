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

## Remaining follow-up beyond this correction pass

- Long-session live-meter growth and cold Volume Match latency need dedicated
  profiling and regression work. Neither is claimed fixed by the limiter benchmark.
- Album receipt expansion (per-track target/peak presentation) and broader
  controller/CSS simplification remain follow-up product/maintenance work.
- Native audio-device behavior and the combined by-ear comparison remain
  physical tests; browser captures and host bridge tests do not establish them.

## Combined listening checklist

Compare baseline and final processing on transient/dense tracks, wide stereo,
quiet breakdowns into loud choruses, already-mastered audio, and high-rate
sources. Sweep intensity and manual compression; toggle Original/Mastered and
Volume Match at a fixed playhead; compare audition with the saved master.
Record one consolidated taste decision after the objective suite is green.
