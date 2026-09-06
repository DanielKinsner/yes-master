# YES Master — listening follow-up implementation plan

Prepared September 5, 2026. **Implementation in progress; local checkpoints below.**

## Implementation evidence

### Checkpoint 1 — VM edit attenuation (September 5)

- Start: `7a5d71c`, Windows, i9-13900K (24 cores), dev app opt-level 1 / dependencies 3. Focusrite USB Audio is present; no app was running at inspection. Device rate/buffer and native playback are not established by these offline tests.
- Preserved pre-existing AdvancedPanel/fields/App.css/adaptive-strength and landing capture/assets/design edits. No private audio regenerated or staged; no push/release.
- Baseline: 93 audio-controller tests and 842 frontend tests passed. Real `MasteringSource` output reproduced a **16.26 dB** warm-VM jump on an uncached 0.01 dB EQ edit (ignored local log `test-output/listening-vm-red.log`).
- Cache misses now retain the last **applied same-source attenuation**, without inheriting a boost or an obsolete worker's result. VM OFF remains unity; source/cache changes and device recreation reset retention. Existing live coefficient crossfades handle the transition; export processing is untouched.
- Regression now passes through pending and newly measured audio. Additional coverage includes 20 pending edits, stale cache insertions, invalid/boosted fallbacks; existing cold/off/A-B/source-generation tests remain. Strict Clippy all-targets passed. Full desktop Rust suite passed, including 450 library tests (2 ignored diagnostics) and all ordinary integration suites; no snapshot/tolerance changes.
- Logs: `test-output/listening-vm-{red,green,clippy,rust}.log` (local/ignored). Native settings-edit/Focusrite listening, five-minute stress, fixture lane, and platform-specific evidence remain separate outstanding checks. This checkpoint corrects the demonstrated fallback, not a blanket performance or release pass.

### Checkpoint 2 — shared playback PCM and durable failure context

- Playback/prewarm caches, A/B sources and preview workers now share immutable
  decoded PCM. The 60-minute/96 kHz buffer is 2.76 GB; three deep copies measured
  1.987–2.212 seconds, versus below the diagnostic's microsecond display precision
  for shared clones. Pointer-identity coverage proves the allocation change.
- Playback reply failures preserve the original error and source/request/mode
  context in the existing durable diagnostic log, on the command caller rather
  than the audio callback. A persisted-log regression verifies the context.
- Full Rust suite passed (450 library tests, 4 ignored diagnostics, all ordinary
  integrations). The initial flagged run found no convention-folder private
  fixture; it is not counted as private-audio evidence. The runner now accepts
  `AMS_REAL_FIXTURE_PATH` so existing files can stay in place. All four real-fixture
  contracts then passed on `TEST-3min-MONO-48khz.wav`, including import/analyze,
  actual WAV render/reanalysis and metering snapshot (delivered -12.82 LUFS,
  -1.10 dBTP). Logs: `listening-pcm-fixtures.log`, `listening-real-fixture-mono.log`.
- Strict Clippy passed for the PCM/log change. These desktop-private source/cache
  types do not change native bridge signatures or shared DSP behavior.
- [Measured stages and limits](../listening/2026-09-05-follow-up-evidence.md)
  distinguish reduced setup copies from the still dominant full-chain work.
  Three bit-identical DSP experiments showed no speedup and were discarded.
  Native mono import and VM edit control-state evidence is recorded separately;
  hardware audio continuity, five-minute stress and global worker bounds remain open.
- [Concrete export/loading options](2026-09-05-export-options.md) preserve the
  unresolved owner choices. No codec, naming, manifest or loading behavior changed.

### Checkpoint 3 — preview worker lifecycle

- Confirmed: source epochs discarded obsolete results but detached old-source
  renders continued. One process-wide preview permit now bounds heavy preview
  work across source/device lifetimes. Busy requests retain one latest pending
  request and retry on the existing controller tick; no waiting on the audio thread.
- Source changes/device teardown cancel obsolete jobs. The same chain state
  continues across 8192-frame blocks, with cancellation before blocks, tail and
  SRC phases and during EBU feeds. A/B on the same source retains useful in-flight
  work. The VM window allocation also moved entirely onto the worker.
- Exact sample comparison with the former whole-slice chain/tail/SRC path passes;
  asynchronous cancellation returns no gain; 20 source lifetimes cannot release
  a still-running worker's permit. Full Rust suite with the existing 3-minute
  192 kHz stereo fixture passed. Final library check: 451 passed, 4 ignored;
  strict Clippy all-targets passed. iPhone check + tests passed (46 tests, 1 ignored);
  Android host tests (26) and arm64 API-29 cross-check passed.
- This bounds **preview** work, not overlapping imports/exports. VM's eight-second
  calculation and an already-entered SRC call are still non-interruptible;
  cancellation is cooperative, not an instantaneous command-latency guarantee.
  No approximation, preset tuning, independent chunk filtering or chunk LUFS
  averaging was introduced. Native dropout/callback deadline evidence remains open.
- Fixed build stamping to watch the resolved branch ref, index and source paths;
  the native restart now reports `1f2d4f6+` at 19:24 rather than the stale `7ae9fc2+`
  seen in the earlier dev binary. This identifies the compiled local dirty state;
  it is not exact-commit CI. Logs use the `listening-worker-*` and
  `listening-native-final-dev.log` names under ignored `test-output/`.

### Checkpoint 4 — Width, inactive region, mono and Album completion UI

- Width retains the last same-context resolved value/thumb during asynchronous
  recalculation and labels it updating. Delayed/out-of-order replies and track
  changes are covered. Manual width remains explicit. Numeric target/ceiling
  overrides now say Custom; inherited values say Profile, genuinely unresolved
  values Auto. No settings were changed to align labels.
- Standard hides inactive loop shading, preserves remembered Advanced geometry,
  and keeps looping disarmed. The intermittent native start-position anomaly
  remains open: existing hook/controller loop regressions pass, but this is not
  a completed 20-transition native reproduction attempt.
- Known one-channel sources show Mono; stereo/unknown width buckets stay intact.
  The supplied 3-minute mono file was verified in the native Insight panel.
- Successful Album export opens its existing full receipt next to the export
  workflow; dismissal/reopen and keyboard focus restoration work without focus
  theft. Visual review caught rail clipping and corrected it using a body portal.
  The headless check now probes visible receipt corners as well as all four
  metrics/results. Cancelled exports never acquire a success heading.
- Isolated UI checkout (baseline `9238a55` plus only this slice): 845 frontend
  tests and all 31 headless scenario/viewport checks passed. Final evidence:
  `test-output/listening-ui-check/test-output/headless/2026-09-06T02-23-52-495Z`.
  Album completion and Track screenshots were visually reviewed at minimum and
  laptop sizes. Canonical captures were regenerated through the existing pipeline;
  its historical loss of untouched studioArtwork metadata was repaired by carrying
  that original HEAD section forward, without changing generated hashes/digests.
  The owner's independent capture-script fix and UI edits remain outside this commit.
- Extra keyboard probes: Manual compressor + expanded Tools, Track/Album at
  1360x740 and 1920x1080, reach the delivery controls with visible focus. At 1920x1080
  the normal Track rail has zero overflow; fully expanded Manual needs 57 px
  (Album 97 px in the isolated baseline). At minimum, intentional travel is
  397/437 px. No few-pixel defect was demonstrated, so scrolling was not hidden.
  At 200% equivalent layout (960x540 CSS viewport, device scale 2) the delivery
  controls remain reachable. This is browser layout evidence, not native zoom;
  compressed center A/B text at that effective width remains a separate limit.
- Existing Width mono/identity/side/mid/clamp/full-chain and Loud golden,
  compressor-link/asymmetry, finite-rate, metering and export invariants all passed
  in the Rust checkpoint. No objective Width or Loud audio defect was found in
  those named cases. This does not claim the entire proposed listening matrix.
  Native Album export interaction and VM-off saved-file listening remain targeted
  follow-up evidence; the already-passed original owner session is retained.

### Checkpoint 5 — combined tree and native saved-file evidence

- Combined working tree (including the preserved independent UI edits): 846
  frontend tests and all 31 headless checks passed. Evidence:
  `test-output/headless/2026-09-06T02-33-13-657Z`. Current deterministic captures
  were refreshed with the owner's existing capture script; pre-refresh bytes
  remain in ignored `test-output/listening-original-captures/`. These combined
  generated assets remain uncommitted with the independent work.
- The rebuilt native app imported the existing `TEST-10min-192khz.wav`; READY
  was observed at the first check 51 seconds later (not an exact completion time).
  Over 313 seconds of native audition the playhead reached 5:11 and remained
  playing. Coverage: 20 edits across Intensity/High/Density with VM on and Preview
  LUFS off; 6 Intensity edits with Preview LUFS enabled; 6 High edits with both
  level options off; 6 A/B switches at 1.5-second cadence plus 8 at 150 ms.
  Sampled meters stayed finite and full-track Preview LUFS completed. No playback
  error was observed. Native process working set was about 961–962 MiB after
  settling; its lifetime peak was 2721 MiB, including import/measurement work.
  This is transport/state evidence, not captured output audio or callback timing.
- Actual native Album export of the four imported fixtures completed under
  ignored `test-output/listening-native-album-2026-09-05/`, preserving the current
  album title, Cinematic arc, flow amount and track settings. Its receipt opened
  automatically and showed delivered LUFS, true peak, target and ceiling for all
  four tracks. Native dismissal and reopen were verified. All source hashes were
  unchanged; existing owner outputs were not modified.
- Independent FFprobe/FFmpeg validation: all five WAVs (four tracks + continuous)
  decode as 48 kHz / 24-bit / stereo. Track durations are 120, 60, 180 and 600 s;
  continuous duration 961.6 s includes the two requested 0.8 s gaps. Per-track
  FFmpeg integrated LUFS / true peak: -15.1/-6.4, -12.6/-4.5, -11.2/-1.7,
  -14.2/-6.0. These match the receipt at FFmpeg's printed 0.1-unit precision.
  No continuous-file loudness claim was invented for a receipt without that field.
- Logs and artifacts: `listening-native-stress.json`, `listening-native-album-ui.txt`,
  `listening-native-album-independent.json`, `listening-native-source-hashes-*.json`.
  No audio or private outputs entered git. This closes the native Album completion
  interaction/measurement check above; musical Album listening is still separate.
- Remaining: targeted ears-on VM transition/dropout and VM-off Track saved-file
  comparison; the intermittent loop-start sequence (including native region
  drawing) and wider Width/Loud listening matrix; exact output buffer/deadline
  instrumentation and further full-chain throughput work. Loading workflow,
  automatic import measurement, manifest/suffix and MP3 scope remain unresolved
  product choices in the owner queue. No release/push/deployment was activated.

## Start here

**First: reproduce and correct the brief Volume Match level jump. Then address high-rate/long-file responsiveness.** Follow with targeted Width/Loud checks, loop behavior, and the reported UI/export issues. Keep the import/loading product discussion and additional codecs separate from those corrections.

The owner broadly likes the sound and explicitly wants taste left alone unless there is an objective mechanical problem. This plan is not permission to re-voice presets, enable gated features, or activate a release.

Read the [reconciled owner handoff](../listening/2026-09-05-owner-handoff.md) for the actual answers. Its observations supersede the original guide's generated warnings and broad mission labels. **Do not ask the owner to repeat details already recorded there.**

This plan, the reconciled handoff, the original owner exports, and the reusable guide are now maintained in the repository; see the [listening records index](../listening/README.md). The September 5 owner pass is recorded in the live agent instructions and evidence ledgers. Planning and this documentation integration did not implement application fixes, retune sound, or authorize release activity.

## Evidence and boundaries

- The listening session names owner-verified build **`e600a21`**, running through `npm run tauri dev` on Windows with Focusrite USB output and studio monitors.
- The read-only planning inspection found local HEAD **`7ae9fc2ce60a7876b8fe3ca198dbe0f44445f8a9`**. The committed delta since the listening build concerns website analytics and related docs/dependencies; do not silently equate either commit with the exact running app plus its working-tree changes.
- At the planning inspection, the working tree had other edits to `AdvancedPanel.tsx`, `fields.tsx`, `App.css`, an adaptive-strength test, the landing capture script/assets, and untracked guide/design files. The guide has since been included in this documentation integration; preserve the independent application/design work. The visible UI changes inspected here concern the Album Adaptive explanation/tooltip; they are not evidence that the listening issues have been fixed.
- Static code findings below are leads and current behavior descriptions, not new native playback results. No tests, benchmarks, or app sessions were run for this plan.
- Before implementation, read the current `AGENTS.md` and task-relevant sections of the live docs; reuse material already understood in this session. The July 24 quality plan remains the forward queue, `beta-go-no-go.md` remains the release gate, and the September 4 audio-correctness ledger supplies the immediate history. Do not reopen shipped historical work simply because it appears in an old checklist.
- Implement in small, independently verifiable changes. When an objective problem is established, add a meaningful regression that fails before the correction. Do not change expected audio snapshots merely to hide a regression.

### Preserve the successful listening results

| Existing result | How to treat it |
| --- | --- |
| Normal A/B is seamless, preserves playhead, and gives a fair Volume Match comparison | Keep it passed for the tested conditions. Use it as a regression guard while fixing setting changes and stress cases. |
| Quiet-to-loud contrast is natural at normal settings | Keep it passed. Compression near maximum Intensity is not automatically a defect. |
| Intensity and Manual threshold/ratio sweeps felt smooth; compressor Off was auditioned | Preserve those successful checks. |
| The saved Track Master file sounded right, had intact ends/correct format, and reanalysis matched its receipt | Preserve the observed export pass. Do not turn performance notes written in that section into an export corruption report. |
| Center and sides were stable at normal settings | Preserve that limited positive result. Broader Width behavior still needs the requested mechanical review. |

The owner still selected **“Not ready / stopped here.”** This plan does not replace that with signoff. Genuine remaining listening limits are the unestablished Volume-Match-OFF export-comparison condition and the explicitly untested Album cases; they do not erase the successful checks above.

## Work order

Priority describes the proposed implementation order, not a newly imposed beta gate.

| Step | Deliverable | Completion evidence |
| --- | --- | --- |
| 0 | Identify the actual baseline and available reproduction material | Build/configuration recorded; existing edits protected; no blanket fixture request. |
| 1 | Remove the Volume Match setting-change level jump | Failing regression corrected, measured/native transition check, unchanged export behavior. |
| 2A | Explain high-rate/long-file cost with measurements | Separate timings for decoding, live playback, VM, whole-track landing, and analysis; baseline report. |
| 2B | Fix proven playback/resource bottlenecks in small steps | Same-fixture before/after measurements and native stress evidence; no silent approximation of preview loudness. |
| 3A | Stabilize the Width Auto readout | Delayed/out-of-order response regression and visible UI check. |
| 3B | Mechanically check Width and Loud | Existing and targeted DSP/meter tests; explicit finding or “no defect found in this scope,” without retuning. |
| 4 | Resolve loop/playback state and Standard's region presentation | Native reproduction or a precise unreproduced record; state/UI regressions for any correction. |
| 5A | Make Album export completion/receipt easy to find | Complete, readable receipt after export; preserved measurements and cancelled-export behavior. |
| 5B | Correct label consistency and incidental rail scrolling | Truthful labels and viewport/keyboard verification, with intentional scrolling retained where needed. |
| 5C | Settle Album file naming and manifest presentation | Small documented output-design choice before changing the file contract; non-overwrite coverage. |
| 6 | Scope and then deliver the requested extra export format | WAV + MP3 mastering-export specification and real encoded-file validation. |
| 7 | Close with focused verification and one short owner check | Evidence for changed behavior; original successful results retained; unresolved items named precisely. |

The analysis/loading discussion can happen after **2A** supplies evidence. Steps 3–5 need not wait for an undecided new loading experience or codec design. If an intermittent bug stays unreproduced after the bounded checks below, record that limit and continue independent work; do not invent a fix or close the issue as passed.

## 0. Establish the baseline without repeating the interview

1. Record the implementation-start commit, existing uncommitted changes, launch/build profile, audio device, output rate, and buffer settings where observable. Do not reset, stage, or incorporate another task's edits accidentally. Use an isolated checkout if that is needed to protect concurrent work, with the baseline stated explicitly.
2. Use the existing material; **do not regenerate the step 2A fixtures**. All 15 files have been located and their headers verified; use the [fixture inventory and owner additions](../listening/2026-09-05-performance-fixtures.md). On this machine the folder is `C:\Users\Daniel Kinsner\OneDrive\Documents\Vera_save created documents here` (one directory named `Vera_save created documents here`, not a `Vera\_save...` pair). Resolve Documents on other machines and reuse the supplied files. Additional named listening material:
   - `doors open neon nights remix`: whole track, Punch 100% Intensity, Preset Density 0/50/100, Volume Match on; source reported around −3.7 dBTP.
   - `lay the money on the desk original (1)`: 0:35–2:00, Universal 100% Intensity, Preview LUFS on/off and Volume Match comparisons.
   - Album outputs: `E:\fghgfhjghjhg`, supplied by the owner for possible consistency analysis.
3. The performance files share one looped/trimmed 48 kHz source; high-rate versions are soxr-upsampled per the owner. No source content above 24 kHz is expected, which is acceptable for this load comparison but does not prove behavior on native ultrasonic-rich content. Only `TEST-30min-loop.wav` is 16-bit; all others are 24-bit. MONO files are true one-channel. Reuse existing synthetic probes for additional mechanical edge cases; do not recreate this supplied set. Ask for a private source only when a specific unresolved behavior needs it. Never put private audio or masters in git.
4. Note that the dev build is already optimized: the inspected Cargo profile uses `opt-level = 1` for the app and `3` for dependencies. Compare dev and release when profiling; do not dismiss the report as an entirely unoptimized debug build.

**Done:** The next agent can reproduce against a named configuration and distinguish baseline behavior from existing edits. No sound or UI changes are required for this step.

## 1. Volume Match setting-change level jump

**Owner evidence:** Mastered + Volume Match, changing essentially any setting causes about **one-third of a second** at a much louder/unmatched level before returning to the matched level. The owner calls this a major preview problem and says it does not occur when Preview LUFS is enabled. The trigger is already known.

**Code lead:** In `audio.rs`, `apply_preview_volume_match_gain_cached` uses **`1.0` on a cache miss**. `publish_preview_coeffs` publishes those coefficients before the background result arrives. This could produce the reported temporary loss of attenuation. It does not prove that compression/limiting actually bypassed. The current cold-lookup unit test explicitly expects the initial unity fallback, so retain its no-synchronous-work guarantee while distinguishing first measurement from an edit during an already-matched audition.

### Bounded implementation

1. Reproduce a **warm matched state → uncached settings edit → new measurement** with a deterministic signal and settings that need meaningful attenuation. Observe both the applied gain transition and actual output audio.
2. Test the owner's setting-change sequence in the native app. Contrast VM on / Preview LUFS off with Preview LUFS on, then VM off. Include isolated EQ, Intensity, and Density edits so the result is not tied to one slider.
3. If the fallback is responsible, change the transition policy so recomputation does not temporarily remove the established attenuation. A same-source retained attenuation plus a smooth move to the new valid value is a working hypothesis; validate it and choose the best implementation within this task without an additional permission step. Handle prior boosts conservatively and never reuse another track's measurement.
4. Cover first-ever VM measurement, turning VM off, rapid edits, stale worker results, same-source A/B, track changes, and device recreation. Keep cold measurement off the audio/command-critical path and preserve the existing source-epoch protection.

### Acceptance

- A regression reproduces the **edit after a valid match** problem before the fix; it does not merely assert the chosen internal variable value.
- The native sequence no longer produces the temporary unmatched blast. Start with 20 representative setting edits across the contrasted modes, including the supplied Punch context when available; adjust the workload to cover the demonstrated failure and explain the evidence, rather than treating the count alone as proof. Distinguish intentional level/tone changes from a temporary compensation reset.
- No new blocking measurement, playhead reset, or stale-track gain application occurs. Pending status remains truthful.
- Ordinary A/B retains its successful behavior.
- Toggling audition Volume Match does not change exported PCM or receipt levels. Keep the limiter and all existing export safety behavior intact.

**Likely files:** `src-tauri/src/audio.rs`, relevant live coefficient application in `dsp.rs`, and existing audio-controller/preview tests. Follow the measured cause across subsystem boundaries; this starting file list does not limit investigation or the necessary correction.

## 2. High-rate and long-file responsiveness

### 2A. Measure separate costs before selecting a fix

The report contains both long **measurement waits** and real **dropout/lag**. EQ could already be audible while Preview LUFS was still measuring. Treat these as separate behaviors.

**Starting hypothesis, with corrections:** the owner's timings imply about **1.57–1.96 million source frames/second**, suggesting a dominant per-frame cost. They do not establish that frame count is the only cost. Three minutes at 192 kHz is **34.56M frames**, versus **28.8M** for ten minutes at 48 kHz: comparable work, not identical (20% more frames). The [fixture note](../listening/2026-09-05-performance-fixtures.md) records the calculations.

**Confirm the call path before instrumenting.** `audio.rs` does spawn background workers using `std::thread::Builder::new().spawn(...)`. Within the inspected landing request, `engine::preview_landing` calls `render_preview_landing_window`; `MasteringChain::process_interleaved` runs a serial `chunks_mut` frame loop, followed by optional sample-rate conversion and one ebur128 feed with `Mode::I | Mode::TRUE_PEAK`. No Rayon/parallel-chunk work was found in those application paths; inspect the chosen dependencies before extending that statement to the entire process.

Time the **stateful chain and tail flush**, **copies/allocation**, **sample-rate conversion**, **K-weighting/filtering and block-energy accumulation**, **global gated integration**, and **true-peak analysis** separately. Source-rate frame throughput does not directly measure the integration throughput after delivery-rate resampling. Compressor/limiter/filter state prevents naive independent chunk rendering. Block-energy reduction is a candidate for parallel work, but the whole loudness pass is not automatically chunk-independent: preserve K-filter history, overlapping 400 ms blocks/100 ms hops, boundary/tail handling, and one global absolute/relative gate. Never average chunk LUFS values or reset the filters per chunk and call the result equivalent. True-peak interpolation also needs correct boundaries. Prove equivalence against the current reference before adopting a parallel measurement path.

| Reproduction case | Owner baseline / purpose |
| --- | --- |
| 10 / 15 / 20 minutes at 48 kHz | Preview measurement approximately 17 / 22 / 35 seconds. |
| 30 minutes at 48 kHz WAV | Long wait, without a supplied numerical timing. |
| 60 minutes at 96 kHz | Approximately 3 minutes 30 seconds measuring; rapid track switching occasionally produced a recoverable error. |
| 3 minutes at 192 kHz | Approximately 22 seconds measuring after each settings edit; EQ could already be audible. |
| 10 minutes at 192 kHz | Rapid A/B with many active settings produced substantial dropout/lag but no timeout. |
| 2 minutes at 384 kHz | `TEST-2min-STEREO-384khz.wav`; owner reports increased lag, never a timeout. |
| 1 minute at 705.6 kHz | `TEST-1min-STEREO-705khz.wav` (header confirms 705600 Hz); increased lag, never a timeout. |
| 1 minute at 768 kHz | `TEST-1min-STEREO-768khz.wav`; increased lag, never a timeout. |

Start with the **3-minute/192 kHz**, **10-minute/192 kHz**, and **60-minute/96 kHz** cases plus a short 48 kHz control. Use the other durations to characterize scaling if needed; do not run an enormous full cross-product before learning anything.

Include the three extreme-rate fixtures as a separate stress extension. `decode.rs::validated_sample_rate` rejects zero but has no upper rate ceiling. That code fact and the owner's successful imports do not prove every downstream DSP/device configuration safe at arbitrary rates. Do not add an import ceiling as a substitute for diagnosing the reported lag; retain the successful imports while measuring resource use and checking downstream behavior.

Record:

- Build profile, device rate/buffer, source duration/rate/channels, settings, cold/warm state.
- Import/analysis completion, decode/prewarm time, first audible playback, audible response to settings, VM readiness, and whole-track Preview LUFS readiness separately.
- Audio callback time relative to its actual buffer deadline, underruns/gaps, command latency, peak memory, and active/pending worker counts. Keep any diagnostic instrumentation temporary or dev-only.
- No-match mode, VM only, Preview LUFS only, and any combined state actually supported by the UI; test Original and Mastered.
- Normal A/B cadence separately from deliberately rapid clicking; test quick track switches during analysis and preview measurement.

**Current implementation baseline, open to improvement:** Whole-track landing runs off-thread; VM returns its representative-window result first. Same-source work has one active worker and one latest pending edit. Source epochs reject stale results. Prewarming can avoid synchronous first-play decode, but the September 4 ledger explicitly records synchronous decode as a remaining fallback. Preserve responsive audition, correct loudness, stale-source isolation, and bounded work; the particular worker/cache structure may change. These existing mechanisms do not prove callback deadlines or globally bounded work across repeated source changes.

**Deliverable:** A compact baseline report identifying which measured stage accounts for each symptom. Do not infer CPU behavior from a spinner or assume all old worker jobs stop merely because their results are ignored.

### 2B. Correct the demonstrated bottleneck, one change at a time

**DSP exploration is explicitly in scope.** The agent may inspect the full preview/render/measurement path, compare alternative algorithms and data layouts, vectorize suitable arithmetic, improve limiter/compressor execution, eliminate redundant passes, and prototype bounded parallel work where dependencies permit it. This is not limited to caching or UI fixes. Tie experiments to a measured cost, compare before/after on the same fixtures, and land only the smallest proven improvement. Lower CPU use, memory use, and callback cost are all valid improvements.

Optimize the implementation while preserving intended sound and numerical contracts. Preserve exact output where feasible; if operation ordering changes floating-point results, quantify the delta against existing tolerances, null/difference comparisons, and independent loudness/true-peak references. Do not silently lower precision, weaken peak protection, change release/knee behavior, discard whole-track evidence, or rewrite preset voicing for speed. A deliberate sonic tradeoff is a separate owner decision, not forbidden research: bring the measured benefit and audible/numerical tradeoff to the owner before adopting it. Known mechanical correctness repairs remain in scope with regressions and targeted listening where sound changes.

Choose changes from evidence, for example:

- Eliminate avoidable decoding, buffer copies, or controller work on ordinary same-source A/B.
- Keep obsolete source work from consuming resources indefinitely; bound/cancel/coalesce work where profiling demonstrates accumulation, with correct cleanup and source ownership.
- Correct cache identity or unnecessary invalidation only when two requests are actually equivalent. Do not reuse a measurement for changed processing.
- Protect playback against measured contention among analysis, decoding, preview measurement, and live DSP. A user-visible change to analysis scheduling/loading belongs to the discussion below; do not smuggle it into a performance patch.
- Reduce measured full-chain or measurement cost while preserving the September 4 whole-track loudness contract and delivered-file measurement accuracy.

**Do not:** restore synchronous VM computation, use a short excerpt as if it were exact whole-track Preview LUFS, hide a pending measurement, simply lengthen timeouts, enable gated processing, or retune presets to make a benchmark cheaper.

### Acceptance

- Same-fixture before/after evidence identifies the improvement and shows that the work was not merely moved into another wait.
- On the documented native reference setup, normal A/B and settings editing remain responsive during a high-rate stress run (start with **five minutes**, then adjust to the observed failure window and explain coverage), with no observed task-induced dropouts or callback deadline misses attributable to the corrected path. If this target is not met, report the remaining failure; do not call performance fixed.
- Extreme clicking has bounded work and converges to the last requested source/state without a stuck transport, stale playback, or an orphaned pending measurement. Record residual audible discontinuities rather than claiming perfection from “no crash.”
- Source changes, cancellation/failure, and late results cannot poison the selected track or leave persistent error/pending UI after recovery. Distinguish valid recovery feedback from stale errors.
- Add durable error capture before repeating the intermittent rapid switch into the 60-minute/96 kHz track. On the **next occurrence**, write the complete original error text/cause to the existing diagnostic log together with timestamp, selected/requested track identity, source rate/duration, playback/VM/Preview LUFS mode, and request/generation identifiers where available. Preserve the event after the toast clears and playback recovers. Test the logging with an injected error; do not require a reliable natural trigger before adding this evidence path, and do not claim the original message was recovered until an actual occurrence is captured.
- Warm equivalent requests reuse correct work; settings that change the measured result still remeasure. Final settled Preview LUFS/export agreement remains within existing tested tolerances.
- Keep wall-clock targets in native benchmark evidence rather than flaky CI assertions. Mechanical tests cover bounded work, state correctness, and cancellation. The stronger real-hardware requirement is not satisfied by an offline throughput example or browser mock.
- If only one powerful Windows machine is available, state that scope. Validate on Mac and a more constrained setup when available before making broader responsiveness claims; do not request hardware specs the tools can observe.

### Analysis/loading decision after the baseline

**Important finding:** `analyze_tracks_core_with_progress_sync` already loops through tracks sequentially **within one batch**. The command populates the profile store after the batch, and the frontend applies the returned results together. Multiple overlapping batches or other jobs may still run concurrently. Thus the observed “all analyzed at once” appearance does not prove a per-batch parallel analyzer.

The choice is **when each completed track becomes usable and how remaining work shares resources**:

| Option | Benefit | Cost / evidence needed |
| --- | --- | --- |
| Make completed tracks usable sooner, protect playback while the rest prepare | Earlier useful work | Requires incremental result/profile publication, truthful per-track progress, correct cancellation/failure handling, and proof that background work does not impair audition. |
| Complete the batch before opening the editing experience | Clear preparation stage and predictable entry | A potentially long up-front wait; it does not remove later Preview LUFS costs after settings edits. |

**Owner decision (2026-09-05 follow-up interview):** Make each completed track usable immediately after analysis/profile readiness, show progress for the remaining tracks, and prioritize playback over background work. The owner accepted this recommendation. Verify concurrent analysis keeps audition responsive; implementation and this evidence remain pending.

The requested discussion has settled this workflow. Measure time-to-first-usable-track, total batch time, and playback impact during implementation. Preserve per-track backend profile readiness, not just frontend analysis text. Later settings changes can still incur Preview LUFS waits.

**Automatic Preview LUFS owner decision (same follow-up interview):** If Preview LUFS is already enabled for the selected mode, start measurement when the selected track becomes ready. Show "Measuring," cancel obsolete work when switching tracks, and protect playback with bounded work. Other imported tracks wait until selected; do not queue whole-track measurements for the batch. Enabled state/defaults are unchanged. Verify resource contention, source/settings lifetime, and cancellation during implementation.

## 3. Width and Loud: separate display behavior from sound

### 3A. Width Auto readout flicker

**Owner evidence:** Other settings edits make Width alternate between “Auto” and “Auto · [resolved value].”

**Code lead:** The guardrail-readout effect clears `guardrailReadout` for each settings dependency change, then restores it after an asynchronous reply. `AdvancedPanel` uses that readout for Width's resolved Auto value. This is a strong explanation for the display transition, not a reason to remove all invalidation: clearing stale processing readouts protects truthful signal-chain feedback.

**Change scope:** Stabilize the Width presentation and slider position through recalculation without presenting an old value as the current resolved result. Keep any last-known value explicitly scoped/identified while updating; clear it on source/context changes. A delayed response must not roll a manual value back or replace a newer result. Avoid broad changes to all guardrail displays just to repair one readout.

**Acceptance:** With slow and out-of-order mocked replies, repeated settings edits do not flash the Width control between incompatible states or jerk its thumb to a fallback minimum. Auto remains distinguishable from an explicit number. Track/Album/source changes cannot show another context's numeric result. Keyboard entry, drag, reset-to-Auto, and actual native readout behavior remain correct.

**Consistency recommendation:** Keep useful resolved information for Width. Audit the meaning of the other controls before standardizing labels; do not add invented “Auto” values to controls whose neutral/default behavior is different. Record any broader presentation choice separately from the flicker fix.

### 3B. Width processing and Loud compression/meter checks

Reuse the existing Width unit tests and full-chain/fingerprint fixtures first. They already cover zero-to-mono, identity at 1, side amplification, pure-mid preservation, mono handling, clamping, and full-chain side preservation. Add only missing behavior-driven coverage.

**Width matrix:** mono, centered stereo, side-rich stereo, anticorrelated material, and asymmetric left/right content; Auto → manual → Auto, representative widths including bounds, at 44.1/48/96/192 kHz as relevant to a finding. Check the control → resolved coefficients → live audio → saved output path. Respect the intended M/S behavior: width 0 intentionally removes side content. Separate source-dependent guardrails from a broken slider.

**Loud matrix:** transient material, sustained/dense material, and asymmetric channels; approximately 50/90/100% Intensity; Preset versus Off creative compression; Preview LUFS/VM conditions relevant to the complaint. Check gain reduction, sample/true peak, actual L/R output envelopes, and what each live meter displays. Do not assume equal-looking meters prove channel collapse, or mistake peak hold/ballistics for a stopped audio engine.

**Acceptance:** Tests either expose a specific violated DSP/meter contract and accompany its correction, or the agent records that no objective defect was found within the named cases. Existing bounds and tolerances remain intact. No speculative tuning is performed to make Loud less limited at extreme Intensity or Punch brighter at 0% Density.

The Punch 0%/50% Density preference and Loud “breathes more lower down” observation remain valid taste notes. A correct, aggressive setting is allowed. Any proposed voicing change needs a later owner listening decision, not an automatic patch from this plan.

## 4. Loop anomaly and Standard region presentation

Split the two reports.

**A. Unreproduced start-position anomaly:** switch tracks → draw a loop region → Play → toggle Loop. The owner says playback did not begin at the region or beginning and could not reproduce it. Do not invent “no playback at all” as the exact symptom.

Start with a bounded sequence of **20 track/region/play transitions**; adjust the sequence/count to evidence and record the actual coverage, including while analysis/prewarm is unfinished, both source modes, near-end regions, and rapid selection changes. Inspect command ordering, stale play requests, pending loop region, and backend/source identity. Use native behavior as well as hook tests. If reproducible, isolate the smallest failure and add a regression. If still unreproduced, preserve the sequence and attempt count as an open observation without demanding that the owner reproduce it again before other work continues.

**B. Region remains visible in Standard:** Existing code intentionally disarms looping on entry to Standard while retaining per-track region memory for return to Advanced. Existing tests pin this. Standard passes that region to the waveform with region editing disabled.

**Proposed correction if the visible residual selection is confirmed:** Hide/de-emphasize the inactive loop selection in Standard while preserving its remembered geometry for Advanced. Do not delete the region memory or enable hidden looping to reconcile the UI.

**Acceptance:** Standard neither loops nor accepts the Advanced loop shortcut; its waveform does not suggest an active loop. Returning to Advanced restores the selection but requires explicit re-arming. Track changes cannot carry a previous source's loop or seek. Any change to start-position behavior follows an established transport contract, not a guess about the unreproduced event.

## 5. Export presentation and small UI corrections

### 5A. Album completion and receipt

The receipt already has delivered loudness, true peak, target, and ceiling rendering; it is placed in the Sidebar and details are collapsed. The owner found the result hard to discover. **Improve the completion experience; do not rebuild a measurement feature that already exists.**

Proposed behavior: after a successful Album export, present a clear completion summary and visible access to the per-track results beside the relevant export workflow, using the existing report. Give the destination a clear reveal/open action if supported. Keep one primary receipt location rather than duplicating an entire report in two places. Preserve receipt access after dismissal/navigation and avoid unnecessary focus theft.

**Acceptance:** A successful native export exposes its result without hunting in a lower-left dropdown. All four per-track metrics remain truthful, including missing/legacy values and informational target shortfalls. Keyboard/screen-reader users can reach and dismiss the result. Cancelled or failed exports cannot show a success receipt. Verify against actual exported files when testing correctness, not mock receipt numbers alone.

The owner supplied `E:\fghgfhjghjhg` for inspection if useful. Read existing outputs without altering them. Do not declare that the original album missed receipt fields unless inspection establishes that fact.

### 5B. Auto labels and tiny right-rail scroll

- **True mono Source Insight:** `src/lib/source-insight.ts` currently buckets `analysis.stereo_width` without channel count, so a one-channel file can read “Narrow — Mono-leaning stereo image.” Branch on a known source channel count of **1** before the width buckets and show **“Mono”** with a truthful one-channel note. The backend PCM has channels, but current Rust/TypeScript `AnalysisResult` does **not** carry them; `ImportedTrack.channels` already does. Prefer wiring that matching track metadata through `SourceInsight`/`sourceInsightRows` rather than assuming `analysis.channels` exists or expanding a shared wire contract unnecessarily. Unknown channel count must not be inferred from low width or absent correlation; retain existing stereo classifications for two-channel files, including dual mono. Regression: `TEST-3min-MONO-48khz.wav` reports Mono; the other true mono fixture agrees; existing stereo fixtures and width buckets are unchanged. Use a focused unit regression plus real import/UI verification, not a private-audio-dependent CI test.
- Align Track/Album LUFS target and ceiling presentation where their semantics agree; display actual effective values and distinguish inherited/automatic from explicit settings. Do not change settings just to make labels match.
- Reproduce the small scroll with the current fully expanded Advanced rail after accounting for concurrent CSS/tooltip changes. Inspect real overflow causes before changing spacing.
- At normal desktop sizes where the content can fit, remove incidental few-pixel overflow without hiding controls, shrinking text excessively, or using `overflow: hidden` as a concealment fix.
- At the supported **1360×740** minimum, shorter windows, and enlarged text/zoom, retain intentional scrolling whenever content actually needs it. The owner asked to remove accidental scrolling, not make an arbitrarily tall expanded panel fit every viewport.
- Keep delivery-format copy accurate while WAV is the only implemented output. Removing or relocating repetitive copy can be a layout choice; do not imply MP3 already ships.

**Acceptance:** Track and Album states with all relevant panels expanded have reachable controls, visible focus, no accidental clipping or nested scroll trap, and truthful values. Check 1360×740 plus a larger desktop viewport and 200% zoom. Use actual pointer/keyboard reachability and screenshots, not only DOM existence or auto-scrolling clicks.

### 5C. Album names and manifest: define the intended output change

Current implementation deliberately writes **`NN-<source-stem>.wav`** inside a collision-safe album folder, plus continuous audio and **`manifest.json`**. The existing owner decision selected an album-titled subfolder. The lack of “mastered” is therefore a requested naming improvement, not proof that a suffix routine broke.

**Owner filename decision (follow-up interview):** Track `Song_mastered.wav`; Album `01-Song_mastered.wav`, with one underscore. Number Album files in final left-rail order at export. Code inspection confirms this ordering already flows through the track array, plan inputs, one-based positions and filenames. Implement the new default suffix with collision/source protection; do not rename existing exports.

**Manifest owner decision (follow-up interview):** write the manifest in a **`metadata/` subfolder**. The owner accepted the recommendation, with a concern about users receiving a file they do not understand. Explain supporting export details in the in-app receipt, including that they are not needed to play or share the audio. The alternative of removing JSON and persisting only an in-app receipt was not selected. Implementation remains pending; existing owner exports are untouched.

**Acceptance after a choice:** For `metadata/`, update the actual returned path, folder creation, manifest references, cleanup, and receipt links. For removal, stop emitting new JSON and remove the assumption of a mandatory manifest path throughout report types, completion logic, UI, tests, and affected bridges; keep the in-app report usable and make its persistence/reopen behavior explicit. Neither option deletes old owner outputs. In both cases, continuous-file assembly, cancellation/failure cleanup, and collision-safe non-overwrite behavior remain correct; legacy receipts/projects stay readable. Keep this independent of a DSP or codec change.

## 6. Extra export formats: a separate feature specification

The owner requests **MP3** as a smaller-file, broadly playable choice within the normal mastering export. **The follow-up interview supersedes the earlier interpretation of WAV → MP3 without mastering:** no separate converter, Original-source selector or bypass mode is wanted. Preserve existing DSP/delivery behavior at 0% Intensity and do not add special workaround explanations. Retain WAV and add MP3; Track/Album scope and quality defaults remain unresolved.

Before implementation, write a bounded specification covering:

1. Track versus Album availability, MP3 quality controls/default, and how format selection affects Standard's currently fixed delivery promise.
2. Keep MP3 on the normal mastering export path, independent of the audition A/B side. Preserve 0% Intensity and delivery behavior; no Original-source export choice or new bypass semantics.
3. Encoder choice and distribution on Windows/Mac, installation/offline behavior, and applicable packaging/redistribution requirements. Verify current primary documentation when choosing the dependency; this plan has not selected one.
4. Extension/filter handling, supported channel/rate conversion, output destination safety, cancellation, failure cleanup, and persistent project/settings compatibility.
5. What the receipt measures. MP3 can add padding/delay and change decoded peaks. Do not label pre-encode PCM figures as measurements of the delivered encoded file. Define and verify the delivered-file checks and user copy.

**Acceptance:** A real MP3 opens/decodes in an independent player; requested format/quality, duration accounting, channels/rate, and receipt semantics match the implementation. MP3 uses the normal mastering path with existing 0% behavior. Existing WAV exports, levels, receipts, and non-overwrite behavior remain correct. Test Track and Album only for the surfaces included in the agreed increment, and do not imply unavailable coverage.

The extra-format request conflicts with current WAV-only/fixed-Standard product descriptions. When its scope is settled, update the corresponding internal `PRODUCT.md`/behavior/help documentation as part of that authorized decision; ask only about unresolved product choices. Public claims must match working, verified functionality and publication authorization. This feature need not delay the objective corrections in steps 1–5.

## Decisions that need an informed discussion

These are genuine product choices, not missing answers from the listening form. They do not block reproducing the documented bugs.

| Decision | Recommendation to bring to the owner | Evidence needed first |
| --- | --- | --- |
| When imported tracks become usable — settled | Owner selected incremental readiness, progress for remaining tracks, and playback priority | Implementation must establish time to first usable track, total batch time, and playback/resource impact. |
| Automatic Preview LUFS on import — settled | Owner selected automatic measurement for the ready selected track when already enabled, "Measuring" state, obsolete-work cancellation and playback protection | Implementation must verify resource contention, source/settings lifetime and cancellation; no whole-batch render queue. |
| Album output naming/JSON presentation — settled | Owner selected **metadata/ subfolder** with a plain-language receipt explanation, Track `Song_mastered.wav` and Album `01-Song_mastered.wav` in left-rail order | Implement and verify ordering, default names, returned paths, links, cleanup and non-overwrite behavior. |
| First codec scope — purpose settled | WAV + MP3 in normal mastering export; no separate converter/bypass. Track/Album availability and quality defaults remain open | Short specification including Track/Album/Standard scope, actual encoder feasibility, receipt semantics, and product-doc consequences. |

For routine bug-fix details, use engineering judgment within the authorized scope. Do not turn every test or small UI correction into an owner approval. For the analysis/loading workflow, the request to discuss before changes comes from the owner's written note, not an invented process gate.

## Verification and evidence contract

Do not run a full suite merely to mark this planning document done. During implementation, use focused regressions to establish each defect and run the repository-required checks for the affected scope.

### Required implementation lanes

Use the [verification scope matrix and commands](../TESTING.md#choose-verification-by-scope). Run focused regressions during iteration and affected suites at coherent checkpoints. Installer builds belong to packaging changes or final desktop integration; full Rust tests already include library tests. Do not rebuild every platform before every small commit.

For rendered UI changes, run **`npm run verify:headless`** before calling the UI slice complete. If a captured application surface changes, regenerate the affected deterministic landing assets/manifest through the existing pipeline so the source-digest gate remains honest. Preserve concurrent capture work; do not hand-edit hashes to silence the gate.

Before merging DSP/export or audition-trust changes, run the documented slow fixture lane with available local fixtures. Reuse the supplied fixture set; do not regenerate or relocate it merely to match a storage convention.

When shared types, commands, or shared behavior affect the bridges, include the documented iPhone **check + tests** and Android **host tests + arm64 API-29 cross-check** before integration. Desktop-green alone does not establish bridge compatibility. Required commands and prerequisites remain in `docs/TESTING.md`.

Passing tests are not evidence of native sound quality, low callback latency, or a real saved-file comparison. Record those separately. If local fixtures, a platform, or hardware evidence is unavailable, name that exact limit instead of marking the entire mission failed or the slice universally proven.

### Per-change record

Each implementation result should state:

- Owner observation addressed and exact build/configuration.
- Reproduced behavior or bounded unreproduced outcome; confirmed cause only when demonstrated.
- Small change made and why it addresses that cause.
- Regression failure before/fix pass after; required broader checks.
- Native audio/UI or saved-file evidence where relevant, including before/after performance figures.
- Remaining limits, related untouched preferences, and the next bounded action.

When implementation begins, maintain the appropriate current queue/evidence documents in small commits. Distinguish **owner listening evidence**, **local mechanical evidence**, and **exact pushed-commit CI evidence**. Only claim remote CI for the SHA actually checked; no release tag or public activation follows automatically from this work.

## 7. Final focused owner check

After the objective changes are verified, batch any needed listening into one short session focused on changed behavior:

- The Volume Match setting-change jump and ordinary A/B regression.
- Long/high-rate responsiveness on the previously troublesome actions.
- Width/Loud only where a mechanical correction changed sound or where the owner wants to judge the reviewed behavior.
- A saved-file comparison with audition VM explicitly off if that remaining protocol is needed, plus the specific Album checks still untested if Album evidence is being closed.

Do not make the owner re-answer the entire guide. Carry forward the September 5 passes as evidence for their original build and conditions; add targeted regression evidence for the new build rather than claiming old listening automatically approves new bytes.

Leave preset taste changes, adaptive-compressor/Phase-B/album-character activation, mobile expansion, release activation, and promotional work outside this correction pass. The pre-existing small Custom-receipt target and 0.2 LU LRA questions remain in their current queue; they are not newly proven audio defects from this session.

## Code map for the implementing agent

Line anchors reflect the inspected working tree and can move. Reconfirm functions before editing.

| Concern | Starting points |
| --- | --- |
| VM fallback and coefficient publication | [audio.rs](../../src-tauri/src/audio.rs#L1476) — `apply_preview_volume_match_gain_cached`, `publish_preview_coeffs`, worker/result handling. |
| Preview worker lifecycle and full-track work | [audio.rs](../../src-tauri/src/audio.rs#L1539) — `PreviewWorkerGate`, `try_spawn_lufs_preview_worker`, source epochs, prewarm/play paths. |
| Sequential batch analysis and result publication | [engine.rs](../../src-tauri/src/engine.rs#L86) and [useTrackMaster.ts](../../src/hooks/useTrackMaster.ts#L1383). |
| Width readout lifecycle | [useTrackMaster.ts](../../src/hooks/useTrackMaster.ts#L1075), [AdvancedPanel.tsx](../../src/components/AdvancedPanel.tsx#L353), `fields.tsx`. |
| Width/Loud DSP and existing probes | [dsp.rs](../../src-tauri/src/dsp.rs#L1454), `tests/audio_invariants.rs`, `tests/preset_fingerprint.rs`, `tests/realtime_meter.rs`. |
| Loop disarm/selection presentation | [useTrackMaster.ts](../../src/hooks/useTrackMaster.ts#L2518), `App.tsx`, `StandardView.tsx`, `Waveform.tsx`; existing hook integration tests. |
| Album receipt placement and fields | [App.tsx](../../src/App.tsx#L517), [AlbumExportReceipt.tsx](../../src/components/AlbumExportReceipt.tsx#L12), Sidebar and album export tests. |
| Album naming/manifest and output safety | [album_render.rs](../../src-tauri/src/album_render.rs#L854), album-render/non-overwrite tests. |
| Existing export promises and verification | `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`, `docs/TESTING.md`, `docs/plans/2026-09-04-audio-correctness.md`. |

## Next agent's first action

Read this plan and the reconciled handoff, refresh the relevant repo instructions/state, and—when asked to implement—start with **steps 0 and 1**. Establish the warm Volume Match → settings edit reproduction and land a coherent verified correction, then continue the authorized plan through checkpoints without waiting for routine permission. Respect any narrower scope or explicit stop in the initiating request. Pause only work dependent on unresolved owner choices; continue independent corrections. Do not re-run the questionnaire, retune presets, or start the MP3 feature before its scope is settled.
