# Export formats implementation evidence

This ledger follows [the implementation plan](2026-09-09-0857-feat-export-formats-beta-launch-plan.md).
The installed Windows `07021f1b` five-check owner PASS remains the baseline;
new-format Windows package evidence is recorded separately below. Mac remains open.

| Unit | Status | Evidence / remaining work |
| --- | --- | --- |
| U1 | Windows qualified and bundled; Mac open | Pinned source build, package integrity gate, independent codec matrix, app re-import and installed Windows sidecar verified. Mac thin/universal execution remains unavailable. |
| U2 | Type/receipt checkpoint complete | Explicit encoding validation, delivered identity independent of metering, legacy WAV/MP3 compatibility, updated wire samples. Command dispatch is connected with the adapters in U3/U4. |
| U3 | Implementation complete; Windows verified | Track encoding, read-back, precision/rate policy, cancellation and atomic no-clobber persistence; full fixture lane and installed seven-format delivery verified. Mac qualification remains separate. |
| U4 | Windows backend checkpoint complete | All five new formats preserve order/overrides/gaps and single continuous encoding; cancellation and manifest-failure ownership verified. |
| U5 | Complete | Seven shared choices, captured settings, actual receipts, AIFF aliases and docs. Final single frontend run: 859 PASS. Headless: 38 scenario/viewport checks PASS, including Album formats. |
| U6 | Windows candidate checked; remaining gates open | Installed `7cd36ab6`, independent Track/Album files, cancellation/retry and high-rate delivery verified. One unexplained audition pause, disconnected/player/owner/accessibility evidence and Mac remain open. |
| U7 | Concrete Windows candidate prepared; not release-ready | Exact NSIS/MSI, hashes, verified updater signatures, source archives and modified-LAME rebuild proof retained. Mac, exact remote CI, relink permission, key recovery and owner release/date decisions remain open. |
| U8 | Not authorized / not started | No push, tag, publication, deployment or spending. |

## U1 Windows source package

- Binary: `test-output/audio-encoders/windows-qualified/yes-master-encoder-x86_64-pc-windows-msvc.exe`
- SHA-256: `a5ddafb94420c1414fff34aee027482a86b611c028abdc5c414fd3de94603eb0`
- Size: **2,695,168 bytes**; PE x86-64.
- FFmpeg **8.0.3**, GCC/MSYS2 UCRT64 **15.2.0**, libogg **1.3.5**, libvorbis **1.3.7**.
- Runtime imports: `bcrypt.dll`, `KERNEL32.dll`, `SHELL32.dll`, Windows UCRT API sets.
  No development/runtime DLL is bundled or required.
- FFmpeg archive signature verified against the published upstream key. Source
  archive hashes, build configuration, toolchain information, license texts and
  binary hash are retained in the sealed package manifest.
- **40 independent decode cases PASS:** FLAC, AIFF, AAC/M4A, ADTS AAC and Ogg
  Vorbis, 44.1/48 kHz, mono/stereo, 257-frame and two-second-plus-137-frame signals.
  Tests assert codec/container, rate, channels, finite non-silence, exact 24-bit
  lossless PCM and lossy programme/tail correlation. Independent decoder:
  FFmpeg `2023-04-10-git-b18a9c2971` (test-only AutoPod installation).
- The app's Symphonia decoder re-imported the generated format matrix; AIFF
  support uses Symphonia's existing `aiff` feature, with exact PCM comparison.
- Package negative tests reject wrong PE architecture, corrupt PE signature,
  and unbundled Windows/Mac libraries. Sealed verification checks every source,
  license, configuration and executable hash. Qualification runs the encoder
  with developer tools removed from PATH.

The first diagnostic build revealed missing MSYS2 `cmp`; install `diffutils`
and rebuild from scratch resolved it. Only the clean `windows-qualified`
identity above is eligible for subsequent delivery/package work.

Short-file qualification also reproduced truncated Ogg tails and ADTS auto-probe
failure. [The encoder contract](../third-party/audio-encoders.md#programme-and-padding-contract)
records the tested per-packet Ogg page policy and explicit short-ADTS zero padding.
These are required adapter behavior, not a relaxed duration assertion.

Build outputs, sources and synthetic audio remain ignored under `test-output/`.
They do not travel with git; the committed build scripts reproduce the package
and the qualification workflow is prepared but **has not run on remote CI**.
No private audio or `YES_Master_Video_Packet/` content is staged.

## U2 contract checkpoint

WAV and MP3 now report explicit delivered codec/container/rate/precision facts.
Optional fields default when older jobs are read. Receipt identity no longer
falls back to WAV when an explicitly identified job has no measurements;
unavailable measurements retain the existing `measurements_are_rendered=false`
marker. Invalid/contradictory encoding requests are rejected.

- Frontend: **856 tests PASS**, production build PASS.
- Rust: strict all-target Clippy PASS; format/quality validation tests PASS;
  **54 contract + 3 existing MP3 + 1 wire-sample tests PASS**.
- iPhone: all-target check PASS; **46 tests PASS, one existing ignored**.
- Android: **26 host tests PASS**, API-29 arm64 NDK check PASS.
- No rendered controls changed in this checkpoint. New-format UI remains hidden.

## U3 Windows Track checkpoint

The desktop command accepts an explicit encoding request while preserving legacy
WAV/MP3 entry points. It checks the packaged encoder's SHA-256 before rendering,
quantizes new integer formats once through the existing WAV writer, and measures
a fresh decode before collision-safe finalization. Release builds resolve only
the encoder beside the app executable; debug builds may use the identical
hash-bound staged package. No PATH encoder or frontend command execution exists.

Two integration findings changed the initial approach:

- Hound's mono float WAV channel mask is front-left; AAC requires explicit mono
  layout. The adapter supplies the known mono/stereo input layout without mixing.
- Symphonia 0.5 AAC read-back retained priming despite gapless mode. New formats
  therefore use FFmpeg's container-aware decode into job-owned temporary float
  WAV, measured in bounded chunks. This preserves the U1 delay/padding contract;
  no programme samples are trimmed to satisfy tests. App re-import is separately
  verified, and independent FFmpeg checks validate the actual engine files.

The shared writer's `exists` + Unix rename could replace a racing file. It now
uses `TempPath::persist_noclobber` with the existing sibling naming policy.
WAV PCM snapshots remain unchanged. Temporary PCM writing now checks cancellation
between bounded sample groups; encoder/decode children drain bounded stderr and
are killed/reaped on cancellation.

- **15 Track format/precision combinations PASS**, including reference mastered
  PCM equivalence, Volume Match invariance, prior-file collisions and source bytes.
- **108 direct edge cases PASS:** all new lossy quality choices at 44.1/48 kHz,
  mono/stereo, short/silent audio; lossless 16/24-bit through 384 kHz.
- **457 Rust library tests PASS, five ignored**, including concurrent writer and
  started-child cancellation regressions. Strict all-target Clippy PASS.
- Existing export suites: **9 hostile I/O + 1 Volume Match + 3 MP3 tests PASS**.
- **Seven retained engine exports independently decoded/measured PASS** under
  `test-output/export-formats-track-u3/`. External FFmpeg confirms actual codecs,
  rates, channels and receipts (LUFS within 0.15, TP within 0.2 dB, LRA within 0.5 LU).
  FLAC/AIFF decode to exactly the WAV master's PCM. `reports.json` records the
  development build stamp; `independent-results.json` records each delivered hash.
- Frontend typecheck PASS. No native UI or installed-candidate credit is claimed.

Slow private-fixture and affected final bridge/package checks remain integration
gates after the Album/UI changes. Mac execution remains unavailable as recorded.

## U4 Windows Album checkpoint

New formats reuse the existing Album assembly at the resolved rate and precision,
then encode each numbered track and the continuous programme separately. The
continuous programme is encoded once. WAV/MP3 still dispatch through their prior
paths. Requested Auto precision resolves from the first planned track as before;
invalid new-format precision is rejected rather than silently converted.

- New matrix PASS for FLAC, AIFF, M4A, AAC and Vorbis: three tracks in reversed
  order, a per-track override, mixed 48/96 kHz and mono/stereo/quad sources,
  0.25/0.5-second gaps, exact comparison to one encode of the reference assembly,
  measured per-file values, actual manifest paths and precision facts.
- Cancellation after a delivered track removes only that job's outputs and
  preserves the earlier completed album for every new format.
- Forced manifest collision PASS: all owned audio is removed; the foreign
  manifest survives unchanged. The job returns an error, never partial success.
- Existing regressions PASS: **6 Album render, 6 Album hostile, 5 Album rate,
  3 MP3**. Strict all-target Clippy passed the new Album module before the small
  Auto-precision follow-up; the final integration lane rechecks the combined code.
- These are backend tests. Installed Album UI and final private-fixture evidence
  remain separate U5/U6/integration checks.

## U5 UI checkpoint

Standard, Advanced and Album share all seven format choices and applicable
quality controls. Lossy selection preserves underlying lossless settings;
receipts retain delivered identity when measurements are unavailable. The hook
regression changes format while the native-save promise is pending and proves
the original request/extension is retained. AIFF/AIF import aliases now have
small synthetic fixtures derived from the existing synthetic WAV (no private audio).

- Frontend: 858 of 859 passed during capture; the only failure was the asset
  gate observing the in-progress capture. All eight asset tests passed after
  capture completed. Typecheck and production build passed.
- `npm run verify:headless` PASS: landing and **37 app scenario/viewport checks**,
  including all new Track formats/default qualities and receipt labels. Evidence:
  `test-output/headless/2026-09-09T16-45-57-890Z/`. Synthetic browser evidence only.
- Refreshed deterministic captures/manifest; inspected the Standard capture.
  Owner photography and real-session landing images were preserved.
- iPhone check + 46 tests PASS (one ignored); Android host tests and API-29
  arm64 cross-check PASS after the shared changes.
- Full Rust integration exposed the missing advertised AIFF fixtures (added),
  then a Windows long-path regression in the new atomic finalizer. That fix is
  being verified separately before candidate packaging; the full lane is not
  yet marked passed.

### Integration correction: Windows long paths

`tempfile` passes paths directly to Win32, unlike `std`'s long-path conversion.
Canonicalizing only the existing temporary file/target parent supplies verbatim
paths for atomic persistence while preserving the returned user path. The
existing four portability tests now PASS. A new regression passes every added
format into a >280-character destination twice and proves the first file survives.
All four new-format Track tests and both Album tests PASS after the correction;
strict all-target Clippy PASS. The complete fixture lane is rerun at this code.

## Final integration verification

- **859 frontend tests PASS in one final run**, `export-final-frontend-retry.log`.
  The preceding run found two old exact packaging-command assertions; they now
  require the encoder preflight and bundle overlay. Product behavior was not
  changed to satisfy them. Production frontend builds/typecheck PASS.
- **643 Rust tests PASS, 17 ignored, 42 suites** with `AMS_RUN_REAL_FIXTURE=1`
  and the existing private fixture. All four private-fixture tests actually ran
  and passed; existing WAV/MP3 snapshots and thresholds were preserved.
  Log: `test-output/export-final-rust.log`.
- Explicit new-format matrices: **4 Track + 2 Album PASS**, including the
  long-path regression. Log: `export-final-format-matrices.log`. Independent
  package qualification remains the separate 40-case U1 evidence.
- Final formatting and strict all-target Clippy PASS,
  `test-output/export-final-clippy.log`.
- Shared bridges: iPhone check + **46 tests PASS, one ignored**; Android
  **26 host tests PASS** and API-29 arm64 cross-check PASS. Logs:
  `export-final-iphone.log`, `export-final-android.log`.
- Final headless **PASS, 38 scenario/viewport checks plus landing**:
  `test-output/headless/2026-09-09T17-00-32-355Z/`. This run reused the already
  built, unchanged production UI via `--skip-build`. The new Album scenario
  covers all five added formats and receipt identities.
- Retain the failed prior headless run `2026-09-09T16-53-21-467Z`: the Album
  test tried the next export without closing the previous receipt (test fixed).
  The unchanged 1360px transport/order scenario also failed insertion/order/
  keyboard checks. Its cause is unresolved; the later passing run is not a
  causal explanation. A receipt screenshot now waits for settled opacity.

## U6 installed Windows candidate

See [the exact installed record](../listening/2026-09-09-export-candidate.md)
for complete hashes, native actions and precise remaining limits.

- Clean source **`7cd36ab641eb35fa737ded13a06b826d2b4cfdb7`**, version 0.9.2,
  embedded **`7cd36ab6 · 2026-09-09 10:03`**. Fresh target and detached checkout.
- NSIS installed successfully; original session survived installation. After
  testing, the original six-track session/settings were restored and confirmed
  after a normal relaunch. MSI was built/signed, not separately installed.
- Actual installed Track UI exported all seven formats through native Save.
  Independent decode/measurements PASS; FLAC/AIFF exactly equal WAV PCM.
  Standard AAC/M4A and 192 kHz-source to 48 kHz AAC delivery also PASS.
- Actual installed Album UI exported six tracks plus continuous output in WAV
  and all five new formats. All **42 files** independently pass format, order,
  programme-frame/padding and measurement checks; integer lossless PCM equals WAV.
- Installed AAC Album encode cancellation cleaned only job outputs and left no
  encoder process. Retry completed. Playback continued through cancellation;
  the first retry paused at 0:53 without an identified cause. A subsequent full
  export and an established high-rate Track audition/export passed. Preserve the
  unexplained pause as an open finding, not a blanket audition PASS.
- Mac access is explicitly deferred by the owner. Disconnected installed
  operation, external player/by-ear and NVDA/VoiceOver evidence remain distinct.

## U7 preparation and exact artifact boundary

The local review candidate is **7cd36ab6**, not the later test/docs tip. Both
installers, updater signatures, source archive and qualified encoder archive
are in `test-output/candidate-7cd36ab6/delivery/`, with complete checksums/sizes.
No public location has been created and these files are not committed.

Both installer signatures verify against the permanent updater public key;
one-byte tamper checks reject each artifact. The encrypted signing key and
DPAPI-protected password were available, and repository secret names were
checked read-only. Cross-machine backup/recovery remains unproven. No signing
secret or private audio was written to git, logs or the candidate delivery.

The source ZIP is an exact `git archive` of 7cd36ab6. LAME crates match its
Cargo.lock checksums. FFmpeg/libogg/libvorbis source/configuration/licenses
match the qualified package. A fresh build from the extracted archive, with
both LAME crates supplied locally and the bundled LAME version changed only in
the private rebuild tree, produced a complete application containing the new
library marker. **Three existing MP3 tests plus one relink proof PASS**: real
MP3 bytes carry `LAME3.101`, independently distinguishable from shipped 3.100,
and decode to finite stereo audio. See [rebuild instructions](../third-party/rebuild-and-relink.md).
This mechanical Windows proof does not grant recipient license permissions or
replace Mac rebuild proof. The source package remains `releaseReady: false`.

Prepared CI/release workflows consume exact qualification packages, run actual
engine matrices, assemble the Mac universal sidecar, bundle notices, and require
source/encoder archives in the asset audit. **No remote workflow has run for
these commits.** Paid signing is not added as a requirement. Beta.1 history and
artifacts remain untouched; proposed beta.2/version/date and the unresolved
transaction are recorded in [the release checkpoint](2026-09-09-export-release-checkpoint.md).
