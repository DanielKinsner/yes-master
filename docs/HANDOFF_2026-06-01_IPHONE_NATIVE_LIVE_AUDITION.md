# Handoff — iPhone-Native Live Audition (2026-06-01)

**Status: shipped to `main`.** The native iPhone app now auditions mastered audio
live, on-device, through the exact desktop DSP. Built/installed/run on a physical
iPhone 16 Pro Max; all automated suites green (one pre-existing desktop failure
noted below, unrelated to this work).

## What shipped

Live mastered audition on the native iPhone app (`apps/iphone-native/`):

- Import/analyze once; **Original plays immediately**; **Mastered is live** (no
  preview-WAV render on control changes).
- **Original↔Mastered on one timeline** — atomic bypass with the playhead
  preserved sample-accurately (no second player), now crossfaded.
- **Style / Intensity** change the chain live (recompute `ChainCoeffs` off-thread
  + 512-frame coefficient crossfade).
- **Loudness (Low/Med/High)** lands the level live via a windowed
  measure-through-the-chain (`engine::preview_landing`), routed through the SAME
  `ceiling_bounded_landing_delta_db` as export → preview ≈ master (~0.5 dB).
- **Volume Match** is a real, audition-only, level-matched gain (uses measured
  mastered LUFS vs analyzed original LUFS). Never reaches export.
- **Create Master** is still the only full-song render/export.
- Click-free transitions (one-pole smoothing on VM/landing/bypass; chain stays
  warm). End-of-track replay. AVAudioSession interruption + route-change handling.

### Architecture (the load-bearing decision)

```
SwiftUI ContentView (unchanged visuals)
  → AuditionController (@MainActor state machine; import/analyze, transport,
       live params, landing+VM, Create Master, interruptions)
    → LiveAudioEngine (AVAudioEngine + ONE AVAudioSourceNode; deinterleaved
         Float32 node format; render callback pulls Rust per block)
      → LiveAuditionBridge (Swift wrapper over the C ABI)
        → live_stream.rs (Rust: one decoded Arc<[f32]> PCM, one cursor, the
             SHARED yes_master_lib MasteringChain; lock-free atomics + mpsc;
             RT-safe process)
```

Swift owns the output graph; Rust owns the persistent stream and pulls the
**shared desktop `MasteringChain`** (no reimplemented DSP, no rodio/cpal at
runtime). Proven by `live_output_matches_desktop_chain_bit_for_bit` (bit-exact).

## How to build / run / test

The `.xcodeproj` is gitignored (XcodeGen-managed). After pulling:

```bash
cd apps/iphone-native && xcodegen generate --spec project.yml   # regenerate project
```

- **Rust (bridge):** `cd apps/iphone-native/rust && cargo test`  (26 tests)
- **Rust (desktop, shared crate):** `cd src-tauri && cargo test --lib`
- **Swift (simulator):**
  `xcodebuild -project apps/iphone-native/YESMasterNative.xcodeproj -scheme YESMasterNative -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test`
- **On device:** find the id with `xcrun devicectl list devices`, then
  `xcodebuild ... -destination 'platform=iOS,id=<DEVICE_ID>' -configuration Debug -allowProvisioningUpdates build`,
  then `xcrun devicectl device install app --device <ID> "<…/YES Master Native.app>"` and `… process launch …`.
  (Signing team `C4ZKT5JJ3K` is set in `project.yml`.) The pre-build script
  `scripts/build-rust-bridge.sh` compiles the Rust bridge automatically.

## Verification status

- ✅ Rust bridge: 26 tests (parity, intensity, landing, smoothing, EOF, bypass).
- ✅ Swift: full suite (controller/engine/bridge/integration incl. a REAL
  `AVAudioEngine` start that guards the -10868 interleaved-format crash).
- ✅ Desktop shared crate: 205 lib tests pass; additive `engine::preview_landing`
  only.
- ⚠️ **Pre-existing (NOT this work):** 4 `dsp::tests::preset_byte_identity::*`
  SHA-snapshot tests fail identically on a clean tree (verified by stashing).
  They're float byte-identity snapshots, opt-level/CPU-sensitive. Investigate or
  regenerate under the canonical profile — unrelated to iPhone audio.

## Remaining work (prioritized)

1. **On-device RT profiling pass (the real spike gate).** Run Instruments on the
   render callback during **Mastered, near-ceiling** playback: confirm zero
   allocation / no ObjC-runtime in steady state, no XRuns over ≥5 min, and watch
   thermal. The chain now runs every frame (warm bypass), so Original costs the
   same as Mastered — verify that's fine on sustained playback.
2. **Denormal flush-to-zero** on the audio thread (set ARM FPCR FZ at stream
   start). The IIR biquad feedback can hit subnormals on quiet tails → CPU
   spikes. Do it iOS-thread-only (don't touch shared `dsp.rs` output / the
   byte-identity snapshots).
3. **Preallocated coeff double-buffer.** `with_coeffs_inheriting_state` clones
   `Vec`s on the audio thread on each param change (bounded, accepted for the
   spike). Replace with a preallocated ping-pong to remove the in-callback alloc.
4. **Loudness rate reconciliation.** Live landing measures at the **source** rate;
   export resamples to the delivery `effective_sample_rate` before landing
   (`engine.rs`). When a delivery profile changes the rate they can diverge >
   the ~0.5 dB window error. Measure the live landing at `effective_sample_rate`
   to close it.
5. **Intensity perceptibility (product decision).** Intensity is wired + matches
   desktop (`preset_scale = 0.4 + 1.2*intensity`, never bypass), but the new
   loudness landing re-levels the output, removing the volume cue — so on a phone
   speaker it's subtle. If you want a wider/`0`=near-bypass range, that's a
   **shared** `from_settings` change (affects desktop too) — capture a listening
   note first per CLAUDE.md.
6. **`yes-dsp` crate extraction (deferred).** `MasteringSettings.album:
   Option<AlbumPlan>` couples the settings to album-workflow types. The live path
   reuses `yes_master_lib::dsp` directly (one `use` line away from a future
   `yes-dsp`). Clean refactor for later; not blocking.
7. **Minor:** `RenderStorage.previewsDirectory` / `pruneObsoletePreviews` are now
   unused by the live path (kept as harmless defensive code + a regression test).
   Remove if you want, or leave.

## Key files

- Rust RT stream + C ABI: `apps/iphone-native/rust/src/live_stream.rs`
- Shared landing: `src-tauri/src/engine.rs` (`preview_landing`,
  `ceiling_bounded_landing_delta_db`)
- Swift: `AuditionController.swift`, `LiveAudioEngine.swift`,
  `LiveAuditionBridge.swift` (under `apps/iphone-native/YESMasterNative/`)
- C header: `apps/iphone-native/rust/include/yes_master_native_bridge.h`
- Plan this implemented (with the agreed divergences): `docs/superpowers/plans/2026-06-02-iphone-native-live-audition.md`
