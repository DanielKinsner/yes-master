# Track Master Stabilization And Preset Listening Plan

Status: draft for external verification and owner review.
Date: 2026-06-15.
Repo: `C:\Users\SM - Dan\Documents\GitHub\yes-master`.

This is a living decision-level plan. It captures the current manual-test
findings, locks the product decisions made so far, and defines how the shared
Track Master presets should sound before any DSP retuning happens. As of
2026-06-16, the retune/listening portion is Wave 10 work, after the mechanical
shippability queue is complete.

## Purpose

Track Master is currently in a stabilization phase. The next work should fix
objective product-safety and UX issues before changing preset calibration.

The preset concern is real, but taste work must not be mixed into mechanical bug
fixes. First, fix the app behaviors that are objectively wrong or confusing.
Then verify whether the presets are wired too conservatively. Only after that
should DSP constants be retuned from structured listening notes.

## Current Manual Findings

- Standard has a small success receipt, but export progress and full receipt
  detail are not prominent enough. The full receipt should be visible in
  Standard without switching to Advanced.
- Repeated exports can overwrite a prior rendered master if the user accepts an
  existing path in the Windows save dialog. YES Master should never overwrite a
  prior render by default.
- New imports should become the selected track immediately and show their
  analyzing state. The current behavior can leave the previous track selected.
- The app should be allowed to close during analysis. Only render/export work
  that is actively writing or preparing output should trigger a close warning.
- The "Landing loudness..." message appears to have a trigger/state bug: it can
  remain stuck until a later settings change causes the landing state to update.
- Live meters need verification and relabeling so target LUFS, momentary LUFS,
  since-play LUFS, and peak dBFS are not confused.
- Returning from Advanced to Standard should preserve Standard-owned choices and
  reset only hidden Advanced edits. The dialog copy currently implies a fuller
  reset than intended.
- First-run tips should become subtle first-run help/coachmarks, not embedded
  rail UI.
- Standard and Advanced preset names should match: `Universal`, `Clarity`,
  `Tape`, and `Oomph`.
- Preset cards in both Standard and Advanced should show artwork/name only.
  Hover/tooltips should carry preset intent.
- The first four Advanced presets should be `Universal`, `Clarity`, `Tape`, and
  `Oomph`.
- The presets currently feel too characterless and insufficiently distinct.
  Do not retune from one impression.

## Reviewer Reconciliation (Claude, 2026-06-15)

External verification pass against the manual-test catalog in
`docs/LISTENING_FINDINGS_2026-06-15.md`. This plan converges with that catalog on
the major calls (bugs before tuning, the preset regression/wiring audit, name
unification). Additions and corrections:

### Additional findings to fold in (missed above)

- The close trap during analysis is almost certainly the **existing S6.8 window
  close-guard** (shipped on the `mechanical-tail` branch into the tested build),
  not a feature still to be built. Action: **fix or revert S6.8** so it can never
  block close on a stalled handler and engages only during actual in-flight
  output. Do NOT implement a second close-guard — it would collide with S6.8.
- **Fresh import shows an active play button + live meters with nothing playing**
  (stale transport/meter state on import). Add to the import work.
- **Original/Mastered toggle clicks + lags slightly on fast toggling** — polish
  item (candidate: short crossfade). Owner notes it is not a dealbreaker.
- **Receipt header reads "2 ITEM TO REVIEWS"** → should be "2 items to review".
- **Receipt shows "ADAPTIVE 50%" + source-profile readout** — confirm shipped vs
  debug-flag; bears on the mode-pill label question.
- The `mechanical-tail` branch already carries 4 committed slices (audit-note
  reconciliation, handoff declutter, the AC-5 listening sheet, and S6.8) plus two
  remaining tail items: **S6.6 (missing-source relink)** and **S6.7 (cancelable
  jobs)**. Slot these below the Phase 0–3 bug fixes or defer explicitly.

### Owner decisions locked 2026-06-15 (supersede conflicting text below)

- **Standard receipt stays lean** with simpler copy plus a clear affordance to
  open the full report in Advanced — NOT a full receipt embedded in Standard.
- **Meter labels: simpler in Standard; full technical labels + tooltips in
  Advanced.**

### Retune gate — measurement-first (supersedes the listening-matrix gate)

The owner cannot run extended/multi-session listening (solo dev, limited time,
44.1/16 test material only). Keep the Listening Matrix as the *spec of what good
looks like*, but move the **gate** to Wave 10:

- Build a **measurement harness**: render each preset over fixed signals (pink
  noise + available WAVs) and report per-band tonal delta vs source, dynamics,
  width, and saturation at intensity 0.5 and 1.0 — the objective preset
  "fingerprint."
- **Quantify the Preset Sound Targets below into numeric fingerprints** so tests
  can assert them (e.g. "Oomph at 1.0 adds >= X dB sub vs source and >= Y dB more
  tonal delta than at 0.5").
- Gate changes with **mechanical tests**; one short ear spot-check confirms math
  ~= perception. No full-matrix listening required.

### Metering nuance

Treat the meter work as **verify-convergence-first**, not relabel-only: the
"since-play hot vs target" reading + the never-settling message suggest the live
audition may genuinely not reach target. If convergence fails, that is a
DSP/landing correctness bug and relabeling alone will not fix it.

## Locked Product Decisions

- Use one living plan document for the stabilization and preset-listening work.
- Do objective safety and UX fixes before DSP retuning.
- Standard keeps a lean success receipt (simpler copy) plus a clear affordance to
  open the full report in Advanced (owner decision 2026-06-15; supersedes the
  earlier "full receipt in Standard" idea).
- If the user picks an existing export path, save to a unique sibling and show
  the actual saved path in the receipt.
- Judge preset character at matched loudness first, not by "louder feels
  better."
- Returning to Standard preserves Standard-owned choices: preset, loudness, and
  intensity. It resets hidden Advanced-only edits.
- The close button must not trap the user during analysis.
- The stuck landing message is treated as a bug, not as a valid endless
  "settling" condition.
- External references such as BandLab or LANDR guidance can anchor the
  discussion, but should not define YES Master's sound by themselves.

## Preset Sound Targets

### Universal

Universal should be relatively neutral and safe, but not weak. It should add a
little low-end confidence, a bit of width, and an all-around finished-master
impact while staying balanced and not over-characterized.

At low intensity, Universal should feel like tasteful mastering polish. At high
intensity, it should still be the safest all-purpose preset, but it should not
feel like a bypass clone or only a loudness change.

### Clarity

Clarity should add vocal/transient articulation, air, and upper detail without
making the track smaller.

It must not trade away body. It should not become brittle, thin, harsh, or
fatiguing. The goal is intelligibility and openness while preserving enough
warmth and weight that the mix still feels full.

### Tape

Tape should emphasize glue and warmth. It should feel rounder, more cohesive,
and subtly saturated.

It should not simply darken the master or remove useful detail. At higher
intensity, the saturation and cohesion should become more obvious, but the
result should still feel musical rather than muffled.

### Oomph

Oomph should be bold but controlled. It should be the most dramatic of the four
shared presets, adding low-end weight, punch, and impact.

At high intensity, Oomph should feel clearly heavier and more physical, but it
must avoid mud, uncontrolled bass bloom, obvious pumping, or damaging
already-mastered material.

## Intensity Expectations

Intensity should change character, not just loudness.

- 0-25 percent: subtle polish, but not indistinguishable from bypass.
- 50 percent: the preset identity should be clear.
- 75-100 percent: the preset should be unmistakable while still musically
  usable.

If intensity mainly changes output level and the preset character does not
become more apparent, that is a failed preset-feel result.

## Work Order

1. Create this draft planning document.
2. Tune this document with owner review and the `/grill me` process.
3. Fix objective UX and safety issues first:
   - Export overwrite prevention.
   - Standard export progress and receipt visibility.
   - Import auto-selection.
   - Close behavior.
   - Landing loudness stuck state.
   - Meter verification and relabeling.
   - Preset naming/order/card cleanup.
   - First-run coachmarks.
4. Add mechanical tests for objective behavior.
5. Run a preset-character regression and wiring audit before retuning:
   - Confirm the adaptive compressor gate is OFF by default.
   - Verify active adaptive guardrails are not over-neutralizing preset moves.
   - Compare current preset strength against earlier substantial-feeling
     behavior if feasible.
   - Confirm Standard and Advanced call the same intended preset paths.
6. Research external mastering references as anchors, not rules, before citing
   them in product or tuning decisions.
7. Defer the listening matrix and any taste calibration to Wave 10.
8. Only retune DSP constants inside Wave 10, after repeatable notes or
   measurement fingerprints exist.

## Stabilization Acceptance Criteria

- Standard shows prominent export progress while a master is being created.
- Standard shows a lean post-export receipt (key metrics + saved path + review
  state) plus a clear affordance to open the full report in Advanced.
- Re-exporting the same track cannot overwrite a previous render by default.
- The receipt always shows the actual final saved path.
- Importing a new track selects that track immediately.
- Import analysis state belongs to the newly imported/selected track.
- Closing during analysis exits normally.
- Closing during export/render prompts only when there is actual in-flight
  output work.
- The landing message clears when landing is resolved, and it updates without
  requiring a later settings change.
- If the live/export target is ceiling-limited, the UI explains that instead of
  presenting an endless settling state.
- Meter labels/tooltips make clear that:
  - target LUFS is the selected delivery target,
  - momentary LUFS is a short live loudness window,
  - since-play LUFS is live integrated loudness over current playback,
  - live peak is dBFS/dBTP-style peak behavior, not LUFS.
  - Standard uses simpler/plainer meter labels; Advanced carries the full
    technical labels + tooltips (owner decision 2026-06-15).
- Standard and Advanced display the same shared preset names.
- Preset cards show name/artwork only, with intent available on hover.

## Preset Regression And Wiring Audit

Before changing DSP constants, audit whether the current chain is unintentionally
reducing preset character.

Questions to answer:

- Is the adaptive compressor gate still OFF by default in the packaged app?
- Are active adaptive guardrails trimming low, bright, width, or density moves so
  strongly that presets become too similar?
- Did a recent change make intensity affect loudness more than tone/dynamics?
- Are Standard and Advanced using the same preset settings end to end?
- Did the preset feel become weaker after adaptive compressor work, guardrail
  work, LUFS landing work, or Standard/Advanced mapping work?

The audit should prefer mechanical evidence where possible: current code, prior
commits, existing DSP snapshots, reference runners, and controlled renders. It
should not rely only on memory.

## Listening Matrix

Preset character must be judged at matched loudness first. Capture real export
impact separately if needed.

Test sources:

- Normal unfinished mix.
- Quiet or high-dynamic-range track.
- Already-mastered hot track.
- Transient-heavy track.
- Bass-heavy track.
- Long 30+ minute file.

For each of `Universal`, `Clarity`, `Tape`, and `Oomph`, test:

- Loudness targets: `-14`, `-11`, and `-9` LUFS.
- Intensities: `0%`, `50%`, and `100%`.
- Volume Match on/off for audition only.
- Original/Mastered switching during playback.

Record:

- Source LUFS, LRA, and true peak.
- Export LUFS, LRA, and true peak.
- Live meter behavior.
- Warnings shown.
- Whether character is obvious at matched loudness.
- Whether intensity changes tone/dynamics or mostly level.
- Exact timestamps for failures.
- Verdict: keep, adjust, or reject.

## Retune Rules

- Do not retune from one track or one impression.
- Do not mix objective UX fixes and subjective DSP retuning in the same commit.
- Do not change export LUFS landing semantics as part of preset-character tuning.
- Do not change compressor mode semantics as part of preset-character tuning.
- If a finding is objective, add a mechanical test.
- If a finding is taste/listening-dependent, capture the listening note before
  changing calibration.
- Treat Oomph carefully: it should be bolder, but not at the cost of obvious
  pumping or mud.

## Verification

Normal fast lane after objective fixes:

```powershell
npm test
npm run build
npm run build:windows
cd src-tauri
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

Before any DSP/export retune merge, also run the slow fixture lane:

```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

Run mobile bridge lanes if shared crate types, Tauri command signatures, or
native bridge-facing behavior changes.

## Open Review Questions

- Which earlier commit or build felt more substantial, if that can be identified?
- Should the final Standard receipt use the exact Advanced receipt layout or a
  shared modal with Standard-specific wording?
- Should meter relabeling happen in both Standard and Advanced, or should
  Standard use simpler meter labels?
- Should the preset regression audit produce rendered comparison files for owner
  listening, or only a written/code-level report first?
- What external references should be reviewed for mastering ranges and preset
  intent before tuning?
