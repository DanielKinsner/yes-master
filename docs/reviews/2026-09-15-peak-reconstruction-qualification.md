# B2 peak reconstruction qualification

Status: **experiment; no production adoption**. Branch `codex/mastering-quality`.
This supplements the [implementation ledger](2026-09-15-mastering-quality-implementation-evidence.md).
Historical measurements, scripts and audio remain unchanged. New output is under
`test-output/mastering-quality-implementation-20260915/`.

## Why another reference check was necessary

The reviewed ebur128 0.1.10 implementation and five local Hann-sinc FIRs do not
resolve the documented output failures. Increasing FFT interpolation factor and
overlap alone also failed. Qualification must distinguish the chosen finite-file
reconstruction from a periodic FFT window or a resampler's startup response.

New observations:

- The first native finite-sinc-bound experiment covered 481 cases and contained
  the independently evaluated direct-sum controls. Only 370 cases passed **all** existing
  FFT/SOXR checks: SOXR's narrower filter produces larger burst peaks in some cases.
  The worst discrepancy was 0.797 dB on a 0.45-fs, 45-degree, 17-frame burst.
- A measured SOXR impulse response accounts for these interior burst differences.
  Combined with finite-sinc bounds/refinement, the Python experiment passed 458/481
  comparisons with the historical direct references. Remaining differences are
  finite-start responses and a long alternating periodic-FFT case. These are
  retained discrepancies, not a silently relaxed pass.
- A 1.4-amplitude first-frame impulse produced 1.466962 in direct SOXR output,
  but 1.337351 with silence before it. The extracted interior impulse response
  predicts the latter. The current SOXR source initializes/flushed stages with
  zeros; its multi-stage finite-start response must not be assumed to equal a
  single zero-extended convolution. [SOXR source](https://raw.githubusercontent.com/chirlu/soxr/master/src/cr.c)
- Adding 2,048 zero samples before/after six selected inputs made SOXR agree with
  the extracted convolution to at most 1.34e-15 per sample. This includes first/
  last impulses, one/17-frame DC, two-frame random stereo, and the burst above.
  See `b2-reconstruction-edges-v1/comparison.json`.
- For 262,143/262,144/262,145 alternating samples, a direct reciprocal sum on a
  1,024× grid near the left edge finds 7.285694/7.285697/7.285699 dB. The new finite
  reconstruction interval contains all three (about 7.265–7.307 dB). The preserved
  FFT reference reports radically different odd/even values: about 8.392/5.457/
  5.129 dB at 32×. Its finite-periodic block extension is not an exact finite-sinc
  oracle. Do not fit a safety margin to that length-dependent discrepancy.

The SOXR cutoff parameter denotes a passband edge, and differs from SWR's cutoff
definition. The actual precision-33 kernel is measured here rather than inferred
from an oversampling label. [FFmpeg resampler documentation](https://www.ffmpeg.org/ffmpeg-resampler.html)

## Experimental reconstruction and explicit uncertainty

For finite PCM, define the full-band signal as the sum of cardinal sinc functions
weighted by the actual samples, with zero samples outside the file. This includes
exterior ringout, original sample maxima, and coherent Nyquist content.

The candidate evaluates local finite sinc convolution by FFT on 4,096-frame cores
with 4,096 frames of input context on each side. It uses 16 fractional phases;
this is a meter only and never changes the limiter or callback signal path.
FFT convolution padding covers every local input/output displacement, avoiding
the periodic-extension error of FFT interpolation on an arbitrary cropped window.

Omitted distant samples are not assumed to vanish. For alternating values
`y[n] = (-1)^n x[n]`, summation by parts bounds a distant interval's sinc
contribution by its maximum partial-sum magnitude divided by nearest distance
and π. Per-4,096-frame prefix extrema replace a whole-file prefix array.

Where this cheap bound is too wide, a separately prepared hierarchy of order-20
source moments estimates the signed far-tail contribution. For source/target
centres separated by `d`, let `q` be their combined radii divided by `abs(d)`.
Accept a node only at `q <= 1/2`. A geometric-series remainder bounds its error
by `node_L1 / abs(d) * q^21 / (1-q)`. Translate the moments into one target
polynomial, then evaluate that polynomial across the core. Refinement changes
work and interval tightness, not the definition of the measured signal.

For the finite sinc sum, the second-derivative supremum is bounded by π² times
the signal supremum in input-sample time. Together with linear-interpolation
remainder, a bound `G` on the complete 16× grid gives a continuous bound
`G / (1 - π²/(8*16²))`. This accounts for between-grid peaks (~0.04196 dB),
not truncated tails; the separate tail bound above is essential. The finite-sum
signal is in the applicable Bernstein space. [Primary mathematical reference,
equation 48](https://d-nb.info/1158599315/34)

Outside the examined exterior extent, `L1/(π*distance)` is already below the
original sample peak. Rust additionally records an amplitude-domain floating
allowance proportional to signal L1 and operation depth. This remains a tested
numerical implementation, not a formally rounded interval-arithmetic proof.

## Second reconstruction response

`extract_soxr_kernel.py` measures the precision-33, 16× SOXR impulse response
using three positions and odd/even input lengths. A ±512-input-sample kernel
retains all but at most 6.72e-14 of measured absolute impulse weight. Repositioning
changes any coefficient by at most 2.23e-16; combined discarded weight/shift
residual is below 2e-13. `b2-soxr-kernel-v1/provenance.json` records the generating
FFmpeg binary hash, version/configuration, filter command and kernel hash.

The standalone Rust polyphase-FIR experiment accepts this numeric kernel from
the qualification driver. It carries complete context and finite ringout with
fixed FFT workspace. The library meter now embeds that exact numeric response
with its generator/hash provenance; this table is measurement data, not private
music or a processing filter. The lowpass continuous allowance is checked against
the independently generated 16/32/64x finite-zero-extension references below.
The numerical residual allowance is 1e-10 times source sample peak, covering the
measured <2e-13 coefficient/truncation residual and fixed FFT error envelope.
This is tested numerical qualification, not formal interval arithmetic or a
universal DAC claim. No delivered-audio processing stage has changed in this slice.

## Acceptance specification before production integration

1. Retain finite-input validation, sample maxima, channel independence, exact
   reset semantics and cancellation. A cancelled/invalid measurement is not silence.
2. Independently validate local sums, far-tail remainders, finite exterior bounds
   and between-grid peaks. Source/block boundary, odd/even and above-full-scale
   cases remain mandatory. Check long coherent signals with direct sums.
3. Target interval width is at most 0.05 dB for the finite full-band reconstruction;
   record width and refinement cost. This is an engineering work/accuracy criterion,
   not a new sonic threshold or a fixed attenuation reserve. Wider bounds must
   remain visible and cannot silently earn an accurate-reading verdict.
4. Compare native results with the independently implemented Python calculation,
   reviewed library at 1/257/4,096-frame chunks, corrected FFT, direct finite sums
   and float SOXR. Retain direct and padded SOXR separately. Classify each changed
   reference assumption with a reproducible primary-signal witness; do not simply
   discard failures because they are inconvenient.
5. Use ITU-R BS.1770-5 / EBU Tech 3341 phase/burst constructions, frequency/phase
   sweeps, impulses, DC, silence, alternating, random and stereo isolation. The
   generated EBU-like examples alone are not a full official compliance-test run.
6. On whole-file music and actual delivery witnesses, compare delivered PCM against
   both independently checked reconstruction responses, including exterior peaks.
   Requested ceiling remains unchanged. Quantization and gain-rounding reserves
   need separate evidence, and final delivered audio must be remeasured.
7. Before B3, record algorithm/kernel versions, numeric reserve, actual peak-error
   envelope, memory, setup/whole-file cost and cancellation. Keep native callback
   and installed/listening claims separate. Qualifying this meter does not certify
   device-rate preview, album joins or encoded delivery.

### Current qualification result

`b2-finite-qualified-v1/comparison.json` now records **481/481** synthetic cases
passing the finite-zero-extension references (SOXR16/32/64 and direct-sum controls)
and the full-band interval-width criterion. The native order-20 refinement reduces
long-alternating uncertainty from over 10 dB to about 0.042 dB while retaining
the independently observed 7.2857 dB peak. Only 4–6 cores needed refinement on
these 262k-frame cases. Historical direct FFT/SOXR values remain alongside the new
reference results, with the scope witnesses above explaining their discrepancies.

The standalone native comparison includes the reviewed ebur128 implementation
at 1/257/4,096-frame input chunks, with both unflushed and explicitly flushed
results. Its three flushed results are identical on all 481 inputs. The expanded
native unit probe also passes direct finite-sum containment at internal/finite
edges, long coherent refinement, non-core-aligned zero-prefix invariance, stereo
isolation, silence, invalid input and cancellation during preparation/interpolation.
Strict all-target Clippy passes (`b2-experiments-clippy-v3.log`).

Whole-file native music/delivery-witness checks now pass **6/6** independent
finite-zero-extension reference and interval-width checks
(`b2-music-qualified-v1/comparison.json`). The cases include owner On/Off/lower
drive, no-target Funk, hot-start CD and the 50 ms witness. This measures the
existing output; it does not make the remaining over-ceiling witnesses safe.

The real-input FFT optimization preserves all **481 synthetic + 6 whole-file**
results within 1.34e-15 amplitude of the retained complex-FFT implementation.
`b2-realfft-parity-v1/` retains both comparisons and verifies the same source
hashes, channels/frame counts, independent reference envelope and interval
criterion. Four direct-sum/edge/stereo/cancellation tests and strict all-target
Clippy pass. The subsequent B3 finalizer now uses the qualified core for final
PCM; [integration evidence and remaining routes](2026-09-15-linear-peak-verification.md)
remain separate from this estimator qualification.

### Controlled real-FFT cost comparison

`b2-realfft-cost-v1/report.json` records three rotating before/after trials of
copied executables, with other task-owned builds/measurements stopped. Both use
the same optimized dev profile and input files; normal OS/thermal variation
remains. Seconds below are medians, with full-band and lowpass work separate:

| Source | Complex full-band / FIR | Real full-band / FIR |
| --- | ---: | ---: |
| 60 s hot-start CD | 4.498 / 1.707 | 2.266 / 0.609 |
| 244.56 s owner On | 23.572 / 6.415 | 14.349 / 3.924 |
| 422.25 s Aphelion | 37.018 / 9.910 | 22.632 / 6.140 |

The owner/long summed median verifier costs drop about 39%. FIR kernel setup
falls from 1.85 to 1.21 ms (prepared once per process). Full-band first-call setup
is included in the first short-input reading. The complete three-source process,
including decode/hash and three library comparison passes, drops from median
87.98 to 54.38 s. Peak working set drops from roughly 300 to 295 MiB; these totals
include decoded programme audio and are not standalone meter allocation counts.
The real-FFT optimization is retained. This remains too costly for repeated
uncached interaction, reinforcing the measured C0 reuse requirement.

### Replayable library core and exact delivery reader

`src-tauri/src/peak_meter/` exposes the qualified reconstruction intervals, their
width and version `finite-sinc20-soxr16-4591255f-realfft-1`. It accepts borrowed
float PCM or bounded channel reads from an immutable, replayable provider. The
subsequent B3 finalizer uses this core for application final PCM. Four additional native
tests establish reset/cancellation/invalid-input handling and exact delivery
replay. `b2-provider-parity-v2/` records **bit-identical 481 + 6** reconstruction
results against the standalone real-FFT implementation, retaining all independent
reference checks. Strict all-target Clippy passes (`b2-provider-clippy-v2.log`).

`wav_writer::DeliveryPcm` stores four bytes of dither PRNG state per 4096 frames.
It can replay the existing quantizer at arbitrary read boundaries without a
second full-track PCM buffer. Mono/stereo, PCM16/24/float, 1/257/4096/8193-frame
reads and actual written WAV decoding agree sample for sample. Metering this
reader equals metering the separately decoded samples. The writer and its seed
are unchanged, including their existing final saturation behavior; this reader
alone does not establish that incoming PCM was protected from clipping.

Filter plans are immutable process-wide data; per-call FFT arrays have fixed
size. Prefix summaries, interval records and optional moment trees grow with
the number of 4096-frame blocks, with fallible source-sized reservations. This
is bounded small metadata per source block, not constant total memory. It never
allocates 16x whole-programme audio. FFT planning still uses the dependency's
ordinary fixed-size allocations. Cancellation is checked between source blocks,
core/phase transforms and tree construction. All this work belongs on workers.
