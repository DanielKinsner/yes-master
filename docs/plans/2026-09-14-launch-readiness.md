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
   October 31 was provisional, not a final publication decision.
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
