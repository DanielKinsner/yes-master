# YES Master — Engine Reference: Presets & Adaptive Processing

*Plain English + the actual math. Written 2026-06-08, cross-checked against the code and the reference-tuning evidence run of the same date.*

> One-paragraph version: **Presets** stamp a tonal/dynamic "shape" on the source. **Intensity** scales how hard that shape is applied. The **adaptive engine** doesn't touch your audio's tone — it only *reins in the preset* when the source already has too much of something (too bright / boomy / dense / wide), so a preset can't overcook a track it doesn't suit. Your **manual EQ and advanced controls are never touched** by any of this — they're added on top, and they're law. Everything is **reduce-only and floored**, so a preset always stays recognizably itself.

Source of truth in code: `src-tauri/src/dsp.rs` (chain + presets), `src-tauri/src/guardrails.rs` (adaptive Tier-1), `src-tauri/src/confidence.rs` (Tier-2 Phase B), `src-tauri/src/reference_tuning.rs` (verification). Visual preset table: `docs/preset-reference.html`.

---

## 1. The signal chain (order of operations)

A source runs through these stages, in order:

1. **Input gain** (user, default 0 dB)
2. **Subsonic high-pass** — per-preset, 22–30 Hz, 4-pole. Clears rumble below useful bass.
3. **7-band tonal EQ** — the musical EQ. Frequencies: **80 Hz** (sub) · **200 Hz** (low shelf) · **400 Hz** (low-mid) · **1.5 kHz** (mid) · **3.5 kHz** (high-mid) · **6 kHz** (air shelf) · **12 kHz** (sparkle shelf). Each band = `preset move (scaled by Intensity, possibly trimmed by the guardrail) + your manual move`.
4. **Harmonic saturation** — per-preset tanh drive (warmth).
5. **3-band compressor** — per-preset threshold/ratio/attack/release, engagement scaled by the density macro (and trimmed by the guardrail on dense sources).
6. **Stereo width** — per-preset M/S widener (trimmed toward neutral by the guardrail on already-wide sources).
7. **Output gain** (user, default 0 dB)
8. **True-peak limiter** — lookahead brickwall at the delivery ceiling.
9. **Loudness landing** — the Delivery Profile owns the final integrated-LUFS target (e.g. Streaming = −14 LUFS).

---

## 2. Presets — the "shape"

A preset is a fixed table of baseline moves (`PresetCalibration` in `dsp.rs`). Cheat-sheet of the EQ bands that differ (dB):

| Preset | Low 200 | Low-Mid 400 | Mid 1.5k | Air 6k | Width | Ratio | Target LUFS |
|---|---|---|---|---|---|---|---|
| Universal | +0.3 | −0.1 | 0 | +1.1 | 1.04 | 1.45 | −14.0 |
| Clarity | +0.6 | −1.0 | −0.8 | +1.7 | 1.02 | 1.45 | −13.4 |
| Tape | 0 | +0.3 | −1.4 | +1.9 | 0.99 | 1.65 | −13.8 |
| Spatial | +0.1 | −0.8 | −0.3 | +1.3 | **1.16** | 1.8 | −13.1 |
| Oomph | **+4.2** | **−3.0** | −2.6 | −1.2 | 0.95 | 1.7 | −12.0 |
| Warmth | +0.8 | +0.7 | −1.8 | −0.8 | 0.98 | 2.0 | −14.7 |
| Punch | +0.8 | −1.8 | +1.6 | +0.8 | 1.04 | 2.8 | −10.9 |
| Loud | +0.4 | −1.6 | +1.8 | +1.2 | 1.03 | 3.5 | −10.4 |
| Custom | 0 | 0 | 0 | 0 | 1.0 | 1.8* | −14.0 |

*Custom's compressor is present but density defaults to 0 (off) until you dial it. Full values + sub/high-mid/sparkle (all 0 on every preset) are in `preset-reference.html`.*

### How the presets were built
The four "main" presets — **Universal, Clarity, Oomph, Tape** — were **reverse-engineered to match a set of website reference masters**. The method (`reference_tuning.rs`): take the unmastered original, master it with the reference site's preset, and measure that result's spectral balance / loudness / dynamic range. Then hand-tune our `PresetCalibration` values until *our* render of the same source lands on the same numbers. The presets are not arbitrary taste — they're fits to measured targets.

**Verification (2026-06-08 run):** three of four presets land within ~0.03 spectral-share of their reference on every band. Oomph is the loosest (it actually *under*-shoots the reference's low end by ~0.115 share — the website's Oomph is even more dramatic than ours). The consistent −2 to −4 dB loudness gap is intentional (we target −14 LUFS; the site exports run hot at −10 to −12).

---

## 3. Intensity — how hard the shape is applied

```
preset_scale = 0.4 + 1.2 × intensity      (intensity ∈ [0,1])
```

- Intensity **0%** → ×0.4 (40% of the preset move)
- Intensity **50%** (default) → ×1.0 (the preset exactly as tabled)
- Intensity **100%** → ×1.6 (160% of the preset move)

Every preset EQ band is multiplied by `preset_scale` before anything else. So Intensity is a continuous dial over a preset's whole character — e.g. Oomph at 40% intensity is roughly a +1.7 dB low push instead of +4.2.

---

## 4. Manual controls are law

```
effective_band_dB = trim( preset_band × preset_scale )  +  your_manual_eq_dB
```

The `+ your_manual_eq_dB` (the Tone Shape knobs and the Visual EQ nodes) is added **after** the preset move and the guardrail trim, and is **never** scaled or trimmed by the adaptive engine. If you boost a band, it boosts — the adaptive system will not fight or undo your explicit choice. (Test: `guardrails_preserve_explicit_user_overrides`.)

---

## 5. The adaptive engine (Tier-1 guardrails)

### What it is, in plain English
The source is analyzed once at import. If it's **already** bright, boomy, dense, or wide, the guardrail **reduces the preset's matching move** so the preset doesn't pile onto a problem the source already has. It is **defensive only**: it pulls back the *preset's* lift, it never cuts the source's own tone, and it never touches your manual EQ.

Four independent axes:

| Axis | Triggered by (from analysis) | Acts on |
|---|---|---|
| **Brightness** (Highs) | presence + air share is high | the preset's 3.5k/6k/12k boosts |
| **Boominess** (Lows) | sub + low share is high | the preset's 80/200 Hz boosts |
| **Density** (Comp) | dynamic range is low (crushed) | the preset's compression amount |
| **Width** | L/R correlation is low (very wide) | the preset's stereo widener |

The detection uses a coarse **6-band** spectral read (sub 20–80, low 80–250, low-mid 250–800, mid 800–2500, presence 2500–6500, air 6500–16k Hz) — deliberately coarse, because the trim only needs to know "which broad region is hot."

### The math (exact, with the real constants)

Each axis computes an **excess** ∈ [0,1] (how far past its threshold the source sits), turns it into a **multiplier** ≤ 1.0, and applies that to the preset move. `strength` is the Adapt Strength slider (0–1, default 0.5); `confidence` defaults to 1.0 (see §6).

**Brightness:**
```
measured  = presence_share + air_share
excess    = clamp01( (measured − 0.30) / 0.12 )          // BRIGHT_DEADBAND, BRIGHT_EXCESS_FULL
bright_mult = 1 − min( excess × strength × confidence , 0.50 )   // EQ_CAP = 0.50
```

**Boominess:**
```
measured = sub_share + low_share
excess   = clamp01( (measured − 0.42) / 0.15 )           // LOW_DEADBAND, LOW_EXCESS_FULL
low_mult = 1 − min( excess × strength × confidence , 0.50 )
```

**Density** (uses the *more* dense of two readings — P95–P10 dynamic range, and EBU LRA):
```
dr_raw   = clamp01( (8.0 − DR_dB)  / (8.0 − 3.0) )       // descending: 0 above 8 dB, 1 below 3 dB
lra_raw  = clamp01( (6.0 − LRA_LU) / (6.0 − 3.0) )       // 0 if LRA unknown
density_raw  = max(dr_raw, lra_raw) × confidence
density_mult = 1 − min( density_raw × strength , 0.60 )  // DENSITY_CAP = 0.60
```

**Width** (mono never trims):
```
width_raw  = clamp01( (0.50 − correlation) / (0.50 − 0.20) ) × confidence   // descending
width_mult = 1 − min( width_raw × strength , 0.70 )      // WIDTH_CAP = 0.70
```

### Three rules that keep presets recognizable

1. **Reduce-only.** Multipliers are ≤ 1.0. The guardrail can shrink a preset's boost; it can never add a cut or grow a move. (Width only ever moves *toward* 1.0, never below.)
2. **Caps.** Each axis can remove at most: EQ **50%**, density **60%**, width **70%** of the preset's move — at *any* strength. So even on the worst-case source, a preset keeps at least half its tonal character.
3. **Character floor (EQ only).** A positive EQ boost is never trimmed below **+0.5 dB**. A preset boost that's already ≤ 0.5 dB is left untouched entirely.

> **Worked example — why Universal reads "−0%" on a boomy track.** Universal's low boost is +0.3 dB. That's below the +0.5 dB floor, so it's *untrimmable* — even on a clearly bass-heavy source the guardrail leaves it alone, and the readout shows Lows −0%. Oomph's +4.2 dB low, by contrast, has lots of headroom: on the boomiest source it can be pulled to at most +2.1 dB (the 50% cap), never lower. That's why Oomph is the one preset where the low guardrail visibly works.

### Two things it deliberately does *not* do
- It does **not** react to the live, post-EQ signal — it decided from the source analysis at import. Cranking your manual EQ doesn't change what it "sees."
- It does **not** cut the source's own frequencies. It only reins in the *preset*. (Corrective, source-aware cutting is a future opt-in feature — see §8.)

---

## 6. The confidence layer (Tier-2 Phase B)

A finer **31-band**, per-window analysis produces a per-axis **confidence** = `coverage × consistency` ∈ [0,1]:
- **coverage** — what fraction of the track shows the trait,
- **consistency** — how steady it is (scattered/fluky → low).

Confidence only ever *shrinks* a Tier-1 trim (`raw × confidence`, never grows it). Its whole job is anti-homogenization: act strongly only where a trait is **broad AND steady**, back off on neutral/scattered material so presets stay distinct.

**It is OFF by default.** With gating off, confidence = 1.0 and the engine is pure Tier-1 (the math above). It only engages for a calibration session (`YES_MASTER_CONFIDENCE_GATING=1`). The 31 bands influence your output **zero** until then.

---

## 7. Verification — how we know the math is right

- **`preset_signature`** (hard pass/fail): pushes test signals through each preset and asserts the EQ *shape* matches the calibration (the right bands move the right way). Includes a `dump_observed_tilts` diagnostic that prints each preset's actual measured band gains.
- **`preset_distinctness` / `preset_loudness_balance`** (hard pass/fail): presets are audibly distinct and loudness-balanced.
- **`reference_tuning`** (measurement, not pass/fail): renders our chain on the website's source and reports the spectral/LUFS/DR **gap vs the website's reference masters** — the "did we reproduce the target" evidence.

As of 2026-06-08 all hard tests pass and the reference gaps are small (§2).

---

## 8. What's intentionally NOT built (and why)

- **No source-tone cutting based on analysis.** By design. The engine protects against the *preset* overcooking, not against the source. A mastering tool that silently EQ'd your audio based on analysis would homogenize masters and override the engineer.
- **Corrective / reference-matching (Match-EQ style)** — derive an EQ from a reference master or a target curve and *apply* it — is a **future, opt-in Tier-2 feature**. Groundwork (deep analysis, confidence, reference-tuning measurement) is in place; the apply-a-target-curve step is not wired. It must be opt-in, never automatic.

---

## 9. The takeaway for tuning

Because everything is reduce-only, floored, capped, and (for manual EQ) untouched, the safe levers are:
- **Intensity** — scale a preset 0.4×–1.6× continuously.
- **Adapt Strength** — how hard the guardrails protect (0 = preset at full force, ignore source).
- **Manual EQ / Advanced** — full override, never second-guessed.

So with a human listening, the outcome space is effectively unbounded. The preset values and the guardrail deadbands are the only "by-ear" numbers left to lock; both are flagged PROVISIONAL in code and gated behind a listening pass — they are not changed without ears.
