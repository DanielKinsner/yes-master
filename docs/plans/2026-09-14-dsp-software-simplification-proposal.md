# DSP and software simplification proposals

Reviewed: 2026-09-14. Source baseline: `a001db96`.
Status: proposals for review; no application changes implemented.

> September 14 transfer clarification: this is the original structural proposal,
> not an active launch work queue. The owner subsequently deferred desktop DSP
> simplification. The crossover experiment was retained on local branch
> `codex/dsp-crossover-equivalence` (`f8e49180`, owner deferral recorded there);
> its implementation is not being merged by this documentation transfer.
> Preserve that deferral when reading the suggested order below.

The ce-simplify-code reuse, quality and efficiency reviews covered representative
paths in DSP, playback/preview, analysis, Track/Album delivery, and frontend state.
This is a targeted structural review, not an exhaustive correctness audit or a
performance benchmark. Findings below were checked against the current source.

The best starting point is to make tests exercise production DSP routines and
centralize equivalent measurement logic. Follow with allocation improvements and
bounded frontend transitions. Larger Album and session refactors should follow
those small checkpoints; they carry more lifecycle and compatibility risk.

## 1. Make crossover tests exercise the production splitter

**Start first. Small scope; direct test-quality benefit.**

Evidence: `src-tauri/src/dsp.rs:1746` contains a test-only LR4 splitter with the
same eight biquad processing calls as the production compressor at line 2224.
The crossover test at line 4539 exercises that separate implementation, with
coefficients reconstructed at 44.1 kHz on every sample.

Generalize the existing splitter, or introduce `LR4State::split`, accepting the
four precomputed coefficients. Use it from the compressor and the crossover test.
Keep coefficient construction outside the sample loop. This gives the crossover
test direct coverage of the routine used by the mastering chain.

Preserve the exact filter-call order, per-channel state, denormal handling and
sample-rate-aware coefficients. Introduce no allocations or dynamic dispatch in
the audio path. Compare old/new output and state bits at several rates, then run
the compressor, preset and audio-invariant coverage plus the fixture lane before
integration. Do not alter crossover tuning or numerical tolerances.

## 2. Centralize equivalent measurement calculations

**Start early. Two small extractions with separate responsibilities.**

The pre-landing BS.1770 calculation is repeated in
`src-tauri/src/engine.rs:364` and `src-tauri/src/engine.rs:1283`: initialize the
meter, feed PCM, sanitize integrated LUFS, find channel true peak and convert to
dBTP. Extract a helper returning named loudness and peak fields. Preserve each
caller's existing early returns and measurement timing: the Track path currently
measures even when no target is selected, whereas the other path can return early.

Decoded-delivery finalization also repeats at `src-tauri/src/mp3.rs:395` and
`src-tauri/src/export_encoding.rs:238`. Extract the shared LUFS/peak/LRA result
calculation, preserving caller-specific error prefixes and the order of empty
frame checks. A fallible fold can replace the temporary vector of channel peaks.

Keep these helpers distinct from the policies that decide what to measure.
MP3 uses gapless Symphonia decoding; the sidecar path has its own decoded WAV
readback. `wav_writer::measure_delivery` measures deterministically quantized PCM
and has different sanitization behavior. Those paths are not interchangeable.
Pre-landing measurement and final-delivery measurement are both necessary.

Verification: silent/short signals, channel counts, target absent/non-finite,
error propagation, existing landing regressions, receipt comparisons and export
format matrices. Retain independent decoding and strict PCM/frame assertions.

## 3. Reuse scratch storage and loop-invariant calculations

**Three independent, bounded efficiency changes. Benefits need measurement.**

- `src-tauri/src/deep_analysis.rs:488` allocates an FFT buffer and computes the
  same Hann window for each detail window. The scan already fixes FFT size at
  line 350. Own the buffer and Hann coefficients at scan scope and refill the
  buffer for each window. Preserve the exact coefficient expression, window
  boundaries and accumulation order. Compare deep-analysis outputs and fixture
  digests, and measure scan time and allocations on the same inputs.
- `src-tauri/src/mp3.rs:386` creates a `SampleBuffer` for every decoded packet.
  Reuse one with a capacity check, following the storage pattern already used
  in `src-tauri/src/decode.rs:171`. Keep delivered-stream rate/channel rejection,
  non-finite rejection and gapless decoding intact; do not replace the decoder
  with the more permissive import decoder. Verify all bitrates, mono/stereo,
  changing packet capacities, frame counts and receipt measurements.
- `src-tauri/src/album_render.rs:740` resamples the same preset arc once per
  non-overridden track. Resolve it once per job, preferably lazily at the first
  use, then index it. Preserve Custom's neutral value and the override exemption.
  Test differing track counts and mixed/all-overridden albums. This is a modest
  cleanup, not a claim that arc interpolation dominates export time.

No speedup percentage is established by this review. Keep each experiment only
if it preserves results and provides useful clarity or measured resource savings.

## 4. Give track-state transitions a single implementation

**Begin with meter resets; stage the larger session-state change.**

`src/hooks/useTrackMaster.ts:792`, `1537`, `1656` and `1744` repeat the same
silent meter fields during device loss and track changes. Extract a pure helper
that supplies fresh silent-meter values, with explicit transition-specific
overrides. Device loss preserves the reported position and sets `deviceLost`;
ordinary track selection/removal resets position and can disarm a loop. Preserve
those differences and the existing playback/Volume Match/Preview LUFS settings.

The broader issue appears at `useTrackMaster.ts:1693`: removing a track requires
coordinated updates across ordered tracks, analysis, waveform, settings, stale
status, loaded kind, loops and Album overrides. Introduce an explicit track
session transition/reducer so import, restore and removal have one place to
maintain per-track state. Keep rail order explicit. Start with one transition
and retain the hook's external interface instead of rewriting the whole hook.

Keep backend eviction and playback/loop side effects outside pure reducers and in
their current order. Preserve project JSON, fallback selection, undo/redo and
stale-analysis rejection. Existing hook integration coverage should exercise
selected/non-selected/loaded removal, import, restore, device loss and Album
ordering; use headless checks for affected rendered behavior.

## 5. Extract render lifecycle handling without changing concurrency

**Stage after the smaller frontend transition work.**

`src/hooks/useTrackMaster.ts:428`, `808`, `1977`, `2110` and `2158` distribute
progress, completion timers, busy state, cancellation and cleanup across preview,
Track export and Album export. Extract an operation lifecycle helper with explicit
start, cancellation-requested, cancelled, failed and completed transitions, while
each operation keeps its own request, save dialog and receipt handling.

Inventory which operations may overlap before choosing its state shape. A single
global discriminated union is only appropriate if mutual exclusion is established;
otherwise keep per-operation state. Do not infer exclusivity from current UI
buttons or collapse the existing flags simply to shorten the hook.

Preserve current busy-state timing around dialogs, per-kind messages and receipt
separation. Preserve the rule that an old completion timer cannot clear new job
progress. Verify back-to-back jobs, cancellation/retry, dialog cancellation,
failures, unmount cleanup and affected restart/busy guards through hook and App
progress/Album tests. Native checks remain necessary for playback during export.

## 6. Share Album delivery orchestration in stages

**Worth doing, but the largest export refactor in this proposal.**

`src-tauri/src/mp3.rs:66` and `src-tauri/src/album_encoding.rs:67` both resolve
delivery properties, assemble lossless temporary WAVs, allocate a destination,
deliver tracks and the continuous programme, update a manifest and clean up.
Extract common steps around `album_render::render_album_plan_impl_with_cancel`
and `unique_export_subdir`, with explicit codec-specific delivery/finalization.

Start with small equivalent helpers. The existing branches differ in error text,
cancellation checkpoints, manifest fields and file synchronization. Preserve
these differences rather than hiding them in an overly generic wrapper. If the
necessary callbacks make the flow harder to read, retain the separate top-level
functions and share only the proven common steps.

Keep deterministic quantization, ordered tracks/gaps, one encode of the complete
programme, source/collision protection and failure cleanup. Preserve serialized
fields including `album_wav_path`. Validate all delivery formats, cancellation
at each stage, failures after partial output, report/manifest identity and exact
PCM/frame contracts. Keep the documented Intel Mac parity failure unresolved
until independently explained; this proposal is not a fix for that failure.

## 7. Reuse identical display-format helpers

**Small optional frontend cleanup.**

`src/lib/chrome-content.ts:7` and `:12` duplicate sample-rate and dBTP formatters
in `src/lib/standard-export.ts:25` and `:30`. Export those helpers or place them
in a small shared module. Preserve Unicode minus, precision and existing copy.
Do not substitute other similarly named formatters without comparison: the
receipt formatter has different behavior below 1000 Hz.

Existing Standard export and chrome tests cover the relevant presentation.

## Deliberately deferred or rejected

- Skipping Volume Match measurement when its toggle is off: `audio.rs:1899`
  does perform that work, but the cache key deliberately ignores the toggle.
  The computation also warms a later toggle. Changing this is a scheduling and
  latency tradeoff, not established exact-behavior simplification. Measure it
  separately and test toggles during an in-flight landing job.
- Merging the six-band and 31-band spectral routines: their separation is
  explicitly intentional and golden-pinned. Scratch reuse inside a routine
  does not require merging their contracts.
- Replacing delivered-file readback with pre-encode measurements, or removing
  output collision/source checks: these would weaken product contracts.
- Combining cancellation helpers without preserving atomic ordering: the shared
  engine helper uses `SeqCst`; some codec checks use `Relaxed`.
- Streaming Album assembly directly during per-track writing: this could avoid
  another disk pass, but changes the output/cleanup lifecycle significantly.
  Profile first. Current Album assembly already uses bounded streaming, so it
  should not be described as a full-album-memory defect.
- Broad DSP rewrites, preset retuning, new concurrency, or file splitting based
  only on size: no evidence here establishes their value. Adaptive Compressor
  calibration and other gated sonic behavior stay under existing decisions.

## Suggested checkpoints and verification status

1. Production crossover helper and pre-landing meter helper, as separate commits.
2. Decoded-delivery finalization and packet-buffer reuse, independently verified.
3. Detail-analysis scratch reuse and Album arc reuse, with measurements.
4. Meter-reset helper and identical display-format helpers.
5. Incremental track/render lifecycle and Album orchestration work.

Use `docs/TESTING.md` for each implementation checkpoint: frontend typecheck,
tests/build; Rust formatting, strict Clippy and affected tests; headless checks
for rendered UI; affected bridge checks for shared contracts; fixture lane before
DSP/export integration. Independent native and exact-platform evidence remain
distinct. `package.json` has a typecheck script but no dedicated frontend lint
script; Rust linting uses Clippy.

Applied application changes: reuse 0, quality 0, efficiency 0. This delivery adds
only this proposal document. No application tests, benchmarks, builds or native
checks were run because no implementation was changed. Existing historical test
results are not presented as validation of these proposals.
