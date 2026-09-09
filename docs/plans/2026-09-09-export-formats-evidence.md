# Export formats implementation evidence

This ledger follows [the implementation plan](2026-09-09-0857-feat-export-formats-beta-launch-plan.md).
The installed Windows `07021f1b` five-check owner PASS remains the baseline;
none of the evidence below substitutes for a new installed package or Mac check.

| Unit | Status | Evidence / remaining work |
| --- | --- | --- |
| U1 | Windows encoder checkpoint; cross-platform qualification open | Pinned source build, package integrity gate, independent codec matrix, app re-import, Tauri staging and qualification workflow implemented. Mac thin/universal execution and installed packaging still open. |
| U2 | Type/receipt checkpoint complete | Explicit encoding validation, delivered identity independent of metering, legacy WAV/MP3 compatibility, updated wire samples. Command dispatch is connected with the adapters in U3/U4. |
| U3 | Windows backend checkpoint complete | Track encoding, read-back, precision/rate policy, cancellation and atomic no-clobber persistence verified below. UI exposure follows U4/U5; installed/fixture integration remains separate. |
| U4 | Windows backend checkpoint complete | All five new formats preserve order/overrides/gaps and single continuous encoding; cancellation and manifest-failure ownership verified. |
| U5 | In progress | Shared selectors, receipt facts, import aliases and UI verification. |
| U6 | Pending; Mac access unavailable | Owner confirmed Mac checks must wait for a later login/session. Continue Windows work. |
| U7 | Preparation | Encoder source/license archives retained; LAME relink permission/materials, exact candidate CI, installer hashes and signing evidence still required. |
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
