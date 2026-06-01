# YES Master iPhone Native - Handoff

_Last updated: 2026-06-01, native import/export hardening._

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
13. `eb302c2 style(iphone-native): match reference iPhone shell`
14. `1a161f1 feat(iphone-native): wire create master action`
15. `5ced404 feat(iphone-native): preserve audition switch position`
16. `eda9d9f feat(iphone-native): add share master action`
17. `0896f6c fix(iphone-native): require full screen portrait`
18. `5d7dc05 test(iphone-native): verify Swift Rust render path`
19. `ae6eb3a feat(iphone-native): prepare mastered previews`
20. `2c8d3ce style(iphone-native): use YES Master icon artwork`
21. `98f7214 style(iphone-native): simplify hero controls`
22. `bfad0fd fix(iphone-native): clarify unreadable wav imports`
23. `09f0c57 fix(iphone-native): handle unavailable imports`
24. `f4cb3a5 test(iphone-native): cover render output safety`
25. `673b93a fix(iphone-native): clean up rejected imports`

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

The main SwiftUI screen has been restyled to match the existing Tauri iPhone reference: dark premium shell, large central import/play panel, Volume Match and LUFS Preview controls, track card, Original/Mastered switch, Style cards, Loudness selector, and Create Master button.

The Rust and Swift bridge can now render a master into an output directory using the fixed iPhone target (`-11 LUFS`, `44.1 kHz`, `24-bit`, `-1 dBTP`). Rust creates a unique WAV path each time so previous renders are not overwritten.

The `Create Master` button is now wired to the Swift render bridge. After a render succeeds, the app stores the rendered WAV path, switches to Mastered, and allows Mastered playback.

Original/Mastered switching now uses a mastered preview render that starts after analysis. This lets the user switch to Mastered before sharing/exporting, and switching while audio is playing tries to preserve playback position.

Native share/export has a first pass: after a master render succeeds, a `Share Master` button appears and shares the rendered WAV through the iOS share sheet.

The iPhone UI has a preset intensity slider. Preset and intensity are passed through the Swift bridge to the shared Rust mastering settings.

The native app now uses the YES Master desktop icon artwork for the iPhone app icon, the header brand mark, and a very faint play-panel watermark behind the central play/import button.

The latest UI cleanup restores the import/play area as the main hero, moves Volume Match and LUFS Preview back into that hero as small checkboxes, removes the separate Listening/Step 4 section, removes the redundant `IMPORT` label above `No track loaded`, hides the loudness metadata line, and shortens preset copy to one-line labels.

WAV support was checked after a device screenshot showed `no suitable format reader found`. Normal WAV files analyze successfully, and a new Rust test confirms the native bridge can analyze one of its own rendered 24-bit WAV files. The likely issue is a specific selected file that had a `.wav` name but did not contain readable WAV audio, or was an empty/cloud-placeholder file after import. Native import now catches empty files and `.wav` files that do not look like WAV audio before analysis, and decode failures show a short plain-language message instead of the long Rust decoder text.

Latest no-human hardening adds coverage for missing/deleted source files, rejects unavailable imports with a short message, and removes copied junk from app storage when validation fails. The Rust bridge now also has direct tests proving native renders do not overwrite the source file when rendering beside it and that repeated rendered WAVs stay unique.

Three track-placement mockups were generated outside git because they were based on a user screenshot:

- `/tmp/yes-master-mockups/mockup-a-hero-file-chip.png`
- `/tmp/yes-master-mockups/mockup-b-header-subline.png`
- `/tmp/yes-master-mockups/mockup-c-bottom-context-line.png`

It is still not a full iPhone mastering app yet: the full import -> analyze -> render -> mastered playback -> share loop still needs hands-on testing on the user's real phone with a supported audio file.

A simulator test now sends a generated WAV through the Swift bridge into Rust analysis/rendering and confirms Rust creates a 44.1 kHz, 24-bit WAV output. This reduces the risk that the UI is only talking to placeholders.

## Lunch Test Plan

When the user is home and the phone is connected:

1. Install the native app from `apps/iphone-native`.
2. Import a supported file: WAV, MP3, M4A, AAC, FLAC, or OGG.
3. Confirm analysis completes.
4. Wait for `Mastered preview ready`.
5. Play Original, then switch Original/Mastered while playing.
6. Adjust Style or Intensity, wait for the preview to rebuild, and A/B again.
7. Tap `Create Master`, then `Share Master`, and confirm the WAV can be saved or shared.

## Next Slice

1. Add automated UI/state coverage for stale share URLs after failed render and settings changes.
2. Add automated audition-state coverage for Mastered requested before preview is ready.
3. Manually test import -> analyze -> Create Master -> Mastered playback -> Share Master on the real iPhone when the user has a device connected.
4. Pick one subtler track-title placement from the three mockups only if the user asks to revisit placement.
5. Consider disabling Create Master with a visible explanation while analysis is pending or failed.
6. Keep the UI spacious; do not add waveform/scrubbing unless the user asks.

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

Latest test result after audition switching and share action: 7 tests passed, 0 failed.

Latest test result after adding the real Swift -> Rust render path check: 8 tests passed, 0 failed.

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
/tmp/yes-master-screenshots/iphone-native-reference-ui.png
/tmp/yes-master-screenshots/iphone-native-pre-phone-test.png
```

Additional verification after the latest slices:

```bash
cd apps/iphone-native/rust
cargo test
```

Rust bridge result: 6 tests passed, 0 failed.

Additional phone-test readiness checks:

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

Result: passed.

```bash
cd apps/iphone-native
xcodegen generate
```

Result: project regenerated and remains reproducible once committed files are up to date.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build
```

Result: iPhone target build succeeded and linked the Rust bridge for `iphoneos`.

Latest simulator launch check:

```bash
xcrun simctl install B3786F0C-1C97-4215-839F-5BC2DC63AAA8 "$HOME/Library/Developer/Xcode/DerivedData/YESMasterNative-hkgdvqoawaqkijbqjgjiapwekgrs/Build/Products/Debug-iphonesimulator/YES Master Native.app"
xcrun simctl launch B3786F0C-1C97-4215-839F-5BC2DC63AAA8 com.yesmaster.iphone.native
xcrun simctl io B3786F0C-1C97-4215-839F-5BC2DC63AAA8 screenshot /tmp/yes-master-screenshots/iphone-native-pre-phone-test.png
```

Result: simulator launched and screenshot captured with the polished reference-style UI.

Latest on-phone feedback follow-up:

```bash
cd apps/iphone-native/rust
cargo test
```

Result: 8 Rust bridge tests passed, 0 failed.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Result: 8 native Swift tests passed, 0 failed.

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

Result: passed.

```bash
cd apps/iphone-native
xcodegen generate
```

Result: project regenerated; only intended source changes were dirty before commit.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'id=00008140-001008D621D3001C' -allowProvisioningUpdates build
xcrun devicectl device install app --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 "$HOME/Library/Developer/Xcode/DerivedData/YESMasterNative-hkgdvqoawaqkijbqjgjiapwekgrs/Build/Products/Debug-iphoneos/YES Master Native.app"
xcrun devicectl device process launch --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 --terminate-existing com.yesmaster.iphone.native
```

Result: signed device build succeeded, installed, and relaunched on the connected iPhone.

Latest simulator screenshot:

```text
/tmp/yes-master-screenshots/iphone-native-preview-slider-ui.png
/tmp/yes-master-screenshots/iphone-native-brand-icon-final.png
/tmp/yes-master-screenshots/iphone-native-simplified-hero.png
```

Latest final icon-polish checks:

```bash
cd apps/iphone-native/rust
cargo test
```

Result: 8 Rust bridge tests passed, 0 failed.

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

Result: passed.

```bash
cd apps/iphone-native
xcodegen generate
```

Result: project regenerated; only the intended icon/UI files were dirty before commit.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Result: 8 native Swift tests passed, 0 failed.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'id=00008140-001008D621D3001C' -allowProvisioningUpdates build
xcrun devicectl device install app --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 "$HOME/Library/Developer/Xcode/DerivedData/YESMasterNative-hkgdvqoawaqkijbqjgjiapwekgrs/Build/Products/Debug-iphoneos/YES Master Native.app"
xcrun devicectl device process launch --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 --terminate-existing com.yesmaster.iphone.native
```

Result: signed device build succeeded, installed, and relaunched on the connected iPhone.

Latest simplification checks:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Result: 8 native Swift tests passed, 0 failed.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'id=00008140-001008D621D3001C' -allowProvisioningUpdates build
xcrun devicectl device uninstall app --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 com.yesmaster.iphone.native
xcrun devicectl device install app --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 "$HOME/Library/Developer/Xcode/DerivedData/YESMasterNative-hkgdvqoawaqkijbqjgjiapwekgrs/Build/Products/Debug-iphoneos/YES Master Native.app"
xcrun devicectl device process launch --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 --terminate-existing com.yesmaster.iphone.native
```

Result: signed device build succeeded. The previous app install was removed first to clear app cache/container, then the current build was installed and launched on the connected iPhone.

Latest WAV/import investigation checks:

```bash
cd apps/iphone-native/rust
cargo test
```

Result: 9 Rust bridge tests passed, 0 failed. New coverage confirms a rendered 24-bit WAV can be analyzed again.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Result: 10 native Swift tests passed, 0 failed. New coverage catches empty files and `.wav` files that do not look like WAV audio.

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

Result: passed.

```bash
cd apps/iphone-native
xcodegen generate
```

Result: project regenerated; only intended source files were dirty before commit.

No TestFlight work was attempted.

Latest final hero-polish pass:

- Landed `3e2756e style(iphone-native): finesse hero layout`.
- The selected track now lives as a small chip inside the hero player area.
- The separate track card was removed to reduce clutter.
- The hero player is taller with more breathing room around the play button.
- Volume Match and LUFS Preview remain as small checkboxes in the hero.
- The intensity slider is visually narrower and lighter.
- Create Master is lower in the normal page flow and no longer overlaps the loudness control.
- Normal analysis/status chatter is hidden; failure text still appears when there is an actual failure.

Latest final hero-polish checks:

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Result: 10 native Swift tests passed, 0 failed.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'id=00008140-001008D621D3001C' -allowProvisioningUpdates build
xcrun devicectl device uninstall app --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 com.yesmaster.iphone.native
xcrun devicectl device install app --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 "$HOME/Library/Developer/Xcode/DerivedData/YESMasterNative-hkgdvqoawaqkijbqjgjiapwekgrs/Build/Products/Debug-iphoneos/YES Master Native.app"
xcrun devicectl device process launch --device 5D9F3B2F-C68D-50E0-A372-DEE3A7A3B610 --terminate-existing com.yesmaster.iphone.native
```

Result: signed device build succeeded. The previous app install was removed first to clear app cache/container, then the current build was installed and launched on the connected iPhone.

Latest screenshot:

- `/tmp/yes-master-screenshots/iphone-native-final-finesse.png`

Latest native hardening pass:

- Landed `09f0c57 fix(iphone-native): handle unavailable imports`.
- Landed `f4cb3a5 test(iphone-native): cover render output safety`.
- Landed `673b93a fix(iphone-native): clean up rejected imports`.
- Missing/deleted import sources now show a short user-facing message instead of a raw file error.
- Empty or fake WAV imports no longer leave copied junk in app-owned storage.
- Native Rust render tests now prove source files are not overwritten and repeated rendered masters stay unique.
- No remote phone install or TestFlight work was attempted.

Latest native hardening checks:

```bash
cd apps/iphone-native/rust
cargo test
```

Result: 11 Rust bridge tests passed, 0 failed.

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check --target aarch64-apple-ios
```

Result: passed.

```bash
cd apps/iphone-native
xcodegen generate
```

Result: project regenerated from `project.yml`.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Result: 12 native Swift tests passed, 0 failed.

```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build
```

Result: simulator build succeeded and linked the Rust bridge.
