# Adaptive DSP — Tier-1 Defensive Guardrails (Spec)

Date: 2026-06-02
Branch: `feat/adaptive-dsp-guardrails`
Status: spec for build; numeric thresholds are provisional and meant to be
calibrated by ear.

## TL;DR

Make YES Master's presets *fit the track* instead of applying the same move to
every source. When a source **already has** a quality, the matching preset move
is **trimmed** — never added to, never inverted. One **Adapt Strength** dial
controls how hard the guardrails work; at strength 0 every preset is byte-for-
byte identical to today.

In scope (v1, "Tier-1 defensive"):

1. Already-bright → trim the preset's air/presence lift.
2. Already-boomy → trim the preset's low/sub lift.
3. Already-dense → soften the preset's compression (and, at the extreme, drive
   the existing Compressor `Off` mode).
4. Already-wide → trim the preset's widening.

Out of scope (deferred to "Tier-2 corrective", see end): pushing a source toward
a target/reference curve, genre classification, reference-track upload, dynamic
notching, per-band width matching. Those are how LANDR/iZotope work; they are a
bigger, separate effort.

## Why this shape (what the research found)

A multi-source, fact-checked survey (LANDR + its patent US9654869B2, BandLab,
iZotope Ozone / Tonal Balance Control, Sonible smart:EQ/comp/limit, eMastered /
CloudBounce / Bakuage, Gullfoss / soothe2 / FabFilter, and the academic LTAS
literature) produced these load-bearing, **sourced** conclusions:

- **Most commercial "smart" mastering is corrective**, not defensive. LANDR's
  patent and iZotope's Tonal Balance Control both pre-compute *target curves* and
  push the source toward them. That is powerful but it is Tier-2. Our Tier-1
  borrows their *analysis* (what to measure) and rejects their *correction* (we
  only trim our own preset moves).
- **Express "on target" as a range, not a point.** Every surveyed tool uses a
  tolerance band (iZotope's "tunnel", Sonible's safe-zone, REFERENCE 3's ±3 dB).
  → We use a **deadband**: inside it, the guardrail does nothing.
- **One graduated strength control** on top of internally complex moves
  (Sonible "Impact", Gullfoss "Tame", BandLab "Intensity", Reiss "severity").
  → One **Adapt Strength** dial, not per-band sliders.
- **Cap the maximum trim so presets stay recognizable.** Gullfoss tops out ~6 dB
  even at extremes; pedagogy treats >3 dB as clearly audible.
  → Hard **per-axis caps** + a character floor, independent of strength.
- **Trim existing moves only; never introduce a new boost or cut.** soothe2 and
  Ozone Stabilizer "Cut Mode" are the clean precedents.
- **Decouple loudness from tonal/dynamics decisions.** Every system keeps the
  LUFS target a separate user choice applied at the limiter. YES Master already
  does this — we preserve it. Adaptive trim **never** feeds back into loudness.
- **Show what was trimmed and why.** Competitors are opaque ("black box"); this
  is an unmet UX surface and it matches our non-negotiable on clear metering and
  review states.

### Neutral reference (sourced, then mapped — read the caveat)

The universal "balanced master" spectrum is a pink-tilted slope of ~4.5–5
dB/octave from ~100 Hz to ~4 kHz, with widest variance in the bass and tightest
in the 200 Hz–4 kHz core. This is **well-sourced**: Pestana, Ma, Reiss, Barbosa &
Black, AES 2013 (772 #1 hits, ~5 dB/oct) and Elowsson & Friberg, AES 2017
(12,345 tracks, 4.53 dB/oct, bass std-dev up to 10 dB).

**Caveat (from the adversarial pass):** turning that continuous slope into
per-band % windows for *our specific 6 bands* is **inference**, not measurement —
our band edges differ from the studies' and from iZotope's. So the per-band
numbers below are **starting points to calibrate against our own references**,
not empirical truth. The deadbands are deliberately wide so imperfect defaults
stay safe.

### Corrections applied from the fact-check (so we don't ship myths)

- Dropped a fabricated "Sage Audio / Waves" citation that appeared in the draft.
- LANDR's 0.45–0.55 deadband is a **spatial/stereo balance angle**, not a tonal
  band-share threshold — not used as a tonal precedent.
- Hafezi & Reiss is **cut-only** EQ (adds cuts), which is *not* our
  trim-existing-only constraint; only soothe2 / Ozone Stabilizer Cut Mode are
  clean precedents.
- Crest factor is **dB** (a dimensionless ratio), not "LU"; units kept straight.
- Wiklund 2021 per-genre numbers are a small student-thesis sample — treated as
  illustrative of *shape*, never as canonical targets.

## Architecture

One signal path, one engine. Guardrails are a thin, **pure** pre-stage that
adjusts how the preset is resolved into coefficients. They live entirely inside
the existing `ChainCoeffs::from_settings` resolution.

### Data carried (the source profile)

A compact, **level-invariant** profile rides along with `MasteringSettings`,
exactly like the existing `source_lufs_integrated` injection
(`src/lib/settings-transitions.ts::applyChainDispatchOverrides`):

- `spectral_balance_6band` (sub / low / low_mid / mid / presence / air shares)
- `dynamic_range_p95_p10_db`
- `dynamic_range_lu` (LRA)
- `stereo_correlation` (Option; None for mono)
- `stereo_width`

These are **shares and ratios**, so they're independent of how loud the source
is — which is why we do **not** need a separate LUFS-normalization step (the
research's #1 pitfall for absolute-LTAS comparison simply doesn't apply to us).

Our actual band edges (`src-tauri/src/analysis.rs`): sub 20–80, low 80–250,
low_mid 250–800, mid 800–2500, presence 2500–6500, air 6500–min(Nyquist,16k) Hz.

### The hook

`ChainCoeffs::from_settings` stays signature-compatible; a new
`SourceGuardrails` is computed from the profile + strength and applied where the
preset is resolved:

- **EQ** (`dsp.rs:744-750`): scale the **preset contribution only**
  (`preset.X_db * preset_scale`), leaving the user's manual `settings.eq_X_db`
  untouched — "manual edits stay explicit overrides."
- **Compression** (`dsp.rs:895-915`): scale the `density` macro down (and, at the
  hard gate, route to `Off`) — but **only in Preset mode**, never in Manual.
- **Width** (`dsp.rs:859-863`): pull the `preset_width` default toward 1.0 — only
  when the user hasn't set an explicit `advanced.width`.

```
SourceGuardrails::from_profile(profile, strength) -> trims   // pure, unit-tested
from_settings(... ) {
    let g = settings.source_profile
        .filter(|_| adaptive_enabled && strength > 0)
        .map(|p| SourceGuardrails::from_profile(p, strength));
    // apply g.* to preset contributions; g == None => byte-identical to today
}
```

### Why this is safe by construction

Guardrails fire **iff** a source profile is present **and** strength > 0. The
`preset_byte_identity` SHA-snapshot tests (`dsp.rs:3061`) run on synthetic pink
noise with hand-built settings and **no profile** → they stay byte-identical with
zero changes. Existing integration tests (`delivery_profile_render`, `contracts`,
`album_sample_rate`) construct settings directly without a profile → unaffected.
The feature activates only through the live-dispatch and export paths that
actually have analysis. Guardrail fixture tests opt in by setting a profile
explicitly.

## The Adapt Strength control (the sensitivity dial)

A single `adaptive_strength` in `[0, 1]` (None = default). This is the knob the
owner tunes by ear.

- **0.0** → guardrails off; presets identical to today (the A/B reference).
- **default** → provisional **0.6** ("standard"); owner taste call.
- Higher → more correction.

Defensive guarantees that hold at **any** strength (this is what stops "throttled
tonal curves"):

1. **Reduce-only, toward-neutral-only.** A trim can only ease a positive preset
   move toward flat; it can never push past flat or flip sign.
2. **Per-axis caps** (independent of strength): a guardrail may remove at most
   **50%** of an EQ move, **60%** of compression, **70%** of widening.
   `applied_trim = min(raw_trim * strength, axis_cap)`.
3. **Character floor.** Positive boosts never trimmed below +0.5 dB; widening
   keeps ≥30% of preset intent unless correlation is critical. "Open" stays open.
4. **Realtime-tunable.** Strength is a multiplier in `from_settings`, so it can be
   swept live during audition like Intensity/EQ — no re-render.

Per-dimension strength (EQ touchier than compression, etc.) is an easy later
refinement; v1 ships one global dial.

## The four guardrails (provisional defaults — calibrate by ear)

Common shape: a **deadband** (no action inside the typical range) then a
soft-knee linear ramp on the excess past the band:
`raw_trim = clamp01((measure - deadband_edge) / excess_at_full)`.

| Guardrail | Trigger (our fields) | Deadband edge (provisional) | Full-trim at | Axis cap | Touches |
|---|---|---|---|---|---|
| **Already-bright** | `presence + air` share | ≥ 0.30 (above natural pink ~0.278) | +0.12 over edge | 50% | preset `air_db`, `sparkle_db`, `high_mid_db` (positive lifts only; **not** `presence_db`, which is 1.5 kHz mid) |
| **Already-boomy** | `sub + low` share | ≥ 0.42 (wider band — bass variance) | +0.15 over edge | 50% | preset `sub_db`, `low_shelf_db` (positive lifts only) |
| **Already-dense** | `dynamic_range_p95_p10_db` (primary) + `dynamic_range_lu` (secondary) | DR ≤ 8 dB / LRA ≤ 6 LU | DR 3 dB / LRA 3 LU | 60% | preset `density` (Preset mode only) |
| **Already-wide** | `stereo_correlation` (`stereo_width` is carried but **not yet** a trigger — see plan) | corr ≤ 0.50 | corr 0.20 | 70% | preset `stereo_width` toward 1.0 (when no explicit user width) |

Notes:
- All thresholds above are **inference**, flagged as provisional. They are
  conservative (wide deadbands) so a wrong guess under-acts rather than misfires.
- **v1 softens compression continuously** (caps at 60% reduction, so an already-
  dense source keeps ≥40% of the preset's compression). The original draft floated
  an automatic hard-gate to Compressor `Off` at the extreme; that's a stronger,
  surprising mode change (flagged by the fact-check), so it is **deferred** — the
  user can still pick `Off` manually. Implemented as built in `guardrails.rs`.
- Sibilance (5–10 kHz) is a documented adjacent case but we have no exact 5–10 kHz
  band; v1 folds it into already-bright via presence/air. No new notch (that's
  Tier-2).
- `transient_density` is a documented modifier (more percussion → naturally more
  bass+air energy); v1 may use it to *soften* the bright/boomy triggers so
  percussive material isn't over-trimmed. Optional, behind the same caps.

## Transparency: "what was trimmed and why"

A small, honest readout per render/preview, e.g.:
`Air lift trimmed −1.8 dB (source presence+air 0.31 vs neutral ≤0.20).`
Shows per-axis trim amount and the source's distance from neutral. We show the
**trim deltas**, not a target curve to chase (showing a target curve is the
iZotope/Tier-2 paradigm). This aligns with the CLAUDE.md non-negotiable on clear
metering and review states.

The natural A/B for evaluating trim during the listening session: preset **as
designed** (strength 0) vs **after trim**, level-matched via the existing Volume
Match.

## Calibration plan

1. Ship with the provisional defaults above as named constants (one place).
2. Add a local calibration example that measures **our-band** neutral shares
   across the owner's reference masters (`tests for presets`) and the private
   fixtures, so defaults can be replaced with measured values. (Local-only; no
   private audio committed.)
3. Listening session: sweep Adapt Strength on already-bright / already-boomy /
   already-dense / already-wide sources per preset; confirm caps preserve preset
   identity; lock the numbers.

## Test plan

- `preset_byte_identity` SHA snapshots stay green (no profile → no trim).
- Pure unit tests for `SourceGuardrails::from_profile`: deadband, ramp, caps,
  floor, reduce-only, strength scaling, mono (no width trim).
- Behavioral fixture tests (synthetic sources): an already-bright source renders
  with **measurably less** air than the same preset on a neutral source; dense →
  less gain reduction; wide → narrower side; boomy → less low lift.
- Manual-override preservation: user `eq_*_db`, Manual compression, and explicit
  `advanced.width` are untouched by guardrails.
- Multi-axis composition: a source that is both bright and dense trims both,
  each within its own cap.
- Slow lane: extend the private-fixture matrix expectations
  (`AMS_RUN_REAL_FIXTURE`).

## Open questions (proceeding with these v1 defaults; change by ear)

These are owner taste calls. None blocks the build — I'm shipping sensible,
tunable defaults so the logic exists for the listening session.

1. **Default strength** → proceeding with **0.6 ("standard"), ON by default**.
2. **Per-axis caps** 50% / 60% / 70% → proceeding as listed; the most likely
   thing to retune per preset.
3. **Deadband widths & neutral windows** → proceeding as listed; the calibration
   example will propose measured replacements.
4. **Auto Compressor `Off` at extreme density** → **deferred**; v1 caps density
   reduction at 60% rather than flipping the mode. Revisit after listening if
   already-mastered material still feels over-compressed.
5. **Transparency "what was trimmed" readout** → **deferred to a follow-up**. v1
   ships the working guardrails + the Adapt Strength control; surfacing the
   per-axis trim deltas (so the user can see *why* a preset eased off) is the next
   increment. Needs the computed trims exposed to the UI.
6. **Multi-axis composition** → proceeding **per-axis independent** (each axis
   trims its own move within its own cap); revisit if it over-acts.

## Out of scope — Tier-2 (future)

Per-preset target tonal-balance curves; genre/style classification; reference-
track upload / matching; resonance detection + dynamic notching; per-band stereo
width matching; active sibilance notch; realtime per-frequency dynamic EQ;
building a private corpus to refine neutral references.

## Citations (real, credibility-tagged)

- Pestana, Ma, Reiss, Barbosa, Black — *Spectral Characteristics of Popular
  Commercial Recordings 1950–2010*, AES 2013 (academic).
- Elowsson & Friberg — *Predicting the perception of performed dynamics…* / LTAS
  slope study, AES 2017 (academic).
- iZotope Ozone Master Assistant & Tonal Balance Control documentation (official
  docs): crest-factor ideal 3–10; genre target curves; "don't aim for ruler-flat".
- LANDR — US Patent US9654869B2 (patent): feature set (loudness, loudness range,
  spectral/spatial balance/masking, centroid, spread, flux, crest factor);
  corrective target-curve approach.
- Sonible smart:EQ / smart:comp / smart:limit (official docs): single "Impact"
  control; median PSR for dynamics; safe-zone idiom.
- oeksound soothe2; Ozone Stabilizer Cut Mode (official docs): attenuate-existing-
  only precedent.
- Soundtheory Gullfoss (official docs / reviews): single Tame/Recover control;
  practical ceiling.
- EBU R128 / ITU-R BS.1770-4 (standards): K-weighted LUFS, LRA, silence gating.
- Sterne & Razlogova, *Social Media + Society* 2019 (peer-reviewed): ML is likely
  a small part of LANDR's otherwise rule-based pipeline — be honest, don't oversell
  "AI".

Overall research confidence: **medium** — architecture and principles are well
grounded; specific numbers need ear calibration. That is exactly the plan.
