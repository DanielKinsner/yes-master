# Adaptive DSP — Next Steps & Backlog

Single entry point for what's left on the adaptive/smart DSP. **Status as of
2026-06-02** (branch `feat/adaptive-dsp-guardrails`, not merged — awaiting the
owner's listening signoff). Detailed options + rationale for each item live in the
finish plan: `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`.

## ✅ Done — do not redo

- Tier-1 defensive engine: 4 guardrails (bright / boomy / dense / wide), single
  Adapt Strength dial, per-axis caps + character floor, byte-identical when off.
- Adapt Strength UI + the per-axis "what was trimmed" readout.
- Post-review fixes: `LRA=0.0` density misfire, preview + slow-lane wiring, bright
  deadband `0.20 → 0.30` (neutral-master misfire), doc drift + default dedup.
- Album labeled Track-Master-only (intentionally unadapted).
- **Post-TRUE-review (2026-06-03):** album non-adaptive *end-to-end* (audition
  flat + backend strips any profile + Adapt control disabled, with tests); durable
  on/off default (B4); readout reads the *realized* EQ trim post-floor (B8); LU→dB
  LRA aliasing removed (B11); stale-profile clear on dispatch (B10); boundary tests
  refreshed (B12). Review trail committed under `docs/reviews/`.

## 🔒 Owner decisions (locked — don't re-ask)

- Neutral-master deadband → **quick bump shipped**; tilt-vs-reference is the
  principled follow-up (below).
- "What was trimmed" readout → **built**.
- Backend-owned-profile refactor → **deferred** (below).
- Album Master → **unadapted + labeled** (done).

## 🎚️ Tier-1 finish (remaining — calibrate by ear)

Tune in one place: `src-tauri/src/guardrails.rs`.

- **Tilt-vs-reference brightness metric** — replace the absolute presence+air
  deadband with a spectral-slope-vs-pink comparison so a flat spectrum reads as
  zero excess by construction. The principled fix behind the deadband bump. [002 §1]
- **Density cap semantics** — reshape the density trim against the engagement
  curve so "keep ≥40%" is honest in dB gain-reduction; revisit the default-vs-cap
  interaction (at 0.6 the dial is a no-op above default on dense material). [002 §2]
- **Whole-track Welch spectral window** — the 6-band tonal read is the first
  ~5.5 s; average across the track. Note: shifts role/character/album inference too. [002 §3]
- **`stereo_width` as a width co-trigger** — computed + carried, currently unused. [002 §4]
- **Per-axis EQ floors / `LOW_DEADBAND` ear-calibration** — bass-forward genres. [002 §6]
- **`LRA → Option<f32>`** cleanup (optional; the minimal guard already shipped). [002 §9]
- **Export-receipt traceability (B5)** — add `effective_adaptive_strength` (+ a
  one-line source-profile digest) to `ExportReport` and the receipt, so a delivered
  master records what adaptation produced it. P1; quick. [review B5]

## 🏛️ Architecture (deferred — owner's call to schedule)

- **Backend-owned source profile (B2)** — derive `source_profile` server-side from
  the track `AnalysisResult`; treat any FE-supplied profile as an override; cache
  it in audio-thread state so the settings-only `update_chain` live path gets it
  too. Closes the preview / slow-lane / dual-mapper class by construction (NOT
  album — album is intentionally flat now). **The TRUE master review escalates this
  to its recommended P0 / root-cause fix** — the FE forgot injection at 3 sites
  already; one mapper (Rust `from_analysis`) is test/tuning-only while production
  uses the TS twin. Coupling (NF-2): also re-touch the `guardrail_readout` input
  contract + the TS readout fetch (`useTrackMaster.ts`) when derivation moves
  server-side. [002 §5; review B2/NF-2]

## 🚀 Tier-2 — the "smart" tier (next milestone, after v1 is locked by ear)

- **Measured neutral** built from the owner's own reference masters in the slow
  lane, ideally **per-preset** (Spatial's neutral ≠ Loud's) — replaces the
  textbook pink number with measured data.
- **PSR / crest closed-loop dynamics** — the only honest "won't crush your
  transients further" defense (compare predicted post-chain PSR vs source).
- **Corrective target-curve** (actively push toward a target, LANDR/iZotope style)
  + genre/style classification + reference-track upload / matching.
- **Resonance / sibilance detection** (soothe / Gullfoss style); per-band stereo
  width matching (Ozone Width Match style).
- **Total-loudness-loss budget (B3)** — when a delivery target is active (incl.
  Custom + an explicit `lufs_offset_db`), the post-chain LUFS landing recoups
  loudness the trims removed, so a multi-axis source can exceed a single per-axis
  cap in final level. Share a loudness-loss budget across axes, or attenuate
  combined strength when predicted pre-limiter loss is high. Today mitigated only
  by the readout's honest "pre-landing" label. [review B3]
- Higher-resolution analysis generally (finer than 6 bands; time-varying) — the
  analysis resolution is the ceiling on how "smart" this can get. [002 §8 + spec
  `2026-06-02-001` "Out of scope"]

## 🧩 Parallel initiative — the OTHER half of the original idea

- **Simple Mode view** — the stripped-down desktop UI (fewer controls, friendly
  preset names, one Loudness dial, non-lossy toggle to Advanced) that *rides on
  this adaptive engine*. The original ask was an either/or — "adaptive/smart DSP
  **or** a simple mode" — and we deliberately built the adaptive **engine first**
  so it could be the trustworthy substance behind Simple Mode. That makes Simple
  Mode the natural next product step now that the engine exists.
  Spec: `docs/SIMPLE_ADVANCED_MODE_NOTE_2026-05-29.md`.

## 🎧 Calibration gate (owner, before merge)

- Listening A/B: Adapt `0` vs `60%` on already-mastered / bright / dense / wide
  **and** neutral sources; confirm neutral does ~nothing, then lock the constants
  in `guardrails.rs`.
- Slow fixture lane (`AMS_RUN_REAL_FIXTURE`) — now exercises the adaptive chain.

## Pointers

- Finish plan (detail + options): `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`
- Spec: `docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md`
- Handoff: `docs/HANDOFF_2026-06-02_ADAPTIVE_DSP_TIER1.md`
- Reviews: `docs/reviews/2026-06-02-adaptive-dsp-tier1-review.md`, `docs/reviews/2026-06-03-adaptive-dsp-desktop-review.md` (+ an untracked `…-GLOBAL-review.md`, unread)
