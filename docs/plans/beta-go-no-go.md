# YES Master — Beta Go / No-Go Checklist

> The single gate list to clear before the free public beta goes live. Every
> item is **blocking** unless marked *advisory*. Check a box only when its
> **evidence** exists — not when it "should be fine." When every blocking box is
> checked, fill in the decision block at the bottom.
>
> **Announcing the beta is an OWNER action.** The agent never publishes a
> non-draft release, enables the live signup form, or posts an announcement
> (see the plan's stop-and-ask triggers). This doc gets you to the point where
> the owner can flip the switch — it does not flip it.

Legend — **Lane:** `agent` (done in the Opus lane) · `payday` (needs the owner's
paid accounts / secrets) · `owner` (only the owner can do it, e.g. by ear or on
real hardware).

---

## 1. Engineering readiness (agent lane — shipped)

- [x] **Beta version 0.9.0** across the three desktop manifests; Windows
      publisher = "Daniel Kinsner". *Evidence:* Slice 1; `version-coherence` +
      packaging tests; `npm run build:windows` produces `YES Master_0.9.0`.
- [x] **Product canon reflects the beta model** (free public beta → paid 1.0,
      Distribution & Business Model, mobile/landing scope). *Evidence:* Slice 2;
      `docs/PRODUCT.md`.
- [x] **Beta-scope UX fixes** land and are tested: per-track view memory,
      album `<AlbumTitle>/` subfolder, autosave-at-analysis-complete,
      export-in-progress tooltip, 1360×740 min-window. *Evidence:* Slices 3–6;
      frontend + Rust lanes green.
- [x] **Auto-updater integrated** (background check + toast + one-click install,
      silent offline). *Evidence:* Slices 7 / 7b; `tauri.conf.json`
      `plugins.updater`; `src/App.tsx` updater toast; frontend + Rust lanes.
- [x] **Premium-parity UI pass** applied (floating surfaces on the shared
      elevation scale). *Evidence:* Slice 9; the before/after A/B set.
- [ ] **The latest completed full CI run is green** (Windows, macOS, Android
      lanes + DSP snapshots). Public-repository runners removed the prior
      billing gate on 2026-07-13; check this box only after the current commit's
      run completes green. *Advisory reminder:* do not merge red.
- [x] **Owner-gate tripwires green** — AC-5, Phase-B, and album-character stay
      OFF for beta (D7). *Evidence:* `src-tauri/tests/owner_gates_default.rs`.

## 2. Signing & release pipeline (payday lane)

- [ ] **Owner accounts created:** Apple Developer and Azure Trusted Signing.
      Normal CI currently runs free while the repository is public. *Owner; per
      `docs/RELEASE_SIGNING_SETUP.md`.*
- [ ] **Signing secrets added** to the repo (Apple + Azure). *Owner.*
- [ ] **`release.yml` activated:** a tagged release builds, **signs**,
      notarizes (macOS), and drafts a GitHub Release with installers
      (macOS universal binary + Windows MSI/NSIS) attached. *Slice 10.*
- [ ] **Signed artifacts are downloadable** from that release (draft is fine for
      testing). *Slice 10.*

## 3. Updater gates (added 2026-07-08 — both blocking)

- [ ] **(i) Bootstrap updater pubkey replaced with the owner's real keypair
      BEFORE the first signed release.** The committed pubkey in
      `plugins.updater` is a throwaway whose private half was discarded; a
      shipped app can only ever accept updates signed by the key baked into it,
      so this is a one-way door. *Owner; steps in `docs/RELEASE_SIGNING_SETUP.md`
      → "Auto-updater signing"; add `TAURI_SIGNING_PRIVATE_KEY` +
      `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets.*
- [ ] **(ii) Update path proven end-to-end, once.** Install a 0.9.0 build,
      publish a draft **0.9.1** release, and confirm on a real machine that the
      app shows the update toast → "Restart to update" downloads, installs, and
      relaunches into 0.9.1. *Owner, on the M4 and/or Windows box.*

## 4. Real-machine confirmation (owner lane)

- [ ] **macOS build installs and runs on the M4** (closes the RC criterion in
      `docs/PRODUCT.md`). *Owner.*
- [ ] **Windows installer runs on the current box** (note SmartScreen behaviour
      until reputation accrues — expected, per `docs/RELEASE_SIGNING_SETUP.md`).
      *Owner.*
- [ ] **Real-time feels snappy on both machines** (the headline promise).
      *Owner.*
- [ ] *Advisory:* friend-run Intel-Mac smoke test (install / import / master /
      export). Non-blocking (D12 — the tolerance golden removed Intel risk).

## 5. Listening gate (owner lane)

- [ ] **The one-sitting listening runbook is executed and signed off**, and its
      filled-in note is saved. Closes the Manual Listening Gate + preset
      distinctness (#7). *Owner; script:
      `docs/plans/2026-07-08-beta-listening-runbook.md`.* Any blocker it surfaces
      is itself blocking; any DSP/preset change it implies is owner-gated.

## 6. Landing page & signup (payday lane — Slice 11)

- [ ] **Download button wired** to the GitHub Releases artifacts (ungated, D5),
      replacing the current `mailto:`.
- [ ] **Beta copy live:** the beta model, a **concrete flip date**, and the $29
      founder-price promise (D2). *Flip-date / pricing copy that deviates from D2
      is a stop-and-ask, not an agent call.*
- [ ] **Signup capture verified end-to-end** against the chosen provider (D4),
      after re-running the hardening plan's Workstream F checklist (pinned by
      `src/landing/BetaSignup.test.tsx`). The form stays safe-disabled until
      this passes. *`npm run verify:landing` green.*

## 7. Legal (owner lane — D6, not a hard gate)

- [ ] **EULA / privacy / refund drafts bundled as-is** from `docs/legal/`.
      Legal is explicitly **not a beta gate** (D6) — owner researches
      independently; a lawyer pass is a paid-flip item at most. Listed here only
      so the drafts ship with the beta rather than being forgotten.

## 8. Go-to-market & announce (owner lane)

- [ ] **Owner GTM checklist** (KVR / BPB / micro-creators, 2–3 weeks of genuine
      participation first) — see `docs/plans/2026-06-30-launch-plan.md` §5.
      Owner lane; not an engineering gate.
- [ ] **Announce the beta.** OWNER ACTION ONLY. The agent never publishes a
      non-draft release or posts the announcement.

---

## Decision

Fill in when the blocking boxes above are checked.

- Date / decider: __________________________
- Build (version + commit + release tag): __________________________
- Blocking items still open: __________________________
- Updater gates (§3) both cleared? YES / NO
- Listening note signed off (§5)? YES / NO
- **DECISION:** GO (owner announces) / NO-GO (blockers above)
