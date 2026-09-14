# Try it on your mix — current handoff

Branch: `vera/web-tryit`. September 14, 2026 local follow-through, authorized by
Dan after reviewing the original implementation at `940297f8`. No deployment or
merge is part of this follow-through.

## Product surface

The landing page lazily opens `src/tryit/TryItModal.tsx`. It imports the desktop
preset artwork, Knob and Standard style/loudness mapping. Audio is read locally
with `File.arrayBuffer()` and `decodeAudioData`; it is not uploaded.

- Loading and replacing a file finish paused. Play/Pause retains the playhead.
- Original/Mastered is an explicit selector, independent of transport. Space
  controls playback; A changes sides, following the desktop shortcuts. Value
  controls retain their own keyboard behavior. Volume Match is off by default.
- A waveform shows the selected excerpt and moving playback cursor. Section
  selection and seeking are separate controls. New sections start at their
  beginning when ready; sound-setting updates preserve the playback position.
- Intensity and Loudness share a row. Phones have a visible Intensity slider.
  Larger portrait phones retain the transport while adjusting settings; the
  smallest screens use normal scrolling so controls aren't obscured.
- Result text is concise. Details discloses excerpt scope, engine timing and
  audition-only edge fades. The primary copy identifies the desktop's additional
  full-track analysis and export capabilities.

## Processing and playback

`engine.worker.ts` owns full-track PCM, waveform peak extraction, loudest-section
selection and WASM rendering/measurement. `processing.ts` keeps one active job
and one replaceable pending job. Control changes invalidate older results
immediately and debounce for 160 ms. The previous accepted comparison stays
usable during an update. Closing/replacing cancels the worker and audio context;
failures settle pending work and offer an actionable error instead of hanging.

`player.ts` owns synchronized audio sources. A/B and Volume Match use 20 ms
linear gain ramps, including rapid reversals. New rendered previews crossfade
at the same playhead, scheduled ahead of the audio render quantum. Both sides
receive 5 ms fades at excerpt boundaries for audition; measurements use the
unmodified PCM. No DSP voicing or desktop processing source changed.

The WASM wrapper includes desktop `types.rs`, `dsp.rs`, and `export_format.rs`
by path. Its guardrail/analysis stubs are inert. **This is the desktop's core
chain without source-aware adjustments, not a promise of identical results to
an analyzed desktop Track Master.** Desktop Standard can use source analysis.
Excerpt loudness and browser decoding/sample rate can also differ from a
whole-track desktop export.

## Rebuilding the engine

The existing checked-in WASM was not rebuilt by this UI/transport change.
Original build instructions (WSL with wasm-bindgen-cli 0.2.128):

```sh
cd web/tryit/wasm
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir ../../../src/tryit/engine target/wasm32-unknown-unknown/release/yes_master_web.wasm
```

`npm run build` packages the worker, WASM and lazy modal. The standalone scratch
page in `web/tryit/index.html` has its own UI and was not updated; it must not be
used as proof of the product modal's current behavior.

## Verification and remaining limits

See `docs/reviews/2026-09-14-web-tryit-audition.md` for this change's evidence.
The original handoff at `940297f8` contains Vera's earlier single-fixture parity
and timing observations; they are not a full desktop/browser equivalence gate.
Real Safari/iOS, long-file memory pressure and subjective music listening remain
separate checks. Browser decoding still materializes the entire input in memory.
The modal's CSS mirrors desktop styling locally; future desktop style changes
need a corresponding visual review here.
