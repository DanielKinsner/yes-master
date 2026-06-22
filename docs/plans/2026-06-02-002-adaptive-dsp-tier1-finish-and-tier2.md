# Adaptive DSP — Tier-1 finish + Tier-2 plan

Date: 2026-06-02
Branch: `feat/adaptive-dsp-guardrails`
Context: post-review (`docs/archive/reviews/2026-06-02-adaptive-dsp-tier1-review.md`,
`docs/archive/reviews/2026-06-03-adaptive-dsp-desktop-review.md`), verified against the
code by a fan-out workflow. The **mechanical bugs are already fixed** (see
"Already done"). This plan covers the **taste / new-math / architecture**
decisions that need Dan's ear or a design call before implementing.

> **The status-tagged backlog / entry point is
> [`docs/ADAPTIVE_DSP_NEXT_STEPS.md`](../ADAPTIVE_DSP_NEXT_STEPS.md).** This doc
> holds the detailed options + rationale; that one is the at-a-glance "what's
> left" (incl. Tier-2 and the parallel Simple Mode view).

## Already done this session (mechanical, committed)

- **LRA=0.0 sentinel** no longer forces a full density trim (`f6c7cc8`).
- **Preview WAV + both slow lanes** now run the adaptive chain; the dead Rust
  `SourceProfile::from_analysis` is now the slow-lane builder (`dc62dab`).
- **Invariant tests**: strength=0 inert with a trigger profile, multi-axis
  composition, manual-override survival (`27f2a4c`).
- **Doc/comment drift + default-strength dedup** (`44a44ef`).

Reviews verdict held up: no false load-bearing claims, and the trim math has no
hidden NaN/invariant bug. What remains is genuinely calibration + design.

---

## Decisions (each: what · options · recommendation · the call that's yours)

### 1. Bright/low deadband misfires on *neutral* masters — HIGH

**What:** a true pink-tilted master has `presence+air ≈ 0.278`, which sits **above**
the `0.20` bright deadband — so a genuinely neutral track loses ~39% of its air
lift at default strength. The feature's whole premise is "do nothing when nothing's
wrong"; right now it over-acts on normal material. (Same risk on `LOW_DEADBAND`
for bass-forward genres.)

**Options:**
- **(a) Quick bump** — raise `BRIGHT_DEADBAND` to ~0.30 (above natural pink), same
  for low. ~10 min, unblocks listening, keeps the simple share metric.
- **(b) Tilt-vs-reference** — compare the source's spectral **slope** (dB/oct) to
  the pink reference, so a flat 1/f reads as *zero excess by construction*,
  independent of our band edges. More principled (it's what the LTAS literature
  the spec cites actually measures), robust, but new math in `compute()`.
- **(c) Per-band pink-reference shares** — deviation from each band's pink-expected
  share. Middle ground.

**Recommendation:** **(a) now to unblock your listening session, (b) as the proper
fix.** The bump is an honest "the 0.20 was demonstrably too low"; the tilt metric is
the real answer and makes the deadband number almost irrelevant.

**Your call:** ship the quick bump now, or wait and do the tilt metric? (And: do you
want me to *measure* your own reference masters to set the number, rather than use
the textbook pink value? — see #8.)

### 2. Density cap vs default strength — MEDIUM

**What:** at default strength `0.6`, a fully-dense source pins `density_mult` to
`0.4` (`min(1.0·0.6, 0.60)`), so the Adapt dial is a **no-op above 0.6** on dense
material, and "keeps ≥40% of compression" is true of the *macro*, not of dB
gain-reduction (the macro also drives engagement/threshold/ratio, so the real GR
change is bigger than 40% suggests).

**Options:** (a) lower the default to ~0.5 (gives the dial headroom under the cap);
(b) apply the trim on the **engagement curve** (post-mapping) so "keep N%" is honest
in dB GR; (c) doc-only reword.

**Recommendation:** **(b) + revisit the default after listening.** Honest semantics
matter more than the exact number.

**Your call:** OK to reshape density-trim against engagement (slightly changes how
dense sources soften), or keep the macro-scaling and just reword?

### 3. Spectral window: first ~5.5 s → whole-track — MEDIUM

**What:** the 6-band tonal read is the **first ~5.5 s** (a `1<<18` FFT cap), while
DR/LRA/correlation are whole-track — so an intro can misclassify the song.

**Recommendation:** Welch-average the 6-band FFT across the whole track (a few
overlapping windows). **Caveat:** `spectral_balance_6band` is *shared* — it also
feeds role/character inference and album bias, so this shifts those slightly too
(all for the better, but it's a wider blast radius than the guardrails alone).

**Your call:** OK to change the shared analysis window (affects role/character/album
inference too), or keep it guardrail-local somehow?

### 4. `stereo_width` as a width co-trigger — MEDIUM

**What:** the width guardrail uses correlation only; the better "already wide" signal
(`stereo_width` = side/(mid+side)) is computed and carried but **unused**.

**Recommendation:** wire it as a co-trigger with correlation (it's already on the
wire). New threshold = calibration.

**Your call:** wire it now, or leave correlation-only for v1 and revisit?

### 5. Backend-owned profile architecture — MEDIUM (robustness)

**What:** the root cause behind every wiring miss is that injection lives in TS
**per call site**, and `update_chain` carries settings only (no track id). The clean
fix: the backend **caches the loaded track's profile** (in `AudioThreadState`,
populated on `play_master`) and attaches it on every coeff build; render derives it
defensively from decoded PCM. This closes preview/album/slow-lane/live **by
construction** and removes the TS↔Rust dual builder.

**Status:** the tactical fixes already closed the *known* holes, so this is
future-proofing + cleanliness, not a current break.

**Recommendation:** do it — it's the difference between "we patched the four sites we
found" and "a fifth site can't forget." But it's a real refactor (audio-thread
state, a new command or analysis-threading), so it's your call on timing.

**Your call:** do the backend-ownership refactor now (cleaner, future-proof), or bank
the tactical fixes and revisit later?

### 6. Album Master scope — product decision

**What:** Album export never injects the profile (deferred). It's a real desktop
output path with no adaptive protection.

**Options:** (a) keep album explicitly unadapted and **say so in UI/docs** (an
"adaptive applies to Track Master" note); (b) bring it in — inject per-track before
`apply_album_shadow` renders each track.

**Recommendation:** depends on whether you think of albums as "already hand-tuned per
track" (then (a)) or "want the same safety net" (then (b)). Leaning (a) for now +
the honest label, since album has its own per-character bias already.

**Your call:** album stays unadapted (labeled), or gets per-track profiles?

### 7. "What was trimmed and why" readout — the calibration enabler

**What:** both reviews call this the **gating dependency for calibration** — the
listening A/B can move the global dial but can't tell you *which* axis (bright/boomy/
dense/wide) eased off or by how much. The trims are already computed; this is
surfacing them (read-only) to the right rail / Advanced panel.

**Recommendation:** **build it before the listening session.** Without it, you're
tuning blind. Label it "chain trims (pre-landing)" to stay honest about the
LUFS-landing recomposition (#8 of the tier1 review).

**Your call:** build the readout now (I'd recommend yes), and where — Advanced panel,
or a dev/review-only overlay?

### 8. Tier-2 north star — measured neutral + closed-loop dynamics

Now that the slow lane exercises the adaptive chain, the big upgrade is to **replace
the textbook pink reference with your own measured neutral**, built from your
reference masters (`AMS_RUN_REAL_FIXTURE`), optionally **per-preset** (Spatial's
neutral ≠ Loud's). Still defensive (trim toward the measured neutral, never past).
Plus a **PSR/crest closed loop** for dynamics ("if predicted post-chain PSR < source
PSR − 1 dB, pull density harder") — the only *honest* "won't crush your transients"
defense. This is the real "smart" tier.

**Your call:** appetite/sequencing — is Tier-2 the next milestone after we lock v1 by
ear, or do you want to ship a clean v1 first?

### 9. (minor) `LRA → Option<f32>` cleanup

The minimal guard I shipped is correct. A deeper cleanup would make
`AnalysisResult.dynamic_range_lu` an `Option<f32>` so "missing" is distinct from
"0 LU" everywhere (not just in the guardrail). Wider blast radius (export checks,
receipts, bindings). Low priority — flag only.

---

## Recommended sequencing

**Before the listening session (small, high-leverage):**
1. Bright/low deadband — at least the quick bump (#1a) so neutral tracks stop losing
   air.
2. The "what was trimmed" readout (#7) — so the session can attribute per-axis.

**During/after listening (calibration):**
3. Density semantics (#2), `stereo_width` co-trigger (#4), and the tilt metric (#1b)
   — tuned by ear, ideally against your measured references.

**Cleanup / robustness (any time):**
4. Backend-owned profile (#5), Welch window (#3), album scope decision (#6).

**Next milestone:**
5. Tier-2 measured-neutral + PSR closed loop (#8).

The questions that gate my next code are #1, #5, #6, #7 — asked separately.
