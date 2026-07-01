# YES Suite — Fable 5 planning package

The full set of Fable-5 plan-elicitation artifacts for the whole YES family, plus the cross-repo
analysis that produced them. Built 2026-07-01 by distilling four independent Opus-4.8 per-repo
briefs and grounding each against its real code.

## The core idea

Fable 5 is scarce (7-day window, 50%-usage cap). Spend it on **expensive-to-reverse judgment**;
let cheaper models (Opus) implement. Four independent sessions converged on the same six planning moves — strong evidence those moves
are right, so they're written down once (the Constitution) and inherited. The apps ship
**independently first**; how they stitch together is a later, optional pass.

## The artifacts (and how they fit together)

| File | Role |
|------|------|
| [`00-FABLE-SUITE-CONSTITUTION.md`](00-FABLE-SUITE-CONSTITUTION.md) | The reusable *how-to-plan* half. Load it into **every** Fable session. Merges the best move from each original brief + the five fixes (drift-audit-first, per-fork altitude, invalidation-trigger ledger, handoff-fidelity self-test, audit-the-locked-decisions). |
| [`01-SUITE-ARCHITECT-BRIEF.md`](01-SUITE-ARCHITECT-BRIEF.md) | **Optional, later** "stitching" session — how the four fit together. Per the owner's plan (ship each app independently first), this is *not* run first and not a prerequisite; the two real cross-repo transfers it would surface are already inlined into the cook cards. Run it only once ≥2 apps are shipping. |
| [`02-yes-master-brief-upgrade.md`](02-yes-master-brief-upgrade.md) | yes-master "cook card" — owner-intent + generative prompts to add to its session. |
| `…/yes-daw/docs/fable5/upgrade-patch.md` | yes-daw cook card. |
| `…/yes-stems/docs/planning/fable5-brief-upgrade.md` | yes-stems cook card. |
| `…/yes-voice/docs/planning/fable5-brief-upgrade.md` | yes-voice cook card. |

Each per-repo session loads: **the Constitution** + **that repo's original brief** + **that repo's
cook card** + **the suite session's frozen output**.

The cook cards are deliberately thin. They carry only what Fable **can't** get from the code — your
intent, what's sacred, and where you want it to think bigger than the brief. They don't pre-list
bugs or "what's stale": Fable reconstructs current state from HEAD itself, and does it better than
any hand-written snapshot.

## Run order for the 7-day window (each app independent first — your plan)

Spend the budget per-app; the biggest open one-way forks deserve the most. A reasonable order:

1. **yes-stems** + **yes-voice** — biggest open forks (stems' memory/resource seam; voice's
   plan-vs-reality reconciliation). Give these the most budget.
2. **yes-daw** — plugin-hosting + user-data/file-format safety.
3. **yes-master** — shortest; late-stage, mostly sequencing + a drift audit.
4. *(optional, later)* **stitching pass** — only once ≥2 apps are actually shipping.

## The one rule that matters most

**Give Fable intent; let Fable do the analysis.** The briefs carry the owner-only context (goals,
constraints, what's sacred) and set it loose on the judgment. They deliberately *don't* pre-chew
the repo — that's the work you're paying the genius to do. Since Fable is agentic, one instruction
covers it (the Constitution's Step Zero): *before inheriting any fact or decision, reconstruct the
real state from the code at HEAD; trust the code over every doc, including the brief.*
