# Phase A Codex Review: Launch-Readiness Remediation

**Date:** 2026-08-31
**Branch:** `codex/launch-readiness-remediation`
**Reviewed range:** `b81d820^..691392d` (`738522f9787c515d78565d84b216c11f56d8d85e` -> `691392d60f7f9fe68edef5ef3f3efbce7d8dc501`)
**Size:** 15 commits, 55 files, 2,735 insertions, 315 deletions
**Mode:** markdown report-only for product code; this review document is the sole authorized repository change
**Disposition:** **HOLD** before the owner's Task 12 decision

The review used the remediation plan, Claude's verified triage and amendments, and `CLAUDE.md` as the controlling contract. The triage amendments were applied where they override the plan, including Task 1's parser-boundary mechanism, Task 7's reachable-but-scrolling geometry contract, and Task 16's initializer correction. Task 16 is outside this Phase A range.

Task 0 has no commit by design. Its baseline conditions remain intact: the local and remote `v0.9.2-beta.1` tag both resolve to `c750da679929bcdd64e6030f890283f40d939181`; draft release `379883047` remains draft, unpublished, and targeted at that SHA. This review did not mutate either object. No preset or DSP implementation was retuned.

## Code Review Results

**Intent:** Verify every Phase A commit for genuine red-before-fix evidence, exact plan/triage compliance, and scope discipline; rerun the required current-tree lanes; assess launch and visual readiness without applying product fixes.

**Reviewers:** correctness, testing, security, reliability, maintainability, project standards, frontend races, local adversarial fallback, root synthesis, and one fresh finding validator.

- Security was included because the range changes the native opener, dependency locks, RustSec CI behavior, and release records.
- Frontend-race review was included because updater listener/query ordering and modal focus restoration are asynchronous.
- The independent cross-model Claude route was attempted, but its worker exited before review because `jq` was unavailable. No cross-model result was treated as evidence; a local adversarial fallback covered that lens.
- The validator independently re-read all seven submitted candidates. It validated all seven; the final eight findings split the validated contrast candidate into its implementation and scan-timing defects.

### Triage Groups

| Group | Findings | Context | Preferred resolution | Why |
|---|---|---|---|---|
| Updater state integrity -- apply queue | #3, #4, #8 | Task 2 and Task 3 leave three user-visible state-ordering holes | Define dismissal and busy-state behavior once, then add deferred transition tests before changing state code | One transition model should prevent recovery loss, stale replay, and restart during album render |
| Accessibility evidence -- apply queue | #1, #2, #7 | The receipt is visually improved, but two gates can false-green and the required lane is red | Settle the receipt before axe, correct the real contrast pair, then make geometry probes fail closed | Gate reliability must be restored before another visual readiness claim |
| Launch records -- apply queue | #5, #6 | The active queue and exact-commit ledger do not describe the branch now under disposition | Reconcile the active resume instructions first, then add the exact Phase A commit map | The owner must not be directed to obsolete candidate bytes or unverifiable branch-level evidence |

### P1 -- High

| # | File | Issue | Reviewer | Confidence |
|---|---|---|---|---|
| 1 | `scripts/verify-app-headless.mjs:1229` | Warning-receipt axe scan can run before its entrance animation settles | adversarial, root | 100 |
| 2 | `src/App.css:20` | Receipt contrast remediation is still below WCAG AA | adversarial, root | 100 |
| 3 | `src/App.tsx:395` | Dismissing the installing toast suppresses later failure recovery | correctness, adversarial, root | 100 |
| 4 | `src/App.tsx:668` | Both updater actions remain enabled during album rendering | root, testing | 100 |

- **#1:** The receipt animates for 160 ms at `src/App.css:2621-2628`, but the ordinary-motion path reaches axe without waiting for animation completion. Exact commit `9f60023` passed while color contrast was recorded as incomplete, not proven clean. Await animation completion and negative-control a known under-contrast receipt selector.
- **#2:** The token comment and focused test use `--bg-1` (`src/App.layout-css.test.ts:259-283`), while `.receipt-path-full` and `.receipt-style-blurb` paint on `--bg-2` (`src/App.css:2878-2907`, `src/App.css:2942-2997`). The current required lane reports 4.45:1 at 1440x900 and 4.07:1 at 1360x740. Correct the actual foreground/background contract and rerun axe at both sizes.
- **#3:** `fail()` maps a null notice to null, while the installing toast retains an enabled dismiss control at `src/App.tsx:659-663` and `src/components/Toast.tsx:44`. A dismiss followed by rejection or unexpected return leaves no Retry or manual recovery. Preserve the clicked version across settlement or define a non-dismissible installing state, then test both settlement paths.
- **#4:** Track export and preview render use `tm.isExporting` and `tm.isRendering`, but album export independently sets `tm.albumRendering` across `renderAlbumPlan` at `src/hooks/useTrackMaster.ts:1929-1991`. Both Restart and Retry omit it. Include album rendering in both guards and behavior-test initial install and Retry during all three busy states.

### P2 -- Moderate

| # | File | Issue | Reviewer | Confidence |
|---|---|---|---|---|
| 5 | `docs/CHANGELOG.md:21` | The three required launch records omit the exact Phase A commit ledger | project standards, adversarial, root | 100 |
| 6 | `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md:455` | The active quality-plan ledger still directs obsolete candidate flow | project standards, adversarial, root | 100 |
| 7 | `scripts/verify-app-headless.mjs:710` | Receipt geometry gate tolerates a missing required action and zero real scroll | testing, reliability, adversarial | 100 |
| 8 | `src/App.tsx:375` | A delayed replay query resurrects a notice the user dismissed | frontend races, root | 75 |

- **#5:** Task 11 line 399 requires the exact Tasks 1-10 commits in `beta-go-no-go`, the owner checklist, and the changelog. None of the Phase A hashes appears in those files. Add a compact exact-commit map while keeping frozen beta.1 evidence distinct.
- **#6:** The 2026-08-31 addendum correctly says beta.1 is audit-blocked at lines 467-479, but the same authoritative file still says C4 is closed at line 455, labels the 2026-07-27 section "read this first" at line 481, and directs U15 to `v0.9.1-beta.1` at lines 499-503. Mark the old block as historical and remove its active resume authority without rewriting executed C1-C3 history.
- **#7:** The verifier documents Done, Show file, and Close as required at lines 589-593, then explicitly exempts a missing Show file at lines 709-711. It also checks pinned-row movement only when `scrolledTo > 0` at lines 724-734. Require all three controls and positive scroll travel from a deterministic overflow fixture.
- **#8:** A listener event can show a version, dismissal can set the notice to null, and the still-pending startup query can call the shared setter afterward. The setter treats null as fresh availability at `src/App.tsx:357-362`, so dismissal is undone. Track dismissal by version or suppress stale query results, and add a deferred-query test.

### Requirements Completeness

- [x] **Task 0:** Baseline and frozen beta.1 objects preserved; no commit expected.
- [x] **Task 1:** Guarded import metadata probing follows the triage-corrected Symphonia panic-boundary mechanism.
- [ ] **Task 2 -- partial:** Backend latch/query and register-then-query implementation exist, but the specified unresolved-listener red test was not written and delayed replay can undo dismissal (#8).
- [ ] **Task 3 -- partial:** Rejection, no-op, Retry, manual origin, and opener failure have genuine tests; pending dismissal loses recovery (#3), album render is not guarded (#4), and the required busy-state tests are missing.
- [x] **Task 4:** Effective sticky-TOOLS opacity is measured in live Chromium and the winning rule is opaque.
- [ ] **Task 5 -- partial:** Loudness naming is correct, but the contrast fix and axe settlement are not sound (#1, #2).
- [x] **Task 6:** Component red tests and implementation cover initial focus, containment, close paths, and restoration. A fixed 50 ms browser wait remains a low-risk flake concern.
- [ ] **Task 7 -- partial:** The production shell matches the triage-reframed contract in current evidence, but the browser gate is not fail-closed (#7).
- [x] **Task 8A:** Loaded Standard is exercised at both desktop viewports and joins the axe matrix.
- [x] **Task 8B:** Album advisory evidence uses real request/seed data and a real Export Album path; no fictional report was introduced.
- [x] **Task 8C:** Landing CTA evidence records both axes and gates the owner-approved current contract without deciding Task 14 taste.
- [x] **Task 9:** Vite/Vitest and helper config selection is explicit and TypeScript emits no shadow config.
- [x] **Task 10:** All three locks resolve patched `anyhow`; current exact audits deny unsound advisories with the one documented Linux exception. The static test should later pin the exact three lock paths.
- [ ] **Task 11 -- partial:** Several stale truths were corrected, but the active resume ledger and exact-commit evidence requirements remain incomplete (#5, #6); two positive truth assertions are also weaker than their test names.

### Commit-by-Commit Verdicts

`SOUND` means the red proof, implementation contract, and task scope are acceptable. `CONCERN` means at least one requested criterion is not met; it does not invite a stylistic rewrite.

| Commit | Task | Red-before-fix verification | Plan and triage match | Scope | Verdict |
|---|---|---|---|---|---|
| `b81d820` | 1 | Reversing the production change makes `import_tracks_turns_zero_sample_rate_parser_panic_into_decode_error` at `src-tauri/tests/contracts.rs:297` hit the Symphonia zero-rate panic instead of returning `CommandError::Decode` | `src-tauri/src/files.rs:62` now routes through the guarded parser at `src-tauri/src/decode.rs:499-500`, exactly matching the Task 1 amendment | Fixture/test extraction and parser consolidation only | **SOUND** |
| `d874d90` | 2 | Reversing the App change makes the focused notice test red, but `src/App.transitions.test.tsx:821-824` resolves listener registration immediately and does not reproduce the specified unresolved-listener ordering | Latch/query and register-then-query direction are correct; delayed query can still undo dismissal (#8) | Only updater store, API, App, tests, and preview fallback | **CONCERN** |
| `5fe2a90` | 3 | Reversing the App recovery change makes seven updater transition tests red for rejection, no-op, Retry, and manual recovery | Core recovery/origin design matches Task 3, but failure after pending dismissal disappears (#3) and album rendering is omitted from both busy guards (#4) | The opener, Rust floor, manifests, lock, UI, API, and tests are all task-required; commit message bridge-lane claims were not counted as independently rerun evidence | **CONCERN** |
| `e1f0b36` | 4 | Reversing CSS makes the structural check fail and the live probe measure transparent computed background with real overlap | Browser-computed alpha/topmost ownership and the winning rule match Task 4 | TOOLS CSS, focused structural test, and probe only | **SOUND** |
| `6986032` | 5 follow-up | No commit-local red test exists; the focused ratio test arrives in `9f60023` | The chosen token was calculated against the wrong surface and current Chromium is red (#2) | CSS token only, but the fix itself is incorrect | **CONCERN** |
| `9f60023` | 5 | Removing `aria-label="Loudness target"` at `src/App.tsx:2321` makes the component assertion red; exact-commit headless passed 28 checks, but warning contrast was incomplete during animation rather than proven clean | Accessible name and coverage ledger match; axe timing can false-green (#1) and the required current lane exposes #2 | Label, tests, axe harness, and testing docs only | **CONCERN** |
| `a20ab26` | 6 | Reversing production focus code makes seven component focus cases fail, including all close restorations at `src/components/ExportReceiptCard.test.tsx:712-759` | StrictMode-safe explicit restoration and containment match Task 6 | Receipt, persistent export ref, focused tests, and browser assertion only | **SOUND** |
| `69c789f` | dependent evidence | Restoring parent assets while retaining the new manifest makes the landing asset digest gate red | Captures refresh tracked public evidence after visual changes; no product behavior is claimed | Only three generated captures and their manifest; acceptable generated consequence | **SOUND** |
| `a15284a` | 7 | Reversing component/CSS makes the new sibling-row structural test red; exact-commit headless passed 28 checks and current geometry records visible controls, a 32x32 close target, and nonzero scroll at both sizes | Production shell matches the Task 7 amendment, but its verifier can pass without Show file or real scrolling (#7) | Receipt shell, test/harness/docs, and dependent capture refresh only | **CONCERN** |
| `9a4bf1f` | 8A | Test-only coverage commit; no production red is expected | `standard-clean` is declared at `scripts/verify-app-headless.mjs:136` and required in the axe matrix at lines 1412-1413; current Standard evidence is clean | Scenario and trace-test coverage only | **SOUND** |
| `63657cc` | 8B | Reversing preview-mock changes makes four new tests fail for folder selection, source rates/channels, rendered rate, and request-owned overrides | `album-advisory` at `scripts/verify-app-headless.mjs:326` drives the real export and current evidence contains all four supported advisories | Preview-only data/driver/tests plus manifest consequence | **SOUND** |
| `acab580` | 8C | Evidence-only correction; no product red is expected | The helper records per-axis containment at `scripts/verify-landing-responsive.mjs:313-338` and gates nav all-axis plus hero horizontal at lines 521-538; owner taste remains queued | Landing verifier plus one owner question only | **SOUND** |
| `1c9b35a` | 9 | Restoring parent build config makes all four new `src/build-config.test.ts:19-67` cases fail | Package scripts explicitly select `vite.config.ts` at `package.json:7-15`; build and tests pass with no emitted shadow config | Build scripts/config/ignore rules, tests, and dependent capture only | **SOUND** |
| `80e2898` | 10 | Parent locks plus deny-unsound policy reproduce `RUSTSEC-2026-0190` as a denied warning; restoring parent CI shape makes the static test red | All three actual CI commands name distinct lockfiles at `.github/workflows/ci.yml:339-341` and current audits pass | Three transitive lock patches, CI, security test, and testing docs only | **SOUND** |
| `691392d` | 11 | Restoring parent docs makes four launch-truth assertions red | Many stale truths are corrected, but Task 11's active ledger and exact-commit requirements are incomplete (#5, #6) | Changes stay within the named active docs and release-readiness tests | **CONCERN** |

### Verification Run by This Review

| Lane | Result | Evidence |
|---|---|---|
| `npm test` | PASS | 82 files, 822 tests |
| `npm run build` | PASS | TypeScript and Vite production build completed |
| `npm run verify:headless` | **FAIL** | Two serious `color-contrast` violations in warning receipt; evidence at `test-output/headless/2026-08-31T22-26-02-687Z` |
| `cargo fmt --check` | PASS | Run in `src-tauri` |
| `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | PASS | Run in `src-tauri` |
| `cargo test --target-dir target\codex-rc` | PASS | 430 library tests passed, 2 ignored; integration binaries green |
| Three exact RustSec commands | PASS | Desktop, iPhone, and Android locks audited with deny-unsound and only `RUSTSEC-2024-0429` ignored |
| `npm audit --audit-level=high --omit=dev` | PASS | 0 vulnerabilities |

Historical verification used isolated temporary worktrees and reverse-production mutations. Those worktrees were removed after evidence capture. Green results in commit messages were not counted unless rerun here or reproduced by a focused historical check.

### Visual and UX Readiness

The current app is materially closer to launch-ready. Headless inspection of loaded Standard, Album advisory, and the warning receipt shows a coherent graphite visual system, clear hierarchy, visible export actions, a usable 32x32 receipt close target, and no obvious clipping at the two supported desktop viewports. The fixed receipt shell is a real UX improvement, and the Standard and Album states read as deliberate product surfaces rather than test scaffolding.

It is not yet visually ready for launch approval. The saved-path and style-blurb text in the receipt are visibly too muted and mechanically below AA; the lane that is supposed to protect that surface is currently red. Headless screenshots and digest freshness also do not replace the owner's installed-WebView taste pass or final by-ear candidate check.

### Actionable Findings

| # | File | Issue | Route | Required response |
|---|---|---|---|---|
| 1 | `scripts/verify-app-headless.mjs:1229` | Axe runs before receipt animation settlement | `gated_auto -> downstream-resolver` | Add deterministic settlement and a negative control |
| 2 | `src/App.css:20` | Receipt text remains below AA on its real backgrounds | `gated_auto -> downstream-resolver` | Correct the actual color pair and focused test |
| 3 | `src/App.tsx:395` | Pending dismissal removes failure recovery | `gated_auto -> downstream-resolver` | Preserve recovery through dismissal and test rejection plus no-op |
| 4 | `src/App.tsx:668` | Album render does not disable updater actions | `gated_auto -> downstream-resolver` | Guard both actions with `albumRendering` and test all busy states |
| 5 | `docs/CHANGELOG.md:21` | Exact Phase A commit map is absent | `gated_auto -> downstream-resolver` | Add exact Tasks 1-10 hashes to all three required records |
| 6 | `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md:455` | Active resume text directs obsolete candidate flow | `gated_auto -> downstream-resolver` | Reconcile the ledger and mark old U15 prose historical |
| 7 | `scripts/verify-app-headless.mjs:710` | Geometry proof fails open | `gated_auto -> downstream-resolver` | Require Show file and positive scroll before invariance checks |
| 8 | `src/App.tsx:375` | Delayed query reopens a dismissed updater notice | `gated_auto -> downstream-resolver` | Define per-version dismissal and add deferred-query coverage |

### Pre-existing Issues

| File | Issue | Disposition |
|---|---|---|
| `src/App.tsx:369-384` | A listener registration that resolves after cleanup can leak its returned unsubscribe function | Pre-existing; not caused by this range. Address with the next updater lifecycle hardening, not by rewriting Phase A history |

### Coverage

- Validator: one fresh batch validated all seven submitted candidates. The final report splits the validated contrast candidate into #1 and #2; no validator finding was dropped.
- Cross-model route: requested Claude Opus at high reasoning through the `claude` route; the worker stopped before provider review because local `jq` was unavailable. Receipt, actual model/effort, and independence are unverified. The local adversarial fallback independently confirmed #1, #2, #3, #6, and #7.
- Security review found no new security vulnerability. The fixed-origin zero-argument opener remains narrow; installed OS-opener behavior is still a Task 13 owner/reality-lane item.
- Testing gaps: Task 2 does not defer listener registration; Task 3 does not test track export, preview render, or album render guards; Task 10's static test does not pin the exact set of three lockfile targets; Task 11's positive truth assertions are weaker than their names.
- Residual low-risk cleanup: stale/misattached updater rustdoc at `src-tauri/src/lib.rs:343-349`; unused exported `ToastAction` at `src/components/Toast.tsx:4`; obsolete track-warning flag in the album advisory fixture at `src/lib/preview-mock.ts:176`; fixed 50 ms focus assertion at `scripts/verify-app-headless.mjs:211`.
- Large files were not treated as defects by size alone. The 1,480-line headless harness and 1,014-line transition test remain future extraction candidates only if a behavior-preserving seam is justified.
- Owner-only gates remain owner-only: installed updater failure matrix, real installer/WebView behavior, signing, final visual taste, final by-ear candidate check, publication, deployment, and announcement.
- No reviewer found scope expansion into preset retuning or DSP changes. No release, tag, draft, main, or publication mutation occurred.

---

### Verdict

> **Owner Task 12 recommendation: HOLD.** Do not advance this branch to replacement-candidate disposition yet.
>
> **Reasoning:** The required current headless lane is red, Task 3 still permits updater interruption/recovery loss, Task 2 can resurrect dismissed UI, Task 7's proof is fail-open, and Task 11 can direct obsolete release action. These are contract defects, not style preferences.
>
> **Fix order:** updater state integrity (#3, #4, #8) -> settled contrast and axe proof (#1, #2) -> fail-closed receipt geometry (#7) -> exact and unambiguous launch records (#5, #6) -> rerun every required lane and repeat the disposition review.

**Prioritized actionable recap:**

1. **P1** `src/App.tsx:395,668` -- repair updater recovery and album-render guards; response: code plus transition tests.
2. **P1** `scripts/verify-app-headless.mjs:1229`, `src/App.css:20` -- settle axe and fix real receipt contrast; response: code, focused ratio test, and both viewport scans.
3. **P2** `src/App.tsx:375` -- stop stale replay from undoing dismissal; response: state contract plus deferred-query test.
4. **P2** `scripts/verify-app-headless.mjs:710` -- make Task 7 proof fail closed; response: required controls plus real-scroll assertion.
5. **P2** `docs/CHANGELOG.md:21`, `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md:455` -- add exact commits and remove obsolete resume authority; response: documentation plus static truth guards.

---

## 2026-08-31 Remediation Verification Addendum

**Reviewed range:** `6ed3b53..9919611` on
`codex/launch-readiness-remediation`, after an explicit
`git pull --ff-only origin codex/launch-readiness-remediation` reported the
branch up to date. The four reviewed commits are `016b29f`, `333c6dc`,
`0cf48ce`, and `9919611`. Product code remained read-only during this review;
historical red proofs ran only in an isolated temporary worktree, which was
clean and removed afterward.

### Per-Finding Verdicts

| # | Fix commit | Verdict | Red-before-fix and current evidence | Mechanism and scope assessment |
|---|---|---|---|---|
| 1 | `0cf48ce` | **RESOLVED** | The original review's ordinary-motion run reproduced the early axe scan. The required standing negative control was rerun here and exited `1`, reporting `#axe-self-test-low-contrast` at both supported viewports. The current full headless lane records zero axe violations for the warning state. | Axe-gated normal-motion scenarios await and recheck finite animations before `runAxeScan` at `scripts/verify-app-headless.mjs:1246-1308`. This resolves the 160 ms receipt-entrance false-green. The nominal five-second deadline does not race an individual `animation.finished` await (`:1268-1274`), so an unusually long finite animation can delay rather than promptly report; that is a non-blocking timeout-diagnostic weakness, not the original early-scan failure. |
| 2 | `333c6dc` | **RESOLVED** | Restoring only the parent `src/App.css` while retaining the new focused test produced one red test: the real `--text-2` / `--bg-2` ratio was `4.519598442640461`, below the `4.8` floor. Current `npm test` is green. | `src/App.css:20-27` raises `--text-2`, and `src/App.layout-css.test.ts:259-290` now measures the actual receipt ground. Capture and manifest refreshes are required consequences because App CSS is a capture input, not unrelated visual work. |
| 3 | `016b29f` | **RESOLVED** | Restoring only the parent `src/App.tsx` made both new pending-dismissal cases red: rejection and unexpected backend resolution failed to surface `Update couldn't install`. | `src/App.tsx:406-414` captures the clicked version before settlement and restores a recoverable failed notice independently of toast dismissal. The change and tests are confined to updater state. |
| 4 | `016b29f` | **RESOLVED** | The same reverse-production run made the `albumRendering` Restart case and failed-toast Retry case red. Together with findings 3 and 8, the focused run reported exactly 5 failures and 34 passes; the existing `isExporting`, `isRendering`, and listener-order cases correctly remained green contract pins. | Both updater actions now include the independent `tm.albumRendering` flag at `src/App.tsx:683-710`. The test seam covers Restart across all three busy flags and Retry while album render begins mid-install. |
| 5 | `9919611` | **NOT RESOLVED** | Restoring the parent docs made the new launch-record test red, but the test is not strong enough to prove its own named contract. It checks only four sentinel hashes in each file at `src/lib/release-readiness.test.ts:89-100`; it checks neither every Phase A hash nor any task-to-commit association. | `docs/CHANGELOG.md:58-68` and `docs/plans/beta-go-no-go.md:29-39` contain real Task 1-11 maps. `docs/plans/2026-08-31-owner-launch-checklist.md:16-23` only gives an ordered hash list and redirects the reader to the changelog for the "full task-to-commit map." The required map is therefore still absent from one of the three records, contrary to the commit message and the original finding's all-three-record contract. No fix was made at this stop checkpoint. |
| 6 | `9919611` | **RESOLVED** | Restoring the parent docs made the historical-resume-authority assertion red. Current unit coverage and direct text inspection agree. | `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md:467-503` makes the 2026-08-31 addendum the only resume authority, labels the 2026-07-27 block historical, and explicitly disclaims its obsolete `v0.9.1-beta.1` U15 direction without rewriting executed history. |
| 7 | `0cf48ce` | **RESOLVED** | The parent verifier mechanically exempted missing Show File and conditioned invariance on `after.scrolledTo > 0`; both are exact fail-open reproductions. The current full lane records all three controls present/topmost and real warning-receipt scroll travel of 123 px at 1440x900 and 200 px at 1360x740. | `scripts/verify-app-headless.mjs:651-750` now requires Done, Show file, and Close, reports a missing scroll region, and makes zero travel a failure before testing pinned-row invariance. This matches the reframed Task 7 geometry contract. No unrelated receipt redesign landed in this fix commit. |
| 8 | `016b29f` | **RESOLVED** | Restoring only the parent `src/App.tsx` made the new deferred-query test red: the dismissed toast reappeared after the startup query resolved. | `src/App.tsx:357-375` records dismissal by version and suppresses same-version replay while allowing a future version to surface. The mechanism addresses the exact event-dismiss-query ordering race. |

### Commit Scope Verdicts

| Commit | Scope verdict | Evidence |
|---|---|---|
| `016b29f` | **SOUND** | Only `src/App.tsx` and the two relevant App test files changed; the extra unresolved-listener case is the exact missing Task 2 contract pin from the first review. |
| `333c6dc` | **SOUND** | Contrast token, focused CSS contract test, and dependent landing captures/manifest only. |
| `0cf48ce` | **SOUND** | Headless verifier/wrapper plus testing documentation only; no production UI, preset, or DSP change. |
| `9919611` | **CONCERN** | Files stay within finding 5/6 documentation and its static test, but finding 5's promised all-record map and enforcement are incomplete. |

### Verification Rerun by This Addendum

| Lane | Result | Evidence |
|---|---|---|
| `npm test` | **PASS** | 82 files, 832 tests. |
| `npm run build` | **PASS** | TypeScript checks and Vite production build completed. |
| `npm run verify:headless` | **PASS** | Landing suite plus 31 app scenario/viewport checks; evidence at `test-output/headless/2026-09-01T00-05-12-466Z`. |
| `npm run verify:headless -- --skip-build --force-axe-fail warning` | **EXPECTED FAIL** | Exit `1`; `color-contrast` at `#axe-self-test-low-contrast`, ratio 1.07:1, at both 1440x900 and 1360x740. Evidence at `test-output/headless/2026-09-01T00-08-44-749Z`. |
| `git diff 6ed3b53..9919611 --stat` | **PASS: Rust untouched** | 17 changed files, all frontend/tests/docs/evidence; no path under `src-tauri`, `apps/iphone-native/rust`, or `apps/android-native/rust`. Rust lanes were therefore not rerun for this frontend/docs-only fix range. |

### Frozen Object Check

- Local and remote `v0.9.2-beta.1` still resolve to
  `c750da679929bcdd64e6030f890283f40d939181`.
- Release id `379883047` remains a draft with 9 assets, `published_at: null`,
  tag `v0.9.2-beta.1`, and target commit `c750da6`.
- No tag, draft, release, main, preset, DSP, or product-code mutation occurred.

### Addendum Verdict

> **Owner Task 12 recommendation: HOLD.** Seven of eight findings are resolved,
> but finding 5 is still incomplete in the owner checklist and its new static
> test can false-green that omission. Do not advance to owner disposition on
> `9919611` as the fully reviewed Phase A tip. Per the stop checkpoint, this
> review records the defect and does not fix it.

**Required response before ADVANCE:** add an actual Task 1-11 to exact-commit
mapping to the owner checklist, strengthen the launch-record test so all three
records must carry the complete mapping rather than four sentinel hashes, then
repeat the focused documentation test and this disposition review. The frozen
beta.1 objects and owner-only reality gates remain untouched.
