# Mastering fixes: selective local integration

## Owner direction and scope

On September 16 Dan approved the recommended sequence: prepare the necessary
fixes, verify the selected combination, merge into **local main**, then resume
dynamic-range/quality experiments on a separate branch. Push, release, deployment,
spending and private-audio commits remain unauthorized. This supersedes the
earlier review pause; it does not select an automatic sonic policy.

Integration is in progress on `codex/mastering-fixes-integration` in the existing
checkout. Local main starts at `7534f612`. A fresh `git fetch origin --no-tags`
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

Four verification imports previously depended on excluded sound-experiment
modules. `verification_common.py` extracts the same hash and FFT/SOXR peak helper
algorithms. This changes verification plumbing only; no estimator or tolerance
is relaxed. Its checks are recorded with the integration evidence.

## Verification status

- Selected production/tree identity against `03df038a`: exact, before any new
  documentation or verification-helper changes.
- Frontend: 897 tests in 89 files pass; production build passes.
- Rust formatting and strict all-target Clippy: pass after the frontend build.
  The first Clippy attempt overlapped Vite replacing `dist` and failed on an
  absent generated asset; the serialized retry passed without a source change.
- Headless UI, desktop full/private-fixture tests, iPhone/Android bridges and
  targeted native checks: running; this document is not yet a merge verdict.

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
