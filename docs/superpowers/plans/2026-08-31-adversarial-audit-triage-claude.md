# Claude Triage of the 2026-08-31 Adversarial Audit

**Verified against:** `main` @ `bb414c1` (same baseline the audit names). Every verdict below was
checked in the actual code with file:line evidence, independently, before adopting the audit's framing.

**Bottom line:** The audit's diagnosis is overwhelmingly accurate. Of 30 findings, 24 are CONFIRMED
in code, 3 are PARTIAL (right defect, wrong detail or overstated severity), 3 could not be verified
locally (owner/GitHub-side or skipped as low-risk). Nothing was refuted outright. The remediation
plan (Tasks 0–21) is sound to execute with the amendments listed at the end.

## Verdict table

| ID | Finding | Verdict | Evidence / note |
|---|---|---|---|
| L-01 | Import metadata parser bypasses panic boundary | ✅ CONFIRMED | `files.rs:92-107` runs its own Symphonia probe with no `catch_unwind`; boundary exists only in `decode.rs:99-116` (applied at :119, :346, :476). The 0 Hz panic is real and known — `symphonia-core units.rs:147-152` `TimeBase::new` panics on zero; CHANGELOG:352-354 records the identical bug fixed on the decode paths. `decode_hostile.rs` never covers `import_tracks`. NOTE: the duration arithmetic at `files.rs:113-116` is correctly guarded (`if sr > 0`, f64 division) — the panic is inside Symphonia's `.format()` call, not our math. |
| L-02 | Updater startup event can be missed | ✅ CONFIRMED | `lib.rs:259-269` fire-and-forget `emit`, version never stored; `App.tsx:339-349` listener attaches via async promise post-mount; no query command in `generate_handler!` (:188-230). Practical race window is small (needs GitHub round-trip to beat React mount), so "blocker" is arguable — but the fix is cheap and the failure is silent, so fix it. |
| L-03 | Updater failure clears notice, no retry/manual path | ✅ CONFIRMED | `App.tsx:351-358`: `.catch(... setUpdateAvailable(null))` — sole render gate for the toast (:595-608). Only `console.warn` sees the failure. Code comment shows it was a *deliberate* choice ("just dismisses the toast — never a modal"), but it's the wrong choice for launch: combined with L-02's no-replay, the user can never recover the notice without an app restart. |
| U-01 | Sticky TOOLS transparent; static test false-green | ✅ CONFIRMED | `App.css:7601-7605` sets `.right-rail-tools { background: transparent }` at top level (no media query) — last unconditional same-specificity rule wins over the opaque rules at :4086 and :6570. `App.layout-css.test.ts:236` asserts against the *first* matching block (:4080), hence false green. The comment at :4083 ("Opaque, not 42%... must not be see-through") documents the intent the cascade currently violates. |
| A-01 | Advanced loudness `<select>` unnamed | ✅ CONFIRMED | `App.tsx:2224-2228` — no aria-label, no id/htmlFor; "LOUDNESS TARGET" is a sibling `<span>` (:2219). Every other select in the codebase is labeled (`fields.tsx:239`, `AlbumPanel.tsx:153-157`, `AdvancedPanel.tsx:216`, `App.tsx:905-909`). |
| A-02 | No committed axe scan of app surface | ✅ CONFIRMED | `verify-app-headless.mjs` scenario table (:75-285) has no axe invocation anywhere. |
| A-03 | Receipt focus escapes; not restored | ✅ CONFIRMED | `ExportReceiptCard.tsx:50-61` handles only initial focus + Escape. No Tab trap, no focus-trap util anywhere in `src/`, no native `<dialog>`, no portal, no `inert` on the shell, no restoration (close just unmounts; focus falls to `<body>`). Zero focus assertions in `ExportReceiptCard.test.tsx`; the two tests that exist (`App.warning-ownership.test.tsx:332-391`) cover exactly the two behaviors that ARE implemented. |
| U-02 | Receipt actions below shell; close ≈21px | 🟡 PARTIAL | Close target CONFIRMED: `.toast-close` (`App.css:2520-2526`) gives ≈19×21px — genuinely under WCAG 2.5.8 AA's 24×24 (this is a *real* sub-24px case, unlike the earlier 40×40 false-critical). "Actions below the visible shell" is OVERSTATED: `.receipt` has `max-height: calc(100vh - 4rem); overflow-y: auto` (`App.css:2570-2580`) so actions are always reachable by scrolling *inside the card*. True weaker claim: nothing pins actions/close (close is absolutely positioned in the *scrolling* header, :2639-2648, so it scrolls away), and static estimate says content (~730px) likely scrolls only at 1360×740 (budget 676px), fits at 1440×900. The audit's per-viewport claim is itself unmeasured. Task 7's grid-shell fix is still the right shape. |
| T-01 | No loaded-Standard headless scenario | ✅ CONFIRMED | Every seeded scenario resolves to Advanced (`view-mode.ts:75` + `preview-mock.ts:408-415`); only `empty` lands Standard, and `StandardView` mounts only with a selected track (`App.tsx:525-531`). Loaded Standard has zero browser coverage. |
| T-02 | `album-warning` stops mid-analysis | ✅ CONFIRMED | Scenario (:204-211) has no `drive` — settles, screenshots, asserts "ALBUM"/"4 TRACKS". Mock (`preview-mock.ts:578-590`) couldn't produce advisories anyway (uniform 44.1k stereo, no overrides). The four advisory strings in `AlbumExportReceipt.tsx:71-89` are unreachable in both drive and data. |
| T-03 | Landing CTA "fully visible" measures horizontal only | ✅ CONFIRMED | `verify-landing-responsive.mjs:313-325` checks left/right only; failure message :508-517 says "not fully visible on screen". `window.innerHeight` used once (:189), never compared to `rect.top/bottom`. |
| B-01 | Ignored emitted vite.config.js can shadow the TS config | ✅ CONFIRMED | `.gitignore:16-17` hides `vite.config.js`/`.d.ts`; Vite's config resolution tries `.js` before `.ts`, so a stale emitted config silently wins locally. |
| S-01 | `anyhow 1.0.102` in three locks | ✅ CONFIRMED | `src-tauri/Cargo.lock:67-68`. (Advisory ID itself to be re-proved mechanically per Task 10's `cargo audit` step.) |
| S-02 | CI RustSec lane doesn't deny unsound | ✅ CONFIRMED | `ci.yml:329-333` runs plain `cargo audit --file …` ×3 — no `--deny unsound`, so unsound advisories warn without failing. |
| D-01 | Active docs contradict code/decisions | ✅ CONFIRMED (spot-checked) | `PRODUCT.md:160` "owner listening remains pending" vs approved 2026-08-25 (CLAUDE.md + OPEN_THREADS). `IDEAS_BACKLOG.md:160` lists the auto-updater as an idea; it's shipped code in `lib.rs`. |
| R-01/R-02/R-05 | Evidence-invalidation logic; owner gates; missing updater lifecycle harness | ✅ AGREE (process) | Follows correctly from the confirmed code findings; no repo contradiction found. |
| R-03 | yesmaster.app DNS/mail unresolved | ⚪ NOT VERIFIED here | Owner/DNS-side; plan handles it as an owner decision — correct disposition. |
| R-04 | GitHub security toggles / protected main disabled | ⚪ NOT VERIFIED here | GitHub settings not readable from this session (GitHub MCP down). Owner can confirm in repo Settings in ~2 minutes. |
| C-01 | Defaults duplicated ≥4 places | ✅ CONFIRMED (undercount) | Full literals in `useTrackMaster.ts:121-163` AND `preview-mock.ts:205-245` — and they've already **drifted**: the mock omits `compression_mode`/`adaptive_strength`. Intensity 0.5 hardcoded in `StandardView.tsx:681-715` and `App.tsx:2110`; ~20 test files re-declare defaults. |
| C-02 | Four duplicate transport reset blocks | 🟡 PARTIAL | Real: 3 byte-identical resets (:1490, :1609, :1696) + 1 near-identical device-lost variant (:764-776). But the audit's cited `:362-393` is the `useState` **initializer**, not a reset block — Task 16 should treat it as the sentinel-value source, not a fourth replacement site. |
| C-03 | `transport.deviceLost` write-only | ✅ CONFIRMED | Six writes in `useTrackMaster.ts` (:367,:735,:768,:2241,:2280,:2666); only test reads. The `:716` local is a different variable. |
| C-04 | Stale Volume Match comments | ⚪ NOT VERIFIED (low risk) | Skipped — prose-only change, verify at execution per Task 0's re-prove rule. |
| C-05 | Four dead residue items | ✅ CONFIRMED (all four) | `--z-dropdown` (App.css:57, zero `var()` uses), `@keyframes clip-pulse` (:1216, unreferenced), `.status-warn` (:3519, no TSX use), duplicate App.css import (`main.tsx:14` + `App.tsx:59` — Vite dedupes, so redundancy not a bug). |
| C-06 | Contradictory rail/compressor CSS ownership | ✅ CONFIRMED (rail family) | Proven via U-01's three competing `.right-rail-tools` owners (:4080, :6569, :7601). Compressor families not independently recounted — re-prove at Task 18 execution. |
| C-07 | Tailwind scans whole checkout | ✅ CONFIRMED | `LandingPage.css:18` is bare `@import "tailwindcss"` — no `source()`/`@source` anywhere in the repo. |
| C-08 | CI actions on mutable refs | ✅ CONFIRMED | `ci.yml` all tags/branches (checkout@v4, rust-toolchain@stable, cache@v4, …); `release.yml` fully SHA-pinned — the pattern source already exists in-repo. |
| C-09 | Tracked Graphify output stale | 🟡 PARTIAL | Tracked: yes (3 files). But `.gitignore` records a **2026-06-09 owner decision** to keep exactly those tracked. "Stale" may be true; "should untrack" contradicts a standing decision. Task 21B's ask-the-owner disposition is acceptable, but the executor must present the 06-09 decision alongside the question, not frame untracking as the default. |
| C-10 | Unpublished iPhone art in runtime assets | ✅ CONFIRMED | 917KB `iphone-standard-ui.jpg` tracked, imported nowhere; only reference is a negative test (`LandingCopy.test.tsx:123`). Deliberately unpublished per U6 — owner choice, as the plan says. |

## Amendments the executor must carry into the plan

1. **Task 1 (L-01):** Keep the fix direction exactly as written (route `files.rs` through the guarded
   probe in `decode.rs`). Do not add zero-rate guards to the duration arithmetic as "the fix" — it's
   already guarded (`files.rs:113-116`); the panic is inside Symphonia's `.format()`
   (`TimeBase::new`), so only the `catch_unwind` boundary fixes it. The red test in the plan
   (crafted 0 Hz WAV through `import_tracks`) is the right proof.
2. **Task 7 (U-02):** Reframe the defect: actions are never *unreachable* (the card scrolls
   internally); the defects are (a) sub-24px close target, (b) close/actions scroll away with
   content, (c) probable internal scrolling at the 1360×740 floor. The grid-shell fix stands, but
   the red geometry test should assert "actions visible without scrolling + close ≥24px", and no
   one should claim the pre-fix state made actions unreachable.
3. **Task 16 (C-02):** The `:362-393` range is the transport `useState` initializer, not a fourth
   reset block. Replace 3 identical + 1 device-lost-variant call sites with the helper; leave the
   initializer as the sentinel-value owner (or derive it from the helper, but don't count it as a
   duplicate to delete).
4. **Task 21B (C-09):** Present the 2026-06-09 "keep tracked" owner decision when asking about
   Graphify artifacts; untracking reverses a recorded decision and needs the owner to knowingly
   reverse it.
5. **L-02 severity note:** The missed-event race needs the GitHub check to finish before React
   mounts — unlikely in practice. Fix it anyway (cheap, silent failure), but don't present it to
   the owner as a likely-hit bug.
6. **R-04:** Unverified from the local machine. Owner should confirm GitHub security-settings state
   directly before anyone acts on it.
7. **Housekeeping:** The audit plan doc itself is currently **untracked** in the repo. Commit and
   push it (and this triage) before any other machine or agent is expected to execute from it.

## What was NOT wrong with the audit

No finding was refuted outright. The audit correctly declined to confirm unused npm deps, DSP
defects, or size-based refactors, and correctly classified the short-phone hero CTA as an owner
taste decision rather than a mechanical failure. The evidence-freeze logic (beta.1 evidence cannot
carry to changed code) is correct and matches this repo's exact-commit evidence discipline.
