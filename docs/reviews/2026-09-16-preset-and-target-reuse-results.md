# Preset coverage and prepared-target reuse

## What the additional comparisons establish

The [broader first-candidate checkpoint](2026-09-16-broader-first-protocol.md)
completes **16 independently verified whole outputs**: Funk/Rich, Universal 50/
Loud 75, -14/-9 targets, matched current/single processing. Source/settings and
coefficient strings match the completed C1 anchors. Every delivered file meets
the qualified native -1 dBTP ceiling and independent peak/LUFS/frame/channel/
rate/finite/full-scale checks. Audio correctness and character eligibility remain
separate. No preset coefficients or production processing were changed.

The fixed preserving selector returns:

| Source / preset | -14 request | -9 request |
| --- | --- | --- |
| Funk / Universal 50 | Single, -14.000; character and target pass | Current, -12.606; character fallback and target miss |
| Funk / Loud 75 | Current, -14.000; character fallback | Current, -13.546; character fallback and target miss |
| Rich / Universal 50 | Current, -20.276; character pass, target miss | Current, -20.276; character pass, target miss |
| Rich / Loud 75 | Current, -14.251; character fallback and target miss | Current, -14.251; character fallback and target miss |

Thus one pair yields a qualified target, two yield a character-qualified result
below target, and five yield an explicitly recorded character fallback. These
are not eight successful new masters. Loud 75 intentionally produces dense
processing; a failed natural-dynamics threshold is not proof that its intended
voicing is defective or permission to change it. The same universal constraint
set is not ready for automatic use across all presets.

Funk Universal 50 illustrates why target-dependent input drive needs care:
its -14 single loses 0.427 dB of section contrast and 1.921 dB of median paired
attack. The -9 single loses 1.694/4.129 dB, additionally fails tone/stereo checks,
and still only reaches -12.185 LUFS. Rich Loud 75's -9 single reaches -9 but
loses 12.226 dB of section contrast and fails multiple tone/stereo constraints.
Target arrival alone does not qualify either choice.

Whole native job cost: **193.136 s wall / 189.406 s CPU / 431 MiB peak working
set**. Chain/SRC/finalization sum to 189.077 s; separate character metrics cost
32.187 s and independent references 446.661 s. Decode/normalization happen once
per source (Funk 0.072 s, Rich 0.124 s), shared across eight renders each.
Original source analysis/anchors are reused, not remeasured. Phase work overlaps
and must not be added as a claimed end-to-end wall time.

Exact inputs, observations and selections are in the
[broader evidence](evidence/2026-09-16-dynamics-research/broader-v1.json).

## Reuse prepared audio when the delivery target changes

The [target-reuse protocol](2026-09-16-target-reuse-protocol.md) takes each -14
selection above, keeps its DSP character, and changes only final delivery to -9.
It reuses already landed 48 kHz float PCM as an offline prepared-buffer proxy;
this is not a claim of bit equality with a future raw-buffer implementation.

All **four new whole files independently pass**. Output differs from its prepared
buffer only by scalar gain within the frozen 1e-6 normalized-sample tolerance.
Section/paired-attack/band/side-mid descriptors remain within 0.0001 dB and
correlation within 1e-6 of the preparation. Existing character failures remain
visible; a scalar target change cannot repair an unsuitable preparation.

| Preparation reused | Delivered at -9 request | Character result | New-target finalization, including clone |
| --- | --- | --- | --- |
| Funk / Universal 50 single | -13.322 LUFS | Pass | 7.717 s |
| Funk / Loud 75 control | -13.546 LUFS | Existing dynamics failures remain | 8.895 s |
| Rich / Universal 50 control | -20.276 LUFS | Pass | 9.511 s |
| Rich / Loud 75 control | -14.251 LUFS | Existing section failure remains | 8.543 s |

For Funk Universal 50, reuse preserves the gentler preparation's dynamics and
tone, while landing 1.137 LU below the target-dependent single. Both miss the
-9 target; the reusable result better fits the owner's experimental direction.
That is a specific measured tradeoff, not evidence that -14 should become a
universal operating reference or that all presets now satisfy a fidelity goal.

The public finalizer used here still remeasures the whole prepared buffer.
Changing the target therefore costs **7.7–9.5 s** even though chain/SRC work is
avoided. Previously prepared -14 and repeated -9 lookups return the exact same
stored Arc buffer with no further finalizer call; observed 0–100 ns lookup
readings reach timer resolution and are not credible app response estimates.

The native child costs 36.216 s wall / 35.422 s CPU / 275 MiB peak working set.
Decode is separate (0.051–0.116 s per buffer); initial preparation costs remain
in the broader evidence. See the
[target-reuse evidence](evidence/2026-09-16-dynamics-research/target-reuse-v1.json).

## Follow-through and limits

The **existing production prepared-measurement API** passes the paired checkpoint:
all **16 complete outputs are bit-identical** to the four independently qualified
target-reuse files, with matching returned gain/LUFS/peak. No additional WAVs
were needed. Its [frozen protocol](2026-09-16-prepared-measurement-reuse-protocol.md)
uses two rounds in fresh/reused then reused/fresh order, with one source decode
and one facts preparation per buffer. Both paths include cloning the immutable
buffer. The deliberately ignored release-unit probe passes; strict Rust Clippy,
formatting and the applicable research Python compilation pass.

| Prepared buffer | One-time facts preparation | Fresh finalization mean | Reused-facts finalization mean | Two fresh calls / prepare + two reused calls |
| --- | --- | --- | --- | --- |
| Funk / Universal 50 | 6.944 s | 7.636 s | 0.263 s | 15.272 / 7.470 s |
| Funk / Loud 75 | 7.002 s | 7.941 s | 0.256 s | 15.882 / 7.514 s |
| Rich / Universal 50 | 9.290 s | 8.272 s | 0.316 s | 16.543 / 9.921 s |
| Rich / Loud 75 | 8.907 s | 8.108 s | 0.319 s | 16.217 / 9.546 s |

For this stage, subsequent evaluations are **25.4–31.0 times faster**, with the
same complete output. Including preparation, two evaluations cost **40.0–52.7%
less observed wall work** than two fresh calls. These are two observations per
path on four known buffers, not a broad platform performance distribution.
The native child costs 99.902 s wall / 98.125 s CPU / 285 MiB peak working set.
Its total includes reference/source decoding, hashing, equality checks and both
competing paths; the table isolates the paired work. Full observations, hashes
and one-time costs are in the
[prepared-measurement evidence](evidence/2026-09-16-dynamics-research/prepared-reuse-v1.json).

This is measured support for retaining an immutable processing result **and its
validated measurements** across compatible target changes. Reusing PCM alone
left most repeated metering work in place. The production app already contains
this API; the experiment demonstrates its value for a possible future resolved
drive plan, not a newly shipped 25-times app speedup. The test module stays behind
`cfg(test)` and private preparation APIs remain private.

The practical next direction is to develop the processing/character choice
separately from final delivery gain, preserving intentional presets and manual
controls, then reuse the valid plan/buffer/facts. The four-buffer proxy does not
settle which operating reference or preset-specific bounds should be adopted.

All new behavior remains in standalone experiments or an explicitly ignored,
`cfg(test)`-only benchmark. The current application still uses the verified
main processing; no new selector, caveat, preset or mode is enabled. Research
diagnostics stay internal, with accurate results retained.

C1 still needs an acceptable source/preset processing contract and measured
budgets before C2's unseen holdout. Broader all-source/quiet-copy coverage and
C3's control/cache/audition/export integration remain required. Initial source
import, product edit settling, callback timing, installed listening and release
evidence remain separate from these offline measurements. The main integration
record retains those verified observations and outstanding B3 limits.
