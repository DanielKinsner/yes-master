# Coordinator notes: independent Claude review

The owner requested a question-led assessment that does not reveal the previous
investigation's suspected causes or recommendations. This file is for the owner
and coordinator, **not the independent reviewer**.

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
