# Broader preset/target first-candidate checkpoint

Frozen before fresh audio; builds from verified local main `ab420654` on the
research branch. This extends the completed Universal 75/-14 development work
without repeating the original full search or opening holdout outcomes.

## Matrix and controls

Known development sources: **Funk and Rich**. Presets/intensities: **Universal 50
and Loud 75**. Requested targets: **-14 and -9 LUFS**. For each of the eight
combinations, render current processing and the existing normalized single
candidate through the corrected production chain, SRC and finalizer. Delivery
remains 48 kHz stereo float at -1 dBTP. Sixteen fresh complete WAVs.

Bind original completed C1 source, analysis anchors, requested settings, operating
drive and coefficient strings by hash. Source PCM decode/normalization is done
once per source and reused across its eight renders. No source re-analysis,
new drive formula, preset change, altered compression mode, saturation change,
manual trim compensation or adjustment of the owner's requested target.

Use unchanged C1 measurements and limits; each candidate's tone/stereo baseline
is its matched current-processing control at the same preset/intensity/target.
Loud 75 is an intentional dense preset: any character-limit failures are diagnostic
evidence, not permission to erase its intended sound or gate its existing use.
The same numerical limits may prove unsuitable for that intent; report the
failure and develop a separate explicit contract before adoption.

Apply `control-inclusive-preserving-v1` to each verified control/single pair.
If no candidate qualifies, retain the explicit research fallback result. If a
character-qualified result misses the target, retain the actual miss. No extra
offsets are launched within this checkpoint, and no limit is changed to earn a
pass. Missing measurements fail the relevant constraint. This remains partial
development coverage, not completion of the all-source/quiet-copy C2 matrix.

## Verification and measurement

Independently verify all sixteen whole files using the existing SOXR16/64 and
LUFS checks, rate/frame/channel/hash identity and finite/full-scale checks.
Native qualified peaks must meet the requested ceiling. Bind the final report,
metrics, selector, frozen protocol and copied executable identities.

Record once-per-source decode/normalization, chain, SRC, finalization, character
metrics and independent reference work separately. Record child wall/CPU/peak
memory; no estimate is substituted for a measured search cost. Original source
analysis is reused and excluded. These offline times are not app import or
settings-settling times. Build/copy first, then require 25 GiB free reserve plus
the sixteen WAVs' bounded size estimate. Preserve all old evidence and failed runs.
