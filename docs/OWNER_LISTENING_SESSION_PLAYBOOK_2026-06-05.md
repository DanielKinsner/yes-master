# Owner Listening-Session Playbook — Adaptive DSP

Date: 2026-06-05
Audience: **Dan** (the only one who can clear the by-ear gate). This is the "sit down and dig in"
procedure, not a backlog — the backlog lives in `docs/ADAPTIVE_DSP_NEXT_STEPS.md`. This doc is the
*sequence of actions + decisions* for the one listening session everything downstream waits on.

## 0. Orientation (read once, 2 min)

- **Nothing is broken or waiting in a degraded state.** `main` is the validated **Tier-1** voicing, green,
  byte-identical-safe, shippable today. Phase B (confidence gating) is **OFF by default** and reduce-only,
  so it cannot overcook a master.
- This session is about **advancing** — deciding whether to turn the next tier *on* and locking its
  provisional numbers by ear. It is a taste/identity gate, not a correctness one.
- The whole 2026-06-03 GLOBAL review is triaged and **closed on code**: 15 already-fixed, 3 stale, 0 open
  objective defects (`docs/reviews/2026-06-05-adaptive-dsp-GLOBAL-review-triage.md`). The only open adaptive
  items are the taste/owner ones below.

## 1. Pre-flight

```powershell
# from repo root — confirm a clean green baseline before you start changing numbers
npm run verify:fast
```

Gather private reference masters grouped by character (keep them under `private-audio-fixtures/`, never
committed): a few **bright**, **boomy/dense**, **wide**, and crucially a few **genuinely neutral / scattered**
ones (the homogenization fix is judged on these — they should *back off*).

## 2. The calibration procedure (Phase B §7.2)

### 2a. Turn the gate on (no rebuild needed)
- From the FE/devtools: `api.setConfidenceGating(true)` (query: `api.confidenceGatingEnabled()`), **or**
- relaunch with the env seed: `YES_MASTER_CONFIDENCE_GATING=1`.

### 2b. A/B protocol (per source)
- Compare **Adapt Strength 0 vs 0.5** with the **per-axis readout open** (Advanced card → "Adaptive trims
  (chain, pre-landing)"). The readout shows each axis's `-NN%` trim + the source context that drove it +
  the live `{coverage, consistency, confidence}` so you can see *why* an axis acted.
- **Expectation to confirm by ear:**
  - **bright / dense / wide & consistent** sources → still adapt (the quality is broad AND steady).
  - **neutral / scattered** sources → now *back off* and keep presets distinct (this is the
    anti-homogenization payoff; it's the main thing Phase B buys over Tier-1).

### 2c. The provisional constants (single source of truth: `src-tauri/src/confidence.rs`)
Lock these by ear. Current placeholders:

| Constant | Now | Governs |
|---|---|---|
| `BRIGHT_PROBLEM_WINDOW_SHARE` | `0.35` | fraction of windows that must read harsh/sibilant before bright counts as a real trait |
| `LOW_31_WINDOW_SHARE` | `0.35` | same, for low/boom |
| `CREST_DENSE` | `2.2` | per-window crest below which a window counts as "dense" |
| `WIDTH_CORR_WIDE` | `= WIDTH_CORR_DEADBAND` | correlation below which a window counts as "wide" |
| `BRIGHT_DISP_FULL` / `LOW_DISP_FULL` | `0.10` / `0.10` | dispersion at which consistency hits 0 (more scatter ⇒ less confidence) |
| `CREST_DISP_FULL` | `1.2` | same, density axis |
| `WIDTH_DISP_FULL` | `0.6` | same, width axis |

Coverage × consistency = per-axis confidence ∈ [0,1], which only *shrinks* a Tier-1 trim (reduce-only).
Raise a `_SHARE`/lower a `_DENSE` to make an axis **harder** to trigger; lower a `_DISP_FULL` to make
scatter **kill** confidence faster.

### 2d. Verify each change preserves the safety net
```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"; cargo test --target-dir target\codex-rc; Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```
The 9 `preset_byte_identity` SHAs + the 6-band golden must stay green (gate-off path is byte-identical;
gate-on is reduce-only). If you want the gate-ON evidence lane, set `YES_MASTER_CONFIDENCE_GATING=1` first.

### 2e. Decide: ship Phase B on-by-default?
Only after 2b reads right on **both** the act-on-it and back-off cohorts. **Recommendation:** keep it
off-by-default until the neutral/scattered cohort demonstrably backs off — that back-off is the entire
justification; without it Phase B is just Tier-1 with extra steps.

## 3. Owner decisions queued (each is a sentence for you; then an agent can execute)

1. **`stereo_width` co-trigger (F10).** It's computed + carried in `SourceProfile` but unread by the width
   guardrail (correlation-only today). Wire it as a side-energy co-trigger (changes sound → needs your
   ears after), or delete the inert field (contract change, no ears)? *My lean: decide direction now; if
   "wire," it belongs in the same listening pass as §2.*
2. **Density-cap shape (F7).** `DENSITY_CAP = 0.60` means a fully-dense source saturates its realized trim
   once `strength·density_raw` hits the cap. Reshape so "keep ≥40%" is honest in dB GR, or leave as-is?
   *Taste — needs a dense source in front of you.*
3. **Tilt-vs-reference brightness metric.** Replace the absolute `BRIGHT_DEADBAND = 0.30` with a
   spectral-slope-vs-pink comparison so a flat spectrum reads zero-excess by construction. The metric is
   *objective to build* (an agent can scaffold it behind the gate); only the final anchor is taste. *Say
   go and I'll build the measurement side without touching the live deadband.*
4. **`LOW_DEADBAND` / per-band EQ floors** for bass-forward genres — pure calibration, needs your ears.
5. **Total-loudness-loss budget (B3).** Owner call on whether to share a loudness-loss budget across axes
   (today only mitigated by the honest "pre-landing" readout label). Tier-2 direction.

## 4. Objective backlog (no ears — an agent can run these while you're away)

Prioritized; status as of this writing:

- **`App.tsx` mechanical extraction (#4)** — in progress (see commit log). Shrinking the 3.2k-line file by
  moving self-contained components to `src/components/`, behavior-preserving, `npm test` + build per slice.
- **`verify:frontend` / `:rust` / `:iphone` wrappers (#9)** — in progress.
- **`LRA → Option<f32>` full cleanup (#8)** — *careful*: most landed already (DR is `Option<f32>`, the LU→dB
  alias is gone, sentinels guarded). What remains is converting the `SourceProfile.dynamic_range_lu`
  `0.0`-coercion to an explicit optional **without changing density thresholds**. Behavior-sensitive — do
  it test-first, prove byte-identity.
- **`deep_analysis` per-window allocation opt (#7)** — remove the per-window mono `Vec` alloc only if output
  stays identical (6-band golden + 31-band confidence tests guard it).
- **More stable-primitive characterization tests (#6)** — `wav_writer` dither/quantization edges, limiter
  true-peak/intersample. (Done already: `unique_album_path`, `convert_interleaved`, `decode`.)

## 5. Pointers

- Phase B architecture + gate: `docs/HANDOFF_2026-06-04_ADAPTIVE_DSP_TIER2_PHASE_B_CONFIDENCE.md`.
- Backlog: `docs/ADAPTIVE_DSP_NEXT_STEPS.md`. Agent queue: `docs/AGENT_WORK_AND_REVIEW_QUEUE_2026-06-05.md`.
- GLOBAL-review triage: `docs/reviews/2026-06-05-adaptive-dsp-GLOBAL-review-triage.md`.
- Constants: `src-tauri/src/confidence.rs` (Phase B) · `src-tauri/src/guardrails.rs` (Tier-1).
- Per-axis readout: `src/App.tsx` `AdaptiveReadout` ← `guardrail_readout` command.

**Tier-2 north star (after v1 locked by ear):** measured-neutral from your own references (per-preset),
PSR/crest closed-loop transient defense, corrective target-curve. None of it should stack on the Phase B
numbers until §2 locks them.
