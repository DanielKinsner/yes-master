# Handoff — Adaptive DSP Tier‑2 **Phase A (Deep Analysis) COMPLETE** → start Phase B

Audience: a fresh Claude/agent in a new window picking up the adaptive/"smart" DSP for
YES Master. This is **self‑contained** — read this and you can continue without
re‑reading the session history. It covers what shipped, **all the math**, the
**product vision**, and a concrete Phase B plan.

> **Merged to `main` and pushed on 2026‑06‑03.** Phase A is on `main` (fast‑forward of
> branch `fix/adversarial-type-review-2026-06-03`). Work from `main` or a fresh branch off it.
> **Phase A changes NOTHING audible** — it is purely additive measurement. It is also
> **not yet ear‑calibrated**; calibration is a Phase B + owner concern. Being on `main`
> ≠ taste‑validated.

---

## 0. One‑paragraph orientation

YES Master has a **Tier‑1 adaptive DSP** (merged earlier): presets *defensively trim*
toward neutral when the source already has a quality (bright/boomy/dense/wide). The
problem: Tier‑1 acts on a **thin read**, so it over‑acts on normal material ("boxes
music in") and **homogenizes presets** (a novice can't always hear preset differences on
a given source). The fix is upstream — **see more before acting**. That is **Tier‑2**, a
three‑phase program: **A = measure richly (DONE, this handoff)**, **B = decide with
confidence (NEXT)**, **C = premium surface (later)**. Phase A adds a backend‑internal
`DeepAnalysis` (31‑band tonal curve + an ordered per‑window time series + loudness‑
stratified aggregates), cached per track beside the existing `SourceProfile`, with the
6‑band read kept **byte‑exact**. Nothing reads `DeepAnalysis` yet except tests — Phase B
wires consumption.

---

## 1. TL;DR — exactly where it stands

- **Tier‑1 (floor):** merged previously; defensive reduce‑only trims, per‑axis caps
  (EQ 50 % / comp 60 % / width 70 %), +0.5 dB EQ character floor, one **Adapt Strength**
  dial (`0..1`). **Default lowered 0.6 → 0.5** this round (commit `5f26f0f`; owner call —
  50 % is a safer/visually‑better start, esp. for low‑dynamic‑range sources where the
  impact read can be off). Byte‑identity invariant intact (`preset_byte_identity` SHAs).
- **Tier‑2 Phase A (this work):** **COMPLETE, on `main`.** 10 tasks, tests‑first,
  10 commits (`90946ab..091b288`). Adds `DeepAnalysis`; **changes no existing value**
  (a byte‑exact 6‑band golden test enforces it). Wire contract unchanged (`#[serde(skip)]`
  field; `bindings.ts` untouched; iPhone bridge compiles). Mobile deep scan gated OFF
  (DECIDE default). Per‑session in‑memory cache (DECIDE default).
- **Gates green at merge:** `cargo test` 342 pass / 0 fail (incl. golden); `cargo clippy
  --all-targets` 0 warnings; `npx tsc -b` clean; `npm test` 159 pass; iPhone
  `cargo check --all-targets` clean. Final whole‑implementation review: **READY TO MERGE**,
  no blockers.
- **The branch also carried** the earlier adversarial‑review backlog fixes (F1–F4 +
  dsp shelf‑gain sanitize + the codex review validation docs + the iPhone‑bridge
  verification‑lane addition) — all now on `main` too.
- **Next milestone: Phase B** — make the engine *consume* `DeepAnalysis` for
  confidence‑gated, anti‑homogenizing, transient‑safe adaptation. Start at §7.

---

## 2. The product story (why this exists — read before the math)

The original product idea has two halves: a **pro** mastering surface and a **Simple
Mode** for novices. Both ride one adaptive engine. Two real problems Tier‑1 exposed:

1. **Over‑action on normal material.** A thin read (first read was a single window /
   a coarse tonal estimate) makes the engine "correct" things that aren't problems → it
   *boxes music in*. Owners hear it as the adaptive version sounding smaller/duller.
2. **Preset homogenization.** Because trims pull every preset toward the same neutral
   when the source is "kind of bright/dense," **presets start to feel the same** on a
   given track. For a novice choosing a preset, that kills the product's value — the
   preset *has* to feel impactful and distinct.

**The deep‑analysis thesis (what Dan + Codex aligned on):** do **serious upfront
analysis** — full‑track, loudness‑weighted, finer bands, temporally aware — so Phase B
can be **surgical and confident** instead of blunt. Specifically Phase B should:
- act **strongly only where the measurement is strong and consistent** (high coverage,
  low dispersion), and **hands‑off where it isn't** (anti‑homogenization);
- **protect transients** (PSR/crest closed loop) so "won't crush the life out of it" is
  an honest guarantee, not a hope;
- **stand down on already‑mastered** material (don't re‑master a master);
- eventually present a **premium staged‑loading UI** whose stages are *real analysis
  phases*, plus a plain‑English "here's what we found" summary (Phase C).

UI note from this round: the **Preset Density** control was moved into the **Per‑Band
Compressor** card (commit `c1bf674`) so density lives with the compressor it shapes; the
**Adapt Strength** dial remains the single global adaptive control.

---

## 3. What Phase A computes — the math (all of it)

The decode already happens once. Phase A runs **two passes** over the decoded PCM and
assembles a `DeepAnalysis`. **Principle: A measures, B decides** — there are **no
thresholds, deadbands, or corrections anywhere in Phase A.**

### 3.1 Long pass — 31‑band ⅓‑octave tonal curve (frequency resolution)
- Reuses the **existing whole‑track Welch pass** (sliding power‑of‑two window up to
  `1<<18` ≈ 5.5 s @48k, 50 % hop, Hann, mono = (L+R)/2). **No extra FFT.**
- The **6‑band read keeps its own direct accumulation, byte‑exact.** Phase A adds a
  **parallel** accumulation in the *same* FFT loop that buckets bins into **31 one‑third‑
  octave bands**. In Phase A the 31‑band is **never** the 6‑band's source (no 31→6 rollup
  — deferred to Phase B). `compute_spectral_balance_31band` in `analysis.rs` is a
  **deliberate copy** of `compute_spectral_balance_6band` with exactly three changes
  (accumulator `[f64;31]`, the band lookup, the `[f32;31]` normalized return). **Do NOT
  DRY the two functions** — the 6‑band is golden‑pinned; merging them risks byte drift.
- **Band mapping (`third_octave_band`)**: a ⅓‑octave band centered at `C` spans
  `[C / 2^(1/6), C * 2^(1/6))`. `HALF_STEP = 2^(1/6) ≈ 1.122462` is the half‑bandwidth
  ratio (a band spans `2^(1/3)`; edges are center ÷/× the half‑step). Centers are the IEC
  61260 nominal set `THIRD_OCTAVE_CENTERS: [f32;31]` = 25 Hz … 20 kHz + a `0.0` padding
  slot (index 30). Because nominal centers are *rounded*, adjacent band edges leave tiny
  gaps/overlaps; out‑of‑band bins are dropped via `continue` and the curve is
  **re‑normalized** over captured energy (same behavior the 6‑band already has for
  out‑of‑range bins). Shares sum to ≈1.0.
- **Harsh / sibilant** shares are summed from the 31‑band curve over tunable ranges:
  `HARSH 2–5 kHz`, `SIBILANT 5–9 kHz` (half‑open; 5 kHz lands in sibilant only).

### 3.2 Short pass — per‑window time series (time resolution)
A separate sliding **16 384‑sample (≈0.34 s @48k) window, 50 % hop (8 192), Hann**, over
the whole track. Each window → a `WindowMetrics`:
- **`loudness_key`** — **momentary‑style K‑weighted loudness** over a **400 ms centered
  span** (not the 0.34 s window — so a single transient can't crown a window "loudest").
  Computed by cascading the existing BS.1770 biquads
  `BiquadCoeffs::k_weighting_pre` → `k_weighting_rlb`, mean‑square → `LUFS = -0.691 +
  10·log10(ms)` (the `-0.691` is the BS.1770 absolute‑gating calibration offset). **Edge
  handling:** the 400 ms span is clamped to available samples at track start/end (shrink,
  never zero‑pad — zero‑padding would falsely lower edge loudness); a window with < ~half
  the span available is `NEG_INFINITY` (treated as non‑finite, excluded from all strata).
  Distinct from `AnalysisResult.lufs_short_term_max` (the true 3 s Mode::S whole‑track
  measure).
- **`sample_peak`** — linear sample peak over the window (cheap; for momentary‑PSR +
  crest). *True* peak is NOT done per window (would need a 4× oversampled pass — deferred
  to Phase B); whole‑track true peak stays the ceiling read.
- **`crest`** — `peak / RMS` (linear), RMS via f64 sum‑of‑squares; `1.0` if RMS≈0.
- **`stereo_width`** — `side_e / (mid_e + side_e)` where mid=(L+R)/2, side=(L−R)/2; `0.0`
  for mono.
- **`stereo_correlation`** — two‑pass Pearson L/R, clamped `[-1,1]`; **`NaN` for mono**
  (degenerate). `1.0` if variance ≈0.
- **`low/mid/high`** — the existing **3‑band** `compute_spectral_balance` read on the
  window (reused; cheap) for temporal brightness clumping. ⚠️ **Inherited SR skew:**
  `compute_spectral_balance` bakes in a ~44.1k‑reference RC band split, so at 48k+ the
  3‑band edges sit slightly off (~3 kHz mid/high split). It's consistent/deterministic
  and matches how `infer_character` reads tone elsewhere — documented at the call site;
  revisit if Phase B wants finer per‑window banding.
- **Window‑count cap:** `MAX_SCAN_WINDOWS = 4200` (~12 min @48k / 8192 hop). Beyond that,
  `scan_windows` **widens the hop** (strides) so the count never exceeds the cap. (Proven
  exhaustively in review: max count is exactly 4200, never 4201 — striding only triggers
  at large hops where integer‑division overshoot can't occur.)

### 3.3 Aggregates derived from the series (`DeepAnalysis::from_parts`)
For each axis (`loudness`, `crest`, `brightness` = per‑window `high` share,
`stereo_width`, `stereo_correlation`) compute an `AxisStrata { whole, loud, body,
dispersion }` over the **same** per‑window loudness keys (so "loud" and "body" mean the
same windows across axes):
- **`whole`** = mean over all **finite‑key** windows (silent/`NEG_INFINITY` excluded).
- **`loud`** = mean of the top **`LOUD_STRATUM_FRACTION = 15 %`** of windows by loudness
  key.
- **`body`** = mean over the central **`[BODY_PCTL_LO=0.25, BODY_PCTL_HI=0.75]`**
  percentile band of windows by loudness key.
- **`dispersion`** = **IQR** (p75−p25, linear‑interpolated percentiles) of the per‑window
  values. **Exception:** the correlation axis uses **IQR of the Fisher‑z transform**
  `z = 0.5·ln((1+c)/(1−c))` (clamped to ±0.999999 to avoid ±∞), because the ordinary
  spread of a bounded `[-1,1]` stat is meaningless.
- **Determinism:** all means accumulate in **f64 in fixed window order**; sorts use
  `f32::total_cmp`; the loud‑stratum tiebreak is an **explicit `(loudness_key,
  window_index)`** sort (robust even if someone swaps in `sort_unstable`); default scalar
  `rustfft`; no parallel float reduction. Determinism is **per‑platform** (Win/macOS/ARM
  floats differ — preset SHAs are already per‑OS), so **no cross‑platform golden**.
- **momentary‑PSR is derivable, not stored:** per window `PSR = 20·log10(sample_peak) −
  loudness_key`. Phase B reconstructs it from the retained series without re‑scanning.

**Why retain the ordered series (not just strata)?** Set‑based strata are order‑invariant:
a *sustained* bright chorus and *scattered* bright hits with the **same loudness multiset**
produce the **same strata** but **different ordered series**. Keeping the ~tens‑of‑KB
ordered `windows` is what lets Phase B distinguish sustained vs scattered (temporal
structure), recompute cuts, and derive PSR. (Integration test B proves the series differ
while strata match.)

---

## 4. Data model & data flow (additive)

```
analyze_one(track_id, path, deep: bool)          // analysis.rs
  ├─ existing measurements (6-band, DR, LRA, corr, width, energy_density, role/character)
  └─ if deep {                                    // Phase A; gated off on mobile (Task 9)
        bands31  = compute_spectral_balance_31band(&pcm.samples, sr, ch)   // long pass
        windows  = deep_analysis::scan_windows(&pcm.samples, sr, ch)       // short pass
        Some(Arc::new(DeepAnalysis::from_parts(bands31, windows)))         // assemble
     } else { None }
  → AnalysisResult { …, #[serde(skip)] deep_analysis: Option<Arc<DeepAnalysis>> }

analyze_tracks_core(tracks)        → analyze_tracks_core_impl(tracks, deep=true)   // desktop
analyze_tracks_core_lite(tracks)   → analyze_tracks_core_impl(tracks, deep=false)  // iPhone bridge
populate_profile_store(store, results):           // engine.rs
   for r in results:
      store.set(r.track_id, SourceProfile::from_analysis(r))   // by_track map (unchanged)
      store.insert_deep(r.track_id, r.deep_analysis.clone())   // by_track_deep map (Arc, new)
```

**`DeepAnalysis` (backend‑internal, `deep_analysis.rs`)** — `#[derive(Debug, Clone)]`,
**not `Copy`** (carries a `Vec`), **no `Serialize`/`Deserialize`** (never on the wire):
```
DeepAnalysis {
  bands_31: [f32; 31],            // ⅓-octave whole-track shares (sum ≈ 1)
  harsh_share: f32, sibilant_share: f32,
  windows: Vec<WindowMetrics>,    // the retained ordered series
  loudness, crest, brightness, stereo_width, stereo_correlation: AxisStrata,
}
WindowMetrics { loudness_key, sample_peak, crest, stereo_width, stereo_correlation,
                low, mid, high: f32 }            // Copy
AxisStrata    { whole, loud, body, dispersion: f32 }   // Copy, PartialEq
```
**Store (`profile_store.rs`)** — a **second map** `by_track_deep: Mutex<HashMap<TrackId,
Arc<DeepAnalysis>>>` beside `by_track`. Methods `insert_deep(id, Option<Arc<…>>)`
(Some→insert, None→remove) and `get_deep(&id) -> Option<Arc<…>>` (clones the Arc, cheap).
**Lock discipline:** every method locks **exactly one** map; the two clear paths call the
setters **sequentially** — **no path ever holds both locks**, so there is no lock‑ordering
hazard. Both maps invalidate in lockstep on **both** clear paths: (a) normal populate
(`set` + `insert_deep` per result), (b) hard‑fail `prune_failed_profiles` (both cleared).

**Why a `#[serde(skip)]` field instead of threading a tuple?** To keep
`analyze_tracks_core(tracks)`'s signature stable — the iPhone bridge calls it and
`contracts.rs` pins it. `serde(skip)` keeps the field off the wire entirely (no serde
bound on `DeepAnalysis`; `bindings.ts` needs no change; deserializes to `None`).

---

## 5. File map (Phase A)

- **`src-tauri/src/deep_analysis.rs` (NEW)** — the whole Phase A model + math:
  constants (`SHORT_WINDOW`, `SHORT_HOP`, `MOMENTARY_MS`, `LOUD_STRATUM_FRACTION`,
  `BODY_PCTL_LO/HI`, `MAX_SCAN_WINDOWS`, `HARSH/SIBILANT_*`); structs `WindowMetrics`,
  `AxisStrata`, `DeepAnalysis`; helpers `percentile_sorted`, `iqr`, `axis_strata`,
  `fisher_z_iqr`, `band_center_hz`, `harsh_sibilant_from_bands`; the short pass
  `scan_windows`/`measure_window`/`kweighted_lufs`/`stereo_window`; `DeepAnalysis::from_parts`.
  **All tuning constants live here (single source of truth).**
- **`src-tauri/src/analysis.rs`** — `analyze_one(.., deep: bool)` builds+attaches
  `DeepAnalysis`; `compute_spectral_balance_31band` + `THIRD_OCTAVE_CENTERS` +
  `third_octave_band` (long pass); `compute_spectral_balance` made `pub(crate)`;
  `compute_spectral_balance_6band` **byte‑exact, golden‑locked** (`spectral_balance_6band_is_byte_exact_golden`).
- **`src-tauri/src/types.rs`** — `AnalysisResult.deep_analysis: #[serde(skip)]
  Option<Arc<DeepAnalysis>>`.
- **`src-tauri/src/profile_store.rs`** — the `by_track_deep` Arc map + `insert_deep`/
  `get_deep` + both clear paths.
- **`src-tauri/src/engine.rs`** — `analyze_tracks_core` (deep on) / `analyze_tracks_core_lite`
  (deep off) / shared `analyze_tracks_core_impl(tracks, deep)`; `populate_profile_store`
  moves `DeepAnalysis` into the store. `AnalyzeRequest` gained `#[derive(Clone)]` (additive,
  wire‑safe — it's Deserialize‑only).
- **`apps/iphone-native/rust/src/lib.rs`** — bridge repointed to `analyze_tracks_core_lite`.
- **`src-tauri/tests/contracts.rs`** — regime gate test + lite‑vs‑core gate test +
  `#[ignore]` cost benchmark.
- **`src-tauri/tests/deep_analysis_integration.rs` (NEW)** — stratification (loud≫body) +
  temporal (sustained≠scattered series, strata match) + tiny‑clip (both absent).

---

## 6. Build / test / verify (exact commands)

Repo: `C:\Users\SM - Dan\Documents\GitHub\yes-master`. **Use `--target-dir target/codex-rc`**
for cargo so you don't lock a running app's `target/`.

```powershell
# Rust (desktop lib + integration)
cd src-tauri
cargo test --target-dir target/codex-rc            # full suite; golden = spectral_balance_6band_is_byte_exact_golden
cargo clippy --all-targets --target-dir target/codex-rc
# Frontend (must stay unchanged — Phase A is serde-skip)
cd ..
npx tsc -b --pretty false
npm test                                            # vitest
# iPhone bridge (shared types changed → ALWAYS check this lane)
cd apps/iphone-native/rust
cargo check --all-targets
# Optional: the deep-scan cost benchmark (ignored by default)
cd ../../../src-tauri
cargo test --target-dir target/codex-rc bench_deep_scan_cost --ignored -- --nocapture
```
Benchmark observed: deep path ≈ **4051 ms** vs lite ≈ **3231 ms** on a 30 s stereo source
(**debug build**, so absolute numbers are inflated; the ~**820 ms delta** is the deep‑scan
cost the mobile path now avoids). Release will be far faster; re‑measure if it matters.

---

## 7. Phase B — the next milestone (concrete plan)

**Goal:** consume `DeepAnalysis` to make Tier‑1's trims **confident, surgical, and
non‑homogenizing**, with honest transient protection. **A measured; B decides** — all
thresholds/confidence live in Phase B. Suggested order:

1. **Wire the read.** `get_deep(track_id)` already exists in the store but has **no
   production consumer** (test‑only — this is the intended Phase A boundary). The chain
   resolves `SourceProfile` via `apply_resolved_profile`; add a parallel resolve for
   `Option<Arc<DeepAnalysis>>` so the chain can read it where present and **degrade
   gracefully to `SourceProfile`** when absent (short clips, mobile). **Keep byte‑identity:
   gate every new behavior on the deep read being present AND strength > 0.**
   ⚠️ Cross‑map TOCTOU: the two store maps use separate locks, so a reader can momentarily
   see profile‑without‑deep. Benign today (no consumer needs them atomic); if Phase B needs
   an atomic profile+deep read, that requires a single lock over a combined struct, not two
   getters.

2. **Confidence / coverage gating (anti‑homogenization core).** Derive, per axis:
   - **coverage** = fraction of windows exhibiting the trait (e.g. how much of the track is
     actually bright), and
   - **consistency** = inverse of `dispersion` (IQR) — low dispersion ⇒ the trait is a
     stable property, high dispersion ⇒ scattered.
   Scale the Tier‑1 trim by `confidence = f(coverage, consistency)`: **act strongly only
   when the quality is strong, broad, and consistent; hands‑off otherwise.** This is the
   direct fix for preset homogenization — neutral/ambiguous sources get *small* trims so
   presets stay distinct.

3. **PSR/crest closed loop (honest transient protection).** Using the retained per‑window
   `sample_peak` + `loudness_key` (momentary‑PSR) and `crest`: if transients are already
   healthy, **don't compress** (or back off density). This makes "won't crush the life out
   of it" a measured guarantee. (Inputs are already retained — no new measurement needed.)

4. **Holistic already‑mastered stand‑down.** Combine per‑stratum sample peak (+ later true
   peak), integrated LUFS, and crest into an "already mastered" interpretation → **stand
   down** (minimal/zero trims). It's an *interpretation of retained inputs*, not a new
   measurement. (Owner asked: yes, treat this as part of the overall detection, not a
   separate feature.)

5. **6‑band → 31‑band rollup unification (the one risky change).** When the chain moves
   from the 6‑band to a 31‑band‑derived tonal read, the 6‑band values **will shift
   slightly** (aggregation changes). At that point you MUST:
   - swap the byte‑exact 6‑band golden for a **per‑band tolerance test** (e.g. ≤0.02 share
     delta), AND
   - add an **album label‑stability fixture** — `album.rs` heavy/acoustic/transition
     **threshold classification can flip a track's arc role**, and byte‑identity does NOT
     cover that. This is *not* "analogous to F1" (F1 kept the FFT size; the rollup changes
     aggregation). Treat this as its own carefully‑reviewed change.

6. **Calibration + owner listening (gate, not yours to clear).** All Phase B thresholds
   are provisional until the owner A/Bs Adapt `0` vs `50 %` on already‑mastered / bright /
   dense / wide **and** neutral sources, confirms neutral does ~nothing, and locks the
   numbers. Use the slow fixture lane with private audio (`AMS_RUN_REAL_FIXTURE`).

**Phase C (later):** premium **staged loading UI** whose stages = the real phases (decode
→ long pass → short pass → assemble), plus a curated **`AnalysisSummary`** on the wire (a
few flags + a plain "what we found" summary) — **never** the raw per‑window series. That
is the only point new types cross the TS contract.

---

## 8. DECIDE calls (defaults baked this round — flip freely)

1. **Mobile = deep OFF (baked).** iPhone bridge uses `analyze_tracks_core_lite` (no deep
   scan; saves ~CPU/battery; mobile is non‑adaptive anyway). **Flip:** point the bridge
   back at `analyze_tracks_core`, or build a real lite‑adaptive phone path (new FFI/Swift
   wiring — out of scope so far).
2. **Persistence = per‑session in‑memory (baked).** `DeepAnalysis` is recomputed on app
   reopen. **Flip:** add `Serialize`/`Deserialize` + a content/version key + invalidation
   to persist it to disk (faster reopen for multi‑track albums).
3. **(minor) Loudness key = momentary‑per‑window (baked)** vs sampling real 3 s short‑term
   per window. Internal only; never surfaces to the user. If flipped, momentary‑PSR
   becomes short‑term‑PSR.

All three are cheap to flip *because Phase A is behavior‑neutral* — nothing audible
depends on them yet.

---

## 9. Gotchas (don't relearn these the hard way)

- **Do NOT DRY `compute_spectral_balance_6band` and `compute_spectral_balance_31band`.**
  The 6‑band is golden‑pinned (`spectral_balance_6band_is_byte_exact_golden`); they are a
  deliberate copy. Both functions carry NOTE comments saying so.
- **Phase A must change no existing value.** The golden test is the tripwire. If you ever
  see it fail, you changed the 6‑band path — that's the gate forcing an explicit decision
  (see §7.5).
- **`DeepAnalysis` is backend‑only.** Never add it to `bindings.ts`; never derive
  `Serialize`. The wire stays unchanged (the field is `#[serde(skip)]`).
- **Always run the iPhone lane** (`apps/iphone-native/rust` `cargo check --all-targets`)
  when you touch shared `yes_master_lib` types — it re‑uses them and is NOT built by the
  desktop test lanes. Cross‑crate arity/field drift has bitten here repeatedly; this is
  why `analyze_tracks_core`'s signature was kept stable.
- **`get_deep` has no production consumer yet** — test‑only by design until Phase B §7.1.
- **Inherited 3‑band SR skew** (§3.2): the per‑window `low/mid/high` uses
  `compute_spectral_balance`'s ~44.1k reference; slightly off at 48k+. Documented at the
  call site; revisit only if Phase B needs finer per‑window banding (the long‑pass 31‑band
  is the accurate tonal curve).
- **Determinism is per‑platform**, not cross‑platform (float differences; preset SHAs are
  already per‑OS). Don't add a cross‑platform golden.
- **Tier‑1 byte‑identity (`preset_byte_identity` SHAs)** must stay green — gate any Phase B
  chain behavior on the profile/deep read being present and strength > 0.
- **`--target-dir target/codex-rc`** for cargo when the app might be running (it locks
  `target/debug/yes-master.exe`). **LF→CRLF** warnings on commit are benign.
- **Tuning constants live ONLY in `deep_analysis.rs`** (Phase A) and `guardrails.rs`
  (Tier‑1). Don't scatter them.

---

## 10. Owner gates (not yours to clear)

- **Listening signoff** of Phase B trims (A/B Adapt 0 vs 50 % across source types; confirm
  neutral ≈ no‑op; lock constants). Default Adapt Strength is **50 %** now.
- **Slow fixture lane** with private reference audio (`AMS_RUN_REAL_FIXTURE`).
- Measured‑neutral / tilt‑vs‑reference (later Tier‑2; needs the owner's reference masters).

---

## 11. Pointers (authoritative sources)

- **Spec (Phase A design, v2.1, APPROVED):**
  `docs/superpowers/specs/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis-design.md`
  — full math, the three‑phase program, the Phase B/C deferral list, the DECIDE calls.
- **Plan (10 tasks, tests‑first):**
  `docs/superpowers/plans/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis.md`.
- **Tier‑1 + earlier Tier‑2 planning handoff:**
  `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2.md` (the engine, B2 backend‑owned profile,
  data flow, `preset_byte_identity`).
- **Tier‑1 build handoff (historical):** `docs/HANDOFF_2026-06-02_ADAPTIVE_DSP_TIER1.md`.
- **Adversarial review validated this round:** `docs/reviews/2026-06-03-codex-adversarial-repo-review.md`.
- **Phase A commits:** `git log --oneline 90946ab..091b288` (Tasks 1→10).

---

## 12. Quick‑start for the next agent

```
1. Read this doc + the spec (§ pointers). Confirm you're on `main`.
2. Run the §6 verification lanes — confirm all green (golden test included).
3. Phase B starts at §7.1: add a deep-read resolve in the chain path, gated, byte-identity
   preserved. Then §7.2 confidence/coverage gating (the anti-homogenization core).
4. ALWAYS run the iPhone `cargo check --all-targets` lane after touching shared types.
5. Tests-first. Small commits. Don't break the 6-band golden or preset_byte_identity.
```
