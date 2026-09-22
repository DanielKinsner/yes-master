# Fresh source-analysis contract results

The [frozen audit](2026-09-21-source-analysis-contract-protocol.md) completes
all 24 production analyses and independently verifies all 24 input transforms.
Fresh actual-file profiles are not gain-invariant across this quiet-copy matrix:
**seven of eight** exceed the profile comparison limits, and **two of eight**
change compressor coefficients. Fresh normalized-copy profiles pass **8/8**,
with exact coefficient debug identity in **8/8** comparisons.

[Analysis evidence](evidence/2026-09-21-dynamics-research/source-analysis-contract-v1.json)
and [verification](evidence/2026-09-21-dynamics-research/source-analysis-contract-verification-v1.json)
retain all results, original and normalized availability, profiles, resolved
settings, deep-analysis presence/counts, binary/jobs, inputs and cost. Protocol
is `2577b0da`, analysis tooling is `951f8ff3`. Shipping code remains unchanged.

## Measured level dependence

Only LRA exceeds the frozen profile tolerances in the seven failing copies.
Spectral shares, P95-P10 DR, correlation and width stay within their budgets.

| Source | Actual original LRA | Actual -40 dB LRA | Actual -80 dB LRA | Normalized LRA at all levels |
| --- | ---: | ---: | ---: | ---: |
| Piano | 12.061390 LU | 11.923247 | 0 | 12.061390 |
| Imaginal | 7.024446 LU | 4.650180 | 0 | 7.024446 |
| Metal | 0.971905 LU | 0.971905 | 0 | 0.971905 |
| Rich | 11.500441 LU | 10.985008 | 0 | 11.500441 |

All four actual -80 sources have unavailable native integrated LUFS, while
the existing analysis API reports its -70 sentinel. All twelve normalized
sources have available integrated LUFS. These are separate facts: normalization
does not make the original below-gate file's loudness available. No meter/API
sentinel or source-file measurement is changed by this research.

At Universal 50, the observed coefficient changes affect all three compressor
bands. Imaginal -40 changes threshold **-12.5 -> -9.687875 dB**, ratio
**1.45 -> 1.3487635**, and makeup **1.9396554 -> 1.2525461 dB**.
Metal -80 changes threshold **-6.25 -> -6.8076806 dB**, ratio
**1.225 -> 1.2450765**, and makeup **0.5739797 -> 0.67000014 dB**.
Other actual-copy coefficient strings match their originals exactly despite
their changed LRA where the active guardrail response does not change.

The [post-audit replay](2026-09-21-source-analysis-coefficient-replay-protocol.md),
frozen at `60fd0310` and implemented at `05244070`, restores only original LRA
inside each quiet copy's source profile. All eight resulting coefficient strings
exactly match their originals, including both previously changed cases. This
isolates LRA as sufficient to explain those coefficient changes. It is an
intervention for diagnosis, not a proposal to replace actual-file LRA with a
fabricated measurement. The production density guard reads LRA only above
0.5 LU and combines that trigger with P95-P10 DR; below-gate zero therefore
has a different role from a measured low-but-available range.

## Closure with the headroom research

All twelve normalized input PCM hashes match the already-qualified static
reference inputs. All four normalized original analyses also pass against
their actual originals and resolve identical base coefficients. Across quiet
normalized copies, maximum profile differences are **0 LU LRA**, **9.53674e-7
dB DR**, and **1.49012e-8 spectral share**; correlation/width are unchanged.

The coefficient-only replay then injects each fresh normalized profile/LUFS
into its existing headroom candidate settings and applies that candidate's same
f32 automatic drive. **All twelve complete coefficient strings exactly match
their verified mastering recipes.** This closes the retained-profile dependency
at the input/recipe level for this fixed matrix. It creates no new master and
does not claim a fresh native-app render, complete cache integration, listening
equivalence, or invariance of every deep-analysis window statistic.

The useful engineering direction is to keep actual-file loudness/availability
separate from a normalized source descriptor used to resolve automatic
processing. The headroom experiment's nine normalized-pool target shortfalls,
historical current-reference tone failures and unresolved current-control
eligibility remain exactly as recorded. No sonic policy is adopted.

## Verification, work and next scope

Native audit cost: **92.497 s wall / 90.938 s CPU / 277.2 MiB peak**. The 24
complete analyses account for 81.471 s (40.779 actual / 40.693 normalized),
input preparation/availability checks for 8.055 s and initial source decoding
for 0.653 s. Twenty derived input WAVs use **1,786,292,360 bytes**; the four
original source files are reused. These files are local analysis inputs, not
masters. Independent full input PCM/shape/transform verification takes 8.124 s.
All twelve normalized input hashes and expected source rates/frame counts match.

Both offline examples pass release build, strict Clippy and formatting. Three
focused audit regressions pass. The production analysis file is included
unchanged, and its hash is bound in the job. Protocols, prior outcomes, source
files, shipping source/lockfiles and agent instructions remain unchanged.
Raw inputs, binaries, replay, report and logs remain in ignored
`test-output/dynamics-research-20260921/source-analysis-contract-v1/`.

Next, qualify the combined normalized-descriptor/static-reference/headroom
recipe against the existing C1 user-control and preset contracts: manual input
trim, Density/Adapt, compressor Off/overrides, dense presets, Custom without a
target and Album behavior. Freeze that bounded matrix before outcomes. Compare
the raw current control and normalized candidates explicitly; the diagnostic
pool is not an adopted replacement for current-control eligibility. Keep
preparation/reuse costs separate from candidate-policy quality. C2 holdout and
C3 production/listening adoption remain unopened.
