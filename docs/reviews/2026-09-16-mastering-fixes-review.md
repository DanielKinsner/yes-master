# Mastering fixes: owner review before integration

## Current direction

Dan's September 16 instruction supersedes the earlier continuous implementation
queue: pause further sound experiments while he reviews the local commits.
Review the correctness, playback, measurement, performance and readout changes
for possible main integration and beta inclusion. Resume sound-quality research
on a separate branch only after the fixes are approved and merged to main.
No merge, push, release, deployment, spending or private-audio commit is authorized
by this review step. No experimental run or audio-code change accompanies it.

The original motivation was Dan's comparison of YES Master and LANDR outputs,
where he heard more dynamic range in the LANDR result. A misunderstanding in
another Codex task contributed to concern that the mastering approach as a whole
was wrong, particularly excessive saturation followed by compression/limiting.
Dan now wants to separate demonstrated defects from opportunities to improve
the sound. This records his context and sequencing decision; it does not establish
LANDR's internal processing or prove a general architectural failure in YES Master.

The proposed character-preserving automatic policy remains unapproved and is
deferred with the research. No sound-preference answer is needed for this review.

## Exact review target

- Existing checkout: `C:/Users/SM - Dan/Documents/GitHub/yes-master`.
- Branch: `codex/mastering-quality`.
- Local main/base: `7534f6125b6f74da673c87d417f1b637ebc1c94c`.
- Reviewed implementation/research snapshot:
  `f88f042b689b6de87ef295d97fb392921dde761b`.
- Subsequent September 16 changes only record this pause and review guide.
- Local main is the common ancestor. Its displayed tracking state is ahead of
  `origin/main` by one; no remote refresh or integration was performed here.

The history contains both application changes and research tooling. The groups
below identify review scope; they are **not an independently tested cherry-pick
recipe**. Some commits also carry diagnostics, fixtures, documentation or generated
assets. Preserve the complete history and ignored evidence while reviewing.
Before an approved selective integration, reconcile dependencies and test the
actual selected result. A full branch merge would also carry the research tools.

## Application changes to review

| Group | What changed in the app | Key commits / evidence |
| --- | --- | --- |
| Density and compression readouts | Auto thumb reflects the requested preset value; the UI distinguishes requested/pre-Adapt values from actual resolved compression. Saved null and Manual initialization remain intact. | `1d71f139`, `7135a6ff`; D1/D2 in the [ledger](2026-09-15-mastering-quality-implementation-evidence.md) |
| Sample conversion and render boundaries | Explicit SRC input/draining and exact output frame counts; cancellation survives landing measurement; tail flushing avoids growing the whole-track allocation. | `c3ddb67b`, `791dcd30`, `ff94f2c4`; A1/A2 and render regressions |
| Peak measurement and output protection | Ceiling protection works when LUFS is unavailable; qualified final-PCM measurement/protection covers Track, rendered preview and Album programme joins. Encoded receipts use decoded output, including unclamped MP3 peaks. | `21f9543d`, `8603ddf8`, `2ffce604`, `370bdc3f`, `7834f4fa`; B1/B2/B4 and completed B3 slices |
| Live playback and gain transitions | Actual device configuration, file/device conversion and selected encoding determine the preview route. Revision tracking distinguishes prepared settings from emitted output. Verified device correction and Volume Match ramps avoid the demonstrated dips/bumps. | `e0e7bdef`, `38ac955e`, `60aa0f6b`, `3fecbbe9`, `6f6d20e6`, `9ad90a10`, `9ebd2f81`, `09740a65`, `22a1aada`; [device preparation](2026-09-15-device-gain-preparation.md), [gain transitions](2026-09-15-combined-gain-transition.md) |
| Reuse and memory | Bounded preview/source/file/device preparation caches reuse matching work; invalidation, cancellation and byte limits remain part of the contract. | `9026e2bb` plus the device-preparation commits above; measured preparation/application/session costs in the ledger |
| DSP numerical correctness | Recursive audio filters retain f64 feedback state/arithmetic while preserving their designed coefficients and f32 stage outputs. This is a real DSP correction, with a measured CPU cost and changed synthetic golden PCM. Old goldens are preserved. | `3f08d249`; [gain-copy diagnosis](2026-09-15-drive-gain-consistency.md) |

Review the final combined change. In particular, `9ebd2f81`'s original device-gain
placement was superseded by `09740a65`, and the combined gain transition requires
`22a1aada`. Do not extract the earlier placement alone as an approved fix.

These are more than unrelated bug fixes: they include planned A/B/D implementation
and performance work. They can change samples or output level through correctness
and protection. Existing preset calibration and the gated Adaptive Compressor,
Phase-B confidence and album-character policies were not enabled or retuned.

## Experimental work to park

The automatic drive selector, normalized-analysis policy, character limits,
fallback/search choices and saturation continuity/antialias candidates remain
research. Representative later commits include `d3caa445`, `f5248831`, `0383516c`,
`fb449bc2`, `e63064ea`, `f88f042b` and the E experiments `ce7ea66f`, `7e4a7327`,
`a5d4ad56`, `529ec92c`. Their presence in Git does not activate their proposed
behavior in normal mastering. Some research-support hooks live beside app code;
review by call path and behavior, not filename alone.

Preserve the frozen baseline, completed reports, copied executables, failed
witnesses and restored private fixtures. Do not start more candidate renders,
acquire/open new holdout material, tune sonic limits or adopt a candidate during
this pause. The [lower-grid result](2026-09-15-drive-lower-grid-results.md) is the
last completed experiment; C1/C2/C3 and E adoption are not complete.

After approved fixes are integrated, create the future sound-research branch
from that verified main state. Bring forward only the needed research tooling
and retained evidence, checking source/settings/algorithm identity before reuse.
Do not automatically carry an experimental policy into the beta fix set.

## Evidence and limits for the review

- The latest app-code checkpoint passed 507 library tests and the full restored
  private-fixture lane. Applicable earlier bridge/UI checks and whole-file
  verification are linked in the ledger; later research-only commits did not
  re-certify every platform at their exact commit.
- Whole-file corrected device outputs and the demonstrated gain-transition
  regressions pass. B3 still lists broader dynamic boundaries and the final-result
  UI contract as open. This guide does not mark that checkpoint complete.
- Latest recorded loaded device session: first checked preparation/application
  about 44.962/45.142 s; new target 2.010/2.244 s; cached target 75/191 ms. Source
  playback starts separately. Review the user-facing cost; these observations are
  not a blanket responsiveness verdict or an isolated speed comparison.
- Wider filter arithmetic cost 19-29% more chain CPU in the recorded paired
  trials. Its numerical benefit and native workload results should be reviewed
  together. Native callback requests for 256 frames actually received 480-1056;
  those runs are not a locked-256 pass.
- Qualified peak measurement retains finite-reconstruction/reference limits;
  the full official EBU sequence set was not acquired. Lossy delivered peak
  excesses remain truthful/advisory; no automatic re-encoding policy was adopted.
- No new owner listening pass, installed-artifact pass, Mac proof or exact-commit
  remote CI is supplied by this documentation step. The current
  [release gate](../plans/beta-go-no-go.md) remains authoritative.

Dan's intended beta scope is the reviewed fixes. Unfinished tonal research does
not become a new blanket beta requirement. Existing correctness, integration,
installed/platform and release requirements still apply to the selected candidate.

## Read-only review commands

From the existing checkout, these pin the review to the frozen snapshot even if
subsequent documentation commits advance the branch:

```powershell
git log --oneline 7534f612..f88f042b
git diff --stat 7534f612 f88f042b
git diff 7534f612 f88f042b -- src-tauri/src/dsp.rs
git diff 7534f612 f88f042b -- src-tauri/src/sample_rate.rs src-tauri/src/output_protection.rs
git diff 7534f612 f88f042b -- src-tauri/src/audio src-tauri/src/audio.rs src-tauri/src/sources.rs
git diff 7534f612 f88f042b -- src/components/AdvancedPanel.tsx src/hooks/useTrackMaster.ts
```

Next action is the owner's review and disposition of the local changes. Further
engineering requested by that review can be handled as a focused follow-up;
the old continuous experiment queue must not resume automatically.
