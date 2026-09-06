# YES Master — Owner Input Queue

The single place where **questions only the owner can answer** accumulate while
an agent is executing a chunk of
`docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`.

**Listening already supplied:** the owner conducted the September 5 native
Windows pass. Read [the reconciled answers](listening/2026-09-05-owner-handoff.md)
and [follow-up plan](plans/2026-09-05-listening-follow-up.md) before adding a
listening or reproduction question. Do not repeat answers supplied in prose or
turn the original export's generated warnings into new owner requests. The rows
below concern real product choices, not an unperformed listening session.

## Why this file exists

Record real unresolved owner choices so independent work can continue without
inventing an answer or repeatedly asking the same question. First check current
instructions and recorded decisions: routine implementation choices and work
already authorized do not belong here.

## Rules

1. **Never block on an owner decision unrelated to available independent work.**
   Pause only dependent work. Add the unresolved choice here and continue
   independent authorized work. Ask at the point the answer is needed; do not
   stop an entire task for a decision unrelated to its next useful step.
2. **Do not invent a default decision.** Preserve existing working behavior when
   appropriate and keep unapproved new functionality unavailable. Do not disable
   working features to appear conservative. Where the owner requires an explicit
   choice (such as the album manifest), mark it pending without selecting an
   option. Claims must remain accurate; no invented date, price, or approval.
3. **This is not a decision log.** Once the owner answers, the decision moves to
   `docs/OPEN_THREADS_AND_DECISIONS.md` and the row here is struck through with
   a pointer to where the decision now lives.
4. **A row is not a reason to ask again after an answer.** Apply current user
   authorization and record settled decisions in the log and affected internal
   docs. No additional permission is needed just to document the answer.

## Schema

| Date | Unit | Question | Conservative default in place | What changes when answered |
|---|---|---|---|---|

---

## Open

| Date | Unit | Question | Conservative default in place | What changes when answered |
|---|---|---|---|---|
| 2026-09-05 | Listening follow-up 2A/2B | **Import readiness/loading experience:** publish each completed track for use while remaining analysis runs, or complete preparation before opening editing? The owner explicitly requested discussion before changes. | Keep the current experience while measuring time to first usable track, total batch time, and playback contention. Per-batch analysis is already sequential; results are published together. | The measured tradeoffs determine incremental readiness/profile publication or an explicit preparation experience. |
| 2026-09-05 | Listening follow-up 2 | **Start Preview LUFS automatically on import when the selected mode already requests it?** | Preserve current behavior; assess selected-track resource cost, cancellation, and source/settings lifetime first. Do not start whole-track work for every imported file by assumption. | A bounded selected-track premeasurement lifecycle can be specified with truthful pending state. |
| 2026-09-05 | Listening follow-up 5C | **Album manifest — two choices only:** write it in `metadata/`, or remove it and rely on the in-app Album receipt. The owner explicitly rejects defaulting to keeping it. The exact master filename suffix is a separate naming choice. | **Decision pending; neither option selected.** No output files are changed during planning. Current behavior is not a chosen keep default. Explain the portable-record/persistence tradeoff and present these two options. | `metadata/`: update paths and cleanup. Removal: stop writing JSON and remove mandatory-manifest assumptions from reports/UI/contracts while retaining a usable in-app receipt and legacy readability. Neither choice deletes existing owner exports. |
| 2026-09-05 | Listening follow-up 6 | **First extra-format scope:** MP3 is requested, including conversion without mastering; settle Track/Album/Standard scope, quality/defaults, output-source selection, and corresponding product-doc updates. | Existing WAV delivery stays truthful; no codec is advertised or implemented by the documentation update. Prepare the bounded specification from the supplied request rather than asking whether the owner wants MP3 again. | Deliver the agreed MP3/source-conversion increment and update product/behavior/help/public descriptions to match verified functionality. |
| 2026-07-24 | U1 (for U4/U6) | **Founder-window dates and exact purchase terms** (R24). `docs/PRODUCT.md` already records the $29→$49 founder/standard split as a settled business model, but the *window* — when it opens, how long it runs, what exactly a beta tester is entitled to and for how long — is not decided. | No date, duration, or entitlement wording appears in any public surface or doc. Where copy must reference the window it stays qualified ("a time-limited founder price will be announced") with no number attached. | U6 landing copy and U4 newsletter/beta-guide copy can state the window. Until then any concrete term is a hard stop, not an agent call. |
| 2026-07-24 | U1 (for U4) | **Newsletter provider, consent storage, retention period, and sender identity.** `src/landing/signup-config.ts` ships `SIGNUP_ENDPOINT = ""` and the form is safe-disabled. No vendor is selected. | Signup form stays safe-disabled. No vendor is inferred, no endpoint is written, no consent text promises a retention period. Download access does not depend on it. | U4 can wire the form, write real consent/retention/unsubscribe copy, and add the provider-outage path. A provisioning/integration review runs before it goes live (hardening plan Workstream F). |
| 2026-07-24 | U1 (for U4/U6) | **Beta end date.** D2 specifies a ~8-week timebox with a concrete flip date announced on the landing page; the date itself is unset. **Provisional answer 2026-08-31 (owner, in-session): 2026-10-31**, pending the owner's launch-cost review — confirm at publish time before it lands in `RELEASE_METADATA` or public copy. | Public copy says the beta is time-boxed without naming a date. **Implemented in U5 (2026-07-25):** `resolveRelease()` in `src/landing/release-config.ts` treats an absent or malformed `betaEndsAt` as a reason the download stays **unavailable**, and a date in the past closes it again — so a stale window can never keep a download live. Pinned by `release-readiness.test.ts`. | The date can be set in `RELEASE_METADATA` (which also needs a published, verified release — see `docs/RELEASE_SIGNING_SETUP.md` "Activating the landing download") and U6 can print it. |
| 2026-07-24 | U1 (for U17) | **Public beta announcement date and publication authorization.** | Nothing is announced. No non-draft release is published, no landing deploy happens. | U17 can deploy and the owner announces. This is an owner action in every case — the answer sets timing, not authority. |
| 2026-07-24 | U3 | **Headless browser runtime for the web E2E lane** — Playwright's bundled Chromium vs the installed Chrome channel. *Agent-settled: U3 explicitly delegates this choice. Logged so the owner can see and reverse it, not because it blocked anything.* | **Bundled Chromium.** `channel: "chrome"` was dropped; the runtime now lives in one place (`scripts/lib/headless-browser.mjs`) and is pinned by the `playwright` entry in `package-lock.json`. CI installs it explicitly with `npx playwright install --with-deps chromium`. A missing browser **fails** the lane instead of skipping. Verified before adopting: the full landing suite produces **identical** results under both runtimes — 0 failures each, 12 viewports, 8 anchor checks, 0 per-viewport metric mismatches. | Nothing is waiting on this. Reversing it is one `channel` argument in `headless-browser.mjs` plus the CI install target. |
| 2026-07-24 | U1 | **Do the `docs/legal/` drafts ship as-is with the beta?** D6 records that legal is explicitly *not* a beta gate and the owner researches independently — which answers "is it blocking" but not "does it ship". | Drafts stay in the repo, unbundled and unlinked from any public surface. | The beta guide and installer can reference them. |
| 2026-09-01 | Task 12 (U14→U16) | **Beta.1 disposition.** Two independent blocker sets are now confirmed in the tagged `c750da6` bytes: the 2026-08-31 adversarial-audit findings, and the 2026-09-01 Mac QA finding that a format-only bit-depth/sample-rate edit silently replaced the active loudness target (fixed on `main` by `1400373`, re-verified at `9225da6`). Reject `v0.9.2-beta.1` and authorize a `v0.9.2-beta.2` tag at the exact reviewed `main` tip? | Beta.1's tag, draft (id 379883047), and evidence stay frozen and untouched; nothing is published; the staged beta.1 installers are not run. | An agent tags `v0.9.2-beta.2` at the named commit, watches the tag-triggered Release run to green, re-verifies asset checksums, and the owner launch checklist re-runs against the beta.2 draft. |
| 2026-09-01 | Ship review S2 | **Custom domain?** `yesmaster.app` was written into the page metadata but was never purchased. | All public URLs use the deployed origin `yes-master.vercel.app` (D1). | Buy the domain, attach it to the Vercel project, update the four `index.html` tags and the same-origin test, re-check the share card in a real chat paste. |
| ~~2026-08-31~~ | ~~Audit T-03 (for Task 14 site activation)~~ | ~~**Must the HERO CTA itself sit above the fold on short phones, or is the fixed nav CTA the acquisition guarantee while the hero CTA stays below the introductory copy?**~~ **ANSWERED 2026-09-01 (copy-first, D10 of the Friday ship plan)** — the fixed nav CTA is the all-axis acquisition control; the hero CTA stays below the introductory copy on short phones. Layout unchanged. Recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-09-01 block); the nav/hero distinction is now permanent in `docs/TESTING.md` "Landing quality gates". | — | — |
| ~~2026-07-27~~ | ~~U14→U16~~ | ~~**Merge the proven release-workflow fix and move the candidate tag?**~~ **ANSWERED same day** — owner approved merge + tag move; decision recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-07-27 freeze banner block). Merge `34f7c88`, tag moved, tag-triggered Release run `30294627200` green with the complete 9-asset draft. | — | ~~Remaining owner click: delete the three stray `yes-master-v0.9.1-manual-*` draft releases created by the proving runs.~~ **Done 2026-08-31** — all four stale 07-27 drafts deleted with owner approval (go/no-go ledger, U14-close row). |
## Answered

| Date | Unit | Question | Decision |
|---|---|---|---|
| ~~2026-08-20~~ | ~~Review follow-up~~ | ~~**Should Space keep driving play/pause when a checkbox or radio has focus?**~~ | **ANSWERED 2026-08-25** — yes, carve out checkbox/radio (Space is their only keyboard toggle); DAW feel kept everywhere else. Shipped `167f066` with a regression test verified red pre-fix. Recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-08-25 block). |
| ~~2026-08-20~~ | ~~Review follow-up~~ | ~~**Do the single-key shortcuts (A / L / ?) need a disable/remap preference?**~~ | **ANSWERED 2026-08-25** — ship as-is for beta; a disable/remap preference is post-beta polish, not a gate. Recorded in `docs/OPEN_THREADS_AND_DECISIONS.md` (2026-08-25 block). |
