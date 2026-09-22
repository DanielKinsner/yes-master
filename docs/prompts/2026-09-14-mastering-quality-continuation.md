# Continue the YES Master sound-quality investigation

Updated September 14, 2026, after the owner reported that the five listening
clips were not different enough to support a useful naked-ear choice. This is
the current continuation request; it supersedes a handoff that merely asks the
owner to rank A–E. The existing measured results remain valid.

## Prompt for a fresh agent

Continue our YES Master investigation and take ownership of reaching a
defensible recommendation. I am a new developer, I have limited music available
for testing, and I cannot confidently judge the subtle differences in the five
clips the previous agent produced. Do not make progress depend on me supplying
a larger library or repeatedly ranking those same clips. I need help deciding
what processing is preferable and why, including an honest recommendation when
the available evidence is imperfect.

We are trying to resolve whether YES Master applies unnecessary compression or
peak reduction, especially through the interaction of preset density, Adapt
strength, input drive, saturation and limiting. I am also concerned that LANDR
and Waves take much longer to analyze a track and may be making better automatic
processing decisions. Use LANDR as a credible empirical benchmark, while keeping
its undocumented proprietary internals and any inference clearly distinguished.
Longer processing time alone is not evidence of better mastering.

The phrase "more dynamic range is not automatically better" must not become a
reason to stop at "it's subjective." Define useful engineering goals, measure
them, and recommend the most supportable option. Preserve the intended preset
character and requested loudness while investigating avoidable peak suppression,
unwanted distortion, instability and poor behavior across different inputs.
More crest factor alone, or closer agreement with one LANDR file alone, is not
a universal quality score.

### Read and restore the existing work

1. Inspect the checkout/branch and preserve local edits. Work from current
   `main`; do not assume the previous machine's path or its last commit is still
   the current head. The live browser demo is already integrated. Do not reopen
   website work, pricing work or the deferred desktop refactor during this task.
2. Read `AGENTS.md`, the latest relevant entries in
   `docs/OPEN_THREADS_AND_DECISIONS.md`, and
   `docs/reviews/2026-09-14-research-handoff.md`.
3. Read `docs/reviews/2026-09-14-dynamics-stage-investigation.md`, then the
   relevant sections of `docs/reviews/2026-09-14-yes-export-vs-landr.md` and
   `docs/reviews/2026-09-14-landr-waves-dsp-assessment.md`. Use their evidence;
   do not reconstruct the investigation from assumptions.
4. If `tests for presets/` or the diagnostic files are absent, retrieve the
   existing archive yourself using the transfer guide. I already authorized
   transporting this corpus via GitHub. The release tag identifier is
   `research-transfer-2026-09-14`; it is a research draft accessible while signed
   into the owner/collaborator account. The ZIP is 1,834,667,666 bytes, SHA-256
   `a76a3ae0a95751df0c91e747a363c776ad073cb5ece22bfb54acd6de7e3b8c53`.
   All 147 files were downloaded back from GitHub, restored into a separate
   Windows checkout, and hash-verified. A normal Git pull does not fetch this
   ignored audio folder. Do not ask me to re-export the supplied files.

### What has already been established

- The source is `tests for presets/It’s a coat-original-test.wav`. LANDR-prefixed
  files are saved LANDR masters; Clarity/Oomph/Tape/Universal files without a
  service prefix are BandLab masters. `YES-compressor-on.wav` and
  `YES-compressor-off.wav` are owner exports. The earlier YES Universal Intensity
  75 export is byte-identical to On. Service download dates/versions are unknown.
- The isolated native Rust harness reproduced **both owner exports byte for
  byte**. It uses actual production DSP, source analysis, SRC and WAV writing.
  No production DSP or defaults were changed by this investigation.
- The reproduced setup is Universal 75%, default density 50%, Adapt 50%, Custom
  target -9 LUFS, -1 dBTP ceiling, neutral manual trims/EQ, delivered 48 kHz
  PCM24. Source rate is 44.1 kHz. The report has full settings and hashes.
- Current YES: -9.0 LUFS, LRA 3.1 LU, crest 8.95 dB. Compressor Off: -9.0 LUFS,
  LRA 3.6 LU, crest 9.42 dB. LANDR High styles: -9.4 to -9.5 LUFS, LRA 3.8–3.9,
  crest 11.07–11.64 dB, approximately -0.3 dBTP. Original crest is 12.72 dB.
- With the compressor retained, diagnostic saturation bypass raises crest to
  11.14 dB and delivers about -9.56 LUFS; limiter bypass raises crest to 10.74
  dB at about -9.18 LUFS. These are interacting stages, not additive percentages
  of blame. Compressor bypass also removes crossover/makeup effects.
- Adapt does ease compression: at requested density 50%, maximum low-band gain
  reduction is 3.05 dB with Adapt off, 1.25 dB at Adapt 50%, and 0.30 dB at 100%.
  It does not directly reduce the preset's input drive or saturation.
- Keeping every stage but applying -3 dB input trim gives about -9.47 LUFS,
  LRA 3.90, crest 11.02 dB and -1 dBTP. This is a promising diagnostic candidate,
  **not an adopted global trim** and not proof of a perceptual winner.
- The current baseline chain reaches roughly -7.37 LUFS before being attenuated
  to -9.0. Investigate whether less drive could avoid some peak suppression;
  reducing final output gain cannot undo previous nonlinear processing.
- Universal 75 resolves saturation amount to 0.0715. That is a drive coefficient,
  not a 7.15% wet blend: positive amounts process the whole signal through
  `tanh(x * drive) / tanh(drive)`, where `drive = 1 + 2 * amount`. Investigate the
  gain/continuity mapping without declaring an untested tonal change a bug fix.
- Density and adaptation have distinct useful functions. The proposed UI keeps
  compression amount in Advanced and makes source adaptation primarily automatic,
  with expandable strength. It is still a proposal. The Auto density thumb and
  frontend-only effective-compression summary have documented accuracy issues.
- A–E are the same 20-second passage at -16 LUFS. I found the differences too
  subtle to make a useful choice. This is neither a preference verdict nor an
  equivalence test. Do not misreport it as passed listening or make repeating it
  the next gate.

### Do the next useful work

Start with a short statement of what you know, your leading hypothesis and the
first concrete test, then execute. Do not spend a whole turn writing another plan.

1. **Build a practical evaluation set.** Inspect existing authorized fixtures;
   find additional no-cost music with documented usable licenses yourself.
   Start with about 6–12 deliberately varied sources and increase coverage if
   findings warrant it: dynamic acoustic/piano, vocals, drums/bass, dense rock or
   electronic material, sparse sections and transitions. Include different
   input levels of the same source, without clipping the input. Separate
   already-mastered songs from genuine premaster mixes. Add deterministic
   synthetic signals for mechanisms with known expected behavior. Do not count
   gain-shifted copies as independent songs or one composer's catalogue as full
   genre coverage. Preserve URLs, licenses, credits, original formats and hashes.
2. **Compare bounded candidates.** Keep the current engine as control. The
   leading hypothesis is source/target-aware input drive and a better-calibrated
   saturation amount, rather than simply deleting compression or maximizing
   dynamic range. Reuse or extend the isolated harness to test those hypotheses
   and retain meaningful compressor/adaptation choices. Change one mechanism at
   a time, then examine interactions. Predeclare candidate ranges and criteria
   before selecting the best-looking outputs. Reserve some sources as a holdout.
3. **Make fair comparisons.** Report achieved integrated LUFS, true peak,
   short-term dynamics, section contrast, transient behavior, spectrum, stereo
   behavior and relevant gain-reduction statistics. Compare music at equal
   playback loudness using gain only. Separately compare delivered target/ceiling
   behavior and mark targets that cannot be reached. LANDR's -0.3 dBTP and YES's
   -1 dBTP ceilings differ; do not silently conflate them or add a comparison
   limiter that changes the evidence. A scalar trim cannot preserve both a
   changed true-peak ceiling and the same integrated loudness.
4. **Test actual mechanisms.** Use tones, multitone signals, bursts/transients,
   silence/tails and level sweeps to investigate unintended aliasing, excessive
   overshoot, attack/recovery, discontinuities, denormals and instability.
   Measure compression activity and saturation/limiter interaction on music.
   Distinguish intentional harmonics/voicing from artifacts. Waveform residuals
   after EQ/crossover phase changes are not automatically distortion, and
   speech-quality scores or an unvalidated audio model are not a mastering judge.
5. **Use LANDR intelligently.** Treat the saved masters as reference behavior
   for this exact source and as a prior for plausible processing, not ground
   truth or evidence of current service-wide performance. Look for repeatable
   differences before generalizing. Additional same-source service comparisons
   may help if obtainable within existing authorization; do not spend money,
   create accounts or upload new music to third parties without the applicable
   authorization/rights. Lack of more LANDR exports must not halt local tests.
6. **Choose and explain.** Give a ranked recommendation: what should remain,
   what should change, the measured advantage, the price in loudness/character,
   which sources regress, and confidence/remaining uncertainty. Use several
   explicit criteria rather than an invented universal quality number. A
   sensible outcome may be keeping current behavior if alternatives offer no
   material advantage. Do not manufacture an audible improvement to satisfy me.
   If listening would still resolve a specific remaining choice, provide a few
   clearly targeted cases and explain exactly what they discriminate; do not
   hand the whole engineering decision back to me.

Proceed autonomously with research, measurement, authorized fixture acquisition
and reversible isolated prototypes. This request does not adopt a preset retune,
enable gated Adaptive Compressor/Phase-B/album features, change prices, resume
the deferred crossover refactor, or publish an application release. Preserve
source files and prior evidence. Document the concrete recommendation and its
validation so it remains usable on another machine. Follow current repository
instructions for integrating any later objective correction or proposed voicing
change; do not treat a diagnostic bypass as a production feature.

## Technical handrails for the receiving agent

### Additional owner question: where should Density Auto place the thumb?

The owner asked whether Auto should sit at the preset's actual default, and
whether raising Density and Adapt together would make the thumb move confusingly.
They requested that this be carried forward because they were leaving the PC.
This is a UI recommendation to evaluate, not an implemented or adopted redesign.

Recommended contract: **the editable thumb represents requested density before
adaptation**. In Auto, place it at the preset's requested default; after a drag,
place it at the user's explicit requested value. Adapt strength may change the
resolved processing but must not move that thumb or overwrite the requested value.
For example, increasing requested density to 75% leaves the thumb at 75%, even
if higher Adapt strength then eases the actual processing for this source.

Current named presets default to 0.50; Custom defaults to 0. Different named
presets still have different threshold/ratio/timing calibrations at that same
macro position. Density percentage is neither wet/dry mix nor measured gain
reduction. Do not invent different default positions based on how aggressive a
preset sounds.

Implementation/review checklist for a separate, bounded UI slice:

- Keep null/unset as Auto in saved settings; displaying 50% must not silently
  save an explicit 0.50. An explicit density value remains in Preset compressor
  mode; do not confuse it with switching to Manual per-band mode.
- Consider an `Auto — preset default 50%` readout and visible return-to-Auto
  action. Actual wording is still open. Returning to Auto restores null.
- Show any source-adaptive easing separately, using backend-resolved values or
  an accurate short explanation. Live gain-reduction meters describe signal
  behavior; they are not another value for the editable density thumb.
- Verify Auto, drag/keyboard edits, reset, preset changes, save/reopen, analysis
  arriving late, Adapt changes, and Preset/Manual/Off transitions. Preserve
  current audio and defaults; run the applicable UI/headless checks if implemented.
- `AdvancedPanel.tsx` currently supplies no `sliderAutoValue` for Density, so
  `NumberField` parks a null value at its minimum. It already supports a supplied
  Auto position. **Do not globally change all NumberField Auto semantics:** Width
  intentionally has a separately resolved Auto-value contract.
- Preserve the original sound-quality investigation as the primary task. This
  small presentation question must not become a reason to rewrite DSP or delay
  the independent evidence work.

### Diagnostic harness and evidence

- Restored harness: `test-output/yes-stage-ablation-20260914/`. `prepare.py`
  records production hashes and creates private guardrail/confidence copies that
  exclude only desktop IPC wrappers. The first attempt to use the browser crate
  natively failed on those wrappers; that was harness packaging, not an audio
  failure. Prefer the existing standalone harness over rediscovering that issue.
- The current harness hardcodes Universal 75, target -9, ceiling -1, output 48k/24.
  Generalize deliberately for new experiments; it is not yet a configurable
  production-quality evaluation runner. Preserve its locked baseline evidence.
- Work in a new ignored experiment directory. The writer avoids overwriting WAVs,
  but the harness also writes same-named JSON/coefficient/settings files; reruns
  into the original output directory can mix evidence. Do not do that.
- `analyze.py` verifies the eight baseline/ablation renders with FFmpeg and scipy.
  Engine DR uses 100 ms windows; some independent statistics use 400 ms. Do not
  substitute one into source-profile decisions as if they were equivalent.
- The archive's log/JSON absolute paths identify the prior Windows machine.
  Rebase runtime paths from the current checkout; do not reinterpret missing
  old absolute paths as missing archived content. Build caches are excluded.
- Whole-WAV byte identity was verified on Windows. If a new platform differs,
  investigate decoding, versions, DSP and float behavior; report the difference
  without claiming that a prior Windows hash already proves Mac parity.
- All original analyses, 18 full diagnostic renders and the five clips are
  available. No additional owner exports are required to resume this work.

## Starting points for acquiring more material

These are leads checked September 14, not a completed evaluation corpus:

- [Scott Buckley's library](https://www.scottbuckley.com.au/library/) states
  CC BY 4.0 terms and offers varied finished arrangements. Follow each track's
  attribution/download details. Its finished tracks are useful already-mastered
  regression material, not automatically premaster ground truth.
- [Incompetech licensing](https://incompetech.com/music/royalty-free/licenses/)
  offers no-charge Creative Commons use with credit; select and record the exact
  title's terms. Do not purchase its attribution-free alternative without approval.
- [Free Music Archive](https://freemusicarchive.org/faq) varies by track license;
  free download alone does not establish permission for every intended use.
- [Cambridge multitrack FAQ](https://www.cambridge-mt.com/ms3/mtk-faq/) explicitly
  directs research users to obtain contributor permission. Do not assume its
  educational download availability authorizes this product-development corpus.
- [CC BY 4.0 terms](https://creativecommons.org/licenses/by/4.0/) support sharing
  and adaptation, including commercial purposes, with attribution, a license
  link and change notices. Record provenance and comply with each selected work's
  actual license. Existing permission for our GitHub fixture archive does not
  grant rights in newly acquired third-party material.

Do not let an unsuitable library block the task: move to material with suitable
terms, use available sources, and continue synthetic mechanism tests independently.

## Suggested skills

Use an evidence-led debugging or explanation skill if it improves the actual
investigation. Use the handoff skill for another transfer. Do not invoke a broad
design/refactor workflow solely because the discussion includes sliders.
