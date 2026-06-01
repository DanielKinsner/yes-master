# iPhone-Native Performance & Wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the native iPhone app fast and honest — run the shared DSP at full speed, make every visible control do what it says, keep masters durable, and drop dead weight — without touching desktop behavior or the DSP math.

**Architecture:** The Swift app calls a thin Rust bridge (`apps/iphone-native/rust`) that delegates to the shared desktop crate `yes_master_lib`. This plan changes only the iPhone bridge + Swift layers. Stage 1 (this document) covers the build fix, control wiring, durability, cleanup, and dead-code removal — all powered by the existing offline render, which the build fix makes fast. Stage 2 (a separate plan, written after Stage 1 lands and we measure on device) covers the processing-UX rework and the real-time-audition spike.

**Tech Stack:** Rust (FFI `staticlib`, `serde_json`, `hound`/`tempfile` dev-deps), Swift / SwiftUI / AVFoundation, XCTest, XcodeGen, Cargo.

---

## Conventions for this plan

- **Branch first.** Do all work on a feature branch (Setup task below), not `main`.
- **Commit trailer.** End every commit message in this plan with:
  `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`
- **Verification environment.** Steps marked **[headless OK]** run on this Windows host (Rust `cargo`). Steps marked **[Mac required]** (Swift tests, `xcodebuild`, device installs, `aarch64-apple-ios` cross-compile) run on the user's Mac. Do not claim a Mac step passed from the host — report it as pending the user's machine.
- **Product module name for tests:** `@testable import YES_Master_Native` (product name "YES Master Native" → underscores).
- The Swift module `ContentView.swift` is a SwiftUI `View` and is not unit-tested in this repo; logic that needs tests is extracted into plain types (`NativeLoudness`, `RenderStorage`, `volumeMatchGainDb`, `TrackPlaybackController`) which ARE unit-tested. `ContentView` wiring is verified by build + the device loop.

---

## File Structure

- `apps/iphone-native/rust/Cargo.toml` — add optimized-dependency profile.
- `apps/iphone-native/rust/src/lib.rs` — add loudness target to render settings + FFI; tests.
- `apps/iphone-native/rust/include/yes_master_native_bridge.h` — FFI signature for loudness.
- `apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift` — `NativeRenderOptions.lufsTarget`; pass through FFI; supported-extension dedup.
- `apps/iphone-native/YESMasterNative/RenderStorage.swift` — **new** testable type owning app storage dirs + pruning.
- `apps/iphone-native/YESMasterNative/VolumeMatch.swift` — **new** pure `volumeMatchGainDb(...)`.
- `apps/iphone-native/YESMasterNative/TrackPlaybackController.swift` — playback volume support.
- `apps/iphone-native/YESMasterNative/ContentView.swift` — `NativeLoudness.lufsTarget`, wire Loudness, debounce, durable Create Master, remove LUFS Preview, Volume Match, use `RenderStorage`.
- `apps/iphone-native/YESMasterNativeTests/*` — new unit tests.
- `package.json` + `apps/iphone/**` + docs — Tauri removal.

---

## Setup Task: Feature branch

- [ ] **Step 1: Create and switch to a feature branch**

Run:
```bash
git checkout -b iphone-native-perf-wiring
git status
```
Expected: on branch `iphone-native-perf-wiring`, clean except the already-untracked spec under `docs/superpowers/specs/`.

- [ ] **Step 2: Commit the approved spec (if not already committed)**

```bash
git add docs/superpowers/specs/2026-06-01-iphone-native-perf-and-wiring-design.md docs/superpowers/plans/2026-06-01-iphone-native-perf-and-wiring.md
git commit -m "docs(iphone-native): add perf/wiring design + plan"
```

---

## Task 1: Build fix — optimized dependencies + timing proxy (Phase 0)

**Files:**
- Modify: `apps/iphone-native/rust/Cargo.toml`
- Modify (tests): `apps/iphone-native/rust/src/lib.rs`

- [ ] **Step 1: Add the optimized-dependency profile** **[headless OK]**

In `apps/iphone-native/rust/Cargo.toml`, append at end of file:
```toml
# Optimize dependencies (yes_master_lib, symphonia, rustfft, ebur128) even in
# Debug app builds, so on-device DSP runs at release speed. The bridge crate
# itself stays unoptimized for fast iteration.
[profile.dev.package."*"]
opt-level = 3
```

- [ ] **Step 2: Confirm both profiles still build** **[headless OK]**

Run:
```bash
cd apps/iphone-native/rust
cargo build
cargo build --release
```
Expected: both succeed. If Cargo prints `profiles for the non root package will be ignored`, the crate is a workspace member — fall back: revert this change and instead document building device installs with `CONFIGURATION=Release` (the build script already switches to `--release` then). Record which path was taken.

- [ ] **Step 3: Add an ignored timing test (debug-vs-release proxy)** **[headless OK]**

In `apps/iphone-native/rust/src/lib.rs`, inside `mod tests`, add:
```rust
    #[test]
    #[ignore = "timing proxy; run with --release -- --ignored --nocapture"]
    fn timing_proxy_analyze_and_render() {
        use std::time::Instant;
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("timing.wav");
        let output_dir = tmp.path().join("out");
        write_long_sine_wav(&input, 30); // 30 seconds, 44.1k stereo

        let path = CString::new(input.to_string_lossy().as_bytes()).unwrap();
        let t0 = Instant::now();
        let analysis = unsafe { yes_master_native_analyze_file_json(path.as_ptr()) };
        let analyze_ms = t0.elapsed().as_millis();
        unsafe { yes_master_native_free_string(analysis) };

        let out = render_master_for_test(&input, &output_dir);
        let render_ms = t0.elapsed().as_millis() - analyze_ms;
        assert!(out.exists());
        eprintln!("TIMING analyze={analyze_ms}ms render={render_ms}ms (30s source)");
    }

    fn write_long_sine_wav(path: &std::path::Path, seconds: u32) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let frames = 44_100 * seconds;
        for n in 0..frames {
            let t = n as f32 / 44_100.0;
            let sample = (t * 220.0 * std::f32::consts::TAU).sin() * 0.2;
            let value = (sample * i16::MAX as f32) as i16;
            writer.write_sample(value).unwrap();
            writer.write_sample(value).unwrap();
        }
        writer.finalize().unwrap();
    }
```

- [ ] **Step 4: Run the proxy in debug and release; record both** **[headless OK]**

Run:
```bash
cd apps/iphone-native/rust
cargo test timing_proxy_analyze_and_render -- --ignored --nocapture
cargo test --release timing_proxy_analyze_and_render -- --ignored --nocapture
```
Expected: both PASS and print a `TIMING ...` line. The release line should be markedly faster than debug (this is the x86 proxy for the on-device speedup; absolute device numbers come from the user). Note both numbers in the task's completion comment.

- [ ] **Step 5: Confirm the existing suite still passes** **[headless OK]**

Run:
```bash
cd apps/iphone-native/rust
cargo test
```
Expected: all existing tests PASS (the ignored timing test is skipped).

- [ ] **Step 6: Commit**

```bash
git add apps/iphone-native/rust/Cargo.toml apps/iphone-native/rust/src/lib.rs
git commit -m "perf(iphone-native): optimize bridge dependencies in debug builds"
```

---

## Task 2: Loudness data path through the bridge (Phase 3)

**Files:**
- Modify: `apps/iphone-native/rust/src/lib.rs`
- Modify: `apps/iphone-native/rust/include/yes_master_native_bridge.h`
- Modify: `apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift`
- Modify (tests): `apps/iphone-native/rust/src/lib.rs` (`mod tests`)

- [ ] **Step 1: Write the failing Rust test for loudness mapping** **[headless OK]**

In `apps/iphone-native/rust/src/lib.rs` `mod tests`, add:
```rust
    #[test]
    fn native_options_map_loudness_target() {
        let low = export_settings_for_options(Some("balanced"), 0.5, -14.0);
        assert_eq!(low.effective_target_lufs(), Some(-14.0));

        let high = export_settings_for_options(Some("balanced"), 0.5, -9.0);
        assert_eq!(high.effective_target_lufs(), Some(-9.0));

        // out-of-range is clamped to a safe mastering window
        let clamped = export_settings_for_options(Some("balanced"), 0.5, 5.0);
        assert_eq!(clamped.effective_target_lufs(), Some(-6.0));
    }
```

- [ ] **Step 2: Run it; verify it fails to compile (arity mismatch)** **[headless OK]**

Run:
```bash
cd apps/iphone-native/rust
cargo test native_options_map_loudness_target
```
Expected: FAIL — `export_settings_for_options` takes 2 args, not 3.

- [ ] **Step 3: Add the loudness parameter to the settings builder**

In `apps/iphone-native/rust/src/lib.rs`, change `fixed_export_settings` and `export_settings_for_options`:
```rust
fn fixed_export_settings() -> MasteringSettings {
    export_settings_for_options(Some("balanced"), 0.5, -11.0)
}

fn export_settings_for_options(
    preset: Option<&str>,
    intensity: f32,
    lufs_target: f32,
) -> MasteringSettings {
    MasteringSettings {
        preset: native_preset(preset),
        intensity: intensity.clamp(0.0, 1.0),
        // ...unchanged eq_* / volume_match / gains / delivery_profile / album...
        advanced: AdvancedSettings {
            lufs_offset_db: Some(lufs_target.clamp(-24.0, -6.0)),
            // ...unchanged remaining advanced fields...
        },
    }
}
```
(Keep every other field exactly as today; only the signature, the `lufs_offset_db` line, and the `fixed_export_settings` call change.)

- [ ] **Step 4: Thread the parameter through the FFI**

In `apps/iphone-native/rust/src/lib.rs`:
```rust
#[no_mangle]
pub unsafe extern "C" fn yes_master_native_render_master_json(
    source_path: *const c_char,
    output_dir: *const c_char,
) -> *mut c_char {
    yes_master_native_render_master_with_options_json(
        source_path,
        output_dir,
        std::ptr::null(),
        0.5,
        -11.0,
    )
}

#[no_mangle]
pub unsafe extern "C" fn yes_master_native_render_master_with_options_json(
    source_path: *const c_char,
    output_dir: *const c_char,
    preset: *const c_char,
    intensity: f32,
    lufs_target: f32,
) -> *mut c_char {
    // ...unchanged path/null checks...
    let settings =
        export_settings_for_options(unsafe { ffi_string(preset) }.as_deref(), intensity, lufs_target);
    // ...unchanged create_dir_all + mastering_render...
}
```

- [ ] **Step 5: Update the existing Rust tests that call the changed functions**

In `mod tests`, update the two existing call sites:
```rust
// in native_options_map_to_shared_preset_and_intensity:
let warm = export_settings_for_options(Some("warm"), 0.8, -11.0);
// ...
let open = export_settings_for_options(Some("open"), 2.0, -11.0);
// ...
let fallback = export_settings_for_options(Some("unknown"), -1.0, -11.0);
```
```rust
// in render_master_with_options_writes_wav: add the 5th arg to the FFI call
let pointer = unsafe {
    yes_master_native_render_master_with_options_json(
        input.as_ptr(),
        output_dir.as_ptr(),
        preset.as_ptr(),
        0.9,
        -9.0,
    )
};
```

- [ ] **Step 6: Run Rust tests** **[headless OK]**

Run:
```bash
cd apps/iphone-native/rust
cargo test
```
Expected: all PASS, including `native_options_map_loudness_target`.

- [ ] **Step 7: Update the C header**

In `apps/iphone-native/rust/include/yes_master_native_bridge.h`, change the signature:
```c
char *yes_master_native_render_master_with_options_json(
    const char *source_path,
    const char *output_dir,
    const char *preset,
    float intensity,
    float lufs_target
);
```

- [ ] **Step 8: Add `lufsTarget` to the Swift bridge options and FFI call**

In `apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift`:
```swift
struct NativeRenderOptions: Equatable {
    let preset: String
    let intensity: Float
    let lufsTarget: Float

    static let `default` = NativeRenderOptions(preset: "balanced", intensity: 0.5, lufsTarget: -11)
}
```
And in `renderMaster(...)`, extend the call:
```swift
let pointer = sourceURL.path.withCString { sourcePathPointer in
    outputDirectoryURL.path.withCString { outputDirectoryPointer in
        options.preset.withCString { presetPointer in
            yes_master_native_render_master_with_options_json(
                sourcePathPointer,
                outputDirectoryPointer,
                presetPointer,
                options.intensity,
                options.lufsTarget
            )
        }
    }
}
```

- [ ] **Step 9: Build the bridge for host to confirm Rust still compiles** **[headless OK]**

Run:
```bash
cd apps/iphone-native/rust
cargo build
```
Expected: success. (Swift compile is verified on Mac in Task 3's build step.)

- [ ] **Step 10: Commit**

```bash
git add apps/iphone-native/rust/src/lib.rs apps/iphone-native/rust/include/yes_master_native_bridge.h apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift
git commit -m "feat(iphone-native): pass loudness target through render bridge"
```

---

## Task 3: Wire Loudness in the UI + debounce reprocessing (Phase 1/3)

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`
- Test: `apps/iphone-native/YESMasterNativeTests/NativeLoudnessTests.swift` (create)

- [ ] **Step 1: Write the failing test for the loudness→target map** **[Mac required]**

Create `apps/iphone-native/YESMasterNativeTests/NativeLoudnessTests.swift`:
```swift
import XCTest
@testable import YES_Master_Native

final class NativeLoudnessTests: XCTestCase {
    func testLoudnessMapsToExportTargets() {
        XCTAssertEqual(NativeLoudness.low.lufsTarget, -14)
        XCTAssertEqual(NativeLoudness.medium.lufsTarget, -11)
        XCTAssertEqual(NativeLoudness.high.lufsTarget, -9)
    }
}
```

- [ ] **Step 2: Add `lufsTarget` to `NativeLoudness`**

In `apps/iphone-native/YESMasterNative/ContentView.swift`, extend the enum:
```swift
enum NativeLoudness: String, CaseIterable, Identifiable {
    case low = "Low"
    case medium = "Medium"
    case high = "High"

    var id: String { rawValue }

    var lufsTarget: Float {
        switch self {
        case .low: -14
        case .medium: -11
        case .high: -9
        }
    }
}
```

- [ ] **Step 3: Include loudness in `currentRenderOptions`**

Change `currentRenderOptions` (currently `ContentView.swift:848`):
```swift
private var currentRenderOptions: NativeRenderOptions {
    NativeRenderOptions(
        preset: selectedPreset.bridgeIdentifier,
        intensity: Float(presetIntensity),
        lufsTarget: selectedLoudness.lufsTarget
    )
}
```

- [ ] **Step 4: Add a debounced reprocess and trigger it from all three controls**

In `ContentView`, add state + helper:
```swift
@State private var previewDebounceTask: Task<Void, Never>?

private func scheduleMasteredPreviewRefresh() {
    previewDebounceTask?.cancel()
    previewDebounceTask = Task {
        try? await Task.sleep(nanoseconds: 300_000_000) // 300ms debounce
        guard !Task.isCancelled else { return }
        refreshMasteredPreviewForCurrentSettings()
    }
}
```
Replace the direct `refreshMasteredPreviewForCurrentSettings()` calls in the Style button (`:513`) and the Intensity slider `onEditingChanged` (`:581`) with `scheduleMasteredPreviewRefresh()`. In the Loudness button action (`:617-619`), add the trigger:
```swift
Button {
    selectedLoudness = loudness
    scheduleMasteredPreviewRefresh()
} label: {
```

- [ ] **Step 5: Run the new unit test** **[Mac required]**

Run:
```bash
cd apps/iphone-native
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative \
  -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```
Expected: PASS incl. `NativeLoudnessTests`. (On host, this is pending the user's Mac.)

- [ ] **Step 6: Commit**

```bash
git add apps/iphone-native/YESMasterNative/ContentView.swift apps/iphone-native/YESMasterNativeTests/NativeLoudnessTests.swift
git commit -m "feat(iphone-native): wire loudness control to render + debounce previews"
```

---

## Task 4: Durable master + storage owner (Phase 3)

**Files:**
- Create: `apps/iphone-native/YESMasterNative/RenderStorage.swift`
- Create (test): `apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift`
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`

- [ ] **Step 1: Write the failing test for storage directories + pruning** **[Mac required]**

Create `apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift`:
```swift
import XCTest
@testable import YES_Master_Native

final class RenderStorageTests: XCTestCase {
    private var root: URL!

    override func setUpWithError() throws {
        root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    }
    override func tearDownWithError() throws { try? FileManager.default.removeItem(at: root) }

    func testMastersDirectoryIsDurableNotCaches() {
        let storage = RenderStorage(baseDirectory: root)
        XCTAssertTrue(storage.mastersDirectory.path.contains("RenderedMasters"))
        XCTAssertFalse(storage.mastersDirectory.path.lowercased().contains("caches"))
    }

    func testPruneObsoletePreviewsKeepsOnlyCurrent() throws {
        let storage = RenderStorage(baseDirectory: root)
        try FileManager.default.createDirectory(at: storage.previewsDirectory, withIntermediateDirectories: true)
        let keep = storage.previewsDirectory.appendingPathComponent("keep.wav")
        let drop = storage.previewsDirectory.appendingPathComponent("drop.wav")
        try Data([0]).write(to: keep)
        try Data([0]).write(to: drop)

        storage.pruneObsoletePreviews(keeping: keep)

        XCTAssertTrue(FileManager.default.fileExists(atPath: keep.path))
        XCTAssertFalse(FileManager.default.fileExists(atPath: drop.path))
    }
}
```

- [ ] **Step 2: Run it; verify it fails (no `RenderStorage`)** **[Mac required]**

Run the `xcodebuild ... test` command from Task 3 Step 5. Expected: FAIL — `RenderStorage` undefined.

- [ ] **Step 3: Implement `RenderStorage`**

Create `apps/iphone-native/YESMasterNative/RenderStorage.swift`:
```swift
import Foundation

struct RenderStorage {
    let importsDirectory: URL
    let mastersDirectory: URL
    let previewsDirectory: URL
    private let fileManager: FileManager

    /// Production: imports/masters in Application Support (durable), previews in Caches (evictable).
    init(fileManager: FileManager = .default) {
        let appSupport = fileManager.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let caches = fileManager.urls(for: .cachesDirectory, in: .userDomainMask)[0]
        self.importsDirectory = appSupport.appendingPathComponent("ImportedTracks", isDirectory: true)
        self.mastersDirectory = appSupport.appendingPathComponent("RenderedMasters", isDirectory: true)
        self.previewsDirectory = caches.appendingPathComponent("MasteredPreviews", isDirectory: true)
        self.fileManager = fileManager
    }

    /// Tests: everything under one base directory.
    init(baseDirectory: URL, fileManager: FileManager = .default) {
        self.importsDirectory = baseDirectory.appendingPathComponent("ImportedTracks", isDirectory: true)
        self.mastersDirectory = baseDirectory.appendingPathComponent("RenderedMasters", isDirectory: true)
        self.previewsDirectory = baseDirectory.appendingPathComponent("MasteredPreviews", isDirectory: true)
        self.fileManager = fileManager
    }

    /// Delete every preview WAV except the one currently in use.
    func pruneObsoletePreviews(keeping current: URL?) {
        guard let files = try? fileManager.contentsOfDirectory(
            at: previewsDirectory, includingPropertiesForKeys: nil
        ) else { return }
        for file in files where file != current && file.pathExtension.lowercased() == "wav" {
            try? fileManager.removeItem(at: file)
        }
    }

    /// Keep the newest `max` files in a directory by modification date; delete the rest.
    func enforceLimit(in directory: URL, max: Int) {
        guard let files = try? fileManager.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: [.contentModificationDateKey]
        ) else { return }
        let sorted = files.sorted {
            let a = (try? $0.resourceValues(forKeys: [.contentModificationDateKey]))?.contentModificationDate ?? .distantPast
            let b = (try? $1.resourceValues(forKeys: [.contentModificationDateKey]))?.contentModificationDate ?? .distantPast
            return a > b
        }
        for file in sorted.dropFirst(max) {
            try? fileManager.removeItem(at: file)
        }
    }
}
```

- [ ] **Step 4: Use `RenderStorage` in `ContentView` and make Create Master durable**

In `ContentView`, replace the two computed directory properties (`renderedMastersDirectoryURL` `:834`, `previewMastersDirectoryURL` `:841`) with a single stored instance and references:
```swift
private let renderStorage = RenderStorage()
// outputs:
//   preview render  -> renderStorage.previewsDirectory
//   create master   -> renderStorage.mastersDirectory
```
In `renderMaster()` (`:1034`), **delete the cache-reuse shortcut** (`:1040-1045` — the `if let masteredPreviewURL, !isPreparingMasterPreview { shareMasterURL = masteredPreviewURL ... return }` block) so Create Master always renders fresh into `renderStorage.mastersDirectory`. This guarantees `shareMasterURL` is durable (Application Support), never a Caches preview. Point the render task's `outputDirectoryURL` at `renderStorage.mastersDirectory`, and the preview task's at `renderStorage.previewsDirectory`.

- [ ] **Step 5: Run tests + build** **[Mac required]**

Run the `xcodebuild ... test` command. Expected: `RenderStorageTests` PASS; project builds.

- [ ] **Step 6: Commit**

```bash
git add apps/iphone-native/YESMasterNative/RenderStorage.swift apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift apps/iphone-native/YESMasterNative/ContentView.swift
git commit -m "fix(iphone-native): keep created masters in durable storage"
```

---

## Task 5: Prune storage so it stops growing (Phase 3)

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`
- Test: `apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift` (extend)

- [ ] **Step 1: Add a failing test for `enforceLimit`** **[Mac required]**

Append to `RenderStorageTests`:
```swift
    func testEnforceLimitKeepsNewest() throws {
        let storage = RenderStorage(baseDirectory: root)
        try FileManager.default.createDirectory(at: storage.mastersDirectory, withIntermediateDirectories: true)
        var urls: [URL] = []
        for i in 0..<5 {
            let u = storage.mastersDirectory.appendingPathComponent("m\(i).wav")
            try Data([UInt8(i)]).write(to: u)
            // stagger modification dates
            try FileManager.default.setAttributes(
                [.modificationDate: Date(timeIntervalSince1970: TimeInterval(1000 + i))], ofItemAtPath: u.path
            )
            urls.append(u)
        }
        storage.enforceLimit(in: storage.mastersDirectory, max: 2)
        let remaining = try FileManager.default.contentsOfDirectory(atPath: storage.mastersDirectory.path)
        XCTAssertEqual(Set(remaining), ["m3.wav", "m4.wav"])
    }
```

- [ ] **Step 2: Run it; verify it passes** **[Mac required]**

`enforceLimit` already exists from Task 4. Run `xcodebuild ... test`. Expected: PASS. (If it fails, fix `enforceLimit`, then rerun.)

- [ ] **Step 3: Call pruning at the right moments in `ContentView`**

- After a successful preview render (in `prepareMasteredPreview`'s success branch, after setting `masteredPreviewURL`): `renderStorage.pruneObsoletePreviews(keeping: masteredPreviewURL)`.
- After a successful Create Master (in `renderMaster`'s success branch): `renderStorage.enforceLimit(in: renderStorage.mastersDirectory, max: 20)`.
- After a successful import (in `handleImportResult`'s success branch, after `analyzeImportedTrack(track)`): `renderStorage.enforceLimit(in: renderStorage.importsDirectory, max: 20)`.

- [ ] **Step 4: Run tests + build** **[Mac required]**

Run `xcodebuild ... test`. Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/iphone-native/YESMasterNative/ContentView.swift apps/iphone-native/YESMasterNativeTests/RenderStorageTests.swift
git commit -m "fix(iphone-native): bound imported/rendered/preview storage growth"
```

---

## Task 6: Remove LUFS Preview; wire Volume Match (playback-only) (Phase 3/4)

**Files:**
- Create: `apps/iphone-native/YESMasterNative/VolumeMatch.swift`
- Create (test): `apps/iphone-native/YESMasterNativeTests/VolumeMatchTests.swift`
- Modify: `apps/iphone-native/YESMasterNative/TrackPlaybackController.swift`
- Modify (test): `apps/iphone-native/YESMasterNativeTests/TrackPlaybackControllerTests.swift`
- Modify: `apps/iphone-native/YESMasterNative/ContentView.swift`

- [ ] **Step 1: Failing test for the volume-match gain math** **[Mac required]**

Create `apps/iphone-native/YESMasterNativeTests/VolumeMatchTests.swift`:
```swift
import XCTest
@testable import YES_Master_Native

final class VolumeMatchTests: XCTestCase {
    func testQuieterSideIsUnchangedLouderSideAttenuated() {
        // original -18 LUFS, master -11 LUFS -> reference is the quieter (-18)
        XCTAssertEqual(volumeMatchGainDb(sideLufs: -18, otherLufs: -11), 0, accuracy: 0.0001)   // original: no change
        XCTAssertEqual(volumeMatchGainDb(sideLufs: -11, otherLufs: -18), -7, accuracy: 0.0001)  // master: -7 dB
    }

    func testLinearGainNeverExceedsUnity() {
        XCTAssertLessThanOrEqual(volumeMatchLinearGain(sideLufs: -11, otherLufs: -18), 1.0)
        XCTAssertEqual(volumeMatchLinearGain(sideLufs: -18, otherLufs: -11), 1.0, accuracy: 0.0001)
    }
}
```

- [ ] **Step 2: Implement the pure gain functions**

Create `apps/iphone-native/YESMasterNative/VolumeMatch.swift`:
```swift
import Foundation

/// dB to apply to `side` so both sides play at the quieter side's loudness.
/// Reference = the quieter of the two, so the returned gain is always <= 0 (never boosts/clips).
func volumeMatchGainDb(sideLufs: Double, otherLufs: Double) -> Double {
    let reference = min(sideLufs, otherLufs)
    return reference - sideLufs
}

func volumeMatchLinearGain(sideLufs: Double, otherLufs: Double) -> Float {
    Float(pow(10.0, volumeMatchGainDb(sideLufs: sideLufs, otherLufs: otherLufs) / 20.0))
}
```

- [ ] **Step 3: Run the test** **[Mac required]**

Run `xcodebuild ... test`. Expected: `VolumeMatchTests` PASS.

- [ ] **Step 4: Failing test — playback controller applies volume**

In `TrackPlaybackControllerTests.swift`, add `volume` recording to the fake and a test:
```swift
    func testPlayAppliesRequestedVolume() throws {
        var events: [String] = []
        let player = FakeTrackAudioPlayer(events: &events)
        let controller = TrackPlaybackController(
            activateForPlayback: { events.append("activate") },
            makePlayer: { _ in player }
        )
        try controller.play(url: URL(fileURLWithPath: "/tmp/x.wav"), volume: 0.5)
        XCTAssertEqual(player.volume, 0.5)
    }
```
And extend `FakeTrackAudioPlayer`:
```swift
    var volume: Float = 1.0
```

- [ ] **Step 5: Add `volume` to the player protocol/impl and `play`**

In `TrackPlaybackController.swift`:
```swift
protocol TrackAudioPlayer: AnyObject {
    var currentTime: TimeInterval { get set }
    var volume: Float { get set }
    func play() throws
    func pause()
}
```
Extend `play` to accept volume and apply it; add a live setter:
```swift
func play(url: URL, startingAt startTime: TimeInterval? = nil, volume: Float = 1.0) throws {
    try activateForPlayback()
    if loadedURL != url {
        player = try makePlayer(url)
        loadedURL = url
    }
    if let startTime { player?.currentTime = max(0, startTime) }
    player?.volume = volume
    try player?.play()
    isPlaying = true
}

func setVolume(_ volume: Float) {
    player?.volume = volume
}
```
In `AVFoundationTrackAudioPlayer`, expose volume over the wrapped `AVAudioPlayer`:
```swift
var volume: Float {
    get { player.volume }
    set { player.volume = newValue }
}
```

- [ ] **Step 6: Remove LUFS Preview and add Volume Match state to `ContentView`**

- Replace the `ListeningMode` enum and `@State private var listeningMode` with `@State private var volumeMatchEnabled = false`.
- In `heroListeningToggles`, delete the LUFS Preview `checkboxButton` and rewrite the Volume Match one to bind to `volumeMatchEnabled`:
```swift
private var heroListeningToggles: some View {
    HStack(spacing: 22) {
        checkboxButton(title: "Volume Match", active: volumeMatchEnabled) {
            volumeMatchEnabled.toggle()
            applyVolumeMatchToActivePlayback()
        }
        Spacer(minLength: 0)
    }
    .padding(.horizontal, 18)
}
```
- Update `checkboxButton` to take an `action` closure instead of `mode`.
- Add the gain application (uses analysis LUFS for original, render measurements for master — store the master's measured LUFS when a preview/master render succeeds, e.g. `@State private var masteredLufs: Double?` set from `job.measurements?.lufsIntegrated`):
```swift
private func currentSideVolume() -> Float {
    guard volumeMatchEnabled,
          let originalLufs = analysisResult?.lufsIntegrated,
          let masterLufs = masteredLufs else { return 1.0 }
    switch selectedAudition {
    case .original: return volumeMatchLinearGain(sideLufs: originalLufs, otherLufs: masterLufs)
    case .mastered: return volumeMatchLinearGain(sideLufs: masterLufs, otherLufs: originalLufs)
    }
}

private func applyVolumeMatchToActivePlayback() {
    playbackController.setVolume(currentSideVolume())
}
```
- Pass `volume: currentSideVolume()` into every `playbackController.play(...)` call (in `toggleAuditionPlayback`, `selectAudition`, and the resume-after-preview branch).

- [ ] **Step 7: Run tests + build** **[Mac required]**

Run `xcodebuild ... test`. Expected: all PASS; no references to `lufsPreview` / `ListeningMode` remain (compiler confirms).

- [ ] **Step 8: Commit**

```bash
git add apps/iphone-native/YESMasterNative/VolumeMatch.swift apps/iphone-native/YESMasterNativeTests/VolumeMatchTests.swift apps/iphone-native/YESMasterNative/TrackPlaybackController.swift apps/iphone-native/YESMasterNativeTests/TrackPlaybackControllerTests.swift apps/iphone-native/YESMasterNative/ContentView.swift
git commit -m "feat(iphone-native): real volume match preview, remove LUFS preview"
```

---

## Task 7: Dead-code dedup — supported extensions (Phase 4)

**Files:**
- Modify: `apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift`
- Test: `apps/iphone-native/YESMasterNativeTests/SupportedExtensionsTests.swift` (create)

- [ ] **Step 1: Failing test that Swift advertises exactly the Rust-supported set** **[Mac required]**

Create `apps/iphone-native/YESMasterNativeTests/SupportedExtensionsTests.swift`:
```swift
import XCTest
@testable import YES_Master_Native

final class SupportedExtensionsTests: XCTestCase {
    func testSupportedExtensionsMatchRustDecoderSupport() {
        let bridge = NativeMasteringBridge()
        XCTAssertEqual(Set(bridge.supportedImportExtensions), ["wav", "mp3", "m4a", "aac", "flac", "ogg"])
    }
}
```

- [ ] **Step 2: Drop the never-supported literals**

In `NativeMasteringBridge.swift`, change `knownAudioExtensions` so it no longer lists extensions the Rust bridge rejects (it filters through `yes_master_native_supports_import_extension`, so `aiff/aif/opus` were always dropped):
```swift
private let knownAudioExtensions = ["wav", "mp3", "m4a", "aac", "flac", "ogg"]
```

- [ ] **Step 3: Run tests + build** **[Mac required]**

Run `xcodebuild ... test`. Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift apps/iphone-native/YESMasterNativeTests/SupportedExtensionsTests.swift
git commit -m "refactor(iphone-native): align import list to decoder support"
```

---

## Task 8: Remove the superseded Tauri app (Phase 4)

Do this LAST, after Tasks 1–7 are merged/validated and you have confirmed the native app builds.

**Files:**
- Delete: `apps/iphone/**`
- Modify: `package.json`
- Modify: `docs/IPHONE_APP.md`, `docs/IPHONE_APP_OVERVIEW.md`, `apps/iphone-native/HANDOFF.md`, `apps/iphone-native/README.md`

- [ ] **Step 1: Confirm nothing outside `apps/iphone` imports it** **[headless OK]**

Run (PowerShell):
```powershell
Select-String -Path package.json,vite.config.ts,tsconfig.json -Pattern "apps/iphone" -SimpleMatch
```
Expected: matches only the `iphone:*` scripts in `package.json` (root vite/tsconfig do not reference it).

- [ ] **Step 2: Delete the Tauri app**

```bash
git rm -r apps/iphone
```

- [ ] **Step 3: Remove the orphaned scripts from `package.json`**

Delete lines for `iphone:build`, `iphone:dev`, `iphone:ios:build`, `iphone:ios:dev`, `iphone:ios:init`, `iphone:tauri:backend-test`, `iphone:tauri:check`, `iphone:typecheck`, `iphone:test` (the 9 `iphone:*` entries, current `package.json:13-21`). Keep `dev`, `build`, `build:mac`, `build:windows`, `preview`, `tauri`, `test`, `test:watch`.

- [ ] **Step 4: Update docs**

- `docs/IPHONE_APP.md` / `docs/IPHONE_APP_OVERVIEW.md`: replace references to the Tauri `apps/iphone` with the native `apps/iphone-native` as the single iPhone app.
- `apps/iphone-native/HANDOFF.md`: remove the "remains the reference/prototype" line and the `Do not delete apps/iphone...` guardrail (line ~49); fix the stale "shows LUFS, true peak, and dynamic range" note (those are now internal-only).
- `apps/iphone-native/README.md`: drop the "reference/prototype" framing.

- [ ] **Step 5: Confirm the desktop app is unaffected** **[headless OK]**

Run:
```powershell
npm test
npm run build
```
Expected: PASS (neither depends on `apps/iphone`).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore(iphone): remove superseded Tauri app in favor of native"
```

---

## Stage 1 wrap-up: device verification (user, Mac + iPhone)

- [ ] Build/install Release (or Debug with the optimized profile) and confirm: analysis + first preview are fast; Loudness Low/Med/High audibly changes level; Volume Match changes only playback (export unchanged); Create Master produces a file that survives and shares; no LUFS Preview control remains.
- [ ] Record on-device timings to decide whether Stage 2's snippet/real-time work is still needed.

---

## Stage 2 (separate plan — write after Stage 1 lands + device timings)

These are intentionally NOT detailed here: Phase 1's full processing-as-state UX and Phase 2's real-time path are coupled, and Phase 2 is a feasibility spike whose outcome determines the code. Writing that code now would be guesswork.

- **Phase 2 spike (first):** prototype a persistent `MasteringChain` over FFI
  (`chain_create / process_block / set_settings / seek / destroy`) driven by
  `AVAudioEngine` + `AVAudioSourceNode`, landing loudness from the pre-analysis
  LUFS. Evaluate RT-safety (no alloc/lock/IO in `process_block`), clean seeking,
  on-device stability. Produce a written go/no-go.
- **If GO:** implement real-time audition (instant preset/intensity/loudness + toggle, no preview files).
- **If NO-GO:** implement 25–30s snippet previews (time-bounded render in the bridge, rendered proactively + debounced; Create Master switches to a full render).
- **Phase 1 UX rework:** processing-as-state screens, guaranteed-instant toggle, single-decode reuse — shaped to whichever Phase 2 path is chosen.

Decision gate before writing Stage 2: if Stage 1's on-device timings make the full-file preview acceptable, Stage 2 may reduce to just the UX polish and skip snippet/real-time entirely (measure-first, per the build-fix directive).

---

## Self-Review (against the spec)

- **Spec coverage:** Phase 0 → Task 1. Phase 3 Loudness → Tasks 2–3. Durable master → Task 4. Cleanup → Tasks 4–5. Volume Match + remove LUFS Preview → Task 6. Analysis-internal-only → satisfied (no numbers added to UI; analysis LUFS reused for Volume Match). Phase 4 dedup → Task 7. Tauri removal → Task 8. Phase 1 (processing-as-state) + Phase 2 (real-time/snippet) → Stage 2 (gated), with the debounce slice pulled forward into Task 3.
- **Type consistency:** `lufsTarget: Float` (Swift) ↔ `lufs_target: f32` (Rust/FFI); `NativeLoudness.lufsTarget`; `RenderStorage.{importsDirectory,mastersDirectory,previewsDirectory,pruneObsoletePreviews,enforceLimit}`; `volumeMatchGainDb`/`volumeMatchLinearGain`; `TrackAudioPlayer.volume` + `play(url:startingAt:volume:)` + `setVolume(_:)` used consistently across tasks.
- **Placeholders:** none — every code step shows the code; verification steps give exact commands and expected results, tagged headless vs Mac.
