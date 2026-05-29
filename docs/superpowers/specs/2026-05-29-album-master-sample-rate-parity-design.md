---
title: Album Master Sample-Rate Parity — Design
status: approved
created: 2026-05-29
origin: brainstorming session 2026-05-29
supersedes_scope: "Deferred to Follow-Up Work" item in docs/plans/2026-05-28-001-release-candidate-finish-plan.md (full Album Master delivery-format parity)
---

# Album Master Sample-Rate Parity — Design

## Problem Frame

Track Master already honors delivery sample rate on export: a CD profile renders
44.1 kHz from a 48 kHz source, streaming renders 48 kHz, etc. It does this through
an offline resampler (`rubato`, wrapped by `sample_rate::convert_interleaved`) wired
into `engine.rs::mastering_render_with_progress` (commit `7b4f679`).

Album Master does none of this. `album_render::render_album_plan_impl`:

1. Ignores the delivery profile's sample rate entirely — the album is always written
   at whatever rate the source files happen to be.
2. Hard-fails when tracks have different source sample rates:
   `"album sample-rate mismatch ... (resampling not yet supported)"`
   (`album_render.rs:166–170`).

This is the documented "deferred" limitation in the RC finish plan. This design
closes it: the album exports at a chosen delivery sample rate **and** accepts mixed
source rates by resampling every track to one common album rate.

## Decisions (from brainstorming)

1. **Scope = both wins.** Honor a delivery sample rate AND accept mixed-source-rate
   albums. Both fall out of the same resampling work.
2. **Explicit album-wide control.** The album's delivery format is chosen on the
   Album panel, not inferred from track 1. (Today bit depth is silently taken from
   the first track; this design makes the album format explicit instead.)
3. **Control owns sample rate + bit depth** (a unified "Delivery Format"). It does
   NOT own LUFS target/ceiling — the album Arc already owns the per-track loudness
   story, and an album-level loudness control would collide with it.
4. **Track Master is not edited.** Zero changes to `engine.rs`'s Track Master path.
   The album path reuses the shared resampler primitive directly and mirrors Track
   Master's naming/order so the two read as cousins, but the working hero stays as-is.
5. **Additive / backward-compatible.** New fields default to "Auto", which reproduces
   today's behavior exactly. No existing album silently changes format.

## Non-Goals (explicitly out of scope)

- **Channel-count parity.** Mono-vs-stereo track mismatch keeps its current hard
  error. Documented as a separate, still-deferred limitation.
- **Album-level loudness/ceiling control.** Owned by the Arc; not part of this format
  control.
- Any edit to the Track Master render path.
- Refactoring Track Master to share a "resample-if-needed" guard (rejected to protect
  the working path; trivial duplication is preferred over editing the hero).

## Architecture

### Shared vs. duplicated logic

The valuable, drift-prone logic — the actual sample-rate conversion — already lives
in one place: `sample_rate::convert_interleaved`. Both Track Master and the album
path call it. `convert_interleaved` already returns an unchanged copy when source ==
target, so the album path needs no separate "skip if equal" guard; it simply calls
the same function. Alignment with Track Master is achieved by matching ordering
(chain → SRC → measure → land) and variable naming, NOT by editing Track Master.

### Data model — `src-tauri/src/types.rs`

Add two fields to `AlbumPlan`, both `#[serde(default)]` so older saved `.ams.json`
projects and older plans deserialize to `None` (= Auto = today's behavior):

```rust
pub struct AlbumPlan {
    // ...existing fields: title, arc, tracks, transitions, intensity...
    #[serde(default)] pub delivery_sample_rate: Option<u32>, // None = Auto
    #[serde(default)] pub delivery_bit_depth: Option<u16>,   // None = Auto
}
```

Semantics:

- `delivery_sample_rate`:
  - `None` (Auto): the **highest source sample rate** among the album's tracks.
    Quality-safe (never forces a needless downsample) and deterministic for mixed
    sources. When all sources already match, Auto == that shared rate == today's
    output.
  - `Some(44100 | 48000 | 96000)`: convert every track to this rate.
- `delivery_bit_depth`:
  - `None` (Auto): the current first-track `effective_bit_depth()` behavior, preserved.
  - `Some(16 | 24 | 32)`: write the album (and per-track WAVs) at this depth.

### Plumbing — `PlanAlbumRequest` / `build_album_plan`

`PlanAlbumRequest` (in `engine.rs`) gains optional `delivery_sample_rate` /
`delivery_bit_depth`. `album::build_album_plan(...)` stamps them onto the returned
`AlbumPlan`. This keeps plan construction in one place (existing pattern) and lets
the manifest report the real delivered format.

### Render path — `src-tauri/src/album_render.rs::render_album_plan_impl`

1. **Resolve format once, up front.**
   - `album_sample_rate` = `plan.delivery_sample_rate` if `Some`, else the max source
     rate (requires knowing each track's source rate; resolve via a light pre-scan of
     decoded rates, or compute the max lazily before pass 2 — implementer's choice as
     long as it's deterministic).
   - `album_bit_depth` = `plan.delivery_bit_depth` if `Some`, else current first-track
     `effective_bit_depth()`.
2. **Remove the hard-fail** at `album_render.rs:166–170`. Replace the
   "sample-rate mismatch → error" branch with per-track resampling.
3. **Per track, mirror Track Master ordering:** decode → DSP chain →
   `convert_interleaved(samples, pcm.sample_rate, album_sample_rate, channels)` →
   `measure_and_apply_ceiling_bounded_landing(...)` at the album rate → write per-track
   WAV at `album_sample_rate` / `album_bit_depth`. Landing at the album rate keeps the
   true-peak/landing math describing the delivered file (matches Track Master).
4. **Downstream uses the album format everywhere:** continuous `album.wav`
   (`wav_spec(common_channels, album_sample_rate, album_bit_depth)`), gap-silence frame
   math (`gap_frames = gap_seconds * album_sample_rate`), and the manifest's
   `sample_rate` / `bit_depth`.
5. **Channel mismatch** keeps its existing hard error (non-goal).

### Frontend — `src/components/AlbumPanel.tsx`, `src/hooks/useTrackMaster.ts`

- New "Delivery Format" group in the existing `album-panel-controls` row (no layout
  overhaul): two `<select>`s — Sample rate (Auto / 44.1 / 48 / 96 kHz) and Bit depth
  (Auto / 16 / 24 / 32-bit), both defaulting to Auto.
- Hook state `albumSampleRate` / `albumBitDepth` with setters, mirroring the
  `albumIntensity` pattern. Threaded into `api.planAlbum(...)` via the extended request.
- The album receipt (`album-export-receipt-meta`) gains the rendered rate + bit depth
  so the delivered format is always visible (honest reporting, mirroring Track Master).

## Error Handling & Honesty

- **Upsample advisory.** When `album_sample_rate` exceeds every source rate (pure
  upsample), surface a gentle advisory worded like Track Master's: supports delivery
  requirements; does NOT restore detail lost to lossy/compressed sources. No quality
  claim.
- **Delivery integrity (optional, matches Track Master).** Treat a requested-rate vs
  rendered-rate disagreement as a technical failure / critical review item rather than
  a silently-wrong receipt.
- **Channel mismatch** remains a clear hard error.

## Testing (TDD — mirror `src-tauri/tests/delivery_profile_render.rs`)

Backend:

- Mixed-source album (44.1 kHz + 48 kHz tracks) renders one continuous file at the
  album rate instead of erroring.
- CD-format album (`delivery_sample_rate = 44100`) from 48 kHz sources → 44.1 kHz
  per-track WAVs, continuous WAV, and manifest.
- 96 kHz album from 44.1 kHz sources → upsampled output; advisory present.
- Auto rate with mixed sources picks the highest source rate.
- Manifest + receipt rate and bit depth match the actual written file headers.
- Backward-compat: an `AlbumPlan` JSON missing the new fields deserializes to `None`
  (Auto) and reproduces today's behavior.
- Track Master's existing tests remain green (no edits to that path).

Frontend:

- `AlbumPanel` renders both selects with Auto defaults.
- A non-Auto selection reaches `api.planAlbum` and is present on the rendered plan.

## Verification (after implementation)

```powershell
npm test
npm run build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test
```

Plus a manual album export at a non-source rate (e.g. a CD-format album from 48 kHz
sources), confirming the written WAV header reports 44.1 kHz / 16-bit and the file
plays back correctly.

## Files Touched

- `src-tauri/src/types.rs` — `AlbumPlan` fields (+ tests).
- `src-tauri/src/engine.rs` — `PlanAlbumRequest` fields only (Track Master render path
  untouched).
- `src-tauri/src/album.rs` — `build_album_plan` stamps the new fields.
- `src-tauri/src/album_render.rs` — resample wiring; remove hard-fail; format applied
  downstream.
- `src-tauri/tests/delivery_profile_render.rs` (or a new album-focused test file) —
  backend scenarios.
- `src/bindings.ts` — `AlbumPlan` / request typings.
- `src/hooks/useTrackMaster.ts` — album delivery-format state + plumbing.
- `src/components/AlbumPanel.tsx` — Delivery Format controls + receipt display.
- Album panel + receipt tests under `src/`.
- `docs/RELEASE_STABILIZATION.md` / `docs/APP_BEHAVIOR.md` — note Album parity landed;
  channel-mismatch still deferred.
</content>
</invoke>
