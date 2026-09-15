# Coordinator notes: independent Claude review

> **Final status:** both the blind assessment and unblinded replication are
> complete. Read the [final synthesis](2026-09-15-mastering-quality-final-synthesis.md),
> [byte-preserved report archive](evidence/2026-09-15-claude-audit/README.md), and
> [portable transfer guide](2026-09-15-mastering-quality-transfer.md).
> Instructions and “follow-up ready” status below describe the earlier setup,
> not an outstanding review. No workspace deletion was performed in the final pass.

The owner requested a question-led assessment that does not reveal the previous
investigation's suspected causes or recommendations. This file is for the owner
and coordinator, **not the independent reviewer**.

## Current status: independent report complete, unblinded follow-up ready

The owner reported completion. Claude's `audit-output/INDEPENDENT_REPORT.md`
and all 18 files listed in `SEALED_MANIFEST.json` match their recorded hashes
(seal timestamp `2026-09-15T02:47:56.771757+00:00`). Integrity verification does
not by itself reproduce its experimental claims.

The [targeted replication prompt](../prompts/2026-09-15-mastering-quality-targeted-replication.md)
is also copied to `FOLLOWUP_REVIEW.md` at the root of the separate workspace.
For this next pass, explicitly authorize the reviewer to read that file and
the named main-checkout evidence. This ends blinding for the follow-up while
preserving the original sealed assessment. New findings belong in a separate
reconciliation directory and report.

Keep the review workspace for this pass. It is a folder without `.git`, not a
second repository: at inspection, it held about 13.8 GiB including supplied
fixtures; `audit-output/renders` accounted for 9.7 GiB and its build cache for
1.5 GiB. These sizes can change during further work.

A verified partial checkpoint is at
`test-output/claude-independent-audit-20260915-text-checkpoint/` in the main
checkout: 74 files / 2,388,313 bytes, covering the sealed report/protocol,
manifests, reproduction source/locks/scripts, cases and measured text results.
It excludes audio and build outputs. Its `CHECKPOINT_MANIFEST.json` records
hashes and the explicit limitation; it is **not a complete evidence archive**.
No files were deleted. After reconciliation, preserve the necessary unique
audio and complete reproduction records, verify that archive, then remove
disposable caches/duplicates or the temporary workspace as appropriate.

## Prepared locally

- Separate workspace: sibling folder `yes-master-independent-audit-20260915/`.
- Source: canonical Git bytes from `a4fb621d88a95b8af549467fb499943acb4274d5`;
  452 application/source/build/license files, with no implementation edits.
- Eight source WAVs with identity, licenses, credits and preparation metadata;
  one owner export with settings/hash for baseline reproduction.
- Neutral, byte-identical `AGENTS.md` and `CLAUDE.md`, `START_REVIEW.md`,
  `REVIEW_INPUTS.md`, fixture manifest and source-hash manifest.
- No Git history, prior research reports, research results, new experiment
  harness, candidate outputs or competitor masters. Existing production code
  and its tests remain legitimate implementation evidence.

Start a **fresh Claude Fable conversation** with that folder as its workspace.
Use this initial message:

> Read START_REVIEW.md and carry out the independent review. Use only this
> workspace for YES source and project context. Design your own tests, then
> record and seal your findings before consulting any earlier assessment.

Do not attach the original continuation brief, recommendation, this coordinator
note or the prior conversation. Disable prior project-memory/context injection
where the client supports it. If the reviewer already saw the findings, start
a fresh context; instructions cannot make it forget them.

## What this can establish

This is an assessment blind to previous conclusions, not a formal double-blind
study. The supplied corpus was selected by the coordinator, source comments and
existing tests reflect development history, and both implementations may share
upstream library limitations. The reviewer chooses its own tests and can acquire
new licensed inputs when justified. Its protocol and original findings must be
sealed before the comparison phase.

Finding the same issue independently increases confidence. Failure to discover
an issue is not itself a disproof; inspect test coverage. A conflicting measured
result calls for exact-input/build/method reconciliation. A later targeted
replication should be explicitly labelled as unblinded and appended separately.
Do not change the independent report to make it agree with earlier work.

## Rebuild on another machine

The [neutral prompt](../prompts/2026-09-15-independent-mastering-audit.md) and
[coordinator preparation script](../../scripts/research/prepare-independent-audit-20260915.py)
travel with Git. The separate workspace and audio do not. After restoring the
existing private archive and expanded source corpus on the receiving machine,
run the preparation script from the repository. It requires unused destination
and temporary archive paths, preserves source audio, and verifies all copied
music hashes. Its references to previous work are intentionally not copied into
the review workspace.

The local preparation verified source-manifest hashes, all input hashes, the
baseline-export hash and identical instruction files. No Claude review has been
launched or completed by this preparation; production behavior is unchanged.
