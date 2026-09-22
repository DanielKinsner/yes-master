# Preset preview delay, transition hiccup and loudness controls

Investigation only, 2026-09-22. The owner reported about 40 seconds before a
new preset's playback volume settled, a brief hiccup near completion, and
confusing live meter/target/profile values. No processing or UI implementation
is changed by this investigation.

## Evidence and limits

- Inspected source: `b17aa228d5d5a1c7d9ee7ee32354a1e7897d8b81`.
- Installed Windows executable SHA-256:
  `54c60536be9b89193ca5c4edb9206d53b732f0c01cff7768729c61a454b17041`.
  This matches the existing installation record and the session log's build stamp.
- Owner-specified source: `Downloads/It’s a coat (5).wav`, 252.36 seconds,
  48 kHz stereo. SHA-256:
  `596506d94f89bbc4cfe03e860745a3b97e932c7512d1d7e9844a3674acdb37c0`.
- The installed log contains matching 12,113,280 source/device frames and
  24,226,560 frames at 96 kHz. The saved selected track uses 96 kHz/24-bit,
  -11 LUFS, Auto ceiling (-1 dBTP), Volume Match off and Adapt 100%.
  Saved settings are Clarity at 85%; the screenshot includes Tape. Neither the
  exact preset pair nor its click times are logged, so this is not an exact
  reproduction of the reported transition.
- Local evidence: ignored `test-output/preview-preset-investigation-20260922/`
  contains the scoped installed log and `investigation.json` with selected
  settings, source identity and parsed timings. No private audio is copied into git.
- Fresh focused frontend verification: 63 tests passed across
  `effective-settings`, `settings-transitions` and `RightRail`.
- No new native callback capture, full-file render or listening test was run.
  The screenshot alone cannot certify output loudness or true peak.

## Why the new volume takes so long

The preset coefficients update promptly; the final preview gain waits for a
whole-track background calculation. `publish_preview_coeffs` preserves prior
attenuation while the new result is pending. The worker renders the actual
processing, prepares the requested delivery-rate PCM, measures it, prepares and
measures the device-rate PCM, then verifies the gain-applied delivery.

Three installed-session cold preparations at 96 kHz show:

| Completion (UTC) | File preparation | Device preparation | Final checks | Stage total |
| --- | ---: | ---: | ---: | ---: |
| 15:35:46 | 22.440 s | 8.306 s | 2.034 s | 32.780 s |
| 15:36:30 | 21.232 s | 8.280 s | 1.956 s | 31.468 s |
| 15:37:28 | 19.540 s | 9.057 s | 1.986 s | 30.584 s |

These stage totals explain most of the reported delay. They exclude the
unlogged click-to-worker interval, the preliminary eight-second Volume Match
window calculation, cancellation/queue time and final application latency.
They do not prove an exact 40-second click-to-audible interval.

Earlier 48 kHz cold rows total 18.951–21.544 seconds. Three rows with both
prepared buffers already cached total 0.956–0.979 seconds. These are different
settings/events, not controlled sample-rate or before/after benchmarks.

Relevant code: `audio.rs::try_spawn_lufs_preview_worker`,
`audio/preparation_cache.rs::measure_device_with_cache`,
`engine.rs::prepare_preview_audio`, `device_preparation.rs::PreparedDevicePcm`.

## Optimization priorities that preserve the audio contract

1. **Share identical PCM measurements.** When delivery and device rates match,
   `PreparedDevicePcm::new` copies identical PCM and calls the same full peak/LUFS
   preparation again. Reusing facts bound to that exact immutable buffer can
   remove duplicate work without changing samples, coefficients or measurement
   precision. The observed device-preparation stage is substantial, but its
   full duration is not yet a measured saving. This shortcut does not apply
   to the screenshot's 96-to-48 kHz conversion.
2. **Profile and parallelize independent measurement work with a fixed budget.**
   Qualified peak preparation runs finite-signal and reconstruction-filter
   passes serially, followed by LUFS. Independent channel/filter work is a
   candidate for bounded parallel execution while retaining each calculation's
   order and results. Benchmark during playback; additional worker CPU must
   not compromise callback deadlines. No speedup is claimed yet.
3. **Retain reusable processing stages.** Current prepared PCM keys include
   delivery rate, although the source-rate nonlinear response does not depend
   on that rate. Separate source-response reuse from delivery/device conversions
   under the existing memory bounds. At this source's size the 96 kHz response
   plus retained source occupies about 291 MB, nearly the 288 MiB file-cache
   budget, so only one such response fits. Exact completed gain results already
   have their own per-track cache; simply adding another preset cache is not
   a new solution to cold preparation.
4. **Avoid unnecessary work at the gain handoff.** A final file-gain update is
   represented as new `ChainCoeffs`, triggering two full DSP chains during the
   512-source-frame crossfade even when processing is otherwise unchanged.
   Investigate a dedicated gain-only ramp at the same pre-conversion position.
   Preserve the existing compensated-edit behavior and converter rounding
   checks; moving the gain wholesale after conversion would repeat a previously
   exposed transition problem.

Do not promise immediate exact whole-song loudness for a previously unmeasured
preset. A short-window estimate could respond sooner, but would be provisional
and would introduce a different preview accuracy contract. It is not needed
to begin the exact-result optimizations above. Preset voicing, limiter quality,
full-file verification and qualified peak bounds remain the acceptance baseline.

## Hiccup: unresolved, with a specific evidence gap

The final result publishes gains/coefficient revisions; it does not intentionally
pause/restart playback. The source inherits DSP/limiter state and crossfades for
512 source frames (10.67 ms at this source's 48 kHz). Extra callback work during
that transition is a plausible lead, not a demonstrated dropout cause.

Existing `native_callback_bench.rs::callback_bench` runs about six seconds of
edits and then cancels its background measurement. It does not wait for this
30-plus-second preparation to complete and does not apply that worker's final
result. Previous zero-miss runs therefore do not cover the reported event.
The separate lifecycle test waits for preparation/application, but does not
record per-callback deadlines or emitted sample continuity at that point.

The next diagnostic should combine those capabilities: the exact source and
resolved preset settings, the production 48-to-96-to-device route, continuous
callback timing and pre-mute PCM capture through cold preparation, final gain
publication, and several seconds afterward. Correlate the worker-complete,
requested-revision and applied-revision times. Compare short output windows
against the uninterrupted reference to distinguish a real output gap from a
gain transient or meter behavior. Keep output muted after processing during
automated device tests; an installed listening confirmation is separate.

## What the screenshot's numbers mean

The current Advanced Master Out order is:

| Screenshot value | Quantity | Scope |
| --- | --- | --- |
| -9.3 | Momentary LUFS | Short current playback window |
| -9.4 | Since-play integrated LUFS | Audio heard during this playback run |
| -3.5 | Live sample peak, dBFS | Digital samples, not a true-peak receipt |
| -11.0 | Requested integrated LUFS | Whole finished track |
| -1.0 | Ceiling, dBTP | Maximum reconstructed peak target |

A louder section can legitimately read -9.3 LUFS within a track whose integrated
target is -11. The since-play measurement also retains earlier presets and the
unsettled preview interval; changing preset does not reset that meter. It is
not a measurement of the complete track under the newest settings.

The displayed -3.5 dBFS does not itself suggest clipping, but a sample-peak
reading does not certify a true-peak bound. An independently checked complete
render under the same resolved settings is needed to establish target accuracy.

`RightRail.tsx` supplies labels and explanatory titles, but the compact waveform
deck rule in `App.css` sets `.readout-label` to `display: none`. This explains
the unlabeled values in the supplied images. Restore readable visible names
and units, and make live/since-play scope apparent without requiring hover.

## Make the controls uniform in meaning and naming

The center loudness picker and Advanced LUFS knob already read the same
`effectiveLoudnessTarget` and use `applyExplicitLoudnessTarget`. The 63 focused
tests pass; no conflicting target-state defect was found.

The menus have different jobs:

- The center picker selects only a loudness target (-14, -11, -9, or Off).
- A Delivery Profile sets LUFS, ceiling, bit depth and sample rate together.
- Editing LUFS explicitly switches the profile to Custom while preserving the
  currently effective other delivery values. Thus Custom alongside -11 is expected.

Recommended presentation: one consistently named **Loudness target** value,
the same low/medium/high wording as Standard for its quick choices, and a clearly
named **Delivery preset** for the multi-setting recipes. Show the selected
recipe's target and ceiling together; a manual override should read as Custom
with those effective values still visible. Keep the complete delivery recipes
and arbitrary numeric LUFS available. Do not make the two dropdowns identical
by silently changing format/rate when the user intends only a loudness edit.

Any resulting UI slice needs rendered inspection and `npm run verify:headless`.
Any DSP/preview change needs meaningful transition regressions, PCM/measurement
comparisons, native completion-boundary evidence and the applicable fixture
lane before integration. This investigation does not claim those future gates.

## Follow-up: multi-core peak measurement (implemented)

Owner-approved the same day (option a of the whole-project review). Priority 2
above is implemented in `peak_meter` (`f34b80e7`); priorities 1, 3 and 4 and the
hiccup diagnostic remain open.

**Where the time went.** A scratch stage benchmark on the owner fixture
`Doors Open.wav` (209.5 s, 48 kHz stereo; 96 kHz file, 48 kHz device) measured
the cold preparation's parts: DSP 2.8 s, both SRC stages 0.4 s, LUFS 0.35 s,
FIR peak pass 4.8 s and the finite-sinc peak pass 20.7–22.9 s. Release LTO was
also tried: DSP ran about 10% faster with bit-identical output, but measurement
did not improve, so it is not part of this change.

**Change.** Both peak passes examine independent 4096-sample blocks and merge
them by maximum. Each channel's blocks are now split into contiguous ranges on
worker threads and merged in block order, so every reported value, including
refinement bookkeeping, equals the single-thread measurement. The
order-dependent refinement pass stays on the calling thread. Workers come from
one process-wide budget: logical cores minus two (reserved for the audio
callback, UI and OS), at most 16, shared by concurrent measurements. Short
signals stay single-threaded. In-memory and dithered delivery PCM share reads;
staged WAV readers open one reader per worker. Preview preparation, WAV and
encoded export receipts and Album measurement all use this path.

**Evidence (office Windows PC, 24 logical cores).**

- Production path before/after, alternating runs, owner fixture with Clarity
  85%, -11 LUFS, -1 dBTP, 96 kHz file and 48 kHz device: 30.0 s and 29.6 s
  before, 8.4 s and 8.6 s after. File gain, LUFS, device gain, true peak and a
  hash of the complete device PCM are identical before and after.
- Regressions: parallel equals serial exactly for stereo/mono/silent-channel
  signals that exercise refinement, uneven splits and more workers than blocks;
  staged 16/24/32-bit WAV workers equal serial delivery reads; ordered, complete
  block ranges; cancellation never returns a result; bounded shared budget. A
  deliberately misordered merge fails the ordering regression.
- Slow fixture lane (`AMS_RUN_REAL_FIXTURE=1 cargo test`, Doors Open): 721
  passed, 0 failed. Strict Clippy, rustfmt, iPhone (46) and Android (26 plus
  arm64 check) bridge lanes pass.
- Native callback bench (`mastering_quality_streaming_callback_bench`, muted,
  96 kHz live file-rate conversion, 48 kHz background preparation (scope
  corrected by the Mac follow-up below), 45 s excerpt of the same fixture so whole preparations run
  inside its window, Realtek default device at 48 kHz with 480–1056-frame
  callbacks): three single-thread runs (`YES_MASTER_PEAK_WORKERS=1`) and three
  budgeted runs each had 0 deadline misses and 0 device errors across 605–606
  callbacks. The worst callback used 14.5–19.5% of its time budget
  single-threaded and 18.6–20.4% multi-threaded (p99 12% versus 17%). The
  multi-threaded worker completed three preparations (1.2 s each) during
  playback, where the single-thread worker completed one (3.3 s).
- Local evidence: ignored `test-output/peak-workers-20260922/`.

**Limits.** No installed build, listening check or Mac timing is claimed. The
native runs used the Realtek device's large callback buffers; a low-latency
interface (for example the Focusrite at 128–256 frames) was not available on
this PC and remains a hand check. The bench measures preparation during
playback; it does not apply the final gain, so the completion-boundary hiccup
diagnostic above is unchanged and still open. Remaining serial time is mostly
DSP (2.8 s) and final verification (about 1.8 s).

## Mac follow-up: corrected benchmark scope

The resumed September 22 M4 check found that `YES_MASTER_CALLBACK_FILE_RATE`
changed only the live SRC route; the concurrent worker retained 48 kHz settings.
The helper now applies that rate to both paths and records each worker's rate.
This changes test instrumentation only, not application audio. The historical
Windows timings above remain valid for their actual workload, but are not proof
of 96 kHz background preparation or final gain handoff.

Six fresh alternating single-worker/budgeted runs on the M4's actual 48 kHz
speakers used requested and granted 256-frame callbacks, the same private 45 s
excerpt and verified 96 kHz worker settings. All 7,244 callbacks had zero
deadline misses, device errors or exhausted samples. Worst callback budget use
was 16.3–16.5% single-worker and 16.4–24.0% budgeted. Budgeted workers completed
two preparations per run (2.70–2.83 s each); the single-worker preparation was
still running when the six-second probe cancelled it. Do not report those
cancelled rows as completed preparation times or a measured speedup ratio.

A separate muted, full-source Mastered lifecycle completed rate edits, seek,
A/B, cold/new/cached landing and paused-cache resume. Its first 96 kHz whole-file
landing settled at 17.70 s and appeared in the emitted-output revision at
17.87 s; these are different events, neither DAC latency nor by-ear proof.
The local source is 206.72 s (SHA-256 `80b6974095a9d0ff838854498dd63a69866b06c1d02a27a10e6f30c86b3e49ca`),
not the 252.36 s Windows source. No controlled cross-machine speedup is claimed.
Evidence remains ignored under `test-output/mac-resume-2026-09-22/`. The
completion-boundary audible hiccup and Focusrite hardware checks remain open.
