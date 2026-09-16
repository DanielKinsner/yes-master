# B3 live device-rate verification

Status: **Original and Mastered file/device conversion integrated; remaining B3 scope below**.
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

### Production Original conversion checkpoint

Original now streams decoded PCM through the qualified finite Rubato adapter at
the retained device rate. The shared output meter, spectrum rate and swap fade
follow conversion. Same-rate data remains unchanged; mono keeps its single-channel
LUFS contract. Mastered still needs its canonical file-rate/gain route, so this
does not close the reproduced Mastered-preview failure.

Runtime conversion failure retains an atomic reason outside the callback. The
control thread pauses the failed source, invalidates pending work and reports a
`playback_error` with a unique failed-source epoch. The frontend shows its existing
error surface once per failed attempt. It treats the failed sink as unloaded so
Play constructs a new source, while retaining playhead and requested controls.
Old-track errors cannot replace the selected track's feedback. The muted negative
probe (`b3-original-native-error-v2.log`) injects nonfinite input and verifies the
error survives source exhaustion; it does not masquerade as normal end-of-file
or device loss. The hook regression covers display, dismissal, retry and stale
track handling.

`b3-original-native-v1.json` exercises actual AudioPlayer Original playback on the
restored owner source, with test-only sink mute. Cold request acceptance/first
observed meter are **240/305 ms**; playing seek **62 ms**; paused seek/resume
**64 ms**; same-source swap **11 ms**, preserving position. Named/default device
reopening is **344/21 ms**, and playback restarts after each. These are snapshot
observations under concurrent work, not a controlled speedup or callback deadline
measurement. No Mastered/installed/listening/Mac claim follows from this probe.

Strict all-target Clippy passes (`b3-original-clippy-v4.log`). Full desktop tests
pass, with **488 library tests** and **all four restored private-fixture tests**
actually run (`b3-original-fixtures-v1.log`). The standalone streaming, source,
allocation and wire tests are included. The optional tick field is regenerated
from Rust; affected iPhone check/46 tests and Android 26 tests/API-29 check pass
(`b3-original-iphone-v1.log`, `b3-original-android-v1.log`). The later recovery
changes are desktop-only; bridge DSP/FFI behavior is unchanged. The WASM stamp is
rebuilt as **dc62b219e9f1** for the shared type source change. **896 frontend tests**,
build and **38 headless checks** pass; the minimum-size Standard capture is
inspected at `test-output/headless/2026-09-16T00-46-07-872Z/app/`.

### Production Mastered file/device conversion checkpoint

Mastered now uses `output_route::mastered_source`: finite source alignment and
tail drain, requested-file-rate SRC, device-rate SRC, then the shared output
meter and swap fade. Source DSP and its coefficient crossfade remain at the
source rate. Landing/Volume Match gains currently remain inside their matched
chain; linear SRC commutation is close numerically but is not claimed bit-equal
to applying those scalars after conversion. A live file-rate edit reconstructs
the converters from cached PCM and preserves the current playhead and pause
state. The nested converters share one error latch, so an inner error cannot
look like an ordinary outer end-of-file. Failed reconstruction uses the existing
visible processing-error/retry route.

`b3-mastered-cascade-unit-v2.log` compares the production constructor against
explicit offline chain/tail/two-converter processing: **72 sample-identical
rate/length cases**, including one-frame and partial-block files. The 144-case
`b3-mastered-cascade-rates-v1.json` retains all measurements; **all 72 preview-on
cases** meet the qualified -1 dBTP ceiling within the declared numerical
tolerance. The earlier 96 kHz/38.4 kHz stress witness with requested 44.1 kHz and
48 kHz device changes from legacy **+21.412 dBTP** to **-1.126912 dBTP**. Independent
whole-file finite SOXR16/64 gives **-1.170551 dBTP**, zero full-scale samples
(`b3-mastered-cascade-independent-v1/comparison.json`). The slight difference
from the previous post-file-gain experiment is retained, not rounded into a
bit-equivalence claim. Stress audio was never played audibly.

`b3-mastered-native-v1.json` exercises actual AudioPlayer with the restored
owner source and test-only sink mute. Cold Mastered request/first meter take
**299/354 ms**. Playing rate edits are accepted in **5.5–11.0 ms**; paused edits
preserve position and remain paused through the explicit additional 100 ms
observation. Paused seek/resume is **64.9 ms**. Four Original/Mastered swaps take
**9.8–10.6 ms** and preserve position. At requested **96 kHz**, first whole-file
preparation/readiness takes **41.96 s**, a new target **1.02 s**, and returning
to the cached target settles within the **75 ms** observation interval. Raw
cache occupancy is 187,875,824 bytes. This lifecycle uses the existing test
settings with density zero; the callback probe below exercises preset compression.

`b3-mastered-callback-v1.json` uses the production constructor at
**44.1 → 96 → 48 kHz**, preset compression, 120 coefficient edits and concurrent
whole-file preparation. All **608 callbacks** finish within their granted
deadlines, with zero device/converter errors or exhausted samples. Median/p95/max
processing are **0.07255/1.06042/1.76270 ms**; construction **0.94 ms**, first
peak **10.17 ms**. Requested 256 frames still grants **480–1,056**. The background
measurement uses its recorded 48 kHz settings; the 96 kHz callback override
deliberately loads both converters. This is a callback-cost probe, not a matching
landing-gain proof. All timings are under concurrent research load.

Strict all-target Clippy passes (`b3-mastered-cascade-clippy-v1.log`). The first
matrix build had a routine missing boolean return and is retained as unit v1.
The first full-suite attempt encountered Windows LNK1104 while the native test
executable was still open after the research documentation commit changed the
build stamp. That is a build-run collision; preserve the failed log and use a
serial retry, rather than interpreting it as a passed fixture lane.

The serial retry **passes the complete desktop suite**, including **490 library
tests**, 17 opt-in tests ignored, and **all four restored private-fixture tests
actually executed** (`b3-mastered-cascade-fixtures-v2.log`). It includes the nested
converter failure regression and five allocation-free cascade paths through
first fill, seek and drain. This slice changes desktop audio modules and their
tests; it changes no shared bridge DTO, engine/DSP source, frontend rendering or
WASM input. The preceding bridge/frontend/headless evidence retains its stated
scope; no redundant packaging or listening lane is claimed.

**Still open at this checkpoint:** codec-specific rate/precision resolution, wider full-song/device
peak planning, and readiness tied to the actual applied settings through buffered
audio. Preparation-ready snapshots above do not time the last crossfade sample.
The 144-case matrix does not prove arbitrary downstream conversion can never
raise a peak; no universal live ceiling, installed, Mac or listening pass is
claimed. Full delivered-file protection remains enforced by the separate qualified
finalizer.

Evidence root: `test-output/mastering-quality-implementation-20260915/`.
Preserve the v1/v2 failed logs, v3 report/copied executable, v4 comparison/witness,
and the matching build logs. No installed, Mac or listening pass is implied.

### Selected-encoding PCM preparation checkpoint

Play, settings updates and prewarm now carry the selected export encoding. The
backend uses `ExportEncoding::delivery_rate` and `delivery_bits`, exactly as
export does, after the source rate is known. If those differ from the requested
profile, a transient Custom copy retains its effective LUFS target, ceiling and
unchanged chain coefficients. The user's requested/saved profile is untouched.
Default omitted encoding stays WAV. Format changes cancel obsolete prewarm,
update a loaded Mastered route and rebuild converters only when the resolved
file rate changes. Prepared-cache identity includes resolved PCM settings;
formats with matching raw PCM can share work, including precision-only changes.

`b3-codec-plan-fixtures-v1.log` passes the complete desktop suite: **492 library
tests**, 18 opt-in tests ignored and **all four restored private-fixture tests
executed**. This includes **672** codec/profile/source-rate/precision combinations
with exact coefficient and requested-JSON preservation, idempotent resolution,
rate invalidation and direct/cached whole-delivery equality. Strict all-target
Clippy passes (`b3-codec-plan-clippy-v2.log`). The first Clippy log retains an
incorrect test Arc argument and flat-IPC argument-count lint, corrected in v2.

`b3-codec-plan-native-v1.json` exercises actual native play/update handlers through
all **seven formats**, using a synthetic 96 kHz file and test-only mute. On the
actual 48 kHz device, MP3/M4A/ADTS AAC/Vorbis resolve to 48 kHz PCM; WAV/FLAC/AIFF
retain 96 kHz. Route/meter observations complete in 0.001–9.83 ms; equal-rate
updates can finish synchronously. These are acceptance observations under load,
not settled-loudness or comparative timing measurements.

`b3-codec-plan-frontend-v4.log` passes **897 tests**. The hook regression verifies
format arguments for prewarm/play/live edits, old-prewarm cancellation and
unchanged requested settings. Its first added assertion used the wrong mock
playback-kind token/batched state setup; the corrected v3 focused run passes.
`b3-codec-plan-headless-v1.log` passes the build, landing checks and **38 app
checks**. The minimum-size format/export capture at
`test-output/headless/2026-09-16T01-37-12-708Z/app/mp3-export-1360x740.png`
was inspected: receipt text and action buttons remain unobscured. No markup,
shared bridge DTO/DSP input or WASM source changed in this slice.

This qualifies **pre-encode PCM**, not the decoded lossy waveform. The B4
advisory lossy-peak scope remains. Wider full-song/device peak protection and
actual applied-generation readiness are still open; no installed/listening/Mac
claim is made.

### Whole-file device-rate witness

`mastering_quality_device_probe` converts **16 retained protected file outputs**
(control and single candidate for all eight original sources) from 48 kHz to
44.1/96 kHz through the actual production `QualitySource`: **32 complete files**.
Manifest/input/output hashes, exact ceil frame counts, converter errors, finite
PCM, full-scale counts, LUFS and qualified peaks are retained in
`b3-device-whole-v1/report.json`. No DSP is rerendered; these are the original
C1 controls from before the later filter-precision correction.

The Imaginal control at 44.1 kHz reaches **-0.775274 dBTP** in the qualified
meter. Independent complete-file SOXR16/64 with the frozen finite padding gives
**-0.815114 dBTP**, above its requested **-1 dBTP** ceiling, with **zero samples
at full scale** and a passing LUFS comparison. This is an inter-sample peak miss,
not PCM clipping. `b3-device-imaginal-independent-v1/comparison.json` retains
the failed ceiling result; it is not waived by the passing earlier short matrix.
The complete matrix's independent checker finished: **31/32 peak passes, 32/32
LUFS passes, zero full-scale samples**. The sole independent ceiling failure is
the same Imaginal 44.1 kHz case (`b3-device-whole-independent-v1/comparison.json`).

This is a supported simulated output rate through production conversion, not
an opened 44.1 kHz native device or a listening result. It establishes the need
for device-specific whole-file planning; it does not establish a universal
margin, scalar commutation, or an implemented device cap. Protected delivered
files remain unchanged. The probe's build and strict all-target Clippy passed.

### Applied output revision checkpoint

Background preparation and emitted output now have separate diagnostic tokens.
Each coefficient publication receives a distinct revision, including early VM
and final landing within one logical settings generation. `MasteringSource`
acknowledges only the first unmixed frame after its 512-frame crossfade. Each
converter propagates revisions on emission, requiring current and preceding
whole input blocks to agree for the locked FFT overlap. The final meter/fade
stage waits until the incoming fade is unity. Seek resets every stage's token.
This changes no sample calculations, filter, preset or UI measurement label.

Two focused regressions pass, including **36 two-converter rate combinations,
each replayed after seek**. The zero-valued suffix must contain no stale PCM
whenever its final revision is acknowledged. The DSP regression retains the
last mixed frame as pending. Strict all-target Clippy and the complete desktop
suite pass: **494 library tests**, 18 opt-in tests ignored, all integration
tests and **all four restored private-fixture tests executed**. Existing exact
cascade, allocation-free, transport, gain-cache and coefficient tests pass.
Logs: `b3-output-revision-unit-v1`, `-clippy-v1`, `-fixtures-v1`.

Muted actual AudioPlayer evidence (`b3-output-revision-native-v1.json`) separates:

| Operation | Preparation-ready observation | Matching revision emitted |
| --- | ---: | ---: |
| First 96 kHz preview preparation | 32.173 s | 32.351 s |
| New target -23 LUFS | 0.767 s | 0.951 s |
| Return to cached -14 LUFS | 0.075 s | 0.189 s |

A paused cached edit requests revision 19 while emitted revision remains 17;
resume reaches 19 after **184.6 ms**. Paused/playing rate edits, seek and A/B also
pass. These are snapshots of application output under concurrent load, **not
DAC/speaker latency or a controlled speed comparison**. The UI's measurement
status still describes background measurement; these new fields establish the
separate output-readiness observation, not a new final-result UI contract.

`b3-output-revision-callback-v2.json` exercises **44.1 → 96 → 48 kHz** with preset
compression, 120 coefficient edits and concurrent preparation: **607 callbacks,
zero deadline misses/device/converter errors or exhausted samples**. Median/p95/
max processing is **0.0617/1.1356/1.9633 ms**; first peak 10.31 ms, construction
0.767 ms, cancellation join 8.10 ms. Requested 256 frames grants **480–1,056**.
Callback v1 accidentally used the default file rate because the environment
variable name was wrong; retain its evidence without claiming the 96 kHz path.
This desktop-only slice changes no bridge/shared DSP/WASM/markup. No installed,
Mac or listening result follows. Device-specific peak protection remains open.

### Device gain and reusable preparation continuation

The [device preparation record](2026-09-15-device-gain-preparation.md) contains
32 independently passing retained-output core cases and 16 exact/independently
passing original-source cases for the first placement. That placement later
exposed a compensated-edit transition regression; all failed evidence remains.
The correction retains source landing in the DSP crossfade and verifies actual
landed/SRC PCM before applying only device-specific correction downstream.

All eight equal-output transition cases now match the prior crossfade exactly.
Full desktop/private fixtures (505 library tests), Clippy, bridges, actual muted
lifecycle and 606 loaded callbacks pass. First preparation/application is
46.821/46.945 s; new target 2.083/2.266 s; cached target 75/189 ms. Session wall,
CPU and memory are separately recorded. These loaded measurements do not prove
a controlled speedup, DAC latency or subjective sound quality.

Fresh corrected full-original-source and independent matrices are complete:
16/16 exact PCM matches and independent peak/LUFS passes, zero full-scale samples.
Dynamic boundary and final-result UI coverage remain open. No installed, Mac or
new listening verdict follows.
