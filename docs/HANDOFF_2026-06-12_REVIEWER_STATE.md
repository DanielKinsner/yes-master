# Reviewer Handoff — Claude oversight state (2026-06-12)

For the next Claude (Fable/Opus) instance taking over the REVIEWER role in
the shippability push. The implementer is Codex GPT 5.5; you review its
commits, you do not implement. The owner (Dan) is hands-off except
listening sessions and [decision] items; he prefers plain-language
summaries and defers technical judgment to you.

## Your role and protocol

- Review each Codex commit against its slice spec in
  `docs/plans/2026-06-12-shippability-roadmap.md` using that file's
  "Review protocol" section. Slice IDs are in commit titles.
- Evidence-based: lane output must be pasted in commit bodies; test counts
  only grow (frontend was 394, rust lib 311+ at last review).
- Hard rule: any commit updating DSP snapshot values = automatic rejection,
  EXCEPT the AC-5 calibration commit (see below) and the already-completed
  PKG-04 macOS recording (0d630f1 — that one was protocol-verified:
  max delta 2.38e-7, evidence in its commit body).

## State at handoff (evening 2026-06-12)

Reviewed and APPROVED by me, commit-by-commit:
- Gate A (c488648, 4ec3bc3, 7ef296b) — CI, version 0.1.0.
- PKG-04 (d4c1c60, 0d630f1) — real macOS snapshot values recorded.
- Gate B (db4f82d, b912a95, dc1e4dc, f901449, a754221, 5663b06, 7c0473f,
  b7206b5, 0582452, 31cf7ad, 58d5847, 8573792, af819af) — all Wave 1 + 2.
- AC-2 (173570d) — closely reviewed: gate defaults OFF, reduce-only is
  structural (threshold_lift.max(0.0), ratio floored at 1.0), byte-identity
  tested adversarially with stale guards. Excellent.

Landed but only STRUCTURALLY reviewed (order/diffstat/test-growth, not
hunk-by-hunk): Gate C (739b4c1, e105756, 0d65d6f, c65b8b9, e9062a3,
93a8e04) and AC-1 (ca2b77b). If you want one deep-read, pick S3.2
(e105756 + 0d65d6f, event identity) — it rewires analysis ownership in
useTrackMaster.ts, the file with the most async-state history.

In flight (two prompts issued, may or may not have run yet):
1. A short docs-only session: local-path scrub (9 docs), .gitignore
   comment, TESTING.md android lane, IPHONE_APP.md preset table,
   verify-fast.ps1 android lane (= S5.1-scrub + S5.3). Many tiny commits
   by design (owner's request).
2. The main session: AC-3 → AC-4 → S5.2 (CSP) → S6.1/S6.2/S6.3, then STOP
   and prepare (not run) the calibration session. It was told to write a
   HANDOFF doc if it nears limits — look for new docs/HANDOFF_2026-06-12*.

## What to watch next

- **AC-3:** frontend must read backend-resolved guard values via the new
  command; any TypeScript re-implementation of the mapping = reject (that
  is how cross-platform parity debt starts).
- **AC-4:** bit-equality bridge tests on both mobile platforms with the
  gate forced ON.
- **AC-5 is the one sacred checkpoint:** it flips the adaptive-compression
  default ON, locks constants, and is the ONLY commit allowed to change
  DSP snapshots. It must NOT run until the owner completes the calibration
  listening session (spec §5 of
  `docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md`). Prompt it
  SOLO, never in a batch. Private fixtures for that session live only in
  `private-audio-fixtures/` on the home machine (gitignored by design).
- **S6.1** must also fix the stale undo/redo claim in
  docs/RELEASE_STABILIZATION.md (same commit).
- **S5.4 (canon refresh)** stays blocked until the owner interrogation
  (use /grill-me); its open questions: mobile product promise, Album
  Master promise, adaptive-engine canon wording, mode-pill label.

## Owner-pending items (do not nag; they're queued)

Listening Session 1 (manual gate sweeps + interrogation questions),
Session 2 (AC-5 calibration, after AC-4 + prep), and the low-stakes
defaults already recorded in the roadmap's decision queue (min-window,
internal-docs pruning, limiter flush RS-09 — defaults apply if unanswered).

## Map

Done: Gates A, B, C; AC-1, AC-2; README rewrite (50b7fe8); LICENSE;
PRODUCT.md partial canon refresh (Standard-first, adaptive section,
mobile/album stubs). Remaining to desktop ship: AC-3..AC-5, Gate E
remainder (S5.2 + owner-gated S5.4), Gate F (S6.x), Gate G cleanup
(Wave 7), listening gates, tag v0.1.0. Mobile afterward per the per-device
plans. Source of truth chain: master audit → roadmap → per-device plans →
compressor spec; when prose disagrees, newer doc + current code win.
