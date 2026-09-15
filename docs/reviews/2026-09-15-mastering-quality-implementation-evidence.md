# Mastering quality implementation evidence

Started September 15, 2026 on local `codex/mastering-quality`, from `7534f612`.
The starting checkout was clean and the revised plan matched that commit.
No push, main integration, publication, spending or private-audio commit is authorized.

## Checkpoint ledger

| Checkpoint | Status | Evidence and limits |
| --- | --- | --- |
| A1 | Complete locally | Same Rubato FFT filter, explicit partial input/draining, checked integer frame counts. 1,690 rate/length conditions pass; finite-tail and fixed-phase stale-frame checks pass. Strict Clippy and desktop library/integration/fixture suite pass. |
| A2 | Complete locally | Synthetic and historical private hot-start CD exports pass exact counts and continuous-reference checks. Corrected frame 955 differs only by expected PCM16 dither. Desktop fixtures and affected bridge lanes pass. |
| B1 | Complete locally | Shared optional-LUFS decision and raw measurement availability; 12 no-target/short-audio PCM16/24/float regressions and historical Funk/50 ms witnesses pass. Current detector still underreads independent peaks: B2/B3 remain open. |
| D1 | Complete | Named Auto 0.50 / Custom 0.00, saved null unchanged; summary identifies pre-Adapt values. Build, all 38 app headless checks, landing checks and 892 frontend tests passed. Fresh Advanced capture inspected: thumb/readout/summary visible without overlap. |
| D2 | Implemented; checkpoint verification passed | Actual guarded coefficients, request-associated source/settings/analysis identity, accurate fallback, unchanged Manual seeding. 972 settings configurations / 2,916 band comparisons; 33 guardrail tests; 894 frontend tests, build/headless, wire golden and bridges pass. |
| C0 | Experiment running | Initial source/PCM reuse and candidate renders complete on owner source; independent final-output checks and expanded experience measurements running. Initial timings are confounded by background builds, not a claimed speedup. |
| B2 | Qualification in progress | Five Hann-sinc candidates failed declared FFT-reference edge cases; none adopted. See below. |
| B3–B4, C1–C3, E | Pending | Existing revised dependencies and adoption/calibration gates remain. |

The subsequent desktop library/integration run (`d2-rust-fixtures.log`) also
passed with all four restored private-fixture tests enabled. Strict all-target
Clippy passed (`d2-clippy.log`). The opt-in native callback probe is excluded
from ordinary test counts and needs its own reported run.

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
48 total taps. The existing ebur128 0.1.10 filter has not been replaced.

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
