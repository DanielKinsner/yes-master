# YES Master iPhone Native - Handoff

_Last updated: 2026-06-01, native import, Rust analysis, original playback, and render bridge path._

## Current Status

`apps/iphone-native` is a separate SwiftUI-native scaffold. The existing Tauri iPhone app in `apps/iphone` is untouched and remains the reference/prototype.

Review and follow-up slices now landed on `main` in small commits:

1. `6ac86c7 fix(iphone-native): make xcodegen reproducible`
2. `49eaca8 fix(iphone-native): link Rust bridge libraries`
3. `c7f394a feat(iphone-native): call Rust bridge from Swift`
4. `8ff9a12 feat(iphone-native): add imported track storage`
5. `f7694ea feat(iphone-native): wire native file import`
6. `d083f72 feat(iphone-native): expose Rust analysis bridge`
7. `f64c1e5 feat(iphone-native): add Swift analysis bridge`
8. `889c24c feat(iphone-native): analyze imported tracks`
9. `93a62d3 feat(iphone-native): add original playback controller`
10. `61a90cb feat(iphone-native): add Rust render bridge`
11. `8504032 feat(iphone-native): add Swift render bridge`
12. `77d9890 fix(iphone-native): render masters to unique paths`

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
- `YESMasterNative/*.swift`: SwiftUI shell, listening mode model, audio-session controller, original playback controller, and bridge boundary.
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
- source track analysis via `yes_master_lib::engine::analyze_tracks`
- master rendering via `yes_master_lib::engine::mastering_render`

The native app can now import a supported audio file into app-owned storage, ask Rust to analyze it, show LUFS, true peak, and dynamic range, then play or pause the imported original track. Playback activates `AVAudioSession` immediately before starting the player.

The Rust and Swift bridge can now render a master into an output directory using the fixed iPhone target (`-11 LUFS`, `44.1 kHz`, `24-bit`, `-1 dBTP`). Rust creates a unique WAV path each time so previous renders are not overwritten.

It is still not a real mastering app yet: the `Create Master` button is not wired to the render bridge, mastered playback is not wired, and native share/export is not wired.

## Next Slice

1. Wire `Create Master` to the Swift render bridge on a background task.
2. Store rendered master paths in app state and enable Mastered playback once render output exists.
3. Preserve playhead when switching Original/Mastered.
4. Add native share/export once render output exists.
5. Keep the UI spacious; do not add waveform/scrubbing unless the user asks.

## Verification So Far

Verified in this pass:

```bash
cd apps/iphone-native/rust
cargo test
```

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

```bash
cd apps/iphone-native
xcodegen generate
```

```bash
cd apps/iphone-native
git status --short
```

`xcodegen generate` leaves git clean after committed files are up to date. The generated `.xcodeproj` is intentionally ignored and reproducible from `project.yml`.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Latest test result after adding the render bridge: 6 tests passed, 0 failed.

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

Screenshot shared during this pass:

```text
/tmp/yes-master-screenshots/iphone-native-current.png
```

No remote iPhone install or TestFlight work was attempted.
