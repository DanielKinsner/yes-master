# Standard View (default) — Design Spec

Date: 2026-06-08 (rev 3 — owner + Codex review folded in)
Status: approved design, ready for implementation planning
Builds on: `docs/SIMPLE_ADVANCED_MODE_NOTE_2026-05-29.md`, `docs/ENGINE_REFERENCE.md`, the iPhone app (`apps/iphone-native`)

> Working title was "Simple Mode." **It isn't a lesser mode — it's THE mode.** The default view *is* YES Master; **Advanced** is a door power users step through. No "Simple" badge in the product; internally we call the default **Standard** (vs **Advanced**).

## 1. Goal & promise

The default desktop experience: a trustworthy first-click master with almost no decisions, riding the validated adaptive engine. **Advanced** is the full mastering desk (the current app), opened deliberately.

- **Standard (default):** pick a sound, YES Master fits it to the track.
- **Advanced:** show the desk, drive it yourself.

Fewer user-facing decisions, not fewer DSP decisions. The phone app already *is* this; the desktop Standard view inherits its shape and shares the desktop's validated adaptive engine.

**Design-quality bar:** Standard is the product's face, not a stripped-down afterthought — first-class polish (motion, hero, micro-interactions). Treat it as flagship UI during build (use the design skills), not a settings panel.

## 2. Architecture — one settings truth, an asymmetric Standard↔Advanced transition

One engine, one export path, **one settings _type_ resolved per track** — never a global blob.

- Settings live per track: `settingsMap[trackId]`, resolved to the hook's **`selectedSettings`** (which already honors `albumIntent` / `followingAlbumIntent`). **`StandardView` MUST bind to the hook's resolved `selectedSettings` and its setters — never a new global settings object.** Advanced already binds this way, so Standard inherits per-track behavior for free.
- A Standard choice (`Warm + 55% + Medium`) writes the *same* `MasteringSettings` an Advanced user would build: `preset = Tape`, `intensity = 0.55`, loudness → explicit LUFS target. Nothing Standard-only is stored.

### 2a. The transition (the load-bearing decision)
Standard and Advanced are **not** a silent two-way toggle (that would secretly carry hidden edits into a different UI — confusing). They are **asymmetric**:

- **Standard → Advanced ("Take control"):** seamless escalation. The current Standard choice **seeds** Advanced's starting point (Warm 55% → Tape selected at 0.55, etc.). Same underlying settings; the user can now tweak everything. Preset names differ only as *labels* over the same value (Warm = Tape); Advanced shows the real name + all 8.
- **Advanced → Standard ("Back to Standard"):**
  - If **no** non-managed edits were made → returns silently.
  - If non-managed edits exist → a single confirm: *"Back to Standard resets your manual edits to the preset's clean sound. Save them as a preset first?"* with **Save as preset** (existing feature) / **Reset & continue** / **Cancel**. On continue, the non-managed fields reset to default.

**Why this is right (and still avoids the "two-truths" disaster):** there is one settings object the whole time. Rather than letting Advanced edits *lurk* under Standard, Standard holds a **hard invariant — its non-managed fields are always default** — enforced by the reset-on-return. Deciding to go Standard *is* deciding the simple result was enough; if you wanted to keep the Advanced work, you save it as a preset. This eliminates the need for any "hidden edits active" banner.

### 2b. What counts as a "non-managed edit" (drives the return warning + the reset)
Standard **owns**: `{preset, intensity, loudness target, delivery format}`. A non-managed edit = any *sound-affecting* field outside that set being non-default:

| Counts as an edit (warns on return; reset clears) | Does NOT count |
|---|---|
| Manual EQ — any of the 7 bands ≠ 0 | preset / intensity (Standard controls) |
| width / warmth / presence-air set | loudness target (Standard's dial) |
| compressor mode ≠ Preset (Manual **or** Off), or any density/per-band override | delivery format / sample rate / bit depth / ceiling / custom-LUFS (mode-owned; Standard overrides — see §6) |
| input gain or output gain ≠ 0 | |

The reset action is a **new `resetToStandardManaged()`** that clears all left-column fields (EQ + width/warmth/presence + compressor→Preset + gains→0) while preserving preset/intensity/loudness/format. (Explicitly broader than the existing `resetToneControls`, which only does EQ + intensity and would under-reset.)

## 3. The four presets (the reference-4)

Standard shows exactly four tiles, friendly-named, all reference-validated:

| Tile | Subtitle | → internal `Preset` | accent |
|---|---|---|---|
| **Balanced** | Clean balance | `Universal` | blue |
| **Bright** | Air & detail | `Clarity` | cyan |
| **Warm** | Glue & body | `Tape` | amber |
| **Heavy** | Sub & weight | `Oomph` | violet |

These cover the tonal space (neutral / bright / warm / heavy); each is matched to a website reference master; loudness on its own dial means no "Loud" tile. Punch and Warmth stay in Advanced's full 8. **Heavy → Oomph is kept 1:1** (helps later iteration); **owner has ear-confirmed Oomph (2026-06-08) — listening gate satisfied.**

## 4. Controls, mapping, and the WYSIWYG preview

Three numbered choices + transport, each mapping directly to `selectedSettings`:

1. **Style** (2×2 tiles) → `preset`.
2. **Intensity** (slider 0–1, default 0.5; Subtle / Full / Pushed; % readout) → `intensity` (drives `preset_scale = 0.4 + 1.2 × intensity`). Also the answer to "presets too opinionated" — dial any preset back toward neutral without leaving Standard.
3. **Loudness** (segmented Low / Medium / High) → explicit integrated-LUFS target **−14 / −11 / −9** (iPhone's shipped values). Decoupled from export *format*.

Transport / monitoring (hero):
- **Original / Mastered** audition (no playhead loss).
- **Volume Match — default OFF.** VM is an *input*-referenced A/B level-match aid (matches the source's loudness for fair tonal comparison); it never changes the export.
- **WYSIWYG preview (hard requirement):** the **Mastered audition must be the real processed sound — preset + adaptive + loudness landing + limiter** — not "preset EQ/dynamics minus the final loudness/limiting stage." What the user hears in Mastered must equal what Create Master exports. *Verification (plan task): confirm the live audition path applies the loudness landing + limiter in real time; close any "pre-landing"/preview-only gap between live preview and export.*

Adaptive guardrails are always on under the hood; never a Standard control.

## 5. Layout — Option B (desktop reflow)

iPhone design language reflowed for desktop width (chosen over a literal phone-panel modal so the waveform/meters have room and it reads as a desktop product). Mockup of record: `docs/simple-mode-mockup.html`.

- **Chrome:** brand · Track/Album tabs · a single **"Advanced"** affordance top-right (NOT a `Standard | Advanced` segmented control — Standard isn't a labeled co-equal mode). In Advanced, a **"Done / Back to Standard"** returns (with the §2a confirm when edits exist).
- **Left column:** hero — track chip + import, big Play, waveform/progress surface (the "fitting to your track" moment, built 2026-06-08), Original/Mastered, Volume Match.
- **Right column:** `1 · Style` → `2 · Intensity` → `3 · Loudness` → **Create Master** CTA.
- Dark navy gradient + blue-accent language carried from the phone.

## 6. Export & the review ceremony (canon update)

- **Create Master = the deliverable action**, mirroring the iPhone: render the finished master → Save / Reveal / Share. It is **not** a preview (live Mastered audition is the preview).
- Standard exports a **known-safe fixed default — 44.1 kHz / 24-bit WAV** (NOT "follows source," which could emit 96k/24-bit). Confirm exact values against the iPhone's fixed-export (`yes_master_native_fixed_export_settings_json`) for parity. Sample rate / bit depth / ceiling live only in Advanced.
- **Review ceremony — intentional canon change (update `docs/PRODUCT.md`):** the blocking Adjust / Export-Anyway gate is an **Advanced** behavior. **Standard has no blocking gate.** Cosmetic/advisory warnings (loudness-vs-reference, etc.) are **suppressed** in Standard — a non-technical user trusting their ears shouldn't have to care. The only thing kept is **one tiny, non-blocking inline note** for a genuine integrity issue (e.g. true-peak slips over) — never a modal. (The fixed-ceiling limiter makes this rare by construction.) **Technical hard-stops** (bad path, non-finite render, requested/rendered sample-rate mismatch) still block in both modes.

## 7. Default view & persistence (versioned, migration-aware)

Gate the default on a one-time persisted flag (`standardViewMigration`):
- Flag absent **and** a prior session/settings exists → **returning user → Advanced** (don't dump power users into Standard on first open after the update).
- Flag absent **and** no prior session → **new user → Standard.**
- Set the flag either way; thereafter reopen the **last used** view.
This makes "returning at ship time" mechanically = "prior session present AND migration flag absent" — testable.

## 8. Desktop ↔ mobile parity

Can't share UI code (React vs SwiftUI), but share what matters:
- The **engine** (already shared via `yes_master_lib`).
- The **(style, intensity, loudness) → `MasteringSettings`** mapping. Re-point the iPhone's `lib.rs:255-261` to the reference-4 (`warm→Tape`, new `heavy→Oomph`, drop `punch` from the Standard set; keep aliases for back-compat). One source of truth for "what a style means."
- **iPhone app UI:** rename `NativeStylePreset` to **Balanced / Bright / Warm / Heavy** (was Balanced/Warm/Open/Punch), matching desktop, with reference-4 accents/subtitles. The iPhone thereby also inherits the adaptive engine it predates.

## 9. Component breakdown (isolation & reuse)

New, focused units (each independently understandable/testable):
- `useViewMode` — `standard | advanced` state + versioned migration default + persistence + the asymmetric transition (seed-forward; warn/reset-on-return).
- `StandardView` — the Option-B layout container; binds to `selectedSettings`; composes existing pieces.
- `StyleTiles` — the four friendly-named tiles → existing `setPreset`.
- `LoudnessSegmented` — Low/Med/High → explicit loudness target.
- `resetToStandardManaged()` — the §2b broad reset (used by the Back-to-Standard transition).
- `hasNonManagedEdits(settings)` — the §2b detector (drives the return warning).
- `styleToPreset` / `loudnessToTarget` — pure mapping module mirroring the Rust `lib.rs` mapping (one FE source of truth).

Reused as-is: `useTrackMaster`, transport, waveform + progress surface, Original/Mastered + Volume Match, Intensity control, export flow, save-as-preset, adaptive engine.

## 10. Testing

- Mapping: `styleToPreset` / `loudnessToTarget` pure-unit tests (Vitest); Rust `export_settings_for_options` tests updated to the reference-4.
- Transition: Standard→Advanced seeds correctly; Advanced→Standard returns silently with no edits, warns + offers save + resets with edits; `hasNonManagedEdits` / `resetToStandardManaged` field coverage.
- Per-track binding: track-switch keeps each track's Standard choices (bound to `selectedSettings`, not a global).
- WYSIWYG: live Mastered audition character/loudness ≈ exported master (no pre-landing gap).
- Export: fixed 44.1k/24-bit; Standard suppresses cosmetic warnings, surfaces only the integrity note, hard-stops still block.
- Default-view migration: no prior session → Standard; prior session → Advanced; remembers last.
- Render parity: a Standard master == an Advanced master built with the same (preset, intensity, loudness).

## 11. Out of scope / future

- **Tier-2 corrective / reference-matching** ("extract a reference's curve and apply it") — opt-in future, never automatic.
- Friendly-name remapping of all 8 in Advanced (Advanced keeps real names).
- Album mode in Standard — **Album Master is Advanced-only in v1** (Standard is Track-only; selecting Album Master implies/requires Advanced until a Standard-album exists).

## 12. Decisions resolved

Default view = **Standard** (no "Simple" label; Advanced is a door) ✓ · first-class polish bar ✓ · **asymmetric transition: seed forward, warn+reset (with save-as-preset offer) on return; Standard always clean; no hidden-edits banner** ✓ · one settings truth, bound per-track to `selectedSettings` ✓ · layout B ✓ · reference-4 with Oomph kept 1:1, ear-confirmed ✓ · tile names Balanced/Bright/Warm/Heavy ✓ · iPhone renamed + remapped to match ✓ · Intensity exposed ✓ · Loudness −14/−11/−9 decoupled from format ✓ · export = fixed 44.1k/24-bit (parity-checked vs iPhone) ✓ · **WYSIWYG Mastered preview = hard requirement (verify live path)** ✓ · Create Master = deliverable (mirrors iPhone), not preview ✓ · Volume Match default OFF (input-referenced A/B aid) ✓ · delivery format mode-owned, excluded from the shared-edit set ✓ · **review ceremony: canon updated — no blocking gate in Standard; cosmetic warnings suppressed; one tiny non-blocking integrity note; hard-stops stay** ✓ · migration-aware versioned default ✓ · Album = Advanced-only v1 ✓ · adaptiveness = current Tier-1 engine ✓.
