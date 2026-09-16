# Single-first development and bounded follow-up results

## Outcome

The existing normalized single candidate meets the frozen C1 target and character
limits on **Funk, Aphelion and Baby**. It does not meet every limit on Metal or
Rich. Three additional lower-drive candidates on each flagged source do not
produce a clear replacement for current processing under those limits. This is
useful development evidence, not approval to ship an automatic selection rule.

Local main remains `ab420654`. Research is on `codex/mastering-dynamics-research`;
production application/DSP sources are unchanged from main. The original branch
at `03df038a`, sources, failures and frozen evidence remain preserved.

The [single-first protocol](2026-09-16-single-first-development-protocol.md) was
frozen at `694cdcfe`; the [flagged follow-up](2026-09-16-flagged-drive-followup-protocol.md)
at `37d5851d`. All use Universal 75, Custom -14 LUFS, -1 dBTP, 48 kHz stereo float,
the corrected production chain/SRC/finalizer, existing source/settings hashes and
unchanged C1 measurement anchors/limits. Coat control/-9 and single/-14 are exact
reproduction anchors, not additional policy cases.

## Five-source comparison

Attack losses below are medians of paired source-anchored attack/body differences;
section losses compare the same fixed quiet/loud source windows. These metrics
are indicators of processing changes, not listening scores.

| Source | Current / single LUFS | What the single candidate changes | Frozen result |
| --- | --- | --- | --- |
| Funk | -14.000 / -14.000 | Median attack loss 3.138→1.927 dB; section loss 1.048→0.479 dB | Pass |
| Aphelion | -13.999 / -14.000 | Median attack loss 3.050→1.412 dB; section loss 2.176→0.594 dB | Pass |
| Baby | -16.124 / -14.000 | Reaches target; median attack loss rises 0.241→1.245 dB, still within limits | Pass; a loudness/dynamics tradeoff, not improvement in every metric |
| Metal | -14.000 / -14.000 | Median attack loss 4.690→0.182 dB; highest-band change increases | Fails tone band 8 |
| Rich | -19.998 / -14.000 | Reaches target, but section loss rises 0.426→3.895 dB and six tone bands fail | Fails character limits |

Metal's source-relative 8–16 kHz band difference is +1.430 dB for the single
candidate versus +1.088 dB for current processing. It exceeds the frozen
1.338 dB allowance by **0.092 dB**. This small margin is not proof of an audible
defect; neither is its larger attack improvement permission to erase the flag.
Additional -3/-6/-12 dB input offsets retain that tone failure and land at
-15.367/-16.111/-16.737 LUFS. Lower drive does not improve every dimension.

Rich's extra -3/-6/-12 candidates land at -15.071/-17.665/-21.134 LUFS. Only -12
passes all character limits. Current processing already passes them at -19.998.
The frozen candidate-only preserving rule selects -12, but current processing
is 1.136 LU closer to target. The gentler candidate reduces section loss further
(0.426→0.195 dB), so this is a tradeoff, not strict dominance on every metric.
**Recommendation:** include the corrected control among eligible results in the
next frozen selector. Do not automatically spend more time to choose a quieter
candidate when the current result already meets the chosen character contract.
Metal's control also fails attack limits; fallback must remain labeled as a
fallback in research evidence, never silently reported as full qualification.

## Actual work and reuse

Audio evaluation includes chain processing, SRC and qualified finalization.

| Source | Current + single | Additional three candidates | Total observed evaluation |
| --- | --- | --- | --- |
| Funk | 24.126 s | Not run | 24.126 s |
| Aphelion | 53.491 s | Not run | 53.491 s |
| Baby | 40.004 s | Not run | 40.004 s |
| Metal | 31.272 s | 34.932 s | 66.204 s |
| Rich | 23.576 s | 39.775 s | 63.351 s |
| **Five-source total** | **172.468 s** | **74.707 s** | **247.175 s** |

The current tone/stereo limits require a matched current-processing control.
Thus initial qualification costs **control plus candidate**, not one render total.
Reuse that control only for matching source, nonlinear settings and algorithm.
Eliminating its cost requires separately validated source/preset constraints.

Source analysis/measurement anchors are reused from the completed C1 reports;
their cost is excluded, not counted as a newly measured speedup. PCM decode and
normalization occur once per source per job (0.098–0.212 s for the five-source
first job). The follow-up reuses four complete verified outputs and their metrics
without re-rendering. Two Coat anchors are freshly rendered, compared bit for bit,
then reuse their preserved files and independent proof. No old WAV is overwritten.

| Separate measurement | First job, including Coat reproduction | Follow-up |
| --- | --- | --- |
| Offline native child wall / CPU | 201.854 / 198.547 s | 77.141 / 75.656 s |
| Native child peak working set | 752 MiB | 431 MiB |
| Fresh character-metric work | 36.239 s | 13.231 s |
| Fresh independent reference work | 390.969 s | 170.643 s |
| New complete WAVs | 10 | 6 |

Reference/metric work ran separately and overlapped other work. These are observed
phase costs, not an end-to-end wall-clock sum, isolated before/after comparison,
all-source-grid speedup, source import time or product settings-change latency.
The [integration record](2026-09-16-mastering-fixes-integration.md) separately
records initial preparation, settings settling, playback response and session cost.

## Verification and exact evidence

- First job: **12 complete rows pass**, comprising ten fresh files and two exact
  Coat reproduction anchors. Follow-up: **10 complete rows pass**, comprising
  six fresh files and four reused verified controls/candidates.
- All complete files pass independent SOXR16/64 peak and LUFS checks, exact
  frame/rate/channel/hash checks, finite PCM and zero full-scale sample checks.
  Qualified native peaks meet -1 dBTP. Character failures remain separate.
- Strict example Clippy, release builds, formatting and Python compilation pass.
  Application code is unchanged, so the already completed main integration
  suites apply; no additional app, installed or listening pass is invented.
- The first follow-up attempt stopped before rendering: comparing parsed JSON
  f64 decimal values to reserialized f32 settings rejected identical DSP inputs.
  `dafd8191` compares both sides at the exact typed DSP field precision and still
  asserts coefficient strings/source/output hashes. V2 succeeds. V1 failure,
  binary and job remain preserved; no audio limit changed.

The [joined verification](evidence/2026-09-16-dynamics-research/verification.json)
binds jobs, native reports, metrics, references, copied executables and protocols
by SHA-256. `summarize_single_first.py` recomputes character comparisons and
requires matching complete independently passing files before writing that record.
Raw artifacts remain ignored under `test-output/dynamics-research-20260916/`;
they do not travel with Git. Builds preceded the 25 GiB reserve/output preflights.
No private audio was committed.

## Owner direction and next work

On September 16 Dan chose the recommended **dynamics-preserving direction for
experiments**, accepting investigation of target shortfalls when reaching the
target would flatten punch/section contrast. He wants polished results without
routine explanatory caveats in the user experience. Keep research diagnostics
in internal evidence; preserve accurate meters/results and existing deliberate
strong-processing controls. This is experimental direction, not approval to ship
a particular threshold, selector, preset change or user-interface redesign.

Next develop/freeze the selector with current-control eligibility and explicit
technical fallback evidence, then the required broader target/preset/intensity
regressions and unseen licensed holdout. Reuse completed research. Do not relax
Metal's frozen limit retroactively or open holdout outcomes before the next
contract is fixed. C1's final numerical/runtime/control contract, C2 and C3 remain
open. B3 dynamic-boundary/final-result UI work and release gates remain distinct.

### Follow-through: current-control eligibility verified

The [new frozen selector](2026-09-16-control-inclusive-selector-protocol.md),
implemented at `c04a6d53`, passes 11 focused regressions and reselects the same
verified whole files without rendering new audio. Its immutable internal result
records source/intent context, algorithm, file/drive identity, actual delivery,
character qualification, target feasibility and rejection/fallback reasons.

The [selection evidence](evidence/2026-09-16-dynamics-research/selector-v1.json)
chooses the passing single candidates for Funk/Aphelion/Baby, current processing
for Rich (character qualified, target shortfall), and current processing for
Metal as an explicit character fallback. A fallback that reaches the target is
not mislabeled as a dynamics/tone pass. Over-ceiling/unverified results, unavailable
character data and mixed contexts are covered. Candidate order cannot alter the
result. This closes the known Rich selection regression; it does not solve
Metal's sonic tradeoff, validate other preset/target combinations or complete C2.
