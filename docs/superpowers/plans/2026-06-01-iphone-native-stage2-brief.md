# iPhone-Native Stage 2 — Kickoff Brief (Real-Time Audition + Processing UX)

> **Audience:** a fresh agent with NO prior context on this work. Read this top to bottom, then the two referenced docs, before doing anything. Do NOT start coding — Stage 2 begins with a measurement gate and a research spike.

## 1. What this app is (orientation)

`apps/iphone-native` is the YES Master iPhone app: **native SwiftUI** UI + a thin **Rust C-ABI bridge** (`apps/iphone-native/rust`) that delegates all DSP to the shared desktop crate **`yes_master_lib`** (`src-tauri`, path dependency in `apps/iphone-native/rust/Cargo.toml`). There is no DSP reimplementation — the iPhone runs the exact desktop mastering chain.

User flow (intentionally bare-bones, LANDR-like): **upload → preset → intensity → loudness → Create Master → share**. Original/Master is an A/B toggle that must feel instant.

Read these before starting:
- Design spec: `docs/superpowers/specs/2026-06-01-iphone-native-perf-and-wiring-design.md`
- Stage 1 plan (already executed): `docs/superpowers/plans/2026-06-01-iphone-native-perf-and-wiring.md`

## 2. What Stage 1 already delivered (branch `iphone-native-perf-wiring`, 8 commits)

1. **Build fix** — `apps/iphone-native/rust/Cargo.toml` has `[profile.dev.package."*"] opt-level = 3`, so the DSP runs at release speed even in Debug app builds. Measured ~13× faster on an x86 host (30s track: ~7.95s → ~0.58s analyze+render). This was the dominant slowness cause.
2. **Loudness wired** — `NativeLoudness.lufsTarget` (Low −14 / Medium −11 / High −9) flows through `NativeRenderOptions.lufsTarget` → FFI `yes_master_native_render_master_with_options_json(..., float lufs_target)` → `lufs_offset_db` (clamped −24..−6). Loudness/Style/Intensity changes trigger a **300ms-debounced** reprocess.
3. **Durable masters** — new `RenderStorage` type. Create Master renders into Application Support `RenderedMasters` (never the evictable Caches preview). Previews live in Caches.
4. **Storage bounded** — `pruneObsoletePreviews` + `enforceLimit(max: 20)` called after preview/import/master success.
5. **Volume Match** — real, playback-only A/B loudness match (`VolumeMatch.swift`: `volumeMatchGainDb`/`volumeMatchLinearGain`; `TrackPlaybackController` gained `volume`/`setVolume`). Never affects export. **"LUFS Preview" control removed entirely.**
6. **Dead code** — extension list trimmed; the old Tauri `apps/iphone` deleted along with its `iphone:*` root scripts.

**Analysis is internal-only** now (no LUFS/numbers shown to the user); its integrated LUFS feeds the loudness landing and Volume Match.

## 3. The Stage 2 decision gate (DO THIS FIRST)

Stage 1's build fix may already make the current **full-file offline preview** fast enough on device. Before building anything:

1. On the user's Mac + iPhone, install the optimized build and time: import → first mastered preview, and a preset/intensity/loudness change → updated preview.
2. **If previews feel instant/acceptable** (e.g. ≲1s for typical songs), Stage 2 likely reduces to the **Processing UX polish only** (Section 5C) — skip the snippet/real-time engineering. Confirm with the user.
3. **If previews still drag** (long songs, slow devices), proceed to the real-time spike (Section 5A) with the snippet fallback (Section 5B).

Measure first. Do not build the real-time engine on spec if the build fix already solved it.

## 4. Hard constraints / guardrails

- **Do NOT change desktop behavior or the shared `yes_master_lib` DSP math.** Stage 2 adds new bridge entry points + Swift; it must not alter the existing desktop render/analysis paths. New Rust for snippets/real-time should live in the bridge crate (`apps/iphone-native/rust`), reusing `yes_master_lib` public APIs, not editing them.
- The bridge crate is its own Cargo build root (own `Cargo.lock`).
- **Verification split:** Rust is verifiable headless on the dev box (`cd apps/iphone-native/rust && cargo test`). Swift/iOS (`xcodebuild`, simulator/device, `aarch64-apple-ios` cross-compile) requires the user's **Mac** — never claim Swift built/tested without it. Swift test target imports `@testable import YES_Master_Native`.

## 5. Stage 2 scope

### 5A. Real-time audition feasibility spike (the core research question)

Goal: can the mastered chain be auditioned **live** (no preview file), so preset/intensity/loudness changes and the toggle are all instant?

Concrete spike steps:
1. **Confirm RT-safety of the chain.** Read `src-tauri/src/dsp.rs`: `MasteringChain` and its per-block path `process_interleaved` → `process_frame_inplace` (around `dsp.rs:1915`); coefficients via `ChainCoeffs::from_settings` (around `dsp.rs:710`); the limiter with 3ms lookahead (around `dsp.rs:1318`). Verify the hot path does NO heap allocation, locking, or I/O per block (lookahead buffers are part of the chain state, preallocated). This determines feasibility.
2. **Design the persistent-chain FFI** (new, in `apps/iphone-native/rust/src/lib.rs` + header): `chain_create(settings_json) -> *mut handle` (boxes a `MasteringChain` + a precomputed constant landing gain), `chain_process_block(handle, in_ptr, out_ptr, frames)`, `chain_set_settings(handle, settings_json)` (or recreate), `chain_seek(handle)`/reset state, `chain_destroy(handle)`. Follow the existing bridge conventions (`#[no_mangle] extern "C"`, `ffi_string`/`string_to_ffi`/`free_string`).
3. **Loudness landing in real time:** the offline path measures whole-file LUFS post-chain then applies a uniform gain (`engine.rs` `ceiling_bounded_landing_delta_db`, ~`engine.rs:106`). Real-time can't pre-measure — so precompute the constant landing gain from the **analysis pass's integrated LUFS** (`analysis.rs` `analyze_one`, ~`analysis.rs:32`, already exposed via `yes_master_native_analyze_file_json`) vs the chosen target, and apply it as a constant in/after the chain.
4. **Swift host:** `AVAudioEngine` + `AVAudioSourceNode` (iOS 16 target — fine). The source node's render callback decodes source PCM (AVAudioFile, or the Rust decoder) and pulls processed blocks via `chain_process_block`. Volume Match becomes a live gain node. Replace/extend `TrackPlaybackController` (currently `AVAudioPlayer`-based) with an engine-based controller for the mastered side.
5. **Go/No-Go criteria:** RT-safe (no alloc/lock/IO per block), clean seeking (reset chain state + jump source index), glitch-free at 44.1/48 kHz and typical buffer sizes on device. Write the decision down.

### 5B. Snippet fallback (if real-time is NO-GO)

- Add a **time-bounded render** to the bridge: render a ~25–30s region (offset + duration). Implement in the bridge crate — decode the source via `yes_master_lib` decode utilities (or slice on the Swift side into a temp WAV), then run the existing `mastering_render` on that slice. Do not modify desktop code.
- Render the snippet **proactively and debounced** so the Original/Master toggle stays instant. **Create Master still does a full-file render** (not a snippet promotion). Snippet region defaults to the current playhead; seeking far outside it triggers a debounced re-render.

### 5C. Processing-as-state UX rework (do in either case; may be the whole of Stage 2 if the gate says so)

- Make processing an explicit, visible state (motion + short progress + "Analyzing" / "Mastering"), attached to upload / preset / intensity / loudness changes — never to the toggle.
- Guarantee the **Original/Master toggle is always instant** (swaps two ready things; shows a preparing state only if not ready).
- Single-decode reuse where practical (avoid decoding the file twice before first playback).
- Shape this to whichever preview path (real-time vs snippet vs fast-full) is chosen.

## 6. Key files & entry points

- Bridge Rust: `apps/iphone-native/rust/src/lib.rs` (FFI), `apps/iphone-native/rust/include/yes_master_native_bridge.h` (header — keep in sync).
- Swift: `apps/iphone-native/YESMasterNative/ContentView.swift` (flow/state), `TrackPlaybackController.swift` (playback; AVAudioPlayer today), `NativeMasteringBridge.swift` (Swift↔Rust), `RenderStorage.swift`, `VolumeMatch.swift`, `AudioSessionController.swift`.
- Desktop DSP (read-only reference): `src-tauri/src/dsp.rs` (chain/limiter/compressor), `src-tauri/src/engine.rs` (`mastering_render`, LUFS landing), `src-tauri/src/analysis.rs` (`analyze_one`), `src-tauri/src/types.rs` (`MasteringSettings`/`AdvancedSettings`/`DeliveryProfile`/`Preset`).

## 7. How to start

1. Read the spec + Stage 1 plan + this brief.
2. Get on-device timings (Section 3) and confirm scope with the user.
3. If proceeding past UX-only: run the real-time spike (5A), write the go/no-go.
4. Use the **superpowers:writing-plans** skill to turn the chosen path into a concrete TDD plan, then **superpowers:subagent-driven-development** to execute. Keep desktop untouched; verify Rust headless, Swift on the user's Mac.
