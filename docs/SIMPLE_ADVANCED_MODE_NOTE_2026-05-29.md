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

