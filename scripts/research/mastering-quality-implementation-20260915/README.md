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

The subsequent `prepare/evaluate/summarize_broader_first.py` helpers bind and
verify the two-preset/two-target extension. `prepare/evaluate/summarize_target_reuse.py`
test the same prepared outputs under another delivery target. The deliberately
ignored `output_protection::prepared_research::prepared_measurement_reuse_benchmark`
compares existing fresh/prepared APIs at release optimization and requires all
sixteen outputs to match four verified files bit for bit. Its module is guarded
by `cfg(test)`; `prepare_measurement_reuse.py` copies the built executable before
preflight, and `summarize_measurement_reuse.py` retains the initial preparation
cost and both timing observations. These experiments do not alter normal app DSP.
See `docs/reviews/2026-09-16-preset-and-target-reuse-results.md` for scope and
remaining limitations. `summarize_recovery_native.py` records post-power Realtek
opening/callback checks separately from historical failures.

The September 18 Universal 50 extension uses `--coverage remaining-universal50`
with the broader preparation/summary helpers. It adds only the six missing
known sources at two targets (24 whole files), with the same DSP and acceptance
limits. `join_universal50.py` requires fresh complete evidence plus exact
revalidation of the retained Funk/Rich measurements and files before joining
eight-source coverage. See `docs/reviews/2026-09-18-universal50-coverage-protocol.md`.

`metal_positive.py prepare/summarize` binds two subsequent +3 dB reproductions
to the retained historical lead and four independently verified corrected-chain
control/single files. The fixed selector is not changed to accept those positive
offsets. See `docs/reviews/2026-09-18-metal-positive-drive-protocol.md`; its outcome
is a separate development observation, not a rewritten Universal 50 selection.
