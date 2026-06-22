# YES Master — Master Shippability Audit (2026-06-12)

Audit-only pass. No source changes were made. This document is the consolidated
source of truth for "is the whole repo — desktop app, iPhone app, Android app —
a shippable piece of software across all devices," superseding (and absorbing)
`docs/archive/reviews/2026-06-13-codex-refactor-audit.md`.

Companion document: `docs/plans/2026-06-12-shippability-roadmap.md` — the
execution plan that turns every finding below into an implementable slice.

## Method and trust levels

Five parallel deep-read sweeps (frontend, Rust backend, mobile bridges,
packaging/release, product/UX) plus a line-by-line validation of the Codex
refactor audit. Every finding carries a trust tag:

- **[V]** — verified by the lead auditor by direct read of the cited lines
  (and, where claimed, by re-running the cited command).
- **[A]** — verified by a sweep agent that read the cited code; lead auditor
  reviewed the evidence chain but did not independently re-read the lines.
- **[I]** — inferred from code; needs one manual run to confirm felt behavior.

Verification baseline re-established during this audit:

| Lane | Result |
| --- | --- |
| `npm test` | Passed — 42 files, 387 tests. `act(...)` warnings from `src/App.transitions.test.tsx` present (see XP-08). |
| Rust / mobile lanes | Green per the 2026-06-13 Codex baseline; not re-run here (no source changes were made between that run and this audit). |

## Part 1 — Verdict on the Codex refactor audit

**Overall verdict: VALIDATED.** All nine findings are real; none refuted. The
audit's evidence style is sound, but its file measurements were taken against
a slightly different tree, and it under-counted two findings. Corrections:

| ID | Verdict | Corrections / notes |
| --- | --- | --- |
| F-01 AIFF/Opus advertised, not decodable | **Confirmed [V]** | Also checked feature unification: rodio's `symphonia-all` does NOT pull in aiff or opus, so the gap is real end-to-end. Absorbed into XP-01. |
| F-02 48 kHz settings copy vs Standard 44.1 kHz | **Confirmed [V]** | Absorbed into XP-02. |
| F-03 Analysis progress has no identity | **Confirmed [V]** | Bigger than Codex scoped it: the same identity-less pattern hits `landing:status`, the restore/open flows never set `isAnalyzing`, restore REPLACES `analysisMap`, and the tick handler ignores `track_id` it already receives. Absorbed into XP-03 (cluster). |
| F-04 Standard recipe duplicated, no parity fixture | **Confirmed [V]** | Wider than Codex scoped it: Swift's vocabulary/loudness side is also unpinned, and Android consumes the recipe through the facade. Absorbed into XP-04. |
| F-05 Dead CSS selector blocks | **Confirmed (substance) [V]** | Codex measured 6,010 lines; the file is 5,290 — its inventory numbers (470/71) are from a different tree state and must be regenerated. All 14 classes Codex named were re-checked: every one has zero production TS/TSX references. |
| F-06 Deep-analysis per-window mono alloc | **Confirmed [V]** | Framing correction: ~262 MiB is cumulative allocation churn across ≤4,200 windows, not peak memory (each Vec drops per window; peak ≈ 64 KiB). Correctly P2, not worse. |
| F-07 12.5 MiB preset PNGs eagerly imported | **Confirmed [V]** | Measured 12,539,183 bytes across 8 PNGs; 7 of 8 unoptimized (universal.png was already done). |
| F-08 `act(...)` warnings in green lane | **Confirmed [V]** | Reproduced on this tree. |
| F-09 Duplicate duration formatters | **Confirmed + extended [V]** | Codex found 3; there are 4. `src/components/AlbumPanel.tsx:36` also duplicates (rounds, like App.tsx; StandardView floors). |

Codex's "Do not do" list and execution order are sound and are carried into
the roadmap. Its P0 assessment ("none found") was wrong only because its scope
was refactor-readiness, not shippability — the sweeps below found P0s outside
that scope.

## Part 2 — Consolidated findings registry

Severity model:

- **P0** — silent audio/data corruption or destruction, or a structural
  go-public blocker. Ship-stopping.
- **P1** — fix before public: crash classes, non-negotiable contract
  violations, user-facing wrong state.
- **P2** — fix soon: contract-drift risk, trust-eroding copy, robustness.
- **P3** — polish, hygiene, deferred-with-eyes-open.

### 2.1 Desktop Rust backend (RS)

**RS-01 [P0] [V] Unstable EQ filters above Nyquist silently corrupt audition
and exports for low-sample-rate sources.**
`src-tauri/src/dsp.rs:81-106` (`high_shelf`) and the peaking/shelf constructors
never clamp `freq_hz` below Nyquist; `ChainCoeffs::from_settings` places fixed
bands at 6 kHz (`high`), 10 kHz (`presence_air`), 12 kHz (`sparkle`). The chain
runs at the SOURCE sample rate in both audition (`audio.rs` `live_sample_rate`)
and offline render (`engine.rs:713` builds `MasteringChain::new(pcm.sample_rate, ...)`;
resampling happens after, at `engine.rs:736-741`). At 8 kHz ω = 2π·6000/8000 > π:
the RBJ formulas are invalid, poles leave the unit circle (numerically checked:
|p| ≈ 3.45), and every preset has a nonzero 6 kHz air baseline — so simply
importing and playing/exporting an 8–16 kHz file (speech memo, old sample)
produces exponential blowup → limiter-pinned screech → f32 overflow → NaN,
which the WAV quantizer writes as silence. No error anywhere. The app imports
these files without complaint.
*Fix shape:* clamp filter `freq_hz` to ~0.45·sample_rate (or return identity
above Nyquist) in the biquad constructors. Bit-identical at 44.1/48/96 kHz, so
existing DSP snapshots prove non-regression.
*Mechanical test:* matrix over `sr ∈ {8000, 11025, 16000, 22050, 44100}` ×
all presets (+ presence_air/sparkle variants): process 1 s of 0.3-amplitude
sine through `MasteringChain`, assert every output sample `is_finite()` and
|s| < 4.0.

**RS-02 [P0] [V] Album re-render silently overwrites prior per-track WAVs and
manifest.** `src-tauri/src/album_render.rs:368-378` writes deterministic
`NN-<stem>.wav` names with truncating `write_wav`; `:416` rewrites a fixed
`manifest.json`. Only the continuous WAV is uniquified. Default output dir is
the shared `app_data/renders/albums` (`engine.rs:627-640`): rendering an album
twice, or two albums with colliding track names, destroys prior renders with
no warning. Direct violation of "exports never overwrite prior renders by
default."
*Mechanical test:* call `render_album_plan_impl` twice into one temp dir;
assert the first render's per-track files and manifest still exist with their
original bytes (mirror of the existing continuous-WAV uniqueness test).

**RS-03 [P0] [V] Explicit export path can overwrite the source file; failed
writes leave plausible-looking corpses.** `src-tauri/src/engine.rs:871-887`
(`explicit_output_path`) checks only emptiness/filename; nothing compares the
output path to the source path. Decode happens first, so rendering onto the
source even "succeeds" while destroying the original — violating "exports
never overwrite source files." Additionally `write_wav` truncates the final
destination immediately, so a mid-write IO failure (disk full, removable
drive) leaves a truncated, header-valid WAV at the user's chosen path.
*Fix shape:* reject canonicalized `output == source` with `InvalidPath`; write
to `*.tmp` + rename (the pattern `project.rs:77-87` already uses).
*Mechanical tests:* (a) render with output == source → error, source bytes
unchanged; (b) injected IO failure → no file at the final path.

**RS-04 [P1] [V] Untrusted header `n_frames` drives `Vec::with_capacity` —
a malformed file can abort the whole app.** `src-tauri/src/decode.rs:70-81`.
A corrupt/crafted FLAC or MP4 header claiming absurd frame counts forces a
multi-hundred-GB reservation → allocator failure → process **abort** (not
catchable) during import/analyze/play, killing unsaved session work.
*Fix shape:* clamp `estimated_capacity` by a bound derived from file size.
*Mechanical test:* tiny fixture whose header claims an absurd count →
`decode_full` returns (Ok/Err) instead of aborting.

**RS-05 [P2] [V] `probe_metadata(...).unwrap_or_default()` hides import
failures.** `src-tauri/src/files.rs:39`. Unreadable/corrupt files import
"successfully" with all-`None` metadata; the user discovers the problem later
in analysis (which deliberately skips failures with an stderr-only log,
`engine.rs:129-131`) with worse context. Pairs with XP-01 and UX-04.
*Mechanical test:* import garbage bytes named `.wav` → explicit per-entry
error (or an explicit `probe_error` field the UI must render).

**RS-06 [P2] [A] `prepare_waveform` `target_pixels` has no upper clamp.**
`src-tauri/src/audio.rs:47-49` (`.max(64)`, no `.min`); `decode.rs:160-161`
allocates from it. `u32::MAX` → ~34 GB reservation → abort. One-line clamp.

**RS-07 [P2] [A] Unbounded full-track PCM memory, multiplied by caches.**
`decode_full` loads whole tracks as f32; prewarm cache + audio-thread
`decoded_cache` clone + playing source copy can hold ~3 copies
(`audio.rs:402, 765-773, 1857-1861`); album render stages the whole album in
RAM (`album_render.rs:241`). A 2 h 96 kHz source ≈ 5.5 GB/copy. Intersects the
open "long-source sweep" gate.
*Cheap first step:* drop the prewarm entry once copied into `decoded_cache`
(halves peak), pinned by a cache-struct unit test. Duration-cap warning and/or
streaming render are the long-term options.

**RS-08 [P3] [A] Spectrum analyzer keeps the old sample rate across
`play_master` switches.** `audio.rs:1853` only `reset()`s; `handle_play`
rebuilds (`:1714`). EQ-panel spectrum maps wrong frequencies (~2×) after a
44.1k→96k mastered switch. Metering only.

**RS-09 [P3] [A] Limiter lookahead never flushed: renders shift ~3 ms and
truncate the final ~3 ms.** `dsp.rs:1390-1476`; no end-of-stream flush in
render paths. Baked into byte-pinned snapshots — fixing changes bytes, so this
is an explicit owner decision, not a quiet fix. Quarantined in the roadmap.

**RS-10 [P3] [A] Robustness batch:** mutex-poison `expect`s inside commands
(`audio.rs:480,493,512,526,637` — map to `CommandError` instead);
`settings_landing_hash` collapses to one hash on serialization failure
(`audio.rs:924` — treat as cache-miss); fixed `session.json.tmp` name allows
cross-save interleaving (`project.rs:80-85` — unique suffix); heavy sync DSP
inside `async fn` commands ties up the tokio pool (move to `spawn_blocking`);
small heap allocs on the audio callback when coefficients change
(`sources.rs:356-365` — preallocated second chain swap); 15 s play-reply
timeout errors while decode continues and later starts audio (`audio.rs:535-589`
— cancellation epoch before `new_sink.play()`).

### 2.2 Desktop frontend (FE)

**FE-01 [P1] [V] Paused Original/Mastered switch loses the playhead on
resume.** `src/hooks/useTrackMaster.ts:1517-1530`. `setPlaybackKind` while
paused intentionally defers the backend reload, so `loadedKindByTrack` holds
the old kind; the next Play hits `!loadedCorrectKind` and calls
`playWithKind(kind, 0)` — paused at 2:40, flip kind, press Play → 0:00.
Violates "Original/Mastered switching must preserve playhead" (the playing
path is correct; only paused is broken). One-expression fix: start from
`transport.currentTimeSec` unless `isAtEnd || !loadedCorrectTrack`.
*Test:* integration — tick `{position_sec: 42, is_playing: false}`, switch
kind, `togglePlay()` → `playMaster` called with ≈42.

**FE-02 [P1] [V] Removing the currently-playing track does not stop
playback.** `useTrackMaster.ts:1120-1124`, deps at `:1130` omit
`loadedTrackId` — the guard compares against a stale `null`. Clicking × on the
playing track leaves audio playing with no row, no transport, no way to stop
except selecting another track. Also leaks `regionByTrack`/`overrideAlbum`
entries.
*Test:* integration — tick `is_loaded:true` for t1, `removeTrack(t1)` →
`api.stopPlayback` called.

**FE-03 [P1] [V] Album export renders with a stale override set — silent
wrong audio in an export path.** `useTrackMaster.ts:1296-1298`; deps at
`:1313-1323` omit `overrideAlbum`. Toggling a track back to "follow album
intent" doesn't refresh the export closure: the track still renders with its
old per-track settings while the UI says it follows the album. (Override-ON
works only by accident — it also writes `settingsMap`.)
*Test:* integration — override t2, toggle OFF, `exportAlbumPlan()` → the
`renderAlbumPlan` input for t2 carries `albumIntent`.

**FE-04 [P1] [V] Advanced quality rail shows another track's export checks
and gates the wrong export.** `src/App.tsx:155` passes
`tm.lastExportReceipt?.checks` with no track scoping (receipt is set at
`useTrackMaster.ts:1445`, cleared only explicitly at `:1824`, never on
`selectTrack`); `App.tsx:286-298` computes `needsReview` from it. After
exporting clean track A, selecting hot track B shows A's rows, suppresses B's
required source preflight ("already-hot source" rows per APP_BEHAVIOR.md),
and labels the button `Export Master` instead of `Export With Review`.
Standard was fixed for exactly this in commit 7bcc021; Advanced was not.
*Fix:* scope like Standard — `receipt.trackId === selectedTrackId`.
*Test:* export clean A, select hot B → rail title "SOURCE CHECK", button
"Export With Review".

**FE-05 [P3] [A] Spacebar globally hijacks Space on focused buttons.**
`useTrackMaster.ts:1558-1578` (deliberate, commented). A keyboard user who
Tabs to "Create Master" and presses Space gets play/pause. Decide + pin.

**FE-06 [P3] [A] Hygiene batch:** keydown listeners re-attach every playback
tick (`togglePlay` deps include `currentTimeSec` — route through a ref);
`ExportReceiptCard` album branch unreachable in production
(`ExportReceiptCard.tsx:20,24,70-80` — wire it or delete it); Intensity knobs
render `aria-label=""` (`StandardView.tsx:587-599`, `Knob.tsx:341`); waveform
`role="slider"` without keyboard operability; dialogs lack a focus trap; stale
"V1 omits FFT fill" comment in `VisualEqPanel.tsx:22-24` above the implemented
FFT fill.

### 2.3 Cross-cutting contracts (XP)

**XP-01 [P1] [V] Import-capability contract is fiction on desktop (Codex
F-01).** Desktop advertises and accepts `aiff` + `opus`
(`src/hooks/useTrackMaster.ts:207-216`, file dialog `:1051`, copy in
`EmptyState.tsx:30` and `App.tsx:193`) but the unified decode feature set
(`src-tauri/Cargo.toml:31-40` + rodio's `symphonia-all`) supports neither;
symphonia 0.5 has no Opus decoder at all. The iPhone bridge's honest list
(`apps/iphone-native/rust/src/lib.rs:20`) explicitly excludes them; the
real-fixture scanner still includes them (`src-tauri/tests/contracts.rs:1957`).
Users get a late decode failure for formats the picker offered.
*Fix shape:* one supported-format contract: drop aiff/opus from the desktop
list/copy/fixture scanner (recommended — adding real decoders is new scope),
derive UI copy and dialog filters from one exported list, and add a contract
test that decodes one fixture per advertised extension through the backend.

**XP-02 [P2] [V] Export-defaults copy contradicts Standard (Codex F-02).**
`src/lib/chrome-content.ts:18-23` presents "Rendered format: 48 kHz, 24-bit
WAV" as the app's export default; Standard — the default view, reachable from
the same Settings dialog — always renders 44.1 kHz/24-bit
(`src/lib/standard-export.ts:21-24`). `App.chrome.test.tsx` pins the wrong
string. Split copy by mode ("Standard: 44.1 kHz… · Advanced: per delivery
profile") and derive from the same helpers export uses.

**XP-03 [P1] [V] Identity-less async events + unowned background analysis (the
F-03 cluster, enlarged).** Five connected defects, one root cause — events and
state with no request/track identity:
1. `AnalysisProgress` carries only `{fraction, label}` (`engine.rs:35-38`,
   `api.ts:265-268`); the handler paints unconditionally
   (`useTrackMaster.ts:482-486`). `RenderProgress` carries `track_id`
   (`engine.rs:23-27`) — the template already exists; the analysis event never
   got it (and the tick handler drops the id it does receive).
2. Two overlapping imports share one boolean `isAnalyzing`; the first
   `finally` hides the second's still-running analysis (`:953-975`).
3. Session restore (`:613`) and project open (`:2021`) run `analyzeTracks`
   without setting `isAnalyzing` — multi-second invisible work; "Not analyzed"
   /"Ready" shown while analysis runs (UX-04 overlaps); their late events
   populate `realAnalysisProgress`, which the clearing effect (`:541-547`)
   never clears (only true→false transitions), so the next import briefly
   shows a stale ~100% bar.
4. Session restore REPLACES `analysisMap` (`:613-620` builds a fresh map) —
   a track imported during the restore window loses its analysis; Create
   Master disables with "Not analyzed".
5. `landing:status` is a bare boolean (`api.ts:282-286`) — same class, low
   exposure.
*Fix shape:* add `batch_id` (and reuse `track_id`) to backend analysis events;
generation-guard the listeners; count in-flight analyses instead of a boolean;
restore/open set the same visible analyzing state; restore merges instead of
replaces; ticks filtered by `track_id`.
*Tests:* the four integration tests specified per sub-item in the roadmap
(S3.2).

**XP-04 [P2] [V] Standard export recipe + vocabulary duplicated across four
ecosystems without a cross-pin (Codex F-04, enlarged).** The fixed recipe
(44.1 kHz / 24-bit / −1 dBTP / custom profile / LUFS clamp −24..−6) is
hand-mirrored in `src/lib/standard-export.ts:14-27` and
`apps/iphone-native/rust/src/lib.rs:191-254` (Android consumes the facade),
each pinned only by its own local test. Swift additionally hardcodes the
style ids and −14/−11/−9 loudness trio (`ContentView.swift:38-61`) with tests
that re-hardcode the same numbers rather than reading
`src/standard-mapping-parity.json` (which covers styles+loudness only, not
the recipe); an unknown style id silently falls back to Universal. A deliberate
desktop recipe change passes desktop-green and silently strands both phones.
*Fix shape:* add a `delivery` block to the parity fixture; assert it from TS,
iPhone-Rust, Android-Rust, and a Swift test that bundles the fixture.

**XP-05 [P2] [A] Hand-written C header is an unpinned FFI contract.**
`apps/iphone-native/rust/include/yes_master_native_bridge.h` is maintained by
hand; an argument-order change still links (symbols unchanged) and becomes
silent UB on device. Generate with cbindgen in the lane and
`git diff --exit-code`, or add a header-vs-extern parity test.

**XP-06 [P2] [V] Four duplicate duration formatters round differently (Codex
F-09 + 1).** `App.tsx:1047-1051` (round), `App.tsx:1534-1537` (floor),
`StandardView.tsx:34-38` (floor, ""-for-null), `AlbumPanel.tsx:35-39` (round).
A 59.6 s track reads 1:00 and 0:59 in adjacent surfaces. Extract
`src/lib/time-format.ts` with explicit policies; unit-test boundaries.

**XP-07 [P2] [V] Dead CSS selector blocks (Codex F-05).** All 14 spot-checked
classes (`mode-pill`, `mode-toggle`, `add-btn`, `io-gain`, `transport-left/right`,
`slider-row/label/input`, `advanced-toggle`, `quality-icon/grade/blurb`,
`workspace-section-label`) have zero production TS/TSX references. Codex's
totals (470 selectors / 71 unused) are from a stale tree — regenerate the
inventory as part of the fix. Inventory test first, then mechanical deletion.

**XP-08 [P2] [V] Green lane emits `act(...)` warnings (Codex F-08).**
Reproduced on this tree from `src/App.transitions.test.tsx`. Make the
first-frame test own its async probe; require clean output.

**XP-09 [P2] [V] 12.5 MiB preset artwork eagerly imported (Codex F-07).**
`src/components/PresetIcon.tsx:11-18`; seven of eight PNGs are 1.6–1.8 MiB
(universal.png already optimized to 527 KiB — apply the same treatment).
Asset-budget test to pin.

**XP-10 [P2] [V] Deep-analysis per-window mono allocation churn (Codex
F-06).** `src-tauri/src/deep_analysis.rs:358-375` (existing TODO). Hoist one
scratch buffer through `scan_windows`; byte-identity fixture proves output
unchanged. Note: churn (~4,200 × 64 KiB), not peak memory.

### 2.4 iPhone app (IP)

**IP-01 [P1] [V] Mastering FFI has no panic guard — an engine panic aborts
the iOS app.** Zero `catch_unwind` in `apps/iphone-native/rust/src/lib.rs`;
`analyze_file_json` (`:46`), `render_master_json` (`:80`),
`render_master_with_options_json` (`:94`), plus `live_create`/`live_set_params`/
`live_measure_landing` in `live_stream.rs` are bare `extern "C"` (only
`live_process` is guarded; no `panic = "abort"` profile). Since Rust 1.81 a
panic unwinding out of `extern "C"` aborts the process — a corrupt-file panic
kills the app instead of returning the `{"error": ...}` contract. Android
wraps the SAME inner calls in `catch_panic` (`android rust lib.rs:57-63`)
because an adversarial review flagged exactly this class; the iPhone facade
never got the fix.
*Test:* forced-panic injection point + assert error JSON, not a crash.

**IP-02 [P1] [A] Import blocks the main thread on copy + double full decode +
deep analysis.** `AuditionController.swift:100-134` (`@MainActor`) runs the
file copy and `engine.load` synchronously; `live_create` does `decode_full`
PLUS `native_adaptive_context_for_path` (a second full decode + deep analysis,
`live_stream.rs:334-352`). Seconds of frozen UI on a multi-minute track.
Android runs the same create on `Dispatchers.IO`.
*Fix shape:* async load with a Preparing state (mirror Android); share the
decode between adaptive-context and stream create (also halves import time).

**IP-03 [P1] [A] No `PrivacyInfo.xcprivacy` — hard App Store rejection.** The
app uses required-reason file-timestamp APIs (`ImportedTrackStore.swift:79,112`,
`RenderStorage.swift:47-52`); since May 2024 ASC rejects uploads without
declared reasons. Zero runtime impact; pure submission blocker.

**IP-04 [P2] [A] Backgrounding/screen-lock kills audition; no recorded
decision.** No `UIBackgroundModes: audio` in Info.plist — lock the screen
mid-listen and playback stops. For a listening-centric product this reads as
an unmade product decision. Owner decision + one plist line if wanted.

**IP-05 [P3] [A] Polish batch:** progress UI is wall-clock theater (staged
labels, parks at 94% — mark indeterminate or plumb real progress); error
display is substring-sniffing on `statusText` (`ContentView.swift:722-732` —
model an error enum); stale-landing race on fast re-import
(`AuditionController.swift:359-372` — Android's `handle != h` guard is the
template); render-time adaptive-context fallback is silent
(`lib.rs:174-178` `.ok()` — WYSIWYG quietly broken on re-analysis failure);
dead `TrackPlaybackController.swift`/`AudioSessionController.swift`;
`UIFileSharingEnabled` advertises a Documents folder the app never writes;
`docs/IPHONE_APP.md` preset table documents the pre-Standard vocabulary —
an agent following it would "fix" the code backwards.

### 2.5 Android app (AN)

**AN-01 [P1] [V] Use-after-free race: `release()` can destroy the engine
while `measureLanding` runs.** `AuditionController.kt:330-343` cancels
`landingJob` (cooperative) then immediately `destroyNative(h)`; the blocking
native call inside the job (`:278-280`) is not interrupted, and Rust-side
`measure_landing` deliberately runs outside the engine mutex
(`audition.rs:261-282`), so nothing serializes it against Drop. The landing
measurement masters an 8 s window and re-fires 250 ms after every tweak —
tapping "Import a different track"/"Start over" inside that window races →
native UAF. Related leak: a cancelled `attach` lets in-flight `createNative`
complete and the handle (whole decoded track) is never destroyed
(`:152-178`).
*Fix shape:* join/await the landing job before destroy (or RwLock:
measurements read, Drop write); run create non-cancellably and destroy the
result on cancellation.

**AN-02 [P1] [A] AAudio stream format never verified after open — potential
heap overflow.** `aaudio.rs:137-163` requests format/channels/rate but never
queries actuals; AAudio treats these as requests. If the stream opens with
fewer channels than the pump writes, `data_callback` (`:98-110`) writes
`num_frames * pump.channels()` floats into an OS buffer sized
`num_frames * actual_channels` — OOB write. Query actuals post-open; fail
`open()` on mismatch (maps to the existing "Playback could not start").
Related: the close barrier waits on STOPPING-entered, not STOPPED
(`aaudio.rs:179-197`) — poll to STOPPED with a deadline before close.

**AN-03 [P1] [A] Release build is unsigned/unconfigured.**
`app/build.gradle.kts:30-34`. Spec-parked as phase A5 by design — but it is
the gating item for any distribution. Owner decision (keystore) + config.

**AN-04 [P3] [A] Polish batch:** no monochrome adaptive-icon layer (Android 13
themed icons render a blob); Done screen shows a path with no share/play
action (iPhone has ShareLink); `copyToCache` never checks
`supportsImportExtension` before copying hundreds of MB (already item B2 in
`docs/archive/plans/2026-06-10-001-android-a4-action-plan.md`); no
MediaSession/foreground service (backgrounded render is lost — planned B1).

### 2.6 Mobile-shared memory (MB)

**MB-01 [P2] [A] Whole-track decode ×2 at create, no size guard, on devices
that cannot absorb it.** `live_stream.rs:334-352`: full f32 decode for the
stream PLUS a second full decode inside adaptive-context. 60-min 44.1 kHz
stereo ≈ 1.2 GB ×2 transient → guaranteed jetsam/lowmemorykiller death with no
user-facing error. Sessions also re-analyze the same file 3-4 times
(create/analyze/render/landing).
*Fix shape:* share the decode (fixes IP-02's second decode too); import-time
duration cap with a clear message, pinned by a unit test.

### 2.7 Packaging, repo hygiene, go-public (PKG)

**PKG-01 [P0] [A] No LICENSE file.** No LICENSE/COPYING anywhere. A public
repo without one is all-rights-reserved; nobody can legally build or
redistribute. Signing/notarization are deferred by canon — licensing is not.
Owner decision required (proprietary notice vs MIT/Apache-2.0); five minutes
once decided.

**PKG-02 [P1] [A] Version is 0.0.0 in all three desktop manifests.**
`package.json:4`, `src-tauri/tauri.conf.json:4`, `src-tauri/Cargo.toml:3`.
Stamps into the MSI/NSIS filename, Windows Apps list, macOS
`CFBundleShortVersionString`; 0.0.0 MSI `ProductVersion` also breaks future
upgrade sequencing. Mobile disagrees too: Android `0.1.0`, iPhone `1.0`,
bridge crates `0.1.0`. Pick a scheme, set once, pin with a coherence test
across all five files.

**PKG-03 [P1] [A] No CI at all.** No `.github/` anywhere. The documented fast
lane exists only as local muscle memory, is PowerShell-only (Mac contributors
can't run it), and the 37 Swift tests are unreachable from the Windows owner
machine — meaning one of the three assert-sides of every cross-language pin
effectively never runs (weakens XP-04/XP-05 in practice). Minimum lane:
windows-latest (npm test/build + cargo fmt/clippy/test) + macos-latest
(cargo test + `npm run build:mac` + xcodebuild test) + android (gradle test +
cargo test).

**PKG-04 [P1] [A] No evidence the Mac build has ever been produced; Mac DSP
snapshots are asserted-equal-to-Windows, not observed.**
`src-tauri/src/dsp.rs:3427-3473` calls `expected_platform_sha(win, mac)` with
identical strings; the only Mac verification in history predates the June DSP
work. "Mac and Windows" is a stated non-negotiable. One CI macos job (PKG-03)
retires this permanently.

**PKG-05 [P2] [A] Go-public hygiene batch:** `csp: null` in
`tauri.conf.json:23-25` (set a baseline; the frontend renders user-controlled
strings); local usernames leak in tracked docs (scrub to
repo-relative paths; history-scrub not worth it); README is written for
agents/owner, not the public ("before any public-release discussion" in a
public repo is self-contradictory; no screenshot/license/install guidance);
~40 internal handoff/review docs + 2.8 MB `graphify-out/graph.json` ship with
the repo (owner taste call: keep as transparent history or move under
`docs/archive/`); 1440×860 min window excludes 1366×768 laptops (owner
decision: support or document); `.gitignore:7` references the old
`album-mastering-studio.exe` name; TESTING.md and `verify-fast.ps1` omit the
Android lane CLAUDE.md mandates.

**Verified clean [A]:** real reverse-DNS identifiers everywhere; full icon
set with mechanical tests; tight capabilities (core+dialog only, no fs
plugin); updater correctly absent; **zero audio files in the entire git
history**; no secrets; build scripts match docs; canon docs reference real
files; backslash paths test-only; `.git` is a sane 53 MB.

### 2.8 Product / UX gaps (UX)

High-impact holes a first-time public user hits (full detail and effort
ratings live in the roadmap, Wave 6):

**UX-01 [P1] [V-absence] Undo/redo is invisible.** `tm.undo/redo/canUndo/canRedo`
are exported but consumed by no component (grep-verified; only test mocks).
Keyboard-only, undocumented. `docs/RELEASE_STABILIZATION.md` claims "Undo/redo
are compact header tools" — doc/code drift; the buttons were lost in a layout
pass. Mouse users believe edits are irreversible.

**UX-02 [P1] [A] No manual (re)analyze action — a hard dead-end.**
`analyzeTracks` is called only from import/restore/open. If restore-time
analysis fails (file moved/locked), failure is `console.warn`-only; export
disables with "Analyze a track first." and no analyze button exists anywhere.

**UX-03 [P1] [A] Nothing long-running is cancelable.** No cancel command in
`src/lib/api.ts` for analysis, preview render, master render, or album
render. A mis-clicked album export or 60-minute master runs to completion.

**UX-04 [P2] [A] Standard's Create Master shows no progress** — the backend
emits real `render:progress` and Advanced renders it; `StandardView.tsx`
never consumes `tm.renderProgress`. The default view's longest operation is a
static "Creating…" label. (Data already exists; smallest high-value fix.)

**UX-05 [P2] [A] Missing-source recovery has no relink flow.** Open-project
aggregates failures into one toast without naming tracks or offering
Locate/Remove; broken tracks stay listed and fail on play.

**UX-06 [P2] [A] Batch:** error toasts are raw backend strings with no next
step (and errors suppress pending project feedback); dropping only
unsupported files does nothing silently (`useTrackMaster.ts:1015-1028`);
keyboard shortcuts undiscoverable (no Help section, no menu); Settings dialog
is descriptive text presented as settings (one functional control) and its
export row is wrong for Standard (XP-02); Help ignores Standard — the default
view — entirely (Advanced vocabulary only, no glossary, no troubleshooting);
jargon (LUFS/dBTP/dBFS) unexplained in Standard surfaces; preset names change
across the Standard/Advanced seam (Balanced↔Universal etc.) with no bridging
copy; Standard shows a loop-region hint for a loop that can only be armed in
Advanced; Album tab from Standard silently teleports into the Advanced shell;
no window-close guard during in-flight render/export; no "export finished"
notification when unfocused.

**UX-07 [P2-P3] [A] Missing table-stakes:** Recent projects / Open Recent
(only the single autosave exists); file-type association + open-with/argv +
single-instance handling; native app menu (macOS especially — default menu
reads unfinished, and the menu is the conventional home of Open Recent/Undo/
shortcuts).

**UX-08 [P3] [A] Too-many-features (simplification wins, owner-taste):**
loudness target editable in three places in one Advanced screen (collapse to
two); Broadcast EU/US + Vinyl Premaster profiles target a pro-broadcast
audience the canon doesn't serve (tuck behind "More profiles" or cut); eight
preset tiles vs Standard's curated four (group 4 + "more" — listening note
first per working style); Standard Intensity shows the same number four ways
(drop the percent readout); Advanced bottom status bar duplicates
MasterOutPanel readouts and TrackHeader state; Settings static rows duplicate
Help prose (merge → fixes the fake-settings problem too).

**Strong points the sweeps verified (keep; do not relitigate):** first-run
funnel and hint system; real-progress analysis UI; export trust surfaces
(receipt, review ceremony, friendly check labels, "Source ·" prefix); the
plain-English analysis Insight dropdown; app-wide drag-drop with format
overlay; 1.5 s session autosave + restore prewarm; race-safe settings
dispatcher; Volume Match isolation (FE + all four backend render paths);
Standard recipe/warning-suppression matching canon exactly; navigation
machine with the album-trap regression pinned; bit-exact mobile engine parity
tests; Android JNI panic containment; iOS security-scoped bookmark handling;
both platforms' interruption/route-change handling; honest Android manifest
(zero permissions).

## Part 3 — Shippability verdicts

**Desktop (Windows):** Closest to shippable. The engine, export ceremony, and
contracts are in genuinely good shape — but P0s RS-01/02/03 (silent
corruption + two overwrite-protection violations), the four FE P1 state bugs,
XP-01's advertised-format fiction, and PKG-01/02 stand between here and
public. All are small-to-medium, mechanically testable fixes.

**Desktop (Mac):** Unverified, not unshippable. Zero current evidence of a
Mac build; DSP cross-platform SHAs are aspirational copies (PKG-04). One CI
macOS job converts this from "unknown" to "verified every commit."

**iPhone:** TestFlight-grade, not store-submittable. Engine parity is
excellent; IP-01 (panic abort), IP-02 (main-thread freeze), IP-03 (privacy
manifest — hard ASC rejection), MB-01 (OOM on long tracks), plus listing
assets and a background-audio decision stand between here and the App Store.

**Android:** Sideload-grade, not store-submittable. AN-01 (UAF race) and
AN-02 (OOB-write class) are exactly what Play pre-launch reports surface;
AN-03 (signing) is parked by design; B1 process-death story planned.

**The repo as a public artifact:** Not yet. PKG-01 (license) is structural;
PKG-02/03/05 (version, CI, README/scrub) are an afternoon plus owner
decisions.

Everything above is decomposed into ordered, agent-implementable slices with
mechanical verification in `docs/plans/2026-06-12-shippability-roadmap.md`.

## Part 4 — Canon freshness and the unattended-feature inventory
(added 2026-06-12, second pass: full comb of all handoff/plan/queue docs
cross-verified against current code)

### 4.1 Is PRODUCT.md still a usable lighthouse?

**Verdict: accurate but incomplete.** Nothing in PRODUCT.md is false — the
five sweeps verified every stated contract holds in code (Volume Match
isolation, compressor Off semantics, Standard export ceremony and warning
suppression, export philosophy, deferred list). But four things the repo
actually builds are missing from the canon, which is exactly what makes
"agent builds toward the lighthouse" unreliable:

1. **Mobile is absent.** "Local desktop mastering app" is the opening
   definition; the iPhone and Android apps appear only as one aside
   ("mirrors the iPhone app's fixed export"). No product definition exists
   for mobile: audience, scope (audition + fixed Standard export?), parity
   expectations, what is deliberately absent on phones.
2. **Primary Workflow describes Advanced, but Standard is the default
   view.** Steps 4–6 (right rail, delivery profile, export review) are the
   Advanced path. The Standard flow (Styles, loudness trio, Create Master)
   exists only inside the export-ceremony subsection. The "UI Responsibility
   Split" section likewise describes Advanced only.
3. **The adaptive engine is invisible.** Source-profile resolution,
   adaptive strength, deep analysis, and confidence machinery — the largest
   engine investment since the chain itself — appear nowhere in PRODUCT.md
   or APP_BEHAVIOR.md (one env-var footnote). The compressor canon's "it is
   not track-aware analysis" remains true of the compressor specifically but
   reads as if the whole app is not track-aware, which is now wrong.
4. **Album Master has no product definition** — an audience mention plus
   deferral lines; album intent, arc kinds, and override semantics live only
   in code.

APP_BEHAVIOR.md is in better shape (current through the orb/first-run/real-
progress work) but shares gaps 1 and 3, and `docs/RELEASE_STABILIZATION.md`
carries the already-flagged undo/redo doc/code drift (UX-01). Roadmap slice
S5.4 (canon refresh) covers the rewrite; it is gated on the owner
interrogation session because gaps 1 and 4 need product answers, not prose.

### 4.2 Unattended features (promised in handoffs/plans, never shipped,
no recorded decision)

Method: every HANDOFF_*, plans/*, superpowers plans+specs, work-queue,
overnight-jobs, playbook, and mobile spec doc was read; each forward-looking
promise was verified against current code. Items that shipped under a
different name (Simple Mode → Standard View, orb/first-run plans, the
refactor backlog, FFI contract tests, App.tsx extraction, etc.) and items
explicitly superseded are recorded in the comb report and NOT listed here —
only genuinely unattended mass. Trust: spot-verified [V] where marked;
remainder [A] from the comb agent's cited evidence.

**UF-A — The adaptive-engine owner-decision queue (2026-06-05 playbook §3 +
ADAPTIVE_DSP_NEXT_STEPS.md) fell off every tracked surface.** The listening
GATES made it into CLAUDE.md; the DECISIONS the listening session was
supposed to produce did not:

| ID | Item | State | Value / Effort |
| --- | --- | --- | --- |
| UF-A1 | `stereo_width` width co-trigger: field computed + carried in `SourceProfile`, never read by the guardrail (`guardrails.rs` reads correlation only; `stereo_width` appears solely as a test default at `:399`) **[V]** | Half-built; "wire or delete" decision never made | Med / S(delete)–M(wire) |
| UF-A2 | Tilt-vs-reference brightness metric — the "principled fix" behind the deadband bump; measurement side was offered as buildable-behind-the-gate | Never built | Med-High / M |
| UF-A3 | Density-cap semantics reshape (F7, `DENSITY_CAP = 0.60` saturation) | Decision never re-asked | Med / M |
| UF-A4 | Total loudness-loss budget across axes (B3) | Not built | Med / M-L |
| UF-A5 | **Phase B confidence-gating calibration**: `CONFIDENCE_GATING: AtomicBool = AtomicBool::new(false)` (`confidence.rs:40`) **[V]** — the entire Tier-2 deep-analysis + confidence investment is deliberately dormant pending an owner A/B that is scheduled nowhere | Machinery shipped, gate-on session untracked | **High** / owner session + S |
| UF-A6 | §7.3 PSR/crest closed-loop transient protection — per-window inputs were retained for it; PSR derived only in a test today | Not built | **High** (already-mastered stress class) / M |
| UF-A7 | §7.4 holistic already-mastered stand-down — owner answered "yes, treat as part of overall detection"; no stand-down logic exists | Not built | **High** / M |
| UF-A8 | Measured neutral from owner reference masters (per-preset) | Not built | Med / M |
| UF-A9 | §7.5 31-band rollup unification + album label-stability fixture | Contingent (only if chain moves to 31-band reads) | Low-Med / M-L |
| UF-A10 | Phase C `AnalysisSummary` — curated plain-English "what we found" wire payload (the non-confidence user-facing payoff; staged-progress half shipped, summary did not) | Not built | Med / M |

**UF-B — iPhone live-audition hardening list (HANDOFF_2026-06-01
"Remaining work", items 1–5; only the double-decode made it into the
roadmap as S4.4):**
- UF-B1: on-device Instruments RT profiling pass ("the real spike gate") —
  never run. Med / S (owner Mac session; pairs with S0.1's mac lane).
- UF-B2: denormal flush-to-zero (ARM FPCR FZ) on the iOS audio thread —
  absent; IIR subnormal CPU spikes on quiet tails risk dropouts. Med / S.
- UF-B3: preallocated coeff double-buffer — `live_stream.rs:181` still
  clones Vecs in the render-callback drain (accepted "for the spike" only).
  Low-Med / S-M. (Same class as desktop RS-10's callback-alloc item.)
- UF-B4: live-vs-export loudness rate reconciliation — live landing measures
  at source rate, export lands after resampling to 44.1 kHz; non-44.1k
  sources can diverge beyond the stated window error, softening WYSIWYG.
  Low-Med / S-M.
- UF-B5: intensity-perceptibility product decision (landing re-levels away
  the volume cue) — taste decision, needs a listening note. Low / decision.

**UF-C — The Android A4 action plan
(`docs/archive/plans/2026-06-10-001-android-a4-action-plan.md`) was queued as
"next session" in ANDROID_NATIVE_SPEC.md and never executed** (zero
android-native commits after A3). These are app-correctness items, distinct
from Wave 8's store-readiness:
- UF-C1: process-death restore (SavedStateHandle) — session lost on every
  background kill. Med / M.
- UF-C2: import fail-fast — `supportsImportExtension` is exported from Rust
  and declared in `NativeBridge.kt:30` but has **zero Kotlin callers** —
  a dead JNI seam **[V]**; plus retry-on-deterministic-failure and
  `LinkageError` mapping. Med / S-M.
- UF-C3: import-cache reaping — imports accumulate timestamped copies
  forever on user phones. Med / S.
- UF-C4: bridge/build tidiness riders (JObject retype, ABI single-source,
  16 KB alignment tripwire, preset-arg asymmetry). Low / S.

**UF-D — Album delivery fast-follows** (sample-rate parity plan, "flagged
for an explicit now-vs-later call" that was never recorded): upsample
advisory (honesty principle: upsampling ≠ restored detail) and album
requested-vs-rendered integrity check — `AlbumRenderReport` carries no
rendered-vs-source rates for either. Low-Med / S-M each.

**UF-E — Small:** LRA `Option<f32>` cleanup remainder (sentinel `f32` +
`>0.5` guard, tagged "(optional)" in two queues — weakly recorded);
two unmerged `origin/vera/*` UI exploration branches with no keep/discard
disposition (likely superseded — one-sentence call, then delete);
audio-seeded orb particle motion (recorded "not v1" — borderline attended).

**Code TODO inventory:** production code is clean — three TODO hits total,
all either tracked (deep-analysis alloc → S7.5) or deliberate (the
AdaptiveReadout debug gate, designed to survive release).

### 4.3 Direction question raised by the owner (2026-06-12): should the
compressor be track-aware?

Recommendation recorded for the roadmap: **yes as destination, not in the
pre-public window.** The inputs already exist (deep-analysis per-window
crest/dynamics, source profile + confidence); the compressor is the one
stage that ignores them, and UF-A5/A6/A7 are its natural prerequisites. But
it is maximally taste-bearing, invalidates the current owner sign-off,
reopens every listening gate and DSP snapshot on desktop AND mobile, and the
canon deliberately renamed "Auto"→"Preset" to stop implying awareness that
didn't exist. Plan: (a) optional pre-launch **suggestion layer** — the
per-band card reads existing analysis and recommends density/mode with
one-click apply through normal user controls; zero render-path change, no
listening gate; (b) post-launch **Adaptive Tier 3: track-aware compression**
specced with the full listening ceremony, sequenced after the UF-A5
calibration session. Both are in the roadmap's Wave 9 / decision queue.
