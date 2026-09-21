# Bounded bidirectional search: four unresolved development sources

The [frozen protocol](2026-09-21-bidirectional-search-protocol.md) evaluates
Universal 50, original levels and -14 LUFS on Piano, Imaginal, Metal and Rich.
It retains the corrected current control and normalized single, then evaluates
exactly -3/+3 dB when the initial pair cannot meet character and target together.
Nine complete outputs are reused; seven missing outputs are rendered. The
historical selector and all historical results remain unchanged.

Status: **verified**. All seven new complete WAVs pass independent SOXR16/64
peak, LUFS, exact frame/rate/channel, finite/full-scale and hash checks. All
nine retained outputs revalidate against their unchanged source/control facts.
No production algorithm, preset, saved setting or user-facing diagnostic changes.

The [hash-bound evidence](evidence/2026-09-21-dynamics-research/bidirectional-v1.json)
records every candidate, individual character failure, old/new selection and cost.

| Source | Selected processing | Delivered LUFS | Character | Target |
| --- | --- | ---: | --- | --- |
| Piano | Single -3 dB | -14.875 | Pass | Shortfall |
| Imaginal | Current | -21.050 | Pass | Shortfall |
| Metal | Single +3 dB, retained | -14.000 | Pass | Pass |
| Rich | Current | -20.276 | Pass | Shortfall |

The two earlier character fallbacks gain qualified alternatives. Only Metal
also meets the target; Piano exchanges target attainment for qualified dynamics.
The two existing qualified current controls remain selected. This is a benefit
in two of four deliberately selected unresolved development contexts, not an
unseen success rate or evidence that all targets can be reached without tradeoffs.

## Character findings

Piano's -3 candidate reduces fixed-section contrast loss from **1.993 dB**
with current processing and **1.610 dB** with the zero-offset single to
**0.676 dB**, below the unchanged 0.75 dB limit. Median/lower-decile paired
attack losses are **0.411/1.379 dB**. Every character limit passes, but delivery
is **-14.875 LUFS**: a 0.875 LU shortfall. Its +3 candidate reaches -14 while
failing section, median-attack and three tone-band limits. This supports the
owner's experimental dynamics-preserving direction without calling the target met.

Imaginal's -3 candidate improves median attack loss to **1.496 dB**, but its
lower-decile loss is **4.132 dB**, beyond the unchanged 3.5 dB limit. It delivers
-15.628 LUFS. +3 reaches -14 with larger dynamics and tone failures. Neither
new candidate clears character qualification; qualified current processing
remains eligible with its original target shortfall.

Metal's -3 candidate still fails the upper tone band. The previously verified
+3 candidate remains the qualifying point inside the new two-direction grid;
its narrow tone margin and prior whole-file evidence are retained. The new
selector may consider that point; the older selector and its fallback reports
are not rewritten.

Rich's -3 candidate still loses **2.201 dB** of fixed-section contrast and fails
three tone bands. +3 loses **5.553 dB**, with additional attack and tone failures.
Current processing retains qualified character and remains the appropriate
experimental selection despite its target shortfall. No additional offset is
rendered to rescue either failed directional pair.

## Cost and limits

Native child: **87.048 s wall / 85.344 s CPU / 664.7 MiB peak working set**.
All four directional pairs fit the provisional <=1.5-times-baseline additional
evaluation budget, and the child fits the 2 GiB memory budget.

| Source | Retained current + single work | Directional pair work | Extra / baseline |
| --- | ---: | ---: | ---: |
| Piano | 22.758 s | 17.675 s | 0.777 |
| Imaginal | 39.430 s | 30.986 s | 0.786 |
| Metal | 21.675 s | 18.237 s | 0.841 |
| Rich | 24.285 s | 27.544 s | 1.134 |

Work means chain + SRC + finalization. Metal's pair includes the historical
+3 timing; baseline costs are retained observations, not newly rerun controls.
These are workload comparisons, not paired speed benchmarks or app latency.
Source analysis/anchors are reused, and their historical cost is not zero.

The seven new evaluations use **84.584 s**: 19.655 chain, 0.864 SRC and 64.064
finalization. Character work is 16.184 s; independent reference work is
230.630 s. Full character/reference job wall observations are 17.791/234.497 s.
Rendering, character and independent stages ran serially, without local build
or research-job overlap. The seven new WAVs use **680,545,900 bytes**; the prior
26 September 18 WAVs remain intact. Raw jobs, copied binary, metrics, logs,
process data and WAVs are in ignored `test-output/dynamics-research-20260921/`.

Release example build, strict example Clippy, Rust formatting, Python compilation
and selector regressions pass. The native processing function is unchanged;
only a new bounded job shape is added to the offline example. All production
source and lockfiles match pushed main. Unaffected application/fixture/bridge/
UI/package evidence is reused under `docs/TESTING.md`. No new native playback,
installed-app, listening, Mac or remote-CI pass is claimed for this local research.

The result supports considering both drive directions within a strict budget,
but the rule is still insufficient for reliable target attainment on these
known sources. Further work should establish the source/preset/control contract
and corrected-policy quiet-copy behavior before expanding a search grid or
opening C2 holdout material. Prepared-buffer/fact reuse and final delivery gain
remain separate from this selection experiment. No owner sonic-policy decision
is required merely to preserve these measurements.

Full C1/C2, intentional dense/manual/no-target/Off/Adapt/Album/save contracts,
B3's remaining native boundaries, listening preference and C3 integration remain
open. This checkpoint does not adopt -14 as a preparation default, +3 as a
constant, Universal 50 as a product default, or new shipping thresholds.
