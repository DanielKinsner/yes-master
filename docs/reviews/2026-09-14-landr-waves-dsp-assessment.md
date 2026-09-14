# LANDR / Waves DSP assessment — September 14, 2026

> **Follow-up: actual YES export supplied.** [The direct file comparison](2026-09-14-yes-export-vs-landr.md)
> measures the owner's Universal Intensity 75 export against the three LANDR
> High files. The initial investigation below did not yet include that export.

## Judgment

YES Master's strongest next DSP investment is better selection of processing
from the source analysis it already computes. Keep the Intensity control and
aim for a trustworthy automatic starting point. A service taking a minute
does not establish that its analysis is deeper, its result is preferable, or
that YES needs an additional minute of computation.

The current native engine has substantial mastering machinery. Its active
source adaptation is deliberately limited: it softens existing preset moves.
It cannot independently introduce corrective EQ, and deeper per-band compressor
adaptation exists behind an off-by-default calibration gate. These are concrete
architectural limits, not proof that every competitor output sounds better.

This assessment refreshes the saved external references and checks current
source code. **It does not contain fresh YES renders, Waves renders, or a blind
listening verdict.** The historical YES comparison in the June report must not
be presented as a benchmark of today's build.

## Scope and evidence

- Owner identifies `tests for presets/*-original-test.wav` as the source,
  LANDR-prefixed files as LANDR exports, and the remaining masters as BandLab.
- One source, seven external masters: four BandLab and three LANDR High styles.
  All eight files are 44.1 kHz stereo PCM16, 10,785,095 frames / 244.559977 s.
- Download dates and service engine versions are unknown. These files describe
  saved service outputs, not necessarily the services' September 2026 engines.
- Native source inspected at HEAD `880f4f00bb3721039638c384071f08d06e29aec8` in
  the shared working tree. Existing unrelated Cargo/doc changes were preserved.
- No audio was uploaded, altered, committed, or purchased. Input SHA-256 hashes
  were checked before and after measurement. DSP and calibration gates were not
  changed. Internal research notes are not a release or public quality claim.

Existing work worth retaining:

- [June reference analysis](../PRESET_REFERENCE_ANALYSIS_2026-06-12.md) already
  investigated this exact source and the three LANDR High exports.
- [Tier-1 design](../plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md)
  deliberately separates defensive trimming from future corrective processing.
- [Reference runner](../../src-tauri/src/reference_tuning.rs) renders through
  the actual native engine and resolves source context through the shared
  evidence path. Its discovery is built around four named preset references;
  it does not currently provide a general LANDR/Waves comparison manifest.

## What the vendors disclose, and what remains unknown

**LANDR:** its support description, updated in 2021 and still accessible, says
its adaptive engine uses machine learning, micro-genre detection and adjustments
over time involving EQ, multiband compression, stereo processing, limiting and
saturation. This describes processor families and intent; it does not specify
the current model, processing order, crossover frequencies, thresholds, timing,
or source-to-parameter rules. [LANDR engine description](https://support.landr.com/hc/en-us/articles/115009725688-What-is-LANDR-Mastering).

Another support page describes choosing processors and their reactions per
song, with additional cohesion decisions for albums. That is a useful account
of decision-making, not an independently verified implementation diagram.
[LANDR track versus album processing](https://support.landr.com/hc/en-us/articles/360005148834-What-s-the-difference-between-mastering-an-album-and-a-single-track-using-LANDR).

LANDR also documents Low, Medium and High loudness choices with different
dynamics tradeoffs. The saved filenames explicitly record High. The absence of
a continuous Intensity slider does not mean the service has one uniquely
correct sound. [LANDR loudness choices](https://support.landr.com/hc/en-us/articles/115009557127-What-are-the-LANDR-loudness-options).

**Waves:** its April 2025 guide describes a neural network trained using
professional mastering examples. Upload, a 30-second preview, and final
full-track processing are separate workflow steps. The guide does not disclose
model architecture or a reproducible DSP chain. [Waves getting-started guide](https://www.waves.com/support/waves-online-mastering-getting-started).

Waves provides three styles, Presence/Depth choices and an optional reference
track. Its FAQ explicitly says there is no dedicated LUFS target control;
references can influence both loudness and other sonic properties. Fewer
controls are a product choice, not evidence that artistic judgment disappeared.
[Waves FAQ](https://www.waves.com/online-mastering/faq).

**Why the wait:** upload, decoding, analysis, job scheduling, processing and
encoding are possible contributors to a remote workflow. Their proportions
are unknown here, including what a particular UI labels “analyzing.” There is
no server trace establishing a full minute of analysis CPU time, no controlled
same-hardware timing comparison, and no evidence of an artificial delay.
Offline model training is also distinct from processing an individual upload.

**Patent correction:** the older repo research cites US9654869B2. Its subject
is autonomous **multi-track** processing, including decisions across individual
signals. Its 2012 priority / 2017 publication and discussed feature-driven
processing are useful historical context. They do not establish the current
LANDR stereo mastering backend. Do not use this patent to assert that a specific
modern LANDR EQ, dynamic notch or width algorithm is known.
[Published patent](https://patents.google.com/patent/US9654869B2/en).

## Fresh mechanical measurements

Independent FFmpeg `ebur128=peak=true`, with the version and full source hashes
recorded in the local JSON. Loudness and true-peak values below have the tool's
0.1-unit display precision. Crest is sample-peak-to-whole-file-RMS, not a
per-transient punch score. LRA describes longer-term loudness variation and
does not prove unchanged attacks, freedom from distortion, or musical quality.

| File | Integrated LUFS | LRA (LU) | True peak (dBTP) | Crest (dB) |
| --- | ---: | ---: | ---: | ---: |
| Original | -12.3 | 4.1 | -1.9 | 12.72 |
| LANDR Balanced High | -9.4 | 3.8 | -0.3 | 11.39 |
| LANDR Open High | -9.5 | 3.9 | -0.3 | 11.64 |
| LANDR Warm High | -9.5 | 3.9 | -0.3 | 11.07 |
| BandLab Clarity | -12.0 | 3.7 | -1.8 | 12.59 |
| BandLab Oomph | -11.9 | 3.4 | -0.3 | 12.38 |
| BandLab Tape | -9.9 | 3.2 | -1.5 | 11.03 |
| BandLab Universal | -10.5 | 3.3 | +0.2 | 12.99 |

Findings for this source:

1. LANDR delivers about 2.8–2.9 LU of additional integrated loudness with LRA
   within 0.2–0.3 LU of the source. Crest falls about 1.1–1.6 dB. This is a
   worthwhile loudness/dynamics tradeoff to compare against, rather than proof
   that all dynamics are preserved.
2. LANDR is not uniformly making the source wider or brighter. Whole-file
   side-to-mid energy falls about 0.8–1.2 dB in all three exports; EQ can also
   affect this ratio, so it does not identify a particular width processor.
3. After whole-file RMS normalization, LANDR Warm has about 2.8 dB less energy
   at 8–16 kHz and 1.2 dB more at 20–60 Hz. Open's changes are more modest,
   within roughly ±1.4 dB across the measured bands. On this input, copying a
   dramatic “Open” high shelf would misread the reference.
4. BandLab styles span much stronger tonal differences: Oomph has roughly
   +3.9 dB at 20–60 Hz and -5.6 dB at 250–500 Hz after RMS normalization.
   Those are output band-energy changes, **not recovered EQ settings**.
5. BandLab Universal measures +0.2 dBTP despite no full-scale PCM samples.
   Sample peak and true peak are different checks. This single saved export
   does not establish the service's current behavior on other inputs.

Welch spectral analysis uses 8,192-sample windows / 4,096 overlap, stereo
power averaging, and identical bands. Compression, saturation and M/S changes
all affect the plotted spectrum; no transfer-function or plugin-chain recovery
is claimed. The additional 400 ms block P95–P10 statistic uses a -60 dBFS
activity floor and differs from the June report's recipe; do not compare those
columns as if they were exactly the same metric.

Local evidence, intentionally ignored by git:

- [Measurement script](../../test-output/landr-waves-research-20260914/analyze_references.py)
- [Full JSON and provenance](../../test-output/landr-waves-research-20260914/reference-measurements.json)
- [CSV](../../test-output/landr-waves-research-20260914/reference-measurements.csv)
- [Tone comparison chart](../../test-output/landr-waves-research-20260914/reference-tone-comparison.png)

## Current YES code: what exists and where adaptation stops

| Area | Current evidence | Implication |
| --- | --- | --- |
| Source analysis | `analysis.rs::analyze_one_with_progress` computes loudness, peaks, LRA, spectral balance and deep analysis; `deep_analysis.rs` retains windows, 31-band context and loud/body summaries. | Detailed analysis already exists. Do not start by adding another full-song scan. |
| Active source adaptation | `guardrails.rs::SourceGuardrails` derives bright/low/density/width multipliers; `dsp.rs::ChainCoeffs::from_settings` applies them to preset settings. | It trims positive preset EQ boosts, compression density and excess widening. It cannot introduce corrective cuts or boost a missing region independently of the preset. |
| Deep confidence | `confidence.rs` defaults `CONFIDENCE_GATING` to false; `profile_store.rs::apply_resolved_confidence` follows that gate. | Measurements exist without all deeper decisions being active. Confidence gating remains reduce-only when enabled. |
| Per-band compression adaptation | `guardrails.rs` defaults `ADAPTIVE_COMPRESSION` to false; `profile_store.rs::apply_resolved_compression_guards` wires per-band PSR and stand-down information. | Existing experimental code eases density, thresholds and ratios on already-dense bands. It is not a trained, general-purpose automatic compressor selector. Calibration remains necessary. |
| Processing blocks | `dsp.rs::MasteringChain` includes filters, three-band compression, transient shaping, M/S width, saturation and a linked lookahead limiter with FIR intersample detection. | YES already has the main processor families vendors name. Ordinary compressor envelopes already respond over time. Do not describe the engine as static gain/EQ only, or say true-peak detection is missing. |
| Loudness delivery | `engine.rs::ceiling_bounded_landing_delta_db` applies measured, ceiling-bounded final gain. | It protects the output ceiling. It does not search alternative compressor/limiter settings to achieve a louder target with a different distortion/transient tradeoff. |

Illustration: if a selected style calls for +2 dB of bass and the source is
already bass-heavy, active adaptation can reduce that boost. It cannot decide
that this source instead needs -1 dB of corrective bass EQ. The manual EQ can
make cuts; the limitation concerns automatic source-based decisions.

There are inspectable commercial precedents for stronger automatic decisions:
Ozone documents matching tone, dynamics and width, plus adjustable EQ/Stabilizer
amounts; sonible describes positions representing signal-adapted compressor
setups. These support the feasibility of an adaptive starting point **with**
creative control; they do not reveal LANDR or Waves internals.
[Ozone Master Assistant](https://www.izotope.com/community/blog/how-to-use-master-assistant-in-ozone),
[sonible Compression Matrix](https://help.sonible.com/hc/en-us/articles/24338576473500-What-is-the-Compression-Matrix-and-what-does-the-Compression-Scope-show).

## Recommended next work, in order

1. **Make a fair current-engine benchmark first.** Generalize the existing
   runner's reference manifest to record service, style, intensity/loudness,
   file hashes, YES revision, full settings and gate state. Existing settings
   inherit the default -14 LUFS delivery target; the saved LANDR High files
   measure near -9.5. That mismatch cannot be scored as an engine failure.
   Test both ordinary product defaults and a controlled loudness/ceiling
   comparison. Keep the delivery requirement explicit; do not silently change
   YES's default ceiling to LANDR's -0.3 dBTP.
2. **Evaluate the existing per-band dynamics work.** Measure default versus
   candidate behavior on dense, transient-heavy and sparse sources, at matched
   playback level. Record band gain reduction, transient changes, loudness
   landing and limiting behavior. A result must improve the targeted problem
   without broad regressions. This assessment does not turn on the gate or
   approve its provisional constants.
3. **Prototype bounded corrective tonal balancing.** Use existing spectral
   and window data to make small, source-dependent cuts/boosts separately from
   the chosen style. Begin with broad tonal balance and deliberate no-op ranges;
   do not force all genres or deliberately dark music toward one spectrum.
   Compare against the unchanged engine on held-out songs. More bands, a genre
   label or a neural network alone is not the acceptance criterion.
4. **Study loudness efficiency where measurements show a gap.** Test source-
   appropriate drive/compression/release choices before final limiting, with
   independent ceiling and artifact checks. Preserve the existing ceiling
   contract. The saved LANDR outputs motivate this experiment but do not identify
   which of their processors produced the result.
5. **Consider time-varying correction after the simpler baseline.** A bounded,
   smoothed adjustment for an intermittent harsh chorus may help where a fixed
   full-song correction hurts a quiet verse. Verify transitions and pumping.
   This is distinct from existing compressor envelopes, and is not evidence
   that LANDR uses a particular dynamic-EQ algorithm.

For a useful development corpus, start with roughly 12–20 diverse authorized
mixes as a practical pilot, keeping a subset out of tuning. Include acoustic,
electronic, vocal-heavy, bass-heavy, bright/thin, sparse and already-dense cases.
This is not a statistical proof of general superiority. Add LANDR Low/Medium
exports of the same sources and Waves references when available to separate
style from loudness effects. New paid exports/uploads require their applicable
authorization; the existing local corpus can support the first experiments.

Mechanical checks should cover decoded length/channels, finite PCM, clipping
and true peak, integrated/short-term loudness, LRA, transient statistics,
frequency-dependent stereo behavior, and spectrum. Synthetic impulses, sine
sweeps, dual tones and stepped envelopes test timing, nonlinear artifacts and
limiter behavior where ground truth is available. For musical preference,
randomize anonymous, time-aligned excerpts and match perceived playback level
without peak clipping. Use A/B preference for “which is better”; ABX answers
whether listeners can distinguish them. Neither a null-test residual nor one
aggregate metric is a sound-quality ranking.

The product goal is a stronger automatic default that earns trust across
different songs, with Intensity retained for taste. That is the most defensible
next step for a small developer; it does not require reproducing a proprietary
training operation or building a hosted service first.
