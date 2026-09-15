# Mastering quality implementation evidence

Started September 15, 2026 on local `codex/mastering-quality`, from `7534f612`.
The starting checkout was clean and the revised plan matched that commit.
No push, main integration, publication, spending or private-audio commit is authorized.

## Checkpoint ledger

| Checkpoint | Status | Evidence and limits |
| --- | --- | --- |
| A | Implemented; integration verification running | Same Rubato FFT filter, explicit partial input/draining, checked integer frame counts. 1,690 rate/length conditions pass; finite-tail and fixed-phase stale-frame checks pass. Actual export and fixture lanes pending. |
| B1 | Implemented; regressions running | Shared optional-LUFS decision and raw measurement availability; no-target/short-audio protection. Current detector only, not B2/B3 qualification. |
| D1 | Complete | Named Auto 0.50 / Custom 0.00, saved null unchanged; summary identifies pre-Adapt values. Build, all 38 app headless checks, landing checks and 892 frontend tests passed. Fresh Advanced capture inspected: thumb/readout/summary visible without overlap. |
| C0 | Experiment started | Protocol below; results pending. |
| B2–B4, C1–C3, D2, E | Pending | Existing revised dependencies and adoption/calibration gates remain. |

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
  suite is rerunning separately; future probes will run a copied executable.
  This is a harness/build conflict, not an audio result.
- Rust 1.98 formatting reordered two guardrails imports. That mechanical change
  triggered the existing demo stamp; the real WASM rebuild passed. No DSP
  coefficient or gate changed. It remains separate from browser/native proof.

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
