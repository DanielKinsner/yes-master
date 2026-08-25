# YES Master — Owner Input Queue

The single place where **questions only the owner can answer** accumulate while
an agent is executing a chunk of
`docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`.

## Why this file exists

The quality program is long-horizon and meant to run unattended. Without this
file an agent that hits an owner-shaped question has two bad options: stall the
whole chunk, or invent an answer. This file gives it a third: **write the
question down, ship the conservative default, and keep going.**

## Rules

1. **Never block on an owner decision.** Add a row here, implement the
   conservative default, finish everything in the unit that does not depend on
   the answer.
2. **The conservative default is always the one that under-promises** —
   unavailable over available, off over on, qualified copy over confident copy,
   placeholder over invented date, unproved over assumed-proved. A default that
   makes a public claim is never conservative.
3. **This is not a decision log.** Once the owner answers, the decision moves to
   `docs/OPEN_THREADS_AND_DECISIONS.md` and the row here is struck through with
   a pointer to where the decision now lives.
4. **A row is not a licence to guess later.** Until it is answered, the
   conservative default stands even if it looks unfinished.

## Schema

| Date | Unit | Question | Conservative default in place | What changes when answered |
|---|---|---|---|---|

---

## Open

| Date | Unit | Question | Conservative default in place | What changes when answered |
|---|---|---|---|---|
| 2026-07-24 | U1 (for U4/U6) | **Founder-window dates and exact purchase terms** (R24). `docs/PRODUCT.md` already records the $29→$49 founder/standard split as a settled business model, but the *window* — when it opens, how long it runs, what exactly a beta tester is entitled to and for how long — is not decided. | No date, duration, or entitlement wording appears in any public surface or doc. Where copy must reference the window it stays qualified ("a time-limited founder price will be announced") with no number attached. | U6 landing copy and U4 newsletter/beta-guide copy can state the window. Until then any concrete term is a hard stop, not an agent call. |
| 2026-07-24 | U1 (for U4) | **Newsletter provider, consent storage, retention period, and sender identity.** `src/landing/signup-config.ts` ships `SIGNUP_ENDPOINT = ""` and the form is safe-disabled. No vendor is selected. | Signup form stays safe-disabled. No vendor is inferred, no endpoint is written, no consent text promises a retention period. Download access does not depend on it. | U4 can wire the form, write real consent/retention/unsubscribe copy, and add the provider-outage path. A provisioning/integration review runs before it goes live (hardening plan Workstream F). |
| 2026-07-24 | U1 (for U4/U6) | **Beta end date.** D2 specifies a ~8-week timebox with a concrete flip date announced on the landing page; the date itself is unset. | Public copy says the beta is time-boxed without naming a date. **Implemented in U5 (2026-07-25):** `resolveRelease()` in `src/landing/release-config.ts` treats an absent or malformed `betaEndsAt` as a reason the download stays **unavailable**, and a date in the past closes it again — so a stale window can never keep a download live. Pinned by `release-readiness.test.ts`. | The date can be set in `RELEASE_METADATA` (which also needs a published, verified release — see `docs/RELEASE_SIGNING_SETUP.md` "Activating the landing download") and U6 can print it. |
| 2026-07-24 | U1 (for U17) | **Public beta announcement date and publication authorization.** | Nothing is announced. No non-draft release is published, no landing deploy happens. | U17 can deploy and the owner announces. This is an owner action in every case — the answer sets timing, not authority. |
| 2026-07-24 | U3 | **Headless browser runtime for the web E2E lane** — Playwright's bundled Chromium vs the installed Chrome channel. *Agent-settled: U3 explicitly delegates this choice. Logged so the owner can see and reverse it, not because it blocked anything.* | **Bundled Chromium.** `channel: "chrome"` was dropped; the runtime now lives in one place (`scripts/lib/headless-browser.mjs`) and is pinned by the `playwright` entry in `package-lock.json`. CI installs it explicitly with `npx playwright install --with-deps chromium`. A missing browser **fails** the lane instead of skipping. Verified before adopting: the full landing suite produces **identical** results under both runtimes — 0 failures each, 12 viewports, 8 anchor checks, 0 per-viewport metric mismatches. | Nothing is waiting on this. Reversing it is one `channel` argument in `headless-browser.mjs` plus the CI install target. |
| 2026-07-24 | U1 | **Do the `docs/legal/` drafts ship as-is with the beta?** D6 records that legal is explicitly *not* a beta gate and the owner researches independently — which answers "is it blocking" but not "does it ship". | Drafts stay in the repo, unbundled and unlinked from any public surface. | The beta guide and installer can reference them. |
| ~~2026-07-27~~ | ~~U14→U16~~ | ~~**Merge the proven release-workflow fix and move the candidate tag?**~~ **ANSWERED same day** — owner approved merge + tag move; decision recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-07-27 freeze banner block). Merge `34f7c88`, tag moved, tag-triggered Release run `30294627200` green with the complete 9-asset draft. | — | Remaining owner click: delete the three stray `yes-master-v0.9.1-manual-*` draft releases created by the proving runs. |
## Answered

| Date | Unit | Question | Decision |
|---|---|---|---|
| ~~2026-08-20~~ | ~~Review follow-up~~ | ~~**Should Space keep driving play/pause when a checkbox or radio has focus?**~~ | **ANSWERED 2026-08-25** — yes, carve out checkbox/radio (Space is their only keyboard toggle); DAW feel kept everywhere else. Shipped `167f066` with a regression test verified red pre-fix. Recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-08-25 block). |
| ~~2026-08-20~~ | ~~Review follow-up~~ | ~~**Do the single-key shortcuts (A / L / ?) need a disable/remap preference?**~~ | **ANSWERED 2026-08-25** — ship as-is for beta; a disable/remap preference is post-beta polish, not a gate. Recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-08-25 block). |
