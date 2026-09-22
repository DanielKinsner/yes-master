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
| Browser app | 38 scenario/viewport checks pass before the landing-only wrap correction |
| Browser landing | Wrap-corrected responsive suite passes, including 200% normal/fallback fonts; preceding failure retained |
| Android JVM | Local Java loopback initialization fails before tests; no test pass claimed |

The native lifecycle initially used an 18-second excerpt for a harness that
seeks to 30 seconds. That input error and log are retained separately; the full
source passes. A zero-test exact-name invocation is not counted. Local Gradle
retries with IPv4 and selector settings did not resolve the Java socket error;
no OS/network/security configuration was changed.

Computer use observed the installed Windows interface and ran the local website
Try It flow with the existing 18-second No Ceiling excerpt: load, analysis,
playback, Original/Mastered switching, Volume Match and Tape selection. This is
interaction evidence, not owner listening approval or final installer proof.

## Remaining launch gates

Final candidate packaging/native interaction and exact-commit cross-platform CI
remain to complete. A real Mac install/import/export check, final candidate
listening, LGPL relinking permission disposition, updater-key recovery evidence
and the authorized public updater transaction remain separate requirements.
The beta lasts 56 calendar days from actual publication; if publication occurs
September 22, its closing date is November 17, 2026. Do not activate that date
or downloads until the actual publication is verified.

The preparation branch disables automatic Vercel deployment as main already
does. Engineering CI and draft review do not publish installers or a website.
`RELEASE_METADATA` remains null; the blocked beta.1 draft/tag remain untouched.
