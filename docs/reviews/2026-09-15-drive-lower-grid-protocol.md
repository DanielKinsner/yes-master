# C1 corrected-chain lower-drive development grid

Status: fixed before processing this new grid. This is a bounded development
comparison, not the C2 holdout freeze or a production automatic policy.

## Question and fixed scope

The completed low-drive diagnostic recovers several dynamics measures but has
large loudness shortfalls. Its -24/-48 dB outputs fail numerical convergence and
remain diagnostic evidence. Test whether intermediate drive reduces the conflict
with the current f64 filter-state production chain. Do not change the existing
character limits, infer a linear reference, or replace historical f32 results.

Reuse the three retained Universal-75 single-rule cases at -14 LUFS: Coat,
Piano and Imaginal. These are known development material, deliberately including
the passing control and difficult dynamics/tone cases. Use their exact requested
settings, normalized source PCM, base operating drive and coefficient strings.
The E current variants are the matching corrected-chain controls. Additional
input offsets are exactly **0, -3, -6 and -12 dB**, compensated after the chain
as in the preceding diagnostic. No midpoint, grid expansion or changed preset
is allowed in this comparison. Three zero-offset outputs must reproduce the
retained control PCM exactly; reuse those WAVs and their qualified measurements.

Every new candidate is a continuous whole-song render, with the existing latency
compensation, actual limiter/compressor observations, production SRC to 48 kHz
and qualified float-output finalizer at the requested -1 dBTP ceiling. Retain all
nine new WAVs. An estimated-output check reserves 25 GiB of free disk afterward.
Hash the protocol, original jobs, source/control reports and sources before work.

## Measurements and comparison rules

Use the original source's fixed C1 section/attack anchors and unchanged
`drive_metrics.LIMITS`. Keep each tone/stereo delta visible; its comparison
reference is the matched normalized single-rule control, as in the separate
normalized-reference diagnostic. Missing required metrics fail. Every whole
output must pass the existing independent SOXR16/64 peak and LUFS comparison,
exact frame/rate/channel checks, finite PCM and zero full-scale samples.
Hash-identical controls can reuse explicitly identified same-scope checks.

Report two hypothetical selections, neither adopted:

- Character-preserving: among technically valid candidates meeting all existing
  character limits, choose a target hit (absolute error <= 0.2 LU) nearest the
  single-rule drive. Otherwise choose the smallest absolute target error, then
  nearest drive. With none qualified, report the verified control as a fallback
  and retain its individual failures and target miss.
- Target-first: use the same target-hit/nearness/error order among technically
  valid candidates, retaining every character failure in the result.

Keep the single rule's result separate. If it meets target and all character
limits, the preserving rule necessarily selects it: show the extra work the
remaining three development candidates would avoid in that case. Do not assume
unsampled monotonicity or describe an available fallback as character-qualified.

Record chain, SRC and finalizer times per output; sum the actual evaluation work
and distinguish it from the reused controls' measurement cost. These are one-pass
offline observations, not controlled throughput, source-preparation savings,
settings settling, playback deadlines or a total native session benchmark.
The source/settings preparation, finite search policy, numerical limits and
fallback need broader corrected-chain testing before the C2 freeze. The owner
sound-priority choice, control mapping and C3 integration remain separate.
