# Independent Verification and Fix Prompt — YES Master Adversarial Review

Use this prompt in a fresh agent task with the repository at:

`C:\Users\SM - Dan\Documents\GitHub\yes-master`

---

You are an independent skeptical reviewer and implementation agent. Your job is
to **confirm, narrow, or disagree with** the findings in:

- `docs/reviews/2026-08-20-full-adversarial-review-plain-english.md`

Work in two strict phases. **Phase 1 is read-only:** do not edit files or update
dependencies while deciding whether the earlier reviewer is right. Do not
assume the earlier reviewer is correct. Reproduce every claim from current code
and current external state. **Phase 2 begins only after the evidence is clear:**
implement the agent-fixable findings you confirmed, including findings confirmed
with narrower scope. Do not implement findings you disagree with, cannot
reproduce, or cannot prove.

Throughout both phases, do not change repository or hosting settings, create or
move tags, publish releases, deploy, promote a deployment, change DNS, enable
DSP features, retune presets, claim listening approval, or manufacture owner-only
evidence. Do not commit or push unless the user separately asks you to.

## Start with current truth

1. Confirm the checkout path, branch, worktree status, remotes, and current HEAD.
2. Preserve all user changes. Run `git pull --ff-only` only if the worktree and
   branch make that safe; otherwise report why you did not pull.
3. Read `AGENTS.md` and its full Required Reading list in order.
4. Treat current code, current active docs, live tests, live GitHub state, and
   historical records as separate evidence streams. Historical prose is not
   current proof.
5. Record the exact commit, OS, Node, Rust, browser, and relevant tool versions
   used for verification.

## Review each finding adversarially

For every item below, return one verdict:

- **Confirmed** — reproduced as stated.
- **Confirmed but narrower** — real, but impact or scope was overstated.
- **Not reproduced** — the stated reproduction does not work on current HEAD.
- **Disagreed** — the behavior exists but is intentional, safe, or not a defect;
  explain the contract and evidence that wins.
- **Blocked** — cannot verify without owner input, credentials, hardware, or a
  state-changing action.

### A. Dependency and security-gate claims

1. Run `npm audit --audit-level=high` from the repository root.
2. Identify the exact dependency paths for every high or critical result and
   distinguish runtime dependencies from build/dev-only dependencies.
3. Run the available Rust advisory scanner without installing a new tool. If
   `cargo deny` is available, run `cargo deny check advisories` from
   `src-tauri` and separate actual vulnerabilities from unmaintained-package
   warnings.
4. Prove or refute whether `quick-xml` is reachable from any YES Master path
   that parses user-controlled XML. Do not infer runtime exposure merely from
   its presence in `Cargo.lock`.
5. Inspect `.github/workflows/*.yml` and determine whether JavaScript and Rust
   advisory checks run in CI or release preflight.
6. Read live GitHub security settings with `gh` if authorized. Verify the state
   of Dependabot alerts/security updates, secret scanning/push protection,
   CodeQL, private vulnerability reporting, and `SECURITY.md`. Do not enable
   anything.

### B. Keyboard and accessibility claims

1. Inspect the actual shortcut handlers and their tests.
2. Build and serve the real production web app. Use Playwright or another real
   browser, not jsdom alone.
3. With keyboard focus on each relevant native control, press Space and Enter.
   At minimum cover a button, checkbox, range input, and select. Record whether
   the native action occurs and whether transport also changes.
4. Verify whether `A`, `L`, and `?` are global single-character shortcuts and
   whether the user can disable, remap, or focus-scope them.
5. Compare the actual behavior to WCAG 2.1.4 and platform conventions. Keep
   legal/conformance claims narrower than the evidence supports.
6. Check whether existing axe/headless coverage could detect these dynamic
   keyboard failures.

### C. Loop-state claim

1. Open an analyzed track in Advanced mode with no waveform region.
2. Confirm that the loop button is disabled.
3. Press `L` and inspect both visible state and backend calls.
4. Draw a region afterward and determine whether looping becomes active without
   another explicit enable action.
5. Force `setLoopRegion` to reject and determine whether the optimistic UI state
   rolls back.
6. Inspect the current tests and state whether they prevent or encode the
   disputed behavior.

### D. Release-workflow supply-chain claim

1. Inventory every `uses:` reference in CI and release workflows.
2. Identify which references are immutable full commit SHAs and which are
   mutable tags or branches.
3. Trace the effective `GITHUB_TOKEN` permission for each release job.
4. List exactly which secrets are exposed to which action or command on each
   matrix platform. Do not assume an empty secret is configured.
5. Verify whether Azure credentials are available to the macOS action when the
   paid signing overlay is configured.
6. Inspect the `artifact-signing-cli` installation for exact version and lockfile
   reproducibility.
7. Compare the result to current official GitHub Actions and Cargo guidance.
   Distinguish a theoretical hardening opportunity from a practical release
   blocker.

### E. Release-document truth

1. Compare the top-level boxes, exact-commit evidence ledger, candidate-freeze
   section, active quality-plan ledger, owner queue, and current code version.
2. Confirm or refute whether old 0.9.1 evidence can be mistaken for current
   0.9.2 proof.
3. Compare `README.md` with the active plan and current candidate state.
4. Do not rewrite owner decisions or treat an old historical row as false merely
   because it is old. The question is whether its current presentation is
   misleading.

### F. GitHub release and public-web truth

1. Check live GitHub Actions at exact HEAD and list current releases/drafts.
2. Check GitHub deployment records for the exact HEAD.
3. Fetch `https://yes-master.vercel.app/` and compare its HTML plus referenced
   JavaScript/CSS artifacts with a clean local production build. Explain any
   environment-related non-determinism before calling it stale.
4. Run the repository's production landing smoke command against the public
   alias.
5. Resolve A, AAAA, CNAME, MX, and relevant TXT records for `yesmaster.app`.
6. Verify whether canonical, social-image, contact, or legal surfaces depend on
   that domain.
7. Use the Vercel CLI only for read-only inspection and only if already installed
   and authenticated. Do not start a login flow, promote a deployment, pull
   secrets, or deploy.

### G. Product and architecture boundaries

1. Confirm that the DSP/export, mobile-bridge, and owner-gate claims are based on
   current tests rather than old prose.
2. Keep listening, taste, real-device installation, updater installation,
   signing, and publication evidence explicitly owner-only.
3. Inspect large files and recent churn, but do not call size alone a defect.
   Recommend a refactor only when a concrete bug or ownership problem supports
   it.

## Phase 2 — implement only confirmed, agent-fixable findings

Before editing, write a short phase-boundary summary in your task commentary:

- what you confirmed;
- what you narrowed or rejected;
- what remains blocked or owner-only;
- the exact fixes you will make.

Do not stop at that checkpoint or ask for fresh approval for fixes already
within this prompt's scope. If at least one finding is confirmed and agent-fixable,
continue directly into Phase 2.

Then implement the confirmed work in small, reviewable vertical slices. Use
test-driven development for behavior changes: add one public-behavior regression
test, run it and capture the expected failure, make the smallest production
change, and rerun it before moving to the next behavior. Do not refactor unrelated
code while a test is red. Preserve all pre-existing user changes.

The earlier review proposed the following possible implementation areas. This is
a claim list, not an instruction to blindly make every change:

1. **Native keyboard behavior** — prevent the global Space transport shortcut
   from swallowing Space on native or ARIA-interactive controls, including a
   child element inside a control. Preserve Space transport on a neutral app
   surface. Replace any confirmed bare single-character shortcuts with documented
   modifier-based shortcuts so the behavior complies with the applicable
   WCAG 2.1.4 contract. Update the shortcut overlay and real-browser coverage.
2. **Loop state** — if confirmed, make the keyboard path obey the same
   no-region rule as the disabled loop button, prevent a later region selection
   from silently activating a previously hidden loop state, and make backend
   rejection leave or restore truthful UI state. Replace tests that encode a bug
   with tests for the intended contract.
3. **Dependency advisories** — update only the necessary direct or lockfile
   dependencies to patched compatible versions. Do not hide an actual
   vulnerability with an ignore. Where an unmaintained transitive package is
   genuinely platform-deferred and not practically replaceable, document that
   distinction rather than presenting it as a current exploit. Keep the npm and
   every affected Rust lockfile consistent.
4. **Security regression gates** — add focused, reproducible npm and Rust
   advisory checks to an appropriate CI or preflight lane if their absence is
   confirmed. Pin any installed audit tool to an exact version and use its lock
   mode. Avoid a gate that fails solely on documented unmaintained transitive
   packages when the intended policy is to block known vulnerabilities.
5. **Release workflow hardening** — if confirmed by current official guidance,
   pin third-party actions to immutable full commit SHAs with readable version
   comments, reduce the default `GITHUB_TOKEN` permission to read, grant write
   only to jobs that demonstrably need it, scope platform-specific secrets to the
   platform steps that consume them, and pin Cargo-installed release tools to an
   exact version with `--locked`. Do not invent a GitHub environment or silently
   change the release contract; list repository-setting protections as owner
   actions instead.
6. **Release and README truth** — reconcile current 0.9.2 candidate boxes and
   exact-commit evidence without deleting valid 0.9.1 history. Old evidence must
   remain clearly historical and must not read as proof for an unverified current
   candidate. Point the README at the active plan and current gate. Do not check
   a box unless the required current evidence actually exists.

If a technically reasonable fix requires an owner product decision — for
example choosing a new shortcut scheme where no current contract exists — do
not guess. Record the smallest concrete decision needed in
`docs/OWNER_INPUT_QUEUE.md` using its existing format, and continue with other
confirmed work.

Research claims about GitHub Actions, WCAG, Cargo, npm, or Vercel against current
primary/official documentation. Cite the exact official pages that influenced a
change. Research is evidence, not permission to mutate external state.

## Minimum verification lanes

Run the repository's documented fast lanes plus the slow fixture lane when the
private fixture directory exists. Include frontend tests/build, headless browser
checks, Rust formatting/lint/tests, iPhone bridge checks/tests, Android host and
ARM64 checks, Windows packaging on Windows, live exact-HEAD GitHub Actions, npm
audit, and the available Rust advisory scan.

Do not substitute these lanes for real-machine installation or owner listening.

## Required output

Lead with one of:

- **The earlier no-go verdict stands.**
- **The earlier verdict should be narrowed.**
- **The earlier verdict is not supported.**

At the Phase 1 boundary, provide:

1. A finding-by-finding verdict table with exact evidence.
2. Reproduction commands and relevant file/line references.
3. False positives or overstated claims from the earlier review.
4. Newly discovered problems, clearly separated from confirmation work.
5. A release recommendation that separates:
   - must fix before a new candidate tag;
   - should fix before public publication;
   - reasonable post-beta hardening;
   - owner-only gates.
6. The initial verification matrix, including failures and skipped lanes.
7. Confirmation that Phase 1 changed no files or external state.

After completing Phase 2, give a self-contained final report that repeats the
verdict and release recommendation, then adds:

1. A complete list of changed files and the reason for each change.
2. The red-then-green evidence for every behavior fix.
3. Dependency and workflow before/after evidence, including exact resolved
   versions and action SHAs.
4. The final verification matrix, with every failure or skipped lane stated
   plainly.
5. A separate owner-action list for settings, credentials, deployment, DNS,
   signing, publication, real-device checks, and listening gates that you did
   not alter.
6. Confirmation that no external state was changed and no commit or push was
   made.

If no finding survives independent verification, make no code changes and say
so. If only some survive, fix only those. Accuracy outranks agreement and scope.

Be willing to disagree. A useful result is accurate, not agreeable.
