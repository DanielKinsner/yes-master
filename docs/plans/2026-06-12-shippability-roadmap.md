# YES Master — Shippability Roadmap (2026-06-12)

Execution plan for `docs/reviews/2026-06-12-master-shippability-audit.md`.
Every slice below references audit finding IDs (RS/FE/XP/IP/AN/MB/PKG/UX) —
read the finding there before implementing; the audit carries the evidence
and line numbers, this document carries the how.

## Operating model

- **Implementer:** a capable non-Claude coding model (Codex GPT 5.5) working
  one slice at a time. Each slice is a single PR/commit-stack with its own
  verification lane. Slices are sized so the implementer needs no context
  beyond this doc + the audit + the cited files.
- **Reviewer:** Claude reviews each commit against the slice's "Done means"
  and "Reviewer checks" sections. The implementer must paste lane output
  (test counts, exit codes) into the PR/commit description — review is
  evidence-based, not vibes-based.
- **The user is NOT in the loop** for any mechanical slice in Waves 0–9.
  Taste, listening, calibration, keystore, and mobile-background-mode calls are
  quarantined in Wave 10 / the owner decision queue; no earlier slice blocks on
  them unless its entry says so.

### Global rules for the implementer

1. Run the repo's fast lane for every slice (CLAUDE.md "Verification"). Rust
   slices add the Rust lanes; shared-crate or `#[tauri::command]` changes add
   BOTH mobile bridge lanes (iPhone: `cargo check --all-targets` + `cargo
   test`; Android: `cargo test` + `cargo ndk -t arm64-v8a --platform 29
   check`). DSP/export slices add the slow fixture lane
   (`AMS_RUN_REAL_FIXTURE=1`).
2. **DSP byte-identity policy:** any change touching `dsp.rs`,
   `deep_analysis.rs`, render paths, or the WAV writer must leave existing
   byte/SHA snapshot tests passing untouched. If a slice intends a byte
   change, it is quarantined as an owner decision (only RS-09 qualifies
   today). "Snapshot test failed so I updated the snapshot" is an automatic
   review rejection unless the slice spec says otherwise.
3. **Tests first or tests-with:** every behavioral fix lands with the
   mechanical test named in its slice, written to fail on the pre-fix code.
   State that the test was observed failing before the fix in the commit
   message.
4. One slice per PR. No drive-by refactors, no scope creep, no formatting
   churn outside touched hunks. If you find an adjacent bug, file it in the
   PR description; do not fix it inline.
5. Copy/UI-string changes must update the pinning tests in the same commit
   (`App.chrome.test.tsx` etc.) — and only those assertions.
6. Net-LOC expectations are stated per wave. The owner was surprised the last
   refactor pass grew the codebase: Waves 1–5 will grow it modestly (fixes +
   tests — correct and expected); Wave 7 is the deliberate net-negative wave
   (dead CSS, dead components, duplicate formatters, asset bytes). Do not
   pad; do not golf.
7. **Interrogate the owner — don't guess.** Whenever a slice touches product
   taste, scope, copy tone, or anything this doc marks as a decision, ask
   the owner OPEN-ENDED questions before implementing — and prefer a real
   interrogation session over a single yes/no. If you are running in an
   environment with the `/grill-me` skill, use it (point it at the slice or
   at S5.4's canon questions) and drive to resolved answers on every branch
   of the decision tree; otherwise ask the questions directly in plain
   language, one topic at a time, concrete options + your recommendation.
   Record every answer where it belongs: product answers into
   `docs/PRODUCT.md` / `docs/APP_BEHAVIOR.md` (via S5.4 or a follow-up
   canon commit), execution answers into this file's decision queue with the
   date. An unrecorded decision is how this repo accumulated the Wave 9
   backlog — do not add to it. The owner explicitly wants these questions;
   asking is never scope creep, silently choosing is.

### Suggested slice order

Wave 0 (infra) → Wave 1 (audio-correctness P0s) → Wave 2 (frontend P1s) →
Wave 3 (contracts) → Wave 4 (mobile crash class) → Wave 5 (go-public) →
Wave 6 (UX) → Wave 7 (cleanup) → Wave 8 (mobile store-readiness mechanics) →
Wave 9 (unattended mechanical reconciliation) → Wave 10 (taste/listening).
Within a wave, slices are independent unless marked. Waves 2 and 3 can
interleave with Wave 1. Wave 0 first — it makes everything else verifiable
without the owner's machine.

---

## Wave 0 — Verification infrastructure (enables unsupervised runs)

### S0.1 — CI bootstrap (PKG-03, PKG-04)
**Files:** new `.github/workflows/ci.yml`.
**Steps:** three jobs. (a) `windows-latest`: `npm ci`, `npm test`,
`npm run build`, then in `src-tauri`: `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`; plus
`apps/android-native/rust: cargo test` and `apps/iphone-native/rust:
cargo test` (host-side both run on any OS). (b) `macos-latest`: `src-tauri:
cargo test` (this empirically validates the duplicated Mac SHA snapshots —
PKG-04), `npm ci && npm run build:mac`, and the iPhone Swift tests
(`xcodegen generate` + `xcodebuild test -scheme YESMasterNative -destination
'platform=iOS Simulator,name=iPhone 16'` — adjust to the project's actual
scheme; if simulator flakiness blocks, land rust+build first and Swift tests
as a follow-up commit). (c) `ubuntu-latest` (cheap): Android
`./gradlew test` JVM tests. Cache cargo + npm. Slow fixture lane stays
local-only by design (`AMS_RUN_REAL_FIXTURE` is simply never set in CI).
**Done means:** green run on a no-op PR; macOS `cargo test` log shows the dsp
snapshot tests executing on mac hardware.
**Reviewer checks:** job matrix matches the documented lanes; no secrets
required; failure of any lane fails the PR.
**Note:** if Mac `cargo test` FAILS on the SHA snapshots, that is a finding,
not a CI bug — it means the Mac snapshots were wrong copies (PKG-04). Report
upstream immediately; do not "fix" snapshots to match without flagging.

### S0.2 — Version coherence (PKG-02)
**Files:** `package.json`, `src-tauri/tauri.conf.json`,
`src-tauri/Cargo.toml`, new `src/lib/version-coherence.test.ts`.
**Steps:** set `0.1.0` in all three desktop manifests (matches Android's
existing `0.1.0`). Add a vitest that reads all five version sources
(3 desktop + `apps/android-native/app/build.gradle.kts` versionName +
`apps/iphone-native/YESMasterNative/Info.plist` CFBundleShortVersionString),
asserts desktop-three equality and `!== "0.0.0"`; assert mobile values parse
semver (full five-way equality is optional — mobile may legitimately lead).
**Done means:** `npm run build:windows` artifact name contains `0.1.0`.
**Reviewer checks:** no other manifest fields touched.

---

## Wave 1 — Audio correctness P0s (Rust). Net LOC: +small (guards + tests).

### S1.1 — Nyquist-safe biquads (RS-01) — **do this one first**
**Files:** `src-tauri/src/dsp.rs` (biquad constructors only).
**Steps:** in `low_shelf`/`high_shelf`/`peaking`/`butter_*` (every
constructor taking `freq_hz`), clamp the effective frequency to
`0.45 * sample_rate` (or return `Self::identity()` when
`freq_hz >= 0.5 * sample_rate` — pick ONE policy, document it in a comment
stating the constraint: RBJ formulas require ω < π). Touch nothing else in
the chain.
**Tests:** new `dsp.rs` unit test `chain_is_finite_at_all_supported_rates`:
for `sr ∈ {8000, 11025, 16000, 22050, 32000, 44100}` × every `Preset` ×
intensity 1.0, plus variants with `presence_air = Some(1.0)` and
`eq_sparkle_db = 3.0`: build `MasteringChain::new(sr, 2, &settings)`, process
1 s of 0.3-amplitude 440 Hz stereo sine, assert every output sample
`is_finite()` && `abs() < 4.0`. Must fail before the clamp (verify at 8000).
**Verification:** full Rust lane + slow fixture lane + **all existing DSP
snapshot/SHA tests pass unmodified** (the clamp may not fire at 44.1/48/96k —
that is the proof of non-regression). Mobile bridge lanes (shared crate).
**Done means:** the matrix test passes; zero snapshot diffs.
**Reviewer checks:** clamp applied in constructors (single choke point), not
call sites; no preset value changes; comment explains ω < π.

### S1.2 — Album per-track render never overwrites (RS-02)
**Files:** `src-tauri/src/album_render.rs`.
**Steps:** route per-track paths and `manifest.json` through the same
uniquify logic as the continuous WAV (reuse/generalize
`unique_album_path`-style collision handling: append `-2`, `-3`… or the
epoch-id scheme `engine.rs::unique_output_path` uses). Keep the
`NN-<stem>.wav` base naming. The manifest must reference the actual written
filenames.
**Tests:** extend the album render test module: render the same plan twice
into one temp dir; assert the first render's per-track files AND manifest
still exist byte-identical, and the second render's manifest lists the new
unique names.
**Verification:** Rust lane + slow fixture lane.
**Reviewer checks:** no change to rendered audio bytes; receipt/report paths
reflect actual filenames.

### S1.3 — Explicit export path guards (RS-03)
**Files:** `src-tauri/src/engine.rs` (`explicit_output_path`,
`mastering_render_with_progress` write site), possibly `wav_writer.rs`.
**Steps:** (a) in `explicit_output_path`, accept the source path as a new
argument; canonicalize both (fall back to lexical comparison when
canonicalize fails on the not-yet-existing output — compare the output's
parent canonicalized + filename); return
`CommandError::InvalidPath("output path would overwrite the source file")`
on match. (b) write the WAV to `<final>.tmp` (unique suffix) then
`std::fs::rename` to final; on any write error, best-effort remove the tmp
and return the error — nothing may exist at the final path on failure.
**Tests:** (a) render with `output_path == source_path` → Err, source bytes
unchanged (hash before/after); (b) point output inside a directory made
read-only (or a path whose parent is a file) → Err and final path absent.
**Verification:** Rust lane + slow fixture lane. Frontend untouched.
**Reviewer checks:** rename is same-volume (tmp lives next to final, not in
system temp); album explicit-dir path gets the same tmp+rename treatment or a
filed follow-up.

### S1.4 — Decode allocation clamps (RS-04, RS-06)
**Files:** `src-tauri/src/decode.rs`, `src-tauri/src/audio.rs:47-49`.
**Steps:** clamp `estimated_capacity` to
`min(claimed, file_size_bytes * 8)` (pass the file size in or query
metadata) — `Vec` still grows organically if the header under-reports. Clamp
`target_pixels` to `.clamp(64, 16_384)`.
**Tests:** hand-write a tiny WAV whose RIFF header claims ~u32::MAX frames
(a few hundred bytes on disk); `decode_full` returns Ok/Err without aborting
and peak RSS stays sane (the test passing at all is the assertion — abort
kills the harness). `prepare_waveform(.., Some(u32::MAX))` returns peaks
≤ 16_384.
**Verification:** Rust lane.
**Reviewer checks:** clamp math can't overflow (`u64` domain, saturating).

### S1.5 — Command robustness batch (RS-10 partial, RS-05)
**Files:** `src-tauri/src/audio.rs`, `src-tauri/src/files.rs`,
`src-tauri/src/project.rs`.
**Steps:** (a) replace the five mutex-poison `expect`s with
`map_err(|_| CommandError::Other("internal state poisoned; restart the
app".into()))?`-style recovery. (b) `files.rs:39`: stop swallowing —
return the probe error for that entry, or (if soft-import is deliberately
kept) add `probe_error: Option<String>` to `ImportedTrack`, populate it, and
expose it through bindings so the UI can render it (coordinate with S6.4's
toast). Decide with the reviewer BEFORE implementing; default
recommendation: hard error per entry, import the rest (matches the existing
partial-success analyze policy). (c) unique tmp suffix in
`write_session_atomic`. (d) `settings_landing_hash`: on serialization Err,
return a sentinel that the cache treats as always-miss.
**Tests:** per change as specified in audit RS-10/RS-05 (poisoned-mutex via
`catch_unwind`-while-holding-guard; garbage `.wav` import; two-thread
distinct-project save; NaN-bearing settings don't share a cache entry).
**Verification:** Rust lane. If `ImportedTrack` changes shape: BOTH mobile
lanes + `npm test` (bindings drift test will catch it — update
`src/bindings.ts` via the documented flow).

---

## Wave 2 — Frontend state-correctness P1s. Net LOC: +small.

All four land in `src/hooks/useTrackMaster.ts` + tests in
`src/hooks/useTrackMaster.integration.test.tsx` (FE-04 also `src/App.tsx` +
an App-level test). They are independent; land as four commits or one PR of
four commits.

### S2.1 — Paused kind-switch preserves playhead (FE-01)
At `useTrackMaster.ts:1529-1530`: start position becomes
`isAtEnd || !loadedCorrectTrack ? 0 : transport.currentTimeSec`.
**Test (write first, observe failing):** import; deliver tick
`{position_sec: 42, is_playing: false, is_loaded: true, track_id: t1}`;
`setPlaybackKind("master")`; `togglePlay()`; assert `api.playMaster` called
with start ≈ 42 (±0.1), not 0. Also assert the playing-path test still
passes (position preserved via estimate) and the at-end path still restarts
at 0.

### S2.2 — Removing the playing track stops playback (FE-02)
Add `loadedTrackId` to `removeTrack`'s dep array (and clean
`regionByTrack`/`overrideAlbum` for the removed id while there — same
callback, same commit).
**Test:** tick `is_loaded: true` for t1 → `removeTrack(t1)` → assert
`api.stopPlayback` called. Second assertion: removing a NON-playing track
does not call it.

### S2.3 — Album export reads fresh override set (FE-03)
Add `overrideAlbum` to `exportAlbumPlan`'s deps.
**Test:** album mode; `toggleOverrideAlbum(t2)` ON (settings diverge);
toggle OFF; `exportAlbumPlan()`; assert `renderAlbumPlan`'s input for t2
deep-equals `albumIntent`, not `settingsMap[t2]`.

### S2.4 — Scope the Advanced rail to the selected track (FE-04)
In `App.tsx`, pass
`lastChecks={tm.lastExportReceipt?.trackId === tm.selectedTrackId ?
tm.lastExportReceipt.checks : undefined}` (mirror commit 7bcc021's Standard
fix; confirm the receipt's track-id field name in `bindings.ts`) and apply
the same scoping to the `needsReview` computations at `App.tsx:286-298` and
the receipt-card render gate at `:207`.
**Test (App-level or RightRail-level):** export clean track A; select track B
with hot analysis (`lufs_integrated: -5`); assert rail shows "SOURCE CHECK"
preflight rows and the export button reads "Export With Review".

---

## Wave 3 — Contract hardening. Net LOC: +moderate (fixtures + tests).

### S3.1 — One import-format contract (XP-01, part of RS-05/UX-06)
**Decision baked in:** DROP aiff/opus from desktop (do not add decoders).
**Files:** `src/hooks/useTrackMaster.ts`, new `src/lib/supported-formats.ts`,
`src/components/EmptyState.tsx`, `src/App.tsx:193`,
`src/components/EmptyState.test.tsx`, `src-tauri/tests/contracts.rs:1957`.
**Steps:** create `src/lib/supported-formats.ts` exporting
`AUDIO_EXTENSIONS = ["wav","mp3","m4a","aac","flac","ogg"] as const` and a
`SUPPORTED_FORMATS_COPY` display string derived from it. Consume it from the
import filter, the file-dialog filter, EmptyState, and the App.tsx hero
line. Remove `opus`/`aiff` from the fixture scanner's extension list. Update
`EmptyState.test.tsx` (currently asserts "Opus" is shown — flip it to assert
the derived copy and that "Opus"/"AIFF" are absent).
**Tests:** (a) vitest: every extension in the dialog filter and the display
copy comes from the single exported list. (b) Rust contract test
`advertised_extensions_decode`: for each extension in the TS list (mirror the
array in the test with a comment pointing at `supported-formats.ts`, or
include_str the TS file and parse the literal — prefer include_str so drift
fails), synthesize/ship a 1-second fixture and assert `decode_full` succeeds.
(m4a/aac/mp3/ogg fixtures: generate tiny ones once and commit under
`src-tauri/tests/fixtures/formats/` — they are synthetic, not private
audio; keep each < 50 KB.)
**Done means:** UI, dialog, copy, fixture scanner, and decode capability all
agree, and a one-sided edit fails a test.

### S3.2 — Event identity + analysis ownership (XP-03 cluster; supersedes
Codex F-03's narrower scope)
Split into two commits.
**Commit A (backend):** `src-tauri/src/engine.rs` — add `batch_id: String`
(caller-supplied or uuid-per-invocation) to `AnalysisProgress`; thread it
through `analyze_tracks` and its progress closure. Add `track_id` to the
landing-status event payload (`audio.rs` emit site). Update
`src/lib/api.ts` event types and `src/bindings.ts` per the drift flow.
Mobile lanes run (shared types may be touched — if `AnalysisProgress` lives
in the shared crate, both bridges rebuild).
**Commit B (frontend, `useTrackMaster.ts`):**
1. Keep a `currentAnalysisBatchRef`; `onAnalysisProgress` ignores events
   whose `batch_id` doesn't match.
2. Replace boolean `isAnalyzing` with an in-flight counter (`useRef<number>` +
   state projection `isAnalyzing = count > 0`); import, restore, and
   open-project all increment/decrement it — restore/open become visible
   work (also resolves UX "invisible restore analysis").
3. Session restore merges: `setAnalysisMap(prev => ({...prev, ...map}))`.
   (Open-project keeps replace — tracks were replaced — but takes the counter
   + batch guard.)
4. Tick handler ignores ticks whose `track_id` doesn't match the loaded/
   selected expectation (preserve the `is_loaded` bookkeeping semantics —
   keep `setLoadedTrackId` unconditional, gate only the transport paint).
5. Landing-status handler ignores mismatched `track_id`.
**Tests (integration, each observed failing first):** (a) overlapping
imports: start A, start B, resolve A → `isAnalyzing` still true; late
events from A's batch don't repaint. (b) restore-then-import: restore's
late resolution does not evict the imported track's analysis. (c) restore
sets analyzing state while pending. (d) select B, deliver tick for A →
`transport.currentTimeSec` unchanged. (e) Rust: contract test asserting the
emitted analysis event carries `batch_id`.
**Reviewer checks:** render-progress pattern reused, not reinvented; no
`useTrackMaster` structural split (explicitly out of scope per the
do-not-do list).

### S3.3 — Export-recipe + vocabulary parity fixture (XP-04)
**Files:** `src/standard-mapping-parity.json`,
`src/lib/standard-export.test.ts`, `apps/iphone-native/rust/src/lib.rs`
(test mod), `apps/android-native/rust` (test), iPhone Swift test target
(`project.yml` resource + new/extended `NativeLoudnessTests.swift`).
**Steps:** add to the fixture:
`"delivery": {"sample_rate": 44100, "bit_depth": 24, "ceiling_dbtp": -1,
"lufs_clamp": [-24, -6]}`. Update the `_readme` to name all five consumers.
Assert from: TS (`standard-export.test.ts` reads the JSON instead of local
literals), iPhone Rust (`fixed_export_settings_match_simple_iphone_target`
reads the include_str'd fixture), Android Rust (same via its existing
fixture include), Swift (bundle the JSON into the test target; assert
`NativeLoudness` trio and `bridgeIdentifier` mapping against it). Also: make
`native_preset`'s unknown-id fallback log in debug builds (audit XP-04 tail).
**Done means:** changing any one of 44_100 / 24 / -1.0 / clamp on one side
fails at least one other side's lane.

### S3.4 — Truthful chrome copy + Standard-aware Help (XP-02, UX-06 partial)
**Files:** `src/lib/chrome-content.ts`, `src/App.chrome.test.tsx`.
**Steps:** Export Defaults rows become view-honest, e.g.
`["Standard · Create Master", "44.1 kHz, 24-bit WAV, −1 dBTP"]` and
`["Advanced · delivery profile", "Streaming Universal — 48 kHz, 24-bit
WAV"]`. Derive the Standard string from `standardExportSettings` constants
(import them; do not restate literals). Help: add a leading "Standard view"
section (Styles, loudness trio, Create Master, where the master lands), a
"Keyboard shortcuts" section (Space, Ctrl/Cmd+Z/Y, Shift+drag loop region —
Advanced only), and a 3-term glossary line (LUFS, dBTP, dynamic range) in
plain words. Retitle the Settings dialog header toward honesty ("Current
defaults" framing) — full real-settings work stays out of scope.
**Tests:** update the chrome pins; add an assertion that the Standard row's
rendered string equals the values exported by `standard-export.ts`.

### S3.5 — FFI header pin (XP-05)
**Files:** `apps/iphone-native/rust/` (build tooling), header.
**Steps:** preferred: add `cbindgen` as a dev/build tool, generate the
header, diff against the committed one in the iPhone lane (`cargo test`
gains a `#[test]` that runs cbindgen-as-lib and compares normalized output,
so no CI plumbing needed). If cbindgen output churn is unmanageable,
fallback: a Rust test that parses the committed header's declarations and
compares name/arity/types against a hardcoded mirror of the extern list
adjacent to the externs.
**Done means:** changing an extern signature without the header fails the
iPhone lane.

---

## Wave 4 — Mobile crash class. Net LOC: +small.

### S4.1 — Panic guards on every iPhone extern (IP-01)
**Files:** `apps/iphone-native/rust/src/lib.rs`, `live_stream.rs`.
**Steps:** wrap every `#[no_mangle] extern "C"` body in
`std::panic::catch_unwind(AssertUnwindSafe(..))`; on Err return the
existing `{"error": ...}` JSON (string fns) / null / no-op-safe default
(handle fns), mirroring Android's `catch_panic` exactly (it is in
`apps/android-native/rust/src/lib.rs:57-63` — copy the shape, share a
helper in the facade if clean). Add a `#[cfg(test)]`-gated injection point
(e.g. a path constant that triggers `panic!`) and a test asserting
`yes_master_native_analyze_file_json` on it returns error JSON.
**Verification:** iPhone lane + Android lane (Android links the facade).
**Reviewer checks:** every extern covered (grep `no_mangle` count vs
guarded count); no behavior change on success paths.

### S4.2 — Android destroy/measure race + create leak (AN-01)
**Files:** `apps/android-native/app/.../AuditionController.kt`; possibly
`apps/android-native/rust/src/audition.rs`.
**Steps:** belt and suspenders. Kotlin: make `release()` wait for the
landing job (`runBlocking { landingJob?.cancelAndJoin() }` is acceptable at
release-time, or restructure release into the scope). Make `prepareJob`'s
`createNative` non-cancellable (`withContext(NonCancellable)`) and destroy
the result if the coroutine was cancelled before attach. Rust: serialize
`measure_landing` against Drop (RwLock: measurements take read, drop takes
write — keeps the deliberate non-blocking-vs-UI-mutex property for
everything except teardown).
**Tests:** Rust host test: thread A loops `measure_landing`, thread B drops
the engine; must not segfault across 100 iterations (run under the normal
lane; it would crash pre-fix). Kotlin JVM test with a fake bridge asserting
destroy is not invoked until the in-flight landing completes.
**Verification:** Android lane (host tests + ndk check).

### S4.3 — AAudio stream verification + close barrier (AN-02)
**Files:** `apps/android-native/rust/src/aaudio.rs`.
**Steps:** after `openStream`, query
`AAudioStream_getChannelCount/getFormat/getSampleRate`; if any differ from
the request, close and return Err (maps to the existing "Playback could not
start" surface). Close barrier: loop `waitForStateChange` until state ==
`STOPPED` (with deadline) before `close`.
**Tests:** host-side unit tests of the new verification logic with the
stream API behind the existing trait/mock seam if present (the crate already
fakes streams for engine tests); device behavior remains a QA item — note
it in the manual sweep list.
**Verification:** Android lane.

### S4.4 — iPhone import off the main thread + single decode (IP-02, MB-01)
**Files:** `AuditionController.swift`, `live_stream.rs`,
`apps/iphone-native/rust/src/lib.rs`.
**Steps:** two parts. (a) Swift: make `importTrack` async — set a
`.preparing` state, run the copy + `engine.load` off the main actor
(`Task.detached` / actor hop), publish completion back on main (mirror
Android's `Dispatchers.IO` create). (b) Rust: `live_create` currently
decodes twice (stream + adaptive context). Add a facade path that decodes
once and feeds both (e.g. `native_adaptive_context_for_pcm(samples, ...)` or
fold context resolution into create). Import-time duration cap: if decoded
duration > 30 min, return the error JSON with a clear message ("This track
is too long to master on this device") — pin the threshold in one constant;
unit-test the cap.
**Tests:** Rust: decode-count proxy (a counter hook or the existing timing
proxy un-ignored and tightened) asserting one decode per create; cap test.
Swift: state-machine test that import enters `.preparing` before completion
(fake/slow stream seam exists per the test rig).
**Verification:** iPhone lane + Android lane; Swift tests via S0.1's mac job.

---

## Wave 5 — Go-public repo work. Net LOC: ~0 (docs/config).

### S5.1 — README rewrite + username scrub (PKG-05)
Rewrite README top: what YES Master is (one paragraph from PRODUCT.md's
core promise), screenshot placeholder block, platform support
(Windows/macOS desktop; mobile in development), build-from-source for both
OSes (PowerShell AND bash variants), license pointer, "internal docs" note.
Move verification-lane detail to `docs/TESTING.md`. Scrub machine-local
absolute paths from the 9 tracked docs (repo-relative paths).
**Verify:** no tracked local-username hits remain;
`git grep -l "C:\\\\Users"` → test files only.

### S5.2 — Baseline CSP (PKG-05/RS-10 tail)
`tauri.conf.json`: `"csp": "default-src 'self'; img-src 'self' asset: data:
blob:; style-src 'self' 'unsafe-inline'; font-src 'self' data:"` — iterate
in `tauri dev` until waveform, preset PNGs, dialogs, and the spectrum render
(asset: protocol needs the img-src entry). Add a vitest config-lint
asserting `security.csp !== null`.
**Verify:** `npm run build` + a dev smoke; all existing layout/chrome tests.

### S5.3 — Lane/docs truth-up (PKG-05 tail, IP-05 doc item)
Add Android lane to `docs/TESTING.md` and an `android` branch to
`scripts/verify-fast.ps1` (toolchain-detection guard). Fix `.gitignore:7`
stale exe name. Fix `docs/IPHONE_APP.md` preset table to the shipped
Standard vocabulary (point it at `standard-mapping-parity.json` as the
source of truth). Fix `docs/RELEASE_STABILIZATION.md` undo/redo claim to
match reality once S6.1 lands (coordinate ordering).

### S5.4 — Canon refresh: make PRODUCT.md the actual lighthouse
(audit Part 4.1) **[gated on an owner interrogation session — run it
per global rule 7 / `/grill-me` before writing a word]**
**Files:** `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`.
**Steps:** an interrogation session with the owner first, covering at
minimum: (a) mobile's product definition — audience, scope (audition +
fixed Standard export?), parity promises, what is deliberately absent on
phones, whether mobile is part of the v1 public push; (b) Album Master's
product promise (album intent, arc kinds, override semantics — currently
code-only); (c) how the canon should describe the adaptive engine
(user-facing claim vs internal mechanism — note the recorded decision that
confidence stays backend-only); (d) the track-aware-compressor direction
(audit Part 4.3). Then rewrite: Primary Workflow becomes the STANDARD flow
with an Advanced subsection (today it describes Advanced); add a Mobile
section; add an Album Master section; add an Adaptive Mastering section
stating what adapts today (loudness/profile) and what deliberately does not
(tone tilt, compressor) and the stated direction; keep the compressor canon
but scope its "not track-aware" sentence to the compressor explicitly.
APP_BEHAVIOR.md gains matching "what the app does now" entries (mobile
apps exist + their behavior contract; adaptive resolution summary).
**Done means:** a fresh agent reading PRODUCT.md alone would build toward
Standard-first desktop + defined mobile scope, and could not conclude "the
app is not track-aware" or "mobile doesn't exist."
**Reviewer checks:** every new canon claim traceable to an owner answer or
verified code behavior; no aspirational features stated as current.

---

## Wave 6 — UX wins (desktop). Net LOC: +moderate. The default-path trio
(S6.1–S6.3) is the highest user-visible value per line of code in this plan.

### S6.1 — Undo/redo buttons restored (UX-01)
**Files:** `src/App.tsx` (header region), `src/App.css` (reuse existing
header-tool styles), App-level test.
Compact header buttons wired to `tm.undo`/`tm.redo`, disabled from
`canUndo`/`canRedo`, `aria-label` + title with the shortcut ("Undo —
Ctrl+Z"). Both views (Standard header too — confirm placement against the
accepted centered layout; if Standard placement is contentious, Advanced
now + Standard noted for the owner pass).
**Test:** buttons render, disabled states track history, click calls
undo/redo.

### S6.2 — Manual re-analyze action (UX-02)
**Files:** `useTrackMaster.ts` (new `reanalyzeTrack(id)` /
`reanalyzeAll()` callbacks reusing the import-analysis pipeline INCLUDING
its counter/batch semantics from S3.2), surfaced as: a "Re-analyze" affordance
on the not-analyzed state in `StandardView.tsx` (where "Not analyzed" copy
renders) and in the Advanced track header area.
**Test:** integration — failed-restore track shows the action; invoking it
calls `api.analyzeTracks` with that track and merges the result; Create
Master enables.

### S6.3 — Standard render progress (UX-04)
**Files:** `src/components/StandardView.tsx`.
Consume `tm.renderProgress` (already in the hook's return): while
`isRendering`/`isExporting`, the Create Master button (or a slim bar under
it) shows the real fraction; label from `kind`. No new backend work.
**Test:** StandardView test — feed `renderProgress {fraction: 0.4}` →
progress visible; `null` → not.

### S6.4 — Unsupported-drop feedback (UX-06, pairs with S3.1/S1.5b)
In the drop handler (`useTrackMaster.ts:1015-1028`): when files were
dropped but zero survive the extension filter, surface a toast naming the
rejected extensions and the supported list (from `supported-formats.ts`).
**Test:** integration — drop `["a.mp4"]` → error/toast state set, no import
call.

### S6.5 — Friendly-error mapping layer (UX-06)
**Files:** new `src/lib/user-errors.ts`, `Toast` call sites.
Map known backend error shapes (`Decode`, `InvalidPath`, `Io`, timeout) to
action-oriented copy ("Couldn't read <name>. The file may be moved or in
use — re-import it or use Re-analyze."); unknown errors keep the raw string
appended in a collapsed/secondary line. Also stop errors suppressing pending
project feedback (render both, audit UX-06).
**Test:** unit-test the mapper; App test that a Decode error renders the
friendly form.

### S6.6 — Missing-source relink flow (UX-05) **[M — schedule after the trio]**
Open-project failure path: per-track recovery state ("source missing") on
the track row + actions Locate… (file dialog, re-probe, re-analyze via
S6.2's pipeline, rebuild waveform) and Remove. Toast names the affected
tracks.
**Test:** integration with a mocked failed probe → row shows state; locate
flow calls probe/analyze with the new path.

### S6.7 — Cancelable long operations (UX-03) **[L — two commits]**
**Commit A (backend):** cancellation tokens for `analyze_tracks` and the
render commands — an `AtomicBool` registry keyed by batch/track id
(S3.2's ids), checked at the existing progress-callback boundaries
(analysis stage boundaries; render block loop). New `#[tauri::command]
cancel_job(id)`. Cancelled jobs return a distinct
`CommandError::Cancelled`; render cancellation removes the tmp file
(S1.3's tmp+rename makes this clean — depend on S1.3).
**Commit B (frontend):** Cancel affordance on the analysis bar and render
progress surfaces; `Cancelled` errors render as quiet info, not red toasts.
**Tests:** Rust — cancel mid-analysis via a progress-callback hook →
Cancelled, no partial outputs at final paths. Integration — cancel button
calls the command and clears busy state.

### S6.8 — Close-guard + completion notification (UX-06 tail)
`onCloseRequested` (Tauri window event): if a render/export/album render is
in flight, confirm dialog ("An export is still running — quit anyway?").
After successful export while window unfocused, fire a system notification
(Tauri notification API — needs the plugin + capability; keep it optional:
skip silently if permission unavailable).
**Test:** unit-test the guard predicate; manual QA note for the dialog
itself.

### S6.9 — Copy/affordance batch (UX-06 tail)
One commit of small verified items: Volume Match tooltip in Standard
(mirror the Advanced one); preset alias subtitles in Advanced ("Universal ·
Balanced in Standard") per `standard-mapping.ts`; hide the loop-region hint
in Standard OR add the loop toggle (recommend: hide the hint —
one-liner); a copy line when Album tab jumps to Advanced ("Album Master
lives in the Advanced view"); WAV-only expectation line in the Delivery
Format card.
**Tests:** update the pinned-copy tests touched; add presence assertions
for the new tooltip/subtitles.

### S6.10 — Recent projects (UX-07) **[M, optional before launch]**
Persist a small MRU (last 8 `.ams.json` paths + timestamps) in the existing
settings store; "Open Recent" list in the project menu/chrome; prune
missing files on open. File-association/single-instance work stays
deferred — file it, don't build it now.

---

## Wave 7 — Cleanup and perf: the net-NEGATIVE wave.
Expected net: **−1,000 LOC or better, −10 MiB assets.**

### S7.1 — CSS selector inventory + dead-block deletion (XP-07)
**Commit A:** new `src/App.layout-css.test.ts`-adjacent inventory test:
parse `App.css` class selectors (regex over `.<ident>` at rule level is
fine), check each against a production-source string scan (`src/**/*.{ts,tsx}`
excluding `*.test.*`), fail on unmatched selectors not in an explicit
allowlist (template-literal/dynamic classes get allowlisted with a comment
naming the construction site). Generate the inventory fresh — Codex's 71
number is stale.
**Commit B:** delete the verified-dead blocks (start with the 14
re-confirmed classes in the audit). Run the visual smoke: `npm run build` +
dev-launch screenshot pass over first-run, Standard, Advanced, export
receipt (agent-run via the preview tooling — no owner needed).
**Reviewer checks:** commit B touches only CSS deletions; allowlist entries
each carry a justification comment.

### S7.2 — One duration formatter (XP-06)
New `src/lib/time-format.ts`: `formatDuration(sec: number | null |
undefined): string` — explicit policy: non-finite/null → "", floor
seconds, `m:ss`. Replace all four call sites (`App.tsx` ×2,
`StandardView.tsx`, `AlbumPanel.tsx`). Policy note: floor (not round)
wins — three of four surfaces are transport-adjacent where 59.6 s reading
"1:00" before the minute boundary is wrong.
**Tests:** unit: null, NaN, Infinity, 0, 59.6 → "0:59", 60 → "1:00",
3599.9. Update any pinned strings that asserted rounded values.

### S7.3 — Clean test lane (XP-08)
Fix `src/App.transitions.test.tsx` first-frame test to own its async
probe (keep `loadRecentSession` unresolved while asserting the first
frame; resolve inside `await act(...)`). Then enforce: vitest setup gains a
`console.error` spy that fails the suite on `not wrapped in act` matches
(scoped allowlist if any third-party noise exists).
**Done means:** `npm test` output contains zero act warnings, enforced.

### S7.4 — Preset asset optimization (XP-09)
Re-encode the seven unoptimized PNGs to match universal.png's treatment
(target ≤ 600 KB each at the rendered resolution ×2; lossless/visually
lossless, no new runtime deps). New asset-budget vitest: each preset file
≤ 700 KB, total ≤ 5 MB.
**Reviewer checks:** visual diff of the preset strip (preview screenshot)
attached to the PR.

### S7.5 — Deep-analysis scratch buffer (XP-10)
Hoist one `Vec<f32>` mono scratch through `scan_windows` →
`measure_window` (resize per window, reuse allocation). Byte-identity
proof: existing deep-analysis tests + a new fixed-fixture test comparing
full `WindowMetrics` output pre/post (commit the expected values).
**Verification:** Rust lane + slow fixture lane + mobile lanes (shared
crate).

### S7.6 — Dead-code deletions (FE-06/IP-05 tails)
One commit: `ExportReceiptCard` album branch (delete — album reports
surface via AlbumPanel; wire-it-instead requires an owner call, default
delete), `TrackPlaybackController.swift` + `AudioSessionController.swift`
(+ their tests), `VisualEqPanel.tsx` stale header comment. Grep-verify
zero production references before each deletion.

### S7.7 — Backend P3 batch (RS-08, RS-10 tails)
Spectrum analyzer rebuilt with the current rate in `handle_play_master`
(+ the pure-helper test from the audit); `spawn_blocking` around the four
heavy command bodies; play-timeout cancellation epoch; callback-alloc
swap (preallocated second chain) — each small, each with the audit's named
test where feasible. RS-09 (limiter flush) is NOT here — owner decision.

---

## Wave 8 — Mobile store-readiness (when mobile ships; not desktop-blocking)

- **S8.1** iPhone `PrivacyInfo.xcprivacy` (IP-03): file-timestamp
  required-reason C617.1; add `ITSAppUsesNonExemptEncryption = false`;
  wire into `project.yml`. Verify with an archive validation run on the mac
  lane.
- **S8.2** iPhone real progress or indeterminate (IP-05): minimum slice =
  replace the staged-percent theater with an indeterminate spinner +
  honest labels; real progress plumbing is a follow-up.
- **S8.3** iPhone error-state enum (IP-05) + stale-landing guard (mirror
  Android's `handle != h`).
- **S8.4** Android monochrome icon layer, Done-screen share/play intent,
  `copyToCache` extension check (already specced as A4 item B2).
- **S8.5** Android signing + `bundleRelease` (AN-03) — needs the owner's
  keystore (decision queue).
- **S8.6** Background-audio decisions (IP-04 + Android foreground service)
  — decision queue first.

---

## Wave 9 — Unattended-feature reconciliation (audit Part 4.2)

The handoff comb surfaced work that was promised, half-built, or queued and
then dropped without a recorded decision. Wave 9 exists to close the mechanical
loops by building, by recording an explicit deferral, or by deleting dead seams.
None of it blocks Waves 0–7. Decision/taste/listening items now live in Wave 10
so the mechanical ship queue can close without waiting for multi-day ear work.

**Mechanical slices (no owner gate; implement like any other wave):**
- **S9.1** Android A4 catch-up (UF-C2, UF-C3, UF-C4): wire the dead
  `supportsImportExtension` JNI seam into the SAF import path (fail fast
  before `copyToCache` — merges with S8.4's check), add import-cache
  reaping (keep current + N most-recent, reap on attach), land the
  tidiness riders. Tests per the A4 plan
  (`docs/plans/2026-06-10-001-android-a4-action-plan.md` — treat it as the
  spec; it was queued and never run). Android lane.
- **S9.2** Android process-death restore (UF-C1): SavedStateHandle per the
  A4 plan's B1 section. Android lane + a JVM state-restore test.
- **S9.3** iPhone audio-thread hardening (UF-B2, UF-B3): FPCR
  flush-to-zero on the render thread; preallocated coeff double-buffer
  replacing the in-callback Vec clones (`live_stream.rs:181`) — same
  pattern as desktop's RS-10 callback-alloc item; do both with one test
  rig. iPhone lane.
- **S9.4** iPhone live-vs-export landing rate reconciliation (UF-B4):
  measure the live landing window at (or corrected to) the export rate so
  WYSIWYG holds on non-44.1k sources; pin with a 48 kHz fixture test
  asserting live and export landings agree within the documented window
  error. iPhone lane.
- **S9.5** Album report honesty fields (UF-D): add source/rendered
  sample-rate fields to `AlbumRenderReport`, surface the upsample advisory
  line and the requested-vs-rendered album check (mirror Track Master's).
  Rust + frontend + both pinned tests.
- **S9.6** `stereo_width` disposition (UF-A1): code-reality disposition
  recorded 2026-06-16 — retain. The field is read by the analysis summary UI,
  album transition/texture scoring, source-profile finiteness checks, reference
  tuning ledgers, and preset/DSP width baselines. No new stereo-width taste
  wiring lands before Wave 10.
- **S9.7** Branch hygiene (UF-E): executed 2026-06-16. `origin/vera/standard-polish`
  and `origin/vera/ui-overhaul` were stale Vera-era experiment snapshots, not
  launch baselines; both remote branch names were deleted.

---

## Wave 10 — Taste/listening/calibration (deferred owner work)

Wave 10 is deliberately after the mechanical shippability queue. The owner
wants the app in a perfect testable state first, then can spend a couple of
days on taste without holding UX/UI or correctness work hostage.

- **S10.1** Manual Listening Gate: normal, already-mastered/compressed,
  long-source, 8 kHz, and 11.025 kHz sweeps; clean-vs-warning export
  comparison by ear.
- **S10.2** Reference Retune / preset voicing: Oomph caution, measurement
  harness, numeric fingerprints, and any DSP constant retune. No preset retune
  lands before this wave.
- **S10.3** Already-mastered matrix listening signoff and Phase-B confidence
  calibration (former S9.8 / UF-A5). Until it runs, Tier-2 confidence remains
  dormant.
- **S10.4** Adaptive engine follow-ons (former S9.9): tilt-vs-reference,
  density-cap reshape, loss budget, PSR transient protection,
  already-mastered stand-down, measured neutral, and AnalysisSummary.
- **S10.5** Compressor intelligence / Adaptive Tier 3 (former S9.10), if the
  owner still wants it after the measured/taste pass.
- **S10.6** iPhone Instruments RT profiling and intensity-perceptibility call
  (former S9.11), bundled with the mobile-shipping decision.
- **S10.7** Mobile background-audio behavior and Android signing/keystore work
  when mobile shipping is scheduled.

## Decisions recorded 2026-06-12 (owner-answered; these supersede the
matching queue items and any conflicting text above)

- **D1 License: source-available proprietary.** `LICENSE` is committed at
  the repo root (PKG-01 closed). Owner may later swap for PolyForm
  Noncommercial; the file carries that note.
- **D2 v1 is desktop-first**, but per-device plans are authored now:
  `2026-06-12-desktop-shippability-plan.md`,
  `2026-06-12-iphone-shippability-plan.md`,
  `2026-06-12-android-shippability-plan.md`. The desktop plan owns launch
  sequencing; this roadmap remains the slice-spec library.
- **D3 Adaptive compressor is IN the MVP.** Spec:
  `2026-06-12-adaptive-compressor-mvp-spec.md` (supersedes S9.10's
  post-launch framing and the pre-launch suggestion-layer option — do not
  build the suggestion layer). The owner moved the listening/calibration
  sitting to Wave 10 on 2026-06-16.
- **D4 Tier-1 adaptive voicing is owner-listened and accepted** (2026-06-11
  live session, 96 kHz auditioning; `201e746` is the artifact). Recorded in
  RELEASE_STABILIZATION. New taste/listening work now waits for Wave 10.

## Owner decision queue (the ONLY user-gated items; async — answer in any
order, nothing in Waves 0–7 blocks on them except where named)

1. **License:** ~~decided~~ — see D1 above.
2. **Min window size (PKG-05/UX):** support 1366×768 laptops (real layout
   work, schedule as a future L slice) or document the requirement in
   README (one line). Default if unanswered: document.
3. **Internal docs corpus (PKG-05):** keep handoffs/reviews/graphify-out
   public, or move under `docs/archive/` with a disclaimer. Default:
   keep, add a README disclaimer line (S5.1 includes it).
4. **Limiter flush (RS-09):** changes export bytes (~3 ms). Decide
   explicitly; if yes, it needs a snapshot-regeneration slice + a listening
   spot-check. Default: defer, document as known behavior.
5. **Background audio on mobile (IP-04/S8.6)** and **Android keystore
   (S8.5)** — Wave 10 / mobile-shipping decision.
6. **Eight presets vs curated grouping (UX-08):** taste/listening-dependent
   per working style — needs your listening note before any change. Default:
   no change.
7. **Mobile in v1:** ~~decided~~ — see D2 above (desktop-first, per-device
   plans authored).
8. **Compressor:** ~~decided~~ — see D3 above (adaptive compressor in MVP
   per the spec; suggestion layer superseded).
9. **Phase B calibration hour:** moved to Wave 10 — see D4 and S10.3.

## Listening gate (quarantined in Wave 10)

Manual Listening Gate, Reference Retune listening (Oomph caution), and
already-mastered matrix signoff remain owner-by-ear items. **One addition:**
include an 8 kHz and an 11.025 kHz source in the manual sweep — after S1.1
they must play/export cleanly (pre-S1.1 they screech; that contrast is the
audible proof the fix mattered). These no longer block closing the mechanical
Wave 0–9 queue.

## Do-not-do list (carried from Codex, still in force)

- DSP retune / preset calibration without owner listening notes.
- The parked one-pole/soft-knee hoist; tauri-specta.
- Splitting `App.css` structurally; broad `useTrackMaster` decomposition.
- Reference-track UX expansion; signing/notarization/autoupdate;
  album channel-count parity (deferred slice).
- New image-optimization runtime dependencies.
- Updating DSP snapshots to make a failing test pass (review-rejection
  offense; see global rule 2).

## Review protocol (for the reviewing model)

Per PR: (1) diff stays inside the slice's named files ±test files;
(2) the named test exists, and the commit message states it failed pre-fix;
(3) lane output pasted and plausible (test counts match the baseline:
frontend 387+new, rust 304+new lib tests; numbers grow, never shrink);
(4) DSP-adjacent → snapshot tests untouched; (5) copy changes → pinned
tests updated in-commit, nothing else re-pinned; (6) no new `unwrap`/
`expect` on user-input paths; (7) net-LOC sanity vs the wave's stated
expectation. Two rejections on the same slice → escalate to the owner with
a one-paragraph summary instead of a third attempt.
