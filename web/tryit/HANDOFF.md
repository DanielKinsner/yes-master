# Try it on your mix — current handoff

Branch: `vera/web-tryit`. Rebuilt September 14, 2026 (afternoon pass, authorized
by Dan after the morning review). No deployment or merge is part of this pass.
Evidence for this pass: `docs/reviews/2026-09-14-web-tryit-rebuild.md`.

## What it is

`src/tryit/TryItModal.tsx` opens over the landing page from the hero's
"Try it on your mix" button. Audio is read locally, decoded at the file's own
sample rate, analyzed and mastered in Web Workers running the desktop's DSP
compiled to WebAssembly, and played through Web Audio. Nothing uploads.

## What's in the folder

| Path | Role |
| --- | --- |
| `web/tryit/wasm/` | The engine crate. `src/lib.rs` includes the desktop's `types.rs`, `dsp.rs`, `export_format.rs`, `analysis.rs`, `deep_analysis.rs`, `confidence.rs` and `guardrails.rs` **by path** and exposes the analysis stages, profile construction and `master_standard`. `src/stubs.rs` holds inert stand-ins for `decode`, `files` and `mp3` (named by the included sources, never called). `build.rs` stamps the binary with a hash of those sources. |
| `web/tryit/wasm/examples/native_check.rs` | Native parity check: runs the same analysis + render on a WAV and prints delivered LUFS/TP for comparison with a desktop export. |
| `scripts/build-tryit-wasm.mjs` | `npm run build:tryit-wasm`: cargo + wasm-bindgen into `src/tryit/engine/`, then writes `sources.stamp.json`. |
| `src/tryit/engine/` | The checked-in build output (wasm, JS glue, typings, stamp). |
| `src/tryit/engine-stamp.test.ts` | Fails when any tracked desktop source changed since the last build, naming the files and the command. |
| `src/tryit/processing.ts` | Worker pool, render cache, neighbour prefetch. |
| `src/tryit/engine.worker.ts` | Worker body: whole-track analysis with real stage progress, excerpt extraction, rendering. |
| `src/tryit/decode.ts` | Container sample-rate sniff (WAV/AIFF/FLAC/MP3/OGG/M4A) and decode at that rate. |
| `src/tryit/player.ts` | Synchronized Original/Mastered sources, 20 ms linear ramps, audition-only edge fades. |
| `src/tryit/analytics.ts` | Vercel Web Analytics custom events, public site only. |
| `src/tryit/tryit.css` | The card's styles; tokens copied from `App.css :root`. |

The earlier standalone scratch page, its `pkg/` copy of the engine and its own
`vercel.json` were deleted in this pass. The modal ships inside the landing
build (root `vercel.json`, `dist/`); there is no second deploy target.

## How it behaves

- **Load**: drop or pick a file. The card shows the desktop's analysis stages
  as they really happen (loudness, dynamics, stereo field, tonal balance,
  mastering context) plus "Choosing your 30 seconds". Loading finishes paused.
- **Analysis is the app's**: whole-track `SourceProfile` via
  `SourceProfile::from_measurements`, adaptive strength 0.5, confidence and
  compression guards `None` — exactly what desktop Track Master resolves while
  the owner gates for Phase-B confidence and the Adaptive Compressor stay off.
- **Render**: the 30 s excerpt runs through `MasteringChain::process_interleaved`
  + `flush_render_tail`, one loudness pass, the ceiling-bounded landing. The
  render carries its own post-landing measurement (no second pass).
- **Feel**: tiles and pills render at once; the intensity slider and window
  drag debounce 120 ms. Every render is cached by exact settings; after each
  user render the other loudness levels for the style, then the other styles,
  are pre-rendered on spare workers (one worker is always left free). Worker
  count = `hardwareConcurrency − 1`, clamped 1..4; the cache budget is 128 MB
  (48 MB when `deviceMemory ≤ 4`).
- **Transport**: Play/Pause keeps the playhead. Original/Mastered is a
  selector independent of transport. Space plays, A flips, ←/→ seek 5 s, Home
  seeks to 0, Shift+←/→ nudge the excerpt window; the waveform is focusable
  and its own arrows nudge the window. Clicking inside the lit window seeks.
  Volume Match is off by default and audition-only.
- **No verdicts**: there is no loudness readout, verdict or warning text
  anywhere in the card. The (i) beside "Analyzed" holds the profile digest and
  the processing rate for anyone who looks.
- **Sample rate**: decoded at the container's declared rate (48 kHz fallback),
  matching the desktop's preserve-source-rate policy. Playback context asks
  for the same rate and falls back to the device rate.
- **Dead-click fix**: hovering/focusing the hero button warms the modal chunk;
  the Suspense fallback is a visible "Loading the demo…" scrim.
- **Blur**: only the scrim blurs (backdrop-filter, viewport-sized). The old
  `filter: blur()` on `.studio-site` rasterized the whole 8,347 px-tall page.

## Rebuilding the engine

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
npm run build:tryit-wasm
```

Then commit `src/tryit/engine/`. `npm test` runs the stamp test; it fails with
the list of drifted files until you rebuild. Windows works (this pass was built
on the office PC); WSL is not required.

## Remaining limits

- Real Safari/iOS playback, phone-class render timing and subjective listening
  remain unverified in a browser session; jsdom and desktop Chrome evidence only.
- `decodeAudioData` still holds the whole decoded file once; the worker keeps
  one interleaved copy. Long files cost memory before the excerpt is cut.
- M4A files whose `moov` box sits after a large `mdat` fall back to 48 kHz
  decode; HE-AAC's SBR doubling is not modelled.
- The (i) digest exposes the profile numbers; nothing else in the card does.
