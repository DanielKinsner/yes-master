# Listening follow-up: concrete output choices

The owner has settled the codec, filename, manifest and loading choices below.
Local implementation and verification are recorded in checkpoint 9 of the
[follow-up plan](2026-09-05-listening-follow-up.md). These are not public release
claims. Old owner outputs are never deleted.

## Album names and manifest

**Owner naming decision:** Track `Song_mastered.wav`; Album
`01-Song_mastered.wav`, using one underscore. Album numbering follows the final
left-rail arrangement at export. Current code preserves that ordering from rail
tracks through analysis inputs, plan positions and output names. Change the
default suffix; preserve existing exports, the collision-safe album folder and
continuous audio. Naming is implemented in the local export checkpoint recorded in the follow-up plan.

**Owner decision:** use `Album/metadata/manifest.json`. The owner accepted this
recommendation but wants users to understand the extra file. Explain in the
in-app receipt that the metadata folder contains supporting export details and
is not needed to play or share the audio. The metadata path and receipt explanation are implemented in that local checkpoint.

Options considered in the interview:

1. `Album/metadata/manifest.json`. Keep portable machine-readable evidence with
   the audio, away from playable files. Update returned paths, receipt links,
   creation and cancellation cleanup together.
2. Stop writing JSON; use the in-app Album receipt. This requires persisted
   reports and a defined reopen/history action, optional legacy manifest paths,
   and removal of the mandatory-file assumption in both native bridges. A
   receipt held only in memory is insufficient for this option.

## Proposed first extra-format increment

**Owner clarification:** retain WAV and add MP3 as a smaller-file, broadly playable
choice in the normal mastering export. No separate converter or `Original
conversion` selector. Preserve existing DSP/delivery behavior at 0% Intensity;
do not add bypass or special explanations for workarounds. This supersedes the
earlier conversion-without-mastering interpretation. Ordinary export claims and
meters remain truthful. **Owner scope decision:** Standard and Advanced Track
export first; the implementing agent verifies the encoder and delivered-file
contracts, then proceeds to Album batch/continuous export without another owner
approval. Album still requires its own verification. **Owner quality decision:**
320 kbps default, with 256, 192 and 128 kbps also available in the same selector
in Standard and Advanced. WAV remains the default export format.

The owner explicitly clarified that 0% was only an example of a possible user
workaround. MP3 uses the chosen preset, Intensity and controls just like WAV;
format selection must not reset mastering settings or select 0%.

Selected implementation: embedded LAME 3.100 through pinned `mp3lame-encoder`
0.2.5 / `mp3lame-sys` 0.1.11, enabled only for desktop app-runner builds.
Windows native export and independent FFmpeg decode passed. The exact dependency
licenses are retained under `docs/third-party/`; see `THIRD_PARTY_NOTICES.md`.
Mac compilation/packaging and the release artifact's notices/source/relinking
arrangement still require release verification.
The [wrapper source and usage](https://github.com/DoumanAsh/mp3lame-encoder) and
[LAME upstream README](https://github.com/lameproject/lame/blob/master/README)
are primary implementation references. Windows flushing/delay and decoder
checks passed; the remaining Mac/package and redistribution checks are recorded
in the release gate. No external encoder installation
or network service should be necessary for the user's export.

The feature must keep collision-safe destinations, source protection, temporary
output cleanup and cancellation. The selected format drives extensions/dialog
filters. Supported MP3 output rates/channels require an explicit conversion policy
and tests against the chosen encoder; high-rate input must not simply be passed
through to an unsupported MP3 configuration.

Receipts must identify the delivered encoding and quality. Decode the completed
MP3 and measure that delivered signal, with documented delay/padding handling;
pre-encode PCM peaks are not delivered-file peaks. Acceptance includes an
independent decoder/player, duration/rate/channel checks, normal mastering-path verification,
WAV regression and unchanged source/prior output bytes. Update Standard's fixed
WAV promise, help and internal product docs with the implemented scope, without
publishing unverified claims.

## Loading workflow discussion

The offline stage report is linked from the implementation plan. Whole-track
DSP, not the final LUFS gate, dominates the measured wait. Per-batch analysis is
sequential; the old UI published results only at batch completion, explaining
the owner's observation that all clips finished together. Checkpoint 9 changes
publication to per-track readiness and real per-track progress. Original options were
to publish each completed track/profile earlier while protecting audition, or
keep a clear up-front preparation stage with real per-track progress. Neither
removes later Preview LUFS costs. Measure time to first usable track, total batch
time and concurrent playback impact during implementation. **Owner selected
incremental readiness in the September 5 follow-up interview:** make each ready
track usable, show progress for the rest, and give playback priority. Concurrent
audition responsiveness still needs verification. **Owner also selected automatic
Preview LUFS for the selected track when ready, if already enabled for the mode:**
show "Measuring," cancel obsolete work on track switches, and protect playback
with bounded work. Other imported tracks wait until selected. Enabled state and
defaults stay unchanged. Preparation uses one heavy-worker permit shared with
live measurements, cancellation, and a bounded last-result cache keyed by source
metadata and resolved processing settings. Checkpoint 9 separates native readiness
and control-state evidence from audio deadline proof.
