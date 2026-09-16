# Local implementation experiments

These probes support the [checkpoint ledger](../../../docs/reviews/2026-09-15-mastering-quality-implementation-evidence.md).
They do not select a shipping sonic policy. Private WAVs and full output logs
stay under ignored `test-output/mastering-quality-implementation-20260915/`.
Every runner refuses an existing output directory/file. Preserve frozen research
and choose a new run name for changed experiments.

Additional probes added during C0:

- `run_bench.py` copies the native executable and measures only its child process
  with Windows process CPU/peak-working-set counters. Use `--timing-only --trials 3`
  for timing runs; stop competing builds/analysis before those runs.
- `summarize_timing.py --runs <completed-run> ... --output <fresh.json>` verifies
  matching final simulation measurements and separates source preparation,
  settings preparation, each edit and whole-session work. Whole-probe process
  time includes all trials/strategies and is not one editing session's cost.
- `plot_candidates.py` produces shareable PNG/SVG plots from a checked comparison.
- Build `mastering_quality_passages` in the same Cargo target directory, copy its
  executable, then pass `<source> <existing-candidate-dir> <fresh-output-dir>`.
  This compares contextual excerpts with continuous output; provisional library
  readings are state diagnostics, not independent peak protection.
- The opt-in ignored `audio::native_callback_bench::mastering_quality_native_lifecycle_bench`
  exercises the full AudioPlayer API separately from the direct CPAL callback
  probe. Read its required environment variables in the source. Both probes mute
  after processing; neither establishes a listening or installed-UI verdict.
- `qualify_fft_bound.py` explores a periodic-FFT between-grid bound. It explicitly
  does not bound finite-window truncation error. Retain failed cases; no production
  peak estimator has been selected from these probes.

## B2 finite reconstruction experiments

Read the [qualification record](../../../docs/reviews/2026-09-15-peak-reconstruction-qualification.md)
before interpreting results. Early rejected candidates are retained; the later
qualified library core now supports the application's final-PCM verification.

- `qualify_sinc_bound.py` / `sinc_tail_moments.py` independently evaluate finite
  cardinal-sinc intervals, omitted-tail bounds and optional signed-tail refinement.
- Build `mastering_quality_peak_probe` and run its four unit tests. Its optional
  third argument is the numeric `kernel.f64le` file from `extract_soxr_kernel.py`.
  The driver compares the native candidate with ebur128 0.1.10 at three chunk sizes.
- `qualify_peak_probe.py` creates fresh synthetic inputs and retains comparisons
  against the historical direct references, including failures.
- `extract_soxr_kernel.py` records an interior impulse response with exact generator
  provenance and independently varied length/position checks. The numeric data
  stays under ignored output until production selection.
- `check_reconstruction_edges.py` establishes the direct/padded SOXR distinction
  and independently evaluates long alternating signals without FFT or moment trees.
- `qualify_hybrid_peak.py` retains the combined Python experiment and every old-
  reference discrepancy. `check_native_peak_bounds.py` adds explicitly zero-extended
  16/32/64× SOXR checks alongside the retained old references.
- `ebu_phase_signals.py` separately synthesizes Tech 3341 cases 15–23. The earlier
  `ebu20plus` stress cases are preserved and do not establish EBU compliance.
- `compare_peak_implementation.py` checks an optimized implementation against a
  retained native run and its independent reference results, after asserting
  source identity and layout. `peak_cost.py` rotates three before/after trials
  using copied executables and reports process CPU/working set separately from
  candidate/FIR measurement times. Run timing after other task-owned CPU work ends.
- `mastering_quality_meter_probe` runs the replayable library core against an
  existing input list. Its two arguments are `<inputs.json> <fresh-output.json>`;
  use the same parity script and retained independent references.
- `mastering_quality_linear_probe <inputs.json> <fresh-directory>` compares
  whole-file residual verification with fresh reconstruction. Inputs may specify
  `gains`, `bits` and `write: true` to retain exact delivery samples.
  `check_linear_reuse.py --run <directory> --output <fresh-directory>` supplies
  separate finite SOXR16/64 and short direct-sinc checks. Read the
  [reuse proof and limitations](../../../docs/reviews/2026-09-15-linear-peak-verification.md);
  an interval that is safe but too wide still needs fresh measurement.

All sources/outputs and copied binaries are local evidence. Use new output names
when changing a runner. These probes are not installed-app or listening proof.

## Current application witnesses

From `src-tauri`, build `mastering_quality_witness` with
`cargo build --target-dir target/codex-rc --example mastering_quality_witness`.
From the repository root:

```powershell
python scripts/research/mastering-quality-implementation-20260915/check_witnesses.py --output test-output/mastering-quality-implementation-20260915/ab-witnesses-v2
```

The script copies the executable outside Cargo's output before using it. It
checks the restored Funk hash, recreates the pruned hot-start variant with the
recorded recipe, loads historical settings, renders through the current engine,
and retains independent meter results. A1/B1 success does not qualify the old
peak meter: remaining independent ceiling failures stay visible.
Use `--qualified` for the B3 witness run: it additionally requires both finite
zero-extended SOXR16/64 references to meet the requested -1 dBTP ceiling.

## C0

Build `mastering_quality_bench` with the same target directory, then copy the
executable to the ignored evidence directory before running. Arguments:

```text
mastering_quality_bench.exe <source.wav> <fresh-output-directory> [--timing-only | --candidates-only] [--trials 3]
```

Version 2 reports the actual `engine::preview_landing` API separately from
strategies that all add whole-file final verification. Trial ordering rotates
the three preparation strategies. Source facts/ordered windows and a two-result
raw-PCM cache are measured separately. Keys and finite active-passage thresholds
are experimental; production identity/version/invalidation requirements remain
in the plan. The original v1 timing run overlapped builds and cannot establish a
repeatable speedup. Controlled trials must run after build activity stops.

Candidate renders retain current coefficients except for a separate experimental
input-drive multiplier. Check them with both references and descriptive metrics:

```text
python scripts/research/mastering-quality-implementation-20260915/check_candidates.py --source <source.wav> --run <candidate-directory> --output <fresh-checked-directory>
```

This checks every complete candidate waveform, applies a target/ceiling-bounded
scalar, writes float PCM, and checks the complete delivered waveform again.
The imported historical shape-measurement helper writes into the **new** output
directory only. FFmpeg LUFS/LRA summaries are rounded to 0.1 LU. No composite
fidelity score or preferred-sound claim is inferred from crest/contrast metrics.
The optional two local search refinements remain to be evaluated.

The opt-in `audio::native_callback_bench::mastering_quality_native_callback_bench`
test uses `YES_MASTER_BENCH_FILE` and a fresh `YES_MASTER_CALLBACK_REPORT` path.
It runs production MasteringSource and meters on a real CPAL callback, then
mutes samples before the device. It measures concurrent whole-file preparation
and rapid coefficient edits. Requested/granted frames and all timing misses are
reported explicitly; this is not installed UI, listening, or complete
AudioPlayer A/B/source/device-lifecycle evidence.

## Peak qualification

`qualify_fir.py --output <fresh.json>` records every candidate failure against
the corrected FFT reference. Hann-sinc support is measured in input samples,
not total oversampled FIR taps. No candidate from v1 qualified, including the
longest filter. The finite-edge/bandwidth specification and independent library
comparison remain open; increasing oversampling or adding an arbitrary margin
is not adoption evidence.

### Subsequent source-analysis and native antialias diagnostics

`mastering_quality_source_analysis` takes a verified original-source manifest,
a specification path and a fresh output directory. It uses actual complete
production analyses at 0/-40/-80 dB and on each normalized copy. The companion
`compare_source_analysis.py` verifies complete inputs and compares profiles,
coefficient identity and availability without rerendering. See the source-analysis
protocol/results under `docs/reviews/2026-09-15-source-analysis-gain-*.md`.

`saturation_native_reference.py prepare --output FRESH --protocol SPEC` reuses
the retained 8x FIR tables and writes four finite synthetic inputs plus a job.
Build the `mastering_quality_saturation_native` example, run it with that job
and a fresh output directory, then use `saturation_native_reference.py check
--job JOB --report REPORT --output FRESH.json`. The independent reference checks
complete causal output including ringout; timing is an isolated native prototype,
not a production/device callback pass. Preserve the exact kernel/input/report
hashes and the native protocol/results before changing filter design.

`compare_saturation_native_cost.py --root EVIDENCE --output FRESH.json` joins
the retained three alternating serial/four-accumulator pairs and qualified
independent outputs. The serial example is pinned at `7e4a7327`; the subsequent
example retains the arithmetic alternative, with its short-filter regression
explicit. This is a research implementation, not a production saturation switch.

`prepare_saturation_whole.py --output FRESH` generates an isolated source copy
pinned to local `7e4a7327` (including the **serial** native filter). It asserts
production DSP identity and binds generated files in `provenance.json`. Build
that standalone manifest, copy its executable, then run `<job.json> <fresh-output>`.
The retained `e-whole-job-v1.json` binds existing C1 cases/coefficients and kernels;
it is local private-source evidence. Production source files are not edited.
`check_completed_device_rows.py` can independently verify these completed PCM
rows; its generic device-oriented filename does not make this an actual-device run.
`evaluate_saturation_whole.py` takes `--report`, `--metrics-map`, and a fresh
`--output` to retain original C1 anchors. After all 16 rows finish, run
`summarize_saturation_whole.py --root EVIDENCE --output FRESH.json` to validate
all joined hashes/checks and locate gain-removed whole-delivery differences.
The protocol/results explicitly separate fixed-duration export tails, complete
exterior native ringout, full-song protection and still-open adoption gates.
