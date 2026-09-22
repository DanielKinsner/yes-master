# Website audition follow-through — September 14, 2026

The owner approved all proposed visual and mechanical improvements to Vera's
website demo. Work is local on `vera/web-tryit`; the deferred desktop DSP branch
is separate. No desktop DSP, preset voicing, release activation or deployment
is included.

## Delivered behavior

- Silent preparation and file replacement; explicit Play/Pause with retained
  position; app-style Original/Mastered and Volume Match beside transport.
- Space for playback and A for A/B, with native input keyboard behavior retained.
- Moving waveform cursor, elapsed time, separate excerpt selection and seeking.
- Compact Intensity/Loudness row, mobile range control, simpler measured results
  and an optional Details disclosure. Visible scope states that this is the core
  Standard chain on an excerpt without desktop source-aware adjustments.
- Worker-based processing with 160 ms debounce, one active job and one newest
  pending job. Older results are discarded. The accepted comparison remains
  available while processing. Closing/replacing cancels outstanding work.
- Scheduled common source starts, 20 ms linear ramps for A/B and updated audio,
  and audition-only 5 ms edge fades. Source and measured PCM remain unchanged.

## Evidence

- `npm test -- src/tryit`: 13 passing tests covering silent load/replacement,
  independent A/B, pause/resume, keyboard handling, obsolete result rejection,
  bounded work, worker failure/cleanup, rapid gain reversals, aligned preview
  replacement and unchanged measured PCM around audition edge fades.
- `npm test`: 871 passed, one existing failure in
  `src/lib/beta-feedback-contract.test.ts:267`. Both the test and beta guide were
  inspected at branch HEAD: the test expects the older undetermined-duration
  wording while the guide already says eight weeks from launch. Neither file
  was changed here; the overall frontend suite is not green.
- Production build and TypeScript checks passed, with an emitted worker and
  unchanged 127,039-byte WASM asset.
- Browser inspection used a generated 40-second stereo WAV on localhost. The
  actual WASM finished paused at 0:00; seeking, explicit playback, pause/resume,
  A/B buttons and the A/Space shortcuts were exercised. Rapid preset changes
  settled on the final selection. The phone slider visibly updated from 50%
  to 75% and 76%, labeling the prior preview while work was pending.
- Layouts inspected at 1366×900, 390×844 and 320×568. Browser viewport tests
  establish layout and interaction evidence, not real phone/Safari qualification.
- `npm run verify:headless -- --out test-output/tryit/headless-final`: passed;
  landing responsive checks and all 38 app scenario/viewport checks passed.
  The first run's Album startup timeout did not recur. Its cause is not established.
  Final small-phone sticky-bar eligibility was checked in the browser separately;
  the broader headless suite does not exercise the new modal interactions.

## Limits

The shared DSP source and checked-in WASM bytes were not modified. Browser
transport changes do not certify native desktop behavior or musical preference.
Real Safari/iOS playback, large-file memory pressure and subjective listening
remain unverified in this pass. `decodeAudioData` still decodes the whole file.

Local generated fixtures and headless evidence are under `test-output/tryit/`
and are not committed. Existing unrelated `src-tauri/Cargo.toml` working-tree
state and the deferred simplification proposal are preserved.
