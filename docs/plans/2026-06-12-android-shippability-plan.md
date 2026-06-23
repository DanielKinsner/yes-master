# Android Shippability Plan (2026-06-12)

Device-scoped execution plan: everything between the current Android app
(sideload-grade, engine excellent) and a Play-submittable product. Desktop
ships first (owner decision 2026-06-12); authored now for a non-Claude
implementer.

Parent docs: `docs/reviews/2026-06-12-master-shippability-audit.md`
(AN-01..AN-04, MB-01, UF-C1..C4),
`docs/plans/2026-06-12-shippability-roadmap.md` (global rules — rule 7:
interrogate the owner), `docs/archive/plans/2026-06-10-001-android-a4-action-plan.md`
(**treat as active spec** — it was queued as "next session" in
ANDROID_NATIVE_SPEC.md and never executed; this plan absorbs it),
`docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md` (AC-4 touches this
app's bridge tests). Toolchain prerequisites: docs/ANDROID_NATIVE_SPEC.md
"Build prerequisites" (JDK 17, NDK r27.2, cargo-ndk 4.1.2 or newer,
`--platform 29`).

Current state, one paragraph: Kotlin app + Rust crate reusing the shared
lib AND the iPhone facade as rlib. JNI panic containment, audio focus,
becoming-noisy, MediaStore publish-with-uniquify, and AAudio teardown
ordering are verified-correct. The gaps: one real UAF race, an unverified
AAudio stream contract, the entire unexecuted A4 plan, and store packaging
parked by design.

## Definition of "Android shippable"

Play submission passes pre-launch report without crash findings; no UAF /
OOB-write class remains; process death restores session state; imports
fail fast on unsupported input and the cache cannot grow unboundedly;
signed AAB builds reproducibly; owner has done one on-device listening
pass.

## Phase 1 — Crash class (do first)

1. **AN-1.1 Destroy/measure UAF race (audit AN-01; roadmap S4.2).**
   `release()` cancels the landing job cooperatively then immediately
   `destroyNative(h)` while the blocking `measureLanding` may still run
   (`AuditionController.kt:330-343`, `:276-287`); Rust-side
   `measure_landing` deliberately bypasses the UI mutex
   (`audition.rs:261-282`) so nothing serializes it against Drop. Fix both
   layers: Kotlin joins/awaits the landing job before destroy; Rust gets
   an RwLock (measurements read, Drop write). Also the create-leak: run
   `createNative` non-cancellably and destroy the result if the coroutine
   was cancelled (`:152-178`). Tests: Rust two-thread drop-vs-measure loop
   (crashes pre-fix); Kotlin fake-bridge ordering test.
2. **AN-1.2 AAudio post-open verification + close barrier (AN-02; S4.3).**
   Query actual format/channels/rate after `openStream`
   (`aaudio.rs:137-163` never does) — mismatch → close + Err (maps to the
   existing "Playback could not start"); the OOB-write class dies here.
   Close barrier: poll `waitForStateChange` to STOPPED with deadline
   before `close` (today it can return while still STOPPING,
   `aaudio.rs:179-197`). Host-side tests via the existing fake-stream
   seam; device stress noted for owner QA.

## Phase 2 — The A4 plan, executed (UF-C; roadmap S9.1/S9.2)

Work the A4 action plan items in its own order, with these audit
amendments:
3. **AN-2.1 Import fail-fast (A4 B2 + UF-C2).** Wire the DEAD JNI seam:
   `supportsImportExtension` is exported from Rust (`rust/src/lib.rs:166`)
   and declared in `NativeBridge.kt:30` with zero callers — call it before
   `copyToCache` so a 300 MB unsupported SAF pick fails in milliseconds,
   not after a full copy. Plus A4's retry-on-deterministic-failure
   suppression and `LinkageError` mapping (a `UnsatisfiedLinkError`
   currently escapes `catch (Exception)`).
4. **AN-2.2 Import-cache reaping (A4 B3 + UF-C3).** Timestamped copies
   currently accumulate forever; reap to current + N most-recent on
   attach. Unit-test the reaper.
5. **AN-2.3 Process-death restore (A4 B1 + UF-C1).** SavedStateHandle so
   a backgrounded-then-killed session restores track + settings instead
   of restarting at Idle. JVM state-restore test.
6. **AN-2.4 Tidiness riders (A4 B4 + UF-C4).** JObject retype, unused dep
   removal, ABI single-source, NDK-fallback pin, 16 KB page-alignment
   tripwire, preset-arg asymmetry. One commit.

## Phase 3 — Contract pins + adaptive inheritance

7. **AN-3.1** Recipe parity (XP-04; roadmap S3.3 Android side — the crate
   already include_str's the fixture; extend to the new `delivery` block).
8. **AN-3.2** AC-4 inheritance proof (compressor spec §4): bridge test,
   dense fixture, gate-ON, bit-equal to desktop.
9. **AN-3.3** MB-01 guard: same 30-minute import duration cap as iPhone
   (constant shared in the facade if practical), clear user message.

## Phase 4 — Product completeness (rule 7: ask, then build)

10. **AN-4.1 Done-screen actions (AN-04).** Share/play intents on the
    receipt (parity with iPhone's ShareLink) instead of a bare path.
11. **AN-4.2 Monochrome adaptive-icon layer (AN-04)** — Android 13 themed
    icons currently render a default blob.
12. **AN-4.3 Background render story (owner question; spec'd as A4/B1
    note).** Today a backgrounded render is lost. Decide: foreground
    service + MediaSession (real fix, M effort) vs. documented limitation
    for v1. Ask the owner; default for v1: keep-screen-on flag during
    render + honest copy, full service post-launch.

## Phase 5 — Store packaging (AN-03; roadmap S8.5; owner-gated start)

13. **AN-5.1 Signing config + AAB.** Needs the owner's keystore decision.
    `signingConfigs` + `bundleRelease`; `apksigner verify` in the lane.
14. **AN-5.2 Release build hygiene.** `isMinifyEnabled`/resource-shrink
    decision (test the release build either way — JNI keep rules needed if
    minified), versionCode scheme aligned with 0.1.0 (roadmap S0.2 test).
15. **AN-5.3 ABI decision.** arm64-v8a-only is Play-acceptable; decide on
    x86_64 (emulator/Chromebook) — gradle already stubs the extension
    point. Default: arm64 + x86_64.
16. **AN-5.4 Listing + data-safety form** (trivial: nothing collected,
    zero permissions) + pre-launch report run.

## Lanes

Every phase: `cd apps/android-native/rust; cargo test; cargo ndk -t
arm64-v8a --platform 29 check`. Kotlin changes: `./gradlew test` (CI job
from roadmap S0.1). Shared-crate or facade touches: desktop Rust lane +
iPhone lane too (CLAUDE.md rule).

## Owner gates

Keystore (AN-5.1), background-render decision (AN-4.3), listing assets,
and one on-device listening pass (plus the AC-5 calibration is
desktop/iPhone per the owner's offer — Android inherits the locked
constants through the shared crate and needs only a spot-check).
