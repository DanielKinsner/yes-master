# Standard View (default) — Design Spec

Date: 2026-06-08 (rev 2 — incorporates owner review notes)
Status: approved design, pre-implementation
Builds on: `docs/SIMPLE_ADVANCED_MODE_NOTE_2026-05-29.md`, `docs/ENGINE_REFERENCE.md`, the iPhone app (`apps/iphone-native`)

> Working title was "Simple Mode." **It isn't a lesser mode — it's THE mode.** The default view *is* YES Master; **Advanced** is a door power users open. So there is no "Simple" badge in the product; internally we call the default **Standard** (vs **Advanced**).

## 1. Goal & promise

The default desktop experience: a trustworthy first-click master with almost no decisions, riding the validated adaptive engine. **Advanced** is the full mastering desk (the current app), opened deliberately.

- **Standard (default):** pick a sound, YES Master fits it to the track.
- **Advanced:** show the desk, drive it yourself.

Fewer user-facing decisions, not fewer DSP decisions. The phone app already *is* this; the desktop Standard view inherits its shape and shares the desktop's validated adaptive engine.

**Design-quality bar:** because Standard is the product's face (not a stripped-down afterthought), it gets first-class polish — motion, hero treatment, micro-interactions. Implementation should treat it as flagship UI, not a settings panel. (Use the design skills during build.)

## 2. Architecture — Standard is a lossless view (not a fork)

One engine, one export path, **one settings _type_ resolved per track** — not a global blob.

- Settings live per track: `settingsMap[trackId]`, resolved to the hook's **`selectedSettings`** (which already accounts for `albumIntent` / `followingAlbumIntent`). **`StandardView` MUST bind to the hook's resolved `selectedSettings` and its setters — never a new single global settings object.** Advanced already binds this way, so Standard inherits per-track behavior for free. (Album mode is out of Standard v1, so no album conflict.)
- A Standard choice (`Warm + 55% + Medium`) writes the *same* `MasteringSettings` an Advanced user would build: `preset = Tape`, `intensity = 0.55`, loudness → explicit LUFS target. Nothing Standard-only is stored.
- Switching to Advanced reveals the full controls on the same settings; switching back **hides** controls without clearing them. Hidden Advanced edits (manual EQ, width, compressor) **persist and still render** — the lossless contract.
- View mode is one piece of persisted UI state (`viewMode: "standard" | "advanced"`). It is **not** part of `MasteringSettings` (it's a view preference, not a track property).

### 2a. "Advanced edits active" indicator — REQUIRED (not optional)
The one place the lossless contract bites: a user tweaks manual EQ/advanced, flips to Standard, and those hidden edits still render — so "Warm 55%" sounds wrong and they don't know why. Standard MUST show, whenever hidden manual EQ / advanced values are non-default:
- a visible indicator: *"Advanced edits are shaping this master,"*
- a one-click **Clear** that resets manual EQ + advanced to default (reuses `resetToneControls` / the tone-reset built 2026-06-08) so the preset behaves purely,
- a **View in Advanced** link.

## 3. The four presets (the reference-4)

Standard shows exactly four tiles, friendly-named, all reference-validated:

| Tile | Subtitle | → internal `Preset` | accent |
|---|---|---|---|
| **Balanced** | Clean balance | `Universal` | blue |
| **Bright** | Air & detail | `Clarity` | cyan |
| **Warm** | Glue & body | `Tape` | amber |
| **Heavy** | Sub & weight | `Oomph` | violet |

These four cover the tonal space (neutral / bright / warm / heavy); every one is matched to a website reference master; loudness on its own dial means no "Loud" tile is needed. Punch and Warmth stay in Advanced's full 8.

## 4. Controls and their mapping

Three numbered choices + transport, each mapping directly to `selectedSettings`:

1. **Style** (2×2 tiles) → `preset`.
2. **Intensity** (slider 0–1, default 0.5; Subtle / Full / Pushed; % readout) → `intensity` (drives `preset_scale = 0.4 + 1.2 × intensity`). Also the answer to "presets too opinionated" — dial any preset back toward neutral without leaving Standard.
3. **Loudness** (segmented Low / Medium / High) → explicit integrated-LUFS target **−14 / −11 / −9** (iPhone's shipped values). Decoupled from export *format*.

Transport / monitoring (hero): **Original / Mastered** audition (no playhead loss), **Volume Match** (listening only), and the **analyzing/loading progress surface** built 2026-06-08 as the "fitting to your track" moment. Adaptive guardrails are always on under the hood; never a Standard control.

## 5. Layout — Option B (desktop reflow)

iPhone design language reflowed for desktop width (chosen over a literal phone-panel modal so the waveform/meters have room and it reads as a desktop product). Mockup of record: `docs/simple-mode-mockup.html`.

- **Chrome:** brand · Track/Album tabs · a single **"Advanced"** affordance top-right (NOT a `Standard | Advanced` segmented control — Standard isn't a labeled co-equal mode; Advanced is a door). In Advanced, a **"Done"** returns.
- **Left column:** hero — track chip + import, big Play, waveform/progress surface, Original/Mastered, Volume Match.
- **Right column:** `1 · Style` (2×2 tiles) → `2 · Intensity` → `3 · Loudness` → **Create Master / Export** CTA → the "Advanced edits active" indicator (§2a) when relevant.
- Dark navy gradient + blue-accent language carried from the phone.

## 6. Export & the review ceremony

- Standard exports to a **known-safe fixed default — 44.1 kHz / 24-bit WAV** (NOT "follows source," which could emit 96k/24-bit). Confirm the exact values against the iPhone's existing fixed-export setting (`yes_master_native_fixed_export_settings_json`) so both surfaces match. Sample rate / bit depth / ceiling controls live only in Advanced.
- The **Export-With-Review confirm-gate (Adjust / Export Anyway) is removed in Standard.** No explanation panel.
- **Kept in both modes:** advisory warnings stay visible; **technical hard-stops** still block a genuinely invalid export (bad path, non-finite render, requested/rendered sample-rate mismatch).

## 7. Default view & persistence (migration-aware)

- **Brand-new user** (no prior session/settings) → **Standard** (the trustworthy first-click).
- **Returning user** (prior session exists at ship time) → **Advanced**, so power users land in the world they left rather than being dumped into Standard on first open after the update.
- Thereafter, reopen the **last used** view. Persisted with existing UI/session preferences.

## 8. Desktop ↔ mobile parity

Can't share UI code (React vs SwiftUI), but share what matters:
- The **engine** (already shared via `yes_master_lib`).
- The **(style, intensity, loudness) → `MasteringSettings`** mapping. Re-point the iPhone's `lib.rs:255-261` to the reference-4 (`warm→Tape`, new `heavy→Oomph`, drop `punch`/`open`/`warmth` from the Standard set; keep aliases for back-compat). One source of truth for "what a style means."
- **iPhone app UI:** rename `NativeStylePreset` to **Balanced / Bright / Warm / Heavy** (was Balanced/Warm/Open/Punch), matching desktop, with the reference-4 accents/subtitles. The iPhone thereby also inherits the adaptive engine it predates.

## 9. Component breakdown (isolation & reuse)

New, focused units (each independently understandable/testable):
- `useViewMode` — `standard | advanced` toggle + migration-aware default + persistence.
- `StandardView` — the Option-B layout container; binds to `selectedSettings`; composes existing pieces.
- `StyleTiles` — the four friendly-named tiles → existing `setPreset`.
- `LoudnessSegmented` — Low/Med/High → explicit loudness target.
- `AdvancedEditsHint` — §2a indicator + Clear (reuses `resetToneControls`) + View-in-Advanced.
- `styleToPreset` / `loudnessToTarget` — pure mapping module mirroring the Rust `lib.rs` mapping (one FE source of truth).

Reused as-is: `useTrackMaster`, transport, waveform + progress surface, Original/Mastered + Volume Match, Intensity control, export flow, adaptive engine. Standard is assembled from existing, tested primitives.

## 10. Testing

- Mapping: `styleToPreset` / `loudnessToTarget` pure-unit tests (Vitest); Rust `export_settings_for_options` tests updated to the reference-4.
- Lossless contract: Standard↔Advanced preserves `MasteringSettings`; a Standard selection == the equivalent Advanced edits; bound to `selectedSettings` (per-track, not global) — test track-switch keeps each track's Standard choices.
- `AdvancedEditsHint`: shows iff hidden manual EQ/advanced non-default; Clear resets them.
- Default-view migration: no prior session → Standard; prior session → Advanced; remembers last.
- Render parity: a Standard master == an Advanced master built with the same (preset, intensity, loudness).

## 11. Out of scope / future

- **Tier-2 corrective / reference-matching** ("extract a reference's curve and apply it") — opt-in future, never automatic.
- Friendly-name remapping of all 8 in Advanced (Advanced keeps real names).
- Album mode in Standard (v1 is single-track, like the phone).

## 12. Decisions resolved

Default view = **Standard** (no "Simple" label; Advanced is a door) ✓ · first-class polish bar ✓ · layout B ✓ · reference-4 with Oomph (Punch dropped) ✓ · tile names Balanced/Bright/Warm/Heavy ✓ · iPhone app renamed + remapped to match ✓ · Intensity exposed ✓ · Loudness −14/−11/−9 decoupled from format ✓ · export = fixed 44.1k/24-bit (parity-checked vs iPhone) ✓ · "Advanced edits active" hint = required ✓ · binds to per-track `selectedSettings` ✓ · review ceremony removed, warnings/hard-stops kept ✓ · migration-aware default (new→Standard, returning→Advanced, remember last) ✓ · adaptiveness = current Tier-1 engine ✓.

No open soft calls remain (internal name "Standard" is the only label-ish choice — veto if preferred).
