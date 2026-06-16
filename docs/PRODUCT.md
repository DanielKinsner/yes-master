# YES Master Product Canon

This is the active product source of truth for YES Master.

YES Master is a local desktop mastering app for real tracks and real albums,
with iPhone/Android companion apps sharing the same engine (see "Mobile
Companions" below; desktop ships first). It is not a certified mastering
engineer replacement, and it is not a throwaway private toy. The bar is
private-solid: good enough that the owner would trust it on personal releases
before considering broader distribution.

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

**Standard is the default view.** The primary workflow is Standard's:

1. Import audio (drag-drop or the hero CTA).
2. Analyze (automatic, with real progress).
3. Audition Original vs Mastered at the same playhead.
4. Pick a Style (Universal / Clarity / Tape / Oomph) and a loudness
   (Low −14 / Medium −11 / High −9 LUFS); shape with Intensity.
5. Create Master — fixed 44.1 kHz / 24-bit WAV at −1 dBTP, no blocking
   review (see "Standard view — export ceremony" below).

The Advanced view extends this for users who want control:

6. Full preset set, EQ/tone, width/warmth, compressor modes.
7. Right rail: quality checks, delivery profile/format, advanced settings.
8. Export with warning-aware review when warnings exist.

Preview/listening is strongly encouraged, but export is allowed after analysis.

## UI Responsibility Split (Advanced view)

Standard deliberately has no judgment rail: one column of creative choices,
one Create Master action, warnings suppressed per the export ceremony below.
The split here describes Advanced.

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

### Standard view — export ceremony

The review ceremony above describes **Advanced**. In **Standard** (the default
view), the deliberate behavior is:

- **No blocking review gate.** `Create Master` renders directly. Standard
  trusts the validated engine + the user's ears.
- **Cosmetic / advisory warnings are suppressed** (e.g. loudness-vs-reference,
  low dynamic range, codec headroom, already-compressed source). A
  non-technical user shouldn't have to weigh them.
- **One tiny, non-blocking integrity note** is kept for a genuine integrity
  issue (true-peak slipping over) — inline text, never a modal. The
  fixed-ceiling limiter makes this rare by construction.
- **Technical hard-stops are always surfaced, never hidden** — but be precise
  about *when*. Pre-render failures (invalid path, cancelled save dialog,
  decode failure) abort before any file is written: these genuinely block.
  Post-render criticals (non-finite / corrupt render, requested-vs-rendered
  sample-rate mismatch, sub-16-bit) are detected *after* the WAV is written,
  so the honest framing is "saved, but this master is invalid — re-render",
  shown prominently instead of celebrating success (not a pre-render block).
  Under Standard's pinned 44.1 kHz / 24-bit format the only post-render
  critical that can realistically fire is a corrupt/non-finite render, which
  is unknowable before rendering. This matches Advanced, which has always
  rendered-then-checked; Standard differs only in suppressing the cosmetic
  warnings.
- **Standard exports a fixed, known-safe default: 44.1 kHz / 24-bit WAV at a
  −1 dBTP ceiling**, with the Standard-chosen loudness (−14 / −11 / −9 LUFS).
  Sample rate / bit depth / ceiling are configurable only in Advanced. This
  mirrors the iPhone app's fixed export.

## Adaptive Mastering

What adapts per track today (all shipped, owner-listened 2026-06-11):

- Loudness landing uses the current track/render measurements.
- A resolved source profile drives Tier-1 guardrails: reduce-only trims to
  preset brightness/low boosts and a reduce-only scale on compression
  density, weighted by per-axis confidence and the user's Adapt Strength.
  A preset stays recognizable by construction (per-axis trim caps).

What deliberately does not adapt today:

- Preset tone tilt is fixed per preset (not reference-matched).
- Phase-B per-window confidence gating is built but OFF by default,
  pending the owner calibration sitting.

Direction (decided 2026-06-12, not yet shipped): per-band track-aware
compression with transient protection and already-mastered stand-down is
part of the MVP, built reduce-only and gated off until owner calibration —
spec: `docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md`. Do not
describe it as current behavior until that spec's final phase lands.

## Compressor Canon

The compressor's baseline is preset/density fallback. It is partially
track-aware today in exactly one dimension: Tier-1 guardrails scale the
density macro down (never up) on dense sources when a source profile
resolves. There is no per-band or transient-aware adaptation yet (see
Adaptive Mastering above for the decided direction).

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

## Mobile Companions (product definition pending)

The repo ships an iPhone app and an Android app (`apps/`) that reuse the
shared engine. Verified facts: both mirror Standard's fixed export recipe
(44.1 kHz / 24-bit / −1 dBTP, Standard loudness trio) and resolve the
adaptive source profile like desktop; engine output is bit-parity-pinned
against desktop. The v1 public push is desktop-first (decision 2026-06-12);
mobile execution plans live in `docs/plans/2026-06-12-iphone-shippability-plan.md`
and `docs/plans/2026-06-12-android-shippability-plan.md`.

**Pending owner definition (do not invent):** mobile audience, scope
promise, and what is deliberately absent on phones. Captured via the
canon-refresh interrogation (roadmap S5.4).

## Album Master (product definition pending)

Album Master exists in Advanced: album intent, per-track override, arc
kinds, album-wide delivery format with mixed-rate resampling, continuous +
per-track renders with a manifest, mixed mono/stereo channel-count resolution,
and above-stereo source fold-down to stereo delivery.

**Pending owner definition (do not invent):** the album product promise —
what album mastering means here beyond consistent loudness and flow.
Captured via the same S5.4 interrogation.

## Deferred

- Public code signing/notarization.
- Autoupdate.
- Store-style distribution.
- New reference-track UX.
- Major Album Master dashboard/report expansion.
- Subjective preset retuning without fresh listening notes.
