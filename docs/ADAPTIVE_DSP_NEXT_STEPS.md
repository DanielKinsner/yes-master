# Adaptive DSP — Next Steps & Backlog

Single entry point for what's left on the adaptive/smart DSP. **Status:** merged
to `main` on 2026-06-03 (merge `2877c6d`), but the owner's by-ear listening signoff
/ calibration is **still pending** — the guardrail numbers are provisional
placeholders, not yet ear-validated (merged ≠ validated). Detailed options +
rationale for each item live in the finish plan:
`docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`.

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
- **Owner follow-up (2026-06-05):** confidence remains backend/devtools-only;
  31-band per-window harsh/sibilant/air/tilt detail now feeds Phase-B confidence
  instead of relying on the coarse 3-band tonal proxy (`aeefee6`). Desktop and
  iPhone now show truthful staged analysis/render progress copy without claiming
  unbuilt genre/style detection (`2683f42`, `611f5ff`). `npm run verify:fast`
  is available for the documented frontend + desktop Rust + iPhone Rust bridge
  lane (`a386d81`).
- **Backend-owned `source_profile` (B2) — DONE (2026-06-02):** the backend is now
  the SINGLE derivation point. `analyze_tracks` derives + caches the profile in a
  `SourceProfileStore` (`src-tauri/src/profile_store.rs`); every Track-Master chain
  entry (live `play_master`/`update_chain`, `render_track_preview`/`master`,
  `guardrail_readout`) resolves from it via `apply_resolved_profile` (FE-supplied
  profile = override; album → None). The audio thread holds the store and the live
  settings-only `update_chain` path resolves by the loaded track (read fresh, so a
  late analysis is picked up). The TS profile mappers (`sourceProfileFromAnalysis`
  / `injectSourceProfile`) were removed — dual-mapper drift dissolved. `play_master`
  + `guardrail_readout` gained an `album` arg; album audition caches `live_album`.
  Byte-identity untouched (fill is at the command layer). Known minor edge: toggling
  album↔track mid-Mastered-playback keeps the prior `live_album` until the next
  play_master (self-heals on the next Mastered click).

## 🔒 Owner decisions (locked — don't re-ask)

- Neutral-master deadband → **quick bump shipped**; tilt-vs-reference is the
  principled follow-up (below).
- "What was trimmed" readout → **built**.
- Backend-owned-profile refactor → **DONE** (B2, see Done section above).
- Album Master → **unadapted + labeled** (done).
- Confidence detail → **backend/devtools only**. Do not add an everyday-user
  confidence UI surface unless the owner explicitly reopens it.
- 31-band detail → **adaptation input now — implemented 2026-06-05.** The
  current feed is per-window harsh/sibilant/air/tilt into Phase-B confidence.
  Confidence-gated behavior remains provisional until listening.
- Premium progress language → **approved and implemented as truthful staged UI**
  for desktop + iPhone analysis/rendering. It is timed UI state, not backend
  stage telemetry yet; do not claim unbuilt steps such as genre detection.

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
- **Export-receipt traceability (B5) — ✅ DONE (2026-06-03).**
  `RenderedMeasurements` + `ExportReport` now carry `effective_adaptive_strength`
  + a one-line `source_profile_digest`; the receipt shows "Adaptive NN% · digest"
  (or "Adaptive: off"). Backend-authoritative (post-B2 the FE no longer holds the
  profile); recorded only when a profile was present AND strength > 0. [review B5]

## 🏛️ Architecture

- **Backend-owned source profile (B2) — ✅ DONE (2026-06-02).** See the Done
  section above. Derivation moved server-side into a `SourceProfileStore`;
  FE-supplied profile is an override; the audio thread resolves the live
  settings-only `update_chain` path by the loaded track; album stays flat; the
  `guardrail_readout` contract + TS fetch were re-touched (NF-2); the TS twin
  mapper was deleted. [002 §5; review B2/NF-2]

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
- **31-band rollup / target-curve future work** — the 31-band confidence feed is
  done, but the core 6-band UI rollup, measured-neutral target work, and any
  corrective target-curve behavior remain future milestones. Confidence remains
  backend-only and gate-controlled; this is not a preset retune or a release-
  stable listening signoff.
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

## 🎧 Calibration gate (owner, before release-stable)

> The adaptive guardrails merged to `main` on 2026-06-03; this gate is now the
> "before treating adaptive DSP as release-stable" listening pass, not a
> pre-merge blocker. (Merged ≠ ear-validated.)

- Listening A/B: Adapt `0` vs `60%` on already-mastered / bright / dense / wide
  **and** neutral sources; confirm neutral does ~nothing, then lock the constants
  in `guardrails.rs`.
- Slow fixture lane (`AMS_RUN_REAL_FIXTURE`) — now exercises the adaptive chain.
- **2026-06-03 UI session:** Reset now restores Adapt Strength to 0.6; the
  Adapt Strength block now shows a per-axis readout with source-share-vs-deadband
  so a `-0%` reads as "source in range." Dan's report that the EQ/Width trims
  "don't move with the slider" was root-caused to the **deadbands** (the slider
  wiring is correct; only axes the source crosses move) — a TASTE decision, fully
  specified (exact constants, The Keeper diagnosis, Loud `low_shelf<floor` quirk,
  candidate ranges, validation process) in
  `docs/HANDOFF_2026-06-03_ADAPTIVE_UI_FIXES_AND_DEADBAND_CALIBRATION.md`.
  **Do not change deadbands without a listening pass.**

## Pointers

- Finish plan (detail + options): `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`
- Spec: `docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md`
- Handoff (start here to code): `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2.md`
- Reviews: `docs/reviews/2026-06-02-adaptive-dsp-tier1-review.md`, `docs/reviews/2026-06-03-adaptive-dsp-desktop-review.md` (+ an untracked `…-GLOBAL-review.md`, unread)
