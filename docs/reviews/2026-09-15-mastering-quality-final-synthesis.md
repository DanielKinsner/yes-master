# Mastering quality: final synthesis and implementation handoff

September 15, 2026 UTC / September 14 evening Pacific. **Authoritative conclusion
of this investigation**, combining Codex's pilot, Claude Fable's sealed blind
assessment, its unblinded replication, and the coordinator's final verification.
Read this before planning implementation. Older reports remain historical evidence;
their conflicting statements are resolved below rather than silently rewritten.

## 1. Plain-English answer

**YES sometimes works harder on the music than it needs to.** On several already
loud inputs it pushes into saturation and limiting, then turns the result down
to the requested loudness. Turning it down cannot restore the peaks already
flattened. Quiet inputs present the opposite problem: the current processing can
leave them short of their target. One fixed trim or “less compression everywhere”
does not solve both problems.

The strongest direction is to make the existing automatic process account for
the incoming track, requested loudness, and available peak headroom together.
Use the least processing that achieves an acceptable result, and report a target
miss honestly when reaching it would exceed validated limits. “Least processing”
must include effects on tone, transients and musical sections, not just one
compressor's gain-reduction meter or the largest possible crest factor.

**This is primarily behind the existing interface.** No new user-facing mode,
knob or analysis step is proposed as a requirement. Density remains the requested
compression amount; Adapt remains source adaptation. An existing misleading
readout and Auto thumb should be corrected. The exact automatic algorithm and
sonic calibration are proposals for implementation work, not adopted defaults.

Before that policy is integrated, correct two objective weaknesses: the
sample-rate conversion buffer handling, and peak protection of the final output.
These are independently supported failures, not an owner taste preference.

## 2. Status, authorization and things to preserve

- This turn was authorized to finish research, consolidate evidence, prepare a
  cross-machine handoff, and **push it on main**. It does not implement production
  DSP changes, adopt a preset retune, publish a release, or deploy anything.
- Production source baseline is `a4fb621d88a95b8af549467fb499943acb4274d5`.
  The three previous local commits (`85f5740`, `55e8ed8`, `2c71e35`) contain
  research and review preparation. Final handoff commits add documentation,
  archived text evidence and isolated research/restore tools only.
- Keep named-preset Density 0.50, Adapt 0.50, all preset voicings and current
  saved-setting meanings during planning. Keep `TBD-CALIBRATION`, Adaptive
  Compressor, Phase-B confidence and album-character gates unchanged.
- Preserve responsive audition, playhead-preserving Original/Mastered switching,
  optional audition-only Volume Match, source protection and collision-safe export.
- September 9 installed Windows listening passed its stated baseline. Do not
  repeat that questionnaire or call it missing. It does not certify future changes.
- The owner found A–E too subtle for a useful choice. That is neither equivalence
  nor a listening win. Do not repeat it as a prerequisite, ask for a large library,
  or stop at “subjective.” An agent can acquire targeted licensed material itself.
- Browser demo is finished. Pricing, publication and the deferred desktop refactor
  are outside this task. Do not turn tomorrow's plan into those workstreams.

## 3. Ranked proposals

| Priority | Proposed outcome | Evidence / confidence | Tradeoff or remaining uncertainty |
| --- | --- | --- | --- |
| **1A — objective correction** | Fix SRC delay removal, suffix copying and exact frame counting. Preserve the selected conversion filter unless separately justified. | Two independently written prototypes; 408 conditions each with exact lengths. Stale samples on 48→44.1 and 96→44.1, short-input loss and floating-length errors. **High** for defects. | Prototype is not production integration. Fractional delay, edges, empty/short input and all supported rate/channel paths need explicit contracts. |
| **1B — objective correction** | Enforce effective peak ceiling on the final-rate floating PCM even without a loudness target or usable integrated loudness; measure and report delivery honestly. Qualify a stronger peak estimator. | Actual no-target PCM24 clips; 50 ms export exceeds ceiling; 25 pilot files exceed the declared independent criterion. **High** for failures; **medium** for a complete replacement design. | Correct protection can lower delivered loudness. Different reconstruction filters disagree; an oversampling number alone is not a proof. Lossy delivery needs separate decoded checks. |
| **2 — automatic policy prototype** | Source/target-aware, bounded whole-chain drive selection behind current controls. Account for final-rate peaks and processing costs. | Source-centred exploratory rule reproduces target/level consistency at −14 across 16 conditions in Claude's matched replication. **Medium-high** for this mechanism, **medium** for general quality. | Quiet/sparse material loses dynamics when pushed to loud targets; runtime and limits not solved. No new holdout remained after exploration. |
| **3 — existing UI accuracy** | Auto thumb represents requested density; readout reflects backend-resolved processing or clearly labels its earlier stage. | Actual Adapt coefficients differ from the frontend's “Effective compression” summary. **High** for mismatch. | Needs affected bridge/UI checks; do not infer enabled gated Adaptive Compressor behavior from diagnostic fields. |
| **4 — sonic calibration research** | Design continuous saturation amount behavior and compare antialiasing alternatives while preserving whole-chain character. | Reproduced zero-boundary gain jump and folded components. **High** for mechanisms, **low-medium** for a replacement choice. | A gentler curve shifted work into the limiter and reduced dynamics at matched loudness. CPU, latency, tone and preset compatibility need evaluation. |

**Do not adopt:** global −3 dB trim; globally lower Density; Adapt 100% everywhere;
compressor/saturation/limiter bypass as default; halved saturation coefficients;
the tested gentle curve as an automatic improvement; a fixed 0.5 dB peak margin
as a proven correction; a 13-pass audition search; or an arbitrary +12 dB input
trim cap. None is established as a universal solution.

## 4. Evidence that supports those conclusions

### Reproduction and independence

The actual owner On and Off exports were reproduced byte for byte through the
established isolated Rust harness. Instrumented controls and a 257-frame block
rerun retained the same identity. Claude independently used the real production
crate with default desktop features disabled, mirrored backend adaptation, and
reproduced On. The final coordinator pass again rendered On through Claude's
production-path harness and obtained the same whole-WAV hash:

| File | SHA-256 |
| --- | --- |
| Owner source | `0b39d396856d38c0d927986e0bb45058e7a92c378dac4bcf27ddcf1552e651c9` |
| On / Universal 75 | `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce` |
| Off | `d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5` |

Claude's original assessment was blind to previous conclusions, with a
coordinator-selected corpus and existing source comments available. Its later
replication was explicitly **unblinded**. This is not a formal double-blind
study, independent population sample, or two votes that settle a claim. The
18 original sealed files and 20 reconciliation files were freshly hash-verified.
They remain unedited in the [Claude archive](evidence/2026-09-15-claude-audit/README.md).

### Corpus and scope

Eight pieces: the owner source; four finished MP3 downloads by Scott Buckley
and Kevin MacLeod; two constructed mixes from MP3 stems; one constructed mix
from FLAC stems. Thus **six additional sources have a lossy input lineage**, not
“two lossy sources.” The three stem sums are unmastered constructed balances,
not artist-approved premasters. Four finished tracks from two creators do not
establish broad genre coverage. Gain-shifted copies are robustness conditions,
not additional songs. Credits, licenses, URLs, transformations and hashes are
in the [corpus manifest](evidence/2026-09-15-mastering-quality/corpus.json).

Codex's frozen pilot includes 384 raw variants and 192 delivered finalists,
plus separate robustness, mechanism and exploratory studies. Claude's targeted
policy comparison adds 221 grid points and 96 production finalists covering
16 source/level conditions × two targets × three policies. Reuse of the same
music increases replication confidence, not population coverage. The original
development/holdout split was respected for the frozen pilot; later centred
selection is exploratory because those sources had already been observed.

The matched main comparison is Universal Intensity **75%**, requested Density
50%, Adapt 50%, neutral manual controls, Custom target −9 or −14 LUFS, −1 dBTP,
source-rate processing, 48 kHz PCM24 delivery and Volume Match off. Claude's
blind broad matrix used Universal **50%**; do not merge its numbers without
checking settings. Only limited 50% checks supplement Codex's 75% study. No
claim covers all preset intensities, platforms or codecs.

### Unnecessary drive and target-dependent costs

On the owner fixture the current chain reaches approximately −7.37 LUFS before
turning down to −9, or still further for −14. Earlier stage ablations showed
that saturation and limiting account for much of its reduced peak contrast;
removing compression alone does not account for the whole difference. Ablation
effects interact and are not additive percentages of blame.

At matched −14 delivery, less input drive retained more peak/section contrast
on five owner/finished sources, with RMS-matched broad-band differences below
about 0.65 dB in that comparison. At −9, the owner example's selected −1.5 dB
trim raised crest from 8.95 to 9.87 dB, LRA from 3.1 to 3.6 LU, and reduced mean
limiter reduction from 0.43 to 0.16 dB. This supports avoiding needless drive
on those conditions, not a perceptual superiority claim or a fixed trim policy.

The exploratory source-centred rule uses centre `−18 − source LUFS`, grid
offsets −12…+6 dB in 1.5 dB steps, the existing ±24 dB input clamp, and lowest
drive within 0.2 LU of target; otherwise minimum error with a lower-drive tie.
It selects by target feasibility, not maximum crest. **−18 and the grid are
research settings, not proposed shipping constants.**

- At −14, Codex's 12 and Claude's expanded 16 conditions met the 0.2 LU target
  criterion and the tested independent ≤−0.9 peak criterion. Same-song crest
  varied by only 0.000–0.012 dB across tested input attenuations. A ceiling
  criterion of ≤−0.9 for a −1 request was the pilot measurement tolerance;
  it is not permission to redefine the product ceiling.
- Claude's attenuate-only rule reached −14 in 7/16, the same target count as
  production, because it did nothing to quiet target-missed sources. It did
  improve several hot-source dynamics measures. Its Loud/drum regression
  undershot to −14.91; two attempted correction iterations reached −14.48 and
  −14.24, still outside the 0.2 LU criterion. A simple one-shot rule is not solved.
- At −9, the centred rule reached the loudness criterion in 13/16, with three
  independent peak failures; very quiet Rich/Imaginal copies hit the input clamp.
  Across these conditions mean LRA fell 1.81 LU and section contrast 1.95 dB
  relative to production; Rich's worst section loss was 7.12 dB. These means
  combine target attainment changes and repeated songs; they are not a quality
  score or an audible damage rate.
- Piano is a clear regression/tradeoff witness. At −9, the centred grid used
  +3.56 dB drive: crest 12.59→10.29 dB, LRA 10.3→8.4 LU, section contrast −1.85 dB.
  Codex's earlier +4.5 dB result was a different, absolute grid, not a disagreement.

The policy should therefore treat quiet-source gain normalization separately
from effective nonlinear drive. An absolute +12 dB trim cap, suggested in the
reconciliation, is **untested** and would make an otherwise identical attenuated
input hit a different bound. Prototype source-relative processing limits and
explicit feasibility/cost checks before selecting a cap. Do not silently change
the requested target or forbid the owner's intentional bold settings.

### Sample-rate conversion: objective and separate from peak reconstruction

YES calls locked Rubato 1.0.1's whole-buffer helper. Its delay-removal copy uses
the delay as the copy length rather than the useful suffix. With an odd output
block, one stale frame survives: frame 955 for 48→44.1 kHz and frame 514 for
96→44.1 kHz. A 0.5-peak 100 Hz sine shows about 0.430 error in the former; a
hot-start music/CD export shows a 0.057 sample error, about −24.9 dBFS. This is
an error amplitude, **not a listening audibility verdict**.

Short conversions also retain startup delay, and floating arithmetic can add
an output frame. Claude's matrix has **408 conditions × three implementations
= 1,224 output files**. Production gives 368/408 exact lengths; each independent
corrected loop gives 408/408. Both preserve the underlying Rubato filter. The
matrix spans six rate pairs, including the two faulty odd-block pairs.

Do not overstate the prototype: Claude's prose says all long residuals are
−128…−137 dBc; its raw summary actually has worst corrected values around
−61…−73 dBc with a 2 ms edge guard. Best interior cases reach approximately
−127…−135 dBc. Normal finite-signal filter edges and different guards explain
why these summaries must not be conflated. Its “all short RMS 0.94–1.00” claim
also overgeneralizes; single-frame sine inputs can be exactly zero, and other
short ratios extend below 0.94. Validate expected finite-input response, not an
infinite sine where the input contains no signal. The affected odd-block pairs
retain approximately half a sample of fractional delay; choose and document
alignment requirements rather than calling that exact sample alignment or inaudible.

### Peak protection: what is wrong, and what the resampler example means

Three separate observations support priority 1B:

1. **Reachable no-target failure.** Custom with no LUFS target skips final
   landing, including ceiling attenuation. Funk at Universal 50, 44.1→48 kHz,
   reaches +0.485 dBFS before PCM quantization; four samples exceed full scale.
   PCM24 clamps them at 0 dBFS. The receipt flags a high peak but cannot repair
   the clipped samples. A 50 ms Streaming file likewise skips attenuation when
   integrated loudness is unavailable and exceeds its −1 dBTP ceiling.
2. **Meter under-read without SRC.** All 24 Imaginal pilot variants plus one
   Funk candidate fail the independent ≤−0.9 criterion. Imaginal is native 48 kHz,
   so its discrepancy does not require SRC. Feeding different block sizes does
   not change the native result. Final verification must not simply reuse an
   insufficient detector and call the agreement proof.
3. **Pre-existing intersample peaks exposed by SRC.** The Funk Universal 75
   pre-SRC file measures about −1.10 using the native meter, but approximately
   +1.66 dB on the corrected 16× finite-block band-limited reference. The wider
   passband conversion produces +1.043 dBFS samples; its 16× reference is about
   +1.09. Near-Nyquist content and detector/filter bandwidth explain much of
   the difference. Claude's >21 kHz ablation supports that account on this file.
   It is not proof that Rubato numerically manufactures 2 dB of extra energy.

Changing SRC filters could change those peaks and the audio bandwidth, but is
not the primary correctness fix. Peak behavior still depends on reconstruction
filter and finite-signal treatment. Do not call a software interpolation result
an exact physical DAC peak or infer audible damage from near-Nyquist stress alone.

The final pass found two **reference-tool** problems, preserved in a separate
[verification addendum](2026-09-15-mastering-quality-final-verification.md):

- Claude's FFT upsampling failed to split the even-length input Nyquist bin.
  A coherent alternating-sample control exposes a factor-of-two error. Correcting
  that, preserving original sample values, checking SciPy and extending file
  edges does not remove the critical music failures. The function name
  `fftexact` was too strong: finite-block reconstruction is not an oracle.
- Both investigations' FFmpeg/SOXR command could negotiate integer samples and
  clip an oversampled peak at 0 dBFS. Force float processing/output. The +3 dBTP
  synthetic control is a reference-command problem, not a SOXR clipping defect.

Claude withdrew its blind 0.5 dB margin recommendation after testing the broader
set. Do not promote 8×/16× labels or single-sine grid-error bounds to a general
maximum-error guarantee for a complete FIR meter. Qualify passband, phases,
transients, edges and above-full-scale floating input against primary reference
signals and multiple implementations. The current standards entry is
[ITU-R BS.1770-5](https://www.itu.int/rec/R-REC-BS.1770/en); −4 is superseded.
[EBU Tech 3341](https://tech.ebu.ch/docs/tech/tech3341.pdf) gives meter tests and
tolerances, not a guarantee that this application satisfies every delivery ceiling.
Claude synthesized examples corresponding to tests 15–19; that is **not a full
official-test-set compliance run**. Include phase/burst tests 21–23 in qualification.

### Saturation, limiting, Density and Adapt

Current positive saturation coefficients process the whole signal through
`tanh(x * (1 + 2a)) / tanh(1 + 2a)`. The coefficient is not a wet percentage.
Zero bypasses; an arbitrarily small positive value adds about 2.357 dB of
small-signal gain, and the owner setting `a=0.0715` adds about 2.922 dB in the
low-level probe. Halving the coefficient does not halve its effect or approach
identity continuously. This mapping is mechanically real; replacing the
established positive curve changes preset tone and level.

Native high-frequency saturation folds harmonics into the audible-frequency
range. On the full-scale 11 kHz / 44.1 kHz probe a component is about −21.64 dBc.
Oversampled references reduce it substantially. Codex's Kaiser-5 reference
around −80 dBc and Claude's stronger-filter reference around −114 dBc have
different rejection floors; this is not conflicting evidence or a universal
limit of the curve. These extreme tones are mechanism probes, not prevalence
or listening evidence on typical music.

The gentle continuous candidate shifts work elsewhere: on the owner −9 case,
limiter maximum/mean reduction changes 2.46/0.43→6.60/0.97 dB and LRA 3.1→2.6 LU.
Do not declare it better because isolated THD or saturation looks lower. Keep
limiting as a useful peak-control stage; optimize the upstream gain allocation
and the complete chain rather than deleting it.

Adapt already eases the compressor: owner low-band maximum reduction falls
3.05→1.25→0.30 dB for Adapt 0/50/100%. It does not directly coordinate preset
input drive and saturation. Density expresses requested compression; Adapt
responds to source properties across eligible stages. They are distinct useful
functions. Retaining their numbers is a stability decision during investigation,
not proof that 50% is universally ideal.

For the existing UI, Auto means a null requested value resolved to the preset
default (currently 50% for named presets, 0% for Custom). The editable thumb
should represent that requested value. Adapt must not move it or overwrite an
explicit request. The “Effective compression” row currently mirrors pre-adaptation
formulas; on the owner case it displays roughly −12.5 dB/1.4 where resolved
low-band values are about −8.46 dB/1.305, with different makeup. Use backend
truth or precise labeling. Diagnostic plans for gated Adaptive Compressor are
not proof that that feature is active.

### LANDR: useful reference, bounded inference

Saved same-source LANDR High masters measure about −9.4…−9.5 LUFS, LRA 3.8…3.9,
crest 11.07…11.64 dB, and approximately −0.3 dBTP. Reproduced YES On is −9.0 LUFS,
LRA 3.1, crest 8.95, with a −1 ceiling request. LANDR demonstrates a plausible
alternative balance for this source; it does not prove its sound is preferable
or identify which algorithms produced it. The different peaks and loudness
must remain explicit. Scalar volume matching cannot simultaneously equalize
both target LUFS and changed peak ceilings. Do not add a limiter to a comparison
and call the resulting difference an unchanged service output.

These are saved files with unknown service dates/versions, not today's complete
LANDR product or multiple independent songs. No new service account, payment
or music upload is required for local progress. Longer LANDR/Waves analysis is
not itself evidence of superior quality. See the original
[same-source comparison](2026-09-14-yes-export-vs-landr.md) and
[stage investigation](2026-09-14-dynamics-stage-investigation.md).

## 5. Tomorrow's planning boundaries and acceptance evidence

Plan small, independently reviewable slices. Choose implementation details after
reading the current checkout, not by copying either prototype without review.

### Slice A: SRC correction

Start at `src-tauri/src/sample_rate.rs`, locked Rubato helper semantics, Codex
`src_fixed.rs`, and Claude `recon.rs`. Compare upstream/local-loop options without
an opportunistic dependency/filter change. Assert integer-rational output lengths,
explicit delay removal and tail flushing. Cover both odd-block pairs, the other
four tested pairs, all supported delivery rates, block boundaries ±1, silence,
nonzero one-frame/short signals, impulses, hot starts, stereo independence, and
same-rate identity. Separate steady-state reference error from edge response and
fractional alignment. Include an actual export witness, not only a copied helper.

### Slice B: final-rate ceiling protection and truthful measurement

Start at `src-tauri/src/engine.rs` (landing helpers and export),
`src-tauri/src/wav_writer.rs`, `src-tauri/src/album_render.rs`, preview and
album/export callers, and existing delivery tests. First map every route; the
research has not tested every route. Separate optional LUFS targeting from
mandatory effective ceiling protection. A missing/gated loudness measurement
must not suppress a valid peak-based downward trim. Measure floating PCM after
SRC before quantization; account for dither/quantization and remeasure delivery.
No upward gain without sufficient measurement/headroom. Silence stays finite.

Qualify the estimator separately before relying on it in selection, limiter,
receipt and warnings. Decide where a more expensive estimator belongs; do not
automatically put a slow full-file FFT or 16× implementation in the audio callback.
Check no-target, short, target-up/down, already-over-ceiling, rate change and
same-rate witnesses in mono/stereo and PCM16/24/float. Include tail/last-peak and
block-boundary tests. Check affected lossless and lossy exports with independent
decoding; codec peak growth is a separate delivery constraint, and any margin or
bounded re-encode policy must be demonstrated. Do not claim PCM tests prove it.

Review preview/export consistency, Volume Match isolation, warnings/receipts,
shared bridge behavior and batch/album implications. Protect originals/prior
renders. A safety attenuation may cause a reported loudness miss; masking the
miss or re-driving until a meter agrees is not an acceptable fix.

### Slice C: bounded source/target-aware selection prototype

Build on corrected SRC and qualified final-rate peak constraints. Preserve
control renders and all gates. Retain source-level invariance when a source is
attenuated without clipping. Explicitly evaluate target error, peak feasibility,
limiter activity, section contrast, transients, tonal/stereo drift and preset
character. Do not combine these into an unvalidated universal quality score.
Test −14 and −9, very quiet copies, near-Nyquist Funk, sparse Rich/Imaginal,
dynamic piano, dense metal and deterministic drums. Acquire a small genuinely
unseen licensed holdout before claiming generalization; the owner need not supply it.

Compare an attenuate-only first candidate, centred/coarse-to-fine alternatives,
and the control. Define fallback behavior and input-relative cost limits before
examining holdout results. A target miss should remain visible/advisory under
existing product contracts; intentional bold manual processing stays available.
Do not expose research constants or extra knobs simply because the prototype uses them.

Claude measured about 7.4 seconds per grid point and 4–18 seconds per production
render on this Windows/i9 machine. A serial 13-point grid is around 100 seconds.
The proposed 3–5-point approach is **not implemented, timed or proven sufficient**;
even multiplying the measured point cost gives roughly 22–37 seconds. Prototype
cached analysis, deterministic reuse, cancellation, stale-result handling and
bounded background work. Prove native audition responsiveness separately. Slow
competitor analysis does not justify blocking this app's audio workflow.

### Slice D: current-control accuracy; Slice E: saturation calibration

The Auto/readout correction can be planned independently, provided it does not
change DSP or persisted requested settings. Start at `AdvancedPanel.tsx`,
`src/lib/compressor-auto.ts`, backend coefficient resolution and bridge types.
Test null Auto, named/Custom defaults, explicit Density, Adapt changes, reset,
loading saved sessions and source readiness. Rendered changes require
`verify:headless`; real backend readouts need affected contract/bridge tests.

Saturation alternatives require gain-normalized isolated tests **and** matched
whole-chain delivery. Include limiter redistribution, harmonic/alias levels,
tone, transients, target attainment, runtime/latency and preset migration risks.
The curve/voicing decision remains open; do not activate it as incidental cleanup
inside either objective bug fix. Targeted listening may later distinguish a
specific calibration choice, but repeating A–E is not the engineering gate.

For implementation, follow the current [scope matrix](../TESTING.md): focused
regressions, affected Rust/bridge checks and the fixture lane before DSP/export
integration, plus native evidence for audio deadlines and targeted listening
where sound changes. Documentation-only research is not an installed build pass,
Mac verification, current CI green status or release activation.

## 6. Where everything lives

1. **Start tomorrow:** [fresh-agent prompt](../prompts/2026-09-15-mastering-quality-planning-handoff.md).
2. **Final verification:** [coordinator addendum](2026-09-15-mastering-quality-final-verification.md).
3. **Portable data:** [September 15 transfer guide](2026-09-15-mastering-quality-transfer.md),
   supplement manifest, archive integrity and actual restore evidence.
4. **Original Codex evidence:** [report](2026-09-15-mastering-quality-recommendation.md),
   [frozen protocol](2026-09-15-mastering-quality-protocol.md),
   [CSV/JSON evidence](evidence/2026-09-15-mastering-quality/),
   [portable reproduction package](../../scripts/research/mastering-quality-20260915/README.md).
5. **Claude:** [archive index](evidence/2026-09-15-claude-audit/README.md), sealed
   blind report, protocol, unblinded reconciliation, raw measurements, scripts,
   settings, manifests and locked harness source.
6. **Private owner/LANDR archive:** original
   [September 14 transfer guide](2026-09-14-research-handoff.md#restore).

The independent review workspace was a source-only sibling folder, **not a
second Git repository**. Keep it for now. Claude had already pruned many of its
own intermediate WAVs and recorded hashes; this final pass regenerated five
critical files and matched those hashes. The supplement preserves reproducible
sources, all small text evidence, small SRC/synthetic probes and selected music
witnesses, not every multi-gigabyte render/cache. Bulk derivable files remain
locally where they existed. No deletion of the review folder or existing work
was performed by this final pass. A verified archive supports later cleanup;
it does not mean every local file has been copied or authorize deleting unique data.

## 7. Conclusions to carry forward

**High confidence:** concrete SRC failures; final-ceiling bypass on no-target/
short files; detector/reference disagreements; positive saturation's gain mapping;
distinct Density/Adapt functions; unnecessary upstream work on demonstrated hot
inputs. **Moderate confidence:** source-relative, target-aware drive is a better
automatic-policy direction than one global trim or compressor retune.
**Still open:** the shipping selector, processing-cost boundaries, qualified meter,
saturation curve/antialiasing design, perceptual preferences and all-platform proof.

The next agent should turn this evidence into an implementation plan with
bounded acceptance criteria. It should not restart the investigation, choose
undemonstrated constants on our behalf, or treat unfinished calibration as a
reason to postpone the confirmed correctness fixes.
