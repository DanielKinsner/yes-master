# Export beta release checkpoint — prepared, not approved

The concrete Windows candidate is app **0.9.2** at
**`7cd36ab641eb35fa737ded13a06b826d2b4cfdb7`**. The proposed next public tag is
**`v0.9.2-beta.2`**, subject to final candidate selection and owner approval.
No tag, push, draft, publication, deployment, announcement or purchase occurred.
The audit-blocked beta.1 artifacts and history have not been modified.

[Installed identity and limits](../listening/2026-09-09-export-candidate.md) and
[live implementation evidence](2026-09-09-export-formats-evidence.md) are the
review record. The complete local Windows delivery folder is
`test-output/candidate-7cd36ab6/delivery/`. Its SHA256SUMS and JSON inventory bind
the NSIS/MSI, updater signatures, qualified encoder and redistribution ZIPs.
Later test/documentation commits are not silently presented as these binaries.

## Prepared release copy

Draft only; use after the final advertised platforms pass:

> YES Master 0.9.2 public beta adds FLAC, AIFF, AAC/M4A, standalone AAC and Ogg
> Vorbis delivery alongside WAV and MP3. Export individual Track masters,
> numbered Album tracks and a continuous Album master. Receipts show the actual
> delivered format and measured file values. M4A is recommended over standalone
> AAC when gapless programme-length metadata matters.

ALAC and Opus remain follow-ups. Do not claim current Mac qualification, complete
external-player compatibility or a public launch from the local Windows results.
The beta end date remains an owner decision; there is no invented expiry date.

## Items preventing release readiness

1. Mac ARM64/x64 qualification and universal/installed Apple Silicon evidence.
   Owner explicitly deferred Mac access; resume there later.
2. Outstanding U6 disconnected/player/listening/accessibility evidence and the
   unexplained first Windows Album retry pause. Preserve successful Windows
   baseline and new-format results while investigating the narrower gap.
3. Completed successful remote CI on the final chosen revision. Pushing and
   remote release actions require approval; local suites are not CI evidence.
4. Owner resolution of the concrete LGPL relinking permission draft, plus the
   complete final platform source/rebuild deliverables. Mechanical Windows
   modified-library rebuild passed; the package intentionally remains not ready.
5. Permanent updater key cross-machine backup/recovery evidence. Local signing,
   public-key verification and tamper rejection passed; no new key was generated.
6. Applicable candidate/tag/publication approval and beta end date. Paid OS
   signing remains governed by the existing D16 decision, not a new blocker.

## Publication and recovery transaction, after approval

Use the existing full-release flow for the configured `/releases/latest`
channel. A quiet full release is public and requires approval. Final workflow
artifacts need their own install/hash proof; they cannot inherit a local
installer's hash-based evidence merely because the source revision matches.

The existing 0.9.0 seed must discover, download, verify, install and relaunch
into the exact candidate while preserving its session. The privately installed
0.9.2 baseline has the same app version and is not an older-version updater
seed; it requires reinstall for this candidate. Do not reuse old updater success
as proof for newly published bytes.

Only after verified public artifacts and the real update succeed, populate
landing release metadata with the actual URLs, hashes, sizes and approved beta
end date. Run real GET/hash checks and production headless/accessibility smoke
against the current production domain before announcing availability. Keep
`RELEASE_METADATA = null` until that transaction is authorized and proved.

If a transaction fails, stop announcement and withdraw affected download CTAs;
retain immutable artifact history and use the documented recovery path. Do not
assume existing clients can be silently downgraded. Demo publication remains a
separate owner approval and the unrelated video packet stays untouched.
