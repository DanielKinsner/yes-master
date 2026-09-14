# "Try it on your mix" — handoff notes (branch `vera/web-tryit`, Sept 14 2026)

The YES Master **Standard** chain compiled to WebAssembly and run in the visitor's browser. Nothing uploads, no server, no database: static files only. Two surfaces:

1. **The modal on the landing** (the product): `src/tryit/TryItModal.tsx` + `src/tryit/tryit.css` + `src/tryit/engine/`. Opened by the "Try it on your mix" button in the hero (`data-testid="hero-try-it"`), lazy-loaded, page blurs behind it.
2. **A standalone page** (scratch / demo): `web/tryit/index.html` + `web/tryit/pkg/`, deployed as Vercel project `yes-master-tryit` → https://yes-master-tryit.vercel.app. Same engine, plainer UI. Fine to delete once the modal ships.

The engine crate is `web/tryit/wasm/` (`yes-master-web`). **It does not modify the app crate**: it includes `src-tauri/src/{types,dsp,export_format}.rs` by `#[path]`, and `src/stubs.rs` supplies inert versions of `guardrails`, `confidence`, `deep_analysis` and `mp3::delivery_rate`. In Standard there is no source profile, confidence or compression guards (`None`), so the identity stubs are exactly the desktop's own no-analysis path.

## Rebuild

```sh
# toolchain (already installed on Dan's WSL): rustup + target wasm32-unknown-unknown + wasm-bindgen-cli 0.2.128
cd web/tryit/wasm
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir ../../../src/tryit/engine target/wasm32-unknown-unknown/release/yes_master_web.wasm   # modal
wasm-bindgen --target web --out-dir ../pkg --no-typescript target/wasm32-unknown-unknown/release/yes_master_web.wasm       # standalone
# parity check against the desktop render (any wav):
cargo run --release --example native_check -- in.wav out.wav universal 0.5 -14
```

Then the normal `npm run typecheck && npm run build`; Vite emits the `.wasm` as an asset (`?url` import) and the modal as its own chunk.

## What I'm confident in (verified)

- **DSP parity.** `master_standard(universal, 0.5, −14)` on the raw "I Know You're Right" mix lands **−14.00 LUFS / −5.89 dBTP**; the desktop engine (`ab_render` example calling `engine::mastering_render_to_path`) gives −14.0 / −5.9 on the same file. Landing math mirrors `engine::ceiling_bounded_landing_delta_db`; loudness measured with the same `ebur128` crate (BS.1770, integrated + true peak).
- **Speed.** 30 s stereo @ 44.1 k masters in ~0.5 s in Chrome (wasm, single thread). 177 s in 2.1 s native.
- **Chrome desktop + Chrome-on-Android-shaped viewport (390×844, touch emulation):** drop/pick file, loudest-30 s default, drag the window, all four tiles, Low/Medium/High, match-loudness, same-playhead flip via button and space, Escape closes, focus returns. No page errors. No horizontal overflow at 390 px. Landing test suite 46/46, typecheck clean.
- **A/B switch** is a 35 ms equal-power crossfade (`setValueCurveAtTime`), not a ramp. Both sources play continuously from the same start time; only gains move.
- **Privacy claim is technically true.** The file is read with `File.arrayBuffer()` → `decodeAudioData`; the only network requests are the page's own assets. The landing loads `@vercel/analytics` only on the public site URL (see `src/main.tsx`) and it never sees audio; if you want the claim checkable in devtools, keep it that way.

## What I could NOT verify from here (look into these first)

1. **Safari / iOS Safari.** No Safari on this machine. Specific risks:
   - `decodeAudioData` on iOS needs the `AudioContext` created inside the user gesture; I create + `resume()` it synchronously in the file-input `onChange` before any `await`. Confirm audio actually plays after the first pick on a real iPhone (not just that the numbers appear).
   - FLAC is not decodable by Safari's `decodeAudioData` → the "Couldn't decode" message path. WAV/MP3/M4A should be fine.
   - `setValueCurveAtTime` overlapping a previous curve throws in some WebKit versions. I `cancelScheduledValues` first; if you see `InvalidStateError` on rapid flips in Safari, fall back to `linearRampToValueAtTime`.
   - `100svh` and `backdrop-filter` on older iOS.
2. **The Intensity knob on a real touchscreen.** `Knob.tsx` uses pointer events with `touch-action: none`, and drag works in desktop Chrome (50 → 90 %). Under Playwright's iPhone emulation the drag did not register (could be the emulation). As insurance there is a plain `<input type=range>` under the knob that only shows on `(hover: none)` devices. Decide whether to keep it, or fix the knob for touch and remove it.
3. **Memory on long files.** Whole-file `decodeAudioData` holds the full track as Float32 (a 6-minute 48 k stereo wav ≈ 138 MB decoded). Fine on desktop; unknown on low-end phones. If it matters, decode only the first N minutes (needs a WAV parser) or cap file size with a friendly message.
4. **Rapid control changes.** Rendering is synchronous on the main thread (~0.5 s). Clicking a tile then a loudness button within that window shows the first result for a moment before the second lands; the verification line now reports the target the render actually used, so it never lies, but the UI freezes ~0.5 s per change. Moving `master_standard` into a Web Worker removes the freeze (the wasm is already dependency-free; `wasm-bindgen --target web` works in workers with `type: "module"`).
5. **Landing copy / tests.** I added a hero button, not copy in `page-copy.json`; `LandingCopy.test.tsx` passed, but if there is a "no untracked strings" rule, move "Try it on your mix" into `page-copy.json`.
6. **Bundle budget.** The modal chunk is 18 KB JS + 14 KB CSS + 127 KB wasm, all lazy; the landing's first paint is unchanged. Preset art PNGs are shared with the app and already in the bundle.
7. **IP exposure.** The Standard chain ships as wasm to every visitor. Compiled, LTO, no symbols beyond the two exports, but not un-reverse-engineerable. This was Dan's call (Standard only, not Advanced).

## Small known gaps
- Keyboard arrows on the knob's hidden fallback input didn't change the value in my run; mouse/pointer drag does. Not investigated.
- No waveform playhead cursor while playing (nice-to-have).
- The `.tryit` CSS mirrors `App.css` `.std-*` / `.knob-*` rules by hand because the landing never loads `App.css`. If those app styles change, the modal won't follow automatically.
- Standalone page and modal share the engine but not the UI code; treat the standalone as disposable.

## Where things are
- Branch: `vera/web-tryit` (5 commits on top of `origin/main` as of Sept 14). Nothing merged.
- Vercel previews for the `yes-master` project are behind Vercel authentication (302). Prod = merge to main.
- Windows-side scratch build of the desktop engine used for the parity check: `C:\Users\Daniel Kinsner\AppData\Local\Temp\yesm-ab-build` (safe to delete).
