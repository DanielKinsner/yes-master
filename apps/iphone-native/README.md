# YES Master iPhone Native

This is the new SwiftUI-native iPhone app scaffold. It lives beside the current Tauri iPhone prototype at `apps/iphone`, which remains the working reference until the native app proves itself on device.

## Direction

- SwiftUI owns the iPhone UI.
- Native iOS owns file import, playback, audio-session handling, and share/export surfaces.
- Rust remains the mastering core through the shared `yes_master_lib` engine.
- Do not reimplement mastering, loudness, limiting, export checks, or DSP in Swift.

## First Slice

The first slice is intentionally small:

1. Native app folder and SwiftUI shell.
2. Rust bridge crate that depends on `yes_master_lib`.
3. Supported-import guard that avoids advertising formats the shared decoder does not currently support.
4. Handoff notes for the next agent.

## Local Checks

Run the native bridge checks:

```bash
cd apps/iphone-native/rust
cargo test
```

When iOS target checks are needed, use the rustup iOS-capable toolchain:

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

## Project Generation

`project.yml` is written for XcodeGen. Once XcodeGen is available:

```bash
cd apps/iphone-native
xcodegen generate
open YESMasterNative.xcodeproj
```

Keep generated build output, private audio, and rendered masters out of git.
