---
date: 2026-05-28
topic: release-candidate-hardening
---

# Release Candidate Hardening Requirements

## Summary

YES Master's release-candidate finish should make Track Master reliable, honest, and ready for private use: long tracks must audition correctly, delivery controls must describe what they actually do, project/settings/help affordances must be real, and final polish must protect the accepted UI direction.

---

## Problem Frame

The current project is past the "can it master a track" stage. The remaining risk is whether a real user can trust the app during a complete session: load a real file, audition Original vs Mastered, adjust sound, understand delivery settings, save/reopen work, export, and know what warnings mean.

Recent work stabilized export review, compressor modes, private fixture harnesses, and the accepted Track Master header. The newest product feedback changes the finish plan in four ways: the app must not imply future smart analysis is outside the product identity; the visible Settings and Help buttons cannot remain dead controls; delivery format needs an export-quality bit-depth/sample-rate story; and a 25-minute track failing to play at all in Mastered preview is an edge case that still needs a clear recoverable message for this release candidate.

---

## Actors

- A1. Track Master user: imports real audio, auditions the master, adjusts sound, saves work, and exports a checked WAV.
- A2. Future implementer: uses this document and the finish plan to avoid silently changing product scope or reopening accepted UI decisions.
- A3. Release verifier: runs automated gates, private slow lanes, and manual listening before calling the build release-candidate.

---

## Key Flows

- F1. Long-track audition
  - **Trigger:** A user loads a long track, switches to Mastered, and turns Preview LUFS on.
  - **Actors:** A1, A3
  - **Steps:** Import/analyze the track, start playback, switch Original/Mastered at the same playhead, toggle Preview LUFS, seek during playback, and keep listening.
  - **Outcome:** Playback continues or fails with a clear, recoverable message; the app does not hang, silently stop, or make Mastered unusable.
  - **Covered by:** R1, R2, R3, R18

- F2. Delivery setup
  - **Trigger:** A user prepares a master for a target such as streaming, CD, vinyl premaster, loud rock, or broadcast.
  - **Actors:** A1
  - **Steps:** Choose a Delivery Profile in the right rail, inspect LUFS/ceiling/bit-depth implications, adjust Custom values if needed, and export.
  - **Outcome:** The UI, render, receipt, and warnings agree about the effective delivery target and format.
  - **Covered by:** R4, R5, R6, R7, R8, R14

- F3. Project continuity and support
  - **Trigger:** A user wants to pause a session, reopen work later, or understand a control without leaving the app.
  - **Actors:** A1
  - **Steps:** Save/Open a project, receive success/error feedback, open Settings or Help, and return to the mastering workflow.
  - **Outcome:** Save/Open is trustworthy, Settings/Help are real surfaces, and none of these flows disturb playback or current project state unexpectedly.
  - **Covered by:** R9, R10, R11, R12, R13

---

## Requirements

**Long-track audition reliability**
- R1. Mastered playback on long real-world tracks, including tracks around 25 minutes, must either play successfully or fail with a clear recoverable message.
- R2. Original/Mastered switching must preserve the current playhead on long tracks just as it does on ordinary song-length tracks.
- R3. If an expensive preview operation or long-file setup cannot complete quickly enough, the app must surface a clear fallback instead of hanging, silently failing, or making the user think the transport is broken.

**Delivery and export truth**
- R4. Delivery Profile belongs in the right rail and remains the authoritative named bundle for delivery target, ceiling, and bit-depth defaults.
- R5. The center loudness quick-select and right-rail LUFS field must agree on the same effective target after every user edit.
- R6. Bit depth must be wired end to end: visible control, effective settings, render output, export receipt, and warning checks must agree.
- R7. Sample-rate delivery must be implemented for this release candidate. Named profiles provide sensible defaults, and Custom allows explicit override to supported rates such as Source, 44.1 kHz, 48 kHz, and possibly 96 kHz.
- R8. Sample-rate conversion must use a high-quality path appropriate for final WAV export, and rendered WAV headers/receipts must match the selected delivery rate.
- R9. Export warnings remain advisory for quality issues and blocking only for technical write/render failures.

**Project, settings, and help**
- R10. Save Project and Open Project must be complete user-facing flows, not just icon buttons. Baseline means explicit Save As/Open for `.ams.json`, clear labels/tooltips, error surfacing, and visible saved/opened status feedback.
- R11. Settings must open a real release-ready baseline app-defaults surface for existing supported behavior: audio/preview behavior, export defaults, project/session behavior, and basic app info.
- R12. Help must open a contextual in-app help panel with short sections for Import/Analyze, Original vs Mastered, Volume Match vs Preview LUFS, Delivery Profile/Format, Export Review, and Save/Open Project.
- R13. Settings and Help must not interrupt playback, clear selections, or mutate mastering settings unless the user explicitly changes an available setting.
- R14. Any preference or help text added for this pass must describe current behavior, not aspirational future behavior.

**Smart analysis product direction**
- R15. The product must not canonize "no smart features" as an identity boundary. Future optional analysis and click-to-apply suggestions for levels, compression, EQ, and related controls are product-aligned when they remain reviewable and user-approved.
- R16. The preferred future smart-feature shape is an "Analyze Suggestions / Apply" panel: suggestions are gathered and reviewed in one place before the user applies chosen moves.
- R17. This pass must not add an automated mastering-engineer replacement promise, silent auto-mastering decisions, or unreviewed smart adjustments.
- R18. UI copy in this pass should preserve the current positioning: YES Master helps users make and review mastering choices; it does not certify professional mastering judgment.

**Final UI and release evidence**
- R19. The accepted Track Master header direction stays intact: mode switch centered at the top, track identity/meta on the left, and Original/Mastered plus preview toggles in the track header.
- R20. Final UI polish may go beyond trust-only fixes where the current look still feels unfinished, but it must stay bounded to release-candidate polish rather than a full redesign.
- R21. The final UI pass must address the known visual issues from review screenshots: first-screen spacing, oddly spaced hero/title treatment, Visual EQ dissatisfaction, and the `ANALYZED` pill likely belonging near the source-check/right-rail area instead of floating in the track header.
- R22. Visual EQ should be refined, not hidden or replaced: keep the current capability while making it calmer, tighter, less toy-like, and better integrated with the mastering surface.
- R23. Undo/redo and readiness state should not require a mostly empty dedicated row. Ready/analysis state should live near track metadata or right-rail source status, and undo/redo should move into a smaller tool affordance.
- R24. The overall Track Master UI should aim for a single-panel working surface where advanced controls and compression can come into play without forcing awkward up/down scrolling or feeling cramped.
- R25. The preferred layout direction is denser main controls while preserving the current ownership model: the main surface shapes sound, and the right rail owns delivery/quality.
- R26. Release-candidate signoff requires a small representative manual listening pass: normal mix, already-mastered/compressed source, long or edge-case source, and one export warning case.
- R27. Private audio, rendered private masters, private fixture ledgers, and fixture-derived screenshots must stay out of git; only aggregate conclusions belong in committed docs.

---

## Acceptance Examples

- AE1. **Covers R1, R2, R3.** Given a 25-minute track is loaded and analyzed, when the user starts Mastered playback with Preview LUFS on, playback starts or the app gives a clear recoverable fallback/message without breaking the session.
- AE2. **Covers R4, R5, R6, R7, R8.** Given the user selects a named Delivery Profile and then customizes LUFS, bit depth, or sample rate, when they export, the visible right-rail values, effective render values, receipt, warnings, and WAV header describe the same delivery result.
- AE3. **Covers R10, R11, R12, R13.** Given audio is paused or playing, when the user opens Settings, Help, Save Project, or Open Project, the app shows a real surface or dialog and returns to the current workflow without losing state.
- AE4. **Covers R15, R16, R17, R18.** Given future work adds per-track suggestions, when the app presents level/compression/EQ recommendations, those suggestions appear in an Analyze Suggestions / Apply flow and remain optional rather than automatic claims of professional mastering replacement.
- AE5. **Covers R26, R27.** Given private fixtures are used for verification, when release evidence is committed, it includes command results and aggregate listening/fixture conclusions without committing private audio artifacts.

---

## Success Criteria

- A long real-world track can be auditioned in Mastered mode when possible, and edge-case failures produce clear recoverable messaging instead of silent non-playback.
- A normal user can choose the needed Delivery Profile, bit depth, and sample rate before export and trust that the rendered WAV matches those choices.
- Settings, Help, Save Project, and Open Project feel like finished release-candidate affordances, not placeholders.
- The final UI pass makes the app feel more finished, especially on the first screen, with less wasted space and less scrolling friction, without reopening the accepted Track Master layout.
- The finish plan can be implemented with minimal additional product invention.

---

## Scope Boundaries

- In scope: Track Master release-candidate hardening, long-track preview reliability, delivery-format honesty, required sample-rate conversion, Save/Open project polish, baseline Settings app defaults, contextual in-app Help, bounded broader visual polish, automated verification, private slow-lane evidence when available, and manual listening notes.
- Deferred for later: a fuller smart-assistant system centered on an Analyze Suggestions / Apply panel for optional per-track levels/compression/EQ moves; a deeper preferences system beyond release-needed settings; major Album Master expansion; reference-track UX; public signing/notarization/autoupdate/store distribution.
- Outside this pass: silent automatic smart adjustments, claims that YES Master replaces certified mastering judgment, and any committed private audio or fixture-derived private artifacts.

---

## Key Decisions

- Treat the 25-minute Mastered preview failure as a release-candidate hardening case: silent non-playback is not acceptable, but a clear edge-case fallback/message is acceptable for this pass.
- Keep smart analysis as product-aligned future work: defer the feature, but do not define it out of the product identity.
- Make delivery format real before release-candidate signoff: bit depth and sample-rate conversion must be wired through export and receipt behavior.
- Finish Settings/Help as release-ready surfaces for current behavior: Settings should cover baseline app defaults, while avoiding a large speculative preference system in this pass.
- Preserve the accepted header layout while allowing broader polish: fix spacing, title treatment, Visual EQ presentation, wasted control rows, scrolling friction, and misplaced status affordances without restarting the whole design.
- Keep manual listening as a release gate: use a small representative pass rather than a full by-ear preset matrix.
- Start implementation with delivery and project chrome first: sample-rate/bit-depth export, Settings/Help/Save/Open, then UI polish and playback hardening.
- Do not begin implementation from this brainstorm without explicit user permission; the likely next step is a fresh window using this requirements doc and the finish plan as handoff context.

---

## Dependencies / Assumptions

- Long-track testing can use local private audio, but evidence committed to the repo must stay aggregate.
- The implementation plan may choose the safest technical route for long-track preview reliability, including fallback behavior, as long as the user-facing requirements above are met.
- Rust formatting is a tooling hygiene gate only: it means running the standard Rust formatter so backend code has consistent style; it is not a product-scope decision.
- Upsampling a compressed source must not be described as restoring lost quality. The product should frame sample-rate conversion as delivery-format control and use the cleanest available WAV export path.
- A new implementation session should ask for explicit permission before editing code, then start from delivery and project chrome unless the user reprioritizes.

---

## Outstanding Questions

### Deferred to Planning

- [Affects R1, R3][Technical] Determine whether long-track preview reliability is best solved through reduced PCM copying, streaming playback, a Preview LUFS fallback, a clear long-file limitation message, or a combination.
- [Affects R7][Technical] Determine the safest high-quality sample-rate conversion path for this release candidate and how to verify rendered WAV sample rate accurately.
