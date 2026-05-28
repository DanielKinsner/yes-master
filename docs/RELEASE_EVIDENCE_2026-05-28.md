# YES Master Release Evidence - 2026-05-28

Branch: `codex/yes-master-rc-finish`

Plan: `docs/plans/2026-05-28-001-release-candidate-finish-plan.md`

## Summary

The RC finish pass completed the code, tooling, packaging, and aggregate
slow-lane evidence work for Track Master stabilization. It does not claim final
listening signoff: manual listening on real material still needs to be done
before calling this a release candidate in the human-taste sense.

## Implementation Slices

- `dd3141b chore: format Rust sources`
- `7b4f679 feat: honor track delivery sample rate`
- `9edaa81 feat: finish project chrome surfaces`
- `ee92a27 fix: unify loudness target writes`
- `4e0150a refactor: centralize live chain push predicate`
- `bdeb7d0 fix: surface mastered preview timeouts`
- `e3f0042 refactor: tighten track master chrome`
- `7d86af8 chore: satisfy rust clippy gate`

## Verification

Portable gates:

| Command | Result |
| --- | --- |
| `npm test` | Passed: 16 files, 142 tests. |
| `npm run build` | Passed. |
| `cargo fmt --check` | Passed from `src-tauri`. |
| `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Passed after installing the local Clippy component. |
| `cargo test --lib --target-dir target\codex-rc` | Passed: 202 library tests. |
| `cargo test --target-dir target\codex-rc` | Passed full backend suite, including contracts, delivery profile render, album render, dither, volume-match export, preset distinctness, loudness balance, and signature tests. |
| `npm run build:windows` | Passed; built the app and produced MSI and NSIS bundles under ignored `src-tauri/target/release/bundle/`. |

Packaging checks:

- `src-tauri/target/release/produce_dialog_smoke.exe` was absent after the
  Windows package build.
- Bundle output was left in ignored build directories and was not committed.

## Track Master Delivery Format

Track Master delivery settings now affect the rendered WAV, export receipt, and
export review:

- Named profiles own target LUFS, ceiling, bit depth, and sample rate.
- Custom Source preserves the source sample rate.
- Custom 44.1 kHz, 48 kHz, and 96 kHz requests use the new Track Master sample
  rate conversion path.
- Export checks flag requested/rendered sample-rate mismatch as a technical
  integrity issue.

Album Master delivery-format parity was deliberately not broadened in this pass.
Album Master should be treated as same-rate for now unless a later slice adds
and verifies full parity.

## Long-Track Preview Hardening

The reported 25-minute Mastered preview non-playback was treated as an RC trust
issue, but this run did not complete a manual 25-minute playback reproduction.
The safe implementation path was to remove silent failure: Mastered preview
readiness timeout now returns a clearer recoverable backend error, and the UI
maps it to user-facing guidance instead of leaving the transport looking broken.

Remaining signoff still needs a real long-track listening pass with Mastered,
Preview LUFS, seeking, and Original/Mastered switching.

## UI Evidence

Before/after Track Master screenshots were captured locally under ignored
`test-output/ui-rc/`. Aggregate review at 1920x1080:

- The empty undo/redo/readiness footer strip was removed.
- Undo/redo became compact header tools.
- Analysis and readiness state moved into the track metadata row.
- The working surface gained vertical space while preserving the accepted
  centered Track Master / Album Master header layout.

Screenshots remain local-only and were not committed.

## Private Slow Lanes

Private source audio, rendered masters, fixture ledgers, and fixture-specific
names were kept out of git. Only aggregate conclusions are recorded here.

Already-mastered fixture matrix:

- Full private manifest run did not complete within the available window. It was
  stopped after more than 20 minutes and its partial output is not counted as
  complete evidence.
- A representative subset completed in about 523.8 seconds.
- Rows: 6.
- Warning-code aggregate: `dynamic_range_low:1`.
- Rendered LUFS range: `-14.00..-14.00`.
- Rendered true-peak range: `-7.34..-4.14`.
- Dynamic-range delta range: `-2.85..0.07`.

Private reference tuning:

- Completed in about 737.1 seconds.
- Rows: 4.
- Warning-code aggregate:
  `comp_density_on_compressed_source:4, dynamic_range_low:4`.
- LUFS gap range: `-4.09..-1.96`.
- Dynamic-range gap range: `-1.10..-0.31`.

## Material Deviations

- U1 used a bounded recoverable-timeout fix rather than a playback pipeline
  rewrite because the repo facts showed the silent failure path could be made
  visible with much lower risk.
- Full private matrix completion was replaced by a representative subset plus a
  documented timeout because the full local manifest exceeded the available run
  window.
- Album Master SRC parity was deferred to avoid widening Track Master RC scope.

## Remaining Watch Items

- Manual listening signoff across a normal mix, an already-mastered/compressed
  source, and a long edge-case source.
- Full private fixture matrix in a longer unattended run.
- Preview LUFS behavior under heavy load and long-track seeking.
- Any accepted tiny A/B switching stutter, if it reproduces in manual review.
- Oomph listening notes before any subjective retune.
- Public signing, notarization, autoupdate, and distribution polish.
