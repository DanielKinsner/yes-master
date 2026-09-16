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
