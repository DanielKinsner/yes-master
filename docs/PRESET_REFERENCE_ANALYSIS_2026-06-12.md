# Preset Reference Analysis - 2026-06-12

## Purpose

This document is the live reference-analysis home for external mastering-service
outputs kept under `tests for presets/`. The audio files are private/local; this
doc keeps only measurement summaries and tuning implications.

Current reference source:

- `tests for presets/It’s a coat-original-test.wav`

Current external references:

- BandLab/reference-set anchors: `universal`, `clarity`, `oomph`, `tape`
- LANDR additions: `Warm High`, `Balanced High`, `Open High`

The goal is not to clone another service. The goal is to learn what commercial
masters choose to change on the same source, then use those choices as
calibration evidence for YES Master presets.

## Provenance

The repo previously had `docs/PRESET_REFERENCE_ANALYSIS_2026-05-14.md`; it was
deleted when the current YES Master foundation was established on 2026-05-26.
That deleted report described the original four external references as
"online-mastered references." The owner identifies those four references as
BandLab-derived. This document carries that forward as owner provenance rather
than as a filename-derived fact.

LANDR public style notes say:

- Warm: vintage warmth, softer compression, thick/smooth sound.
- Balanced: controlled balance, clarity, and depth.
- Open: modern/open, with punch and presence.

LANDR's longer style article adds more useful technical color: Warm has softer
highs and rich low-mid/bass; Balanced is the default full-but-controlled choice;
Open uses a targeted midrange scoop with punchy low end and modern clarity.

Sources:

- https://support.landr.com/hc/en-us/articles/360019272934-What-are-Mastering-Styles
- https://support.landr.com/hc/en-us/articles/115009557127-What-are-the-LANDR-loudness-options
- https://blog.landr.com/mixing-mastering-style/
- https://www.landr.com/online-audio-mastering

## Measurement Method

Two evidence levels are used:

1. Existing YES Master private reference runner data:
   - `cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning`
   - Reports LUFS, dynamic range, spectral low/mid/high gap, width, energy
     density, transient flux, and export warning codes for the four mapped
     presets.

2. Read-only external-file measurement pass:
   - FFmpeg `ebur128=peak=true` for integrated LUFS, LRA, and true peak.
   - Python/Scipy WAV analysis for coarse spectrum, stereo width, macro dynamic
     movement, and energy-flux proxies.
   - LANDR files were measured without writing new runner artifacts.

Interpretation rule: level/loudness, dynamics, tone, and width are separated
where possible. A louder file can appear brighter or fuller from loudness alone,
so spectral conclusions should prefer level-matched deltas and listening notes.

## Current File Metadata

All current reference files are:

- WAV, PCM 16-bit
- 44.1 kHz
- stereo
- 244.56 seconds

The LANDR files have an `Lavf58.76.100` encoder tag, but the same duration and
format as the source. The differences below are mastering choices, not length or
sample-rate mismatches.

## BandLab / Original Reference Set

Recovered 2026-05-14 measurements compared each external master to the original.
The values below are from the deleted historical report and remain useful as
style anchors.

### Level And Dynamics

| Preset | Raw RMS | Peak | Crest | Stereo width | Main identity after level match |
| --- | ---: | ---: | ---: | ---: | --- |
| Universal | +1.62 dB | +1.96 dB | +0.30 dB | +0.25 dB wider | Mostly neutral, with top-end air |
| Clarity | +0.46 dB | +0.21 dB | -0.15 dB | -0.80 dB narrower | Scooped low-mid/mids, brighter air |
| Oomph | +2.31 dB | +1.72 dB | -0.31 dB | -2.72 dB narrower | Big sub lift, heavy low-mid/mid scoop |
| Tape | +2.13 dB | +0.35 dB | -1.77 dB | -0.07 dB | Strong glue/density, quiet material lifted |

### Tonal Deltas After RMS Match

Values are dB deltas vs the original after RMS matching.

| Preset | 20-60 | 60-120 | 120-250 | 250-500 | 500-1k | 1-2k | 2-4k | 4-8k | 8-16k |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Universal | -0.07 | -0.23 | +0.08 | -0.04 | -0.20 | +0.12 | -0.38 | +0.58 | +1.54 |
| Clarity | +0.93 | +0.21 | -0.73 | -1.02 | -1.56 | -2.30 | -1.20 | +1.18 | +2.13 |
| Oomph | +3.77 | +0.06 | -3.31 | -5.92 | -5.40 | -5.47 | -4.44 | -1.56 | -1.61 |
| Tape | -0.61 | -0.30 | +0.20 | +0.53 | +0.14 | -0.97 | -2.44 | +0.97 | +3.67 |

### Current YES Runner Snapshot

The current private reference runner still maps only the four BandLab/reference
anchors. It does not yet ingest LANDR styles as first-class rows.

| Preset | Source LUFS | Reference LUFS | YES LUFS | LUFS gap | Reference DR | YES DR | DR gap | Warning codes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Universal | -12.33 | -10.53 | -14.00 | -3.47 | 3.29 | 2.98 | -0.31 | dynamic_range_low; comp_density_on_compressed_source |
| Clarity | -12.33 | -12.04 | -14.00 | -1.96 | 3.69 | 3.18 | -0.50 | dynamic_range_low; comp_density_on_compressed_source |
| Oomph | -12.33 | -11.87 | -14.00 | -2.13 | 3.37 | 2.27 | -1.10 | dynamic_range_low; comp_density_on_compressed_source |
| Tape | -12.33 | -9.91 | -14.00 | -4.09 | 3.26 | 2.44 | -0.82 | dynamic_range_low; comp_density_on_compressed_source |

Important: the YES LUFS gap is expected because the runner lands YES renders at
`-14 LUFS`. These rows are more useful for dynamics/tone gaps than for judging
absolute loudness parity.

## LANDR High Additions

The LANDR files were exported at LANDR's High loudness setting. LANDR describes
High as its loud/power option, with dynamic-range sacrifice expected. That is a
reasonable comparison to YES Master's Standard `High` / hot-master path, but it
does not reveal the full style-vs-loudness curve by itself.

### Measured LANDR High Summary

| LANDR style | LUFS | LRA | True peak | Macro DR | Tone/width summary |
| --- | ---: | ---: | ---: | ---: | --- |
| Balanced High | -9.4 | 3.8 | -0.3 dBFS | 6.33 dB | Louder, slightly warmer/darker, narrower |
| Open High | -9.5 | 3.9 | -0.3 dBFS | 6.18 dB | Loud, nearly source-neutral broad tone, narrower |
| Warm High | -9.5 | 3.9 | -0.3 dBFS | 6.99 dB | Loud, low/body tilted, darker top, narrower |
| Original | -12.3 | 4.1 | -1.9 dBFS | 6.44 dB | Baseline |

### LANDR High Deltas Vs Original

| LANDR style | LUFS delta | LRA delta | Macro DR delta | Low fraction delta | Mid fraction delta | High fraction delta | Width delta | Correlation delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Balanced High | +2.9 | -0.3 | -0.10 | +0.0255 | -0.0121 | -0.0135 | -0.0470 | +0.0537 |
| Open High | +2.8 | -0.2 | -0.26 | -0.0034 | +0.0037 | -0.0003 | -0.0347 | +0.0400 |
| Warm High | +2.8 | -0.2 | +0.55 | +0.0843 | -0.0528 | -0.0315 | -0.0524 | +0.0596 |

### LANDR Interpretation

Warm is the clearest tone move:

- Adds low/body weight.
- Reduces high-frequency share.
- Narrows the stereo image.
- Preserves more macro movement than expected for a High loudness export.
- Maps best to YES Master `Tape` / `Warmth`, not `Oomph`.

Balanced is controlled rather than hyped:

- Gets loud but does not chase extra brightness or width.
- Slightly increases low share and reduces high share on this source.
- Maps best to YES Master `Universal` / Standard `Balanced`.

Open is not simply "more treble":

- Broad spectrum is nearly source-neutral on this track.
- The likely perceptual identity is punch, presence, and controlled density,
  not a static bright shelf.
- Maps best to a `Clarity` plus `Punch` hypothesis, not current Clarity alone.

## Calibration Implications For YES Master

### What To Use Now

- Keep Standard High / hot master as a valid comparison target for LANDR High.
- Use LANDR High as evidence for how to stay loud while preserving movement.
- Treat LANDR style names as style-plus-dynamics, not EQ-only presets.
- Keep YES Master's `-1 dBTP` Standard ceiling unless the product canon changes.
  LANDR's observed `-0.3 dBFS` true peak is useful competitor evidence, not an
  automatic safety target.

### What Not To Do Yet

- Do not retune `Oomph` from LANDR Warm. It is the wrong style match.
- Do not conclude "Open = add air shelf." The observed Open master stayed close
  to the original broad spectrum.
- Do not tune only EQ. LANDR's public docs and the measurements both point to
  compression/dynamics, loudness, width, and saturation as part of style.
- Do not call a retune complete without owner listening notes.

### Strongest Hypotheses

| YES Master area | External evidence | Hypothesis to test by ear |
| --- | --- | --- |
| Universal / Balanced | BandLab Universal air; LANDR Balanced controlled/tighter | Default should be polished and controlled, not wide/bright by default |
| Clarity | BandLab Clarity mid scoop + air; LANDR Open punch/presence | Clarity may need clearer mid cleanup, but Open-like punch might belong in Punch |
| Tape / Warmth | BandLab Tape density; LANDR Warm smooth/body | Tape should carry glue/density; Warmth should carry smooth body and softer top |
| Oomph | BandLab Oomph sub lift + mid scoop | Keep Oomph anchored to the BandLab/reference-set data until new heavy-style competitor files exist |
| Adaptive compressor | LANDR High preserved movement better than a naive crush | Per-band reduce-only adaptive compression is likely more important than static EQ retune |

## Acquisition Protocol For Future Brands

Use owned/private tracks only. Keep audio and rendered masters out of git.

For every service and every test source:

1. Upload the exact same source WAV.
2. Download lossless WAV output if available.
3. Preserve the original export filename or use:
   `BRAND-track-slug-style-loudness.wav`.
4. Capture the service's style/loudness notes as a screenshot or text note.
5. Record:
   - service name
   - date exported
   - account tier if relevant
   - style/preset name
   - loudness/intensity setting
   - output format
   - whether volume match/reference matching was enabled
   - any reference track used

Recommended source set:

- One low-dynamic-range/already-mastered source, like `It’s a coat`.
- One dynamic normal mix with real transients.
- One bright/thin source.
- One bass-heavy or dense source.
- Optional long track for responsiveness and long-form dynamics.

Recommended export matrix:

| Goal | Minimum capture | Why |
| --- | --- | --- |
| Hot-master parity | All styles at max/high/loud | Compares competitor loud path to YES Standard High |
| Style identity | All styles at medium/default | Separates style from maximum limiting |
| Loudness curve | One style at low/medium/high | Shows what the loudness control changes independent of style |
| Safety/stand-down | Already-mastered source at high | Shows how much the service backs off or overcooks dense material |

For the current LANDR folder, the High exports are useful and intentional. The
next LANDR downloads should add at least one Medium set and one Low/Medium/High
ladder for the same style, probably Balanced or Open.

## Tooling Gap

The current `private_reference_tuning` runner assumes exactly four references:

- `universal`
- `clarity`
- `oomph`
- `tape`

To make multi-brand research clean, add a separate external-master benchmark
lane instead of overloading the preset-tuning lane:

- Input: `tests for presets/manifest.json` or filename convention.
- Rows: brand, style, loudness, source, file path, measurements, deltas.
- Output: ignored JSON/CSV under `test-output/external-master-benchmark/`.
- Doc update: aggregate findings only, not private audio or full generated
  ledgers.

That lets BandLab, LANDR, eMastered, CloudBounce, BandLab Members, Ozone, and
other services accumulate into one comparable dataset without forcing every
brand into YES Master's preset enum.
