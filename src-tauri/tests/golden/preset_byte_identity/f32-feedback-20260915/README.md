# Preserved f32 feedback references

These nine deterministic synthetic references are the exact files from local
commit `3fecbbe9`, before the September 15 recursive-filter precision correction.
They contain no private audio. Custom remains byte-identical; the eight factory
responses change by at most 0.000247 on this decimated probe.

The active references one directory above use the corrected f64 feedback state
and arithmetic, with the same f32 coefficients and stage outputs. The 1e-6 test
tolerance is unchanged. The original failed test log, old/new hashes and measured
deltas are retained in ignored `test-output/mastering-quality-implementation-20260915/`
as `c1-filter-fixtures-v1.log` and `c1-filter-golden-deltas-v1.json`.
See [the numerical diagnosis](../../../../../docs/reviews/2026-09-15-drive-gain-consistency.md)
for the full-chain experiment and its separate listening/performance scope.
