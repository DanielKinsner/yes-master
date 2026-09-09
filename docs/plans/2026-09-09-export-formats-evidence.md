# Export formats implementation evidence

This ledger follows [the implementation plan](2026-09-09-0857-feat-export-formats-beta-launch-plan.md).
The installed Windows `07021f1b` five-check owner PASS remains the baseline;
none of the evidence below substitutes for a new installed package or Mac check.

| Unit | Status | Evidence / remaining work |
| --- | --- | --- |
| U1 | Windows encoder checkpoint; cross-platform qualification open | Pinned source build, package integrity gate, independent codec matrix, app re-import, Tauri staging and qualification workflow implemented. Mac thin/universal execution and installed packaging still open. |
| U2 | Type/receipt checkpoint complete | Explicit encoding validation, delivered identity independent of metering, legacy WAV/MP3 compatibility, updated wire samples. Command dispatch is connected with the adapters in U3/U4. |
| U3–U5 | Pending | Actual Track/Album adapters and UI are not yet implemented. Existing WAV/MP3 remain available. |
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
