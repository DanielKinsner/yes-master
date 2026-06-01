---
title: iPhone-Native Performance & Wiring — Design
status: approved
created: 2026-06-01
origin: brainstorming session 2026-06-01 (review of apps/iphone-native built that day)
scope: apps/iphone-native. Must not change desktop behavior or the shared yes_master_lib DSP — the iPhone app builds on that crate as-is.
---

# iPhone-Native Performance & Wiring — Design

## Problem Frame

`apps/iphone-native` is a native SwiftUI app whose Rust bridge
(`apps/iphone-native/rust/src/lib.rs`) delegates all DSP to the same desktop
crate (`yes_master_lib`, via `rust/Cargo.toml:14`). The mastering math is
therefore correct and identical to desktop. The problems are performance,
UX wiring, and durability — not DSP quality.

1. **Unoptimized DSP on device.** `scripts/build-rust-bridge.sh:9` defaults the
   Rust lib to `PROFILE="debug"` unless the Xcode config is `Release`, and every
   device install in `HANDOFF.md` uses `Debug-iphoneos`. Debug Rust (no inlining,
   no SIMD, overflow checks per op) runs the DSP roughly 10–30× slower than
   release. This is the dominant cause of slow analysis and previews.
2. **Eager, repeated, whole-file rendering.** After import the app runs a
   full-file analysis pass and then *auto-renders the entire master* to disk
   (`ContentView.swift:950`). Every Style tap (`:513`) and Intensity release
   (`:581`) re-renders the whole song again. There is no real-time audition path,
   so "preview" always means a complete offline render (~60 MB WAV for a 4-min
   track).
3. **Double decode before first playback.** `analyzeTrack` and `renderMaster`
   each decode the file separately from disk.
4. **Dead / misleading controls.** The Loudness picker (Low/Med/High) is never
   passed to the render (`currentRenderOptions`, `:849` sends only preset +
   intensity) and never triggers a re-render. The "Volume Match" and
   "LUFS Preview" toggles set `listeningMode` but it is never read to affect audio
   (`:371`). All three are visible controls that do nothing.
5. **Shared "master" can be evicted.** When a preview already exists, Create
   Master reuses the preview URL (`:1040`), which lives in the Caches directory
   (`:841`) — iOS may purge it. The durable `RenderedMasters` dir is effectively
   unused in the common path.
6. **Unbounded storage.** ImportedTracks, RenderedMasters, and MasteredPreviews
   are written and never cleaned up; each preset audition leaves another full WAV.
7. **Analysis paid for, never shown.** `NativeAnalysisResult` (LUFS/true-peak/DR)
   is only used as a "ready" gate; the numbers are never displayed (the
   `HANDOFF.md:72` claim that they are shown is stale after the hero-simplify
   commits).

## Decisions (from brainstorming)

1. **Desktop is untouched (safety rail).** This work must not change desktop
   behavior or the shared `yes_master_lib` DSP. The iPhone app only adds its own
   Rust bridge and Swift layers on top of the existing crate.
2. **Target flow is bare-bones, plain-language (LANDR-like):**
   upload → preset → intensity → loudness → Create Master. No LUFS jargon shown
   to the user.
3. **Loudness targets (export):** Low = **−14 LUFS**, Medium = **−11 LUFS**
   (today's fixed value), High = **−9 LUFS**.
4. **Volume Match = in-app preview/compare only.** A playback gain so Original and
   Mastered can be A/B'd at matched loudness. It **never** changes the export
   level (project non-negotiable).
5. **Remove "LUFS Preview" entirely.** It is jargon and live loudness-dialing
   contradicts quick turnaround.
6. **Keep 4 presets** (Balanced/Warm/Open/Punch) **and the intensity slider** —
   the slider is the manual safety valve against an over-cooked mix.
7. **Processing is an explicit, visible state; the Original/Master toggle is
   always instant.** A control that looks live must behave live. Renders are
   attached to upload / preset / intensity / loudness changes (with motion +
   progress), never to the toggle.
8. **Real-time audition is researched first; 25–30s snippet is the fallback.**
9. **Remove the Tauri `apps/iphone`.** The native app supersedes it with better
   architecture and UI/UX, so the older prototype goes.

## Architecture

### Phase 0 — Optimized dependency build (highest value, lowest risk)

Add to `apps/iphone-native/rust/Cargo.toml` (the crate is its own Cargo root — it
has its own `Cargo.lock` — so profile keys are honored here):

```toml
[profile.dev.package."*"]
opt-level = 3
```

This optimizes the dependencies that do the heavy lifting (`yes_master_lib`,
`symphonia`, `rustfft`, `ebur128`) while leaving the tiny bridge crate
unoptimized for fast compiles. Result: release-speed DSP even in a Debug app
build, with no need to switch Xcode to Release.

- **Headless verification (this machine, Windows host):** `cargo test` in `rust/`
  passes; `cargo build` and `cargo build --release` both succeed; a micro-benchmark
  that times `analyze + render` of a generated WAV in debug vs release quantifies
  the multiplier (x86 proxy for the relative speedup, not absolute device time).
- **Device verification (user, Mac + iPhone):** install and confirm the felt
  speed; optionally also build the Release configuration.
- **Risk:** if Cargo warns that profiles are ignored (would mean the crate is a
  workspace member), fall back to building device installs in Release.

### Phase 1 — Processing-as-state, instant toggle, no wasted work

- **Auto-process on import** with the default preset so a Master is ready to A/B
  immediately. Keep the existing copy-into-app-storage step
  (`ImportedTrackStore.importTrack`).
- **Re-process on preset / intensity / loudness change**, **debounced**
  (≈250–400 ms) with cancellation of any in-flight job (extend the existing
  `previewTask?.cancel()` pattern).
- **Processing UI:** motion + short progress + "Analyzing" / "Mastering" labels
  (reuse/extend `ProcessingSpinner` and `processingBanner`). Disable Create Master
  with a visible explanation while processing.
- **Original/Master toggle only swaps the active playback source** between two
  ready things; it never starts a render. If the master is not ready yet
  (normally it is, because we processed proactively), the toggle shows a preparing
  state instead of freezing.
- **Single decode** reused across analysis + preview where feasible, to remove the
  double-decode.

### Phase 2 — Real-time audition spike, with snippet fallback

**Spike goal:** determine whether the mastered chain can be auditioned live on
iOS. This is high value because real-time makes preset/intensity/loudness
changes *and* the toggle all instant, with no preview file until export.

Proposed real-time architecture:

- **New Rust FFI** (persistent chain) in `rust/src/lib.rs` +
  `rust/include/yes_master_native_bridge.h`:
  - `..._chain_create(settings_json) -> *mut handle`
  - `..._chain_process_block(handle, in_ptr, out_ptr, frames)`
  - `..._chain_set_settings(handle, settings_json)` (or recreate on change)
  - `..._chain_seek(handle)` / reset chain state
  - `..._chain_destroy(handle)`
  Wraps a persistent `yes_master_lib` `MasteringChain` so biquad/limiter/
  compressor state flows across blocks (the chain already processes in chunks).
- **Source PCM** decoded once to memory (or streamed) and fed block-by-block.
- **Drive with `AVAudioEngine` + `AVAudioSourceNode`** (iOS 16 target is fine);
  the render callback pulls processed blocks from `chain_process_block`.
- **Loudness landing in real time:** real-time cannot pre-measure whole-file LUFS,
  so use the analysis pass's integrated LUFS to **precompute the constant landing
  gain** that hits the chosen target (−14/−11/−9), and apply it as a constant.
- **Volume Match:** a live gain node, toggled.

**Go / No-Go criteria:**
- RT-safe: no heap allocation, lock, or I/O inside `process_block` (preallocate;
  the chain operates on passed buffers).
- Seeking works (reset chain state + jump source index) without glitches.
- Stable at 44.1/48 kHz and typical buffer sizes on device.

- **GO →** live audition; preset/intensity/loudness and the toggle are all instant;
  no preview files.
- **NO-GO →** **25–30s snippet preview**:
  - New Rust render of a time-bounded region (start, ~25–30 s) to a temp WAV
    (add a duration/offset to the render path, or decode + slice then render).
  - Rendered **proactively and debounced** so the toggle stays instant; full render
    only at Create Master. Snippet region defaults to the current playhead (0 on
    first preview); seeking far outside it triggers a debounced re-render.

### Phase 3 — Correctness + honest controls

- **Durable master.** Create Master always renders to / resolves a file in
  Application Support `RenderedMasters` (`renderedMastersDirectoryURL`, `:834`) and
  shares *that*. Never set `shareMasterURL` to a Caches preview.
- **Storage cleanup.** Bound the three directories — prune superseded preview WAVs
  on each new render; cap retained imports/masters (policy finalized in the plan,
  e.g. keep current session + last N). Preview dir stays in Caches.
- **Wire Loudness.** `NativeRenderOptions` gains a loudness field mapped to
  `advanced.lufs_offset_db` (−14/−11/−9, `DeliveryProfile::Custom`) in
  `export_settings_for_options` (`rust/src/lib.rs:133`); Loudness changes also
  trigger a debounced re-process (today `:618` does not). The same target feeds the
  real-time landing gain.
- **Wire Volume Match (playback-only).** When ON during A/B, attenuate the louder
  side so Original and Mastered play at matched loudness. Gain derived from the
  original's integrated LUFS (analysis) vs the master target. Applied via
  `AVAudioPlayer.volume` (or the engine gain node); never written to any exported
  file.
- **Analysis stays internal-only.** No on-screen LUFS/TP/DR. Used for the landing
  gain + Volume Match. Keep a single decode/measure; if the offline-snippet path
  makes the separate pass redundant, fold measurement into the render result
  (`NativeRenderedMeasurements` is already returned).

### Phase 4 — Dead code / bloat

- Remove `ListeningMode.lufsPreview` and its UI; collapse `listeningMode` to a
  simple `volumeMatchEnabled: Bool` (or keep the enum minus LUFS Preview).
- Single source of truth for supported extensions: drive the Swift
  `knownAudioExtensions` list (`NativeMasteringBridge.swift:45`) off the Rust
  support check, or drop the always-filtered `aiff/aif/opus` literals.
- Use `NativeRenderedMeasurements` (e.g. for Volume Match) or stop decoding it.
- **Remove the Tauri `apps/iphone`** now that the native app is ahead: delete the
  app, the orphaned `iphone:*` scripts in root `package.json`, and its doc
  references (~13k lines, incl. a committed generated Xcode project). Sequence this
  last, after the native improvements land, and confirm the native app builds
  before deletion.

## Data Flow (offline/fallback shown; real-time noted)

upload → copy to ImportedTracks → analyze(original) *(internal LUFS)* → produce
current-settings Master preview *(real-time stream if GO, else 25–30s snippet)* at
{preset, intensity, loudness} → A/B Original vs Mastered (Volume Match optional,
playback-only) → Create Master → full render to **RenderedMasters** → Share.

Real-time path replaces "produce preview" with a live engine and removes preview
files entirely.

## Error Handling

- Keep existing import hardening (empty / fake-WAV / unavailable source) in
  `ImportedTrackStore` and the friendly decode-error message
  (`friendlyAudioErrorMessage`).
- Processing failure shows a short plain message and reverts to Original.
- Real-time: if the engine fails to start, fall back to the snippet path (or a
  clear message) rather than silently doing nothing.

## Testing

- **Headless (this machine):** `cargo test` in `rust/` incl. new FFI; debug-vs-
  release benchmark; host build check. New Rust tests for: loudness mapping
  (Low/Med/High → −14/−11/−9), time-bounded snippet render writes a shorter WAV,
  real-time chain FFI create/process/seek/destroy (if pursued).
- **Swift unit tests:** processing → ready → instant-toggle state transitions;
  toggle never triggers a render; Volume Match gain math; Create Master URL is in
  RenderedMasters, not Caches; cleanup prunes superseded previews; Loudness change
  re-processes.
- **Device (user):** real-time feasibility result, on-device speed, full
  import → preset → intensity → loudness → Create Master → Share loop.

## Verification (after implementation)

```powershell
# Bridge (headless, host)
cd apps/iphone-native/rust
cargo test
cargo build
cargo build --release

# Cross-compile + device build + Swift tests (user, on Mac)
# cd apps/iphone-native/rust
# cargo check --target aarch64-apple-ios
# cd apps/iphone-native
# xcodegen generate
# xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative \
#   -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Plus the manual device loop above, confirming the toggle is instant, processing is
clearly shown, Loudness changes the master, Volume Match changes only playback, and
the shared file survives (not a cache file).

## Files Touched

- `apps/iphone-native/rust/Cargo.toml` — `[profile.dev.package."*"]` (Phase 0).
- `apps/iphone-native/rust/src/lib.rs` — loudness in render options; snippet/
  real-time FFI; (+ tests).
- `apps/iphone-native/rust/include/yes_master_native_bridge.h` — new FFI symbols.
- `apps/iphone-native/YESMasterNative/ContentView.swift` — flow, processing states,
  instant toggle, remove LUFS Preview, wire Loudness + Volume Match, durable master,
  debounce.
- `apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift` — render options
  (loudness), new bridge calls; use measurements.
- `apps/iphone-native/YESMasterNative/TrackPlaybackController.swift` — Volume Match
  gain; real-time engine controller if GO (likely a new file).
- `apps/iphone-native/YESMasterNativeTests/*` — new state/gain/cleanup tests.
- `apps/iphone-native/HANDOFF.md` — update status; correct the stale "shows LUFS"
  note.
- `apps/iphone/**` + root `package.json` `iphone:*` scripts — removed (Phase 4).
