# September 6, 2026 — end-to-end launch readiness audit

**Verdict: local mastering workflows passed the checks below; public launch remains NO-GO.**
The remaining release work is concrete: replacement-candidate authorization,
MP3 distribution materials, exact-candidate cross-platform installer/update proof,
and targeted owner listening. This audit does not activate a release or overwrite
historical candidate evidence.

Owner summary: [interactive launch checklist](2026-09-06-launch-readiness.html).

## Scope and provenance

- Starting commit: `07021f1b1f90b00c24a7908e65886a639062e9af`, clean `main`.
- Application fixes: `3c9848c8ded9eb27368bc010bfd322f01f801398`; native title
  correction and paint regression: `e4a8537db5a9da88a736a4927ebadc8e26bd351b`.
- macOS Apple Silicon; real native packaged Tauri app controlled with Computer
  Use. Native baseline build was 0.9.2 / `07021f1` (10:18 local). Final package
  recheck is recorded below. Browser tests use the preview bridge, not real audio.
- Existing owner sources were reused locally. Audio, masters, project snapshots,
  diagnostics and raw logs remain ignored under
  `test-output/launch-audit-2026-09-06/`. No private audio was committed or sent.
- Reviewed the current release gate, owner decisions, listening handoff,
  behavior/testing contracts, UI/state wiring, export/preview implementation and
  existing automated coverage. No preset, DSP, codec quality or gated constants
  were retuned. No public push, tag, release or deployment occurred.

## Findings corrected

| ID | Finding and impact | Correction and proof |
|---|---|---|
| A1 | WAV/MP3 and bitrate selectors looked like unrelated default OS controls. Labels referenced an undefined color token. | Shared dark surface, border, native-select arrow, readable labels and existing keyboard semantics. Standard, Advanced and Album share the component. |
| A2 | The hero title lacked the app's visual identity. | Shared YES Master equalizer mark, calm white title, restrained divider and full-name tooltip. Long names truncate without expanding the layout. Native recheck caught the old gradient text fill becoming invisible; explicitly restored visible text fill and added a paint regression. |
| A3 | At 1360×740, Standard's Peak readout overflowed the Preview card into Delivery. The old primary-button visibility test still passed. | Preserve the meter stack's intrinsic height and compact short-window spacing. Before: readout bottom 466.70 vs card 448.55 (18.15 px overflow). After: 443.55 vs 456.55; primary export bottom 729.60 within 740. Added readout-containment regression to the headless gate. |
| A4 | Native Album cancel → retry showed new progress alongside the previous “EXPORT CANCELLED” receipt. | Clear the old report only after a new folder selection actually starts another export. Cancelling the picker preserves the prior receipt. New integration regression failed before the fix and passed after it. |
| A5 | Completed Album export exposed paths but had no direct route to the saved files; its panel also used fallback colors. | “Show files” uses the existing native reveal command; error recovery retains the saved path. Added success/failure regression. Panel now uses the app's actual surface tokens. |

Three deterministic landing screenshots and their source manifest were refreshed;
the new title component is included in capture provenance. These images contain
synthetic preview content, not the owner's audio sessions.

## Actual native computer-use coverage

| Journey | Observed result | Limit |
|---|---|---|
| Restore/import/analyze | Restored existing track; imported a second existing WAV and received ready analysis. | Two ordinary 48 kHz tracks, not the Windows high-rate/long-duration matrix. |
| Audition and editing | Play; Original/Mastered switch preserved advancing time; Volume Match toggle; Clarity edit while playing; Return to start while playing and paused. | UI/transport evidence, not a judgment of sound or physical dropout. |
| Standard export | Real save dialog; 44.1 kHz / 24-bit WAV; completion and full report opened. | One real delivery profile; other rates/formats covered by automated contracts. |
| Advanced export | Real save dialog; 48 kHz / 320 kbps MP3 with the selected mastering settings. | No converter/neutralization workflow introduced. |
| Receipt keyboard recovery | Initial focus, Shift-Tab/Tab wrapping and Escape close returned to export. | Native Mac receipt; screen-reader speech was not separately audited. |
| Album sequence | Two tracks; Cinematic flow; pointer and keyboard reorder preserved selected track; filenames followed final order. | No new by-ear judgment of album cohesion. |
| Album cancellation/retry | Cancelled mid-job; no final/partial album outputs remained for that attempt; retry completed. | Stale cancellation receipt was reproduced and fixed (A4). |
| Album output | Two numbered MP3 tracks, continuous album MP3 and explained `metadata/manifest.json`. | Independent measurements below. |
| Project save/reopen | Native save/open restored Album title, order, selected track and per-track Clarity override after an Undo. | Test project saved locally; original session backed up. |
| Settings/help/support | Audio device list and Refresh worked; Escape closed Settings; current WAV/MP3 help; diagnostics saved through a real dialog. | No hardware unplug/reconnect or updater installation performed. |
| Damaged input | Deliberately invalid `.wav` rejected with recoverable error; two valid tracks and Album settings remained intact. | A tiny synthetic invalid file; larger hostile inputs covered by Rust tests. |

## Independent delivered-file checks

FFprobe inspected encoding and duration; FFmpeg decoded each delivered file and
measured integrated loudness, range and true peak. Values agree with app receipts
within the external tool's displayed rounding. Prior WAV/MP3 hashes still matched
after subsequent export/project work.

| Delivered file | Encoding / duration | LUFS / LRA / true peak |
|---|---|---|
| Standard Track | PCM 24-bit, stereo 44.1 kHz, 206.72 s | −14.0 / 6.9 LU / −4.7 dBTP |
| Advanced Track | MP3 320 kbps, stereo 48 kHz, 206.72 s | −14.0 / 6.9 LU / −4.4 dBTP |
| Album track 01 | MP3 320 kbps, stereo 48 kHz, 162.00 s | −14.5 / 6.0 LU / −5.6 dBTP |
| Album track 02 | MP3 320 kbps, stereo 48 kHz, 206.72 s | −14.0 / 6.6 LU / −5.1 dBTP |
| Continuous Album | MP3 320 kbps, stereo 48 kHz, 368.72 s | −14.2 / 6.4 LU / −5.1 dBTP |

Evidence: ignored `independent-exports.json`, per-file `*-independent.log`, Album
manifest and `export-hashes-before.txt`. The continuous duration equals the two
source durations combined. These measurements establish delivered-file integrity;
they do not substitute for the owner's VM-off preview/export listening comparison.

## Automated verification

| Check | Result / evidence |
|---|---|
| Frontend suite | **852 passed, 3 intentionally skipped**, 82 files passed / 1 skipped; `frontend-final.log`. Baseline was 850 passed; two meaningful regressions added. |
| Full Rust + existing real fixture | **634 passed, 0 failed, 9 ignored** on this Mac configuration; includes four real-fixture tests. `rust-tests.log`. Ignored tests are not claimed as passed. |
| Rust lint / formatting | Strict all-target Clippy and rustfmt passed. Rust source unchanged by this audit. |
| Production frontend build | Passed in final headless and native packaging lanes. |
| App browser lane | **37 scenario/viewport checks passed**, including keyboard, accessibility, export formats, Album ordering, state/error routes and new Standard meter containment and visible title paint. |
| Landing browser lane | Responsive/accessibility/CTA checks passed; screenshots and capture provenance verified. This tests local code, not current public download availability. |
| Security dependencies | `npm ci` reported 0 vulnerabilities. RustSec configured gate passed; 16 allowed unmaintained-dependency warnings remain follow-up work, not zero findings. |
| Native Mac packaging | Baseline `.app` and aarch64 `.dmg` built; strict codesign verification passed (ad hoc). Final application package recheck below. Not universal-Mac, downloaded-installer or notarization evidence. |

Final headless evidence:
`test-output/headless/2026-09-06T17-49-34-737Z/` (landing + app).
Key commands, using the bundled Node runtime because the system Homebrew Node's
dynamic library was broken:

```sh
npm ci
npm test
npm run capture:landing
npm run verify:headless
npm run build:mac
AMS_RUN_REAL_FIXTURE=1 AMS_REAL_FIXTURE_PATH='<existing owner source>' \
  cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-rc
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo-audit audit --deny unsound --ignore RUSTSEC-2024-0429 --file src-tauri/Cargo.lock
```

FFmpeg required a process-local path to the installed x265 library. Neither tool
workaround changed product code or system libraries. Mobile bridge lanes were not
rerun: this slice changes desktop React UI/state only, with no shared Rust types,
commands, serialization or DSP behavior.

## Optimization review: evidence before changes

Current preview code already shares source PCM, bounds heavy background work and
cancels obsolete preparation. Preserve those protections. No speculative sound
change or broad refactor was justified by this audit.

1. **Measure MP3 work first.** Native observations: WAV export 2.025 s; MP3
   export 14.494 s, with 11.247 s in the combined write/encode/verification stage.
   Album export took about 31.5 s for 6:08 of output. Different delivery settings
   and concurrent compilation/testing mean these are observations, not a fair
   WAV-vs-MP3 benchmark. Next: controlled release runs on the existing ordinary,
   high-rate and long files; split encoding and decoded verification timings;
   retain truthful delivered-file measurement and cancellation.
2. **Investigate duplicate MP3 measurement.** `engine.rs` measures prepared PCM
   before MP3 delivery subsequently measures the decoded file. Profile whether
   the first pass is avoidable, audit every consumer of its values, then require
   delivered LUFS/peak/range parity and fixture tests before accepting a change.
3. **Profile transport rendering.** Frequent transport state updates flow through
   the large mastering hook. Measure React commits while playing/editing before
   isolating meter/transport subscriptions; reject changes that harm controls,
   audition responsiveness or stale-result protection.
4. **Consider a small cache cap.** `PreviewLandingCache.by_hash` stores scalar
   gains for setting variants and clears on source invalidation. It has no explicit
   per-source entry cap. This is a bounded-scope maintenance opportunity, not an
   observed large audio-memory leak. Measure a long edit session before choosing
   a cap and test revisits/source invalidation if changed.

## Remaining work, ordered by launch dependency

| Priority / owner | Next action | Done when |
|---|---|---|
| Release blocker / engineering | Assemble exact-version MP3 license/source/relinking materials and reproducible release instructions, as required by the existing gate. | Candidate artifacts include the notices and tested distribution materials. Local successful encoding alone is insufficient. |
| Release blocker / owner | Decide the recorded beta.1 disposition and authorize a replacement beta.2 from a named reviewed commit. | Decision recorded; old candidate evidence preserved as historical. No tag was created in this audit. |
| Release blocker / engineering | After authorization, produce complete Mac universal + Windows assets, signatures and checksums; watch exact-commit CI finish. | Full draft asset audit and exact-commit checks pass. No prior tag's green run is reused. |
| Release blocker / engineering + owner | Install the exact draft artifacts, prove first-run/export/project recovery and updater success/failure/retry on Mac and Windows. | Evidence cites downloaded artifact hashes, version and build. A locally launched `.app` is not this proof. |
| Candidate investigation / engineering | Reproduce initial window fit on a stable Mac display and after a monitor/scale change. Computer Use initially captured a cropped window; native Zoom restored it. Logs reported a different work area than the capture surface. | Establish whether this is automation display resizing or a product defect; if reproducible outside that condition, fix it and add native resize/startup evidence before clearing installation. No speculative cause is treated as confirmed. |
| Listening / owner with engineering support | Target Width, Loud, VM-off preview vs exported audio, Album consistency, and new high-rate/long-file playback/edit transitions on real hardware. | Narrow unresolved checks have pass/fail notes and any issue has a source, setting and timestamp. Reuse the September 5 normal A/B/contrast/export passes. |
| Publication / owner | Confirm beta end date (October 31 was provisional) and later authorize publication. | Real verified release and valid date satisfy landing availability; owner announces. |
| Optional / owner | Founder-window wording; newsletter provider; custom domain; paid signing. | Set only if wanted for launch. Newsletter can remain disabled; custom domain and paid signing are not required for the approved free-beta path. |
| Follow-up / engineering | Controlled performance profiling and dependency-maintenance plan. | Measured improvement or documented no-change conclusion; no preset retuning. |

The current live release gate remains [beta-go-no-go.md](../plans/beta-go-no-go.md).
Browser coverage does not prove native hardware audio, install/update trust,
long-session resource behavior, or a subjective mastering-quality verdict.

## Final native recheck

- Build `3c9848c`: native MP3 selector retained Clarity; Album cancel → retry
  showed fresh progress with no stale cancellation receipt. Retry completed into
  `Launch_QA (2)` rather than replacing `Launch_QA`. “Show files” opened Finder
  with that exact `album.mp3` selected. This verifies the newly added native route.
- Visual recheck caught the title paint regression while the heading still
  existed in accessibility text. Fixed in `e4a8537`, refreshed captures and reran
  the full frontend/headless lanes; both passed. No failure was waived.
- Final local package rebuilt successfully and strict deep codesign verification
  passed. Help displayed **0.9.2 · build e4a8537+ · 2026-09-06 10:54**. The `+`
  identifies the uncommitted audit documentation present during packaging;
  application code matched `e4a8537`. This remains a local QA build.
- Visually inspected final native Standard and Advanced/Album after native Zoom:
  title is visibly painted, the brand mark fits, Standard's complete meter stack
  stays within Preview, and the dark WAV selector fits the delivery area.
- Original session restored from the pre-audit backup after quitting the test
  app. Saved QA project/exports and diagnostics remain in the ignored evidence
  directory. User source files were not changed.
- HTML report checked through Computer Use at desktop and 390px width; section
  switching and checkbox persistence across reload passed. Test checkbox returned
  to unchecked. Images are embedded; no external dependency is needed. The
  in-app browser blocks `file:` navigation, so direct offline navigation was not
  asserted as tested. The permitted localhost preview is the open deliverable.
