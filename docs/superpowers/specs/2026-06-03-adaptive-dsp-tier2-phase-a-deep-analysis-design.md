# Adaptive DSP Tier-2 — Phase A: Deep Analysis (design)

Date: 2026-06-03
Status: design approved (brainstorm); pre-implementation
Branch context: `fix/adversarial-type-review-2026-06-03` (Tier-1 finish + review fixes already landed here)

## 1. Context

The adaptive guardrails (Tier-1) are merged and conservative-by-design, but two
real problems remain:

- **Blunt corrections.** The engine acts on a thin read of the track, so it
  over-acts on normal material and can box music in.
- **Preset convergence.** Adaptation pulls every preset toward "what the source
  needs," which narrows the audible spread between presets on a given source —
  a product risk ("a novice clicks a preset and doesn't hear a difference").

The fix is upstream: **make the engine see more before it acts.** Better sight →
confident where warranted, hands-off where not → less blunt, less homogenizing.

This is **Phase A of a 3-phase program**:

- **Phase A — Deep Analysis (this spec):** measure the track richly. Factual only.
- **Phase B — Confidence-gated adaptation (future):** consume Phase A's signals;
  trims scale by confidence/coverage; PSR/crest transient protection;
  preset-identity protection (the anti-convergence piece); the holistic
  "already-mastered → stand down" read.
- **Phase C — Premium surface (future):** honest staged loading flow (stages map
  to real pipeline phases, no theater) + a plain "what we found" summary.

The load-bearing principle across the program: **Phase A *measures*; Phase B
*decides*.** No thresholds or correction logic live in Phase A.

## 2. Goal & non-goals

**Goal:** produce a rich, whole-track, loudness-aware analysis (`DeepAnalysis`)
that Phase B can use to make confident, surgical, non-homogenizing decisions.

**Phase A explicitly does NOT:**

- Apply any threshold, deadband, confidence score, or correction (all Phase B).
- Change the *intended* audible behavior of the chain. Its only behavior delta is
  a minor numeric shift in the 6-band read (it now derives from the new scan's
  whole-track rollup), analogous to the F1 whole-track-Welch change already
  shipped — accepted, with existing relative tests staying green. The real
  behavior change (less neutering, preserved preset distinctiveness) is Phase B.
- Add named/semantic sections (verse/chorus). Statistical loudness strata only;
  named sections can be layered later for the Phase C summary.
- Touch measured-neutral / tilt-vs-reference (later Tier-2; needs calibration).

## 3. Key principles

1. **Measure fine, expose coarse.** Internal resolution can be as detailed as
   useful; the UI and the TS↔Rust contract stay simple (6 bands + a couple of
   derived flags).
2. **A measures, B decides.** Phase A outputs factual values + threshold-free
   dispersion; Phase B owns all thresholds/confidence.
3. **`DeepAnalysis` is an optional ceiling over the `SourceProfile` floor.** The
   chain must never *require* `DeepAnalysis`; it degrades to the compact
   `SourceProfile` (the existing trims) when `DeepAnalysis` is absent.
4. **One-time, cached.** Phase A runs once at analysis (import / first master).
   Live audition reads the cache — zero added real-time cost (preserves the
   near-real-time-audition non-negotiable).
5. **Byte-identity preserved.** No profile / Adapt Strength 0 → bit-for-bit the
   non-adaptive chain (`preset_byte_identity` snapshots stay valid).

## 4. What Phase A computes

### 4.1 Windowed scan (whole track)

Slide an FFT window across the entire decoded track (mono-summed for the
spectral pass). Per window, measure:

- **Tonal** — 1/3-octave band energies (see §5).
- **Dynamics** — per-window crest (short-term peak / RMS). This is the
  transient-richness signal Phase B's PSR/crest protection will use; the
  existing whole-track DR (P95–P10) and LRA are retained unchanged.
- **Loudness** — short-term LUFS (also the *weighting key* for §4.2).
- **Stereo** — width (side/mid) + L/R correlation.
- **Peak** — true / sample peak.

### 4.2 Loudness-stratified aggregates

For each metric, summarize three ways so a quiet intro can't misrepresent the
master:

- **Whole-track** — all windows.
- **Loudest 10–20%** — the windows with the highest short-term LUFS ("what the
  master is about").
- **Body** — the central/median windows (the track's typical character).

### 4.3 Dispersion (threshold-free)

Per axis, a measure of how much the value varies across the track (e.g.,
variance or inter-quartile spread of the per-window values). This is the
"localized vs persistent" signal — **without** applying any threshold. Phase B
turns strata + dispersion into a confidence/coverage decision using its
(tunable) thresholds.

## 5. Band resolution

- **Internal: 1/3-octave (~31 bands, 20 Hz – 16 kHz).** Engineer-standard RTA
  resolution; cheap (same FFT, more buckets); gives a real spectral curve that
  the future tilt-vs-reference / measured-neutral work can use directly.
- **UI: the existing 6 bands**, computed as a rollup of the 1/3-octave bands.
- **Harsh ≈ 2–5 kHz, sibilant ≈ 5–9 kHz**, as **tunable constants** (single
  source of truth, like the guardrail deadbands) so the edges can be nudged by
  ear without code surgery. These are sub-ranges of the 1/3-octave bins — no
  special-casing.

## 6. Data model

- **`DeepAnalysis`** — a new **backend-internal** struct (Rust only; never
  serialized to TypeScript). Holds the 31-band stratified energies (whole / loud
  / body), per-stratum crest, dispersion per axis, stereo, peak.
- **`SourceProfile`** — unchanged shape: stays the compact, `Copy`,
  `bindings.ts`-mirrored summary the chain consumes. Only its **`spectral_6`
  field** changes origin — it now derives from the scan's whole-track rollup
  rather than the old single-FFT read; the other fields (DR P95–P10, LRA, stereo
  correlation, width) keep their existing whole-track sources unchanged.
- **Profile store** (the B2 `SourceProfileStore`) caches `DeepAnalysis` keyed by
  `TrackId` (alongside / superseding the cached `SourceProfile`); the
  `prune_failed_profiles` invalidation (F3) extends to it.
- **TS↔Rust contract is unchanged.** No 31-band arrays cross the wire. (Phase C
  will add a small curated `AnalysisSummary` for the "what we found" UI — not
  the raw bands.)

## 7. Integration with the existing analysis

- The windowed scan **replaces `analysis.rs::compute_spectral_balance_6band`**:
  the 6-band value becomes a rollup of the scan's whole-track stratum. The scan
  is a superset, so nothing is lost and the coarse read is not computed twice.
- **Fallback (functional requirement, not just defensive):** the scan's larger
  window (~16 k samples, §11) has a *higher minimum track length* than the
  simple 6-band (~1024 samples). For very short clips (~0.02–0.34 s) the scan
  cannot fit a window. In that case (or any scan failure), fall back to the
  existing simple whole-track 6-band computation for the 6-band value;
  `DeepAnalysis` is simply **absent** → adaptation goes inert (correct for a clip
  too short to characterize). This preserves the input range the app accepts.
- **Unchanged:** whole-track integrated/short-term LUFS, true peak, P95–P10 DR,
  LRA, stereo correlation — all stay as computed today.
- **Downstream consumers** of `spectral_balance_6band` (role/character inference,
  album bias) consume the rollup — same semantics, slightly different numbers
  (the same surface F1 already shifted safely; covered by §12).

## 8. Cross-platform

- **Deep analysis runs everywhere** (desktop + mobile) — it's shared Rust
  (`yes_master_lib`), one path, uniform quality, no desktop/mobile drift.
- **`SourceProfile` floor is a *budget-based* fallback**, not a platform nerf:
  if a device/track exceeds a cost budget (very long file, slow device), bound
  the scan (cap analyzed duration / window density) and, in the limit, fall back
  to the `SourceProfile`-only path. The chain already degrades gracefully (§3.3).
- **Mobile is currently fully non-adaptive** (the iPhone bridge sets
  `source_profile: None`). This design does not force adaptation onto mobile; it
  keeps a lite-adaptive phone path open without ever forcing the heavy scan onto
  the device.

## 9. Performance

- **Live audition: zero added cost** (reads the cached result; adaptation itself
  is ~free per sample — only chain coefficients change).
- **One-time analysis (cached):** desktop ~0.2–0.7 s for a typical 3–4 min track
  (FFT scan ~120–200 ms; decode often larger), ~1–1.5 s for long tracks. Mobile
  estimated ~1–2.5 s (rustfft on ARM + decode) — **to be validated on-device**
  (§12). Boundable via mono sum + window cap (§8).

## 10. Components / files touched

- `src-tauri/src/analysis.rs` — new windowed-scan function producing
  `DeepAnalysis`; `compute_spectral_balance_6band` becomes the short-clip
  fallback; 6-band derived from the rollup.
- `src-tauri/src/types.rs` (or a new `deep_analysis.rs`) — the `DeepAnalysis`
  struct (backend-internal; not in `bindings.ts`). `SourceProfile` shape
  unchanged; its derivation switches to the rollup.
- `src-tauri/src/profile_store.rs` — cache `DeepAnalysis`; extend
  `prune_failed_profiles` to it.
- `src-tauri/src/engine.rs` — `analyze_tracks` / `populate_profile_store`
  populate `DeepAnalysis`.
- No `src/` (frontend) or `bindings.ts` changes in Phase A (contract unchanged).

## 11. Implementation parameters (specified, not open)

- **FFT window: ~16384 samples (~0.34 s @ 48 k), 50% hop, Hann.** The
  time-vs-frequency tradeoff: large enough for usable low-end 1/3-octave bands
  (≈2.9 Hz/bin), short enough for ample windows / time resolution. Mono-summed.
- **Dynamics metric:** per-window crest (short-term peak/RMS); whole-track
  DR (P95–P10) and LRA retained unchanged.
- **Loud stratum:** windows in the top 10–20% by short-term LUFS (exact cut a
  constant; start at top 15%).
- **Cache:** keyed by `TrackId`; re-analysis replaces; failed re-analysis prunes
  (F3). Confirm invalidation when source content changes under a reused id.
- **Harsh/sibilant edges + loud-stratum cut:** tunable constants in one place.

## 12. Testing

- **Behavior-shift bounds:** assert the new 6-band rollup ≈ the prior whole-track
  6-band within a tolerance on a known fixture (role/character/album don't shift
  wildly); existing relative adaptive tests, byte-identity snapshots, and
  `contracts.rs` stay green.
- **Stratification proves out (synthetic fixtures):**
  - *Bright-drop-over-dark-body* (loud bright section + longer dark body) →
    loud-stratum brightness ≫ body-stratum; dispersion high. (Phase B will trim
    less here; Phase A asserts the strata/dispersion reflect it.)
  - *Uniformly bright* → all strata bright; dispersion low.
  - *Localized harsh blip* → harsh present in a stratum but low dispersion/spread.
- **Short-clip fallback:** a clip below the window minimum → scan yields nothing,
  6-band falls back to the simple computation, `DeepAnalysis` absent, adaptation
  inert.
- **Determinism:** same input → identical `DeepAnalysis` (no `Date::now`/random).
- **Mobile cost:** a small benchmark in the iPhone test suite analyzes a known
  fixture and prints elapsed ms, so the "deep everywhere" decision rests on a
  real number and we know whether the §8 budget fallback ever fires.

## 13. Acceptance criteria

- `DeepAnalysis` is produced once per analyzed track, cached, with the 31-band
  stratified energies + per-axis dispersion + per-stratum crest/stereo/peak.
- The 6-band read and `SourceProfile` derive from the scan, with the short-clip
  fallback intact; the input range the app accepts is unchanged.
- Live audition adds zero analysis cost; existing tests stay green; the 6-band
  shift stays within the §12 tolerance.
- TS↔Rust contract unchanged. Mobile benchmark recorded.

## 14. Deferred (Phase B / C / calibration — explicitly NOT this spec)

- Confidence/coverage thresholding + the actual adaptive-trim changes (B).
- PSR/crest closed-loop transient protection (B).
- Preset-identity protection / anti-convergence (B).
- Holistic already-mastered "stand down" read (B).
- Staged loading UI + `AnalysisSummary` on the wire + "what we found" (C).
- Measured-neutral / tilt-vs-reference + the by-ear calibration of all thresholds
  (later Tier-2 / owner listening).
