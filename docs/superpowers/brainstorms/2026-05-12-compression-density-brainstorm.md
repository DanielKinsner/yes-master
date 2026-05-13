---
date: 2026-05-12
topic: compression-density
phase: 12.2 (closer)
status: brainstorm complete — ready for spec
---

# Compression Density — Brainstorm

## What We're Building

A 3-band linked-stereo downward compressor wired to the existing `AdvancedSettings.compression_density` slider (currently unwired, labeled "(coming soon)"). The compressor uses Linkwitz–Riley 4th-order (LR4) crossovers at 120 Hz and 4000 Hz to split the signal into low/mid/high, applies per-band envelope-following downward compression, and recombines. Designed for a single audio-engineer user (Dan) who wants engineer-grade controls available *the moment he opens the Advanced panel*, not staged across multiple slices.

The macro slider stays as the one-knob entry point, and **12 new per-band override fields** sit alongside it in `AdvancedSettings`. `Option<f32>` semantics mean `None` lets the macro drive; `Some(v)` overrides for that band/parameter only. No mode toggle, no state-sync invariant — the `Option`'s `Some` vs `None` *is* the mode.

## Why This Approach (vs alternatives we considered)

- **Multiband (not single-band)** — Ozone/LANDR/CloudBounce consensus per research extract; reuses the low/mid/high mental model already in the EQ. Single-band rejected because Dan wants per-band character (slow low, fast high) from day one.
- **LR4 crossovers** — flat magnitude summing at band edges, textbook mastering choice. LR2 has a +3 dB summing bump; linear-phase FIR adds latency and ~3× the code.
- **120 Hz / 4000 Hz fixed** — mastering-classic split. Hard-coded constants, no new schema for crossover frequencies. Dan can rebuild the constants later if needed.
- **Position: between `presence_air` and `width`** — matches research consensus ("compression after corrective EQ, before saturation, before limiter"). Width still operates on the post-comp signal so its stereo math doesn't fight the comp's stereo linking.
- **Macro + per-band overrides in one slice (not staged)** — Dan corrected the assumption that he needs to "live with" the macro before designing per-band controls. He's an audio engineer; he needs the deeper surface *now* for his first personal album. Staging would risk demotivation.
- **Threshold-only macro mapping** — chosen over threshold+ratio because predictability matters when an engineer is reading the slider value. At density=0.5 the threshold drops exactly 12 dB across all bands; ratios are per-band fixed musical defaults.

## Key Decisions Locked

### Topology
- **3-band multiband**, LR4 crossovers (4th-order Butterworth cascade)
- **Crossover frequencies**: 120 Hz (low/mid), 4000 Hz (mid/high) — fixed constants
- **Stereo**: linked L/R per band by default; `compression_link_stereo: Option<bool>` (default `true`) exposes an unlink toggle for the rare case it's wanted
- **Detector**: peak (not RMS) — standard mastering convention, simpler, more aggressive
- **Knee**: soft 6 dB fixed per band (not user-tunable in v1)
- **Lookahead**: none (the limiter already provides lookahead; comp doesn't need it for mastering)
- **Makeup gain**: auto-calculated per band as `(threshold_drop_db × (1 − 1/ratio)) / 2` so the engineer doesn't compensate manually. Could become an override later.

### Chain position
Insertion point: between `presence_air` (high-shelf) and `width` (M/S transform):

```
input gain → low_eq → mid_eq → high_eq → warmth → presence_air → COMP (3-band) → width → saturation → limiter → VM → user output gain
```

### Macro mapping (when overrides are `None`)
- `compression_density` 0..1 → uniform threshold `0 dBFS` (no comp) to `-24 dBFS` (heavy comp), applied to all 3 bands at once.
- Per-band fixed ratios: **low 2.5:1, mid 2:1, high 1.8:1**.
- Per-band fixed attack/release: **low 30 ms / 300 ms, mid 15 ms / 150 ms, high 5 ms / 80 ms** (slow low, fast high — standard mastering musical fit).
- Linear identity early-return when `density.unwrap_or(0.0) < 1e-4` (skip the entire crossover network — byte-equivalent untouched-slider path).

### Schema additions (all `Option<f32>` / `Option<bool>`, all `#[serde(default)]`)
12 per-band override fields:
- `compression_low_threshold_db`, `compression_low_ratio`, `compression_low_attack_ms`, `compression_low_release_ms`
- `compression_mid_threshold_db`, `compression_mid_ratio`, `compression_mid_attack_ms`, `compression_mid_release_ms`
- `compression_high_threshold_db`, `compression_high_ratio`, `compression_high_attack_ms`, `compression_high_release_ms`

Plus:
- `compression_link_stereo: Option<bool>`

Existing `compression_density: Option<f32>` keeps its current type and meaning.

### v1 niceties
- **Per-band GR meter in StaleBar** — three dB readouts (Low/Mid/High GR), atomic-snapshot pattern (mirror of `ClippingIndicator`'s `Arc<AtomicU32>::fetch_max` on |reduction|, swap-and-reset on each 50 ms tick).
- **Already-compressed input advisory** — `run_export_checks` measures source DR (already in `AnalysisResult.dynamic_range_lu`); if `dr_lu < 6.0 AND compression_density > 0.3 AND no per-band overrides`, surface a `QualityCheck { level: Warning, code: "comp_density_on_compressed_source" }`. Doesn't block, just warns.
- **Stereo unlink** — exposed in AdvancedPanel as a checkbox.

### UI surface (AdvancedPanel)
- **Macro**: existing `compression_density` slider — relabel from "Compression (coming soon)" to "Compression density".
- **Per-band subsection** (collapsed by default to keep the panel scannable):
  - 3 columns (Low / Mid / High), each with: Threshold (dB), Ratio (X:1), Attack (ms), Release (ms). All `NumberField`s. Empty/blank state = "macro" (i.e., `None`).
  - "Link stereo" checkbox at the top of the subsection.
- **GR meters**: 3 small readouts in `StaleBar` alongside the existing clipping indicator — `"L: -2.3 dB | M: -1.1 dB | H: -0.4 dB"` format, green/yellow/red color bands.

## Open Questions for the Spec Author

None blocking. The spec should pin:
- Exact biquad coefficient computation for LR4 LP+HP cascades (RBJ cookbook + Butterworth Q=0.7071 cascaded twice).
- Exact envelope follower math: `env_n = max(|x_n|, alpha × env_{n-1})` with attack and release alphas computed as `exp(-1 / (time_ms/1000 × sr))`.
- Exact gain-reduction curve at the soft knee.
- Test plan: 5–8 unit tests against closed-form expectations (band split summing flat at unity, threshold knee behavior, attack/release time-constants, linked-stereo behavior, macro→per-band override fallback).
- Memory layout for the 3 per-band envelope states (`ChannelState` grows).
- Atomic-snapshot pattern for the 3 GR readouts (one `AtomicU32` per band, `fetch_max` on |reduction_db × 100| stored as integer; 0 = no reduction).

## Scope and Estimate

- Backend Rust: ~600–900 lines (3-band crossover network + 3-band detector + 3-band gain stage + makeup gain + GR snapshot wiring).
- Frontend: ~150–250 lines (12 new NumberFields, link checkbox, GR meter component, label).
- Schema: 12 + 1 new fields, all `#[serde(default)]` and `Option<_>`.
- Tests: 6–8 new unit tests in `dsp.rs::mod tests` + 1–2 new contract tests for export-check wiring.
- Verification: same triad (cargo test, cargo check --tests, npm run build); expected count 69–73/73 pass (was 61).

Exceeds the original HANDOFF "~300–500 line" estimate by ~2×. That's expected and accepted because Dan wants the engineer-grade surface available immediately — under-building risks demotivation on the personal album he's mastering.

## Next Steps

→ Write the design spec at `docs/superpowers/specs/2026-05-12-compression-density-design.md` pinning all the math and the exact field-by-field UI mapping.
→ Then write the TDD-ordered plan at `docs/superpowers/plans/2026-05-12-compression-density.md` for `/goal`-driven execution.
→ After compression_density ships, queue typography pass + SVG preset icons as the next two Phase 12-tail slices so `/goal` keeps the loop moving without re-eliciting decisions.
