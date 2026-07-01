# yes-master — Fable session cook card

> Add-on to [`2026-07-01-yes-master-fable-brief.md`](2026-07-01-yes-master-fable-brief.md) +
> the [Constitution](00-FABLE-SUITE-CONSTITUTION.md). Its structure is good — keep it. This card
> is only the stuff **Fable can't derive from the code**: your intent, and where you want it to
> think bigger than the brief. It does **not** pre-list bugs or "what's stale" — Fable's Step Zero
> reconstructs current state from HEAD itself, and does it better than a hand-written snapshot.

## What only you know (feed this; it's not in the code)

- The finish line is **trust + sellable**, not feature count. Two milestones, kept separate: a
  free time-boxed public beta first (harvest reviews/warm list), then the paid 1.0 flip.
- **Calibrate with measurement + tests, not extended listening** — limited test material and
  time. "The math should suffice." Only genuinely taste-dependent calls get ears.
- Positioning leads with **performance + simplicity + honesty**, not privacy. "Real-time" is a
  literal marketing promise, so it's a real product constraint, not a nicety.
- Sacred, non-negotiable: Original/Mastered preserves playhead; Volume Match off by default and
  never changes export level; exports never overwrite; the Adaptive Compressor (AC-5) + Phase-B
  stay **gated OFF** until an owner listening signoff — do not enable or retune them.

## The one instruction that beats any hand-written status

Trust the code over every doc, **including this brief and my own risk notes** — they drift. Build
your own picture of "where is this really" from HEAD, then plan.

## Cook here — think past my framing

- If this were your product to launch: where's the honest line between the free beta and the paid
  1.0, and what's the single thing most likely to make the launch feel *cheap* vs *trustworthy*?
- The export gate + license activation is the one new, security-sensitive seam and it's the same
  engineering seam as the paywall. Rule on its actual shape (tamper posture, offline behavior),
  and sequence it so the free beta ships **without** it.
- What would you build for this launch that isn't anywhere on my roadmap?

## Cross-repo note (not a dependency)
yes-master is the *furthest along* on release plumbing — its signing playbook
(`RELEASE_SIGNING_SETUP.md`) is the one the sibling apps will copy later. Plan yes-master's
signing/licensing/updater for yes-master; it just happens to become the family reference when you
get to stitching.
