# Reference-headroom candidate results

The [frozen twelve-file experiment](2026-09-21-reference-headroom-protocol.md)
passes all independent output checks and all eight quiet-copy comparisons.
Every new candidate meets the unchanged character limits under the qualified
static reference. Imaginal and Rich now have eligible normalized candidates;
their previous normalized three-point pools had none. Their large loudness
shortfalls remain explicit, and no sonic policy is adopted.

[Joined evidence](evidence/2026-09-21-dynamics-research/reference-headroom-v1.json)
and [verification](evidence/2026-09-21-dynamics-research/reference-headroom-verification-v1.json)
bind the twelve complete WAVs, measurements, exact settings, coefficients,
references, jobs, probe, process costs and independent logs. Protocol is
`a813c81d`, tooling is `b2f7538b`; all work is local.

## New candidate and comparison

| Source | Derived drive | Delivered LUFS | Section contrast change | Attack p10 change |
| --- | ---: | ---: | ---: | ---: |
| Piano | -8.786718 dB | -17.086933 | -0.145394 dB | -1.166796 dB |
| Imaginal | -10.200452 dB | -23.980768 | +0.156848 dB | -0.601812 dB |
| Metal | -10.695177 dB | -15.387812 | -0.014858 dB | -0.774995 dB |
| Rich | -7.112978 dB | -20.711742 | -0.213334 dB | -1.527435 dB |

Values above are original-level outputs; all copies retain qualification. The
same full chain, SRC and finalizer were used. Preset settings, compression,
transient shaping and saturation remain active. No limiter gain reduction is
observed in this matrix; maximum compressor reduction is 0.51 dB for Piano,
0.37 dB for Rich and below the 0.01 dB stored precision for Imaginal/Metal.
These activity readings support the mechanism but do not establish quality alone.

Imaginal's best earlier normalized attack p10 loss was 4.132509 dB, beyond
the 3.5 dB limit; the new candidate loses 0.601812 dB. Rich's best earlier
normalized section loss was 2.201290 dB, beyond 0.75; the new candidate loses
0.213334 dB. All other frozen character conditions also pass against the
static reference. The improvement in preservation costs substantial delivery
level: Imaginal misses -14 by 9.981 LU and Rich by 6.712 LU.

The **separate normalized-pool diagnostic**, comparing the existing zero/-3/+3
candidates and the new candidate, chooses:

| Source | Choice at original, -40 and -80 dB input | Delivered LUFS |
| --- | --- | ---: |
| Piano | Existing single -3 | -14.875332 |
| Imaginal | New headroom | -23.980768 |
| Metal | Existing single zero | -14.000000 |
| Rich | New headroom | -20.711742 |

All eight copy identities match; twelve groups give three qualified targets,
nine qualified shortfalls and no missing normalized candidate. This comparison
excludes raw current processing by its explicit diagnostic scope. It does not
replace the existing control-inclusive selector or settle current-control
eligibility. At original level, current processing is about 2.931 LU louder for
Imaginal and 0.435 LU louder for Rich than the new headroom candidate.

The historical matched-current comparison remains separate: **10/12 new
candidates pass**. Original-level Imaginal and Metal fail its upper tone-band
bound; their quiet copies pass its changed bounds. The qualified static
reference resolves that measurement dependence, but does not establish an
owner preference for the resulting tone. No historical limit or failure changed.

## Verification and work

All twelve files have exact expected 48 kHz stereo frames, finite PCM, no
full-scale samples, native peak <=-1 dBTP, passing padded SOXR16/64 references,
and finite loudness independently agreeing within 0.019232 LU. The highest
independent peak is -1.041582 dBTP. The 0.002 dB peak and 0.11 LU agreement
tolerances remain unchanged; complete gating-block logs establish availability.

Maximum copy differences: PCM **4.76838e-7** (budget 1e-4), loudness and drive
**0.000001908** LU/dB (budget 0.001), anchored attack **0.000003927 dB**
(budget 0.01). Section, tone and stereo comparisons also pass. Native rendering
took **174.087 s wall / 169.672 s CPU / 792.8 MiB peak**, below 2 GiB.
Included stage sums are 3.904 s decode/descriptors, 37.712 s chain, 1.814 s SRC
and 125.817 s finalization; remaining wall time includes I/O and hashing.
Character measurement takes 29.148 s and independent checking 387.396 s.
Stages ran sequentially. The twelve new WAVs total **1,130,523,216 bytes**.

Static-reference preparation is reused from the prior checkpoint and is not
included in native timing. This is neither an app-wide speedup nor a complete
production preparation budget. The normalized-pool comparison uses four
candidates; it does not prove that this single headroom candidate alone gives
the best delivery for every source.

Release build, strict Clippy, formatting, six diagnostic regressions and seven
availability regressions pass. The native render function matches the prior
verified probe. All previous audio and evidence remain hash-identical; shipping
source and lockfiles remain unchanged. Raw files are ignored under
`test-output/dynamics-research-20260921/reference-headroom-v1/`.

## Next dependency

Continue with the [fresh source-analysis audit](2026-09-21-source-analysis-contract-protocol.md).
The experiments above retain the original source profile/guards; fresh analysis
may respond differently to gated quiet material. Measure that before claiming
end-to-end gain-copy consistency. Full user-control and preset intent, current
fallback policy, runtime/cost freeze, C2 holdout and C3/listening remain open.
