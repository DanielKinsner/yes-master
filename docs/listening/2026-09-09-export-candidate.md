# Windows export candidate — September 9, 2026

**Installed implementation candidate, not release approval.** The previous
installed Windows `07021f1b` owner five-check PASS remains intact. The owner
confirmed Mac checks must wait for a later Mac login; no repeat request is needed.

## Exact identity

- App version: **0.9.2**.
- Source: **`7cd36ab641eb35fa737ded13a06b826d2b4cfdb7`**.
- Embedded stamp: **`7cd36ab6 · 2026-09-09 10:03`**, without a dirty marker.
- Clean detached checkout: `../yes-master-export-7cd36ab6`.
- Fresh Cargo target: `src-tauri/target/export-7cd36ab6`.
- Installed executable: `%LOCALAPPDATA%/YES Master/yes-master.exe`.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| NSIS `YES Master_0.9.2_x64-setup.exe` | 13,944,428 | `4fbf57277023c2f928f4789db7d938f335971c22c211f1d9baf1516c65321d93` |
| MSI `YES Master_0.9.2_x64_en-US.msi` | 16,224,256 | `ffc7ff3ee805223ddaf56730f86a7eb567221fe4226804f51052ecb9320574b2` |
| Installed app executable | 27,827,200 | `9b1f95574172e8a024c74ccbab4e4ca4702c4155b575225cfbb2c85c39b7812b` |
| Installed encoder | 2,695,168 | `a5ddafb94420c1414fff34aee027482a86b611c028abdc5c414fd3de94603eb0` |

Local delivery folder: `test-output/candidate-7cd36ab6/delivery/`. It contains
both installers, their updater signatures, a qualified Windows encoder archive,
the source/redistribution archive, `SHA256SUMS.txt`, and `artifacts.json`.
These ignored artifacts do not travel with git. Private audio is outside this
delivery folder and is not a release asset.

Both updater signatures verify against the committed permanent public key;
one-byte installer mutations are rejected. These are updater signatures, not
Authenticode signing or a real installed updater transaction. NSIS was installed
(exit 0); MSI was built and signed but not separately installed. Installed code
matches the bundle after accounting for Tauri's expected bundle-type marker.
FFmpeg, LAME and project notices were confirmed in the installation.

The earlier `a17e52b4` candidate was withdrawn when its stamp showed `+` after
Tauri's line-ending rewrite. The build-stamp check now compares actual content;
all candidate evidence here refers to the replacement `7cd36ab6` bytes.

## Installed Windows evidence

Actual Windows UI Automation and native Save/folder choosers exercised the
installed application. These runs did not use preview mocks or direct backend
IPC. A remote-debugging launch attempt was rejected by automatic approval review
as "blocked by policy"; that launch did not run. Native UI Automation was the
accepted alternative. No strict disconnected-machine proof is claimed.

- **Track Advanced:** WAV, MP3, FLAC, AAC/M4A, ADTS AAC, Ogg Vorbis and AIFF
  exported a real 18-second source through the native Save dialog. Independent
  FFmpeg/ffprobe confirmed actual codecs/containers, AAC-LC, rate/channels,
  finite non-silence, frame bounds and receipt measurements. FLAC and AIFF PCM
  equal the WAV master exactly. Meter agreement: LUFS within 0.15, true peak
  within 0.2 dB, LRA within 0.5 LU.
- **Standard:** native AAC/M4A export produced the intended 44.1 kHz stereo
  output and completion toast. Independent decode confirmed the full programme.
  Automated UI tests cover the remaining Standard choices.
- **Album:** six existing tracks, 8:13 arrangement, Cinematic flow, mixed WAV/MP3
  sources and configured gaps. All five new formats plus WAV produced six
  numbered tracks and a continuous master. Independent checks passed for all
  **42 files**, including actual formats, manifest order, frame/padding bounds,
  per-track measurements and new-format continuous measurements. FLAC/AIFF
  tracks and continuous PCM equal WAV exactly. Backend tests separately cover
  deliberate reordering, per-track overrides and above-stereo/rate conversion.
- **Cancellation/retry:** cancelled an installed AAC/M4A Album job after staging
  completed and encoding had begun. It reported cancellation, left no audio or
  manifest in that job's destination, and left no encoder process. Retry in the
  same destination completed all six tracks and continuous output.
- **Audition:** playback advanced during cancellation. The first retry trace
  paused at 0:53 without an identified cause; retain this unresolved observation.
  A second complete Album run advanced continuously from the beginning through
  0:52 while rendering/encoding, with no reported error. The later pass does not
  explain or erase the earlier pause. Observed encoder working sets stayed
  bounded (raw samples are retained in the trace; no whole-program PCM buffer
  is accumulated by the new delivered-file meter).
- **High-rate Track:** imported the existing 12-second 192 kHz stereo fixture
  using the native Open dialog; exported AAC-LC at 48 kHz. Independent decoding
  confirmed finite signal and programme frames. A run that established playback
  before opening Save retained playback through completion (0:01 to 0:04).
  The first immediate Play/export probe did not establish playback and earns
  no simultaneous-playback credit.
- **Session:** installation preserved `session.json` byte-for-byte. After the
  temporary UI probes, the original six-track session was restored and the app
  relaunched without diagnostic flags. Session/settings equality was checked
  again, excluding only the save timestamp. The test import is not left in the
  owner's session.

Raw evidence is under `test-output/candidate-7cd36ab6/`: `artifact-identity.json`,
`signature-verification.txt`, `session-restoration.json`, native dialog/receipt
records, `native-exports/independent-native-results.json`,
`independent-native-album-results.json`, `standard-high-rate-independent.json`,
`cancel-retry-audition.json`, and `audition-recheck.json`. Audio remains local and
private. Some receipt snapshots preceded asynchronous build-info arrival; exact
provenance also comes from installed executable hashes and the native engine log.

## Remaining checks

- Mac ARM64/x64 encoder execution, universal assembly, actual installed Apple
  Silicon candidate and VoiceOver. Deferred by the owner's access answer.
- Strict disconnected installed export and applicable external-player/by-ear
  checks; FFmpeg decoding is not owner listening or player compatibility proof.
- Installed keyboard/NVDA workflow evidence for the affected controls/receipts.
  UI Automation semantic access and headless keyboard tests do not replace it.
- Explain/reproduce the first Album retry's unexplained pause before giving the
  entire audition-under-export gate a PASS.
- Owner checks only the changed formats and affected sound/playback, using this
  candidate. Do not reopen the five baseline checks without a relevant finding.
- Exact final-revision remote CI, final Mac/distribution bytes, LGPL permission,
  key backup/recovery, release/date approval and real updater/publication gates.

Later test/documentation commits do not change the identity of these installers.
Any future application or packaging change requires a fresh identified candidate.
