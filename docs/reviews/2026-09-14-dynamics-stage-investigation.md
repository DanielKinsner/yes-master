# Dynamics investigation: reproduced owner export and isolated processing stages

Date: September 14, 2026. Scope: private, local diagnostic research following the
owner's Density/Adapt questions. No production DSP, preset calibration, gates or
UI changed. This follows the [owner export comparison](2026-09-14-yes-export-vs-landr.md).

For another machine, use the [research transfer guide](2026-09-14-research-handoff.md).
The [compact render measurements](evidence/2026-09-14-dynamics.csv) travel with Git;
the complete audio/evidence archive is stored separately on GitHub with owner authorization.

**Owner follow-up:** A–E were reported too subtle for a useful naked-ear choice.
No preference/equivalence verdict follows. [The next investigation brief](../prompts/2026-09-14-mastering-quality-continuation.md)
requests broader suitable test material, objective mechanism/robustness evidence
and a ranked recommendation, rather than another identical listening exercise.

## Result

Both harness baselines reproduce the owner's Compressor On and Off WAVs **byte
for byte**. On this source and Universal 75 configuration, bypassing saturation
or limiting changes peak contrast substantially more than switching the preset
compressor off. Adapt strength is working as a compressor-easing control; it
does not cover all the processing responsible for the resulting density.

This establishes a useful investigation target, not that saturation/limiting is
bad or that either should be removed. Higher crest factor is not a quality score.

## Reproduction and method

The isolated Rust harness includes production `dsp.rs`, analysis, types, sample
rate conversion and WAV writer directly by path. Private copies of guardrails
and confidence disable only six desktop IPC wrappers and a related import using
their existing cfg boundaries; their algorithms and gated constants are unchanged.
The first attempt to compile the browser crate natively failed because those IPC
wrappers require desktop dependencies. That packaging failure was not an audio
failure; the standalone harness avoids the wrappers.

Settings: Universal, Intensity 0.75, default requested density 0.50, Adapt 0.50,
Custom target -9 LUFS, default -1 dBTP ceiling, zero manual EQ/input/output trims,
48 kHz PCM24. Original is stereo 44.1 kHz PCM16. Analysis uses the entire track.
The chain runs at source rate in 4096-frame chunks, flushes its lookahead tail,
then uses production SRC, ceiling-bounded scalar landing and deterministic PCM24
dither. Output is 11,738,879 stereo frames, matching the owner's files.

SHA-256 equality of complete WAV files:

| Owner file / diagnostic baseline | SHA-256 |
| --- | --- |
| `YES-compressor-on.wav` / `on_baseline.wav` | `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce` |
| `YES-compressor-off.wav` / `off_baseline.wav` | `d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5` |

This proves output equivalence for these fixtures, without claiming to have
recovered the owner's original application build metadata. Recorded checkout
at harness preparation: `63e557a01714781c4528547aad9329bb49139584`. Production
module hashes were rechecked after all renders; the shared checkout's unrelated
work was preserved.

Saturation bypass sets that coefficient to zero only inside the test process.
Limiter bypass raises its threshold to +60 dBFS while retaining its delay and
flush; all diagnostic samples remain far below that threshold. Delivery still
uses a scalar ceiling bound. No bypass is added to the application.

## Stage comparisons

Eight full-track renders cover all on/off combinations of compressor, saturation
and limiter. FFmpeg independently measured delivered loudness/LRA/true peak;
scipy decoded PCM and calculated peak-to-RMS crest. Independent crest and native
float crest agree within 0.00001 dB. Delivered LUFS agrees within 0.06 LU.

| Processing | LUFS | LRA (LU) | Crest (dB) |
| --- | ---: | ---: | ---: |
| Current YES, all stages | -9.0 | 3.1 | 8.95 |
| Compressor off | -9.0 | 3.6 | 9.42 |
| Saturation bypassed, compressor retained | -9.6 | 3.4 | 11.14 |
| Limiter bypassed, compressor retained | -9.2 | 3.7 | 10.74 |
| Saturation and limiter bypassed, compressor retained | -14.3 | 4.1 | 15.74 |
| Compressor off, saturation bypassed | -10.1 | 4.0 | 11.63 |
| Compressor off, limiter bypassed | -9.3 | 3.8 | 10.92 |
| All three bypassed | -12.6 | 4.1 | 14.19 |

Crest describes peak contrast and is unchanged by a uniform playback-volume
adjustment. However, these are **not all equally loud delivered masters**:
some bypass versions cannot reach -9 LUFS while retaining the -1 dBTP ceiling.
They remain quieter instead of adding another limiter. The listening clips below
remove the loudness advantage using gain alone.

Relative to current YES, compressor bypass adds 0.47 dB crest; saturation bypass
adds 2.19 dB; limiter bypass adds 1.79 dB. These are controlled chain interventions,
not additive percentages of causation. Saturation bypass also changes what feeds
the limiter; compressor bypass removes its crossover and makeup as well as gain
reduction. The high crest with both nonlinear stages bypassed is not proof of
better transients: EQ, crossover phase, width and the transient shaper can alter
the peaks relative to the source.

The current chain meters approximately -7.37 LUFS before applying -1.63 dB of
final attenuation to reach -9.0. That does not by itself establish a defect, but
it motivates testing whether less input drive can meet the desired loudness with
less peak reduction. Turning down the final output cannot recover earlier peaks.

## Density and adaptation experiments

The native source profile is DR P95-P10 **7.196 dB on the engine's 100 ms blocks**,
LRA **4.063 LU**, correlation **0.732**, width **0.134**. The earlier independent
400 ms dynamics statistic is a different measure and must not be substituted
into the engine's profile as though it were the same input.

Nine density/adaptation combinations were evaluated: requested density 25/50/75%
and Adapt 0/50/100%. The existing baseline supplies the 50/50 cell, avoiding a
repeat render. All nine reach -9 LUFS on this song.

At requested density 50%:

| Adapt | Resolved threshold | Ratio | Maximum low/mid/high gain reduction | Crest |
| --- | ---: | ---: | --- | ---: |
| 0% | -12.50 dB | 1.450 | 3.05 / 1.63 / 0.35 dB | 8.55 dB |
| 50% | -8.46 dB | 1.305 | 1.25 / 0.34 / 0.00 dB | 8.95 dB |
| 100% | -5.00 dB | 1.180 | 0.30 / 0.00 / 0.00 dB | 9.28 dB |

Gain-reduction snapshots are per-band maxima across the track, at the existing
meter's 0.01 dB resolution, not averages or a total reduction across bands.
At default Adapt 50%, increasing requested density 25 -> 50 -> 75% changes crest
9.34 -> 8.95 -> 8.66 dB and LRA approximately 3.40 -> 3.14 -> 2.73 LU.
These controls have real, distinguishable effects. Neither restores the peak
contrast suppressed by the rest of this chain. This is one source/preset test,
not validation of every slider value or source type.

Two additional tests kept all stages, density and Adapt at baseline values but
changed input trim:

| Input trim | Delivered LUFS | LRA | Crest | True peak |
| --- | ---: | ---: | ---: | ---: |
| 0 dB | -9.00 | 3.14 | 8.95 dB | -2.62 dBTP |
| -3 dB | -9.47 | 3.90 | 11.02 dB | -1.00 dBTP |
| -6 dB | -12.18 | 4.03 | 13.71 dB | -1.00 dBTP |

Lower input drive offers a measurable tradeoff here. It is not a recommendation
to apply a fixed -3 or -6 dB trim to every track.

## Most useful next engineering questions

1. **Coordinate drive with source level and requested loudness.** Test a bounded
   input-drive search with explicit target/ceiling constraints, preserving a
   user's ability to choose a dense result. Compare at the same delivered LUFS
   where feasible and report missed targets honestly. Avoid promising that extra
   crest and the same loudness are always simultaneously possible.
2. **Review the saturation amount mapping.** Current Universal 75 resolves to
   0.0715, but that is not a 7.15% wet blend. The entire signal passes through
   `tanh(x * drive) / tanh(drive)`, with `drive = 1 + 2 * amount` whenever amount
   is positive. Its limit as amount approaches zero from above still shapes the
   signal; amount zero bypasses it. Consider a continuous, gain-calibrated blend
   or drive mapping as an experimental alternative. This is a voicing decision,
   not grounds to silently replace the current sound.
3. **Test more material before choosing defaults.** Use dynamic, already-dense,
   bass-heavy and sparse mixes; include the same source at different input
   levels. Add synthetic transients, tones and multitone signals to measure
   attack/recovery and harmonic changes. Judge musical preference through blind,
   level-matched comparisons, not a single aggregate dynamics number.

The earlier proposal to keep compression amount and source adaptation, but give
them clearer UI hierarchy, remains reasonable. This evidence makes whole-chain
drive/saturation coordination a higher-value sound investigation than simply
combining their two sliders or globally disabling preset compression. Defaults
and the owner-gated Adaptive Compressor remain unchanged.

## Ready for listening and reproduction

Five anonymous 20-second clips were created locally at -16.0 LUFS each by
independent FFmpeg measurement. All are stereo 48 kHz float WAV with 5 ms edge
fades, gain matching only, and no added limiter. They use the source's loudest
20-second interval on a one-second grid, **195–215 seconds**. Their true peaks
remain below -1 dBTP. The candidates are current YES, saturation-bypass YES,
input-trim-minus-3 YES, LANDR Balanced High and the original. This loud passage
is useful for dynamics comparison but cannot establish whole-song quality.

- [Listening instructions](../../test-output/yes-stage-ablation-20260914/listening/README.md)
- [A](../../test-output/yes-stage-ablation-20260914/listening/A.wav),
  [B](../../test-output/yes-stage-ablation-20260914/listening/B.wav),
  [C](../../test-output/yes-stage-ablation-20260914/listening/C.wav),
  [D](../../test-output/yes-stage-ablation-20260914/listening/D.wav),
  [E](../../test-output/yes-stage-ablation-20260914/listening/E.wav)
- [Reveal key](../../test-output/yes-stage-ablation-20260914/listening/reveal-key.json)
- [Harness](../../test-output/yes-stage-ablation-20260914/src/main.rs)
- [Preparation / reproducible IPC exclusions](../../test-output/yes-stage-ablation-20260914/prepare.py)
- [Independent analysis script](../../test-output/yes-stage-ablation-20260914/analyze.py)
- [Independent measurements and byte-equality proof](../../test-output/yes-stage-ablation-20260914/independent-measurements.json)
- [All 18 render measurements](../../test-output/yes-stage-ablation-20260914/all-render-results.csv)
- [Source, compiler, harness and dependency provenance](../../test-output/yes-stage-ablation-20260914/completed-provenance.json)

No by-ear verdict has been made. No new installed-app, other-platform or release
claim follows from this harness. Private originals, renders, clips and local
scripts remain under ignored directories and do not travel with a normal Git pull.
The owner subsequently authorized their GitHub archive transfer, documented in
the guide linked above.
