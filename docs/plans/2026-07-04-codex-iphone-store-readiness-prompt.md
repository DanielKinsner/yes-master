# Codex prompt — iPhone store-readiness batch (Wave 8, keystore-free)

Drafted 2026-07-04 by Fable for the owner to paste into Codex. Every file:line
pointer below was re-verified against `main` on 2026-07-04. Nothing in this
batch is listening-gated and nothing needs the owner's keystore.

---

You are working in the YES Master repo. **Read `AGENTS.md` first** and follow
it exactly. Prefer current code reality over any historical prose — including
this prompt's pointers: re-verify each one before acting.

**BRANCH FIRST**: create `feat/iphone-store-readiness` off `main` before
touching anything — the owner pushes local state without checking. Commit in
very small chunks, one slice per commit.

**Scope:** the keystore-free iPhone store-readiness items from Wave 8
(`docs/plans/2026-06-12-shippability-roadmap.md` S8.1–S8.3) plus the IP-05
leftovers (`docs/reviews/2026-06-12-master-shippability-audit.md:355`).
Android S8.4 is ALREADY SHIPPED — verified 2026-07-04: monochrome icon layer
(`apps/android-native/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml:5`),
Done-screen share/play intents
(`apps/android-native/app/src/main/java/com/yesmaster/app/MainActivity.kt:352,375-381`),
import extension check (`MasteringViewModel.kt:83`). Do not redo those; your
ledger update records them as verified-done. S8.5 (Android signing — owner
keystore) and S8.6 (background-audio — owner decision) are gated: **do not
touch them.**

## Slices

1. **S8.1 — privacy manifest completeness.**
   `apps/iphone-native/YESMasterNative/PrivacyInfo.xcprivacy` already exists
   (FileTimestamp / C617.1, no tracking, no collected data). Remaining:
   (a) add `ITSAppUsesNonExemptEncryption: false` to the `info.properties`
   block in `apps/iphone-native/project.yml` (verified missing 2026-07-04);
   (b) audit the Swift app for other required-reason APIs (UserDefaults →
   CA92.1, disk-space, boot-time — no UserDefaults callers existed at last
   check) and extend the manifest only for APIs actually used;
   (c) confirm xcodegen puts the `.xcprivacy` into the app bundle resources
   (the `sources: - path: YESMasterNative` include should — verify, wire
   explicitly if not). A real archive-validation run needs the owner's Mac —
   leave a one-line TODO in your handoff; do not attempt it.

2. **S8.2 — progress honesty: verify + close.** The staged-percent theater
   the audit flagged is already gone: `ContentView.swift:277-297` renders an
   indeterminate `ProcessingSpinner` + honest `statusText` ("Analyzing
   audio...", "Creating master..."). Verify nothing percent-staged remains
   anywhere in the iPhone app, then close S8.2's minimum slice in the ledger
   with that evidence. Real progress plumbing stays a follow-up — do NOT
   build new FFI for it in this batch.

3. **S8.3a — typed decode errors across the bridge.**
   `AuditionController.audioErrorState`
   (`apps/iphone-native/YESMasterNative/AuditionController.swift:522-532`)
   still substring-sniffs `error.localizedDescription` ("no suitable format
   reader", "decode error") to map to `.decodeFailed`. Replace the sniffing
   with a typed error signal from the Rust facade
   (`apps/iphone-native/rust`): a stable error-code enum on the
   render/import FFI surface, mapped to `AuditionErrorState` in Swift. Add a
   Swift test pinning the mapping and a Rust test pinning the code values.
   **LOUD WARNING:** the iPhone facade crate is re-used by the Android bridge
   as an rlib — ANY facade change runs BOTH mobile lanes (AGENTS.md):
   `apps/iphone-native/rust`: `cargo check --all-targets` + `cargo test`;
   `apps/android-native/rust`: `cargo test` +
   `cargo ndk -t arm64-v8a --platform 29 check`. If you change anything
   wire-visible, hunt for parity/golden tests and update them deliberately —
   never loosen a golden to make it pass; if a golden fights you, stop and
   record why in the handoff instead.

4. **S8.3b — stale-landing guard on fast re-import.** `refreshLanding`
   (`AuditionController.swift:423-437`) guards with
   `generation == landingGeneration`, bumped by `scheduleLandingRefresh` on
   settings churn. Verify whether a fast re-import / track swap bumps
   `landingGeneration` — and whether the detached `measureLanding` against
   the OLD `engine.stream` can land a stale gain on the new track. Android's
   guard is the template:
   `apps/android-native/app/src/main/java/com/yesmaster/app/AuditionController.kt:350`
   (`if (handle != h || landing.error != null) return@launch`). If the race
   is real, bump the generation at import/attach and pin it with a controller
   test; if it's already safe, close it with the evidence instead of adding
   code.

5. **IP-05 leftovers (small; each its own commit).**
   (a) `apps/iphone-native/rust/src/lib.rs:230-232`:
   `native_adaptive_context_for_path` swallows the resolve error with
   `.ok()` — WYSIWYG silently degrades when re-analysis fails. Surface the
   failure honestly (typed signal the Swift side can distinguish from
   "no adaptive context") without changing any DSP behavior.
   (b) Plist truthfulness: `project.yml:32-33` sets `UIFileSharingEnabled` +
   `LSSupportsOpeningDocumentsInPlace`, but masters are written to
   Application Support (`RenderStorage.swift:11-15`) — users see an empty
   Documents folder in Files. EITHER move rendered masters to
   `Documents/RenderedMasters` (keeps the flags honest; check
   `RenderStorage.enforceLimit` still applies) OR drop the two flags. Pick
   ONE, state the choice loudly in your handoff; do not do both.

## Verification

Full fast lane per AGENTS.md (`npm test`, `npm run build`,
`npm run build:windows`; in `src-tauri`: `cargo fmt --check`, clippy with
`-D warnings`, `cargo test --lib`, `cargo test`, all with
`--target-dir target\codex-rc`) + BOTH mobile lanes on every facade-touching
commit. No DSP constants, no preset changes, nothing listening-gated. Update
`docs/OPEN_THREADS_AND_DECISIONS.md` thread #11 and the Wave-8 rows
minimally-truthfully in the same commits as the behavior they describe.

End with a short handoff note: what shipped, what closed-as-already-done,
the one decision you made in 5(b), and the remaining owner-gated items
(S8.5 keystore, S8.6 background-audio, S8.1 archive validation on the Mac).
