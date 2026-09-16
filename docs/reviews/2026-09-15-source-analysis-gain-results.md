# C1 actual source-analysis gain results

The [pre-result specification](2026-09-15-source-analysis-gain-experiment.md)
completed **48 real production analyses**: eight original sources, three gain
levels, with actual and peak-normalized PCM measured separately. All source and
transformed-WAV hashes are verified. Evidence remains under ignored
`test-output/mastering-quality-implementation-20260915/c1-source-analysis-v1/`.
This changes no production analysis, adaptation gate or mastering policy.

## Findings

| Comparison across original versus -40/-80 copies | Actual-level analysis | Normalized analysis |
| --- | ---: | ---: |
| Exact Universal-75/50 and Loud-75 coefficient matches | 33/48 | **48/48** |
| Largest available LUFS shift error | 3.0833 LU | **0.000000383 LU** |
| Largest LRA difference | 12.0614 LU | **0** |
| Largest P95-P10 dynamic-range difference | 1.1865 dB | **0.000000954 dB** |
| Unavailable integrated measurements | 8/16 | **0/16** |

Actual-level differences are not all numeric defects: absolute loudness gating
changes eligible content, and every -80 copy is below the integrated gate.
The serialized analysis retains the existing display floor, while this diagnostic
separately records the actual EBU result as unavailable. No floor is counted as
a target hit or a normalization descriptor. Source-analysis peaks still use the
existing EBU meter; this is not the qualified final-output peak lane.

Normalized profiles differ by at most 1.49e-8 in a spectral share and 9.54e-7 dB
in P95-P10 range. Those tiny differences do not change any of the 48 resolved
coefficient sets. Original input versus its own normalized analysis also yields
**24/24 identical coefficient sets** across the eight sources/three settings.
The deep scan actually runs; normal AnalysisResult serialization excludes its
internal window arrays. Derived profiles, guard diagnostics and resulting
coefficients are retained, without enabling the gated compressor/confidence code.

## Preparation cost and recommendation

Across all 24 actual-level analyses, median complete analysis is **3.431 s**;
across the 24 normalized analyses, **3.376 s**. Normalized transformation/write
median is **0.288 s**, versus **0.223 s** for actual copies. These loaded offline
measurements include complete file analysis and test-file I/O; they are not a
controlled production speedup or callback timing. Decoding, transformations and
analysis are separately recorded per case. A normalized descriptor is reusable
source information; target or preset edits need not calculate it again.

Carry a separately identified normalized descriptor into corrected-chain C1
experiments. Preserve actual source measurements for display and receipts.
This eliminates the observed source-analysis dependency on these artificial gain
copies without changing the tested original-source coefficients. It does not
establish arbitrary-source invariance, resolve selector character/fallback limits,
or justify a global replacement of source analysis. C2's frozen holdout and C3's
matching-plan/control/lifecycle requirements remain open.

## Reproduction and limits

Build/run `mastering_quality_source_analysis` with a verified original-source
manifest, the specification and a fresh output directory. Then run
`compare_source_analysis.py --report REPORT --output FRESH.json`; it requires
all six conditions per source and verifies every complete input WAV. The v2
comparison additionally reports original versus normalized-source coefficient
identity; v1 remains preserved. No audio is rerendered during this comparison.

Strict example Clippy v2 and build v1 pass. The first Clippy failure attempted to
serialize an internal guard type; the example now records its explicit fields
without changing that production type. No runtime or numerical assertion was
relaxed. Copied executable SHA-256:
`f53b5d1b16524dca2071314d5666597a2e6bf9052e0f07e8df7a84b9c9e63246`.
`c1-source-analysis-provenance-v1.json` retains source/script/specification/report
hashes. Existing passing production suites remain applicable: this slice adds
only an analysis example, comparison tool and internal evidence documents.
