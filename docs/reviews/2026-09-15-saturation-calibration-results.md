# Saturation calibration: development results

The [first frozen protocol](2026-09-15-saturation-calibration-protocol.md) ran
**1,296 isolated comparisons**. Evidence is
`test-output/mastering-quality-implementation-20260915/e-isolated-v1/`;
it preserves the exact script/protocol, FIR bytes/hashes and complete report.
No production curve, preset or gated feature changed.

## Continuity and antialiasing are different findings

At amount 1e-8, the positive control's small-signal gain is **+2.365528 dB**.
Both smoothstep candidates approach identity continuously: the 0.005 knee gives
about **3.26e-11 dB** and the 0.02 knee **2.04e-12 dB**. Both recover the positive
curve at/above their knee. Which ramp preserves the intended control response
still needs whole-chain calibration; continuity alone does not choose it.

For the 108 common amount/rate/frequency/amplitude cases:

| Method | Worst folded nonharmonic level, dBc | Median offline processing per 32,768 samples |
| --- | ---: | ---: |
| Current positive curve | -16.46 | 1.15 ms |
| 2x, 65-tap filter | -31.03 | 6.10 ms |
| 4x, 129-tap filter | -61.39 | 13.08 ms |
| 8x, 257-tap filter | -70.93 | 24.20 ms |

These are float64 Python/SciPy timings under concurrent work, **not native
callback cost**. All three kernels were designed once in 26.2 ms and reused.
Their causal up/down delay would be 32 source frames; this periodic isolated
experiment removes that delay and cannot certify a streaming implementation.
The independently constructed analytic reference converged within 2.23e-16
for retained harmonic coefficients.

The short filter is **not an acceptable fidelity result**: fundamental loss
reaches **3.398 dB**. For the 19.002 kHz, 44.1 kHz, amplitude-1.4, amount-0.0715
case, 4x improves alias energy from -17.51 to -71.17 dBc while attenuating the
fundamental **2.462 dB**. Gain normalization must not conceal that loss. Increasing
only the factor leaves this passband problem essentially unchanged.

## Frozen second filter iteration

Investigate a better filter before paying for whole-song processing. Retain the
same positive curve, rates, coherent frequencies, amplitudes, three positive
amounts, analytic reference and separate gain/harmonic/alias reporting. Compare
**128 × factor + 1 taps / cutoff 0.96** and **512 × factor + 1 taps / cutoff 0.99**,
Kaiser beta 10, at 2x/4x/8x. These are two new development alternatives, not a
retroactive change to the first report. Keep factor-1 controls; do not rerun
the unchanged continuity grid. Each kernel is reused for all matching cases.

Provisional screening limits, declared before these results: fundamental loss
at the tested frequencies **no more than 0.1 dB**; folded energy **at most -60 dBc**
and at least **20 dB reduction** where the control's folded energy exceeds
-80 dBc. Report each failed case, in-band harmonic changes, full matched-waveform
error, latency and processing cost. These are mechanism-screening limits, not
audibility thresholds or an owner voicing decision. No factor is selected solely
by its label. Whole-chain/final-output/native/C-policy/listening requirements in
the first protocol remain open.

## Second iteration results

Both 432-case runs completed; exact pre-result protocol/script copies live with
`e-isolated-filter128-v2` and `e-isolated-filter512-v2`. The frozen-limit assessment
is `e-filter-screening-v2.json`. All reference-convergence checks pass.

| Filter taps / cutoff | Factor | Passes / 108 | Worst alias, dBc | Worst fundamental loss | Median offline processing |
| --- | ---: | ---: | ---: | ---: | ---: |
| 128 × factor + 1 / 0.96 | 2 | 93 | -30.83 | 0.000026 dB | 8.00 ms |
| same | 4 | 106 | -58.56 | 0.000039 dB | 16.59 ms |
| same | 8 | 108 | -119.47 | 0.000046 dB | 38.01 ms |
| 512 × factor + 1 / 0.99 | 2 | 93 | -30.83 | 0.0000013 dB | 35.54 ms |
| same | 4 | 106 | -58.56 | 0.0000019 dB | 76.16 ms |
| same | 8 | 108 | -126.34 | 0.0000032 dB | 149.99 ms |

Both 8x alternatives pass the declared fundamental/alias screen. However, the
shorter one's worst gain-matched error is still **-26.54 dBc**: at 7.002 kHz /
44.1 kHz / amplitude 1.4 / amount 0.13, its legitimate third harmonic is near
the transition band. That distortion is separate from folded energy and is
not waived by the screening pass. The longer 8x candidate's worst full matched
error is **-125.31 dBc** for this tested matrix. Its symmetric causal up/down
filter adds **512 source frames** (11.61 ms at 44.1 kHz), versus 128 frames
(2.90 ms) for the shorter design. Neither timing is a measured native callback.

Recommendation for whole-chain experimentation: retain the positive curve and
carry the longer 8x filter as a fidelity reference, with the shorter design as
the cost comparator. Investigate a more efficient staged filter only against
that reference and preserve its passband/harmonic response. Both kernel tables
are reusable; longer per-sample processing does not disappear because coefficient
design happens once. These results do not authorize a preset/curve migration,
and they do not establish whole-chain or audible improvement.

## Native finite-filter and cost prototype

The [native protocol](2026-09-15-saturation-native-protocol.md) reuses both exact
8x coefficient tables. It implements causal f64 polyphase interpolation, the
existing isolated positive f64 curve, and FIR decimation with bounded mirrored
rings. Tables and the constant curve denominator are prepared once. Four finite
stereo inputs include unequal-channel start/end impulses, a DC step, tone and
high-frequency end burst, retaining above-full-scale input and complete ringout.

All **8/8** complete native outputs match independent SciPy `upfirdn` traversal
within **8.89e-16 maximum absolute error**, against the predeclared **1e-10** limit.
Partitions of 1, 257 and 1,024 frames yield identical native samples. Output
length is exactly N+2L, with combined filter delay L; no edge is cropped to pass.
Evidence: `e-native-inputs-v1/`, `e-native-v1/` and `e-native-comparison-v1.json`.

| Native prototype, stereo at 44.1 kHz | Short 8x / 1,025 taps | Long 8x / 4,097 taps |
| --- | ---: | ---: |
| Phase-table preparation | 0.0058 ms | 0.0252 ms |
| Median processing of 32,768 frames plus complete tail | 46.78 ms | 209.54 ms |
| Per-case median 256-frame processing | 0.340-0.382 ms | 1.536-1.584 ms |
| Largest observed 256-frame processing | 0.942 ms | 11.367 ms |
| Blocks exceeding the 5.805 ms audio interval | 0/1,548 | 3/1,584 |
| Combined filter delay | 2.902 ms | 11.610 ms |
| Retained kernel / stereo state bytes | 16,456 / 36,928 | 65,608 / 147,520 |

Processing values summarize three fresh-state traversals per signal. Timers
include collection/observation overhead and possible scheduling effects; these
are elapsed native prototype measurements, not CPAL callbacks. The long filter's
three interval exceedances remain a performance concern with cause unresolved.
Neither a fixed-block native-device pass nor production f32 behavior is claimed.
The short filter's previously measured legitimate-harmonic loss is not waived
by its lower cost. Upfront table preparation is negligible; its reuse cannot
remove the ongoing per-sample convolution cost.

Strict example Clippy and build pass. Copied native executable SHA-256:
`ee9dcb279a3d5458bc604f6de8049adea16873355521cc5411313208dafcbfba`.
The native report binds its job hash; the independent comparison verifies every
kernel/input/output hash. The existing analytic frequency/alias comparison
remains separate from this finite-convolution traversal. Next, compare matched
whole-chain outputs and limiter redistribution under the existing qualified
finalizer, keeping the long filter as a fidelity reference. Actual callback,
selected-C and owner calibration/listening gates remain open.

## Native accumulation experiment

The [predeclared accumulation experiment](2026-09-15-saturation-native-optimization-protocol.md)
changes only the f64 dot-product addition order. The original serial implementation
is preserved at `7e4a7327` and in `e-native-v1.exe`; the four-accumulator version
is preserved at `a5d4ad56`. Neither is production saturation.

All eight alternate outputs pass the unchanged independent **1e-10** limit;
maximum error is **1.089e-14**. All **48/48** repeated output hashes match their
respective qualified implementation, and each run retains chunk invariance.
`e-native-opt-cost-v1.json` verifies the job, inputs, outputs and reference reports
before aggregating three alternating pairs. Other whole-chain research was running.

| Filter | Serial median complete processing | Four accumulators | Median paired change | Observed maximum block, serial / alternate |
| --- | ---: | ---: | ---: | ---: |
| Short 8x | 47.23 ms | 52.48 ms | **11.10% slower** | 0.851 / 1.047 ms |
| Long 8x | 210.89 ms | 193.67 ms | **8.12% faster** | 3.563 / 3.274 ms |

These are 32,768 stereo frames plus full ringout. The long filter improves in
all 12 matched signal/pair comparisons (3.80-10.64%); the short one regresses in
all 12 (3.58-15.23%). Kernel/state sizes and latency are unchanged. Median table
preparation stays below 0.03 ms and state construction below 0.043 ms. None of
the 4,644 short or 4,752 long blocks per implementation exceeds its nominal
5.805 ms interval in these subsequent trials. That does **not** invalidate the
three earlier exceedances, establish their cause, or certify actual callbacks.

Recommendation: retain this as a measured long-filter optimization candidate;
do not apply it indiscriminately to the shorter filter. The modest long-filter
benefit does not remove its ongoing convolution cost or establish a production
budget. Strict example Clippy/build pass. Alternate executable SHA-256:
`5fd194f24f7c57312dc9ee0b04e3a6150b2523b5626cdaa8f1b0082dfc889ec0`.
The whole-chain comparison below remains pinned to the **serial** implementation;
its timings must not be credited to this alternative.

## Whole-chain development comparison

The [frozen whole-chain protocol](2026-09-15-saturation-whole-chain-protocol.md)
uses the existing C1 single candidate on Coat/Piano/Imaginal at -14 and actual
source-level current processing on Coat at -9. Each compares current f32 curve,
f64 evaluation of that same curve, short 8x and long 8x. Production coefficients,
settings and source/drive identities are asserted. The isolated source copy is
pinned to `7e4a7327`; no production curve, preset, continuity knee or C selection
is changed.

All **four current controls reproduce fresh unmodified production PCM exactly**.
F64-only raw differences are at most **3.577e-7**, passing the declared 1e-5
isolation limit. All **16 complete 48 kHz float files** have independently correct
frame counts and pass full-file SOXR16/64 peak and LUFS checks, with zero
full-scale samples. All meet their respective -14/-9 target; the highest native
qualified peak is -1.92428 dBTP against -1. Exact file hashes bind the native,
independent and source-anchored metric reports in `e-whole-summary-v1.json`.

The comparison flushes combined FIR/limiter latency and retains the original
N-frame aligned interval. This verifies the fixed-duration export contract;
complete exterior FIR ringout is covered separately by the finite native probe.
Whole songs do not acquire extra duration. The source/matched metrics use the
original frozen C1 section and attack anchors.

### Quality indicators remain separate

- Piano's section-contrast failure, Imaginal's paired-attack failures and Coat's
  -9 section-contrast failure remain under the proposed C1 limits with **all four
  variants**. Coat's -14 single candidate passes those descriptive limits. The
  filters do not resolve the C1 character-policy problem.
- Imaginal's maximum limiter reduction changes from **4.89892 dB** to **5.02631 /
  4.91372 dB** with short/long filters; active fraction changes from **19.5334%**
  to **19.5734 / 19.5598%**. Its delivered peak changes from -1.92428 to
  **-2.50161 / -2.37714 dBTP** at the same target. Limiter activity is redistributed,
  not uniformly reduced.
- Relative to matched current processing, the largest whole-song tonal-band
  change is **0.03132 dB**, side/mid change **0.00239 dB**, correlation change
  **0.000155**, and fixed-section contrast change **0.00624 dB**. Small aggregate
  changes do not prove sample equivalence or inaudibility.
- The largest raw waveform changes are Imaginal's **0.32514 / 0.28812** for
  short/long filters. Independent gain-removed delivery scans locate both at
  **139.4446875 s**, away from file boundaries. Error RMS relative to current is
  **-44.61 / -45.13 dB** there across the full song; Piano's is -104.04 / -109.05 dB.
  These are changes from the current chain, not error versus an ideal whole-chain
  reference. The isolated analytic alias tests cannot attribute every musical
  waveform difference solely to removed aliasing or establish audible preference.

### Preparation and recurring processing cost

Both kernel files are read and phase tables prepared **once in 0.317 ms**, then
reused across the complete comparison. The serial-filter whole-chain times below
include limiter observation; they are single loaded traversals, not isolated CPU
benchmarks or actual audio callback durations.

| Case | Current chain | Short 8x chain | Long 8x chain | SRC range | Final verification/landing range |
| --- | ---: | ---: | ---: | ---: | ---: |
| Coat single, -14 | 2.884 s | 22.233 s | 93.191 s | 0.207-0.281 s | 12.066-16.249 s |
| Piano single, -14 | 2.268 s | 17.954 s | 68.118 s | 0.168-0.193 s | 9.951-10.456 s |
| Imaginal single, -14 | 4.262 s | 34.219 s | 148.475 s | 0.031-0.051 s | 17.339-20.377 s |
| Coat current, -9 | 3.571 s | 23.047 s | 87.418 s | 0.208-0.366 s | 10.803-14.107 s |

Ranges include all four variants, including the f64-only control. File writing,
independent references and metric scans are separate research cost; these sums
are not a measured user's total session. Table reuse saves almost no time relative
to per-sample filtering. A cache may reuse a matching rendered result, but cannot
remove nonlinear work after a relevant settings change.

**Recommendation:** keep the long filter as a fidelity reference and evaluate a
more efficient implementation against its full response before production use.
The shorter filter's known legitimate-harmonic loss remains a rejection reason
despite its lower cost. This run proves finite whole-chain integration and final
output compliance on four development cases; it does not justify an unconditional
filter migration. Broader rates/sources, actual callback and session budgets,
the selected C policy, continuity-knee calibration and targeted owner listening
remain open. Current intended preset voicing and gated constants remain unchanged.

Evidence: `e-whole-job-v1.json`, `e-whole-source-v1/provenance.json`,
`e-whole-v1/report.json`, `e-whole-independent-v1/comparison.json`,
`e-whole-metrics-v1.json` and `e-whole-summary-v1.json`. Standalone Clippy/build
pass. Copied whole-chain executable SHA-256:
`f18e67324999681e7b41fceb733967515afa1a4bf6aa236ea47ef0ef8a0a4306`.

## Vectorized native convolution

The [separately frozen vector experiment](2026-09-15-saturation-vector-protocol.md)
uses explicit AVX four-f64 multiply/add without FMA, selected once during kernel
preparation. Other processors use the original serial fallback. This run uses
an **Intel Core i9-12900K on Windows**; it does not establish ARM or other CPU cost.

All **8/8** independently checked vector outputs pass the unchanged 1e-10 limit,
with maximum error **1.089e-14**. All eight hashes exactly match the preceding
four-accumulator output, confirming the preserved addition order for this matrix.
All **48/48** alternating-trial output hashes match their implementation anchors.
Forcing the serial fallback on the same host also reproduces **8/8** original
serial outputs exactly. The report identifies `avx-four-f64` versus `serial-f64`;
forcing the fallback is not a different-platform test.

| Filter | Serial median processing | Vector median processing | Median paired reduction | Serial / vector block interval exceedances |
| --- | ---: | ---: | ---: | ---: |
| Short 8x | 46.820 ms | 38.840 ms | **16.61%** | 0/4,644 / 0/4,644 |
| Long 8x | 211.058 ms | 143.203 ms | **31.79%** | 13/4,752 / 0/4,752 |

Values cover the same 32,768 stereo frames plus complete tail. Long-filter block
median/p95/max is **1.0656/1.2781/3.1474 ms** with vectors, versus
**1.5710/1.8920/9.7891 ms** serial. The short vector maximum is 1.2576 ms versus
1.0243 ms serial despite lower typical cost. Timing outliers and the thirteen
new serial interval exceedances remain recorded; their scheduling/CPU cause is
unresolved. These prototype timers do not certify actual callback deadlines.

Vector table/function preparation takes a median **0.0150/0.0261 ms** for
short/long; state construction **0.0058/0.0427 ms**. Exact coefficient and ring
buffer sizes, curve and filter delay are unchanged; small dispatch metadata is
additional. Preparation remains negligible beside per-sample processing.

Recommendation: carry this runtime-selected vector implementation into subsequent
whole-chain/native-device experiments with the long fidelity reference. It gives
a demonstrated throughput benefit on this host while preserving the tested
numerical response. It does not erase the short filter's harmonic-loss finding,
prove the earlier whole-song timings improved by exactly these percentages, or
authorize production saturation/preset adoption. The earlier whole-song comparison
remains pinned to the serial implementation.

Evidence: `e-native-avx-job-v1.json`, the six `e-native-avx-{base-pN,pN}/` runs,
`e-native-avx-fallback-v1/`, `e-native-avx-comparison-v1.json` and
`e-native-avx-cost-v1.json`. Strict example Clippy/build pass. Copied executable:
`de155f55574122c0b4a7da2cb4c153fd48b97c00c3a78be04cc374afcbf757d8`.
