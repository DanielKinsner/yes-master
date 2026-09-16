# Device-rate gain preparation

Local B3 continuation on `codex/mastering-quality`; this correction checkpoint
passes its declared local verification. **The transition regression at `9ebd2f81`
is corrected.** Static protection and transition fidelity are separate checks.
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
two final f32 products are reserved for device correction and attenuation-only
Volume Match. The upstream landing/SRC residual is measured separately.

## Corrected production implementation

The live route is DSP with the existing compensated file-landing crossfade,
file SRC, device SRC, device-specific correction, Volume Match, then final
meters/swap fade. VM is unity in the DSP and remains optional, attenuation-only
and audition-only. This is float audition, without simulated integer dither or
lossy decoding. Exports retain their finalizer, quantization and encoder rules.

The worker retains source-rate raw PCM as well as the file/device raw facts.
For a new target it applies the actual effective f32 source gain, runs the actual
converters, and compares **every resulting sample** with the scaled cached
response. The measured residual bounds reconstruction error; actual LUFS is
remeasured. If the bound becomes too loose (interval >0.0501 dB or residual-only
inflation >0.001 dB), fresh measurement occurs **before** choosing the correction.
Thus excess uncertainty cannot first cause unnecessary attenuation. Final
corrected device PCM still receives complete verification. No floating-point
gain/SRC commutation or excerpt-only proof is assumed.

Source/file responses share a two-entry **288 MiB** LRU budget. Equal-rate source
and file PCM share one allocation. Device raw responses have a separate two-entry
**192 MiB** budget; total retained PCM is capped at **480 MiB**. These caps exclude
source playback, transient render/SRC buffers and measurement workspace. Keys
include weak source identity, source/channel format, raw coefficients, file rate,
and device rate for device entries. Target/precision edits reuse raw DSP/facts
and reverify whole delivery. Source/processing/rate changes invalidate matching
facts. Oversized responses are not retained. The file-only path can finish its
owned buffer in place; device verification retains source PCM while temporarily
copying file delivery and constructing actual landed/SRC PCM. One shared worker
permit bounds expensive preparation across source/device lifetimes.

The gain plan carries both settings-publication and raw-processing revisions.
File landing participates in the existing DSP crossfade; target edits can require
converter buffers to drain again. VM-only edits can reuse the raw revision. The
post-device stage waits for matching raw PCM and completes its device-rate ramp
before acknowledging the plan. Preparation readiness and applied output remain
distinct; a paused prepared edit cannot claim its output has been emitted.
File-only prewarm results never count as device-qualified gains.

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
settings latency or a controlled system comparison. The full independent result
`b3-device-plan-independent-v1/comparison.json` is complete: **32/32 peak and
LUFS passes**, zero full-scale samples.

The original `b3-device-plan-unit-v1.log` retains the FFT NaN panic. The corrected
unit v2 and focused nonfinite-input regression pass. The post-device audio unit
run passes **109 tests**, including 72 exact finite production/offline cascades,
cache source/rate/byte-limit checks, matching-revision gain updates, concurrent
mailbox consistency and allocation-free gain/seek behavior. Strict all-target
Clippy v2 passes; v1's two clone-on-Copy diagnostics remain recorded.

## Preserved regression and correction evidence

The frozen compensating-gain diagnostic `b3-gain-transition-v1.json` compares
the new path with the prior compensated-master crossfade. A user-output change
of +/-12 dB has an exactly compensating landing change, so the intended steady
output stays constant. Across four source/file/device routes, the new ordering
produces 5 ms-window dips of 9.37-12.00 dB and up to +1.89 dB excess during the
transition. This is an objective interaction defect, not an owner sonic-policy
choice. The ordinary unit/whole-file/native deadline passes did not establish
transition fidelity. The corrected path must stay within **0.01 dB** of the
prior crossfade's 5 ms windows for this equal-output case; that numeric budget
is fixed before implementing the correction.

The corrected source-landing placement passes all eight cases **exactly: 0.0 dB**
minimum/maximum window difference (`b3-gain-transition-v2.json`). The same frozen
0.01 dB criterion is now an ordinary regression, not an opt-in report. New tests
also verify actual prepared/streaming PCM at the existing small-gain shortcut
(16 gain/rate combinations), exact 72 cascades, and fresh residual measurement
before gain planning when reuse would be inaccurate.

The correction passes strict all-target Clippy and the complete desktop suite:
**505 library tests**, 19 opt-in tests ignored, all integrations and all four
restored private-fixture tests executed. iPhone check/46 tests and Android 26
host tests/API-29 arm64 check pass (`b3-transition-correction-{clippy-v2,
fixtures-v1,iphone-v1,android-v1}.log`). The Clippy v1 syntax failure is retained;
no assertion or tolerance was relaxed. The copied test executable SHA-256 is
`5b5bdf2df4ce78512f33f430d1e897a91b32e27ec2f62589c91c783291f4e21f`.

### Corrected native preparation and response

`b3-transition-correction-native-v1.json` uses actual muted AudioPlayer at
source/file/device 44.1/96/48 kHz under concurrent research load:

| Operation | Preparation-ready observation | Matching revision emitted |
| --- | ---: | ---: |
| First target preparation | 46.821 s | 46.945 s |
| New -23 LUFS target | 2.083 s | 2.266 s |
| Return to cached -14 LUFS | 75.14 ms | 189.50 ms |

Paused cached edits remain unapplied until resume (189.92 ms). Playing/paused
rate edits, seek/resume and A/B pass with retained playhead. Cold playback accepts
in 242 ms and first meter arrives in 301 ms. File/source and device raw preparation
are 30.695/14.021 s. Whole-file delivery checks are 0.653/1.335 s initially and
0.651/1.319 s on the new target. The latter includes actual gain/SRC and residual
scanning, while both raw-cache lookups take about 0.001 ms. Retained source/file
and device PCM is **274,156,584 / 93,923,320 bytes**. No nonlinear DSP rerender
is required for the new target, but the necessary conversion/verification is
more expensive than the defective post-full-gain placement's scalar-only edit.

The associated 100 ms process observer records **50.840 s** session wall,
**50.359 s** last observed CPU, **767,135,744 bytes** peak working set and
**749,006,848 bytes** maximum observed private allocation. These observations
include setup/transport/edits, not just the first wait. Compared with the earlier
placement, this run uses about 168 MiB more peak working set. Different concurrent
loads preclude claiming a CPU/wall speedup from those two runs.

`b3-transition-correction-callback-v1.json` records **606 callbacks**, 120 edits,
**zero deadline misses/device/converter errors/exhausted samples**, median/p95/max
**0.05325/1.09983/1.83890 ms**, first peak 10.27 ms, construction 0.708 ms and
cancellation join 2.685 ms. Requested 256 frames grants **480-1,056**; this is
not a locked-256 pass or DAC/speaker latency measurement.

### Whole original-source evidence

The first-placement original-source matrix is complete across
`b3-device-live-v1/` (two Imaginal rates) and `b3-device-live-v2/` (the other
seven sources at both rates): **16/16 exact production/prepared PCM matches**.
All 16 independently pass peak/LUFS with zero full-scale samples, split across
`b3-device-live-imaginal-independent-v1/`, `b3-device-live-imaginal96-independent-v1/`
and `b3-device-live-v2-independent-v1/`. A CP1252 manifest read initially corrupted
Coat's Unicode filename; the failed v1 log remains. Correct UTF-8 paths and all
source hashes were checked before resuming only missing cases. No audio was lost
or requested again.

The transition correction changes gain/SRC ordering; fresh whole-file evidence
is complete under `b3-device-live-v3/` and `b3-device-live-v3-independent-v1/`.
All eight verified original sources at 44.1/96 kHz produce **16/16 exact sample
matches**, **16/16 independent peak/LUFS passes**, and zero full-scale samples.
The maximum qualified reading is **-1.000023365 dBTP** against a -1 request.
Imaginal 44.1 kHz measures **-1.00002408 dBTP** qualified and **-1.039898 dBTP**
independently; its WAV SHA-256 is
`eca130fc03f351270758ffa6623b3811de758b3f56c4bc5da73064b3a2046c32`.
The copied production test completes in 797.32 s under concurrent research load;
this includes all 16 offline traversals and is not a user's preparation wait.
The incremental checker reads only fully written hashed WAVs and requires all
16 rows plus the producer's complete status before declaring matrix completion.

### Earlier first-placement measurements (superseded)

The following observations describe `9ebd2f81`, which had the transition defect;
they remain evidence of static behavior and cost, not current transition proof.
Its full desktop run passed 502 library tests plus all integrations/private
fixtures. Its iPhone/Android compatibility checks passed.

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
No comparison to unmeasured older memory/CPU is inferred.

## Remaining scope

The subsequent [combined-gain transition correction](2026-09-15-combined-gain-transition.md)
closes a separate 1.023 dB bump when device correction and Volume Match change
inversely. The six corrected/interrupted cases stay within 6.472e-7 dB, with exact
settled PCM, 507 library/full fixture passes and zero observed deadline misses
across 655 rapid-gain and 606 EQ-edit native callbacks. Updated lifecycle and
separate session resource observations are in that record.

Dynamic settings transitions beyond the equal-output regression, seek/fade boundaries
and the final-result UI contract remain distinct from steady-state whole-file
protection. No blanket ceiling guarantee for every transient or hardware
reconstruction, installed app, Mac or new listening verdict is inferred.
