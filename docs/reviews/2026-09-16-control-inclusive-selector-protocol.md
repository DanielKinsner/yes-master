# Control-inclusive preserving selector, development v1

Frozen after the single-first/flagged development results, before applying the
new rule. This supersedes neither of those experiments' recorded selections.
Owner direction: preserve dynamics in experiments; keep normal product use
polished and measured results accurate. No production change is authorized here.

## Fixed rule

- Retain unchanged C1 character and 0.2 LU research target limits. Do not relax
  Metal's tone failure or substitute a combined quality score.
- Accept only candidates belonging to the same source, requested intent and
  processing version, with whole-file technical verification. Retain explicit
  technical rejection reasons. Duplicate identifiers or mixed contexts fail.
- Include the corrected current-processing control alongside normalized single
  candidates at input offsets 0/-3/-6/-12 dB. Current processing is not merely
  an emergency fallback. Offsets remain separate from user input trim.
- First consider technically valid rows with no character failures. Among target
  hits, prefer current processing, then the smallest absolute candidate offset.
  This preserves existing processing when it already meets the contract. If
  none hits the target, select the smallest absolute target error among character
  qualified rows; ties prefer current processing, then smaller offset, then ID.
- If no row qualifies for character, return the technically verified current
  control with an explicit internal `character_fallback` result and its failures.
  This is not a character PASS. If that control is technically invalid, return
  `no_verified_fallback`, never a newly invented unverified fallback.
- A qualifying target hit needs no further search. Otherwise further search is
  bounded to the listed offsets; no monotonicity assumption or limit relaxation.
  This checkpoint only reselects existing whole files; it does not implement a
  production search scheduler or measure the time saved by hypothetical stops.

## Verification before broader audio

Use the five known development sources in the September 16 joined evidence.
Retain exact file hashes and source/intent identity. Record both the earlier
frozen selection and the new resolved selection; do not regenerate audio or
reuse a file at a different request. Coat's -9 control is excluded from this
-14 comparison because it has a different requested intent.

Regression expectations: Rich chooses its already character-qualified current
control instead of the quieter -12 candidate. Metal remains an explicit
character fallback. Funk/Aphelion/Baby retain their qualifying single candidates.
Test order independence, valid-control preference on a target tie, missing
character data, an over-ceiling candidate, no verified fallback, duplicate IDs
and mixed source/settings context. Missing measurements fail qualification.

Produce an immutable internal selection record with context, algorithm version,
chosen file hash/drive identity, actual loudness/peak, character qualification,
target feasibility and reason. Keep this research-only; no saved controls, app
warnings, presets, saturation curve or gating changes. Broader targets/presets,
quiet-copy regressions, numerical/runtime freeze and unseen holdout remain next.
