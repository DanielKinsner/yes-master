# Adaptive DSP Desktop Review

Date: 2026-06-03
Branch reviewed: `feat/adaptive-dsp-guardrails`
Scope: desktop YES Master app, especially Track Master live audition, preview render, export, Album Master export, DSP math, analysis inputs, and verification lanes.

## TLDR

The adaptive layer is the right kind of v1: it is defensive, narrow, and mostly wired into the core coefficient builder instead of becoming a second mastering engine. The core math checks out for the main safety invariants: it reduces only positive preset moves, does not flip EQ signs, does not narrow width past neutral, preserves manual EQ/width/compressor overrides, and leaves the no-profile path byte-identical.

It is not ready to trust as release evidence yet. The biggest problem is not the DSP math; it is coverage and data plumbing. The private already-mastered matrix and reference tuning runners never set `advanced.source_profile`, so they do not exercise the adaptive guardrails at all. The offline "preview WAV" render also skips profile injection, so it can differ from live audition and final Track Master export. Album Master is explicitly out of scope in the handoff, but that leaves an important desktop output path without the new protection.

The second problem is interpretation. Some adaptive caps are coefficient-level caps, not final rendered-output guarantees. When Delivery Profile LUFS landing is active, the export/live-preview landing stage can add broadband makeup after the guardrail trims and partially recoup the loudness those trims removed. That does not break the reduce-only math, but it means a final A/B can move by more than a single per-axis cap suggests.

The third problem is calibration quality. The brightness deadband is low enough that a pink-ish neutral spectrum can trigger a large high-band trim. The 6-band FFT reads only the first roughly 5.5-5.9 seconds despite the "up to 30 seconds" comment, so intros can steer the whole track. Non-finite LRA is coerced to `0.0`, which the density guardrail reads as maximally dense.

## ELI5

The new feature is like a cautious helper that says, "This song is already bright, wide, or squashed, so I will turn down only the part of the preset that would add more of that." That is a good idea.

The helper has two practical problems right now. Some important tests and render paths never invite the helper into the room, so they still use the old behavior. And when the helper does listen, it sometimes judges the whole song from the first few seconds or from rough guesses that are too sensitive.

## What Was Reviewed

- Product/behavior/architecture/testing/stabilization docs listed in `AGENTS.md`.
- Branch diff from `main` to `HEAD`.
- Adaptive implementation in `src-tauri/src/guardrails.rs`.
- DSP hook points in `src-tauri/src/dsp.rs`.
- `SourceProfile` and settings contract in `src-tauri/src/types.rs` and `src/bindings.ts`.
- Frontend injection in `src/lib/settings-transitions.ts` and `src/hooks/useTrackMaster.ts`.
- Track render/preview and Album Master render paths in `src-tauri/src/engine.rs` and `src-tauri/src/album_render.rs`.
- Slow-lane fixture/reference runners in `src-tauri/src/fixture_matrix.rs` and `src-tauri/src/reference_tuning.rs`.
- Adaptive tests and nearby hook/UI tests.

I did not run private audio or manual listening. This is a code/math/wiring review, not a taste signoff.

## Verdict

The strategy is sound for Tier 1. A defensive trim layer is a better first move than a corrective target-curve engine because it has a small blast radius and aligns with the product rule that users may overcook a track, but the app should make risk visible and avoid making already-processed sources worse by default.

The implementation is partially applied well:

- Good: guardrails live inside `ChainCoeffs::from_settings`, so the live chain and offline render can share the same coefficient logic.
- Good: no source profile means the old preset output stays unchanged.
- Good: manual EQ offsets, Manual compressor mode, Compressor Off, explicit width, limiter, ceiling protection, LUFS landing, and metering are not bypassed.
- Not good enough: the source profile is injected by frontend call site, not centrally from backend analysis, so paths are already missing it.
- Not good enough: the slow-lane evidence for already-mastered inputs does not exercise the adaptive layer.
- Not good enough: the final rendered-output behavior under LUFS landing is not covered by the coefficient-level cap tests.
- Not good enough: the provisional tonal/density measurements need hardening before they are used as "smart" defaults.

Priority calibration:

- Release-evidence blockers: slow-lane coverage, offline preview parity, and source-profile ownership.
- Product-scope decision: whether Album Master should stay explicitly unadapted or gain per-track profiles.
- Calibration blockers before listening signoff: LRA sentinel, spectral windowing, bright deadband, and per-axis trim readouts.

## Findings

### P1 - Private fixture and reference lanes do not exercise the adaptive layer

Evidence:

- `settings_for_matrix_case` sets `source_lufs_integrated` but never `advanced.source_profile` (`src-tauri/src/fixture_matrix.rs:117-126`).
- The matrix then renders with those inert settings (`src-tauri/src/fixture_matrix.rs:204-228`).
- `settings_for_reference_preset` has the same omission (`src-tauri/src/reference_tuning.rs:294-303`).
- `SourceGuardrails` only runs when `settings.advanced.source_profile` exists (`src-tauri/src/dsp.rs:756-761`).

Impact:

The release stabilization docs make the already-mastered matrix the slow lane for DSP/export changes. Right now that lane would validate the old chain for the exact stress class this feature is meant to improve. A green matrix would not prove the adaptive layer helped already-mastered material, and it would not catch a bad adaptive default.

Recommendation:

Use `SourceProfile::from_analysis(&source_analysis)` in both fixture/reference settings builders, or move profile injection into backend render entry points so all non-UI runners inherit it. Add explicit ledger columns for `adaptive_strength` and per-axis trim deltas once those are exposed.

### P1 - Offline Track Preview render skips source-profile injection

Evidence:

- `updatePreview` calls `api.renderTrackPreview(selectedTrackId, selectedTrack.path, selectedSettings)` without `injectSourceProfile` (`src/hooks/useTrackMaster.ts:1208-1218`).
- Track Master export does inject the profile (`src/hooks/useTrackMaster.ts:1244-1248`).
- Live Mastered audition uses `withSourceLufs`, which now includes `applyChainDispatchOverrides` and profile injection (`src/hooks/useTrackMaster.ts:1293-1308`, `src/lib/settings-transitions.ts:181-203`).
- Backend preview render trusts the settings it receives and does not derive a profile from the decoded track (`src-tauri/src/engine.rs:335-362`, `src-tauri/src/engine.rs:549-585`).

Impact:

Normal Original/Mastered audition is adapted once analysis has a valid 6-band profile, and final Track Master export is adapted when the selected analysis has that profile. But the "preview render" WAV can be old-chain output while export is adapted-chain output. The comment says that button is useful for auditing the would-be master in another player, so this is a real WYSIWYG break.

Recommendation:

Call `injectSourceProfile(selectedSettings, selectedAnalysis)` in `updatePreview`, or better, derive the source profile in backend render from the decoded/analyzed source.

### P2 - Delivery LUFS landing can recompose adaptive trims

Evidence:

- Adaptive caps are applied while resolving chain coefficients (`src-tauri/src/dsp.rs:744-773`, `src-tauri/src/dsp.rs:879-940`).
- Track Master export then measures the post-chain render and applies ceiling-bounded LUFS landing when an effective target exists (`src-tauri/src/engine.rs:637-679`).
- The live Export LUFS preview path computes the same landing gain and applies it through `export_landing_gain_lin` (`src-tauri/src/audio.rs:865-945`, `src-tauri/src/dsp.rs:2041-2043`).
- The landing delta can be positive when trims reduce loudness and free true-peak headroom (`src-tauri/src/engine.rs:106-127`).

Impact:

The guardrails still reduce only the preset coefficient moves they touch. But the final mastered output is not guaranteed to preserve that same apparent reduction under a delivery target, because LUFS landing may add broadband gain after the chain. A source that triggers several axes can therefore sound closer in final loudness than the coefficient deltas imply, and the final rendered A/B can exceed any single per-axis cap in user-perceived level or brightness.

Recommendation:

Add a render-level multi-axis test with an active delivery profile. Assert both the coefficient trims and the final rendered measurements, so the product knows the difference between "caps were respected in the chain" and "final export stayed audibly bounded after LUFS landing." The per-axis readout should say it reports chain trims, not post-landing tonal deltas.

### P2 - The 6-band spectral trigger reads only the first few seconds

Evidence:

- The comment says "Up to 30 seconds" (`src-tauri/src/analysis.rs:322`).
- The loop caps `fft_size` at `1 << 18` frames (`src-tauri/src/analysis.rs:325-328`).
- At 48 kHz, 262,144 frames is about 5.46 seconds. At 44.1 kHz, it is about 5.94 seconds.
- The FFT starts at frame 0, not the middle and not an average across the track (`src-tauri/src/analysis.rs:338-353`).

Impact:

An intro can classify the whole song as bright, boomy, or neutral even when the body of the track is different. Dynamics and correlation are whole-track, so the profile mixes whole-track and first-window measurements.

Recommendation:

Use Welch-style averaging across several windows, or at least sample beginning/middle/end and average band power. Fix the comment if the cap remains intentional.

### P2 - Brightness deadband likely fires on neutral pink-tilted material

Evidence:

- Brightness is `presence + air`; no trim at or below `0.20`, full raw trim by `0.32` (`src-tauri/src/guardrails.rs:29-32`, `src-tauri/src/guardrails.rs:94-97`).
- With the actual band edges in the code, a 1/f pink spectrum has about `0.278` of its 20 Hz-16 kHz power in 2.5 kHz-16 kHz:
  `ln(6500/2500) + ln(16000/6500)` divided by `ln(16000/20)`.
- At default strength `0.6`, that is roughly a 39% trim of positive high-band preset moves before the 50% EQ cap.
- The neutral unit-test fixture uses `presence=0.08, air=0.05` (`src-tauri/src/guardrails.rs:211-214`), which is far darker than the pink reference implied by the band edges.

Impact:

Neutral mixes can lose air by default. That contradicts the handoff expectation that neutral mixes should do little or nothing.

Recommendation:

Raise the bright deadband near the measured pink/reference range, or switch to spectral slope/tilt against a reference rather than raw high-band share. Add a pink-noise profile test that asserts no bright trim at default strength.

### P2 - Non-finite LRA is converted to `0.0`, causing a full density trigger

Evidence:

- Analysis stores non-finite LRA as `0.0` (`src-tauri/src/analysis.rs:65-67`, `src-tauri/src/analysis.rs:154-160`).
- `sourceProfileFromAnalysis` forwards that value (`src/lib/settings-transitions.ts:216-223`).
- The density guardrail uses `max(DR ramp, LRA ramp)` and `LRA=0.0` is below the full-trim threshold (`src-tauri/src/guardrails.rs:104-112`).

Impact:

If EBU LRA is unavailable or non-finite, the source can be treated as maximally dense even when the P95-P10 dynamic range is healthy. This is most likely on short, sparse, or odd inputs, but those are exactly the cases where "unknown" should not mean "fully compressed."

Recommendation:

Represent unknown LRA as `Option<f32>` in `SourceProfile`, or ignore LRA values <= 0.5 LU unless another density signal agrees. Add a regression test: healthy P95-P10 + unknown/sentinel LRA must not density-trim.

### P2 - The density guardrail scales a macro, not actual compression amount

Evidence:

- `scale_density` multiplies the density macro (`src-tauri/src/guardrails.rs:151-154`).
- That macro feeds engagement, overdrive, threshold, and ratio together (`src-tauri/src/dsp.rs:942-952`).
- At default strength `0.6`, a full density trigger already hits the 60% cap: density multiplier becomes `0.4` (`src-tauri/src/guardrails.rs:55`, `src-tauri/src/guardrails.rs:110-112`).

Impact:

The statement "keeps >=40% of compression" is true only for the density macro, not necessarily gain reduction in dB or perceived transient preservation. The default is also not moderate on fully triggered dense material; it applies the maximum density trim.

Recommendation:

Reword the guarantee to "keeps >=40% of the density macro" or cap a later, more meaningful compression measure. For Tier 2, use crest/PSR or post-render dynamic-range delta to avoid making already-dense tracks objectively flatter.

### P2 - `stereo_width` is carried but ignored by the width guardrail

Evidence:

- `SourceProfile` includes `stereo_width` (`src-tauri/src/types.rs:121-132`).
- The spec says already-wide uses `stereo_correlation` plus `stereo_width` (`docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md:177-182`).
- `SourceGuardrails::compute` reads only `stereo_correlation`; `stereo_width` is never used (`src-tauri/src/guardrails.rs:114-119`).

Impact:

Pearson correlation catches phasey/wide material, but it is a coarse proxy. The app already computes side-energy width, which is closer to the actual "already wide" question for M/S widening.

Recommendation:

Use `stereo_width` as a co-trigger, or remove it from the profile/spec until it is active.

### P2 - Source-profile ownership is split and drift-prone

Evidence:

- Rust has `SourceProfile::from_analysis` (`src-tauri/src/types.rs:135-149`), but `git grep` finds no callers.
- Type comments say export is backend-derived from `AnalysisResult` (`src-tauri/src/types.rs:113-117`), but current Track Master export is frontend-injected (`src/hooks/useTrackMaster.ts:1244-1248`).
- The TS mirror builds the profile separately (`src/lib/settings-transitions.ts:211-223`).

Impact:

This split already explains the preview and slow-lane misses. It also creates a future drift risk if Rust and TypeScript disagree about sentinel handling, fallback values, or added fields.

Recommendation:

Make Rust the source of truth. Either pass `AnalysisResult`/track id into render commands and call `SourceProfile::from_analysis`, or create one shared backend helper used by Track Master render, preview render, fixture matrix, reference tuning, and Album Master if it is brought into scope.

### P2 - The feature lacks the transparency needed to tune it by ear

Evidence:

- The handoff defers "what was trimmed and why" (`docs/HANDOFF_2026-06-02_ADAPTIVE_DSP_TIER1.md:51-56`).
- The computed multipliers are private to `SourceGuardrails`; the UI only exposes a global strength slider (`src-tauri/src/guardrails.rs:80-85`, `src/App.tsx:2137-2145`).

Impact:

The owner can A/B Off vs 60%, but cannot tell whether the change came from bright, boomy, dense, or wide logic. That makes calibration slower and more subjective than necessary.

Recommendation:

Expose read-only per-axis trim readouts in the right rail or Advanced panel, at least in developer/review builds. This should happen before final listening calibration.

### P3 - Test coverage proves the math but not the product promise

Good coverage exists:

- Pure guardrail tests cover identity, deadbands, caps, floor, mono width, and strength scaling (`src-tauri/src/guardrails.rs:216-321`).
- DSP coefficient tests show bright trims high EQ without touching low, and dense/wide changes compression/width (`src-tauri/src/dsp.rs:3099-3183`).
- No-profile byte snapshots protect the old path (`src-tauri/src/dsp.rs:3246-3425`).
- UI tests cover the Adapt Strength slider (`src/App.adaptive-strength.test.tsx`).

Missing coverage:

- No test that Track Master export with a real 6-band analysis sends `advanced.source_profile`.
- No test that offline preview render receives the same profile as export.
- No fixture/reference runner test asserting `advanced.source_profile` is present.
- No render-level multi-axis composition test proving bright+dense+wide caps compose as intended under active delivery LUFS landing.
- No manual override preservation test covering user EQ, Manual compression, and explicit width together.
- No `profile = Some(...)` plus `adaptive_strength = 0` render-level byte identity test.
- No real-pink/neutral profile test for the bright deadband.
- No LRA-sentinel regression test.

Recommendation:

Add the missing mechanical tests before treating the branch as release-candidate evidence. The fastest high-value tests are the export payload profile test, fixture settings profile test, LRA sentinel test, and pink-neutral no-trim test.

### P3 - Spec drift and small polish issues

- The spec says bright touches `presence_db`, but code correctly leaves `presence_db`/`eq_mid_db` alone and trims only `high_mid`, `air`, and `sparkle` (`docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md:177-180`, `src-tauri/src/dsp.rs:736-773`).
- The same default strength is encoded in Rust (`src-tauri/src/guardrails.rs:16-17`) and frontend fallback UI (`src/App.tsx:2137-2145`).
- Album Master guardrails are explicitly out of scope in the handoff (`docs/HANDOFF_2026-06-02_ADAPTIVE_DSP_TIER1.md:51-62`), but Album Master is still a desktop export surface. Product/UI copy should not imply the adaptive layer covers album renders until it does.

## Math Check

The core formulas are coherent:

- Bright: `presence + air`, deadband `0.20`, full raw trim by `0.32`, EQ cap 50%.
- Boomy: `sub + low`, deadband `0.42`, full raw trim by `0.57`, EQ cap 50%.
- Dense: max of P95-P10 DR ramp and LRA ramp, density cap 60%.
- Wide: correlation ramp, width cap 70%, mono does not trim.

The reduce-only mechanics hold:

- `floor_boost` leaves cuts and zero alone and never raises a small boost.
- EQ trims apply to `preset.* * preset_scale` before user EQ is added, so manual EQ remains explicit.
- Width trim pulls only preset width above `1.0` toward neutral, and explicit `advanced.width` bypasses it.
- Compression trim applies only outside Manual mode, and Compressor Off still bypasses creative compression.
- No profile or strength `0` makes `SourceGuardrails` inert.

Those are coefficient-chain guarantees. They do not, by themselves, guarantee that the final rendered master under a Delivery Profile will preserve the same apparent per-axis reduction after LUFS landing. The landing stage can apply broadband gain after the chain when it has target loudness and true-peak headroom, so final rendered A/B behavior needs its own render-level tests.

The caveats are about input validity and interpretation:

- High-band share is not the same as harshness.
- Correlation is not the same as useful stereo width.
- Density macro is not the same as gain reduction or transient preservation.
- A single first-window FFT is not whole-track spectral balance.
- Coefficient caps are not final post-landing loudness or tonal-delta caps.

## Better Path

The best path is not to throw this out. Keep the defensive Tier-1 idea, but move it from "frontend-injected optional helper" to "backend-owned source-aware render behavior."

Recommended sequence:

1. Centralize `SourceProfile` creation in Rust and use it in Track Master export, offline preview render, fixture matrix, and reference tuning. Keep live `playMaster`/`updateChain` covered too, either by preserving the current analyzed-track injection there or by making the backend own/store the analyzed profile for the loaded track.
2. Decide explicitly whether Album Master should remain out of scope. If out of scope, say so in UI/docs. If in scope, inject per-track profiles before `apply_album_shadow` renders each track.
3. Add render-level tests for multi-axis adaptation under active LUFS landing, so coefficient caps and final export behavior are both visible.
4. Fix the LRA sentinel behavior.
5. Replace the first-window spectral balance with averaged windows.
6. Recalibrate bright/low deadbands using pink/reference fixtures, not synthetic dark-neutral fixtures.
7. Use `stereo_width` with correlation for width detection.
8. Add per-axis trim readouts so listening calibration has attribution, and label them as chain trims if they are pre-landing.
9. Add the missing WYSIWYG and slow-lane tests.

Longer-term Tier 2 can add closed-loop checks: compare source vs render dynamic range, true peak, and PSR/crest behavior, then reduce density or warning severity based on measured harm. That would be a better way to honor the "do not make already-mastered sources objectively worse without review" product goal than trying to infer everything from one static profile.

## Bottom Line

This is a promising architecture and the defensive math is mostly well designed. The implementation falls short at the edges that matter for release confidence: some desktop paths and all private evidence lanes do not exercise the guardrails, delivery LUFS landing can change the final rendered meaning of coefficient-level caps, and the current measurements are too provisional to be treated as smart defaults without more visibility and calibration.

Fix the wiring and slow-lane coverage first. Then tune the numbers by ear with per-axis readouts. After that, this becomes a strong v1 rather than a clever but under-verified layer.
