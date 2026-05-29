# YES Master - Codex Adversarial Review (2026-05-29)

## Scope

Target repo: `C:\Users\SM - Dan\Documents\GitHub\yes-master`

Review focus: bloat, dead code, bad or incorrect DSP math, CSS/UX/UI hygiene,
unwired or poorly wired surfaces, and general product-risk cleanup. This was a
read-only source review. No source fixes were made as part of the review.

## Verification Run

- `npm test` passed: 17 test files, 147 tests.
- `npm run build` passed.
- `cargo fmt --check` passed.
- `cargo clippy --target-dir target\codex-review-clippy --all-targets -- -D warnings` passed.
- `cargo test --lib --target-dir target\codex-review` passed: 208 tests.
- `cargo test --target-dir target\codex-review-full` passed the full backend suite.
- Headless Chrome screenshots were taken for current UI and resized UI evidence.

Note: one concurrent cargo run failed with a Windows linker/open-file error
because two cargo test commands shared the same target directory. A separate
full-suite rerun passed, so that failure is treated as review-run contention,
not a product failure.

## Summary

The repo is not broadly broken: build, lint, formatting, frontend tests, and
backend tests pass. The material risks are narrower and sharper:

| Severity | Area | Finding |
|---|---|---|
| P1 | DSP/live metering | Mono live LUFS is about +3.01 LU hotter than real mono analysis/export. |
| P1 | DSP/export | WAV TPDF dither is implemented at twice the documented/intended amplitude. |
| P2 | Bundle/product hygiene | Preset art ships about 12.2 MB of 1024px PNGs for tiny rendered icons. |
| P2 | Repo hygiene | Generated/stale screenshots are tracked under `test-output/`. |
| P3 | DSP/dead code | Public legacy `process_sample` path intentionally diverges from real frame DSP. |
| P3 | UX/CSS | Right rail is a scroll/reachability watch item, not a launch overlap bug. |
| P3 | Maintainability | Major product surfaces are concentrated in very large files. |
| P3 | Security hygiene | Tauri CSP is disabled with `csp: null`; acceptable only if the app stays local-only. |

## Findings

### P1 - Mono live LUFS is inflated by about +3.01 LU

`src-tauri/src/sources.rs:403-406` duplicates mono input into both live meter
channels:

```rust
let l = self.frame_main.first().copied().unwrap_or(0.0);
let r = if channels >= 2 { self.frame_main[1] } else { l };
```

The same pattern exists for original playback in `MeteredPcmSource`. The
momentary meter then sums left and right energy in `src-tauri/src/dsp.rs:1609-1612`,
where the comment explicitly says duplicate mono produces a `+3 LU`
stereo-vs-mono offset. A later test comment at `src-tauri/src/dsp.rs:3981-3983`
also acknowledges that feeding the same signal to both channels adds `+3 LU`.

Impact: mono files can show live Momentary/Integrated LUFS about 3.01 LU louder
than their real mono loudness. That is a signoff problem because source
analysis/export paths use the actual channel count, while the live UI meters
report duplicated-stereo energy for mono.

Recommendation: live LUFS meters should respect the source channel count, or
the mono duplication must be made explicit in UI and export analysis. The better
fix is channel-count-correct live LUFS and a mono regression test.

### P1 - WAV TPDF dither amplitude is too hot

The header comment says TPDF should be `+-1 LSB` peak amplitude:
`src-tauri/src/wav_writer.rs:10-14`.

The implementation generates triangular noise in `[-2, 2)` LSB:
`src-tauri/src/wav_writer.rs:46-55`, then adds it directly before 16/24-bit
quantization at `src-tauri/src/wav_writer.rs:82-90`.

Impact: the export path adds roughly twice the intended dither amplitude for
integer WAV output. This is not catastrophic, but it is mastering-DSP math that
should be exact. The tests currently pin the `+-2 LSB` behavior instead of
testing against standard TPDF amplitude, for example
`src-tauri/src/wav_writer.rs:449-465`.

Additional hygiene issue: `write_samples_into_writer` creates a fresh
deterministic dither RNG on every helper call (`src-tauri/src/wav_writer.rs:123-124`).
The test at `src-tauri/src/wav_writer.rs:305-352` intentionally locks in dither
stream resets between segmented writes. That makes repeated identical segments
reuse the same deterministic dither pattern rather than one continuous render
stream. If album rendering writes track/gap segments through this helper, the
noise floor is deterministic per segment boundary.

Recommendation: decide the intended dither contract explicitly. If this is
standard TPDF, generate `[-1, 1)` LSB peak triangular noise. For continuous
album renders, prefer one RNG stream across the whole file unless byte-identical
segmented output is a hard product requirement.

### P2 - Preset art is oversized for its rendered use

`src/components/PresetIcon.tsx:3-18` statically imports eight `1024x1024` PNGs.
All eight are included in the Vite build. The UI renders them at much smaller
sizes:

- `src/App.css:1908-1909`: `clamp(48px, 4.4vw, 74px)`
- `src/App.css:4747-4748`: `clamp(48px, 3.8vw, 67px)`
- `src/App.css:4945-4946`: `78px`
- `src/App.css:5064-5065`: `37px`

Current asset sizes:

| Asset | Size |
|---|---:|
| `clarity.png` | 1678.0 KB |
| `loud.png` | 1644.5 KB |
| `oomph.png` | 1721.7 KB |
| `punch.png` | 1682.4 KB |
| `spatial.png` | 1698.8 KB |
| `tape.png` | 1738.1 KB |
| `universal.png` | 515.2 KB |
| `warmth.png` | 1566.5 KB |

Impact: about 12.2 MB of PNG art ships for icon-sized display. This is bundle
bloat, startup/binary-size bloat, and review noise.

Recommendation: downscale to the largest real display size plus density needs,
then convert to WebP/AVIF or ship multiple raster sizes. Lazy loading is less
important than not shipping 1024px PNGs for 37-78px UI elements.

### P2 - Generated and stale screenshots are tracked in `test-output`

Tracked generated artifacts:

```text
test-output/current-ui.png
test-output/preview-1600-bottom.png
test-output/preview-1600-presetfix.png
test-output/preview-1600.png
test-output/preview-check-1.png
test-output/preview-check-full.png
test-output/tauri-project-dialogs-smoke/native-dialog-save-as.ams.json
test-output/ui-tighten-e.png
test-output/ui-tighten-e2.png
```

Impact: the repo mixes source with generated visual evidence. At least one
tracked screenshot is stale relative to the current YES Master branding, which
means visual artifacts can mislead future review work. This also fights the
repo's own convention that private/release evidence lives under ignored
`test-output` subdirectories unless explicitly preserved.

Recommendation: untrack old generated screenshots, keep only intentional small
fixtures, and store dated release evidence under docs with clear provenance or
leave it ignored locally.

### P3 - Public legacy `process_sample` is a dead-code trap

`src-tauri/src/dsp.rs:2185-2187` exposes `MasteringChain::process_sample` as a
legacy fallback for callers not migrated to `process_frame_inplace`.

The test at `src-tauri/src/dsp.rs:4101-4134` proves the trap:

- legacy `process_sample` is intentionally unchanged by `low_mid`
- `process_frame_inplace` does include `low_mid`

Impact: production audio appears to use frame/interleaved processing, so this is
mostly dead code today. But leaving a public method that looks like equivalent
DSP while knowingly skipping a live EQ band is dangerous. A future caller could
use it and get a different mastering chain.

Recommendation: delete it if no production caller remains, or make it private
test-only scaffolding. If it must stay public, it should be made equivalent to
the real frame path and tested as such.

### P3 - Right rail is a scroll/reachability watch item, not a launch overlap bug

Correction from the initial review wording: this is not a startup layout bug.
`src-tauri/tauri.conf.json:17-20` declares:

```json
"width": 1920,
"height": 1080,
"minWidth": 1440,
"minHeight": 860
```

The app opens at `1920x1080`, and the supplied launch-size screenshot shows no
right-rail overlap. `1440x860` is only the allowed resize floor.

When `ADVANCED CONTROLS` is expanded, the right rail becomes a vertical
scrolling surface. That is expected: Source Check, Delivery Profile, Advanced
Controls, Per-Band Compressor, Delivery Format, Tools, and the sticky Export
group exceed the visible rail height. CSS also explicitly makes the rail
scrollable in the desktop layout at `src/App.css:4453-4455`, and gives the
export group sticky/bottom styling at `src/App.css:4868-4871`.

Impact: do not treat the current `1920x1080` view as broken. The remaining risk
is only reachability at the declared resize minimum: the sticky export group
must not hide any control that cannot be scrolled fully into view.

Recommendation: add a browser/native screenshot smoke check at launch size and
minimum resize with `ADVANCED CONTROLS` expanded, asserting that Delivery Format
and Export remain reachable through scroll.

### P3 - Large-file concentration raises regression risk

Current file sizes:

| File | Lines |
|---|---:|
| `src/App.tsx` | 2917 |
| `src/App.css` | 4459 |
| `src/hooks/useTrackMaster.ts` | 1825 |
| `src-tauri/src/dsp.rs` | 4339 |
| `src-tauri/src/audio.rs` | 3289 |

Impact: none of these line counts is automatically wrong, but they make review,
change isolation, and accidental coupling harder. The CSS especially contains
multiple historical layout passes and viewport-specific overrides, which makes
it easy to overstate or miss UI behavior without visual verification.

Recommendation: do not start with a broad refactor. Extract only when fixing
active bugs: right-rail layout tests with colocated CSS, meter/channel-count
tests near audio sources, and dither contract tests near the writer.

### P3 - CSP is disabled

`src-tauri/tauri.conf.json:23-24` sets:

```json
"csp": null
```

Impact: for a local-only Tauri app with bundled assets, this may be acceptable
during development. It becomes a real security hygiene issue if the app ever
loads remote assets, renders user-controlled HTML, or opens broader web content.

Recommendation: before release, either document why `csp: null` is safe for the
shipping threat model or add a restrictive CSP that matches the app's actual
asset/runtime needs.

## Non-Findings / Corrections

- The right rail does not overlap at the actual `1920x1080` launch size shown in
  the supplied screenshot.
- The right rail's up/down scrolling with expanded Advanced Controls is expected
  behavior, not by itself a bug.
- The automated test/build/lint suite passed after the cargo target-dir
  contention rerun.

## Recommended Fix Order

1. Fix mono live LUFS channel-count handling and add mono live-meter tests.
2. Fix or explicitly re-spec WAV dither amplitude and RNG continuity.
3. Downscale/convert preset art and confirm build output shrinkage.
4. Untrack stale `test-output` screenshots and keep release evidence intentional.
5. Delete or make equivalent the legacy `process_sample` path.
6. Add right-rail launch/minimum-resize visual reachability checks.
7. Decide and document CSP posture before release.
