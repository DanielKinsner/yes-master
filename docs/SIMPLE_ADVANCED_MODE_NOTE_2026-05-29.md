# Simple / Advanced Mode Product Note

Date: 2026-05-29

## Core Idea

YES Master is already strong as a hands-on mastering workbench. The product opportunity is not to make the whole app less advanced, but to add an opinionated Simple Mode that gives users a trustworthy first-click master while preserving the full desk for people who want control.

Simple Mode should mean fewer user-facing decisions, not fewer DSP decisions.

## Simple Mode

Simple Mode should be limited to:

- Presets only, likely four:
  - Natural
  - Warm
  - Open
  - Punch or Loud
- Analysis-aware guardrails always on.
- Export profile:
  - Streaming
  - CD
  - Custom, if needed.
- Original / Mastered toggle.
- Volume Match toggle.
- LUFS Preview toggle.
- Loudness choice:
  - Low
  - Medium
  - High
- Export action.
- Clear option to bounce into Advanced Mode.

No explanation panel should be required if the analysis works. The output should simply feel tasteful and appropriate.

## Advanced Mode

Advanced Mode remains the full mastering desk:

- Current preset system and intensity.
- EQ and tone shaping.
- Width, warmth, presence/air.
- Per-band compressor.
- Delivery profile and format controls.
- Source checks, meters, export receipt, and deeper diagnostics.

Advanced Mode is for taking the wheel, not for fixing Simple Mode.

## Guardrail Model

Presets should define intent. Analysis should define fit.

Examples:

- If a source is already bright, reduce Open/Clarity/Tape/Loud air lift.
- If a source is already bass-heavy or boomy, reduce Warm/Oomph low lift.
- If a source is already very dense, soften Loud/Punch compression.
- If stereo width is already high or correlation is low, limit Spatial/Open widening.

These guardrails should apply to the automatic preset baseline in both Simple and Advanced modes. Manual Advanced edits should remain explicit user overrides.

## Product Shape

The user-facing promise becomes:

- Simple Mode: pick a sound, and YES Master fits it to the track.
- Advanced Mode: show the desk, and let the user drive.

This keeps YES Master transparent and controllable while making the first-click path feel more grounded and confident.

## Additions (Claude, 2026-05-29)

Building on the above — gaps worth capturing before we spec:

### Simple is a *view*, not a second engine
Simple/Advanced is a UI toggle over ONE signal path — same DSP, same export, same guardrails. Switching is non-lossy: Advanced tweaks persist (hidden) when you drop to Simple and reappear on the way back. This is the load-bearing de-risk: Advanced *is* the current app, so adding Simple can't break what already works.

### The "review/trust" ceremony goes away — but warnings and hard-stops stay
"No explanation panel if analysis works" means specifically: retire the **Export With Review → Adjust / Export Anyway** confirm-gate. KEEP the advisory warnings visible (always-on, both modes) and KEEP the **technical** hard-stops that block a genuinely broken export (invalid path, non-finite render, requested/rendered sample-rate mismatch). This is consistent with the non-negotiable "warnings are advisory unless the export is technically invalid" — but it removes the #1 jump-fix-queue item we just built, so it's a deliberate call, not a freebie.

### Loudness vs export profile — one model cleanup
Today `DeliveryProfile` already *owns* the target LUFS, so "Loudness Low/Med/High" and "Export profile" overlap (the master review flagged this). Decouple for Simple: **Loudness Low/Med/High = how loud** (the target, ≈ −16 / −14 / −10ish, TBD) and **Export profile = where it's going** (sample rate, bit depth, ceiling). Because loudness is its own control, the four presets are **tone/character only** — no "Loud" tile needed (its loudness comes from the dial).

### Preset naming layer
Simple can present friendly tone names (Warm / Balanced / Open / Punch, à la LANDR) that *map* to internal presets (Universal/Clarity/Warmth/…). Advanced keeps the real names + all 8 + Custom.

### "Analysis defines fit" has tiers — pick v1's tier
Codex's guardrail model is the right idea; naming the tiers lets us scope:
- **Tier 0** — fixed presets, no adaptation (just a clean Simple view).
- **Tier 1 (defensive)** — analysis *trims* a preset's moves when the source already has that quality (already-bright → less air, already-dense → less compression). Cheap; uses analysis we already compute (`spectral_balance_6band`, DR/crest, stereo correlation); directly fixes the "preset over-cooks this track" feeling.
- **Tier 2 (corrective)** — analysis actively moves the source *toward* a reference curve (the LANDR closed loop). Bigger.

Recommendation: **Simple v1 = the view + Tier 1**, over the current engine (loudness landing is already adaptive). **Tier 2 is the v2 ambition.** Simple does not need Tier 2 to ship.

### What LANDR's flow confirms
Their screenshots (Analyzing → Detecting genre → Detecting sub-genre → Building custom plugin chain → a 3-way Style + 3-way Loudness UI) show the dead-simple front is *enabled by* heavy analysis — the adaptive chain IS the product. Takeaways: (1) it validates "presets define intent, analysis defines fit"; (2) **showing the analysis steps during the wait** is itself a confidence / perceived-value move worth borrowing, even if our analysis is faster.

### Open decisions for the brainstorm
1. v1 adaptiveness tier (Tier 1 recommended).
2. The four presets + their Simple-facing names.
3. Loudness↔profile decoupling — confirm.
4. Scope of removing the review ceremony (confirm warnings + hard-stops stay).
5. Default mode (Simple for new users?) and whether the choice is remembered.
6. Whether Simple shows a brief, visible "analyzing / fitting to your track" moment, LANDR-style.

