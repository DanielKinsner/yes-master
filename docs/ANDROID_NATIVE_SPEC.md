# Android Native App — Build Spec

Spec date 2026-06-10. Owner-readable; written to be skimmed on a phone.
Branch: `feat/android-native`. Execution follows the autonomous-build
pattern: small green commits, full verification per phase, handoff at the
end.

## The one-paragraph version

The entire mastering engine — analysis, adaptive guardrails + confidence,
presets, render, export checks, decode — is already a portable Rust library
proven on iPhone behind a C ABI (`apps/iphone-native/rust`). Android is a
Kotlin/Compose shell around that same core: JNI instead of Swift FFI, Oboe
instead of AVAudioEngine, SAF/MediaStore instead of the iOS document picker.
No DSP is written, ported, or retuned. **Correctness is proven by
construction, not by ear**: the same sample-level parity test that pins the
iPhone bridge (`bridge_render_matches_shared_render_path`) pins Android —
same input + settings ⇒ bit-identical master on every platform.

## Non-negotiables

- **Desktop stays untouched.** Every shared-crate change is feature-gated
  with defaults unchanged; the full desktop lane + iPhone lane run green at
  every phase boundary. The engine lock applies exactly as on desktop.
- **iPhone stays untouched.** Android *depends on* the existing bridge crate
  (it already builds as `rlib`); nothing in `apps/iphone-native/` changes.
- **No listening gate for parity.** Rendered output is pinned bit-identical
  by tests. The only by-ear item is Android **live-audition feel** (buffer
  sizes/latency on real devices), which is device QA, not DSP signoff.

## Architecture (mirror of the proven iPhone shape)

```
apps/android-native/
  rust/        yes-master-android-bridge (cdylib .so)
               ├─ depends on yes_master_lib (default-features = false)
               ├─ depends on the iphone bridge crate as rlib
               │  (aliased `native_bridge`; it is platform-neutral Rust —
               │   analyze/render/settings facade + live_stream)
               ├─ JNI shims (jni crate): thin string-in/JSON-out wrappers
               │  over the existing yes_master_native_* functions
               └─ audio engine: oboe (Rust bindings) output callback that
                  pulls frames from live_stream IN-PROCESS — no per-buffer
                  JNI hop at all (simpler than the iPhone's Swift↔C hop)
  app/         Kotlin + Jetpack Compose (Gradle project)
```

Swift → Kotlin mirror map:

| iPhone (exists) | Android (new) |
| --- | --- |
| ContentView.swift | Compose screens (import hero → analyzing → style/loudness → audition → export → receipt) |
| NativeMasteringBridge.swift | NativeBridge.kt (JNI; decodes the same JSON keys — wire-key test mirrors the Swift one) |
| LiveAudioEngine + LiveAuditionBridge + AudioSessionController | Rust oboe engine + thin AudioController.kt (start/stop, focus, route changes via AudioManager) |
| AuditionController / TrackPlaybackController | PlaybackViewModel |
| ImportedTrackStore.swift | ImportRepository (SAF picker → copy to app cache → real path for Rust) |
| RenderStorage.swift | ExportRepository (render to cache → publish via MediaStore into Music/YES Master) |
| VolumeMatch.swift | Check in A3 whether this is UI state or math; if math, expose from Rust instead of porting (single source) |

The live_stream C surface Android reuses as-is (14 functions): create /
process / set_bypass / set_params / set_volume_match / set_landing_gain /
snap_controls_to_targets / seek / position / duration / channels /
sample_rate / measure_landing / destroy.

## Phases

### A0 — Toolchain + shared-crate dep gating (~1 session)
- Make `tauri`, `tauri-plugin-dialog`, `rodio` **optional** deps of
  `yes-master`, owned by the existing `app-runner` feature (still default-on
  → desktop builds identically; `audio.rs`/`main.rs` gated accordingly).
  Today they compile even in `default-features = false` builds; on Android
  cross-compiles rodio→cpal→oboe-from-source is pure friction. Bonus: the
  iPhone bridge build gets lighter too.
- Install Android SDK + NDK (r27+, handles the 16 KB page-size requirement),
  Rust targets (`aarch64-linux-android` primary, `x86_64-linux-android` for
  emulator), `cargo-ndk`. **Requires accepting Google's SDK licenses —
  owner go-ahead needed before this phase runs.**
- Verify lane added to CLAUDE.md:
  `cargo ndk -t arm64-v8a check` on the android crate + host `cargo test`.
- Gate to pass: full desktop fast lane + iPhone bridge lane green with the
  feature change; android crate skeleton cross-compiles.

### A1 — Android bridge crate (~1–2 sessions)
- `yes-master-android-bridge`: JNI externs wrapping the existing C ABI
  (`analyze_file_json`, `render_master_with_options_json`, version,
  supports_extension) — JSON strings across the boundary, same contract
  Swift uses.
- Host-side tests (no device needed): the inner functions are plain Rust,
  so the analyze/render/wire-key tests run in normal `cargo test`. The
  sample-parity guarantee is inherited — same engine code path the iPhone
  bridge already pins bit-identical against desktop.
- Extend `src/standard-mapping-parity.json` consumers: a host-side Rust
  test in the android crate asserts the same style→preset map (third
  assert-side, as designed).

### A2 — App foundation → sideloadable MVP (~3–5 sessions)
- Gradle + Compose project, minSdk 29 (Android 10: scoped storage uniform,
  AAudio mature, covers the vast majority of devices — owner can lower
  later if wanted).
- Flow: SAF import (copy to cache) → analyze (progress UI) → style tiles
  (Balanced/Bright/Warm/Heavy) + loudness (Low/Medium/High) + intensity →
  render master → MediaStore export → receipt (same JSON keys as the
  desktop receipt; wire-key test on the Kotlin decode side).
- Deliverable: debug APK you can sideload and use end-to-end **without**
  live audition.
- Verification: host cargo tests + `cargo ndk` build + `gradlew
  assembleDebug` + a JVM unit test for the Kotlin JSON decode against
  `src/wire-samples.json` (fourth consumer of the existing gate).

### A3 — Live audition (~3–5 sessions + device QA)
- oboe output stream (low-latency, float) whose callback pulls from
  live_stream — all in Rust. Kotlin gets start/stop/seek/params via JNI.
- Original/Mastered toggle preserving playhead (same `seek` + bypass
  semantics as iPhone), Volume Match, Preview LUFS, snap-on-track-switch.
- Audio focus + route-change handling in AudioController.kt.
- **Owner QA here**: latency/buffer feel on your physical device(s); this is
  the only by-ear item in the whole project.

**As built (2026-06-10):** AAudio direct (the risk-#2 fallback) instead of
oboe — the NDK's stable C API needs no cmake/C++ build and zero new Rust
deps; the stream requests the SOURCE sample rate (chain coefficients are
computed for it — parity) and lets the OS resample. "Preview LUFS" shipped
as desktop-Standard's forced-WYSIWYG landing (measure → always apply,
250 ms settle debounce — the exact iPhone shape), so Volume Match is the
only audition toggle and the desktop mutual-exclusion rule cannot be
violated. VolumeMatch.swift's mirror-map question resolved: it is math, so
it now lives in Rust (`audition::volume_match_linear_gain`, pinned against
the Swift formula); Kotlin only picks which side is being heard.

### A4 — Pins + polish (ongoing, slice-sized like the iPhone branches)
- Kotlin-side wire-key tests (mirror of `NativeMasteringBridge` decode pins).
- Receipt polish, error states (missing file, unsupported format, storage
  denied), import-control locking during export — crib the ~25 iPhone
  polish slices; most translate 1:1.

### A5 — Packaging (deferred until wanted)
- Release signing, ABI splits, Play listing. Out of scope until the
  sideloaded app has been lived with. Desktop release priorities (listening
  gates) remain ahead of this in CLAUDE.md.

## New dependencies (the complete list)

| Dep | Where | Why |
| --- | --- | --- |
| `jni` | android crate only | JNI externs |
| `oboe` (rust bindings, prebuilt feature) | android crate only | low-latency audio out |
| Android SDK/NDK + Gradle/Kotlin/Compose | tooling | the platform |

Nothing new in `yes-master` itself — only feature re-plumbing of deps it
already has.

## Risks, honestly

1. **rodio/cpal/tauri on android targets** — dodged entirely by A0's
   feature gating (they simply don't compile into mobile builds).
2. **oboe-rs build friction** (cmake/prebuilt) — use the prebuilt-fetch
   feature; fallback is a ~100-line AAudio FFI shim (AAudio is a plain C
   API, no C++ needed).
3. **Device audio variance** — the only real unknown. Mitigation: oboe's
   own device workarounds + A3 device QA with adjustable buffer size.
4. **SAF copy-to-cache** doubles disk for huge files — acceptable MVP
   tradeoff; file-descriptor passthrough is a later optimization.
5. **Emulator audio is useless for latency feel** — functional tests on
   emulator, feel tests on hardware only.

## Owner decisions

| # | Decision | Outcome |
| --- | --- | --- |
| D1 | Google SDK/NDK license acceptance | **approved 2026-06-10**; toolchain installed |
| D2 | minSdk | 29 (Android 10) — default taken |
| D3 | MVP cut | A2 shipped without live audition; A3 is the next phase |
| D4 | Test hardware | owner can't device-test for a while; APK is sideload-ready when they can |

## Execution record

A0–A2 completed 2026-06-10 on `feat/android-native` (A0 `96ebcef`, A1
`64baf0b`, A2 `7631962`):

- Toolchain: JDK 17, SDK platform 35 + build-tools 35/34, NDK r27.2,
  rust android targets, cargo-ndk, Gradle 8.11.1 (wrapper committed).
- **The feared dep gating proved unnecessary** — rodio/tauri-utils
  cross-compile for android as-is; the desktop crate has ZERO changes.
- iPhone facade touch: one visibility-only change
  (`export_settings_for_options` `pub(crate)` → `pub`) so the android crate
  can assert the parity fixture; iPhone lane re-verified green.
- Deliverable: `gradlew assembleDebug` → 25.9 MB arm64 debug APK
  (engine statically linked, 4.3 MB `.so`, zero requested Android
  permissions). The 31.5 MB first reported was measured before the
  packaging excludes dropped the two stray cdylibs.
- Pins landed: analyze/render wire-key tests (Kotlin-decoded keys),
  parity-fixture third (Rust) and fourth (Kotlin JVM) assert-sides,
  wire-samples drift gate consumed by the Gson models.
- Cross-platform matrix at A2 head: desktop FE 336/336 + Windows bundle,
  desktop Rust full lane exit 0, iPhone 37/37, android 5/5 + arm64 check,
  JVM 3/3.

Sideload: `apps/android-native/gradlew assembleDebug` →
`app/build/outputs/apk/debug/app-debug.apk` → enable "install unknown
apps" on the phone, copy, tap.

Build prerequisites on a fresh machine (the A0–A2 toolchain lived in the
original build environment, NOT necessarily yours — re-provisioned on the
owner's Windows machine 2026-06-10): JDK 17 on `JAVA_HOME`, Android SDK
(platform 35, build-tools 35) + NDK r27.2 with `sdk.dir` in
`apps/android-native/local.properties` (gitignored) or `ANDROID_HOME`,
rust targets `aarch64-linux-android`/`x86_64-linux-android`, and
**cargo-ndk ≥ 3.5.6** — the 16 KB page alignment Android 15+ requires is
supplied by cargo-ndk's default linker flag, not by NDK r27 itself.

## A3 execution record (2026-06-10)

Landed in four slices on `main` after an adversarial review of A0–A2
closed 7 confirmed findings (JNI panic guards, truthful MediaStore
receipt, IS_PENDING finalize guard, rememberSaveable choices, gradlew
file mode, cargoNdk input coverage, stale APK size):

- Facade: `live_stream` goes `pub` (second visibility-only change; iPhone
  lane re-verified green).
- `audition` module: AuditionEngine over the shared LiveStream — UI ops
  serialized behind one mutex, start() snaps controls per the facade
  contract, Volume Match math single-sourced in Rust. 11 host tests.
- `aaudio` module: ~100-line FFI shim, low-latency float, error callback
  flags a lost stream for rebuild-on-start.
- Kotlin: AuditionBridge + AuditionController (audio focus, becoming-noisy
  pause, 10 Hz position poll, EOF auto-pause/restart-from-top, 250 ms
  landing debounce) + transport card on the Ready screen. JVM tests pin
  the side-selection/epsilon logic and the landing wire decode.

Known A2/A3 limitation (A4 backlog): no process-death story — a render
killed in the background is lost (no SavedStateHandle/WorkManager), and
session state resets to Idle on relaunch.

**Owner QA still pending (D4):** latency/buffer feel on hardware — the
only by-ear item. Functional flow verified by tests + emulator-free lanes.

**Next session:** execute
`docs/plans/2026-06-10-001-android-a4-action-plan.md` — batched A4 work
(process-death restore, import fail-fast, cache reaping, bridge tidiness)
with anchors, lane commands, and the already-fixed list so nothing is
redone.
