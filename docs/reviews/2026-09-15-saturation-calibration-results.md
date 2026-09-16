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
