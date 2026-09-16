# Device-rate gain preparation

Local B3 continuation on `codex/mastering-quality`; verification in progress.
This does not adopt a drive policy, alter preset coefficients, or certify an
installer, physical 44.1 kHz device, listening session or release.

## Why the device needs its own result

The retained full Imaginal control is protected at file rate, but its 48 to
44.1 kHz playback conversion reaches -0.775274 dBTP in the qualified meter and
-0.815114 dBTP in independent SOXR16/64. The original failed files and reports
remain intact in `b3-device-whole-v1/` and `b3-device-whole-independent-v1/`.
See [the live-rate evidence](2026-09-15-live-rate-verification.md).

The device result retains the verified file's desired gain and constrains it
against the complete raw device-rate response. It does not choose a second
loudness target or impose a universal headroom margin. Actual gain multiplication,
every delivered sample and final LUFS are verified on the worker. The existing
qualified reconstruction and format-specific residual bounds remain in force;
two f32 products are reserved for landing and attenuation-only Volume Match.

## Production implementation under verification

The live route is raw DSP, file SRC, device SRC, landing gain, Volume Match,
then final meters/swap fade. Raw DSP has unity landing/VM scalars. Uniform gains
are deliberately applied after conversion; floating-point commutation is not
assumed. This is float audition, without simulated integer dither or lossy decode.
File exports retain their established finalizer, quantization and encoder rules.

The control thread publishes a gain plan with both a settings-publication revision
and the matching raw-processing revision. The gain stage waits for matching raw
PCM through both converters before acknowledging the plan. Its transition lasts
the existing 512-frame/48 kHz duration, scaled to device rate. Target-only/VM edits
reuse the raw revision and need no new DSP crossfade or converter-buffer refill.
The existing preparation-pending state and applied-output diagnostic remain
distinct; a paused prepared edit cannot claim its output has been emitted.

Two independent LRU caches retain file and device raw responses, each with at
most two entries and a 192 MiB PCM budget. Keys include weak source identity,
source/channel format, actual raw coefficients, intermediate file rate, and
device rate for device entries. Target/precision edits reuse raw responses and
still verify each requested complete delivery. Source/processing/rate changes
invalidate the appropriate facts. Oversized responses are finished once in place;
they are not retained or copied just to bypass a budget. File prewarm results
never masquerade as device-qualified gains. One shared worker permit continues
to bound expensive preparation across source/device lifetimes.

Preparation failures use the existing playback-error surface for the matching
request; stale/cancelled work cannot publish a successful device result. The new
cancellable SRC entry checks cancellation during input validation and FFT blocks.
Nonfinite PCM is rejected before Rubato, whose FFT backend otherwise panics on
the retained NaN witness. Finite above-full-scale input remains valid.

## Completed core evidence

`b3-device-plan-v1/report.json` contains **32 complete outputs**: control and
single candidate for all eight original sources, each converted to 44.1/96 kHz.
These inputs are the frozen pre-precision-correction C1 files; no raw DSP is
rerendered in this experiment. Every qualified peak is below the requested -1
dBTP ceiling, with a maximum reading of -1.000023365 dBTP. Three subsequent gain
edits per file also complete whole-file verification; returning to gain 1 yields
exactly identical PCM. Input/output hashes and per-phase timings are retained.

The Imaginal control at 44.1 kHz uses gain 0.97445613 (about -0.225 dB). Its new
qualified reading is -1.00002384 dBTP and independent SOXR16/64 gives -1.039868
dBTP. Initial device preparation takes 19.467 s and one whole-file verification
0.431 s under concurrent research load. These are core measurements, not native
settings latency or a controlled system comparison. The full independent queue
is retained under `b3-device-plan-independent-v1/` and is still running.

The original `b3-device-plan-unit-v1.log` retains the FFT NaN panic. The corrected
unit v2 and focused nonfinite-input regression pass. The post-device audio unit
run passes **109 tests**, including 72 exact finite production/offline cascades,
cache source/rate/byte-limit checks, matching-revision gain updates, concurrent
mailbox consistency and allocation-free gain/seek behavior. Strict all-target
Clippy v2 passes; v1's two clone-on-Copy diagnostics remain recorded.

## Outstanding evidence

The complete desktop/private-fixture suite passes: **502 library tests**, 19
opt-in tests ignored, all integrations and all four restored private-fixture tests
executed (`b3-post-device-fixtures-v1.log`). iPhone check/46 tests and Android
26 host tests/API-29 arm64 check pass. The full original-source production matrix
is still running. The first
complete Imaginal 44.1 kHz production comparison has **zero differing PCM
samples** against its measured plan, at -1.00002384 dBTP. Its fresh independent
reference passes at **-1.039866 dBTP**, with passing LUFS and zero full-scale
samples (`b3-device-live-imaginal-independent-v1/comparison.json`).

Muted native AudioPlayer `b3-post-device-native-v1.json` passes playing/paused
rate edits, seek, A/B and output revision checks. At source/file/device rates
44.1/96/48 kHz, initial measurement/application is **49.779/49.834 s**, new -23
LUFS target **1.154/1.220 s**, cached -14 target **75.24/75.25 ms**. A paused cached
edit remains unapplied until resume, then acknowledges after **68.83 ms**.
First raw file preparation is 33.185 s; device preparation 15.438 s; file/device
whole-file verification 0.702/0.339 s. The next target hits both raw caches and
still verifies complete file/device delivery in 0.707/0.336 s. Retained file/device
PCM is **187,875,824 / 93,923,320 bytes**. These loaded observations expose the
extra upfront/device verification cost; they are not a controlled speedup claim.

`b3-post-device-callback-v1.json` passes **608 native callbacks** during 120 edits
and concurrent preparation, with **zero deadline misses, device/converter errors
or exhausted samples**. Median/p95/max processing is **0.0683/1.2214/2.0287 ms**;
first peak 10.08 ms, construction 0.733 ms, cancellation join 2.968 ms. Requested
256 frames grants **480-1,056**, so there is no locked-256 pass.

The lifecycle observer retains 445 process samples: total wall **52.453 s**, last
observed CPU **53.297 s**, peak working set **591,114,240 bytes**, maximum observed
private allocation **572,698,624 bytes**. The original observer emitted null
aggregate maxima because PowerShell treated its ordered dictionaries differently;
all raw observations are intact. `b3-post-device-native-v1-resource-summary.json`
recomputes those maxima with the original report hash. The observer now uses
property objects; the passing native test was not rerun solely for aggregation.
No comparison to unmeasured older memory/CPU is inferred. The independent complete-file core queue
must finish. First preparation, target-change settling, actual output revision,
callback responsiveness and total session CPU/memory/wall cost must be reported
separately. Dynamic settings transitions, seek/fade boundaries and the final-result
UI contract remain distinct from steady-state whole-file protection. No blanket
ceiling guarantee for every transient or hardware reconstruction is inferred.
