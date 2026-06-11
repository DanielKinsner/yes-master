# Analysis Orb — Design

Date: 2026-06-11
Status: Approved direction, pending owner spec review.

## Goal

Make the analysis wait — the one unavoidable friction point — feel engaging
and premium instead of dead. A particle "orb" animates in the waveform region
while analysis runs, then dissolves into the track's real waveform when the
preview is ready. The wait visibly produces the thing the user waited for.

Reference inspiration: LANDR's analysis modal (point-cloud orb + staged
copy). Deliberate differences:

- **Not a modal.** The orb lives in the waveform's empty slot and never
  blocks the UI. A returning user importing their 50th track sees a pretty
  loading state, not a ceremony.
- **Honest copy.** The existing `ANALYSIS_PROGRESS_STAGES` labels in
  `useTrackMaster.ts` (Analyzing audio → Reading tonal balance → Checking
  dynamics → Evaluating stereo field → Building mastering context →
  Preparing preview) roughly map to real analysis work. Keep them; no
  invented theater like "Detecting sub-genre".
- **The morph ending.** The orb's particles fly into position to form the
  actual rendered waveform peaks. LANDR's orb is the same blob for everyone;
  ours becomes the user's track.

## Scope decision (owner-confirmed)

v1 is **orb + morph ending**, keeping the existing timer-paced stage labels.
Real backend progress events are a follow-up slice (see Future).

## Behavior

1. While `isAnalyzing` is true for the selected track and no waveform is
   shown, the orb renders in the waveform display region: a rotating
   point-cloud sphere (~2,000–2,500 particles) with gentle noise
   displacement, in the app's existing blue palette. The current stage label
   and progress bar render with it (reusing `analysisProgress`).
2. When analysis completes **and the waveform peak data is available**, the
   morph plays: each particle animates (~1.2–1.6 s, eased) to a position on
   the real waveform shape, then the actual waveform rendering crossfades in
   underneath and the particles fade out.
3. If analysis fails, the orb fades out and the existing error path renders
   unchanged.

## Rules

- The morph is presentation only: transport readiness, audition, and all
  interactivity never wait on the animation. If the user interacts mid-morph
  (play, seek, switch track), the morph cuts immediately to the real
  waveform.
- `prefers-reduced-motion` is honored: static placeholder plus the existing
  label/progress, simple crossfade to the waveform, no particle motion.
- The animation pauses when the window is hidden
  (`document.visibilityState`).
- Very fast analyses (short files) are fine: the orb may exist for under a
  second; the morph still plays but never delays readiness (see above).
- v1 targets the Track Master single-track waveform region in both Standard
  and Advanced views. Album batch analysis keeps the current status chips.

## Implementation shape

- Canvas 2D particle system, `devicePixelRatio`-aware, requestAnimationFrame
  driven. No WebGL, no new dependencies, no Rust changes in v1.
- The particle system runs on the UI thread/GPU while analysis runs in
  native Rust — they do not compete for the same budget.
- A self-contained component (e.g. `src/components/AnalysisOrb.tsx`) owning
  canvas lifecycle, sizing, and the orb/morph/done state machine. Inputs:
  `isAnalyzing`, `analysisProgress`, and the waveform peak data (morph
  targets). The host (waveform region) decides when to mount/unmount it.
- Particle→waveform target assignment is a pure helper (peaks in, target
  positions out) so it is mechanically testable.
- A working visual prototype of the orb, stages, and morph was built and
  approved during design review (canvas, ~2,200 particles, app blues); the
  implementation should match its feel.

## Testing

- Unit tests on the pure helpers: target assignment from peak data, state
  transitions (orb → morph → done; fail → fade; interact → cut).
- Component smoke test: mounts during `isAnalyzing`, unmounts/cuts on
  interaction, honors reduced motion.
- The look and feel of the animation itself is taste — manual owner signoff
  on the visual, per repo working style.

## Future (separate slices, not v1)

- **Real progress:** emit `analysis:progress` events from Rust mirroring the
  existing `render:progress` pattern, so the bar and stage labels track
  actual work instead of a timer.
- **Audio-seeded motion:** seed particle displacement from the track's own
  envelope/spectrum so the orb is literally moved by the user's audio.

## Out of scope

- Album Master analysis visuals.
- Export/render-time visuals (render already has a real progress bar).
- Any change to analysis behavior, timing, or results.
