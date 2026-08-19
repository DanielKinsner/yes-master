# Current App Behavior

This document describes what the current program does now, not what older
handoffs planned.

## Application

- Tauri desktop app.
- React/TypeScript UI.
- Rust backend for decode, analysis, playback, DSP, render, export checks, and
  project/file operations.
- Product name shown to users: YES Master.
- Package/repo identity: `yes-master`.
- Supported minimum window size: **1360×740** (Q16, verified 2026-07-08;
  enforced by `tauri.conf.json` `minWidth`/`minHeight`). The app is usable on
  **1366×768** laptops — the Advanced desk tiles cleanly (sidebar / main / rail)
  with no horizontal scroll, and Standard keeps its preset tiles, intensity
  knob, and Create Master action fully visible. Only cosmetic bottom padding
  below the intensity knob compresses at the extreme floor (the clamp-sized
  Standard blocks reach their minimum and `overflow: hidden` trims the excess,
  no control is clipped). The dense Advanced view scrolls vertically on short
  viewports, as it already did at any height below ~1230 px.

## Track Master

Track Master supports:

- Importing local audio. Above-stereo sources (e.g. 5.1) fold down to stereo
  at decode (common 5.1 order, LFE excluded) so analysis, audition, and
  export all process the same stereo signal the chain masters; header probes
  and album records still report the file's real channel count.
- Source analysis.
- Waveform display.
- Original/Mastered audition at the same playhead.
- Region selection and loop playback.
- Presets and intensity.
- Visual EQ/tone shaping.
- Volume Match for audition only.
- Delivery profile selection.
- Advanced controls. The Advanced rail carries a global **Reset all**
  (2026-08-19; it confirms first — a misclick must not wipe a tuning) that returns every non-managed control — gains, EQ gains and
  band positions, feel controls, compressor, adaptive strength — to its
  first-open state in one undo step; style, intensity, loudness target and
  delivery format are Standard-managed and stay. Signed-gain knobs (Tone
  Shape, input/output gain) are bipolar: blue above 0 dB, amber below,
  matching the Visual EQ.
- Explicit compressor modes.
- Per-band compressor detail.
- Delivery format selection.
- Explicit save destination for export.
- Warning-aware export review.
- Post-render export receipt/checks.
- Baseline Settings and contextual Help dialogs.
- Visible Save/Open Project feedback.
- Live Master Out meters with plain Standard labels and technical Advanced
  labels/tooltips: live loudness is not the selected target, since-play LUFS is
  the current playback run, and peak is dBFS/peak behavior. Meters have
  real ballistics (2026-08-19): peaks fall at a controlled rate and a
  peak-hold pip marks the highest recent peak (red above −1 dBFS). The
  Advanced per-band compressor card shows live gain reduction per band
  while the master plays. The Original/Mastered flip and the played span of
  the waveform react visually to which source is audible; all such motion
  is presentation-only and respects `prefers-reduced-motion`.
- A first-run guide in Standard view: subtle floating coachmarks that walk a
  new user to the Original→Mastered flip, then a send-off and one Advanced
  pointer. Dismissible, never blocking, persisted under
  `yes-master:first-run-guide`, resettable from Settings ("Show first-run
  tips again"). Entering Advanced or pre-flipping to Mastered ends it
  silently. Spec: `docs/superpowers/specs/2026-06-11-first-run-guide-design.md`.
- A first-launch welcome hero: when no track is loaded, the empty state
  shows the orb idling as the brand visual with the product promise and
  import CTA. Never a wall — import and drag-drop always work; reduced
  motion gets a static glyph.
- An analysis orb: a particle point-cloud animates in the waveform slot
  while analysis runs and persists through the waveform decode, then
  morphs into the track's real waveform when peaks arrive. Presentation
  only — playback readiness never waits on it; any interaction cuts it;
  `prefers-reduced-motion` disables it. Spec:
  `docs/superpowers/specs/2026-06-11-analysis-orb-design.md`.
- Real analysis progress: the backend emits `analysis:progress` events at
  the actual analyzer phase boundaries (decode → dynamics → stereo field →
  tonal balance → deep scan), batch-rescaled across multi-track imports.
  The UI's stage labels and bar track real work; the old paced timer
  remains only as a pre-first-event fallback.
- A clean Standard `Create Master` confirms itself: success card with file
  name, landed LUFS, and a Show file action. Invalid renders keep the
  prominent re-render alert; album receipts stay in their own flow.

Track Master Delivery Profile and Delivery Format are authoritative for Track
Master exports:

- Named profiles set their owned target LUFS, ceiling, bit depth, and sample
  rate together.
- Custom Source keeps the source sample rate.
- Custom format can request 44.1 kHz, 48 kHz, or 96 kHz.
- The rendered WAV, export receipt, and export checks report the effective
  rendered sample rate and bit depth.
- A requested/rendered sample-rate mismatch is treated as a technical integrity
  issue.

Album Master now honors an album-wide Delivery Format (sample rate + bit depth,
chosen on the Album panel; Auto = highest source rate / first-track bit depth).
It resamples each track to that single album rate and resolves one album-wide
channel count. Mixed mono/stereo albums render stereo, with mono tracks upmixed,
and sources above stereo fold down to stereo delivery. Mixed-rate and mixed
channel-count albums therefore render one continuous file instead of failing.

Album exports land in an `<AlbumTitle>/` subfolder of the chosen directory
(Q25 option ii): the per-track WAVs, the continuous album, and `manifest.json`
stay together and never mix with a prior render. The title is sanitized for
filesystem safety and an empty title falls back to `Album`; an existing
subfolder is never overwritten (a fresh ` (2)` suffix is used instead).

Album-layer sound shaping (owner decisions 2026-07-03 D7/D9):

- The album layer modulates per-track loudness via the user-chosen arc plus
  source compensation only. The character-inference system (per-track genre
  labels → extra loudness pulls and EQ/width/warmth/intensity biases) is
  built but **gated OFF by default** pending an owner listening session
  (`YES_MASTER_ALBUM_CHARACTER` env opt-in; `album.rs::ALBUM_CHARACTER`).
- A track marked **Override** gets full sound exemption: its own settings
  and its own loudness target render unmodified — no arc offset, no album
  bias — while the album delivery format (rate / depth / channels) still
  applies. The album manifest, render report, and Album panel receipt all
  mark overridden tracks ("Override: track N rendered with its own
  settings").

Mastered preview readiness timeouts surface recoverable user-facing guidance
instead of silent non-playback.

## Project, Settings, And Help

- Save Project and Open Project use `.ams.json` project files.
- Save/Open success and cancelled dialogs surface calm visible feedback.
- Open Project restores state, selects a track when possible, and reports
  recovery issues that need user action.
- Settings covers current baseline app defaults and app info.
- Settings includes an Audio Output selector. System default is the fallback;
  choosing a named output stores the device name locally and reopens the
  playback stream so the next Original/Mastered audition uses that device.
- Help explains current Import/Analyze, Original vs Mastered, Volume Match vs
  Preview LUFS, Delivery Profile/Format, Export Review, and Save/Open behavior.
- Help also offers "Save diagnostics report": a plain-text file (app version,
  recent log tail, session summary) assembled locally and written to a
  user-chosen path. The app keeps a small rotating log (~2 MiB cap, error
  paths and lifecycle only) under app-data `logs/`; panics are captured
  there too. There is no telemetry — nothing leaves the machine unless the
  user saves the report and shares it themselves.

## Export Checks And Review

The backend checks rendered-output measurements for:

- True peak above critical/safe streaming thresholds.
- Loudness above very-loud territory.
- Dynamic range below the low-dynamic-range threshold.
- Bit depth below 16-bit.
- Non-finite LUFS metering.
- Already-compressed source combined with moderate/heavy preset compression.

Before an export receipt exists, the right rail derives preflight review rows
from current source analysis for true peak, loudness, and dynamic range. These
rows make already-hot or already-compressed sources visible before the user
treats export as a clean path.

The export button is warning-aware:

- No review rows: `Export Master`.
- Warning or critical review rows: `Export With Review`.
- First `Export With Review` click opens the review gate, a modal.
- `Adjust Settings` (and `Escape`, and the scrim) closes it and does not export.
- `Export Anyway` calls the existing export path.

Quality rows are advisory when the app can write a file. Technical failures
still stop export through the render/save path: invalid paths, cancelled save
dialogs, decode/render/write failures, or corrupt/non-finite render state.

### One owner per warning (U10)

Every warning, blocker, and advisory has exactly one owner and is presented
once. Where a second surface must restate a fact to be usable — a decision gate
needs to show what you are deciding about — that surface is **modal**, and the
surface behind it is made `inert`, so no user ever meets the same fact twice.

| Fact | Single owner | Location | Notes |
|---|---|---|---|
| Source analysis (loudness / dynamic range / spectrum / stereo / true peak) + Re-analyze | `SourceInsight` | Under the track title, `INSIGHT` disclosure | Structured rows from `sourceInsightRows`. Since 2026-08-18 the right rail carries no `SOURCE CHECK` card — the rail is configuration → processing → delivery → export. |
| "New analysis to review" (`REVIEW`) | `SourceInsight` | Insight row badge | An **unacknowledged analysis revision**, not a warning: keyed on `track_id + measured_at_iso` (`lib/source-insight.ts`), cleared the moment the Insight disclosure opens (clicking the badge also opens it — 2026-08-19), persisted in localStorage, returns after Re-analyze or a new source. Findings keep their own status regardless. |
| Export check rows (post-export) | `SourceInsight` | Insight disclosure, "Last export" group | Fed by the receipt's checks; the pre-export gate still derives its rows from the same analysis (`derivePreflightChecks` in RightRail). |
| Pre-export decision | `ExportReviewDialog` | Modal over the rail | Restates the warned rows **because** the rail is inert behind it; it is the decision point, not a second report. |
| Post-export result + quality rows | `ExportReceiptCard` | Modal | `aria-modal`, so it owns the checks while up. Rows built by `buildQualityRows`. |
| Standard hard-stop ("saved, but…") | `standardExportNotes().invalid` | Standard rail, `role="alert"` | Standard suppresses cosmetic warnings entirely. |
| Standard integrity note | `standardExportNotes().integrityNote` | Standard rail, inline | One tiny note, never a modal. |
| Album delivery advisories (upsampled / upmixed / folded / override) | `AlbumExportReceipt` | Sidebar receipt | One line each, one place. |
| Per-track album scan state (role, target, override, concern) | `SequenceRowFacts` | Sidebar track row | A chip's visible text plus its `.sr-only` expansion is **one** indicator rendered for two audiences, not a duplicate. |
| Follow/Override control + its explanation | `OverrideBanner` | Track workspace header | The control; the sidebar chip is the scan indicator. Distinct purposes, distinct locations. |
| Disabled-action reasons | the owning component | `title` + `aria-describedby` on the control | Computed once per action and reused for both, so the two cannot drift. |
| Recoverable errors | `formatUserError` | `Toast`, `tone="danger"` | One mapping, one surface. |

`src/App.warning-ownership.test.tsx` pins this table.

## Compressor Modes

The app has an explicit compressor mode field:

- `Preset`: current preset/density fallback behavior.
- `Manual`: user per-band values replace preset compression.
- `Off`: bypasses creative/preset compression only.

`Off` does not bypass the limiter, ceiling protection, LUFS landing, metering,
or export warnings.

The per-band compressor card labels preset fallback values as `Preset`, not
track-aware `Auto`. If a low-dynamic-range source is loaded while `Preset` mode
is active, the card gives local guidance to lower density or switch Off if
movement collapses.

## Private Fixture And Reference Lanes

Private audio is local-only and ignored by git.

The already-mastered matrix runner measures preset/compressor cases against
private fixtures:

```powershell
cd src-tauri
cargo run --example private_fixture_matrix -- --manifest ..\private-audio-fixtures\manifest.json --output ..\test-output\private-fixture-matrix
```

The private reference tuning runner compares YES Master presets against external
reference masters:

```powershell
cd src-tauri
cargo run --example private_reference_tuning -- --references "..\tests for presets" --output ..\test-output\private-reference-tuning
```

Both lanes write ignored ledgers and rendered WAVs. Do not commit private audio,
rendered private masters, waveform images from private audio, or fixture-specific
ledgers.

Both lanes honor `YES_MASTER_CONFIDENCE_GATING`. Leave it unset for the default
release path; set it to `1` only when collecting Phase B confidence-gate evidence
against the same private fixtures/references.

## Reference Retune Snapshot

The 2026-05-26 reference retune preserved `-14 LUFS` delivery landing and
compressor-mode semantics. External references were hotter than YES Master
exports; that is expected for this slice.

Observed aggregate after the retune:

```text
universal  ref -10.53 LUFS, YES -14.00 LUFS, DR gap -0.31 LU, warnings dynamic_range_low|comp_density_on_compressed_source
clarity    ref -12.04 LUFS, YES -14.00 LUFS, DR gap -0.50 LU, warnings dynamic_range_low|comp_density_on_compressed_source
oomph      ref -11.87 LUFS, YES -14.00 LUFS, DR gap -1.10 LU, warnings dynamic_range_low|comp_density_on_compressed_source
tape       ref  -9.91 LUFS, YES -14.00 LUFS, DR gap -0.82 LU, warnings dynamic_range_low|comp_density_on_compressed_source
```

Oomph remains the least-matched preset and needs careful listening before any
further subjective retune.

## Current Gaps

1. Manual listening signoff is still required; automated tests cannot approve
   taste.
2. The full private fixture matrix runner evidence is complete; listening
   signoff on already-mastered outputs remains.
3. Oomph needs listening notes before another targeted tuning pass.
4. The already-integrated autoupdater still needs an end-to-end update proof.
   Its permanent key is configured. Paid Apple notarization and Windows
   Authenticode are post-beta trust upgrades; the $0 beta may ship ad-hoc /
   unsigned installers with explicit OS-warning guidance (D16, 2026-07-20).
   Store-style distribution remains deferred.
5. Cloud-placeholder sources (OneDrive Files-On-Demand, Dropbox online-only)
   are untested territory: a dehydrated file can stall decode/analysis for
   the duration of a network hydration, and no unit test can construct one
   without a cloud provider. Path SHAPES are pinned (UNC shares, verbatim
   `\\?\` paths, >260-char nesting — `src-tauri/tests/portability_paths.rs`);
   hydration BEHAVIOR awaits the owner's exploratory pass
   (docs/OWNER_SMOKE_TEST.html #13) and its notes.
6. Sessions store absolute source paths, so a `.ams.json` moved to another
   machine restores tracks whose sources are missing (honest per-track
   recovery errors, no crash). A "relink missing sources" affordance is
   backlog (IDEAS_BACKLOG), not built.
