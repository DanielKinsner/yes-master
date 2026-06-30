# Landing Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebrand the marketing landing page to **YES Master** and rewrite its pitch to lead with performance + simplicity + honesty (privacy demoted), replace the dead `mailto:` CTA with a real free-beta email-capture form, and keep the `verify:landing` checks green.

**Architecture:** The landing is a Vite + React 19 + Tailwind v4 app under `src/landing/`, rendered by `src/LandingPage.tsx` when `wantsAppShell()` is false (`src/main.tsx:5-9`). All copy is hardcoded JSX strings (no constants file / i18n) — edits are surgical string changes plus one new component for email capture. Verification is via `npm run verify:landing` (Playwright over 12 viewports) which asserts specific copy strings, so its expected-copy list must move in lockstep.

**Tech Stack:** React 19, Vite 6, TypeScript 5.6, Tailwind CSS v4, Playwright (verify script).

## Global Constraints

- Brand is **YES Master** (never "Y.E.S. Master", "Your Endgame Sound", or "Instantly master") — decided 2026-06-30.
- Positioning leads with **performance / simplicity / honesty**; "local/offline" is a single speed-and-convenience line, never a privacy headline (see `docs/plans/2026-06-30-launch-plan.md` §2). Do NOT headline privacy.
- Approved hero headline: **"Master your track in real time — and see exactly what it did."**
- The launch is **desktop-first (Mac + Windows)**; mobile is deferred — landing must not promise mobile as available *now*.
- Beta is **free and email-gated**; the paid flip is a one-time **$29 founder → $49**. Beta sign-ups keep $29.
- Keep all existing section anchor ids (`top`, `mobile`, `standard`, `advanced`, `get-started`) — `verify:landing` asserts them.

---

### Task 1: Rebrand + rewrite the Hero

**Files:**
- Modify: `src/landing/Hero.tsx` (proof-point array ~lines 7-23; tagline ~line 47; headline ~lines 50-51; CTA labels ~lines 74-85)
- Modify: `src/landing/Nav.tsx` (brand text ~line 17; CTA label ~lines 32-36)

**What to change (match on the current string, not just line number):**

- [ ] **Step 1: Nav brand.** Replace `Y.E.S. Master` (Nav.tsx) with `YES Master`. Change the nav CTA label `Join desktop beta` → `Join the free beta` (keep its `href="#get-started"`).

- [ ] **Step 2: Hero eyebrow/tagline.** Replace `Y.E.S. Master / Instantly master your track` with `YES Master`.

- [ ] **Step 3: Hero headline.** Replace the `Your Endgame <span ...>Sound.</span>` headline with:
  ```
  Master your track in real time —
  <span class="block text-brand">and see exactly what it did.</span>
  ```
  (Preserve the existing `<h1>` classes and the `block text-brand` span pattern.)

- [ ] **Step 4: Hero sub-headline.** Immediately under the headline add one supporting line (reuse the existing sub-paragraph styling if present, else a `<p>` matching the hero text color):
  `Drop a track, pick a vibe, and hear the full mastering chain as you listen. No upload, no waiting, no black box.`

- [ ] **Step 5: Hero proof points.** Rewrite the three proof-point objects (currently Local-first / Real-time control / Release-ready) to:
  1. **Real-time, every tweak** — `The full chain runs as you listen — no upload, no reprocessing wait.`
  2. **Simple by default, deep when you want it** — `Master in one move, or open the full metering and album tools when you're ready.`
  3. **No black box** — `A pass/fail receipt shows your LUFS, true-peak, and dynamic range — push it hard and still see the truth.`
  (Keep the existing icon components/keys; only change title + description text.)

- [ ] **Step 6: Hero CTAs.** Change the primary CTA label `Join desktop beta` → `Join the free beta` (keep `href="#get-started"`). Leave `See Advanced control` (`href="#advanced"`) as-is.

- [ ] **Step 7: Verify build compiles.**
  Run: `npm run build`
  Expected: build succeeds, no TypeScript errors.

- [ ] **Step 8: Commit.**
  ```bash
  git add src/landing/Hero.tsx src/landing/Nav.tsx
  git commit -m "feat(landing): rebrand to YES Master + performance-led hero"
  ```

---

### Task 2: Reframe ProofDeck + CrossPlatform (demote privacy, soften mobile)

**Files:**
- Modify: `src/landing/ProofDeck.tsx` (headline ~lines 23-25; cards ~30-52; image alts ~40, 56)
- Modify: `src/landing/CrossPlatform.tsx` (tagline ~21; headline ~22-26; feature cards; alt ~36)

- [ ] **Step 1: ProofDeck alts.** Image alt text already says "YES Master …" — leave. Keep headline `One engine. Three ways to trust it.` (on-brand, and `verify:landing` checks "One engine").

- [ ] **Step 2: Demote privacy in cards.** In ProofDeck/CrossPlatform feature cards, find any card whose copy leads with privacy/"never leave your machine"/"No cloud" and reframe to performance/convenience. Specifically, change the CrossPlatform card titled `No cloud` to:
  - Title: `No upload, no wait`
  - Body: `Processing runs on your device, so every change is instant — nothing to upload or download.`

- [ ] **Step 3: Soften the mobile section to roadmap framing.** The launch is desktop-first; do not imply mobile is available now. In CrossPlatform.tsx:
  - Tagline (`The same endgame sound`) → `Coming to your pocket`
  - Headline (`Master anywhere. Same engine. Same truth.`) → `Same engine, headed to iPhone & Android.`
  - Add a small line under the headline: `Desktop first. The mobile companions share the exact same engine — coming after launch.`

- [ ] **Step 4: Verify build compiles.**
  Run: `npm run build`
  Expected: success.

- [ ] **Step 5: Commit.**
  ```bash
  git add src/landing/ProofDeck.tsx src/landing/CrossPlatform.tsx
  git commit -m "feat(landing): reframe proof + mobile copy (perf over privacy, mobile as roadmap)"
  ```

---

### Task 3: Replace the mailto CTA with a free-beta email-capture form

**Files:**
- Create: `src/landing/BetaSignup.tsx` (new email-capture component)
- Modify: `src/landing/FinalCTA.tsx` (replace the `mailto:` `<a>` with `<BetaSignup />`)
- Create: `src/landing/signup-config.ts` (single config point for the email-provider endpoint)

**Interfaces:**
- Produces: `BetaSignup` (default export, no required props) — renders an email `<input type="email" required>` + submit button, POSTs to `SIGNUP_ENDPOINT`, shows a success state on 2xx and an inline error otherwise.
- Consumes: `SIGNUP_ENDPOINT` from `signup-config.ts`.

- [ ] **Step 1: Config point.** Create `src/landing/signup-config.ts`:
  ```ts
  // Paste your email-provider form-action URL here (Buttondown / MailerLite / Kit).
  // The form POSTs `email` as form-encoded data. Empty string = form shows a
  // "coming soon" disabled state instead of submitting.
  export const SIGNUP_ENDPOINT = "";
  export const SIGNUP_FIELD = "email"; // form field name your provider expects
  ```

- [ ] **Step 2: Component.** Create `src/landing/BetaSignup.tsx` — controlled email input, submit handler that `fetch`es `SIGNUP_ENDPOINT` (method POST, `application/x-www-form-urlencoded`, body `${SIGNUP_FIELD}=<encoded email>`), with three states: idle, submitting, success ("You're on the list — we'll email your download link."), error ("Something went wrong — try again or email hello@yesmaster.app."). If `SIGNUP_ENDPOINT === ""`, render the button disabled with label `Beta sign-up opening soon`. Style with Tailwind to match the existing FinalCTA button (reuse its classes).

- [ ] **Step 3: Wire into FinalCTA.** In `src/landing/FinalCTA.tsx`, keep the headline `Stop chasing the master.` / `This is the one you stop on.`. Replace the `mailto:hello@yesmaster.app` `<a>` with `<BetaSignup />`. Replace the subtext `Works offline. No signup.` with:
  `Free during the beta. When it launches it's a one-time $29 (then $49) — beta testers keep $29 forever. Mac & Windows.`

- [ ] **Step 4: Verify build compiles.**
  Run: `npm run build`
  Expected: success, no TS errors.

- [ ] **Step 5: Commit.**
  ```bash
  git add src/landing/BetaSignup.tsx src/landing/signup-config.ts src/landing/FinalCTA.tsx
  git commit -m "feat(landing): free-beta email capture replaces dead mailto CTA"
  ```

---

### Task 4: Update verify:landing assertions + verify the page

**Files:**
- Modify: `scripts/verify-landing-responsive.mjs` (expected-copy list)

- [ ] **Step 1: Update expected copy.** In `scripts/verify-landing-responsive.mjs`, the expected-copy assertions currently include `"Your Endgame"` and `"Master anywhere"` (both removed by Tasks 1-2). Update the expected-copy array to the new strings that must be present: `"see exactly what it did"`, `"One engine"`, `"Stop chasing the master"`, and `"Same engine"`. Keep the title assertion (`"YES Master"`) and all section-id assertions unchanged.

- [ ] **Step 2: Run the visual verification (preferred in this environment).** Start the dev server and screenshot the landing with the preview tools (the Playwright `verify:landing` may not bind in a headless sandbox). Confirm: brand reads "YES Master" everywhere, the new hero headline renders, the email form appears in the final section, no "Y.E.S." / "Your Endgame Sound" remains, and there's no horizontal overflow at mobile + desktop widths.

- [ ] **Step 3: Run the verifier if the environment allows.**
  Run: `npm run verify:landing`
  Expected: PASS (title "YES Master", all section ids present, new copy strings found, anchors work, no overflow/broken images). If it can't bind a server in this environment, rely on Step 2's preview screenshots and note that CI will run the verifier.

- [ ] **Step 4: Commit.**
  ```bash
  git add scripts/verify-landing-responsive.mjs
  git commit -m "test(landing): update verify:landing copy assertions for rebrand"
  ```

---

## Deferred (explicit, not silently dropped)

- **Interactive Original/Mastered A/B hero demo.** The A/B toggle lives inline in `src/App.tsx:1363-1381`; embedding it on the landing needs extraction into a standalone component AND royalty-clear before/after audio the owner must provide. Track as a follow-up — it is the single highest-leverage conversion asset, but it is blocked on owner-supplied audio.
- **Real email list.** `SIGNUP_ENDPOINT` is intentionally empty until the owner picks/creates a provider (Buttondown / MailerLite / Kit). Form ships in a graceful "opening soon" state until then.
- **Real download link.** The CTA captures emails during beta; wiring an actual installer URL happens in the release-pipeline plan once signed artifacts exist on GitHub Releases.

## Self-Review

- **Spec coverage:** Brand rename (T1-2), performance/simplicity/honesty rewrite (T1-2), privacy demoted (T2), email capture replaces mailto (T3), beta + $29 promise on page (T3), mobile softened to roadmap (T2), verifier kept green (T4). Covered.
- **Placeholders:** `SIGNUP_ENDPOINT=""` is a documented config seam with a defined graceful fallback, not a code gap. No other placeholders.
- **Consistency:** `BetaSignup` name + `SIGNUP_ENDPOINT`/`SIGNUP_FIELD` used consistently across T3. Anchor ids untouched so `verify:landing` section checks still pass.
