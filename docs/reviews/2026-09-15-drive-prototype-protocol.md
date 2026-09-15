# C1 drive prototype: measurement protocol

Status: development specification, before new C1 comparisons or holdout audio
acquisition. C0 results remain development evidence. Numeric selection limits
will be calibrated on development material and frozen in a separate versioned
manifest before any holdout processing. No production drive policy is enabled.

## Controls and scope

Keep the frozen historical output and the corrected production control. Compare
current processing, C0's single source-relative rule, its attenuate-only first
candidate, and a maximum eight-point initial source-relative grid with at most
two refinements. Reuse completed C0 development results; do not call the original
eight sources unseen. Evaluate full continuous songs; passage ranking is not
qualified. Every finalist receives whole-file delivery checks. Legacy descriptive peak fields from the music-analysis
script are not used as final protection or ceiling certification.

Separate source normalization, experimental operating drive, preset push,
deliberate manual trim and final gain. The experimental orchestration resolves
an immutable plan before chain construction; it must not save its answer into
requested input gain, Density, Adapt or target. Keep existing preset coefficients
and gated policies unchanged. A requested manual trim must still change the
nonlinear response. No target and Manual/Off/album behavior remain production
controls until their specific policy is resolved and checked.

## Measurements fixed before development comparisons

- Source identity is the exact decoded source/file hash, transformation and
  analysis version. Reuse existing analysis/settings where available. Record
  whether existing level-dependent guards were retained or recomputed; an
  invariant analysis anchor for synthetic gain copies is an experiment boundary,
  not proof that arbitrary production analyses are gain-invariant.
- Construct unclipped float gain copies at unity, -40 and -80 dB. Retain actual
  integrated-loudness gate availability separately. A normalized, channel-aware
  source measurement can supply an operating-level descriptor below the absolute
  gate; do not label it the actual source LUFS. Use one original-source passage
  map for every copy. Compare normalized responses, selected drive after removing
  source normalization, whole output and target/peak results.
- Peak correctness: reject nonfinite samples, wrong frame/channel/rate counts,
  technically invalid output or a qualified bound above the requested ceiling.
  Retain independent finite-zero-extended SOXR/direct-reference scope and the
  existing 0.002 dB independent numerical tolerance. Do not increase the requested
  ceiling or use integer clipping as protection. Recheck affected finalists with
  the integrated qualified finalizer before any production adoption.
- Target error: report signed delivered integrated LUFS minus requested LUFS.
  Use full-precision EBU readings and the existing 0.2 LU research feasibility
  tolerance; retain the separate 0.25 LU product informational threshold.
  Below-gate LUFS is unavailable, not a fabricated floor or successful target.
- Section dynamics: retain the original source's fixed ten-second RMS sections
  and its originally eligible quiet/loud indices. Report every section's change
  after removing common gain, plus the fixed loud-minus-quiet contrast change.
  Eligibility is never recomputed on an attenuated copy to improve its score.
- Attacks: retain source-derived transient times from the existing music-analysis
  tool. Report each aligned transient's peak-to-surrounding-RMS change and the
  distribution, not just global crest. The existing detector uses 10 ms RMS
  novelty, at least 200 ms between candidates and up to 40 source anchors;
  attack spans -20 to +30 ms and body spans +30 to +150 ms. Global crest and LRA remain separate
  descriptors; neither is a quality objective by itself.
- Tone: use the existing fixed spectral bands and remove common RMS gain before
  comparing each band's level with the source and corrected control. Retain the
  existing 48 kHz analysis resampling and Welch 8192/4096 segment/overlap method. Keep band
  changes individually visible; do not collapse them into a hidden score.
- Stereo: report left/right energy, mid/side energy ratio and correlation with
  source/control differences, plus per-channel peak compliance. Mono cases have
  unavailable stereo descriptors. Preserve cancellation-prone stereo sources.
- Limiter activity: instrument the experiment's actual gain history, recording
  reduction percentiles/max and fraction of active samples. Compressor/limiter
  reduction is explanatory evidence, not a substitute for attack/section/tone
  measurements or listening.
- Time/storage: report source preparation, raw response rendering, delivery
  verification and full selection separately, with candidate count and retained
  bytes. Repeat controlled timing separately from parallel development work.
  Native edit/readiness and callback measurements remain their own evidence.

Invalid metrics stay unavailable with a reason. An unavailable required constraint
cannot produce a feasible selection. Preserve failed candidates and corrected
controls; do not remove difficult sections or repeatedly widen the search.

## Initial numeric candidates from completed C0 development

Paired source-anchored attack deltas are measured per transient before taking
percentiles; subtracting two independently computed medians gives a different
statistic and is not used here. C0's -14 single-rule outputs show:

| Source | Section contrast change | Paired attack median | Paired attack p10 | Largest normalized tonal-band change |
| --- | ---: | ---: | ---: | ---: |
| Owner | -0.18 dB | +0.28 dB | -0.88 dB | 1.30 dB |
| Funk | -0.48 dB | -1.93 dB | -3.20 dB | 2.10 dB |
| Aphelion | -0.59 dB | -1.41 dB | -2.22 dB | 1.54 dB |

For the next development iteration, compare a dynamics-preserving candidate with
section-contrast loss at most **0.75 dB**, paired median attack loss at most
**2 dB**, and paired p10 attack loss at most **3.5 dB**. These are experimental
thresholds selected from these measured tradeoffs, not published quality standards
or accepted sonic constants. Preserve tonal/stereo intent with separate checks:
each band's absolute normalized source deviation may exceed the corrected
control's deviation by at most 0.25 dB (with a 0.5 dB comparison floor); mid/side
change may exceed the control's absolute change by 0.25 dB, and correlation change
by 0.02. Report every individual metric even when these checks pass. The dense
preset and broader original-source results can reject these candidates.

For comparison, the -9 single-rule owner/Funk/Aphelion outputs lose 1.04/1.73/2.43 dB
of section contrast and have paired attack medians -1.22/-4.16/-3.33 dB. Funk still
misses the target by 3.4 LU. This motivates explicit constraint/target conflicts
rather than automatically increasing drive until a target is reached.

Evaluate the simple rule first as a distinct candidate. In the bounded experiment,
select a technically valid, character-qualified target hit when one exists;
otherwise retain the qualified candidate with the smallest absolute target error
and an explicit infeasibility reason. Among equally feasible hits, prefer the
candidate closest to the source-relative single-rule operating drive, avoiding
crest maximization. Keep a corrected, qualified control fallback when character
data or feasible candidates are unavailable. Also report the target-first outcome
without treating it as permission to adopt that tradeoff. The eight-point initial
grid and at most two refinements remain the count budget; qualified-verifier
cost must be reported separately from C0's old-meter candidate times.

These values/rules are an initial **development** version. They must be frozen
with exact transforms and measured runtime budgets before a new holdout is opened.
The owner has not yet answered the automatic-dynamics direction, and production
adoption remains gated on C2 and the remaining B3/control requirements.

## Development, freeze and holdout

The implemented development probe now separates normalized source PCM, its
operating descriptor, automatic drive, preset push, deliberate input trim and
final gain. Normalization uses the maximum absolute sample across all channels,
then measures that normalized PCM's integrated loudness; it is not labeled as
the original source's LUFS. Original-source analysis/guards remain the anchor
for artificial -40/-80 dB copies. This tests the drive rule's consistency given
that anchor, not gain-invariance of arbitrary fresh production analyses.
Every frame's actual limiter reduction is observed, including the drained tail;
the reported distribution aligns with delivered audio after removing warmup.

Initial coarse offsets relative to the single rule are -9/-6/-3/0/+3/+6/+9/+12
dB, the same eight-point C0 budget. Refine an unresolved target/character tradeoff
with at most two local midpoints between the best character-qualified coarse
candidate and its adjacent candidate toward the better target result. Reevaluate
each midpoint; do not infer unsampled monotonicity or discard other candidates.
These proposals refine the development comparison, not shipping bounds. If the
single rule already meets target and every character constraint, the specified
nearness tie-break necessarily selects it; the extra development grid can then
quantify the cost of work an eventual implementation could avoid.

The first owner Universal-75 full-song run has 20 protected float deliveries.
At -14, the single rule meets target with -0.184 dB section-contrast change and
+0.282 dB paired attack-median change. At -9, the single rule meets target but
loses 1.043 dB section contrast; the initial qualified grid choice instead lands
at -10.5397 LUFS with -0.301 dB contrast change. Broader preset/source checks,
midpoint refinements and independent finalist verification remain in progress.
No new holdout has been opened. The numeric limits and owner direction remain
development/adoption gates, not an inferred listening preference.

The expanded complete Coat/Piano development runs add 100 deliveries and their
source-anchored metrics. Coat Universal-50 also selects the single rule at -14;
at -9 the coarse preserving choice misses by 1.646 LU. Piano Universal-75/50
select -3 dB relative drive at -14, missing by 0.804/0.875 LU; at -9 the coarse
choices miss by 6.706/6.778 LU. Every tested Loud-75 group on these two sources
has no candidate satisfying all initial character constraints. The corrected
control fallback is explicit and retains its character failures. These results
reject a blanket claim that the simple rule or this grid satisfies every preset;
they do not justify changing Loud's voicing or widening thresholds to hide misses.
Independent whole-file finalist checks for these additional groups are running.

The application restart interrupted Funk after 31 fully written deliveries.
Those files/report are retained unchanged. The recovery-capable probe verifies
the original job/source hashes, descriptor, estimator version, each retained
WAV hash, typed settings and exact resolved coefficient string, then writes a
new report and only missing deliveries to a fresh directory. The first recovery
attempt exposed JSON f64 comparison rounding of a serialized f32; comparison now
restores the typed settings before checking, with no numeric tolerance added.
Recovery elapsed time is recorded separately; it cannot reconstruct the missing
wall-clock duration of the interrupted session. The remaining five original
sources continue from their existing jobs; no completed research is discarded.
The resumed run has passed those checks and continued after the 31st case.
A deliberately mismatched job hash is rejected before any delivery is written.
The original report remains authoritative for its recorded numeric diagnostics;
JSON parsing/reserialization in a combined recovery report can change an f64
decimal by one rounding step. File hashes and exact coefficient identities are
unchanged, and the original report hash is retained. Four metric regression
tests and strict all-target Clippy pass (`c1-metric-regressions-recovery-v1.log`,
`c1-b3-clippy-recovery-v1.log`). The read-only limiter accessor's rebuilt WASM
source stamp is `46580659ee75`; its stamp test passes. No DSP processing was
changed by this offline-prototype checkpoint.

Develop on the original eight sources and their gain copies at -14/-9 LUFS,
covering Universal 75, Universal 50 and the deliberately dense Loud preset.
Retain the known piano/Rich/Imaginal/drum failures. C0's existing candidate grid
is a starting experiment, not an adopted drive cap or shipping constant.

Use these results to propose numerical attack/section/tone/stereo/limiter limits,
source-relative search bounds and runtime budgets. Keep target-first and
dynamics-preserving outcomes visible while the owner choice is unresolved.
Do not select solely by crest or minimum reduction. Record an explicit reason
when no candidate satisfies constraints, and use a separately verified fallback.

Before opening a new holdout, freeze the exact candidate rules, numeric limits,
fallback, budget, source licenses/hashes/transforms and source-only scoring-window
method. Candidate metadata discovery has identified freely licensed original FLAC
albums, but no holdout audio or candidate outcomes have been examined. Acquire
three or four new pieces covering acoustic, percussive, dense and sectional
material. Record limited creator diversity and source lineage. A failed holdout
stays a failure; later tuning requires a new independent validation set.

This protocol fixes measurement methods before C1 development. The later numeric
freeze is still required; this document does not claim C1/C2 completion or supply
an unanswered sonic-policy decision.
