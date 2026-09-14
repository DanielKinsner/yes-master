# Website try-it rebuild — September 14, 2026 (afternoon)

Follows the morning assessment of `vera/web-tryit`. The owner directed: keep
the demo simple (no verdicts or warnings anywhere), make "real time" tangible,
add seek and keyboard control, fix the dead click, match the app's visuals,
decode at the file's own rate, bring the desktop's source analysis ("smarts")
into the demo with visible analysis stages, guard the checked-in WASM against
desktop DSP drift, trim all dead weight, and add analytics. Volume Match stays
in the card, off by default. No deployment or release activation.

## What changed

**Desktop crate (build-only, no behavior change)**
- `guardrails.rs` and `confidence.rs`: the six `#[tauri::command]` functions
  are gated `#[cfg(not(target_arch = "wasm32"))]` so the files compile into
  the browser engine unchanged.
- `analysis.rs`: `compute_stereo_width` / `compute_stereo_correlation` are
  `pub(crate)` (they were private).
- `types.rs`: `SourceProfile::from_measurements` is the single constructor;
  `from_analysis` delegates to it. Same sentinels, one place.
- `cargo check` and `cargo clippy --lib -- -D warnings` pass on the desktop crate.

**Engine (`web/tryit/wasm`)**
- Includes `analysis.rs`, `deep_analysis.rs`, `confidence.rs`, `guardrails.rs`
  by path; the guardrail/confidence stubs are gone. Per-stage analysis exports
  (`analyze_loudness/dynamics/stereo/tonal`, `build_profile`), and
  `master_standard` now takes the whole-track source LUFS and profile JSON and
  returns a `Render` (samples + post-landing LUFS/TP), so a render costs the
  chain plus ONE loudness pass instead of three.
- Render uses `MasteringChain::process_interleaved`, the desktop render's entry point.
- `build.rs` hashes the tracked desktop sources into `version()`;
  `scripts/build-tryit-wasm.mjs` writes the same hash to
  `src/tryit/engine/sources.stamp.json`; `src/tryit/engine-stamp.test.ts`
  fails, naming the files, when they drift. Verified: editing `Cargo.toml`
  after a build made the test fail; rebuilding made it pass.
- Release profile: opt-level 3, LTO, `panic = "abort"`, `strip`. Output
  423,708 bytes (182 KB gzipped).

**Browser side**
- `processing.ts`: `EngineWorker` (FIFO, progress streaming, timeout) and
  `PreviewEngine` (track worker + pool, snapped excerpt priming, LRU render
  cache with a byte budget, user-priority scheduling, neighbour prefetch that
  always leaves one worker free).
- `engine.worker.ts`: interleave once, real stage progress, loudest-30 s pick,
  excerpt extraction with its own source measurement, render.
- `decode.ts`: container sample-rate sniff for WAV/RF64, AIFF, FLAC, MP3,
  Ogg Vorbis/Opus, M4A; decode on an `OfflineAudioContext` at that rate.
- `TryItModal.tsx`: analysis-stage checklist while preparing; "Analyzed" chip
  with an (i) holding the profile digest and rate; seek by click inside the
  window; ←/→, Home, Shift+←/→ and a focusable waveform; discrete controls
  render immediately, continuous ones debounce 120 ms; verdict/status text
  removed entirely; analytics events (`tryit_*`).
- `tryit.css`: rewritten from scratch (299 → 150 lines), tokens copied from
  `App.css :root`, play button styled like the app's `.play-btn`, scrim-only
  `backdrop-filter` blur.
- Landing: hero button warms the chunk on hover/focus; visible loading fallback.
- Deleted: `web/tryit/index.html`, `web/tryit/pkg/`, `web/tryit/vercel.json`,
  `web/tryit/.vercelignore`.

## Evidence

- `npm test`: 889 passed, 1 failed — the pre-existing
  `src/lib/beta-feedback-contract.test.ts` beta-duration mismatch noted in the
  morning review; unrelated and untouched. `src/tryit`: 31 tests across
  processing, player, modal, decode and stamp.
- `tsc --noEmit` (app project): clean.
- `npm run verify:headless -- --out test-output/tryit/headless-rebuild`:
  PASSED (landing responsive checks and all 38 app scenario/viewport checks).
  The production build emits `engine.worker-*.js` and `yes_master_web_bg-*.wasm`.
- Browser (Chrome, office PC, 24 cores → 4 workers, local 40 s 44.1 kHz stereo
  WAV fixture, dev server):
  - Stage list observed in order: Analyzing audio → Reading loudness →
    Checking dynamics → Evaluating stereo field → Reading tonal balance →
    Building mastering context. "Analyzed" chip, digest and "44.1 kHz, your
    file's own rate, on 4 threads" shown.
  - Ready to play 0.78 s after the file was handed over (opt-level 3;
    1.29 s at opt-level "s").
  - Cold render (unique intensity value, includes the 120 ms debounce):
    475–488 ms at opt-level 3; 597–776 ms at "s".
  - Warm switch (prefetched style or loudness): 31 ms.
  - A key flipped the side; ← → seeked; Shift+→ moved the window 5 s;
    playback continued through all of it; no console errors.
  - 375×812: no horizontal overflow, card fits without scrolling.
  - Blur evidence: `.studio-site` measured 8,347 × 1,234 px at a 1,249 × 1,245
    viewport — the layer the old full-page `filter: blur(14px)` rasterized,
    6.7× the viewport the scrim's `backdrop-filter` now blurs instead.
    Frame timing could not be measured in the hidden preview pane.

## Limits

- No real phone or Safari run; iOS AudioContext-in-gesture and `OfflineAudioContext`
  rate support are reasoned from spec and Chrome behaviour, not observed.
- Subjective listening of the adaptive result in the browser was not done;
  the engine is the desktop code, and the desktop's listening signoffs stand
  for that code. The morning review's parity statement ("core chain without
  source-aware adjustments") no longer applies: the profile path is included.
- The pre-existing beta-copy test failure remains outside this work.
- `docs/OWNER_INPUT_QUEUE.md`, `docs/PRODUCT.md` and `src-tauri/Cargo.toml`
  carried unrelated uncommitted edits before this pass and were left untouched.
