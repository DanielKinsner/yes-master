# Album Master mode — Phase B implementation plan

This is the design doc the codex port plan (`album-mastering-port-plan-v2.md` §B)
gates Phase-B code on. **No Rust commits land for Phase B until this plan is
approved by Dan.** Once approved, an implementing session reads this doc top
to bottom and follows the architecture and stop conditions verbatim.

## Product spec (re-stated for clarity)

The user has 2–12 finished tracks (analyzed or master-ready). They want the
final delivery to feel like a coherent record, not a list of singles. Album
mode provides:

1. **Reorderable track list** — drag tracks into the desired sequence.
2. **Per-track role inference** with manual override. Roles:
   Opener / AlbumTrack / Single / Ballad / Closer / Interlude. Role drives
   per-track loudness offset and character bias (per Codex's `arc.py`).
3. **Album arc** — a target loudness/intensity trajectory across the
   record. Four named arcs from Codex (Cinematic / Afterhours / ClubPeak /
   FeverDream), each a 6-point intensity curve with cosine-easing resample
   to the actual track count. Plus a `Custom { lufs_offsets: Vec<f32> }`
   variant for full manual control.
4. **Transitions** — Direct or Gap only (Gap configurable 0–5 s of silence).
   No interlude generator (Codex's `interludes.py` is explicitly NOT ported).
5. **Render** — produces per-track WAVs (`01-track-title.wav`,
   `02-track-title.wav`, …), one continuous `album.wav` joined per the
   transition specs, and a `manifest.json` documenting the plan.

## Source material referenced

In priority order, only the pieces actually needed:

1. `../album-mastering-studio/src/album_mastering_studio/arc.py` — album arc
   planner. Specifically: `ARC_PRESETS` (the four 6-point curves +
   role labels), `_resample_curve` (cosine-easing resample),
   `_character_loudness_offset` (the per-character ±dB offsets),
   `_handoff` (transition classification), `_mastering_bias` (per-character
   EQ moves).
2. `../album-mastering-studio/src/album_mastering_studio/character.py` —
   per-track character inference. Labels: `heavy_djent`, `return_acoustic`,
   `transition`, `acoustic_folk`, plus the default unlabeled case.
3. `../album-mastering-studio/src/album_mastering_studio/pipeline.py` —
   skim only for album-related orchestration (the `plan_album` /
   `render_album_master` entry points). Codex's bulk pipeline is bound to
   the Python sidecar; we re-implement orchestration in Rust.

**Not read by this phase:** Codex's `dashboard.py`, listening packets,
codec previews, release readiness, agent-trail docs, `interludes.py`'s
generator (only the transition concept is borrowed).

## Anti-features (explicit non-goals for Phase B)

These are off-limits unless explicitly added in a later phase:

- Interlude generator (no synthesized transition material — Direct / Gap only).
- Album story / role-justification copy generation (Codex flavor, not function).
- Per-track per-band override AT THE ALBUM LEVEL (per-track DSP comes from
  the existing Track Master settings; the album layer only modulates
  `lufs_offset_db` and `intensity` per-track based on arc + role).
- Boundary preview (render only the join between two tracks). Deferred.
- Separate album dashboard HTML.

## Data model

All new types live in `src-tauri/src/types.rs`, follow existing serde
patterns (`#[serde(rename_all = "snake_case")]`, optional fields with
`#[serde(default)]`), and round-trip cleanly in `.ams.json`.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AlbumArcKind {
    /// Codex curve (0.32, 0.52, 0.78, 1.00, 0.70, 0.46). Invitation → climb
    /// → peak → release → afterglow.
    Cinematic,
    /// Codex curve (0.78, 0.66, 0.55, 0.43, 0.34, 0.28). Bright → dim
    /// → private.
    Afterhours,
    /// Codex curve (0.46, 0.62, 0.78, 0.96, 1.00, 0.74). DJ-set energy ramp.
    ClubPeak,
    /// Codex curve (0.58, 0.34, 0.86, 0.48, 1.00, 0.39). Deliberately
    /// unstable.
    FeverDream,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AlbumArc {
    /// One of the four named Codex arcs. The runtime resamples its 6-point
    /// curve to the actual track count via cosine easing.
    Preset { preset: AlbumArcKind },
    /// Manual per-track LUFS offset table — one entry per track, in
    /// playback order.
    Custom { lufs_offsets: Vec<f32> },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionKind {
    /// Tracks join with no silence. Sample-accurate butt-splice.
    Direct,
    /// `duration_seconds` of digital silence between tracks.
    Gap,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TransitionSpec {
    pub kind: TransitionKind,
    /// 0..=5 seconds. Ignored when `kind = Direct`.
    pub duration_seconds: f32,
}

impl TransitionSpec {
    pub const fn direct() -> Self {
        Self { kind: TransitionKind::Direct, duration_seconds: 0.0 }
    }
    pub const fn gap(seconds: f32) -> Self {
        Self { kind: TransitionKind::Gap, duration_seconds: seconds }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlbumTrackEntry {
    pub track_id: TrackId,
    /// Position in playback order (1-indexed for the manifest; the Vec
    /// position in `AlbumPlan::tracks` is the canonical order).
    pub position: u32,
    /// Inferred role with optional manual override.
    pub role: TrackRole,
    /// `true` when the user has manually picked the role and it should not
    /// be re-inferred on re-plan.
    #[serde(default)]
    pub role_locked: bool,
    /// Per-track LUFS shift applied by the arc planner (negative = quieter
    /// than the album-intent target).
    pub arc_lufs_offset_db: f32,
    /// Per-track intensity multiplier (1.0 = the album-intent intensity;
    /// > 1.0 pushes harder for this track; < 1.0 softens).
    pub intensity_scale: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlbumPlan {
    pub title: String,
    pub arc: AlbumArc,
    /// Tracks in playback order. The Vec position is authoritative.
    pub tracks: Vec<AlbumTrackEntry>,
    /// `tracks.len() - 1` entries, transitions[i] connects tracks[i] → tracks[i+1].
    pub transitions: Vec<TransitionSpec>,
    /// Album-level intensity multiplier the arc respects. Clamped to [0, 2].
    pub intensity: f32,
}
```

### MasteringSettings extension

`MasteringSettings` gains an `Option<AlbumPlan>` field bound to `mode = "album"`.
For `mode = "track"` the field is `None`. Existing per-track `MasteringSettings`
on each track stays the source of truth for that track's DSP; the album layer
only modulates the per-track `lufs_offset_db` and `intensity` at render time.

```rust
pub struct MasteringSettings {
    // … existing fields …
    /// Album mode only. None for Track Master mode.
    #[serde(default)]
    pub album: Option<AlbumPlan>,
}
```

(Project-mode is already a separate `ProjectState.mode: ProjectMode` field;
the `AlbumPlan` on `MasteringSettings` carries the actual data once the
user enters album mode.)

## Algorithm — arc → per-track offsets

Direct port of Codex's `arc.py:build_album_arc` lines 102–195, simplified
for the smaller scope (no interlude generator, no character-of-the-album
inference for now). The math:

```text
Inputs:
  N = len(tracks)
  arc_curve[6]    = ARC_PRESETS[arc_kind].curve
  intensity ∈ [0, 2]
  per-track analysis: stats[i] (AnalysisResult)

Steps:
  1. curve = resample(arc_curve, N) via cosine easing  (arc.py:202-218).
     For each i ∈ 0..N:
       position = i * 5 / (N - 1)
       lower    = floor(position)
       upper    = min(lower + 1, 5)
       fraction = position - lower
       eased    = 0.5 - 0.5 * cos(π * fraction)
       curve[i] = arc_curve[lower] * (1 - eased) + arc_curve[upper] * eased

  2. For each track i:
       energy        = stats[i].energy_density_score  (or 0.5 fallback)
       arc_offset    = (curve[i] - 0.5) * 3.2 * intensity   (dB)
       source_comp   = (0.5 - energy) * 0.45                 (dB)
       char_offset   = character_loudness_offset(stats[i].inferred_character, i, N)
                       per arc.py:287-299:
                         acoustic_folk:     -0.72   (-0.25 if first track)
                         transition:        -1.25
                         heavy_djent:       +0.82
                         return_acoustic:   -1.05   (-0.20 if last track)
                         (other / None):     0
       track_offset  = arc_offset + source_comp + char_offset

  3. role = arc_role_at(arc_kind.roles, i, N):
       For now we ONLY use the role labels for the manifest. The structural
       role (TrackRole enum: Opener/Single/Ballad/AlbumTrack/Closer/Interlude)
       comes from the per-track analyzer; the arc's per-position label
       (Codex's "invitation/threshold/centerpiece/..." strings) is a
       human-readable manifest annotation only.
       Render-relevant role assignment:
         i = 0          → Opener
         i = N - 1      → Closer
         track.duration < 90s AND track.transient_flux < 0.4 → Interlude
         else default to the per-track AnalysisResult.inferred_role
```

`AlbumTrackEntry.arc_lufs_offset_db` is set to `track_offset`. `intensity_scale`
defaults to 1.0 in v1 (the character-bias / `_mastering_bias` table from
arc.py:302-352 is deferred — those are EQ moves that would require the album
layer to override per-band gains, a bigger refactor than the scope allows).

## Algorithm — transitions

Direct + Gap only. The album planner can suggest a default:

```text
  for each adjacent pair (left, right):
    if left.character == "transition" or right.character == "transition":
        TransitionSpec::Gap(0.5)
    else:
        TransitionSpec::Direct
```

The user can override per-transition in the UI. Gap duration is clamped to
`[0.0, 5.0]` seconds at the spec level and on every input event.

## Render pipeline

A new `render_album` command (renames the existing stub `render_album_master`
where needed) consumes `AlbumPlan` plus the per-track `MasteringSettings`.

```rust
#[tauri::command]
pub async fn render_album(
    plan: AlbumPlan,
    per_track_settings: HashMap<TrackId, MasteringSettings>,
    track_paths: HashMap<TrackId, PathBuf>,
    app: tauri::AppHandle,
) -> CommandResult<AlbumRenderReport>
```

The implementation lives in `src-tauri/src/engine.rs` and:

1. **Per-track render.** For each `AlbumTrackEntry` in playback order, build
   an effective `MasteringSettings` by cloning the user's per-track settings
   and shadowing `lufs_offset_db`/`intensity`:

   ```
   effective.advanced.lufs_offset_db = Some(
       per_track_settings[entry.track_id]
           .effective_target_lufs()
           .unwrap_or(-14.0)
           + entry.arc_lufs_offset_db
   );
   effective.intensity *= entry.intensity_scale;
   ```

   Then call the existing `mastering_render_with_progress` to produce a
   per-track WAV named `NN-<sanitized_title>.wav` where `NN` is the 2-digit
   playback order (`01`, `02`, …).

2. **Continuous album WAV.** Stream the per-track outputs into a single
   continuous file via `write_samples_into_writer` (which now does TPDF
   dither per Phase A4). Between tracks, write `transitions[i]` worth of
   silence frames when `TransitionKind::Gap` — `Direct` writes nothing
   between, producing a sample-accurate butt-splice.

3. **Manifest.** Write `manifest.json` next to the album WAV containing the
   plan + per-track measured LUFS (post-render) + per-track output path.

Progress events: emit `render:progress` with `track_idx / total_tracks +
within_track_fraction / total_tracks` so the UI can render a single
album-wide bar.

## UI sketch

Track Master / Album Master toggle in the top header already exists. In
Album mode the workspace replaces the single-track view with:

- **Sidebar** unchanged (track list + per-track metadata chips). Drag-handle
  on each track to reorder. Sidebar order = `AlbumPlan.tracks` order.
- **Workspace** shows the active track (whichever is selected) with the
  existing waveform, transport, presets, knobs. The Macros row carries an
  additional badge: "Album: -1.05 LUFS / ×0.94 intensity (arc + character)".
- **New top strip in the workspace**: Arc dropdown (Cinematic / Afterhours /
  ClubPeak / FeverDream / Custom) and an album-intensity slider. Below the
  dropdown, a horizontal lane representing the album with one tile per
  track showing track number + title + role chip; click a tile to select
  that track in the workspace.
- **Between each pair of track tiles**: a small Transition picker
  (Direct / Gap with seconds spinner).
- **Right rail's Export Master** swaps to **Export Album** in album mode;
  clicking renders all tracks, the continuous WAV, and the manifest, then
  opens the output folder.

## Tests

In `src-tauri/src/dsp.rs::mod tests` and `src-tauri/tests/album_render.rs`:

1. **Curve resample**: a unit test asserting `_resample_curve` (ported)
   produces the published 6-point Cinematic curve verbatim when `N = 6`,
   and produces monotonically interpolated values when `N = 3` and `N = 12`.
2. **Character loudness offset**: unit tests pinning the exact per-character
   ±dB values from arc.py:287-299, including the first/last-track edge cases.
3. **AlbumPlan serde round-trip**: a plan with 4 tracks + 3 transitions
   (Direct, Gap 1.5s, Direct) serializes and deserializes back to the same
   value.
4. **End-to-end smoke** in `tests/album_render.rs`:
   - Build 3 synthetic tracks (5s each — sine, pink, square envelope) under
     `private-audio-fixtures/` or in-memory.
   - Run `render_album` with `Cinematic` arc.
   - Assert: 3 per-track WAVs exist with `01-`, `02-`, `03-` prefixes; one
     continuous `album.wav` exists; `manifest.json` exists and round-trips
     through serde back to a valid `AlbumPlan`.
   - Assert: the continuous WAV's measured integrated LUFS lands within
     ±1 LU of the arc-modulated target (i.e. the arc actually moved the
     loudness).

All existing tests must continue to pass — Album mode is purely additive.
`MasteringSettings.album = None` (the default for older `.ams.json`) means
all current code paths see the same data structure they already see.

## Files expected to change

```
src-tauri/src/types.rs           — new types (AlbumArc, TransitionKind/Spec,
                                    AlbumTrackEntry, AlbumPlan); MasteringSettings
                                    gains Option<AlbumPlan>.
src-tauri/src/engine.rs          — render_album command + per-track shadow
                                    helpers + manifest writer.
src-tauri/src/lib.rs             — register render_album in the invoke handler.
src-tauri/tests/album_render.rs  — new integration test (3-track smoke).
src-tauri/src/dsp.rs::mod tests  — arc resample + character offset unit tests.
src/bindings.ts                  — TS mirrors of the new Rust types.
src/hooks/useTrackMaster.ts      — setAlbumArc, setTransition, reorderTracks
                                    callbacks; selectedAlbumPlan derived state.
src/components/AlbumPanel.tsx    — new component for the Arc strip + track
                                    lane + transition pickers.
src/App.tsx                      — Album-mode wiring (the existing top tab
                                    already toggles; Album shows AlbumPanel
                                    above the workspace).
docs/PRODUCT.md                  — note Album mode is now implemented;
                                    update the Album Master section as
                                    appropriate (this requires Dan approval
                                    since PRODUCT.md is canon).
```

## Stop conditions

Per the plan v2 §B:

- `docs/ALBUM_MASTER_PLAN.md` exists and is approved. ← this doc, approval
  step is the gate.
- `MasteringSettings.album` round-trips through `.ams.json` cleanly.
- `render_album` works end-to-end on a real 3-track input.
- Album mode UI ships with reordering, role display, arc dropdown,
  Direct/Gap transitions.
- Real-album smoke test exists in `src-tauri/tests/album_render.rs`.
- Track Master mode is untouched and still works.
- `cargo test --lib && cargo test`: all green.

## Scope boundary — explicit follow-ups (not Phase B)

- Per-character mastering-bias EQ moves (arc.py:302-352). Requires the album
  layer to override per-band gains, which means a new "effective EQ" stage
  in `ChainCoeffs`. Deferred to a separate phase.
- Codex's full transition style picker (`_choose_style` plus `INTERLUDE_STYLES`).
  We ship Direct + Gap only; the rich transition system is its own future
  workstream.
- Album-level quality checks beyond the per-track ones we already run.
- An "Album Insights" card analogous to the per-track one.

## Implementation order

Once approved, the implementing session does:

1. Land the data model (`types.rs` additions + serde round-trip test).
2. Implement the arc resample + character offset math (pure Rust, unit-tested
   in isolation; no I/O yet).
3. Implement `render_album` end-to-end on synthetic 3-track input; pass the
   integration smoke test.
4. Wire the frontend: Album mode tab routes to a new `AlbumPanel` component;
   useTrackMaster gets the new callbacks.
5. Polish: manifest format, edge cases (1 track, identical tracks, ordering
   changes), and the progress event aggregation.

Each step is its own commit, all on a `phase-b-album-master` branch. The
branch merges to master once the smoke test passes.

---

**End of plan. Awaiting Dan's approval before any Phase B code lands.**
