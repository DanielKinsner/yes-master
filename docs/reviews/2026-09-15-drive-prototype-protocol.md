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
Independent whole-file finalist/control checks now pass 9/9 expanded Coat and
16/16 Piano files. The refined comparisons also pass their applicable checks.

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
The resumed Funk run completed all 60 cases, rendering only the missing 29.
Recovery work took 779.864 seconds; the interrupted total remains unavailable.
A deliberately mismatched job hash is rejected before any delivery is written.
The original report remains authoritative for its recorded numeric diagnostics;
JSON parsing/reserialization in a combined recovery report can change an f64
decimal by one rounding step. File hashes and exact coefficient identities are
unchanged, and the original report hash is retained. Four metric regression
tests and strict all-target Clippy pass (`c1-metric-regressions-recovery-v1.log`,
`c1-b3-clippy-recovery-v1.log`). The read-only limiter accessor's rebuilt WASM
source stamp is `46580659ee75`; its stamp test passes. No DSP processing was
changed by this offline-prototype checkpoint.

## Bounded refinement checkpoint

`refine_drive.py` retains the complete coarse evidence and performs at most two
new midpoints per unresolved preset/target group. It stops for a qualified hit,
no qualified anchor, or no adjacent measured candidate with better target error.
Each new result is measured before choosing the next midpoint; no unsampled
monotonicity is assumed. The combined report retains both selections, every
candidate, native/metric hashes and the extra render/verification cost. The
runner restricts itself to the existing automatic Track control scope and checks
that each fresh control's exact coefficient string matches the coarse control.
Six regression tests cover selection, unavailable data, source anchors and the
adaptive midpoint decisions (`c1-refinement-regressions-v1.log`).

For owner Universal-75 at -9, offsets -1.5 and -0.75 dB are the two refinements.
The first gives **-9.2288 LUFS**, section-contrast change **-0.536 dB**, paired
attack median **-0.477 dB**, and passes all character constraints. The second
hits -9 but fails section contrast. The retained choice remains a research
target miss outside **0.2 LU**; the separate **0.25 LU** product threshold is
unchanged. Compared with the corrected control's -1.531 dB section/ -1.706 dB
attack changes, the choice preserves more measured contrast. Limiter activity
above 0.01 dB falls from 83.0% to 28.5% and p95 reduction from 1.261 to 0.317 dB;
those are explanatory measurements, not a standalone sonic-quality verdict.
All five selected/control checks in `c1-coat-refined-independent-v1` pass; four
recheck existing coarse outputs and one is the new selected refinement.

Owner Universal-50's -0.75 dB refinement meets -9 and all current constraints,
with -0.740 dB section contrast and -0.803 dB paired attack median. Its expanded
refined report passes all nine independent selected/control checks. Piano's
refinements still cannot produce a character-qualified -14 hit; the best -9
results miss by 6.027/6.100 LU for Universal-75/50. All 16 selected/control
independent checks pass; target/character failures remain explicit.
The completed Funk coarse run adds 60 deliveries and **15/15** independent
selected/control checks. Universal-75/50's simple rule meets -14, while their
preserving -9 choices miss by 4.635/4.695 LU. Loud-75's -14 preserving choice
misses by 0.576 LU; its -9 group uses an explicit corrected-control fallback.

Remaining original-source renders now use at most four concurrent render
processes on this 16-core/128 GiB machine. Their timings remain loaded-machine
development observations, not controlled interaction/session budgets. The old
serial coordinator will stop at its existing-output guard after Aphelion because
Metal is owned by a separate worker; that expected controller stop must not be
mistaken for a failed DSP case or cause completed files to be rerendered. The
Metal and Baby have now completed; the remaining coarse queue is Imaginal.
No new holdout is open,
and neither the sonic direction nor shipping limits have been adopted.

Funk's refinements improve the preserving Universal-75/50 -9 misses to
4.527/4.402 LU, still far outside target tolerance. Loud-75 at -14 finds a
qualified hit after one midpoint (-14.080 LUFS). Its refined selected/control
report passes **15/15** independent checks. Metal needs no midpoint: at -14,
Universal-75/50 select +3 dB relative to the simple rule because the single
candidate fails the control-relative highest tonal-band constraint. Its -9
groups and both Loud groups retain explicit control fallbacks. Metal's **14/14**
independent checks pass. Aphelion's coarse **14/14** checks also pass, while its
refinements and Rich/Baby metrics proceed. Passing protection is distinct from
meeting character/target constraints.

The original-source -40/-80 dB jobs now run with the same fixed coarse grid,
three preset groups, original analysis/guards and source anchors. These are
new gain-copy comparisons; no original evidence is replaced. A 25 GiB free-disk
guard stops a queue before starting another job if storage becomes insufficient.

Before evaluating copy outcomes, `compare_gain_copies.py` declares development
numeric consistency budgets: 0.001 LU for raw/delivered loudness, 0.001 dB for
automatic operating drive and normalization after removing copy attenuation,
0.01 dB for anchored section/attack/band/side differences, 0.00001 correlation,
and 0.0001 maximum absolute sample difference. These are reproducibility checks,
not audibility standards or new sonic limits. Required missing metrics fail.
Unnormalized control/attenuate-only responses are reported separately; they are
not expected to be gain invariant. Selected candidate/fallback identity must
also agree. This separates a consistent normalized DSP response from a selector
that might change because its corrected-control comparison changes with level.

The [first gain-copy diagnosis](2026-09-15-drive-gain-consistency.md) now records
Piano/Coat numerical failures and Coat's level-dependent selector limits.
The original eight-source coarse grid is complete. Do not weaken the declared
budgets or treat the fixed-reference diagnostic as an adopted selector.

### Completed original-source development grid

All eight original coarse runs and the bounded midpoint refinements are now
complete: **480 coarse plus 37 refinement deliveries**. Coarse selected/control
reports contain **118 independent checks**; the refined reports add **104**
checks, with Metal's existing 14 reused because no midpoint was justified. All
completed independent reports pass peaks, LUFS comparison and zero full-scale
samples. This is output-protection evidence, separate from feasibility.

Preserving selections have the following signed misses after refinements. A
fallback remains explicitly labeled even when it happens to hit the target.

| Source | Universal-75, -14 / -9 LUFS miss | Universal-50, -14 / -9 LUFS miss | Loud-75 outcome |
| --- | --- | --- | --- |
| Coat | 0 / -0.229 | 0 / 0 | Control fallback at both targets |
| Piano | -0.804 / -6.027 | -0.875 / -6.100 | Control fallback at both targets |
| Funk | 0 / -4.527 | 0 / -4.402 | -14 hit after refinement; -9 fallback |
| Metal | 0 / fallback | 0 / fallback | Control fallback at both targets |
| Aphelion | 0 / -2.664 | 0 / -2.693 | Control fallback at both targets |
| Rich | -5.032 / fallback | -5.133 / fallback | Control fallback at both targets |
| Baby | 0 / -3.042 | 0 / -3.063 | -14 hit at -3 offset; -9 fallback |
| Imaginal | -2.913 / -7.469 | -2.255 / -7.446 | Control fallback at both targets |

No extra grid expansion is justified merely by an unresolved target miss. The
gain-copy work exposes a separate control-reference dependence on input level,
and the [precision correction](2026-09-15-drive-gain-consistency.md) changes
recursive arithmetic without changing coefficients. Original f32 results and
copied executables remain frozen. Corrected-arithmetic selector validation and
the eventual numeric/holdout freeze are still required; these historical C1
tables do not certify a production automatic policy.

### Independent reference storage

`check_drive_finalists.py --stream-padding` builds the same 2,048-frame leading
and trailing zeros in FFmpeg's double-precision filter graph instead of writing
another padded full-song WAV. `verify_reference_padding.py` proves exact padding
bytes and **10 bit-identical SOXR16/64 outputs** across mono/stereo short/edge
signals and the retained two-second stress witness. Both versioned reports are
retained in `reference-stream-padding-v1/v2/`.

Five full owner-song checks also produce identical peak, loudness, file hash and
pass fields to the earlier retained-WAV method (`c1-coat-stream-padding-v1`,
`reference-stream-padding-v2/whole-song-parity.json`). This avoids **939,438,440
bytes** of additional retained padding for that comparison. All old padded
evidence remains intact. This establishes a storage saving, not a controlled
runtime improvement, and keeps the full-file independent verification intact.

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
