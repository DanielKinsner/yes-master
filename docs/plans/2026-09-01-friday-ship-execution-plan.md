# Friday Ship Execution Plan (2026-09-01)

**What this is.** The decision-complete work list that turns the 2026-09-01
whole-product ship review into commits, for an executor session (any capable
agent) to run without making owner decisions. It serves the live release gate
`docs/plans/beta-go-no-go.md` and the owner's click-by-click
`docs/plans/2026-08-31-owner-launch-checklist.md`; it does not replace either.

**Target.** Free public beta on Mac + Windows, **Friday 2026-09-04**.
Candidate: `v0.9.2-beta.2` (not yet tagged; see D3).

**Review it comes from.** Verdict READY WITH SMALL FIXES at `main` `5c39af4`
(all local lanes green; CI run `33561171227` all seven jobs green). Full report:
https://claude.ai/code/artifact/9e1d619d-bff9-4554-9f57-573fc25f3442 — an
executor with the Artifact tool can read it; everything needed to execute is
also in this file.

**Drift rule.** If this plan contradicts current code or docs, verify reality,
follow reality, and write the discrepancy in the deviation log (§8). Do not
improvise a new owner decision; the stop triggers in §7 are the only reasons to
contact the owner.

---

## 1. Decisions locked (owner, 2026-09-01, in session)

| # | Decision | Consequence |
|---|---|---|
| D1 | **No custom domain exists.** The owner has not bought `yesmaster.app`. Every absolute URL the landing page publishes points at the deployed origin `https://yes-master.vercel.app`. | Slice S2 rewrites the four head tags in `index.html` and drops the `hello@yesmaster.app` address from the signup error copy. Buying a domain is a post-launch owner item (queued in `docs/OWNER_INPUT_QUEUE.md` by S1). |
| D2 | **GitHub writes.** The executor may push its own commits from this plan to `main` (docs and the two small code fixes). The **17 open Dependabot PRs are closed, not merged**, with a one-line comment; nothing dependency-related merges before the tag. | S4. Reversible (a closed PR can be reopened). Post-launch backlog: merge them in CI-gated batches (§9). Owner confirmed 2026-09-01: no veto; push and close. |
| D3 | **Beta.1 disposition and the beta.2 tag are NOT decided.** | The executor never tags. It stops at Checkpoint 2 and Checkpoint 3 with the exact green SHA, and the owner records the disposition in `docs/OWNER_INPUT_QUEUE.md` (row "Beta.1 disposition") when ready. |
| D4 | **Window sizing is read from the display, three tiers.** Tier 1 (must): at startup read the primary monitor's logical work area; open at 1920×1080 centered when it fits, maximized when it does not. Tier 2 (Tier B): when the work area is below the layout floor (1360×740 logical) apply a **native** webview zoom to fit, clamped 0.8–1.0, never above 1.0 — not CSS zoom. Tier 3 (post-launch): remember window geometry across launches and a UI-scale preference. Opening maximized for everyone was rejected: it changes how the app looks on large monitors in launch week. | S3 (Tier 1), S6.5 (Tier 2), §9 (Tier 3). Owner hand-tests on the 4K office monitor at 250 % (simulates a 1536×864 laptop → maximize) and 300 % (1280×720 → zoom-to-fit). |
| D5 | **Tier B polish proceeds** after Checkpoint 2 unless the owner says stop: one commit each, in the order listed in S6. | Each Tier B commit moves the tag target; Checkpoint 3 re-reports the final green SHA. |
| D6 | **Beta guide gets a display-scaling limitation line** even though D4 ships, because the minimum window is still 1360×740 logical. | S1. |
| D7 | **Mac updater proof:** do it on Thursday if the M4 is at hand. If it has not happened by the time the owner says "publish", the executor writes the accepted risk into go/no-go §3(ii) **without asking again**, and the first patch release (0.9.3) is the Mac updater's proving run. | §5 owner lane. |
| D8 | **Out of scope for Friday:** the app-wide Style/Preset rename, loudness-control ownership in Advanced, email provider, paid signing, any DSP change, any layout change. | §10 parking lot. |
| D9 | **Vocabulary rule (decided, applied narrowly now, fully post-launch).** A **Style** is one of the eight factory characters (Universal … Loud). A **Preset** is a recipe the *user* saved ("My presets", "Save as preset"). Under that rule most current copy is already right; the one public inconsistency is the landing's "Eight presets", which becomes **"Eight styles"**. The Advanced compressor's "Preset" mode/"Preset density"/"Preset values from Universal" refer to the Style's values and are renamed in the post-launch pass, not now (tests, docs and a mode enum hang off them). | S2 (landing word); §9 (the rest). |
| D10 | **Hero CTA stays below the introductory copy on short phones.** The fixed nav CTA is the acquisition guarantee on every viewport; the hero button is a second door, not the only one. Decision closes owner-queue row T-03 as "copy-first". No layout change. | S1 records it and makes the nav/hero distinction permanent in `docs/TESTING.md`. |
| D11 | **Three small taste calls, decided:** (a) the section label "EQUALIZER (Dynamic)" becomes **"EQUALIZER"** — the chain is a static 7-band EQ with adaptive reduce-only trims, and "dynamic EQ" means a different tool to any engineer; (b) the receipt's "Mastering target" chip uses a **short profile name** ("Streaming · −14.0 LUFS"), with the long descriptive name staying in the Delivery Profile dropdown where there is room; (c) the per-band compressor card's "Effective compression" line **stays** — it is the only place attack/release are shown at idle, so it is not a duplicate. | S6.6, S6.7; (c) is a no-op. |

Plain-English "so what": the app is done; this plan removes the things that
would embarrass the launch (a share card pointing at a domain nobody owns, a
window bigger than a laptop screen, a stale step in the owner's own checklist,
an unattended-looking public repo), then adds four small user-facing polish
items, and stops before anything that publishes.

---

## 2. Lanes

**Executor lane** (this plan): S0–S7 below.

**Owner lane** (§5): install on both machines, the one by-ear check, the
Windows updater proof, the optional Mac updater proof, publishing, confirming
the beta end date, and saying the trigger phrase that unlocks the landing flip.
Date-gated: everything in the owner lane happens after Checkpoint 3.

---

## 3. Verification rules (floor, not ceiling)

- The per-slice verify lists below are the **minimum**. After every slice,
  re-read the conditional rules in `CLAUDE.md` "Verification" ("if you touch
  X, also run Y") against the **actual diff**, and run whatever they add. A
  slice that grows beyond its predicted footprint grows its verify list.
- Many frontend tests read documentation files
  (`src/lib/release-readiness.test.ts`, `version-coherence.test.ts`,
  `beta-feedback-contract.test.ts`, `LandingCopy.test.tsx`, `LandingMeta.test.ts`
  and others). **Every docs-only slice still runs `npm test`.**
- Never report a lane green that you did not run in this session, at this tip.
- Local convention for Rust: `--target-dir target\codex-rc` (see `CLAUDE.md`).
- Pull `origin/main` before every commit; another agent or the owner may have
  pushed.

---

## 4. Ordered slices

### S0 — Orient and drift-check (no commit)

1. `git fetch origin && git status -sb`; confirm `main` is at or after `5c39af4`
   and in sync with origin. If the tip moved, `git log 5c39af4..HEAD --oneline`
   and note it in the deviation log.
2. Confirm the four review facts still hold before touching anything:
   - `grep -n "yesmaster.app" index.html src/landing/BetaSignup.tsx` → 5 hits.
   - `grep -n -E '"maximized"|"center"' src-tauri/tauri.conf.json` → none.
   - `grep -n "right-click" docs/plans/2026-08-31-owner-launch-checklist.md` → Step 2.
   - `gh pr list --state open --limit 30` → the Dependabot PRs (17 on 09-01).
     If `gh` fails with an auth error on this machine, run it with
     `GITHUB_TOKEN=` cleared for that call (stale env token gotcha).
3. Run the fast frontend lane once to establish the baseline: `npm test`.

### S1 — Docs only (one commit) → **Checkpoint 1**

Files and exact changes:

- `docs/plans/2026-08-31-owner-launch-checklist.md`, Step 2, item 3: replace
  the right-click → Open instruction with: *"First launch: macOS will block
  it. Open **System Settings → Privacy & Security**, find the YES Master
  message, click **Open Anyway**, then **Open**. (Right-click → Open no
  longer bypasses Gatekeeper for unsigned apps on macOS 15 and later.)"*
  Keep the "unidentified developer … no Apple Developer account yet" note.
- `docs/BETA_TESTING.md`, "Known limitations": add one bullet after the
  minimum-window bullet: *"**High display scaling is tight.** The minimum
  window is 1360×740 logical pixels, so on a 1080p display at 150 % scaling
  the app opens maximized and fills the screen; at 175 % or above the right
  rail may not fit. Lower the display scale for that session if you hit it."*
- `docs/OWNER_INPUT_QUEUE.md`, "Open" table: add a row dated 2026-09-01,
  unit "Ship review S2", question *"Custom domain? `yesmaster.app` was
  written into the page metadata but was never purchased."*, conservative
  default *"All public URLs use the deployed origin `yes-master.vercel.app`
  (D1)."*, what changes when answered *"Buy the domain, attach it to the
  Vercel project, update the four `index.html` tags and the same-origin test,
  re-check the share card in a real chat paste."*
- **Close owner-queue row T-03 (D10).** In `docs/OWNER_INPUT_QUEUE.md`
  strike the T-03 row through in the same style as the answered rows and
  point it at a new dated line in `docs/OPEN_THREADS_AND_DECISIONS.md`
  ("2026-09-01 — hero CTA stays copy-first on short phones; the fixed nav CTA
  is the all-axis acquisition control"). In `docs/TESTING.md`, in the
  landing-lane section (search for the CTA containment / `summary.json`
  wording), add one paragraph stating the permanent rule: the nav CTA is
  gated on both axes at every viewport; the hero CTA is gated horizontally
  only and may sit below the fold on short phones by design. Do not change
  any test threshold.
- `git mv VERA-SHIP-PLAN.md docs/archive/plans/2026-07-12-vera-ship-plan.md`.
  Add one line at the top of the moved file: *"Archived 2026-09-01 — July
  snapshot, superseded by `docs/plans/beta-go-no-go.md` and
  `docs/plans/2026-09-01-friday-ship-execution-plan.md`."*
  Then `grep -rn "VERA-SHIP-PLAN" --include=*.md .` and fix any link.

Verify: `npm test` (docs are read by tests). Commit:
`docs: ship-review S1 — checklist macOS step, scaling caveat, domain queue row, archive July plan`.

**Checkpoint 1:** report the commit SHA, `npm test` totals, and the deviation
log so far. The owner grades the run before code starts. Do not wait for a
reply longer than it takes to write the report; if none arrives, continue to
S2 (the checkpoint exists so the owner *can* intervene, not so the executor
idles).

### S2 — Landing origin (one commit)

- `index.html`: change the four absolute URLs to the deployed origin:
  `rel="canonical" href="https://yes-master.vercel.app/"`,
  `og:url` = `https://yes-master.vercel.app/`,
  `og:image` and `twitter:image` = `https://yes-master.vercel.app/og-card.jpg`.
  Leave every other tag (including `noindex`) untouched.
- `src/landing/BetaSignup.tsx:88`: the error copy becomes
  *"Something went wrong — try again in a moment."* (no email address; the
  product's support channel is the GitHub forms, and this state is
  unreachable while signup is disabled).
- `src/landing/BetaSignup.test.tsx:144`: invert the pin — assert the error
  text does **not** contain `yesmaster.app` and does contain "try again".
- `src/landing/LandingMeta.test.ts`: add one test *"every absolute URL in the
  head shares one origin"* — parse canonical, `og:url`, `og:image`,
  `twitter:image`, assert all four `new URL(...).origin` values are equal and
  equal to `https://yes-master.vercel.app`. This stops a half-update.
- `src/landing/Nav.tsx:5-7`: fix the stale comment (the mobile section and
  its sentence were removed 2026-09-01; say so, do not describe it as still
  present).
- **"Eight presets" → "Eight styles" (D9)** in `src/landing/ProofDeck.tsx:90`.
  Update the C-06 claim text in `docs/CAPABILITY_EVIDENCE_MATRIX.md:47` to
  the new wording (evidence unchanged) and the matching line in
  `docs/landing-brief.md:46` ("eight styles"). Check
  `src/landing/LandingCopy.test.tsx` for a pin on the old phrase and update
  it.
- `grep -rn "yesmaster.app" src index.html docs/*.md` afterwards: the only
  remaining hits may be historical docs under `docs/plans/` and
  `docs/archive/`. `docs/landing-brief.md` and
  `docs/CAPABILITY_EVIDENCE_MATRIX.md` must carry no live claim about the
  domain; if they do, qualify it and log the deviation.

Verify: `npm test`, `npm run build`, `npm run verify:headless`. Commit:
`landing: publish head URLs from the deployed origin (D1); same-origin test`.

### S3 — Window fits the screen (one commit)

Facts verified 2026-09-01 against the locked `tauri 2.11.1` sources:
`AppHandle::primary_monitor()`, `Monitor::work_area()` (physical rect),
`Monitor::scale_factor()`, `WebviewWindow::maximize()` all exist.

- `src-tauri/tauri.conf.json`: add `"center": true` to the `main` window.
- `src-tauri/src/lib.rs` (inside the existing `#[cfg(feature = "app-runner")]`
  `run()`): add a pure helper and call it from `setup` after diagnostics init:

  ```rust
  /// True when the monitor's logical work area cannot hold the configured
  /// default window. Logical, not physical: Windows display scaling shrinks
  /// the usable canvas (1080p at 150 % is 1280×720 logical) and the app is
  /// composed for 1920×1080 logical (tauri.conf.json).
  pub(crate) fn should_maximize_for_work_area(
      work_w: f64, work_h: f64, default_w: f64, default_h: f64,
  ) -> bool {
      work_w < default_w || work_h < default_h
  }
  ```

  In `setup`: get the main window (`app.get_webview_window("main")`), its
  primary monitor; convert `work_area().size` to logical by dividing by
  `scale_factor()`; if the helper returns true, call `window.maximize()` and
  log `diagnostics::info("window maximized: work area {w}x{h} logical < 1920x1080")`.
  Every error path is ignored (a missing monitor must never block launch).
  Read the 1920×1080 constants from named consts next to the helper; do not
  read them back from the config at runtime.
- Unit tests in `lib.rs` (same `update_availability_tests` style):
  `1536×864 → true` (1080p at 125 %), `1280×720 → true` (150 %),
  `1920×1080 → false`, `2560×1440 → false`, `1920×1000 → true` (short work
  area).

Verify (from `src-tauri`): `cargo fmt --check`,
`cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings`,
`cargo test --target-dir target\codex-rc`; then from the root
`npm run build:windows` (the fast lane's packaging step; the `.msi`/`setup.exe`
it emits are dev artifacts, not release bytes). No shared crate type or command
signature changes, so the mobile bridge lanes are not triggered — re-check
that claim against the actual diff. Commit:
`desktop: maximize when the logical work area is smaller than the default window (D4)`.

Owner hand-test (§5, item 1) covers the real-display proof. Remembering the
user's last window geometry (`tauri-plugin-window-state`) is the mature next
step but is a new dependency; it is post-launch (§9), not this slice.

### S4 — Close the Dependabot PRs (no commit)

For each open Dependabot PR (17 on 2026-09-01; re-list with `gh pr list`):
`gh pr close <n> --comment "Closing until after the public beta tag; dependency bumps are merged post-launch in CI-gated batches (docs/plans/2026-09-01-friday-ship-execution-plan.md §9)."`
Do not merge any. Do not edit `.github/dependabot.yml` (weekly cadence is
fine; new PRs after launch are expected). Verify: `gh pr list --state open`
is empty (or contains only non-Dependabot PRs).

### S5 — Push and watch CI → **Checkpoint 2**

1. `git pull --rebase origin main` (stop if there are conflicts you cannot
   resolve mechanically; log them).
2. `git push origin main`.
3. Watch the CI run for the pushed SHA to completion (`gh run list --branch main --limit 1`,
   then `gh run watch <id>` or poll `gh run view <id>`). All seven jobs green
   is the bar; a red job means fix-forward on `main` with its own commit and
   re-run — do not tag around it (you are not tagging anyway, D3).
4. **Checkpoint 2 report:** the exact SHA, CI run id, every lane you ran
   locally with totals, and the deviation log. State explicitly: *"This SHA is
   tag-ready pending the owner's beta.1 disposition (D3). Tier B polish follows
   unless told to stop; the final SHA is re-reported at Checkpoint 3."*

### S6 — Tier B polish (one commit each, in this order)

Each item: write the failing test first where a test is named, make it pass,
run `npm test` + `npm run build` (add the Rust lane only if Rust changes),
commit, continue. Copy is pinned by tests throughout this repo; update the
pins deliberately and say so in the commit message.

1. **One name for import.** All three entry points say **"Import audio"**:
   Advanced sidebar footer button in `src/App.tsx` (currently "Import Audio"),
   Standard tracks rail in `src/components/StandardView.tsx:178` (currently
   "+ Add Tracks"), empty state already correct. Known ripple: several tests
   find a button by the exact text "Import audio" and may now match the
   sidebar button first (`src/App.scenarios.test.tsx:156`,
   `App.transitions.test.tsx:685`, `App.user-errors.test.tsx`); scope those
   queries to the empty state / rail they mean rather than weakening them.
   Commit: `ui: one label for import (Import audio)`.
2. **Settings App Info "Build" row.** Remove the `["Build", "Local desktop build"]`
   row from `SETTINGS_GROUPS` in `src/lib/chrome-content.ts` (Help already
   shows the real version and build stamp). Update `src/App.chrome.test.tsx`
   pins. Commit: `settings: drop the placeholder Build row (Help owns the build stamp)`.
3. **Friendlier unmapped errors.** In `src/lib/user-errors.ts` add two
   mappings before the generic fallback, keeping the raw text as `detail`:
   - prefix `render error:` → *"The render failed and no file was written. Try again; if it repeats, save a diagnostics report from Help."*
   - contains `audio device unavailable` → *"No audio output device is available. Choose an output in Settings, then press Play."*
   Tests in `src/lib/user-errors.test.ts` for both. Commit:
   `errors: map render and audio-device failures to plain language`.
4. **Say why an export was renamed.** The engine diverts to a `__{n}` sibling
   when the chosen file already exists (never-overwrite is a non-negotiable;
   keep it). Surface it:
   - `src/hooks/useTrackMaster.ts` `runExport`: after the job returns, compare
     `chosenOutputPath` with `job.output_paths[0]`; when they differ, store
     `divertedFrom: chosenOutputPath` on the `ExportReceipt` (add the optional
     field to the interface; `null`/absent otherwise).
   - `src/components/ExportReceiptCard.tsx`: under "File saved", when
     `divertedFrom` is set, one line: *"Saved as {newName} — {chosenName}
     already existed and was left untouched."*
   - `src/components/StandardView.tsx` "Master created" card: the same
     sentence as a second meta line when set.
   - Tests: one `App.*.test.tsx` case that mocks `renderTrackMaster` to
     return a different `output_paths[0]` than the save dialog path and
     asserts the sentence in both surfaces; one negative case (same path → no
     sentence). Commit: `export: tell the user when a render was saved under a new name`.
5. **Fit-to-floor zoom (D4 Tier 2).** Intent: a display whose logical work
   area is *below* the layout floor (1360×740) should still show the whole
   console rather than clipping the right rail. Mechanism: the webview's
   **native** zoom (the same thing Ctrl+minus does in a browser — crisp, and
   CSS breakpoints respond because CSS pixels scale with it), never the CSS
   `zoom` property (that is what `useWebviewZoomShortcuts` in `App.tsx`
   deliberately pins to 1 because it blurred and broke breakpoint math; leave
   that code alone).
   - First verify the API in the locked crate: `grep -n "pub fn set_zoom" ~/.cargo/registry/src/*/tauri-2.11.1/src/webview/*.rs`.
     If it is absent, stop this item and log it — do not substitute CSS zoom.
   - In the same `setup` block as S3, after the maximize decision: compute
     `factor = min(work_w / 1360, work_h / 740, 1.0)`, clamp to `>= 0.8`, and
     call `window.set_zoom(factor)` only when `factor < 1.0`; log the factor
     via `diagnostics::info`. Pure helper `fit_zoom_for_work_area(work_w, work_h) -> f64`
     with unit tests: `1280×720 → 0.941…` (height-limited: 720/740),
     `1536×864 → 1.0`, `1920×1080 → 1.0`, `1000×600 → 0.8` (clamped).
   - Revert rule (write it in the commit message): if the owner's 300 %
     hand-test says the zoomed console reads worse than a scrollbar would,
     this item is reverted, not tuned.
   - With the 0.8 clamp, a 1080p display fits down to 175 % scaling
     (1097×617 logical → factor 0.807). Update the S1 caveat in
     `docs/BETA_TESTING.md` so it says the right rail may not fit **at 200 %
     or above** (it said 175 % before this item), and run `npm test`.
   Verify: Rust lane as in S3; `npm run build:windows`. Commit:
   `desktop: native zoom-to-fit below the 1360x740 layout floor (D4 tier 2)`.
6. **"EQUALIZER (Dynamic)" → "EQUALIZER" (D11a).** `src/App.tsx:2285`
   section label. Grep `src` and `docs` for the old label (tests, TESTING.md
   scenario notes) and update pins. Commit: `ui: the equalizer label no longer says Dynamic`.
7. **Short profile names on the receipt chip (D11b).** Add
   `DELIVERY_PROFILE_SHORT: Record<DeliveryProfile, string>` beside
   `DELIVERY_PROFILE_DISPLAY` in `src/bindings.ts`:
   `streaming-universal → "Streaming"`, `apple-music → "Apple Music"`,
   `cd → "CD"`, `vinyl-premaster → "Vinyl"`, `loud-rock → "Loud Rock"`,
   `broadcast-eu → "Broadcast EU"`, `broadcast-us → "Broadcast US"`,
   `custom → "Custom"`. Use it only in
   `ExportReceiptCard.tsx` `masteringTargetLabel` (so the chip reads
   "Streaming · -14.0 LUFS"); the Delivery Profile dropdown keeps the long
   names. Update the pin at `src/components/ExportReceiptCard.test.tsx:186`
   and add a test that every profile has a short name (drift guard, same
   shape as the existing display-map tests). Commit:
   `receipt: short delivery-profile name on the mastering-target chip`.

### S7 — Records, push, watch CI → **Checkpoint 3**

- `docs/CHANGELOG.md`: one dated 2026-09-01 entry (newest first) listing
  what this plan shipped, in the ledger's milestone style.
- `docs/plans/beta-go-no-go.md` §9 ledger: one row per lane you ran at the
  final SHA (frontend-unit, native-synthetic, browser-headless, and the
  Windows package build), commit-exact, in the existing table format.
- `docs/OPEN_THREADS_AND_DECISIONS.md`: a short 2026-09-01 block pointing at
  this plan and its decisions table (D1–D8).
- `npm test` (docs are read by tests), pull, push, watch CI to green.
- **Checkpoint 3 report:** final SHA, CI run id, lanes, deviation log, and the
  same tag-readiness sentence as Checkpoint 2. **Stop.** Everything after this
  is owner lane.

---

## 5. Owner lane (after Checkpoint 3)

1. **Scaling hand-test (5 min, office PC, 4K monitor).** Settings → System →
   Display → Scale. **250 %** makes the 4K screen behave like a 1536×864
   laptop: launch the dev build (`npm run tauri dev`) or the installer from
   S3; you should see the app open **maximized**, right rail and Create
   Master visible. **300 %** behaves like a 1280×720 laptop: after S6.5 the
   app should open maximized *and* zoomed slightly smaller so nothing clips;
   before S6.5 it is maximized but tight (the documented caveat). Judge the
   300 % result by eye: crisp and complete = keep; worse than a scrollbar =
   tell the executor to revert S6.5. Set the scale back to your normal value.
   If Tier 1 does not fit at 250 %, that is a stop-and-fix for the executor,
   not a doc note.
2. **Beta.1 disposition (D3).** When ready, tell the executor (or the next
   session) *"reject beta.1, tag beta.2 at <SHA>"* using the Checkpoint 3 SHA.
   The tagging procedure is the existing U14 close-out in the go/no-go ledger
   (tag, watch the Release run's four jobs, verify 9 draft assets, download
   `SHA256SUMS.txt` + installers, compare hashes, record rows).
3. **Install + listen + updater.** Follow `docs/plans/2026-08-31-owner-launch-checklist.md`
   Steps 1–4 against the beta.2 draft (Step 2 now has the corrected macOS
   wording). Uninstall the staged beta.1 on the Windows box first — both are
   version 0.9.2, so the updater would never offer beta.2 over beta.1.
4. **Optional Mac updater proof (D7, ~45 min).** On the M4:
   `git checkout v0.9.1-beta.1 && npm ci && npm run build:mac`, install that
   0.9.1 app, then after publishing beta.2 confirm the update toast → install
   → relaunch into 0.9.2. If skipped, the executor writes the accepted risk
   into go/no-go §3(ii) when you say so.
5. **Publish** the draft (not pre-release), confirm the Windows updater proof,
   confirm the beta end date (pencilled 2026-10-31).
6. **Trigger phrase for the landing flip:** *"published, updater proven, listen
   OK, Oct 31 confirmed."* Only then does an executor fill `RELEASE_METADATA`
   in `src/landing/release-config.ts`, remove `noindex` (the test forces it),
   run `npm test` + `npm run verify:headless`, push, and verify the live page.
7. After the flip: paste the live URL into Discord or Slack and look at the
   preview card; click both downloads from a phone and a desktop; GitHub
   Settings → Code security toggles; fill the go/no-go Decision block.

---

## 6. Do-not-touch (absolute)

- `src-tauri/src/dsp.rs`, `guardrails.rs`, `confidence.rs`, `album.rs`
  constants and every preset byte-identity snapshot under
  `src-tauri/tests/golden/`. Listening was signed off 2026-08-25.
- The three OFF gates (Adaptive Compressor, Phase-B confidence, album
  character) — `src-tauri/tests/owner_gates_default.rs` enforces them.
- `.github/workflows/release.yml`, the updater and opener code paths in
  `lib.rs` (proven green 08-31; any edit reopens the proving run).
- `src/hooks/useTrackMaster.ts` structure and `src/App.css` architecture —
  edit lines the slices name, no refactors.
- Dependencies, lockfiles, Rust toolchain, Tauri/plugin versions.
- Landing layout and section order (hero-CTA question is queued, T-03).
- Anything in `docs/legal/`, pricing copy, the founder-window terms.

---

## 7. Stop-and-ask triggers (the only reasons to contact the owner)

- Tagging or publishing anything (D3 says not yet).
- A slice would change what the user hears, or any DSP-adjacent file.
- A fix needs more than the plan's footprint (e.g. S3 needs anything beyond
  maximize/center; S6.4 needs a backend change).
- CI red that is not obviously caused by this plan's own diff.
- Merge conflicts you cannot resolve mechanically.
- Anything touching pricing, legal, the beta end date, or the domain.
- Spending money.

Everything else: execute without asking.

---

## 8. Deviation log (required at every checkpoint)

At Checkpoints 1, 2 and 3, report: every judgment call you made, every plan
claim that did not match reality (with the file:line you found instead),
every ripple beyond a slice's stated footprint and the extra verification it
triggered, and any test pin you changed and why. Deviations handled silently
are how executors drift; deviations logged are how the plan gets better.

---

## 9. Post-launch backlog (not this plan)

- Merge the Dependabot bumps in three CI-gated batches: GitHub Actions majors
  (`actions/cache` 4→6, `actions/upload-artifact` 4→7 — check the workflow
  inputs changed), npm (react/react-dom patch, jsdom 25→29 is dev-only,
  tailwind vite patch, tauri cli patch), cargo (serde_json, uuid, thiserror,
  chrono, tauri-build 2.6.3, futures-executor, jni 0.21→0.22 Android-only).
  None were security fixes on 2026-09-01 (`npm audit` and RustSec clean).
- Bump the Node-20 GitHub actions (`checkout@v4`, `setup-node@v4`, `setup-java@v4`).
- **Window geometry memory (D4 Tier 3):** adopt `tauri-plugin-window-state`
  so size/position/maximized persist across launches, clamped to the
  monitors present at launch; add a "UI scale" preference in Settings that
  drives the same native zoom as S6.5; revisit the 1360×740 layout floor.
- Mac updater proving run on the first patch release if D7 was skipped.
- Vocabulary pass under the D9 rule: rename the compressor mode "Preset" →
  "Style" (mode enum stays `preset` on the wire; only labels change),
  "Preset density" → "Style density", "Preset values from Universal" →
  "Universal's values", and sweep Help/Settings copy. Tests and
  `docs/APP_BEHAVIOR.md` "Compressor Modes" follow.
- Loudness has one owner in Advanced.
- Relink missing sources; corrupt `user_presets.json` recovery;
  `engine.rs:190` `eprintln!` → `diagnostics::warn`.
- `docs/USER_GUIDE.html` is linked from nothing: link from Help or archive.
- Custom domain (D1) when bought.

---

## 10. Parking lot (explicitly not in scope)

The app-wide Style/Preset rename (D9 rule is decided; the sweep is
post-launch), loudness-control ownership in Advanced, the 1360×740
Intensity-chip clip at the floor, the hero layout (D10 decided: unchanged),
email provider, Apple/Windows paid signing, any mobile app work, any
refactor of the state hook or CSS, opening maximized by default on large
monitors (rejected under D4).

---

## 11. Execution record (2026-09-01, office PC, executor session)

**Outcome.** S0–S7 shipped on `main`; nothing tagged or published (D3).
Checkpoint 2 SHA `0f67d87` (CI run `33568823137`, 7/7 green). Checkpoint 3
SHA `59d2a0b` (CI run `33572999889`, 7/7 green). This record commit sits on
top and changes docs, one script comment, and the unlinked user guide only.

| Slice | Commit | Note |
|---|---|---|
| S1 | `c5dec99` | docs (checklist Mac step, scaling caveat, domain row, T-03 closed, Vera plan archived) |
| S2 | `c9f1b67` | head URLs from the deployed origin, same-origin test, "Eight styles" |
| S3 | `0f67d87` | maximize when the work area is smaller than 1920×1080 (D4 tier 1) |
| S4 | — | 17 Dependabot PRs closed, none merged |
| S6.1 | `25d3b53` | "Import audio" at all three entry points |
| S6.2 | `eb8330b` | Settings Build row removed |
| S6.3 | `c953bfa` | render / audio-device errors mapped |
| S6.4 | `06e8a41` | diverted-export sentence on both receipts |
| S6.5 | `5418039` | native zoom-to-fit below the floor (D4 tier 2; revert-not-tune) |
| S6.6 | `030f134` | "EQUALIZER" |
| S6.7 | `7d16b06` | short profile name on the receipt chip |
| S7 | `59d2a0b` | changelog, ledger rows, decisions pointer |

**Lanes at the final code tip `7d16b06`:** `npm test` 82 files / 844 tests
(835 at S0); `npm run build` green; `npm run verify:headless` PASSED
(landing suite + 31 `/app` checks); `cargo fmt` / `clippy -D warnings`
clean; `cargo test` 611 passed / 0 failed / 7 ignored; `npm run
build:windows` both bundles. `npm test` re-run at `59d2a0b`.

**Deviation log** (every judgment call, plan-vs-reality mismatch, and
ripple beyond a slice's footprint):

- **S0.** Tip was `1f57638`, three docs-only commits past `5c39af4`; only
  the plan file differed. All four review facts held.
- **S1.** `docs/TESTING.md` still said "both acquisition CTAs are fully on
  screen", which contradicts the new nav/hero rule; reworded to "contained
  (per the nav/hero rule below)". The beta-guide caveat described S3
  behaviour one commit ahead of the code (same push). T-03 struck in place
  in the Open table rather than moved. No Vera links existed to fix.
- **S2.** `BetaDownload.tsx` carried the same stale "mobile section above"
  comment as `Nav.tsx`; both fixed, comment-only. No `LandingCopy` pin on
  "Eight presets" existed. `docs/USER_GUIDE.html` kept the old words until
  this record commit.
- **S3.** Used `window.primary_monitor()` (same monitor as the AppHandle
  call). Added a non-finite / non-positive scale guard. Re-checked the
  mobile-bridge claim against the diff: both bridges use
  `default-features = false`, so nothing under `app-runner` reaches them.
- **S6.1.** The landing asset gate hashes every non-test component, so any
  console copy change forces `npm run capture:landing`; it rewrites only the
  three deterministic PNGs and the manifest digest, carrying the owner
  plates forward. Recaptured at S6.1, S6.4, S6.6 and S6.7. Test queries were
  scoped to the empty-state hero by container because the sidebar button
  now shares the exact text.
- **S6.2.** No existing pin on the Build row; a negative pin was added.
- **S6.3.** `audio.rs` also emits "audio output device unavailable (<name>)",
  which the plan's literal substring misses; both shapes matched explicitly.
- **S6.4.** The compare is on the file NAME, not the whole path: a divert
  only ever changes the name, and a differently spelled directory must never
  trip the sentence (pinned by the hook's negative case). Coverage went
  wider than "one App case": hook, card, Standard card, and two App-level
  cases, because no App test previously drove Create Master end to end.
- **S6.5.** The plan's aside called 1280×720 height-limited; it is
  width-limited (1280/1360 = 0.941, the value the plan gave). Degenerate
  inputs return 1.0 (no zoom) after a first test draft exposed that
  `f64::min` discards NaN.
- **S7.** The OPEN_THREADS block cites D1–D11 (the table has eleven rows,
  not eight). Ledger rows name `7d16b06` as the code tip and say why.

**Owner lane additions (beyond §5):**

- **Re-plate the three landing images (owner, 2026-09-02).** The
  real-session plates `owner-standard-session`, `owner-advanced-session`
  and `owner-album-session` still show "+ Add Tracks", "Import Audio" and
  "EQUALIZER (DYNAMIC)". Re-shoot from the same session on current `main`,
  replace by hand, and update the manifest's `ownerCaptures` block (its own
  note describes the step). Land before the landing flip (§5 item 6). The
  share card's console text is unreadable at its size; refresh it only if
  convenient.
- `docs/USER_GUIDE.html` wording was fixed here, but the guide is still
  linked from nothing (§9).
