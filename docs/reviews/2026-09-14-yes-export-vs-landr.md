# Actual YES export versus saved LANDR masters — September 14, 2026

> Latest follow-up: [stage-isolation research](2026-09-14-dynamics-stage-investigation.md)
> reproduces the owner exports byte for byte and resolves part of the stage
> attribution left open in this earlier comparison. For audio and measurements
> on another machine, use the [research transfer guide](2026-09-14-research-handoff.md).

## Finding

The supplied YES Universal Intensity 75 export is slightly louder than all three
saved LANDR High masters, with materially lower peak contrast and less loudness
variation. Its broad midrange spectrum stays close to the source, with a modest
upper-treble lift, while its side-to-mid energy increases. The LANDR exports
make different tonal choices and reduce side-to-mid energy on this source.

This shifts the practical question from whether YES can produce enough
loudness to how much dynamics it should preserve at a chosen loudness. The
measurement supports a denser result at this YES setting; it does not identify
which stage caused it or establish an audible quality winner.

For a broadly useful Universal starting point, my recommendation is to test
whether a lower processing amount can retain more peak and section contrast
at comparable delivery loudness. This does not prescribe a preset retune or
establish that the user's Intensity 75 choice is undesirable.

## Provenance and method

- Owner supplied `tests for presets/It_s_a_coat-original-test_YESmaster_mastered_universal intensity75.wav`.
- SHA-256: `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce`.
- YES: stereo PCM24, 48 kHz, 11,738,879 frames, 244.559979 seconds.
- Original and three LANDR High references: stereo PCM16, 44.1 kHz,
  10,785,095 frames, 244.559977 seconds. Durations agree within one 44.1 kHz sample.
- Original/LANDR identities come from the owner; Universal and Intensity 75
  are recorded in the supplied filename. Exact YES build, delivery target and
  other controls are not embedded in the inspected WAV metadata. Do not infer
  an exact code revision or default configuration from these measurements.
- Five files were freshly measured with independent FFmpeg `ebur128=peak=true`
  at their native sample rates. Results have 0.1 LU/dB display precision.
- Native PCM was decoded with scipy; 24-bit PCM is left-justified in int32 and
  normalized accordingly. Independent FFmpeg `astats` reproduced YES sample
  peak (-2.648146 dBFS) and RMS (-11.598021 dBFS), validating this scaling.
- Spectral analysis uses the same 44.1 kHz grid for every file; YES is resampled
  in memory with scipy `resample_poly`. Welch uses 8192-sample windows and 4096
  overlap, stereo power averaging and frequency-bin-width integration. Whole-
  file RMS is matched before interpreting band-energy differences.
- Correlation on downsampled mono in three sections estimates small relative
  offsets: about 2–3 ms for YES, 11.3 ms for LANDR Open, and zero for Balanced/
  Warm. These estimates can be affected by processing and are not exact latency
  proof. The 3-second loudness chart and whole-track metrics do not depend on
  sample-perfect alignment. Future listening/null comparisons should align first.
- All five input hashes were unchanged after processing. No audio was uploaded
  or rewritten. DSP, settings and calibration gates were not changed.

## Loudness, peaks and dynamics

| Version | Integrated LUFS | LRA (LU) | True peak (dBTP) | Whole-file crest (dB) | Active 400 ms crest median (dB) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original | -12.3 | 4.1 | -1.9 | 12.72 | 10.78 |
| YES Universal 75 | -9.0 | 3.1 | -2.6 | 8.95 | 8.62 |
| LANDR Balanced High | -9.4 | 3.8 | -0.3 | 11.39 | 10.86 |
| LANDR Open High | -9.5 | 3.9 | -0.3 | 11.64 | 11.21 |
| LANDR Warm High | -9.5 | 3.9 | -0.3 | 11.07 | 10.47 |

- YES is 0.4–0.5 LU louder than these LANDR masters, and 3.3 LU above source.
- YES's whole-file peak-to-RMS contrast is about 2.1–2.7 dB lower than LANDR's.
  The median 400 ms crest comparison points in the same direction; this is not
  solely one unusual peak affecting the whole-file result.
- YES LRA falls 1.0 LU below source; LANDR falls only 0.2–0.3 LU. The chart
  shows YES relatively lifting quieter passages and reducing louder passages
  after integrated-loudness normalization. Active 400 ms RMS P95–P10 spread is
  4.81 dB for YES, versus 6.15–6.92 dB for LANDR and 6.46 dB for the source.
- YES also leaves more peak headroom. At equal -14 LUFS playback normalization,
  its approximate true peak becomes -7.6 dBTP, versus -4.9 to -4.8 for LANDR.
  A uniform volume change cannot restore that relative peak contrast.
- No sample reaches full scale in these files, and all measured true peaks are
  below zero. That does not rule out saturation, pumping or other audible
  processing artifacts. Their absence cannot be established by these metrics.

Crest is sample peak relative to RMS; active block crest uses blocks above
-40 dBFS RMS. LRA is longer-term variation. Neither is a perceptual punch score
or a universal “higher is better” target. Dense music can intentionally benefit
from tighter dynamics.

## Tone and stereo

Selected output band-energy changes versus original after whole-file RMS match:

| Band | YES Universal 75 | LANDR Balanced High | LANDR Open High | LANDR Warm High |
| --- | ---: | ---: | ---: | ---: |
| 20–60 Hz | -0.68 dB | +0.75 dB | +0.72 dB | +1.23 dB |
| 120–250 Hz | +0.11 dB | -0.92 dB | -1.25 dB | -0.99 dB |
| 250–500 Hz | +0.21 dB | -0.75 dB | -1.03 dB | -1.04 dB |
| 2–4 kHz | +0.21 dB | +0.36 dB | +1.40 dB | -0.78 dB |
| 8–16 kHz | +1.38 dB | -1.08 dB | -0.21 dB | -2.83 dB |

YES changes the 120 Hz–4 kHz band averages by only about +0.1–0.2 dB, while
LANDR reduces 120–500 Hz in all three styles. YES has more upper-treble energy
relative to its overall level than any of these LANDR exports. That is a
measured tonal distinction; calling it pleasant air or harshness requires
listening. The output spectra do not recover EQ settings because dynamics,
saturation, filters and stereo processing can all affect them.

Side-to-mid energy versus original rises **0.86 dB** in YES. It falls **0.80–1.24
dB** in LANDR. Broadband L/R correlation is 0.683 for YES, 0.732 for original,
and 0.772–0.792 for LANDR. These indicate more relative side content in YES;
they do not identify a particular widening stage or prove a mono-compatibility
defect. Mono listening and frequency-specific checks would answer that separately.

## Interpretation and next comparison

This file does not support a claim that LANDR is superior because it processes
longer or makes a louder master. YES is already producing a loud result, with
more restrained midrange tonal changes but stronger dynamics reduction at this
setting. If natural contrast is the priority, LANDR's measured dynamics behavior
is the better target in this particular comparison. That is a design judgment,
not a blind listening verdict or a ranking across songs.

The most informative next export is Universal Intensity 50 with the same source,
delivery target and other controls as this file. Compare it with Intensity 75
and LANDR at matched playback loudness. If less processing improves contrast
while retaining the desired sound, this may be largely a setting/default choice.
If substantial density reduction persists at lower amounts, inspect which
compressor, saturation and limiter stages contribute before changing them.
An intensity change affects multiple stages, so it is not itself a causal
isolation test. Separate stage experiments would establish causality.

> **Later controlled pair now available:** the owner supplied Compressor On/Off
> exports with Adapt at 50%. See the final section below. The proposed first
> compressor comparison has been completed; do not request these same files again.

One song and one YES setting cannot establish a generally better service or
prove that new adaptive algorithms are necessary. This follow-up adds actual
export evidence to the earlier [architecture/vendor assessment](2026-09-14-landr-waves-dsp-assessment.md).

Local evidence (ignored by git):

- [Comparison script](../../test-output/yes-landr-export-comparison-20260914/compare_export.py)
- [Full measurements and provenance](../../test-output/yes-landr-export-comparison-20260914/comparison.json)
- [CSV](../../test-output/yes-landr-export-comparison-20260914/comparison.csv)
- [Comparison chart](../../test-output/yes-landr-export-comparison-20260914/yes-vs-landr.png)
- [Independent PCM scale cross-check](../../test-output/yes-landr-export-comparison-20260914/yes-astats.txt)

## Follow-up: Preset density and Adapt strength

The owner asked how these controls work after seeing the dynamics comparison.
Current UI, defaults, coefficient construction and guardrail code were reviewed.
No DSP, presets, gate states or product UI were changed by this review.

| Control | Default / meaning | What increasing it does |
| --- | --- | --- |
| Preset density | Unset/Auto resolves to 0.50 for named presets, 0 for Custom. 0 bypasses the preset compressor; 0.50 reaches the preset baseline before adaptation; above 0.50 pushes beyond that baseline. | Lowers the compressor threshold, raises its ratio and changes automatic makeup gain. It is a parameter macro, not a percentage of signal compressed. |
| Adapt strength | 50% by default. Applies source-based defensive trims in Track Master with a resolved source profile. | Can reduce preset compression density and trim preset bass/treble boosts and excess widening where the source triggers a guardrail. Turning it Off removes these reductions. |
| Intensity | Separate from Preset density. Preset scale is `0.4 + 1.2 * intensity`, so 50% = 1.0 and 75% = 1.3. | Scales preset EQ, input gain, saturation and transient-shaping intent. It does not directly change the preset compressor threshold/ratio formula, but changes the signal fed to the dynamics stages. |

Preset density only controls the preset compressor behavior in Preset mode.
Manual mode uses explicit per-band overrides, with preset/density-derived
fallbacks for missing fields; automatic density trimming does not override those
manual choices. Off bypasses this compressor. Saturation, transient shaping,
limiting and delivery gain remain, so Compressor Off is not a promise of
unchanged dynamics or a dry bypass.

The active density guard uses source LRA and the P95–P10 dynamic-range statistic.
It scales requested density downward. At full triggering and Adapt 50%, density
can be halved; at Adapt 100%, the cap permits up to a 60% reduction. This leaves
at least 40% of a positive requested density, rather than automatically selecting
Off. Those percentages concern the density parameter, not actual gain reduction.
Adapt does not directly reduce preset saturation or input gain. It is not a
guarantee of preserved transients through the complete chain.

Illustration using the original's rounded independent measurements, assuming
Universal, requested density 0.50, a valid source profile and the default-off
confidence/adaptive-compressor gates:

| Adapt strength | Effective density | Compressor threshold | Ratio | Calculated band makeup gain |
| --- | ---: | ---: | ---: | ---: |
| Off | 0.50 | -12.50 dBFS | 1.45:1 | +1.94 dB |
| 50% | about 0.34 | about -8.54 dBFS | about 1.31:1 | about +1.00 dB |
| 100% | 0.20 | -5.00 dBFS | 1.18:1 | about +0.38 dB |

These are illustrative settings computed from inspected formulas, not captured
settings from the user's export or a new audio render. Native analysis precision
can differ. The deeper per-band Adaptive Compressor remains a separate
off-by-default gate; moving Adapt strength does not turn that feature on.

Code evidence:

- `src/hooks/useTrackMaster.ts`: default Preset mode, null density, Adapt 0.5.
- `src-tauri/src/dsp.rs::ChainCoeffs::from_settings`: preset-relative density,
  manual/off handling, source trimming, threshold/ratio and automatic makeup.
- `src-tauri/src/dsp.rs::PRESET_UNIVERSAL`: baseline -12.5 dBFS / 1.45:1,
  15 ms attack / 250 ms release. These are settings, not measured gain reduction.
- `src-tauri/src/guardrails.rs::SourceGuardrails`: reduce-only active guardrails
  with density cap 0.60. `profile_store.rs` excludes Album adaptation.

Two presentation findings are confirmed by code review and remain unmodified:

1. `AdvancedPanel.tsx` supplies a null Preset density without `sliderAutoValue`;
   `fields.tsx::NumberField` falls back to `min=0`. The UI says Auto but parks
   its thumb at zero, despite the non-Custom engine default being 0.50. A later
   UI fix should show the requested preset default clearly and distinguish it
   from the lower source-adapted value.
2. The idle “Effective compression” summary uses
   `src/lib/compressor-auto.ts::compressorAutoReadouts`, which computes raw
   preset/density values without the source guardrail. The engine may use
   lighter settings. The live gain-reduction meter is a separate measurement;
   these findings do not establish a fault in that meter. A later fix should
   use backend-resolved effective settings or label the summary as a baseline.

Existing frontend checks: 24 tests passed across `App.adaptive-strength`,
`App.compressor-mode` and `lib/compressor-auto`. These verify existing UI/control
contracts, not native sound quality or the two missing presentation cases.
[Illustrative calculations](../../test-output/yes-control-review-20260914/illustrative-control-math.json)
are kept with ignored local evidence.

The prior export shows the combined chain reduces dynamic contrast; it does
not prove the preset compressor caused all that reduction. A better causal
follow-up than changing Intensity alone is to keep Intensity 75 and other
controls fixed, compare Preset compression with Compressor Off, and normalize
playback loudness. If contrast remains similarly reduced, investigate saturation,
limiting and the input/gain structure. Lower density and higher Adapt can then
be compared separately. No default should be retuned from this one file alone.

## Completed owner comparison: Compressor Preset versus Off

The owner exported both requested files, confirming Adapt strength 50% for both,
after instructions to keep Universal, Intensity 75 and every other control fixed.
Files: `tests for presets/YES-compressor-on.wav` and `YES-compressor-off.wav`.
Both are stereo PCM24 / 48 kHz / 11,738,879 frames. All input hashes were verified
unchanged. The On file is byte-for-byte identical to the previously supplied
Universal Intensity 75 export, providing continuity with the earlier measurements.

- On SHA-256: `639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce`.
- Off SHA-256: `d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5`.

| Measurement | Original | YES On | YES Off | LANDR Balanced High |
| --- | ---: | ---: | ---: | ---: |
| Integrated LUFS | -12.3 | -9.0 | -9.0 | -9.4 |
| LRA (LU) | 4.1 | 3.1 | 3.6 | 3.8 |
| True peak (dBTP) | -1.9 | -2.6 | -2.1 | -0.3 |
| Whole-file crest (dB) | 12.72 | 8.95 | 9.42 | 11.39 |
| Active 400 ms crest median (dB) | 10.78 | 8.62 | 9.10 | 10.86 |
| Active 400 ms RMS P95–P10 spread (dB) | 6.46 | 4.81 | 5.34 | 6.31 |

**Finding:** the mode change restores 0.5 LU of longer-term variation and about
0.47 dB of whole-file peak-to-RMS contrast, at the same measured integrated
loudness. The 400 ms crest median also rises about 0.48 dB. Thus the preset
compressor contributes to the tighter result, but bypassing it leaves most of
the difference in peak contrast between YES and the source/LANDR.

The Off export's LRA is relatively close to LANDR (3.6 versus 3.8–3.9 LU),
while its whole-file crest still sits around 1.7–2.2 dB below the three LANDR
exports. These are separate time scales; neither metric alone is an audible
quality verdict. Do not convert their changes into a universal percentage of
“compression” or assign an additive percentage of causality to a processor.

Tone and stereo hardly move between On and Off: RMS-matched band changes differ
by at most about 0.19 dB across the measured bands, and side-to-mid energy differs
by about 0.001 dB. Both exports have finite PCM, no samples at full scale and
measured true peaks below zero. This does not certify absence of saturation,
pumping or other audible artifacts.

The controlled change is the **complete compressor mode**, including its
crossover, makeup gain and downstream reactions. It does not isolate compressor
gain reduction from those effects. Bypass changes waveform phase as well;
the saved zero-shift sample residual is explicitly not a distortion score.
Exact application build and full remaining settings are still not embedded in
the WAV metadata; the controlled settings come from the owner's report.

Code inspection confirms that Compressor Off leaves transient shaping, preset
saturation, input gain, EQ and limiting in place. The remaining peak-contrast
difference therefore warrants isolating those stages and their interactions,
starting with saturation/input drive and limiting. It does not prove that the
limiter or saturation is defective, and it does not justify globally disabling
preset compression or changing Adapt strength. Uniform final loudness gain
cannot restore crest factor.

New local evidence, with the original and LANDR context reused only after
verifying its source hashes:

- [Pair measurement script](../../test-output/yes-compressor-ab-20260914/compare_pair.py)
- [Full measurements](../../test-output/yes-compressor-ab-20260914/comparison.json)
- [CSV](../../test-output/yes-compressor-ab-20260914/comparison.csv)
- [Matched-level comparison chart](../../test-output/yes-compressor-ab-20260914/compressor-on-off.png)

No DSP code, preset settings or calibration gates were changed. The requested
owner On/Off comparison is complete; no repeat export is needed for this check.

## Proposed integration of density and adaptation

**Subsequent investigation:** [stage-isolation results](2026-09-14-dynamics-stage-investigation.md)
now reproduce both owner exports byte for byte and quantify compressor,
saturation, limiting, adaptation and input-drive effects. The stage-attribution
uncertainty recorded above was the state before that experiment. No production
sound or UI has changed.

Owner question: do both controls deserve a place, and how should they work
together more smoothly? This is an agent recommendation for discussion, not an
adopted product decision or an implementation change.

Keep both underlying functions. Density expresses the user's desired preset
compression amount. Adaptation reduces eligible preset processing based on the
source and also affects EQ and width. Combining them into one slider would hide
distinct choices. Their current presentation gives them too little context.

Recommended hierarchy:

- Standard retains its current style/Intensity/loudness focus, with source
  adaptation operating automatically. Do not add a second dynamics dial there.
- In Advanced, keep the Compression panel with Preset / Manual / Off. Rename
  “Preset density” to “Compression amount” (or “Amount” within that panel),
  with clear Less / More endpoints and a marked Preset default. Zero remains
  compressor bypass; this does not bypass saturation or limiting.
- Move Adapt strength out of the Width/Warmth/Presence grid into a small global
  “Adapt to track” section. Present enabled/disabled state first; put the exact
  strength under an expandable adjustment. Initially retain the existing 50%
  default and existing saved strengths. Do not silently reset an existing track
  when changing the UI or when toggling adaptation back on.
- Explain adaptation as reducing preset boosts, compression and widening where
  needed. It is not complete automatic mastering or protection from all dynamic
  changes. Show an actual adjustment message only when the resolved backend
  result justifies it, e.g. “Preset compression eased for this track.” A source
  inside the guardrails can show no adjustment; unavailable analysis and Album
  must not display a false active-adjustment state.

Keep one editable compression amount: the requested value. The slider should
not move under the user's pointer when adaptation changes the effective result.
In the Auto/unset state, show the named preset's requested default at 0.50
(Custom remains 0), rather than parking the thumb at zero. Show the resolved
threshold/ratio and measured live gain reduction separately, using backend
values instead of the current frontend-only preset summary. A short explanation
usually communicates adaptation better than two competing percentage sliders.

Changing the compression amount must not turn adaptation off; changing adaptation
must not overwrite the requested amount or manual compressor values. Preserve
the existing Manual and Off contracts. The underlying calculation order remains
requested preset settings, applicable source trims, resolved chain coefficients,
then signal-dependent processing. No new algorithm or gate activation is implied.

This presentation improvement can preserve existing audio output. A later
implementation should verify save/reopen/reset and mode transitions, truthful
Auto/effective readouts and absence of DSP changes. It is separate from the
subsequent saturation/limiting investigation: simpler controls alone do not
restore the measured peak contrast. The linked stage report now establishes
specific contributions on this fixture; cross-track calibration and listening
preference remain open.
