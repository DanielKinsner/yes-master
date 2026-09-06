# Listening follow-up: concrete output choices

These are unresolved product choices, not implementation approval requests for
the independent fixes. Existing output behavior remains until an answer is
recorded; that is not a selected default. Old owner outputs are never deleted.

## Album names and manifest

Proposed per-track filename: `01-Song__master.wav`, matching Track Master's
existing suffix. Alternative: align both surfaces on `__mastered`. Preserve the
collision-safe album folder and continuous audio in either case.

Exactly two manifest choices:

1. `Album/metadata/manifest.json`. Keep portable machine-readable evidence with
   the audio, away from playable files. Update returned paths, receipt links,
   creation and cancellation cleanup together.
2. Stop writing JSON; use the in-app Album receipt. This requires persisted
   reports and a defined reopen/history action, optional legacy manifest paths,
   and removal of the mandatory-file assumption in both native bridges. A
   receipt held only in memory is insufficient for this option.

## Proposed first extra-format increment

Retain WAV and add MP3 to Standard and Advanced Track export. Present an explicit
`Mastered` / `Original conversion` source choice independent of audition A/B.
Original conversion decodes the source and bypasses the entire mastering chain;
Intensity zero is not bypass. Proposed MP3 default: 320 kbps with a quality
selector. The owner still needs to choose Track first versus Track and Album
together, and confirm this quality/surface scope.

Encoder feasibility candidate: embedded LAME via a maintained Rust wrapper,
bundled for offline use. No new dependency has been selected or installed.
The [wrapper source and usage](https://github.com/DoumanAsh/mp3lame-encoder) and
[LAME upstream README](https://github.com/lameproject/lame/blob/master/README)
are primary starting references. Before adoption, validate Windows/macOS builds,
encoder flushing/delay metadata, applicable LGPL redistribution obligations and
the actual pinned crate's license/build behavior. No external encoder installation
or network service should be necessary for the user's export.

The feature must keep collision-safe destinations, source protection, temporary
output cleanup and cancellation. The selected format drives extensions/dialog
filters. Supported MP3 output rates/channels require an explicit conversion policy
and tests against the chosen encoder; high-rate input must not simply be passed
through to an unsupported MP3 configuration.

Receipts must identify the delivered encoding and quality. Decode the completed
MP3 and measure that delivered signal, with documented delay/padding handling;
pre-encode PCM peaks are not delivered-file peaks. Acceptance includes an
independent decoder/player, duration/rate/channel checks, mastering-bypass proof,
WAV regression and unchanged source/prior output bytes. Update Standard's fixed
WAV promise, help and internal product docs with the implemented scope, without
publishing unverified claims.

## Loading workflow discussion

The offline stage report is linked from the implementation plan. Whole-track
DSP, not the final LUFS gate, dominates the measured wait. Per-batch analysis is
already sequential, while results are published at batch completion. Options are
to publish each completed track/profile earlier while protecting audition, or
keep a clear up-front preparation stage with real per-track progress. Neither
removes later Preview LUFS costs. Measure time to first usable track, total batch
time and concurrent playback impact before selecting either. Automatic Preview
LUFS on import remains dependent on resource/cancellation policy; do not queue a
whole-track render for every imported file by default.
