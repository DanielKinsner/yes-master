# Reconciliation of the mastering findings (unblinded follow-up)

Date: 2026-09-15 UTC (2026-09-14 evening Pacific). Task: `FOLLOWUP_REVIEW.md`.
This report is separate from the sealed blind report; the 18 blind-phase files
were re-verified before work started and are unchanged (see
`SEALED_MANIFEST.json` here and `EVIDENCE_MANIFEST.json#original_seal_verified`).
Production source, defaults, calibration and gates are unchanged. No listening
claims are made. Everything below is a measurement on the production
implementation unless marked as emulation.

## 0. Conditions, matching and provenance

- **Source.** Main checkout `yes-master` at `2c71e35`, no working-tree changes.
  `git diff a4fb621..HEAD -- src-tauri src` is empty: main is still the
  baseline revision both investigations started from. The review snapshot's
  `src-tauri/src` differs from main only by CRLF line endings in six non-DSP
  files (`audio.rs`, `diagnostics.rs`, `fixture_matrix.rs`,
  `listening_bench.rs`, `main.rs`, `sources.rs`) and `Cargo.lock`; `dsp.rs`,
  `engine.rs`, `sample_rate.rs` are byte-identical. Locked `rubato` is 1.0.1
  in both.
- **Fixture mapping verified by SHA-256.** input01 = coat, input02 = piano,
  input03 = aphelion, input04 = funk, input05 = metal, input06 = baby,
  input07 = rich, input08 = imaginal (Codex `sources/*.wav` and the owner
  file are byte-identical to the fixtures).
- **Harness.** `harness/` is the blind harness plus: a verbatim copy of the
  production `convert_interleaved` (it is `pub(crate)`), an independently
  written corrected SRC loop, Codex's `src_fixed.rs` copied unchanged
  (SHA-256 `ece0007a…d579e`), a `stats` command (engine compressor
  gain-reduction meters + limiter gain reduction by bypass ratio), and a
  `sat=<amount>` coefficient override. The verbatim SRC copy was proven
  bit-identical to a real production export (§1). All renders use the
  production export function `mastering_render_to_path`.
- **Settings matched to Codex** for every replication: Universal **75 %**,
  requested density 0.5, Adapt 0.5, -1 dBTP, chain at source rate, delivery
  48 kHz PCM24, targets -9 and -14 via the Custom profile (identical
  pre-landing coefficients for both targets, Volume Match off). The blind
  matrix used Universal 50 %; where a blind number is quoted it is labelled.
- **Meters.** engine = the crate's ebur128 (what the limiter, landing and
  receipt use; 4x at 48 kHz); bs1770 = the 48-tap 4-phase FIR published in
  ITU-R BS.1770-4 Annex 2, own NumPy; ffmpeg = `ebur128=peak=true`;
  soxr16 = Codex's `aresample` 16x soxr precision 33; fftexact16 = chunked
  FFT zero-padding 16x (ideal band-limited interpolation; chunk-size
  independent to 0.001 dB). On the EBU Tech 3341 true-peak test signals
  15–19 all five meters are within the +0.2/−0.4 dB tolerance except soxr16
  on signal 19 (+3 dBTP), which FFmpeg clips to 0 dBFS
  (`results/tp_meters_synth.json`).
- **Primary references** (text extracted, `refs/`): ITU-R BS.1770-4 Annex 2
  Appendix 1 table: maximum 4x under-read 0.554 dB at f = 0.45 fs, 0.688 dB
  at 0.5 fs; 8x 0.136/0.169 dB; 16x 0.034/0.042 dB. EBU Tech 3341 §2.6:
  true-peak tolerance +0.2/−0.4 dB including passband ripple and under-read.
- **Runtime.** Same machine as the blind phase (i9-13900K, Rust 1.95.0,
  Python 3.13.5, NumPy 2.2.6, SciPy 1.17.1, FFmpeg 7.1.1 with soxr).

## 1. Claim table

| Claim | Exact matched test | Result | Verdict | Cause of any disagreement | Recommended action | Confidence / remaining limit |
| --- | --- | --- | --- | --- | --- | --- |
| **1a** Rubato `process_all_into_buffer` copies the wrong number of frames; 48→44.1 kHz leaves output frame 955 wrong (~0.43 error on a 0.5-peak 100 Hz sine) | Locked rubato 1.0.1 `src/lib.rs` read; own generator (0.5-peak sines, 4 frequencies, 20 lengths straddling chunk boundaries, 6 rate pairs, 1224 cases); analytic-sine residual per sample (`results/src_probe_summary.txt`) | Code: `copy_frames_within(frames_to_trim, 0, frames_to_trim)` copies `delay` frames instead of `output_len − delay`. Rubato reports 48→44.1: block 1911, delay 955 → 956 useful frames, one stale. Measured error at index 955 = **0.430** (100 Hz), 0.642 (19 kHz), second-largest error 1.7e-3; 36/36 long cases. Also **96→44.1 kHz** (block 1029, delay 514): error 0.579 at index 514, 12/12 cases, a pair Codex did not test. The five other pairs are clean (block = 2·delay). | **Confirmed and extended** | — | Fix the loop (any of: rubato upstream fix, the independently written loop, or Codex's). Regression test: both odd-block pairs, a hot-start clip. | High. Shared limit of every corrected loop: the true delay on the affected pairs is 955.5/514.5 frames, so the corrected output is offset by exactly +0.5 sample (fit-derived, `time_offset_samples`); inaudible, but "exact" means exact to half a sample. |
| **1b** Very short conversions retain start-up delay and become nearly silent; floating frame count adds a frame | Same probe; production returns `ceil(ratio·n)` in f64 | Inputs shorter than one input block (2058 frames at 44.1 kHz ≈ 47 ms; 2080 at 48 kHz) are never trimmed: output RMS/reference 0.00–0.42 (prod) vs 0.94–1.00 (corrected); 44.1→48 and 44.1→96 add one frame for every length that is an exact multiple (40/408 prod lengths wrong, 0/408 for both corrected loops). | **Confirmed** | — | Same fix; add the length assertion to the SRC unit tests (the current test tolerates ±2 frames). | High. |
| **1c** Isolated correction (Codex `src_fixed.rs`) is correct at boundaries | Same 408 cases through Codex's loop and my own loop | Both loops: 408/408 exact lengths, residual −128 to −137 dBc on all long cases, identical to each other and to production on the clean pairs. Boundary lengths (k·block ± 1) pass. | **Confirmed** | — | Either loop is acceptable; keep the +0.5-sample note. | High. |
| **1d** It is a real defect on the shipped path, distinct from filter regrowth | input08 (48 kHz) → Custom, no target, 44.1 kHz, 32-bit float export vs chain → verbatim copy: **bit-identical** (15,598,577 frames). Hot-start 48 kHz clip (input04 resampled by soxr, 60–120 s) → CD profile 16-bit -14 LUFS (`results/src_music_checks.json`) | Delivered CD file has a single wrong sample at frame 955 (21.66 ms): error 0.057 (**−24.9 dBFS**) against a −3 dB landing; on whole-track exports that start quietly the error is small (0.0011 on input08) because the stale value is the sample at t = 0. Nothing to do with regrowth. | **Confirmed** | — | Fix as above. | High. Blind phase never exercised 48→44.1 or 96→44.1 (only 96→48 and same-rate 44.1), so the blind report could not have found it. |
| **2a** 25/192 pilot deliveries fail the FFmpeg ≤ −0.9 dBTP criterion (24 Imaginal + funk `continuous_drive_t14`) | All 25 retained WAVs metered with five meters (`results/tp_meters_failing_set.json`) | 25/25 read > −0.9 by FFmpeg. Engine −1.000 on all; exact peaks −0.38 … −0.62 dBTP; engine under-read 0.44–**0.62 dB**. Imaginal is 48 kHz native (no SRC): pure detector under-read within the BS.1770-4 stated 4x bound (0.55/0.69). soxr16 vs fftexact differ by up to 0.16 dB (drive_t14: −0.216 vs −0.380), i.e. the soxr reference is not ground truth either. | **Confirmed** (magnitude narrowed: max exact excess over −1 is 0.62 dB) | — | Final-rate true-peak verification with ≥ 8x oversampling (bound 0.17 dB) or 16x (0.04 dB) and a downward trim; report the receipt from the same estimator. | High for the numbers. Whether a 0.5 dB excess is audible is not claimed. |
| **2b** Blind report: Custom + no target + 44.1→48 kHz clips the delivered file; short inputs skip landing | Production `render` on input04, Universal 50 (blind settings), Custom, target null, 48 kHz, **24-bit** and 32-bit float; 50 ms clip, Streaming (`results/tp_meters_blind_defects.json`) | Pre-quantisation samples reach **+0.485 dBFS** (4 samples > 1.0); the 24-bit file the user receives is clamped at **0.000 dBFS**, exact TP +0.77; receipt reports TP 0.0 with `true_peak_high`. 50 ms clip: engine −0.777, exact −0.568, no landing (loudness unmeasurable). UI reachability: the loudness slider's reset (`onLoudnessTarget(null)`) and Advanced reset write `lufs_offset_db: null` under Custom, which `effective_target_lufs()` maps to "no landing", so the ceiling trim is skipped too. | **Confirmed** on the user-received file | — | Apply the downward ceiling trim after SRC whenever a ceiling is effective, independent of the loudness target and of whether loudness could be measured. | High. |
| **2c** "SRC regrowth": rubato converts a −1.10 dBTP limited file to +1.04 dBFS; soxr/polyphase only to −0.21 | Funkorama chain output (Universal 75) at 44.1 kHz and its conversions by production rubato, my loop, soxr (precision 33) and swresample; five meters; near-Nyquist band energy; stage ablation (`results/tp_meters_regrowth_input04.json`, `results/near_nyquist_ablation_input04.json`, `results/rubato_response.json`) | Reproduced exactly (+1.043 rubato, −0.215 soxr, −0.148 swr). But the **pre-SRC 44.1 kHz file itself has an exact band-limited peak of +1.66 dBFS** while the engine reads −1.10: the peak (t = 68.82 s, half-sample position) is formed by 21–22.05 kHz content (−33.8 dBFS in that band, higher than 20–21 kHz). Removing content above 21 kHz drops the exact peak to −0.02 dBTP. Ablation: source MP3 already has a 21–22 kHz bump (−41.6 dBFS); the chain's HF shelves raise it; the saturator adds ~2.3 dB more there (exact TP 0.28 with `no_sat` vs 1.66); transient shaper negligible. Rubato's filter is flat to 0.99 Nyquist (0.000 dB ripple, −123 dB at Nyquist) and interpolates these peaks faithfully (+1.09 exact at 48 kHz); soxr's transition band attenuates 21–22 kHz and hides them. | **Narrowed / cause changed** | Both reports attributed the +2 dB to the resampler. It is mostly the limiter's 12-tap 4x detector missing intersample peaks made of near-Nyquist content (up to 2.8 dB here, beyond the tone bound because transient/near-fs/2 content is unbounded per BS.1770-4 App. 1), exposed by a wideband resampler. | Address at the source: detector/verification at ≥ 8x, and treat near-Nyquist energy (un-oversampled saturator, HF shelves on lossy sources) as the driver. Do not "fix" by choosing a gentler SRC filter; that only hides the peaks. | High for the mechanism on this file; measured on one lossy-source track plus Imaginal. Audibility of near-Nyquist ISPs on real DACs is not claimed. |
| **2d** Blind report: a 0.5 dB internal margin keeps FFmpeg ≤ ceiling | Complete 25-file failing set + the synthetic ISP cases + Codex's centred −9 Imaginal outputs (§3) | Max engine under-read 0.62 dB (imaginal drive_t14) → a gain-only 0.5 dB margin leaves exact ≤ −0.9 in 21/25 and ≤ −1.0 in only 9/25; Codex's centred −9 Imaginal outputs (replicated) reach exact −0.11 and **+0.05 dBTP** (engine −1.24/−1.00), needing > 1 dB. | **Refuted as a correction; downgraded to an empirical margin** (changed after unblinding) | The blind set only contained under-reads ≤ 0.5 dB. | Replace with a demonstrated estimator (≥ 8x, ideally 16x) at the final rate; a margin is at best a stop-gap. | High. |
| **2e** Meter feed-block size ruled out | `peak-verification.json` (64 … full file) | Codex's blocks 64–16,978,042 give identical native readings; my `measure` feeds the whole file and matches the receipt to 1e-6. | **Confirmed** | — | — | High. |
| **3a** Codex centred search reaches −14 in 12/12 with per-song crest spread < 0.013 dB; at −9 gain bounds and peak failures remain | Own implementation of the rule (centre −18 − source LUFS, offsets −12…+6, clamp ±24, least drive within 0.2 LU else min error, tie → lower drive), grid points via chain → verbatim SRC → engine meter → production landing rule, finalists through production `render`, 96 finalist renders + 221 grid points (`results/policy_summary.txt`, `results/policy_compare.json`) | −14: **16/16** conditions within 0.2 LU (coat/aphelion at −14.12/−14.19 are ceiling-bound), exact TP ≤ −0.9 in 16/16, crest spread across 0/−6/−12 dB copies **0.000–0.012 dB** (Codex 0.013). −9: 13/16 reached; rich −12 dB and imaginal −12 dB hit the +24 dB clamp (−10.45/−9.63); Imaginal −9 outputs have exact TP −0.11, −0.10, +0.05 (engine −1.24/−1.00). | **Confirmed** | — | See §2 recommendation. | High for consistency; the sources are reused, not holdouts. |
| **3b** Blind rule A (attenuate-only) vs C vs production, fairly | Same matrix, same metrics (`results/policy_summary.txt`) | −14 (n = 16): A hits the target in 7/16 (= P; A never changes a target-missed case), crest +1.11 dB mean (0 … +5.63), section contrast never lower (+0.34 mean), attack/body +0.64 dB, tone deviation −0.07 dB, limiter max GR −0.49 dB, M/S within 0.02 dB of P (C within 0.16 dB; the preset widener's +0.6…0.9 dB dominates all three). C hits 16/16 but crest −2.45 dB mean (rich 22.99 → 16.47, imaginal 22.16 → 15.19), section contrast −0.37 mean (min −3.14, rich −6 dB), attack/body −0.89, tone deviation +0.16 dB; C is more dynamic than A on hot sources (coat 15.55 vs 14.06; metal 14.35 vs 12.49) because it selects zero limiter activity. −9 (n = 16): A barely moves (crest +0.16 mean; only coat and metal at 0 dB attenuate); C reaches 13/16 at the cost of LRA −1.81 LU, section contrast −1.95 dB mean (rich −7.12), attack/body −3.21 dB, tone deviation +1.05 dB, limiter max GR +2.29 dB, 3 exact-TP failures. Piano regression reproduced: C at −9 needs +3.56 dB drive: crest 12.59 → 10.29, LRA 10.3 → 8.4, contrast −1.85 dB (Codex: +4.5 dB, 12.59 → 9.77, 10.3 → 7.8; the difference is Codex's absolute grid vs the centred grid). | **Confirmed (Codex) / narrowed (blind)**: A is a safe subset of C on hot sources and does nothing for quiet ones; C honours the target on quiet sources at a dynamics cost the target implies. | — | See §2. | High on the 16 conditions. Reused sources; no listening. |
| **3c** Blind synthetic regression (A under-shoots on Loud/drums) and the proposed iteration | Production path, Loud 50 % on the synthetic drum train at −14 (`results/rule_a_drums_iteration.json`) | A: g = −1.86 dB → **−14.91 LUFS** (0.91 LU short, reproduced). One bounded iteration → −14.48; a second → −14.24. Iteration converges slowly because the reduced drive lowers limiter/saturator loudness gain non-linearly while the ceiling stays bound. | **Confirmed; iteration is not a resolution** (changed after unblinding) | — | Do not call the iterated rule resolved; a bounded search with a feasibility check (as C does) is needed. | High. |
| **3d** Runtime | Same runs | Grid point (chain + SRC + measure) 7.4 s single-threaded on a 3–6 min track; production render 4–18 s. P = 1 render; A = 1 grid point + 1 render; C = 13 grid points + 1 render (≈ 100 s serial, parallelisable, source-rate chain only). | Measured | — | A cached-analysis bounded search of 3–5 points is affordable; 13 is not for audition. | Medium (this machine only). |
| **4a** Saturation mapping is discontinuous at zero: +2.357 dB fundamental gain for ε amount, +2.922 at 0.0715, halving does not halve | Production chain with all other stages disabled and `sat=` override; −24 dBFS 1 kHz at 44.1 kHz (`results/sat_probe.json`) | +2.357 (ε), +2.637 (half), **+2.922** dB (0.0715), bypass 0; analytic small-signal gain (1+2a)/tanh(1+2a) = +2.365/+2.647/+2.935 dB; native vs analytic ≤ 1.0e-7 (Codex 2.2e-7). Code confirms `if amount > 0`. | **Confirmed** | — | Voicing decision for the owner (continuous identity point); not adopted here. | High. |
| **4b** Aliasing: full-scale 11 kHz at 44.1 kHz gives −21.64 dBc folded (current), −22.51 (half), −23.47 (ε); 4x/8x reference −79.89/−80.60 dBc | Same probes; oversampled references with SciPy default (Kaiser β 5), Kaiser β 14, and exact FFT-domain oversampling | Native −21.64/−22.51/−23.47 dBc reproduced exactly (tanh is odd; the folded components are the 3rd/5th/… harmonics at 33/55 kHz landing near 11 kHz). Reference: −79.89/−80.60 dBc with SciPy defaults (Codex's numbers), but **−113.9 dBc** (the numerical floor, equal to bypass) with a Kaiser-14 window or exact FFT oversampling at 4x, 8x and 16x. | **Confirmed; the reference figure narrowed** | Codex's −80 dBc is the stopband floor of the default 81-tap Kaiser-5 filter, not the oversampled curve's residual. | Any antialiasing work should quote its own filter; the achievable rejection at 4x is not limited to 58 dB. | High. |
| **4c** Gentle continuous curve `tanh(2ax)/(2a)` at matched −9 on coat: +1.5 dB drive, limiter max/mean 6.60/0.97 dB, LRA 2.6 (control 3.1) | Emulation validated to 3.6e-7 against the full chain: chain(no_sat, no_limiter) → NumPy curve → limiter-only chain → verbatim SRC → landing (`results/sat_interaction_coat_t9.json`) | Control: −9.00, crest 8.95, LRA 3.1, limiter 2.46/0.43. Continuous +1.5 dB: −9.00, crest 10.40, **LRA 2.6, limiter 6.60/0.97**, active 87.5 % (control 83 %). Continuous +0 dB: −9.50 (missed), crest 11.12, LRA 3.4, limiter 5.22/0.40. | **Confirmed** | — | Do not adopt the curve from THD/alias numbers alone; the loudness has to come from somewhere. | High (one track, one target). |
| **5a** Keep Density and Adapt as separate functions with current defaults | Coefficient resolution on coat: density 0.25/0.5/0.75 × Adapt 0/0.5/1 (`results/density_adapt_coeffs_input01.json`); blind coefficient checks | Density sets the preset threshold/ratio/makeup (−6.25/1.225 → −12.5/1.45 → −14/1.7); Adapt eases them (0.5: −8.46/1.305, makeup 1.94 → 0.99 dB; 1.0: −5.0/1.18), matching the owner document. On coat Adapt does not touch EQ or width (in range); on bright/boomy/wide synthetics (blind A9) it trims EQ/width. Compressor Off removes crossover and makeup as well (Codex/owner caveat agreed). | **Confirmed** | — | Retain both; no default change proposed. | High for mechanisms; "current defaults are right" is an owner voicing question. |
| **5b** Blind "UI compressor read-out passes" vs the adapted case | `compressorAutoReadouts` (unadapted formula) vs engine-resolved coefficients at Adapt 0.5; code path of `compression_plan_for_resolved_settings` and the panel | The idle row shows −12.5 dB · 1.4:1 for coat at density 0.5 while the engine runs −8.46 dB · 1.305:1 with half the makeup. The adapted plan display is only active behind the gated Adaptive Compressor (`is_adaptive_compression_enabled()`, default off), and the per-axis Adapt read-out is behind a localStorage debug flag. | **Narrowed** (blind UI-1 verified only the unadapted formula, which is exact) | — | Scoped correctness fix: show the Tier-1-resolved values (or label the row "preset, before Adapt"). Not a UI design project. | High. |

## 2. Ranked recommendation (experimental; behind the existing interface)

1. **Fix the SRC loop** (claim 1). Objective, reproducible, one function.
   Regression: both odd-block pairs (48→44.1, 96→44.1) on a hot-start clip
   and a 10 ms clip; exact-length assertion. Smallest slice: replace the
   `process_all_into_buffer` call in `sample_rate.rs` with the explicit loop
   (production behaviour otherwise unchanged; the +0.5-sample offset remains
   and should be documented).
2. **Always apply the downward ceiling trim after SRC, and verify the final
   peak with ≥ 8x oversampling** (claims 2b, 2a, 2d). Smallest slice: in the
   export path, run the ceiling trim whenever an effective ceiling exists
   even when the target is `None` or loudness is unmeasurable; use a 16x
   estimator (FFT or long polyphase) for that trim and for the receipt.
   Cost: up to ~0.6 dB less loudness on ceiling-bound files (Imaginal), up
   to ~2.7 dB on the Funkorama-type case where the limiter detector misses
   near-Nyquist peaks; the receipt becomes honest. This replaces the blind
   0.5 dB margin proposal.
3. **Automatic drive: bounded, source-and-target-aware search** (claim 3).
   The already-selected loudness target is the user's intent: for a quiet
   stem mix the target says "bring it up" and C does that at a dynamics cost
   the target implies; for a hot source it says "do not crush then turn
   down", where A and C agree and C preserves more. Recommended policy:
   Codex's selection rule with two changes justified by the −9 rows:
   (i) bound the search by feasibility at the final rate with the 8x/16x
   estimator (the −9 Imaginal exact +0.05 dBTP outputs must not be
   selected), and (ii) cap allowed drive above unity (e.g. +12 dB) so that
   a target-missed result is returned honestly instead of rich −12 dB at
   +24 dB with a 7 dB section-contrast loss. Keep it experimental; a 3–5
   point search from cached analysis is affordable, 13 is not. Explicit
   costs: at −9, LRA −1.8 LU, section contrast −2 dB, attack/body −3 dB and
   +1 dB tonal deviation on average versus production; those are the price
   of reaching a −9 target on sparse material, not a defect. The blind
   attenuate-only rule is a strict subset (hot sources only) and its
   synthetic under-shoot is not fixed by iteration.
4. **Compressor read-out correctness** (claim 5b): show the Tier-1-resolved
   threshold/ratio in the idle row or label it. Keep Density and Adapt.
5. **Saturation** (claim 4): continuity at zero and antialiasing are real
   mechanisms; the gentle curve moves work into the limiter and lowers LRA at
   matched loudness. Owner voicing decision; measure any candidate with the
   limiter interaction, not THD alone. The alias reference should use a
   proper filter (achievable rejection at 4x is > 100 dB on the test tone,
   not 58 dB).

**Explicit regressions of the recommended items.** (2) lowers delivered
loudness on ceiling-bound files by the true under-read (0.4–0.6 dB typical,
up to 2.7 dB on near-Nyquist-heavy lossy sources). (3) reduces crest by up to
12 dB and section contrast by up to 3 dB on quiet stem mixes at −14 and more
at −9 (that is the target being met); it may return "target missed" on very
quiet sources; it costs 3–5 extra source-rate chain passes per export.

**Conclusions changed after unblinding.** (a) The 0.5 dB margin is not a
demonstrated correction (2d). (b) The "SRC regrowth" is mostly detector
under-read of near-Nyquist intersample peaks, not a rubato filter flaw (2c).
(c) The attenuate-only rule's synthetic under-shoot is not resolved by a
bounded iteration (3c). (d) The blind UI-1 pass was too broad; the idle
read-out is inaccurate under Adapt (5b). (e) 96→44.1 kHz shares the SRC
defect (1a), which neither report had tested.

## 3. Detailed evidence pointers

- Claim 1: `results/src_probe_summary.txt`, `results/src_probe_analysis.json`,
  `results/src-probe/` (1224 small WAVs + `probe.json`),
  `results/src_music_checks.json`, `results/rubato_response.json`,
  `results/src-impulse/`.
- Claim 2: `results/tp_meters_synth.json` (Tech 3341 signals),
  `results/tp_meters_codex_finalists.json`, `results/tp_meters_failing_set.json`
  + `results/tp_failing_set_summary.json`, `results/tp_meters_blind_defects.json`
  (+ receipts `renders/defect_*.json`), `results/tp_meters_regrowth_input04.json`,
  `results/near_nyquist_ablation_input04.json`.
- Claim 3: `results/policy_compare.json` (grid, selection, 96 finalists with
  receipt/independent metrics/stage stats/hashes), `results/policy_summary.txt`,
  `results/policy_ms_check.json`, `results/rule_a_drums_iteration.json`,
  logs `results/policy_compare_full.log`.
- Claim 4: `results/sat_probe.json`, `results/sat_interaction_coat_t9.json`.
- Claim 5: `results/density_adapt_coeffs_input01.json`; blind
  `audit-output/results/coeff_checks.json`.
- Every deleted intermediate render is listed with SHA-256 in
  `results/deleted_renders_hashes.json`; all are reproducible from the
  scripts and fixtures.

## 4. Coverage limits and open questions

- Eight pieces, two lossy sources, constructed stem mixes; no listening.
- Attack/body, section contrast and short-term spread use Codex's window
  definitions re-implemented here; they are window statistics, not
  perceptual scores.
- Exact true peak is defined on the band-limited reconstruction of the
  samples; DACs and lossy encoders treat 21–22 kHz content differently, so
  the practical exposure of the Funkorama-type peaks is unknown.
- The recommended search (§2 item 3) was not implemented; its −9 behaviour
  with a feasibility bound is inferred from the grid rows, not rendered.
- Only Universal was compared; other presets were not re-run at 75 %.

## 5. Cleanup inventory

Computed by `scripts/inventory.py` (`results/cleanup_inventory.json`); nothing
was deleted beyond the hash-then-delete of this task's own intermediate
renders.

| Category | Location | Size | Disposition |
| --- | --- | ---: | --- |
| Reproducible build caches | `audit-output/build/target`, `audit-output/reconciliation-20260915/build/target`, Codex `test-output/…/target`, `pylibs` | 1.62 + 1.63 + 0.20 + 0.13 GB | Rebuildable from the locked manifests; safe to delete after the binaries' hashes are recorded (they are, in both evidence manifests). |
| Duplicate inputs | Codex `test-output/…/sources/*.wav` (7 files) and `tests for presets/It's a coat-original-test.wav` are byte-identical to `fixtures/inputs/input0N.wav` | ≈ 0.73 GB | Keep one copy (the fixtures with `manifest.json`); the others are duplicates. |
| Derivable variants | `audit-output/variants` (gain/rate copies), `audit-output/synthetic`, `reconciliation-20260915/renders/policy_inputs` | see JSON | Regenerated by the blind scripts / `policy_compare.py`; deletable after preservation. |
| Unique evidence, blind | the 18 sealed files, `audit-output/results`, `specs`, `cases`, `EVIDENCE_MANIFEST.json`; retained blind renders (10.45 GB, hashed in the blind manifest) | ≈ 10.5 GB | Keep the small set always; the retained renders are reproducible but were part of the blind seal's evidence manifest, so keep until an archive exists. |
| Unique evidence, reconciliation | `reconciliation-20260915/{RECONCILIATION.md, SEALED_MANIFEST.json, EVIDENCE_MANIFEST.json, results, refs, scripts, harness/src}` (0.13 GB) and retained renders (≈ 1 GB after pruning) | ≈ 1.2 GB | Keep the small set; retained renders (defect cases, hot-start CD file, SRC probe WAVs, tone probes) are reproducible. |
| Codex evidence | compact set in git (`docs/reviews/evidence/2026-09-15-mastering-quality`); full private set `test-output/mastering-quality-20260915` (21.9 GB incl. 192 finalists) | 21.9 GB | Unique retained audio/logs; not touched. The 25 files re-metered here are hashed in this manifest. |
| Minimum archive for another machine | fixtures + manifest, `src-tauri` snapshot (or checkout at `a4fb621`), the two small evidence sets above, this report, the harness sources and lockfiles, toolchain notes | < 2 GB with fixtures | Everything else rebuilds or re-renders. |
