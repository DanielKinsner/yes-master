> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Android A4 batches; executed via docs/plans/2026-06-12-android-shippability-plan.md Phase 2. _(Status: COMPLETE.)_ See docs/CHANGELOG.md for the project ledger.

# Android A4 Action Plan — 2026-06-10

Self-contained plan for the next work session. Written cold-start-ready:
assume no memory of the session that produced it. Scope is the Android app
only; desktop queue items (listening gates, album channel parity) are
unaffected.

## Starting point

- HEAD at or after `cd5b2e4` (docs: A3 execution record). A3 live audition
  is COMPLETE and verified: android host 16/16, JVM 7/7, iPhone 37/37,
  arm64 check, assembleDebug → 30.6 MB debug APK.
- Toolchain lives on this machine (provisioned 2026-06-10). Preflight:
  JDK 17 at `C:\Program Files\Eclipse Adoptium\jdk-17.0.19.10-hotspot`
  (JAVA_HOME set user-level; long-lived shells may need it re-exported),
  SDK at `%LOCALAPPDATA%\Android\Sdk`, NDK 27.2.12479018,
  `apps/android-native/local.properties` carries sdk.dir.
  **cargo-ndk needs `--platform 29`** (API-21 default sysroot has no
  libaaudio). Lanes: see CLAUDE.md "Android bridge" section.

## Already fixed — do NOT redo

The 2026-06-10 adversarial review (32 agents, 6 dimensions over
`14f8a24..b7cac8d`) confirmed 7 findings; all closed same day:

| Finding | Commit |
| --- | --- |
| JNI panic guards (mastering shims + input-side exception clear) | `a3ee1cb` |
| Audition shims born with panic guards | `68d0fab` |
| MediaStore receipt names the row actually created (uniquification) | `fd923aa` |
| IS_PENDING finalize inside the orphan-cleanup guard | `fd923aa` |
| rememberSaveable for in-progress choices | `fd923aa` |
| Riders: main-thread mkdirs, export-name sanitation, render-cache delete | `fd923aa` |
| gradlew executable bit + cargoNdk input coverage + toolchain props | `bf50685` |
| ndkVersion pin (strip works), `--platform 29` link fix | `440c888` |
| Stale doc claims (APK size, 16 KB attribution, build prereqs) | `cd5b2e4` |

## Batches (sized for one ~5 h session, in priority order)

### B1 — Process-death minimum story — M, ~75 min
The one CONFIRMED review finding whose code fix is still open (currently
documented as a known limitation in ANDROID_NATIVE_SPEC.md).

1. `MasteringViewModel` gains a `SavedStateHandle` (AndroidViewModel +
   SavedStateHandle constructor; default factory provides it). On every
   transition into `Ready`/`Done`, persist the restorable core: sourcePath,
   displayName, analysis JSON (re-serialize via Gson), style id, loudness
   name, intensity.
2. On init, if saved state exists AND the cached import file still exists
   (`File(sourcePath).exists()` — cacheDir is OS-reapable), restore
   directly to `Ready` (analysis decoded from the saved JSON — do NOT
   re-analyze) and re-arm the audition controller. Missing file → Idle.
3. Render-killed-in-background stays unfixed by design this pass
   (foreground service / WorkManager is A5-grade); the restore above turns
   "everything lost" into "back at Ready, press Create Master again".
   Update the known-limitation paragraph in ANDROID_NATIVE_SPEC.md.

JVM tests: save/restore codec round-trip (pure: UiState.Ready ↔ saved
primitives), restore-skips-when-file-missing decision helper.
Anchors: MasteringViewModel.kt:47 (ctor), :75-94 (analyze), :135+ (reset).

### B2 — Import fail-fast + error-state polish — S/M, ~60 min
A4 spec line "error states (missing file, unsupported format, storage
denied)". All anchors in MasteringViewModel.kt / Repositories.kt.

1. Wire the already-exported `NativeBridge.supportsImportExtension` into
   `importTrack`: resolve display name first, check the extension, fail
   fast with the supported-list message BEFORE copy-to-cache. (Note: SAF
   names can lack extensions — treat extensionless as "try it".)
2. Deterministic vs transient failures: analysis `error` payloads from the
   bridge (corrupt/unsupported file) must NOT offer "Retry analysis" —
   retry is for IO exceptions only. Anchor: analyze() catch + the
   `analysis.error?.let { error(it) }` path (review note: the catch
   currently sets retrySourcePath for both classes).
3. Catch `LinkageError` around first bridge touch and map to a readable
   Error state (UnsatisfiedLinkError is an Error, escapes
   `catch (e: Exception)` — review note).
4. publishToMusic failure messages: map SecurityException /
   IllegalArgumentException to plain-language storage messages.

### B3 — Cache hygiene — S, ~20 min
Render-cache deletion landed (`fd923aa`); imports still accumulate
timestamped copies forever (review note). On successful import, delete
everything in `cacheDir/imports` except the new copy. Careful with B1:
the restore path depends on the CURRENT import surviving — reap on
import, never on startup.

### B4 — Bridge/build tidiness riders — S, ~45 min, all mechanical
From the review's note tier; none user-visible, all cheap while the files
are warm:

1. JNI receiver params are typed `JClass` but Kotlin `object` externs pass
   the singleton instance — retype to `_this: JObject` in both shim
   modules (lib.rs + audition.rs), no behavior change.
2. Drop the unused direct `yes_master_lib` dep from
   apps/android-native/rust/Cargo.toml (all access goes through the
   facade; feature sets identical by unification).
3. Single-source the ABI list: derive the cargo-ndk `-t` args and
   `abiFilters` from one shared list (the "add emulator x86_64" comment
   currently implies one edit site; there are two).
4. cargoNdk task NDK fallback: prefer the exact `ndkVersion` pinned in the
   android block over newest-by-name (kills the lexicographic-max note).
5. 16 KB alignment tripwire: post-build check in the cargoNdk task
   (llvm-readelf from the pinned NDK: every LOAD segment Align == 0x4000)
   so an old cargo-ndk can never silently ship 4 KB-aligned libs.
6. Preset-argument asymmetry (NUL/decode failure silently falls back to
   the default preset while paths error): return error JSON for the
   render/analyze shims; for void audition setters a doc comment is
   enough.

### B5 — A4 polish from the spec's iPhone-mirror list — M, time-permitting
1. Import-control locking during export: verify by state machine (Working
   screen has no import affordances — confirm and add a JVM transition
   test rather than UI code if true).
2. Receipt subset decision is recorded (5 iPhone-mirror keys; adaptive
   digests deliberately not shown) — no code unless wanted.
3. Audition buffer-size knob: DEFER until D4 device QA shows a need;
   AAudio defaults first.

### Owner gate (no agent work) — D4 device QA, the only by-ear item
Sideload `apps/android-native/app/build/outputs/apk/debug/app-debug.apk`.
Listen for: toggle latency Original↔Mastered (should be click-free, same
spot), seek behavior (clean landing, no ringing), intensity/loudness sweep
during playback (smooth ~10 ms ramps, no zipper), headphone unplug
(pauses), phone call (pauses, resumes after), landing loudness vs the
exported file (should match — forced WYSIWYG), Volume Match A/B fairness.
File notes; calibration changes stay listening-gated per repo rules.

## Verification per batch

Kotlin-only batches: `gradlew test` (+ `assembleDebug` before ending).
Rust-touching batches (B4.1/.2/.6): android host `cargo test` +
`cargo ndk -t arm64-v8a --platform 29 check` + iPhone lane (facade is a
dependency). Desktop lanes only if src-tauri/src is touched (nothing here
touches them). Final: the full matrix from the A3 execution record.

## Parked / declined (do not re-derive)

Foreground-service renders (A5 with packaging), facade crate rename (the
"iphone" version string — Xcode churn), oboe migration (AAudio is
sufficient until device QA says otherwise), adaptive digests on the mobile
receipt, wire-samples.json gaining the landing payload (android-local pins
cover it).
