# Adaptive DSP Post-Review Commit Stack Review

Date: 2026-06-03
Branch: `feat/adaptive-dsp-guardrails`
Reviewed range: `f6c7cc8` through `a96bf5f`, with emphasis on the latest local
post-review commits.

## TLDR

The recent stack lands in a much better place than the original adaptive pass.
The real review blockers were addressed: `LRA=0.0` no longer forces density
trims, offline preview and the slow evidence lanes now exercise the adaptive
chain, the bright deadband no longer catches a pink-neutral source, and the UI
now exposes per-axis chain trims.

I do not see a P0/P1 blocker in the adaptive math itself. The remaining issues
are boundary hardening and documentation hygiene. The highest-risk gap is that
Album Master is labeled "unadapted" by product decision, but the backend does
not enforce that boundary if a settings payload already carries
`advanced.source_profile`. The second real gap is stale profile handling: the
frontend injectors add `source_profile` when analysis is complete, but they do
not explicitly clear it when analysis is missing.

My read: this is close to mergeable after two small code hardening fixes, the
doc cleanup below, and the owner listening/slow-fixture gate.

## Verdict

The strategy is still sound for Tier 1: defensive guardrails that only reduce
preset moves toward neutral, with caps and floors preserving preset character.
The post-review commits moved the implementation from "promising but not
release-evidenced" to "mostly wired and test-backed."

What is not finished is source-profile ownership. The tactical fixes are good,
but the repo still has a hidden per-track analysis snapshot flowing through
settings, with builders in both TypeScript and Rust. That is acceptable for this
slice if the edges are sanitized, but it is not the final architecture.

## Findings

### P2 - Album "unadapted" is not enforced at the backend boundary

Album Master is now explicitly a Track-Master-only/non-adaptive scope decision in
the UI/docs. Normal frontend album export appears to honor that because
`exportAlbumPlan` passes `albumIntent`/per-track settings without calling
`injectSourceProfile`.

The backend, though, does not enforce the decision. `apply_album_shadow` clones
the incoming `MasteringSettings`, applies album arc/character changes, and
returns it without clearing `advanced.source_profile`
(`src-tauri/src/album_render.rs`). The new test named
`album_shadow_is_profile_agnostic_so_album_stays_unadapted` proves this helper
does not inject a profile, but it also proves it preserves one if already
present.

That means a stale/project/API payload with `advanced.source_profile = Some(...)`
would make album rendering adaptive despite the product label.

Recommendation: if Album Master is intentionally out of scope, strip
`shadowed.advanced.source_profile = None` inside the album render path and add a
test that a profile-bearing input is cleared before chain construction. If Album
is later brought back into scope, make that a deliberate per-track injection
path instead.

### P2 - Source-profile injection does not clear stale profiles

`injectSourceProfile` returns the original settings object when analysis lacks a
6-band balance. `applyChainDispatchOverrides` similarly only writes
`source_profile` when `sourceProfileFromAnalysis` returns a profile.

That is usually fine because normal UI settings do not appear to store injected
profiles permanently. But `advanced.source_profile` is a serde/TypeScript field
on `AdvancedSettings`, so if it ever enters project state, a preset, undo state,
or a hand-constructed API payload, a later "no analysis/no 6-band" path will not
clear it.

Recommendation: make both TS helpers explicit:

```ts
advanced: { ...settings.advanced, source_profile: profile }
```

where `profile` is `null` when analysis cannot build one. Longer term, move
profile ownership behind a backend helper so every render/playback entry point
gets either the current profile or an explicit `None`.

### P3 - Handoff and plans reference review docs that are currently untracked

`docs/HANDOFF_2026-06-02_ADAPTIVE_DSP_TIER1.md`,
`docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`, and
`docs/ADAPTIVE_DSP_NEXT_STEPS.md` reference review docs under `docs/reviews/`.
In the current worktree, those review docs are untracked.

If the branch is pushed with only the committed docs, these links will point to
files that are not in repo history.

Recommendation: either commit the referenced review docs as part of the review
trail, or remove/soften the links. The new next-steps doc also says
`...GLOBAL-review.md` is "untracked" and "unread"; that is honest locally, but
it should not ship as stable project documentation unless that state is
intentional.

### P3 - The handoff still contradicts the current state

The handoff says the per-axis "what was trimmed" readout shipped, then later
lists "What was trimmed and why" as deferred. The new
`docs/ADAPTIVE_DSP_NEXT_STEPS.md` correctly marks the readout as built, but the
handoff still has both states.

Recommendation: update the handoff deferred list to remove the readout item, or
reword it as "richer/next-gen readout" if there is still a future transparency
slice.

### P3 - The backlog's backend-profile note blurs the Album decision

`docs/ADAPTIVE_DSP_NEXT_STEPS.md` is useful as a single entry point, but the
backend-owned-profile item says it would close "preview/album/slow-lane/live by
construction." That is only true if Album Master becomes adaptive. The current
owner decision is the opposite: Album Master is intentionally unadapted.

Recommendation: split the architecture note into two explicit outcomes:
backend-owned profile should close Track Master preview/export/live and slow
lanes by construction, while Album should either strip profiles by construction
or have a separately approved per-track profile path.

## What improved

- `LRA=0.0` sentinel handling now avoids the original false full-density trim.
- Rust `SourceProfile::from_analysis` is no longer dead; slow fixture/reference
  lanes use it, so release evidence now exercises the adaptive chain.
- Offline Track Preview now uses the same injected source profile as Track
  Master export, closing the earlier WYSIWYG gap.
- Track Master live/play and export paths both carry the source profile when
  analysis has a 6-band balance.
- Bright deadband moved from `0.20` to `0.30`, with a pink-neutral regression
  test.
- Per-axis readout is computed by Rust and labeled as chain trims before LUFS
  landing, which matches the actual guarantee.
- The new next-steps doc is valuable: it gives future agents one place to see
  what is done, what is owner-locked, and what remains Tier 1 vs Tier 2.

## Math and wiring assessment

The coefficient-side math still checks out for the Tier-1 promise:

- Positive preset high/air and low/sub boosts are scaled down, never flipped.
- Preset cuts are untouched.
- User manual EQ offsets are not second-guessed.
- Preset width is pulled toward neutral only when the preset widens above `1.0`.
- Compression softening applies in Preset mode, not Manual mode.
- Strength `0` remains inert even with a trigger profile.

The important caveat remains: these are pre-landing chain trims. Delivery LUFS
landing can still recompose final broadband loudness after the adaptive chain.
That is acceptable if the UI/readout continues to describe "chain trims" rather
than promising final tonal/loudness deltas.

## Recommended next actions

1. Clear `source_profile` in album render if Album stays out of scope.
2. Clear stale `source_profile` in TS injection helpers when analysis cannot
   build a profile.
3. Commit or de-reference the review docs currently linked from committed docs.
4. Fix the handoff/readout deferred contradiction.
5. Tighten the next-steps backend-profile note so it does not imply Album will
   be adapted by the refactor.
6. Run the owner listening gate and slow fixture lane before merge.

## Verification performed

Read-only review plus lightweight hygiene:

- `git diff --check 6480d96..HEAD` passed.
- No full `npm`/`cargo` suite was run during this review pass.

