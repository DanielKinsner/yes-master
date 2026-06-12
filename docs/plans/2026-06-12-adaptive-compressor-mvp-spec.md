# Adaptive Compressor — MVP Spec (2026-06-12)

Owner decision (2026-06-12): track-aware compression is **part of the MVP**.
"This product needs to compete so I don't want to launch with skipped
important layers that other high-end products have at jump." Owner commits
listening-gate time on Mac, Windows, and iPhone.

This spec is written for a non-Claude implementer (Codex GPT 5.5) under the
operating model of `docs/plans/2026-06-12-shippability-roadmap.md` — read
its global rules first, especially rule 2 (DSP byte-identity), rule 3
(tests observed failing first), and rule 7 (interrogate the owner; use
`/grill-me` where available). Supersedes roadmap item S9.10's
"post-launch Tier 3" framing.

## 1. What already exists (read before designing anything)

The compressor is ALREADY track-aware in one dimension. Do not build a
parallel system — extend this one:

- `src-tauri/src/guardrails.rs` — Tier-1 `SourceGuardrails::compute` derives
  a `density_mult` from the whole-track source profile (P95−P10 dynamics +
  LRA ramps, `:142-154`), scaled by axis confidence and Adapt Strength,
  **reduce-only by construction**, capped at `DENSITY_CAP = 0.60` (`:66`).
- `src-tauri/src/dsp.rs:973` — the user's density macro is passed through
  `guardrails.scale_density(base)` before the preset mapping at `:931-978`
  (density 0 = bypass, 0.5 = preset baseline, 1.0 = pushed ~3 dB harder).
- `src-tauri/src/deep_analysis.rs` — per-window short-window metrics
  (crest, momentary K-weighted loudness, 31-band detail) already computed;
  PSR is derivable and currently derived only in a test (`:693-706`).
- `src-tauri/src/confidence.rs` — Phase-B per-window confidence machinery,
  shipped but dormant behind `CONFIDENCE_GATING` (default false, `:40`),
  runtime-togglable via env seed + Tauri command. **This is the rollout
  pattern to clone, and this feature is its first production consumer.**
- The owner has listened to Tier-1 voicing and accepted it (2026-06-11 live
  96 kHz auditioning; see `201e746`). The current Preset-mode sound is the
  validated reference point.

What is missing vs. high-end competitors (LANDR/BandLab class):
(a) **per-band** adaptation — today's scaling is one global multiplier;
(b) **transient protection** — no closed-loop PSR/crest guard (unattended
item UF-A6); (c) **already-mastered stand-down** — no holistic backoff on
sources that are already masters (UF-A7, owner already answered "yes" to
this direction in the Phase-A handoff); (d) **visibility** — the adaptation
is silent; the card shows static "Preset" values.

## 2. Safety frame (this is why MVP is feasible)

Every adaptive output must be **reduce-only relative to the preset
baseline**: the adapted compressor is an interpolation between two
already-validated endpoints — current Preset behavior (owner-listened) and
lighter/Off (identity-safe). Adaptation may ease density, raise thresholds,
or lower ratios per band; it may NEVER exceed the preset baseline
aggression. Users who want more push use Manual or the density macro —
unchanged. This bounds the entire output space by behavior the owner has
already signed off on, which is what makes shipping it in the MVP sane.

Corollaries:
- Confidence scales adaptation toward zero (low confidence → plain preset
  baseline). Reuse the Phase-B axis-confidence shapes; do not invent a new
  confidence system.
- Mono, very short, or analysis-failed tracks → identity (plain Preset).
- `Manual` and `Off` modes are untouched. The mode enum does NOT change
  (no wire churn across the four ecosystems). Adaptation lives inside
  `Preset` mode, which is honest once the canon sentence is updated
  (roadmap S5.4 already scopes the rewrite).

## 3. Architecture

### 3.1 Per-band guardrails (new, in `guardrails.rs` or a sibling module)

Extend the guardrail computation with a per-band compression plan:

```
pub struct BandCompressionGuard {
    pub density_mult: f32,     // reduce-only, [1-BAND_DENSITY_CAP, 1.0]
    pub threshold_lift_db: f32, // >= 0, raises (softens) the band threshold
    pub ratio_mult: f32,       // (0, 1.0], eases the band ratio
}
pub struct CompressionGuards {
    pub low: BandCompressionGuard,
    pub mid: BandCompressionGuard,
    pub high: BandCompressionGuard,
    pub stand_down: f32,        // 0..1 holistic already-mastered backoff
    pub reasons: Vec<GuardReason>, // machine-readable "why" for UI/receipt
}
```

Inputs, all already measured or one derivation away:
- **Per-band PSR** from deep-analysis window detail: aggregate the 31-band
  window features into the chain's low/mid/high crossover bands (use the
  same crossover corners as `ChainCoeffs`), take low-percentile PSR (e.g.
  P10) per band as the "already dense here" signal. Promote the test-only
  PSR derivation (`deep_analysis.rs:693-706`) into a production function
  with its existing test retained.
- **Whole-track DR/LRA** ramps — reuse the Tier-1 functions, do not fork
  the constants.
- **Stand-down classifier** (UF-A7): a source is "already a master" when
  (tunable, locked at calibration): integrated LUFS hotter than ~-10,
  true peak near ceiling, LRA below ~6, AND per-band P10 PSR uniformly low.
  Output is a 0..1 factor that scales ALL adaptation toward maximum easing
  (NOT toward bypass of safety stages — limiter/ceiling/landing untouched,
  per the compressor canon).
- **Confidence**: multiply each band's adaptation by the relevant axis
  confidence exactly as Tier-1 does (`density_raw * confidence.density.confidence`
  is the template, `guardrails.rs:153`).

All constants live in one block with `// LOCKED-BY-LISTENING(date)` or
`// TBD-CALIBRATION` markers. Initial values: propose from the fixture
matrix data (already-mastered runner aggregates), mark every one
TBD-CALIBRATION.

### 3.2 Application point (in `dsp.rs`, minimal diff)

`from_settings` already resolves per-band threshold/ratio from preset +
density (`:931-1107` region). Apply `CompressionGuards` exactly where
`scale_density` applies today, but per band: thresholds get
`+ threshold_lift_db`, ratios get `* ratio_mult`, band density share gets
`* density_mult`. When the runtime gate (3.4) is OFF or guards are absent →
bit-identical to today (this is the regression contract).

### 3.3 Resolution + WYSIWYG (backend-owned, command layer)

The guards must be identical in audition and render. Follow the existing
backend-owned injection pattern used for the resolved source profile
(desktop command layer + `apply_resolved_profile` in the iPhone facade):
resolve `CompressionGuards` wherever the source profile/confidence are
resolved today, so the live chain, preview landing, master render, and the
mobile facades all inherit the same plan from the shared crate with **zero
wire-format changes**. The mobile apps gain the behavior for free; verify
with the existing bit-parity bridge tests plus one new fixture (Phase AC-4).

### 3.4 Rollout gate (clone the proven pattern)

`static ADAPTIVE_COMPRESSION: AtomicBool = AtomicBool::new(false)` in the
shared crate, with env seed (`YES_MASTER_ADAPTIVE_COMPRESSION`) and a Tauri
command, mirroring `confidence.rs:34-48` verbatim in shape. Everything in
this spec lands gate-OFF (byte-identical, all snapshots untouched —
satisfies roadmap rule 2). The default flips to ON in one final
calibration commit that also locks constants and regenerates snapshots,
citing the owner's listening evidence. That commit is the ONLY one allowed
to change DSP snapshot bytes, and it must say so.

### 3.5 UI (desktop Advanced card + Standard invisibility + receipt)

- Per-band compressor card (`AdvancedPanel.tsx` region): in Preset mode
  with active guards, show the adapted values with an "Adaptive" tag per
  band and one plain-language guidance line built from `reasons`
  ("Low band is already dense — easing compression there"). New Tauri
  command `resolve_compression_plan(track_id)` (or extend an existing
  analysis/settings query) returns the per-band effective values + reasons
  so the frontend NEVER re-implements the mapping (no TS mirror — that is
  how parity debt starts).
- Standard view: no new controls. Adaptation is part of the Standard
  promise; at most the existing insight surface gains one sentence.
- Export receipt: append the guard summary to the existing adaptive digest
  so renders are traceable ("density eased 22% low / 8% mid, stand-down
  0.4, confidence 0.9").
- Display-label question ("Preset" vs "Adaptive" as the mode pill text)
  goes to the owner in the calibration session — one-word decision,
  UI-only either way.

## 4. Phases (each is one PR; lanes per roadmap global rules)

- **AC-1 Measurement plumbing.** Per-band PSR production function +
  band rollup + stand-down classifier inputs. Pure functions, unit-tested
  against synthesized signals (dense sine vs. transient clicks vs. real
  fixture summaries). Zero behavior change anywhere. Shared-crate change →
  run BOTH mobile lanes.
- **AC-2 Guards + gated application.** `CompressionGuards` resolution +
  `dsp.rs` application behind the gate (default OFF). Regression contract
  test: gate OFF → `ChainCoeffs::from_settings` output bit-identical for a
  matrix of presets/settings (assert against today's values). Gate ON unit
  tests: dense source → reduce-only per band, never exceeding baseline;
  dynamic source → identity; low confidence → identity; stand-down source
  → maximum easing with limiter/ceiling/landing untouched. Property test:
  for random profiles, adapted aggression ≤ preset baseline in every band.
- **AC-3 Resolution command + Advanced card UI + receipt digest.**
  Frontend reads the backend plan; pinned component tests for the Adaptive
  tags, guidance line, and that Manual/Off modes hide all of it.
- **AC-4 Mobile inheritance proof.** No mobile code changes expected;
  add one bridge test per platform rendering a dense fixture with the gate
  forced ON and asserting output matches desktop bit-for-bit (extend
  `bridge_render_matches_shared_render_path`). Android lane + iPhone lane.
- **AC-5 Calibration + flip (owner session, then one commit).** See §5.
  The commit: lock constants (`LOCKED-BY-LISTENING(2026-06-..)`), flip the
  default, regenerate snapshots, re-run the slow fixture lane AND the
  already-mastered matrix with gate-on cases added, update PRODUCT.md /
  APP_BEHAVIOR.md compressor canon (coordinates with roadmap S5.4), update
  the parity fixture if any shared constant is consumed cross-language.

## 5. Calibration session (owner, ~60-90 min, Mac/Windows/iPhone)

Prep (agent, before the session): build with gate togglable at runtime;
queue the private already-mastered fixtures + 2-3 normal dynamic tracks +
one quiet/acoustic track; print the constant sheet with proposed values.

Script: for each source × {Universal, Loud, Clarity, Oomph}: A/B gate
OFF vs ON at intensity 0.5 and 1.0. Listen for: (1) already-mastered
sources — ON should be audibly more transparent (less pumping/density)
with no loudness penalty beyond the documented landing behavior; (2)
dynamic sources — ON vs OFF should be indistinguishable (identity claim);
(3) transients (Punch fixture) — ON must not soften impact MORE than OFF
(reduce-only sanity). Record per-case keep/adjust notes against specific
constants. iPhone: spot-check 2 cases through the live audition (inherits
via shared crate). Decisions to capture: constants, the mode-pill label,
and whether the Phase-B `CONFIDENCE_GATING` default also flips in the same
commit (the playbook session §2 can run in the same sitting — same
fixtures, same ears; recommend doing both, one commit each).

## 6. Out of scope (do not touch)

Limiter/lookahead behavior (RS-09 stays an owner decision); the mode enum
and wire formats; Manual/Off semantics; preset EQ voicing; Volume Match;
any new runtime dependency. The pre-launch "suggestion layer" idea from the
audit (Part 4.3a) is superseded by this spec — do not build both; the
guidance line in 3.5 delivers the explanation UX.

## 7. Sequencing vs. the roadmap

AC-1/AC-2 can start immediately after Wave 1 (they touch the same crate as
S1.1 — rebase carefully, land S1.1 first). AC-3 after Wave 2 (same hook
file). AC-5 happens whenever the owner sits down; everything before it
ships gate-OFF and is releasable at any moment. If the public push date
arrives before AC-5, the honest fallback is: ship with the gate OFF
(today's validated sound) and flip in the first update — the owner decides
at that moment, not by default.
