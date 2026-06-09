# Export Metrics & LUFS Targeting — Inquiry Findings (2026-06-09)

**Input:** `REVIEW_CHECKPOINT_export_metrics.md` (owner, 2026-06-09) — external
pyloudnorm/scipy measurement of `So_Ouch_-_Original__master.wav` (96k/32f,
Custom profile, target −9 LUFS, ceiling Auto, adapt 50%, preset comp 0.5)
vs in-app readouts. External rig verified against Source Check to 0.05 dB.

**Status:** BOTH inquiries CLOSED with definitive code answers. No bugs in
measurement, gating, or targeting math. Two real transparency gaps in what the
UI displays; remediated by the `fix/export-metrics-2026-06-09` series.

---

## Inquiry 1 — the −10.26 LUFS vs −9 target undershoot

**Verdict: intended, documented, ceiling-bounded behavior — not a targeting
bug. But the shortfall was invisible at export time.**

1. **Measurement point** (`engine.rs:728-767`): one full BS.1770 pass
   (`ebur128`, `Mode::I | LRA | TRUE_PEAK`) over the **post-chain,
   post-SRC samples** — i.e. the exact signal written to disk. The landing
   gain applied afterward is uniform, and integrated LUFS / true peak shift
   by exactly the same dB under uniform gain (LRA is gain-invariant), so the
   stored measurements are shifted mathematically instead of re-measured
   (`engine.rs:733-737`, `:785-788`). No limiter runs after the landing, so
   the math is exact, not an estimate.

2. **Integrated, properly gated** — `loudness_global()` from the same
   `ebur128` crate + mode flags that `analyze_tracks` uses
   (`analysis.rs:59`). The Source Check meter and the targeting measurement
   share one implementation; the external rig matched it to 0.05 dB.

3. **Adaptive does NOT ease off the target.** The landing delta is a pure
   function of (measured LUFS, measured TP, target, ceiling) —
   `ceiling_bounded_landing_delta_db`, `engine.rs:182-203`. Adapt strength
   never enters it. Adaptive trims shape the chain upstream, which changes
   the pre-landing crest, but there is no protect-dynamics logic in targeting.

4. **Single-pass by design** — and correct, because the only post-measurement
   operation is uniform gain (see 1). No convergence loop is needed.

**The actual mechanism** (`engine.rs:191-197`): upward landing gain is
`min(target − measured, ceiling − measuredTP)`. The landing **refuses to add
limiting or clipping to buy loudness** — it pushes until true peak reaches the
delivery ceiling, then stops. Ceiling "Auto" on the Custom profile resolves to
**−1.0 dBTP** (`types.rs:626-631`; pinned at `types.rs:1178-1188`). History:
the pre-B6 policy refused ALL upward gain; B6 relaxed it to ceiling-bounded
(`engine.rs:177-181`).

**The file's fingerprint confirms it:** delivered TP −0.94 dBTP ≈ the −1.0
ceiling (the 0.06 dB is scipy `resample_poly` vs ebur128's oversampling
estimator). The post-chain signal sat ≈ −13.5 LUFS with only ~3.2 dB of peak
headroom when 4.5 dB was needed; high-DR piano/orchestra keeps a large crest
even after the chain's gentle limiting (crest 12.2 dB delivered), so headroom
runs out 1.26 LU before −9.

**Prediction for the checkpoint's discriminating experiment:** adapt 0% +
comp OFF will undershoot **more**, not less — less compression leaves a higher
crest, so the landing hits the ceiling sooner. Both branches of the
experiment's decision tree point at the wrong suspect.

**Visibility before this series:** the WYSIWYG/Preview-LUFS path plays the
true delivered loudness (`engine.rs:335+` routes the same landing math), and
`RenderedMeasurements.lufs_integrated` carries the delivered value onto the
receipt payload (`engine.rs:816-817` → `useTrackMaster.ts:1395`) — but the
receipt card rendered only sample rate + bit depth from it, and nothing
anywhere said "target not reached." That gap is fixes (1) and (2) below.

---

## Inquiry 2 — what the Export Complete chips measure

**Verdict: the `BRIGHT 0.02 / LOW 0.37 / DR 22.2DB / LRA 13.6LU / CORR 0.74`
chip is `SourceProfile::digest()` — deliberately SOURCE-describing data
(the trigger inputs that drove adaptation), rendered without a visible
"source" label directly under output-describing chips.**

- The chip row is the B5 adaptive-traceability digest, composed dynamically in
  Rust at `types.rs:169-186`:
  `"bright {presence+air} / low {sub+low} / DR {p95−p10 dB} / LRA {LRA LU} / corr {corr}"`,
  all fields from `SourceProfile` (source analysis). The doc comment is
  explicit: *"One-line human-readable summary of the trigger inputs, for the
  export receipt's 'what adaptation produced this master' line (B5)."*
  (Dynamic composition is why a grep for the rendered labels finds nothing in
  the FE.)
- It renders at `App.tsx:2911-2913` with only a hover tooltip ("Source profile
  that drove adaptation") distinguishing it from the delivered-output chips
  (96 kHz / 32-bit float) it sits beside. Mislabeling is a fair reading.
- **True peak −0.94 in Quality notes IS the delivered output** — the
  `streaming_headroom_low` check (`exports.rs:42-49`) formats
  `report.measured_true_peak_dbtp`, which comes from `job.measurements`
  (rendered output, math-shifted post-landing). Its data path differs from the
  digest because the Codex 2026-05-13 P0 audit rewired the *measurements* to
  describe the rendered output (`engine.rs:728-731`,
  `useTrackMaster.ts:1385-1390`); the digest was always meant to be source.
- **Post-export instrumentation exists and is sufficient:** the render path
  measures integrated, LRA, and TP of the final buffer;
  `RenderedMeasurements.dynamic_range_lu` already holds the delivered LRA
  (≈10.9 for this file) — it was measured, stored, and simply never rendered.
  The only thing not measured post-render is stereo correlation (no field).
- External-vs-displayed reconciliation: displayed LRA 13.6 = source LRA
  (external source rig agrees); delivered LRA 10.9 lives unrendered on the
  payload; displayed CORR 0.74 = source correlation (external: source 0.741,
  master 0.721 — close because the chain barely touches width here).

---

## Remediation (this branch, `fix/export-metrics-2026-06-09`)

1. **Receipt renders the delivered master's numbers** — LUFS / TP / LRA from
   `job.measurements`, labeled as the master. Pure FE; data already flowed.
2. **`target_not_reached` quality note** — Info-level check when the delivered
   LUFS lands > 0.25 LU below `effective_target_lufs()`, naming the shortfall
   and the ceiling-bound reason when TP ≈ ceiling. Gated on a new
   `ExportReport.measurements_are_rendered` flag (`#[serde(default)]`) so the
   legacy source-analysis fallback (album path) can never false-fire it.
3. **The adaptive digest chip gets a visible "Source" prefix** so source-
   derived stats cannot read as master measurements.

Out of scope, deliberately: changing the ceiling-bounded landing policy
(owner-gated design decision; the checkpoint's own framing — "known,
explainable, visible" — is satisfied by 1–3), and measuring post-render
stereo correlation (no current consumer).
