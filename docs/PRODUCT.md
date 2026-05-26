# YES Master Product Canon

This is the active product source of truth for YES Master.

YES Master is a local desktop mastering app for real tracks and real albums. It
is not a certified mastering engineer replacement, and it is not a throwaway
private toy. The bar is private-solid: good enough that the owner would trust it
on personal releases before considering broader distribution.

## Core Promise

Drop in audio, hear what the mastering chain is doing, shape it quickly, and
export a technically checked master without risking the source file.

The app should make the safe path obvious without preventing expert or taste-led
choices. If a user wants to push a track too hard, YES Master can allow that.
The product responsibility is to make the consequences legible through meters,
quality checks, and export review.

## Audience

- Musicians and producers finishing tracks.
- Album-minded creators who need consistent loudness and flow.
- Users with already-processed AI/generated or service-mastered tracks who want
  to apply taste without accidentally making the file worse.

The already-mastered/AI-generated case is a stress-test class, not the whole
product identity.

## Primary Workflow

1. Import audio.
2. Analyze.
3. Audition Original vs Mastered at the same playhead.
4. Choose a preset and adjust intensity/tone.
5. Use the right rail to inspect quality, delivery, and advanced settings.
6. Export.
7. Review warnings when present.

Preview/listening is strongly encouraged, but export is allowed after analysis.

## UI Responsibility Split

Main UI owns creative sound:

- Presets.
- Intensity.
- Tone shape / EQ.
- Saturation, warmth, width, limiter character.
- Visual waveform and audition controls.

Right rail owns judgment and delivery:

- Quality Check.
- Delivery Profile.
- Advanced Controls.
- Per-band compressor detail.
- Delivery format.
- Export action and export review.

## Export Philosophy

Technical failures can block export:

- Invalid path.
- User cancels save dialog.
- Decode/render/write failure.
- Non-finite or corrupt render state.

Quality warnings should not block export when a file can be written:

- True-peak or codec headroom risk.
- Very loud integrated LUFS.
- Low dynamic range.
- Already-compressed source with additional compression.
- Measurable signs the output may be worse than the source.

When warnings exist, the export flow should move through review:

- Primary button: `Export With Review`.
- Review actions: `Adjust Settings` and `Export Anyway`.

No-warning path:

- Primary button: `Export Master`.

## Compressor Canon

The current automatic compressor behavior is preset/density fallback. It is not
track-aware analysis.

Required compressor modes:

- `Preset`: current preset/density fallback behavior.
- `Manual`: user values override preset fallback.
- `Off`: bypass creative/preset compressor only.

`Off` must not bypass:

- Limiter.
- Ceiling protection.
- LUFS landing.
- Metering.
- Export warnings.

Do not hide extra compression behind Compressor Off. If the target cannot be
reached cleanly, land as far as ceiling/headroom allows and warn.

## Loudness And Safety

The app should trust the final safety stages more than nerfing tone upfront.
Creative controls can be bold. Export/quality stages must measure what happened
and make problems visible.

Volume Match is for fair audition only:

- Optional.
- Off by default.
- Does not change export level.

Export LUFS targeting should use the current track/render measurements wherever
possible. Global guesses are not enough for final delivery behavior.

## Release-Candidate Meaning

YES Master is release-candidate only when:

- Track Master import/analyze/audition/export is stable.
- Real-time controls respond while audio plays.
- Exports are not objectively worse by default on already-processed material
  without clear warnings and review.
- Warnings are visible before the user treats the export as done.
- Private-fixture slow lane has been run for DSP/export changes.
- Windows packaging works locally.
- Known temporary instrumentation is removed or deliberately documented.

## Deferred

- Public code signing/notarization.
- Autoupdate.
- Store-style distribution.
- New reference-track UX.
- Major Album Master dashboard/report expansion.
- Subjective preset retuning without fresh listening notes.
