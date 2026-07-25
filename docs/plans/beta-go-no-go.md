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

Legend — **Lane:** `agent` (mechanical code/release work) · `owner` (only the
owner can do it, e.g. by ear, on real hardware, or by publishing). Paid signing
is post-beta advisory under D16.

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

## 2. $0 release pipeline (agent + owner lanes)

- [x] **`release.yml` implements the $0 lane:** one universal macOS build,
      Windows MSI/NSIS, mandatory updater signatures, SHA-256 checksums, a
      draft release, and a final asset audit. Partial Apple/Azure secret groups
      fail closed; absent paid groups do not block the beta. *Agent; D16.*
- [ ] **A draft `v0.9.0-beta.1` workflow run is green** and contains `.dmg`,
      `.app.tar.gz`, `.msi`, `.exe`, updater `.sig` files, `latest.json`, and
      `SHA256SUMS.txt`. *Agent prepares; owner authorizes tag/push.*
- [ ] **Draft artifacts are downloadable and their checksums match.** *Owner or
      agent, before publish.*
- [ ] *Advisory / post-beta:* Apple Developer notarization and Azure Artifact
      Signing are configured when funding permits. They reduce OS-warning
      friction but do not block the $0 beta.

## 3. Updater gates (added 2026-07-08 — both blocking)

- [x] **(i) Permanent updater key configured before the first release.** The
      public key is committed, both GitHub signing secrets exist, and a local
      Windows updater-enabled build emitted signed artifacts. The encrypted
      private key remains outside git. *Evidence: 2026-07-20 release lane.*
- [ ] **(ii) Update path proven end-to-end, once.** Install a 0.9.0 build,
      then owner-publish a full **0.9.1** release quietly before the public
      announcement. GitHub's `/latest` channel cannot see drafts. Confirm on a
      real machine that the app shows the update toast → "Restart to update"
      downloads, installs, and relaunches into 0.9.1. The 0.9.1 files become
      publicly accessible at this step even though they are not announced.
      *Owner authorization required; M4 and/or Windows box.*

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

## 6. Landing page & optional signup

- [x] **Download button wired** to GitHub Releases (ungated, D5/D16), with
      Windows + universal Mac expectations and OS-warning disclosure.
- [ ] **Beta copy live:** the beta model, a **concrete flip date**, and the $29
      founder-price promise (D2). *Flip-date / pricing copy that deviates from D2
      is a stop-and-ask, not an agent call.*
- [ ] *Advisory:* **signup capture verified end-to-end** against the chosen provider (D4),
      after re-running the hardening plan's Workstream F checklist (pinned by
      `src/landing/BetaSignup.test.tsx`). The form stays safe-disabled until
      this passes. The ungated download remains live without signup.
      *`npm run verify:landing` green.*

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

## 9. Exact-commit evidence ledger (added 2026-07-24 — U1 / R17 / KTD12)

> **A checked box above is a claim. This table is the evidence behind it.**
>
> Every release-candidate proof item records **commit SHA, platform,
> artifact/version, the command or procedure run, the result, and where the
> evidence lives.** A green result from another checkout, a rebuilt artifact, or
> an earlier SHA **is not release evidence** (KTD12). Release evidence is
> invalidated by a new release commit unless the ledger names the check as
> demonstrably commit-independent *and says why*.

**Evidence layer** must be one of these, and they are **not interchangeable**:

| Layer | Proves | Does not prove |
|---|---|---|
| `browser-headless` | Responsive layout, deterministic UI journeys, keyboard semantics, link/asset behavior, preview-state handling | Native dialogs, real audio device behavior, signing, updater install, listening quality |
| `frontend-unit` | State rules, copy invariants, accessible names/states, deterministic rendering | Cross-page integration or installed behavior |
| `native-synthetic` | Engine/import/render/export/project contracts with generated audio | Installed shell integration, real device output, subjective sound |
| `private-fixture` | Behavior on representative real program material without committing audio | Owner preference or public licensing |
| `installed-machine` | Installer, OS dialogs, file flows, updater, app shell, artifact identity | Subjective audio quality by itself |
| `owner-listening` | Character, safety, transition perception, real-world confidence | Broad mechanical regression coverage |
| `production-smoke` | Deployed landing, public links, current release artifacts, public metadata | Future availability or offline installed behavior |

### Ledger

| Date | Unit | Commit SHA | Platform / toolchain | Artifact / version | Command or procedure | Evidence layer | Result | Evidence location |
|---|---|---|---|---|---|---|---|---|
| 2026-07-24 | U1 | `6b5db20` | Windows 11 / Node 24 | source `0.9.0` | `npm test` | frontend-unit | PASS — 62 files, 564 tests. 8 new canon invariants; byte-identity and mobile-claim gates each verified to FAIL on a forced regression before acceptance. | `src/lib/release-readiness.test.ts` |
| 2026-07-24 | U2 | `fd25574` | Windows 11 / Node 24 | `package-lock.json` | `npm audit` | frontend-unit | PASS — **0 vulnerabilities**. Cleared `brace-expansion` GHSA-3jxr-9vmj-r5cp and `postcss` GHSA-r28c-9q8g-f849, both dev/build-path only, both by in-range transitive patch bump. Runtime dependency surface unchanged. | commit diff (`package-lock.json`, 3 entries) |
| 2026-07-24 | U2 | `fd25574` | Windows 11 / Node 24 | `dist/` | `npm run build` | frontend-unit | PASS — every emitted asset content-hash **identical** to the pre-update build, so the postcss bump provably changed nothing in the Vite/Tailwind output. | build output |
| 2026-07-24 | U2 | `fd25574` | Windows 11 / Node 24 | `package-lock.json` | `npm ci` then `npm install` | frontend-unit | PASS — fresh install reproduces the lock byte-identically and the follow-up install is idempotent. | — |
| 2026-07-24 | U3 | `39132a7` | Windows 11 / Playwright bundled Chromium **149.0.7827.55** | `dist/` from source `0.9.0` | `npm run verify:headless` (cold: no `dist/`, no server) | **browser-headless** | PASS — landing suite (12 viewports, 8 anchor checks) + 18 `/app` scenario-viewport checks across 11 named scenarios. Zero console errors/warnings. **Synthetic: proves no native dialog, audio, signing, updater, or listening behavior.** | `test-output/headless/<ts>/` |
| 2026-07-24 | U3 | `39132a7` | Windows 11 / Chrome channel 150.0.7871.186 **vs** bundled Chromium 149.0.7827.55 | same `dist/` | landing suite run under both runtimes | browser-headless | PASS — **identical**: 0 failures each, 12 viewports each, 8 anchor checks each, 0 per-viewport metric mismatches. This is the evidence behind the bundled-Chromium decision. | `test-output/parity-chrome/`, `test-output/parity-chromium/` |
| 2026-07-24 | U3 | `39132a7` | Windows 11 | — | `--force-fail`; then `PLAYWRIGHT_BROWSERS_PATH` pointed at an empty dir; then node-PID diff around both runs | browser-headless | PASS (negative controls) — forced assertion exits **1** naming scenario/route/viewport/screenshot; a missing browser exits **1** rather than skipping; node PID set is **identical** before and after on both the pass and fail paths, with no listener left on the port. | run logs |
| 2026-07-24 | U12 | `42b23cc` | Windows 11 / rustc stable | source `0.9.0` | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | native-synthetic | PASS — fmt and clippy clean; 417 lib tests + all integration binaries, 0 failures. **Preset byte-identity snapshots pass unchanged** — the mechanical proof that no DSP value moved. | `src-tauri/` |
| 2026-07-24 | U12 | `42b23cc` | Windows 11 / rustc stable | — | `cargo test --test evidence_runner_paths` | native-synthetic | PASS — 7 tests, **5 of them red before the fix**. The documented `--output ../test-output/...` invocation was rejected outright before this; renders could be written inside the private source dir. | `src-tauri/tests/evidence_runner_paths.rs` |
| 2026-07-24 | U12 | `42b23cc` | Windows 11 / rustc stable | — | iPhone bridge `cargo check --all-targets` + `cargo test`; Android bridge `cargo test` | native-synthetic | PASS — iPhone 46 tests, Android 26 host tests. Run because U12 touches the shared crate. | `apps/*/rust/` |
| 2026-07-25 | U10 | `e4b5117` | WSL2 Ubuntu / Node 24 | source `0.9.0` | `npm test` | frontend-unit | PASS — 66 files, 617 tests, 3 skipped. Includes 6 new one-owner-per-warning tests (**each verified RED against pre-fix source**), 11 named scenario cases (S-D1/S-F1/S-F3), 3 new receipt-action tests, 1 export failure→retry test. | `src/App.warning-ownership.test.tsx`, `src/App.scenarios.test.tsx` |
| 2026-07-25 | U10 | `e4b5117` | WSL2 Ubuntu / Playwright bundled Chromium **149.0.7827.55** | `dist/` from source `0.9.0` | `npm run verify:headless` | **browser-headless** | PASS — **19** scenario/viewport checks (was 18; +`S-E1-rapid-ab`). Zero console errors/warnings. `scenarioCoverage`: S-D1 ×2, S-E1 ×1, S-F1 ×5, S-F2 ×9, S-F3 ×2. **Synthetic: proves no native dialog, audio, signing, updater, or listening behavior.** | `test-output/headless/2026-07-25T18-37-28-416Z/` |
| 2026-07-25 | U10 | `e4b5117` | WSL2 Ubuntu / Chromium 149 | same `dist/` | reachability (`mustReach`) at 1360×740 and 1440×900 | browser-headless | PASS — 24 target checks over 10 scenario/viewport pairs; every one found, with a real layout box, inside the viewport, and topmost at its own centre. Per-target geometry recorded (album-12 @1360×740: Delivery Format 182px tall at top=278; Export Album 40px at top=350). **Negative controls:** an injected `::after{inset:0}` cover ⇒ 12 named failures; a non-matching selector ⇒ 10 "not found" failures. | `summary.json` → `scenarios[].reachability` |
| 2026-07-25 | U10 | `e4b5117` | WSL2 Ubuntu / Chromium 149 | same `dist/` | S-E1 rapid A/B — 6 alternating switches, Volume Match toggled mid-run | browser-headless | PASS — playhead **0.15 → 1.356 monotonic** across all six switches (no rewind), `aria-pressed="true"` on the requested side every time (no stalled direction), no stale "still preparing" error after. **Closing owner remains U15** — this cannot judge how the switch sounds. | `summary.json` → `scenarios[].driverPayload` |
| 2026-07-25 | U10 | `e4b5117` | Windows 11 / rustc 1.95.0, run against this WSL checkout | source `0.9.0` | `cargo test --lib` | native-synthetic | PASS — 418 passed, 0 failed, 2 ignored. **All 9 preset byte-identity snapshots pass unchanged** (`universal`, `clarity`, `warmth`, `punch`, `oomph`, `tape`, `spatial`, `loud`, `custom`) — the mechanical proof that a chunk of UI work moved no sound. | `src-tauri/` |
| 2026-07-25 | U10 | `e4b5117` | Windows 11 / rustc 1.95.0 | — | `cargo test --lib export_name_suggestion` | native-synthetic | PASS — S-F3's native half: first free of `name.wav` / `name-2.wav` / `-3`, extension preserved, extensionless names suffixed rather than clobbered. | `src-tauri/src/exports.rs` |

### Candidate freeze

U14 ends by tagging the release candidate at the exact commit whose evidence
this ledger records. **From that tag until U17 closes or the candidate is
rejected, `main` is frozen** — no commits land on `main`. Work discovered during
U15–U17 goes to `docs/OWNER_INPUT_QUEUE.md` and, if code is needed, onto a
branch that cannot feed the candidate. The freeze is a plan rule, not a
branch-protection change, so **the U14 ledger entry must announce it** or a
parallel agent session will not see it.

**Freeze status: NOT IN FORCE.** No candidate is tagged. Work on `main` normally.

---

## Decision

Fill in when the blocking boxes above are checked.

- Date / decider: __________________________
- Build (version + commit + release tag): __________________________
- Blocking items still open: __________________________
- Updater gates (§3) both cleared? YES / NO
- Listening note signed off (§5)? YES / NO
- **DECISION:** GO (owner announces) / NO-GO (blockers above)
