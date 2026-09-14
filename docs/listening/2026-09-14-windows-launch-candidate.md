# Windows launch candidate — September 14, 2026

**Installed candidate; public release remains unapproved and not ready.**

## Identity and completed checks

- Source `cf3ddcc9e1f1ff955c92cdd9d72f30339b931b45`, app 0.9.2.
- Clean detached worktree `../yes-master-launch-cf3ddcc9`; fresh Cargo target
  `src-tauri/target/launch-cf3ddcc9`.
- Compiler-recorded build stamp: `cf3ddcc9 · 2026-09-14 08:26`, without a dirty marker.
- NSIS SHA-256: `f55c25b484231bcaebb4e509b91fc41dd686c4042b01b800ef5c0b3aef0d93dc`.
- Installed application SHA-256:
  `0107c147aa96cc63f3c4763924383008b090ecd392c361fbe1a0fbc9fd10c9ab`.
- Installed encoder SHA-256:
  `9a473b40770b9b321187abdac77ac71837ee5c40784cab9a1fe876377ac9d86f`.
- Both MSI and NSIS built. NSIS installed with exit 0. Session JSON was identical
  before and after installation. MSI has not been separately installed.
- Both installer updater signatures verify against the committed permanent key.
  One-byte mutations fail verification. These are not Authenticode signatures
  or an installed updater transaction. The key/passphrase remain outside git.
- Exact archived source rebuilt into a full Windows application with the private
  LAME 3.101 marker. Three existing MP3 tests and the modified-library encode/
  decode proof passed. The modified application was never installed. Mechanical
  rebuild evidence does not grant the pending recipient permission.

Local evidence: `test-output/launch-20260914/candidate-cf3ddcc9/`.
Installers and signatures are in its `delivery/` directory. These ignored files
do not travel with git. No private audio is included in that directory.

## Changed behavior and engineering evidence

The M4A movie clock now matches the delivered audio rate. With independent
FFmpeg 9.0.1, the prior 88337-frame case reproducibly decoded as 88332 frames;
the corrected file decodes as exactly 88337. A regression through the real
application encoder failed before the fix and passed after it. Eight rate,
channel and length combinations pass exact-length decoding. Track and Album
format matrices pass (five Track, two Album tests), as do 40 package cases.

Full ordinary Rust plus the private fixture lane: **643 passed, 18 ignored,
42 suites**, including all four private-fixture cases. Strict all-target Clippy
passed. These are local engine checks, not owner listening or remote CI.

## Installed interaction boundary

The earlier installed build was inspected through Windows Computer Use. After
the new installer completed, Computer Use reported a physical-Escape stop when
the agent attempted to launch the replacement. All further desktop interaction
stopped. **A new-candidate launch, playback, export and keyboard pass is not
claimed.** The earlier September 9 baseline owner PASS remains valid for its
named artifact and unchanged scope.

Creating a portable NVDA copy was separately rejected by automatic approval
review as "blocked by policy"; it was not retried through another route. NVDA
screen-reader evidence remains open, distinct from native UI Automation semantics.

## Remaining targeted checks

- Launch this installed candidate and verify the displayed clean build stamp.
- Check changed M4A export in Standard/Advanced and Album, then play the saved
  files in an external player. Confirm the affected output by ear.
- Repeat Album cancel/retry while auditioning with evidence for the first pause.
  Historical trace and retained app logs were reviewed: the playhead stops at
  0:53 during encoding, with no recorded device-loss event. The original test
  script's explicit Pause is after the observation loop, so it does not explain
  the trace. No root cause or blanket audition PASS is asserted.
- Installed keyboard/NVDA and disconnected export remain open.
- Mac installed/VoiceOver, final remote CI, recipient permission, cross-machine
  key recovery and actual public updater transaction remain separate gates.

The owner selected **eight weeks from actual launch**. Do not start the beta
clock during candidate preparation or repeat the provisional October 31 question.
