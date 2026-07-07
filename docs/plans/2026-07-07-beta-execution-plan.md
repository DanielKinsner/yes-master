# YES Master — Beta Launch Execution Plan (2026-07-07)

> **Decision-complete execution plan.** Grilled and locked with the owner on
> 2026-07-07 (Fable 5 session). This plan is written so a cheaper model can
> execute it WITHOUT making strategic decisions — every judgment call below is
> already made. Strategy source: `docs/plans/2026-06-30-launch-plan.md` (still
> authoritative for GTM/pricing). This plan sequences the remaining work from
> "repo today" to "public free beta live".
>
> **If something here contradicts current code/docs reality, verify reality
> first (drift check), then follow reality and note the discrepancy — do not
> silently improvise a new decision.**

## Decisions locked 2026-07-07 (sweep these into OPEN_THREADS — Slice 0)

| # | Decision |
|---|---|
| D1 | Beta = **free public beta, Mac + Windows together** (owner has an M4 MacBook Pro for build confirmation + listening). Not Windows-first. |
| D2 | Beta timebox: **~8 weeks**, stated as a concrete flip date on the landing page once the beta ships. Beta users keep the $29 founder price. |
| D3 | Owner accounts (Apple Developer, Azure Trusted Signing, email provider, GitHub Actions billing fix) start at **payday (~2026-07-10)**. All engineering that needs secrets is payday-gated; everything else proceeds now. |
| D4 | **Email provider decided at payday** (shortlist per `src/landing/signup-config.ts` comments; Buttondown recommended). Signup form stays safe-disabled until wired. |
| D5 | Download flow: **ungated download button + optional email signup beside it** ("get the founder discount — join the list"). NOT email-gated. Closes launch-plan §5 ambiguity. |
| D6 | Legal (EULA/privacy/refund drafts in `docs/legal/`): **not a beta gate.** Owner researches independently (industry friends). Never stalls engineering. Lawyer pass is a paid-flip item at most. |
| D7 | Listening: **one minimal owner sitting** (scripted runbook, ~60–90 min) covers everything beta-blocking. **AC-5, Phase-B confidence gating, and the album character system ship gated OFF in beta** and are calibrated post-beta, informed by beta feedback. |
| D8 | Canon edits **A–G approved** (see Slice 2). Closes OPEN_THREADS #8 and Part B Q7, Q9, Q10, Q11, Q12, Q13. |
| D9 | Mobile promise (canon wording, owner's words): *phones go live when the owner judges them ready; they are Standard mode on phones by design and never fully mimic desktop.* Matches what is built (4 presets, 3 loudness levels, fixed safe export). |
| D10 | UX fixes **Q24 (per-track view memory), Q25 (album subfolder), Q29 (autosave at analysis-complete)** are in beta scope. |
| D11 | Decided as technical calls: export-during-export stays a hard block **+ tooltip** (Q27); device-loss threshold stays **2 s** (Q30); min-window-size is **documented, not a layout slice** (Q16); width slider keeps the shipped honest-Auto treatment (Q23); stereo_width disposition stays Wave 10 (Q18); parked stays parked — P2 one-pole/soft-knee hoist + P4 tauri-specta (Q19). |
| D12 | macOS ships as a **universal binary**; an Intel-Mac smoke test via the owner's friends is **non-blocking** (the tolerance golden already removed Intel test risk). |
| D13 | Beta version number: **0.9.0** (numeric for MSI compatibility; 1.0.0 reserved for the paid flip). |
| D14 | Premium-parity UI pass is in beta scope (owner's open-floor concern: some surfaces got the premium pass, others feel deferred). |
| D15 | Owner devices for verification: M4 MacBook Pro (macOS), current Windows box, iPhone 16 (mobile stays parked regardless). No Android device — Android work stays parked. |

## Lanes

Two lanes run in parallel. **Opus lane** = code/docs work executable now.
**Owner lane** = things only Dan can do, mostly payday-gated.

### Owner lane (Dan)

| When | Task |
|---|---|
| Payday (~07-10) | Enroll Apple Developer ($99/yr) — identity verification has lead time, start first. |
| Payday | Set up Azure Trusted Signing (~$10/mo). |
| Payday | Fix GitHub Actions billing (macOS runners for `release.yml`). |
| Payday | Pick + create the email provider account; hand the endpoint/API key to the agent. |
| Payday | Add signing secrets to the repo per `docs/RELEASE_SIGNING_SETUP.md`. |
| After runbook exists | Execute the one-sitting listening runbook (Slice 8) on Windows + the M4; write the listening note. |
| First signed build | Install + run the macOS build on the M4 for real (closes the RC criterion). Confirm real-time snappiness on both machines. |
| Anytime | Legal research (non-blocking). 2–3 royalty-clear before/after snippets for the landing A/B hero. Begin genuine KVR/community participation (2–3 wks before any promo). |
| Non-blocking | Friend-run Intel Mac smoke test (install, import, master, export). |

### Opus lane — ordered slices

Commit small (per CLAUDE.md). Run the fast verify lane per touched surface;
slow fixture lane only for DSP/export-touching slices (none below should touch
DSP). **Do-not-touch list is at the bottom and is absolute.**

---

#### Slice 0 — Ledger sweep (docs only)
Update `docs/OPEN_THREADS_AND_DECISIONS.md`: record D1–D15 above with a
2026-07-07 dated block; mark closed: #8/S5.4, Part B Q7, Q9, Q10, Q11, Q12,
Q13, Q16, Q18 (deferred-confirmed), Q19, Q23, Q24 (approved→in progress), Q25,
Q27, Q29, Q30. Note D7 keeps Q3/Q4/Q6/7a parked (post-beta). Reference this
plan file.

#### Slice 1 — Version + metadata
- Bump version to **0.9.0** everywhere Tauri needs it (`tauri.conf.json`,
  `Cargo.toml`, `package.json` — follow the repo's actual wiring).
- Set real Windows publisher metadata (publisher name = owner's chosen
  publisher string; ask ONLY if truly absent from repo/launch docs).
- Verify: `npm run build`, `npm run build:windows` still green.

#### Slice 2 — Canon edits A–G (docs only, pre-approved; one commit per file is fine)
Source of wording: launch plan §2/§3 + D9. Exact edits:
- **A** `docs/PRODUCT.md` intro: replace the "private-solid… before considering
  broader distribution" bar with: public free beta → paid 1.0; good enough to
  charge for.
- **B** `docs/PRODUCT.md`: add a "Distribution & Business Model" section —
  free time-boxed beta (~8 wks, concrete flip date announced at beta launch),
  single SKU $29 founder → $49 one-time perpetual license, permanent
  export-locked demo (full chain + receipt visible; render/export gated), sold
  direct via Lemon Squeezy (MoR), signed installers on GitHub Releases, Tauri
  updater.
- **C** `docs/PRODUCT.md` Mobile Companions: replace the "pending owner
  definition" block with D9's wording + deliberate absences (no album
  mastering, no advanced controls, no custom delivery formats on phones).
- **D** `docs/PRODUCT.md` Public Surface: landing page is a **supported product
  surface** — marketing + ungated download hub + optional email capture, later
  checkout. Closes "role pending".
- **E** `docs/PRODUCT.md` Deferred list: REMOVE "Public code signing/
  notarization" and "Autoupdate" (now launch-blocking); add a pointer to this
  plan. Same removal in `docs/RELEASE_STABILIZATION.md` Deferred section.
- **F** `docs/PRODUCT.md` Release-Candidate Meaning: add (i) macOS packaging
  criterion parallel to the Windows line, (ii) "installers are signed by the
  release pipeline".
- **G** `CLAUDE.md` **and byte-identical** `AGENTS.md`: broaden the first
  non-negotiable to acknowledge iPhone/Android bridges + landing page as real,
  CI-tested surfaces (desktop still ships first); add
  `docs/plans/2026-06-30-launch-plan.md` and THIS plan to Required Reading;
  landing page confirmed in agent scope for launch work.
- Also: `README.md` consistency pass against the above (it already leads with
  performance — verify only, minimal diffs).

#### Slice 3 — UX fix: per-track view memory (Q24)
Spec (from OPEN_THREADS B24): add `view_by_track_id` to ProjectState; the
force-bounce to Advanced for dirty tracks still happens but STOPS overwriting
the remembered view; only explicit user view choices persist.
- Frontend tests for: remembered view restored on track switch; force-bounce
  does not overwrite; persists through save/open.
- Verify: `npm test`, integration tests green.

#### Slice 4 — UX fix: album export subfolder (Q25, option ii)
Album exports render into an `<AlbumTitle>/` subfolder of the chosen export
dir (sanitize the title for filesystem safety; empty title falls back to
`Album`). Continuous render + manifest live in the same subfolder. Receipts/
manifest paths updated. Never overwrite existing dirs — existing "never
overwrite by default" rules apply.
- Rust tests for path construction incl. empty-title and illegal-char cases.

#### Slice 5 — UX fix: autosave at analysis-complete (Q29) + Q27 tooltip
- Fire an explicit autosave when analysis completes (bypasses the 1.5 s
  debounce; owner-measured 7–10 s anxiety window disappears).
- Add a plain tooltip to the disabled export control during an active export
  ("An export is already running — it finishes or fails before the next one
  starts.").
- Frontend tests for both.

#### Slice 6 — Min-window resolution (Q16; decided 2026-07-08 after drift finding)
Drift finding (Slice 0–2 checkpoint): the shipped min-window is 1440×860,
which cannot fit a 1366×768 laptop — the original "just document it" call
(D11) was made against a stale assumption. Owner-decided resolution:
**experiment, verify, fallback**:
1. Lower the Tauri min-size to ~1360×740 and verify at that size: layout-css
   tests + screenshots of Standard, Advanced, and the export/review surfaces.
2. If nothing visibly breaks → keep it; document "works on 1366×768" in
   `docs/APP_BEHAVIOR.md`.
3. If layout genuinely breaks → revert the min-size, document "requires
   1440×900 or larger" as the honest beta requirement, and add a post-beta
   small-screen layout item to the ledger.
No speculative layout rework either way. Judgment call on "visibly breaks"
(overlap/clipping/unusable controls = breaks; merely snug = fine) is the
executor's.

#### Slice 7 — Tauri updater integration
- Integrate the Tauri updater (currently absent), manifest pulled from GitHub
  Releases per the drafted `release.yml`.
- Generate the updater signing keypair; **private key goes to the owner (never
  committed)** — document handling in `docs/RELEASE_SIGNING_SETUP.md`.
- Updater must fail silent-and-graceful offline (matches local-first promise).
- This slice needs NO paid accounts — it can land before payday.

#### Slice 7b — Updater user-visible surface (owner-decided 2026-07-08)
Slice 7 shipped check-and-emit only (`updater:available`) with no frontend
listener — the updater is invisible to users. Owner decision: **toast +
one-click install**.
- Frontend listens for `updater:available`; shows a non-blocking toast
  ("Update available — vX.Y") using the existing toast surface, dismissible.
- Clicking the toast's action ("Restart to update") invokes download +
  install + relaunch (Tauri updater's standard flow) via a Rust command.
- **Never interrupts work in progress:** while an export/render is running,
  the action is disabled with the same in-progress affordance as the export
  controls; it re-enables when the export finishes. Audio playback does not
  block install (the relaunch is user-initiated), but the app must not
  auto-relaunch on its own — install only ever fires from the click.
- Failed/offline download: log + toast dismisses gracefully; never a modal.
- Tests: toast renders on the event; action disabled during export; command
  wiring. Verify: frontend lane; Rust lanes if `lib.rs`/commands change
  (bridge lanes only if shared types/signatures change — floor-not-ceiling
  rule applies).

#### Slice 8 — Listening runbook (docs only)
Write `docs/plans/2026-07-08-beta-listening-runbook.md` (shipped): a scripted ~60–90 min
single sitting covering ONLY beta-blocking listening — normal source,
already-mastered source, long source (25-min seek/timeout case), preset
distinctness post-85%-lean (Universal/Clarity/Tape/Oomph at matched loudness,
closes threads #7 and the Manual Listening Gate), 8 kHz + 11.025 kHz Nyquist
sources, live-control sweeps during playback, Volume Match on/off, one clean
+ one warning export compared by ear, macOS snappiness note. Each step:
what to play, what to listen for, pass/fail line to fill in. Output = the
listening note that closes the gate.

#### Slice 9 — Premium-parity UI pass (D14)
Owner's concern: *"some things look like they had a premium pass and others
feel deferred."* Method (screenshot-driven, no redesign):
1. Inventory every user-visible surface (Standard, Advanced, dialogs, toasts,
   export review, album panel, settings/help, empty states, error states).
2. Grade each against the app's own best-in-class surfaces (the premium ones
   define the bar — do not invent a new style).
3. Fix the laggards: spacing/typography/motion/state-styling consistency,
   using existing tokens (`App.css` custom properties). NO new design
   language, NO `.std-tile` consolidation (intentional drift, stays).
4. Produce before/after screenshots for the owner's eye; owner-taste calls get
   flagged, not guessed.
- Verify: layout-css tests + full frontend lane; visual A/B set for owner.

#### Slice 10 — [PAYDAY-GATED] Release pipeline activation
After owner adds secrets + fixes CI billing:
- Activate `release.yml`: tagged release → build, sign (Azure Trusted
  Signing + Apple Developer ID + notarization), publish installers + updater
  manifest to GitHub Releases. macOS artifact = universal binary (D12).
- Cut `v0.9.0-beta.1` tag → first signed release (can be a private/draft
  release for owner install-testing).
- Owner then does the real-machine macOS confirm + Windows install check.

#### Slice 11 — [PAYDAY-GATED] Landing page launch wiring
- Wire the Download button to the GitHub Releases artifacts (ungated, D5) —
  replace the current `mailto:`.
- Wire the signup form to the chosen provider endpoint (D4); re-run the
  hardening plan Workstream F checklist against it before enabling (pinned by
  `src/landing/BetaSignup.test.tsx`).
- Add the beta model + concrete flip date + founder-price promise copy
  (launch plan §5 item 1, D2).
- Run `npm run verify:landing`.

#### Slice 12 — Beta go/no-go checklist (docs only)
Assemble `docs/plans/beta-go-no-go.md`: listening note exists; macOS + Windows
real-machine installs confirmed; signed artifacts downloadable; landing copy
live with flip date; signup capture verified end-to-end; legal drafts bundled
as-is (D6); owner GTM checklist (KVR post etc.) is owner lane. Beta announce
is an OWNER action — the agent never publishes/announces.
Two updater gate items (added 2026-07-08): (i) **replace the bootstrap
updater pubkey** with the owner's real keypair BEFORE the first signed
release (per `docs/RELEASE_SIGNING_SETUP.md` — shipped apps can never accept
updates signed by a different key); (ii) **prove the update path end-to-end**
once: install a 0.9.0 build, publish a draft 0.9.1 release, confirm the
toast → install → relaunch flow works on a real machine.

---

## Do-not-touch list (absolute)

- Any DSP constant, preset calibration, or `TBD-CALIBRATION` value.
- The gated-OFF systems: Adaptive Compressor (AC-5), Phase-B confidence
  gating, album character system — they stay OFF; the tripwire tests
  (`src-tauri/tests/owner_gates_default.rs`) must stay green.
- Export LUFS landing semantics, compressor-mode semantics, limiter behavior.
- Preset byte-identity snapshots (nothing here should touch DSP; if a slice
  unexpectedly does, STOP and flag).
- `.std-tile` / `.tile` consolidation (intentional drift).
- Parked refactors: P2 one-pole/soft-knee hoist, P4 tauri-specta.
- Private audio / rendered masters: never committed.

## Stop-and-ask-owner triggers

Stop and ask ONLY for: anything that changes sound; anything that publishes
publicly (push to public remote, publishing a non-draft release, enabling the
signup form live, posting announcements); pricing/flip-date copy deviations
from D2; legal doc content beyond mechanical fill-in. Everything else in this
plan is pre-decided — execute.

## Post-beta parking lot (explicitly NOT this plan)

Paid flip work (export gate + Lemon Squeezy activation — same seam in
`src-tauri/src/engine.rs` `render_track_master`/`render_album_plan`, build
together), AC-5/Phase-B/album-character calibration sittings, Tier-1/Tier-2
adaptive follow-ons, mobile revival, Microsoft Store, Advanced/Studio tier.
