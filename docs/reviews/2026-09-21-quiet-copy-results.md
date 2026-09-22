# Corrected-policy quiet-copy results

Local C1 development checkpoint on `codex/mastering-bidirectional-research`.
The [frozen protocol](2026-09-21-quiet-copy-protocol.md) covers four known sources,
Universal 50, -14 LUFS/-1 dBTP, and -40/-80 dB source copies. All 32 planned
whole WAVs were rendered; 16 original-level references remain unchanged.
Independent checks are complete: **31/32 pass all checks**; Metal's -80 dB
current control fails the frozen loudness-availability guard. All 32 pass
peak, finite/full-scale and exact format/frame checks. All 24 normalized
outputs pass both complete-file checks and numerical copy budgets. The
checkpoint is completed research with preserved failures, not a qualified policy.

The [joined evidence](evidence/2026-09-21-dynamics-research/quiet-copy-v2.json)
contains native, character, independent and selection results. The
[verification record](evidence/2026-09-21-dynamics-research/quiet-copy-verification-v2.json)
binds artifacts, validation, interruption recovery and the failed witness.

**The normalized processing is consistent, but the selection policy is not
source-level independent.** All 24 normalized candidates pass every frozen copy
budget. Only four of eight copy groups retain the original selected policy and
offset. Changing the comparison reference changes Metal's qualifying candidates;
unavailable current-control loudness prevents a fallback on two very quiet groups.
Neither observation is a reason to loosen the existing limits or expand the grid.

## Selection and character

The primary comparison uses each copy's current processing as its tone/stereo
reference, with unchanged source anchors and qualification limits.

| Source | Original choice | -40 dB copy | -80 dB copy |
| --- | --- | --- | --- |
| Piano | Single -3, -14.875 LUFS | Same choice, -14.875 | Same choice, -14.875 |
| Imaginal | Current, -21.050 LUFS | Current, -25.329 | No eligible fallback |
| Metal | Single +3, -14.000 LUFS | Single 0, -14.000 | Single 0, -14.000 |
| Rich | Current, -20.276 LUFS | Current, -22.155 | No eligible fallback |

Piano retains qualified character with a target shortfall. Metal reaches the
target under either choice, but selection identity fails on both copies. The
two -40 dB current choices retain qualified character with larger target
shortfalls: Imaginal changes by -4.279 LU and Rich by -1.878 LU. Matching the
policy name does not establish matching audio or delivery level. Across eight
groups there are two qualified targets, four character-qualified shortfalls and
two explicit `no_verified_fallback` results.

For Metal's -40 dB zero-offset candidate, the upper tone-band deviation is
1.067125 dB. Its allowed bound moves from 0.981036 dB with the original control
to 1.399971 dB with the copy-specific control. The normalized candidate barely
changes numerically; the changed bound makes it eligible. This isolates a
reference-policy problem, rather than a normalization precision failure.

The fixed-original-reference diagnostic recovers Metal +3 on both copies and
matches original selection identity in six of eight groups. It does not fix
unavailable current fallback or level-dependent current processing. Imaginal's
-40 dB current control also fails character under that fixed reference, becoming
an explicit `character_fallback`; only five of eight original reason/target
outcomes match. An arbitrary import has no known hypothetical unattenuated
original, so this diagnostic is not a deployable reference rule.

## Numerical consistency and measurement availability

| Maximum across 24 normalized comparisons | Observed | Frozen budget |
| --- | ---: | ---: |
| Absolute delivered PCM difference | 0.0000007153 | 0.0001 |
| Delivered LUFS difference | 0.000001908 LU | 0.001 LU |
| Normalized descriptor difference | 0.0000002355 LU | 0.001 LU |
| Resolved drive difference | 0 dB | 0.001 dB |
| Compensated normalization difference | 0.0000002357 dB | 0.001 dB |
| Section contrast difference | 0.0000002435 dB | 0.01 dB |
| Maximum paired attack difference | 0.000003401 dB | 0.01 dB |
| Maximum tone-band difference | 0.0000002926 dB | 0.01 dB |
| Side/mid difference | 0.00000002027 dB | 0.01 dB |
| Correlation difference | 0.000000002122 | 0.00001 |

All four -80 dB current-control outputs have unavailable native integrated
loudness, recorded as null. This is distinct from technical invalidity and from
an invented -70 LUFS measurement. The frozen selector requires finite loudness
and rejects those controls. Piano/Metal have eligible normalized alternatives;
Imaginal/Rich do not. No nonfinite value is serialized as a measured result.

**Metal -80 current fails the frozen independent availability check.** FFmpeg
shows I=-70 and integrated threshold=0, but its independent peak is -69.071192
dBFS rather than below -70. The extra guard therefore does not qualify native
null as independently confirmed unavailable. All 1,904 logged momentary
readouts are below -70 (maximum -82.9 LUFS). This supports a follow-up on the
availability classifier, not an assertion that the meters actually disagree.
The reference report's `availability_mismatch`, availability=true and -70 fields
are the frozen checker's diagnostic classification; **-70 is not established as
measured integrated loudness**. Preserve those raw fields and the failed result.
Do not remove the peak guard retroactively or promote this control to eligibility.

The summary now archives completed failed checks instead of refusing to emit
an evidence report. It records independent qualification separately and excludes
failed reference checks from eligible candidates. This changes reporting and
failure propagation, not the frozen numerical/character/availability criteria.

Original analysis/guards and source-anchored scoring windows were retained.
This verifies corrected normalized processing with those fixed inputs; it does
not establish fresh production analysis invariance or manual-control behavior.

## Cost, recovery and evidence

Native child: 584.180 s wall, 547.875 s CPU, 793.6 MiB peak working set. The
2 GiB native-memory budget passes. The 32 new WAVs use 3,014,728,576 bytes.
Chain/SRC/finalization work totals 114.412/6.839/437.344 s; character measurement
work totals 75.718 s and independent reference work 968.523 s. Character and
independent stage wall observations are 79.113/974.016 s. Source decode and copy
descriptor work totals 0.398/4.317 s. The largest independent delivered peak
is -1.037999 dBFS, below the requested -1 dBTP ceiling. The full diagnostic matrix renders every candidate;
these timings are not a conditional early-stop or app-latency benchmark.

The first native attempt stopped before creating audio: raw JSON decimal
serialization differed from the same value represented through production f32
settings. Version 2 deserializes both settings through `MasteringSettings` before
exact comparison. No tolerance or DSP code was changed. The failed v1 job,
executable, process report, log and empty output directory remain preserved.

The conversation timed out after v2 rendering completed. Recovery verified the
complete native/process reports and every one of the 32 saved WAV hashes, then
continued character/reference measurement without rerendering. Native rendering,
character measurement and independent checks run sequentially; the interruption
gap is not included in processing throughput.

Raw jobs, copied executables, audio and logs remain ignored under
`test-output/dynamics-research-20260921/quiet-copy/`. The original bidirectional
evidence and all historical failures remain intact. Production source, preset
voicing, saved values, release status and pushed main remain unchanged.

Twenty-three selector regressions and three availability regressions pass,
including the new measured selection-shift and failed-guard witnesses. Python
compilation, document links and whitespace checks pass. An initial test command
from the script directory could not resolve historical repo-relative inputs;
that log is retained, and the suite passes from the repository root without
changing test assertions. The previously passing release example build, strict
Clippy and Rust formatting remain applicable: native code has not changed since
`6f8ca6de`. The rendering function still matches the prior single-first example.
AGENTS/CLAUDE remain byte-identical. No native playback, installed-app,
listening, holdout, Mac or new remote-CI qualification is claimed.

## Next bounded research

Continue C1 by defining a reproducible comparison reference whose operating
level does not depend on arbitrary input gain. Separate preset tone intent,
dynamics qualification and valid below-gate fallback behavior. The earlier
low-drive convergence attempt failed its frozen budgets; it is not an established
linear reference and cannot silently replace the current bound. Any new rule
needs its own frozen protocol and must preserve these failed witnesses.

Then verify the automatic/manual, dense-preset, no-target, Off/Adapt, Album and
saved-session contracts in the [control table](2026-09-21-c1-control-contract.md),
along with broader known-source/target coverage and runtime bounds. C2 unseen
material stays closed until the policy is ready; C3 app integration,
responsiveness and targeted listening remain separate. No sonic policy is
adopted by this checkpoint, and no new push or deployment is performed.
