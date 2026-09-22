# C1 actual source-analysis gain diagnostic

Specification before the first run; analysis-only development experiment.
No production analysis, adaptation gate, preset or selected drive policy changes.

The existing C1 copy experiment deliberately retains original analysis/guards.
Measure what actual production analysis supplies for the eight restored sources
at 0/-40/-80 dB, and for peak-normalized versions of each copy. Use exactly C1's
f32 gain multiplication and f64 division by the largest absolute channel sample,
rounded back to f32. Preserve these complete float source transformations with
hashes in a fresh ignored evidence directory. Do not overwrite source material.

Use the actual complete-file analysis API, including its deep scan. Record each
analysis, derived SourceProfile, and the resulting Universal-75, Universal-50 and
Loud-75 coefficients under existing default-off calibration gates and Adapt 50%.
Preserve actual source LUFS separately from the normalized analysis descriptor;
do not interpret a display floor as an available integrated measurement.

This diagnoses which source facts can be reused, and which level/gating changes
affect the resolved chain. Compare actual and normalized analysis separately.
Record all numeric/profile/coefficient differences and unavailable values rather
than compressing them into a quality score. Exact normalized coefficient identity
is a diagnostic, not an audibility requirement. Reuse the existing C1 numeric
comparison limits where applicable (0.001 LU/dB, 1e-5 correlation); any differing
classification remains visible. No changed source-analysis rule is adopted by
this observation or counted as a successful holdout result.

Measure decode, transformation/write, and complete analysis wall time separately.
These concurrent development timings are not controlled callback/session costs.
An eventual implementation must preserve actual-source display/receipts while
using an explicitly identified preparation descriptor, if evidence supports it.
