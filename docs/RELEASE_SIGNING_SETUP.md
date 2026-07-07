# Release & Code-Signing Setup

How to turn the `.github/workflows/release.yml` pipeline into real, signed,
downloadable installers. You only do this setup once. Until you do, the pipeline
still works — it just produces **unsigned** builds that trip an "unknown
developer" warning on download.

## How a release works (the 30-second version)

1. Bump the version in three files (they must match): `package.json`,
   `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`.
2. Commit, then create and push a tag: `git tag v0.1.0-beta.1 && git push origin v0.1.0-beta.1`.
3. The workflow builds macOS (Apple Silicon + Intel) and Windows installers and
   creates a **draft** GitHub Release with them attached.
4. You open the draft release on GitHub, sanity-check the files, and click
   **Publish**. (It's a draft on purpose — nothing goes public until you say so.)
5. The landing page's download button points at that release (wired up in the
   release-pipeline follow-up).

You can also run it manually from the Actions tab ("Run workflow") without a tag.

---

## macOS signing (stable — do this first)

Goal: a **Developer ID Application** certificate (the one for apps distributed
*outside* the Mac App Store) + notarization.

1. **Join the Apple Developer Program** — $99/yr, https://developer.apple.com/programs/.
   Allow a day or two for identity verification.
2. In the Apple Developer portal, create a **Developer ID Application**
   certificate. Download it and double-click to install it in your Mac's
   Keychain. (Not "Apple Development" — that one can't notarize for distribution.)
3. In Keychain Access, export that certificate **and its private key** as a
   `.p12` file. Give it a password — this is your `APPLE_CERTIFICATE_PASSWORD`.
4. Base64-encode the `.p12` so it can live in a secret:
   - macOS: `base64 -i certificate.p12 | pbcopy`
5. Create an **app-specific password** for notarization at
   https://appleid.apple.com (Sign-In & Security → App-Specific Passwords).
   This is `APPLE_PASSWORD`.
6. Find your **Team ID** on your Apple Developer account membership page
   (`APPLE_TEAM_ID`).

Add these in GitHub → repo **Settings → Secrets and variables → Actions → New repository secret**:

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | the base64 string from step 4 |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12` password from step 3 |
| `APPLE_ID` | your Apple account email |
| `APPLE_PASSWORD` | the app-specific password from step 5 |
| `APPLE_TEAM_ID` | your Team ID from step 6 |
| `KEYCHAIN_PASSWORD` | any random string (a throwaway password for the CI keychain) |

Once these exist, the macOS jobs sign and notarize automatically. No code changes needed.

---

## Windows signing (Azure Trusted Signing)

Azure Trusted Signing is the cheap modern path (~$10/mo, and as of 2026 open to
individuals — no company required). Unlike old certificates, there's no big
up-front cert purchase.

1. In the Azure portal, set up **Trusted Signing** (a.k.a. Azure Artifact
   Signing): create a Trusted Signing **account** and a **certificate profile**,
   and complete the identity verification.
2. Create an **App Registration** (Microsoft Entra ID) and a **client secret**.
   Grant it the "Trusted Signing Certificate Profile Signer" role on your
   account. You'll get a tenant ID, client ID, and client secret.
3. Add these GitHub secrets:

   | Secret | Value |
   |---|---|
   | `AZURE_TENANT_ID` | your Entra tenant ID |
   | `AZURE_CLIENT_ID` | the App Registration's client ID |
   | `AZURE_CLIENT_SECRET` | the App Registration's client secret |

4. Edit `src-tauri/tauri.windows-signing.conf.json` and replace `YOUR_AZURE_ACCOUNT`,
   `YOUR_CERT_PROFILE`, and the endpoint URL (e.g. `https://wus2.codesigning.azure.net`)
   with your real values.

> ⚠️ **Verify the signing CLI before the first signed release.** The crate/binary
> used in the workflow (`trusted-signing-cli`) and in the `signCommand` is in flux
> upstream — it has been renamed (e.g. `artifact-signing-cli`) and the tauri-action
> integration has broken and been fixed more than once. Before relying on it,
> check the current official guide — https://v2.tauri.app/distribute/sign/windows/ —
> and confirm the install line in `release.yml` and the `signCommand` match. This
> is the one piece we agreed to confirm together once your Azure account exists.

**Heads-up on SmartScreen:** even correctly signed, brand-new Windows software
shows a "Windows protected your PC" warning until your download earns
"reputation" (a few weeks / some number of installs). This is why we sign from
the very first beta build — to start that clock as early as possible.

---

## Auto-updater signing

The Tauri auto-updater is wired in (Slice 7): on launch the app checks these
same GitHub Releases for a newer signed version (`src-tauri/tauri.conf.json` →
`plugins.updater`). It needs its **own** signing keypair — separate from the OS
code-signing certs above — so the app can verify an update genuinely came from
you before trusting it.

> ⚠️ **The `pubkey` currently committed in `plugins.updater` is a throwaway
> bootstrap; its private half was discarded. You MUST generate your own keypair
> before the first release, or update artifacts can never be signed to match
> the app's expected key.**

1. Generate the keypair once (it must stay stable forever — every shipped app
   trusts this exact key):

   ```
   npm run tauri -- signer generate -w yes-master-updater.key
   ```

   It prints the **public key** and writes the password-protected **private
   key** to `yes-master-updater.key`.
2. Paste the printed public key into `src-tauri/tauri.conf.json` →
   `plugins.updater.pubkey`, replacing the bootstrap value, and commit it.
3. Add two GitHub secrets (Settings → Secrets and variables → Actions):

   | Secret | Value |
   |---|---|
   | `TAURI_SIGNING_PRIVATE_KEY` | the full contents of `yes-master-updater.key` |
   | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | the password you set in step 1 |

4. Keep `yes-master-updater.key` in a password manager. **If you lose it,
   already-installed apps can no longer auto-update** — you'd have to ship a new
   pubkey and every user would re-download once by hand. Never commit it.

Once the private-key secret exists, the release pipeline automatically emits and
signs the updater artifacts (per-installer `.sig` files + `latest.json`) via the
`src-tauri/tauri.updater.conf.json` overlay and attaches them to the draft
release; the app reads `latest.json` from the release's
`latest/download/latest.json` URL. Offline or before any release exists, the
check fails silently and the app is fully usable.

## What's intentionally not here yet

- **Linux.** Out of scope for launch (desktop = Mac + Windows).
