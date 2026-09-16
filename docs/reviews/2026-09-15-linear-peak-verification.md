# Whole-file peak verification with reusable preparation

Local B3 work, September 15, 2026. This supplements the
[reconstruction qualification](2026-09-15-peak-reconstruction-qualification.md).
It changes final scalar protection/measurement, not preset voicing or the limiter.

## Why reuse is valid

For either declared linear reconstruction, let `R` be the reconstruction operator,
`x` the prepared finite PCM and `y = g*x + e` the exact delivered PCM. If the
prepared peak interval is `[L,U]`, then a valid delivery interval is:

`[max(0, abs(g)*L - K*max(abs(e))), abs(g)*U + K*max(abs(e))]`.

`K` bounds the reconstruction's response to a unit sample error. For a finite
cardinal-sinc sum, the nearest term is at most one and there are at most two
terms at distance `k - 1/2`. Bounding the harmonic sum by its first term plus an
integral gives the conservative value `1 + 2/pi * (2 + log(2*N - 1))` for `N>1`.
The one-sample bound is one. This includes positions outside the file.

For the qualified lowpass response, use its maximum polyphase absolute coefficient
sum, divided by the same between-grid denominator, plus its numerical envelope.
Take the larger reconstruction bound and retain a small floating arithmetic
allowance. This is an amplitude-domain bound derived from length and filter
response, not a universal dB margin. The numerical reconstruction scope and
non-formal-interval-arithmetic limits remain those in the B2 record.

Every delivered sample is read. A product of two f32 samples/gains fits exactly
in f64; the gain-rounding error is measured during the in-place multiply. The
deterministic delivery reader then measures quantization/dither error against
that float buffer. Summing the two maximum errors conservatively covers their
combined residual. Final integrated LUFS and LRA are computed from the actual
delivery PCM, including absolute-gate membership; they are not shifted estimates.

## Qualification and limits

`mastering_quality_linear_probe` prepares 121 stress inputs and six retained
full-file witnesses. It tests -12/0/+6 dB at PCM16/24/float on stress material,
and -3 dB PCM16 on the whole-file cases. The 1,095 results all intersect a fresh
qualified reconstruction interval. The selected stress inputs include finite
edges, silence, DC, stereo, phase/frequency/rate variations, EBU-like bursts,
and long odd/even alternating signals. Deliberate integer clipping is included
as an error-bound stress case; it does not earn an audio-quality verdict.

`check_linear_reuse.py` independently checks exact delivered samples using
zero-extended float SOXR16/64, plus direct finite-sinc sums for short inputs.
**1,095/1,095 pass** (`b3-linear-independent-v1/comparison.json`). The original
inputs, copied executable, fresh-meter results and quantized sample files remain
under ignored `test-output/mastering-quality-implementation-20260915/`.

The residual scan took 0.33-0.60 seconds on the tested whole songs versus
19-26 seconds for fresh reconstruction (single experimental measurements,
not a repeated performance guarantee). However, these PCM16 reuse intervals
were roughly 0.060-0.069 dB wide, exceeding the retained 0.0501 dB numerical
criterion. The production finalizer therefore performs a fresh measurement when
reuse is too imprecise. Quiet/dither-dominated audio also takes this fallback.
Do not quote the scan-only timing as the complete PCM16 delivery cost. Normal
PCM24/float residuals are much smaller; actual per-export diagnostics record
whether the fallback was needed.

The subsequent filter-precision correction exposed a reporting limit on the
50 ms PCM16 regression: the reused bound was safe and only 0.049908 dB wide,
but its added quantization residual exceeded the existing 0.002 dB receipt-to-
fresh-reading comparison. The finalizer now also refreshes when the residual
envelope alone adds more than **0.001 dB** to its upper reading. This is a
reporting-precision criterion, separate from the 0.0501 dB reconstruction-width
criterion; the requested ceiling, output gain, dither and test tolerance are
unchanged. It preserves normal PCM24/float reuse and forces a fresh pass for
the affected short PCM16 output. All 12 short/no-target/profile/precision cases
pass (`c1-filter-short-receipt-v1.log`); the prior failure remains in
`c1-filter-fixtures-v2.log`. Wider fixture/cache validation is recorded in the
implementation ledger.

## Application integration in progress

`output_protection::finalize` prepares the final-rate float master, chooses an
optional loudness gain constrained by the qualified peak and a format-specific
rounding/dither reserve, then verifies every delivered sample. The strict gain
cap is rounded down; the old tiny-loudness-delta shortcut cannot waive a peak
attenuation. Integer sample headroom prevents quantizer saturation from acting
as protection. Invalid/uncertain/over-ceiling output returns an error before
writing; cancellation does not return a successful reading.

Track export/rendered preview, the live-preview background planning helper and
album per-track delivery now use this shared finalizer. The exact writer seed
and quantizers remain unchanged. The live callback/limiter is unchanged. Album
programme joins now have verified common-gain correction with exact component
parity; see the [checkpoint ledger](2026-09-15-mastering-quality-implementation-evidence.md).
Device-rate live audition and decoded-codec verification remain separate B3/B4
work; this is not a complete B3/B4 verdict.

The original five actual-export witnesses now pass their finite-zero-extension
SOXR16/64 checks (`b3-witnesses-v1/`). No-target Funk and 50 ms PCM24/float outputs
are below the requested -1 dBTP ceiling. The corrected hot-start CD witness
retains its exact count and original dither-residual tolerance. Owner On/Off
exports retain both frozen WAV hashes (`b3-owner-controls-v1/`). Their receipts
now use the qualified reconstruction, with estimator version/width in diagnostics.

The receipt tests that formerly demanded equality to the underreading ebur128
peak were changed to a fresh read with the declared meter, keeping the original
0.002/0.02 tolerances. Independent ceiling/reference tests are retained separately.
Focused protection, engine, album receipt and audio invariant checks pass.
The broader desktop library/integration suite also passes with all four private
fixture tests actually run (`b3-rust-fixtures-v2.log`). Strict all-target Clippy
passes. iPhone check/46 tests and Android 26 tests/arm64 API-29 check pass
(`b3-iphone-v1.log`, `b3-android-v1.log`). Subsequent album correction is recorded in the ledger; native/device work remains open.
