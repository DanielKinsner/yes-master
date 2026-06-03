# FINAL Master Review — Adaptive DSP (the gate before coding)

Date: 2026-06-03
Branch: `feat/adaptive-dsp-guardrails` · **actual HEAD `a96bf5f`** (the residual doc's header says `6480d96`; the 2 extra commits are docs-only, so no engineering claim is affected)
Method: independent **6-agent read-only fleet** re-verifying every claim in `2026-06-03-adaptive-dsp-residual-master-review.md` against the code *as it stands now*, plus a fresh-eyes audit of the 5 commits added since the last review (`70d0507..a96bf5f`) and a completeness/plan critique. This is the last accuracy gate; code is written from THIS doc.
Read-only honored (another agent is live in `apps/iphone-native/*`); gate numbers (234 Rust / 161 TS) taken from the residual doc, not re-run.

---

## Top-line verdict

**The residual master review is accurate and safe to code from.** Independent verification:
- **Section A ("already fixed") — all 10 GENUINELY fixed**, test-backed, verified at HEAD. Do not re-do any of them.
- **Section C ("de-noise") — all 4 dismissals are safe.** Nothing real is being buried; the one item with a live remainder (LRA LU/dB aliasing) is correctly escalated to Section B, not hidden.
- **Section B residual items — real**, with **two corrections and one understatement** (below).

But this gate surfaces a **HIGH plan-sequencing flaw**, two **MEDIUM** coupling/process gaps, and a real **LOW** readout caveat that the same-author doc didn't catch. Fold these in before coding.

---

## Corrections to the residual doc (verify-then-trust)

1. **B2 is mis-scoped — the Rust `from_analysis` does NOT run production "slow lanes (live/preview/export)."** It is called from **only two sites**, both test/tuning harnesses: `fixture_matrix.rs:129` and `reference_tuning.rs:305` (the file's own comment calls it "the (otherwise dead) Rust `SourceProfile::from_analysis`"). **All production paths (live, preview, Track-Master export) use the TS `sourceProfileFromAnalysis`**; album export injects nothing. So the genuine dual-mapping hazard is narrower than stated: *TS drives all production; Rust drives only the evidence/tuning fixtures* — they must be hand-synced so the test harness stays representative of shipping behavior. The "`update_chain` carries settings only, no track id" sub-claim is **fully accurate** and is the real per-call-site fragility.

2. **Stale line citations** (substance correct, locations drifted): B7 width-trim is at `guardrails.rs:130-135`, **not** `~114-119` (that region is now the LRA sentinel). B10's `unwrap_or` spans `types.rs:149-155`, **not** `~144`.

3. **B4 is understated.** It's not only reset/clear/saved-`null` that re-enable adaptation — **every freshly imported track defaults adaptive-on**, because `DEFAULT_SETTINGS.advanced` (`useTrackMaster.ts:57-80`) omits `adaptive_strength` entirely → serializes `null` → `unwrap_or(0.6)`. The opt-out is fragile against *any* `null` path, and "on by default" is in fact "on unless you explicitly drag to 0."

---

## New findings from this gate (not in the residual doc)

### 🔴 NF-1 (HIGH, plan-sequencing) — the album lock (B1) has ZERO frontend regression test, and the plan files that test as P2 while the decision it protects is P0
B1 is real and verified: album **export** builds `renderTracks` from raw `albumIntent`/override settings with **no** `injectSourceProfile` (`useTrackMaster.ts:1142-1154`), while album **audition** reuses the live-track path which **does** inject regardless of mode (`applyChainDispatchOverrides`, `settings-transitions.ts:198-201`). The **only** album-scope test is the Rust `apply_album_shadow` profile-agnostic test, which asserts an *incoming* profile is **preserved** — it does **not** prove the FE export path omits injection. `App.album-export.test.tsx` has **zero** references to `source_profile`/adaptive. **Consequence:** a future "fix B11" (wiring the inert Adapt-Strength control) could add `injectSourceProfile` to `exportAlbumPlan` and silently break the owner's lock **with all 161 TS + 234 Rust tests still green.** The album guarantee is convention-only. **The album-export regression test must ship in the same commit as the B1 decision (P0), not in P2 hardening.**

### 🟠 NF-2 (MEDIUM, coupling) — the backend-ownership refactor (B2/P0#1) must also re-touch the already-shipped readout
The `guardrail_readout` command (`guardrails.rs:256-261`) consumes the **FE-injected** profile via `settings.advanced.source_profile`, and its hook fetch builds that profile with `injectSourceProfile(selectedSettings, selectedAnalysis)` (`useTrackMaster.ts:576`). If B2 moves derivation server-side and treats the FE profile as an override, **both the readout command's input contract and the TS readout-fetch site must change**, or the listening-session readout reads a stale/absent profile and shows wrong/inactive trims. The readout is a **sixth** profile-consuming surface (alongside live/preview/export/album/slow-lanes) — it belongs in B2's blast-radius list and the P0 work item.

### 🟠 NF-3 (MEDIUM, process) — the entire review trail is UNTRACKED; commit it before coding
`git rev-parse HEAD` = `a96bf5f` (doc header says `6480d96`). More importantly: this residual review, the GLOBAL review, the desktop review, the commit-stack review, and the tier-1 review are **all untracked** (`??`) — only the `2026-05-29` trio is in git. The committed HANDOFF even links to two of these untracked files. **The source-of-truth driving the whole coding phase is unsaved and is lost on a clean checkout or branch switch.** Commit the `docs/reviews/2026-06-0[23]-*.md` trail (and this doc) before code is written.

### 🟡 NF-4 (LOW, real) — the readout reports the PRE-FLOOR multiplier, not the realized dB removed
`readout_for` sets the EQ trims as `1.0 - g.{bright,low}_mult` (`guardrails.rs:229-232`) — the raw strength-scaled cap fraction. But the chain's actual EQ trim passes through `floor_boost` (`guardrails.rs:187-195`), which clamps to the **+0.5 dB character floor**. Worked example: a 0.8 dB preset boost with `bright_mult` 0.5 → `floor_boost` lands at 0.5 dB → **realized** removal 37.5%, but the UI shows **"Highs −50%."** Density and width readouts are exact (no floor); only the two EQ axes can overstate on near-floor boosts. For a by-ear calibration session this matters — the number should read true dB. **Fix:** compute the EQ readout as the realized fraction (apply `floor_boost` to the actual preset bands), or hover-disclose "requested trim, before the +0.5 dB floor."

### 🟡 NF-5..7 (LOW, cosmetic/coverage)
- **Stale boundary test (`d6519b7`):** `deadband_means_no_action_just_inside` still uses brightness 0.20 — now well *inside* the 0.30 deadband, so it no longer tests the edge it claims; and `bright_source_trims_only_air`'s "past the 0.20 deadband" comment is stale. Update to 0.30.
- **Surface-map completeness:** two surfaces the doc leaves implicit (both safe): **iPhone has no adaptive path** (zero `source_profile`/`from_analysis` refs in `apps/iphone-native`), and there is **no distinct live-album path** — album audition *is* the live-track path (`selectedSettings == albumIntent`), which is the root of B1. The readout also *runs* in album mode; only its display is `!albumMode`-gated.
- **Existing coverage for `profile=Some + strength=0`:** a **coefficient-level** test already exists (`dsp.rs:3294`, asserts byte-identity vs no-profile chain). Only the **render-level** version is missing — reword the P2 item so it complements rather than duplicates.

---

## Confirmed solid (don't touch)
- **Section A, all 10:** slow-lane injection + tests, preview WYSIWYG + non-vacuous parity test, LRA=0 sentinel (guardrail + `from_analysis` fallback) + test, bright deadband 0.30 + **real-pink** regression test, `from_analysis` live + false comment removed, the per-axis **readout reuses `compute()`** (structurally cannot drift — single function feeds readout/live/preview/export), spec table corrected, "down-only" comment fixed, default-strength dedup, the three invariant tests.
- **Deadband 0.30 is numerically correct:** ideal pink `presence+air = 0.2777`, so 0.30 clears it by ~0.022 (thin but valid); `LOW_DEADBAND 0.42` correctly didn't need to move (pink `sub+low = 0.378`). The pink test is self-validating (synthesizes real pink noise, runs the real analyzer).
- **Section C de-noise is fair** — F1–F5/F9/F11/F12 are stale-not-wrong (all addressed); F7's "4th saturation stage" framing correctly dropped (`saturation_amount` is not density-driven); the "40% air loss" is moot at deadband 0.30; F3 is genuinely narrow.

---

## Final action plan (corrected sequence)

### P0a — process, do first (cheap, unblocks everything)
0. **Commit the review trail** (`docs/reviews/2026-06-0[23]-*.md` + this doc) so the coding source-of-truth is version-controlled (**NF-3**). Update this doc's header HEAD to `a96bf5f`.

### P0 — before the listening gate (correctness + a stable, test-protected baseline)
1. **Backend-owned `source_profile` (B2)** — derive server-side from the track `AnalysisResult`; FE profile becomes an override; cache in audio-thread state so the live `update_chain` path (no track id) gets it too. Closes the preview/slow-lane class and the dual-builder drift by construction. **Must also update the `guardrail_readout` command contract + the TS readout-fetch site (`useTrackMaster.ts:576`) in the same change (NF-2).** *(Owner-timed — it's a real refactor; see decisions.)*
2. **Decide AND implement album scope (B1/B11)** — either wire per-track injection in `exportAlbumPlan` (album becomes adaptive) **or** make album non-adaptive end-to-end (stop album-mode live injection + disable the Adapt-Strength control in album mode + strip the profile in `apply_album_shadow`, B8). **Ship the album regression test in the same commit (NF-1)** — payload-equality (album per-track settings carry no `source_profile`) or render-equality (album == non-adaptive track). No "audition adaptive / ship flat."
3. **Null-opt-out (B4)** incl. giving `DEFAULT_SETTINGS` an explicit value and a durable "Off" representation (+ honest "Auto 60%" label), **and** gate the **LRA LU→dB aliasing (B10)** — carry P95-P10 as `Option`, use only `lra_raw` when it's `None`. Add the `healthy DR + missing/zero LRA ⇒ no density trim` test.

### P1 — calibration-enabling (still before listening)
4. Recalibrate brightness toward measured pink/reference + **Welch-average the 6-band FFT (B6)** — mind the shared-`spectral_balance_6band` blast radius (role/character/album-bias also shift). Fix the **pre-floor readout caveat (NF-4)** so the by-ear pass reads true dB.
5. **Export-receipt traceability (B5)** — add `effective_adaptive_strength` (+ profile digest) to `ExportReport`.

### P2 — hardening & Tier-2 direction
6. `stereo_width` co-trigger (B7); TS stale-profile clear (B9); density-lever reshape (B12); refresh the stale boundary test (NF-5); the remaining tests (multi-axis under active LUFS landing; **render-level** `profile=Some + strength=0`); doc hygiene (HANDOFF 158/224 → 161/234, remove the shipped-but-deferred readout bullet).
**Tier-2 north star:** measured neutral from Dan's own reference masters (slow-fixture lane), optionally per-preset; tilt-vs-reference deadband; PSR/crest closed loop.

---

## Two calls that are the owner's alone (gate P0 #1 and #2)
1. **On-by-default at 0.6** — defensible (sole-user calibrator, byte-identical off path), but until B1/B2 land the "preset as designed" baseline is unstable across surfaces and `null`/reset silently re-arm, with no receipt record of what was heard. Fixing the wiring is what makes on-by-default *safe* to ship.
2. **Album adaptation scope** — in (wire it) or out (non-adaptive end-to-end). Decide before the listening session; it changes what the album A/B even compares.

---

## Bottom line
The fix stack did the right work in the right order; the residual review is an accurate map of what's left. **Greenlit to code from it**, with these gate corrections: B2 is test/tuning-only (not production slow lanes), B4 is broader than stated (every new track is adaptive-on), the album lock needs its regression test **promoted to P0** so it can't silently regress, the backend-ownership refactor must **re-touch the readout**, the readout itself **overstates near-floor EQ trims**, and the **review trail must be committed** before code. None of it is a redesign. Land P0a→P0, then listen.

*Produced by a read-only 6-agent verification fleet over `feat/adaptive-dsp-guardrails` @ `a96bf5f`; no files mutated, no builds, no branch changes (a second agent was live in the tree).*
