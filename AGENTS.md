# YES Master Agent Instructions

This is the active YES Master repo. **AGENTS.md and CLAUDE.md must stay
byte-identical — edit both together.**

## Judgment and autonomy

Explore broadly and choose implementation details autonomously within the user's
task. Plans describe outcomes, dependencies, and starting hypotheses, not mandatory
code structure. Improve or replace an approach when evidence supports it; explain
meaningful changes to the approach and preserve the intended product contracts.

- Current user instructions and recorded decisions supersede older process rules.
  Check existing authorization before asking; do not ask again for an authorized step.
- DSP correctness fixes and performance research may include algorithms, data
  layouts, SIMD, caching, and bounded parallelism. A file, constant, or subsystem
  is not off-limits merely because it affects sound. Prove the correction or
  improvement; intentional preset voicing or sonic tradeoffs remain owner choices.
- Continue through authorized work after verified checkpoints. Respect an explicit
  user stop or scope limit; a routine progress report is not a request for permission.
- For an unresolved owner choice, pause only dependent work and continue independent
  work. Preserve existing behavior where appropriate without treating it as the
  owner's answer. Do not disable working features simply to be "conservative."
- Update affected internal docs with an authorized decision/change. Ask about an
  unresolved decision, not whether to document one already made. Public claims
  must still match verified functionality and publication authorization.

## Read what the task needs

Start with the user's request and the latest relevant status in
`docs/OPEN_THREADS_AND_DECISIONS.md`. Read the sections needed to understand the
change; do not ingest every linked document for every task or reread unchanged
material already understood in this session.

| Task | Read as needed |
| --- | --- |
| Behavior or product decisions | `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md` |
| Implementation and verification | `docs/ARCHITECTURE.md`, `docs/TESTING.md` |
| Current audio/listening work | `docs/RELEASE_STABILIZATION.md`, the handoff and plan below |
| An unanswered owner question | `docs/OWNER_INPUT_QUEUE.md`, then existing decisions/notes before asking |
| Release or beta work | Current quality-plan ledger and `docs/plans/beta-go-no-go.md` |
| Marketing/business claims | `docs/CAPABILITY_EVIDENCE_MATRIX.md`, relevant sections of `docs/plans/2026-06-30-launch-plan.md` |

### Which beta document is authoritative

- `docs/plans/2026-07-07-beta-execution-plan.md` is **executed history.** Consult
  it for decisions D1–D16, not an open work queue.
- `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md` is the **active
  forward queue.** Resume from current status, not the original unchecked outline.
- `docs/plans/beta-go-no-go.md` is the **live release gate.** Its actual evidence
  requirements remain; a local fix does not activate a release.

`docs/CHANGELOG.md` records shipped history; `docs/IDEAS_BACKLOG.md` holds ideas;
`docs/archive/` holds retired plans. Historical deferrals are context, not a ban
on investigation. Reassess an old implementation choice when the current task
and evidence justify it; do not execute unrelated parked backlog by default.

## Product contracts and owner decisions

- Local-first desktop mastering; Mac and Windows stabilize first. Linux is deferred
  and mobile product expansion stays parked. The landing page **is** in agent scope.
- Prioritize Track Master stabilization for the current task. Audition stays responsive;
  Original/Mastered switching preserves playhead. Volume Match is optional, off by
  default, audition-only, and never changes export level.
- Exports protect source files and prior renders by default. Quality warnings are
  advisory unless output is technically invalid. Allow bold processing with truthful
  meters, warnings, and review states.
- Preserve intended sound and measurement contracts while optimizing. Do not retune
  presets or activate Adaptive Compressor, Phase-B confidence, or album-character
  behavior without the required owner decision/listening signoff. Keep the Adaptive
  Compressor's gated `TBD-CALIBRATION` constants unchanged pending that signoff.
- Do not commit private audio/masters without explicit approval for those fixtures.
  Use existing owner-supplied files where available; ask only for a specific missing
  source needed to resolve a finding.
- Public push, release activation/publication, deployment, spending, and unresolved
  business/signing/updater decisions require applicable user authorization. Reuse
  authorization already given; an internal code/docs change is not publication.

## Listening already conducted

**Latest: September 9 installed Windows baseline PASSED.** The owner completed
all five checks on installed 0.9.2 / `07021f1b` and reported all passed. Read
`docs/listening/2026-09-09-installed-check.md` for exact scope and artifact proof.
This closes that Windows questionnaire. Continue the requested export-format
expansion; test affected behavior without repeating unchanged listening gates.
Future formats, Mac installers and release/updater activation require their own
applicable evidence; this signoff does not publish a release.

The owner completed the **2026-09-05 native Windows listening session** via
`npm run tauri dev`, Focusrite USB/studio monitors, owner-verified `e600a21`.
Read [the reconciled handoff](docs/listening/2026-09-05-owner-handoff.md),
[follow-up plan](docs/plans/2026-09-05-listening-follow-up.md), and
[existing fixtures](docs/listening/2026-09-05-performance-fixtures.md) for this work.

Normal A/B, normal musical contrast, and the observed Track Master export comparison
passed. Do not call the session missing, repeat the questionnaire, or ask for details
already supplied in prose. The September 5 verdict was "Not ready / stopped here";
the later September 9 installed pass above supersedes it for the tested Windows
baseline. Target new checks to unresolved or changed behavior. Preset taste stays as-is.
September 4 implementation history is in `docs/plans/2026-09-04-audio-correctness.md`.
Release activity remains parked.

## Verification and integration

- Use focused tests/experiments during iteration and affected suites at coherent
  checkpoints. Follow the scope matrix and commands in `docs/TESTING.md`; full
  packaging/platform lanes belong at applicable integration/release checkpoints,
  not before every small commit. Do not repeat passing work without a relevant change.
- Add a meaningful automated regression for a reproducible objective defect where
  feasible; reuse existing coverage and record native/manual evidence for behavior
  that a test cannot establish. Never weaken a contract or hide a failure to pass.
- Rendered UI changes require `npm run verify:headless` before that UI slice is
  complete. Shared type/behavior changes require affected bridge checks before
  integration; DSP/export/audition-trust changes require the fixture lane before
  merge. Keep native, listening, browser, and exact-commit CI evidence distinct.
- Commit small, coherent, independently working changes. Inspect git status and
  preserve unrelated work. Check remote divergence when integration needs it;
  never blindly pull into a shared dirty checkout or stage another task's edits.
- Report what changed, what the evidence establishes, and precise remaining limits.
  Record actual completed work in the live queue; keep speculative causes labeled.
