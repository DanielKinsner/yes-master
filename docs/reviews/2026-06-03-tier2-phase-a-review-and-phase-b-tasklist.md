# Adversarial Review — Tier‑2 **Phase A (Deep Analysis)** + Phase B task list

**Audience:** the agent who implemented Phase A, and whoever picks up Phase B next.
**Reviewer:** Claude (Opus 4.8), independent adversarial pass.
**Date:** 2026‑06‑03.
**Reviewed:** `main` @ `366235c` (the Phase A‑complete handoff). Phase A code range
`90946ab..091b288`. Companion doc: `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2_PHASE_A_COMPLETE.md`.

---

## 0. Verdict (TL;DR)

**Phase A is implemented correctly, and the handoff is honest.** The "purely additive,
behaviour‑neutral, gates‑green" claims all hold up — I re‑ran every gate myself (not just trusted
the doc) and they pass exactly as stated. The byte‑exact 6‑band golden invariant is real and load‑
bearing. The integration architecture (serde‑skip backend type, dual single‑lock store maps,
lite/core split, iPhone bridge repoint) is sound.

There are **two real *latent* math defects** in `deep_analysis.rs`. Neither affects anything today —
nothing in production consumes `DeepAnalysis` yet, no test or audible path is touched, the 6‑band
golden is untouched — so they are **not merge blockers**. But Phase B is specifically designed to
consume the exact values these defects corrupt, so **fix both before wiring Phase B** (they are
cheap and behaviour‑neutral to fix now). Everything else is low‑severity doc/precision.

| ID | Severity | What | Fix before Phase B consumes it? |
|----|----------|------|----|
| **F1** | **Medium** | Per‑window momentary loudness span is ~570 ms & off‑centre, not the documented "400 ms centered" | **Yes** — §7.3 PSR loop + all strata key on it |
| **F2** | **Medium** | Mono track ⇒ `stereo_correlation` strata `whole/loud/body` become **NaN** | **Yes** — §7.2 width/correlation gating reads it |
| F3 | Low | `sample_peak` is the **mono‑downmix** peak, not per‑channel; doc says "sample peak" | Doc/caveat; relevant to §7.4 |
| F4 | Low | A track in `[1024, 16384)` frames has a `SourceProfile` but **no** `DeepAnalysis` | Advisory for §7.1 wiring |
| F5 | Low | Spec/plan say "25 Hz…16 kHz / 31 bands"; code & handoff are "25 Hz…20 kHz / 30 real + padding" | Doc reconcile |
| F6 | Nit | Handoff calls `lufs_short_term_max` "the true 3 s Mode::S measure"; it falls back to `integrated+LRA/2` for short clips | Doc precision |

**Do not chase these three — I refuted them adversarially** (listed so you don't re‑investigate):
the `avail >= mom_frames/2` guard (dimensionally consistent, and inert for real tracks); the
absence of per‑window K‑weighting filter warm‑up (standard BS.1770 practice, sub‑0.01 LU); the
"poisoned‑mutex silent no‑op" (unreachable — nothing panics under the lock).

---

## 1. What I independently verified (evidence, not assertion)

Re‑ran on current `main` with `--target-dir target/codex-rc`:

- `cargo test` → **342 passed / 0 failed** (matches the handoff exactly), incl.
  `spectral_balance_6band_is_byte_exact_golden` ✅ and the lite‑vs‑core contract gate
  `analyze_tracks_core_lite_skips_deep_analysis_but_core_includes_it` ✅.
- `cargo clippy --all-targets` → **0 warnings**.
- `npx tsc -b` → **clean (exit 0)**; `npm test` (vitest) → **159 passed**.
- `apps/iphone-native/rust` `cargo check --all-targets` → **clean** (the shared‑type bridge still
  compiles; the field add was wire‑safe).

Methodology: a full manual read of `deep_analysis.rs` + every integration diff, then a 5‑dimension
adversarial workflow (17 agents) where **every candidate finding was handed to a skeptic instructed
to refute it against the actual code**. Only findings that survived refutation are listed above.

Independently **confirmed correct** (high‑value, so you can trust these):

- **The `MAX_SCAN_WINDOWS = 4200` cap is proven** — count is exactly 4200, never 4201. The reason
  the handoff gives is right: striding only engages when `D = total_frames − SHORT_WINDOW ≥ 4200·8192`,
  which forces `q = floor(D/4199) ≥ 8193`, so `floor(D/q) = 4199` exactly. The overshoot‑to‑4201
  regime (`q < 4199`) is never reached. Verified analytically *and* numerically across the boundary.
- **`compute_spectral_balance_31band` is a faithful, standalone copy** of the 6‑band Welch loop
  (pow‑2 ≤ `1<<18`, 50 % hop, Hann, skip‑DC, `take(bins)`), with exactly the three documented
  changes, and it never feeds the 6‑band path → golden cannot drift. ✅ Keep them un‑DRYed.
- **`third_octave_band` never double‑counts a bin** (single return on first match); nominal‑centre
  rounding leaves ~0.79 % overlaps (resolved to the lower band) and ~1.5 % gaps (dropped + re‑
  normalised). No band of energy is systematically lost.
- **Store lock discipline is correct**: every method locks **exactly one** map; both writers
  (`populate_profile_store`, `prune_failed_profiles`) touch both maps in lockstep via sequential
  single‑lock setters; no path holds both locks → no deadlock. The cross‑map TOCTOU is real but
  benign (no production reader of `get_deep`).
- **`#[serde(skip)]` is correct** and the `Arc` move into the store is a refcount bump, not a deep
  clone. The only direct serializer (iPhone FFI) runs the lite path where `deep_analysis` is always
  `None`.
- **The pure aggregators are correct** (`percentile_sorted`, `iqr`, `axis_strata` body/loud bands,
  determinism via f64 + `total_cmp` + explicit `(key, index)` tiebreak). I recomputed the pinned
  `fisher_z_iqr` value `0.7554661` by hand — it's right.
- **Panic‑safety** on empty/mono/odd‑channel/truncated‑tail inputs holds.

---

## 2. Findings (detail + fixes)

### F1 — Medium — Momentary loudness span is ~570 ms and off‑centre, not "400 ms centered"
**`src-tauri/src/deep_analysis.rs`, `measure_window`, lines ~290‑302.**

The handoff §3.2 and the spec §5.2 both define `loudness_key` as a **400 ms span centred on the
window** ("so a single transient can't crown a window 'loudest'"). The code integrates over a
different interval:

```rust
let half   = mom_frames / 2;                       // 9600 @48k
let mom_lo = start.saturating_sub(half);           // start − 9600   ← anchored on window START
let mom_hi = (start + window / 2 + half).min(total);// start + 17792  ← anchored on window CENTRE
```

The two bounds use **different anchors**. With `window = 16384`, the realised span is
`[start − 9600, start + 17792)` = **27 392 frames ≈ 570 ms** (≈43 % too wide), centred at
`start + 4096` — i.e. **~85 ms *before*** the true window centre (`start + 8192`). The upper bound
already equals the correct centred upper bound; only `mom_lo` is wrong, low by exactly `window/2`.

Why it matters for Phase B: `loudness_key` is the **universal stratification key** for all five axes
(`from_parts` keys loud/body off it) **and** is what §7.3 tells you to reconstruct momentary‑PSR
from (`20·log10(sample_peak) − loudness_key`). The off‑centre span means a window inherits loudness
from ~85 ms of its pre‑roll, silently weakening the very property the design claims. Nothing pins
the span today (tests assert only finiteness/ordering), which is why it passed.

**Fix** (anchor both bounds on the window centre):
```rust
let half   = mom_frames / 2;
let center = start + window / 2;
let total  = samples.len() / channels;
let mom_lo = center.saturating_sub(half);
let mom_hi = (center + half).min(total);
```
Yields a true 400 ms centred span. Add a unit test pinning `mom_hi − mom_lo == mom_frames` and
`midpoint == start + window/2` for an interior window so the definition can't silently drift again.

### F2 — Medium — Mono track poisons `stereo_correlation` strata with NaN
**`src-tauri/src/deep_analysis.rs`, `axis_strata` (lines ~92‑134) + `from_parts` (lines ~177‑203).**

`axis_strata` excludes a window only when its **loudness key** is non‑finite
(`.filter(|(_, k)| k.is_finite())`) — it never checks the **value**. The means are f64 accumulations,
so a NaN value propagates to NaN. For a **mono** source, `measure_window` sets
`stereo_correlation = f32::NAN` for every window while `loudness_key` is finite ⇒
`DeepAnalysis.stereo_correlation.{whole, loud, body}` all become **NaN**. (`dispersion` escapes
because `from_parts` overwrites it with `fisher_z_iqr`, which drops non‑finite.)

This is **production‑reachable** — `analyze_one` passes the real `pcm.channels` through to
`scan_windows`, so a genuine mono file long enough to yield windows produces the NaN strata. It is
**untested**: the mono unit test only asserts `loudness`/`crest`; the integration tests are stereo‑
only. NaN is poisonous to downstream comparisons (every IEEE compare against NaN is false), so a
Phase B width/correlation gate (§7.2) reading these would silently mis‑route.

**Fix** (make the strata robust to NaN values, and lock it):
- Preferred: filter value‑finiteness in `axis_strata` too (drop windows whose `value` is non‑finite),
  which uniformly future‑proofs every axis; **or** special‑case correlation in `from_parts` by
  passing only finite‑corr `(val, key)` pairs.
- Then add a mono test asserting `da.stereo_correlation.whole.is_finite()` and define the mono
  contract (e.g. `0.0`) so Phase B has a defined value. Also add an `axis_strata` test covering a
  finite‑key/NaN‑value pair (the gap the current tests leave open).

### F3 — Low — `sample_peak` is the mono‑downmix peak, not per‑channel
**`src-tauri/src/deep_analysis.rs`, `measure_window`, lines ~277‑287.**

`sample_peak` is `max(|0.5·(L+R)|)`, not `max(|L|, |R|)`. The doc (§3.2) just says "linear sample
peak", which an engineer could read as the per‑channel max. For hard‑panned/decorrelated material
the mono‑sum peak sits up to ~6 dB below the channel peak.

Note: the workflow **refuted** the "this biases PSR low" claim — momentary‑PSR is self‑consistent
because `loudness_key` uses the *same* downmix, so both terms drop together. The real residual risk
is §7.4 already‑mastered stand‑down, which compares per‑stratum peak against an **absolute** full‑
scale reference; there the mono‑sum understates panned masters.

**Fix:** documentation precision (note it's the mono downmix, conservative‑low for panned material).
If Phase B wants a per‑channel peak, **add a new field** — do **not** redefine `sample_peak` to
`max(|L|,|R|)`, as that would desync it from the mono‑downmix `loudness_key` and break the currently
self‑consistent PSR.

### F4 — Low (advisory) — `SourceProfile` present does **not** imply `DeepAnalysis` present
**Gating in `src-tauri/src/analysis.rs` `analyze_one`.**

6‑band/`SourceProfile` needs `≥ 1024` frames; the deep scan needs `≥ SHORT_WINDOW (16384)`. A track
in `[1024, 16384)` frames gets `by_track = Some`, `by_track_deep = cleared`. This is correct‑by‑
spec (the maps stay in lockstep on *invalidation*; only their *contents* differ), but it's a trap
for §7.1: a Phase B consumer keying `get_deep` off "a profile exists" will get `None` for that band.

**Fix:** treat `get_deep() == None` as a **first‑class "no deep data → fall back to `SourceProfile`"**
case (the §7.1 plan already says "degrade gracefully" — just make sure the keying doesn't assume
`get().is_some() ⇒ get_deep().is_some()`). Optionally add a `~2048`‑frame fixture test asserting
profile‑present / deep‑absent to pin the gap band.

### F5 — Low — Stale band range in spec/plan
`docs/superpowers/specs/...-design.md` §4.1 ("31 bands, 20 Hz–16 kHz") and the plan's Task‑6 comment
("25 Hz…16 kHz (31 bands)") are **stale**. The code & handoff are correct: `THIRD_OCTAVE_CENTERS` is
**30 real centres (25 Hz…20 kHz) + a `0.0` padding slot at index 30**. Reconcile the two older docs
so a Phase B rollup author doesn't assume the curve stops at 16 kHz (band index 29 carries 20 kHz).

### F6 — Nit — `lufs_short_term_max` wording
Handoff §3.2 calls `AnalysisResult.lufs_short_term_max` "the true 3 s Mode::S whole‑track measure".
It actually falls back to `integrated + LRA/2` for sub‑3 s/silent clips; the **pure** Mode::S value
is the `Option` field `lufs_short_term_max_3s`. Harmless for real full‑length masters, but a Phase B
author wanting guaranteed Mode::S should use the `_3s` field. One‑line doc fix.

---

## 3. Did they prescribe the right Phase B directions? — Yes, validated.

The handoff's Phase B plan (§7) is sound and feasible against **what Phase A actually retained**.
Point by point:

1. **§7.1 Wire the gated deep read** — ✅ feasible; `get_deep` exists and is test‑only as claimed.
   Apply **F4**: treat absent deep as first‑class fallback. The TOCTOU note is accurate.
2. **§7.2 Confidence/coverage gating (anti‑homogenisation core)** — ✅ feasible: coverage is
   derivable from `windows: Vec<WindowMetrics>` and consistency from the stored per‑axis IQR
   `dispersion`. **Depends on F2** for the width/correlation axes (NaN strata would poison the gate).
3. **§7.3 PSR/crest closed loop** — ✅ the inputs are retained (no new measurement). **Depends on F1**
   — the `loudness_key` feeding PSR must be the correct centred span before you build transient
   protection on it, or the "won't crush the life out of it" guarantee rests on a mis‑defined number.
4. **§7.4 Already‑mastered stand‑down** — ✅ an interpretation of retained inputs. Mind **F3** for any
   absolute full‑scale comparison on `sample_peak`.
5. **§7.5 6‑band → 31‑band rollup (the one risky change)** — ✅ and the warning is **well‑founded**:
   I confirmed in `album.rs` that arc‑role classification (`heavy_score`/`acoustic_score`/
   `transition_score`) thresholds read `spectral_balance_6band` + `energy_density_score`, and the
   per‑character LUFS offsets span ~2 dB (`HeavyDjent +0.82` vs `Transition −1.25`). A borderline
   track's label really can flip on a 6‑band value shift, and `preset_byte_identity` SHAs do **not**
   cover album labels. So: swap the byte‑exact golden for a per‑band tolerance test **and** add the
   album label‑stability fixture, exactly as prescribed.
6. **§7.6 Calibration + owner listening** — ✅ correctly framed as an owner gate, not yours to clear.

**The DECIDE defaults** (mobile deep OFF, per‑session in‑memory cache, momentary‑per‑window key) are
all honoured in code and are cheap to flip as stated.

---

## 4. Recommended Phase B task list

**Phase B‑0 — Pre‑req cleanup (do first; behaviour‑neutral & cheap *now*, painful to debug *later*).**
1. Fix **F1** (centre the momentary span) + add the span‑bounds regression test.
2. Fix **F2** (NaN‑robust correlation strata) + add the mono‑strata + finite‑key/NaN‑value tests.
3. Reconcile docs **F5** and **F6**; add the §3.2 `sample_peak` mono‑downmix note (**F3**).
4. Re‑run all gates (the §6 lanes). These changes don't touch the 6‑band path, so the golden stays
   green; re‑pin nothing audible.

> Rationale: §7.2 and §7.3 consume exactly the `loudness_key`/`stereo_correlation` that F1/F2
> corrupt. Fixing them before wiring means you never have to debug a confidence gate or a PSR loop
> through a latent measurement bug.

**Phase B proper — follow the handoff §7 order, with the validations above:**
5. §7.1 Wire the gated deep read (`present && strength > 0`), degrade gracefully, **F4‑aware** keying.
   Keep `preset_byte_identity` green.
6. §7.2 Confidence/coverage gating — the anti‑homogenisation core. (Needs B‑0 #2.)
7. §7.3 PSR/crest closed loop. (Needs B‑0 #1.)
8. §7.4 Holistic already‑mastered stand‑down. (Mind F3.)
9. §7.5 6‑band→31‑band rollup — **its own carefully‑reviewed change**: tolerance test + album label‑
   stability fixture. Treat as the risky one.
10. §7.6 Owner calibration/listening gate (slow fixture lane, `AMS_RUN_REAL_FIXTURE`).

---

## 5. Repo state as a whole — healthy

- `main` is **in sync** with `origin/main`; working tree clean.
- **No tracked audio anywhere** (`git ls-files *.wav *.flac *.aiff *.mp3` is empty) — the private‑
  audio non‑negotiable is intact. Only one tiny tracked test artifact
  (`test-output/.../native-dialog-save-as.ams.json`), benign.
- All gates green (re‑verified, §1).
- **Housekeeping nit (not a defect):** 7 fully‑merged local branches
  (`codex/yes-master-rc-finish`, `feat/adaptive-dsp-guardrails`,
  `fix/adversarial-type-review-2026-06-03`, `fix/review-backlog-2026-05-29`,
  `iphone-native-perf-wiring`, `master`, `ui-mockup-meters-dots-reveal`) plus a large fleet of stale
  `origin/codex/iphone-*` branches could be pruned. Optional cleanup; nothing blocks Phase B.

---

## 6. Verification commands (re‑run before any Phase B merge)

```powershell
cd src-tauri
cargo test  --target-dir target/codex-rc      # golden = spectral_balance_6band_is_byte_exact_golden
cargo clippy --all-targets --target-dir target/codex-rc
cd ..
npx tsc -b --pretty false
npm test
cd apps/iphone-native/rust
cargo check --all-targets                      # ALWAYS after touching shared yes_master_lib types
```

Slow fixture lane before DSP/export merges (Phase B calibration):
```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"; cargo test --target-dir target/codex-rc; Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

---

*Bottom line: clean, honest Phase A. Land the two latent‑math fixes (F1, F2) as Phase B‑0 before you
wire consumption, follow the handoff's §7 plan (it's correct), and treat the 6→31 rollup as the one
genuinely risky change. Nice work on the byte‑exact golden — it's what made "changes nothing audible"
a checkable claim instead of a hope.*
