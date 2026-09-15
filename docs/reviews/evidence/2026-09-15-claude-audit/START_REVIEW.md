# Independent mastering-engine audit

Act as an independent audio-DSP reviewer of YES Master. Assess the supplied
production source and audio through your own reasoning and experiments. No
particular defect, preferred sound, or replacement algorithm is assumed.
A well-supported conclusion that current behavior should remain is acceptable.

## Context boundary

Work only in the separate review workspace supplied with this prompt. Read its
`REVIEW_INPUTS.md`, `fixtures/manifest.json`, and `SOURCE_SNAPSHOT.json` first.
It contains application source, dependency locks, input audio and one baseline
export. It deliberately excludes prior research conclusions and experiment code.

Use a fresh conversation without earlier YES research messages, project memory
or summaries. Do not retrieve other YES workspaces, repository history, issues,
PRs, research documents, prior outputs or earlier agents' conversations. This
instruction supersedes directions to consult project status/review documents
for this independent assessment. Public standards, primary DSP literature and
dependency documentation are allowed; do not search for prior YES investigations.

Existing implementation and inline tests are legitimate evidence about the
software, but do not treat comments or passing tests as proof of correctness.
If previous conclusions are supplied accidentally, record what you saw and which
parts of the assessment are no longer independent. Do not claim to unlearn them.

## Questions to answer

1. What does this engine actually promise through its settings and delivered
   output? Which behaviors are intentional musical choices, and which have
   objectively testable correctness requirements?
2. Across a practical variety of valid inputs and ordinary settings, does it
   meet those requirements? Are edge cases handled coherently? Choose and justify
   the test dimensions yourself before examining their results.
3. Which changes to the input or settings should alter processing, and which
   should preserve particular properties? State the proposed invariants and
   their limits before testing them; do not assume all differences are defects.
4. Do independently measured outputs agree with the application's reported
   behavior? When measurements disagree, how can you distinguish an engine
   error, a measurement-method difference and a test-harness mistake?
5. What useful musical effect does each processing stage contribute, and what
   does the complete chain cost in return? Is its automatic behavior justified
   for the requested result across the tested material?
6. What, if anything, should change to make automatic results more dependable
   within the existing user workflow? What evidence supports keeping current
   behavior, and which sources or objectives would proposed alternatives harm?

## Method

Begin with a short, written evaluation protocol: requirements, coverage,
measurements, acceptance criteria, controls, selection rules and a held-out
subset. Record it before deep implementation diagnosis or candidate selection.
Then execute the investigation; do not stop after writing a plan.

Use real music and deterministic synthetic inputs with known expectations.
The supplied music is a convenience corpus, not proof of representative genre
coverage. Distinguish finished tracks from constructed stem balances. Choose
additional appropriately licensed, no-cost sources yourself if an important
coverage gap warrants it. Do not require the owner to supply a large library
or make subtle listening rankings before objective work can proceed.

Exercise the actual production implementations. Build your own isolated harness
if useful. Where practical, reproduce the supplied baseline export from its
recorded settings before trusting that harness. Preserve source bytes and logs;
record any packaging adaptations. A result from copied or simplified DSP must
not silently stand in for the production engine. Record platform/version limits.

Use several relevant criteria rather than one invented quality score. More
dynamic range, smaller waveform residuals, or resemblance to another master
does not by itself establish better mastering. For listening comparisons use
gain-only loudness matching and document unavoidable constraint differences.
Do not add unreported processing to make examples appear comparable.

For every suspected problem, create a minimal reproducible case, test competing
explanations and a useful control, and check whether it appears in realistic
material. Separate proven defects, engineering tradeoffs and unresolved musical
preferences. Report null results and regressions as well as improvements.
Existing passing listening checks are not being reopened by this assignment.

Keep the supplied application source, production defaults, preset calibration
and feature gates unchanged. Put experiments and proposed alternatives in new
files under `audit-output/`. No deployment, release, public push, paid service,
new account, or third-party audio upload is authorized. Marketing, pricing,
website work and broad architectural refactoring are outside this assessment.

## Deliverable and independence checkpoint

Write `audit-output/INDEPENDENT_REPORT.md` with:

- A ranked recommendation, including "keep current behavior" where justified.
- Exact conditions, source/build hashes, commands and retained evidence.
- Achieved results, advantages, costs, regressions and coverage limitations.
- Confidence for each conclusion, what would disprove it, and questions still open.
- An exposure log stating whether any previous conclusions were encountered.

Freeze this report, protocol, test scripts and evidence manifest by recording
their SHA-256 hashes in `audit-output/SEALED_MANIFEST.json`. Finish the independent
assessment before requesting any earlier report. Do not claim to confirm or
refute findings you have not seen.

A later comparison may reconcile agreements and disagreements with other work.
Write that as a separate addendum; preserve the original independent findings.
