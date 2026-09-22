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
