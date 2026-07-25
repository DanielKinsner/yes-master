# Release, Updater & Optional Code-Signing Setup

The public beta has a **$0 release path** (D16, 2026-07-20). It produces a
universal Mac build and Windows MSI/NSIS installers, signs every updater
artifact with the permanent Tauri key, generates SHA-256 checksums, and leaves
the GitHub Release as a draft for owner review.

Apple notarization and Windows Authenticode are optional post-beta trust
upgrades. Without them, users will see Gatekeeper or SmartScreen prompts; link
`docs/BETA_INSTALL.md` anywhere the beta is offered.

## How a $0 beta release works

1. Keep the version identical in `package.json`, `src-tauri/tauri.conf.json`,
   and `src-tauri/Cargo.toml`.
2. Commit, then create and push a beta tag such as `v0.9.0-beta.1`. Tagging and
   pushing are owner-confirmed public actions.
3. The workflow builds one `universal-apple-darwin` Mac artifact plus Windows
   MSI/NSIS artifacts. It always uses the free updater-signing secrets.
4. The workflow creates a **draft, non-prerelease** GitHub Release. The beta
   identity remains in its tag/title; non-prerelease is required because the
   app reads GitHub's `/releases/latest` channel after publication.
5. A final audit requires the installers, updater archives/signatures,
   `latest.json`, and `SHA256SUMS.txt`. It fails if the draft is incomplete.
6. The owner downloads, installs, listens, and checks the draft. Only the owner
   publishes it.

The workflow can also be dispatched manually from GitHub Actions. It still
creates a draft; it never silently publishes.

## Permanent Tauri updater signing — configured

Updater signing is separate from Apple/Azure OS signing and costs nothing. It
proves that an update was produced by the holder of the permanent private key.
Every shipped app trusts the public key in `src-tauri/tauri.conf.json`, so this
identity must remain stable.

Configured 2026-07-20:

- The permanent public key is committed in `plugins.updater.pubkey`.
- GitHub Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` exist.
- The encrypted private key is outside the repository at
  `C:\Users\SM - Dan\.yes-master\keys\yes-master-updater.key`.
- Its local password-recovery file is at
  `C:\Users\SM - Dan\.yes-master\keys\yes-master-updater.key.password.dpapi`.
  That DPAPI file is tied to this Windows account/machine.

**Owner backup required:** store the encrypted private key and its passphrase
in a cross-machine password manager. Never commit either. If the key is lost,
already-installed apps cannot accept future automatic updates and users need a
manual reinstall.

Release builds apply `src-tauri/tauri.updater.conf.json`, which emits and signs
the updater archives and `.sig` files. The app reads `latest.json` from:

`https://github.com/DanielKinsner/yes-master/releases/latest/download/latest.json`

The remaining beta gate is an end-to-end 0.9.0 → 0.9.1 update proof on a real
installed build. Because GitHub's `/latest` channel excludes drafts, this proof
requires the owner to publish the full 0.9.1 release quietly before the public
announcement; its files become publicly accessible at that moment.

## Optional macOS trust upgrade (post-beta)

Goal: remove most Gatekeeper friction with a Developer ID Application
certificate and Apple notarization.

1. Join the Apple Developer Program and create a **Developer ID Application**
   certificate.
2. Export the certificate and private key as a password-protected `.p12`.
3. Base64-encode the `.p12`.
4. Create an Apple app-specific password and record the Team ID.
5. Add the complete GitHub secret group:

   | Secret | Value |
   |---|---|
   | `APPLE_CERTIFICATE` | base64 `.p12` |
   | `APPLE_CERTIFICATE_PASSWORD` | `.p12` password |
   | `APPLE_ID` | Apple account email |
   | `APPLE_PASSWORD` | app-specific password |
   | `APPLE_TEAM_ID` | Developer Team ID |
   | `KEYCHAIN_PASSWORD` | random temporary CI-keychain password |

The workflow signs/notarizes only when the **entire** group exists and fails on
a partial group. No code change is required.

## Optional Windows trust upgrade (post-beta)

Goal: add Authenticode through Azure Artifact Signing. Until then, Windows
builds remain usable after the user accepts SmartScreen.

1. Create an Azure Artifact Signing account and certificate profile; complete
   identity verification.
2. Create a Microsoft Entra App Registration, client secret, and grant the
   certificate-profile signer role.
3. Add the complete GitHub secret group:

   | Secret | Value |
   |---|---|
   | `AZURE_TENANT_ID` | Entra tenant ID |
   | `AZURE_CLIENT_ID` | App Registration client ID |
   | `AZURE_CLIENT_SECRET` | App Registration client secret |

4. Replace the placeholders in
   `src-tauri/tauri.windows-signing.conf.json` with the account, certificate
   profile, and regional endpoint.

The workflow uses the current `artifact-signing-cli`. It signs only when the
complete Azure group exists and fails on a partial group.

## Activating the landing download (added 2026-07-25, U5)

Publishing a release does **not** turn the landing page's download on. The page
renders from `src/landing/release-config.ts`, where `RELEASE_METADATA` ships as
`null`. Until it is populated with a release that passes every check below, the
page shows an inactive action and a plain reason, and points at the releases
index rather than `/releases/latest`.

This split is deliberate. `/releases/latest` 404s until a full non-draft release
exists, and nothing in the build, the test suite, or the deploy notices a link
that is only dead in production — which is exactly how the old CTA survived.

`resolveRelease()` activates the download only when **all** of these hold:

| Requirement | Why it gates the download |
|---|---|
| `publication` is `published` | Drafts and prereleases are invisible to GitHub's `/releases/latest`, so a downloader would get an app that can never update itself. |
| `updaterChannel` is `latest` | Must match the channel baked into the shipped app. |
| `betaEndsAt` is a real date, still in the future | A time-boxed beta with no stated end is a promise that cannot be kept. An expired window closes the download again on its own. |
| `verifiedAt` is set and is **not** earlier than `publishedAt` | Verification that predates publication verified a different build. |
| A `.exe` and a universal `.dmg`, both hosted under this repository's releases | An artifact URL pointing at any other host is a supply-chain hole, not a config option. |
| Each artifact has a positive byte size and a 64-character SHA-256 | The size is shown to the visitor before they commit to the download; the digest is what `SHA256SUMS.txt` is checked against. |

Anything missing, malformed, or stale resolves to unavailable. There is no
partial state and no override.

**Owner procedure once a release is published and verified:**

1. Publish the release (owner action — see the $0 release path above).
2. Record the two artifact URLs, byte sizes, and SHA-256 digests from
   `SHA256SUMS.txt`.
3. Set the beta end date. It is an owner decision and is tracked in
   `docs/OWNER_INPUT_QUEUE.md`; until it is answered the download stays closed
   even with a perfect release.
4. Populate `RELEASE_METADATA`, run `npm test` and `npm run verify:headless`,
   and confirm the landing lane now reports `verified-public`.
5. Deploy (U17).

Do not populate `RELEASE_METADATA` to "make the button work". Editing that
constant is what publishes a download to the public.

## Intentionally out of scope

- Linux launch artifacts.
- Mac App Store / Microsoft Store distribution.
- Making optional email capture a download gate.
