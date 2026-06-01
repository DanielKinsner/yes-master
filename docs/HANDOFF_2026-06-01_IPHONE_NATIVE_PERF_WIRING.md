# Handoff — iPhone-Native Perf & Wiring (2026-06-01)

Quick resume notes for a fresh agent picking this up, possibly on a different machine. Keep, skim, or ignore — it's here so the context isn't lost.

## Where things stand

- **Branch:** `iphone-native-perf-wiring` (forked from `main`). **Not merged. Not pushed** (local only as of this writing — if you're on another machine, the branch needs to be pushed first).
- **Stage 1 is complete**: 11 commits (`b676021`…`5301c44`). All planned work landed.
- Scope was `apps/iphone-native` only. Desktop app and the shared `yes_master_lib` DSP math were intentionally **not** changed.

## DO THIS FIRST on a Mac

The Swift was written and reviewed on a **Windows** box with **no Apple SDK**, so it was **never compiled**. The Rust side is verified (see below); the Swift is not. Before anything else:

```bash
cd apps/iphone-native
xcodegen generate
xcodebuild -project YESMasterNative.xcodeproj -scheme YESMasterNative \
  -destination 'platform=iOS Simulator,name=iPhone 17' CODE_SIGNING_ALLOWED=NO test
```

Expect it to mostly work (whole-file `ContentView.swift` was reviewed symbol-by-symbol), but a compiler may surface a small fix or two. The 4 new Swift test files are: `NativeLoudnessTests`, `RenderStorageTests`, `VolumeMatchTests`, `SupportedExtensionsTests` (+ a volume case in `TrackPlaybackControllerTests`).

## Verification status

- ✅ **Rust bridge** (`cd apps/iphone-native/rust && cargo test`): 12 passed, 1 ignored (timing proxy). Verified.
- ✅ **Desktop unaffected**: `npm test` (149 passed) + `npm run build` succeeded after the Tauri-app removal.
- ⚠️ **Swift/iOS**: pending a Mac. Not yet compiled or run.

## What changed (and why), with the non-obvious bits

1. **Build fix** (`365cd39`) — `apps/iphone-native/rust/Cargo.toml` got `[profile.dev.package."*"] opt-level = 3`. This was the #1 slowness cause: device builds run Debug, and unoptimized debug Rust ran the DSP ~13× slower. **Gotcha:** with this profile, a *Debug* build already optimizes the deps, so debug and release timings look ~identical — that's expected, not a bug. Measured baseline (x86, 30s track): ~7.95s → ~0.58s.
2. **Loudness** (`25859cb`, `fd5e4fb`) — Low −14 / Med −11 / High −9 LUFS. **FFI ABI changed:** `yes_master_native_render_master_with_options_json` now takes a 5th arg `float lufs_target`. The C header (`rust/include/yes_master_native_bridge.h`) and the Swift call site must stay in sync. Rust clamps to −24..−6; default stays −11 so untouched behavior is unchanged.
3. **Durable masters** (`cc841cd`) — new `RenderStorage`. Create Master always renders fresh into Application Support `RenderedMasters`; the old "share the Caches preview" shortcut was removed (Caches can be purged). Previews stay in Caches.
4. **Storage bounded** (`044ef84`) — `enforceLimit(max: 20)` on imports/masters, `pruneObsoletePreviews` after each preview.
5. **Volume Match** (`edec3ad`) — real, **playback-only** A/B match (`AVAudioPlayer.volume`). Uses the analysis integrated LUFS (original) and the render's measured LUFS (master); attenuates the louder side only, never boosts, **never touches the exported file**. The dead **"LUFS Preview" control was removed entirely**.
6. **Extension dedup** (`7069806`) — Swift `knownAudioExtensions` trimmed to the Rust-supported set.
7. **Tauri app removed** (`1abc6aa`) — `apps/iphone` (~13.2k lines, incl. a committed generated Xcode project) and its 9 `iphone:*` root scripts deleted. `apps/iphone-native` is now the only iPhone app.

## Known minor issues (not blockers)

- `ImportedTrackStore` and `RenderStorage.importsDirectory` independently compute the same `ApplicationSupport/ImportedTracks` path. Works today; if one path is ever customized the other won't follow. Small future cleanup.
- `SupportedExtensionsTests` calls the real bridge FFI, so it needs the linked static lib (fine on device/simulator; would fail on a host that can't load the lib).

## Next: Stage 2

Don't dive into the real-time engine blind. **Measure on device first** — the build fix alone may have made the current full-file preview fast enough, in which case Stage 2 shrinks to UX polish. The full plan (real-time audition spike → snippet fallback → processing-UX rework), with concrete DSP entry points and go/no-go criteria, is in the Stage 2 brief.

## Doc map

- Design spec: `docs/superpowers/specs/2026-06-01-iphone-native-perf-and-wiring-design.md`
- Stage 1 plan (executed): `docs/superpowers/plans/2026-06-01-iphone-native-perf-and-wiring.md`
- Stage 2 kickoff brief: `docs/superpowers/plans/2026-06-01-iphone-native-stage2-brief.md`
- Per-app status: `apps/iphone-native/HANDOFF.md`
