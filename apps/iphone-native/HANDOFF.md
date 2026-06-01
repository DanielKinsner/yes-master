# YES Master iPhone Native - Handoff

_Last updated: 2026-06-01, XcodeGen/linking/Swift-to-Rust review fixes._

## Current Status

`apps/iphone-native` is a separate SwiftUI-native scaffold. The existing Tauri iPhone app in `apps/iphone` is untouched and remains the reference/prototype.

Review fixes now landed on `main` in small commits:

1. `6ac86c7 fix(iphone-native): make xcodegen reproducible`
2. `49eaca8 fix(iphone-native): link Rust bridge libraries`
3. `c7f394a feat(iphone-native): call Rust bridge from Swift`

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
- `YESMasterNative/YESMasterNative-Bridging-Header.h`: imports the Rust C ABI header for Swift.
- `rust/`: Rust bridge crate depending on `yes_master_lib`.
- `rust/include/yes_master_native_bridge.h`: C ABI header for Swift integration.
- `scripts/build-rust-bridge.sh`: builds the static Rust library for device and simulator:
  - device: `aarch64-apple-ios`
  - simulator: `aarch64-apple-ios-sim` and `x86_64-apple-ios`, combined with `lipo`

`NativeMasteringBridge` now calls Rust for:

- bridge version
- supported import extension filtering
- fixed export settings JSON

The native app is still not a real mastering app yet. Import, analyze, original/mastered playback, render, and share/export are not wired.

## Next Slice

1. Add native document import with supported types only: WAV, MP3, M4A/AAC, FLAC, OGG/Vorbis.
2. Copy imported files into app-owned storage before analysis/render.
3. Add the Rust analyze bridge call using `yes_master_lib::engine::analyze_tracks`.
4. Add native playback with `AVAudioSession` activation immediately before play.
5. Add render/export bridge call using the shared Rust engine and the fixed `-11 LUFS`, `44.1 kHz`, `24-bit`, `-1 dBTP` export target.

## Verification So Far

Verified in this pass:

```bash
cd rust
cargo test
```

```bash
cd rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

```bash
cd apps/iphone-native
xcodegen generate
git status --short
```

`xcodegen generate` leaves git clean after committed files are up to date. The generated `.xcodeproj` is intentionally ignored and reproducible from `project.yml`.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build
```

Simulator build succeeds and links `-lyes_master_iphone_native_bridge` from `rust/build/iphonesimulator/Debug`.

```bash
cd apps/iphone-native
PROJECT_DIR="$PWD" PLATFORM_NAME=iphoneos CONFIGURATION=Debug ARCHS=arm64 scripts/build-rust-bridge.sh
lipo -info rust/build/iphoneos/Debug/libyes_master_iphone_native_bridge.a
```

Device Rust bridge output is arm64.
