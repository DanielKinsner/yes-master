# Mastering correctness verification

This branch retains the peak, PCM, codec and native-callback verification tools
used by the mastering fixes. They write to fresh ignored output locations and
must preserve whole-file checks, hashes, failed witnesses and stated tolerances.

The drive-selection, source-normalization and saturation experiment runners are
preserved on `codex/mastering-quality` at `03df038a`, with their full original
README. They are deliberately excluded from the fixes integration. Historical
research reports in `docs/reviews/` describe that snapshot, not production policy.

`verification_common.py` extracts the existing hash and peak helper functions so
these checks do not depend on the excluded sound-experiment modules. Peak
reconstruction methods and acceptance tolerances are unchanged.

See `docs/reviews/2026-09-16-mastering-fixes-integration.md` for exact source
identity, retained evidence and the current integration/research sequence.
