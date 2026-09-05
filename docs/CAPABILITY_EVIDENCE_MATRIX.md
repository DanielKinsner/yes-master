# YES Master — Capability & Evidence Matrix

**One place where every public claim is bound to a named evidence source.**
Created by U1 of `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`
(KTD5, R5). Copy review alone cannot stop a current-looking but unproved claim
from shipping; this table can.

## How to use it

- **Every visible public claim** — landing copy, platform statement, pricing
  statement, beta promise, screenshot — needs a row here before it ships.
- A claim's **Status** is one of:
  - **Proved** — a named, currently-passing evidence source backs it.
  - **Qualify** — true but overstated as written; ship a narrower sentence.
  - **Remove** — not backed and not fixable by qualifying; delete it.
  - **Owner** — blocked on an owner answer; the conservative default holds and
    the question is a row in `docs/OWNER_INPUT_QUEUE.md`.
- **Evidence source** names a test, doc, or artifact — never "obviously true".
  Evidence layers (browser headless / frontend unit / native Rust synthetic /
  private fixture / installed-machine / owner listening / production smoke) are
  defined in the plan's Verification Contract and are **not interchangeable**.
- **U6 owns the copy edits**; this file records the verdict, not the wording.
- A claim with no row is a defect. Adding a claim without adding its row is the
  failure mode this file exists to catch.

## Status at U1

This began as an audit of the landing surface, not a record of completed work.
U5 executed the release/download rows (C-10, C-11) and U6 executed the copy
rows (C-05, C-07 label, C-15, C-16, C-17, C-22) and added C-27 through C-34 for
the sections the rewrite introduced. Rows still marked **Qualify**, **Owner**,
or **Missing** have not been executed; U7 owns the remaining asset provenance
work. A row's status is the current verdict, and the date in an "Applied" cell
says when it stopped being a to-do.

---

## A — Product capability claims

| ID | Claim (as currently published) | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| C-01 | "One-click mastering. Your Endgame Sound." (2026-09-01; previously "Master your track in real time. Hear every move. Ship with proof.") | `Hero.tsx` h1 | Proved | Real-time chain is the shipped architecture (`docs/ARCHITECTURE.md`); receipt surface pinned by `ExportReceiptCard` tests. Perceived responsiveness on real hardware is an **owner** item (go/no-go §4). | U6 |
| C-02 | "The full chain runs as you listen — no upload, no reprocessing wait." | `Hero.tsx` proof point | Proved | Local-first engine; no network path in the audition chain. `docs/APP_BEHAVIOR.md`. | U6 |
| C-03 | "Simple by default, deep when you want it" | `Hero.tsx` proof point | Proved | Standard/Advanced split is shipped; `StandardView.tsx` / `AdvancedPanel.tsx` and their tests. | U6 |
| C-04 | "A pass/fail receipt shows your LUFS, true-peak, and dynamic range" | `Hero.tsx` proof point | Proved | `export_checks_for_report` + `ExportReceiptCard` tests; native synthetic E2E covers the receipt (U14). | U6 |
| C-05 | ~~"You get a clean 44.1 kHz / 24-bit WAV, true-peak safe, every time."~~ → narrowed to the fixed delivery format + limiter ceiling (see C-28) | `ProofDeck.tsx` Standard card | **Applied (U6, 2026-07-25)** | The absolute "true-peak safe, every time" guarantee was replaced by the C-28 wording ("a 44.1 kHz / 24-bit WAV with the limiter's ceiling at −1 dBTP"), which is Proved against `docs/PRODUCT.md`. Row reconciled 2026-08-31 (audit D-01) — the copy shipped in U6 but this row was never flipped. | U6 |
| C-06 | "Eight styles, a 7-band EQ, compressor modes, width and warmth, live metering, and export review with a measured receipt." (was "Eight presets"; reworded 2026-09-01 under the D9 vocabulary rule — a Style is a factory character, a Preset is a user-saved recipe) | `ProofDeck.tsx` Advanced card | Proved | All eight styles exist (`preset_slug`, `preset_fingerprint.rs`); 7-band EQ fields in `MasteringSettings`; `AdvancedPanel.tsx`. | U6 |
| C-07 | ~~Hand-authored receipt numbers with no illustrative marker~~ → landing surfaces are deterministic captures gated by the asset manifest | `ProofDeck.tsx` / `src/assets/landing/manifest.json` | **Applied (U7)** | U7 shipped the deterministic capture pipeline (`npm run capture:landing`) with a manifest hash gate (`verify-landing-assets`) that fails the suite whenever a captured surface drifts from the app source — the exact "source them from a deterministic capture" resolution this row asked for. Row reconciled 2026-08-31 (audit D-01). | U7, U6 |
| C-08 | "Every master ships with a receipt … No guesswork." | `ProofDeck.tsx` Technically-checked card | Proved | As C-04. | U6 |

## B — Platform and availability claims

| ID | Claim (as currently published) | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| C-10 | ~~"Download the free beta" → `https://github.com/DanielKinsner/yes-master/releases/latest`~~ → **the download action is now rendered from resolved release state** | `BetaDownload.tsx` / `release-config.ts` | **Removed (U5, 2026-07-25)** | The unconditional link is gone. `resolveRelease()` is pure and total, defaults to unavailable for absent/malformed/draft/prerelease/unverified/stale metadata, and `RELEASE_METADATA` ships `null`. Nothing on the page links to `/releases/latest`; unavailable states route to the releases *index*, which resolves whether or not a release exists. Pinned by `release-readiness.test.ts` (state model, incl. a negative control that the valid fixture *does* activate) and by the headless landing lane at all 12 viewports. | U5 (activation: U16, U17) |
| C-11 | "Windows + universal Mac" | `BetaDownload.tsx` | Proved (as intent) → **binding implemented, artifacts pending** | `release.yml` builds one universal macOS artifact + Windows MSI/NSIS; `release-readiness.test.ts` pins that contract. U5 added the binding: a release cannot activate without a repo-hosted `.exe` and universal `.dmg`, each with a positive byte size and a 64-char SHA-256. Binding to *real* artifacts still needs a published release (U16/U17). | U5, U16 |
| C-12 | "No email required" | `BetaDownload.tsx` | Proved | D5 / D16: download is ungated. `BetaSignup.test.tsx` pins the form as optional and safe-disabled. | U4 |
| C-13 | "Hosted on GitHub Releases" | `BetaDownload.tsx` | Proved | `release.yml`; KTD3. | U5 |
| C-14 | "Early beta installers are not yet backed by paid publisher certificates, so Windows or macOS may ask you to confirm that you trust the download." | `BetaDownload.tsx` | Proved | D16; `docs/RELEASE_SIGNING_SETUP.md`. This is a correctly-qualified claim and is the model for the rest. | U5 |
| C-15 | ~~"Coming to your pocket" / "Same engine, headed to iPhone & Android." / "coming after launch"~~ → **"iPhone and Android are not currently available."** | `CrossPlatform.tsx` | **Applied (U6, 2026-07-25)** | Mobile is **parked** (D9, D15, KTD9) and has no owner-approved release date. "Coming after launch" is a roadmap promise the product has not committed to. Restate as not currently available, with no implied schedule. See the canon conflict note below. | U6 |
| C-16 | ~~Six present-tense mobile feature claims — "A/B in sync", "Four styles", "Intensity control", "Real-time meters", "Quality checks", "No upload, no wait"~~ → **removed entirely** | `CrossPlatform.tsx` | **Applied (U6, 2026-07-25)** | Each is *mechanically* true of the built bridges (`docs/PRODUCT.md` Mobile Companions; iPhone `ContentView.swift` four-preset picker). But they read as a feature list for a product a visitor can obtain, and they cannot. Present them as what the shared engine already does on the bridges, not as an available product. | U6 |
| C-17 | ~~iPhone screenshot (`iphone-standard-ui.jpg`) presented adjacent to desktop beta proof~~ → **removed from the page** | (no longer published) | **Applied (U6, 2026-07-25)** | R7 forbids a mobile UI image standing as current desktop-beta proof, and this asset has no capture-commit binding. Needs the U7 manifest treatment or an explicit non-current label. | U7 |
| C-18 | "Mac & Windows." | `FinalCTA.tsx` footnote | Proved | D1: beta ships Mac + Windows together. | U6 |

## C — Pricing and beta-promise claims

| ID | Claim (as currently published) | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| C-20 | "Free during the beta." | `FinalCTA.tsx` | Proved | D16 / `docs/PRODUCT.md` Distribution & Business Model. | U6 |
| C-21 | "When it launches it's a one-time $29 (then $49)" | `FinalCTA.tsx` | Proved *as a business model* | `docs/PRODUCT.md` Distribution & Business Model; launch plan §pricing; D2. The **numbers** are settled canon. | U6 |
| C-22 | "beta testers keep $29 forever" | `FinalCTA.tsx` | **Owner** | This is an **entitlement promise**, not a price. R24 makes the founder window a public time-limited launch window; the exact terms, duration, and what a beta tester is entitled to are not decided. Conservative default: no entitlement wording until answered. → `docs/OWNER_INPUT_QUEUE.md` row 1. | U6 |
| C-23 | Beta end date / "concrete flip date" (D2 requires one; none is published) | not yet on the page | **Owner** | Undecided. Conservative default: the beta is described as time-boxed with no date, and a missing beta end date keeps the download **unavailable** in the U5 release-state model. → owner queue row 3. | U5, U6 |
| C-24 | "Optional email updates" signup | `FinalCTA.tsx` / `BetaSignup.tsx` | **Owner** | No provider selected; `SIGNUP_ENDPOINT = ""` and the form is safe-disabled (pinned by `BetaSignup.test.tsx`). Conservative default holds. → owner queue row 2. | U4 |
| C-25 | Beta-end behavior — installers withdrawn, installed builds keep working, no kill switch (R26/KTD13) | `docs/BETA_TESTING.md` ("There is no kill switch") + `BetaTerms.tsx` (C-33) | **Applied (U4)** | The settled policy is stated in the beta guide and on the landing terms surface; C-33 records the landing-page instance as Proved. Row reconciled 2026-08-31 (audit D-01). | U4 |
| C-26 | GitHub account required to file structured feedback (R19/KTD4) | `docs/BETA_TESTING.md` ("GitHub issues are public, and posting requires a free GitHub account") | **Applied (U4)** | Disclosed in the beta guide before the handoff to GitHub, as this row required. Row reconciled 2026-08-31 (audit D-01). | U4 |
| C-27 | "It reads what your mix already has and eases its own moves to fit" / "adds impact where the material can take it" | `Hero.tsx`, `SoundCharacter.tsx` | Proved | `docs/PRODUCT.md` "Adaptive Mastering": loudness landing uses current track/render measurements, and a resolved source profile drives **reduce-only** Tier-1 guardrails weighted by per-axis confidence. Shipped and owner-listened 2026-06-11. Deliberately framed as restraint, never as rescue — already-processed input is a normal case, not a failure. | U6 |
| C-28 | "a 44.1 kHz / 24-bit WAV with the limiter's ceiling at −1 dBTP" (Workflow + Standard card) | `Workflow.tsx`, `ProofDeck.tsx` | Proved | `docs/PRODUCT.md` line 55 / "Standard exports a fixed, known-safe default: 44.1 kHz / 24-bit WAV at −1 dBTP". Replaces C-05's absolute "true-peak safe, every time". | U6 |
| C-29 | "Styles are starting points, not lanes" — every style runs the same loudness landing and safety stages; Intensity spans subtle to pushed | `SoundCharacter.tsx` | Proved | `docs/PRODUCT.md` "Loudness And Safety" (creative controls bold, export/quality stages measure and surface); preset set pinned by `preset_fingerprint.rs`; Intensity is a shipped control with its own a11y tests (U9). | U6 |
| C-30 | Album: one loudness across the record, one delivery format, a receipt per track, per-track override, nothing silently altered | `AlbumProof.tsx` | Proved | `docs/PRODUCT.md` "Album Master" — owner-defined promise (2026-07-03), plus album-wide format with mixed-rate resampling, mixed mono/stereo resolution, per-track override, per-track receipts. Album UI acceptance closed in U10. | U6 |
| C-31 | "It does not produce DDP images, cue sheets, or ISRC metadata, and it does not guarantee gapless playback." | `AlbumProof.tsx` | Proved | `docs/PRODUCT.md` "Album Master": these are **explicitly out of scope for v1**. Stated on the page because a reader who sees "album" will otherwise assume them. | U6 |
| C-32 | "Telling us things is optional … Nothing is held back from anyone who never does." | `BetaTerms.tsx` | Proved | R19; `docs/BETA_TESTING.md` states the same promise (U4). The two surfaces must not disagree. | U6 |
| C-33 | "a build you already installed keeps working. There is no kill switch and nothing expires on your machine." | `BetaTerms.tsx` | Proved | R26 / KTD13 — settled beta-end policy. Also stated in `docs/BETA_TESTING.md` (U4); this is C-25's landing-page instance. | U6 |
| C-34 | "At launch YES Master is a one-time purchase: a $29 founder price, $49 standard." + "not a subscription" | `BetaTerms.tsx` | Proved *as a business model* | `docs/PRODUCT.md` "Distribution & Business Model": a single perpetual license, **$29 founder → $49 standard, one-time purchase (no subscription)**. The **window's** dates/duration/entitlement remain owner-blocked (C-22, owner queue row 1) and are explicitly not claimed. | U6 |

---

## Canon conflicts found by this audit

Recorded here because they are the reason the matrix is needed, and because each
one is a documentation reconciliation, **not** a new product decision.

1. **`docs/landing-brief.md` forbids exactly what the landing page does.** The
   brief's hard rules say *"No roadmap, no 'coming soon'"* and *"Present tense
   only"*. The shipped `CrossPlatform.tsx` section is titled "Coming to your
   pocket" and says "coming after launch". One of the two has to move. Under D9
   (phones go live when the owner judges them ready) and KTD9/R4 (mobile is
   parked and must not read as currently available), the resolution is: the
   brief permits **one restrained, date-free statement that mobile is not
   currently available**, and continues to forbid schedules, "coming soon", and
   roadmap language. Applied in U1.

2. **The brief's feature pillar claims mobile in the present tense.**
   *"Cross-platform. Windows and macOS desktop, with iPhone and Android
   companions on the same engine."* reads as three shipping products. The
   engine-sharing fact is true; the availability implication is not. Amended in
   U1.

3. **`AGENTS.md` / `CLAUDE.md` still call landing-page scope an open owner
   decision** in the Verification section, while their own Non-Negotiables list
   (and D16, and `docs/PRODUCT.md` Public Surface) already state it is in scope.
   Stale parenthetical; corrected in U1.

4. **Two beta documents, no stated precedence.** Fixed in U1 — see the
   authority note in `AGENTS.md`/`CLAUDE.md` Required Reading.

## Studio handoff application — 2026-09-04

This marketing-only revision applies the supplied `PAGE_COPY.md`, keeping the
release closed. The following rows supersede the historical wording/surface
locations above for the redesigned page; they do not add native capabilities.

| ID | Current wording or visual | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| S-01 | Standard: four styles, Intensity, Low/Medium/High; separate WAV; 44.1 kHz / 24-bit and −1 dBTP limiter setting | Workflow | Proved | C-28; PRODUCT.md Primary Workflow and Standard export ceremony; StandardView tests | Studio redesign |
| S-02 | Advanced: seven-band EQ, eight styles, width/warmth, Preset/Manual/Off creative compression, custom formats | Hero teaser / ProofDeck | Proved | C-06; PRODUCT.md Compressor Canon; AdvancedPanel and VisualEqPanel tests. No active per-band adaptive claim. | Studio redesign |
| S-03 | Same-playhead Original/Mastered; optional Volume Match, off by default, audition-only | ProofDeck | Proved | APP_BEHAVIOR.md Track Master; audio controller tests; C-02. No native timing benchmark claimed. | Studio redesign |
| S-04 | Rendered-file measurements; Standard filename/loudness/Show file versus detailed Advanced Track Master review/receipt | ProofDeck | Proved | APP_BEHAVIOR.md Standard Create Master / Export Checks; ExportReceiptCard tests; C-04/C-28. Replaces the old claim that both modes give the same receipt. | Studio redesign |
| S-05 | Report fields, illustrated: LUFS, dBTP, LU, quality checks; no numeric results or "all good" status | ProofDeck | Proved as illustration | Explicit figcaption; LandingCopy test rejects invented readings/statuses. This replaces C-07/C-08's old example receipt; no actual render evidence is claimed. | Studio redesign |
| S-06 | Source analysis can ease selected boosts and compression density; already-processed tracks welcome | SoundCharacter | Proved | C-27; PRODUCT.md Adaptive Mastering, reduce-only Tier-1 restraint. Gated per-band/Phase-B logic is not marketed. | Studio redesign |
| S-07 | Four existing character styles share loudness/safety stages; Intensity controls strength | SoundCharacter | Proved | C-29; existing preset artwork; preset_fingerprint.rs; supplied owner Intensity capture. Subjective character descriptions, no outcome guarantees. | Studio redesign |
| S-08 | Album Follow/Override, own settings/target under common delivery format; individual/continuous WAVs, manifest, per-track receipts; no DDP/cue/ISRC/gapless guarantee | AlbumProof | Proved | C-30/C-31; PRODUCT.md Album Master; album export integration tests. Diagram explicitly labelled; actual Album capture expandable. | Studio redesign |
| S-09 | Save/reopen projects referencing source audio on disk | AlbumProof | Proved | APP_BEHAVIOR.md Project, Settings and Help; project persistence tests. No portable embedded-audio archive claim. | Studio redesign |
| S-10 | Free beta, no email gate/account; installed build keeps working; feedback optional, GitHub account to post | BetaTerms / FinalCTA | Proved | C-12/C-20/C-25/C-26/C-32/C-33; unchanged BetaDownload/BetaSignup and release tests | Studio redesign |
| S-11 | Planned $29 founder / $49 standard purchase; founder dates/terms when confirmed | BetaTerms | Qualified business model | C-21/C-22/C-34; OWNER_INPUT_QUEUE. No dates, eligibility or entitlement promised. | Studio redesign |
| S-12 | Studio/chassis artwork, with real owner Standard/Advanced displays, separate from HTML copy | Hero | Proved as artwork provenance | Supplied HERO_ASSETS.md; `manifest.json` studioArtwork hashes; standalone asset gate. Owner captures are snapshots, not proof of current native behavior. | Studio redesign |
| S-13 | Watch demo shown as unavailable; Explore the beta routes to current closed state | Hero / FinalCTA | Proved | No video/audio URL or handler; aria-disabled and visible explanation; headless studio interaction checks. Release resolver and signup config unchanged. | Studio redesign |
| S-14 | Front-facing blank chassis, tape-machine detail and studio listening scene | Hero / SoundCharacter / FinalCTA | Decorative artwork only | Owner-requested refinement; built-in generation prompts in `docs/landing-mockups/studio-refinement-artwork.md`; manifest hashes. The Advanced screen is the unchanged owner capture in a separate HTML image. The generated musician is not a customer or testimonial; tape imagery promises no specific hardware emulation. | Studio refinement |
