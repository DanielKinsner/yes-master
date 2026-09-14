# Launch readiness — September 14, 2026

**Public launch: not ready. A demo video is optional.** This assessment refreshes
the September 9 release checkpoint against live GitHub state and current source.
The owner requested launch preparation and suggested removing the demo link.

## Verified current state

- Local and remote main: `af406855fbf5f0057b97d24be5a197a0082c0589` before this
  local landing/docs change.
- [CI run 34402480068](https://github.com/DanielKinsner/yes-master/actions/runs/34402480068)
  completed **failure** at that exact revision. Both Mac encoder build jobs stop
  at `scripts/build-audio-encoders.sh: line 56: configure_host[@]: unbound variable`.
  Windows encoder qualification reports M4A decoded frames **88332/88337**.
  This observation does not yet distinguish encoder behavior from verifier or
  independent decoder behavior. Preserve the frame contract while investigating.
- Desktop Windows/bridge and macOS/iPhone jobs were skipped. Web headless,
  dependency advisories, Android host and both snapshot jobs passed.
- GitHub release listing still contains only draft `v0.9.2-beta.1`, whose
  artifact blockers are recorded in the live go/no-go checklist. Do not publish it.
- `https://yes-master.vercel.app` returns HTTP 200 with the expected title.
  Current source still has `RELEASE_METADATA = null`; HTTP availability does not
  certify downloads or full production browser behavior.
- Existing September 9 installed Windows owner PASS remains valid for
  `07021f1b`. New-format installed evidence remains tied to candidate `7cd36ab6`.
  No unchanged listening questionnaire needs repeating.

## Shortest path to a public beta

1. Resolve both encoder CI failures; obtain completed green CI on the selected
   candidate revision. Build replacement artifacts instead of publishing beta.1.
2. Complete the changed-format Windows checks: investigate the unexplained
   audition pause, external-player and disconnected operation, accessibility,
   and targeted installed listening. Existing successful evidence stays valid.
3. Qualify both Mac architectures and test the universal installer on the owner's
   Mac. A Windows-first launch is a possible scope decision, not the current
   two-platform contract: landing metadata and release workflow currently require
   Mac assets as well. It would not waive the remaining Windows gates.
4. Resolve the existing relinking-permission draft, finish final source packages,
   and prove permanent updater-key backup/recovery. Confirm the beta end date;
   **Resolved September 14:** the owner selected eight weeks from actual launch
   (publication date plus 56 calendar days), superseding provisional October 31.
5. Approve the exact replacement candidate and public transaction. Verify final
   installer bytes and the actual updater install/relaunch, then activate real
   landing download metadata and verify the deployed site before announcing.

Newsletter signup and paid OS certificates are already non-blocking for the free
beta. No demo production, redesign, custom domain purchase, or new feature set is
needed to satisfy this launch request.

## Local landing change

Removed the inactive Watch demo button, its unavailable-video note and unused
styles. Updated the existing browser check to require their absence.
`npm run verify:headless` passed: build/typecheck, landing responsive and
accessibility checks, and 38 app scenario/viewport checks. Evidence directory:
`test-output/headless/2026-09-14T15-05-19-894Z/`. The phone hero screenshot was
also visually inspected. This is local browser evidence, not native or remote CI
proof. No push, deploy, tag or release publication was performed during this
assessment. The unrelated video packet is preserved.

## Implementation follow-through

The owner subsequently authorized working through this checklist. Engineering
fixes, verification and CI preparation proceed; the final candidate/public
transaction, recipient permission and unresolved dates still need their specific
evidence/decisions. The planned launch remains Mac + Windows unless changed.

### Encoder corrections

- Reproduced the exact Windows CI failure locally with independent FFmpeg 9.0.1:
  an 88337-frame M4A decoded as 88332 frames. With the same encoder and source,
  setting the MOV movie timescale to the output sample rate yields 88337 frames.
  This corrects sample rounding in edit-list duration; no audio samples or
  verification tolerance are removed. Applied in the real desktop encoder and
  the package qualification matrix. [FFmpeg's movie_timescale option](https://www.ffmpeg.org/ffmpeg-formats.html)
  documents the container clock control.
- A new real-encoder regression failed before the application fix and passed
  after it. Five Track tests and two Album tests pass, including eight independent
  gapless M4A decodes. All 40 package format cases pass with FFmpeg 9.0.1.
- Mac configure arguments now always contain the common flags. This avoids
  expanding an empty array under macOS Bash 3.2 with `set -u`. Shell syntax passes;
  actual Mac source builds remain the remote CI proof.
- CI now supplies the independent decoder to the real-engine regression and
  retains qualification/build evidence on failures as well as successes.
- The first Mac retry passed configuration, then exposed libvorbis 1.3.7's
  obsolete `-force_cpusubtype_ALL` linker flag. The retained build script now
  removes only that flag from the extracted configure script on Mac; pinned
  upstream archives remain unchanged. The next CI run supplies execution proof.

### Fresh Windows candidate and verification limits

[Candidate cf3ddcc9](../listening/2026-09-14-windows-launch-candidate.md) is built
and installed with hashes, permanent-key signature/tamper checks and preserved
session state. The full local Rust/fixture lane passed 643 tests; strict Clippy
passed. Desktop control was stopped with Escape before the new-candidate launch,
so its installed playback/export/keyboard checks remain open. Automatic approval
review separately blocked portable NVDA creation. No desktop input followed
the stop and no NVDA workaround was attempted.

The exact cf3ddcc9 source archive also rebuilt into a complete Windows application
with a deliberately modified LAME marker; three existing MP3 tests and the real
modified-library encode/decode proof passed. A reusable script now prepares and
records that mechanical proof, with CI coverage for all three desktop targets.
It deliberately leaves recipient-permission and release gates unresolved.
