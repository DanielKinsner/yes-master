# B3 live device-rate verification

Status: **new correctness failure reproduced; candidate correction unintegrated**.
Keep B3 open. This does not reopen the owner's completed baseline questionnaire.

## Reproduction

`audio::delivery_rate_probe::mastering_quality_device_delivery_rates` constructs
the actual `MasteringSource`, then pulls it through Rodio 0.20.1's
`UniformSourceIterator`, as the mixer does. It collects PCM offline, never opens
a device or plays these stress signals. Separate native callback/lifecycle proof
remains valid for its previous scope.

The 144-case matrix covers 44.1/48/96 kHz input, 44.1/48/96 kHz requested PCM,
44.1/48 kHz device output, target absent/-14, Preview LUFS off/on, and 997 Hz or
0.4-times-source-rate tones. Custom processing is flat, compressor Off, warmth
zero, stereo amplitudes 0.4/0.32. The live source retains its normal lookahead
lead and finite stop; the file renderer flushes/aligns its tail as before.

`b3-device-rates-v3.json` demonstrates the problem. For a 96 kHz, 38.4 kHz tone
with 44.1 kHz requested export and a 48 kHz device, the export planner removes
the ultrasonic content and calculates gain **25.84166**. Protected export
measures **-19.71292 LUFS / -1.00001 dBTP**. The live linear converter instead
folds that content into the audible range: **+23.37437 LUFS / +21.41173 dBTP**
after applying the export-derived gain. This is a synthetic stress witness,
not an assertion that ordinary music has these levels. It proves that a
protected export cannot certify the current live path, even at settled settings.

Local Rodio source confirms linear sample interpolation with no equivalent
lowpass; export uses the qualified Rubato FFT conversion. The requested codec
rate can also differ from preview's requested PCM rate, which is recorded
explicitly by the probe. Existing metadata does not yet resolve those routes
into a common audition plan. A target-independent file finalizer is insufficient
to close this live-rate defect.

Retained failed harness runs: v1 omitted Custom's required `id`; v2's
`Vec::collect` exercised Rodio's channel-converter `size_hint` underflow after
the first sample. V3 uses per-sample iteration like the native mixer. Neither
failure changed DSP assertions or peak tolerances. V3 passes capture/technical
execution, **not** live peak compliance.

## Candidate in progress

`quality_source.rs`, currently compiled only for tests, streams the same locked
Rubato FFT configuration as the corrected offline helper. It preallocates
adapter buffers, accounts for partial input, removes delay once and drains to
the exact frame count. It does not alter the filter or preset coefficients.
In `b3-quality-src-v1.log`, all **168** rate/channel/length combinations match
offline samples exactly, with unchanged adapter pointers/capacity. A seek check
discards previous filter state and matches a fresh conversion of the requested
suffix. This is not proof of zero internal allocation or a native timing budget.

The completed v4 rate probe compares this candidate with the preserved linear
path and saves the worst synthetic source/live/export witness under a fresh
ignored directory. Bandlimited conversion removes the aliasing, but **2/72**
Preview-on cases still exceed the -1 dBTP ceiling. For the 96 kHz stress source,
44.1 kHz requested export gives +1.34705 dBTP at a 48 kHz device and +0.17554 dBTP
at 44.1 kHz. Merely replacing the converter is not a sufficient correction.

After the Codex restart, `check_live_rate_witness.py` independently reconstructed
all four retained worst-case files with finite zero extension and SOXR16/64.
The largest reference peaks are +21.282731 dBTP for the legacy live path,
+1.304735 dBTP for the candidate live path, and -1.273684 dBTP for the protected
export. This confirms a real remaining live excess, not just the native meter's
bounded uncertainty. `b3-device-rates-v4-independent/comparison.json` records
the per-channel readings, source/report/tool hashes and full-scale sample counts.

A test-only finite-alignment option now primes and drains `MasteringSource` to
the file renderer's boundaries without consuming the swap fade during discarded
lookahead. All 64 rate/channel/length comparisons are bit-exact; seek and fade
checks pass, together with all 14 source tests (`b3-alignment-v2.log`). The first
fade test incorrectly assumed a positive sample from the processed DC fixture;
it now checks the exact unfaded reference times the required envelope. This
option is not active in playback.

The eight-case `b3-finite-gain-v1.json` witness identifies the residual. With
finite alignment, neutral streamed PCM equals offline PCM exactly. A matching
44.1 kHz export/device rate now meets the ceiling. Applying the 44.1 kHz export
gain to the 48 kHz signal still yields **+1.354811 dBTP**, whether gain is before
or after SRC (only about 0.000004 dB difference here). Actual-device verification
requires gain **0.7625335** relative to that export-derived gain. The corrected
result is -1.000010 dBTP natively and **-1.050087 dBTP** independently; uncapped
pre/post witnesses independently measure +1.304739/+1.304734 dBTP. Moving gain
alone cannot repair incompatible rate planning. Independent values are in
`b3-finite-gain-v1-independent/comparison.json` with hashes and full-file bounds.

## Allocation and native candidate checkpoint

The streaming iterator now publishes runtime failure codes through an atomic
slot retained by the caller, without formatting errors on the callback. Invalid
input stops iteration and remains failed until reconstruction. Both conversion
and identity paths reject nonfinite/incomplete input. Production still needs to
consume this slot and surface the error; the test-only module is not integrated.

The allocation harness exposed a same-rate seek problem, **not an FFT allocation**:
Rodio's sample-wise buffer preserves a partial channel position across seeks,
making the remaining stereo suffix odd-length. Pulling complete frames in the
identity adapter fixes it. Seeking the malformed error-test input also exposed
Rodio's subtraction underflow; failed sources now retain their error and reject
seek, requiring reconstruction. The first failed harness log is retained.

`b3-streaming-allocation-v2.log` passes all 17 tests, including the 168 streaming
parity cases and existing offline SRC/meter tests. The tracking allocator counts
**zero allocations** during first pull, seek and complete draining for 44.1→48,
96→44.1, 96→48 and 48→48 kHz. Initial converter output is 89.1/48.2/46.2/0.7 µs
in that run; these are adapter-only observations, not controlled device budgets.

`b3-streaming-native-v1.json` exercises the actual source-rate MasteringSource,
finite alignment and streaming converter on the default 48 kHz CPAL device,
muted after processing, during 120 coefficient edits and concurrent preparation.
There are **607 callbacks, zero deadline misses/device errors/exhausted samples**.
Callback median/p95/max are **0.0036/0.99968/1.9283 ms**. Buffering concentrates
FFT/DSP work into occasional callbacks, so the tiny median is not a throughput
improvement claim. Construction takes 0.2964 ms; the first source-meter update
is observed after 10.18 ms. Requested 256 frames again yields **480–1,056**, not
a fixed-256 pass. Other development jobs were running; controlled timings and
the complete production lifecycle remain separate. Meters in this probe are
still pre-SRC diagnostics. The stress-tone witnesses were never played.

Strict all-target Clippy passes (`b3-streaming-clippy-v2.log`). The desktop
library suite passes **483 tests, 12 ignored** (`b3-streaming-lib-v2.log`); the
three standalone streaming tests also pass. This checkpoint changes no active
playback route, so earlier full private-fixture/bridge evidence is not relabeled
as proof of this unintegrated candidate. Production adoption still needs the
affected fixture, bridge and native checks.

## Shared output meters and canonical file-rate route

Original/Mastered now share the same `OutputMeter` implementation. Both active
sources reset existing momentary/integrated meter storage on seek; the previous
momentary-meter reconstruction allocated a replacement buffer. The shared code
preserves their current source-rate readings and Mono's single-channel LUFS sum.
The test-only pipeline can move the meter slots out of either source, then run
`MeteredSource` after conversion. It meters the actual emitted frames without
also calculating the discarded source-rate meters, and applies the swap fade at
the emitted rate.

The 16 source tests include exact converted PCM/per-channel peaks and independent
EBU integrated-loudness comparisons on mono/stereo 997 Hz/38.4 kHz inputs. A
thread-local tracking allocator confirms **zero allocations through the complete
MasteringSource → converter → output-meter first fill, seek and drain**. The
first attempt to compile that check in an external integration test could not
access library-test-only allocation fingerprints; it now runs in the correct
library test context. The failed log remains, with no weakened runtime assertion.

`b3-output-meter-native-v1.json` repeats the muted native candidate with its
meters after SRC: **609 callbacks, zero deadline misses/device errors**, median/
p95/max **0.0297/1.02148/1.5871 ms**, 0.7121 ms construction and a 10.61 ms first
observed peak update. The driver still grants **480–1,056 frames**. This is
loaded-machine evidence, not an isolated speedup comparison or full AudioPlayer
lifecycle proof. `b3-output-meter-fixtures-v1.log` passes the complete desktop
suite, including **485 library tests** and all **four enabled private-fixture
tests**. Strict all-target Clippy passes (`b3-output-meter-clippy-v2.log`), as do
the latest 16 source checks (`b3-output-meter-v4.log`). The converter route remains
test-only; shared bridge contracts and preset/DSP coefficients are unchanged.

The eight-case `b3-finite-gain-v2.json` also tests the canonical file-rate route:
process/align → requested file SRC → export gain → device SRC. This reproduces
the signal an external player receives and matches the corresponding offline
conversion **sample-for-sample**. The worst retained direct-route mismatch changes
from +1.35481 dBTP to **-1.12691 dBTP**; independent SOXR16/64 gives **-1.170554
dBTP**. None of these eight cases needs additional device attenuation. Preserve
`b3-finite-gain-v2-independent/comparison.json` and its hashes. This small matrix
does not prove that arbitrary downstream conversion can never increase a peak.

**Engineering recommendation:** audition the requested file-rate signal before
device conversion, retain the actually opened device configuration, and validate/
cache any additional device cap against that same signal. Direct source→device
conversion with a gain derived from another rate cannot represent the export.
Wider rate/codec/seek tests, rate-change invalidation, live scalar placement and
matching readiness generations remain integration work. This route decision is
independent of the unresolved automatic-drive sonic policy.

### Actual output configuration checkpoint

`audio/output_route.rs` now opens CPAL explicitly and connects the existing Rodio
mixer and sinks. Inspection found that Rodio 0.20's `try_from_device_config`
internally retries other supported formats without returning the successful
configuration. Merely retaining its argument would be wrong on that path. The
new adapter retains the configuration of the successful CPAL build and preserves
Rodio's format heuristic/order, default-device alternatives and error precedence.
It does not change user device preferences. Runtime device errors set an atomic
flag; the control thread pauses playback, records the error and uses the existing
device-loss state, with no formatting or log I/O in the callback.

Three injected fallback regressions and **102 affected audio tests** pass
(`b3-output-config-v3.log`, `b3-output-config-audio-v1.log`), as does strict
all-target Clippy (`b3-output-config-clippy-v2.log`). The muted native v2 probe
opens default and explicitly selected Realtek streams at **48 kHz, stereo F32**.
Sink progress is observed after 37.49/33.06 ms; pause/seek/resume advances after
53.87/54.15 ms, with no device errors. These are snapshot observations under
concurrent research load. Actual hardware fallback was not forced; its ordering
is covered by injected failures. The earlier v1 probe only tested successful
default opening and did not qualify the hidden fallback. Retain both reports
and the intermediate compile-error log. No DAC/listening/installed/Mac or new
converter-route proof is implied.

Before completing production integration: route both audition kinds consistently, meter the actual
converted audio, reconcile requested/codec/device planning rates and obsolete
settings, and measure initial fill, seek/A-B/edit latency and callback deadlines.
The current test-only iterator also needs its processing-error state surfaced
appropriately before it can become a production source. No silent truncation or
unqualified live-ceiling claim is an acceptable final correction.

Evidence root: `test-output/mastering-quality-implementation-20260915/`.
Preserve the v1/v2 failed logs, v3 report/copied executable, v4 comparison/witness,
and the matching build logs. No installed, Mac or listening pass is implied.
