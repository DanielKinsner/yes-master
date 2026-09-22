# E native vectorized convolution experiment

Declared before building/running this alternative. Preserve the exact two FIR
tables, phase layout, positive curve, complete finite boundaries and partition
checks. The prior four-accumulator version improves the long filter only 8.12%
and slows the short filter 11.10%; it remains separately preserved.

On x86_64 with detected AVX support, process four f64 multiplies/adds per vector,
without fused multiply-add or altered coefficients. Choose the function once
when preparing each kernel. Use the original serial scalar implementation when
AVX is unavailable; do not require unsupported instructions globally. All vector
loads must stay inside both slices and scalar remainder handling is unchanged.
Record the actually selected implementation. This is an isolated native example,
not a production callback or cross-platform SIMD adoption.
Use the diagnostic `YES_MASTER_SATURATION_SCALAR` flag for one explicit fallback
run on this AVX-capable host; require its output to match the original serial
anchor exactly. This does not claim execution on a different processor platform.

Require the unchanged <=1e-10 maximum error on all eight independently constructed
finite-convolution cases and identical output across native partitions. Compare
exact output hashes with the prior four-accumulator implementation as a separate
addition-order check; a difference stays visible even if numerical qualification
passes. Retain every output and executable.

Run three alternating serial/vector pairs on the same frozen inputs with fresh
state: baseline/vector, vector/baseline, baseline/vector. Verify repeated output
hashes against each implementation's qualified anchor. Report preparation, full
processing, block median/p95/max, interval exceedances, and all paired ratios.
Do not claim hardware-independent speed, actual callback deadlines, changed
whole-song fidelity, or production adoption from this cost comparison.
