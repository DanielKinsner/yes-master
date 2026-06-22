> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Codex refactor audit; 3 P1 closed by S3.x contract-hardening slices. _(Status: SUPERSEDED.)_ See docs/CHANGELOG.md for the project ledger.

# YES Master Codex Refactor Audit - 2026-06-13

Report-only audit. No source changes were made.

> **Reconciliation note — 2026-06-15:** All three P1 findings (F-01, F-02, F-03)
> were verified **closed in current `main`** (HEAD `a1e2da7`). This audit was run
> against a tree that predated the S3.x contract-hardening slices, so its P1
> line references describe pre-merge code. Shipped fixes: F-01 → S3.1 (`739b4c1`),
> F-02 → S3.4 (`e9062a3`), F-03 → S3.2 (`e105756` + `0d65d6f`). Per-finding status
> notes are inline below. The P2 findings (F-04…F-09) were not re-verified here.

## Executive summary

Overall health grade: B for shipped behavior, C+ for refactor readiness.

The repo is green across the requested fast lanes, and the recent stabilization work is materially reflected in tests and implementation. The remaining refactor risk is not "needs architecture astronautics"; it is a handful of places where product contracts are duplicated or async state is still global.

Highest-leverage refactors:

1. Make import capability a single tested contract across desktop UI, desktop decode, fixture scanning, and native bridges.
2. Give analysis progress a request or batch identity so stale events cannot paint the current track's state.
3. Add small parity/inventory tests before cleanup: Standard export recipe parity, CSS selector inventory, and shared duration formatting.

No P0 blockers were found. Two P1 issues are direct product-contract drift; one P1 is an async correctness risk around recently added real progress. The P2s are mostly contained cleanup slices with good mechanical tests.

## Verification baseline

Green lanes run before auditing:

| Lane | Result | Notes |
| --- | --- | --- |
| `npm test` | Passed | 42 test files, 387 tests. React `act(...)` warnings emitted in `src/App.transitions.test.tsx`. |
| `npm run build` | Passed | Vite build completed; preset PNG assets dominate static output. |
| `npm run build:windows` | Passed | MSI and NSIS bundles produced under ignored Tauri target output. |
| `cd src-tauri; cargo fmt --check` | Passed | No formatting drift. |
| `cd src-tauri; cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Passed | No warnings. |
| `cd src-tauri; cargo test --lib --target-dir target\codex-rc` | Passed | 304 lib tests. |
| `cd src-tauri; cargo test --target-dir target\codex-rc` | Passed | Full desktop Rust suite passed; ignored dump tests remained ignored. |
| `cd apps/iphone-native/rust; cargo check --all-targets` | Passed | iPhone bridge check lane green. |
| `cd apps/iphone-native/rust; cargo test` | Passed | 37 passed, 1 ignored timing proxy. |
| `cd apps/android-native/rust; cargo test` | Passed | 16 passed. |

Android `cargo ndk -t arm64-v8a --platform 29 check` was not run because this audit made no shared crate, facade, or Android source changes. It remains required for source changes in that area.

## Findings table

| ID | Priority | Area | Title | Effort | Risk |
| --- | --- | --- | --- | --- | --- |
| F-01 | P1 | Import/decode contract | Desktop advertises and accepts AIFF/Opus that the shipped decode stack and native bridges do not support | M | High |
| F-02 | P1 | Export defaults copy | Settings chrome preserves an ambiguous 48 kHz default while Standard export is fixed at 44.1 kHz | S | Medium |
| F-03 | P1 | Analysis progress/state | Analysis progress events are global and generationless, so stale jobs can paint current UI state | M | High |
| F-04 | P2 | Standard/native parity | Standard export recipe is duplicated between TypeScript and the iPhone facade without a shared parity fixture | S | Medium |
| F-05 | P2 | CSS cleanup | `App.css` has provably unused legacy selector blocks after the stabilization UI churn | M | Medium |
| F-06 | P2 | Deep analysis perf | Deep analysis allocates a mono buffer for every scan window | M | Medium |
| F-07 | P2 | Frontend assets | Preset artwork is large and eagerly imported into the main UI path | S | Low |
| F-08 | P2 | Test hygiene | The green frontend lane emits known React `act(...)` warnings | S | Medium |
| F-09 | P2 | Time formatting | Duplicate duration formatters round the same track differently | S | Low |

## P0 findings

None found.

## P1 findings

### F-01 - Desktop advertises and accepts AIFF/Opus that the shipped decode stack and native bridges do not support

> **Status 2026-06-15: CLOSED by S3.1 (`739b4c1`).** The desktop import surface now
> derives from a single `src/lib/supported-formats.ts` contract listing only
> `wav, mp3, m4a, aac, flac, ogg`; no AIFF/Opus remains in the UI list, the dialog
> filter, or the displayed copy. Verified in current `main`: `supported-formats.ts`,
> the `AUDIO_DIALOG_FILTER` import at `useTrackMaster.ts:36`, and the negative
> assertions in `supported-formats.test.ts` / `EmptyState.test.tsx`. The
> `useTrackMaster.ts:207-216` lines cited below are pre-S3.1. (The `files.rs:39`
> probe-swallow sub-point is tracked separately under the S1.5 robustness work.)

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src/hooks/useTrackMaster.ts` | 207 | `const AUDIO_EXTENSIONS = [` |
| `src/hooks/useTrackMaster.ts` | 209 | `"aiff",` |
| `src/hooks/useTrackMaster.ts` | 216 | `"opus",` |
| `src/components/EmptyState.tsx` | 30 | Displayed supported-format copy includes AIFF and Opus. |
| `src-tauri/Cargo.toml` | 31 | `symphonia = { ... features = [` |
| `src-tauri/Cargo.toml` | 33-40 | Features include `mp3`, `aac`, `isomp4`, `flac`, `wav`, `pcm`, `ogg`, `vorbis`; no AIFF or Opus feature is enabled. |
| `apps/iphone-native/rust/src/lib.rs` | 20 | `Supported formats: wav, mp3, m4a, aac, flac, ogg.` |
| `apps/iphone-native/rust/src/lib.rs` | 305-314 | Bridge tests explicitly assert `aiff`, `aif`, and `opus` are unsupported. |
| `src-tauri/tests/contracts.rs` | 1957 | Real-fixture scanner still treats `opus` and `aiff` as supported fixture extensions. |
| `src-tauri/src/files.rs` | 39 | `let metadata = probe_metadata(path).unwrap_or_default();` |

Why it matters:

The desktop file picker and empty-state copy tell users AIFF and Opus are valid import formats, while the active decode stack and native bridge contract say they are not. `probe_metadata(...).unwrap_or_default()` can also let a file through with default metadata after metadata probing fails, moving the failure later into analysis or decode. This is a trust problem for a local mastering app: the import surface should not advertise formats it cannot actually analyze and render.

Concrete proposed fix:

Create one explicit supported-audio-format contract for desktop import, displayed copy, fixture scanning, and native bridges. Either remove AIFF/Opus from desktop and fixture scanning, or add real decode support and fixtures for them. Do not leave this as prose; make the UI list and dialog filters derive from the same source or generated fixture.

Risk:

High. This can produce user-visible failed imports and makes cross-platform behavior drift silently.

Mechanical test:

Add a desktop contract test that compares the UI-supported extension list, file-picker filter list, and backend fixture-supported extension list. Add or update bridge tests to assert the intentional native support set. If AIFF/Opus are kept, add real decode fixtures and a test that each advertised extension can be analyzed through the backend.

### F-02 - Settings chrome preserves an ambiguous 48 kHz default while Standard export is fixed at 44.1 kHz

> **Status 2026-06-15: CLOSED by S3.4 (`e9062a3`).** `src/lib/chrome-content.ts`
> now derives the Standard row from `STANDARD_EXPORT_DELIVERY` via
> `standardExportFormatCopy()` (44.1 kHz / 24-bit WAV / −1 dBTP) and shows a
> separate `Advanced · delivery profile` row for the 48 kHz Streaming Universal
> profile. `App.chrome.test.tsx` asserts both rows from the same helper, so the
> copy can no longer drift from the exported values.

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src/lib/chrome-content.ts` | 18-22 | Settings `Export Defaults` rows include `Delivery profile`, `Streaming Universal`, `Rendered format`, and `48 kHz, 24-bit WAV`. |
| `src/App.chrome.test.tsx` | 65-73 | Test pins `48 kHz, 24-bit WAV` as the Settings copy. |
| `src/lib/standard-export.ts` | 21 | `target_sample_rate: 44_100,` |
| `src/lib/standard-export.ts` | 22 | `bit_depth: 24,` |
| `src/lib/standard-export.ts` | 23 | `ceiling_dbtp: -1,` |

Why it matters:

The current product contract makes Standard the default path and fixes Standard export to 44.1 kHz / 24-bit WAV. The Settings chrome still surfaces a single `48 kHz, 24-bit WAV` default, which appears to describe the default export path even though it is really the Advanced/Streaming Universal profile. That is small text, but it is high-trust text: users look there to understand what will be rendered.

Concrete proposed fix:

Split the copy by mode or derive it from the existing settings helpers. For example: `Standard Create Master: 44.1 kHz, 24-bit WAV, -1 dBTP` and `Advanced Streaming Universal: 48 kHz, 24-bit WAV`. Keep the row labels stable, but remove the single ambiguous default.

Risk:

Medium. The app may export correctly, but the settings panel can mislead users about the rendered format.

Mechanical test:

Update `App.chrome.test.tsx` so it asserts view-specific export defaults from the same helper used by export code, not a hard-coded standalone string. Add a regression assertion for `44.1 kHz` in Standard-facing copy.

### F-03 - Analysis progress events are global and generationless, so stale jobs can paint current UI state

> **Status 2026-06-15: CLOSED by S3.2 (`e105756` + `0d65d6f`).** `AnalysisProgress`
> now carries `batch_id` (`src-tauri/src/types.rs:952`, emitted via
> `engine.rs:analysis_progress_event`); the frontend ignores late events from
> superseded analyses and merges a late session-restore without evicting the
> current track's analysis. Covered by `useTrackMaster.integration.test.tsx`
> (overlapping-imports stale-filter + late-restore merge) and the Rust contract
> test `async_event_identity_payloads_serialize`.

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src-tauri/src/engine.rs` | 23-27 | `RenderProgress` includes `track_id`, `kind`, and `fraction`. |
| `src-tauri/src/engine.rs` | 35-37 | `AnalysisProgress` only includes `fraction` and `label`. |
| `src/lib/api.ts` | 265-268 | `AnalysisProgressEvent` only exposes `fraction` and `label`. |
| `src/hooks/useTrackMaster.ts` | 473-539 | One effect wires landing, analysis, playback, render, and file-drop events. |
| `src/hooks/useTrackMaster.ts` | 482-483 | `onAnalysisProgress` unconditionally sets `realAnalysisProgress` from the incoming event. |
| `src/hooks/useTrackMaster.ts` | 580-632 | Session restore runs `api.analyzeTracks(...)` without owning the same visible `isAnalyzing` state as the import pipeline. |
| `src/hooks/useTrackMaster.ts` | 925-985 | Import analysis uses global `isAnalyzing` and the same global progress event. |
| `src/hooks/useTrackMaster.ts` | 2017-2030 | Project open can also run analysis while the same listener is active. |
| `src/hooks/useTrackMaster.ts` | file measurement | 2,176 lines, 57 `useCallback`s, 11 `useEffect`s. |

Why it matters:

The recent real-analysis-progress UI is only as accurate as the event it trusts. Render progress already carries identity; analysis progress does not. If restore/open/import analyses overlap or complete out of order, an older analysis event can update the currently visible Standard analysis state. This is exactly the kind of async drift that a large hook makes hard to reason about.

Concrete proposed fix:

Add an `analysis_request_id` or `batch_id` to backend `AnalysisProgress` and the TypeScript event type. Extract a small `useTrackAnalysisJobs` controller only around analysis start/progress/settle state, with a current generation ref that ignores stale progress. Use the render progress contract as the template. Avoid a broad `useTrackMaster` split in this slice.

Risk:

High. It is user-facing state correctness, and it can regress without any compile failure.

Mechanical test:

In `useTrackMaster.integration.test.tsx`, mock two in-flight `analyzeTracks` calls, fire progress for the older request after the newer one starts, and assert the visible progress remains tied to the current request. Add a Rust contract test that analysis progress events include the request identity.

## P2 findings

### F-04 - Standard export recipe is duplicated between TypeScript and the iPhone facade without a shared parity fixture

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src/lib/standard-export.ts` | 11-24 | Comment says the function mirrors iPhone, then independently sets loudness, ceiling, bit depth, and sample rate. |
| `src/lib/standard-export.ts` | 21-24 | `target_sample_rate: 44_100`, `bit_depth: 24`, `ceiling_dbtp: -1`, `gain_db: standardLoudnessGain(...)`. |
| `apps/iphone-native/rust/src/lib.rs` | 216-241 | iPhone facade independently sets `lufs_offset_db`, `ceiling_dbtp: Some(-1.0)`, `bit_depth: Some(24)`, and `target_sample_rate: Some(44_100)`. |
| `src/lib/standard-export.test.ts` | 34-42 | TypeScript pins the fixed recipe locally. |
| `apps/iphone-native/rust/src/lib.rs` | 319-325 | Native pins the fixed recipe locally. |
| `src/standard-mapping-parity.json` | file measurement | Existing cross-language fixture covers Standard style/loudness mapping, but not the fixed export recipe. |

Why it matters:

The product intentionally aligns desktop Standard and the iPhone facade. The existing tests prove each side's current local expectation, but not that both sides still share one product contract. A future change can update one local test and forget the other.

Concrete proposed fix:

Extend `src/standard-mapping-parity.json` or add `src/standard-export-parity.json` with target sample rate, bit depth, ceiling, and default loudness/gain assumptions. Make TypeScript and native tests read the same fixture. Add an Android host test through the facade if the Android crate remains dependent on the iPhone rlib.

Risk:

Medium. The current values match, but the guardrail is weak.

Mechanical test:

Cross-language parity tests should fail if any one of `44_100`, `24`, `-1.0`, or the mapped loudness gain changes on only one side.

### F-05 - `App.css` has provably unused legacy selector blocks after the stabilization UI churn

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src/App.css` | file measurement | 6,010 lines; selector inventory found 470 distinct class selectors, with 71 class names not referenced as strings in production `src` TS/TSX files. |
| `src/App.css` | 487-523 | `.mode-pill` / `.mode-toggle` block has no production TS/TSX string hit. |
| `src/App.css` | 570-579 | `.add-btn` block has no production TS/TSX string hit. |
| `src/App.css` | 1020-1031 | `.io-gain` / `.io-gain .slider-row` block has no production TS/TSX string hit. |
| `src/App.css` | 1672-1673 | `.transport-left,.transport-right` selectors have no production TS/TSX string hit. |
| `src/App.css` | 2315-2327 | `.slider-row`, `.slider-label`, `.slider-input` block has no production TS/TSX string hit outside the unused IO gain styling. |
| `src/App.css` | 2443-2451 | `.advanced-toggle` block has no production TS/TSX string hit. |
| `src/App.css` | 3296-3341 | `.quality-icon`, `.quality-grade`, `.quality-blurb` block has no production TS/TSX string hit. |
| `src/App.css` | 3926 | `.workspace-section-label` has no production TS/TSX string hit. |

Why it matters:

This is not a request to split `App.css`; that was explicitly declined in the prior backlog. The material issue is that recent UI replacement left dead selector blocks behind. Dead CSS makes future visual work riskier because obsolete constraints can look intentional, and snapshot/layout tests can accidentally preserve stale surfaces.

Concrete proposed fix:

First add a selector inventory test with a small allowlist for genuinely dynamic classes. Then delete only verified dead blocks in a cleanup PR. Keep the change mechanical and avoid a structural CSS split.

Risk:

Medium. Deleting CSS without an inventory can cause accidental visual regressions, but leaving verified dead blocks increases maintenance risk.

Mechanical test:

Add `src/App.css-selector-inventory.test.ts` or similar that parses class selectors and fails on unmatched selectors unless allowlisted. Run `npm test` and a browser/visual smoke around first-run, Standard, Advanced, and export receipt surfaces before deleting.

### F-06 - Deep analysis allocates a mono buffer for every scan window

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src-tauri/src/deep_analysis.rs` | 11-21 | `SHORT_WINDOW = 16_384` and `MAX_SCAN_WINDOWS = 4_200`. |
| `src-tauri/src/deep_analysis.rs` | 254-296 | `scan_windows` loops over capped windows. |
| `src-tauri/src/deep_analysis.rs` | 358-375 | TODO notes a mono copy per window; `let mono_slice: Vec<f32> = ...collect();`. |
| `src-tauri/src/deep_analysis.rs` | calculated measurement | Worst-case short-lived mono allocation is `16_384 * 4_200 * 4 = 275,251,200` bytes, about 262.5 MiB before FFT scratch. |

Why it matters:

This is not in the real-time audition loop, so it is not a release blocker by itself. It does run during analysis and bridge analysis, though, and the allocation churn is avoidable. The file already marks the issue with a TODO; the measurement makes it worth scheduling.

Concrete proposed fix:

Hoist one reusable mono scratch buffer inside `scan_windows` and pass it into `measure_window`, or add borrowed helper variants for the two consumers that need mono data. Preserve output bytes; do not retune DSP or thresholds in this slice.

Risk:

Medium. Performance-oriented DSP refactors can accidentally alter analysis results if the scratch lifecycle is wrong.

Mechanical test:

Use existing deep-analysis tests plus a fixed byte-identity fixture that compares pre/post metrics for a deterministic stereo sample. If acceptable, add a tiny test-only allocation counter or benchmark to prove per-window allocation count drops.

### F-07 - Preset artwork is large and eagerly imported into the main UI path

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src/components/PresetIcon.tsx` | 11-18 | Static imports load all eight preset PNG files. |
| `src/assets/presets` | directory measurement | 8 PNG files total 12,539,183 bytes, about 11.96 MiB. |
| `npm run build` output | measurement | Seven preset images are about 1.60-1.78 MiB each; `universal` is about 527 KiB. |

Why it matters:

This is a local desktop app, so the cost is not the same as a public web landing page. Still, these are icon/tile assets in a first-run and preset-selection path. The current size adds package weight and image decode work without improving mastering behavior.

Concrete proposed fix:

Replace the preset PNGs with appropriately sized optimized assets, such as 256/512 pixel variants. Prefer no new runtime dependency. If WebP/AVIF support is accepted for the target webviews, keep a fallback policy explicit.

Risk:

Low. The behavior risk is visual fidelity, not audio correctness.

Mechanical test:

Add an asset budget test that asserts each preset asset and total preset artwork bytes stay below agreed thresholds. Pair with a visual smoke of the preset strip.

### F-08 - The green frontend lane emits known React `act(...)` warnings

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `npm test` output | measurement | Passed, but emitted `An update to App inside a test was not wrapped in act(...)`. |
| `src/App.transitions.test.tsx` | 341-372 | The first committed frame test renders inside `act(...)`, then waits on async probing that still schedules state updates. |

Why it matters:

The repo treats the fast lane as a release gate. A green test suite with expected warning noise trains reviewers to ignore output, and future real `act(...)` warnings can hide in the same noise.

Concrete proposed fix:

Make the first-frame test fully control the async probe. For example, keep `loadRecentSession` unresolved while asserting the initial frame, then resolve it inside `await act(...)`. The test should still prove the first committed frame, but without leaving known state updates outside React's test transaction.

Risk:

Medium. It is not a product bug, but it weakens the signal of the main verification lane.

Mechanical test:

Run `npm test` and require no `act(...)` warning from `src/App.transitions.test.tsx`. If the harness supports stderr capture, add a local assertion; otherwise the mechanical check is the clean lane output.

### F-09 - Duplicate duration formatters round the same track differently

Evidence:

| File | Line | Excerpt or measurement |
| --- | ---: | --- |
| `src/App.tsx` | 1047-1051 | `formatDuration` uses `Math.round(seconds)`. |
| `src/App.tsx` | 1534-1537 | `formatTime` uses `Math.floor(seconds)`. |
| `src/components/StandardView.tsx` | 34-38 | `fmtDuration` uses `Math.floor(seconds)` and returns empty for null/non-finite values. |
| `src/App.tsx` | 611, 666, 892, 1174-1175 | App-level duration formatting is used in multiple visible surfaces. |
| `src/components/StandardView.tsx` | 152, 525 | Standard view duration formatting is used in visible Standard surfaces. |

Why it matters:

The same track can display as different durations in adjacent surfaces. A 59.6 second track can become `1:00` in one formatter and `0:59` in another. That is a small inconsistency, but it is a classic cheap refactor with a clear test.

Concrete proposed fix:

Extract `src/lib/time-format.ts` with explicit policies: null/non-finite handling and floor-vs-round behavior. Use it from `App.tsx` and `StandardView.tsx`.

Risk:

Low. The main risk is snapshot churn in tests that pinned the old rounded string.

Mechanical test:

Unit-test null, non-finite, 59.6 seconds, exact minute boundaries, and a Standard/App smoke that proves both surfaces render the same duration text.

## P3 findings

None worth reporting. Style-only cleanup and broad decomposition ideas were intentionally excluded.

## Do not do list

Do not spend this refactor pass on:

- DSP retune or preset calibration without owner listening notes.
- The parked one-pole / soft-knee hoist.
- Tauri Specta or broader command codegen.
- Splitting `App.css` as a structural goal.
- A broad `useTrackMaster` L-scope split.
- Reference-track UX expansion, signing/autoupdate work, or album channel-count parity.
- New image-optimization dependencies unless the asset-budget test proves a simple asset replacement is insufficient.

## Suggested execution order

1. Import capability contract (F-01)
   - Verification: `npm test`, `npm run build`, `cd src-tauri; cargo test --target-dir target\codex-rc`, `cd apps/iphone-native/rust; cargo test`, `cd apps/android-native/rust; cargo test`.

2. Analysis progress identity (F-03)
   - Verification: `npm test`, `npm run build`, `cd src-tauri; cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings`, `cd src-tauri; cargo test --target-dir target\codex-rc`.

3. Export-copy and Standard recipe parity (F-02, F-04)
   - Verification: `npm test`, `npm run build`, `cd apps/iphone-native/rust; cargo check --all-targets`, `cd apps/iphone-native/rust; cargo test`, `cd apps/android-native/rust; cargo test`.

4. Test and formatting hygiene (F-08, F-09)
   - Verification: `npm test` with clean output and `npm run build`.

5. CSS and asset cleanup (F-05, F-07)
   - Verification: `npm test`, `npm run build`, and a browser/visual smoke of first-run, Standard, Advanced, and export receipt surfaces.

6. Deep-analysis allocation cleanup (F-06)
   - Verification: `cd src-tauri; cargo fmt --check`, `cd src-tauri; cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings`, `cd src-tauri; cargo test --target-dir target\codex-rc`, plus the slow fixture lane before merging if DSP/export behavior is touched.
