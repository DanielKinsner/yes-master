# Fresh-agent start: plan implementation from the completed mastering research

Use this prompt after pulling `main` on the receiving machine. The investigation
concluded September 15 UTC / September 14 evening Pacific. Start here rather
than restarting the older continuation prompt or the blind review.

## Copy/paste prompt

Continue YES Master's mastering-quality work from the completed research.
Inspect the checkout and preserve existing changes. Read `AGENTS.md`, the latest
relevant status in `docs/OPEN_THREADS_AND_DECISIONS.md`, then:

1. `docs/reviews/2026-09-15-mastering-quality-final-synthesis.md` — authoritative
   conclusions, ranked proposals, evidence limits, regressions and planning slices.
2. `docs/reviews/2026-09-15-mastering-quality-final-verification.md` — final
   measurement corrections and freshly verified scope.
3. `docs/reviews/2026-09-15-mastering-quality-transfer.md` — restore the existing
   private GitHub archives yourself when the fixtures are absent; authorization
   for transporting this research corpus already exists. A Git pull alone does
   not fetch private audio. Do not ask me to supply the same exports again.
4. The Claude archive index and original Codex report/protocol referenced by the
   synthesis, followed by the specific raw evidence needed for each proposed slice.

Help me turn these discoveries into a concrete implementation plan. Explain it
in plain language first, then specify small changes, dependencies, validation,
tradeoffs and decisions that genuinely remain open. Review the actual current
code before choosing a design. Do not treat either agent's suggestion as proven
merely because it appears in a report; the final synthesis corrects both.

The recommended order is: (A) SRC buffer/delay/frame-count correctness;
(B) qualified final-rate peak protection even without a loudness target or
measurable integrated loudness; (C) source/target-aware bounded whole-chain drive
selection; (D) accurate existing Density Auto/readout behavior; (E) separate
saturation continuity/antialiasing calibration. A and D can be planned separately;
C depends on trustworthy output constraints. No production implementation was
performed by the research handoff.

I want the main improvement behind the existing interface. Keep Density and
Adapt as distinct functions; do not invent a new required mode or knob. Current
production defaults, preset voicings and calibration gates stayed unchanged
during investigation. Treat automatic selection and saturation calibration as
proposals to review, not already adopted constants. The +12 dB cap, −18 LUFS
search centre, 3–5 candidate runtime claim and fixed 0.5 dB peak margin are not
approved shipping solutions. Use the existing byte-identical export reproduction
to establish your baseline before DSP experiments; a changed cross-platform hash
must be investigated, not silently accepted as equivalent.

I have limited test music and found the earlier A–E examples too subtle to rank.
Do not make progress depend on repeating them, me supplying a large library, or
an unhelpful “it's subjective.” Acquire a small targeted licensed holdout yourself
if needed. LANDR is an empirical same-source reference with different ceilings,
not an infallible target or a known proprietary implementation. Prior installed
Windows listening passed its documented baseline and must not be reopened as
missing. New code still needs the applicable affected tests and native evidence.

The browser demo is finished. Pricing, the deferred desktop refactor, deployment
and release activation are outside this task. Preserve source files, saved user
intent, responsive audition and Volume Match's audition-only behavior. Do not
delete the independent workspace or research audio while planning. Begin with
your concise reading of the conclusions and a proposed implementation sequence,
then work through the plan with me.

## Optional skills for the receiving agent

Use a diagnosis skill for a disputed objective mechanism, the repository's
verification workflow for code changes, and a planning/handoff skill if useful.
Read any selected skill before applying it. No extra skill or prior conversation
memory is required to understand the evidence. User instructions and current
repository contracts take precedence over an old generic approval checklist.
