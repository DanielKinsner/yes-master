# Evaluation protocol — YES Master independent engine audit

Written 2026-09-14 before any render, measurement or candidate selection.
Source survey only (reading `src-tauri/src/*.rs`, `types.rs`, `engine.rs`,
`dsp.rs`, `guardrails.rs`, `analysis.rs`, `wav_writer.rs`, `sample_rate.rs`,
`exports.rs`, the frontend `compressor-auto.ts`) preceded this document so the
promises could be stated in the engine's own terms. No output has been
rendered or measured at the time of writing. The held-out subset below is
fixed here and is not consulted while forming any recommendation.

## 0. Scope and what is held fixed

* Production source under `src-tauri/` is read-only. Nothing in it is edited.
  Feature gates stay at their shipped defaults: adaptive compression gate
  OFF, confidence gate OFF, album character OFF. Adapt strength default 0.5.
* Harness: `audit-output/harness` (separate Cargo package, path dependency on
  the unmodified crate, `default-features = false` = the mobile-bridge
  feature set; the DSP, analysis, SRC, landing and WAV writer do not depend
  on the skipped desktop features). Lockfile copied from `src-tauri/Cargo.lock`.
  Release profile (`opt-level 3`). Deviation from the desktop build is
  recorded as a packaging adaptation and is checked by reproducing the
  supplied baseline export byte-for-byte (§6 control C1).
* Scripts under `audit-output/scripts`, results under `audit-output/results`,
  renders under `audit-output/renders` (not sealed; hashes recorded in the
  evidence manifest instead of the audio bytes).

## 1. Requirements — what the engine promises

Derived from settings semantics, receipt fields and export checks. Each is
tagged **objective** (testable correctness requirement) or **musical**
(intentional choice; only direction/consistency is checked, not merit).

| id | Promise | Kind | Where it is made |
|----|---------|------|------------------|
| R1 | Delivered true peak never exceeds the effective ceiling (profile or Custom `ceiling_dbtp`). | objective | `ceiling_bounded_landing_delta_db`, limiter, `true_peak_high` check |
| R2 | Delivered integrated loudness equals the effective target, except when the ceiling bounds it; then the receipt must say `target_not_reached` and the true peak must sit at the ceiling. | objective | `effective_target_lufs`, landing stage, `target_not_reached` check |
| R3 | The receipt's LUFS / true peak / LRA describe the delivered file (post-dither PCM), not the source. | objective | `RenderedMeasurements`, `measure_delivery` |
| R4 | Delivered format: requested sample rate, requested bit depth, TPDF dither on 16/24-bit, source length preserved and sample-aligned, finite samples. | objective | `effective_sample_rate`, `flush_render_tail`, `wav_writer` |
| R5 | Rendering is deterministic (same input + settings → identical bytes). | objective | dither RNG seeded per call; tests pin byte identity |
| R6 | Custom preset with neutral values, compression Off, Custom profile with no target, is an identity chain. | objective | `PRESET_CUSTOM_NEUTRAL`, `compression_active`, comments |
| R7 | Volume Match is audition-only and never changes the export. | objective | render forces `volume_match=false` |
| R8 | Stereo processing is channel-symmetric and mono-safe: swapped channels give swapped output; identical L/R stays identical. | objective | linked-stereo limiter/compressor, M/S width |
| R9 | Adaptation is reduce-only and level-invariant: guardrails only shrink positive preset moves, never below the 0.5 dB floor, never beyond per-axis caps; strength 0 is inert; the `SourceProfile` inputs are shares/ratios. | objective | `guardrails.rs` |
| R10 | Presets are distinct in stated directions (Oomph = more low, Spatial = wider, Loud/Punch = denser, Tape/Warmth = softer top). Intensity scales "how hard the preset works", 0.5 = full character, monotone. | musical (direction only) | `PresetCalibration`, `preset_scale` |
| R11 | Delivery profiles carry the documented targets (-14/-16/-14/-18/-10.5/-23/-24 LUFS and matching ceilings, rates, bit depths). | objective | `DeliveryProfile` |
| R12 | Edge inputs (silence, sub-lookahead length, DC, clipped/over-full-scale, near-Nyquist, mono, 96 kHz, > 2 channels) render without error, without NaN, and obey R1/R4. | objective | decode clamping, `flush_render_tail` |
| R13 | Live audition landing and export landing use the same rule (same measured level in and out). | objective | `preview_landing` shares `ceiling_bounded_landing_delta_db` |

Non-promises noted for context: the preset `target_lufs` field is documented
as intent only; the saturator, transient shaper and EQ voicings are musical.
The compressor threshold/ratio readouts shown in the UI
(`compressor-auto.ts`) mirror the engine's preset math and are a UI promise
checked in §4 (UI-1).

## 2. Coverage — test dimensions and why

Chosen before results. Dimensions are those that plausibly change a promise:

1. **Material** (8 supplied + synthetic). Supplied: 4 finished mixes
   (input02–05), 3 constructed stem sums (input06–08), 1 owner track with
   unknown history (input01). Synthetic, deterministic, generated by script
   with fixed seeds: pink noise (-20 dBFS RMS), 1 kHz sine at -20/-6/-1 dBFS,
   two-tone SMPTE IMD (60 Hz + 7 kHz, 4:1), 15 kHz and 19 kHz tones at
   -6 dBFS (aliasing probe), drum-like impulse train (exponential decays,
   120 BPM), a full-scale square wave, a signal with inter-sample peaks above
   0 dBTP, DC-offset pink noise, digital silence, a 50 ms clip, a 2.5 s clip,
   a -60 dBFS noise floor probe, and a decorrelated-stereo (wide) noise pair.
   Music is a convenience corpus; genre coverage is not claimed.
2. **Preset** — all 8 named presets + Custom neutral.
3. **Intensity** — 0.0, 0.25, 0.5 (default), 0.75 (baseline), 1.0.
4. **Delivery profile** — Streaming (-14/-1/48k/24), Apple (-16), CD
   (-14/-1/44.1k/16), Vinyl (-18/-3), Loud Rock (-10.5), Broadcast EU (-23),
   Custom as the baseline (-9/-1/48k/24), Custom with no target.
5. **Compression mode** — Preset (default density), Off, Manual with one
   explicit set, density 0.0 / 1.0.
6. **Adapt strength** — 0.0, 0.5 (default), 1.0.
7. **Source level** — 0, -6, -12 dB gain-only versions of the same input.
8. **Source sample rate** — 44.1k (native), 48k, 96k (resampled by FFmpeg
   from the same source, documented).
9. **Channels** — stereo, mono, dual-mono, swapped L/R, 6-channel fold.

Not every combination is run. The main matrix is dimension 1 × 2 at default
settings (Streaming profile, intensity 0.5) on the full-length supplied
inputs. Dimensions 3–9 are run on a fixed subset: input02 (finished,
dynamic), input04 (finished, dense), input06 (stem sum) and the synthetic
set. Full-length audio is used whenever a landing result is judged, because
landing depends on whole-track integrated loudness.

### Held-out subset (fixed now, not used while forming recommendations)

* input05 (Metalmania, finished, dense)
* input07 (Rich, constructed stem sum)
* synthetic drum train at -6 dB source level, 96 kHz

These are rendered once with the main matrix for completeness of R1–R4
reporting only; they are excluded from any tuning, candidate comparison or
selection. Any recommendation is re-checked on them last, after it is fixed.

## 3. Measurements (independent of the engine unless stated)

| Measure | Tool | Notes |
|---------|------|-------|
| Integrated LUFS, LRA, true peak | FFmpeg 7.1.1 `ebur128` filter (`peak=true`) | primary independent meter |
| Integrated LUFS, true peak | own NumPy BS.1770-4 implementation | second independent meter used to arbitrate disagreements |
| Integrated LUFS, LRA, true peak | `ebur128` crate through the harness | the engine's own meter (for R3 only) |
| Sample peak, DC, crest factor, RMS, P95–P10 of 100 ms RMS | NumPy | |
| Long-term 1/3-octave spectrum (Welch, Hann 8192) and its difference vs source | NumPy | tonal effect per stage/preset |
| L/R Pearson correlation, side/mid energy | NumPy | width |
| THD (harmonics 2–10) on sine, IMD (SMPTE), alias spur level on HF tones | NumPy FFT | distortion cost |
| Output length, cross-correlation lag vs source | NumPy | alignment |
| SHA-256 of output files | Python hashlib | determinism, reproduction |
| Chain coefficients (`ChainCoeffs` fields) | harness JSON dump | R9/R10 invariants without audio |

Loudness matching for any listening or waveform comparison is gain-only;
no other processing is added to make examples comparable.

## 4. Acceptance criteria (fixed before results)

| id | Criterion | Pass | Investigate | Fail |
|----|-----------|------|-------------|------|
| A1 (R1) | independent TP − ceiling | ≤ +0.1 dB | +0.1 … +0.3 | > +0.3 dB |
| A2 (R2) | \|LUFS − target\| when not ceiling-bound | ≤ 0.5 LU | 0.5 … 1.0 | > 1.0 LU |
| A2b (R2) | when short of target, receipt carries `target_not_reached` and TP ≥ ceiling − 0.15 dB | both true | one true | neither |
| A3 (R3) | receipt vs FFmpeg: LUFS / TP / LRA | ≤ 0.2 LU / 0.15 dB / 0.5 LU | up to 2× | beyond |
| A4 (R4) | rate, depth, finite; length = round(N·ratio) ± 2 frames; lag = 0 at same rate | all | length off by ≤ 1 ms | else |
| A5 (R5) | two renders → identical SHA-256 | identical | — | differ |
| A6 (R6) | identity chain, float stage: max abs diff = 0; delivered 24-bit: ≤ 1 LSB | | | |
| A7 (R7) | VM on vs off export: identical bytes | | | |
| A8 (R8) | swapped-channel output equals swapped output, and L==R in → L==R out: max abs diff ≤ 1e-6 (float) | | | |
| A9 (R9) | every guardrail multiplier in [1−cap, 1]; strength 0 → coeffs identical to no-profile; trims never raise a boost | | | |
| A10 (R10) | direction holds on ≥ 6 of 8 music inputs (e.g. Oomph low-band share > Universal's) ; intensity monotone in the coefficient dump | | | |
| A12 (R12) | no error, no NaN, A1 holds on every edge input | | | |
| A13 (R13) | preview landing gain vs export landing delta: ≤ 0.1 dB | | | |
| UI-1 | UI compressor readout formula equals engine coefficients for the same density | exact | | |

Distortion and aliasing results have no pass/fail; they are reported as
costs with the level at which they occur. A finding is called a **defect**
only if it violates an objective promise on a realistic input under ordinary
settings, and reproduces in a minimal case with a control that isolates it.

## 5. Controls

* **C1 Reproduction**: render input01 with the baseline settings from
  `fixtures/manifest.json`; expect byte-identical `export01.wav`. Establishes
  that the harness runs the production engine.
* **C2 Meter agreement**: FFmpeg vs NumPy vs crate on the unprocessed
  sources; disagreement beyond A3 tolerances disqualifies the meter, not the
  engine.
* **C3 Identity**: R6 chain on every source; validates decode → write.
* **C4 Gain-only**: Custom neutral + target -14 → the landing alone; checks
  landing math against a hand computation.
* **C5 Repeat**: every matrix render run twice (hash compare) on a subset.
* **C6 Level control**: -6 dB source version; separates "level-dependent
  chain drive" from adaptation.
* **C7 Competing explanations**: for any suspected defect, at least one
  alternative cause is tested (meter, SRC, dither, harness) before it is
  attributed to the engine.

## 6. Selection rule for any recommendation

A change is recommended only if, with the held-out subset untouched:

1. it fixes a defect under §4, or improves ≥ 2 objective or documented
   musical criteria on ≥ 6 of the 8 music inputs and the synthetic set;
2. it does not regress A1–A9 on any input;
3. its cost (distortion, LRA loss, crest loss, spectral deviation) is
   reported with numbers;
4. it then holds on the held-out subset.

Otherwise the recommendation is "keep current behavior" for that stage,
with the evidence that supports keeping it.

## 7. Order of work

1. Build harness; C1 reproduction; C2 meter agreement.
2. Synthetic controls (C3, C4, C5) and R12 edge inputs.
3. Main music matrix (8 inputs × 9 presets, defaults).
4. Dimension sweeps on the subset (profiles, intensity, compression,
   adapt strength, level, sample rate, channels).
5. Stage attribution (Q5) by coefficient overrides on the production chain.
6. Defect reproduction and competing-explanation tests.
7. Candidate alternatives only if §6 warrants; held-out check last.
8. Report, seal.
