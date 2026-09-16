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
