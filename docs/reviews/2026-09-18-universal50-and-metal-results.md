# Universal 50 coverage and the retained Metal drive lead

Status: both checkpoints verified. All **24 new coverage files and two new
Metal follow-up files** pass independent peak/LUFS, exact frame/
rate/channel, finite/full-scale and hash checks. All qualified native peaks meet
the -1 dBTP request; the highest bound is -1.000010 dBTP. Retained Funk/Rich
files and observations revalidate exactly. No audio or limits were replaced.

## What the broader measurements show

The [frozen coverage checkpoint](2026-09-18-universal50-coverage-protocol.md)
adds Coat, Piano, Imaginal, Metal, Aphelion and Baby at Universal 50, -14/-9,
with matched current and normalized single processing. The eight retained
Funk/Rich Universal 50 outputs complete **32 outputs / 16 matched pairs** across
the original eight sources. Original source levels only; this is development
coverage, not unseen holdout or completion of the quiet-copy matrix.

The single candidate meets the unchanged character limits on **four of eight
tracks at -14**: Coat, Funk, Aphelion and Baby. All four reach -14. At -9, **all
eight single candidates fail at least one character constraint**. No acceptance
limit or coefficient was changed. A louder result does not establish a better
processing choice, and a numerical qualification does not establish listening
preference. These are two related observations per known track, not 16 independent
test pieces or a statistical success-rate estimate.

The unchanged control-inclusive selector resolves **four qualified targets,
five character-qualified shortfalls and seven character fallbacks**. A fallback
is technically valid current processing with its character failures retained;
it is not a character pass. All eight -9 comparisons remain unresolved by this
two-candidate rule, including those whose fallback happens to reach -9.

| Source | -14 selected output | -9 selected output |
| --- | --- | --- |
| Coat | Single, -14.000; character + target pass | Current, -9.000; character fallback |
| Piano | Current, -14.000; character fallback | Current, -11.657; character fallback + shortfall |
| Imaginal | Current, -21.050; character pass + shortfall | Current, -21.050; character pass + shortfall |
| Metal | Current, -14.000; character fallback | Current, -9.000; character fallback |
| Aphelion | Single, -14.000; character + target pass | Current, -9.510; character fallback + shortfall |
| Baby | Single, -14.000; character + target pass | Current, -16.386; character pass + shortfall |
| Funk (retained) | Single, -14.000; character + target pass | Current, -12.606; character fallback + shortfall |
| Rich (retained) | Current, -20.276; character pass + shortfall | Current, -20.276; character pass + shortfall |

The [fresh evidence](evidence/2026-09-18-dynamics-research/universal50-v1.json)
and [joined eight-source evidence](evidence/2026-09-18-dynamics-research/universal50-joined-v1.json)
bind inputs, copied binary, protocol, complete reports, per-indicator measurements
and selections. Joining preserves the retained observations and costs exactly.

Some concrete changes at Universal 50:

- Coat at -14: fixed-section contrast loss improves from **1.312 to 0.184 dB**.
  Both versions reach -14. Its single at -9 loses **1.026 dB**, exceeding the
  same 0.75 dB section limit.
- Piano: the single's section loss is **1.610 dB at -14**, then **4.070 dB** at
  the -9 request. The latter reaches -9.052 LUFS but additionally fails attack
  and tone checks. Lowering the preset intensity alone has not resolved this.
- Imaginal: the single's median/lower-decile paired attack losses grow from
  **3.754/6.235 dB** at -14 to **7.078/9.330 dB** at its -9 request, which
  delivers -9.988. The current control retains qualified character at -21.050.
- Baby: the single improves target attainment from the current -16.386 to -14
  while staying inside the limits, but its median attack loss rises from
  **0.224 to 1.268 dB**. This is not an improvement on every indicator. Its -9
  single loses **4.625 dB** of median attack and fails additional checks.
- Metal: at -14 the current control loses **4.415 dB** of median attack;
  the single loses **0.229 dB** but still fails the top tone band. Reproducing
  the historical +3 dB candidate is a separate experiment, described below.

## Cost of the new coverage

Native rendering finished before the character job began, and the independent
references started after character measurement. The later Metal probe build
overlapped independent verification; reference timings are not an isolated
throughput benchmark. No research render overlapped the new native rendering.

| Once-per-source preparation / four evaluations | Decode + normalization | Chain + SRC + finalization |
| --- | ---: | ---: |
| Coat | 0.138 s | 78.280 s |
| Piano | 0.121 s | 43.977 s |
| Imaginal | 0.190 s | 79.725 s |
| Metal | 0.105 s | 44.096 s |
| Aphelion | 0.207 s | 106.441 s |
| Baby | 0.171 s | 83.326 s |

The 24 evaluations use **435.846 s** of measured stage work: **96.055 s chain,
5.180 s SRC, 334.611 s finalization**. The native child takes **445.084 s wall /
435.906 s CPU / 753.5 MiB peak working set**, including decode, hashing and I/O.
Character measurement work is **68.925 s**. Source analysis and anchors are
reused and excluded from these new observations. The 24 new WAVs occupy
**2,634,518,304 bytes**; old evidence and private sources remain intact.
Independent reference work totals **892.498 s**. Full character/reference job
wall observations, including startup/read/hash/reporting, are **72.009/905.835 s**.

Finalization accounts for about 77% of the measured evaluation work. This is
consistent with the already verified value of retaining prepared peak/loudness
facts; this matrix does not itself test that optimization. See the
[paired reuse measurements](2026-09-16-preset-and-target-reuse-results.md).
These new costs are offline evaluation costs. Initial app preparation, target
settling, native playback responsiveness and whole user-session cost remain
separate evidence lanes; no new app-wide speedup is claimed.

## Separate Metal reproduction

The [two-render protocol](2026-09-18-metal-positive-drive-protocol.md) reuses the
historical C1 finding at +3 dB relative to the single rule, at Universal 50/75
and -14. The older arithmetic's +3 candidates qualified, while +6 failed the
median-attack constraint. The corrected engine must verify this independently.
The frozen `control-inclusive-preserving-v1` grid excludes positive offsets,
so this follow-up cannot retroactively rewrite the broader selector outcomes.

Both corrected-chain +3 candidates **meet the -14 target and every unchanged
character constraint**, with independent whole-file delivery passes. The four
retained corrected controls/singles are verified by source/settings/coefficients,
file hashes and their complete independent evidence before the two new renders.
No historical f32 output is substituted for a corrected-chain result.

| Metal / -14 | Median attack loss: current / zero-offset / +3 | +3 section loss | Top-band source deviation: zero-offset / +3 / allowed |
| --- | ---: | ---: | ---: |
| Universal 50 | 4.415 / 0.229 / **1.080 dB** | 0.109 dB | 1.067 / **0.950** / 0.981 dB |
| Universal 75 | 4.690 / 0.182 / **1.071 dB** | 0.110 dB | 1.430 / **1.309** / 1.338 dB |

The middle candidate retains substantially more attack than the current control
and clears the tone constraint that rejected the zero-offset single. It has less
attack preservation than the zero-offset single, so this is a measured tradeoff,
not a dominance claim. The tone margins are only **0.031/0.028 dB**; this does
not establish robustness across different sources, settings or measurement
methods. The +3 relative offset is an observed development point, not a new
shipping drive constant. Earlier negative-offset failures remain unchanged.

Additional evaluation work is **20.559 s** (9.859/10.700 s per intensity).
Decode/normalization happen once, **0.067 s**. Native child cost is **21.264 s
wall / 20.453 s CPU / 348.4 MiB peak working set**. Character work is **3.214 s**;
independent reference work is **48.570 s** (49.529 s full reference job wall).
The two new files use **146,272,488 bytes**. Rendering, references and character
reporting run sequentially after the broad checkpoint finishes. Costs of retained
controls and source analysis remain in their original records, not counted as
new or represented as free historical preparation.

The [Metal evidence](evidence/2026-09-18-dynamics-research/metal-positive-v1.json)
binds the two new outputs, four retained rows, original historical leads, frozen
protocol and copied executable. The fixed selector and its earlier seven
character-fallback outcomes have not changed. This is specific evidence that
the next search should consider both directions, rather than only more attenuation.

## Direction and remaining limits

Continue developing a processing choice separately from delivery gain, with a
bounded search in **both directions** where the first candidate fails. Retain
the corrected current control as an eligible candidate. The current simple formula is
not qualified as a general automatic policy by these measurements. Preserve the
original target misses and failures while investigating the available directions.
An experimental -14 preparation reference is not a chosen universal operating
level, and Universal 50 is not a new default intensity.

The source/preset/control contract, full corrected-policy quiet-copy coverage,
runtime/fallback freeze, new licensed holdout and C3 production integration
remain open. Dense presets retain their intended behavior and separate evidence.
No saturation curve, Adaptive Compressor, Phase-B or album-character behavior
is adopted. Local main stays `ab420654`; research remains local on
`codex/mastering-dynamics-research`.

## Verification and recoverability

The release example builds and strict example Clippy pass for both native
checkpoints. Rust formatting and research Python compilation pass; the existing
11 selector regressions pass with the unchanged selector. Only research examples,
scripts, evidence and internal docs changed in this session. Per `docs/TESTING.md`,
unaffected full app/fixture/bridge/UI/package lanes are reused from the verified
integration; no new native interaction, installed, listening or remote CI claim.

Raw jobs, logs, copied binaries, reports and private WAVs are preserved under
`test-output/dynamics-research-20260918/`. Reports distinguish the earlier
preflight failures from successful runs: Coat's v1 report lacked Universal 50;
the older Metal Universal 75 report lacked a convenience label. Both stopped
before audio, and the corrected provenance checks retained every acceptance
requirement. No threshold was relaxed to repair either harness issue.

The [checkpoint verification record](evidence/2026-09-18-dynamics-research/verification.json)
records artifact hashes, stage timing/scheduling, validation and unchanged main.
