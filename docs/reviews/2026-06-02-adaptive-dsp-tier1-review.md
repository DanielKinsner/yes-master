# Review — Adaptive DSP Tier-1 Guardrails

Date: 2026-06-02 · **Revised 2026-06-03** (folds in two more passes — see
[Reconciliation](#reconciliation--how-this-doc-was-corrected))
Reviewer: Claude (multi-agent review — 6 dimensions, each finding adversarially verified)
Scope: the `adaptive-dsp` commit series `0d802af^..df7dc5d` (branch
`feat/adaptive-dsp-guardrails`, not merged), desktop app.
Method: parallel reviewers on math / wiring / spec-fidelity / analysis-inputs /
tests / strategy; every high/medium finding re-read against the code by an
independent verifier that tried to refute it. The 2026-06-03 revision reconciles
this against an independent desktop review (`2026-06-03-adaptive-dsp-desktop-review.md`)
and a review-of-this-review (Codex), verifying each cross-claim against the code.
Line refs below are from the reviewed tree.

---

## Verdict

**The idea is right and the core is well-built.** The trim math is clean and its
four defensive guarantees (reduce-only, never-flip-sign, never-narrow,
mono-never-trims-width) genuinely hold for every input I could construct. The
architecture — a *pure, level-invariant pre-stage inside `ChainCoeffs::from_settings`
that is byte-identical to the old chain when no profile is present* — is exactly
the right shape, and the `preset_byte_identity` SHA snapshots prove the
non-adaptive path is untouched.

**But it is not ready to trust for the listening gate yet**, for three reasons
that all trace to a single architectural decision and two un-calibrated numbers:

1. **Injection is wired per-frontend-call-site, not at the engine.**
   *(Corrected in revision — the original draft over-broadened this; see the
   [injection map](#corrected-desktop-injection-map-what-actually-runs-the-guardrails).)*
   Live Track Master audition and Track Master export **are** wired and consistent.
   But the **offline preview-WAV render** silently runs the *old* chain while
   export runs the adapted chain — a real WYSIWYG bug, since that button exists to
   audit the would-be master in another player. The **private fixture/reference
   slow lanes never set `source_profile`**, so the evidence lane that CLAUDE.md
   makes the gate for DSP/export merges validates the non-adaptive chain on the
   exact stress class this feature targets. **Album Master** export also skips it
   (deferred by the handoff — a product gap, not a code defect). iPhone is
   *intentionally* out of scope (`docs/IPHONE_APP.md:61`), not a defect. The root
   cause behind all the desktop misses is the same: injection lives in TS at each
   call site, so any path that forgets it runs un-adapted.
2. **A degenerate-LRA value misfires the density guardrail.** A non-finite EBU
   LRA is coerced to `0.0`, which the density ramp reads as *maximally
   compressed* and applies a full 60% compression trim — on a source that may be
   perfectly dynamic.
3. **The brightness trigger fires on genuinely neutral material.** A true
   pink-tilted master sits *above* the 0.20 deadband, so "neutral" tracks lose
   ~40% of their air lift at the default strength. The number is provisional by
   design, but it's provisional in the wrong direction (over-acts, not under-acts).

None of these is a deep design flaw; all are fixable in a focused 1–2 day pass,
and the right sequence is **fix the wiring + ship the "what was trimmed" readout
*before* the listening session**, because the session cannot produce calibrated
numbers without attribution.

---

## Findings, ranked

| # | Severity | Finding | Where |
|---|---|---|---|
| 1 | **High** (release-evidence) | **Slow lanes don't exercise the feature.** The private fixture matrix and reference-tuning runners set `source_lufs_integrated` but never `advanced.source_profile`, so the mandated already-mastered evidence lane renders the *old* chain | `fixture_matrix.rs:124`; `reference_tuning.rs:301`; gated at `dsp.rs:756-761` |
| 2 | **High** (WYSIWYG bug) | **Offline preview-WAV render skips injection** → the "audit in another player" WAV is old-chain while export is adapted-chain | `useTrackMaster.ts:1208-1218` (no `injectSourceProfile`) vs `:1247` (export injects); `engine.rs:335-362` trusts received settings |
| 3 | **High** | Non-finite LRA coerced to `0.0` → density ramp reads "maximally dense" → spurious full 60% compression trim | `analysis.rs:159`, `guardrails.rs:110-112`, `types.rs:144` |
| 4 | **High** | Brightness deadband `0.20` fires on a pink-tilted neutral master (presence+air ≈ 0.28 for true pink) → neutral sources lose ~40% of air lift at default strength | `guardrails.rs:30,95-97`; `analysis.rs:361` |
| 5 | **High** | Spectral 6-band trigger reads only the **first ~5.5–6 s** (single FFT, 2¹⁸ cap) while DR/LRA/correlation are whole-track → inconsistent window misreads atypical intros | `analysis.rs:322-331` |
| 6 | **Medium** (product gap, deferred) | Album Master export never injects the profile → album deliverable ≠ track audition. Documented as deferred; becomes a defect only if UI/copy implies adaptive coverage | `album_render.rs:65-98` (`apply_album_shadow`); handoff "Deferred" |
| 7 | **Medium** | Density "macro" is a blunt lever: one scalar drives engagement+threshold+ratio+overdrive; "keeps ≥40% of compression" is true of the macro, not of gain-reduction-in-dB; at strength 0.6 the cap already saturates a fully-dense trigger | `guardrails.rs:111-112,153`; `dsp.rs:938,942-952` |
| 8 | **Medium** | Multi-axis loudness recomposition via the **LUFS-landing stage** (not the limiter): combined trims lose LUFS *and* free true-peak headroom, so the landing makeup recoups the loss → all-four-axis delta can exceed any single per-axis cap (inert only on Custom-with-no-target). *Unique to this pass.* | `engine.rs:106-127`; `dsp.rs:665,2041-2043`; `guardrails.rs:121-126` |
| 9 | **Medium** | `SourceProfile::from_analysis` (Rust) is **dead code**; the only injector is the TS twin → silent drift trap (and the type comment claims backend-derived export, which isn't true) | `types.rs:113-117,140` (no callers) vs `settings-transitions.ts:sourceProfileFromAnalysis` |
| 10 | **Medium** | Whole-track Pearson correlation is blind to localized/per-band widening; the better measure (`stereo_width` side-fraction) is computed, carried on the wire, and **never read** by `compute()` | `analysis.rs:226-246,397-431`; `guardrails.rs:115-119` |
| 11 | **Medium** | No "what was trimmed and why" readout → the listening session can A/B the global dial but cannot attribute *why* a preset eased off; deferred, but it is the gating dependency for calibration, not polish | spec §"Open questions" #5; handoff "Deferred" |
| 12 | **Low** | Spec text drift (code is correct): bright "Touches" wrongly lists `presence_db` (1.5 kHz mid, correctly untrimmed); width row claims a `stereo_width` secondary signal that isn't implemented | spec table vs `dsp.rs:737,789-791` |
| 13 | **Low** | `EQ_BOOST_FLOOR_DB` is global, not per-band; two source-of-truth copies of the default strength (Rust const + TS `?? 0.6`); low-share `0.42` deadband may over-fire on legitimately bass-forward genres | `guardrails.rs:17,60`; `App.tsx:2140` |
| — | scoped out | iPhone shares the engine but never injects → runs un-adapted. **Intentional**, documented "no adaptive/smart analysis in v1" — not a defect | `docs/IPHONE_APP.md:16,61`; `IPHONE_APP_OVERVIEW.md:36` |

### Corrected desktop injection map (what actually runs the guardrails)

This table replaces the original draft's over-broad "audition ≠ deliverable"
claim. Each row verified against the code/docs in the 2026-06-03 revision.

| Path | Injects `source_profile`? | Status |
|---|---|---|
| Live audition (`play_master` / `update_chain`) | ✅ yes — TS `applyChainDispatchOverrides` (`useTrackMaster.ts:305`) | working |
| Track Master export (`renderTrackMaster`) | ✅ yes — TS `injectSourceProfile` (`useTrackMaster.ts:1247`) | working |
| Offline preview WAV (`renderTrackPreview`) | ❌ no | **real WYSIWYG bug** (#2) |
| Album Master export (`apply_album_shadow`) | ❌ no | **deferred** → product gap (#6) |
| Fixture / reference slow lanes | ❌ no | **release-evidence gap** (#1) |
| iPhone | ❌ no | **out of scope by docs** — intended, not a defect |

So for a single Track Master, audition **does** equal the deliverable. The genuine
*user-facing* desktop break is the preview-WAV button; album is a deferral; the
slow lanes are an evidence hole; iPhone is scoped out. The common root cause is
per-call-site TS injection — see the architectural fix below.

### Test gaps (all confirmed)
- **No WYSIWYG test** that export-path injection == live-path injection.
- **No fixture/reference test** asserting `advanced.source_profile` is present in
  the slow-lane settings — so finding #1 is an untested regression hiding in the
  seam (the exact lane meant to catch it).
- **No album-export injection test** (consistent with the deferral, but it should
  be a deliberate "asserts absence" test once the scope decision is recorded).
- **No multi-axis composition test** (bright AND dense AND wide together, each
  capped independently).
- **No manual-override-preservation test** at the adaptive layer (user `eq_*_db`,
  Manual compression, explicit `advanced.width` untouched).
- **No test** that `profile = Some` + `strength = 0` is byte-identical (only the
  no-profile path is snapshotted).
- The behavioral render test is sound (it isolates the limiter with quiet noise)
  but its single inequality assertion is weak — several bug modes would still pass
  it; it's backstopped only by the coefficient-level `..._without_touching_low`
  test.

---

## What's genuinely good (don't change it)

- **The trim math.** `compute()` is a clean deadband → linear ramp → strength
  scale → per-axis cap → character-floor pipeline. The cap correctly caps the
  *trim* (`min(raw*strength, cap)`), not the multiplier. `descending_ramp`'s
  degenerate `soft == full` guard is correct. `floor_boost` correctly passes
  cuts/zero through and never *raises* an already-tiny boost.
- **The four invariants hold for all inputs** I tested, including negative
  correlation, shares that don't sum to 1, and the intensity (`preset_scale`)
  ordering (trim acts on the already-scaled preset move, before the user offset
  is added — so manual edits stay explicit overrides).
- **Byte-identity is real.** No profile ⇒ identity closures ⇒ chain output is
  literally unchanged; the SHA snapshots prove it.
- **Level-invariance is the right call.** Shares/ratios mean no LUFS
  normalization is needed before comparison — the survey's #1 pitfall genuinely
  doesn't apply here.
- **EQ band mapping is correct, and more correct than the spec.** The bright
  guardrail trims 3.5 k / 6 k / 12 k (`high_mid`/`air`/`sparkle`) and correctly
  leaves `presence_db` (1.5 kHz, the "mid" band) alone — the spec's "Touches"
  list is wrong to include it.
- **Defensive-trim is the right v1 frame.** Steelmanned against a corrective
  target-curve (LANDR/iZotope) and a Sonible-style single-Impact engine, "only
  ease our own preset moves, never push past flat" is the correct,
  low-blast-radius starting point for a single-engineer tool with a hard
  "exports never silently mutate" non-negotiable.

---

## The math, checked

| Axis | Formula (verified) | Holds? |
|---|---|---|
| Bright | `raw = clamp01((presence+air − 0.20)/0.12)`; `mult = 1 − min(raw·s, 0.50)` | ✅ caps trim at 50%, floors boosts at +0.5 dB |
| Boomy | `raw = clamp01((sub+low − 0.42)/0.15)`; `mult = 1 − min(raw·s, 0.50)` | ✅ same |
| Dense | `raw = max(ramp(DR,8,3), ramp(LRA,6,3))`; `mult = 1 − min(raw·s, 0.60)` | ⚠️ macro-level cap only; LRA-0.0 hazard (#2) |
| Wide | `raw = ramp(corr,0.50,0.20)` (mono ⇒ 0); `mult = 1 − min(raw·s, 0.70)` | ✅ never narrows (`preset_width ≤ 1.0` passes through) |

Two places the math diverges from the **stated guarantee** (not from the code's
intent):

- **#3 deadband:** for power-share bands summing to 1.0, a true 1/f (pink)
  spectrum yields presence+air ≈ `(ln(6500/2500)+ln(16000/6500)) / ln(16000/20)`
  ≈ 0.278 — above the 0.20 edge. The unit-test "neutral" fixture
  (`presence 0.08, air 0.05`) is mid-heavy synthetic and *doesn't* reflect how
  real FFT power-shares fall out, which is why the misfire isn't caught.
  **Fix:** raise the bright deadband to ~0.30, or compare against pink-reference
  shares per band; add a real-pink-noise test asserting no trim.
- **#5 density:** the 60% cap is on the macro `[0,1]`. Because density drives
  `preset_engagement = (density·2).min(1)` *and* the threshold scale toward 0 dBFS
  *and* the ratio toward 1.0, a worst-case dense source loses far more than 60% of
  actual gain reduction in dB. And since the cap saturates at `s = 0.6`
  (`min(1.0·0.6, 0.60) = 0.60`), the default strength already behaves like full
  strength on a fully-dense trigger. **Fix:** either reword the spec to "≥40% of
  the density macro," or apply the cap on `engagement` (post-mapping) so "≥40% of
  compression character" is true in the gain-reduction sense.

---

## Strategy critique — where it falls short, and the better path

**Was the strategy good?** Yes, with caveats. Defensive-only trimming is the
correct v1, the research framing is honest about sourced-vs-inferred, and the
caps/floor keep presets recognizable. The shortfalls are detection quality and
plumbing, not philosophy:

1. **Share-based tonal detection misclassifies the two cases you most need to
   catch.** Relative tilt ("what fraction of energy is up top") is not the same as
   "this master is harsh." A bright-but-quiet-highs source and a
   dark-but-bass-heavy source both fool it. **Better:** add **spectral slope /
   tilt** (dB-per-octave regression, which you nearly have from the 6 bands) as a
   co-trigger alongside share — slope is what the LTAS literature the spec cites
   actually measures.
2. **Correlation is a coarse "wide" proxy.** It conflates *phasey* with *wide* and
   is blind to per-band or localized widening. You already compute the better
   measure (`stereo_width` = side/(mid+side)) and then ignore it. **Better:**
   consume side/mid as a co-trigger; the wire field is already there.
3. **Density-macro scaling can't promise "I won't crush your transients
   further."** That promise needs a **PSR / crest-factor** target, not a macro
   scalar. You already compute `lufs_short_term_max_3s`. **Better (Tier-2):**
   closed-loop — if predicted post-chain PSR < source PSR − 1 dB, pull density
   harder; this is the only *honest* "don't make it worse" dynamics defense.
4. **The analysis window is inconsistent** (#4): tonal shares from the first ~6 s,
   dynamics/stereo from the whole track. **Better:** Welch-average the 6-band FFT
   across the track (a few overlapping windows), so the brightness/boominess
   decision reflects the song, not the intro.
5. **No transparency = no calibration.** Default-ON at 0.6 is *defensible*
   (single user = calibrator; damage is bounded; the limiter does **not**
   silently recompose trims — that worry was checked and refuted) but the missing
   readout means the listening A/B can only judge the *global dial*, not *attribute
   per-axis*. Treat the readout as day-one scope.

### The single architectural change that fixes the most at once
Make the **backend own and store the per-track analysis profile**, and attach it
wherever a chain is built — not just at offline "render entry points." This is the
sharpened version of the original recommendation (the first draft said "inject at
the backend render entry point," which is necessary but **not sufficient**):

- Offline commands (`renderTrackMaster`, `renderTrackPreview`, fixture matrix,
  reference tuning) all carry a track id / path, so they can derive the profile
  from `AnalysisResult` at the entry point. ✅ entry-point injection covers these.
- **The live sweep path does not.** `update_chain(settings, …)` (`audio.rs:107`)
  carries **settings only — no track id**; `play_master` (`audio.rs:80`) has the
  id but the high-frequency knob-sweep updates flow through `update_chain`. So the
  backend must cache the active track's profile (it already holds the current
  source) and re-attach it on every coeff build, or live injection stays in TS.

Done right, this one move:
- closes the **preview-WAV**, **album-export**, and **slow-lane** gaps by
  construction (every chain build inherits the profile),
- covers the **live** path too (via the stored profile, not just render commands),
- removes the **dead Rust `from_analysis` / TS-twin drift trap** (one source of
  truth — the currently-unused Rust mapping becomes the real one),
- removes the need for a WYSIWYG test to babysit hand-wired call sites.

The current per-call-site TS injection is exactly why so many paths already forgot
to do it (preview, album, and both slow lanes). iPhone stays opt-out by design.

### Concrete plan
**Deliverable now (1–2 days, before the listening session) — priority order:**
1. **Centralize profile construction in the backend** and make it own/store the
   per-track analysis. Wire it into `renderTrackPreview`, the fixture matrix, and
   reference tuning **first** (closes #1 and #2 — the release-evidence hole and
   the WYSIWYG bug). Decide album scope explicitly (#6): if it stays out, say so in
   UI/docs; if in, inject per-track before `apply_album_shadow` renders each track.
2. Gate the LRA-`0.0` sentinel (#3): treat non-finite / ≤0.5 LRA as "unknown,"
   zero its ramp contribution; mirror in `sourceProfileFromAnalysis`; add the
   regression test (`DR=10, LRA=0 ⇒ no density trim`).
3. Raise the bright deadband to ~0.30 (or pink-reference per band) (#4); fix the
   first-window FFT — Welch-average across the track (#5); add a real-pink-noise
   no-trim test.
4. Ship the per-axis "what was trimmed" readout (#11) — the trims are already
   computed; surface them. This is the gating dependency for calibration, not
   polish.
5. Add slope and side/mid co-triggers (#10); reword or re-anchor the density cap (#7).
6. Add the missing tests: slow-lane profile-present, WYSIWYG parity, multi-axis
   composition, manual-override preservation, `profile=Some + strength=0`
   byte-identity.
7. Delete or actually wire the Rust `from_analysis` (#9); collapse the two
   default-strength copies to one (#13).

**Tier-2 north star (after listening locks v1 numbers):** build the neutral
reference from **Dan's own reference masters** in the slow-fixture lane
(`AMS_RUN_REAL_FIXTURE`) — replace the inferred octave-slope deadbands with
*measured* per-band shares, optionally per-preset (Spatial's neutral ≠ Loud's).
Still defensive (trim toward the owner's measured neutral, never past it), but
informed by data that exists rather than a 6-band inference from per-octave slope
studies. Add the PSR closed-loop for dynamics.

---

## Adversarial corrections folded in (so the record is honest)

- **"Caps compose multiplicatively through the limiter's makeup" — refuted.** The
  brickwall limiter is pure peak attenuation with no makeup; the compressor's
  auto-makeup *drops* when density is trimmed; Volume Match is off by default and
  only attenuates. The real loudness-recomposition path is the **LUFS-landing
  stage** when a delivery target is set (#6), which is a narrower and accurate
  version of the concern.
- **"Scooped-mid triggers both bright and boomy" — downgraded to Low.** Real but
  bounded by caps/floor; flagged in the spec as a known per-axis-independence
  tradeoff.
- **"Density macro is the wrong lever" — downgraded to Medium / Tier-2 input.**
  Mechanically accurate, but the spec explicitly scopes target-style dynamics to
  Tier-2; file it as the Tier-2 dynamics direction, not a Tier-1 defect.

---

## Reconciliation — how this doc was corrected

This review has now survived two more passes. Recording what each changed, so the
provenance is honest:

**Independent desktop review (`2026-06-03-adaptive-dsp-desktop-review.md`)** —
converged on ~9 of the same findings with the same evidence and the same
backend-ownership recommendation (strong cross-confirmation). It contributed the
finding this pass *missed*: the **private fixture/reference slow lanes never set
`source_profile`** (now finding #1), verified at `fixture_matrix.rs:124` and
`reference_tuning.rs:301`. For *release confidence* that is the most important
single item, because CLAUDE.md makes that lane the merge gate.

**Review-of-this-review (Codex)** — fair and mostly correct; three corrections
landed, each re-verified here against the code/docs:

1. **The original finding #1 was over-broad.** Live audition and Track Master
   export *are* wired and consistent (`useTrackMaster.ts:305` / `:1247`); the real
   desktop break is the preview-WAV path. Table + verdict rewritten; see the
   [injection map](#corrected-desktop-injection-map-what-actually-runs-the-guardrails).
2. **iPhone is out of scope by docs, not a defect** (`IPHONE_APP.md:61`). Reframed.
3. **"Backend render entry point" was necessary but not sufficient.** `update_chain`
   (`audio.rs:107`) carries settings only — no track id — so the live path needs
   the backend to *own/store* the analysis profile, not just inject at render
   entry points. Recommendation sharpened.

   *Minor pushback (verified):* Codex's framing implied finding #1 had called the
   Track Master live/export paths broken. It hadn't — the original already scoped
   the misses to album/preview/iPhone and noted TM export injects. The fair part of
   the hit is the over-broad "audition ≠ deliverable" headline and the iPhone
   bundling, both now corrected.

**What stays unique to this pass:** the multi-axis loudness recomposition via the
LUFS-landing stage (#8) — neither external pass identified it; it was
adversarially verified here (the brickwall limiter has no makeup; the
recomposition is the post-chain landing gain when a delivery target is active).
