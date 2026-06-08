# Simple Mode — Design Spec

Date: 2026-06-08
Status: approved design, pre-implementation
Builds on: `docs/SIMPLE_ADVANCED_MODE_NOTE_2026-05-29.md`, `docs/ENGINE_REFERENCE.md`, the iPhone app (`apps/iphone-native`)

## 1. Goal & promise

A stripped, opinionated **Simple Mode** for the desktop app that gives a trustworthy first-click master with almost no decisions, riding the now-validated adaptive engine. Advanced Mode remains the full mastering desk, untouched.

- **Simple:** pick a sound, YES Master fits it to the track.
- **Advanced:** the current app — show the desk, drive it yourself.

Simple Mode means **fewer user-facing decisions, not fewer DSP decisions.** The phone app already *is* this product; the desktop Simple view inherits its shape and finally shares the desktop's validated adaptive engine.

## 2. Architecture — Simple is a lossless view (not a fork)

One state, one engine, one export path. Simple/Advanced is a **UI filter over the same `MasteringSettings`**, not a second code path.

- A Simple choice (`Warm + 55% + Medium`) writes the *exact same* settings an Advanced user would build: `preset = Tape`, `intensity = 0.55`, loudness → explicit LUFS target. Nothing Simple-only is stored.
- Switching Simple → Advanced reveals the full controls on the same settings; switching back **hides** controls without clearing them. Any hidden Advanced edits (manual EQ, width, compressor) **persist and still render** — that's the lossless contract and the de-risk (Advanced is the current app; Simple can't break it).
- Mode is a single piece of UI state (`viewMode: "simple" | "advanced"`), persisted. It does **not** live in `MasteringSettings` (it's a view preference, not a track property).
- Optional (low priority): a subtle "Advanced edits active" hint in Simple when hidden manual EQ/advanced values are non-default, so a power user isn't surprised by inherited tweaks.

## 3. The four presets (the reference-4)

Simple shows exactly four tiles, friendly-named, all reference-validated:

| Tile | Subtitle | → internal `Preset` | accent |
|---|---|---|---|
| **Balanced** | Clean balance | `Universal` | blue |
| **Bright** | Air & detail | `Clarity` | cyan |
| **Warm** | Glue & body | `Tape` | amber |
| **Heavy** | Sub & weight | `Oomph` | violet |

Rationale: these four cover the tonal space (neutral / bright / warm / heavy), every one is matched to a website reference master, and with loudness on its own dial no "Loud" tile is needed. Punch and Warmth (the iPhone's current non-reference tiles) are **not** in Simple; they remain available in Advanced's full 8.

## 4. Controls and their mapping

Three numbered choices + transport. Each maps directly to `MasteringSettings`:

1. **Style** (2×2 tiles) → `settings.preset`.
2. **Intensity** (slider 0–1, default 0.5; labels Subtle / Full / Pushed; % readout) → `settings.intensity` (drives `preset_scale = 0.4 + 1.2 × intensity`).
3. **Loudness** (segmented Low / Medium / High) → explicit integrated-LUFS target **−14 / −11 / −9** (the iPhone's shipped values). Decoupled from export *format*.

Transport / monitoring (in the hero):
- **Original / Mastered** audition toggle (no playhead loss) — existing live-chain.
- **Volume Match** toggle (listening only; never changes the export).
- The **analyzing/loading progress surface** built this session (waveform-deck progress bar) is the "fitting to your track" moment.

The adaptive guardrails are **always on** under the hood in both modes; they are never surfaced as a Simple control.

## 5. Layout — Option B (desktop reflow)

The iPhone's design language, reflowed for desktop width (chosen over the literal phone-panel-as-modal so the waveform/meters have room and it reads as a desktop product). Mockup of record: `docs/simple-mode-mockup.html`.

- **Chrome:** brand · Track/Album tabs · **Simple | Advanced** segmented toggle (top-right).
- **Left column:** hero (track chip + import, big Play, waveform/progress surface, Original/Mastered, Volume Match).
- **Right column:** `1 · Style` (2×2 tiles) → `2 · Intensity` (slider) → `3 · Loudness` (segmented) → **Create Master / Export** primary CTA → `Advanced ⌄` affordance.
- Dark navy gradient + blue-accent visual language carried from the phone.

## 6. Export & the review ceremony

- Simple auto-targets a **streaming-safe WAV** (sample rate follows source; default bit depth). **No format picker in Simple** — sample rate / bit depth / ceiling live only in Advanced.
- The **Export-With-Review confirm-gate (Adjust / Export Anyway) is removed in Simple.** No explanation panel.
- **Kept in both modes:** advisory warnings stay visible; **technical hard-stops** still block a genuinely invalid export (bad path, non-finite render, requested/rendered sample-rate mismatch).

## 7. Default mode & persistence

- First launch → **Simple** (the trustworthy first-click). Afterward, reopen the **last used** mode.
- Persisted alongside existing UI/session preferences.

## 8. Desktop ↔ mobile parity

They can't share UI code (React vs SwiftUI), but they share what matters:
- The **engine** (already shared via `yes_master_lib`).
- The **(style, intensity, loudness) → `MasteringSettings`** mapping. Today the iPhone's `lib.rs:255-261` maps `balanced→Universal, warm→Warmth, open→Clarity, punch→Punch`. **Re-point it to the reference-4** (`warm→Tape`, new `heavy→Oomph`, drop `punch` from Simple) so desktop and mobile run the identical validated four. This mapping becomes the single source of truth for "what a style means," used by both surfaces.
- The iPhone thereby inherits the adaptive engine it predates (it already routes through the shared chain; aligning the preset mapping is the remaining step).

## 9. Component breakdown (isolation & reuse)

New, focused units (each independently understandable/testable):
- `useViewMode` — the `simple | advanced` toggle + persistence (pure-ish hook; one job).
- `SimpleModeView` — the Option-B layout container; composes existing pieces.
- `StyleTiles` — the four friendly-named tiles → `setPreset` (thin wrapper over preset selection).
- `LoudnessSegmented` — Low/Med/High → explicit loudness target.
- A small `styleToPreset` / `loudnessToTarget` mapping module (mirrors the Rust `lib.rs` mapping; one source of truth on the FE).

Reused as-is from the current app: the hook (`useTrackMaster`), transport, waveform + progress surface, Original/Mastered + Volume Match, Intensity knob/slider, export flow, adaptive engine. Simple is assembled from existing, tested primitives — minimal new surface.

## 10. Testing

- Mapping: `styleToPreset` / `loudnessToTarget` pure-unit tests (Vitest), mirrored by the Rust `export_settings_for_options` tests updated to the reference-4.
- Lossless contract: switching Simple↔Advanced preserves `MasteringSettings` (no clearing of hidden Advanced edits); a Simple selection produces the same settings as the equivalent Advanced edits.
- Default-mode persistence: new session → Simple; remembers last.
- Render parity: a Simple master == an Advanced master built with the same (preset, intensity, loudness).

## 11. Out of scope / future

- **Tier-2 corrective / reference-matching** ("extract a reference's curve and apply it") — opt-in future feature, never automatic.
- Friendly-name remapping of all 8 presets in Advanced (Advanced keeps real names).
- Album mode in Simple (v1 Simple is single-track, like the phone).

## 12. Decisions resolved

Layout B ✓ · reference-4 with Oomph (Punch dropped) ✓ · Intensity exposed ✓ · Loudness −14/−11/−9 decoupled from format ✓ · review ceremony removed, warnings/hard-stops kept ✓ · default Simple for new users, remember last ✓ · adaptiveness = current Tier-1 engine ✓.

Tile names (Balanced/Bright/Warm/Heavy) and the "Advanced edits active" hint are the only soft calls — confirm at spec review.
