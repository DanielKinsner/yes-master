# Reproduce the September 15 mastering-quality investigation

Research tooling, **not imported by the application**. It makes private instrumented
modules from the verified production sources. The current positive saturation
arithmetic is unchanged; a negative diagnostic coefficient selects the labelled
experimental curve. `src/src_fixed.rs` is used only by the SRC probe, not music
rankings. No defaults, presets, calibration gates, UI or release state are changed.

Read the [recommendation](../../../docs/reviews/2026-09-15-mastering-quality-recommendation.md)
and [frozen protocol](../../../docs/reviews/2026-09-15-mastering-quality-protocol.md).
Original experiments are under `test-output/mastering-quality-20260915/`.
**Never run this package in that existing evidence directory.**

## Platform and source identity

These commands and whole-WAV identity were checked on Windows. They use `.exe`
names and PowerShell. The same hashes are not established for macOS. This is an
offline test harness, not native playback, latency, installed-app or release proof.

- Production source baseline: `a4fb621d88a95b8af549467fb499943acb4274d5`.
- `expected-source.json` contains hashes of every included production module,
  the existing compile stubs and the native toolchain. Setup refuses source drift.
  A later documentation-only commit can retain identical source hashes.
- Rust 1.95.0, Cargo's included lockfile (Rubato 1.0.1); FFmpeg/FFprobe 7.1.1
  full build from gyan.dev with soxr; Python 3.13, NumPy 2.2.6, SciPy 1.17.1.
- Optional charts used Matplotlib 3.11.2 with its separately installed NumPy
  2.5.3. Analysis used NumPy 2.2.6. Plotting does not change measurement data.
- Allow roughly 25 GB for restored owner evidence, acquired sources, renders
  and logs, plus build dependencies. Downloads and the full grid take time.

Use a checkout with the specified source hashes; preserve work in an active
checkout. Restore the private owner archive using the
[existing transfer guide](../../../docs/reviews/2026-09-14-research-handoff.md#restore).
It supplies `tests for presets/`; neither owner nor downloaded music is in Git.

## Fresh setup and exact baseline

From the repository root, choose an unused experiment name:

```powershell
$experiment = 'test-output/mastering-quality-reproduction'
if (Test-Path -LiteralPath $experiment) { throw 'Choose a fresh directory' }
Copy-Item -LiteralPath 'scripts/research/mastering-quality-20260915' -Destination $experiment -Recurse
python -m venv "$experiment/venv"
$py = "$experiment/venv/Scripts/python.exe"
& $py -m pip install -r "$experiment/requirements-analysis.txt"
& $py "$experiment/setup.py"
cargo build --release --locked --manifest-path "$experiment/Cargo.toml"
& $py "$experiment/verify_reproduction.py"
```

The setup script requires `test-output/<experiment>` depth because native module
paths refer to the checkout. It refuses rerunning setup over generated evidence.
`verify_reproduction.py` writes fresh On, Off and 257-frame-block On files and
asserts whole-WAV SHA-256 equality to the owner's exports. Expected On:
`639ac1dd5d049040d8b689636c08bf2890b39929b72601703d1b93938f6022ce`;
Off: `d33c8cae3238161a8d7f812884c2137d980189485b3618def8af93a8baab6ea5`.
Any mismatch is a finding to investigate before using candidate results.

## Acquire and prepare the frozen corpus

```powershell
& $py "$experiment/acquire.py"
```

Seven additional no-cost CC BY pieces: four finished MP3s and three constructed
stem sums. `expected-corpus.json` records URLs, credits, licenses, hashes, original
formats and transformations. `pages/*-downloads.json` freezes ccMixter download
metadata, excluding preview mixes when selecting stems. Each download is checked
against original byte count/hash; prepared WAV and stem hashes are also checked.
MP3s decode to float without clipping existing headroom. Stems are summed from
zero and scalar-normalized to -6 dBFS peak, without bus mastering. These are
constructed balances, not artist-approved premaster mixes. No service upload.

The resumable curl downloader may need retrying if a host times out. Changed or
missing remote files must be reported; do not silently substitute and keep the
old corpus identity. Use `--subset piano,funk,...` only for diagnostic acquisition;
it writes a subset manifest, so run the full command before the full experiment.

## Frozen music matrix and independent measurements

Run each line to completion before the next dependent line. The runner uses at
most three concurrent jobs. Each source has 48 pre-landing variants: 13 trims in
each of three families, plus nine isolated controls. Target -9/-14 share a raw
render only after a native coefficient-equality assertion. Selection requires
all 48 records, then renders 24 delivery finalists per source.

```powershell
& $py "$experiment/run_matrix.py" grid development
& $py "$experiment/run_matrix.py" finalists development
& $py "$experiment/analyze_music.py" development
& $py "$experiment/run_matrix.py" grid holdout
& $py "$experiment/run_matrix.py" finalists holdout
& $py "$experiment/analyze_music.py" holdout
& $py "$experiment/run_robustness.py"
& $py "$experiment/run_centered.py"
& $py "$experiment/centered_finalists.py"
& $py "$experiment/default_checks.py"
& $py "$experiment/measure_landr.py"
& $py "$experiment/make_comparison_clips.py"
```

The source-centred follow-up is exploratory: the original holdout has already
been observed. It centres trim on `-18 - source LUFS`, then uses the same grid
offsets and selection rule, retaining the +/-24 dB production gain clamp.
`default_checks.py` adds Universal Intensity 50 without tuning other presets.

Independent measures include FFmpeg LUFS/LRA/true peak, common-rate RMS-matched
band energy, crest, 400 ms crest, source-anchored 10-second section contrast,
short-term loudness spread, attack/body windows, correlation and M:S. Inspect
`analyze_music.py` for exact windows and thresholds. The original peak criterion
is retained: <=-0.9 dBTP with a -1 request, allowing FFmpeg's reported precision.
`CEILING FAILURE` is expected for the demonstrated failures; it is recorded while
the run continues. Do not widen the threshold to make the corpus pass.

The runner can resume interrupted jobs using existing JSON and metric caches.
**Resume only with identical scripts, source bytes, job settings and binaries.**
For any changed condition use a new directory: cache keys do not capture every
dependency. A completed native variant refuses overwrite; some metadata and
summary writers do overwrite same-named files. Keep original evidence intact.

## Mechanisms, SRC and peak reconstruction

```powershell
& "$experiment/target/release/yes-mechanisms.exe" "$experiment/mechanisms-v2" "$experiment/verified-reproduction/source.json"
& $py "$experiment/analyze_mechanisms.py"
& "$experiment/target/release/yes-src-sweep.exe" "$experiment/src-sweep"
& "$experiment/target/release/yes-src-check.exe" "$experiment/src-check"
& $py "$experiment/analyze_src.py"
& $py "$experiment/oversampling_reference.py"
& "$experiment/target/release/yes-peak-probe.exe" "$experiment"
& $py "$experiment/compare_src_filters.py"
& "$experiment/target/release/yes-peak-meter.exe" "$experiment"
& $py "$experiment/verify_peak.py"
```

504 mechanism cases cover three source rates, tones/levels, multitone, burst,
impulse, step, silence and tiny tails with eight processing modes. The neutral
Custom probe explicitly sets unity input gain; it does not inherit Custom's
baseline push. Synthetic SRC cases compare 45 conditions each for current and
isolated corrected processing. The corrected loop uses integer frame accounting;
the independent analyzer computes expected length independently, using Rust's
positive half-frame rounding for input generation.

Analytical oversampling uses SciPy `resample_poly` defaults (Kaiser beta 5), 4x/8x,
the same tanh curve, and coherent integer-second FFT windows. It is not a music
candidate or a production oversampler. Noise-floor residuals and roundoff can
differ at negligible levels between numerical evaluation orders; do not treat
every last decimal as a separate quality claim. `verify_peak.py` uses soxr
precision 33; this repeated the original peak results to within 0.001 dB on
Imaginal and matched the quoted Funkorama result. Reference filters have different
responses; their waveform differences are not automatically distortion scores.

## Compact evidence and plots

```powershell
& $py "$experiment/build_evidence.py"
& $py -m pip install --target "$experiment/pylibs" matplotlib==3.11.2 numpy==2.5.3
& $py "$experiment/plot_report.py"
```

The packaged exporter writes `compact-evidence/` inside this fresh experiment,
not tracked application files. It records source/compiler/harness hashes, current
reproduction proof, all 192 pilot rows, default and centred checks, mechanisms,
credits and LANDR metrics. Non-finite diagnostic dB values use explicit strings
such as `"-Infinity"` in compact JSON; that denotes silence, not zero dB. The
full ignored logs preserve original output, including failed ceiling checks.

Compare compact artifacts with the
[committed evidence](../../../docs/reviews/evidence/2026-09-15-mastering-quality/).
Do not publish new audio or replace original evidence as a side effect of rerunning
the package. A production correction would need the affected application, bridge
and fixture checks described in `docs/TESTING.md`; this research does not satisfy
those integration gates.
