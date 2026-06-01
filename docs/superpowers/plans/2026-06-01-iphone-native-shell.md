# iPhone Native Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a separate SwiftUI iPhone app that uses the shared Rust YES Master engine for analysis and rendering.

**Architecture:** Keep `apps/iphone` as the Tauri reference app until the native app proves import, audition, render, and share. Add `apps/iphone-native` beside it. SwiftUI owns iPhone UI, file import, playback, audio session, and share/export; Rust stays responsible for mastering analysis, render settings, export checks, and the mastering math.

**Tech Stack:** SwiftUI, AVFoundation, UniformTypeIdentifiers, ShareLink/file exporter, XcodeGen-style project metadata, Rust staticlib/cdylib/rlib bridge depending on `yes_master_lib`.

---

## File Structure

- `apps/iphone-native/README.md`: short owner-facing description and commands.
- `apps/iphone-native/HANDOFF.md`: living status, guardrails, and next slice.
- `apps/iphone-native/project.yml`: native iOS project shape, separate bundle id, Rust static library hook.
- `apps/iphone-native/YESMasterNative/*.swift`: SwiftUI shell, app state, audio session, and bridge boundary.
- `apps/iphone-native/rust/Cargo.toml`: iPhone-native Rust bridge crate only.
- `apps/iphone-native/rust/src/lib.rs`: C ABI surface over shared YES Master types.
- `apps/iphone-native/rust/include/yes_master_native_bridge.h`: Swift-visible bridge contract.

## Task 1: Scaffold The Separate Native App

**Files:**
- Create: `apps/iphone-native/README.md`
- Create: `apps/iphone-native/HANDOFF.md`
- Create: `apps/iphone-native/project.yml`
- Create: `apps/iphone-native/YESMasterNative/YESMasterNativeApp.swift`
- Create: `apps/iphone-native/YESMasterNative/ContentView.swift`
- Create: `apps/iphone-native/YESMasterNative/AudioSessionController.swift`
- Create: `apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift`

- [x] **Step 1: Create scaffold files only**

Expected result: the native folder exists beside `apps/iphone`; no desktop files change.

- [ ] **Step 2: Generate/open the native Xcode project**

Run from `apps/iphone-native` after XcodeGen is available:

```bash
xcodegen generate
open YESMasterNative.xcodeproj
```

Expected result: a standalone SwiftUI app project with bundle id `com.yesmaster.iphone.native`.

## Task 2: Add The Rust Bridge

**Files:**
- Create: `apps/iphone-native/rust/Cargo.toml`
- Create: `apps/iphone-native/rust/src/lib.rs`
- Create: `apps/iphone-native/rust/include/yes_master_native_bridge.h`

- [x] **Step 1: Add a bridge crate that depends on `yes_master_lib`**

Expected result: Rust can serialize the fixed iPhone export settings from shared YES Master types.

- [x] **Step 2: Add guard tests for supported import formats**

Expected result: AIFF/AIF/Opus are not advertised in the native bridge until decoder support is deliberately added.

- [ ] **Step 3: Build iOS static library output**

Run:

```bash
cd apps/iphone-native/rust
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo build --target aarch64-apple-ios
```

Expected result: `target/aarch64-apple-ios/debug/libyes_master_iphone_native_bridge.a`.

## Task 3: Connect The First Real Native Flow

**Files:**
- Modify: Swift bridge files under `apps/iphone-native/YESMasterNative/`
- Modify: Rust bridge files under `apps/iphone-native/rust/`

- [ ] **Step 1: Swift file importer uses only supported types**

Use WAV, MP3, M4A/AAC, FLAC, and OGG/Vorbis first. Do not advertise AIFF/AIF/Opus until the shared decoder supports them.

- [ ] **Step 2: Swift activates AVAudioSession immediately before playback**

Before `AVAudioPlayer` or `AVAudioEngine` starts, set category `.playback` and activate the session.

- [ ] **Step 3: Rust bridge exposes analyze**

Swift calls the Rust bridge with an app-owned copied file path. Rust calls `yes_master_lib::engine::analyze_tracks`.

- [ ] **Step 4: Rust bridge exposes render**

Swift calls Rust with track path, preset, loudness, and output path. Rust calls the shared render path and returns output metadata.

## Task 4: Keep The Reference App Stable

**Files:**
- Modify only `apps/iphone/**` when intentionally fixing the Tauri reference app.
- Do not change desktop app behavior without explicit review.

- [ ] **Step 1: Run Tauri iPhone checks if reference app is touched**

```bash
npm run iphone:typecheck && npm run iphone:test
cd apps/iphone/src-tauri && cargo test
```

- [ ] **Step 2: Run native bridge checks when native bridge changes**

```bash
cd apps/iphone-native/rust && cargo test
```

## Self Review

- Coverage: separate native app, Rust parity boundary, import-format mismatch, audio session ownership, simple UI direction, and no desktop behavior changes are covered.
- Placeholder scan: remaining unchecked tasks are future execution steps, not missing details for the current scaffold.
- Type consistency: Swift names and Rust bridge names are intentionally stable and mirrored in the C header.
