# September 22 launch preparation

Dan requested end-to-end preparation for tomorrow, all applicable tests, computer
use and visual polish. Work starts from freshly fetched main `b69277b8` in the
isolated `codex/launch-sept22` worktree. The active sound-research checkout and
its unadopted policies are preserved. Existing September 9 Windows listening
approval remains valid for its recorded scope, not for arbitrary future bytes.

## Repairs and presentation

- Integrate the separately verified Advanced reset-slot alignment from
  `68983921`/`b7eebf49`, with its geometry/click checks and refreshed deterministic
  landing captures. No DSP or preset change.
- Update desktop `rustls` to 0.23.45 and `rustls-webpki` to 0.103.15. This clears
  [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html).
  Neither mobile bridge lockfile includes rustls; all three locks are audited.
- Explicitly request `platform-tools` from Android SDK setup. The action's v3
  default requests the retired `tools` package and fails before project tests.
- Compare Album JSON peak fields at their actual report precision, with exact
  f32 bit equality. The previous untyped-JSON comparison widens to f64, and the
  parser differs by one f64 ULP for values such as -10.692057609558105. PCM
  equality, independent measurements and frame-count assertions are unchanged.
- Correct Standard and Album marketing copy for all seven formats, distinguish
  defaults from fixed restrictions, and remove the old unapproved price figures.
  No replacement price or purchase entitlement is adopted.
- Use release-aware Try It CTA wording, an accurate full-track hint for sources
  shorter than 30 seconds, and a limiter-setting description without a universal
  clean-output promise. Processing and audition behavior are unchanged.
- Allow the Standard delivery shelf to wrap at 200% zoom. Longer accurate labels
  exposed a real horizontal overflow in both normal and fallback fonts; do not
  hide it with overflow clipping.
- Native Help inspection found another old WAV/MP3-only description. Correct
  it for all seven delivered formats and retain the source-protection wording.

## Verified local evidence

Ignored logs and reports: `test-output/launch-20260921/` in this worktree.

| Lane | Result |
| --- | --- |
| Frontend | 898 tests pass after copy and Try It changes |
| Production build | TypeScript and Vite pass |
| Rust format / strict all-target Clippy | Pass |
| Desktop + private fixture | 716 tests pass; four opt-in real-fixture tests execute using the existing Doors Open source |
| Track / Album format matrices | 5 + 2 explicit ignored tests pass, using independent FFmpeg 9.0.1 |
| Lossless rejection control | Explicit negative test passes |
| Native muted lifecycle | Original and Mastered lifecycle pass on existing No Ceiling source; seven codec routes pass |
| iPhone bridge | All-target check and 46 tests pass; one ignored test remains distinct |
| Android bridge | 26 host tests and ARM64 API-29 cross-check pass |
| Dependency security | npm high-severity gate and all three RustSec lockfile gates pass |
| Browser app and landing | Full final headless run passes at `48810bf8`, including 38 app checks and 200% normal/fallback landing fonts |
| Android JVM | Exact-head remote lane passes; local Java loopback initialization fails before tests |
| Encoder package script tests | Both corruption/architecture and missing-runtime dependency tests pass |
| Try It Rust | Correct `wasm32-unknown-unknown` target check passes; frontend engine-stamp/runtime tests pass |

The native lifecycle initially used an 18-second excerpt for a harness that
seeks to 30 seconds. That input error and log are retained separately; the full
source passes. A zero-test exact-name invocation is not counted. Local Gradle
retries with IPv4 and selector settings did not resolve the Java socket error;
no OS/network/security configuration was changed.

Computer use observed the installed Windows interface and ran the local website
Try It flow with the existing 18-second No Ceiling excerpt: load, analysis,
playback, Original/Mastered switching, Volume Match and Tape selection. This is
interaction evidence, not owner listening approval.

## Installed Windows candidate `48810bf8`

MSI and NSIS build successfully. Both detached updater signatures verify against
the app's permanent public key; a one-byte mutation is rejected for each. These
are updater signatures, not OS Authenticode signatures. NSIS installs successfully.
Its executable matches the built candidate byte-for-byte after Tauri's expected
three-byte bundle marker change (`UNK` to `NSS`). Installed SHA-256:
`5eeea8699eb073f4c640ec03fae74d1bd273843536dd01368b41bcb972c9463c`.
The receipt independently displays build `48810bf8`.

Computer use with existing owner-supplied short No Ceiling sources verifies:

- Open a local project, import/analyze another source, Standard and Advanced
  layout, Original/Mastered/Original switching with advancing playhead, Volume
  Match during playback, and natural playback completion.
- Standard WAV export at 44.1 kHz / 24-bit: 1,499,400 frames (34 seconds),
  independent FFmpeg loudness -14.0 LUFS, matching the receipt.
- Advanced AAC/M4A at 48 kHz / 256 kbps: independent decode gives exactly
  1,632,000 frames (34 seconds), -14.1 LUFS and -4.4 dBTP; receipt agrees.
- Album reorder and per-track override, save/reopen preserving order, format,
  override and per-track views. Cancel a live Album render at 93%: app returns
  Ready, says no files written, and directory readback confirms no Album output
  or temporary files. Immediate retry completes.
- Album M4A files independently decode to 34 and 18 seconds; continuous output
  is exactly 2,496,000 frames / 52 seconds, -11.1 LUFS, -1.3 dBTP. Manifest and
  receipt identify the override, rendered format and delivered measurements.
- Help, Settings and shortcut overlays render; Escape dismisses the dialogs,
  and the question-mark shortcut opens the keyboard reference.

Evidence includes `installed-identity.json`, `delivery-hashes.json`,
`signatures-verified.log`, native receipt/UI text, independent probe/measurement
logs and the locally saved project. Source audio and renders remain ignored.
UI Automation indexes were inconsistent for native picker/pop-up controls;
visible screenshot coordinates and keyboard entry established the successful
results. Failed input attempts are not app failures or completed actions.

The web-only crate's exploratory host `cargo test` invocation fails because it
imports desktop command wrappers on a non-WASM target; it is not a supported
host test lane. The actual WASM target check and shipped-browser runtime are
verified separately. No dependency or cfg was weakened to force that command.

## Remaining launch gates

The Help-only correction needs a refreshed package and focused native check;
exact-commit cross-platform CI remains in progress. A real Mac install/import/export check, final candidate
listening, LGPL relinking permission disposition, updater-key recovery evidence
and the authorized public updater transaction remain separate requirements.
The beta lasts 56 calendar days from actual publication; if publication occurs
September 22, its closing date is November 17, 2026. Do not activate that date
or downloads until the actual publication is verified.

The preparation branch disables automatic Vercel deployment as main already
does. Engineering CI and draft review do not publish installers or a website.
`RELEASE_METADATA` remains null; the blocked beta.1 draft/tag remain untouched.
