# Handoff — Adaptive DSP Tier‑2 **Phase B (Confidence Gating + 31-Band Input)**

Audience: a fresh Claude/agent (or Dan) picking up the adaptive/"smart" DSP for YES Master in a
new window. This is **self‑contained** — read this and you can continue without re‑reading the
session history. It covers what shipped, the **current state**, the **one gate that's left**
(by‑ear calibration), and the concrete next steps.

> **On `main` and pushed (2026-06-05).** The Phase B confidence-gating machinery is built and wired,
> Codex follow-up fixed the remaining traceability plumbing (`24a51c3`, `09979ad`), and the owner-
> approved 31-band adaptation input is now implemented (`aeefee6`). Desktop + iPhone also have
> truthful staged processing copy (`2683f42`, `611f5ff`). The gate is still **OFF by default**, so
> `main` behaves like the validated Tier-1 voicing unless the owner explicitly enables
> `YES_MASTER_CONFIDENCE_GATING` / `set_confidence_gating`. Enabling it remains a one-flag,
> owner-by-ear decision (see §3).

---

## 0. One‑paragraph orientation

YES Master has a three‑phase adaptive program. **Tier‑1** (shipped earlier): presets *defensively
trim* toward neutral when the source already has a quality (bright/boomy/dense/wide); one **Adapt
Strength** dial (`0..1`, default **0.5**). **Tier‑2 Phase A** (shipped 2026‑06‑03): a backend‑internal
`DeepAnalysis` — a 31‑band tonal curve + an ordered per‑window time series + loudness‑stratified
aggregates — computed per track. **Tier‑2 Phase B** (this work): *consume* that DeepAnalysis to make
the Tier‑1 trims **confident, surgical, and non‑homogenizing**. The Phase B "anti‑homogenization core"
is implemented end‑to‑end and now uses per-window 31-band harsh/sibilant/air/tilt detail where the
old confidence prototype used a coarse tonal proxy, but it remains **off by default** pending owner
by‑ear calibration. "A measures, B decides" — and B is now built, just not yet *tuned*.

---

## 1. TL;DR — exactly where it stands

- **Phase A:** complete, on `main`. See `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2_PHASE_A_COMPLETE.md`.
  Two latent‑math bugs found in a review this session and **fixed**: F1 (momentary‑loudness span was
  ~570 ms off‑center, now a true 400 ms centered span) and F2 (mono tracks NaN‑poisoned the
  `stereo_correlation` strata, now value‑finite‑filtered).
- **Phase B §7.1 (wire) + §7.2 (confidence gating):** **built and on `main`, gated OFF.**
  - `confidence.rs` derives, per Tier‑1 guardrail axis (brightness, low/boom, density, width), a
    **coverage** (fraction of loudness‑finite windows exhibiting the trait) × **consistency**
    (`1 − dispersion/scale`) = **confidence** in `[0, 1]`.
  - Bright/low confidence now consumes per-window 31-band detail from `deep_analysis.rs`: harsh
    (~2-5 kHz), sibilant (~5-9 kHz), air (~10-16 kHz), low, and tilt. Airy sources can back off
    brightness confidence instead of being treated as harsh by a broad high-band bucket.
  - `SourceGuardrails::compute_with_confidence` scales each defensive trim by its axis confidence,
    **reduce‑only** (confidence can only *shrink* a trim, never add one). Low confidence ⇒ smaller
    trim ⇒ presets stay distinct (the anti‑homogenization fix). Full confidence reproduces Tier‑1
    **byte‑for‑byte** (`raw * 1.0 == raw`).
  - The chain resolves a `Confidence` from the cached DeepAnalysis at every entry point (render
    preview/master, both live‑audition paths, `guardrail_readout`, the iPhone bridge, the headless
    fixture/reference lanes) via `apply_resolved_confidence`.
- **The owner‑calibration gate is now RUNTIME‑toggleable** (Codex P1): `CONFIDENCE_GATING` is an
  `AtomicBool` (default `false`). Flip without a rebuild via the `set_confidence_gating` Tauri
  command (`api.setConfidenceGating(true)`) or the `YES_MASTER_CONFIDENCE_GATING=1` env seed.
- **All confidence constants are PROVISIONAL.** They have **no audible effect** until the gate is on
  AND Adapt Strength > 0. The owner locks them by ear (§3).
- **Verification:** re-run the §8 lanes on the current HEAD. The original Claude handoff was green at
  `4210f48`; Codex follow-up added targeted green checks for export-report confidence plumbing and
  active-adaptation-only digest recording before updating this doc.

---

## 2. The story (two adversarial reviews drove this)

1. **My Phase A review** (`docs/reviews/2026-06-03-tier2-phase-a-review-and-phase-b-tasklist.md`):
   verified Phase A is honest, found F1/F2 (fixed), validated the Phase B plan, and prescribed a
   "Phase B‑0" (fix F1/F2 first, since §7.2/§7.3 consume exactly those values).
2. **Codex's review** (`docs/reviews/2026-06-04-adaptive-deep-analysis-adversarial-review.md`):
   reviewed the Phase B commits. **Essentially fully cleared this session** — see §4.

Both reviews converged on: *the mechanism is sound; don't enable it until the validation lanes,
the stereo/low edge cases, and the runtime flag are squared away.* They now are.

---

## 3. ⭐ THE ONE GATE LEFT: by‑ear §7.2 calibration (owner — not the agent's to clear)

Everything downstream waits on this. The confidence machinery is provisional until the owner A/Bs it.

**To enable + calibrate (no rebuild needed):**
1. Turn the gate on: `api.setConfidenceGating(true)` from the FE/devtools, **or** relaunch with
   `YES_MASTER_CONFIDENCE_GATING=1`. (Query state: `api.confidenceGatingEnabled()`.)
2. A/B **Adapt Strength 0 vs 0.5** on **bright / dense / wide** sources (should still adapt where the
   quality is broad AND consistent) vs **neutral / scattered** sources (should now *back off* and keep
   presets distinct — the homogenization fix). Use the slow fixture lane with private audio:
   `cd src-tauri; $env:AMS_RUN_REAL_FIXTURE = "1"; cargo test --target-dir target/codex-rc`.
3. Tune the **provisional constants** — all in `src-tauri/src/confidence.rs` (single source of truth):
   coverage thresholds (`BRIGHT_PROBLEM_WINDOW_SHARE`, `LOW_31_WINDOW_SHARE`, `CREST_DENSE`, the
   reused `WIDTH_CORR_WIDE`) and dispersion scales (`*_DISP_FULL`). Lock them by ear.
4. **Traceability for the calibration session (already built, Codex P3 #6):**
   - `GuardrailReadout.confidence` carries the live per‑axis `{coverage, consistency, confidence}`
     (validate *why* an axis acted, by eye).
   - `RenderedMeasurements.confidence_digest` records a one-line `"bright 0.85 / low 0.20 / ..."` and
     `ExportReport.confidence_digest` preserves it through export-check/report plumbing. It is
     recorded only when a source profile exists, confidence resolved, and Adapt Strength is > 0.
   - ⚠️ **These are exposed on the wire but NOT rendered in any everyday-user UI element** — owner
     decision (2026‑06‑04, reaffirmed 2026‑06‑05): keep confidence backend/devtools-only. Inspect via
     the readout response / devtools during calibration.

Until the gate flips on, **`main` is the Tier‑1 voicing the owner already trusts.**

---

## 4. Codex review status (2026‑06‑04) — what's cleared, what remains

| Finding | Status |
|---|---|
| P1 — runtime flag (vs compile‑time const) | ✅ `23ae132` + `ee8672a` (AtomicBool + command + env seed + pure `resolve_source_confidence`) |
| P2 — width false‑negative on side‑heavy/anti‑phase stereo | ✅ `80a53d7` (width coverage gates on stereo content, not mono loudness) |
| P2 — low/boom coverage vs consistency window mismatch | ✅ `a671049` (low dispersion uses the loudness‑finite set) |
| P2 #1 — evidence lanes validate the wrong chain once gated on | ✅ `33d1133` (fixture/reference lanes analyze `deep=true` + resolve confidence like the app) |
| P3 #6 — confidence traceability | ✅ `49d78f6` + `4210f48` + `24a51c3` + `09979ad` (readout, receipt digest, ExportReport propagation, active-adaptation-only digest) |
| P3 #7 — backend cache not evicted on track removal | ✅ `fcfa1d5` (`evict_source_profile`, FE‑wired) |
| (iPhone parity) | ✅ `a5bbc0f` — iPhone now runs the deep‑capable path + resolves adaptive context like desktop (⚠️ reverses the old "mobile = deep OFF" default; confidence still gate‑respecting there) |
| P3 — stale wiring comments + whitespace | ✅ `25f2957` (whitespace item was a **false positive** — autocrlf/CRLF, committed blob is clean LF) |
| **P2 #3** — does the **31‑band** curve feed adaptation, or stay Phase‑C readout? | ✅ `aeefee6` — 31-band per-window detail feeds Phase-B confidence now |
| **P2 #6** — bright/low per‑window tonal uses the coarse 3‑band `compute_spectral_balance` (44.1k‑referenced SR skew), not the precise 31‑band | ✅ `aeefee6` — bright/low confidence uses 31-band low/harsh/sibilant/air/tilt metrics with focused tests |
| Premium progress UI | ✅ `2683f42` desktop + `611f5ff` iPhone. Truthful timed stage copy, not backend telemetry. |

### Still Open After Codex Follow-up

These are the threads I did **not** fix in the follow-up commits:

1. Owner by-ear calibration of the Phase B gate and constants. Do not call Phase B tuned until this
   happens against private audio.
2. PSR/crest closed-loop transient protection (§7.3): still planned, not built.
3. Holistic already-mastered stand-down (§7.4): still planned, not built.
4. 31-band rollup swap (§7.5): risky future change; needs tolerance tests plus album label-stability
   coverage before touching the current 6-band golden path.
5. Backend event telemetry for processing progress: not built. The shipped desktop/iPhone staged UI is
   timed against the active analysis/render task and uses truthful labels, but it is not per-stage
   backend progress.
6. Confidence UI surface: intentionally **not** built. Confidence remains backend/wire-only unless the
   owner asks for a visible calibration or summary surface.

---

## 5. Architecture / where things live (Phase B)

- **`src-tauri/src/confidence.rs`** — the whole Phase B confidence model + **the gate**:
  - The runtime gate: `CONFIDENCE_GATING: AtomicBool` + `is_confidence_gating_enabled()` /
    `set_confidence_gating_enabled()` / `init_confidence_gating_from_env()`; the
    `set_confidence_gating` / `confidence_gating_enabled` **Tauri commands**.
  - `resolve_source_confidence(deep, album, gating_enabled) -> Option<Confidence>` — the **pure**
    decision (flag is an explicit param, so both gate states are deterministically testable with no
    global mutation; the app passes `is_confidence_gating_enabled()`).
  - `Confidence { bright, low, density, width: AxisConfidence }`, `AxisConfidence { coverage,
    consistency, confidence }`, `Confidence::{full, from_deep, digest}`. **All provisional constants
    live here.** (`Confidence`/`AxisConfidence` derive `Serialize` for the readout; the
    `AdvancedSettings.source_confidence` field is `#[serde(skip)]` so they stay off the wire there.)
- **`src-tauri/src/deep_analysis.rs`** — per-window 31-band detail input:
  `WindowMetrics` carries `low_31`, `harsh_31`, `sibilant_31`, `air_31`, and `tilt_31`, measured via
  a lightweight per-window FFT and one-third-octave buckets.
- **`src-tauri/src/guardrails.rs`** — `SourceGuardrails::compute_with_confidence` (the reduce‑only
  seam; `compute` delegates with `Confidence::full()`); `GuardrailReadout.confidence` (the readout
  surface); `WIDTH_CORR_DEADBAND` is `pub(crate)` (reused by confidence's per‑window "wide" test).
- **`src-tauri/src/profile_store.rs`** — `apply_resolved_confidence` (delegates to
  `resolve_source_confidence` with the runtime gate); the dual single‑lock map (`by_track` +
  `by_track_deep`); `evict` / `evict_source_profile`.
- **`src-tauri/src/dsp.rs`** — the chain seam: `ChainCoeffs::from_settings` reads
  `settings.advanced.source_confidence` (`unwrap_or_default()` → `full()` when `None` → byte‑identical).
- **`src-tauri/src/engine.rs` / `audio.rs`** — every chain entry resolves confidence:
  `render_track_preview`/`render_track_master`, both live-audition paths, and the
  `RenderedMeasurements.confidence_digest` record (`engine.rs`). The digest records only when the
  adaptive path is active (source profile present + confidence present + strength > 0).
- **`src-tauri/src/lib.rs`** — `run()` seeds the gate from env at startup; the two new commands are in
  the invoke handler.
- **`apps/iphone-native/rust/src/lib.rs`** — bridge resolves adaptive profile + confidence via a
  `NativeAdaptiveContext` (gate‑respecting); seeds env in `export_settings_for_options_with_context`.
- **`src-tauri/src/fixture_matrix.rs` / `reference_tuning.rs`** — the private evidence lanes now
  analyze the source `deep=true` + resolve confidence like the app, and seed the env gate.
- **`src/bindings.ts` / `src/lib/api.ts` / `src/hooks/useTrackMaster.ts`** —
  `AxisConfidence`/`Confidence`/`GuardrailReadout.confidence`,
  `RenderedMeasurements.confidence_digest`, and `ExportReport.confidence_digest`; the hook preserves
  the digest through `runExportChecks`. `api.setConfidenceGating` / `api.confidenceGatingEnabled`
  toggle/query the runtime gate. `useTrackMaster` also owns the desktop staged analysis progress
  labels returned to the UI.
- **`apps/iphone-native/YESMasterNative/AuditionController.swift` / `ContentView.swift`** — iPhone
  staged analysis/render labels and progress bar state. This mirrors the desktop product choice
  without exposing confidence.

---

## 6. Next milestones (after calibration)

The audible feature steps are all **calibration‑gated** (provisional until the owner locks numbers):

1. **Calibrate §7.2** (owner, §3) — the prerequisite for everything below.
2. **§7.3 PSR/crest closed loop** — honest transient protection from the retained per‑window
   `sample_peak` + `loudness_key` (now correct after F1). If transients are healthy, don't compress.
3. **§7.4 holistic already‑mastered stand‑down** — combine per‑stratum peak + integrated LUFS + crest
   into an "already mastered" interpretation → minimal/zero trims. (Mind F3: `sample_peak` is the
   **mono‑downmix** peak, conservative‑low for hard‑panned material — see the Phase A review.)
4. **§7.5 6‑band → 31‑band rollup (the one risky change)** — when the chain moves to a 31‑band‑derived
   tonal read the 6‑band values shift slightly. You MUST: swap the byte‑exact 6‑band golden for a
   per‑band tolerance test, **and add an album label‑stability fixture** — `album.rs` arc‑role
   classification thresholds read `spectral_balance_6band` + `energy_density_score`, and per‑character
   LUFS offsets span ~2 dB, so a borderline track's label can flip (byte‑identity does NOT cover this).
5. **Phase C** — backend-driven progress telemetry + a curated `AnalysisSummary` on the wire (a few
   flags + a plain "what we found" summary). The desktop/iPhone staged UI copy is already in place,
   but it is not backend event telemetry. The `GuardrailReadout.confidence` surface is a starting point.

---

## 7. Gotchas (don't relearn these the hard way)

- **Phase B is OFF by default and behaviour‑neutral.** `resolve_source_confidence` returns `None`
  unless the gate is on; `None` → `full()` → byte‑identical Tier‑1. The 9 `preset_byte_identity` SHAs
  and the 6‑band golden are the tripwires.
- **Confidence is reduce‑only.** It can only shrink a Tier‑1 trim, never add/flip one. Worst case it
  under‑adapts; it can't overcook a master. Safe to ship provisional.
- **Constants are provisional → owner‑locked by ear.** Don't treat the current numbers as tuned.
- **Don't expose confidence in the FE UI yet** (owner decision, 2026‑06‑04). It's backend/wire‑only.
- **Runtime gate, not compile‑time.** Use `is/set_confidence_gating_enabled()` and the
  `YES_MASTER_CONFIDENCE_GATING` env / `set_confidence_gating` command. (The old
  `CONFIDENCE_GATING_ENABLED` const is gone.) Test the gate‑ON path via `resolve_source_confidence(..,
  true)` — do **not** mutate the global in tests (it races parallel tests that assert the default).
- **iPhone now runs DeepAnalysis** (battery cost) for desktop‑parity adaptation — a real product call
  (`a5bbc0f`), reversing the old mobile‑deep‑OFF default. Confidence stays gate‑respecting there.
- **Don't DRY `compute_spectral_balance_6band` / `_31band`** (6‑band is golden‑pinned).
- **Use `--target-dir target/codex-rc`** for cargo when the app might be running. **LF→CRLF** commit
  warnings are benign (autocrlf=true; `git diff --check` will *false‑positive* on the CR — the
  committed blob is clean LF).

---

## 8. Verification (exact lanes to run on current HEAD)

```powershell
# Desktop (lib + integration; golden + 9 preset_byte_identity SHAs included)
cd src-tauri
cargo test  --target-dir target/codex-rc
cargo clippy --all-targets --target-dir target/codex-rc
# Frontend
cd ..
npx tsc -b --pretty false
npm test
# iPhone bridge — now carries real adaptive logic, so CHECK *and* TEST it (CLAUDE.md updated)
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
# Slow fixture lane (before any DSP/calibration merge; needs private audio)
cd ../../../src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"; cargo test --target-dir target/codex-rc; Remove-Item Env:\AMS_RUN_REAL_FIXTURE
# To exercise the gate-ON path in any lane: set YES_MASTER_CONFIDENCE_GATING=1 first
```

Codex follow-up verification before this doc refresh:

```powershell
npx vitest run src/hooks/useTrackMaster.integration.test.tsx
cd src-tauri
cargo test --test contracts --target-dir target\codex-rc
cargo test --test contracts export_receipt_records_adaptive_traceability_b5 --target-dir target\codex-rc
cargo fmt --check
```

Final Codex verification after the follow-up commits:

```powershell
npm test                         # 161 passed
npm run build
npm run build:windows             # MSI + NSIS produced under ignored target/
cd src-tauri
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc   # 287 passed
cargo test --target-dir target\codex-rc
cd ..\apps\iphone-native\rust
cargo check --all-targets
cargo test                        # 30 passed, 1 ignored
```

---

## 9. Pointers (authoritative sources)

- **Phase A handoff:** `docs/HANDOFF_2026-06-03_ADAPTIVE_DSP_TIER2_PHASE_A_COMPLETE.md` (all the Phase A
  math + the original §7 Phase B plan).
- **Phase A review + Phase B task list:** `docs/reviews/2026-06-03-tier2-phase-a-review-and-phase-b-tasklist.md`.
- **Codex Phase B review:** `docs/reviews/2026-06-04-adaptive-deep-analysis-adversarial-review.md`.
- **Spec / plan:** `docs/superpowers/specs/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis-design.md`,
  `docs/superpowers/plans/2026-06-03-adaptive-dsp-tier2-phase-a-deep-analysis.md`.
- **This session's commits:** `git log --oneline 366235c..HEAD` (Phase A review -> F1/F2 -> Phase B
  §7.1/§7.2 -> runtime gate -> Codex review fixes -> traceability follow-up).

---

## 10. Quick‑start for the next agent

```
1. Read this doc. Confirm you're on `main` and check `git log --oneline -8` for the latest follow-up
   commits.
2. Run `npm run verify:fast` or the explicit §8 lanes — confirm all green (golden + preset_byte_identity included).
3. Phase B is BUILT but OFF. 31-band confidence input is implemented. Do NOT enable the gate by default
   or tune constants without owner listening.
4. Keep changes byte-identity-safe (gate off => Tier-1). Tests-first. Small commits. ALWAYS run the
   iPhone `cargo check --all-targets` AND `cargo test` lanes after touching shared `yes_master_lib`.
5. Do not add a FE surface for confidence unless the owner asks (backend-only for now).
```
