# Final research review and transfer tools

Research only. None of these modules is imported by YES Master.

- `verify.py`: validates both Claude seals; checks corrected FFT interpolation
  against original sample values and SciPy; re-renders the owner control and
  five pruned witnesses through the production harness; compares final peak
  references, including float-forced SOXR. It reads the original measurement
  lists and resolves their historical paths against the current roots.
- `check_src.py`: remeasures nine retained SRC tone WAVs with a fixed analytic
  amplitude and explicit fractional delay. Records the stale sample separately
  so an edge guard cannot hide it. It checks current and both prototype lengths.
- `package.py`: coordinator-only selection/packaging of the original two
  experiment trees. `inventory` is read-only; `text` copies original records
  byte for byte into the repo archive; `build` writes a new private supplement.
  It refuses an existing package destination. It does not delete anything.
- `restore.py`: standard-library archive/member SHA-256 checks, explicit two-root
  mapping, path validation and refusal to overwrite different existing data.
  `--verify-only` checks all payloads without restoring. See the
  [transfer guide](../../../docs/reviews/2026-09-15-mastering-quality-transfer.md).
- `test_restore.py`: focused corruption/path/collision/round-trip checks.

Use a fresh output directory for changed experiments. `verify.py` and
`check_src.py` refuse their completed JSON outputs. To rerun elsewhere, copy
the research script and change its `OUT` assignment; do not change sealed files.
The original independent workspace has a neutral `AGENTS.md` for its blind
phase. That phase is complete; use the final planning handoff for current work,
and do not interpret the archived instructions as a request to repeat blinding.

Matched environment: Windows, Rust 1.95.0 (locked harness dependencies), Python
3.13.5, NumPy 2.2.6, SciPy 1.17.1, SoundFile 0.12.1, FFmpeg 7.1.1 with SOXR.
`verify.py` resolves native executable suffixes but cross-platform WAV identity
is not established. The tests here do not replace application/bridge/fixture,
native listening or real-time performance checks when implementation begins.

```powershell
python -m unittest discover -s scripts/research/final-review-20260915 -p test_restore.py -v
python scripts/research/final-review-20260915/check_src.py
python scripts/research/final-review-20260915/verify.py
```

The FFT reference splits an even input length's Nyquist bin, preserves original
samples and uses overlapping blocks with zero extension at file edges. It is a
finite-block band-limited reconstruction reference, **not an exact DAC model or
a shipping metering algorithm**. The SOXR command explicitly uses double float
before/after resampling and `pcm_f64le` output to prevent format negotiation from
clipping above-full-scale reference peaks.

The original Claude names/scripts remain unchanged in the archived evidence,
including the flawed `fftexact` implementation and pruning helpers. Do not run
those helpers over preserved evidence. The
[final verification addendum](../../../docs/reviews/2026-09-15-mastering-quality-final-verification.md)
explains which conclusions survive corrected measurements.
