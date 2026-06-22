> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Consolidated code-from-this review; superseded by the deep-analysis review + 2026-06-12 shippability audit. _(Status: SUPERSEDED.)_ See docs/CHANGELOG.md for the project ledger.

# TRUE Master Review — Adaptive DSP (the doc to code from)

Date: 2026-06-03
Branch: `feat/adaptive-dsp-guardrails` · **HEAD `a96bf5f`** (the one commit since
the fix stack, `a96bf5f`, is docs-only — no engineering claim is affected)
Reviewer: Claude — consolidates and supersedes the review chain below, with
**every retained claim re-verified against the current tree by me this pass.**

Supersedes / reconciles:
`2026-06-02-adaptive-dsp-tier1-review.md` · `2026-06-03-...-desktop-review.md` ·
`2026-06-03-...-post-review-commit-stack-review.md` · `2026-06-03-...-GLOBAL-review.md`
· `2026-06-03-...-residual-master-review.md` · `2026-06-03-...-FINAL-master-review.md`

Gates (run at `6480d96`, unchanged at `a96bf5f` — diff is docs-only):
`cargo test --lib` **234 pass** · `npm test` **161 pass** · `tsc` clean ·
`clippy --all-targets -D warnings` clean · all `preset_byte_identity` snapshots green.

> **Do I agree with the FINAL review?** Yes. I independently re-checked each of its
> corrections and new findings against the code; all hold (see the verification log
> at the bottom). Its only soft spot is framing — "B2 is mis-scoped" is a
> *sharpening*, not a correction: the residual doc already split production (TS) from
> slow-lanes (Rust); FINAL makes explicit that the Rust path is **test/tuning only**.
> Everything else (B4 escalation, the album-test gap, the readout coupling, the
> pre-floor readout overstatement, the untracked trail, the line drift) is real and
> folded in below.

---

## 0. Where this actually stands

The fix stack closed every release-blocking item from the first review, with tests,
and the gates are green. **The residual is small and well-understood:** one real
cross-surface behavior gap (album), one architectural root-cause fix that dissolves a
whole bug class, a fragile opt-out, an untraceable receipt, two readout/measurement
accuracy issues, and process hygiene — plus the by-ear calibration that was always
the gate. **None of it is a redesign.** It is safe to code from this doc once the P0a
process step and the P0 correctness work land.

---

## A. Verified FIXED — do not re-do (re-checked at HEAD)

Slow-lane injection (+tests) · preview-WAV WYSIWYG (+ non-vacuous parity test) ·
`LRA=0` sentinel (guardrail + `from_analysis` fallback, +test) · bright deadband
`0.20→0.30` with a **real-pink** regression test · dead `from_analysis` now wired +
the false "backend-derived export" comment removed · the per-axis readout (reuses
`compute()`, so it structurally cannot drift from the applied multipliers) · spec
table corrected (the code was already right) · "down-only" landing comment fixed ·
default-strength bare literal removed · the three invariant tests (strength-0 inert
with a trigger, multi-axis composition, manual-override preservation).

The GLOBAL review's F1–F5/F9/F11/F12 "CONFIRMED·HIGH" rows describe the **pre-fix**
code (that fleet ran while the tree was being edited) — they are all in this list.
Do not act on them.

---

## B. REAL & still unaddressed — ranked, line-verified at HEAD

### B1 · HIGH — Album: audition is adaptive, export is not (and it has no test)
- Album **export** builds `renderTracks` from `albumIntent`/override settings with
  **no `injectSourceProfile`** (`useTrackMaster.ts:1143-1154`) → renders the
  non-adaptive chain.
- Album **audition** is the live-track path, which injects regardless of mode
  (`applyChainDispatchOverrides`, `settings-transitions.ts:198-201`; `selectedSettings
  == albumIntent` in album mode). So you **hear adaptive, ship flat**, and a track
  masters differently standalone vs in an album.
- **The lock is convention-only.** The sole album-scope test is the Rust
  `apply_album_shadow` profile-agnostic test, which only asserts an *incoming* profile
  is **preserved** — it does **not** prove the FE export path omits injection.
  `App.album-export.test.tsx` has **zero** `source_profile`/adaptive references. A
  future "wire the album control" change could add injection to `exportAlbumPlan` and
  **silently break the owner's lock with all 161 TS + 234 Rust tests still green.**
- **Fix (P0):** pick one — wire per-track injection in `exportAlbumPlan` (album
  becomes adaptive) **or** make album non-adaptive end-to-end (stop album-mode live
  injection + disable the Adapt-Strength control in album + strip the profile in
  `apply_album_shadow`). **Ship the album regression test in the same commit** —
  payload-equality (album settings carry no `source_profile`) or render-equality.

### B2 · MEDIUM (root cause) — injection is per-frontend-call-site; the Rust mapper is test-only
Correctness of "on by default" rests on the frontend remembering to inject at every
render/playback entry point. It already forgot three (preview + both slow lanes, now
patched) and still forgets album export (B1). The Rust `SourceProfile::from_analysis`
runs **only the evidence/tuning fixtures** (`fixture_matrix.rs:129`,
`reference_tuning.rs:305`) — **all production paths use the TS
`sourceProfileFromAnalysis`**. So there are two mappers that must be **hand-synced**,
or the slow lane stops being representative of shipping behavior.
- **Fix (P0, highest leverage):** make the backend derive `source_profile` from the
  track `AnalysisResult` server-side; treat any FE-supplied profile as an override;
  cache it in audio-thread state so the live `update_chain` path (settings-only, no
  track id — `audio.rs:107`) gets it too. Closes the preview/slow-lane class, the
  dual-mapper drift, and B1 by construction.
- **Coupling (NF-2):** this refactor **must also re-touch the readout** — the
  `guardrail_readout` command consumes `settings.advanced.source_profile`, and its
  hook fetch builds the profile with `injectSourceProfile(...)` (`useTrackMaster.ts:576`).
  The readout is a **sixth** profile-consuming surface; if derivation moves
  server-side, both the command's input contract and the TS fetch site must change or
  the listening-session readout shows stale/inactive trims.

### B3 · MEDIUM — LUFS-landing can recompose the trimmed loudness
When an effective delivery target is active — **including Custom + an explicit
`lufs_offset_db`**, not only named profiles — the post-chain landing stage
(`engine.rs:106-127`, applied last `dsp.rs:2041-2045`, mirrored in live preview)
applies **broadband makeup**; because the four trims both lose LUFS *and* free
true-peak headroom, the landing recoups the loss, so an all-four-axis source can show
a level-matched delta **exceeding any single per-axis cap**. The brickwall limiter has
no makeup; the compressor's auto-makeup *drops* when density is trimmed — the landing
stage is the only recomposition path. Real but narrow. **Fix (Tier-2):** a shared
total-loudness-loss budget across axes, or attenuate combined strength when predicted
pre-limiter loss exceeds a threshold. (Mitigated today only by the readout's honest
"pre-landing" label.)

### B4 · MEDIUM — "on by default" is "on unless you drag to exactly 0"
`adaptive_strength = null` ⇒ engine default 0.6 (on) everywhere
(`App.tsx:2153` display `?? ADAPTIVE_STRENGTH_DEFAULT`; backend `unwrap_or(0.6)`).
And `null` is the norm, not the exception:
- `DEFAULT_SETTINGS.advanced` **omits `adaptive_strength`** (`useTrackMaster.ts:57-80`)
  → **every freshly imported track is adaptive-on.**
- The Advanced **reset writes `null`** (`App.tsx:2072`); field-clear / slider
  double-click also write `null` → they silently **re-enable** adaptation.
- A project saved at `null` reloads adaptive-on. The only durable "Off" is exactly 0.
- **Fix (P0):** give `DEFAULT_SETTINGS` an explicit value, pick one durable "off"
  representation, exclude `adaptive_strength` from the reset null-sweep, and label the
  default honestly ("Auto 60%", not a blank that reads as off).

### B5 · MEDIUM — delivered masters are untraceable as adaptive
`ExportReport` (`types.rs:781-792`) records loudness/TP/format/checks but **no**
`adaptive_strength` or "guardrails applied" flag. Given adaptation silently changes
preset character and (per B1) differs across surfaces, two otherwise-identical masters
can't be told apart, and you can't know after the fact what a delivered file used.
**Fix (P1):** add `effective_adaptive_strength` (+ a one-line profile digest) to
`ExportReport` and the receipt.

### B6 · MEDIUM — tonal trigger reads only the first ~5.5 s
The 6-band tonal read is a single FFT over the first ~5.5–6 s (`analysis.rs:322-331`,
`1<<18` cap) while DR/LRA/correlation are whole-track — an intro can classify the
whole song. The misleading "30 s" comment is fixed; the windowing is not.
**Fix (P1):** Welch-average the 6-band FFT across the track. **Caveat:**
`spectral_balance_6band` is *shared* with role/character/album-bias inference, so
widening the window shifts those too — wider blast radius than the guardrails alone.

### B7 · MEDIUM — `stereo_width` computed, carried, never used
The width guardrail is correlation-only (`guardrails.rs:131-135`); the more on-point
"already wide" measure (`stereo_width` = side/(mid+side), populated on the wire) is
ignored. **Fix:** wire it as a co-trigger, or drop it from the profile+spec.

### B8 · MEDIUM — readout overstates near-floor EQ trims (calibration-relevant)
`readout_for` reports `bright_trim/low_trim = 1.0 − mult` (`guardrails.rs:229-230`) —
the raw strength-scaled cap fraction — but the chain's EQ trim passes through
`floor_boost`, which clamps to the **+0.5 dB character floor**. Example: a 0.8 dB
preset boost at `bright_mult 0.5` lands at 0.5 dB (37.5% realized removal) but the UI
shows **"Highs −50%."** Density/width are floor-free and exact; only the two EQ axes
overstate, and only near the floor. **For a by-ear calibration session the number must
read true.** **Fix (P1, with the readout):** compute the EQ readout as the realized
fraction (apply `floor_boost` to the actual preset bands), or hover-disclose
"requested trim, before the +0.5 dB floor."

### B9 · LOW — album boundary not enforced at the backend
`apply_album_shadow` never injects a profile but also never **strips** one (its test
asserts an incoming `Some(...)` is preserved). A stale/hand-built/API payload carrying
`advanced.source_profile` would make album adaptive despite the label. Narrow today
(the profile isn't persisted), but the guarantee rests on "nothing upstream sets it."
**Fix:** if album stays out of scope, `shadowed.advanced.source_profile = None` +
a clearing test.

### B10 · LOW — TS injectors don't clear a stale profile
`injectSourceProfile`/`applyChainDispatchOverrides` only *write* `source_profile` when
analysis yields one; they never null it when analysis is missing
(`settings-transitions.ts`). Same narrow risk as B9; folds into the backend-ownership
fix (B2).

### B11 · LOW — second LRA hazard: LU aliased into a dB threshold
Distinct from the fixed `LRA=0` sentinel: when P95-P10 is missing, `from_analysis`
does `unwrap_or(if lra > 0.5 { lra } else { 100.0 })` (`types.rs:149`), placing an
**LU** value into the **dB** DR ramp (8/3 dB). A non-zero fallback (e.g. 4 LU)
mis-judges against dB thresholds. Very narrow window, cheap principled fix: carry
P95-P10 as `Option`, use only `lra_raw` when it's `None`.

### B12 · LOW — Adapt control inert-but-editable in album; minor drift; stale tests
- The Adapt-Strength control renders and accepts edits in album mode
  (`App.tsx:2151-2159`, not gated) but album render is inert — a no-op + text note.
  Resolved by whichever way B1 is decided.
- **Density lever:** one scalar drives engagement/threshold/ratio; "keeps ≥40% comp"
  is true of the macro, not dB GR, and the cap saturates at the default 0.6 on a
  fully-dense trigger. Tier-2 reshape (on engagement, or PSR-target), not a defect.
- `EQ_BOOST_FLOOR_DB` global, not per-band; default strength still two literals
  (Rust const + `bindings.ts 0.6`).
- **Stale boundary test:** `deadband_means_no_action_just_inside` (`guardrails.rs:313`)
  uses brightness 0.20 — now well *inside* the 0.30 deadband, so it no longer tests the
  edge; `bright_source_trims_only_air`'s "past the 0.20 deadband" comment is stale.
  Update to 0.30.
- **Test wording:** a coefficient-level `profile=Some + strength=0` byte-identity test
  already exists (`dsp.rs:3294`); only the **render-level** version is missing — reword
  the gap so it complements, not duplicates.

### Doc / process hygiene
- **The entire review trail is untracked** (`git status` → all `docs/reviews/2026-06-0[23]-*.md`
  are `??`), and a **committed** doc (the finish-plan) links to untracked review files.
  The source-of-truth driving the coding phase is lost on a clean checkout. **Commit it.**
- HANDOFF still lists the readout as **both shipped and deferred**; its verification
  block still shows the old **158/224** counts (actual 234/161).
- The next-steps backend note says backend ownership closes "preview/album/slow-lane/live
  by construction" — only true for album **if album becomes adaptive**, which contradicts
  the unadapt decision. Split it.

---

## C. De-noise (verified safe to dismiss — don't spend time here)
1. GLOBAL F1–F5/F9/F11/F12 "CONFIRMED" — stale (pre-fix code); all addressed (Section A).
2. F7 "overdrive is a separate 4th saturation stage" — wrong; same threshold/ratio
   mechanism (`saturation_amount` is not density-driven). The saturation-at-0.6 numeric
   point is real (B12).
3. "Every neutral track loses ~40% of its air" — idealized full-span 1/f worst case;
   real masters with top rolloff/bass tilt can sit under even the old 0.20 — and it's
   moot at deadband 0.30.
4. F3 as a broad hazard — only fires on short/degenerate (unmeasurable-LRA) sources.

---

## D. Action plan (the corrected sequence)

### P0a — process, first (cheap, unblocks everything)
0. **Commit the review trail** (`docs/reviews/2026-06-0[23]-*.md` incl. this doc) so the
   coding source-of-truth is version-controlled.

### P0 — before the listening gate (correctness + a test-protected baseline)
1. **Backend-owned `source_profile` (B2)** — derive server-side; FE profile is an
   override; cache for the live path. **Re-touch the `guardrail_readout` contract + the
   TS readout fetch (`useTrackMaster.ts:576`) in the same change.** Closes the
   preview/slow-lane class + dual-mapper drift by construction.
2. **Decide AND implement album scope (B1)** — wire it, or non-adaptive end-to-end
   (+ disable the album control + strip in `apply_album_shadow`). **Ship the album
   regression test in the same commit** — the lock is currently untested and can
   silently regress.
3. **Fix the `null` opt-out (B4)** — explicit `DEFAULT_SETTINGS` value, durable "Off",
   honest "Auto 60%" label, exclude from the reset sweep. **And gate the LU→dB LRA
   aliasing (B11)** — P95-P10 as `Option`, use only `lra_raw` when `None`; add the
   `healthy DR + missing/zero LRA ⇒ no density trim` test.

### P1 — calibration-enabling (still before listening)
4. Recalibrate brightness toward measured pink/reference + **Welch-average the 6-band
   FFT (B6)** (mind the shared-analysis blast radius). **Fix the pre-floor readout
   overstatement (B8)** so the by-ear pass reads true dB.
5. **Export-receipt traceability (B5).**

### P2 — hardening & Tier-2 direction
6. `stereo_width` co-trigger (B7); album backend strip + TS stale-clear (B9/B10);
   density-lever reshape (B12); refresh the stale boundary test (B12); the remaining
   tests (album==track equality, multi-axis under active LUFS landing, **render-level**
   `profile=Some + strength=0`); doc hygiene.
**Tier-2 north star:** measured neutral from Dan's own reference masters (slow-fixture
lane), optionally per-preset; tilt-vs-reference deadband (makes the deadband number
almost irrelevant); PSR/crest closed loop — the only honest "won't crush it further"
dynamics defense.

---

## E. Two calls that are the owner's alone (they gate P0 #1 and #2)
1. **On-by-default at 0.6.** Defensible (sole-user calibrator, byte-identical off path),
   but until B1/B2 land the "preset as designed" baseline is unstable across surfaces
   (live = adaptive, track-export = adaptive, album-export = flat, every new track = on,
   `null`/reset silently re-arm) with no receipt record of what was heard. Fixing the
   wiring is what makes on-by-default *safe* to ship.
2. **Album adaptation scope (B1).** In (wire it) or out (non-adaptive end-to-end).
   Decide before the listening session — it changes what the album A/B even compares.

---

## F. Verification log (this pass, against HEAD `a96bf5f`)
- `from_analysis` callers = exactly `fixture_matrix.rs:129` + `reference_tuning.rs:305`
  (test/tuning); production = TS. ✓ confirms B2 scope.
- `App.album-export.test.tsx` — zero `source_profile`/adaptive refs. ✓ confirms B1 test gap.
- `DEFAULT_SETTINGS.advanced` (`useTrackMaster.ts:57-80`) omits `adaptive_strength`. ✓
  confirms B4 ("every new track on").
- Width compute `guardrails.rs:131-135`; `unwrap_or` `types.rs:149`. ✓ line drift fixed.
- Readout EQ trim `1.0 − mult` (`guardrails.rs:229-230`) vs `floor_boost`. ✓ confirms B8.
- Stale boundary test `guardrails.rs:313-323` (brightness 0.20). ✓ confirms B12.
- Coeff-level strength-0 byte-identity test exists `dsp.rs:3294`. ✓ reword test gap.
- Review trail untracked (`git status`). ✓ confirms process item.
- `engine.rs:106-127` landing makeup; `dsp.rs:2041` comment now correct. ✓ confirms B3.
- HEAD `a96bf5f`; only commit since `6480d96` is docs (`a96bf5f`, backlog). ✓ gates still valid.

## Bottom line
The fix stack did the right work in the right order; the residual review mapped what's
left; the FINAL review's corrections are accurate and now folded in. **Greenlit to code
from this doc.** Commit the trail (P0a), land the backend-ownership + album + opt-out +
LRA work (P0) with the album test promoted alongside the decision, then ship the receipt
flag and the readout/window fixes (P1) and listen. None of it is a redesign.
