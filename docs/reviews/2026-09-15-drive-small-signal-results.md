# C1 low-drive response: completed diagnostic

The [frozen diagnostic](2026-09-15-drive-small-signal-protocol.md) retains all
coefficients and compares additional 0/-24/-48 dB drive, compensated after the
chain. This is outside the frozen candidate grid and is **not a new selector**.
No preset, source-analysis policy, character limit or fallback has changed.

## What the checks establish

- All **four zero-offset controls reproduce the retained E production PCM
  exactly**. Their existing WAVs and independent results are reused by exact
  SHA-256, format, frame count, ceiling and native-LUFS identity.
- All **eight new complete float outputs** pass fresh independent SOXR16/64 peak
  and LUFS checks, with exact independently calculated frame counts and zero
  full-scale samples. Native qualified readings are at or below the -1 request.
  These are protected outputs with explicit target misses, not target successes.
- All eight low-drive renders report zero limiter reduction and less than
  0.01 dB compressor reduction (the existing compressor maximum telemetry is
  quantized down to 0.01 dB). Other nonlinear response cannot be assumed absent.
- **None of the four low-drive pairs passes the predeclared 1e-5 numerical
  convergence limit.** Maximum compensated source-rate differences between
  -24/-48 are 0.002587 (Coat single), 0.007328 (Piano), 0.070823 (Imaginal) and
  0.010455 (Coat current). Do not call these a proven linear reference, or enlarge
  the tolerance after seeing them. The remaining level-dependent/numerical
  contribution is not isolated by this experiment.

## Dynamics, tone and delivery remain separate

The table shows existing zero-offset processing and the -48 diagnostic; the
retained -24 results approach similar metrics but fail waveform convergence.
Section contrast and attack values are changes from the same original source,
using its frozen section and paired-attack anchors. Attack columns are median
and lower-decile change; larger values alone do not establish a preferred master.

| Case | Delivered LUFS, current / -48 | Section contrast change, current / -48 | Paired attack median, current / -48 | Paired attack p10, current / -48 |
| --- | ---: | ---: | ---: | ---: |
| Coat single, target -14 | -14.000 / -14.332 | -0.184 / -0.019 dB | +0.282 / +0.881 dB | -0.880 / -0.469 dB |
| Piano single, target -14 | -14.000 / -18.700 | -1.586 / +0.101 dB | -1.145 / +0.183 dB | -2.208 / -0.795 dB |
| Imaginal single, target -14 | -14.000 / -25.575 | -0.491 / +0.146 dB | -3.842 / +1.350 dB | -6.311 / -0.383 dB |
| Coat current, target -9 | -9.000 / -14.332 | -1.531 / -0.019 dB | -1.706 / +0.881 dB | -2.704 / -0.469 dB |

The low-drive Piano and Coat responses clear their earlier descriptive dynamics
failures without changing their requested preset coefficients. Those failures
cannot simply be dismissed as unavoidable static preset response. Imaginal's
attack failures also disappear, but its two highest measured bands now fail the
existing tone allowance against the matched single candidate. Maximum matched
band change is **2.0143 dB**. Its much larger target miss is visible above.

These are diagnostic comparisons, not a replacement score for the original
development matrix. The tone/stereo reference here is the matched E current case;
the original source facts, individual deltas and every failure are retained.

## Recommendation and limits

Keep the current C1 character failures and explicit target misses visible. Do
not weaken the dynamics limits on the assumption that preset EQ made them
unavoidable. Investigate a separately frozen lower-drive candidate/fallback
iteration, with a finite budget and tone/preset intent preserved. The original
grid, normalized-reference diagnostic and this response experiment remain
separate evidence. A direct linearized-stage reference or further convergence
diagnosis would need its own declared scope before being used to change scoring.

The results support giving character preservation priority with an honest target
miss, rather than guaranteeing every requested loudness. They do not choose the
owner's unresolved intentional sonic tradeoff, prove an audible improvement,
or qualify production C3. Candidate limits, corrected-chain selection, runtime
budgets and a rule frozen before genuinely new holdout material remain open.

Strict example Clippy/build pass. Evidence root is
`test-output/mastering-quality-implementation-20260915/`:
`c1-small-signal-job-v1.json`, `c1-small-signal-v1/report.json`,
`c1-small-signal-independent-v1/comparison.json` and
`c1-small-signal-metrics-v1.json`. The combined independent report explicitly
marks **four reused controls and eight fresh checks**. Job/protocol/report/output
hashes and original anchors are retained. Eight new WAVs use about 801 MB; existing
research is untouched and the job reserves 25 GiB after planned output.
Copied diagnostic executable SHA-256:
`8e4101d2ce1b8ef061e175cf11110ddf32496a57b53cec9cc2dcf8301d471f23`.
