# Mastering quality implementation evidence

Started September 15, 2026 on local `codex/mastering-quality`, from `7534f612`.
The starting checkout was clean and the revised plan matched that commit.
No push, main integration, publication, spending or private-audio commit is authorized.

## Checkpoint ledger

| Checkpoint | Status | Evidence and limits |
| --- | --- | --- |
| A1 | Complete locally | Same Rubato FFT filter, explicit partial input/draining, checked integer frame counts. 1,690 rate/length conditions pass; finite-tail and fixed-phase stale-frame checks pass. Strict Clippy and desktop library/integration/fixture suite pass. |
| A2 | Complete locally | Synthetic and historical private hot-start CD exports pass exact counts and continuous-reference checks. Corrected frame 955 differs only by expected PCM16 dither. Desktop fixtures and affected bridge lanes pass. |
| B1 | Complete locally | Shared optional-LUFS decision and raw measurement availability; 12 no-target/short-audio PCM16/24/float regressions pass. The subsequent B3 finalizer also closes the independent Funk/50 ms peak failures. |
| D1 | Complete | Named Auto 0.50 / Custom 0.00, saved null unchanged; summary identifies pre-Adapt values. Build, all 38 app headless checks, landing checks and 892 frontend tests passed. Fresh Advanced capture inspected: thumb/readout/summary visible without overlap. |
| D2 | Implemented; checkpoint verification passed | Actual guarded coefficients, request-associated source/settings/analysis identity, accurate fallback, unchanged Manual seeding. 972 settings configurations / 2,916 band comparisons; 33 guardrail tests; 894 frontend tests, build/headless, wire golden and bridges pass. |
| C0 | Complete as an initial feasibility checkpoint | Sixty owner/Funk/Aphelion deliveries pass both preserved whole-file references; source-relative single render versus bounded search, three-trial short/typical/long reuse costs, native callback/lifecycle and passage-state limits are recorded. Recommendations below; no sonic policy adopted. |
| B2 | Qualified library core implemented | Combined finite-sinc interval and measured SOXR response pass 481 explicit finite-zero-extension cases and six whole-file witnesses. Real FFTs reduce owner/long verifier cost about 39% in three isolated trials. The replayable library core matches all 487 standalone results exactly; its exact-dither reader matches written PCM16/24/float. Reference-scope discrepancies are retained and independently explained. |
| B3 | In progress | Qualified finalizer is used by Track/rendered preview, background preview planning and album per-track PCM. All five original exports pass independent finite SOXR16/64 checks; owner On/Off hashes remain exact. Whole-file residual reuse passes 1,095 independent checks, with fresh-meter fallback for excessive uncertainty. Desktop library/integration/private-fixture suite passes. Album programme joins and device-rate/live-toggle contracts remain open. |
| B4, C1–C3, E | Pending | Existing revised dependencies and adoption/calibration gates remain. |

The subsequent desktop library/integration run (`d2-rust-fixtures.log`) also
passed with all four restored private-fixture tests enabled. Strict all-target
Clippy passed (`d2-clippy.log`). The opt-in native callback probe is excluded
from ordinary test counts and needs its own reported run.

The current B3 mathematical reuse proof, exact-output checks, format-specific
reserve and limitations are recorded in
[linear peak verification](2026-09-15-linear-peak-verification.md).
`b3-rust-fixtures-v2.log` passes the desktop library/integration suite with all
four existing private-fixture tests actually run. The first run retained an
obsolete receipt-to-ebur128 equality failure; the corrected tests use the declared
meter on saved PCM, retain their original tolerances, and do not replace the
independent reconstruction checks. No new listening/installed/Mac verdict.

### First verification details

- D1: `test-output/mastering-quality-implementation-20260915/d1-headless-verified/`
  and `frontend.log`. Landing capture manifest refreshed by the existing command.
  Existing Manual materialization and preset coefficients are unchanged.
- Full frontend testing exposed an older beta-duration assertion. The recorded
  September 14 owner decision is eight weeks from launch; `afaf0a41` corrects
  that assertion only. Public beta copy and release state are untouched.
- Fresh application On/Off exports under `c0-coat-v1/` match respectively
  `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce` and
  `d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5`.
- A/B1 first full test command encountered Windows executable locking because
  Cargo attempted to relink the running C0 example. The library/integration
  suite passed separately; subsequent probes run a copied executable.
  This is a harness/build conflict, not an audio result.
- `cargo test --target-dir target/codex-rc --lib --tests` passed with
  `AMS_RUN_REAL_FIXTURE=1` and the original owner source selected explicitly;
  all four private-fixture contracts ran. See `rust-fixtures-tests.log`.
  Strict all-target Clippy passed after the A/B1 changes. No exact-commit CI,
  installed-app or new listening verdict is implied.
- Rust 1.98 formatting reordered two guardrails imports. That mechanical change
  triggered the existing demo stamp; the real WASM rebuild passed. No DSP
  coefficient or gate changed. It remains separate from browser/native proof.

### A2/B1 historical witnesses

`ab-witnesses-v1/comparison.json` records the current real application export,
preserved historical settings/measurements, hashes and independent checks.
The probe executable was copied out of Cargo's output before running; only
timing/measurement scripts and code enter git. Pruned hot-start inputs were
recreated from the restored Funk source with the recorded SOXR precision-28
48 kHz conversion and 60–120 second trim, in the fresh output directory.

- CD: 2,646,000 frames at 44.1 kHz, PCM16. Frame 955 is now
  `[-0.02447509765625, -0.04803466796875]`, rather than the historical stale
  `[-0.00830078125, 0.00909423828125]`. Maximum error against the continuous
  zero-padded reference times a single landing gain is `4.55722e-5`, within
  1.6 PCM16 steps. Existing Rubato fractional delay remains unchanged.
- No-target Funk PCM24/float: 9,669,904 frames; uniform attenuation is applied.
  The native receipt reads approximately −1 dBTP; checked FFT16 reads
  −0.71259 dBTP. This proves the bypass fix and exposes the remaining detector
  error; it is **not** a qualified −1 dBTP delivery pass.
- The existing 50 ms witness delivers all 2,400 frames in PCM24/float with
  uniform attenuation despite unavailable integrated LUFS. The native meter
  reads approximately −1 dBTP, checked FFT16 −0.79136 dBTP. B2 remains required.
- Cancellation during the new measurement pass returns the existing cancelled
  render-job result; invalid audio still returns an error.

### D2 readout evidence

`guardrails::readout_for` resolves `ChainCoeffs` once and presents actual band
threshold/ratio/makeup and envelope times inverted from the 48 kHz canonical
coefficients. Timing conversion is display-only; it never re-enters DSP. The
separate gated Adaptive Compressor plan is not the Tier-1 display authority.
The hook retains the request's source/settings/analysis/mode identity with its
response and rejects stale data during render, before effect cleanup as well.
Preset/density defaults used to enter Manual have not changed.

Verification: `d2-wire.log`, `d2-guardrails.log`, `d2-frontend.log` (89 files,
894 tests), `d2-headless.log` (landing plus 38 app cases), real WASM build and
`d2-iphone.log` / `d2-android.log`. Headless uses the preview bridge and does not
certify installed UI or native audio. Refreshed deterministic captures retain
the accurate pre-Adapt fallback where the preview backend has no resolved data.

### B2 first candidate rejection

`qualify_fir.py` rechecked 12 corrected FFT controls and compared five finite
Hann-windowed, per-phase DC-normalized sinc filters on 478 synthetic cases.
Support/factor pairs were 48/4, 64/8, 128/16, 512/16 and 2048/16. Support is in
**input samples** (FIR length = support × factor + 1), not the existing library's
48 total taps. This initial experiment did not replace the ebur128 0.1.10 filter;
the later qualified B3 finalizer is recorded above.

Every candidate underread some finite alternating/near-Nyquist cases. Even
2048/16 was 2.05275 dB below the checked FFT32 reference on alternating-4095
and 0.1674 dB below on a faded 0.499-fs tone. No constant reserve or larger
oversampling factor is accepted as a correction. Full results remain in
`b2-fir-exploration-v1.json`; meter qualification and finite-edge/bandwidth
specification continue. Synthesized EBU-like cases are not an official
compliance-test-set verdict.

## C0 development protocol, version 1

This is an offline feasibility experiment, not adoption of a new sonic policy.
Retain the historical controls unchanged. Use the restored eight-source corpus
and source hashes from the existing research manifest. Use fresh output under
`test-output/mastering-quality-implementation-20260915/`; never regenerate into
the frozen research directories. Full-song candidate outputs are retained locally.

Before comparisons, recheck the corrected FFT interpolator on odd/even DC,
alternating and random controls, and compare selected float-forced SOXR outputs.
The FFT reference retains its documented bandwidth/finite-edge limitations.
Candidate success requires finite samples, exact frame/channel identity and
the requested ceiling on both checked references (numerical tolerance 0.002 dB).
The historical 0.2 LU comparison tolerance and product 0.25 LU advisory stay distinct.
No composite fidelity score or shipping character limit is chosen in C0.

Separate preparation from selection:

- Time current deep analysis and decoded-source availability independently.
- Derive passage indices from existing ordered deep windows, accounting for
  stride. Record starts/tails, loud/quiet activity, attacks and high-frequency
  passages. This map is descriptive and cannot certify channel peaks.
- Compare repeated full render/measure with reuse of matching upstream PCM and
  measurements, then moving initial settings preparation before the first edit.
  Use the same candidate policy in those preparation comparisons.
- Keep at most two upstream PCM results. Count bytes, hits/misses, rendered
  samples and measured samples; do not describe an old scalar cache as new work.
- For target-only reuse, assert identical complete chain coefficients and
  delivery rate before using retained raw audio. Whole-file final measurements
  remain required. Source identity, analysis revision and algorithm version
  belong in a production key; this single-source experiment is not that cache.

Compare these development candidates: corrected current processing; a one-render
rule that subtracts measured source LUFS, preset gain and the saturator's
small-signal gain from the target; the historical attenuate-only first candidate;
and a bounded search around that source-relative rule. Search offsets are
−9, −6, −3, 0, +3, +6, +9, +12 dB, with at most two ±1.5 dB refinements.
These are an initial development search extent, not input-trim caps or shipping
constants. Normalize source and apply operating drive separately in temporary
chain coefficients, leaving saved manual gain/Density/Adapt unchanged.
Select feasible lowest-drive candidates; preserve explicit target misses.

Report target error, per-channel peaks, crest, LRA, source-anchored section
contrast, tone and stereo separately. Do not infer preferred sound from crest.
Retain all evaluated points and measured stage costs. Passage screening must
be compared with continuous full-file output and adequate preceding context;
do not select a passage shortcut on cost alone.

Experience measurements must separate initial preparation, first checked master,
edit settling, repeated settings, playback/meter response and total session work.
An offline block-throughput probe cannot establish native callback deadlines.
Native availability/results will be explicitly recorded, including requested
versus granted device blocks, cancellation, source/device switches and A/B.
No unseen holdout is opened in C0; C2 rules/limits must be frozen first.

### C0 owner-source quality comparison

`c0-coat-checked-v1/comparison.json` retains 20 final float deliveries, all finite,
with exact frame/channel identity and both checked FFT16/float SOXR16 peaks at
or below the requested −1 dBTP (0.002 dB numerical tolerance). The immutable
application On/Off controls remain byte-identical. These experimental deliveries
use a separate input multiplier and independent landing; no saved setting changes.

| Candidate | Target / delivered LUFS | Crest dB | LRA LU | Source-anchored section contrast change dB |
| --- | --- | --- | --- | --- |
| Source | — / −12.3 | 12.72 | 4.1 | 0 |
| Current processing | −14 / −14 | 8.95 | 3.1 | −1.53 |
| Single source-relative render | −14 / −14 | 13.81 | 4.0 | −0.18 |
| Attenuate-only | −14 / −14 | 14.06 | 4.1 | −0.17 |
| Current processing | −9 / −9 | 8.95 | 3.1 | −1.53 |
| Single source-relative render | −9 / −9 | 9.64 | 3.5 | −1.04 |
| Bounded candidate, extra drive −4.16 dB | −9 / −10.5 | 12.04 | 4.0 | −0.30 |

LUFS/LRA here are independent FFmpeg readings rounded to 0.1 LU. These numbers
show a target/preservation tradeoff, not preferred sound. At −14 every initial
grid point is feasible and its lowest-drive point lies on the search boundary;
that is not an established optimum. Crest can exceed the source, so maximizing
crest alone is not a fidelity objective. Broader cases and frozen C1/C2 limits
remain necessary. The PNG/SVG under `c0-coat-figure-v1/` plots all retained points.

### C0 controlled preparation/reuse timings

`c0-timing-summary-v2.json` summarizes three sequential trials per strategy for
60-second Funk hot-start, 244.56-second owner, and 422.25-second Aphelion. All
builds and other research processes initiated by this task were stopped before
the timing sequence. Strategy order rotates; OS scheduling/thermal variation
remains visible. Aphelion is previously examined development evidence here,
not a new unseen holdout. The summary asserts identical final simulation
measurements for matching settings across strategies/trials.

| Measurement (seconds, median unless stated) | 60 s | Owner 244.56 s | Aphelion 422.25 s |
| --- | ---: | ---: | ---: |
| Hash + decode + deep analysis (one preparation) | 0.98 | 3.73 | 6.64 |
| Actual preview API first result | 0.96 | 4.34 | 7.25 |
| Actual preview API target edit | 0.99 | 4.49 | 7.37 |
| Re-render + final check target edit | 1.08 | 4.80 | 7.78 |
| Matching raw cache + final check target edit | 0.088 | 0.366 | 0.659 |
| Initial settings moved into preparation | 1.08 | 5.13 | 9.03 |
| First final check after that preparation | 0.087 | 0.387 | 0.747 |
| Session: re-render + final checks | 7.68 | 35.59 | 60.87 |
| Session: raw cache + final checks | 6.86 | 33.39 | 58.53 |
| Session: first result prepared upfront | 8.48 | 31.81 | 60.14 |
| Maximum two-result raw cache, MiB | 43.95 | 179.12 | 309.27 |

Session values include initial settings preparation and the ten-edit sequence;
they exclude common source preparation. Existing scalar hits on return/replay
are already nearly immediate and must not be credited as a new cache benefit.
All strategies that add final checks retain that whole-file work. Their current
library peak meter is provisional; B2's eventual qualified verifier changes the
absolute budget. Detailed per-edit median/worst values are in the summary.

The isolated child processes peaked at 121.33/447.98/768.09 MiB working set,
including source, transient delivery allocations and cache. Whole-probe wall
times 95.68/439.29/717.13 seconds include **all twelve sessions**, source analysis
and two actual exports; they are not a user's one-session wait. Owner On/Off
hashes again match the frozen controls in this separate run.

**Recommendation for C1:** retain immutable source analysis/window descriptors,
prepare the currently selected settings once, preserve existing scalar-result
reuse, and evaluate a byte-capped cache of matching pre-landing audio. Target-only
reuse is substantial; indiscriminate preset precomputation has no demonstrated
benefit. Two full files cost 309 MiB on the long case and are an experiment size,
not an unbounded production allocation policy. New EQ/Density/Adapt/preset
settings still require nonlinear processing; source preparation does not erase
that cost. Use the single-rule candidate as the low-cost quality baseline and
retain bounded full-song search for difficult target/preservation tradeoffs.

### C0 completion and initial C1 budget

The completed `c0-funk-checked-v2/` and `c0-aphelion-checked-v2/` comparisons each
add 20 independently checked deliveries, including per-channel FFT/SOXR/sample
maxima. Together with the owner comparison, all 60 retained outputs pass the
preserved requested-ceiling check. No new unseen holdout was used.

At −14 LUFS the single rule reaches the target on all three sources; source-
anchored contrast changes are −0.18/−0.48/−0.59 dB for owner/Funk/Aphelion versus
−1.53/−1.05/−2.18 dB for current processing. At −9 it reaches the target on owner
and Aphelion, but Funk delivers −12.4 LUFS. No initial bounded-grid Funk candidate
reaches −9; the highest drive delivers −11.1 LUFS with −4.23 dB contrast change.
This is a preserved infeasibility/character tradeoff, not grounds to widen the
grid automatically until any target is reached. The Aphelion single rule at −9
also loses more contrast (−2.43 dB) than its −14 result. Broader C1/C2 regression
cases and explicit preservation criteria remain necessary.

Raw rendering/old-meter costs for all 19 candidates were 123/81/181 seconds for
owner/Funk/Aphelion; individual single candidates took about 4.9–5.4/3.8–4.0/
8.4–9.3 seconds. These quality-run timings overlapped other experiments and are
descriptive, not the controlled comparison above. The independent references
add substantial whole-file analysis cost. A production candidate budget must be
remeasured with B2's verifier; a search count alone is not a responsiveness budget.

Proceed to C1 with a single-rule baseline, a maximum eight-point initial grid
and at most two refinements, full-song finalist checks and explicit infeasibility.
Do not adopt passage screening yet: preceding context improves local agreement,
but ranking/missed-case qualification remains open. Source facts and a byte-capped
matching raw cache are the recommended preparation work. First-result preparation
can move useful work upfront; broad preset precomputation is not supported.

The owner was asked the focused automatic-dynamics direction with concrete
owner-track evidence; no answer is recorded yet. Equal-loudness local review
files under `c0-owner-level-matched-v1/` compare current processing and the lower-
drive candidate at −14.0 LUFS. No listening verdict or sonic adoption is inferred.
Continue independent output protection and both offline tradeoff measurements.

### B2 subsequent qualification

See [peak reconstruction qualification](2026-09-15-peak-reconstruction-qualification.md)
for the exact finite-file models, failures, independent direct-sum/zero-padding
witnesses and current result. The combined standalone native candidate passes
481/481 finite-zero-extension cases and 6/6 owner/export witnesses against SOXR
at 16/32/64×, retaining old direct FFT/SOXR differences separately. The full-band
interval widths on the six witnesses are 0.042–0.049 dB. Nine separately synthesized
Tech 3341 phase/burst cases 15–23 also pass their published response tolerances.

The official EBU v5 download returned HTTP 403; no official sequences were acquired
and no full official-test-set compliance claim is made. The earlier four exploratory
`ebu20plus` signals inserted their amplitude change at nonzero phase; those remain
valid stress inputs but are not the new zero-crossing EBU-like construction.

Actual raw witness peaks still fail the requested −1 ceiling under the new meter:
Funk approximately −0.669 and 50 ms approximately −0.748 dB. This is expected while
B3 was unimplemented at that checkpoint, supplying specific protection regressions.
Those final-PCM witnesses are now fixed by the B3 work above. The then-current
standalone meter costs 24–37 seconds on owner material including its FIR response
under concurrent research load. Optimize and isolate its preparation/per-file cost
before production integration; no realtime callback change is authorized by a
numerical qualification result alone.

### Native playback and cancellation

`native-callback-coat-v1.json` exercised real CPAL/production MasteringSource,
meters and spectrum while a whole-song preview worker ran and 120 EQ changes
arrived. Processing was muted only after DSP and meter work. The default Realtek
stereo output ran at 48 kHz: requested 256 frames, **granted 480–1056**. Across
608 callbacks, median/p95/max processing was 0.202/0.396/0.598 ms, with no observed
processing-deadline overrun, device error or exhausted source. First peak update
arrived after 10.35 ms; 111 of 120 snapshots had changed LUFS. The first worker
result took 5.19 seconds; cancellation-to-join took 1.92 ms. An independent FFT
research process was also active. This is evidence under concurrent load, not
a fixed-256-block pass or a listening verdict.

`native-lifecycle-coat-v3.json` then exercised the real AudioPlayer API and native
output, with a test-only muted sink: source playback/first meter 255 ms; first
master result 4.56 seconds while playhead advanced; ordinary settings changes
4.32–6.39 seconds; returning to initial settings within the 60 ms observation
interval; 30 rapid EQ changes settled after 9.29 seconds. Original/Mastered A/B
took 18.4/19.7 ms and preserved position. Named-device/default-device reopening
took 458/27 ms and stopped playback, as the existing product contract specifies.
Switching to the restored Funk source took 85.7 ms; stopping pending preparation
took 10.5 ms. These are snapshot-observed times, not exact worker-completion times.

Failed lifecycle runs v1/v2 are retained. V1 incorrectly expected a loaded stream
after device selection despite the documented unload/stop behavior. V2 could read
an old snapshot as a ready master or fail a cache hit before observing playhead
progress. The corrected probe waits for the new playback generation and actual
progress. No production behavior or timing threshold was relaxed. The mute helper
is compiled only for tests; it never changes OS volume or device preferences.
No installed UI, Mac, listening or release proof is implied by either probe.

### Context before selected passages

`c0-passages-coat-v1/comparison.json` compares six source-selected passages across
three candidate policies with 0, 2 and 10 seconds of preceding context: 54 cases.
The source grid is aligned to the rational SRC phase, and retained continuous
full-file raw outputs are the comparator. With zero context, maximum local
sample error was 0.5357 even though integrated loudness errors were below 0.009 LU.
Two seconds reduced the worst sample error to 0.000561; ten seconds to 0.000547.
Neither established sample equivalence. The remaining state/SRC/block contribution
is unresolved; provisional library peaks here compare state, not final protection.
Passage ranking and missed-peak rates are not yet qualified. Keep full-song
evaluation; an excerpt shortcut cannot be selected from small loudness errors.

### B2 FFT reconstruction scope findings

`b2-fft-exploration-v1.json` and `b2-fft-bound-v1.json` retain failed FFT candidates.
Increasing overlap/factor changes finite-periodic reconstruction near file edges.
The provisional 16×/262144-core/32768-guard variant included exterior ringout and
a mathematical between-grid bound for each periodic FFT polynomial. It still
read 1.3545 dB below the preserved reference on alternating-262143. The grid bound
does not cover truncation or periodic-extension differences; no candidate is
adopted and no fixed reserve is used to conceal this failure.

A separately constructed direct finite cardinal-sinc sum also exposed reference
scope differences: a two-frame random case peaks before the first sample, which
the preserved reference's nominal-file crop excludes. Its 1.3761 dB direct peak
compares with 0.9894 dB in that reference. Long coherent near-Nyquist cases need
further independent reconstruction checks. Historical references/results remain
unchanged and their limitations remain explicit.
