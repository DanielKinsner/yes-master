# Source-rate live preview and consistent loudness controls

The owner authorized implementing source-rate preview, further live-speed work,
consistent loudness controls and explicit upstream/downstream verification after
the [initial investigation](2026-09-22-preset-preview-investigation.md). Upfront
analysis/export cost may take priority behind audition responsiveness. No preset
retuning, public push, release or deployment is included.

## Implemented behavior

- Live commands and selected-track prewarm use a transient source-rate, 32-bit
  float settings copy. Effective target/ceiling and all processing coefficients
  survive profile normalization. Requested/saved/export settings remain intact.
- Delivery rate, precision and encoding no longer alter the live rate, recreate
  playback or cancel matching preparation. Actual device conversion remains
  mastering-quality SRC. Export still uses its requested delivery/codec rules.
- Equal source/device rates share immutable PCM and its exact whole-file
  measurements. No second copy or reconstruction is required for that response.
- Independent preview peak calculations run with a maximum of four workers,
  bounded by available parallelism minus one, with serial fallback. This changes
  scheduling, not the qualified algorithms, arithmetic or measurement tolerance.
  Workers join before returning or releasing cancellation/worker-budget state.
- Existing nonlinear response/gain caches, complete residual validation, final
  peak protection, source epochs, revision alignment and compensated transitions
  remain. Export measurement remains serial. Retained memory caps are unchanged;
  shared same-rate buffers are conservatively charged to both cache budgets.
- Loudness Target includes every distinct delivery-profile target plus the
  existing -11, -9 and Natural choices. Both controls use the same short names
  and one-decimal LUFS notation. Custom shows its actual target. Target-only
  changes retain ceiling and delivery format; profile selection sets all three.
  The -11 quick choice is named Loud rather than implying a platform requirement.
- Compact Advanced meters display Now LUFS, Since play LUFS and Peak dBFS.
  Tooltips distinguish a current passage, playback history and sample peak from
  whole-track target/true-peak ceiling. Delivery copy identifies source-rate preview.

## Measured native evidence

Source: owner's `It’s a coat (5).wav`, 252.36 seconds, 48 kHz stereo,
SHA-256 `596506d94f89bbc4cfe03e860745a3b97e932c7512d1d7e9844a3674acdb37c0`.
The completion probe uses the investigation's saved Clarity settings at -11 LUFS,
starting from Tape at 30 seconds. This is a reproducible preset transition, not a
claim that the original unknown preset pair has been recovered. The requested
delivery setting remains 96 kHz in both runs. Windows default device was
Speakers (Realtek(R) Audio), 48 kHz; output is captured before test-only mute.

| Measurement | Prior route | Source-rate optimized route |
| --- | ---: | ---: |
| Cold complete preview preparation | 27.110 s | 6.707 s |
| Final gain ready to emitted revision | 163.225 ms | 14.687 ms |
| Captured callbacks | 2,978 | 923 |
| Processing deadline misses | 0 | 0 |
| Completion-boundary deadline misses | 0 | 0 |
| Silent frames in half-second boundary window (<1e-9 amplitude) | 0 | 0 |

The observed cold preparation reduction is 75.3% for this case. These are native
application-source/callback observations, not DAC latency or a universal preset
deadline. The baseline had the old 48 -> 96 -> 48 kHz route; the optimized case
has source/device both at 48 kHz. The initial investigation separately records
the prior source-rate setting taking about 19–22 seconds in installed logs.

The separate real AudioPlayer lifecycle passed playing/paused export-rate edits
with unchanged playback generation, seek/resume, four Original/Mastered switches,
new target and cached target/resume. First cold landing reached emitted output
in 6.065 s, a new -23 target in 0.907 s, return to cached -14 in 0.075 s, and paused
cached-target resume in 0.065 s. Seven codec selections retain live source rate
and generation. Browser tests cannot supply this native evidence.

The reported end-of-preparation hiccup was not reproduced in either completion
capture. There is no silent dropout in their boundary PCM, and no device error,
source exhaustion or conversion failure. This does not close owner listening or
prove absence under every device/load. No unproven cause is recorded as fixed.

## Quality and downstream verification

Whole original source was passed through the production streaming source and
compared with separately traversed prepared PCM at 44.1/48/96 kHz device rates.
All three complete outputs are bit-identical between those paths. Native whole-
track LUFS is -10.9961/-11.0000/-11.0000, consistent with the -11 target. This
explains why a momentary -9-ish passage alone is not evidence of a target miss.

All three outputs also pass the independent, frozen SOXR16/64 finite-edge peak
and FFmpeg LUFS checker: no full-scale samples, ceiling <= -1 dBTP within the
existing .002 dB verification tolerance, and LUFS disagreement <= .11 LU.
No estimator limit, PCM identity assertion or preset snapshot was weakened.

- Desktop Rust full suite: **719 passed, 0 failed, 36 ignored**, with
  `AMS_RUN_REAL_FIXTURE=1` and the supplied source explicitly selected. This
  includes the four real-fixture tests, export/album/Volume Match invariants,
  profile/bridge contracts, callback allocation and gain-transition regressions.
- Preview regressions cover 672 format/profile/rate/precision combinations,
  same-rate one-pass measurement, exact serial/parallel peak facts, edge/silence/
  multichannel input, cancellation/retry and invalid samples.
- Strict all-target release Clippy passes. Rust formatting passes. After adding
  explicit thread-start failure handling, the five peak-meter regressions and
  strict Clippy pass again; numerical algorithms remain unchanged.
- Frontend: **913 tests passed**; production build passes. Deterministic landing
  captures refreshed after the intentional UI changes. Initial full headless
  pass: landing plus 40 app cases. The final full headless pass also passes all
  40 cases, including visible/unclipped meter labels and every named target's
  text width. Evidence: `test-output/headless/2026-09-22T16-51-24-282Z/`.
  Reviewed captures include 1360x740 flow layout and 1440x1000 Advanced; the
  target column was widened to keep full names/values visible.
- iPhone facade host all-target check and **46 tests pass**, one ignored. This
  is a compile/host contract lane, not iOS device or Mac listening evidence.
- Android facade: **26 host tests pass** and arm64 API-29 `cargo ndk check` passes.
  The first cross-check invocation used the wrong working directory and was
  corrected before the passing run; no check was skipped. This is not device listening.
  Both facade host lanes and the Android cross-check were repeated successfully
  after the final worker thread-start error handling change.
- Windows MSI and NSIS builds pass from clean source commit
  `81a98eab09e9810aed033aca753ddf70fbb4176d`, embedded stamp
  `81a98eab · 2026-09-22 09:56`. Local copies and the hash manifest are in
  `test-output/preview-preset-investigation-20260922/windows-81a98eab/`.
  NSIS SHA-256: `92b8f8dc8658fcb411cb2392e452878a38596336134f7cc79a9ac5a1de2fe3e6`.
  MSI SHA-256: `f31f7cca9d872d9604154103229daaeb53654f1bfc3f20f99a76e0be556bde15`.
  Neither installer was run. The installed application retains its baseline
  `54c60536be9b89193ca5c4edb9206d53b732f0c01cff7768729c61a454b17041` hash.

## Evidence and limits

Local evidence is under ignored
`test-output/preview-preset-investigation-20260922/`: baseline/source completion
captures, completion-pcm-check, native lifecycle/codec reports, whole-file inputs
and outputs, independent comparison, Rust/Clippy/frontend/bridge logs. Source
audio, rendered masters and these private captures are not committed.

No public push/deployment/release occurs. Packaging is complete; installation
and listening of this build remain separate and unperformed. New owner listening
remains unclaimed; the existing unrelated Windows
listening verdict is not reopened or expanded by these tests.
