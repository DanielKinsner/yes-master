# iPhone-Native Live Audition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the native iPhone app audition like Track Master: import/analyze once, play Original immediately, hear Mastered live, hear Style/Intensity/Loudness changes without full-song preview renders, and render the final WAV only when Create Master is tapped.

**Architecture:** Swift owns the iPhone audio output with `AVAudioEngine` and one `AVAudioSourceNode`. Rust owns the mastering math and the persistent live stream state. The desktop `AudioPlayer` is the behavioral reference, not the iPhone output engine; preview WAVs are only a temporary safety fallback until live audition is proven on device.

**Tech Stack:** SwiftUI, AVFoundation, XCTest, Rust C ABI, pure Rust DSP core, Symphonia decode, ebur128 loudness measurement, Rubato resampling, Hound WAV writing, XcodeGen, Cargo.

---

## Revision Summary

This plan replaces the earlier "try desktop `AudioPlayer` first" path.

Claude's clean-slate critique is folded in where it improves the final product:

- Extract the mastering DSP into a portable Rust core crate.
- Keep `rodio`, `cpal`, and Tauri out of the iPhone live playback path.
- Let Swift own the iOS output graph.
- Give Rust a persistent live handle with one decoded PCM buffer and one frame cursor.
- Switch Original/Mastered with a bypass flag, not with two players or two files.
- Update Style/Intensity/Loudness by changing live DSP parameters, not by rendering a new WAV.
- Keep Volume Match audition-only and structurally absent from export.
- Keep Create Master as the only full-song render.

What I am keeping from our earlier plan:

- Fix the preview cleanup bug first because it explains the current phone symptom.
- Preserve desktop behavior and DSP sound as the reference.
- Verify on a real iPhone early, before building too much UI around a theory.
- Keep the old file-preview path only as a temporary escape hatch, not the final UX.

## Current Evidence

- Current native app playback is file-based:
  - `apps/iphone-native/YESMasterNative/TrackPlaybackController.swift` uses `AVAudioPlayer(contentsOf:)`.
  - Original plays the imported source file.
  - Mastered plays `masteredPreviewURL`.
  - If the preview file is missing, Mastered cannot play.
- Current native app still renders full-song preview files:
  - Import analyzes, then calls `prepareMasteredPreview`.
  - Style and Loudness schedule `prepareMasteredPreview`.
  - Intensity schedules `prepareMasteredPreview` when the slider is released.
- The current "preview ready, then Mastered fails" bug has a concrete likely cause:
  - `RenderStorage.pruneObsoletePreviews(keeping:)` compares raw `URL` values.
  - Apple paths can refer to the same file as `/var/...` and `/private/var/...`.
  - Raw equality says those differ, while `resolvingSymlinksInPath()` says they match.
  - So the just-rendered preview can be deleted immediately after the app says it is ready.
- Commit `cc3bee881fce8154919f495ab927163c8df20453` was not live either:
  - It rendered one full-file preview for current settings.
  - It did not pre-render all four presets.
  - It reused that cache preview as the share file, which later work correctly stopped doing.
- Desktop Track Master already has the target behavior:
  - It streams PCM through `MasteringChain`.
  - It updates coefficients while playback continues.
  - It crossfades settings changes over about 512 frames.
- Desktop uses `rodio` over `cpal`.
  - That is fine for Mac/Windows desktop.
  - It is not the right first-choice output owner for a native iPhone app.

## Definition Of Done

- Clean install on the user's iPhone removes old app data.
- Import WAV and MP3 successfully.
- Original playback starts after import/analyze without waiting for a mastered render.
- Tapping Mastered uses the same playhead and does not require a preview WAV.
- Original/Mastered switching preserves playhead while playing and paused.
- Style changes are audible while playback continues.
- Intensity slider movement is audible during movement, without a render loop.
- Loudness changes are audible live or near-live.
- Volume Match is off by default, audition-only, and never changes Create Master output.
- Create Master performs the full-song mastered WAV render and shares a durable file in `RenderedMasters`.
- The app never says "Mastered preview ready" for a missing cache file.
- The iPhone source-node playback path does not require `rodio`, `cpal`, or Tauri. If the temporary analyze/export control plane still pulls them into the final binary, that must be recorded as remaining extraction debt before release.
- Desktop build/tests still pass after shared Rust changes.

## Product Boundary

The iPhone app is a separate app. Do not convert the desktop app, and do not let mobile work weaken the desktop Track Master.

The iPhone v1 remains Simple-only:

- Styles: Balanced, Warm, Open, Punch.
- Intensity.
- Loudness: Low, Medium, High.
- Original/Mastered.
- Volume Match.
- Create Master.

No Advanced mode, album workflow, AUv3 plugin, public signing, or new preset-retuning work belongs in this plan.

## Chosen Architecture

### Layer 1: `yes-dsp`

Create a portable Rust crate for the mastering chain:

- DSP coefficients.
- Preset curves.
- Compressor/limiter/saturation/width chain.
- Mastering settings needed by the DSP.
- Ceiling-bounded loudness landing math.
- Block processing.

Rules:

- No Tauri.
- No rodio.
- No cpal.
- No file I/O.
- No decode.
- No export.
- No JSON in the realtime callback.
- `serde` can be feature-gated for desktop/control-plane contracts, but the DSP crate itself must be able to build without app-shell dependencies.

### Layer 2: Offline Audio Core

Keep heavier non-realtime work outside the live callback:

- Decode/import support.
- Analysis.
- Full-song export.
- ebur128 full-song measurement.
- Rubato resampling.
- WAV writing.
- Export LUFS landing.

This can start by staying in `src-tauri` while extraction is underway, but the final iPhone bridge should not need the desktop `AudioPlayer` dependency tree just to analyze or export.

### Layer 3: Rust Realtime Handle

Add a hand-written C ABI for live audition:

- `create_from_file`: decode once, allocate buffers, create DSP state.
- `process`: fill output frames for the source node.
- `set_bypass`: switch Original/Mastered without changing playhead.
- `set_params`: reconstruct full Simple-mode `MasteringSettings` from Style/Intensity/Loudness off the audio callback.
- `set_volume_match`: enable/disable Rust-computed audition-only match gain.
- `seek_frame`: move the one shared cursor.
- `snapshot`: report position/meters to Swift.
- `destroy`: free all Rust-owned memory.

Realtime rules:

- The render callback does not allocate.
- The render callback does not lock.
- The render callback does not log.
- The render callback does not parse JSON.
- The render callback does not call Swift async/actors.
- Parameter changes are computed off the callback and crossfaded into the stream.
- The render callback calls the exported C function directly with a raw handle pointer. It does not call a Swift object method.

### Layer 4: Swift Audio Engine

Swift owns:

- `AVAudioSession`.
- `AVAudioEngine`.
- One `AVAudioSourceNode`.
- Play/pause.
- UI state.
- Import/share UI.

Rust owns:

- Decoded PCM buffer.
- Frame cursor.
- DSP chain.
- Bypass state.
- Live settings state.
- Representative-window loudness and Volume Match gain state.
- Audition meters.

### Why `AVAudioSourceNode`

Use `AVAudioSourceNode` for v1 because it is the simplest native iOS output path that still gives us a realtime render callback.

Do not start with AUv3:

- AUv3 is useful later if YES Master becomes a plugin.
- It adds extension packaging and host/plugin complexity that does not help the first iPhone app.

Do not start with raw AudioUnit:

- It is lower-level and useful only if `AVAudioSourceNode` fails timing, format, or latency requirements on device.

## Loudness Strategy

Live audition and final export should feel aligned, but they do not need to be identical workloads.

Live:

- Measure a representative fixed window through the chain for each settings version, around 8 seconds.
- Cache that measurement until the source, Style, Intensity, Loudness, seek-selection policy, or delivery rate changes.
- Do not use a continuously rolling landing gain; it can drift between intro/chorus and sound like pumping.
- Run the representative measurement at export's `effective_sample_rate`, using the same resample-before-measure ordering as export.
- Route live landing through the same `ceiling_bounded_landing_delta_db` math as export.
- Crossfade from the previous cached live scalar to the new cached live scalar.
- Make Low/Medium/High audible quickly.
- Avoid blocking playback.

Export:

- Decode/process the full file.
- Resample to the delivery rate before final loudness measurement.
- Measure the full render.
- Apply final LUFS landing with true-peak/ceiling protection.
- Write the durable WAV.

The final export is authoritative. Live loudness is for audition trust and quick creative feedback.

## Volume Match Strategy

Volume Match is audition-only.

Implementation:

- It starts off.
- Rust computes the match gain from representative Original and Mastered loudness windows.
- Swift only toggles Volume Match on/off; it does not calculate or pass the gain.
- The enabled state applies as smoothed gain in the live stream.
- It affects Original/Mastered listening fairness.
- It is not included in Create Master settings.
- Export code continues to force Volume Match off.

## Retired From The Earlier Plan

Do not implement these old-plan ideas as the primary path:

- Do not make desktop `AudioPlayer` the iPhone live playback engine.
- Do not rely on `rodio`/`cpal` to own phone output.
- Do not solve audition by rendering preview WAVs.
- Do not pre-render every preset/loudness/intensity combination.
- Do not build AUv3 for v1.

The desktop live player remains valuable as the proof of behavior and as code to borrow concepts from: one cursor, live chain updates, and short crossfades.

## File Structure

- Modify: `apps/iphone-native/YESMasterNative/RenderStorage.swift`
  - Normalize preview URLs before pruning.
- Modify test: `apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift`
  - Cover `/var` vs `/private/var` equivalent paths.
- Create: `crates/yes-dsp/Cargo.toml`
  - Portable DSP crate metadata.
- Create: `crates/yes-dsp/src/lib.rs`
  - Public exports for settings, coefficients, and chain processing.
- Create: `crates/yes-dsp/src/landing.rs`
  - Pure ceiling-bounded loudness landing math shared by live audition and export.
- Create: `crates/yes-dsp/src/settings.rs`
  - Minimal DSP settings and preset types.
- Create: `crates/yes-dsp/src/dsp.rs`
  - The mastering chain from `src-tauri/src/dsp.rs`.
- Modify: `src-tauri/Cargo.toml`
  - Depend on `yes-dsp`.
- Modify: `src-tauri/src/dsp.rs`
  - Re-export the extracted DSP, or shrink this file after moving the implementation.
- Modify: `src-tauri/src/types.rs`
  - Re-export or adapt DSP settings so desktop contracts remain stable.
- Modify: `apps/iphone-native/rust/Cargo.toml`
  - Depend on `yes-dsp` for realtime processing.
  - Stop needing `yes_master_lib` for the live callback path.
- Create: `apps/iphone-native/rust/src/live_stream.rs`
  - Rust-owned decoded PCM, cursor, DSP chain, bypass, params, and realtime process function.
- Modify: `apps/iphone-native/rust/src/lib.rs`
  - Export live-stream C ABI symbols.
- Modify: `apps/iphone-native/rust/include/yes_master_native_bridge.h`
  - Add typed live-stream C ABI declarations.
- Create: `apps/iphone-native/YESMasterNative/LiveAudioEngine.swift`
  - Own `AVAudioEngine`, `AVAudioSourceNode`, and audio-session activation.
- Create: `apps/iphone-native/YESMasterNative/LiveAuditionBridge.swift`
  - Thin Swift wrapper around live-stream FFI.
- Create: `apps/iphone-native/YESMasterNative/AuditionController.swift`
  - State machine for Original/Mastered, playhead, settings, Volume Match, and Create Master separation.
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`
  - Stop auto-rendering previews for audition.
  - Delegate playback and live settings to `AuditionController`.
  - Keep Create Master as offline render/share.
- Create tests: `apps/iphone-native/YESMasterNativeTests/AuditionControllerTests.swift`
  - Prove toggling/settings behavior without a real audio device.
- Create tests: `apps/iphone-native/YESMasterNativeTests/LiveAudioEngineTests.swift`
  - Prove engine lifecycle calls the bridge in the right order.
- Update docs: `apps/iphone-native/HANDOFF.md`, `docs/IPHONE_APP.md`, `docs/IPHONE_APP_OVERVIEW.md`
  - Replace "mastered preview render" language with live audition language.

---

## Task 1: Fix Preview Cleanup Safety

This fixes the current phone symptom even though live audition will remove preview dependence later.

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/RenderStorage.swift`
- Modify: `apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift`

- [ ] **Step 1: Add failing URL-normalization coverage**

Add this test to `RenderStorageTests`:

```swift
func testPruneObsoletePreviewsKeepsSymlinkEquivalentCurrentURL() throws {
    let storage = RenderStorage(baseDirectory: root)
    try FileManager.default.createDirectory(at: storage.previewsDirectory, withIntermediateDirectories: true)

    let keep = storage.previewsDirectory.appendingPathComponent("keep.wav")
    let drop = storage.previewsDirectory.appendingPathComponent("drop.wav")
    try Data([0]).write(to: keep)
    try Data([0]).write(to: drop)

    let listedKeep = try XCTUnwrap(
        FileManager.default.contentsOfDirectory(at: storage.previewsDirectory, includingPropertiesForKeys: nil)
            .first { $0.lastPathComponent == "keep.wav" }
    )
    let alternateKeep = URL(fileURLWithPath: listedKeep.path.replacingOccurrences(of: "/private/var", with: "/var"))

    storage.pruneObsoletePreviews(keeping: alternateKeep)

    XCTAssertTrue(FileManager.default.fileExists(atPath: keep.path))
    XCTAssertFalse(FileManager.default.fileExists(atPath: drop.path))
}
```

- [ ] **Step 2: Run the test and confirm it fails before the fix**

Run:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Expected before fix: `testPruneObsoletePreviewsKeepsSymlinkEquivalentCurrentURL` fails because `keep.wav` is deleted.

- [ ] **Step 3: Normalize URLs in pruning**

Change `RenderStorage.pruneObsoletePreviews(keeping:)` to compare resolved URLs:

```swift
func pruneObsoletePreviews(keeping current: URL?) {
    let current = current?.resolvingSymlinksInPath()
    guard let files = try? fileManager.contentsOfDirectory(
        at: previewsDirectory,
        includingPropertiesForKeys: nil
    ) else { return }
    for file in files where file.pathExtension.lowercased() == "wav" {
        guard file.resolvingSymlinksInPath() != current else { continue }
        try? fileManager.removeItem(at: file)
    }
}
```

- [ ] **Step 4: Run Swift tests**

Run:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Expected: all Swift tests pass.

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/YESMasterNative/RenderStorage.swift apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift
git commit -m "fix(iphone-native): keep current preview across Apple path aliases"
```

---

## Task 2: Extract Portable DSP Core

This creates the shared sound core before changing phone playback.

**Files:**
- Create: `crates/yes-dsp/Cargo.toml`
- Create: `crates/yes-dsp/src/lib.rs`
- Create: `crates/yes-dsp/src/landing.rs`
- Create: `crates/yes-dsp/src/settings.rs`
- Create: `crates/yes-dsp/src/dsp.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/dsp.rs`
- Modify: `src-tauri/src/types.rs`

- [ ] **Step 1: Create the crate shell**

Create `crates/yes-dsp/Cargo.toml`:

```toml
[package]
name = "yes-dsp"
version = "0.1.0"
edition = "2021"
rust-version = "1.77"

[lib]
name = "yes_dsp"

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }

[features]
default = []
serde = ["dep:serde"]
```

Create `crates/yes-dsp/src/lib.rs`:

```rust
pub mod dsp;
pub mod landing;
pub mod settings;

pub use dsp::{ChainCoeffs, MasteringChain};
pub use landing::ceiling_bounded_landing_delta_db;
pub use settings::*;
```

- [ ] **Step 2: Add shared loudness landing math**

Create `crates/yes-dsp/src/landing.rs`:

```rust
pub fn ceiling_bounded_landing_delta_db(
    measured_lufs: f32,
    measured_true_peak_dbtp: f32,
    target_lufs: f32,
    ceiling_dbtp: f32,
) -> f32 {
    if !target_lufs.is_finite() || !measured_lufs.is_finite() || measured_lufs <= -70.0 {
        return 0.0;
    }
    let delta_db = target_lufs - measured_lufs;
    let headroom_db = (ceiling_dbtp - measured_true_peak_dbtp).max(0.0);
    let applied_delta_db = if delta_db < 0.0 {
        delta_db
    } else {
        delta_db.min(headroom_db)
    };
    if applied_delta_db.abs() > 1.0e-4 {
        applied_delta_db
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downward_landing_applies_full_delta() {
        assert_eq!(ceiling_bounded_landing_delta_db(-10.0, -1.0, -14.0, -1.0), -4.0);
    }

    #[test]
    fn upward_landing_is_clamped_by_headroom() {
        assert_eq!(ceiling_bounded_landing_delta_db(-10.0, -3.0, -6.0, -1.0), 2.0);
    }

    #[test]
    fn silent_or_non_finite_inputs_apply_no_gain() {
        assert_eq!(ceiling_bounded_landing_delta_db(-80.0, -60.0, -14.0, -1.0), 0.0);
        assert_eq!(ceiling_bounded_landing_delta_db(f32::NAN, -1.0, -14.0, -1.0), 0.0);
        assert_eq!(ceiling_bounded_landing_delta_db(-10.0, -1.0, f32::NAN, -1.0), 0.0);
    }
}
```

- [ ] **Step 3: Move only DSP-owned settings into `yes-dsp`**

Create `crates/yes-dsp/src/settings.rs` with the DSP-facing types currently used by `src-tauri/src/dsp.rs`:

- `Preset`
- `DeliveryProfile`
- `CompressionMode`
- `AdvancedSettings`
- `MasteringSettings`
- compressor band settings used by the chain

Keep app/project/export-only types in `src-tauri/src/types.rs`.

- [ ] **Step 4: Move DSP implementation**

Move the mastering chain implementation from `src-tauri/src/dsp.rs` into `crates/yes-dsp/src/dsp.rs`.

In the moved file, replace:

```rust
use crate::types::*;
```

with:

```rust
use crate::settings::*;
```

- [ ] **Step 5: Re-export for desktop compatibility**

Shrink `src-tauri/src/dsp.rs` to:

```rust
pub use yes_dsp::dsp::*;
```

In `src-tauri/src/types.rs`, re-export the shared DSP settings so existing desktop callers do not churn all at once:

```rust
pub use yes_dsp::settings::{
    AdvancedSettings, CompressionMode, DeliveryProfile, MasteringSettings, Preset,
};
```

In `src-tauri/src/engine.rs`, use the shared landing function:

```rust
use yes_dsp::ceiling_bounded_landing_delta_db;
```

- [ ] **Step 6: Wire desktop dependency**

Add to `src-tauri/Cargo.toml`:

```toml
yes-dsp = { path = "../crates/yes-dsp", features = ["serde"] }
```

- [ ] **Step 7: Verify the core has no app-output dependencies**

Run:

```bash
cargo tree --manifest-path crates/yes-dsp/Cargo.toml
```

Expected: no `tauri`, no `rodio`, no `cpal`.

- [ ] **Step 8: Verify desktop still compiles and tests**

Run:

```bash
cd src-tauri
cargo test --lib
cargo test
```

Expected: tests pass.

- [ ] **Step 9: Commit**

```bash
git add crates/yes-dsp src-tauri/Cargo.toml src-tauri/src/dsp.rs src-tauri/src/types.rs src-tauri/src/engine.rs
git commit -m "refactor: extract portable mastering dsp core"
```

---

## Task 3: Add Rust Live Stream Handle

This is the heart of live audition. It does not own audio output; it fills buffers when Swift asks.

**Files:**
- Modify: `apps/iphone-native/rust/Cargo.toml`
- Create: `apps/iphone-native/rust/src/live_stream.rs`
- Modify: `apps/iphone-native/rust/src/lib.rs`
- Modify: `apps/iphone-native/rust/include/yes_master_native_bridge.h`

- [ ] **Step 1: Add `yes-dsp` to the iPhone bridge**

In `apps/iphone-native/rust/Cargo.toml`, add:

```toml
yes-dsp = { path = "../../../crates/yes-dsp", features = ["serde"] }
```

- [ ] **Step 2: Define typed realtime ABI**

Add declarations to `apps/iphone-native/rust/include/yes_master_native_bridge.h`:

```c
typedef struct YMRealtimeHandle YMRealtimeHandle;

typedef struct YMRealtimeStatus {
    int code;
    double position_sec;
    float peak_dbfs;
    float lufs_momentary;
    float live_landing_gain_db;
    float volume_match_gain_db;
    unsigned int sample_rate;
    unsigned int channels;
} YMRealtimeStatus;

typedef struct YMRealtimeCreateOptions {
    unsigned int output_sample_rate;
    unsigned int output_channels;
    unsigned int delivery_sample_rate;
} YMRealtimeCreateOptions;

YMRealtimeHandle *yes_master_native_live_create_from_file(
    const char *source_path,
    YMRealtimeCreateOptions options
);
void yes_master_native_live_destroy(YMRealtimeHandle *handle);
YMRealtimeStatus yes_master_native_live_process_f32(
    YMRealtimeHandle *handle,
    float *output_interleaved,
    unsigned int frame_count
);
YMRealtimeStatus yes_master_native_live_seek_frame(YMRealtimeHandle *handle, unsigned long long frame);
YMRealtimeStatus yes_master_native_live_set_bypass(YMRealtimeHandle *handle, bool bypassed);
YMRealtimeStatus yes_master_native_live_set_params(
    YMRealtimeHandle *handle,
    const char *preset,
    float intensity,
    float lufs_target
);
YMRealtimeStatus yes_master_native_live_set_volume_match(YMRealtimeHandle *handle, bool enabled);
YMRealtimeStatus yes_master_native_live_snapshot(YMRealtimeHandle *handle);
```

- [ ] **Step 3: Implement the handle**

Create `apps/iphone-native/rust/src/live_stream.rs`.

The handle owns:

- decoded interleaved `f32` PCM
- output sample rate
- output channel count
- delivery/effective sample rate used for representative loudness measurement
- atomic frame cursor
- atomic bypass flag
- DSP chain
- current and pending coefficients
- 512-frame crossfade state
- representative Original loudness and peak
- representative Mastered loudness and peak
- cached live landing gain, computed with `ceiling_bounded_landing_delta_db`
- smoothed audition gain for Volume Match
- meter state

The `process_f32` function:

- reads from the decoded PCM buffer at the current frame cursor
- if bypassed, writes Original frames to output while still advancing a warm Mastered chain into scratch so Original/Mastered toggles remain click-free
- if not bypassed, processes frames through `MasteringChain`
- applies audition-only Volume Match when enabled
- advances the one shared cursor
- returns zeros after end-of-file
- performs no allocation and no lock inside the callback path

Settings behavior:

- `set_params(preset, intensity, lufs_target)` is intentionally Simple-only for v1.
- Rust reconstructs the full `MasteringSettings` from those three values.
- Future Advanced-mode support must replace this with a full typed settings struct; do not stretch the Simple ABI.
- `set_params` starts representative-window measurement off the render callback and keeps the previous cached live landing gain until the new one is ready.

Volume Match behavior:

- `set_volume_match(handle, enabled)` toggles only the enabled state.
- Rust computes `volume_match_gain_db` from representative Original and Mastered loudness.
- If no representative measurement is ready, Volume Match uses unity gain and reports `volume_match_gain_db = 0.0`.
- Swift does not compute or pass Volume Match gain.

- [ ] **Step 4: Export the module**

In `apps/iphone-native/rust/src/lib.rs`, add:

```rust
mod live_stream;
```

- [ ] **Step 5: Add Rust tests**

Add tests that prove:

- handle creation rejects missing files with a non-zero status
- bypass processing returns source samples
- mastered processing changes samples for a non-neutral preset/intensity
- seek changes the cursor
- toggling bypass preserves cursor
- changing params does not require recreating the handle
- changing params keeps old live landing gain until the new representative measurement is ready
- Volume Match gain is computed in Rust from representative Original/Mastered measurements
- `process_f32` returns interleaved stereo output in the expected channel order

- [ ] **Step 6: Run Rust checks**

Run:

```bash
cd apps/iphone-native/rust
cargo test
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

Expected: both pass.

- [ ] **Step 7: Verify the iPhone live path avoids desktop output deps**

Run:

```bash
cd apps/iphone-native/rust
cargo tree --target aarch64-apple-ios -i cpal
cargo tree --target aarch64-apple-ios -i rodio
```

Expected final state: neither `cpal` nor `rodio` is required by the live playback path. If they still appear only because analyze/export temporarily depends on `yes_master_lib`, record that as remaining control-plane extraction work before final release.

- [ ] **Step 8: Commit**

```bash
git add apps/iphone-native/rust/Cargo.toml apps/iphone-native/rust/src/lib.rs apps/iphone-native/rust/src/live_stream.rs apps/iphone-native/rust/include/yes_master_native_bridge.h
git commit -m "feat(iphone-native): add rust live stream processor"
```

---

## Task 4: Add Swift Live Audio Engine

Swift owns iPhone output and asks Rust for frames.

**Files:**
- Create: `apps/iphone-native/YESMasterNative/LiveAuditionBridge.swift`
- Create: `apps/iphone-native/YESMasterNative/LiveAudioEngine.swift`
- Create tests: `apps/iphone-native/YESMasterNativeTests/LiveAudioEngineTests.swift`

- [ ] **Step 1: Wrap the FFI**

Create `LiveAuditionBridge.swift` with Swift methods:

- `load(url:)`
- `destroy()`
- `seek(frame:)`
- `setBypass(_:)`
- `setParams(style:intensity:loudness:)`
- `setVolumeMatch(_:)`
- `snapshot()`
- `rawHandleForRenderCallback()`

Only this file should touch raw C pointers outside the render callback setup. The production render callback must capture the raw `YMRealtimeHandle` pointer and call `yes_master_native_live_process_f32` directly, not call a Swift bridge method.

- [ ] **Step 2: Build `LiveAudioEngine`**

Create `LiveAudioEngine.swift`.

Responsibilities:

- configure `AVAudioSession`
- create one `AVAudioEngine`
- create one `AVAudioSourceNode`
- pin the node format to interleaved `Float32` with one `AudioBufferList` buffer
- before starting, assert `mNumberBuffers == 1` and `mBuffers[0].mNumberChannels == channelCount`
- in the source node render callback, call `yes_master_native_live_process_f32(rawHandle, outputPointer, frameCount)` directly
- if the callback receives an unexpected buffer layout, write silence and report an error status for diagnostics
- start/stop/pause the engine
- expose playhead and meter snapshots
- never create separate Original and Mastered players
- handle `AVAudioSession.interruptionNotification` by pausing safely and allowing resume
- handle `AVAudioSession.routeChangeNotification` by pausing or rebuilding the engine when the output route/sample rate changes

- [ ] **Step 3: Add lifecycle tests**

Tests should use a fake bridge and prove:

- loading a track creates one live handle
- play starts the engine once
- pause does not destroy the handle
- Original/Mastered toggle calls `setBypass`
- seek calls `seek(frame:)`
- settings changes call `setParams`
- Volume Match calls `setVolumeMatch`
- the render callback path calls the C function directly with a raw handle
- the engine refuses to start with a non-interleaved or unexpected buffer layout
- interruption notification pauses playback without destroying the loaded handle
- route-change notification pauses or rebuilds the engine without losing the loaded source

- [ ] **Step 4: Run Swift tests and generic iOS build**

Run:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build
```

Expected: tests pass and build succeeds.

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/YESMasterNative/LiveAuditionBridge.swift apps/iphone-native/YESMasterNative/LiveAudioEngine.swift apps/iphone-native/YESMasterNativeTests/LiveAudioEngineTests.swift
git commit -m "feat(iphone-native): add avaudioengine live output"
```

---

## Task 5: Add AuditionController

This is the Swift state machine that keeps playback behavior sane.

**Files:**
- Create: `apps/iphone-native/YESMasterNative/AuditionController.swift`
- Create tests: `apps/iphone-native/YESMasterNativeTests/AuditionControllerTests.swift`

- [ ] **Step 1: Create controller state**

`AuditionController` owns:

- current imported track URL
- selected side: Original or Mastered
- selected style
- intensity
- loudness
- Volume Match state
- play/pause state
- current playhead
- live engine
- offline render bridge for Create Master

- [ ] **Step 2: Define behavior**

Controller behavior:

- Import/analyze loads the live handle but does not render a mastered preview.
- Play starts from the current playhead.
- Original/Mastered changes call `setBypass` and preserve cursor.
- Style changes call `setParams`.
- Intensity movement calls `setParams`.
- Loudness changes call `setParams`.
- Volume Match calls `setVolumeMatch`.
- Create Master calls the existing offline render path and writes a durable WAV.

- [ ] **Step 3: Add tests**

Tests should prove:

- import does not render a preview
- switching to Mastered does not require `masteredPreviewURL`
- toggling side preserves playhead
- intensity changes do not call render
- style changes do not call render
- loudness changes do not call render
- Create Master is the only path that calls render
- Volume Match is not included in export settings

- [ ] **Step 4: Run tests**

Run:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/YESMasterNative/AuditionController.swift apps/iphone-native/YESMasterNativeTests/AuditionControllerTests.swift
git commit -m "feat(iphone-native): add live audition controller"
```

---

## Task 6: Wire ContentView To Live Audition

Remove preview-render behavior from normal listening.

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`
- Create tests: `apps/iphone-native/YESMasterNativeTests/ContentViewLiveAuditionTests.swift`

- [ ] **Step 1: Replace playback state**

Remove normal-audition dependence on:

- `masteredPreviewURL`
- `previewTask`
- `previewDebounceTask`
- `isPreparingMasterPreview`
- `prepareMasteredPreview`
- `scheduleMasteredPreviewRefresh`

Keep:

- `shareMasterURL`
- `renderTask`
- `isRendering`

- [ ] **Step 2: Connect controls**

Wire UI controls to `AuditionController`:

- play button calls controller play/pause
- Original/Mastered segmented control calls side switch
- Style cards call settings update
- Intensity slider calls settings update during movement
- Loudness picker calls settings update
- Volume Match checkbox calls audition-only volume match
- Create Master calls offline render

- [ ] **Step 3: Update status copy**

The UI should no longer say mastered preview is preparing or ready.

Use states like:

- `Ready`
- `Playing Original`
- `Playing Mastered`
- `Rendering Master`
- `Master Ready`

- [ ] **Step 4: Run tests and build**

Run:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build
```

Expected: tests pass and build succeeds.

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/YESMasterNative/ContentView.swift apps/iphone-native/YESMasterNativeTests
git commit -m "feat(iphone-native): wire ui to live audition"
```

---

## Task 7: Representative Loudness And Volume Match

Make loudness and Volume Match useful without pretending live audition is the final export pass.

**Files:**
- Modify: `apps/iphone-native/rust/src/live_stream.rs`
- Modify tests under `apps/iphone-native/rust/src/`
- Modify: `apps/iphone-native/YESMasterNative/AuditionController.swift`
- Modify or delete: `apps/iphone-native/YESMasterNative/VolumeMatch.swift`
- Modify: `apps/iphone-native/YESMasterNativeTests/VolumeMatchTests.swift`

- [ ] **Step 1: Add representative loudness measurement**

In Rust live stream state, measure a representative fixed window through the active chain whenever the source or settings version changes.

Target:

- around 8 seconds
- measured at export's `effective_sample_rate`
- resampled before measurement when the delivery/effective rate differs from the source or output rate
- cached until the settings version changes
- non-blocking
- does not drift as the song plays
- does not use a rolling gain that changes between intro and chorus
- reset or soften after seek only if the representative-window policy deliberately follows the new listening region

- [ ] **Step 2: Apply a live landing scalar**

When Loudness changes:

- recompute chain params off the callback
- measure the representative Mastered window
- compute the live landing scalar with shared `ceiling_bounded_landing_delta_db`
- keep the previous scalar until the new measurement is ready
- crossfade the gain change

- [ ] **Step 3: Compute Volume Match in Rust**

Volume Match behavior:

- measure representative Original loudness
- measure representative Mastered loudness after live landing
- compute one attenuation gain for the louder side
- cache `volume_match_gain_db`
- Swift only toggles enabled/disabled
- if measurement is unavailable, use unity gain and report `0.0 dB`
- stop using Swift-side `volumeMatchGainDb` / `volumeMatchLinearGain` for live playback

- [ ] **Step 4: Keep export separate**

Confirm Create Master still uses full-song export measurement and landing.

Add a test that Volume Match is false in export settings even when audition Volume Match is true.

- [ ] **Step 5: Add Rust tests**

Add tests that prove:

- representative loudness is cached and does not drift while playback advances
- changing Style/Intensity/Loudness invalidates the cached measurement
- live landing uses the same `ceiling_bounded_landing_delta_db` output as export for the same measured LUFS/true-peak inputs
- representative measurement uses the effective delivery sample rate, not blindly the source rate
- Volume Match gain is computed in Rust from Original/Mastered representative loudness
- Volume Match never amplifies above unity

- [ ] **Step 6: Run checks**

Run:

```bash
cd apps/iphone-native/rust
cargo test
cd ..
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Expected: tests pass.

- [ ] **Step 7: Commit**

```bash
git add apps/iphone-native/rust/src/live_stream.rs apps/iphone-native/rust/src/lib.rs apps/iphone-native/YESMasterNative/AuditionController.swift
git add apps/iphone-native/YESMasterNative/VolumeMatch.swift apps/iphone-native/YESMasterNativeTests/VolumeMatchTests.swift
git commit -m "feat(iphone-native): add live loudness audition"
```

---

## Task 8: Device Spike Gate

Do this before polishing UI or removing fallback code.

**Files:**
- No source changes unless the device test reveals a bug.

- [ ] **Step 1: Build for the phone**

With the phone connected:

```bash
xcrun devicectl list devices
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS,id=<DEVICE_ID>' -configuration Debug build
```

Expected: build succeeds.

- [ ] **Step 2: Clean install**

```bash
xcrun devicectl device uninstall app --device <DEVICE_ID> com.yesmaster.iphone.native
xcrun devicectl device install app --device <DEVICE_ID> "<path to YES Master Native.app>"
xcrun devicectl device process launch --device <DEVICE_ID> com.yesmaster.iphone.native
```

Expected: app launches with old data removed.

- [ ] **Step 3: Manual spike script**

On the phone:

1. Import WAV.
2. Play Original.
3. Switch to Mastered while playing.
4. Move Intensity.
5. Change Style.
6. Change Loudness.
7. Toggle Volume Match.
8. Seek or restart playback.
9. Unplug/replug headphones or change output route if available.
10. Interrupt playback with an iOS interruption test if available.
11. Repeat with MP3.
12. Try one long track and record memory behavior.

Expected:

- audio starts quickly
- no full preview render is triggered
- Original/Mastered preserves playhead
- settings are audible while playing
- no clicks, crashes, or stuck silence
- headphone/route changes pause or recover cleanly
- long tracks do not create unacceptable memory pressure

- [ ] **Step 4: Record result**

Update `apps/iphone-native/HANDOFF.md` with:

- device model
- iOS version
- build configuration
- files tested
- pass/fail notes
- any audible glitches
- peak memory note for the long-track test

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/HANDOFF.md
git commit -m "docs(iphone-native): record live audition device spike"
```

---

## Task 9: Remove Preview-Render UX Debt

Only do this after the device spike passes.

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`
- Modify: `apps/iphone-native/YESMasterNative/RenderStorage.swift`
- Modify docs: `apps/iphone-native/HANDOFF.md`, `docs/IPHONE_APP.md`, `docs/IPHONE_APP_OVERVIEW.md`

- [ ] **Step 1: Remove normal preview generation**

Delete or isolate:

- preview debounce
- preview render state
- preview-ready status language
- Mastered playback from cache URL

- [ ] **Step 2: Keep final render storage**

Keep `RenderedMasters` for Create Master output.

Ensure exports never overwrite:

- the source file
- prior renders
- current preview/fallback files

- [ ] **Step 3: Update docs**

Docs should say:

```text
The iPhone app auditions Mastered live. Changing Style, Intensity, or Loudness does not render a full preview file. Create Master is the full render/export path.
```

- [ ] **Step 4: Run checks**

Run:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build
```

Expected: tests pass and build succeeds.

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/YESMasterNative apps/iphone-native/HANDOFF.md docs/IPHONE_APP.md docs/IPHONE_APP_OVERVIEW.md
git commit -m "refactor(iphone-native): remove preview render audition path"
```

---

## Task 10: Final Verification

**Fast checks:**

```bash
cd apps/iphone-native/rust
cargo test
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
cd ..
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build
```

**Desktop safety checks after shared Rust changes:**

```bash
npm test
npm run build
cd src-tauri
cargo test --lib
cargo test
```

**Phone acceptance:**

1. Clean install.
2. Import MP3.
3. Play Original.
4. Switch to Mastered.
5. Move Intensity continuously.
6. Change Style.
7. Change Loudness.
8. Toggle Volume Match.
9. Tap Create Master.
10. Share Master.
11. Repeat with WAV.

Expected final user-facing result:

```text
The phone app now auditions Mastered live; changing Style, Intensity, or Loudness no longer kicks off full-song preview renders. Create Master is the full render.
```

## Risk Register

- DSP extraction can sprawl if app/project/export types move with it. Keep `yes-dsp` limited to the chain and its settings.
- Realtime callbacks are unforgiving. Any allocation, lock, log, JSON parse, or Swift async hop in the callback is a bug.
- Live loudness is an audition approximation. Final export loudness remains full-song and authoritative.
- Debug builds can make heavy audio work look worse than release. Optimize Rust dependencies for device builds and test on real hardware.
- `panic=abort` and `catch_unwind` are not a rescue pair. Use boundary validation so panics do not cross FFI; if release uses `panic=abort`, assume a panic ends the process.
- If `AVAudioSourceNode` fails device timing, only then consider raw AudioUnit.
- `AVAudioSourceNode` buffer layout must stay pinned to interleaved `Float32`; deinterleaved output without conversion will corrupt stereo.
- Keeping the Mastered chain warm during Original playback costs CPU but avoids cold-toggle clicks. If device profiling says it is too expensive, replace it with a reset plus short crossfade and re-run click tests.
- Decode-once PCM can create memory pressure on long tracks. The spike must record memory behavior; if it is too high, switch to chunked decode/ring-buffer streaming before shipping.
- iOS route changes and interruptions are normal user behavior, not edge cases. The live engine must pause/rebuild cleanly when headphones, speakers, or phone interruptions change the output path.

## The Bet

Tauri is not the reason desktop can preview quickly. The fast part is streaming decoded PCM through Rust DSP while playback continues. The correct iPhone version should copy that behavior, not the desktop shell.

The best plan is therefore:

1. Fix the current cache bug.
2. Extract the DSP math.
3. Put iOS-native audio output in Swift.
4. Put live processing state in Rust.
5. Prove it on the phone early.
6. Remove preview-render audition once live playback works.
