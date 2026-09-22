# September 22 launch preparation

Dan requested end-to-end preparation for tomorrow, all applicable tests, computer
use and visual polish. Work starts from freshly fetched main `b69277b8` in the
isolated `codex/launch-sept22` worktree. The active sound-research checkout and
its unadopted policies are preserved. Existing September 9 Windows listening
approval remains valid for its recorded scope, not for arbitrary future bytes.

## Current review candidate — September 22

The frozen application/artifact revision is
`e6db1f467838d94aec9b3cfbc4272e1b977a8adb`. Later documentation updates do not
change that identity. [PR 47](https://github.com/DanielKinsner/yes-master/pull/47)
contains the preparation work; main and public downloads remain unchanged.

The Windows and universal Mac jobs in
[Release run 35700965101](https://github.com/DanielKinsner/yes-master/actions/runs/35700965101)
both pass. The complete 14-asset `yes-master-v0.9.2-manual-9` draft is review-only.
Its final audit fails because the source manifests deliberately retain
`releaseReady: false` while recipient permission and final release evidence
remain unresolved. This is not a successful Release workflow or launch GO.
Full [CI 35700809284](https://github.com/DanielKinsner/yes-master/actions/runs/35700809284)
completes successfully at this exact candidate, including Windows/Mac desktop,
all three encoder/relink jobs, Android, iPhone Swift/bridges, browser, snapshots
and security lanes. Final documentation changes pass 59 affected contract tests;
they do not create a different installer or claim CI for unbuilt application bytes.

The actual workflow-built Windows setup is installed and has completed a native
M4A export with independent decode/measurement proof. The universal Mac package
passes both-architecture encoder execution and final bundled-hash checks. A real
Mac install and by-ear judgment remain separate from those automated checks.

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
- Native keyboard testing then reproduced Tab leaving Help for the obscured
  interface. The shared Help/Settings/Shortcuts dialog now contains forward
  and reverse focus, skips disabled/hidden controls, preserves focus during
  content updates, and returns focus to the opener after closing. Five focused
  regressions fail before the repair and pass afterward; real-browser Help and
  Settings traversal is included in the full headless gate.

## Verified local evidence

Ignored logs and reports: `test-output/launch-20260921/` in this worktree.

| Lane | Result |
| --- | --- |
| Frontend | 903 tests pass, including five new native-dialog regressions |
| Production build | TypeScript and Vite pass |
| Rust format / strict all-target Clippy | Pass |
| Desktop + private fixture | 716 tests pass; four opt-in real-fixture tests execute using the existing Doors Open source |
| Track / Album format matrices | 5 + 2 explicit ignored tests pass, using independent FFmpeg 9.0.1 |
| Lossless rejection control | Explicit negative test passes |
| Native muted lifecycle | Original and Mastered lifecycle pass on existing No Ceiling source; seven codec routes pass |
| iPhone bridge | All-target check and 46 tests pass; one ignored test remains distinct |
| Android bridge | 26 host tests and ARM64 API-29 cross-check pass |
| Dependency security | npm high-severity gate and all three RustSec lockfile gates pass |
| Browser app and landing | Final dialog-repair headless gate passes: 40 app checks, including Help/Settings traversal, plus responsive/200% normal/fallback landing checks |
| Android JVM | Exact-head remote lane passes; local Java loopback initialization fails before tests |
| Encoder package script tests | Both corruption/architecture and missing-runtime dependency tests pass |
| Try It Rust | Correct `wasm32-unknown-unknown` target check passes; frontend engine-stamp/runtime tests pass |
| Native callback stress | Three explicit ignored tests pass on Realtek at 48 kHz using device-default buffers; 1,868 callbacks total, zero measured deadline misses, device errors or exhausted samples |
| Research helper contracts | Seven restore tests and eleven selector tests pass; the latter use the existing hash-bound private inputs from the research checkout without changing shipping policy |
| Native output/error paths | Default/explicit Realtek open, pause, seek and resume pass; injected nonfinite conversion failure pauses and surfaces the correct playback error |

The native lifecycle initially used an 18-second excerpt for a harness that
seeks to 30 seconds. That input error and log are retained separately; the full
source passes. A zero-test exact-name invocation is not counted. Local Gradle
retries with IPv4 and selector settings did not resolve the Java socket error;
no OS/network/security configuration was changed.

The 716-test desktop run leaves 35 explicitly ignored diagnostics/opt-in tests;
the applicable encoder matrices, native lifecycle/stress/error probes, negative
control and remote snapshot lanes are executed separately as recorded here.
Unrelated research benchmarks and owner-report writers are not silently counted
as executed. The iPhone timing proxy remains ignored in its normal 46-test run.

The native device inventory probe completes with three of four exposed endpoints
opening. The default Realtek speakers, Realtek digital output and ASUS
noise-canceling endpoint open; the separate ASUS Virtual Speaker returns
`0x8889000A`. Its cause is not established. This is not an all-device pass, and
no driver, endpoint preference or system setting is modified to erase it.

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
- The 52-second continuous M4A also opens and advances to natural completion in
  Windows Media Player Legacy. This establishes an external-player route, not
  a by-ear judgment of the output.
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

## Final local Windows candidate `2bc36054`

The Help-only candidate `3078c98f` is built, installed and checked natively;
its seven-format text and footer identify that exact build. Both updater
signatures verify and tamper controls fail as expected. Installed executable
SHA-256: `d8df2775e7d9e4646c0514b3e1d049d2043da1b9749683586244b9e684f50da7`.
All original session fields match the saved backup, excluding only its save
timestamp; the eight tracks, selections and settings are preserved. WAV and
Volume Match off are restored.

The subsequent keyboard repair is built and installed at
`2bc36054a784c1be68c2aefe649806fd76e65bf8`. The native Help footer identifies
that build. Forward/reverse Tab remains in Help and Settings, Escape closes,
and focus returns to each opener. Help lists all seven export formats. The
full original eight-track session again matches the saved backup, excluding
only the save timestamp. Engine/export paths are unchanged by the Help and
keyboard commits; the earlier audio pass remains scoped to those paths.

Final local delivery is `test-output/launch-20260921/delivery-2bc36054/`:

| Artifact | SHA-256 |
| --- | --- |
| NSIS setup EXE | `e70f0b616568f40099242234dcaea5dd1d00bf09618d0747859d0239e88fd58b` |
| MSI | `a3f35e19c16ca94ce00d11098cdb7362600921bfa39f5a58b9b66412ba4254ea` |
| Installed EXE | `6366c17c51783253b0d39715c8b0447087f51d2c8600c6c744cf18f0f75bf565` |

NSIS installation returns zero. Both updater signatures verify and one-byte
tamper controls fail. Administrative MSI extraction returns zero; its application
matches the NSIS application except for Tauri's expected three-byte bundle
marker. Both contain the identical qualified encoder and license notices. MSI
payload inspection is not a separate MSI installation. These are local build
artifacts; later CI-built installers require their own byte/installation proof.

The owner review archive `YES-Master-2bc36054-Windows-review-r2.zip` is
74,670,419 bytes, SHA-256
`79762bb3fdf087a072d50ce960ae82c4ffbbccb7284164c518a1b3f7bfdf2c13`.
Its 20 payload files include installers, signatures and corresponding source;
every inventory hash is verified after ZIP extraction/readback. The README
states the review-only status and remaining gates. No private audio, rendered
masters or keys are included. The tracked source includes its historical
native-dialog smoke project fixture, not the owner's working session.

The three muted callback stress tests use the production native device-default
buffer policy. The device grants 480–1,056 frames, not a claimed fixed 256.
Streaming/gain cases exercise 44.1 kHz file conversion into the 48 kHz chain;
the gain case applies 1,200 edits at 5 ms intervals. Preview work cancels and
joins in under 3 ms in all three runs. This is a bounded local measurement,
not a universal latency, listening or installed-interface guarantee.

## Website and repository follow-through

The [website follow-up](2026-09-21-launch-website.md) corrects canonical/share
URLs to the verified production origin and patches development-only Vitest.
Its full 903-test frontend suite and production build pass; npm audit reports
zero advisories. Production package versions and integrity hashes are unchanged.

Repository Dependabot alerts/security updates, secret scanning with push
protection, and private vulnerability reporting are enabled and read back
successfully on September 21. No automatic merging is enabled. Existing owner
launch instructions now point away from beta.1 and the obsolete October 31 date.
The resulting four Dependabot PRs remain unmerged. Their four CI runs were
canceled to free qualification runners; they are not counted as passing tests.
The existing Vercel integration also automatically created four READY previews
for those bot branches. Final domain readback confirms production stayed on
`ab42065495b8c998b696b7ae15e3792fc4d4eb7d`, deployment
`dpl_4gsheBu1MwmX6FJXuzzL6uHRHQix`; neither preparation branch deployed.

The newly populated GitHub dependency graph reports 11 moderate alerts on
unchanged `main`: two Vitest entries fixed by the website/tooling follow-up,
five `glib` entries (the existing Linux-only Tauri/GTK exception), and four
`serde_with` entries. Two copies of each Rust finding are in historical audit
harness lockfiles; those evidence snapshots are not rewritten. The
[`serde_with` advisory](https://github.com/jonasbb/serde_with/security/advisories/GHSA-7gcf-g7xr-8hxj)
requires use of `KeyValueMap` on attacker-controlled data. The candidate has
no `KeyValueMap` use; its only direct dependent is `tauri-utils` 2.9.1, whose
source uses only the `skip_serializing_none` macro. Dependency-tree/source
readback is retained. This is an applicability assessment, not an upgrade or
claim that the GitHub alerts are closed. RustSec gates retain their existing
single explicit Linux exception; no new ignore or lowered threshold is added.

## Remote qualification and remaining launch gates

The full earlier CI run `35690437843` at `48810bf8` completes successfully,
including all three encoder/source-rebuild jobs, Windows desktop/bridges,
macOS desktop/iPhone Swift, Android, snapshots, headless and security lanes.
Later candidates retain their own separate status.

CI run `35693446694` is associated with head `2bc36054`; GitHub checks out its
synthetic merge `56e92233`. Both have exactly the same Git tree
`2e54d139a81ae56975e2310004fc137f48308a6c`. Downloaded ARM and Intel Mac
source/relink evidence names `56e92233`, passes all 21 archive hashes for each
platform, and proves a full application rebuild plus real MP3 encode with the
modified library. Do not relabel that
source commit as the head commit. Manual Release run `35693471170` builds
`2bc36054` directly and prepares a draft, never a public release.

All three direct-head Release qualifications now pass, including modified-library
application rebuild and MP3 encode. Downloaded ARM, Intel and Windows source
packages each pass 21 manifest hash checks and name `2bc36054` consistently.
The local and direct-head Mac source ZIPs contain the same 1,425 files: 624
are byte-identical and 801 UTF-8 text files differ only by CRLF versus LF.
ZIP timestamps also differ by host timezone. No missing or other differing
files were found; distinct archive SHA-256 values are retained as such.

The `2bc36054` draft run then exposes a universal-Mac packaging defect:
`lipo -verify_arch arm64 x86_64 OUTPUT` consumes the output path as an
architecture name. Move the input path before the variadic verification option.
Keep both-architecture verification, signing and execution checks intact.
Regular Mac CI now assembles and executes the universal encoder, so this path
is no longer exercised only during Release. Both CI and Release also compare
the final bundled encoder against the same hashes embedded in the Rust build.
A new mutation/missing-binary regression fails against the original checker
and passes after extending it to verify a final bundled path. All three encoder
script tests and all 903 frontend tests pass locally.

This packaging correction requires a replacement candidate and remote build.
The already-tested canonical-domain, Vitest and documentation follow-up is
folded into that candidate. Earlier artifact identities above stay historical;
they are not silently presented as the repaired universal-Mac release.

The replacement `5a9225ff` passes the complete local headless gate (40 app
scenarios plus the landing suite). Inspection then identifies a separate Mac
signing mismatch before declaring its artifacts ready: the qualified thin
encoder has CodeDirectory flags `0x2`, while the pinned Tauri CLI 2.11.1 signs
executable sidecars with hardened runtime (`--options runtime`). Re-signing
therefore changes bytes after Rust has embedded their SHA-256. Both thin and
universal staging now apply that signing mode using the final packaged filename
before recording the hash. Runtime hash enforcement remains exact. The pinned
[Tauri signing implementation](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.1/crates/tauri-macos-sign/src/keychain.rs)
and [Apple identifier derivation](https://github.com/apple-oss-distributions/Security/blob/main/OSX/libsecurity_codesigning/lib/signer.cpp)
explain why both signing options and the filename must match.

A small Mac-only regression compiles real ARM, Intel and universal executables,
proves the old staging/signing sequence changes the hash, then checks that the
corrected sequence survives the sidecar rename and Tauri's signing command.
It runs as an independent CI lane before the expensive package qualifications.
The existing three cross-platform encoder regressions pass locally; the Mac
signing regression is explicitly skipped on Windows and passes on the real Mac
runner at `e6db1f46`. Both thin CI packaging and universal Release packaging now
also pass their actual post-bundle hash checks.

The superseded `5a9225ff` universal package independently demonstrates the defect:
Rust's expected encoder SHA-256 is
`31c903ebc2c08c6f34441f03496f3a3d56bbe8f9287616b085323e0313957e44`,
while the signed bundled encoder is
`e97d3e32b6c9b54a262f6ffff66e7e1e90cd73a7f9ee06bd44eb60536584e357`.
The new gate rejects it. Its downloaded package is retained as a negative
control and its draft is clearly marked superseded. No runtime assertion or
trust check was weakened to get the repaired package through.

## Final downloaded artifacts and installed proof

Both the direct-head Release and PR CI encoder qualifications pass on Windows,
Apple Silicon and Intel Mac. Downloaded direct-head source/relink packages each
verify all 21 manifest hashes and prove a complete application rebuild plus real
MP3 encoding with the modified library. CI checks synthetic merge `82c227c8`,
whose tree is identical to `e6db1f46`:
`a4ce69597606c1934462589a10406dcff306cdbb`. Keep those source commit labels distinct.
The earlier full CI at `2bc36054` also completed successfully.

All 14 final downloaded assets match GitHub's recorded size/SHA-256; the
checksum ledger closes over every other asset. Both corresponding-source
manifests pass all 15 hashes and all three encoder manifests pass all 13 hashes.
Both source ZIPs contain exactly the candidate's 1,429 tracked files. Windows
matches local `git archive` byte-for-byte; Mac has 624 identical files and 805
UTF-8 files differing only by CRLF versus LF. Archive hashes remain distinct.
Both source manifests still say `releaseReady: false`; their generic pending
text is not proof that the separately recorded rebuild tests failed.

| Final artifact | SHA-256 |
| --- | --- |
| Windows NSIS setup | `964715dd354dd8412d629c724c2ec39dea0da2d85f5e7f39360fe948a0d4a695` |
| Windows MSI | `9ab6d6f99499f134cfb63dbfa1099d699e908726b20f08074696e3f0b9aeb56e` |
| Installed Windows EXE | `1b0093a5b0d554eb1d12cb34d78a5b14d4ea1b7bd66045d3aab9a96f9104ecb4` |
| Universal Mac DMG | `d49fe293210f0207bab141f7718d8e58ea60279ef58e9ff8e9ac6c164b34b71e` |
| Universal Mac updater bundle | `a2e036cb4774b24538c0441b6bae0caad42ac6e0fb74e5525e3aa8f8a5a4c5a3` |

All three detached updater signatures verify against the permanent application
public key, and one-byte tamper controls are rejected. They are not paid OS
publisher signatures. `latest.json` names the correct draft tag, repository,
assets and signatures for both platforms.

NSIS installation and administrative MSI extraction both return zero. The
installed app and MSI payload are identical except for Tauri's expected
three-byte `NSS`/`MSI` marker. Both contain encoder SHA-256
`816e8f8c4bc1b48195503dacf038278dacba55594f5fa452c38b09872a968a44`,
matching the qualified Windows package and the hash embedded in the app.
All four encoder license files match their qualified source hashes. This is
an NSIS install and an MSI payload check, not two separate installed passes.

Computer use opens the installed app, restores/analyzes the short test project,
selects Track AAC/M4A at 256 kbps, renders to a new destination and inspects the
receipt. It identifies clean build `e6db1f4 · 2026-09-22 08:36`. Independent
FFmpeg 9.0.1 decoding yields exactly 1,632,000 stereo frames at 48 kHz (34 s),
-11.1 LUFS and -1.8 dBTP; the receipt reports -11.1 LUFS and -1.74 dBTP.
The output remains local and ignored. WAV is restored after the check.
The original eight-track session is reopened through the native project dialog;
its complete saved data matches both the pre-task and pre-install backups,
excluding only the save timestamp. The app remains installed on `e6db1f46`.

The Mac bundle independently contains ARM and Intel slices in both executables;
both main slices name clean `e6db1f4` builds. Final encoder SHA-256
`b3622fcd32e2c1236853c27d334a2788afd9fb996f55ab9193f1d67f1ea29c72`
matches both target build manifests in the post-package runner checks and the
Intel main's embedded constant. The optimized ARM executable does not retain
the value as one contiguous ASCII literal; package validation does not assume
that compiler representation. No Mac installation is inferred from parsing it.

The local review archive `YES-Master-e6db1f46-release-review.zip` is
242,534,778 bytes, SHA-256
`ed6e61c107e3589f7b956d07eec5a0f4c530ed3632ca7526157810dc719c7242`.
All 14 payload hashes are checked again after ZIP readback. It contains release
assets, inventory, verification report and review-only instructions, with no
private audio, rendered masters, live session or signing secrets.

Evidence is retained under `test-output/launch-20260921/release-e6db1f46/`.
The prior installed `2bc36054` support flow also saves a diagnostics report
through native Save As, confirms success and returns focus to Help's opener.
Its correct build log/session sections are verified locally; the private report
is not included in the release archive.

An earlier superseded Windows job built successfully but received GitHub's
`Resource not accessible by integration` response while creating its draft.
The existing maintainer session prepared unpublished drafts, and final Actions
uploads succeeded without changing workflow-token permissions or creating a
public tag. Superseded manual drafts 7 and 8 remain labeled as failure evidence.

The historical Intel Mac exact Album PCM failure in run `34864394286` remains
unexplained. Its retained log ends at the equality assertion; it contains no
sample vectors or mismatch files from which to prove a cause. Current passing
qualification demonstrates non-reproduction, not diagnosis. Exact PCM checks
remain intact and newer failures retain compact mismatch evidence.

A real Mac install/import/export check, final candidate
listening, LGPL relinking permission disposition, updater-key recovery evidence
and the authorized public updater transaction remain separate requirements.
Full screen-reader and physically disconnected operation remain unverified;
automated semantics and local processing checks do not establish those layers.
The beta lasts 56 calendar days from actual publication; if publication occurs
September 22, its closing date is November 17, 2026. Do not activate that date
or downloads until the actual publication is verified.

The preparation branch disables automatic Vercel deployment as main already
does. This candidate has no public installer or production-site publication.
`RELEASE_METADATA` remains null; the blocked beta.1 draft/tag remain untouched.
