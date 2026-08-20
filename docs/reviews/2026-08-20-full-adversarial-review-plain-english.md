# YES Master Full Adversarial Review — Plain-English Version

**Date:** August 20, 2026  
**Reviewed version:** `main` at commit `7d2e0cb`  
**Review type:** Read-only, full-repository review

## Bottom line

YES Master is in better mechanical shape than the release checklist makes it
look. The audio engine, exports, desktop packaging, mobile bridges, browser
tests, and current GitHub build all passed broad testing.

However, I would **not create the next beta tag yet**. Three problems should be
fixed first:

1. The dependency files contain newly reported security problems that the
   current GitHub build does not check for.
2. The new keyboard shortcuts interfere with normal keyboard use and create an
   accessibility problem.
3. The release workflow gives signing secrets and write access to action code
   that is referenced by changeable version tags.

These are fixable engineering problems. They do not mean the audio engine is
bad or that the project needs a rewrite.

## What is working well

The strongest part of the project is still its tested audio and export
contract.

- The audio engine's formatting, linting, unit tests, integration tests, and
  private-audio fixture tests passed.
- Export protections passed, including source-file protection, collision-safe
  filenames, Volume Match staying out of exports, loudness landing, true-peak
  control, receipts, and album rendering.
- The iPhone bridge passed its checks and tests.
- The Android bridge passed its host tests and its ARM64 build check.
- The current Windows build produced both MSI and NSIS installers.
- The landing page passed 13 browser sizes, automated accessibility checks,
  zoom checks, reduced-motion checks, and link checks.
- The app preview passed 24 browser scenarios.
- The movable seven-band EQ passed all 46 drag, reset, ordering, preset, undo,
  and mode-switch checks.
- GitHub Actions is green at the exact commit reviewed, including Windows,
  macOS, Android, iPhone Swift tests, web tests, and cross-platform audio
  snapshots.
- The owner-gated audio features remain safely off. Nothing in this review
  changed preset sound, adaptive-compressor settings, Phase B, or album
  character.

This is strong evidence that the core product is mechanically stable. It is
not a substitute for installing it on real machines or listening to it.

## Problems to fix before the next beta candidate

### 1. The security checks are missing newly reported dependency problems

The normal GitHub build is green, but two separate security checks fail when
run directly.

The JavaScript dependency check reports two high-severity problems:

- `brace-expansion` is at 5.0.8 and should move to at least 5.0.9.
- `nanoid` is at 3.3.16 and should move to at least 3.3.18.

Both are currently in development or build tooling, not in the desktop app's
normal runtime path. That lowers the immediate danger, but the repository's
own beta rules say unexplained high-severity warnings must be cleared.

The Rust dependency check reports two high-severity denial-of-service problems
in `quick-xml` 0.39.4. It arrives indirectly through Tauri's `plist` support.
The fixed line starts at `quick-xml` 0.41.0, which is used by `plist` 1.10.0.
I did not find a YES Master feature that feeds untrusted XML into this parser,
so this also looks like a low-exposure dependency problem rather than a known
way to attack the app. It still needs to be updated and re-tested.

The larger process problem is that GitHub Actions does not run either security
check. That allows the project to look fully green after a new advisory is
published. Dependabot security alerts, secret scanning, and code scanning are
also disabled for the public repository, and there is no registered security
reporting policy.

**Recommended action:** update the lockfiles, run the full test suite again,
then add focused JavaScript and Rust advisory checks to GitHub Actions. Configure
the Rust check so known unmaintained Linux-only dependencies are documented
instead of burying real vulnerabilities in noise. Turn on the free repository
security features and add a private security-reporting path before inviting a
public beta audience.

Evidence:

- [`package-lock.json`](../../package-lock.json)
- [`src-tauri/Cargo.lock`](../../src-tauri/Cargo.lock)
- [GitHub's `brace-expansion` advisory](https://github.com/advisories/GHSA-rgw5-rvv9-x895)
- [GitHub's `nanoid` advisory](https://github.com/advisories/GHSA-2v37-7h3g-55p8)
- [RustSec `quick-xml` CPU advisory](https://rustsec.org/advisories/RUSTSEC-2026-0194.html)
- [RustSec `quick-xml` memory advisory](https://rustsec.org/advisories/RUSTSEC-2026-0195.html)

### 2. The global keyboard shortcuts interfere with normal controls

The app now treats Space as Play/Pause almost everywhere. The handler stops the
browser's normal Space action when focus is on buttons, checkboxes, sliders, or
drop-down menus.

I reproduced this in a real browser: after focusing the Import button, Space no
longer activated the button, while Enter still did. The same handler is more
serious for a real checkbox such as **Link stereo**, because Space is the normal
way to toggle a checkbox and Enter does not replace it.

The single-key `A`, `L`, and `?` shortcuts create a second accessibility issue.
They are active across the app, but the user cannot turn them off, remap them,
or limit them to a focused control. That conflicts with the accessibility rule
for single-character shortcuts and can cause accidental actions for people who
use speech input.

**Recommended action:** let normal interactive controls keep their standard
Space behavior. Add a preference to disable single-key shortcuts, require a
modifier key, or scope them to a dedicated focus area. Add real-browser tests
that operate the app with Tab, Space, and Enter. The existing automated
accessibility scan cannot detect this behavior by itself.

Evidence:

- [`src/hooks/useTrackMaster.ts`](../../src/hooks/useTrackMaster.ts)
- [`src/components/AdvancedPanel.tsx`](../../src/components/AdvancedPanel.tsx)
- [`src/App.tsx`](../../src/App.tsx)
- [W3C guidance for single-character shortcuts](https://www.w3.org/WAI/WCAG21/Understanding/character-key-shortcuts.html)

### 3. The release workflow trusts changeable action versions with signing secrets

The release workflow gives repository write access to the whole workflow. Its
main packaging action receives the updater private key and password. It may
also receive Windows signing credentials later.

That action is referenced as `tauri-apps/tauri-action@v0`. The other actions are
also referenced by version tags such as `@v4` or `@stable`. A version tag can be
moved. A full commit identifier cannot.

This does not prove that any current action is malicious. It means a future
compromise or moved tag would have more power than necessary: access to signing
material, release artifacts, and a repository write token.

The optional Windows signing tool is also installed without locking it to an
exact tested version and dependency set.

**Recommended action:** pin every release action to a reviewed full commit
identifier. Give read access by default and grant write access only to the jobs
that create or audit the draft release. Protect release secrets with a GitHub
environment that requires approval. Pass Windows credentials only to the
Windows signing step. Pin the signing tool to an exact version and install it
with its locked dependencies.

Evidence:

- [`.github/workflows/release.yml`](../../.github/workflows/release.yml)
- [GitHub's secure-use guidance](https://docs.github.com/en/actions/reference/security/secure-use)

## Important problems that should follow immediately

### The `L` shortcut can turn on a loop that does not exist yet

The loop button is correctly disabled until the user draws a region on the
waveform. Pressing `L` bypasses that rule. It changes the internal state to
"loop on" even though no loop region exists. If the user later draws a region,
looping starts without a clear new action.

The current automated test expects this incorrect behavior, so the test protects
the bug instead of preventing it.

**Recommended action:** ignore `L` until a region exists, keep the interface and
audio engine in the same state, and undo the visible state change if the audio
engine rejects the request.

### The release documents show two different versions of reality

The current release ledger correctly says that the old 0.9.1 candidate was
rejected and that 0.9.2 must repeat the candidate gates. The top of the same
checklist still shows 0.9.1 and old build evidence as completed.

The public README also calls an executed hardening plan "in progress," names
0.9.0 as the upcoming beta, and points readers to the historical beta plan
instead of the active quality plan.

This is not a product failure, but it creates a real release risk: somebody can
read the green boxes at the top and mistake old proof for current proof.

**Recommended action:** reset or clearly supersede every candidate-specific box,
then record the new 0.9.2 evidence only after it exists. Update the public README
to describe the current project and point to the active plan.

Evidence:

- [`docs/plans/beta-go-no-go.md`](../plans/beta-go-no-go.md)
- [`README.md`](../../README.md)

### The public website is not ready to become the official launch address

The public Vercel address, `yes-master.vercel.app`, loads and passes the remote
landing-page smoke test. However, the JavaScript and CSS it serves do not match
the landing-page build made from the reviewed commit.

GitHub records a successful Vercel deployment for the reviewed commit, but that
deployment address is protected by Vercel login. The public alias is serving a
different landing artifact.

The official `yesmaster.app` name currently has no website record and no email
record. It does not resolve, even though the page's canonical address, social
image URLs, legal drafts, and contact address use that domain.

This is expected to remain closed until the owner's final launch step. It must
be completed and verified before public announcement.

The Vercel command-line tool is installed on this machine, but its saved login
is invalid. The next step is `vercel login`. On another machine without the
tool, install it with `npm i -g vercel` first.

## Work that should stay parked

The largest files are still large:

- `src/App.css` is about 8,400 lines.
- `src/hooks/useTrackMaster.ts` is about 3,000 lines.
- `src/App.tsx` is about 2,250 lines.
- The main Rust audio and DSP files are both over 5,000 lines.

Size alone is not a reason to rewrite working code. The test suite around these
areas is valuable, and broad cleanup would create avoidable audio and release
risk.

The shortcut defects do reveal one useful narrow boundary: global keyboard
behavior should have a single owner and a focused browser test suite. That seam
can be extracted after the behavior is corrected. Large DSP refactors should
remain parked unless a specific bug or measurable maintenance problem justifies
them.

## What still requires the owner

Even after the engineering problems are fixed and a new candidate is tagged,
the beta is not ready to publish until the owner completes the real-world gates:

- Delete the four stale 0.9.1 draft releases.
- Install the exact new candidate on the current Windows machine and M4 Mac.
- Confirm that real-time controls feel responsive on both machines.
- Complete the single listening session on the current 0.9.2 sound.
- Prove the updater from the seed build to the new published candidate.
- Set the beta end date.
- Decide the public announcement date and explicitly authorize publication.
- Configure and verify the official domain before using it in public metadata.
- Keep the newsletter form disabled until a provider and consent rules are
  chosen.

Agent tests cannot replace installation, listening, taste, publication, or
signing decisions.

## Recommended order of work

1. Update the JavaScript and Rust dependency locks and add security gates to CI.
2. Correct the keyboard shortcuts and invalid loop state; add real-browser tests.
3. Harden the release workflow and protect its secrets.
4. Re-run every normal, native, browser, packaging, and security gate.
5. Reconcile the live release checklist and public README with 0.9.2.
6. Confirm the intended Vercel deployment and configure the official domain.
7. Tag `v0.9.2-beta.1`; the candidate freeze starts again at that tag.
8. Perform the owner installation, listening, updater, and publication sequence.

## Review boundary

This review did not change code, audio settings, documentation, releases,
deployment settings, or tags. The repository was clean at the end of the review.
