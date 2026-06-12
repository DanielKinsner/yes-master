# Adaptive / Deep Analysis Adversarial Review - 2026-06-04

## Scope

- Repo: `.`
- Branch reviewed: `main` at `b5d0771`
- Adaptive/deep-analysis range reviewed: `2877c6d..HEAD`, where `2877c6d` is the adaptive guardrails merge point.
- Required docs read: `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md`, `docs/RELEASE_STABILIZATION.md`.
- Review posture: adversarial, read-only for code. This file is the only artifact created.

## Verification Performed

- `npm test` passed: 159 tests.
- `cargo test --lib --target-dir target\codex-rc` passed: 279 tests.
- `git diff --stat 2877c6d..HEAD` shows 41 changed files, 5934 insertions, 345 deletions.
- `git diff --check 2877c6d..HEAD` failed on trailing whitespace in `src/App.css:3490-3495`.
- Not run: full `npm run build`, `npm run build:windows`, full `cargo test`, private fixture lane, iPhone native bridge, slow real-fixture lane.

## Findings

### P1 - Deep Analysis is computed, but Phase B is not product-active yet

**Location:** `src-tauri/src/analysis.rs:110-127`, `src-tauri/src/confidence.rs:28-33`, `src-tauri/src/profile_store.rs:145-155`, `src-tauri/src/dsp.rs:782-795`

Desktop analysis now computes `DeepAnalysis` by default, but the confidence gate that would let that deeper read affect the DSP is compiled off:

- `analysis.rs` computes 31-band and per-window `DeepAnalysis` on the desktop path.
- `CONFIDENCE_GATING_ENABLED` is `false`.
- `apply_resolved_confidence` therefore resolves `source_confidence` to `None`.
- `dsp.rs` maps `None` to the default full-confidence path, which reproduces Tier-1 guardrails.

**Impact:** If we talk about the current app as if "deep adaptive analysis" is actively shaping masters, that is not true yet. The product is still using Tier-1 adaptive guardrails at the selected adaptive strength. The deeper read is staged data unless the owner-calibration gate is flipped.

**Recommendation:** Treat Phase B as inactive/calibration-gated in UI copy, docs, and handoffs until it is actually enabled. If by-ear owner calibration needs iteration, consider a dev-only runtime flag or hidden internal switch instead of a compile-time const, so the calibrated path can be exercised without source edits.

### P2 - The 31-band analysis is not wired into adaptive decisions

**Location:** `src-tauri/src/deep_analysis.rs:177-203`, `src-tauri/src/confidence.rs:110-147`, `src-tauri/src/guardrails.rs:118-129`

The new 31-band curve is stored in `DeepAnalysis`, and `harsh_share` / `sibilant_share` are derived from it. But the adaptive guardrails still make their actual trim decisions from:

- the Tier-1 `SourceProfile.spectral_6` rollup in `guardrails.rs`;
- per-window 3-band `high` / `low` shares in `confidence.rs`;
- crest and stereo correlation.

The 31-band harsh/sibilant detail does not currently gate or refine brightness, harshness, sibilance, air, or low-end decisions.

**Impact:** This does not yet deliver the product idea of "deeper bands from the get-go" in the way an everyday user or engineer would assume. It measures richer data, but the sound-shaping path is still coarse. It also means Deep Analysis may not prevent over-neutering caused by broad 6-band decisions.

**Recommendation:** Decide whether 31-band is Phase C readout-only or Phase B adaptation input. If it is meant to improve mastering decisions now, wire harsh/sibilant/tilt features into either `Confidence::from_deep` or a new guardrail input, then add tests proving that a harsh-only track, sibilant-only track, and airy-but-not-harsh track produce different adaptive behavior.

### P2 - Private evidence lanes will validate the wrong chain once confidence gating is enabled

**Location:** `src-tauri/src/fixture_matrix.rs:117-130`, `src-tauri/src/fixture_matrix.rs:202-205`, `src-tauri/src/reference_tuning.rs:199-205`, `src-tauri/src/reference_tuning.rs:299-311`

The private fixture matrix and reference tuning lanes say they exercise the "SAME adaptive chain the shipping app runs", but both skip deep analysis and inject only `source_profile`.

That is currently equivalent enough because confidence gating is off. It stops being equivalent the moment `CONFIDENCE_GATING_ENABLED` becomes true, because the app path will resolve `source_confidence` from cached `DeepAnalysis`, while the private lanes will still run full-confidence Tier-1 behavior.

**Impact:** These are the exact lanes that should protect already-mastered and reference-calibration behavior. If they stay as-is, they can bless an adaptive change that the real app does not run, or miss a confidence-gated regression that users will hear.

**Recommendation:** Before enabling Phase B, make the fixture matrix and reference tuning use the same resolver as preview/export, or explicitly analyze with `deep = true` and inject/resolve `source_confidence`. Add one regression that fails if a confidence-gated app render and the fixture/reference render disagree for the same source/settings.

### P2 - Side-heavy stereo can be misread as silent for confidence coverage

**Location:** `src-tauri/src/deep_analysis.rs:366-397`, `src-tauri/src/deep_analysis.rs:400-438`, `src-tauri/src/confidence.rs:153-170`

Window loudness is computed from a mono downmix: `0.5 * (L + R)`. Stereo width/correlation is computed separately from left/right and mid/side energy. For a side-heavy or anti-phase section, the mono downmix can approach silence while the stereo metrics clearly show very wide/phasey audio.

`axis_confidence` excludes windows whose `loudness_key` is non-finite. That means a phase-heavy section can contribute strong width/correlation measurements but be excluded from the width confidence coverage that would allow the guardrail to act.

**Impact:** When Phase B is enabled, the new confidence layer can under-trim widening on exactly the material most likely to need cautious width handling. This is a false-negative risk in the "do not overcook stereo field" area.

**Recommendation:** Use a stereo-aware loudness key for coverage, closer to channel-summed BS.1770 energy, or make width coverage use finite stereo energy/correlation instead of mono-downmix loudness. Add a regression for side-dominant stereo where Tier-1 sees width risk and Phase B does not collapse confidence to zero.

### P2 - Low/boom confidence uses inconsistent window filtering

**Location:** `src-tauri/src/deep_analysis.rs:98-116`, `src-tauri/src/confidence.rs:110-131`, `src-tauri/src/confidence.rs:153-170`

`axis_strata` filters out windows unless both the loudness key and the value are finite. `axis_confidence` coverage skips only non-finite loudness keys. The low/boom axis then computes consistency with `iqr(&lows)` over the raw low-share series instead of a low stratum filtered through the same loudness-key window set.

**Impact:** Low confidence can be calibrated from a different population than low coverage. Quiet/silent/tail windows may affect consistency differently than the coverage denominator, which makes the confidence number harder to reason about and easier to mis-tune.

**Recommendation:** Give low its own stored `AxisStrata` or compute low dispersion from the same finite-key/value window pairs that coverage uses. Add a test with loud boomy sections plus quiet/silent tails to prove the confidence result tracks the audible sections rather than analysis bookkeeping.

### P2 - Bright/low confidence still uses the old approximate 3-band helper

**Location:** `src-tauri/src/analysis.rs:275-293`, `src-tauri/src/deep_analysis.rs:326-345`, `src-tauri/src/deep_analysis.rs:170-174`

The temporal confidence path builds a mono slice per window and calls `compute_spectral_balance`, which assumes a 44.1 kHz reference and approximate first-order bands. The code comments acknowledge the sample-rate skew and that `_sample_rate` is unused.

**Impact:** Even after adding 31-band analysis, the confidence triggers for bright/low traits are still driven by a coarse, sample-rate-agnostic 3-band read. At higher sample rates or edge-case material, this can undermine the promise of a more precise upfront analysis.

**Recommendation:** Either make the window-level tonal read sample-rate-aware, or derive temporal bright/low/harsh/sibilant features from a windowed FFT path that shares band definitions with the 31-band curve. At minimum, add 44.1/48/96 kHz regression fixtures that prove the same musical content lands in comparable confidence bands.

### P3 - Export/readout traceability will be insufficient when confidence gating is on

**Location:** `src-tauri/src/engine.rs:788-811`, `src/App.tsx:3101-3117`, `src/bindings.ts:351-354`

Rendered measurements record `effective_adaptive_strength` and `source_profile_digest`, but not whether confidence gating was active or what the per-axis confidence values were.

**Impact:** Today this is mostly harmless because the gate is off. Once Phase B turns on, an export receipt that says "Adaptive 50%" will not reveal whether brightness, low, density, or width were effectively reduced to 5%, 50%, or 100% of Tier-1 behavior. That weakens debugging, user trust, and owner A/B calibration.

**Recommendation:** When the gate is enabled, record a compact confidence digest and expose it in advanced receipt/readout surfaces. This can stay hidden from casual users, but engineers and calibration sessions need to know what actually happened.

### P3 - Backend adaptive/deep profile cache is not explicitly evicted when a track is removed

**Location:** `src-tauri/src/profile_store.rs:28-98`, `src/hooks/useTrackMaster.ts:978-1017`

The frontend `removeTrack` clears track-local React maps, but there is no backend command to remove the corresponding `SourceProfileStore` entries. The store only clears on re-analysis soft clears or hard-failure pruning.

**Impact:** In long sessions with many imported/removed tracks, backend `SourceProfile` and `DeepAnalysis` entries can accumulate. This does not look like an immediate audio-corruption bug because track IDs are unique in normal usage, but DeepAnalysis can hold per-window series, so it is avoidable memory retention.

**Recommendation:** Add a backend `evict_source_profile(track_id)` or `remove_track_state(track_id)` command and call it from `removeTrack`. A bounded LRU or project-open clear would also work if explicit eviction is not desired.

### P3 - Public comments/contracts still describe older wiring

**Location:** `src-tauri/src/types.rs:124-126`, `src/bindings.ts:67-70`, `src/bindings.ts:264-277`, `src/App.tsx:2174`, `src-tauri/src/fixture_matrix.rs:126-129`, `src-tauri/src/reference_tuning.rs:308-310`

Several comments still imply frontend/TS injection or older defaults:

- `types.rs` says `SourceProfile` is injected by TS.
- `bindings.ts` still describes injected source snapshots in a way that reads frontend-owned, while the backend now owns profile resolution.
- `App.tsx` says adaptive reset returns to `0.6`, but the actual default is `0.5`.
- Fixture/reference comments claim the same adaptive chain while skipping DeepAnalysis.

**Impact:** These are not runtime bugs, but this repo is already using prose contracts as implementation guardrails. Stale comments increase the chance that Claude/Codex, future maintainers, or release notes describe the adaptive path incorrectly.

**Recommendation:** Update the comments after the intended Phase B decision is made. Keep the docs explicit about backend-owned source profile resolution, confidence gate state, and whether 31-band analysis is readout-only or behavior-affecting.

### P3 - Adaptive range fails whitespace check

**Location:** `src/App.css:3490-3495`

`git diff --check 2877c6d..HEAD` fails because the adaptive range introduced trailing whitespace around `.compressor-density-field`.

**Impact:** This is not a product bug, but it is an objective release-gate hygiene failure if `diff --check` or a formatting gate is used.

**Recommendation:** Remove the trailing whitespace and keep `git diff --check` in the quick pre-merge lane.

## Product Decision Questions

1. Should "Deep Analysis" be described as active today, or as staged/calibration-gated until `CONFIDENCE_GATING_ENABLED` is flipped?
2. Should 31-band harsh/sibilant/tilt features affect the master now, or remain internal readout data for a later phase?
3. Should the private fixture/reference lanes be considered valid Phase B evidence before they resolve `source_confidence` the same way preview/export does?
4. Should owner ear-calibration use a runtime/dev flag instead of source-code edits, so the gated path can be A/B tested repeatedly and safely?
