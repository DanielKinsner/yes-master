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

- [x] **Beta version 0.9.1** across the three desktop manifests (bumped from
      0.9.0 by U14 — 0.9.0 is now only the updater seed U16 proves the update
      path from); Windows publisher = "Daniel Kinsner". *Evidence:* Slice 1 for
      the original mechanism; U14 commit `2955a8a` + `version-coherence` tests
      for the 0.9.1 bump; `npm run build:windows` produces `YES Master_0.9.1`.
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
| 2026-07-25 | U11 | `1d5ffaf` | WSL2 Ubuntu / Node 24 | source `0.9.0` | `npm test`, `npm run build` | frontend-unit | PASS — 67 files, 626 tests, 3 skipped; build clean. 9 new delight-constraint tests. **Three guards verified by injection:** deleting a reduced-motion opt-out, adding `height` to a keyframe, and removing the verdict re-key each turn their test red. | `src/App.delight.test.tsx` |
| 2026-07-25 | U11 | `1d5ffaf` | WSL2 Ubuntu / Playwright bundled Chromium 149.0.7827.55 | `dist/` from source `0.9.0` | `npm run verify:headless` | **browser-headless** | PASS — **24** scenario/viewport checks (19 + 5 reduced-motion). Reduced-motion passes run the SAME assertions with `reducedMotion: "reduce"`, so a state that only reads correctly with motion enabled fails. Motion-end and reduced-motion screenshots both written. | `test-output/headless/2026-07-25T18-49-10-282Z/` |
| 2026-07-25 | U11 | `1d5ffaf` | Windows 11 / rustc 1.95.0, run against this WSL checkout | source `0.9.0` | `cargo test --lib` | native-synthetic | PASS — 418 passed, 0 failed, 2 ignored. **All 9 preset byte-identity snapshots pass unchanged** — re-checked after U11 because the rule is to check it every time, not only when the change looks DSP-adjacent. | `src-tauri/` |

| 2026-07-25 | U11 | `e2440b1` | WSL2 Ubuntu / Chromium 149 | `dist/` from source `0.9.0` | **visual pass** — read the 1360×740 and 1440×900 screenshots by eye, then re-render after fixing | browser-headless | **2 DEFECTS FOUND AND FIXED** (`0c69f67`), both invisible to every automated assertion on that viewport: the workspace title clamp was gated behind `min-height: 820px` so it was OFF at the supported 740px-tall minimum (long filename wrapped to 8 lines, pushing transport/waveform off-screen); and the sticky export mask was transparent where the TOOLS row sits, so Delivery Format scrolled through it. Re-render confirms both clean. 2 regression tests; `npm test` 628 passed. | `test-output/headless/2026-07-25T19-20-23-046Z/` |

| 2026-07-25 | U4 | `85d35f6` | WSL2 Ubuntu / Node 22, **vitest 4.1.9** | source `0.9.0` | `npm test`, `npm run build`, `npm run verify:headless` | frontend-unit + browser-headless | PASS — 68 files, 651 tests, 3 skipped; build clean; headless 24/24. 23 new tests parse both issue forms as **real YAML**. Negative-controlled: demoting the privacy block, dropping a required flag, and adding Linux to the OS list each turn a test red. | `src/lib/beta-feedback-contract.test.ts` |
| 2026-07-25 | U4 | `85d35f6` | WSL2 Ubuntu / Node 22 | `package-lock.json`, `dist/` | `npm audit`; build-output comparison with and without the new devDependency | frontend-unit | PASS — `yaml` 2.9.0 added as a **dev**Dependency (zero own dependencies). `npm audit` reports **0 vulnerabilities**; runtime dependency surface untouched; `dist/assets` **identical** with and without it, proven by building both ways rather than assumed from "it's dev-only". | commit diff |

> **⚠ Toolchain correction (2026-07-25).** The `npm ci` run during the check
> above replaced a stale installed **vitest 2.1.9** with the **4.1.9** that
> `package.json` and the lockfile have both specified throughout. Every green
> frontend run recorded earlier in this session executed on 2.1.9. The suite
> passes on 4.1.9 (68 files / 651 tests) and the headless lane is green, so
> nothing was concealed — but rows dated before this one were produced on a
> runner that did not match the lockfile, and that is the kind of thing this
> ledger exists to say out loud.

| 2026-07-25 | U5 | `a8f79de` | WSL2 Ubuntu / Node 22, vitest 4.1.9 | source `0.9.0` | `npm test`, `npm run build` | frontend-unit | PASS — 68 files, 680 tests, 3 skipped; build clean. **29 new tests**: 19 rendering (`BetaDownload.test.tsx`) + 10 state-model (`release-readiness.test.ts`). Both drive the real resolver rather than hand-built objects, so tests and resolver cannot drift apart while both stay green. | `src/landing/BetaDownload.test.tsx`, `src/lib/release-readiness.test.ts` |
| 2026-07-25 | U5 | `a8f79de` | WSL2 Ubuntu / Playwright bundled Chromium 149.0.7827.55 | `dist/` from source `0.9.0` | `npm run verify:headless` | **browser-headless** | PASS — landing suite (12 viewports) now also asserts, on the **real built page**, that no anchor targets `/releases/latest`, that a closed release state renders zero download actions plus one inactive action, and that its reason is *visibly painted* and `aria-describedby`-associated. App suite 24/24 unchanged. **This lane observes S-A1 and S-I1 only** — recorded in `summary.json.scenarioCoverage`, because S-A2 needs a verified release and S-B1 a real draft. | `test-output/headless/2026-07-25T20-19-10-910Z/` |
| 2026-07-25 | U5 | `a8f79de` | WSL2 Ubuntu / Node 22 + Chromium 149 | — | **negative controls** (3) | frontend-unit + browser-headless | PASS — (1) disabling the beta-end-date gate turns `rejects an active state without the beta end date` **red**; (2) re-adding a `/releases/latest` anchor turns **7** rendering tests red **and** fails the landing lane at **all 12 viewports** naming the href; (3) the state-model suite opens with a control proving the valid fixture **does** activate — without it every rejection test would pass vacuously. | run logs |
| 2026-07-25 | U5 | `a8f79de` | WSL2 Ubuntu | `dist/` | bundle inspection | frontend-unit | PASS — the test fixture (`release-fixture.ts`) is **absent from `dist/`**, and the only remaining `/releases/latest` occurrences in the bundle are two internal diagnostic strings, never an `href`. Checked by grepping the emitted JS rather than assuming tree-shaking. | `dist/assets/LandingPage-*.js` |
| 2026-07-25 | U6 | `b674c76` | WSL2 Ubuntu / Node 22, vitest 4.1.9 | source `0.9.0` | `npm test`, `npm run build` | frontend-unit | PASS — 69 files, 697 tests, 3 skipped; build clean. 17 new copy invariants rendered via `renderToStaticMarkup`, so every claim asserted is content a visitor receives with **no JavaScript and no animation** — the "essential content understandable when animations do not run" half of S-A1/S-A2/S-I1. | `src/landing/LandingCopy.test.tsx` |
| 2026-07-25 | U6 | `b674c76` | WSL2 Ubuntu / Node 22 | — | **negative controls** (4) | frontend-unit | PASS — restoring "coming after launch", moving `CrossPlatform` back to second, restoring "beta testers keep $29 forever", and restoring "true-peak safe, every time" turn **6** tests red between them, including the positional hierarchy and audience-order assertions. | run logs |
| 2026-07-25 | U6 | `b674c76` | WSL2 Ubuntu / Playwright bundled Chromium 149.0.7827.55 | `dist/` from source `0.9.0` | `npm run verify:headless` | **browser-headless** | PASS — landing suite green at all 12 viewports against the rebuilt page (9 required sections, `requiresNavAnchor` added so `#mobile` can exist without a nav slot); app suite 24/24 unchanged. | `test-output/headless/2026-07-25T20-36-42-426Z/` |
| 2026-07-25 | U6 | `590298d` | WSL2 Ubuntu / Chromium 149 | `dist/` | **visual pass** — read the rebuilt sections at 1440×900 and 390×844 by eye | browser-headless | **3 DEFECTS FOUND AND FIXED**, none of which any assertion caught: the newsletter button read "Email updates opening **soon**" (banned roadmap word, promising a schedule for a feature with no provider selected); it kept the full CTA gradient while disabled, directly beneath a correctly-inert download button; and its success state promised to "save your founder price" — the owner-blocked C-22 entitlement — **pinned by a passing test**. ⚠ **Evidence correction:** the first capture passed `newPage({ viewportSize })`, which Playwright ignores, so both the "desktop" and "phone" runs were 1280×720 — a phone layout I had not actually seen. Re-captured at a verified 390×844 before accepting. | scratchpad captures |
| 2026-07-25 | U7 | `5877c54` | WSL2 Ubuntu / Node 22, vitest 4.1.9 | source `0.9.0` | `npm test`, `npm run verify:landing-assets` | frontend-unit | PASS — 70 files, 704 tests, 3 skipped. The asset gate is offline and browserless: it compares committed bytes to a committed manifest. 3 captures bound to capture-input digest `abd3daa1bd74…`. | `src/assets/landing/manifest.json`, `src/lib/landing-assets.test.ts` |
| 2026-07-25 | U7 | `5877c54` | WSL2 Ubuntu / Playwright bundled Chromium 149.0.7827.55 | `dist/` from source `0.9.0` | `npm run capture:landing` | **browser-headless** | PASS — Standard, Advanced and Album-4 captured at 1440×1000 from a production build, each settled on the **playhead** (`[role="slider"][aria-valuenow]`, ~6s) rather than the "READY" text, so no capture shows an empty waveform. **Synthetic: proves layout only — nothing about installation, real audio, or how anything sounds.** | `src/assets/landing/*.png` |
| 2026-07-25 | U7 | `5877c54` | WSL2 Ubuntu | `src/assets/landing/` | eager-imagery budget | frontend-unit | PASS — **330 KB of 1465 KB**, down from **2530 KB**. Captures lazy with intrinsic sizes; icons 400px→168px (264 KB→41 KB); hero 1382 KB→286 KB plus a 102 KB 1280w variant. Budget was **genuinely violated before this unit** (2530 KB) and the gate reported it rather than being tuned to pass. | verify output |
| 2026-07-25 | U7 | `5877c54` | WSL2 Ubuntu | — | **negative controls** (4) | frontend-unit | PASS — (1) appending to `src/App.css` ⇒ FAIL naming the digest change; (2) one byte appended to a capture ⇒ FAIL naming hash and size; (3) **an unrelated `docs/TESTING.md` edit ⇒ still PASSES**, which is the acceptance criterion that stops the gate becoming noise; (4) budget lowered to 200 KB ⇒ FAIL naming the exact overage. | run logs |
| 2026-07-25 | U8 | `d024bcb` | WSL2 Ubuntu / Playwright bundled Chromium 149.0.7827.55, axe-core 4.12.1 | `dist/` from source `0.9.0` | `npm run verify:headless` | **browser-headless** | PASS — **13 viewports** (320×568 added for S-A3). axe-core at 1440×900 and 390×844: **28 passes, 0 violations, 0 serious/critical**. Keyboard: first Tab stop is the skip link, visible with a focus indicator, CTA reachable. **200% zoom: 0 horizontal overflow** (1440/1440), CTA on screen. Reduced-motion pass carries every load-bearing sentence. **Synthetic: axe is static analysis and proves nothing a screen reader would tell you.** | `test-output/headless/2026-07-25T21-20-09-092Z/landing/summary.json` |
| 2026-07-25 | U8 | `d024bcb` | WSL2 Ubuntu / Node 22, vitest 4.1.9 | source `0.9.0` | `npm test` | frontend-unit | PASS — 71 files, 709 tests, 3 skipped. 5 browserless head-metadata tests, including indexability tied to release state so it fails in **both** directions. | `src/landing/LandingMeta.test.ts` |
| 2026-07-25 | U8 | `d024bcb` | WSL2 Ubuntu / Chromium 149 | `dist/` | **negative controls** (3) | browser-headless | PASS — removing an image `alt` ⇒ axe **critical** `image-alt` at both scanned widths; removing the `noindex` meta ⇒ FAIL naming the unavailable release state; removing the skip link ⇒ FAIL at every viewport. | run logs |

> **Scope note on the 2026-07-25 rows above.** The U10/U11 lanes were run on the
> tree at `1d5ffaf`; `docs(beta): C2 complete` is documentation only and touches
> no source, test, or script. The visual-pass row and its fixes are at
> `0c69f67`/`e2440b1`, after it. Stated rather than left to inference, per this
> ledger's own rule that a green result from a different tree is not evidence.

| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / Node 24.15.0, vitest 4.1.9, fresh `npm ci` | source `0.9.1` | `npm test`; `npm audit` | frontend-unit | PASS — **72 files, 712 tests, 0 failed**; audit **0 vulnerabilities**. **First real-Windows run of the frontend suite in this program, and it found a defect the WSL2/macOS runs could not:** `detectPlatform(undefined)` triggered the default parameter and fell back to the real jsdom navigator, whose UA embeds the host OS — the assertion passed on Linux and inverted on Windows. Fixed (`7b8fc84`) to assert the intended contract (`{}` → `other`) host-independently. | `src/lib/release-readiness.test.ts` |
| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / rustc 1.95.0, native NTFS checkout | source `0.9.1` | `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test --lib`; full `cargo test` **with `AMS_RUN_REAL_FIXTURE=1`** | native-synthetic **+ private-fixture** | PASS — fmt and clippy clean; lib 418 passed / 2 ignored; full integration **36 binaries, 588 passed, 0 failed, 7 ignored** (all `#[ignore]`-marked long-runners). **Zero "Skipping real-fixture" lines** — the slow lane genuinely ran against `private-audio-fixtures/` on this machine. All 9 preset byte-identity snapshots pass unchanged — no sound moved. fmt/clippy/lib ran at `f4fd637`; the `f4fd637..b00c4c8` diff is `.mjs` scripts and PNGs only (no Rust input), the full suite ran at `b00c4c8`. | `src-tauri/`, run log |
| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / rustc 1.95.0 | new `tests/track_master_e2e.rs` | `cargo test --test track_master_e2e` | native-synthetic | PASS — the U14 synthetic-WAV flow: import probe → analysis (live progress) → waveform peaks → render → re-analysis of the rendered file → receipt + advisory checks from `RenderedMeasurements` → **collision-safe second export** (`__1` divert, first render byte-identical after) → project save → **reload** (settings/selection/view survive; reloaded path re-analyzes to the same LUFS), plus a mono variant. **Negative-controlled:** expecting `__2` instead of `__1` turns the flow test red. Receipt LUFS agrees with an independent measurement of the written file within 0.5 LU. | `src-tauri/tests/track_master_e2e.rs` |
| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / rustc 1.95.0; cargo-ndk 4.1.2, NDK r27.2, `--platform 29` | bridge crates | iPhone: `cargo check --all-targets` + `cargo test`; Android: `cargo test` + `cargo ndk -t arm64-v8a --platform 29 check` | native-synthetic | PASS — iPhone check clean, **46 passed / 1 ignored**; Android **26 passed**, arm64 NDK check clean. Run because U14's version bump touches the shared crate's `Cargo.toml`/locks. | `apps/iphone-native/rust/`, `apps/android-native/rust/` |
| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / Playwright bundled Chromium (lockfile-pinned) | `dist/` from source `0.9.1` | `npm run verify:headless` | browser-headless | PASS — landing suite (13 viewports, axe, keyboard, zoom, reduced-motion) + **24 `/app` scenario/viewport checks**. `scenarioCoverage` names S-D1, S-E1, S-F1, S-F2, S-F3 instances. Also run cold at session start against `fcf89a8` (U8 re-verification per the resume protocol) — green both times. | `test-output/headless/2026-07-27T15-44-52-678Z/` |
| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / Node 24 | `src/assets/landing/` | `npm run capture:landing`; `npm run verify:landing-assets` | browser-headless + frontend-unit | PASS — the version bump changed `preview-mock.ts` (a listed capture input) and **the U7 gate correctly failed the old captures**; three fresh 1440×1000 captures bound to digest `77e28589b93a…`, eager imagery unchanged at 330 KB, 7/7 asset tests green. Provenance note: the manifest's `sourceCommit` string reads `f4fd637` because the capture ran from a dirty tree before `b00c4c8` existed; freshness is decided by the content digest, which matches the committed inputs. **Windows tooling defects fixed first (`f315105`):** `URL#pathname` mangled a checkout path containing a space into `C:\C:\…%20…`; bare `npm`/`npx` spawns ENOENT on Windows; `process.kill(-pid)` silently orphaned the preview server; `vite preview` bound `::1` while the probe polled `127.0.0.1`. Every prior run of this tooling was WSL2/macOS, which is why all four survived U7. | `src/assets/landing/manifest.json` |
| 2026-07-27 | U14 | `b00c4c8` | Windows 11 / rustc 1.95.0 release profile | **`YES Master_0.9.1_x64_en-US.msi`** (9,957,376 bytes) · **`YES Master_0.9.1_x64-setup.exe`** (7,916,332 bytes) | `npm run build:windows` (runs `tsc -b` + `vite build` via `beforeBuildCommand`, then `tauri build --bundles msi,nsis`) | installed-machine (artifact identity only) | PASS — both bundles built from the `b00c4c8` tree. SHA-256: msi `b9a73739992eac1d77b847889d3ded964bc8e049d0ca2ebf8fc7f84412c23a24`, nsis-setup `553989b9ec69a774a8d2b71b2e113f745aa2f336f100e509e28b4d1fdc3141e1`. **Draft artifacts, unsigned, NOT published** — updater signatures and publication are U16's transaction. These are the bytes U15 installs; installing and running them proves nothing yet (that is U15's job). | `src-tauri/target/release/bundle/` (local only, not committed) |
| 2026-07-27 | U14 | tip pending push | GitHub Actions | — | remote CI (snapshot-diagnostics, web-e2e, windows, macos, android) | — | **OPEN — the one U14 mechanical gate that cannot run locally.** Requires the owner-authorized push of the U14 commits; check the boxes in §1 only after the tip run completes green. | GitHub Actions |

### Candidate freeze

U14 ends by tagging the release candidate at the exact commit whose evidence
this ledger records. **From that tag until U17 closes or the candidate is
rejected, `main` is frozen** — no commits land on `main`. Work discovered during
U15–U17 goes to `docs/OWNER_INPUT_QUEUE.md` and, if code is needed, onto a
branch that cannot feed the candidate. The freeze is a plan rule, not a
branch-protection change, so **the U14 ledger entry must announce it** or a
parallel agent session will not see it.

**Freeze status: DECLARED 2026-07-27 (U14 close-out).** Every local mechanical
lane is green (rows above). `main` is frozen as of the U14 close-out docs
commit: **no further commits land on `main`** except the owner-authorized
candidate tag `v0.9.1-beta.1`, which goes on the close-out commit itself. Two
owner actions complete C4: (1) authorize the push of the U14 commits so the
remote CI row above can go green at the tip, and (2) authorize the tag. The
commits after evidence commit `b00c4c8` are docs-only (`git diff b00c4c8..HEAD
--stat` shows only `docs/`), which is why the mechanical evidence remains valid
at the tag point per this ledger's commit-independence clause. A parallel agent
session seeing this line: do not commit to `main`; stage work on a branch and
queue it in `docs/OWNER_INPUT_QUEUE.md`.

---

## Decision

Fill in when the blocking boxes above are checked.

- Date / decider: __________________________
- Build (version + commit + release tag): __________________________
- Blocking items still open: __________________________
- Updater gates (§3) both cleared? YES / NO
- Listening note signed off (§5)? YES / NO
- **DECISION:** GO (owner announces) / NO-GO (blockers above)
