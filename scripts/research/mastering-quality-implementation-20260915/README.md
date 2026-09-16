# Mastering correctness verification

Local main retains the peak, PCM, codec and native-callback verification tools
used by the mastering fixes. They write to fresh ignored output locations and
must preserve whole-file checks, hashes, failed witnesses and stated tolerances.

The full drive-selection, source-normalization and saturation experiment runners are
preserved on `codex/mastering-quality` at `03df038a`, with their full original
README. They were deliberately excluded from the fixes integration. Historical
research reports in `docs/reviews/` describe that snapshot, not production policy.

`verification_common.py` extracts the existing hash and peak helper functions so
these checks do not depend on the excluded sound-experiment modules. Peak
reconstruction methods and acceptance tolerances are unchanged.

See `docs/reviews/2026-09-16-mastering-fixes-integration.md` for exact source
identity, retained evidence and the current integration/research sequence.

`codex/mastering-dynamics-research` starts from verified local main `ab420654`.
It restores only the needed C1 measurement helpers and adds the separately frozen
single-first development checkpoint. Its protocol is
`docs/reviews/2026-09-16-single-first-development-protocol.md`. These tools never
run from normal mastering and do not adopt automatic sonic limits.

The completed five-source comparison and two-source follow-up are recorded in
`docs/reviews/2026-09-16-single-first-development-results.md`. `prepare_single_first.py`
and `prepare_flagged_drive.py` bind inputs, specification and copied probe hashes
and check disk space after the build. `run_single_first.py` records child-process
cost; the two `evaluate_*` helpers apply the frozen source/control measurements.
`check_completed_device_rows.py` independently verifies whole files and only
reuses matching previously passing PCM. `summarize_single_first.py` requires
complete hash-bound reports and recomputes all character constraints before
joining the evidence. The failed follow-up v1 and successful v2 remain separate.
