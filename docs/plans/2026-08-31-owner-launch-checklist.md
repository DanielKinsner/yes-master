# YES Master — Owner Launch Checklist (2026-08-31)

> **⚠ 2026-08-31 (later the same day): candidate `v0.9.2-beta.1` is
> AUDIT-BLOCKED / NO-GO pending owner disposition.** The adversarial audit
> confirmed launch blockers in release-bound code (hostile-import panic,
> updater notice loss/recovery, sticky-TOOLS transparency, receipt
> accessibility) — see
> `docs/superpowers/plans/2026-08-31-adversarial-audit-remediation-and-launch-readiness.md`
> and the Claude triage beside it. Remediation is landing on
> `codex/launch-readiness-remediation`. **Do not run the steps below against
> the beta.1 draft** until the owner has recorded a beta.1 disposition (Task
> 12); if beta.1 is rejected, this checklist re-runs against the replacement
> `v0.9.2-beta.2` candidate. Beta.1 is not yet rejected — its tag, draft
> (id 379883047), and evidence stay frozen and untouched.

This is the exact remaining path to a live free public beta, written for the
owner: what to click, what you should see, and what to do if you don't see it.
Gate references: `docs/plans/beta-go-no-go.md` (§3ii, §4, §5, §6, §8).

**Where things stand (pre-audit record):** candidate `v0.9.2-beta.1` is tagged
at `c750da6`, the draft release (id 379883047) built green with all 9 assets,
checksums verified independently on the Windows box, and the four stale 0.9.1
drafts are deleted. The draft is NOT flagged prerelease, so publishing it makes
it `/releases/latest` — which the auto-updater and the landing page both
require.

**Cost to launch: $0.** Optional post-beta spends: Apple Developer $99/yr
(removes the Mac "unidentified developer" warning on future releases), Azure
Trusted Signing ~$10/mo (softens Windows SmartScreen), Buttondown (email list,
free ≤100 subscribers). None block anything below.

Staged on the office PC: `Downloads\yes-master-0.9.2-beta\` — the draft's
`.msi`, `setup.exe`, `.dmg`, `SHA256SUMS.txt` (all verified MATCH), plus
`SEED - YES Master_0.9.0_x64_en-US.msi` for step 4.

---

## Step 1 — Install the candidate on the Windows box (~5 min)

1. Open `Downloads\yes-master-0.9.2-beta\`.
2. Double-click `YES.Master_0.9.2_x64_en-US.msi`. It installs over the local
   dev build.
3. If SmartScreen shows "Windows protected your PC": click **More info →
   Run anyway**. Expected until download reputation accrues; this is the
   documented $0-launch behavior.
4. Launch YES Master. **You should see:** the app opens, version 0.9.2, and it
   feels snappy in real-time audition.

## Step 2 — Install on the M4 Mac (~10 min)

1. On the Mac, sign into GitHub in a browser → `DanielKinsner/yes-master` →
   **Releases** → the draft **YES Master 0.9.2 — Public Beta** (drafts are only
   visible to you while signed in).
2. Download `YES.Master_0.9.2_universal.dmg`, open it, drag YES Master to
   Applications.
3. First launch: **right-click the app → Open → Open**. Expected "unidentified
   developer" warning (no Apple Developer account yet).
4. **You should see:** the app opens and feels snappy. This closes the
   PRODUCT.md real-machine RC criterion.

## Step 3 — Final by-ear spot-check (~15–20 min, either machine)

The one remaining listening gate (everything else was approved 2026-08-25).
On an *installed* build from steps 1–2:

1. Import a normal track → master it → A/B Original/Mastered a few times.
2. Import an already-mastered track → confirm the app doesn't wreck it.
3. Export once → listen to the exported file itself.

**If anything sounds wrong, stop here and tell the agent — that's blocking.**
The full 2026-07-08 runbook exists if you want a script, but is not required.

## Step 4 — Prove the updater, then publish (~10 min, Windows box)

Order matters: the toast can only appear *after* the release is published.

1. Uninstall YES Master (Settings → Apps → YES Master → Uninstall).
2. Install `SEED - YES Master_0.9.0_x64_en-US.msi` from the staging folder.
3. On GitHub → Releases → open the draft → **Edit** (pencil) →
   **Publish release**. Do NOT tick "Set as a pre-release". This makes the
   files public but announces nothing — that's the plan (quiet publish).
4. Launch YES Master 0.9.0. **You should see:** within about a minute, an
   update toast → click it / "Restart to update" → the app downloads,
   installs, relaunches. Check the version: **0.9.2**.
5. If no toast appears or the update fails: tell the agent — blocking gate
   §3(ii).

## Step 5 — Tell the agent to flip the landing page (~2 min of your time)

Say: *"published, updater proven, listen OK, Oct 31 confirmed"* (or give a
different end date — Oct 31 is pencilled, provisional). The agent then:

- populates `RELEASE_METADATA` in `src/landing/release-config.ts` with the
  published artifact URLs, sizes, SHA-256s, `publishedAt`, `verifiedAt`, and
  `betaEndsAt`;
- runs `npm test` + `npm run verify:headless`, confirms `verified-public`;
- commits (owner-approved merge under the freeze) and pushes — Vercel
  auto-deploys;
- verifies the live page at `yes-master.vercel.app` shows both download
  buttons with sizes and checksums.

## Step 6 — Three free GitHub security toggles (~2 min)

Repo → **Settings → Code security** (recommended before announcing, from the
2026-08-20 review): enable **Dependabot alerts + security updates**,
**Secret scanning + push protection**, **Private vulnerability reporting**.

## Step 7 — Announce, whenever you're ready

Your mic, your timing (go/no-go §8). GTM notes:
`docs/plans/2026-06-30-launch-plan.md` §5 (KVR / BPB / micro-creators — the
plan recommends 2–3 weeks of genuine participation before posting links).
Fill in the Decision block at the bottom of `docs/plans/beta-go-no-go.md`
when you flip the switch.

---

*Prepared by the 2026-08-31 session. Evidence for every checked box:
`docs/plans/beta-go-no-go.md` §9 ledger, 2026-08-31 rows.*
