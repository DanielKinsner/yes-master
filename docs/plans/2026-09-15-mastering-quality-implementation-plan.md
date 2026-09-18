# Mastering quality: implementation plan with upfront preparation

Date: September 15, 2026. Reviewed checkout: `5426041c` on `main`.
Revision 2, following plan review and Dan's upfront-analysis direction.
Status: **fixes integrated locally; C1 coverage extended, September 18**. Implementation was authorized
September 15 on local `codex/mastering-quality` from `7534f612`. See the
[checkpoint ledger](../reviews/2026-09-15-mastering-quality-implementation-evidence.md).
Sonic-policy adoption and calibration remain gated. This file replaces the
earlier draft as the current plan.

**Current owner sequencing supersedes the execution order below:** Dan approved
preparing and testing the selected fixes, merging that verified result into local
main, then resuming sound-quality research on a separate branch from verified main.
The [integration record](../reviews/2026-09-16-mastering-fixes-integration.md) tracks
that work; the [review guide](../reviews/2026-09-16-mastering-fixes-review.md) retains
the snapshot and limits. Preserve completed work rather than restarting research.
Unfinished sonic experiments are not a new blanket beta gate. The sound-priority
choice was subsequently answered for experiments: prioritize punch and section
contrast, with polished user experience and accurate results rather than routine
explanatory caveats. Shipping numerical/calibration choices remain open. Local
merge approval does not adopt a production sonic policy or
authorize push, release, deployment, spending or private-audio commits.

Local main is now `ab420654`. Research continues on
`codex/mastering-dynamics-research` with the
[frozen single-first checkpoint](../reviews/2026-09-16-single-first-development-protocol.md).
The existing branch/evidence at `03df038a` remain preserved.

The [single-first and bounded follow-up results](../reviews/2026-09-16-single-first-development-results.md)
complete 22 independently checked rows using 16 new files and preserved anchors.
Three of five single candidates meet frozen limits; extra search on the two
flagged sources does not establish a clear replacement for their current control.
The implemented control-inclusive selector now retains qualified current
processing as eligible. C1's final contract, C2 and C3 remain open.

The [broader preset and target-reuse follow-through](../reviews/2026-09-16-preset-and-target-reuse-results.md)
adds 16 broader whole files and four prepared-target files, independently passing
technical delivery checks. The paired existing prepared-measurement API produces
16 bit-identical outputs at 0.256–0.319 s incremental finalization means versus
7.636–8.272 s fresh; one-time preparation is counted separately. This supports
separating the processing choice from final delivery gain and reusing valid
buffer/facts. Broader character failures keep the source/preset contract in
development; no C2 holdout or C3 adoption is marked complete.

The [September 18 extension](../reviews/2026-09-18-universal50-and-metal-results.md)
adds 24 independently verified outputs, joining retained Funk/Rich results into
original-level Universal 50 current/single coverage across all eight sources at
-14/-9. The fixed selector returns four qualified targets, five character-qualified
shortfalls and seven character fallbacks. All eight -9 singles fail a character
constraint. Two additional corrected-chain Metal +3 dB reproductions pass the
unchanged character and target limits at Universal 50/75, with narrow tone
margins. This supports testing bounded search in both directions, separately
from delivery-gain reuse; it does not change the existing selector or adopt a
drive constant. Full corrected-policy quiet-copy coverage, source/preset/control
mapping, runtime/fallback freeze, holdout and production adoption remain open.

## In plain language

Fix the output and start testing smarter preparation alongside those fixes.
Today a loud input can get flattened and then turned down. A quiet input can
miss the target. We want the existing workflow to make a better decision for
each track without asking the owner to operate another control.

**Dan's direction:** current initial analysis feels very quick. A longer initial
preparation step is worthwhile if it buys better audio quality/fidelity and
faster subsequent interaction. Treat this as a design priority, not a reason
to optimize only for the shortest import time. No particular delay, candidate
count or sonic threshold has been chosen.

The working sequence is:

1. **First checkpoints: A, B1, D1 and C0.** Correct SRC buffer/frame handling,
   remove the loudness-dependent ceiling bypass, fix Density Auto's display,
   and measure the value/cost of reusable upfront preparation. These can be
   worked on independently; C0 does not wait for production meter integration.
2. **Develop B2/B3 and the offline C prototype alongside one another.** Qualify
   and integrate final-rate peak protection while testing source-relative drive
   with the corrected research references and isolated corrected SRC. Include
   a simple single-render candidate. Measure quality and time from the outset.
3. **Integrate a successful C approach after the output and control contracts
   are ready.** Reuse prepared source information and matching results, keep
   audition responsive, and verify the final whole-song output. B4 checks actual
   encoded delivery; additional encoding passes require a demonstrated need.
4. **D2 and E remain distinct slices.** Complete backend-resolved readouts;
   separately calibrate saturation continuity/antialiasing and its interaction
   with the selected drive policy.

A/B correctness and D1 do not depend on a preferred mastering sound. C's offline
experiments do not depend on completing every production export integration.
Production adoption still requires trustworthy output constraints. E must not
become an incidental preset retune inside a correctness patch.

The product aim is **prepare the song once, reuse what remains valid while the
owner tweaks, then verify the exact result**. Longer preparation is useful only
when it improves decisions or removes repeated work. Dan's observations of Waves
and LANDR motivate this tradeoff; their waiting times do not establish their
internal algorithms, measured fidelity or the cause of their responsiveness.

## What changed after review

- Moved C's performance/quality feasibility experiment to an explicit first
  checkpoint, rather than leaving early profiling as an optional sentence.
- Removed B2/B3 production completion as a dependency of the offline prototype.
  Its output checks use the corrected research tools with stated scope.
- Added a source-relative single-render baseline and reusable upfront analysis/
  preset preparation. The historical 16/16 result came from a search and does
  not establish that the single-render candidate will succeed.
- Distinguished initial preparation time, time until predicted export LUFS
  settles after an edit, and realtime playback/meter responsiveness.
- Made extra quantization/encoding passes evidence-dependent while retaining
  mandatory PCM protection and truthful decoded-format reporting.
- Made the agent responsible for developing measurable acceptance criteria.
  Dan's unresolved sound preferences do not block the initial experiments.

## Evidence reviewed and baseline established

Read the [authoritative synthesis](../reviews/2026-09-15-mastering-quality-final-synthesis.md)
and [final corrections](../reviews/2026-09-15-mastering-quality-final-verification.md)
first. The [Codex report](../reviews/2026-09-15-mastering-quality-recommendation.md),
[frozen protocol](../reviews/2026-09-15-mastering-quality-protocol.md), and
[Claude archive](../reviews/evidence/2026-09-15-claude-audit/README.md) supply
historical evidence, not independent permission to adopt their proposals.

The conclusions survive review of current code and the specific evidence below:

| Finding | Evidence to retain as a regression | What it does not establish |
| --- | --- | --- |
| SRC copy, delay and frame-count defects | Locked Rubato helper; both corrected loops; 408-condition SRC summary; actual hot-start CD witness | A new filter is needed, or every finite-input edge is sample-exact |
| Ceiling protection is skipped | No-target Funk PCM24 clipping; 50 ms Streaming witness; current early returns and target-only call site | One shared-helper edit covers every output route |
| Peak detector under-reads | Corrected final verification retains all 25 pilot failures, including native-48-kHz Imaginal | A fixed margin or an oversampling factor guarantees the ceiling |
| Drive should respond to source and target | Matched production/attenuate-only/centred results; piano, Rich, Imaginal and Loud/drums regressions | The research grid, its centre, cap or runtime are shipping choices |
| Density readout omits Adapt | Nine coefficient combinations on the owner source and current frontend formulas | The gated Adaptive Compressor is active |
| Saturation has a zero-boundary jump and folded harmonics | Native probes and matched whole-chain saturation/limiter comparison | A gentler isolated curve is a better master |

Concrete raw evidence:

- [SRC results](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/src_probe_summary.txt)
  and [music witness](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/src_music_checks.json).
- [Corrected peak-reference checks](../reviews/evidence/2026-09-15-final-review/verification.json)
  and [fixed-reference SRC checks](../reviews/evidence/2026-09-15-final-review/src-focused.json).
- [Policy comparisons](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/policy_compare.json),
  [drum iteration regression](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/rule_a_drums_iteration.json),
  and [separate stereo check](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/policy_ms_check.json).
- [Resolved Density/Adapt coefficients](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/density_adapt_coeffs_input01.json)
  and [saturation/limiter interaction](../reviews/evidence/2026-09-15-claude-audit/audit-output/reconciliation-20260915/results/sat_interaction_coat_t9.json).

### Fresh checks on this receiving machine

- The checkout was clean; `git diff a4fb621 -- src src-tauri` was empty.
- The original owner files were present. The expanded corpus was absent, so
  the three private draft assets were downloaded and restored using the
  [existing transfer procedure](../reviews/2026-09-15-mastering-quality-transfer.md).
  All 3,543 unique objects and 4,507 restored paths passed their hashes.
  The independent sibling workspace was restored and retained.
- Initial reproduction setup refused source hashes. Investigation established
  that all ten expected modules match historical bytes after CRLF restoration;
  there was no source-code difference. A separate ignored source copy restores
  those bytes only after asserting each exact expected SHA-256. Production files
  and historical manifests were not edited and hash assertions were not relaxed.
- A fresh locked harness build with this machine's Rust **1.98.0** reproduced
  On, Off and On with 257-frame blocks exactly. Historical Rust was 1.95.0;
  this is fresh Windows reproduction with a different compiler, not a claim
  that the historical environment was restored unchanged.

| Render | Fresh whole-WAV SHA-256 |
| --- | --- |
| On | `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce` |
| Off | `d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5` |
| On, 257-frame blocks | `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce` |

Local proof: `test-output/mastering-quality-planning-source-20260915/`
contains `checkout-byte-reconciliation.json` and
`test-output/baseline/reproduction-proof.json`; restore proof is
`test-output/mastering-quality-planning-restore-proof-20260915.json`.
These ignored files/audio do not travel with a Git pull. The private transfer
archives remain the corpus transport route. No production DSP experiment,
installed-app check, new listening verdict or Mac check was performed here.

## Current-code map and design consequences

| Area | Current behavior | Consequence for the plan |
| --- | --- | --- |
| `src-tauri/src/sample_rate.rs` | Calls Rubato 1.0.1 `process_all_into_buffer`; takes its output length | Replace the convenience loop locally while retaining the FFT filter |
| `src-tauri/src/engine.rs` | Track render processes at source rate, flushes the chain, converts to codec delivery rate, meters, then lands only with a target | Separate optional loudness adjustment from output peak protection |
| `engine.rs::measure_and_apply_ceiling_bounded_landing` | Returns immediately for no target; scalar helper returns zero for unusable/gated LUFS | Both call-site and gain-rule changes are necessary |
| `engine.rs::preview_landing_with_cancel` | Whole-file background render; returns unity for no target/unusable LUFS | Preview needs the same policy and separate measurement availability |
| `src-tauri/src/audio.rs` | One active preview worker plus latest pending work, cancellation, source epochs, gain cache and coefficient generations | Reuse lifecycle machinery; extend the result from a scalar gain to a resolved plan only when C is ready |
| `audio.rs::settings_landing_hash` | Deliberately excludes `source_lufs_integrated` and Volume Match | A source-dependent selector needs backend source-analysis identity in its own key; cannot inherit this cache unchanged |
| `src-tauri/src/wav_writer.rs` | Deterministic dither/quantization; measures that PCM using the existing peak detector | Preserve deterministic quantization and replace/qualify peak measurement without using clamping as protection |
| `src-tauri/src/album_render.rs` | Active album route lands each track after rate/channel conversion; continuous WAV copies delivered track PCM and inserts gaps | Check joins as well as individual tracks; preserve per-track/continuous PCM parity and album intent |
| `src-tauri/src/album_encoding.rs`, `export_encoding.rs`, `mp3.rs` | Distinct encoded delivery paths with decoded measurements | Integrate qualified decoded peak checks; track PCM proof cannot certify these paths |
| `AdvancedPanel.tsx`, `fields.tsx` | Density omits the existing `sliderAutoValue` prop; null falls back to the slider minimum | Supply the preset default for display while keeping null persisted |
| `compressor-auto.ts`, `AdvancedPanel.tsx` | Pre-Adapt formulas serve the summary, knob defaults and Manual-mode materialization | Separate display truth from requested/manual initialization behavior |
| `src-tauri/src/exports.rs` | Informational target shortfall begins above 0.25 LU; generic peak advisories use fixed thresholds | Keep advisory ownership; add accurate failure reasons/effective-ceiling context where affected |

The active album implementation is `album_render.rs`; `album.rs` is the planner.
Old comments mentioning an additional album-simple path must not invent a second
implementation queue. Trace actual callers as each shared signature changes.

## A — sample-rate conversion correctness

### A1: frame accounting and explicit draining

Implement a small, fallible loop in the existing SRC module, using locked
`Fft<f32>` and `process_into_buffer`. Prefer this over a dependency-wide upgrade
for the correction: the faulty behavior is in the convenience wrapper, and both
research loops preserve the selected filter. An upstream fix can be evaluated
later against the same regressions.

- Define output length as `ceil(input_frames * output_rate / input_rate)` using
  checked integer arithmetic. Preserve the existing ceil convention exactly,
  without floating-point rounding accidents.
- Consume partial input explicitly; feed zeros until startup delay plus the
  required useful output exists. Remove the delay once and retain the entire
  useful suffix. Check capacity arithmetic and progress; propagate errors.
- Keep empty-input and valid same-rate identity behavior. Make rate/channel/
  interleaving preconditions explicit, including the current same-rate fast
  path's bypass of validation; cover caller expectations before tightening it.
- Keep the existing filter's fractional-delay behavior in this slice. Document
  and test the half-output-sample residual on the two known odd-block pairs.
  A fractional-delay compensator would be a separate filter change requiring
  new evidence, not a prerequisite for removing the corrupt sample.

### A2: production-route proof

Automated cases: the six original rate pairs, supported delivery-rate families,
both 48/96→44.1 odd-block pairs, lengths at block multiples ±1, empty and
nonzero single-frame inputs, short DC/impulses, silence, hot starts, final-frame
impulses and independent stereo channels. Discover the current accepted rate
set from settings/codec resolution rather than assuming three research rates
cover it. Assert exact lengths with an independent oracle.

Keep separate checks for stale-frame error, finite-input edge response and
interior frequency/phase response. Do not fit away the failure or compare a
zero-valued one-frame sine to a nonexistent infinite waveform.

Exercise an actual application render through the affected rate pair, including
the hot-start CD witness. Retain the old control and explain all intended sample/
length changes. Unaffected On/Off hashes remain controls; expected changed
outputs receive new evidence only after explaining the differences.

**Done:** exact counts and no stale sample across the matrix; retained tail;
same-rate identity; production export witness passes; affected Rust and fixture
checks pass. A does not establish peak-detector correctness.

## B — output protection and measurement

### B1: remove the loudness dependency

Use one decision function with optional *usable* loudness measurements/target
and an independently available peak measurement. Proposed rule:

1. Compute the requested loudness gain only when target and measured LUFS are
   usable; otherwise the desired gain is zero.
2. Bound that gain by `ceiling - measured_peak`. This bound can be negative
   even when there is no loudness target.
3. Unknown peak/headroom cannot authorize a boost. A missing integrated-LUFS
   result on valid short audio must not suppress known peak attenuation.
4. Silence is a valid finite-output case. Invalid audio/measurement failure is
   distinct from silence; do not invent a successful reading.

Keep measurement availability explicit internally rather than using a display
floor such as −60/−70 as proof. Preserve public compatibility deliberately at
the serialization boundary. Keep requested LUFS intact and remeasure delivery;
absolute loudness gating means a gain offset alone is not an exact receipt.

B1 can fix the demonstrated bypass using the current detector, but it is only
the **bypass correction**, not completion of the qualified ceiling guarantee.

### B2: qualify the peak estimator, then integrate it

Preferred starting design: a reusable bounded-memory, stateful interpolation
meter for offline/background measurement, carrying filter history across chunks
and explicitly flushing finite-signal boundaries. Compare a reviewed library
implementation with a local polyphase-FIR candidate before selecting one.
Factor, filter bandwidth, length and uncertainty allowance are experiment
outputs. No 8×/16× label or fixed 0.5 dB margin is accepted as proof.

Qualification must include original sample maxima, above-full-scale floating
input, DC, alternating samples, odd/even lengths, frequency/phase sweeps,
bursts, transients, last peaks, finite edges, mono/stereo and chunk invariance.
Compare primary signals and multiple independently constructed references;
include the corrected FFT reference and float-forced SOXR with their bandwidth
differences recorded. Do not mistake agreement between identical code paths
for independent verification.

Use [ITU-R BS.1770-5](https://www.itu.int/rec/R-REC-BS.1770/en) and
[EBU Tech 3341](https://tech.ebu.ch/docs/tech/tech3341.pdf), including phase/burst
cases 21–23, for the qualification specification. The official sources were
checked during planning. Record the actual signals used and scope; synthesized
examples alone do not constitute a full official test-set compliance run.

Before integration, write down estimator error tolerances, finite-edge treatment,
numeric reserve and the independent ceiling acceptance criterion. The requested
ceiling stays unchanged; the historical pilot's ≤−0.9 test for a −1 request is
not a replacement product ceiling. Persist estimator/version provenance in
diagnostics or evidence so a changed number is explainable.

Place the expensive verifier on render/background workers. Keep the realtime
audio callback free of full-file analysis or search. Any change to its limiter
detector/live meter is a separately measured performance/sonic change; the
offline fix must not quietly substitute a new limiter algorithm.

### B3: delivered PCM and every shared route

Wire final-rate float protection into Track export, rendered preview, live
preview planning, album per-track delivery and shared bridge callers. Test
device-rate playback against export-rate measurement explicitly. Current preview
rate resolution does not use the Track encoder's `delivery_rate` call; codec
and device rate differences need coverage, not an assumed parity claim.

Keep deterministic dither and quantization. Measure their actual output using
the qualified meter. First determine whether a justified format/estimator-specific
numeric reserve can satisfy the delivered-PCM criterion in one quantization.
This is not the rejected universal 0.5 dB margin. If demonstrated residual
failures require another pass, calculate a downward correction from the retained
float master and quantize again with the same seed under a bounded procedure.
Derive the reserve and any retry bound from evidence; do not repeatedly attenuate
already quantized PCM. A knowingly over-ceiling PCM file plus a warning is not
completion of this protection fix.

Test PCM16/24/float, target absent/up/down, gated/short/silent input, output gain,
Custom/profile ceilings, source-rate and changed-rate paths, tails and channel
conversion. Source/prior-file protection, cancel/error cleanup and truthful
receipts remain mandatory.

Measure the assembled album programme at butt joins and gaps. A boundary failure
is not yet established by research. If found, reconcile protection of component
tracks and the assembled programme before finalizing either: an independent
continuous-file trim would break the existing PCM-parity contract. Retain arc
targets, per-track overrides and the single continuous encode.

### B4: encoded delivery

Check decoded FLAC/AIFF and MP3/M4A/ADTS AAC/Vorbis independently. Start with
measurement and truthful reporting of actual encoded delivery. Lossless delivery
must retain the protected PCM contract. For lossy output, characterize peak
growth and target misses across the declared codec/rate/bitrate matrix before
adding correction machinery. A format with no demonstrated correction need
does not acquire an extra encode by default.

If lossy failures justify correction, compare measured headroom planning with
a downward, bounded re-encode from retained pre-encode PCM. Never use cumulative
lossy transcoding. Specify exhaustion behavior and cost before integration:
retain an honest advisory/review state for valid but nonconforming delivery;
fail technically invalid output. An advisory-only codec remains explicitly
advisory; do not claim an enforced codec ceiling. Preserve working formats.

**Done for B:** demonstrated witnesses fixed; estimator qualification recorded;
affected final PCM and decoded format tests pass; no missing-target bypass;
preview/export differences explained; remaining valid delivery misses reported
accurately. A production drive policy requires the integrated qualified output
constraint. Earlier offline experiments use the separately checked research
references and cannot claim a production ceiling guarantee.

## C — reusable upfront preparation and source/target-aware drive

### C0: early quality/performance feasibility checkpoint

**Run near the start, alongside A/B1/D1.** The byte-identical baseline is already
established. Timing current behavior can start immediately. Candidate quality
comparisons use isolated corrected SRC and the corrected final-review peak
references, with versioned settings and explicit finite-edge/bandwidth limits.
Recheck their controls on this machine before relying on new measurements.
This does not require B2's production implementation or B3's route integration.

The question is not simply whether we can analyze for longer. It is which
preparation produces better masters or avoids enough repeated processing to
justify its cost. Compare three preparation strategies using the same candidate
policy so preparation benefits are not confused with a changed mastering rule:

| Strategy | Work before normal adjustment | What the experiment must establish |
| --- | --- | --- |
| Existing preparation | Current analysis and caches | Cold/warm reference time, fidelity and repeated work |
| Reusable source preparation | Reuse existing measurements; add only missing section/peak/transient context that a decision actually consumes | Better initial choices or fewer candidate evaluations without hiding difficult passages |
| Source preparation plus current-settings preparation | Prepare an initial drive/result for the selected preset, intensity, target and relevant controls | Whether extra initial work improves the first master and later adjustments enough to retain |

Inspect existing `analysis.rs`, `deep_analysis.rs`, `profile_store.rs` and
`audio.rs` before adding scans. Desktop already has whole-song and ordered
window measurements, cached profiles and decoded audio. Existing information
may be underused; collecting it again is not an improvement. Reusing measurement
data does not enable the gated Adaptive Compressor or Phase-B confidence policy.

Candidate preparation work, each justified by its consumer:

1. **Source facts:** verified source identity, loudness/gate availability,
   channel-aware peak information, tonal/stereo balance and changing dynamics.
2. **A map of useful passages:** loud/quiet active sections, strong attacks,
   transitions, difficult high-frequency passages and file starts/tails. Keep
   their original sample positions. A single whole-song average misses why a
   track is difficult; a section map can guide which candidates to investigate.
3. **Initial processing response:** a bounded set of probes for the currently
   selected settings, recording target error, final-rate peaks, limiter work and
   preservation metrics. This is settings-dependent preparation, not a source
   fact that can be reused for every preset.
4. **Reusable results:** keep valid analyses, measured candidates and the chosen
   result so the same settings do not repeat the entire job. Limit storage and
   worker concurrency; measure whether selective precomputation earns its cost.

Existing deep-window peaks use a mono downmix and long tracks can stride the
window scan. Those values are useful descriptors, not a per-channel maximum or
a guaranteed full-track true-peak scan. Do not reuse them as final protection.

Test carefully chosen passages as a candidate-screening shortcut. Compressors,
filters and limiters carry history: include justified preceding context or
correctly captured state, and compare passage predictions with full-track
renders. Do not splice unrelated excerpts and treat their processing as the
original continuous song. If ranking errors or missed failure cases remain,
expand coverage or use full-song evaluation. Whole-song final verification stays.

#### Reuse and invalidation contract

| Change | Work that can remain valid | Work requiring verification/recalculation |
| --- | --- | --- |
| Same source, replay or A/B switch | Source scan and matching processing result | Playback state only; Volume Match remains separately managed |
| Return to exactly evaluated settings | Source scan and matching candidate/result | Revalidate identity/version and readiness before use |
| Preset, intensity, EQ, Density, Adapt or manual gain edit | Source facts and passage locations | Processing response and final result for the new settings |
| Target change | Source facts; raw candidate audio only if coefficients/stages are unchanged | Drive selection, gain decision and final delivery checks |
| Delivery rate/format change | Source facts; upstream results only where stage dependencies match | Conversion/encoding, peak protection and delivery measurement |
| Source content or analysis/DSP algorithm changes | Only explicitly unaffected data | Invalidate dependent analyses and candidates; never reuse a stale answer |

Treat source preparation and settings preparation as separate caches. Existing
landing-cache hashing strips the injected source LUFS field, so it cannot simply
be repurposed as a source-aware optimizer cache. Prefer backend source identity
and analysis revision. Intermediate audio is worth caching only when its exact
stage dependencies match and the measured memory cost is acceptable.

#### Measure the experience, not only candidate throughput

Use fixed representative short, typical and long tracks and a scripted sequence
of target, preset, intensity, EQ, Density and Adapt edits, plus repeated settings,
rapid dragging, A/B, source/device switching and cancellation. Report:

- Time to usable source playback/basic information.
- Total upfront preparation and time to the first fully checked master.
- Time until predicted export LUFS settles after each kind of edit, including
  cache hits/misses and median/worst or p95 across sufficient repeated trials.
- Actual audio callback/deadline behavior and live-meter updates while work runs.
  A delayed prediction is not automatically an audio stall; both need evidence.
- CPU, peak memory/cache size, candidate count, analysis/render/meter time,
  cancellation delay, and total work over the whole editing session.
- Target/peak correctness and preservation metrics on the same outputs. A faster
  approximation cannot earn a quality pass by avoiding difficult passages.

Dan accepts a longer first preparation step for demonstrated quality or later
interaction benefits. Do not reject an approach solely because initial import
is slower than today. Compare longer-preparation/faster-edit and shorter-
preparation/slower-edit alternatives; keep options that offer a useful measured
tradeoff. No arbitrary maximum delay or promised speedup is adopted now, and
extra preparation does not itself prove that per-sample DSP has become cheaper.

**C0 exit:** a reproducible baseline and comparison report recommending what to
prepare, what to cache, an initial candidate strategy and measured time/memory
budgets for further work. If benefits are absent, identify which added work to
drop. This checkpoint informs C1/C2; it does not choose shipping sonic constants.

### C1: freeze the experiment and resolve automatic intent

Use C0's evidence to choose the prototype architecture. Continue offline work
with the checked research references and isolated SRC correction while B2/B3
develop. Keep the immutable historical control and a control using the same
corrected offline output path as the candidates. When production A/B is ready,
repeat the affected finalists through that path before integration; any detector/
SRC difference can invalidate a selection. Shipping codec claims require B4.

Represent source normalization, automatic operating drive, preset push, manual
input trim and final output gain separately. Do not write the optimizer's
answer into persisted `input_gain_db`, Density, Adapt or the requested target.
Quiet-source normalization is not itself a nonlinear-processing budget: a
gain-attenuated copy must not hit a different arbitrary absolute cap.

Put selection in backend orchestration before chain construction, producing an
immutable internal resolved plan. `ChainCoeffs::from_settings` should remain
deterministic coefficient resolution, not launch a search or render a song.
The plan should identify source/settings, algorithm version, selected drive,
constraints, measured delivery, feasibility and fallback reason.

For automatic mastering, compare:

- Current chain behavior with the same corrected output path as the candidates.
- **A source-relative single-render rule:** derive a candidate input operating
  level from source measurements and requested intent, render it once, then
  perform final peak/loudness checks and scalar landing. The rule/constants are
  experimental. "Single render" still includes analysis and measurement costs;
  it is not evidence that source statistics exactly predict nonlinear processing.
- The attenuate-only first candidate, retaining its known quiet-source and
  Loud/drum limitations as regression cases.
- A bounded source-relative coarse-to-fine search informed by prepared data.

Keep evaluated candidates; do not assume monotonic target response or promise
that three to five candidates suffice. The historical source-relative 16/16
result used candidate search, not the proposed single-render rule. Final
selection needs whole-file verification. Evaluate preparation strategy and
selection strategy separately before combining the promising choices.

Reject technically invalid/over-ceiling candidates first. Evaluate target error,
limiter activity, transients, fixed musical-section contrast, tone and stereo
individually. Keep the existing 0.2 LU research target tolerance for comparison;
do not silently change the product's 0.25 LU informational threshold. Higher
crest or less compressor reduction alone cannot choose the winner.

### C2: regression and genuinely new holdout

The agent owns the acceptance specification. Before development comparisons,
define measurement methods, comparison controls, applicable hard correctness
requirements and what counts as unavailable data. Use development results to
propose numerical processing/character limits and runtime budgets, with their
tradeoffs visible. Freeze those candidate limits, selection rules and fallbacks
before opening the unseen holdout; do not tune them to turn a holdout failure
green. Changing a rule afterward creates a new development iteration and needs
new independent validation.

- Repeat −14/−9 on all original sources and unclipped gain-attenuated copies,
  including very quiet copies and the known piano/Rich/Imaginal/drum failures.
- Add preset/intensity coverage beyond Universal 75, including Universal 50
  and intentionally dense presets, without retuning their calibration.
- Reuse the original source's sample-aligned section windows for its gain
  copies. Report loudness-gate availability separately, so changing eligible
  sections does not masquerade as a processing difference.
- Acquire a small no-cost licensed holdout, proposed three or four new pieces
  covering dynamic acoustic, sparse/percussive, dense and sectional material;
  prefer available lossless sources. Freeze licenses, hashes, transformations,
  scoring windows and decision rules before examining candidate outcomes.
  Constructed stem balances stay labeled. The owner need not supply a library.
- Declare source-relative cost limits and failure/fallback rules before the
  holdout. Limits remain candidate engineering/sonic choices until validated;
  do not adopt −18, +12 dB, the old grid or a universal quality score.
- Report objective fidelity/dynamics indicators individually and inspect why
  they changed. Extra source analysis can improve gain/processing decisions;
  it does not repair SRC defects or automatically remove the saturator's aliasing.
  A claimed improvement in sound cannot be inferred merely from more analysis,
  a higher crest factor or faster execution. E evaluates its own mechanisms.

If no acceptable candidate exists, use a fully verified fallback and retain the
target miss/reason. A corrected control must pass output protection too. Fallback
cannot be selected merely to conceal a failed experiment.

### C3: responsive integration and control compatibility

Production adoption depends on A and B1–B3, the successful C0–C2 evidence and
resolved control behavior. Encoded outputs retain the applicable B4 checks.
An offline prototype passing research references does not bypass these gates.

Extend the existing preview worker/caches to carry the resolved plan and landing
gain together. Preserve one active job plus latest pending work, cancellation,
source epochs, setting generations, A/B reuse and playhead continuity. Key on
backend source identity/analysis revision, relevant requested settings, delivery
rate/encoding where relevant, and algorithm version. Never accept an obsolete
plan because its scalar gain happens to match.

Use the same resolved plan in audition and export. Reuse a winning rendered
buffer only when all relevant inputs match. Recompute Volume Match against the
selected mastered chain; its optional scalar remains audition-only. Pending
work may keep the current safe audition behavior, but cannot be labeled a
settled result for newer settings. Export must obtain the matching final plan.

Fit the extra initial preparation into existing import/preparation and progress
surfaces. No additional required mode, knob or analysis button. The owner can
understand whether the song is being prepared or a changed result is being
calculated, without being shown internal search/filter details. Preserve safe
playback and normal control use where available; edits during preparation cancel
or supersede settings-dependent work without throwing away valid source facts.

Re-run C0's experience benchmarks against the integrated implementation, using
the budgets selected from evidence. This is confirmation of early feasibility,
not the first speed check. No multi-pass search or blocking work in the audio
callback. If a budget fails, optimize the cause, reallocate useful upfront work
or use a verified fallback; do not hide an unresponsive workflow behind a longer
progress animation. A previous settings result is never relabeled as current.

Control-policy recommendation for discussion: automatic drive should belong to
the existing loudness-targeting workflow. Keep Density and Adapt's existing
functions/defaults intact. Derive the automatic base without treating deliberate
manual input trim as a source-level error to cancel. Apply user intent explicitly
and verify final delivery afterward. Test that an intentional manual increase
still affects processing; preserving the saved number alone is insufficient.
Define behavior for Custom/no target, Manual compressor, Off, Adapt 0, album
Follow/Override and existing saved sessions before enabling the selector. The
initial prototype can isolate normal Track automatic use while these policies
are resolved; other working paths retain their existing processing plus A/B
corrections. This is a prototype boundary, not a shipping feature removal.

**Done for C prototype:** reproducible source-level consistency, acceptable
target/character tradeoffs on regressions and unseen material, explicit misses,
measured quality/preparation/interaction tradeoffs, and no hidden changes to
requested controls. Production adoption needs affected bridge/fixture/native
checks and a focused review of any remaining consequential sonic choice. The
agent develops and evaluates the candidates; Dan decides unresolved intentional
voicing preferences from a concrete comparison, rather than inventing DSP
thresholds. Repeating the earlier A–E listening task is not a gate.

## D — Density Auto and honest readouts, independently

### D1: small display correction

Use `NumberField.sliderAutoValue` for named-preset 0.50 and Custom 0.00 while
`compression_density` remains null. Adapt/source changes must not move the
requested thumb. A precise initial summary label such as **Preset compression
before Adapt** can correct the present overclaim without changing coefficient
resolution or Manual-mode initialization.

### D2: backend-resolved display

Prefer a small backend readout from the actual `ChainCoeffs` resolution after
profile/guardrail application. Include per-band threshold, ratio, timing and
makeup where displayed, with source/settings identity and readiness. Do not
reuse the gated Adaptive Compressor plan as proof of active Tier-1 processing.

Keep pre-Adapt/requested values used by `materializeManualCompressor` separate
from display data. A change to how entering Manual seeds values is its own
behavior decision; it must not hitchhike on this readout correction. Stale or
unavailable backend values get an accurate pending/unavailable or pre-Adapt
label, never a falsely current “effective” value.

Test named/Custom Auto, explicit zero/other values, Adapt changes, source
loading/switching, reset, saved sessions, keyboard/pointer operation, Manual/Off
and preset/intensity changes. D1 needs frontend tests/build and
`npm run verify:headless`; D2 also needs affected command/type/bridge checks.
**Done:** honest requested/effective distinction with unchanged DSP and persisted
intent. D1 need not wait for D2 or C.

## E — saturation continuity and antialiasing calibration

Keep a separate experiment with the current positive curve as control. Compare
continuity near zero and antialiasing choices separately before combining them.
Evaluate gain-normalized isolated harmonics/aliasing and matched whole-chain
delivery, including redistribution into the limiter, tonal balance, transient/
section behavior, target feasibility, stereo, latency and CPU.

Compare retaining the current positive curve with oversampling against a
continuous mapping candidate; a more complex antialiasing method needs evidence
to justify its implementation cost. Retain exact filter/settings descriptions.
Re-evaluate the chosen C policy with any E candidate because the loudness/drive
response changes. Numeric curve and preset migration choices require focused
calibration/listening evidence. Do not change the gated `TBD-CALIBRATION`
constants or enable Adaptive Compressor, confidence or album character.

## Checkpoints and dependencies

Each row is a coherent checkpoint; split further only for independently working
changes. An experiment completion is not production adoption.

| Checkpoint | Depends on | Main exit evidence |
| --- | --- | --- |
| A1–A2: SRC correction | Fresh baseline | Exact frame/edge matrix and actual export witness |
| B1: ceiling bypass correction | Baseline; A for converted witnesses | No-target/short-input failures fixed, coverage of shared callers |
| D1: Auto thumb and precise labeling | No DSP dependency | Frontend/build/headless, saved settings unchanged |
| C0: early preparation/quality/speed experiment | Baseline; checked research references and isolated corrected SRC for candidate comparisons | Initial wait, first master, edit-settling time, audio deadlines, quality and reuse tradeoffs |
| B2: estimator qualification | Valid reference tools | Declared accuracy/edge specification and independent tests |
| B3: PCM/preview/album integration | A, B1, B2 | Final PCM, tail/join, receipt and native consistency checks |
| B4: encoded delivery | B3 | Independent decode/reporting first; correction only where evidence justifies it |
| C1–C2: offline drive prototype | C0; checked offline output path, not B2/B3 production completion | Simple-rule/search comparison, preparation strategy, frozen limits, regressions and unseen holdout |
| C3: responsive product integration | A, B1–B3, successful C0–C2, control decisions; B4 for affected codecs | Shared prepared/resolved plan, early budgets confirmed natively, saved/manual intent |
| D2: resolved readout | Backend readout contract | Coefficient/readout parity and affected bridges |
| E: calibrated saturation candidate | A/B; compare against selected C behavior | Whole-chain evidence, runtime and focused voicing decision |

Apply the [testing scope matrix](../TESTING.md). Rust checkpoints use focused
regressions, formatting, strict Clippy and the affected suite. DSP/export/
audition integration requires the slow fixture lane using an existing verified
source, plus native evidence where tests cannot establish behavior. Shared
behavior/types require affected iPhone check/tests and Android host/API-29 checks;
this is compatibility validation, not mobile product expansion. Run the explicit
ignored Track/Album format matrices when encoding paths change, with the
documented independent decoder. Keep native Windows and Mac evidence separate.

The finished browser demo is not a redesign queue. Shared DSP changes may still
invalidate its source stamp; perform the necessary local WASM rebuild/contract
checks if affected, without reopening demo design or deploying it. Record
exact-commit CI only when actually run; no push/release/deployment is authorized
by this planning request. Source files and previous exports remain protected.

## Decisions for discussion

September 16: the owner deferred the C/E sound decisions until the local fixes
are reviewed and approved for main integration. The proposals below are retained
for later research; do not reopen the preference questionnaire during this pause.

**Recorded direction — upfront preparation:** Dan considers current analysis
very quick and accepts a longer initial analysis/preparation step when it yields
audio quality/fidelity or later performance gains. Explore and quantify that
tradeoff early. No need to ask again whether extra initial preparation is allowed.
This does not select a sonic curve, authorize paid services/audio uploads, or
choose an unlimited runtime budget.

1. **Automatic dynamics budget:** recommended direction is to stop short of a
   loudness target when further processing exceeds validated character limits,
   and report the shortfall. The alternative is more target-first automatic
   processing within the peak ceiling. The owner was asked this concrete
   tradeoff during planning; it remains pending unless answered in-session.
   It does not block C0/C1: evaluate both target attainment and preservation
   tradeoffs on development material, then bring a specific recommendation.
   Numerical limits must be tested, not guessed from a broad preference.
2. **Existing-control mapping for C:** resolve a precise behavior table before
   adoption, especially manual input gain and Adapt 0. The proposed separation
   above preserves deliberate manual drive and current Density/Adapt meanings;
   it is not yet an adopted new automatic policy.
3. **Saturation character/migration:** choose from measured whole-chain
   candidates later. There is no useful curve decision to make today.

Engineering choices (SRC loop details, qualified meter, caching, candidate
budget) should be resolved by implementation evidence. They do not require
the owner to pick filter lengths or repeat a broad questionnaire. The first
implementation work, once requested, starts with A/B1/D1 and C0 as independent
checkpoints. B's confirmed failures and C's initial preparation experiments
can progress without resolving C/E sonic choices.
