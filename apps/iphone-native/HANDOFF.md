# YES Master iPhone Native - Handoff

_Last updated: 2026-06-01, native scaffold slice._

## Current Status

`apps/iphone-native` is a separate SwiftUI-native scaffold. The existing Tauri iPhone app in `apps/iphone` is untouched and remains the reference/prototype.

The native direction is:

1. Import a supported audio file with native iOS document picking.
2. Copy the file into app-owned storage before analysis/render.
3. Use SwiftUI for Style, Loudness, Listening mode, Play, and Create Master.
4. Use native AVFoundation playback and activate `AVAudioSession` immediately before playback.
5. Call Rust only for analysis, mastering/rendering, and export checks.

## Guardrails

- Do not change desktop behavior for this native app.
- Do not reimplement DSP in Swift.
- Do not delete `apps/iphone` until the native app proves the full flow on device.
- Do not advertise AIFF/AIF/Opus in native import until decoder support is deliberately added.
- Keep private audio and rendered masters out of git.

## What Exists Now

- `project.yml`: native iOS project shape using bundle id `com.yesmaster.iphone.native`.
- `YESMasterNative/*.swift`: SwiftUI shell, listening mode model, audio-session controller, and bridge boundary.
- `rust/`: Rust bridge crate depending on `yes_master_lib`.
- `rust/include/yes_master_native_bridge.h`: C ABI header for Swift integration.

## Next Slice

1. Generate/open the Xcode project from `project.yml`.
2. Build the Rust static library for `aarch64-apple-ios`.
3. Link the Rust library and header into the Swift project.
4. Replace `NativeMasteringBridge` placeholder calls with real C ABI calls.
5. Implement native document import with supported types only.

## Verification So Far

Run from this folder:

```bash
cd rust
cargo test
```

Run iOS target check when touching the Rust bridge:

```bash
cd rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```
