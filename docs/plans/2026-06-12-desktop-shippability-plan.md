# Desktop Shippability Plan — Windows + macOS (2026-06-12)

Device-scoped execution plan. Decisions recorded 2026-06-12: **v1 public
push is desktop-first** (mobile follows; its plans are authored separately:
`2026-06-12-iphone-shippability-plan.md`,
`2026-06-12-android-shippability-plan.md`). License: source-available
proprietary — `LICENSE` is committed at the repo root. The **adaptive
compressor is in the MVP** — spec:
`2026-06-12-adaptive-compressor-mvp-spec.md`.

Parent documents (read first, in order):
1. `docs/reviews/2026-06-12-master-shippability-audit.md` — findings + evidence.
2. `docs/plans/2026-06-12-shippability-roadmap.md` — slice specs + global
   rules (rule 7: interrogate the owner / `/grill-me` on every taste call).

This plan ORDERS the desktop-relevant slices into a launch sequence; the
slice content lives in the roadmap. Where this plan and the roadmap
disagree on sequencing, this plan wins (it is newer and decision-informed).

## Definition of "desktop shippable"

All of: PRODUCT.md "Release-Candidate Meaning" satisfied on BOTH OSes; the
three P0 corruption/overwrite findings fixed and pinned (RS-01/02/03); the
four FE P1 state bugs fixed and pinned; advertised import formats all
actually decode (XP-01); repo is public-presentable (LICENSE ✓, version,
README, scrub, CI green on windows + macos); adaptive compressor landed
gate-OFF at minimum, with the AC-5 flip done or an explicit owner ship-OFF
decision; owner listening/taste gates explicitly deferred to Wave 10.

## Launch sequence

**Gate A — infrastructure (1 PR each, immediately):**
S0.1 CI (windows + macos + android-host jobs — the macOS job is what
converts "Mac build never verified" PKG-04 from unknown to continuously
proven), S0.2 version coherence 0.1.0.

**Gate B — correctness (the actual ship-stoppers):**
S1.1 Nyquist clamp → S1.2 album overwrite → S1.3 source-overwrite +
tmp/rename → S1.4 decode clamps → S1.5 robustness batch. Then S2.1–S2.4
(frontend P1 quartet). Wave 1 and Wave 2 are independent; run in parallel
if two implementer sessions are available.

**Gate C — contracts:** S3.1 import-format contract → S3.2 event identity
(both commits) → S3.3 recipe parity fixture → S3.4 chrome copy → S3.5
header pin. S3.2 is prerequisite plumbing for cancelability (S6.7) and for
the analysis-batch ids the adaptive work logs.

**Gate D — adaptive compressor (parallel track from here):** AC-1 → AC-2
(after S1.1 lands; same crate) → AC-3 (after Wave 2; same hook file) →
AC-4. Everything gate-OFF; releasable at any moment.

**Gate E — go-public repo:** S5.1 README + scrub (LICENSE already done),
S5.2 CSP, S5.3 lane/docs truth-up, S5.4 canon refresh (REQUIRES the owner
interrogation session — bundle its questions into Listening Session 1's
sit-down; see below).

**Gate F — UX before public:** the default-path trio S6.1 (undo/redo
buttons) + S6.2 (re-analyze) + S6.3 (Standard render progress), then S6.4,
S6.5, S6.9. S6.6 (relink) and S6.7 (cancelability) are strongly
recommended pre-launch but may slip to fast-follow on owner call. S6.8,
S6.10 fast-follow.

**Gate G — cleanup:** Wave 7 in roadmap order (S7.1–S7.7). Net-negative
LOC wave; schedule whenever an implementer session is free — nothing
depends on it, nothing in it blocks launch except S7.3 (clean test lane)
which should land before CI is treated as the release gate.

**Gate H — Wave 10 listening/taste sign-off (owner; deferred 2026-06-16):**

Listening no longer blocks closing the mechanical shippability list. Wave 10
owns the existing Manual Listening Gate items from `docs/RELEASE_STABILIZATION.md`
— normal / already-mastered / long-source sweeps, control sweeps during
playback, long-source seek with Preview LUFS, clean-vs-warning export
comparison by ear — plus the 8 kHz / 11.025 kHz audible proof and preset
voicing work.

Adaptive-compressor calibration, Phase-B confidence, and preset voicing also
live in Wave 10 so the owner can spend a couple days on taste after the app is
otherwise in a complete testable state.

**Gate I — ship:** tag `v0.1.0`; `npm run build:windows` + `npm run
build:mac` artifacts from CI or local; README screenshot; repo public.
Signing/notarization stay deferred per canon (document the unsigned-build
caveat in the README install section).

## Standing notes for the implementer

- The owner listened to Tier-1 adaptive voicing and accepted it
  (2026-06-11 live 96 kHz session; `201e746` exists because of it) —
  recorded in RELEASE_STABILIZATION. Do not reopen Tier-1 constants.
- Oomph remains the least-reference-matched preset; no preset retuning
  outside Wave 10.
- Open owner decisions that do NOT block desktop launch: min-window size
  (default: document 1440×860 requirement in README), internal-docs
  pruning (default: keep + disclaimer), limiter flush RS-09 (default:
  defer, documented), Wave 10 adaptive-engine/menu/listening items.
