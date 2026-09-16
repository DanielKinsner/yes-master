# C1 normalized reference experiment

Development revision 2; specification written before evaluating this rule.
This changes no production selector, voicing, threshold or owner decision.

## Question and fixed comparison

The original C1 selector measures tone/stereo allowance against the current
processing control. That control changes when the source is attenuated, even
when the source-relative candidates use identical normalized drive. The retained
gain-copy diagnosis isolates this reference dependence from the separately fixed
filter arithmetic. Holding the original control fixed was only a diagnostic:
production cannot recover an unknown original input level.

Evaluate the already defined **single source-relative candidate** as the
tone/stereo reference for each preset/target group. This is computable from the
actual source, preserves that candidate's preset coefficients and adds no new
candidate render to the existing comparison. Use that group's own single
candidate for each input-level copy; never borrow the unattenuated control.

Keep all other development-v1 rules unchanged: source-anchored dynamics/attack
limits, tone/stereo slack and floor, target tolerance, maximum eight coarse
points/two refinements, candidate tie-break, technical validity and explicit
corrected-control fallback. Score both preserving and target-first modes. A
fallback's input-level dependence remains visible; this experiment cannot
declare arbitrary source analysis or fallback audio gain invariant.

## Evidence and acceptance

Re-score completed original/quiet-copy metrics without rendering or remeasuring
audio. Retain metric-file hashes, exact limits, the script/specification hashes,
old and new selections, each candidate's changed constraint failures, and
fallback/target/character outcomes. Refuse incomplete metric reports, ambiguous
candidate groups or missing single/control references. Compare candidate
identity by preset, target, policy and relative offset, independently of row IDs.

The diagnostic consistency criterion is identical candidate and fallback status
across 0/-40/-80 dB for each tested group. Also report whether more target hits
or fallbacks result and the selected source-relative metrics individually. A
consistency pass alone is **not** a sonic improvement, a numeric PCM pass, a
justification to loosen character limits or a production adoption gate.

These are the original frozen f32-chain development results. The experiment
isolates the selector's reference choice; the old numeric failures remain.
Any promising rule still requires corrected-chain evaluation, actual source
analysis, measured cost, a frozen holdout protocol and genuinely new holdout
validation before C3. The owner's automatic-dynamics/control decisions remain
separate. Do not reopen the frozen baseline or call these sources unseen.
