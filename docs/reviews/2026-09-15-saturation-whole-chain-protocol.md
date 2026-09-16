# E whole-chain antialias comparison, development 1

Frozen before rendering this comparison. Production DSP and presets are unchanged.
Use current f64-filter-state production arithmetic and the exact native prototype
FIR tables. Work in a generated ignored source copy with pinned source hashes.

## Matched cases and variants

Use the completed C1 requested settings/coefficients/drive for these four cases:
Coat, Piano and Imaginal Universal-75 single candidate at -14; Coat Universal-75
current-processing control at -9. The three single candidates use their existing
source normalization. The control uses actual source PCM with zero automatic
drive. Verify source/report hashes and resolved coefficients before rendering.
These are development cases, not selected production C policy or new holdouts.

For each, compare current f32 saturation, the same positive curve evaluated in
f64 with f32 stage output, and that positive f64 curve with each retained 8x
FIR (1,025 and 4,097 taps). This separate f64-only control prevents attributing
curve arithmetic differences to antialiasing. No new continuity knee is selected.
All other processing, settings and source input are matched.

## Required proof and reporting

The instrumented current-curve variant must reproduce a fresh unmodified
production chain exactly before variant comparisons. Require correct finite
lengths, complete compensated filter/limiter tails, finite output and full-file
48 kHz float protection through the existing qualified finalizer. Keep all 16
outputs with hashes; independently verify their complete peak and loudness.
Flush the combined group delay and retain exactly the original N-frame aligned
interval, preserving the export duration contract. Exterior FIR ringout beyond
that interval remains separately qualified by the complete native finite probe;
do not append it to a song or imply that a fixed-duration export contains it.
F64-only raw sample difference must be <=1e-5 versus the matched current curve;
that is a numeric isolation limit, not an audibility threshold. A failure remains
visible and does not justify weakening the limit.

Record actual limiter reduction percentiles/max/active fraction, raw/delivered
LUFS, final gain/peak, filter delay, kernel/state preparation and whole-chain/
SRC/finalization time separately. Use the existing source-anchored C1 section,
paired attack, tonal-band and stereo metrics, and report differences against
both source and matched controls individually. C1's proposed character limits
can describe outcomes but cannot certify a preferred E curve or preset migration.

Reuse filter tables across matched cases. No repeated table design per sample,
per channel or per candidate. The instrumented wrapper is offline; its thread-
local dispatch is not a proposed realtime integration. Native callback/session
cost, broader source/rate coverage, the eventual selected C policy and targeted
owner calibration/listening remain separate requirements. The long native
prototype's three block-interval exceedances remain unresolved performance
evidence; an offline full-song pass does not erase them.
