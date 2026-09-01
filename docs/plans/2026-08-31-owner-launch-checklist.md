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
>
> **⚠ 2026-09-01: a second, independent blocker is confirmed in the beta.1
> bytes.** A hands-on packaged-app QA pass on the Mac (at `3d9ba6c`) found
> that changing only Track Master bit depth or sample rate silently replaced
> the active loudness target with a hidden analysis offset (a track showing
> Streaming −14.0 LUFS jumped to 0.3 LUFS after a format-only edit; a silent
> stress fixture showed 56.0 LUFS). The defect exists in the tagged `c750da6`
> bytes, so **the staged beta.1 installers on the office PC carry it too.**
> Fixed on `main` by `1400373` (format edits now route through the Custom
> transition rule that captures the effective loudness/ceiling first, with
> regression tests), re-verified at `9225da6` — see the 2026-09-01 rows in
> `docs/plans/beta-go-no-go.md`. With both the audit blockers and this
> defect in the tagged bytes, the recommended Task-12 disposition is:
> **reject `v0.9.2-beta.1`, tag `v0.9.2-beta.2` at the exact reviewed
> `main` tip, re-run the Release workflow, and run this checklist against
> the beta.2 draft.**
>
> **Remediation is COMPLETE and independently reviewed.** Exact task↔commit
> map on `codex/launch-readiness-remediation` — self-contained here on
> purpose: this is the record you act from, so it must not lean on another
> file. Task 1 `b81d820` import panic shield · Task 2 `d874d90` updater
> latch/replay · Task 3 `5fe2a90` failed-install recovery · Task 4
> `e1f0b36` sticky-TOOLS opacity · Task 5 `6986032` + `9f60023` contrast
> token, loudness name/axe gate · Task 6 `a20ab26` receipt focus trap ·
> Task 7 `a15284a` receipt shell · Task 8 `9a4bf1f` + `63657cc` +
> `acab580` loaded-Standard, album advisories, CTA honesty · Task 9
> `1c9b35a` vite config · Task 10 `80e2898` RustSec gate · Task 11
> `691392d` docs reconcile · capture refresh `69c789f` · Codex review
> `6ed3b53` · review fixes `016b29f` (updater), `333c6dc` (contrast),
> `0cf48ce` (axe settle + geometry) · launch records `9919611` plus this
> map. The same map is repeated in `docs/CHANGELOG.md` (2026-08-31 entry)
> and `docs/plans/beta-go-no-go.md`. Your Task-12 disposition decides what
> happens next; any beta.2 build must name the exact reviewed commit it
> was built from.

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
3. First launch: macOS will block it. Open **System Settings → Privacy &
   Security**, find the YES Master message, click **Open Anyway**, then
   **Open**. (Right-click → Open no longer bypasses Gatekeeper for unsigned
   apps on macOS 15 and later.) Expected "unidentified developer" warning (no
   Apple Developer account yet).
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
