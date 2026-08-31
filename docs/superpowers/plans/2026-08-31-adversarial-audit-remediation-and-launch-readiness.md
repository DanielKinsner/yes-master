# YES Master Adversarial Audit Remediation and Launch Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** PLAN ONLY — written from the 2026-08-31 adversarial audit. No remediation, release, deployment, repository-setting change, or publication is authorized by this document.

**Goal:** Resolve the audit-blocked `v0.9.2-beta.1` candidate according to the owner's disposition and, if rejected, replace it with a mechanically verified candidate that cannot strand hostile imports, lose updater notifications, hide updater recovery, or ship the confirmed Advanced/receipt accessibility defects; then close the remaining owner-controlled launch gates and execute the verified post-launch simplification backlog without changing approved sound.

**Architecture:** Keep the replacement-candidate lane narrow and test-first: harden the existing Rust parser boundary, make updater state replayable and recoverable, correct the winning UI rules, and expand the real Chromium gate. Preserve release history and require exact-commit evidence. After launch, centralize frontend defaults and transport resets, remove only proven residue, and consolidate CSS by selector ownership rather than by file size.

**Tech Stack:** React 19, TypeScript 5.6, Vite 6, Vitest 4, axe-core 4.12, Playwright 1.61 bundled Chromium, Tauri 2, Rust/Cargo, Symphonia, GitHub Actions, GitHub Releases updater, Tailwind CSS 4, Vercel static hosting.

**Spec:** `docs/PRODUCT.md`; `docs/APP_BEHAVIOR.md`; `docs/ARCHITECTURE.md`; `docs/TESTING.md`; `docs/RELEASE_STABILIZATION.md`; `docs/plans/2026-06-30-launch-plan.md`; `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`; `docs/plans/beta-go-no-go.md`; `docs/OWNER_INPUT_QUEUE.md`; `docs/CAPABILITY_EVIDENCE_MATRIX.md`.

## Global Constraints

- This plan was written against `main` at `bb414c1145b82428c4bacdbd013af97534b10aa9`. At execution time, re-read `AGENTS.md` and every document in **Spec**, fetch/recheck current `main`, and re-prove each finding before editing.
- Treat `v0.9.2-beta.1` at `c750da6` as **audit-blocked / NO-GO pending owner disposition**, not as a movable label and not yet as formally rejected. Never force-move, reuse, or silently delete its tag, draft release, artifacts, or evidence.
- Candidate rejection, remediation merge, branch push, tag creation, tag push, draft deletion, release publication, activation merge, activation push, preview deployment, production promotion, DNS changes, repository-setting changes, and announcement are distinct permissions. Obtain each authorization at the point named in this plan; one never implies another.
- Work on `codex/launch-readiness-remediation` only after the owner asks to execute this plan. Keep one behavior per commit and keep the replacement-candidate work separate from post-launch elegance work.
- Track Master stabilization stays ahead of feature expansion. Do not retune presets, change DSP behavior, enable Adaptive Compressor, alter its calibration constants, or reinterpret the 2026-08-25 listening approval.
- Preserve private audio and rendered private masters outside git. Generate hostile fixtures in tests; do not add user audio.
- Default to focused Vitest/Cargo checks and bundled-Chromium headless evidence. Do not keep GUI apps or preview servers running; installed-app/device work is confined to the explicit owner reality lane.
- A visual screenshot is agent evidence, not owner taste approval. A headless run is not installed-WebView, real-device, responsiveness, updater-transaction, signing, publishing, DNS, or listening evidence.
- Large files are not defects by themselves. Do not split `src/App.tsx`, `src/hooks/useTrackMaster.ts`, `src-tauri/src/audio.rs`, or `src-tauri/src/dsp.rs` unless a separately proved ownership seam justifies it.
- Do not perform a wholesale `App.css` rewrite, introduce cascade layers, or mix the post-launch CSS consolidation into the replacement candidate.
- Do not delete the unused iPhone image or untrack Graphify artifacts without an explicit owner choice.
- If a red test does not reproduce the stated defect, stop that task, record the new evidence, and revise this plan or the live queue before changing production code.

## Baseline Verdict

The application is close, and current CI/release plumbing is materially stronger than the old audit history suggests, but the present candidate is not launchable. Four product-defect groups directly block it:

1. The import metadata path duplicates Symphonia probing outside the existing panic boundary. A crafted zero-sample-rate WAV can panic that command path and leave the frontend import request unresolved.
2. Updater install failure clears the only notice, with no retry or safe manual recovery; updater availability is also an edge-triggered startup event that can be missed before the frontend listener attaches.
3. The final `.right-rail-tools` declaration is transparent, so content visibly scrolls under the sticky TOOLS surface. The current source test reads the first matching declaration and is false-green.
4. The Advanced loudness selector has no accessible name, and the export receipt lacks complete modal focus containment/restoration. Receipt actions can also fall outside the initial visible shell, and its close target is too small.

Security gate S-01/S-02 and active-document drift D-01 are additional candidate-evidence blockers. If the owner rejects beta.1, a replacement must be rebuilt because the remediation changes release-bound Rust/frontend files. Until that owner decision, the old green CI, artifact hashes, install evidence, and draft-release evidence remain frozen beta.1 evidence; they cannot be carried forward as proof for different code.

### Existing evidence to preserve, not overclaim

- Current HEAD workflow run `33412552057` was seven-for-seven green at audit time. It proves the pre-remediation HEAD lanes, not the replacement candidate.
- Candidate-era workflow run `32873776284` was seven-for-seven green and release run `33409477883` was four-for-four green with nine draft assets. They remain frozen beta.1 evidence until owner disposition and useful release-machinery history afterward, never proof of repaired code.
- npm audit was clean at the audit snapshot. RustSec still found the narrow `anyhow` unsoundness issue and Linux-only glib exception addressed in Task 10.
- `https://yes-master.vercel.app/` was reachable, closed/noindex, and not activated for download. The brand metadata points at `yesmaster.app`, whose DNS/mail ownership remains unresolved in Task 14.

### Visual and UX readiness snapshot

- The graphite desktop visual system is coherent enough to preserve; the plan does not call for a redesign. It is not visually ready while the confirmed sticky-TOOLS overlap and receipt-shell defects remain.
- Standard's loaded workflow appears substantially complete, but its absence from the permanent headless matrix makes that confidence non-durable until Task 8A.
- Advanced needs the small control-label, sticky-surface, and receipt-focus fixes in Tasks 4–7; these are targeted usability corrections, not taste-driven restyling.
- Album's product path is present, but the named warning screenshot is currently weak evidence because it stops before the supported post-export advisory. Task 8B fixes the evidence rather than inventing a new modal.
- The landing page is mechanically strong across its current matrix and deliberately closed. The short-phone hero CTA ordering is a genuine owner-taste/spec choice; the fixed nav CTA remains the blocking acquisition control until that decision.

## What the Audit Did Not Justify

- No unused npm dependency or dead TypeScript module was independently confirmed.
- No DSP or approved-preset defect was confirmed. Sound changes are out of scope.
- `src-tauri/src/audio.rs`, `src-tauri/src/dsp.rs`, `src/App.tsx`, and `src/hooks/useTrackMaster.ts` are large, but size alone is not a refactor trigger.
- The single-key `A`, `L`, and `?` shortcuts are a known owner-accepted beta exception. Keep them unchanged for this candidate; a post-beta settings/remapping decision remains optional.
- The 917,184-byte `src/assets/landing/iphone-standard-ui.jpg` is unpublished owner art, not automatically disposable code.
- The Linux-only `glib 0.18.5` advisory does not block the Windows/macOS-first beta when it is explicitly isolated by advisory ID and Linux remains deferred; new unsound advisories must still fail CI.

## Finding Coverage Ledger

| ID | Confirmed finding | Disposition | Planned task |
|---|---|---|---|
| L-01 | Import metadata parser bypasses the untrusted-audio panic boundary | Replacement-candidate blocker | 1 |
| L-02 | Updater startup event can be missed | Replacement-candidate blocker | 2 |
| L-03 | Updater failure disappears with no retry/manual recovery | Replacement-candidate blocker | 3 |
| U-01 | Sticky TOOLS winning rule is transparent; static test is false-green | Replacement-candidate blocker | 4 |
| A-01 | Advanced loudness `<select>` is unnamed | Replacement-candidate blocker | 5 |
| A-02 | App surface has no committed axe scan | Candidate quality gap | 5 |
| A-03 | Receipt focus escapes and is not restored | Replacement-candidate blocker | 6 |
| U-02 | Receipt actions can start below the visible shell; close target is about 21 px | Replacement-candidate blocker | 7 |
| T-01 | Loaded Standard has no permanent browser scenario | Candidate regression gap | 8 |
| T-02 | `album-warning` stops mid-analysis instead of proving a supported album advisory | Candidate regression gap | 8 |
| T-03 | Landing CTA says “fully visible” while measuring only horizontal clipping | Owner-taste/spec decision before activation | 8 and 14 |
| B-01 | Local Vite/Vitest can select ignored emitted config rather than the TypeScript source | Candidate reproducibility gap | 9 |
| S-01 | Three locks contain `anyhow 1.0.102` affected by RUSTSEC-2026-0190 | Candidate security blocker | 10 |
| S-02 | CI RustSec lane does not deny unsound advisories | Candidate security blocker | 10 |
| D-01 | Active launch/product/evidence documents contradict current code and owner decisions | Candidate evidence blocker | 11 |
| R-01 | `v0.9.2-beta.1` evidence is invalid after release-bound fixes | Replacement candidate required | 12 |
| R-02 | Real install, responsiveness, listening, updater, terms, publication, and GO remain owner gates | Owner-only | 12–14 |
| R-03 | `yesmaster.app` metadata has no verified domain/mail configuration | Owner decision before activation | 14 |
| R-04 | GitHub security toggles and protected-main policy are disabled | Owner/admin hardening | 14 and 21 |
| R-05 | The required updater negative-path lifecycle harness does not exist | Blocking owner risk decision or isolated-harness evidence | 13 |
| C-01 | Production/default settings are duplicated in hook, preview, UI, and tests | Post-launch simplification | 15 |
| C-02 | Four transport reset blocks are duplicated | Post-launch simplification | 16 |
| C-03 | `transport.deviceLost` is write-only in production | Post-launch dead state | 16 |
| C-04 | Volume Match source-of-truth comments describe obsolete behavior | Post-launch correctness | 17 |
| C-05 | One CSS token, one keyframe, one class, and one App CSS import are proven dead/duplicate | Post-launch cleanup | 17 |
| C-06 | `App.css` has contradictory ownership in rail/compressor selector families | Post-launch targeted consolidation | 18 |
| C-07 | Tailwind scans docs/tests/Graphify and makes build corpus checkout-dependent | Post-launch build tightening | 19 |
| C-08 | CI actions use mutable major tags/branch | Post-launch supply-chain hardening | 20 |
| C-09 | Tracked Graphify output is stale | Post-launch owner/tool decision | 21 |
| C-10 | Unpublished iPhone art remains under runtime assets | Post-launch owner decision | 21 |

## Dependency Order

| Phase | Must be green before proceeding | Result |
|---|---|---|
| A — Remediate | Tasks 0–11 | A tested replacement-candidate commit, still untagged and unpublished |
| B — Recandidate | Task 12 plus explicit owner authorizations | A verified beta.2 tag and mutable GitHub draft both read back against one peeled commit SHA |
| B — Reality/public-release lane | Task 13 owner checks and explicit publication authorization | Real install, responsiveness, sound, updater evidence, then the first public activation point at GitHub Releases |
| B — Site activation | Task 14 owner decisions/authorization | Coherent live landing bound to the verified public GitHub release |
| C — Simplify | Tasks 15–21 after launch | Smaller ownership surface without destabilizing launch |

## Planned File Map

| Area | Existing files | New files |
|---|---|---|
| Import hardening | `src-tauri/src/decode.rs`, `src-tauri/src/files.rs`, `src-tauri/tests/decode_hostile.rs`, `src-tauri/tests/contracts.rs`, `src-tauri/tests/common/mod.rs` | none |
| Updater | `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, three `Cargo.lock` files, `src/App.tsx`, `src/lib/api.ts`, `src/components/Toast.tsx`, updater/Toast tests | `src/lib/release-links.ts` |
| UI/accessibility | `src/App.tsx`, `src/App.css`, `src/components/RightRail.tsx`, `src/components/ExportReceiptCard.tsx`, component/layout tests | none |
| Browser evidence | `scripts/verify-app-headless.mjs`, `scripts/verify-landing-responsive.mjs`, `src/lib/preview-mock.ts`, scenario/mock tests, `docs/TESTING.md` | none |
| Public monitor/lifecycle | activation metadata, landing smoke, GitHub/Vercel state | `scripts/verify-public-beta-availability.mjs`, `.github/workflows/public-beta-availability.yml`, availability tests, `docs/plans/public-beta-closure-runbook.md` |
| Build/security | `package.json`, `tsconfig.json`, `tsconfig.node.json`, `.gitignore`, `.github/workflows/ci.yml`, three Cargo locks, release-readiness tests | `src/build-config.test.ts` |
| Docs/evidence | active Spec documents, `docs/BETA_TESTING.md`, `docs/CHANGELOG.md`, owner checklist | none |
| Settings cleanup | hook, preview mock, tone reset, Standard/Advanced UI, settings fixtures | `src/lib/default-settings.ts`, `src/lib/default-settings.test.ts`, `src/test-utils/mastering-settings.ts` |
| Transport cleanup | `src/hooks/useTrackMaster.ts`, hook integration tests | `src/lib/transport-state.ts`, `src/lib/transport-state.test.ts` |
| CSS/build debt | `src/App.css`, `src/App.layout-css.test.ts`, `src/LandingPage.css` | no new runtime file |

---

## Phase A — Replacement-Candidate Remediation

### Task 0: Re-enter safely and preserve the audit-blocked candidate

**Files:** read-only baseline; later `docs/plans/beta-go-no-go.md`, `docs/OPEN_THREADS_AND_DECISIONS.md`, and `docs/plans/2026-08-31-owner-launch-checklist.md`.

- [ ] Read `AGENTS.md` and all files in **Spec** from the current checkout.
- [ ] Run `git status --short --branch`, `git rev-parse HEAD`, `git log --oneline --decorate -8`, and `git tag --points-at c750da6`.
- [ ] Confirm the worktree is understood before branching. Preserve unrelated owner changes; stop if any planned file has overlapping edits that cannot be isolated.
- [ ] Fetch and recheck `main`, then create `codex/launch-readiness-remediation` only after the owner has asked to execute this plan.
- [ ] Record `v0.9.2-beta.1` and draft release `379883047` as the frozen current candidate pending owner disposition. Do not edit or delete either object.
- [ ] Reproduce each focused red test in Tasks 1–10 before its production change. A finding that no longer reproduces is removed from the implementation queue rather than “fixed” speculatively.

**Acceptance:** Work begins from a named SHA on an isolated branch, no candidate object has moved, and every production edit has a red mechanical reason.

**Commit:** none.

### Task 1: Put import metadata probing inside the single untrusted-audio boundary

**Files:** `src-tauri/src/decode.rs:41-45,99-115,475-508`; `src-tauri/src/files.rs:1-121`; `src-tauri/tests/common/mod.rs`; `src-tauri/tests/decode_hostile.rs:8-48,106-115`; `src-tauri/tests/contracts.rs:267-310`.

- [ ] Move the generated `crafted_wav(channels, sample_rate, bits, data, data_size_lie)` fixture from `decode_hostile.rs` into `tests/common/mod.rs`; import it from both integration-test binaries.
- [ ] Add `import_tracks_turns_zero_sample_rate_parser_panic_into_decode_error` to `src-tauri/tests/contracts.rs`. It must write `crafted_wav(2, 0, 16, &[0_u8; 400], None)`, call `files::import_tracks`, and require `CommandError::Decode` containing `malformed audio file`.
- [ ] Run the focused red test:

```powershell
Set-Location src-tauri
cargo test --test contracts import_tracks_turns_zero_sample_rate_parser_panic_into_decode_error --target-dir target\codex-rc -- --nocapture
```

- [ ] Rename `decode_panic_boundary` to `guard_untrusted_audio_parse`; retain the existing `catch_unwind(std::panic::AssertUnwindSafe(body))` call and the typed malformed-file error.
- [ ] Add a crate-visible `ProbedAudioMetadata` in `decode.rs` with `duration_seconds: Option<f64>`, `sample_rate: Option<u32>`, and `channels: Option<u16>`.
- [ ] Add `pub(crate) fn probe_audio_metadata(path: &Path) -> CommandResult<ProbedAudioMetadata>` and one private inner parser. It must use `guard_untrusted_audio_parse`, reject explicit zero sample rate/channel values, and calculate duration only from finite non-zero header values.
- [ ] Rewrite `probe_audio_format` as a strict mapping over `probe_audio_metadata`; retain its existing 44.1 kHz/missing-channel compatibility behavior and its public return type.
- [ ] Route `files::import_one` through `decode::probe_audio_metadata`.
- [ ] Delete the duplicate Symphonia imports, local `TrackMetadata`, `first_decodable_track`, and `probe_metadata` from `files.rs`.
- [ ] Run focused and full Rust tests:

```powershell
cargo test --test contracts import_tracks_turns_zero_sample_rate_parser_panic_into_decode_error --target-dir target\codex-rc -- --nocapture
cargo test --test decode_hostile --target-dir target\codex-rc
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

**Acceptance:** The hostile import returns promptly with a typed error; decode, waveform, album probe, and normal import behavior remain green; `files.rs` owns no Symphonia parser.

**Commit:** `fix(import): centralize guarded metadata probing`

### Task 2: Make updater availability replayable after frontend startup

**Files:** `src-tauri/src/lib.rs:24-39,42-57,166-185,188-230,240-270`; `src/lib/api.ts:128-134,391-394`; `src/App.tsx:242-243,335-358`; `src/App.transitions.test.tsx`.

- [ ] Add a red Rust unit test around a small `UpdateAvailability` store proving that an available version can be remembered and read more than once without consuming it.
- [ ] Add a frontend transition test where the backend reports a version before the event-listener promise resolves; the update notice must still appear.
- [ ] Add `UpdateAvailability(std::sync::Mutex<Option<String>>)` under `app-runner`, manage it in `run()`, and expose methods that return a descriptive error if the mutex is poisoned.
- [ ] When `run_update_check` finds an update, write the version into the managed store before emitting `updater:available`.
- [ ] Add `#[tauri::command] fn available_update_version(state: tauri::State<'_, UpdateAvailability>) -> Result<Option<String>, String>` and register it in `generate_handler!`.
- [ ] Add `api.availableUpdateVersion()` to `src/lib/api.ts`.
- [ ] In `App`, register `onUpdaterAvailable` first and query `availableUpdateVersion` only after registration succeeds. Apply both paths to the same idempotent state setter so an event/query overlap cannot duplicate UI.
- [ ] Preserve browser-preview behavior: a missing native updater command remains non-fatal and produces no console warning/error.
- [ ] Run:

```powershell
& 'C:\Program Files\nodejs\npm.cmd' test -- src/App.transitions.test.tsx
Set-Location src-tauri
cargo test --lib --target-dir target\codex-rc
```

**Acceptance:** Detection before, during, or after React listener registration yields one visible notice; no network failure blocks app startup; querying does not consume availability.

**Commit:** `fix(updater): replay startup availability state`

### Task 3: Retain failed updater installs with retry and fixed-origin manual recovery

**Files:** `src/App.tsx:242-243,351-357,595-607`; `src/App.transitions.test.tsx:42-106,220-253,816-876`; `src/components/Toast.tsx`; `src/components/Toast.test.tsx`; `src/App.css:2428-2556`; `src/lib/api.ts:128-134`; new `src/lib/release-links.ts`; `src-tauri/src/lib.rs:188-230,271-296`; `src-tauri/Cargo.toml`; `apps/iphone-native/rust/Cargo.toml`; `apps/android-native/rust/Cargo.toml`; all three Cargo locks.

- [ ] Add red tests named for these behaviors: install is disabled while pending; rejection remains visible; Retry invokes exactly one new request; an unexpected successful return is treated as recoverable; manual recovery opens the fixed Releases index; opener failure leaves the literal safe URL visible; export/render disables both install and retry.
- [ ] Change `Toast` from one `action` to `actions?: readonly ToastAction[]`. Each action carries `label`, `onClick`, optional `disabled`, and optional `disabledTitle`. Retain the dismiss button and add a `.toast-actions` layout.
- [ ] Replace parallel updater booleans with this single union:

```ts
type UpdateNotice =
  | { status: "available"; version: string }
  | { status: "installing"; version: string }
  | { status: "failed"; version: string; manualOpenFailed: boolean };
```

- [ ] Enter `installing` synchronously on click. Disable duplicate clicks. If `install_update` rejects **or returns without restarting**, enter `failed` and keep the version.
- [ ] Render the failure copy `Update couldn't install. Retry, or download it manually.` with `Retry` and `Download manually` actions.
- [ ] Export `RELEASES_INDEX_URL = "https://github.com/DanielKinsner/yes-master/releases"` from `src/lib/release-links.ts` for frontend copy/tests. Keep the same fixed string as a private Rust constant and add a release-readiness assertion that the two source constants remain equal. Do not accept a frontend-supplied URL in the native command.
- [ ] Add optional `tauri-plugin-opener = { version = "2", optional = true }`, include it only in `app-runner`, initialize it beside updater/dialog, and add a zero-argument `open_release_page` command that calls the fixed URL. Follow the [official Tauri opener contract](https://v2.tauri.app/plugin/opener/) and raise the declared Rust floor consistently from `1.77` to `1.77.2` in the three manifests.
- [ ] Add `api.openReleasePage()`. If it rejects, keep the notice visible, set `manualOpenFailed: true`, and paint the literal Releases URL so it can be copied.
- [ ] Do not grant arbitrary shell/open permissions and do not use `window.location` inside the desktop WebView.
- [ ] Run:

```powershell
& 'C:\Program Files\nodejs\npx.cmd' vitest run src/App.transitions.test.tsx src/components/Toast.test.tsx src/lib/release-readiness.test.ts
& 'C:\Program Files\nodejs\npm.cmd' run build
& 'C:\Program Files\nodejs\npm.cmd' run verify:headless
Set-Location src-tauri
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --target-dir target\codex-rc
```

- [ ] Because the shared desktop manifest and locks changed, run `& 'C:\Program Files\nodejs\npm.cmd' run verify:iphone` and `& 'C:\Program Files\nodejs\npm.cmd' run verify:android` before candidate freeze.

**Acceptance:** Offline/install failure never clears recovery UI or interrupts the app; retry is single-flight; manual recovery cannot open an attacker-controlled origin; the unexpected backend no-op cannot leave an eternal “installing” state.

**Commit:** `fix(updater): keep failed installs actionable`

### Task 4: Make sticky TOOLS opaque and gate the effective cascade

**Files:** `src/App.css:4053-4087,6569-6572,7597-7605`; `src/App.layout-css.test.ts:12-16,225-237`; `scripts/verify-app-headless.mjs:87-101,526-575,822-832`.

- [ ] Add a browser probe backed by preview state `clean` at 1440×900 and 1360×740. Scroll `.right-rail` until a `.rail-section` genuinely intersects `.right-rail-tools`; fail if the probe produces no overlap.
- [ ] Do not disturb the ordinary clean screenshot. Add a dedicated `clean-tools-overlap` alias and a general `beforeScreenshot(page, report)` scenario hook that runs after settle/drive but before the screenshot, returns a serializable evidence payload, and stores it on that scenario's result. Keep the existing `window.__abRun` payload for legacy scenarios until a separate migration.
- [ ] Read `getComputedStyle(tools).backgroundColor`; parse its alpha and require `alpha >= 0.999`. Record the color, alpha, and overlap count in `summary.json`; a merely translucent sticky surface is not a pass.
- [ ] At an actual rectangle-intersection point, call `document.elementFromPoint`; require the topmost element to be `.right-rail-tools` or its descendant. Store the sampled point/topmost element in the pre-screenshot evidence so the overlap screenshot and measurements describe the same frame.
- [ ] Run `& 'C:\Program Files\nodejs\npm.cmd' run verify:headless` and retain the expected red evidence showing `rgba(0, 0, 0, 0)`.
- [ ] Change only the winning `.right-rail-tools` rule to `background: var(--bg-1)` while preserving its border, radius, and hairline behavior.
- [ ] Leave the widely used first-match `block()` helper unchanged. Delete only its false effective-style claim for TOOLS; add a focused `blocks(".right-rail-tools")` structural check that rejects any direct `background: transparent` declaration. Never model CSS precedence as “last matching text block wins.” Browser-computed style is authoritative because specificity, `!important`, media conditions, and source order all participate.
- [ ] Re-run headless and inspect `clean-tools-overlap-1440x900.png` and `clean-tools-overlap-1360x740.png` for text showing through the sticky surface.

**Acceptance:** Both viewports prove real overlap and an opaque computed background; no rail content paints through TOOLS; no unrelated selector family is rewritten.

**Commit:** `fix(ui): keep advanced tools mask opaque`

### Task 5: Name the loudness control and add committed app accessibility scans

**Files:** `src/App.tsx:2201-2236`; `src/App.loudness-target.test.tsx`; `scripts/verify-app-headless.mjs`; `docs/TESTING.md`.

- [ ] Add a red component assertion that the loudness-profile `<select>` has accessible name `Loudness target`.
- [ ] Add `aria-label="Loudness target"` without changing the visible `LOUDNESS TARGET` copy or selection behavior.
- [ ] Add a reusable app-surface axe helper to `verify-app-headless.mjs`. Load the committed `axe-core/axe.min.js` from `node_modules`, then run WCAG 2.0/2.1 A and AA tags.
- [ ] Scan loaded Advanced `clean` and the open warning receipt through the pre-screenshot hook after their drivers settle. The warning scan must complete while the receipt remains open, before Task 6's post-screenshot focus assertion closes it. Task 8 must opt the newly added `standard-clean` scenario into the same helper.
- [ ] Fail on `violations`; persist rule ID, impact, node count, target selectors, and failure summary for both `violations` and `incomplete` in `summary.json`. Incomplete items remain review evidence, not passes. Do not suppress a rule merely to green the lane.
- [ ] Compose the axe collector with any existing pre-screenshot collector; never overwrite TOOLS or receipt evidence. Add an `axeCoverage` ledger that initially requires exactly one result for Advanced clean and warning receipt at both 1440×900 and 1360×740. Each result must record a positive total across axe `passes + incomplete + violations`; missing, duplicate, or zero-rule scans fail the lane. Task 8 extends the expected matrix with both Standard viewports.
- [ ] If the first real scan exposes an additional reproducible violation, add a focused test and a separate smallest-possible fix commit before proceeding.
- [ ] Run `& 'C:\Program Files\nodejs\npm.cmd' test -- src/App.loudness-target.test.tsx` and `& 'C:\Program Files\nodejs\npm.cmd' run verify:headless`.

**Acceptance:** `select-name` has no affected node, Advanced and warning-receipt states have zero A/AA violations, and incomplete checks are visible evidence rather than silently discarded. Task 8 adds the third Standard state before candidate freeze.

**Commit:** `fix(a11y): name loudness target and gate app states`

### Task 6: Contain export-receipt focus and restore it to the persistent Export button

**Files:** `src/App.tsx:225-303,472-522,611-619`; `src/components/RightRail.tsx:10-39,191-201`; `src/components/ExportReceiptCard.tsx:1-61,130-142`; `src/components/ExportReceiptCard.test.tsx:129-147`; `scripts/verify-app-headless.mjs`.

- [ ] Add a `React.StrictMode` component harness with a persistent Export button and conditionally mounted receipt.
- [ ] Add red assertions for initial dialog focus; Tab from the initially focused dialog to Close; Shift+Tab from the initially focused dialog to Done; Tab wrapping from Done to Close; Shift+Tab wrapping from Close to Done; Escape closure; and focus restoration to Export.
- [ ] Add optional `exportButtonRef?: React.Ref<HTMLButtonElement>` to `RightRail` and attach it to `button.right-rail-export`.
- [ ] Let `App` own one `useRef<HTMLButtonElement>(null)` and pass it to both `RightRail` and `ExportReceiptCard` as `returnFocusRef?: React.RefObject<HTMLElement | null>`.
- [ ] In `ExportReceiptCard`, keep the latest `onClose` in a ref, focus the dialog once on mount, enumerate enabled focusable descendants for Tab/Shift+Tab wrapping, and handle Escape without re-installing the listener on every render.
- [ ] Do not restore focus from effect cleanup: StrictMode's development probe would run that cleanup while the receipt is still mounted. Route Done, ×, backdrop, and Escape through one explicit `closeAndRestore` coordinator; call `onClose`, then queue a generation-checked focus only when the dialog has unmounted and the persistent Export target remains connected. Do not restore focus to the conditional `Export Anyway` control.
- [ ] Add a StrictMode regression proving the effect probe never moves focus back to Export while the receipt remains open, then test restoration independently for Done, ×, backdrop, and Escape.
- [ ] Extend the browser warning scenario: focus Done, press Tab, assert focus remains inside `.receipt`, close with Escape, and assert `.right-rail-export` owns focus.
- [ ] Run the receipt component test and full headless lane.

**Acceptance:** Background project/header controls cannot receive keyboard focus while the receipt is open; every close path restores focus to the persistent Export button.

**Commit:** `fix(a11y): contain export receipt focus`

### Task 7: Give the receipt a fixed shell, visible actions, and a usable close target

**Files:** `src/components/ExportReceiptCard.tsx:140-345`; `src/App.css:2520-2526,2570-2579,2626-2636,2644-2648,8408-8414`; `scripts/verify-app-headless.mjs`; `docs/TESTING.md:169-180`.

- [ ] Run a dedicated reduced-motion warning-receipt geometry capture at both 1440×900 and 1360×740; emulate `prefers-reduced-motion: reduce`, wait two animation frames after `.receipt` becomes visible, and measure before any reachability helper scrolls. Add red checks requiring `.receipt` itself to fit the visible viewport shell, `.receipt-actions` to sit fully inside the receipt **and** satisfy `actions.bottom <= window.innerHeight`, and `.receipt-close` to be at least 24×24 px.
- [ ] Before any scroll, sample the centers of Done, Show file, and Close with `document.elementFromPoint`; require each control or its descendant to be topmost and its rectangle to be inside the viewport.
- [ ] Wrap the information body/details in `.receipt-scroll-region`; keep `.receipt-actions` and `.receipt-footer` outside it.
- [ ] Make `.receipt` a grid with rows `auto minmax(0, 1fr) auto auto` and `overflow: hidden`.
- [ ] Give `.receipt-scroll-region` `min-height: 0`, vertical scrolling, a small right inset, and stable scrollbar gutter.
- [ ] Make `.receipt-close` exactly 32×32 px with zero padding and centered content.
- [ ] Add a structural component assertion that `.receipt-scroll-region`, `.receipt-actions`, and `.receipt-footer` are sibling grid rows in that order. Set `.receipt-scroll-region.scrollTop` to a non-zero value and prove action/footer rectangles do not move.
- [ ] Re-run component, screenshot, geometry, reachability, and axe checks at both viewports.

**Acceptance:** Show file and Done are initially visible at both supported desktop sizes; only the information body scrolls; the close target is 32×32 px; existing receipt hierarchy is preserved.

**Commit:** `fix(ui): keep export receipt actions visible`

### Task 8: Close the browser-scenario gaps without inventing product behavior

**Files:** `scripts/verify-app-headless.mjs`; `scripts/verify-landing-responsive.mjs:251-259,313-325,506-516`; `src/lib/preview-mock.ts:67-92,153-158,205-245,572-590,847-866`; `src/lib/preview-mock.test.ts`; `src/App.scenarios.test.tsx`; `src/landing/Hero.tsx:66-121`; `docs/TESTING.md`.

#### 8A — Loaded Standard

- [ ] Add the harness entry with `name: "clean"` and `label: "standard-clean"`, so it uses the supported preview query while producing distinct evidence, at 1440×900 and 1360×740.
- [ ] Drive `Back to Standard`; if `Reset & continue` appears, click it; wait for `.standard-view`.
- [ ] Require `Choose the character you want.`, `Set how strong the effect is.`, `Choose your target loudness.`, `Standard WAV`, and controls `Advanced`, `Create Master`, `Original`, and `Mastered`.
- [ ] Require `button.std-create-master` to pass `mustReach` at the minimum viewport.
- [ ] Generate `standard-clean-1440x900.png` and `standard-clean-1360x740.png` under the existing ignored evidence location; do not add them to git unless the current evidence policy explicitly requires it.
- [ ] Opt `standard-clean` into the axe helper from Task 5, extend the required `axeCoverage` matrix to six entries, and require zero A/AA violations.
- [ ] Update the scenario trace comment/assertion in `src/App.scenarios.test.tsx` so S-F1 explicitly names loaded Standard and its `standard-clean` evidence label.

#### 8B — Real album advisory

- [ ] Add a red preview-mock test for `?scenario=album-warning`: the directory picker returns `/preview/exports`; the seeded project contains source rates `[44_100, 48_000]`, source channels `[1, 2, 4]`, and `overridePositions: [3]`; the report renders 48 kHz stereo with four records and track 3 `override_album: true`.
- [ ] Separate album mock behavior into `albumDirectory: "path" | "cancel"`, `albumReport: "clean" | "advisory"`, and scenario-owned per-position source format metadata. Keep the base scenario conservative; for `album-warning`, seed positions as 1 = 44.1 kHz mono, 2 = 48 kHz stereo, 3 = 48 kHz four-channel, 4 = 48 kHz stereo, with `overridePositions: [3]`.
- [ ] Apply that source format metadata when `previewProject()` constructs `ImportedTrack` records. In `render_album_plan`, join plan positions to seeded tracks by track ID/path and derive source rates/channels from those `ImportedTrack` records; derive `override_album` from the actual `request.tracks[]`, not from the seed. Add a mock unit leg whose request override differs from the seed so broken UI-to-render wiring cannot be masked. Do not fabricate an unrelated report or hand a track-only `run_export_checks` result to Album.
- [ ] Add an `album-ready` settle condition requiring the expected track count and no `analyzing…` text.
- [ ] Drive the real `Export Album` button, wait for `Last export:`, and scroll `.album-export-receipt` into view.
- [ ] Assert the visible advisories `Upsampled source 44.1 kHz`, `Upmixed source mono`, `Folded source 4 ch to stereo`, and `Override: track 3 rendered with its own settings` at both desktop viewports.
- [ ] Keep the compatibility query `?scenario=album-warning`, but set its evidence label/purpose to `album-advisory` and describe only the supported post-export advisory behavior.

#### 8C — Honest landing CTA evidence

- [ ] Change the CTA measurement helper to record left, right, top, and bottom containment independently for both nav and hero actions.
- [ ] Keep candidate gating aligned with the current pre-activation classification: require the fixed nav CTA on both axes and the hero CTA horizontally unclipped at all 13 viewports; record hero top/bottom/all-axis containment non-blockingly in `summary.json`.
- [ ] Add one dated owner question to `docs/OWNER_INPUT_QUEUE.md`: must the **hero** CTA itself be above the fold on short phones, or is the fixed nav CTA the acquisition guarantee while the hero CTA remains below introductory copy?
- [ ] Rename the current candidate assertion/message to `hero CTA horizontally unclipped` and document that the fixed nav CTA is the all-axis acquisition guarantee pending the Task 14 owner decision. This keeps Task 12 headless green without silently deciding taste.
- [ ] In Task 14, if the owner requires hero-above-fold, move the hero CTA group directly after the headline, tighten phone-only gap/padding, retain paragraph/proof points in the DOM, and promote hero all-axis containment to blocking. If the owner chooses copy-first, retain layout and make the nav/hero distinction permanent in `docs/TESTING.md`.
- [ ] Require owner visual approval of the changed mobile order if the hero-above-fold branch is selected.

**Acceptance:** Loaded Standard is permanently covered; Album proves a real supported advisory after export; landing tests say exactly what they measure; no fictional album quality modal is introduced.

**Commits:** `test(ui): cover loaded standard workflow`; `test(ui): exercise album delivery advisories`; `test(landing): make CTA containment evidence honest`; one owner-selected landing layout/docs commit in Task 14 only if the decision requires it.

### Task 9: Make frontend config selection deterministic before rebuilding

**Files:** `package.json:7-14`; `tsconfig.json:23`; `tsconfig.node.json`; `.gitignore:14-17`; ignored local `vite.config.js` and `vite.config.d.ts`; new `src/build-config.test.ts`.

- [ ] Add a red source contract requiring every Vite/Vitest script to pass `--config vite.config.ts`, requiring `tsconfig.node.json` to use `noEmit`, and rejecting ignored emitted root configs.
- [ ] Add `typecheck` as `tsc --noEmit -p tsconfig.json && tsc --noEmit -p tsconfig.node.json`.
- [ ] Change `dev`, `build`, `preview`, `test`, and `test:watch` to explicitly select `vite.config.ts`; make `build` run `npm run typecheck` before Vite.
- [ ] Replace `composite: true` with `noEmit: true` in `tsconfig.node.json` and remove the now-unused project reference from `tsconfig.json`.
- [ ] Remove only the `.gitignore` entries for `vite.config.js` and `vite.config.d.ts`.
- [ ] Before deleting the two ignored local files, verify they are generated outputs corresponding to `vite.config.ts`; remove those exact files and no broader pattern.
- [ ] Run `& 'C:\Program Files\nodejs\npm.cmd' test`, `& 'C:\Program Files\nodejs\npm.cmd' run build`, and `& 'C:\Program Files\nodejs\npm.cmd' run verify:headless` from a clean shell.

**Acceptance:** A dirty local checkout cannot silently substitute emitted JavaScript config; typechecking emits no config files; CI and local scripts use the same TypeScript config.

**Commit:** `build: make Vite config selection explicit`

### Task 10: Patch the Rust unsoundness advisory and make CI fail closed on future unsound advisories

**Files:** `src-tauri/Cargo.lock`; `apps/iphone-native/rust/Cargo.lock`; `apps/android-native/rust/Cargo.lock`; `.github/workflows/ci.yml:300-333`; `src/lib/release-readiness.test.ts:47-116`; `docs/TESTING.md`.

Advisory references: [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html) and the explicitly isolated Linux GTK3 issue [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html).

- [ ] Prove the present blind spot with `cargo audit --deny unsound --ignore RUSTSEC-2024-0429 --file src-tauri/Cargo.lock`; expect RUSTSEC-2026-0190 for `anyhow 1.0.102`.
- [ ] Update only the transitive patch in all three lockfiles:

```powershell
cargo update --manifest-path src-tauri/Cargo.toml -p anyhow --precise 1.0.103
cargo update --manifest-path apps/iphone-native/rust/Cargo.toml -p anyhow --precise 1.0.103
cargo update --manifest-path apps/android-native/rust/Cargo.toml -p anyhow --precise 1.0.103
```

- [ ] Add a red static release-readiness assertion requiring all three CI commands to include `--deny unsound --ignore RUSTSEC-2024-0429 --file`.
- [ ] Add one adjacent CI comment: the exact exception is the Linux GTK3/Tauri path, Linux is deferred for this beta, and the exception must be removed when upstream leaves `glib 0.18`.
- [ ] Document the same exact-ID exception and local commands in `docs/TESTING.md`. Do not suppress a category or all warnings.
- [ ] Run all three RustSec commands and `& 'C:\Program Files\nodejs\npm.cmd' audit --audit-level=high`.

**Acceptance:** All locks resolve `anyhow >=1.0.103`; a new unsound advisory fails CI; the sole glib exception is explicit, narrow, and removable.

**Commit:** `chore(security): patch anyhow and deny new unsound advisories`

### Task 11: Reconcile active documentation and make the most important truth mechanically sticky

**Files:** `docs/OPEN_THREADS_AND_DECISIONS.md`; `docs/PRODUCT.md:157-177`; `docs/APP_BEHAVIOR.md:280-290`; `docs/BETA_TESTING.md:63-77`; `docs/RELEASE_STABILIZATION.md`; `docs/CAPABILITY_EVIDENCE_MATRIX.md`; `docs/IDEAS_BACKLOG.md:160-171`; `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`; `docs/plans/beta-go-no-go.md`; `docs/plans/2026-08-31-owner-launch-checklist.md`; `docs/CHANGELOG.md`; `src/lib/release-readiness.test.ts`.

- [ ] Add red static assertions for the few launch truths whose drift would cause unsafe action: Adaptive Compressor is built but gated off; owner listening was approved and only one final installed-build spot-check remains; shipped updater/navigation entries are absent from the idea backlog; capability rows C-05, C-07, C-25, and C-26 are Applied.
- [ ] Replace the stale “everything the agent lane could do is done” and beta.1-ready banners with `audit-blocked / NO-GO pending owner disposition` and a link to this plan. Do not call beta.1 rejected before Task 12's owner decision.
- [ ] Preserve beta.1 evidence in the ledger as frozen current-candidate evidence pending disposition. If the owner later rejects it, append a dated superseded marker; never rewrite it as beta.2 proof.
- [ ] Move closed listening rows out of any table labeled OPEN while preserving their dated history.
- [ ] Correct PRODUCT: listening is approved; Adaptive Compressor exists but remains gated off; active Tier-1 restraint is distinct from disabled per-band adaptive processing.
- [ ] Correct APP_BEHAVIOR/BETA_TESTING: presets are approved, further retuning requires a new listening note, and the remaining sound gate is one final installed-candidate check.
- [ ] Correct the capability matrix rows to match already-shipped proof copy, receipt labeling, beta terms, and testing guidance.
- [ ] Remove only the already-shipped updater and navigation-machine ideas from the idea backlog. Keep the stale Graphify item open until Task 21.
- [ ] In `docs/RELEASE_STABILIZATION.md:84-87,249-254`, reconcile the contradictory sweep/status wording only; preserve shipped-history entries and do not rewrite unrelated waves.
- [ ] In the active quality plan's C4/C5 ledger and freeze block around U14–U17, record beta.1 as audit-blocked pending disposition, name this remediation branch/plan, and leave U15–U17 open. Do not edit executed C1–C3 history.
- [ ] Update beta go/no-go, owner checklist, and CHANGELOG with the exact remediation commits and remaining owner gates after Tasks 1–10 are green.
- [ ] Run `& 'C:\Program Files\nodejs\npm.cmd' test` and `git diff --check`.

**Acceptance:** No active document calls beta.1 launch-ready, no closed listening decision is presented as open, no new candidate evidence is borrowed from the old SHA, and tests guard the highest-risk truth statements.

**Commit:** `docs: reconcile launch truth after adversarial audit`

---

## Phase B — Exact-Commit Candidate and Owner Reality Lane

### Task 12: Verify the remediation commit, obtain beta.1 disposition, and build beta.2

**Files:** release-bound files from Tasks 1–11; `docs/plans/beta-go-no-go.md`; `docs/plans/2026-08-31-owner-launch-checklist.md`; GitHub CI/Release evidence.

- [ ] Pre-qualify the local remediation branch. Run `git status --short --branch`, `git diff --check`, record `git rev-parse HEAD`, and run the complete local gate below. This is evidence for owner disposition, not yet the final candidate SHA because the disposition ledger and merge still follow.

```powershell
& 'C:\Program Files\nodejs\npm.cmd' test
& 'C:\Program Files\nodejs\npm.cmd' run build
& 'C:\Program Files\nodejs\npm.cmd' run build:windows
& 'C:\Program Files\nodejs\npm.cmd' run verify:headless
& 'C:\Program Files\nodejs\npm.cmd' run verify:landing-assets
& 'C:\Program Files\nodejs\npm.cmd' audit --audit-level=high

Set-Location src-tauri
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

- [ ] Run the real-fixture lane because this is a replacement release candidate, even though no DSP behavior was intentionally changed:

```powershell
Set-Location src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
try {
  cargo test --target-dir target\codex-rc
} finally {
  Remove-Item Env:\AMS_RUN_REAL_FIXTURE -ErrorAction SilentlyContinue
}
```

- [ ] Run `& 'C:\Program Files\nodejs\npm.cmd' run verify:iphone`; run Android host tests and `cargo ndk -t arm64-v8a --platform 29 check` with the documented JDK/SDK/NDK toolchain.
- [ ] Run the three exact RustSec commands from Task 10.
- [ ] Inspect fresh Advanced minimum-height, Standard, warning receipt, album advisory, and 320×568 landing evidence. Record owner-taste items as owner gates, not mechanical passes.
- [ ] Present the reproduced blocker evidence and request the owner's explicit beta.1 disposition. If the owner does not reject beta.1, stop: do not merge, push, retag, or relabel its evidence.
- [ ] After rejection, append a dated superseded marker to beta.1 evidence without erasing it and commit that marker on the remediation branch.
- [ ] Request remediation-merge authorization. Update local `main` to the reviewed base, then use `git merge --ff-only codex/launch-readiness-remediation`; if fast-forward is impossible, stop and reconcile/review rather than creating an untested merge commit.
- [ ] Prove local `main` and the remediation branch resolve to the same SHA. Freeze that final prospective tag SHA and rerun **every** local command above, the real-fixture lane, iPhone lane, Android host/ARM64 lane, RustSec commands, screenshot inspection, `git diff --check`, and clean-status check. Make no release-bound or ledger change after this final gate and before tagging.
- [ ] Request main-push authorization. Push that exact main SHA; a local/remote feature-branch push is not a substitute. Watch all seven push-to-main CI jobs to completion and record their URLs and exact SHA.
- [ ] Request authorization to create annotated tag `v0.9.2-beta.2`. Use a signed tag only if the owner already has an appropriate signing identity.
- [ ] Verify the local annotated tag peels to the tested commit with `git rev-parse "v0.9.2-beta.2^{commit}"` before requesting separate authorization to push the tag.
- [ ] Watch the tag-triggered Release workflow. Require four green jobs and a still-draft release containing exactly the expected nine assets.
- [ ] Independently download and hash MSI, setup EXE, DMG, and `SHA256SUMS.txt`; validate `latest.json` version `0.9.2`, beta.2 URLs, and all three updater signatures.
- [ ] Read the release through the GitHub API and record `draft: true`, `prerelease: false`, `published_at: null`, tag name, target field, created/updated timestamps, and exact asset names/sizes. A draft is mutable; the peeled tag commit and downloaded hashes are the identity evidence.
- [ ] Record beta.2 evidence in a ledger-only commit permitted by the freeze. Request separate authorization to push that docs commit, then require its CI run green; candidate artifact evidence remains tied to the earlier tagged SHA.
- [ ] Delete the superseded beta.1 draft only after beta.2 is fully verified and only with separate destructive-action authorization. Retain its tag/history.

**Acceptance:** the beta.2 annotated tag peels to the exact locally/CI-tested SHA; the mutable draft readback and downloaded bytes match it; the draft is unpublished; beta.1 is called rejected/historical only after the recorded owner disposition; the post-tag ledger commit has its own green CI.

**Commits:** implementation commits from Tasks 1–11, then `docs(release): record beta.2 candidate evidence` after the tag workflow.

### Task 13: Close installed-build, responsiveness, listening, and updater gates

**Files:** `docs/plans/beta-go-no-go.md`; `docs/plans/2026-08-31-owner-launch-checklist.md`; `docs/OWNER_INPUT_QUEUE.md`; owner-controlled machines, updater key backup, and private audio.

- [ ] Owner installs beta.2 MSI/EXE on Windows and universal DMG on the M4 Mac; verifies version/build stamp and launches each installed build.
- [ ] Owner verifies real-time/near-real-time audition responsiveness, Original/Mastered playhead preservation, and no obvious installed-WebView layout failure on both platforms.
- [ ] Owner performs the single final by-ear check on an ordinary track and an already-mastered track, then listens to one exported file. Record only the owner's conclusion, not private audio.
- [ ] Owner confirms beta end date, founder-window terms, whether the legal drafts ship as written, public origin, and a reachable launch contact. Publication makes the binaries public even while the landing remains closed, so these decisions precede publication.
- [ ] Record the 0.9.0 seed's SHA-256, displayed build stamp/source SHA, configured updater endpoint, and embedded permanent public key. Confirm the owner-controlled production signing-key backup/recovery evidence without exposing private material.
- [ ] Record the exact evidence Tasks 2–3 really provide: replayable availability, command-boundary offline/rejection/no-op recovery, single-flight retry, fixed-origin manual recovery, and artifact signature verification. Do not call those installed lifecycle evidence.
- [ ] The repo has no isolated updater lifecycle harness, and beta.2 cannot update itself without a newer signed feed. Add an owner-queue gate for the remaining U16 matrix: installed/staged offline, generic install failure, malformed manifest, wrong origin, missing artifact, bad signature, interrupted install, and rollback. Default is blocking; proceed only after either (a) those cases run on a disposable install against an owner-authorized isolated signed feed, or (b) the owner explicitly accepts and records the residual beta risk in `docs/plans/beta-go-no-go.md`. Never corrupt the sole installed candidate.
- [ ] Do **not** claim the new failure UI was observed in the old 0.9.0 seed; that binary cannot contain beta.2's frontend fix.
- [ ] Obtain explicit public-release GO and publication authorization. Also obtain advance rollback authorization to return beta.2 to draft and keep `RELEASE_METADATA = null`/`noindex` if the updater transaction fails. Do not describe publication as private or merely a test; it creates a public `/releases/latest` release.
- [ ] Publish the full non-prerelease beta.2 release.
- [ ] Immediately read `/releases/latest` and live `latest.json`; require beta.2, the exact URLs/signatures/hashes already verified, `draft: false`, `prerelease: false`, and a non-null publication timestamp.
- [ ] Install the existing 0.9.0 seed and prove the real happy-path 0.9.0 → 0.9.2 transaction: discovery, signed download, install, relaunch, and version check.
- [ ] If discovery/install/relaunch fails, execute the pre-authorized rollback below, verify `/releases/latest` no longer returns beta.2, verify the updater manifest is no longer selected, keep the landing closed, and stop. GitHub's release API supports returning a release to draft; retain the tag and local verified assets. See <https://docs.github.com/rest/releases/releases#update-a-release>.

```powershell
$Beta2ReleaseId = (& gh api repos/DanielKinsner/yes-master/releases/tags/v0.9.2-beta.2 --jq .id).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($Beta2ReleaseId)) {
  throw "Could not resolve the beta.2 release ID; rollback did not start."
}

& gh api --method PATCH "repos/DanielKinsner/yes-master/releases/$Beta2ReleaseId" -F draft=true | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw "GitHub did not return beta.2 to draft."
}

$DraftState = (& gh api "repos/DanielKinsner/yes-master/releases/$Beta2ReleaseId" --jq .draft).Trim()
if ($LASTEXITCODE -ne 0 -or $DraftState -ne "true") {
  throw "Rollback verification failed: beta.2 is not draft."
}

$LatestRaw = & gh api repos/DanielKinsner/yes-master/releases/latest --jq .tag_name 2>&1
$LatestExit = $LASTEXITCODE
$LatestText = ($LatestRaw | Out-String).Trim()
if ($LatestExit -eq 0) {
  if ($LatestText -eq "v0.9.2-beta.2") {
    throw "Rollback verification failed: beta.2 is still selected as /releases/latest."
  }
} elseif ($LatestText -notmatch "HTTP 404|Not Found") {
  throw "Rollback verification failed without the expected latest-release 404: $LatestText"
}

$CacheBust = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
$ManifestUrl = "https://github.com/DanielKinsner/yes-master/releases/latest/download/latest.json?rollback_check=$CacheBust"
$ManifestResponse = Invoke-WebRequest -Uri $ManifestUrl -SkipHttpErrorCheck
if ($ManifestResponse.StatusCode -notin 200, 404) {
  throw "Rollback verification failed: updater manifest returned HTTP $($ManifestResponse.StatusCode)."
}
if ($ManifestResponse.StatusCode -eq 200 -and $ManifestResponse.Content -match "v0\.9\.2-beta\.2") {
  throw "Rollback verification failed: the live updater manifest still selects beta.2."
}
```
- [ ] Record honestly that the real seed proves only the transaction; its old frontend cannot prove beta.2's new recovery presentation.
- [ ] Update exact-commit evidence and stop on any failed install, responsiveness, listening, or update result.

**Acceptance:** Both installed platforms run; the owner approves responsiveness and sound; updater evidence is labeled at its true layer; the installed/staged negative matrix is evidenced or its residual risk is explicitly accepted in beta-go-no-go; terms/origin/contact/legal decisions are recorded before public release; live latest metadata matches verified bytes; and a real 0.9.0 seed updates successfully. A failed transaction followed by safe withdrawal is a correctly handled **NO-GO**, not Task 13 acceptance, and Task 14 must not begin.

**Commit:** `docs(release): record installed beta2 reality-lane evidence`

### Task 14: Resolve origin/terms, activate the landing page, and define the public-beta lifecycle

**Files:** `index.html:12,29,38-46`; `src/landing/LandingMeta.test.ts`; `src/landing/BetaSignup.tsx:88`; `src/landing/BetaSignup.test.tsx`; `src/landing/BetaTerms.tsx`; `src/landing/release-config.ts`; `scripts/verify-public-beta-availability.mjs`; `.github/workflows/public-beta-availability.yml`; availability tests; new `docs/plans/public-beta-closure-runbook.md`; legal documents; owner checklist; Vercel project/DNS.

- [ ] Keep `RELEASE_METADATA = null`, `noindex`, and no announcement until Task 13 is green.
- [ ] Implement the recorded public-origin decision:
  - `yesmaster.app`: add it to the existing Vercel project with separate DNS authorization, apply Vercel-provided records, prove DNS resolution plus Vercel Verified/TLS, and send/receive a real message through the MX/SPF/DKIM/DMARC-configured `hello@yesmaster.app` mailbox.
  - `https://yes-master.vercel.app/`: update canonical, OG URL/image, Twitter image, signup error contact, and legal contact to that origin plus an owner-supplied reachable email; a working web origin is not mailbox evidence.
- [ ] Strengthen `LandingMeta.test.ts` to require the exact selected origin and coherent canonical, `og:url`, `og:image`, and `twitter:image` values.
- [ ] Resolve Task 8C's hero-CTA decision and obtain owner visual approval if copy order changes.
- [ ] Populate `RELEASE_METADATA` only from the published, independently verified beta.2 assets, hashes, sizes, timestamps, and beta end date.
- [ ] Keep the activation commit limited to landing/configuration/tests/monitoring/docs. Record both identities: desktop artifacts remain bound to the beta.2 tagged SHA, while the public site is bound to the later activation SHA. If any release-bound desktop file changes, invalidate the candidate and return to Task 12.
- [ ] Remove `<meta name="robots" content="noindex,nofollow" />` in the same activation change; retain the existing two-direction release-state test so indexability cannot flip early or remain closed after verified activation.
- [ ] Add a tested `verify-public-beta-availability.mjs` that reads the committed release metadata and selected origin, then checks the two artifact URLs, GitHub API asset names/sizes/digests, release draft/prerelease/latest state, `latest.json`, install guide, issue/feedback forms, newsletter link, and beta-end state. Initial activation still downloads and hashes the two installers; the recurring job may compare GitHub's recorded SHA-256 digest to avoid downloading large installers daily.
- [ ] Give that verifier explicit lifecycle modes derived from committed state and UTC time: before `betaEndsAt`, `published` means the exact beta.2 release and artifacts must remain available; at/after `betaEndsAt`, the job must fail with a named-owner withdrawal-required message until the committed state is `withdrawn` and beta.2 is no longer public/latest. After closure, Unavailable/noindex, inaccessible beta.2 artifact URLs, and a non-beta.2-or-404 `latest.json` are the expected green state.
- [ ] Add `public-beta-availability.yml` with `workflow_dispatch` and daily `17 15 * * *` schedule, immutable checkout/setup-node SHAs already used by the release workflow, one failing job as the `DanielKinsner`/repository-owner alert, and no write permission. Emit a visible T-7-day warning in the job summary, then fail at expiration until closure evidence is committed. Never auto-mutate a release, tag, deployment, DNS record, or repository setting.
- [ ] Before launch, create and obtain owner approval for `docs/plans/public-beta-closure-runbook.md`. It must name the exact beta tag/release ID/artifact URLs, `betaEndsAt`, responsible owner, read-only checks, separate closure authorizations, rollback, evidence locations, and the promise already made in `BetaTerms.tsx`: installed builds keep working after beta distribution closes and there is no kill switch.
- [ ] The Vercel CLI is not installed on this machine. Strongly recommend the owner separately authorize `& 'C:\Program Files\nodejs\npm.cmd' install --global vercel` (the explicit-executable form of `npm i -g vercel`); then separately authorize login/link and any `vercel env pull` secret-material access. Verify the linked team/project before use.
- [ ] Choose one production delivery path: the existing Git-connected Vercel deployment. The CLI may inspect environment/project/logs, but this plan does not also run `vercel deploy` or `vercel promote`.
- [ ] Before activation edits, fast-forward `codex/launch-readiness-remediation` to the current post-tag-ledger `main`. If it cannot fast-forward cleanly, stop and reconcile before writing the activation commit.
- [ ] Run tests, commit the complete activation change on the remediation branch, and record its SHA. Request preview-deployment authorization, then request branch-push authorization while explicitly noting that the Git connection will create that preview. Verify the preview and bind evidence to its deployment ID and source SHA.
- [ ] After preview passes, owner records final landing GO and separately authorizes the activation merge. Merge with `git merge --ff-only codex/launch-readiness-remediation`; require `git rev-parse main` to equal the preview source SHA. If equality is impossible, stop, construct/test/preview the new final commit, and repeat this gate.
- [ ] Separately request main-push authorization and authorization for the automatic production deployment caused by that push. Only after both approvals, push the exact previewed main SHA; record production deployment ID and source SHA.
- [ ] After that exact SHA is live, run exactly one production smoke command matching the recorded origin:

```powershell
# Run exactly one, matching the recorded owner decision.
& 'C:\Program Files\nodejs\npm.cmd' run verify:landing-smoke -- https://yesmaster.app/
& 'C:\Program Files\nodejs\npm.cmd' run verify:landing-smoke -- https://yes-master.vercel.app/
```

- [ ] Verify `/`, `/app`, `/og-card.jpg`, release links, install guide, feedback forms, canonical/social metadata, and both initially downloaded artifact hashes. Run the availability workflow manually once and require green.
- [ ] Before production push, obtain advance rollback authorization. If production smoke/monitor fails, create a normal revert of the activation commit, request/consume the authorized rollback push, verify Vercel returns to the prior Unavailable/noindex deployment, keep the GitHub release state explicit, and stop the announcement.
- [ ] With separate admin authorization, owner may enable Dependabot alerts/security updates, secret scanning/push protection, and private vulnerability reporting; read settings back afterward. Keep protected-main/ruleset work in Task 21 so a misnamed required check cannot block activation.
- [ ] Owner separately authorizes announcement/public outreach only after production smoke and the first monitor run are green.
- [ ] At `betaEndsAt`, stop and request explicit beta-closure authorization; the schedule and the pre-approved runbook do not themselves authorize mutation. Target only `v0.9.2-beta.2`: if a later stable release exists, do not alter it. Return beta.2 to draft (or use another owner-approved GitHub mechanism that makes every exact beta.2 artifact URL unavailable), preserve its tag/history/evidence, and do not delete verified local artifacts.
- [ ] In a separately tested, previewed, and authorized closure commit, retain the immutable beta metadata but set `publication: "withdrawn"`, restore `noindex,nofollow`, and keep the landing's resolver/UI explicitly Unavailable. Repeat the same preview-SHA-equals-main-SHA, separate merge, main-push, automatic-production-deploy, and smoke authorization sequence used for activation.
- [ ] Prove closure from an anonymous client with cache-busting: `/releases/latest` is not beta.2 or returns 404; `latest.json` does not select beta.2 or returns 404; each exact beta.2 installer/signature URL is unavailable; the production landing is Unavailable and noindex; the daily monitor is green in withdrawn mode; and an already-installed build launches offline without a kill switch. Record the release, deployment, commit, and owner evidence without deleting launch history.

**Launch acceptance:** One canonical HTTPS origin is live and coherent; release links/hashes point to the verified public candidate; terms, legal state, and contact details are real; activation is a committed/tested SHA deployed through one path; noindex is removed only with verified metadata; production rollback is executable; the daily read-only monitor and first manual run are green; the owner-approved closure runbook exists; and the owner records final GO.

**Closure acceptance:** After the owner-authorized beta end, beta.2 is not public/latest, its direct public artifact URLs are unavailable, the site is withdrawn/noindex, the installed app still launches offline, the monitor is green in withdrawn mode, and immutable tag/evidence history remains intact.

**Commits:** Create `release: activate verified public beta landing` before launch preview. At the authorized beta end, create `release: close public beta distribution`. Preview, merge, push, and deploy each reviewed commit only after its own separate owner gates above.

---

## Phase C — Post-Launch Elegance and Maintenance Debt

Run this phase after the beta is stable. Every behavior/configuration change starts with a red test where applicable; prose corrections, generated artifacts, and owner decisions use the named existing verification instead. Nothing in this phase may change approved audio output.

### Task 15: Establish one production settings factory and one test builder

**Files:** `src/hooks/useTrackMaster.ts:121-163,1467`; `src/lib/preview-mock.ts:205-245`; `src/lib/tone-reset.ts:11-19`; `src/components/StandardView.tsx:677-715`; `src/App.tsx:2098-2113`; representative settings-heavy tests; new `src/lib/default-settings.ts`, `src/lib/default-settings.test.ts`, and `src/test-utils/mastering-settings.ts`.

- [ ] Add red tests proving Volume Match defaults off, compression mode is `preset`, adaptive strength equals `ADAPTIVE_STRENGTH_DEFAULT`, EQ frequencies equal `EQ_BAND_DEFAULTS`, and two factory calls return distinct `advanced` and `eq_bands` objects.
- [ ] Move the complete production literal from `useTrackMaster.ts:121-163` into `createDefaultMasteringSettings()` without changing a value. Export `DEFAULT_INTENSITY = 0.5` from the same module.
- [ ] Keep a module-local default object only for immutable read fallbacks; use a fresh factory result for every newly imported track.
- [ ] Make `preview-mock.ts` consume the factory, including explicit `compression_mode` and `adaptive_strength`.
- [ ] Replace the duplicate intensity constant in tone reset, Standard, and Advanced with `DEFAULT_INTENSITY`.
- [ ] Add `makeMasteringSettings(overrides)` under `src/test-utils`, deep-merging only `advanced` and `eq_bands` over a fresh factory result.
- [ ] Migrate ordinary test fixtures in small groups. Keep explicit literals where missing optional fields, serialized wire shape, backward compatibility, or schema drift is the behavior under test.
- [ ] Run frontend tests/build/headless after production migration and after each fixture group.

**Acceptance:** One production module owns defaults; preview and real app begin from semantically identical fresh objects; tests remain explicit where shape drift matters.

**Commits:** `refactor(settings): centralize production defaults`; `test: share mastering settings fixtures`.

### Task 16: Centralize transport telemetry resets and remove write-only `transport.deviceLost`

**Files:** `src/hooks/useTrackMaster.ts:362-393,735-776,1490-1502,1609-1621,1696-1708,2241,2280,2666`; `src/hooks/useTrackMaster.integration.test.tsx`; new `src/lib/transport-state.ts` and test.

- [ ] Add red helper tests proving a track-boundary reset stops playback, clears every meter/spectrum sentinel, resets position/loop, preserves playback kind/Volume Match/export preferences, and returns fresh nested telemetry values.
- [ ] Add a second mode proving device loss can preserve event position and loop while stopping and silencing playback.
- [ ] Implement `resetTransportTelemetry(state, { currentTimeSec, resetLoop })` in `src/lib/transport-state.ts`; spread preserved state and explicitly reset `isPlaying`, position, loop, peak left/right/combined, all three compression-reduction bands, momentary/integrated LUFS, and spectrum.
- [ ] Replace the new-import, selection, removal, and device-loss blocks with the helper. Keep late-tick and loop behavior unchanged.
- [ ] In a separate commit, delete `deviceLost` from `TransportState` and its six writes. Keep the local backend-tick variable because it still controls `isPlaying` and the canonical `playbackDeviceLost` banner.
- [ ] Change tests that read `transport.deviceLost` to assert banner state, stopped playback, preserved position, silenced telemetry, backend-latch dismissal, and no revival on the next tick.
- [ ] Run focused hook integration tests, full frontend tests, build, and headless.

**Acceptance:** `rg "transport\.deviceLost" src` returns no usage; four reset blocks have one owner; all device-loss behavior remains mechanically covered.

**Commits:** `refactor(transport): centralize silent telemetry resets`; `refactor(transport): remove duplicate device-lost state`.

### Task 17: Correct stale Volume Match comments and remove only proven residue

**Files:** `src/hooks/useTrackMaster.ts:486-509,1331-1332,2221-2223`; `src-tauri/src/types.rs:629-634`; `src/bindings.ts:263-266`; `src/App.css:57,1216-1224,3519-3524`; `src/App.tsx:59`; `src/main.tsx:14`; `src/build-config.test.ts`.

- [ ] Replace contradictory Volume Match comments with one precise contract: audition attenuates deterministic estimated chain push; finite source LUFS caps that estimate at the limiter-bounded maximum plausible push; missing/non-finite source LUFS falls back to the raw estimate; export level is unchanged.
- [ ] Run the existing limiter-bound, missing-source, settings-transition, and export Volume Match tests. Do not change DSP or serialized values.
- [ ] Add a route-boundary source assertion that `main.tsx` dynamically imports `./App.css` for `/app` and `App.tsx` does not import it again.
- [ ] Delete only unused `--z-dropdown`, unreferenced `@keyframes clip-pulse`, unreferenced `.status-warn`, and the duplicate `App.tsx` CSS import.
- [ ] Re-run exact `rg` searches, layout/delight tests, build, and headless. Do not expand this into automated selector deletion.

**Acceptance:** Comments match the tested implementation; the four exact residue items are gone; landing does not eagerly load App CSS; no visual behavior changes.

**Commits:** `docs(code): align Volume Match comments with implementation`; `chore(css): remove confirmed dead residue`; `refactor(frontend): keep app CSS at the route boundary`.

### Task 18: Consolidate the proven `App.css` selector families by ownership

**Files:** `src/App.css`; `src/App.layout-css.test.ts`; `scripts/verify-app-headless.mjs`.

Current inventory at audit time: 1,268 rules, 1,164 selector/context combinations, 229 repeated groups, and 648 earlier same-selector/same-property declarations shadowed. Those counts include intentional fallbacks and are **not** deletion targets.

- [ ] Extend the clean browser scenario to record effective TOOLS background, preset-summary display/font-size/padding, and Manual compressor grid display/columns/padding/gap at both supported desktop viewports.
- [ ] Capture preset-summary values before clicking Manual; then capture the manual grid. Fail if TOOLS is transparent, summary is hidden/illegible, or the manual grid ceases to be two columns.
- [ ] Remove effective-style claims from static tests that select an arbitrary source block; keep source tests only for real structural ownership.
- [ ] Consolidate `.right-rail-tools` to one direct property owner while preserving the opaque hotfix and final effective border/radius/shadow/overflow behavior.
- [ ] In a separate commit, consolidate `.compressor-preset-summary` to one base block plus the intentional `.is-mode-off` state, preserving final computed typography, padding, hairline, and visibility.
- [ ] In another commit, consolidate `.compressor-knob-grid` and its child styling to one direct owner, preserving final two-column geometry and materials.
- [ ] After each family, compare computed records/screenshots, run focused layout/component tests and full headless, and require no same-context property to have two owners in that family.
- [ ] Recount duplicate groups after the three commits and record the delta. Do not set a global “zero duplicate selectors” goal.
- [ ] Reassess physical file splitting only after property ownership is stable; do not split merely because the file remains long.

**Acceptance:** The three confirmed contradictory families each have one intentional owner, computed behavior is unchanged except the earlier TOOLS fix, and no broad cascade migration occurs.

**Commits:** `test(ui): gate effective rail and compressor styles`; `refactor(css): consolidate right-rail tools ownership`; `refactor(css): consolidate compressor summary ownership`; `refactor(css): consolidate compressor knob-grid ownership`.

### Task 19: Restrict Tailwind to production landing sources

**Files:** `src/LandingPage.css:18`; `src/build-config.test.ts`; build/headless scripts.

- [ ] Add a red build-config assertion requiring explicit Tailwind source boundaries.
- [ ] Change the import to:

```css
@import "tailwindcss" source(none);
@source "./LandingPage.tsx";
@source "./landing/**/*.tsx";
@source not "./landing/**/*.test.tsx";
```

- [ ] Follow Tailwind's official source-detection contract: <https://tailwindcss.com/docs/detecting-classes-in-source-files>.
- [ ] Build once, add a unique Tailwind-looking token to an unrelated temporary docs/test file, rebuild, and prove the landing CSS hash is unchanged; remove the temporary file before committing.
- [ ] Run fresh `& 'C:\Program Files\nodejs\npm.cmd' ci`, `& 'C:\Program Files\nodejs\npm.cmd' test`, `& 'C:\Program Files\nodejs\npm.cmd' run build`, and `& 'C:\Program Files\nodejs\npm.cmd' run verify:headless`.
- [ ] Compare all landing viewports and confirm no production class was omitted.

**Acceptance:** Docs, tests, and Graphify cannot change generated landing CSS; production landing classes remain complete; no new dependency is added.

**Commit:** `build: scope Tailwind to production landing sources`

### Task 20: Pin CI actions to reviewed immutable commits

**Files:** `.github/workflows/ci.yml`; `src/lib/release-readiness.test.ts`; release workflow as the pattern source.

- [ ] Add a red static test that permits repository-local actions only when `uses:` begins `./`, and requires every third-party `uses:` value to end in `@` plus exactly 40 lowercase hexadecimal characters. Reject version tags, branches, and shortened SHAs.
- [ ] Reuse the already-reviewed immutable SHAs/comments from `.github/workflows/release.yml` for checkout, setup-node, and rust-toolchain when the same versions are intended.
- [ ] Resolve each remaining action's current major ref through GitHub's refs API at implementation time: `actions/cache@v4`, `actions/upload-artifact@v4`, `actions/setup-java@v4`, `android-actions/setup-android@v3`, and `gradle/actions/setup-gradle@v4`.
- [ ] If a ref resolves to an annotated tag object, dereference that tag object to its commit before pinning. Review the upstream release/tag and retain a human-readable `# vN` comment.
- [ ] Run YAML parsing, release-readiness tests, and the full CI workflow after push authorization.

**Acceptance:** Every third-party CI action is pinned to a full reviewed commit SHA; version comments remain readable; the static test prevents a mutable ref from returning.

**Commit:** `ci(security): pin continuous-integration actions`

### Task 21: Resolve owner-controlled residue and repository hygiene

**Files:** `graphify-out/GRAPH_REPORT.md`, `graphify-out/graph.json`, `graphify-out/.graphify_labels.json`; `src/assets/landing/iphone-standard-ui.jpg`; `docs/IDEAS_BACKLOG.md`; `docs/CAPABILITY_EVIDENCE_MATRIX.md`; `docs/OWNER_INPUT_QUEUE.md`; GitHub repository settings.

#### 21A — iPhone art

- [ ] Ask the owner to choose preserve, delete, or republish.
- [ ] Preserve: move it outside runtime sources to `docs/archive/assets/landing/iphone-standard-ui.jpg` and record provenance.
- [ ] Delete: remove only that exact file; note that Git history remains recoverable.
- [ ] Republish: first add capture-commit freshness proof or an explicit non-current label.
- [ ] In all branches, keep the image absent from runtime bundles until the owner selects republish.

#### 21B — Graphify

- [ ] Run only after Tasks 15–20 so the result is not immediately stale.
- [ ] Resolve the configured Graphify launcher and retry `graphify --help` with approved access or its explicit interpreter path; the audit's access-denied `uv` result proves a permission problem, not a broken launcher. Repair/reinstall only if the approved retry independently proves the launcher defective.
- [ ] Use the tool's incremental update flow and preserve `.graphify_labels.json`.
- [ ] Require valid JSON, repository-relative paths, no `C:\Users\`, `/Users/`, or `/home/` paths, and presence of current modules `FirstRunOverlay.tsx`, `debug-flags.ts`, `release-config.ts`, `SourceInsight.tsx`, and `useNavigationMachine.ts`.
- [ ] Record source commit, tool version, scope, and point-in-time/non-authoritative status in the report or `graphify-out/README.md`.
- [ ] If portable regeneration still fails, ask the owner whether to untrack all three artifacts; do not keep presenting stale June output as current architecture.

#### 21C — Optional product/repository decisions

- [ ] Leave single-key shortcuts unchanged unless the owner reopens remapping/settings after beta.
- [ ] If not completed before launch, ask the owner about Dependabot, secret scanning/push protection, private vulnerability reporting, a minimal protected-main ruleset, repository description/topics, and whether legal drafts ship as written.
- [ ] Update the owner queue and idea backlog only after each decision/action is real.

**Acceptance:** No owner asset is silently destroyed, architecture artifacts are either current and portable or explicitly retired, and external repository changes are evidenced rather than inferred.

**Commit:** one standalone commit per owner decision; Graphify refresh uses `chore(graphify): refresh portable architecture map`.

---

## Final Verification Matrix

| Evidence | Replacement candidate | Owner/public lane | Post-launch refactors |
|---|---:|---:|---:|
| `npm test` | required | required after activation metadata | required per task |
| `npm run build` | required | required after activation metadata | required per rendering/build task |
| `npm run build:windows` | required | installed Windows proof | when desktop release-bound files change |
| `npm run verify:headless` | required + screenshot inspection | production smoke is additional | required per visual task |
| Rust fmt/clippy/test | required | tied to candidate SHA | required for Rust changes |
| iPhone/Android bridge lanes | required because shared manifests/locks change | CI evidence | required when shared types/manifests change |
| Real fixture lane | required for recandidate confidence | owner listening remains separate | required only for DSP/export changes |
| CI | seven green jobs on exact SHA | release workflow four green jobs | green before merge |
| Installed Windows/M4 | not substitutable by agent | owner required | as risk warrants |
| By-ear | not substitutable by tests | owner required once pre-activation | required before any later sound change |
| Updater transaction | command-boundary tests plus staged lifecycle matrix or recorded owner exception | real 0.9.0 → 0.9.2 owner proof | repeat for later releases |
| DNS/deploy/publication | prohibited without permission | owner/authorized executor | not applicable |
| Beta withdrawal | not part of candidate creation | owner-authorized close, anonymous/cache-busted proof, installed offline launch | not applicable |

## Plan Self-Review

- [x] Every confirmed audit finding is mapped to a task or an explicit owner decision.
- [x] Candidate blockers are separated from post-launch elegance work.
- [x] Every behavior change begins with a red test and names focused/full verification.
- [x] Commit boundaries are narrow and release/destructive actions have explicit approval stops.
- [x] Prior green evidence is not reused across a release-bound code change.
- [x] No DSP retune, adaptive enablement, owner-art deletion, Graphify untracking, push, tag, publish, deploy, DNS, or repository-setting change is implied.
- [x] The plan treats browser, real-device, listening, and owner-taste evidence as different lanes.
- [x] Public activation, monitoring, rollback, beta withdrawal, and the installed-app no-kill-switch promise each have an explicit evidence and authorization boundary.
