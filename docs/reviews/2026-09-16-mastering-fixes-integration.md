# Mastering fixes: selective local integration

## Owner direction and scope

On September 16 Dan approved the recommended sequence: prepare the necessary
fixes, verify the selected combination, merge into **local main**, then resume
dynamic-range/quality experiments on a separate branch. Push, release, deployment,
spending and private-audio commits remain unauthorized. This supersedes the
earlier review pause; it does not select an automatic sonic policy.

The selected fixes are **verified for local main** on
`codex/mastering-fixes-integration` in the existing checkout. Local main starts
at `7534f612`. A fresh `git fetch origin --no-tags`
confirmed main is one commit ahead with no incoming divergence. The revised plan
is preserved. The original `codex/mastering-quality` remains at `03df038a`.

## What was selected

36 small commits replay the production and correctness-verification portions of
the original history, retaining their source commit IDs in the commit messages.
All application source, desktop/mobile manifests and locks, regression tests,
generated bindings, synthetic goldens, landing assets and WASM match the reviewed
`03df038a` snapshot exactly. This includes the final fixes for both the compensated
gain-transition dip and the inverse-gain ramp bump, never the superseded placement
alone. The completed documentation remains available as historical evidence.

The drive candidate runners, normalization/selection experiments, and saturation
variants are excluded. Existing diagnostic getters and test-only callback/PCM
probes remain because they support verification; no candidate selector is called
by production. Named preset calibration and gated features are unchanged.

The selection manifest and binary patches are retained under ignored
`test-output/integration-20260916/`. The sole pre-existing working-tree difference
was Cargo.toml line endings: its clean-filter blob matched HEAD before staging.
Its exact working-tree bytes were saved there before the branch switch.

Verification imports previously depended on excluded sound-experiment
modules. `verification_common.py` extracts the same hash and FFT/SOXR peak helper
algorithms. This changes verification plumbing only; no estimator or tolerance
is relaxed. The Windows process-counter structure was also separated from the
drive runner. All 22 retained Python modules import successfully; a fresh
synthetic check reproduces the original peak helper exactly and verifies hashes.

## Verification status

- Selected production/tree identity against `03df038a`: exact, before any new
  documentation or verification-helper changes.
- Frontend: 897 tests in 89 files pass; production build passes.
- Rust formatting and strict all-target Clippy: pass after the frontend build.
  The first Clippy attempt overlapped Vite replacing `dist` and failed on an
  absent generated asset; the serialized retry passed without a source change.
- Headless UI: landing passes and all 38 app scenario/viewport checks pass;
  relevant Track/Album captures inspected.
- Desktop: 716 tests pass overall (507 library); 33 opt-in tests are explicitly
  ignored in that suite. All four private-fixture tests execute using the
  restored `Doors Open.wav`, whose hash is in the evidence summary.
- iPhone: all-target check and 46 tests pass (one ignored). Android: 26 host
  tests and arm64 API-29 check pass.
- Fresh whole-source Imaginal and Coat at 44.1/96 kHz device rates: 4/4 outputs
  match the prepared PCM and prior preserved WAV hashes exactly. Fresh independent
  SOXR16/64, loudness, frame/rate/channel and finite/full-scale checks all pass.
  The existing wider 16-output evidence remains applicable to identical code.
- Windows MSI/NSIS package build passes at `b7a8eb35`; installers and hashes are
  copied under ignored `test-output/integration-20260916/package/`. No installation
  or release is performed. Later changes affect test-only probes/documentation.
- Actual production lifecycle passes. Under concurrent whole-file verification,
  cold playback/first meter are 605/660 ms; first preparation/output application
  49.687/49.865 s; new target 2.640/2.820 s; cached target 75/250 ms; paused cached
  resume applies in 187 ms. Session: 54.630 s wall, 53.719 s observed CPU and
  731 MiB peak working set. This is not an isolated speed comparison.
- Native callback probes on the working VP2756 monitor output: 652 callbacks
  during 1,200 gain edits and 607 during 120 EQ edits, with zero deadline misses,
  device/converter errors or exhausted samples. Maximum callback work is
  1.773/1.831 ms. The explicit device-default policy grants 480–1,056 frames;
  no fixed-256 claim is made.

The [machine-readable summary](evidence/2026-09-16-fixes-integration/verification.json)
records output hashes, timing, failures and package identities. The adjacent
selection manifest maps each replayed commit to its source. All new raw outputs,
logs, copied probes and failure records remain under the ignored integration
directory. No private audio is committed.

### Native device limitation diagnosed separately

**Post-power-interruption update, September 16:** the same empty-production-stream
probe now opens the default Realtek output. Fresh muted Realtek gain/EQ callback
checks pass: 659 callbacks during 1,200 gain edits and 609 during 120 EQ edits,
zero deadlines/device/converter errors or exhausted samples. Maximum callback
work is 1.690/1.701 ms; granted device-default buffers are 480–1,056 frames.
This uses the restored 48 kHz Imaginal source through 96 kHz file processing to
48 kHz output. See the [recovery evidence](evidence/2026-09-16-dynamics-research/post-power-native-v1.json).
No device preferences changed. The failure below remains historical with its
underlying cause unresolved; it is no longer the current endpoint-open result.
This update does not claim fixed-256, installed or by-ear validation. An initial
incorrect exact test-name filter ran zero tests and was not counted; the corrected
filter ran the expected single device diagnostic.

The old timing probe's fixed-256 request fails twice on the current default
`Speakers (Realtek(R) Audio)` with `0x887C003A`; an explicit default-buffer trial
also fails. A new test-only empty-production-stream diagnostic confirms that this
endpoint currently cannot open, while the VP2756, ASUS noise-canceling, BenQ and
Realtek Digital outputs open. The ASUS Virtual Speaker also fails (`0x8889000A`).
The production fallback sequence succeeds on VP2756, matching the route used for
fresh callback measurements. No Windows device preferences were changed.

The underlying Realtek failure cause is unresolved; neither a passing monitor
run nor the earlier historical Realtek run clears current Realtek-specific proof.
The probe now supports explicit device/default-buffer selection and records
requested versus granted scope. This diagnostic-only change is absent from the
application build. It does not conceal the failed runs or loosen a timing limit.

## Scope of remaining work

Current preview text says **Measuring preview level…**. It describes measurement
activity, not a promise that a prepared revision has already reached the device.
Existing backend revision probes separately establish output application. A new
final-result UI contract remains later B3/C3 work; it is not silently marked done.

The selected fixes preserve the documented static whole-file PCM protection and
the specific corrected gain-transition cases. Existing tests cover interrupted
gains, coefficient changes, seek, fades, finite SRC and pending/revision handling.
They do not establish a universal reconstructed-peak ceiling during every possible
live settings edit or at the DAC. Broader dynamic-boundary qualification remains
open and must precede any broader guarantee or dependent policy integration.

The prior 45-second native result was first background landing/output preparation
at a particular source/file/device rate combination under research load, not
ordinary import/analysis timing and not a measured before/after slowdown. Dan's
current `npm run tauri dev` observation was that analysis felt unchanged. Keep
source analysis, preparation, edit settling, playback response and session cost
separate in future comparisons.

No fresh subjective listening, installed-artifact, Mac or remote-CI pass is
claimed. Those remain distinct release/candidate evidence under the live
[beta gate](../plans/beta-go-no-go.md). Local integration does not publish a beta
or adopt new preset voicing. The recorded f64 filter CPU cost and finite-reference
limitations remain in the [review guide](2026-09-16-mastering-fixes-review.md).

## Next research checkpoint

After verified local integration, branch from that main state and restore only
the needed experiment tooling from `03df038a`. Preserve original sources, frozen
baseline, failed witnesses and prior reports; verify source/settings/algorithm
identity before reusing evidence. Resume from the completed lower-drive grid,
not the original investigation. Compare simple processing with conditional,
bounded search and measure each stage separately. No automatic dynamics limit,
fallback policy, preset retuning or saturation candidate is adopted by this step.
