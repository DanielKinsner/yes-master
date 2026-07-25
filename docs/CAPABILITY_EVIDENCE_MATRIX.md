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

This is an audit of the landing surface **as it stands today**, not a record of
completed work. Nothing in the Qualify/Remove/Owner rows has been changed yet —
U5, U6 and U7 execute them. U1's job was to make the list exist and be honest.

---

## A — Product capability claims

| ID | Claim (as currently published) | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| C-01 | "Master your track in real time — and see exactly what it did." | `Hero.tsx` h1 | Proved | Real-time chain is the shipped architecture (`docs/ARCHITECTURE.md`); receipt surface pinned by `ExportReceiptCard` tests. Perceived responsiveness on real hardware is an **owner** item (go/no-go §4). | U6 |
| C-02 | "The full chain runs as you listen — no upload, no reprocessing wait." | `Hero.tsx` proof point | Proved | Local-first engine; no network path in the audition chain. `docs/APP_BEHAVIOR.md`. | U6 |
| C-03 | "Simple by default, deep when you want it" | `Hero.tsx` proof point | Proved | Standard/Advanced split is shipped; `StandardView.tsx` / `AdvancedPanel.tsx` and their tests. | U6 |
| C-04 | "A pass/fail receipt shows your LUFS, true-peak, and dynamic range" | `Hero.tsx` proof point | Proved | `export_checks_for_report` + `ExportReceiptCard` tests; native synthetic E2E covers the receipt (U14). | U6 |
| C-05 | "You get a clean 44.1 kHz / 24-bit WAV, true-peak safe, every time." | `ProofDeck.tsx` Standard card | **Qualify** | Standard's fixed export recipe is real (`docs/PRODUCT.md` Mobile Companions / Standard export). But **"true-peak safe, every time"** is an absolute guarantee; the product's own policy is that a user may overcook a track and receive an advisory warning. Narrow to the delivery format plus the true-peak ceiling the limiter targets. | U6 |
| C-06 | "Eight presets, a 7-band EQ, compressor modes, width and warmth, live metering, and export review with a measured receipt." | `ProofDeck.tsx` Advanced card | Proved | All eight presets exist (`preset_slug`, `preset_fingerprint.rs`); 7-band EQ fields in `MasteringSettings`; `AdvancedPanel.tsx`. | U6 |
| C-07 | Receipt panel showing `-11.0 LUFS / -0.8 dBTP / 8.4 LU / All good / Ready to ship` | `ProofDeck.tsx` `receipt` const | **Qualify** | These are **hand-authored numbers rendered as if they were product output**, with no marker that they are illustrative. They are plausible but they are not a measurement of anything. Either source them from a deterministic capture (U7) or label the block as an example. | U7, U6 |
| C-08 | "Every master ships with a receipt … No guesswork." | `ProofDeck.tsx` Technically-checked card | Proved | As C-04. | U6 |

## B — Platform and availability claims

| ID | Claim (as currently published) | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| C-10 | ~~"Download the free beta" → `https://github.com/DanielKinsner/yes-master/releases/latest`~~ → **the download action is now rendered from resolved release state** | `BetaDownload.tsx` / `release-config.ts` | **Removed (U5, 2026-07-25)** | The unconditional link is gone. `resolveRelease()` is pure and total, defaults to unavailable for absent/malformed/draft/prerelease/unverified/stale metadata, and `RELEASE_METADATA` ships `null`. Nothing on the page links to `/releases/latest`; unavailable states route to the releases *index*, which resolves whether or not a release exists. Pinned by `release-readiness.test.ts` (state model, incl. a negative control that the valid fixture *does* activate) and by the headless landing lane at all 12 viewports. | U5 (activation: U16, U17) |
| C-11 | "Windows + universal Mac" | `BetaDownload.tsx` | Proved (as intent) → **binding implemented, artifacts pending** | `release.yml` builds one universal macOS artifact + Windows MSI/NSIS; `release-readiness.test.ts` pins that contract. U5 added the binding: a release cannot activate without a repo-hosted `.exe` and universal `.dmg`, each with a positive byte size and a 64-char SHA-256. Binding to *real* artifacts still needs a published release (U16/U17). | U5, U16 |
| C-12 | "No email required" | `BetaDownload.tsx` | Proved | D5 / D16: download is ungated. `BetaSignup.test.tsx` pins the form as optional and safe-disabled. | U4 |
| C-13 | "Hosted on GitHub Releases" | `BetaDownload.tsx` | Proved | `release.yml`; KTD3. | U5 |
| C-14 | "Early beta installers are not yet backed by paid publisher certificates, so Windows or macOS may ask you to confirm that you trust the download." | `BetaDownload.tsx` | Proved | D16; `docs/RELEASE_SIGNING_SETUP.md`. This is a correctly-qualified claim and is the model for the rest. | U5 |
| C-15 | "Coming to your pocket" / "Same engine, headed to iPhone & Android." / "coming after launch" | `CrossPlatform.tsx` | **Qualify** | Mobile is **parked** (D9, D15, KTD9) and has no owner-approved release date. "Coming after launch" is a roadmap promise the product has not committed to. Restate as not currently available, with no implied schedule. See the canon conflict note below. | U6 |
| C-16 | Six present-tense mobile feature claims — "A/B in sync", "Four styles", "Intensity control", "Real-time meters", "Quality checks", "No upload, no wait" | `CrossPlatform.tsx` `features` | **Qualify** | Each is *mechanically* true of the built bridges (`docs/PRODUCT.md` Mobile Companions; iPhone `ContentView.swift` four-preset picker). But they read as a feature list for a product a visitor can obtain, and they cannot. Present them as what the shared engine already does on the bridges, not as an available product. | U6 |
| C-17 | iPhone screenshot (`iphone-standard-ui.jpg`) presented adjacent to desktop beta proof | `CrossPlatform.tsx` | **Qualify** | R7 forbids a mobile UI image standing as current desktop-beta proof, and this asset has no capture-commit binding. Needs the U7 manifest treatment or an explicit non-current label. | U7 |
| C-18 | "Mac & Windows." | `FinalCTA.tsx` footnote | Proved | D1: beta ships Mac + Windows together. | U6 |

## C — Pricing and beta-promise claims

| ID | Claim (as currently published) | Surface | Status | Evidence source | Owning unit |
|---|---|---|---|---|---|
| C-20 | "Free during the beta." | `FinalCTA.tsx` | Proved | D16 / `docs/PRODUCT.md` Distribution & Business Model. | U6 |
| C-21 | "When it launches it's a one-time $29 (then $49)" | `FinalCTA.tsx` | Proved *as a business model* | `docs/PRODUCT.md` Distribution & Business Model; launch plan §pricing; D2. The **numbers** are settled canon. | U6 |
| C-22 | "beta testers keep $29 forever" | `FinalCTA.tsx` | **Owner** | This is an **entitlement promise**, not a price. R24 makes the founder window a public time-limited launch window; the exact terms, duration, and what a beta tester is entitled to are not decided. Conservative default: no entitlement wording until answered. → `docs/OWNER_INPUT_QUEUE.md` row 1. | U6 |
| C-23 | Beta end date / "concrete flip date" (D2 requires one; none is published) | not yet on the page | **Owner** | Undecided. Conservative default: the beta is described as time-boxed with no date, and a missing beta end date keeps the download **unavailable** in the U5 release-state model. → owner queue row 3. | U5, U6 |
| C-24 | "Optional email updates" signup | `FinalCTA.tsx` / `BetaSignup.tsx` | **Owner** | No provider selected; `SIGNUP_ENDPOINT = ""` and the form is safe-disabled (pinned by `BetaSignup.test.tsx`). Conservative default holds. → owner queue row 2. | U4 |
| C-25 | Beta-end behavior — installers withdrawn, installed builds keep working, no kill switch (R26/KTD13) | not yet stated publicly | **Missing** | Settled policy with no public surface yet. Must appear in the beta guide. | U4 |
| C-26 | GitHub account required to file structured feedback (R19/KTD4) | not yet stated publicly | **Missing** | Settled policy with no public surface yet. Must be disclosed *before* the handoff to GitHub, not after. | U4 |

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
