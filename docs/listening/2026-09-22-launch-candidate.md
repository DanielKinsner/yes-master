# September 22 launch candidate review

Prepared application revision: `e6db1f467838d94aec9b3cfbc4272e1b977a8adb`,
version 0.9.2. Windows is installed locally; your original eight-track session
is restored. Help and export receipts display `e6db1f4` as the build prefix.
The existing September 9 Windows listening approval remains valid for its
recorded scope. This page addresses the new artifact and unresolved launch work.

[PR 47](https://github.com/DanielKinsner/yes-master/pull/47) identifies the work.
The unpublished `yes-master-v0.9.2-manual-9` draft contains the Windows x64 setup
and MSI, universal Mac DMG, updater bundles/signatures, exact source and encoders.
Manual drafts 7 and 8 are superseded; beta.1 remains frozen and blocked.

Agent verification is recorded in the
[launch execution record](../reviews/2026-09-21-launch-preparation.md): all three
platform encoder/relink qualifications, both installer builds, 14 downloaded
asset hashes, three valid signatures with tamper rejection, installed Windows
M4A output and independent decode/measurements. The local review ZIP is under
`test-output/launch-20260921/YES-Master-e6db1f46-release-review.zip`.
Full [CI 35700809284](https://github.com/DanielKinsner/yes-master/actions/runs/35700809284)
completes successfully at the frozen candidate. This proves those mechanical
checks; it is not a public-release approval.

## Remaining owner evidence and decisions

1. **Targeted final listening:** on the installed build, use a familiar source
   and compare its new compressed export with the in-app mastered result at
   comparable playback level. Report any audible glitch, unexpected level or
   missing beginning/end. The native test M4A is retained locally as
   `test-output/launch-20260921/native-exports/Final-e6db1f46.m4a`.
   There is no need to repeat the unchanged September 9 questionnaire.
2. **Real Mac:** install the candidate universal DMG on the M4, confirm the
   `e6db1f4` Help footer, import a familiar track, audition Original/Mastered,
   and export/play a WAV and an M4A. Record actual startup/import/playback/export
   behavior. Mac runner tests do not establish this installed experience.
3. **Recipient relinking permission:** decide the
   [concrete proposed exception](../third-party/lgpl-relink-exception-DRAFT.md).
   It has not been granted. All three modified-library rebuilds pass, but
   successful packaging does not grant the missing permission.
4. **Permanent updater-key recovery:** confirm a secure independent backup and
   successful recovery of the existing key/password on the other machine.
   Do not paste either secret into chat. Local signing success is already proven.
5. **Public launch transaction:** after the applicable gates and beta.1
   disposition are resolved, explicitly approve the final release/tag and site
   activation. Rebuild/requalify any changed distribution terms; do not merely
   flip the current source manifests to ready. Verify the authorized public
   updater path, then activate exact download metadata and the actual publication
   date plus 56 calendar days. Pricing remains undecided; beta access is free.

Full screen-reader and physically disconnected checks remain unverified. One
non-default ASUS virtual endpoint fails to open; its cause is unresolved, while
the default Realtek and two other exposed endpoints open. These limits remain
in the execution record. Paid OS publisher signing stays post-beta advisory.
The site changes have not been deployed and no release has been published.
