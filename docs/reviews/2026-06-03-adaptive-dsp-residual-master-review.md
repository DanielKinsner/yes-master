# Master Review — Adaptive DSP, residual state after the post-review fix stack

Date: 2026-06-03
Branch: `feat/adaptive-dsp-guardrails` (not merged)
Reviewer: Claude
Scope: the **current tree at HEAD** (`6480d96`), after the 10-commit fix stack
`f6c7cc8…6480d96` that an agent built from the `2026-06-02` master review.
Method: re-verified every claim in the two newest reviews
(`2026-06-03-...-post-review-commit-stack-review.md` and
`2026-06-03-...-GLOBAL-review.md`) against the code **as it stands now**, plus my
own independent gate runs. The job here is the one Dan asked for: **separate what
is already fixed, what is overstated/not-real, and what is REAL and still
unaddressed.**

Gates I ran on the current tree: `cargo test --lib` **234 passed**, `npm test`
**161 passed**, `tsc --noEmit` clean, `cargo clippy --all-targets -D warnings`
clean. All `preset_byte_identity` snapshots green.

---

## Read this first: the GLOBAL review is partly stale

The `GLOBAL-review.md` was run by an 8-agent fleet **while another agent was live
editing the tree** (it says so itself). The result: its consolidated table
**F1–F5, F9, F11, F12 describe the *pre-fix* code** — it cites the old `0.20`
deadband, "preview passes raw `selectedSettings`," "`from_analysis` is dead,"
"no readout," etc. Those are all **already fixed**. Its *new* findings (N1–N7)
and the `commit-stack-review`'s residual P2/P3 items are where the real,
still-open work is. Both reviews remain trustworthy on the *engineering* — the
core math and safety invariants re-verified clean — but the F-table severities
must not be acted on at face value.

---

## A. Already fixed (do not re-do) — verified at HEAD

| Was flagged | Fixed by | Verified now |
|---|---|---|
| Slow lanes don't set `source_profile` (06-02 #1 / GLOBAL F1) | `dc62dab` | `fixture_matrix.rs` + `reference_tuning.rs` call `SourceProfile::from_analysis`; regression tests assert presence |
| Preview-WAV skips injection / "audit-WAV diverges" (06-02 #2 / GLOBAL F2,N3) | `dc62dab` | `useTrackMaster.ts:1243-1246` wraps `injectSourceProfile`; WYSIWYG parity test deep-equals preview vs export profile |
| `LRA=0.0` → full density trim (06-02 #3 / GLOBAL F3) | `f6c7cc8` | `guardrails.rs` ignores LRA ≤ 0.5; `from_analysis` no longer falls back to the sentinel; regression test green |
| Bright deadband `0.20` fires on pink-neutral (06-02 #4 / GLOBAL F4) | `d6519b7` | now `0.30` (above pink 0.278); dark test fixture replaced with real pink shares; `pink_neutral_source_gets_no_bright_trim` green |
| Dead Rust `from_analysis` + false "backend-derived export" comment (06-02 #9 / GLOBAL F9) | `dc62dab`,`44a44ef` | now the slow-lane builder (live); `types.rs` comment corrected to the real injection map |
| No "what was trimmed" readout (06-02 #11 / GLOBAL F11) | `f112f1f` | `guardrail_readout` command **reuses `compute()`** (no display/apply drift); labeled "chain trims (pre-landing)" |
| Spec table over-claims `presence_db`/`stereo_width` (06-02 #12 / GLOBAL F12) | `44a44ef` | spec corrected; **code was already right** |
| Stale "down-only" landing comment (GLOBAL F8 note) | `44a44ef` | `dsp.rs:661,2041` now read "down, or up when ceiling headroom allows" |
| Two bare copies of default strength (06-02 #13b) | `44a44ef` | `App.tsx` imports `ADAPTIVE_STRENGTH_DEFAULT` (bindings↔Rust still manual but documented — see C7) |
| Missing invariant tests | `27f2a4c` | `strength_zero…inert_with_trigger`, `compose_across_bright_dense_and_wide`, `preserve_explicit_user_overrides` (coefficient-level) |

The mechanical/release-blocking findings from the first round are **closed and
test-backed.** What follows is what's actually left.

---

## B. REAL & still unaddressed — ranked, verified at HEAD

### B1 · HIGH — Album mode: you audition adaptive, you ship non-adaptive
The owner "locked album unadapted," but the lock only covers **export**. Live
album playback still injects.
- Album **export** (`useTrackMaster.ts:1143-1154`) builds `renderTracks` from
  `albumIntent`/override settings with **no `injectSourceProfile`** →
  `api.renderAlbumPlan` renders the non-adaptive chain.
- Live playback (`withSourceLufs` → `applyChainDispatchOverrides`,
  `useTrackMaster.ts:304-306`) injects the profile **regardless of mode**. In
  album mode `selectedSettings == albumIntent`, so album auditioning **is**
  adaptive.

Net: in album mode, what you hear ≠ what ships, and the *same track* masters
differently standalone (adaptive) vs in an album (not). The UI note ("Adaptive
applies to Track Master export, not Album renders," `App.tsx:2169`) tells the
user, but they still *hear* the adaptive version while auditioning. This is the
single most important remaining behavior gap. **Fix:** either inject per-track in
`exportAlbumPlan` (album becomes adaptive) **or** stop album-mode live playback
from injecting (album becomes truly non-adaptive end-to-end) — pick one;
"editable-but-inert + adaptive audition" is the one combination to avoid.
Add an `album == track` render-equality (or render-difference) test either way.

### B2 · MEDIUM (root cause) — injection still lives per-frontend-call-site
"On by default" correctness rests on the frontend remembering to inject at every
render/playback entry point. It already forgot three (preview + both slow lanes,
now patched) and still forgets album export (B1). The Rust `from_analysis` is now
live but only in the slow lanes, so there are now **two active mapping
implementations** (TS `sourceProfileFromAnalysis` for live/preview/export, Rust
`from_analysis` for slow lanes) that must be kept in sync by hand.
**Fix (highest leverage):** make the backend derive `source_profile` from the
track `AnalysisResult` server-side and treat any FE-supplied profile as an
override; cache it in audio-thread state so the live `update_chain` path (which
carries settings only — `audio.rs:107`) gets it too. This closes B1, the preview
class, the slow lanes, and the dual-builder drift **by construction.** Tracked as
the finish-plan's decision #5 (owner-timed); it is the difference between "we
patched the sites we found" and "a fifth site can't forget."

### B3 · MEDIUM — LUFS-landing can recompose the trimmed loudness
Still substantively unaddressed (the readout's honest "pre-landing" label is
mitigation, not a fix). When an effective delivery target is active — **including
Custom + an explicit `lufs_offset_db`**, not only named profiles — the post-chain
landing stage (`engine.rs:106-127`, applied last at `dsp.rs:2041-2045`, mirrored
in live preview) applies **broadband makeup**, and because the four trims both
lose LUFS *and* free true-peak headroom, the landing can recoup that loss. So a
hot+bright+wide+dense source that triggers all four axes can show a level-matched
audible delta **exceeding any single per-axis cap**. The brickwall limiter has no
makeup and the compressor's auto-makeup *drops* when density is trimmed, so the
landing stage is the sole recomposition path — real but narrow (needs an active
target). **Fix (Tier-2):** a shared total-loudness-loss budget across the four
axes, or attenuate combined strength when predicted pre-limiter loss exceeds a
threshold.

### B4 · MEDIUM — the `null`-strength opt-out is a trap
`adaptive_strength = null` means "engine default 0.6 (on)" everywhere:
display `value={a.adaptive_strength ?? ADAPTIVE_STRENGTH_DEFAULT}`
(`App.tsx:2153`); backend `unwrap_or(ADAPTIVE_STRENGTH_DEFAULT)`. But the Advanced
**reset writes `null`** (`App.tsx:2072`), as do field-clear and slider
double-click. So reset/clear silently **re-enables** adaptation, the only durable
"Off" is dragging to exactly `0`, and a project saved at `null` reloads
adaptive-on. **Fix:** pick one durable "off" representation, exclude
`adaptive_strength` from the reset null-sweep, and label the default honestly
("Auto 60%", not a blank that reads as off).

### B5 · MEDIUM — delivered masters are untraceable as adaptive
`ExportReport` (`types.rs:781-792`) records loudness/TP/format/checks but **no**
`adaptive_strength` or "guardrails applied" flag. Given adaptation silently
changes preset character and (per B1) differs across surfaces, there is no way to
tell two otherwise-identical masters apart, or to know after the fact whether a
delivered file was adapted and at what strength. **Fix:** add
`effective_adaptive_strength` (+ a one-line profile digest) to `ExportReport` and
the receipt.

### B6 · MEDIUM — tonal trigger reads only the first ~5.5 s
The 6-band tonal read is a single FFT over the first ~5.5–6 s
(`analysis.rs:322-331`, `1<<18` cap) while DR/LRA/correlation are whole-track —
an intro can classify the whole song. The misleading "30 s" comment is fixed; the
**windowing itself is not.** **Fix:** Welch-average the 6-band FFT across the
track. Caveat (good catch from the finish plan): `spectral_balance_6band` is
*shared* with role/character/album-bias inference, so widening the window shifts
those too — slightly wider blast radius than the guardrails alone.

### B7 · MEDIUM — `stereo_width` computed, carried, never used
The width guardrail is correlation-only (`guardrails.rs:114-119`); the more
on-point "already wide" measure, `stereo_width` = side/(mid+side), is populated on
the wire and ignored. **Fix:** wire it as a co-trigger with correlation, or drop
it from the profile+spec until it is.

### B8 · LOW — album boundary isn't enforced at the backend
`apply_album_shadow` (`album_render.rs`) is profile-agnostic: it never injects a
profile **but also never strips one** (the new test asserts it preserves an
incoming `Some(...)`). So a stale/hand-built/API settings payload carrying
`advanced.source_profile` would make album rendering adaptive despite the product
label. Narrow today (the profile isn't persisted into project/preset state), but
the guarantee rests on "nothing upstream ever sets it" rather than enforcement.
**Fix:** if album stays out of scope, `shadowed.advanced.source_profile = None`
in the album path + a test that a profile-bearing input is cleared.

### B9 · LOW — TS injectors don't clear a stale profile
`injectSourceProfile` / `applyChainDispatchOverrides` (`settings-transitions.ts`)
only *write* `source_profile` when analysis yields one; they never set it to
`null` when analysis is missing. Same narrow risk as B8 (a profile that leaked
into stored settings would survive a later "no analysis" path). **Fix:** write
`source_profile: profile` with `profile = null` when none can be built; or fold
into the backend-ownership fix (B2).

### B10 · LOW — second LRA hazard: LU aliased into a dB threshold
Distinct from the (now-fixed) `LRA=0` sentinel: when P95-P10 is missing,
`from_analysis` does `unwrap_or(if lra > 0.5 { lra } else { 100.0 })`
(`types.rs:144`), placing an **LU**-scaled value into the **dB**-thresholded DR
ramp (8/3 dB). A non-zero fallback (e.g. 4 LU) mis-judges against dB thresholds.
Very narrow window (P95-P10 absent but a real LRA present), but the principled fix
is cheap: carry P95-P10 as `Option` and use only `lra_raw` when it's `None`,
instead of aliasing.

### B11 · LOW — Adapt Strength control is editable but inert in album mode
The control renders and accepts edits in album mode (`App.tsx:2151-2159`, not
gated), writing into `albumIntent`, but album render never injects (B1). Today
it's a no-op with only a text note. Resolved automatically by whichever way B1 is
decided (wire it, or hide/disable it in album mode).

### B12 · LOW — minor drift / Tier-2 inputs
- **Density lever** (`guardrails.rs:111-112`, `dsp.rs:938-952`): one scalar drives
  engagement/threshold/ratio; "keeps ≥40% of compression" is true of the *macro*,
  not dB gain-reduction, and the cap already saturates at the default 0.6 on a
  fully-dense trigger (the dial is a no-op above 0.6 there). Tier-2 direction
  (reshape on engagement, or PSR-target), not a Tier-1 defect.
- **`EQ_BOOST_FLOOR_DB`** is global, not per-band.
- **Default strength** still has two literals (Rust const + `bindings.ts` `0.6`);
  better than before (no bare literal in `App.tsx`) but not codegen-synced.

### Doc hygiene (P3, real but trivial)
- Handoff still lists the per-axis readout as **both shipped and deferred**;
  its "Verification" block still shows the **old 158/224** counts (actual
  234/161).
- Committed docs link to review files that are **untracked** in the worktree —
  commit them with the review trail or soften the links.
- The next-steps backend note says backend ownership closes
  "preview/album/slow-lane/live by construction" — only true for album **if album
  becomes adaptive**, which contradicts the unadapt decision. Split the note.

---

## C. Not-real / overstated (de-noise — don't spend time here)

1. **GLOBAL F1–F5/F9/F11/F12 "CONFIRMED"** — describe pre-fix code; all addressed
   (Section A). Stale, not wrong-in-spirit.
2. **F7 "overdrive is a separate 4th saturation stage"** — GLOBAL itself
   corrected this; it's the same threshold/ratio mechanism. The
   saturation-at-0.6 numeric point stands (that's B12).
3. **"Every neutral track loses ~40% of its air"** — the 0.278 figure is
   *idealized* full-span 1/f; real masters with top-octave rolloff or bass tilt
   can sit under even the old 0.20. Worst case, not guaranteed — and moot now that
   the deadband is 0.30.
4. **F3 as a broad hazard** — only fires on short/degenerate (unmeasurable-LRA)
   sources, not normal long dynamic masters. (The fix was still correct.)

---

## D. Recommended sequence

**P0 — before the listening gate (correctness + a stable baseline)**
1. **Backend-owned `source_profile` (B2)** — derive server-side from the track
   analysis; FE profile is an override; cache for the live path. Closes B1, the
   preview/slow-lane class, and the dual-builder drift by construction. Highest
   leverage; every review converges here.
2. **Decide album scope (B1/B11)** — wire it (falls out of #1) or make album
   non-adaptive end-to-end (stop live injection + disable the control). No
   "audition adaptive / ship flat."
3. **Fix the `null`-opt-out (B4)** and **gate the second LRA aliasing (B10)** —
   both are small and both currently mislead (one re-arms silently, one
   mis-triggers); add the `healthy DR + missing/zero LRA ⇒ no density trim` test.

**P1 — calibration-enabling (still before listening)**
4. Recalibrate brightness toward measured pink/reference and **Welch-average the
   6-band FFT (B6)**; the readout (already shipped) makes the by-ear pass possible.
5. **Export-receipt traceability (B5)** so the listening A/B is attributable to a
   recorded strength.

**P2 — hardening & Tier-2 direction**
6. `stereo_width` co-trigger (B7); album backend strip + TS stale-clear (B8/B9);
   density-lever reshape (B12); doc hygiene; the missing tests (album==track
   equality, multi-axis under active LUFS landing, render-level
   `profile=Some + strength=0` byte-identity).

**Tier-2 north star (after listening locks v1 numbers):** measured neutral from
Dan's own reference masters (slow-fixture lane), optionally per-preset;
tilt-vs-reference deadband; PSR/crest closed loop — the only honest "won't crush
it further" dynamics defense.

---

## E. Two calls that are the owner's alone

1. **On-by-default at 0.6.** Defensible (sole-user calibrator, bounded blast
   radius, byte-identical off path) — **but** until B1/B2 land, the "preset as
   designed" baseline is unstable across surfaces (live=adaptive,
   track-export=adaptive, album-export=flat, and `null`/reset silently re-arm),
   with no in-receipt record of which was heard. Fixing the wiring is what makes
   "on by default" *safe* to ship.
2. **Album adaptation scope (B1).** In (wire it) or out (non-adaptive
   end-to-end). Decide before the listening session, because it changes what the
   album A/B is even comparing.

---

## Bottom line

The fix stack did the right work in the right order and closed every
release-blocking item from the first review, with tests. The residual is smaller
and more focused: **one real behavior gap (album audition≠deliverable), one
architectural root-cause fix that dissolves a whole class of wiring bugs, and a
short list of medium/low hardening items** — plus the by-ear calibration that was
always going to be the gate. None of it is a redesign. Land B1/B2 + the opt-out
and LRA fixes, ship the receipt flag, then listen.
