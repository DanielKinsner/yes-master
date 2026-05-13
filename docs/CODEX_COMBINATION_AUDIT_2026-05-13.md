# Codex Combination Audit - 2026-05-13

## Overall Take

This repo is the right foundation to combine with the deeper Codex mastering logic. It already avoids the worst failure mode of the older app: there is no Python DSP path, no Web Audio approximation path, and the main mastered audition runs through the same Rust `MasteringChain` used by export.

The core architecture is promising:

- Native Rust DSP is centralized in `src-tauri/src/dsp.rs`.
- Realtime mastered playback streams source PCM through `MasteringSource` in `src-tauri/src/audio.rs`.
- Slider changes are pushed to the running chain through `update_chain`.
- Export uses the same `MasteringChain` in `src-tauri/src/engine.rs`.
- Dither is real TPDF at integer output depths, with tests.
- LUFS metering is no longer a casual approximation; the Rust meters are BS.1770-oriented and tested.

The remaining problems are not "throw it away" problems. They are mostly truth-in-product problems: some surfaces still imply render-preview workflow, session restore is too sticky, first master playback can still block on full decode, and export QC is not measuring the actual rendered output before reporting.

## Verification Run

Commands run from `C:\Users\SM - Dan\Documents\GitHub\album-mastering-studio-claude-build`:

- `npm run build` passed.
- `cargo test` passed after clearing a stale locked Windows test process.

Note: the first `cargo test` attempt hit `LNK1104` because `target/debug/deps/contracts-ef136c4bdf47e34c.exe` was still held open by stale PID `28212`. After stopping that process, the full Rust suite passed: 74 lib tests, 2 album render tests, and 39 contracts tests. The real-fixture tests took roughly 275 seconds.

## Findings

### P0 - Export receipts use source analysis as mastered-output truth

Location:

- `src/hooks/useTrackMaster.ts:890-902`
- `src-tauri/src/engine.rs:1589-1622`

`exportMaster` renders a mastered WAV, but then builds `ExportReport` from `selectedAnalysis`, which is the source track analysis:

- `measured_lufs: selectedAnalysis.lufs_integrated`
- `measured_true_peak_dbtp: selectedAnalysis.true_peak_dbtp`
- `measured_dynamic_range_lu: selectedAnalysis.dynamic_range_lu`
- `sample_rate: 44_100`

That means the export receipt and quality warnings can describe the original file, not the rendered master. This is the highest-risk mismatch because it makes the app sound more rigorous than the evidence it is showing.

Recommendation:

Make `render_track_master` return post-render measurements, or immediately analyze the rendered output path before calling `run_export_checks`. The report should use rendered LUFS, rendered true peak, rendered dynamic range, actual output sample rate, and effective bit depth.

### P1 - First Mastered playback can still block on full decode

Location:

- `src-tauri/src/audio.rs:535-555`
- `src-tauri/src/audio.rs:883-915`
- `src/hooks/useTrackMaster.ts:917-933`

The master play path sends a command to the audio thread and waits up to 15 seconds for a reply. On cache miss, `handle_play_master` calls `decode_full(path)` before it creates the `MasteringSource` and starts playback. The single-entry decode cache helps repeated toggles, but the first Mastered click on a long WAV can still feel like a freeze.

This matters because the product promise is instant A/B. If Mastered takes a perceptible first-load stall, the user loses trust even if subsequent cached toggles are fast.

Recommendation:

Separate "prepare decoded PCM for realtime audition" from "press play", or stream decode into the DSP source instead of requiring full-file decode before the sink starts. A practical near-term shape is: when a track is selected/imported, start background decode and waveform prep; only enable Mastered once the realtime source is ready. Longer term, a streaming decode source would avoid full-album memory and first-click stalls.

### P1 - Startup restores tracks automatically, which fights "fresh install" testing

Location:

- `src/hooks/useTrackMaster.ts:211-266`
- `src/hooks/useTrackMaster.ts:268-287`

The app automatically loads the autosaved session on mount, restores tracks, then re-analyzes and regenerates waveforms. That is useful for a mature DAW-style project workflow, but it is exactly why test sessions feel haunted by yesterday's song.

Recommendation:

Change startup to open clean by default. Offer an explicit "Restore last project" action, or show a non-blocking restore banner. Autosave can still exist, but it should not silently load audio files into a supposedly fresh session.

### P1 - Primary UI still contains a render-preview mental model

Location:

- `src/App.tsx:1595-1648`

The main status/control strip says mastered playback is live, but it also shows:

- `Rendering preview WAV...`
- `Render audit WAV`
- `Re-render audit WAV`
- a debug-style `live: applied/attempts` badge

That is much better than "Native Preview" versus "Live Preview", but it still puts the user back in the old question: "Do I need to render something before I can compare?"

Recommendation:

Keep the main workflow brutally simple:

1. Play/Pause.
2. Original/Mastered.
3. Volume Match.
4. Loop/Clear.
5. Export.

Move "Render audit WAV" into an export/tools drawer, rename it as a secondary utility, and remove live update counters from the normal user surface. If live update diagnostics are still useful, gate them behind a dev/debug flag.

### P2 - Sample-rate controls are exposed but not honored

Location:

- `src/App.tsx:1878-1888`
- `src-tauri/src/types.rs:160-169`
- `src-tauri/src/types.rs:232-235`
- `src-tauri/src/engine.rs:1620-1622`

The UI exposes target sample rate options, and delivery profiles carry output sample-rate intent, but the renderer writes `pcm.sample_rate` regardless of profile or `advanced.target_sample_rate`.

This is correctly documented in `types.rs`, but the UI does not show that caveat. A user can choose CD or 48 kHz and receive the source sample rate instead.

Recommendation:

Either hide/disable sample-rate selection until SRC is implemented, or add high-quality SRC and make `write_wav` use the effective output sample rate. Do not keep an active-looking control that cannot change the file.

### P2 - Stale A/B preparation command remains on the Tauri surface

Location:

- `src-tauri/src/audio.rs:54-68`
- `src/lib/api.ts:107-117`

`prepare_ab_preview` still returns synthetic handles and a hard-coded `-2.4 dB` volume-match offset. It does not appear to drive the simplified current UI, but leaving it callable keeps an old "prepare preview" concept in the backend contract.

Recommendation:

Delete the command and API wrapper if it is unused. If an A/B preparation command is needed later, it should prepare the realtime native audition path, not a fake preview handle.

### P2 - Full `cargo test` is green but slow enough to discourage frequent use

Location:

- `src-tauri/tests/contracts.rs`, real-fixture cases

The real-fixture tests are valuable, but in this run the contracts suite spent over 60 seconds each in `mastering_render_processes_real_fixture_if_present` and `phase_12_1_real_fixture_metering_snapshot`, finishing the contracts binary in about 275 seconds.

Recommendation:

Keep the real-song tests, but split test commands into fast unit/contract checks and opt-in real-fixture smoke checks. The migration bar can still require the slow lane when audio behavior changes, but daily UI/product cleanup should have a fast green path.

## Positive Findings

- The Rust DSP surface is materially better than the dual Python/Web Audio architecture.
- The realtime update mechanism has a real test: `audio::tests::mastering_source_applies_live_coeff_updates_via_channel`.
- The limiter, inter-sample peak checks, compressor behavior, dither, delivery-profile shadowing, LUFS targeting, and session persistence all have meaningful Rust tests.
- Dither is implemented at the final integer WAV write stage, which is the right location.
- The current UI is already much closer to the desired hierarchy than the older multi-preview build: one Play button and an Original/Mastered toggle are present.

## Recommended Next Steps

1. Fix export receipts to measure the rendered master, not the source analysis.
2. Make startup clean by default, with explicit restore.
3. Remove render-preview/audit controls from the main listening workflow.
4. Make first Mastered playback non-blocking through background prepare or streaming decode.
5. Hide sample-rate controls until resampling is real, or implement SRC.
6. Delete unused preview/A-B preparation contracts.

My opinion: keep this repo as the base. It has the right spine. But before layering more mastering intelligence onto it, make the product surface tell one story: press Play, switch Original/Mastered, move controls, hear the Rust chain now.
