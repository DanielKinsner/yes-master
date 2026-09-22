# YES Master — independent mastering-engine audit

Reviewer: Claude (Fable 5.1), acting as independent audio-DSP reviewer.
Date: 2026-09-14/15 (UTC). Workspace: `yes-master-independent-audit-20260915`.
Source baseline: Git revision `a4fb621d88a95b8af549467fb499943acb4274d5`
(`SOURCE_SNAPSHOT.json`); no implementation edits were made. Protocol:
`audit-output/PROTOCOL.md`, written before any render.

## 0. Exposure log (independence checkpoint)

No previous YES conclusions, reports, research documents, experiment code,
issues or PRs were encountered or requested. The only YES material read was
the supplied source snapshot, `REVIEW_INPUTS.md`, `fixtures/manifest.json`,
`SOURCE_SNAPSHOT.json`, `START_REVIEW.md`, `THIRD_PARTY_NOTICES.md` and the
crate's own inline tests and comments. Inline comments do mention earlier
internal findings (for example the "B2 landing-matrix finding" and dates of
owner listening decisions); those are treated as documentation of intent,
not as results, and nothing in this report relies on them. Every conclusion
below comes from measurements made in this workspace.

## 1. Ranked recommendations

| rank | recommendation | kind | confidence |
|------|----------------|------|------------|
| 1 | **Fix**: the ceiling promise is not enforced when the loudness landing is skipped. With a Custom delivery profile, no loudness target and a 44.1 → 48 kHz export, a hot source is delivered clipped at 0.0 dBFS against a -1 dBTP ceiling (§5.1). Root causes: the FFT resampler (`rubato`) overshoots limited waveforms by up to 1.6 dB, and the limiter runs before conversion. Minimal fix without touching calibration: always apply the ceiling-bounded *downward* trim after SRC, even when there is no target (the trim code already exists in `ceiling_bounded_landing_delta_db`; it is only skipped because `effective_target_lufs()` is `None`, and for sub-400 ms or gated-silent inputs because the LUFS gate returns 0). | proven defect, low-risk fix | high |
| 2 | **Change the automatic gain structure so the limiter's drive depends on the requested target, not on the source level** (§5.2, §7). Today the chain applies a fixed preset push (about +5 dB at Universal 0.5, up to +9 dB at Loud) before the limiter and lands the loudness afterwards by pure gain. Hot sources are therefore limited far beyond what the target needs and then turned down 3–8 dB; a source already at -14 LUFS (input03) loses 3 dB of crest and is then attenuated 4.4 dB. The same mix exported at 0 / -6 / -12 dB produces masters 4 LU apart in loudness and 6.5 dB apart in crest. The safe form is an **attenuate-only, target-aware pre-gain** (candidate A): it never pushes harder than today, changes nothing on quiet sources, and on every hot source preserved 0.5–4.9 dB more crest and 0.6–2.5 LU more LRA while landing at exactly the same loudness and ceiling. It can be applied today through the existing Input Gain control, so it fits the current workflow; the engine could compute it automatically from the audition-side measurement it already makes. | tradeoff resolved by evidence; no calibration change | high on the mechanism, medium on the exact gain rule (one synthetic case under-corrected by 0.9 LU; a damped or iterated rule is needed) |
| 3 | **Keep a true-peak safety margin or use a sharper detector.** The engine meets its ceiling by its own libebur128-style 4x meter, but FFmpeg's 4x meter reads the same files up to +0.5 dB above the ceiling on real music delivered at native 48 kHz, and +1 dB on Nyquist-rich synthetic material. A fixed internal margin of 0.5 dB (land to ceiling − 0.5) kept FFmpeg readings at or below the ceiling in every observed case at a cost of 0.5 dB loudness only when the ceiling binds. | measurement-method difference with delivery consequences | high on the discrepancy, medium on the exact margin |
| 4 | **Document, do not change**: the "Custom neutral" preset is not an identity chain; it applies a hidden +1.5 dB gain push (`baseline_gain_push_db` of the Custom calibration). With a -1 dBTP ceiling this limits any source peaking above -2.5 dBTP (a -1 dBFS sine loses 1.6 dB of peak). The rest of the chain is exactly identity when that push is compensated. Either set the Custom push to 0 (this changes the byte-identity golden for Custom) or state it in the UI. | intent/documentation mismatch | high |
| 5 | **Keep current behavior** for: loudness landing math and receipt accuracy; delivery-profile targets, rates and bit depths; determinism; Volume Match isolation from export; channel symmetry and mono handling; adaptive guardrails (reduce-only, capped, strength-0 inert, level-invariant inputs); preset directions and intensity monotonicity; the UI compressor read-out formula; preview/export landing parity; TPDF dither; sample-rate consistency of the EQ. All measured and passing (§4). | keep | high |
| 6 | **Musical costs to be aware of, no change recommended without listening**: the tanh saturator is the largest dynamics reducer in the chain (3.5–4.6 dB of crest on hot sources) and is not oversampled, so high-level HF content aliases (a 15 kHz tone at -6 dBFS produces a 900 Hz product 28 dB down); THD at Universal 0.5 is 0.16 % at -20 dBFS, 3.5 % at -6 dBFS, 8 % at -1 dBFS; the 24 Hz LR4 high-pass raises peaks by 0.4–1.3 dB on bass-heavy sources through phase rotation, which the limiter then removes; the multiband compressor's equal makeup gives a +1 dB low / +2 dB high tilt on dynamic material; Loud's static makeup raises quiet passages by 8.7 dB before landing. | engineering tradeoffs | high on the numbers |

## 2. What the engine promises (Q1)

The engine's user-visible contract, derived from settings, receipts and export
checks, is listed in `PROTOCOL.md` §1 (R1–R13). In short: a delivered true
peak at or under the profile ceiling; integrated loudness at the profile
target unless the ceiling bounds it, in which case the receipt says so; receipt
numbers describing the delivered PCM; requested rate and depth with TPDF
dither; deterministic, sample-aligned, finite output; Volume Match never in the
export; symmetric stereo processing; adaptation that only reduces preset
moves; distinct presets whose "amount" scales monotonically with Intensity.

Intentional musical choices (not judged on merit here): the preset EQ
voicings, saturation "warmth", transient shaping, width, the compressor
time constants, and the decision that the ceiling is a hard delivery spec
while the target is an intent that may be missed.

## 3. Conditions, build and reproduction

* Platform: Windows 11 Pro 10.0.26200, i9-13900K. Toolchain: cargo/rustc
  1.95.0, Python 3.13.5, NumPy 2.2.6, SciPy 1.17.1, libsndfile 1.2.0 via
  soundfile 0.12.1, FFmpeg 7.1.1 (`results/environment.txt`).
* Harness: `audit-output/harness` — a separate Cargo package with a path
  dependency on the unmodified `src-tauri` crate, `default-features = false`
  (drops the Tauri window/updater/LAME features, which the DSP does not use;
  this is the mobile-bridge feature set), release profile. Lockfile copied
  from `src-tauri/Cargo.lock` (ebur128 0.1.10, rubato 1.0.1, symphonia
  0.5.5, hound 3.5.1). **Packaging adaptation**: no frontend `dist`, no LAME.
* **Control C1**: rendering `input01.wav` with the manifest's baseline
  settings through the harness produced a file with SHA-256
  `639ac1dd…6022ce`, byte-identical to the supplied `export01.wav`. The
  harness therefore runs the production engine.
* The crate's own unit tests (`cargo test --lib --no-default-features
  --locked`, isolated target dir): 443 passed, 0 failed, 5 ignored.
* Commands: see `scripts/*.py` (each is self-describing) and `cases/*.json`.
  Ledgers: `results/{controls,edge,main,sweeps,tones}.json/.csv`;
  `results/stage_attribution.*`, `results/coeff_checks.*`,
  `results/candidate_{dev,heldout}.*`, plus probe outputs listed in
  `EVIDENCE_MANIFEST.json`. Renders are retained for the C1 control, the
  main matrix, the edge set and the candidate outputs; intermediate renders
  were hashed into `results/deleted_renders_hashes.json` and deleted after
  measurement because the workspace disk filled (they are reproducible from
  the cases files).
* Independent meters: own NumPy BS.1770-4 implementation and FFmpeg
  `ebur128`. Agreement with the engine's ebur128 crate on the unprocessed
  sources and on the baseline export: integrated loudness within 0.01 LU,
  LRA within 0.02 LU, true peak within 0.01 dB (44.1 → 48 k delivery).

## 4. Requirements results (Q2, Q3, Q4)

Legend: pass / investigate / fail per PROTOCOL §4.

| id | result | evidence |
|----|--------|----------|
| A1 true peak ≤ ceiling | **pass by the engine's own meter and by my 4x meter (≤ +0.03 dB) on all 72 main renders, all sweeps and all edge inputs except two**; **investigate** by FFmpeg (+0.2…+0.5 dB on the 48 kHz-native input08 renders) and by 16x reconstruction (+0.06…+0.14 dB on input08); **fail** for the no-target Custom/48 k path (§5.1, +1.0 dB, clipped) and for sub-400 ms inputs (clip_50ms: receipt itself reports -0.78 dBTP against -1.0, landing gate skipped). | `results/main_tp_and_overshoot.txt`, `results/tp16_input08.txt`, `results/limiter_detector_underread.txt`, `results/edge.json`, `results/probe_notarget48_*.json` |
| A2 loudness = target | **pass**: every render that could reach the target landed within 0.005 LU (independent meter) — 72/72 main, 70/70 sweeps, all profiles (-14, -16, -18/-3, -10.5, -23, -24/-2). Every shortfall carried `target_not_reached` with true peak at the ceiling (A2b pass, 26/26 cases). | `results/main.csv`, `results/sweeps.csv` |
| A3 receipt = delivered | **pass**: receipt vs independent: LUFS ≤ 0.005 LU, LRA ≤ 0.02 LU, true peak ≤ 0.02 dB on every 48 kHz delivery; on 44.1 kHz/16-bit CD and 48 k-native deliveries the receipt under-reads true peak relative to FFmpeg/16x (see A1). | ledgers, columns `A3_*` |
| A4 format / length / alignment | **pass**: requested rate and depth in every case (PCM_24 / PCM_16 for CD / FLOAT for 32); length = round(N·ratio) + 0…1 frames; identity control lag 0 and max-abs-diff 0.0; onsets coincide sample-exactly on the drum train. Cross-correlation "lags" of 80–170 samples seen on processed renders are the low-frequency phase rotation of the 24 Hz LR4 high-pass and the compressor's LR4 crossover (they appear at stage "+comp", not at "+limiter"), not a delay. | `results/controls.json`, `results/stage_attribution.txt` |
| A5 determinism | **pass** (identical SHA-256 on repeat, C5). | |
| A6 identity chain | **fail as stated, pass when the +1.5 dB push is compensated**: Custom neutral = exact +1.5 dB gain (max abs diff 0.0 with `input_gain_db = -1.5`); 24-bit delivery differs from source by one dithered LSB. | `results/controls.json`, `renders/probe` hashes |
| A7 Volume Match never exported | **pass** (identical SHA with VM on/off). | |
| A8 stereo symmetry | **pass**: swapped input gives swapped output to 2.4e-7 (one 24-bit LSB); dual-mono stays dual-mono; mono renders; 6-channel folds to stereo (LFE excluded) and renders. | `results/sweeps.json`, `results/edge.json` |
| A9 adaptation | **pass**: reduce-only, within caps (EQ 50 %, width 70 %, density 60 %), strength 0 byte-identical to no profile, 0.5 dB boost floor honoured, mono never trims width. Note: the density trigger reads steady-state pink noise as "maximally dense" (P95–P10 ≈ 1 dB), and on the dense stem sum input06 adaptation lowered the delivered loudness by 1.1 LU (-15.28 → -16.35) by softening compression. | `results/coeff_checks.txt` |
| A10 preset direction / intensity | **pass**: Oomph highest 31–63 Hz share and narrowest, Spatial widest (lowest correlation, highest side/mid), Loud lowest LRA and crest, Punch second, Tape/Warmth lowest 8–16 kHz, Clarity highest, on 8/8 inputs; every preset scalar monotone in Intensity. Intensity does not enter the compressor (engine and UI agree). | `results/main.csv`, `results/coeff_checks.txt` |
| A12 edge inputs | **pass** for silence, 50 ms, 2.5 s, DC offset, -60 dBFS, 19 kHz, full-scale square, +3 dBTP inter-sample probe, bursts, decorrelated noise, over-full-scale float, mono, 6-channel: no errors, all finite, no NaN. The 50 ms clip is the one A1 exception above. | `results/edge.csv` |
| A13 preview vs export landing | **pass**: preview gain -6.244 dB vs export -6.248 dB (input01); -16.347 vs -16.35 LUFS predicted vs delivered (input06). | `results/probe_chain_input01_baseline.json` |
| UI-1 compressor read-out | **pass**: engine thresholds/ratios equal the UI formula for all 9 presets × 5 densities. | `results/coeff_checks.txt` |

Sample-rate consistency: the static EQ response differs by ≤ 0.1 dB between
44.1, 48 and 96 kHz above 40 Hz; the same music resampled to 48 k or 96 k
before import lands within 0.02 LU and 0.03 dB of the 44.1 k result
(input02, input06). input04 differs by 1.5 dB in delivered peak between the
44.1 k and 48 k-native paths; §5.1 explains why (SRC overshoot absorbed by
the landing).

## 5. Findings with reproduction and competing explanations

### 5.1 Proven defect — ceiling violated and output clipped when the landing is skipped

* **Minimal case**: `input04.wav` (finished mix, +0.08 dBTP source),
  Universal 0.5, Custom profile, target *none*, ceiling -1.0, 48 kHz, 24-bit.
  Delivered: sample peak 0.00 dBFS (clamped by the 24-bit quantizer), true
  peak +0.08 dBTP (own meter), receipt reports 0.0 dBTP and raises
  `true_peak_high`. Command in `results/probe_notarget48_input04.json`.
* **Mechanism, isolated**: the chain's 44.1 kHz output has sample peak -1.10
  dBFS (limiter operating point). Converting that file alone through the
  engine's SRC (identity settings) gives +0.49 dBFS / +0.49 dBTP at 48 kHz;
  converting the same file with soxr gives -0.19 dBTP. The engine's
  `rubato::Fft` resampler therefore adds ≈ 0.7 dB more overshoot than a
  high-quality SRC, on top of ≈ 0.9 dB that the limiter's 12-tap-per-phase
  detector under-reads on this material (`results/src_overshoot_input04.txt`,
  `results/limiter_detector_underread.txt`).
* **Controls**: input01 and input02 through the same path stay at -1.02 and
  -1.10 dBTP (they are not flat-topped); SRC of unprocessed music is level-
  and loudness-neutral (≤ 0.015 LU, ≤ 0.2 dB peak); the normal streaming
  path is protected only because the landing trims by the measured
  post-SRC overshoot (it turned input04 down 1.6 dB more than loudness
  required). The 50 ms edge case shows the same gap through the "silent"
  LUFS gate.
* **Realistic exposure**: any hot, finished master exported with "Custom, no
  target" to a different rate; also any sub-400 ms export. Not the default
  profile.
* **What would disprove it**: a render through this path with delivered
  true peak ≤ -1.0 dBTP on input04; not observed.

### 5.2 Over-limiting of hot sources and under-delivery of quiet ones (level dependence)

* Universal 0.5, streaming, chain output before landing: input01 -7.75 LUFS
  / limited at -1.1 dBTP, then attenuated 6.25 dB to reach -14. Delivered
  crest 9.2 dB where the source (12.7 dB) needed no limiting at all to reach
  -14 by gain alone (its true peak would have been -3.5 dBTP).
* Post-limiter attenuation, Universal: input01 -6.15 dB, input02 -2.28,
  input03 -4.44 (source already at -14.05 LUFS), input04 -1.62, input05
  -7.09; Loud: -7.85 / -3.60 / -4.55 / -1.52 / -8.21
  (`results/main_tp_and_overshoot.txt`).
* Quiet sources: input06 (-21.7 LUFS) delivered -16.35 with Universal
  (-2.35 LU short), input07 (-27.4) -20.24 (-6.24 short), input08 (-26.9)
  -20.47 (-6.47 short); only Loud approaches -14. The receipt states the
  shortfall correctly.
* Level sweep: input02 at 0 / -6 / -12 dB → -14.00 / -16.31 / -18.10 LUFS,
  crest 13.0 / 17.8 / 19.6 dB. input04: -14.00 / -15.46 / -16.48.
* Stage attribution (`results/stage_attribution.txt`): on input01 the
  cumulative loudness push into the limiter is +1.2 dB input gain, +0.4 dB
  EQ, +1.0 dB compressor makeup, +2.2 dB saturator small-signal gain (drive
  1.11 normalised by tanh(1.11)); the saturator removes 4.6 dB of crest, the
  limiter a further 2.0 dB. Total chain push ≈ +5.5 dB regardless of the
  source or the target.
* Competing explanations tested: adaptation (strength 0 vs 1 changes
  delivered peak by ≤ 0.3 dB on input02/04 — not the cause); SRC (excluded,
  §5.1 shows it only affects peaks); the target itself (profiles from -24 to
  -10.5 all show the same pre-landing level).
* Assessment: a design choice with a measurable cost, not a code error. The
  automatic result is not dependable across source levels or across sources
  of different loudness. See §7 for the tested alternative.

### 5.3 True-peak metering differences (Q4)

* On a synthetic +3 dBTP reference all meters agree within 0.01 dB.
* On the delivered 48 kHz-native renders (input08) the engine's meter reads
  exactly -1.00, 16x reconstruction -0.86…-0.94, FFmpeg -0.5…-0.8. On the
  limiter's raw 44.1 kHz output for input04 and the synthetic drums:
  engine -1.10, 16x -0.84/-0.82, FFmpeg -0.1. A 0.5 dB internal margin
  (`renders/probe/margin_input08_ceil-1.5`) gave engine -1.50, 16x -1.36,
  FFmpeg -1.2.
* Distinguishing engine error from method: the engine's meter is a faithful
  libebur128 port and BS.1770-4 permits this under-read near Nyquist; FFmpeg
  uses a sharper interpolator; both are "compliant". The engine is not wrong
  by its stated method, but delivery platforms do not all use its method,
  and the landing pushes exactly to the meter's ceiling with no margin.

### 5.4 Chain costs (Q5) — numbers

| stage | effect (Universal 0.5) | cost |
|-------|------------------------|------|
| 24 Hz LR4 high-pass | -2.2 dB at 31.5 Hz band | +0.4…+1.3 dB peak from phase rotation on bass-heavy sources, later removed by limiting; LRA/crest otherwise unchanged |
| 7-band EQ + shelves | +0.4 dB loudness; the documented voicing (+0.9 dB at 8 k, +1.3 dB at 16 k) | none beyond the voicing |
| multiband compressor | +1.0…+1.8 dB loudness (makeup), LRA -1 LU on dynamic material | tonal tilt +1 dB lows / +2 dB highs (input02); Loud: +8.7 dB on -60 dBFS material |
| transient shaper | +0.1…0.3 dB peak | negligible at 0.5 |
| width | correlation 0.73 → 0.68 (Universal), 0.28 → 0.07 (Spatial) | none measurable on loudness |
| tanh saturator | +2.2…2.8 dB small-signal gain; crest -3.5…-4.6 dB on hot sources | THD 0.16 % @ -20 dBFS, 3.5 % @ -6, 8 % @ -1; IMD -34 dB; alias 15 k→900 Hz at -28 dB re fundamental at -6 dBFS (none with Custom) |
| limiter | crest -1.9…-2.0 dB after the saturator | detector under-reads 0.3…0.9 dB near Nyquist |
| landing | exact loudness | on hot sources this is a 1.6…8 dB turn-down after the above |

### 5.5 Smaller observations

* `analysis.rs` seeds `recommended_universal.advanced.lufs_offset_db` with
  `-14 - source_lufs` (an offset), but `effective_target_lufs()` treats the
  field as an absolute target in Custom mode. The frontend replaces the value
  when switching to Custom, so no user-facing effect was found; a raw use of
  the recommendation with a Custom profile would target the wrong level.
* Channels beyond the first two are not compressed or widened; decode folds
  above-stereo sources first, so this is unreachable from files.
* Above 2 channels of mono content the mono file lands 0.9 LU short of
  target because the mono loudness is 3 dB lower and the ceiling binds;
  coherent, not a defect.

## 6. Advantages of current behavior (evidence for keeping)

* Landing, receipt, format, determinism, symmetry and adaptation invariants
  all hold with margins far inside the acceptance criteria (§4).
* The receipt is honest: every shortfall and every ceiling excess in this
  audit was reported by the engine itself (`target_not_reached`,
  `true_peak_high`).
* The "ceiling is the spec, target is intent" policy never produced a
  delivered file over the ceiling in the default streaming path.
* Adaptation is conservative and cannot flip a preset's character.
* Presets are audibly distinct in the documented directions on all inputs.

## 7. Candidate alternative and held-out check (Q6)

Candidate: target-aware pre-gain, using only the existing Input Gain
control. Pass 1 measures the chain's own loudness L1 at unity gain (the
audition already computes this for the live landing); pass 2 renders with
`input_gain_db = min(0, target − L1)` (A, attenuate-only) or
`clamp(target − L1, −24, +6)` (B). The production landing then trims the
residual as today. No calibration, preset or gate was changed.

Development set (input01, 02, 03, 04, 06, 08, synthetic drums, and the
-6 / -12 dB level variants), Universal and Loud, `results/candidate_dev.txt`:

* **A**: crest preserved better in 11/18 cases (+0.5 … +4.9 dB), LRA higher
  in 10/18 (+0.7 … +2.5 LU), spectral deviation from the source lower in
  8/18 and never higher by more than 0.02 dB, true peak within the ceiling
  in every case, delivered loudness unchanged in 17/18. The one regression
  (synthetic drums, Loud) landed 0.9 LU short because a −1.9 dB pre-gain
  removed more chain loudness than 1.9 dB (the chain is nonlinear); a damped
  step (about 0.7×) or one iteration removes it. Cases that were already
  ceiling-bound (quiet sources, level variants) are byte-for-byte unchanged
  by construction.
* **B**: same gains on hot sources, but on quiet sources it pushes 2.7 … 6 dB
  harder into the limiter: closer to target by 1 … 5.7 LU at a cost of
  1 … 5.8 dB crest and up to 0.4 LU LRA, with a true-peak excess of 0.1 dB
  by my meter on input08. This is the loudness-versus-dynamics preference
  the user should choose, not an automatic default.
* Level invariance: with A, input02 at -6 dB and input04 at -12 dB remain
  ceiling-bound exactly as today (A cannot help quiet exports); with B they
  move 1–2 LU closer to target.

Held-out subset (input05, input07, drums -6 dB at 96 kHz), evaluated once
after the rule was fixed — see §7.1.

### 7.1 Held-out results

`results/candidate_heldout.txt`, rule unchanged from the development set:

| case | current crest / LUFS | A crest / LUFS | B crest / LUFS |
|------|----------------------|----------------|----------------|
| input05 Universal (finished metal, -9.8 LUFS source) | 7.06 dB / -14.00 | **12.70 dB / -14.00** (pre-gain -7.4 dB, LRA +0.12, spectral deviation -0.28 dB, TP -2.75) | same as A |
| input05 Loud | 6.31 / -14.00 | **10.29 / -14.00** (LRA +0.40) | same as A |
| input07 Universal (stem sum, -27.4 LUFS) | 23.27 / -20.24 (ceiling-bound) | unchanged | 18.47 / -15.44 (+6 dB push: 4.8 LU closer, 4.8 dB less crest) |
| input07 Loud | 18.15 / -15.01 | unchanged | 17.47 / -14.31 |
| drums -6 dB @ 96 kHz, Universal / Loud | 20.72 / -20.18 ; 17.41 / -16.34 | unchanged | 18.43 / -17.43 ; 16.84 / -15.69 |

True peak within the ceiling in 6/6 cases for both variants. The held-out
behaviour matches the development set exactly: A helps only where the chain
overshoots and does so strongly (the already-mastered input05 keeps 5.6 dB
more crest at the same delivered loudness), never changes a ceiling-bound
case, and never moves the delivered loudness; B buys loudness on quiet
sources with proportional crest loss.

Selection rule (PROTOCOL §6): A meets it (fixes a measured cost on ≥ 6 of 8
music inputs, no regression of A1–A9, costs reported, held-out consistent);
B does not (it trades one documented objective for another). Recommendation
2 in §1 is therefore A, with B offered as an explicit user option.

## 8. Coverage limits, open questions, what would disprove each conclusion

* Corpus: 4 finished mixes, 3 constructed stem sums, 1 owner track; no
  classical, spoken word, or extreme-genre coverage. Synthetic inputs cover
  the DSP edge cases, not musical taste. No listening tests were done; all
  musical statements are directional measurements.
* Album mode, MP3/AAC/OGG encoders and the live audition path were not
  exercised beyond confirming the audition landing rule equals export.
* The harness omits the desktop feature set (Tauri/LAME); C1 shows the
  rendered bytes are identical, so this does not affect any DSP conclusion.
* Disproving §5.1: show the 24-bit delivery of input04 through Custom /
  no-target / 48 k at ≤ -1.0 dBTP. Disproving §5.2: show a source whose
  pre-landing chain loudness tracks the target rather than the source level.
  Disproving §5.3: a BS.1770-4 conformance argument that FFmpeg's reading
  is out of tolerance on these files (my 16x figures sit between the two
  meters, so the engine's reading is at the low edge, not outside it).
* Open: the right damping/iteration for candidate A; whether Loud's +8.7 dB
  static makeup on quiet passages is intended; whether a 0.5 dB ceiling
  margin should be internal or a user option; whether `rubato::Fft` should be
  replaced or its output trimmed by design.
