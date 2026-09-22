# C1 gain-copy consistency — development findings

This records new comparisons against the preserved eight-source development
corpus. It changes no production voicing or selector policy. Evidence paths are
under ignored `test-output/mastering-quality-implementation-20260915/`; private
audio remains outside Git. All eight original-source coarse runs are complete
(480 deliveries, including Coat's separate 20/40 runs). Whole-file finalist
checks, refinements and the -40/-80 dB copy queues continue.

## Two different failures

The [declared numerical budgets](2026-09-15-drive-prototype-protocol.md) were
written before examining copy outcomes. `compare_gain_copies.py` verifies source,
report and WAV hashes and keeps the original analysis/guards and scoring anchors.
Its comparisons do **not** establish gain invariance of fresh production analysis.
The existing control and attenuate-only approaches are explicitly level dependent.

| Comparison | Normalized candidate PCM | Selected candidate/fallback |
| --- | --- | --- |
| Piano, -40 dB | Fails: maximum sample difference 0.001627; one delivered-LUFS difference 0.002312 LU exceeds 0.001 LU | All coarse selections unchanged |
| Coat, -40 dB | Fails: maximum sample difference 0.000823 | Loud's preserving outcomes change; the control-based tone/stereo limits change with input level |

Operating drive and resolved coefficients match on these normalized comparisons.
Piano's raw LUFS changes by at most 0.000014 LU; section/attack/band/side/correlation
differences meet the declared budgets. Small differences in summary measures do
not excuse the failed sample criterion. These budgets are reproducibility checks,
not an audibility standard.

Coat's `c1-coat-m40-consistency-v4.json` adds an explicitly diagnostic selector
comparison: hold the **original** control's tone/stereo limits fixed while scoring
the copy candidate. All six preserving candidate/fallback identities then match.
The actual frozen selector results remain failures. This isolates dependence on
the reference limits; it does not adopt that diagnostic as a shipping selector.
A production rule needs a reproducible level-independent reference or justified
absolute constraints, validated before the unseen holdout is opened.

## Reproduced numerical mechanism

`mastering_quality_precision_probe` reconstructs Piano's retained
`universal75-t14-02-bounded` candidate (-9 dB relative to the single rule) from
its exact requested settings and operating drive. It checks the resolved
coefficient string and reproduces **both original and -40 dB delivered WAV
sample arrays exactly**, using their retained final gains. No new full-file peak
search or source analysis is required to diagnose this response.

`c1-piano-m40-precision-v1.json` records:

- Normalized input difference: maximum 5.96e-8, RMS 5.22e-9.
- After the first production f32 subsonic high-pass: maximum 0.0001165.
- Full raw response difference: maximum 0.0002233; delivered maximum 0.0003799.
- Giving both raw responses the same final gain still leaves maximum 0.0003085.
  Final gain differences are therefore not the sole cause.
- A diagnostic high-pass bypass reduces the full raw difference to 0.00002852.
  This is mechanism isolation, **not** a proposed removal of the high-pass.
- Keeping the same f32 filter coefficients, but using f64 arithmetic/state and
  f32 output at every filter boundary, leaves at most 8.94e-8 after the complete
  eleven-filter input/EQ sequence.

These results identify recursive filter arithmetic as a substantial contributor.
The wider-state experiment does not yet cover the complete nonlinear chain,
compressor split filters, performance or audible differences. Saturation is
continuous in the input sample; this finding is separate from E's known
amount-zero calibration discontinuity.

**Next correction experiment:** evaluate wider filter state without changing
coefficients or preset intent, compare complete responses and final verification,
then measure callback/session cost. Retain the failed f32 results and frozen
baseline. No blanket fidelity or listening verdict is inferred.

## Replay and retained failures

Build `cargo build --manifest-path src-tauri/Cargo.toml --example
mastering_quality_precision_probe --target-dir src-tauri/target/codex-rc`.
Run the executable with the original report path, copy report path, candidate id
and a fresh output JSON path. The Python comparator takes repeatable
`--base RUN METRICS`, one `--copy RUN METRICS`, and a fresh `--output` path.
Coat needs both original runs; its first report predates the top-level source-gain
field, so every row must explicitly prove gain zero.

Piano consistency v1 and Coat consistency v3/v4 are retained. Coat v1 used an
incorrect extra-run path; v2 encountered the legacy report's missing top-level
gain field. Both controller errors occurred before writing a result and are
preserved in their logs. Neither is an audio failure or a reason to rerender
completed source material. The precision example built and replayed successfully;
strict all-target Clippy also passes with it present.

The original eight sources and these gain copies are development material.
No unseen holdout has been opened, and unresolved sonic-policy choices remain
separate from these engineering experiments.

## Full-chain precision correction

The isolated `wide_state_probe.rs` evaluates the entire nonlinear chain,
including compressor split filters and finite tail, with identical coefficient
strings. Only `BiquadState` feedback arithmetic/storage widens to f64; every
filter still returns f32, and the original denormal floor is retained exactly.
Original, -40 and -80 dB copies use the same source facts/settings/drive. Each
narrow original reproduces the previously retained C1 WAV exactly before the
fresh whole-file finalizer check. The production DSP was not modified during
these comparisons.

Five complete candidates on four source files (`c1-wide-*-v1/`) produce:

| Candidate | Maximum delivered copy difference, narrow | Wide | Median chain time narrow / wide |
| --- | ---: | ---: | ---: |
| Piano Universal-75, -14, offset -9 | 0.000379920 | 0.000000536 | 1.752 / 2.096 s |
| Piano Universal-75, -14, single | 0.000412028 | 0.000000477 | 2.034 / 2.506 s |
| Metal Universal-75, -14, single | 0.000317380 | 0.000000477 | 1.553 / 1.998 s |
| Coat Universal-75, -9, single | 0.000642583 | 0.000000596 | 2.360 / 2.813 s |
| Imaginal Universal-75, -14, single | 0.000555597 | 0.000000358 | 3.961 / 4.802 s |

All ten wide copy comparisons have delivered LUFS drift at most **0.00000191
LU**. All 30 original/copy/narrow/wide deliveries complete qualified whole-file
protection. Original versus corrected audio is intentionally different: this
corrects numerical sensitivity, not the preset coefficients. It does not solve
the separately reproduced selector-reference dependence.

Times are medians of three alternating AB/BA whole-song trials, excluding input
copy, SRC and finalization. Both implementations use opt-level 3. **19–29% more
chain CPU** is the measured cost; these trials run under concurrent research
load and do not establish an isolated session-time increase. Finalization and
later cached-target verification remain separate costs. Original/-80 corrected
files also receive independent complete-file SOXR16/64/LUFS checks, with the
versioned reports retained alongside the native results.

### Production regression and native cost

`tests/filter_precision.rs` generates deterministic low-frequency/noise material
at 44.1/48/96 kHz, mono/stereo, and compares normalized -40/-80 dB copies. Input
differences stay below 2e-7. The original chain fails **all 12** cases, reaching
**0.00473543** maximum output difference (`c1-filter-precision-before-v1.log`).
The same test after the arithmetic correction passes at unchanged **1e-5**
tolerance; maximum difference is **0.000000567**. This is a numerical regression,
not an audibility threshold. Production now uses that corrected state/arithmetic.

The existing synthetic preset snapshots detect the deliberate arithmetic change.
All nine old references are preserved in
`src-tauri/tests/golden/preset_byte_identity/f32-feedback-20260915/` and Git.
Custom is byte-identical; the eight factory references change by at most
0.000246522 on the decimated probe. Regenerated references retain the **1e-6**
tolerance. `c1-filter-golden-deltas-v1.json` records both hashes and exact deltas.
No preset calibration, user value or gated constant changes.

`c1-filter-callback-v2.json` exercises the production **44.1 → 96 → 48 kHz**
route with 120 edits and concurrent preparation: **607 callbacks, zero deadline
misses, device errors, converter errors or exhausted samples**. Median/p95/max
are **0.0491/1.16216/1.80730 ms**. Construction is 0.80 ms; first observed peak
10.21 ms; cancellation joins in 2.91 ms. Requested 256 frames grants 480–1,056,
so this is not a fixed-256 pass. No improved callback speed is claimed from
the difference between two loaded runs. The first attempt completed processing
but could not save to a relative report path; that failed log remains and v2
uses an absolute path.

Production validation passes: `c1-filter-fixtures-v3.log` includes **492 library
tests, every integration suite and all four restored private-fixture tests**.
The unchanged preset fingerprint safety/distinctness/tolerance checks pass.
Strict all-target Clippy v2, iPhone check/46 tests v2, Android 26 tests/API-29
check v2, rebuilt WASM stamp **87f21d44e1a0**, 897 frontend tests and the frontend
build pass. No rendered markup changed; the preceding 38 headless checks retain
their UI scope. The full-suite v1/v2 failures preserve the old snapshots and
short PCM16 receipt issue; neither assertion/tolerance was relaxed.

`c1-filter-lifecycle-v1.json` confirms actual muted playback with the corrected
chain: cold acceptance/first meter **308/373 ms**, playing and paused rate edits,
59.7 ms paused-seek/resume and 9.0–16.4 ms A/B swaps preserve position. First
96 kHz landing preparation settles after **39.74 s**, a new target after
**0.931 s**, and returning to the cached target within **75.24 ms**. Raw occupancy
remains **187,875,824 bytes**; new-target whole-file verification is 0.815 s.
The full probe takes 41.65 s. These loaded lifecycle observations separate
preparation, settings settling and playback; they are not a controlled speedup
over the earlier narrow run or a timestamp of the last output crossfade sample.
No new installed/Mac/listening verdict or automatic sonic policy is inferred.

### Replay provenance

`prepare_wide_state_probe.py --output FRESH` creates an ignored standalone
project. It reads pre-correction DSP from **local `3fecbbe9`** without checking
out another branch, writes narrow and wide copies, and copies the production
dependency lock. It never modifies the frozen sibling or production DSP.
Build with `cargo build --offline --manifest-path FRESH/Cargo.toml --target-dir
src-tauri/target/codex-rc`; run with original C1 report, candidate id and a fresh
output directory. Source/report/WAV hashes and coefficient identity are asserted.

Retained v3 experiments use original DSP SHA-256
`24a7c6d1730cc4a2c96c41bc120c0841a4d058d4e53c74bd1a102f5642b0f461`.
All 595 registry dependencies are verified against the production lock.
Build v1 was stopped after discovering an unfrozen dependency resolution; v2
retains missing-import/type errors. v3 builds and runs with equal optimization;
v4 sets the product MSRV for strict Clippy. Later replay materializes the narrow
module explicitly so production's correction cannot silently change the control.
The v5 pinned replay reproduces all six Piano original/-40/-80 narrow/wide WAV
hashes, gain/loudness/peak results and arithmetic differences **exactly**, after
the production correction (`c1-wide-piano-replay-v5/replay-verification.json`).
Its strict Clippy/build logs pass. This is replay integrity, not another
independent reference or a new timing comparison.

## Independent loudness availability

The first -80 dB independent checks stopped on otherwise valid quiet controls:
native LUFS was unavailable while FFmpeg printed its initial **-70 LUFS / zero
threshold** pair. [FFmpeg 9.0's primary implementation](https://raw.githubusercontent.com/FFmpeg/FFmpeg/n9.0/libavfilter/f_ebur128.c)
initializes that display and updates the integrated calculation only after a
block enters the absolute gate. The checker now records availability separately;
empty/empty has **no numeric LUFS comparison**, while any availability disagreement
fails. A real result rounded to -70 with an updated threshold still receives the
unchanged 0.11 LU comparison. Peak/full-scale requirements are unchanged.

Seven Python regressions pass, including missing summary, mismatched availability
and rounded -70 cases. Actual FFmpeg probes verify silence, below-gate and
above-gate inputs (`c1-independent-gating-native-v1.json`). Earlier Piano/Metal
-80 failed reports remain; fresh v2 reference checks resume the completed audio
without rerendering it. This does not turn unavailable target or character data
into a feasible candidate.

## Normalized selector reference: first seven-source result

The [separately frozen development experiment](2026-09-15-drive-selector-reference-experiment.md)
re-scores completed metrics against each input copy's own single normalized
candidate for tone/stereo allowance. It leaves dynamics limits, slack, floor,
search/tie-break and corrected-control fallback unchanged. No audio is rendered
or remeasured; the old f32 numeric failures and original analysis anchors remain.
The eight-source specification/manifest precede evaluation. Imaginal's remaining
metrics were unavailable at this first checkpoint, so the seven-source result
is explicitly separate (`c1-normalized-reference-seven-v1.json`).

| Source | Old preserving consistency | Normalized-reference consistency |
| --- | ---: | ---: |
| Coat | 8/12 | 12/12 |
| Piano | 12/12 | 12/12 |
| Aphelion | 12/12 | 12/12 |
| Funk | 8/12 | 12/12 |
| Metal | 0/12 | 12/12 |
| Baby | 10/12 | 12/12 |
| Rich | 12/12 | 12/12 |
| Total | **62/84** | **84/84** |

These compare candidate identity/fallback at -40/-80 dB across three preset/
intensity groups and two targets. Target-first stays 84/84 in both rules. The
new reference changes 13 of 42 original preserving outcomes: fallbacks increase
**16 to 17**, and target hits (including fallbacks) decrease **20 to 18**. Fifteen
new-rule selections have explicit character failures (fallbacks), versus thirteen
before. Thus input-level consistency improves without establishing a universally
better master or a successful candidate for every preset. For example, Metal
Universal-75 at -9 selects a qualified candidate landing about 2.164 LU below
target, while Baby Universal-75 at -9 falls back about 7.124 LU below target.
These misses are preserved rather than converted into passing quality claims.

Recommendation: carry the normalized reference into corrected-chain development
because it resolves the isolated reference dependence; retain both target-first
and preserving comparisons. Resolve its fallback/character limitations, actual
source-analysis behavior and measured budget before freezing any holdout rule.
No production selector, preset calibration or owner preference is adopted.

### Completed eight-source selector result

`c1-normalized-reference-v1.json` completes the same frozen rule on all eight
sources and both quiet copies. Preserving candidate/fallback identity improves
from **73/96 to 96/96**; target-first stays **96/96**. Across 48 original groups,
fallbacks increase **18 to 21**, target hits including fallbacks decrease **20 to
18**, and selected character failures increase **15 to 19**. Those failures are
explicit fallback outcomes, not feasible preserving candidates. Imaginal adds
large preserved misses: Universal-75's -9 fallback is **11.452 LU** below target.
The full result therefore confirms reference consistency but does not close C1's
character/fallback limits or authorize C2/C3 adoption.

All **960 quiet-copy renders** and their complete metrics/consistency comparisons
are now retained. All 16 original f32 copy runs fail the declared normalized-PCM
numeric gate; the independently verified f64 production correction is separate
and does not retroactively relabel these old files. The final independent queue is complete: **243/243 selected/control files**
pass peak and LUFS/availability checks, with zero full-scale samples. Forty-eight
below-gate controls correctly have no numeric LUFS comparison. The aggregate
`c1-all-quiet-independent-summary-v1.json` verifies all 16 completed report hashes
and each declared selected/control set; earlier failed reports remain intact. Fresh production source-analysis and normalized-
descriptor comparisons proceed under a separate specification; these earlier
copy outcomes retain the original analysis/guards by design.

The [actual production-analysis diagnostic](2026-09-15-source-analysis-gain-results.md)
is now complete: 48 analyses of actual/normalized source copies. Actual-level
analysis changes 15/48 resolved coefficient sets; normalized analysis preserves
48/48 copy comparisons and 24/24 original-versus-normalized coefficient sets.
This supports a separate reusable normalized source descriptor for the next
corrected-chain experiment, while actual-source display/receipts remain intact.
It does not resolve the selector's remaining character/fallback limitations.
