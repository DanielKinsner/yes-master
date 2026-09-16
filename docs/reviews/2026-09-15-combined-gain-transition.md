# Device correction and Volume Match: combined transition

## Diagnostic fixed before correction

Continue B3's dynamic-boundary work using the actual production `GainSource`.
Two valid factors can change inversely while their total level stays the same:
device correction 0.5 / Volume Match 1, then device correction 1 / Volume Match
0.5. Independently interpolating both factors can produce an intermediate gain
of 0.75 × 0.75 = 0.5625, exceeding either endpoint's 0.5 gain. This is an objective
transition issue; no preset or preferred loudness needs to be chosen.

Probe stereo constant PCM at 44.1/48/96 kHz, both uninterrupted and interrupted
one quarter into the ramp by another equal-total pair (0.625 / 0.8). Require
matching stereo samples, final requested revision and the existing **0.01 dB**
equal-output transition limit throughout. Preserve the initial failure log.
This isolates the downstream envelope; it does not establish that every whole
song or device produces those exact correction/Volume Match pairs.

A correction must retain exact settled two-multiply f32 PCM, matching revision
and converter-drain rules, attenuation while raw output is pending, interrupted
ramp continuity, seek behavior and callback allocation bounds. Validate the
focused tests, desktop suite/fixtures and actual muted callbacks. Static file
finalization and current source/prepared waveform evidence remain reusable only
if their code and settled arithmetic are unchanged. No export gain, saved setting,
Volume Match default or preset calibration change is intended.

## Result and correction

`b3-combined-gain-before-v1.log` preserves all six failures: uninterrupted
compensation rises **1.0230507 dB** above its equal endpoints; the interrupted
case rises **0.7762-0.7784 dB**. The corrected gain stage interpolates the combined
gain and represents that envelope as a valid factor pair for pending attenuation
and interrupted ramps. Its final frame still applies the exact requested two
f32 multiplications in the original order. Converter/revision matching, ramp
duration, source landing, export processing and saved settings are unchanged.

All six corrected cases deviate by at most **6.472e-7 dB**, well inside the same
0.01 dB limit (`b3-combined-gain-after-v1.log`). Additional zero-gain transitions
are finite, monotonic, stereo-consistent and allocation-free, with exact settled
PCM and final revision. Existing concurrent publication, raw-revision wait,
small attenuation, seek and allocation regressions pass.

Strict all-target Clippy and the full desktop suite pass: **507 library tests**,
20 opt-in tests ignored, all integration tests and **all four restored private
fixture tests executed**. Logs are `b3-combined-gain-{clippy,fixtures}-v1.log`.
No frontend, shared bridge command/type or bridge live-stream implementation
changed. Their prior scope remains separate; this is fresh desktop/native proof,
not a fresh physical-mobile, installed-app or Mac verdict. The existing 16-file
whole-source verification remains applicable to unchanged settled PCM; no frozen
output was replaced or relabeled.

## Actual native callback and lifecycle checks

The copied tested executable has SHA-256
`236853cc9338ca31dce0e75eb076503b8df1f05facc633c3f0ec72fc366b8bb4`.
Both muted native probes run source/file/device **44.1/96/48 kHz** with concurrent
whole-file preparation. The new probe sends 1,200 compensating scalar edits at
5 ms intervals, shorter than the 10.67 ms gain ramp; its DSP coefficients stay
fixed. The existing probe separately sends 120 EQ edits at 50 ms intervals.

| Probe | Callbacks | Median / p95 / maximum processing | Deadline misses / device or converter errors / exhausted samples |
| --- | ---: | ---: | ---: |
| 1,200 scalar edits | 655 | 0.0473 / 0.8874 / 1.7740 ms | 0 / 0 / 0 |
| 120 EQ edits | 606 | 0.0460 / 1.0544 / 1.8443 ms | 0 / 0 / 0 |

The device again grants **480-1,056 frames**, despite a 256-frame request.
These are observed native callbacks, not a locked-256 guarantee. First meters
arrive in 10.47/10.13 ms; preparation cancellation joins in 2.72/2.62 ms.
Evidence: `b3-combined-gain-callback-v1.json` and
`b3-combined-gain-eq-callback-v1.json`. Different edit loads do not establish a
before/after speedup, and neither probe supplies a listening verdict.

The actual AudioPlayer lifecycle also passes playing/paused rate changes,
seek/resume, A/B playhead continuity and prepared versus applied revision checks:

| Operation | Preparation-ready observation | Matching output revision emitted |
| --- | ---: | ---: |
| First target | 44.962 s | 45.142 s |
| New -23 target | 2.010 s | 2.244 s |
| Cached -14 target | 75.44 ms | 190.56 ms |

Cold playback accepts in 226 ms, first meter in 284 ms. A paused cached edit stays
unapplied until resume, then acknowledges in 184.17 ms. The separate process
observer records **48.741 s total wall / 47.516 s last observed CPU**, **757,518,336
bytes peak working set** and **748,892,160 bytes maximum observed private memory**.
Evidence is `b3-combined-gain-native-v1.json` and its `.resources.json` companion.
Initial preparation, settings settling, actual output application and whole-session
resource cost remain separate. No hardware/DAC latency or controlled speedup claim.

This closes the identified combined-scalar bump. Arbitrary DSP changes, seek/fade
boundary reconstruction and final-result UI remain separate open B3/C3 work;
the regression does not prove a ceiling for every time-varying transient.
