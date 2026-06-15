# Handoff — 2026-05-28 (UI pass + independent-review fixes)

Audience: the next agent picking up YES Master on a fresh machine. This is a
pointer document — it references existing artifacts rather than repeating them.
Read `CLAUDE.md` first (required reading + verification commands).

## Where things stand

- Branch: everything is **merged to `main` and pushed to `origin/main`**.
  Working tree is clean. Latest commit at handoff time: `66a0f60`.
- Two bodies of work landed this session, both fully verified and pushed:
  1. **Independent code-review fixes + fixture-matrix evidence.** See
     `docs/review-2026-05-28.md` (the original review, all 15 items addressed),
     `docs/RELEASE_EVIDENCE_2026-05-28.md`, and `docs/RELEASE_STABILIZATION.md`
     (the "Already-Mastered Input Matrix" gate is now marked complete — full
     18-case run passed, no silent regression).
  2. **A UI pass** (commits between `b4fa970` and `66a0f60` — run
     `git log --oneline b4fa970..66a0f60` for the list). In order: honest
     MASTER OUT meter, then true L/R stereo peak metering (real per-channel
     peak added to the audio thread), distinct idle/live/rendering status
     dots, undo/redo buttons removed (keyboard shortcuts retained), preset
     cards ~13% larger, Visual EQ de-toyed (muted palette + flat ring nodes),
     EQ nodes shrunk, `universal.png` normalized to 1024² + all preset images
     +15%, and a final dead-CSS sweep.

Per-change detail lives in the commit messages — read those rather than asking
for a recap.

## What is NOT done / open items

- **Manual listening gate** — still pending and only a human can clear it.
  See `docs/RELEASE_STABILIZATION.md` "Manual Listening Gate" and the plan's
  Done Criteria. Nothing automated can approve taste.
- **Album Master** delivery-format parity remains deliberately deferred (see
  the plan's Scope Boundaries). Do not widen Album scope without the user.
- **Oomph preset** is the least-matched against references; do not retune any
  preset without fresh listening notes (plan Key Decision 7).
- **Active plan of record:** `docs/plans/2026-05-28-001-release-candidate-finish-plan.md`.
  Most of its units are complete in prior sessions; treat it as the source of
  truth for remaining RC scope, not this handoff.

## Known cosmetic / nice-to-have (not bugs)

- The Visual EQ SVG uses `preserveAspectRatio="none"`, so circular nodes render
  as slight ellipses. Making them perfectly round means reworking how the plot
  scales — bigger surgery, left intentionally untouched.
- `src/components/VisualEqPanel.tsx` node palette is muted-but-distinct; the
  user likes a "touch of color." If asked for more restraint, it's a one-line
  palette tweak.

## Gotchas a fresh agent will hit

- **Responsive CSS has FOUR viewport regimes** in `src/App.css`
  (`@media` at ~1280+/820+, 1700+/960+, 1280–1699/820+, and a compact
  fallback for <1280 or <820). Preset/tile sizing is defined independently in
  each — a change in one regime is invisible in the others. The compact/base
  rules govern short screens (height < 820), including the 1280×800 preview.
- **Assets:** edit images in `src/assets/...`, never `dist/` (Vite regenerates
  `dist/` with content-hashed filenames every build; `dist/` is gitignored).
- **Preview screenshots are flaky** with this app at large/odd viewports
  (capture times out — heavy preset PNGs + large DOM). Reliable workaround:
  use `preview_eval` to read computed styles / element sizes as objective
  verification, and screenshot at the 1280×800 desktop preset when you need a
  picture.
- **Never commit** private audio, rendered masters, or `test-output/` (all
  gitignored); commit only aggregate evidence into docs.
- Verification (the gate before any "done" claim) lives in `CLAUDE.md` /
  `docs/TESTING.md`. Backend tests use `--target-dir target\codex-rc` to avoid
  locking a running app on Windows.

## User working style

A memory note already exists (auto-memory `user-communication-style`): the
user prefers plain-language "so what" explanations over deep technical detail
and defers technical-judgment calls to the agent. Lead with the bottom line.

## Suggested skills for the next session

- `superpowers:verification-before-completion` — invoke before any completion
  claim, commit, or merge. This was used throughout; keep it up (run the gate,
  read the output, then claim).
- `superpowers:brainstorming` — before building any NEW feature/UI, not just
  tweaking existing behavior.
- `superpowers:systematic-debugging` — if a bug/regression surfaces.
- Preview/verify tooling — `mcp__Claude_Preview__*` (start/eval/screenshot)
  for UI work, or the project `verify` / `run` skills to drive the real app.
- `superpowers:requesting-code-review` / `finishing-a-development-branch` if
  opening more branches/PRs.

## First moves for the next agent

1. `git log --oneline -15` and skim the recent commit messages for full detail.
2. Read `CLAUDE.md`, then the plan and `docs/RELEASE_STABILIZATION.md`.
3. Ask the user what the next focus is (the RC's remaining work is mostly the
   human listening gate; new feature work should start with brainstorming).
