# E native FIR accumulation experiment

Declared before the optimized run. The first long-filter native prototype spends
about 210 ms on 32,768 stereo frames and has three observed 256-frame interval
exceedances. Table construction is already negligible. Investigate accumulation
throughput while preserving the exact FIR coefficients, phase layout, curve and
finite boundaries.

Replace the serial dot-product accumulator with four independent accumulators,
then combine them and handle remaining coefficients. This changes f64 addition
order, not the filter. It must meet the unchanged <=1e-10 complete finite-output
error versus the independent convolution and preserve native chunk invariance.
Keep the first executable/output unchanged as the baseline.

Run three alternating baseline/optimized pairs using the same frozen four-signal,
two-filter job, with fresh processor state and output folders each time. Verify
each repeated output hash against its own qualified first run; report any change.
Report table/state preparation, per-256-frame duration, complete processing,
interval exceedances and median ratios separately. Simultaneous full-chain
research makes these alternating loaded measurements, not isolated-system or
actual-device deadline guarantees. Preserve outliers. No production curve/filter
adoption, callback pass or whole-chain sonic verdict follows from a cost win.
