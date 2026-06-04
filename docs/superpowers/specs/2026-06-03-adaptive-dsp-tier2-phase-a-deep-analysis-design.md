# Adaptive DSP Tier-2 — Phase A: Deep Analysis (design) — v2.1

Date: 2026-06-03
Status: design v2.1 — APPROVED to implement (tests-first). v2 revised after a
grounded 5-lens adversarial review; v2.1 applies that reviewer's
implementation-readiness fixes. Branch context:
`fix/adversarial-type-review-2026-06-03`.

## Changelog v1 → v2 (what the review changed)

- **Phase A is now purely additive / behavior-neutral.** The existing whole-track
  spectral pass *also* buckets into 31 bands (no extra FFT); the 6-band read and
  `SourceProfile` are **untouched**. This dissolves the v1 §B cluster (6-band
  tolerance, the "analogous to F1" claim, the wrong consumer list, the
  byte-identity miscite) — Phase A no longer changes any existing value. The
  6-band→rollup unification is **deferred to Phase B**.
- **Dual-resolution scan** (was a single 16 384 window, which was 16× coarser in
  the low end than what ships today).
- **Retain a compact per-window time series** (was: only 3 rolled-up strata +
  scalar dispersion). Resolves PSR storage, temporal structure, and re-derivable
  cuts in one move.
- **Pinned the fuzzy terms** the v1 §11 banner wrongly called "specified": the
  loudness key, "body," the dispersion estimator, determinism, and the cache rule.
- **Owner calls** remain, marked `DECIDE:` — two material (mobile gate,
  persistence) plus one minor (loudness-key definition).

## Changelog v2 → v2.1 (implementation-readiness fixes)

- **6-band stays byte-exact, test-enforced.** The 6-band keeps its own direct
  accumulation; the 31-band is a *parallel* accumulation in the same FFT loop and
  is never the 6-band's source in Phase A. A byte-exact golden test on the six
  floats is added *before* the refactor (tests-first).
- **Data flow pinned:** `DeepAnalysis` rides as `#[serde(skip)]
  Option<Arc<DeepAnalysis>>` on `AnalysisResult` — chosen over tuple-threading so
  `analyze_tracks_core`'s signature (called by the iPhone bridge + pinned by
  `contracts.rs:136`) stays stable and the wire contract stays genuinely unchanged.
- **Per-window peak = sample peak** (cheap); whole-track true peak kept for the
  ceiling read; per-window true-peak deferred to B with its 4× cost budgeted.
- **"momentary-PSR"** qualified (uses the momentary loudness key, not 3 s).
- **Per-window broad tonal pinned** to the existing 3-band (low/mid/high).
- Non-finite/silent window exclusion applies to **all** strata (not just the loud
  cut); two-map clear uses a **fixed lock order, never both locks held**;
  momentary-key **edge handling** at track start/end defined.
- §13 reworded: Phase A retains the *ordered series*; the temporal *measure* is
  Phase B.

## 1. Context

Tier-1 adaptive guardrails are merged and conservative-by-design, but the engine
acts on a thin read → it over-acts on normal material (boxes music in) and
homogenizes presets (a novice may not hear the difference between presets on a
given source). The fix is upstream: **see more before acting.**

Three-phase program:

- **Phase A — Deep Analysis (this spec):** measure the track richly. Factual only;
  changes nothing audible.
- **Phase B — Confidence-gated adaptation (future):** consume A; trims scale by
  confidence/coverage; PSR/crest transient protection; preset-identity protection
  (anti-convergence); holistic already-mastered stand-down; the 6-band→31-band
  rollup unification.
- **Phase C — Premium surface (future):** honest staged loading (stages = real
  phases) + a plain "what we found" summary.

Load-bearing principle: **A measures; B decides.** No thresholds or correction
logic in Phase A.

## 2. Goal & non-goals

**Goal:** produce `DeepAnalysis` — a rich, whole-track, loudness-aware, *temporally
ordered* measurement that a later Phase B can use for confident, surgical,
non-homogenizing decisions.

**Phase A explicitly does NOT:**
- Apply any threshold, deadband, confidence score, or correction (all Phase B).
- **Change any existing value or behavior.** It is purely additive: the 6-band
  read, `SourceProfile`, role/character/energy/album scoring all run exactly as
  today. The only new thing is a backend-internal `DeepAnalysis` cache entry.
- Add named/semantic sections (verse/chorus). Loudness strata + a per-window
  series only; named sections can be layered later for the Phase C summary.
- Touch measured-neutral / tilt-vs-reference (later Tier-2; needs calibration).

## 3. Key principles

1. **Measure fine, *retain* fine, expose coarse.** Internal detail is computed
   AND kept (per-window series, backend-only); the UI and TS↔Rust contract stay
   simple (6 bands + a couple of derived flags).
2. **A measures, B decides.** Phase A outputs factual values + a threshold-free
   per-window series; Phase B owns all thresholds/confidence.
3. **`DeepAnalysis` is an optional ceiling over the `SourceProfile` floor.** The
   chain never *requires* `DeepAnalysis`; it degrades to the compact
   `SourceProfile` (existing trims) when `DeepAnalysis` is absent.
4. **One-time per session, cached.** Phase A runs once per analyze; live audition
   reads the cache (zero added real-time cost). The cache is in-memory, so it is
   recomputed on app reopen (see §6 / `DECIDE: persistence`).
5. **Byte-identity preserved trivially.** Phase A touches no chain code, so the
   non-adaptive path is bit-identical by construction (the `preset_byte_identity`
   snapshots stay green because they never read spectral values — they are the
   "we didn't touch the chain" invariant, NOT coverage of any spectral change).

## 4. What Phase A computes — dual-resolution scan

The decode already happens. Phase A runs two passes over the decoded samples.

### 4.1 Long-window tonal pass (frequency resolution)

This is the existing whole-track spectral pass (post-F1: a Welch-averaged window
up to 2¹⁸ samples, ≈0.18 Hz/bin) — **reused, not duplicated.** The **6-band read
keeps its own direct accumulation, unchanged.** Phase A adds a **parallel**
accumulation in the *same* FFT loop that buckets the magnitudes into **30 nominal
one-third-octave bands** (25 Hz–20 kHz, IEC 61260) for `DeepAnalysis`, held in a
fixed `[f32; 31]` whose index 30 is a `0.0` padding slot. The two accumulations
are independent: in Phase A the 31-band is **never** the 6-band's source (no
31→6 rollup — that's deferred to Phase B), so the 6-band output is byte-exact vs.
today (enforced by a golden test, §12). No extra FFT.

Rationale: low-end ⅓-octave needs fine frequency resolution (the bottom bands are
only a few Hz wide); the long window already provides it and feeds the future
tilt-vs-reference / measured-neutral work directly.

### 4.2 Short-window time pass (time resolution)

A separate sliding short window (**16 384 samples ≈ 0.34 s @ 48 k, 50 % hop,
Hann**) over the whole track, producing a **per-window series** of the
*time-varying* axes:
- **Per-window loudness key** — see §5.2 (used for stratification).
- **Per-window peak** — **sample peak** of the mono downmix `0.5·(L+R)` (cheap; for
  momentary-PSR + crest, §5.3; same downmix as the loudness key, so PSR is self-consistent
  — understates hard-panned material vs a per-channel max).
  Whole-track *true* peak stays the ceiling read; per-window true peak is deferred
  to Phase B (and would add a 4× oversampled pass — budgeted in §9, not run here).
- **Per-window crest** — peak/RMS over the window (transient richness).
- **Per-window stereo** — width (side/mid energy ratio) + L/R correlation. The
  spectral pass is mono = (L+R)/2; the stereo axis uses L/R (width/correlation are
  degenerate on a collapsed mono signal).
- **Per-window broad tonal** — the existing **3-band (low/mid/high)**
  `compute_spectral_balance` read per window (reused; cheap), for temporal
  brightness clumping. The fine ⅓-octave curve comes only from the long pass
  (§4.1) — the short window would be 16× coarser in the low end than what ships.

The short window is deliberately not used for the ⅓-octave curve (it would be 16×
coarser in the low end than what ships).

### 4.3 Retained output: per-window series + derived aggregates

`DeepAnalysis` keeps **the compact per-window series itself** (tens of KB per
track), plus derived aggregates for convenience:
- **Loudness-stratified aggregates** per axis: whole-track, **loudest 10–20 %**,
  **body** (§5.1).
- **Dispersion** per axis (§5.4).

Retaining the ordered series (not just set-based aggregates) is what lets Phase B
later distinguish *sustained* from *scattered* (temporal structure — see §13),
recompute strata/cuts without rescanning, and derive PSR — for ~tens of KB.

## 5. Definitions (pinned — these were the v1 "fuzzy-but-labeled-settled" items)

### 5.1 Strata
- **Whole-track:** all windows.
- **Loudest:** the top **15 %** of windows by the §5.2 loudness key (tunable
  constant `LOUD_STRATUM_FRACTION`).
- **Body:** the central **25th–75th percentile** band of windows by loudness key
  (tunable `BODY_PCTL_LO=0.25`, `BODY_PCTL_HI=0.75`). "Median/body" in v1 was
  under-defined; this is the decided meaning.
- **Exclusion (all strata):** silent / non-finite windows (loudness key → −inf)
  are dropped **before** computing *any* stratum or percentile — not just the loud
  cut. Whole-track, loud, and body are all over the finite-window set.

### 5.2 Loudness key (renamed; v1 "per-window short-term LUFS" was a misnomer)
A 0.34 s window cannot carry a BS.1770 *short-term* (3 s) measurement. The
per-window loudness key is **momentary-style K-weighted loudness** integrated over
a **400 ms** centered span per window (BS.1770 momentary), in LUFS-like dB.
- 400 ms (not the 0.34 s window) so a single transient can't crown a window
  "loudest" (v1 risk).
- Distinct from `AnalysisResult.lufs_short_term_max_3s`, which remains the true
  3 s Mode::S whole-track measure (`analysis.rs:97-98`). `DECIDE (minor):` use
  momentary-per-window (default) vs sampling the real 3 s short-term per window.
- **Edge handling:** the 400 ms span is wider than the 341 ms window, so at the
  track's first/last windows it would run past the buffer. Clamp the integration
  to the available samples (shrink, don't zero-pad — zero-padding would falsely
  lower edge loudness); a window with < ~half the span available is treated as
  non-finite and excluded (§5.1).

### 5.3 momentary-PSR (so Phase B's transient defense is reconstructible — v1 stored only crest)
Store **per-window sample peak AND per-window loudness key** so
**momentary-PSR** = `sample_peak − loudness_key` is derivable per window and per
stratum without re-measuring. It is *momentary*-PSR (keyed on the §5.2 momentary
loudness, not 3 s short-term) — tied to the §5.2 `DECIDE`; if that flips to true
3 s short-term, this becomes short-term-PSR. Per-window crest (peak/RMS) is also
kept (different metric). Whole-track *true*-peak PSR for the ceiling read stays
on the existing whole-track true peak; per-window true-peak PSR is Phase B (§9).

### 5.4 Dispersion estimator
**IQR (inter-quartile range)** of the per-window values, per axis (robust, defined
units). For the bounded correlation axis, use IQR of the **Fisher-z-transformed**
correlation (ordinary variance of a bounded [−1,1] stat is meaningless). One
estimator per axis, fixed.

## 6. Data model — "alongside" (additive)

- **`SourceProfile` is unchanged** — same shape, same derivation (`from_analysis`),
  still the compact `Copy`, `bindings.ts`-mirrored artifact the chain consumes.
  `resolve_effective_profile` / `apply_resolved_profile` / FE-override precedence
  (pinned by `contracts.rs:136`) are untouched.
- **`DeepAnalysis`** is a **new backend-internal struct** (Rust only; never
  serialized to TS). It holds the per-window series + strata + dispersion + the
  31-band curve. Because it carries a per-window `Vec`, it is **not `Copy`** →
  store it behind `Arc<DeepAnalysis>`. Derive only `Debug` (Arc needs it); no
  `Serialize`/`PartialEq` needed.
- **Data flow (decided):** `analyze_one` attaches the computed `DeepAnalysis` to
  `AnalysisResult` via a new `#[serde(skip)] pub deep_analysis:
  Option<Arc<DeepAnalysis>>` field (defaults to `None` on deserialize).
  `populate_profile_store` moves it into the store. Chosen over threading a
  `(AnalysisResult, DeepAnalysis)` tuple specifically so `analyze_tracks_core`'s
  signature stays stable — the iPhone bridge calls it and `contracts.rs:136` pins
  it. `serde(skip)` means the wire contract is **genuinely unchanged** (no
  softening of "contract unchanged" needed). Cost: the ~10 `AnalysisResult` test
  literals gain `deep_analysis: None`; add an `AnalysisResult` test constructor /
  `..` helper to contain future field-addition churn (the exhaustive-literal
  lesson from the iPhone-bridge fix).
- **Store:** add a second map `Mutex<HashMap<TrackId, Arc<DeepAnalysis>>>` beside
  the existing `SourceProfileStore` map. `get()` clones the `Arc` (cheap), not the
  data (`profile_store.rs:59`'s `.copied()` stays valid for `SourceProfile`).
- **Invalidation must clear both maps in lockstep, on BOTH existing clear paths:**
  (a) `prune_failed_profiles` (hard-fail, F3), and (b) the soft `set(id, None)` in
  `populate_profile_store` (too-short/silent). v1 mentioned only (a).
- **Lock discipline:** use a **fixed lock order** (SourceProfile map, then
  DeepAnalysis map) and **never hold both locks at once** — take/drop one, then
  the other. The audio thread only ever locks the SourceProfile map, so that is
  the entire cross-thread surface; keeping the DeepAnalysis map off the audio
  thread avoids any lock-ordering hazard with it.
- **Contract unchanged.** No 31-band arrays cross the wire. Phase C later adds a
  small curated `AnalysisSummary` — not the raw series.
- **Lifetime:** in-memory only (`#[derive(Default)] Mutex<HashMap>`), so the cache
  is **per session** — reopening a 12-track album re-scans all 12 before audition
  is hot. `DECIDE: persistence` — persist `DeepAnalysis` to disk (faster reopen,
  needs serde + a content/version key + invalidation) vs accept per-session
  recompute for v1.

## 7. Integration with existing analysis (additive — nothing replaced in Phase A)

- The long pass (§4.1) is the existing spectral computation. The 6-band keeps its
  **own direct accumulation**; the 31-band is a **parallel** accumulation in the
  same loop and is **never the 6-band's source in Phase A**. So
  **`compute_spectral_balance_6band` and its 6-band output are byte-exact vs.
  today** (golden test, §12), and all current consumers are unaffected in Phase A:
  - `compute_energy_density_score` (`analysis.rs:99`) — reads the 6-band
    presence+air.
  - album heavy/acoustic/transition classification (`album.rs:127,143`) — threshold
    logic that can flip a track's arc role.
  - `reference_tuning.rs`.
  - (NOTE for the deferred Phase-B rollup: role/character inference does **NOT**
    read the 6-band — `infer_character` (`analysis.rs:152`) uses the **3-band**
    `compute_spectral_balance`. v1 mis-stated this.)
- Both callers of `compute_spectral_balance_6band` — `analysis.rs:91` and
  `album_render.rs:307` (album export energy_density) — keep using it as-is.
- **Short clips:** the short pass (§4.2) needs ≥ one 16 384-frame window; below
  that, `DeepAnalysis` is simply **absent**. This does **not** disable adaptation:
  `SourceProfile` still derives from the existing 6-band (whose floor is ~1024
  frames), so **Tier-1 trims still fire for clips between ~1024 and ~16 384
  frames.** Fully inert only below the 6-band's own ~1024-frame floor — exactly as
  today. (v1 wrongly said adaptation "goes inert"; that contradicted §3.3.)

## 8. Cross-platform / mobile / budget

- Shared Rust (`yes_master_lib`) → one implementation, no desktop/mobile logic
  drift.
- **`DECIDE: mobile`** — the deep scan runs inside `analyze_one`, which the iPhone
  bridge already calls via `analyze_tracks_core` (`apps/iphone-native/rust/src/lib.rs`).
  Mobile is currently fully non-adaptive (`source_profile: None`), so today the
  deep scan would run on-device **and be discarded.** Decide:
  - **(a) Gate it off the mobile FFI path** (run only the cheap legacy 6-band on
    mobile) — recommended until a lite-adaptive phone path is actually built
    (which needs new FFI/Swift wiring, out of scope here); or
  - **(b) Accept the cost** now to keep one path.
- **Budget cap (concrete, not "boundable"):** hard-cap the short-pass window count
  at `MAX_SCAN_WINDOWS` (default sized to ~12 min @ 48 k / 0.17 s hop ≈ 4200
  windows); beyond that, stride the hop so the cap holds. The long pass already
  self-bounds (single Welch). This makes the bound falsifiable.
- Validate the real number: a benchmark in the iPhone test suite analyzes a known
  fixture and prints elapsed ms (so the decision rests on data, not the estimate).

## 9. Performance

- **Live audition: zero added cost** (reads cache; adaptation is ~free per sample —
  only chain coefficients change).
- **One-time per analyze (cached):** desktop ~0.3–1 s for a typical 3–4 min track
  (the long pass ≈ today's cost; the short pass adds windowed FFT + per-window
  loudness/crest/stereo — budget **all** of these, not just the FFT). Mobile
  estimated ~1–2.5 s — to be validated (§8). Bounded by `MAX_SCAN_WINDOWS`.
- **Deferred cost (not in Phase A):** per-window *true* peak would need a 4×
  oversampled pass per window — explicitly **not** run here (Phase A uses sample
  peak); if Phase B wants it, add that cost to this budget then.

## 10. Components / files

- `src-tauri/src/analysis.rs` — add the 31-band **parallel** accumulation to the
  long pass (the 6-band's own direct accumulation is **untouched**; the 31-band is
  never its source in Phase A); add the short-window time pass; assemble
  `DeepAnalysis`. `compute_spectral_balance_6band` byte-exact (golden test, §12).
- `src-tauri/src/deep_analysis.rs` (new) — the `DeepAnalysis` struct + the
  per-window series types (backend-internal; not in `bindings.ts`). Derives `Debug`.
- `src-tauri/src/types.rs` — add `#[serde(skip)] pub deep_analysis:
  Option<Arc<DeepAnalysis>>` to `AnalysisResult` (defaults `None`; off the wire);
  add a test constructor/helper so the ~10 fixture literals don't each need manual
  field additions now and on future fields.
- `src-tauri/src/profile_store.rs` — add the `Arc<DeepAnalysis>` map; extend BOTH
  clear paths (`prune_failed_profiles` + the soft `set(_, None)`); fixed lock order,
  never both locks (§6).
- `src-tauri/src/engine.rs` — `analyze_tracks` / `populate_profile_store` move the
  `DeepAnalysis` off the `AnalysisResult` into the store, alongside `SourceProfile`.
- No `src/` / `bindings.ts` changes (contract unchanged — the new field is
  `serde(skip)`).

## 11. Implementation parameters (now actually specified)

- **Long pass:** existing Welch window (≤2¹⁸); add 31-band ⅓-octave bucketing.
- **Short pass:** 16 384 samples / 0.34 s, 50 % hop, Hann, mono=(L+R)/2 for tonal,
  L/R retained for stereo.
- **Loudness key:** 400 ms momentary K-weighted (§5.2).
- **Strata:** loud = top 15 %; body = 25th–75th pctl (§5.1) — tunable constants.
- **Dispersion:** IQR; Fisher-z for correlation (§5.4).
- **Harsh/sibilant:** 2–5 kHz / 5–9 kHz, tunable constants (single source of truth).
- **Determinism (§5/§13):** f64 accumulators in **fixed window order**; loud-stratum
  sort key = `(loudness, window_index)` (stable tiebreak); exclude/floor non-finite
  (silent → −inf) windows **before** the cut; default scalar `rustfft`; no parallel
  float reduction. Determinism is **per-platform** (Win/macOS/ARM floats differ —
  preset SHAs are already per-OS, `dsp.rs`), so NO cross-platform golden test.
- **Cache rule (decided, not "confirm…"):** recompute `DeepAnalysis` whenever
  `SourceProfile` is recomputed, under the same trigger; clear under the same two
  paths (§6).

## 12. Testing

- **Golden 6-band test (tests-first, write BEFORE the refactor):** a byte-exact
  assertion on `compute_spectral_balance_6band`'s six returned floats for a fixed
  fixture. It must pass on today's code and stay equal after the 31-band parallel
  accumulation lands — this is what *enforces* the "additive / changes nothing"
  guarantee (the existing relative tests — `sums_to_unity`, `mid>0.5`,
  `low>bright` — would not catch a value shift).
- **Additive invariant:** existing analysis tests, `contracts.rs`, and
  `preset_byte_identity` snapshots stay green unchanged (the latter never read
  spectral values, so they are the "chain untouched" invariant, not coverage of
  the 6-band). Add: assert `DeepAnalysis` is produced/cached for a normal track
  and absent (but `SourceProfile` present) for a 1024–16 384-frame clip.
- **Short-clip regimes (two, per §7):** (a) ~1024–16 384 frames → `DeepAnalysis`
  absent, `SourceProfile` present, Tier-1 trims still fire; (b) <1024 frames → both
  absent, fully inert.
- **Stratification / temporal (synthetic fixtures):**
  - *Sustained bright chorus* vs *scattered bright hits* with the **same loudness
    multiset** → assert Phase A's **retained ordered series differs** between the
    two (set-based strata/dispersion alone are order-invariant and would match).
    Phase A only proves the series *preserves order*; the temporal *measure/
    decision* that acts on it is Phase B.
  - *Bright-loud-section + dark-body* → loud-stratum brightness ≫ body; high
    dispersion. *Uniformly bright* → all strata bright, low dispersion.
  - *Localized harsh blip* → harsh present but low coverage in the series.
- **momentary-PSR derivable:** from stored per-window sample peak + loudness key,
  assert momentary-PSR matches a direct computation on a fixture (no re-scan).
- **Determinism:** same input → identical `DeepAnalysis` **on the same platform**
  (fixed-order f64; not asserted cross-platform).
- **Mobile cost:** the §8 benchmark records elapsed ms.

## 13. Acceptance criteria

- `DeepAnalysis` produced once per analyze, cached behind `Arc`, holding: the
  retained **ordered** per-window series, 31-band long-pass curve, loud/body/whole
  strata, IQR dispersion (Fisher-z for correlation), and per-window sample
  peak + loudness key (momentary-PSR-ready).
- "body," dispersion, and the loudness key are defined by constant, not example.
- Phase A changes **no** existing value/behavior; all current tests green; the two
  store maps invalidate in lockstep on both clear paths.
- Contract unchanged; mobile decision recorded; budget cap enforced.

## 14. Deferred (Phase B / C / calibration — explicitly NOT this spec)

- **6-band → 31-band rollup unification** (B): when the chain moves to the rollup,
  it WILL shift the 6-band slightly → at that point add a per-band tolerance test
  (e.g. ≤0.02 share delta) AND an **album label-stability fixture** (album.rs
  threshold classification can flip an arc role; byte-identity does not cover it).
  Not "analogous to F1" (F1 kept the FFT size; the rollup changes aggregation).
- **Confidence/coverage thresholding + actual adaptive-trim changes** (B).
- **PSR/crest closed-loop transient protection** (B) — inputs now retained (§5.3).
- **Preset-identity protection / anti-convergence** (B) — its lever is
  confidence-gated *smaller* trims (strata + dispersion → "hands-off where not");
  no extra Phase-A measurement needed.
- **Holistic already-mastered stand-down** (B) — its inputs (per-stratum true+sample
  peak, integrated LUFS, crest) are retained; it is an interpretation, not a
  measurement.
- **Staged loading UI + `AnalysisSummary` on the wire + "what we found"** (C).
- **Measured-neutral / tilt-vs-reference + by-ear calibration of all thresholds**
  (later Tier-2 / owner listening).

## DECIDE: (owner calls before/at implementation)

1. **Mobile (§8):** gate the deep scan off the iPhone FFI path (recommended) vs
   accept the discarded cost.
2. **Persistence (§6):** persist `DeepAnalysis` to disk (fast reopen) vs
   per-session recompute (simpler) for v1.
3. **(minor) Loudness key (§5.2):** momentary-per-window (default) vs real 3 s
   short-term sampled per window.
