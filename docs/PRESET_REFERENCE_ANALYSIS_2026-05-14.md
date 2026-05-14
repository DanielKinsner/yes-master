# Preset Reference Analysis - 2026-05-14

## Scope

Private fixtures live in `tests for presets/` and are gitignored. The analysis compared:

- `It's a coat-original-test.wav`
- `It's a coat-universal-test.wav`
- `It's a coat-clarity-test.wav`
- `It's a coat-oomph-test.wav`
- `It's a coat-tape-test.wav`

Goal: use the online-mastered references as calibration targets for how different the app's presets should feel, not as an exact clone of another service.

## Method

Each mastered file was compared against the original with two lenses:

1. Raw level and dynamics deltas:
   - RMS level delta
   - peak delta
   - crest factor delta
   - short-term loudness range delta
   - stereo side/mid delta

2. Tone after RMS matching:
   - The mastered file was RMS-matched back to the original.
   - Broad log-frequency bands were measured to see the spectral profile independent of loudness.

Bands:

`20-60`, `60-120`, `120-250`, `250-500`, `500-1k`, `1-2k`, `2-4k`, `4-8k`, `8-16k`

## Reference Results

| Preset | Raw RMS | Peak | Crest | Stereo Width | Main Identity After Level Match |
|---|---:|---:|---:|---:|---|
| Universal | +1.62 dB | +1.96 dB | +0.30 dB | +0.25 dB wider | Mostly neutral, with top-end air |
| Clarity | +0.46 dB | +0.21 dB | -0.15 dB | -0.80 dB narrower | Scooped low-mid / mids, brighter air |
| Oomph | +2.31 dB | +1.72 dB | -0.31 dB | -2.72 dB narrower | Big sub lift, heavy low-mid/mid scoop |
| Tape | +2.13 dB | +0.35 dB | -1.77 dB | -0.07 dB | Strong glue/density, quiet material lifted |

## Tonal Deltas After RMS Match

Values below are dB deltas vs the original after level matching.

| Preset | 20-60 | 60-120 | 120-250 | 250-500 | 500-1k | 1-2k | 2-4k | 4-8k | 8-16k |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Universal | -0.07 | -0.23 | +0.08 | -0.04 | -0.20 | +0.12 | -0.38 | +0.58 | +1.54 |
| Clarity | +0.93 | +0.21 | -0.73 | -1.02 | -1.56 | -2.30 | -1.20 | +1.18 | +2.13 |
| Oomph | +3.77 | +0.06 | -3.31 | -5.92 | -5.40 | -5.47 | -4.44 | -1.56 | -1.61 |
| Tape | -0.61 | -0.30 | +0.20 | +0.53 | +0.14 | -0.97 | -2.44 | +0.97 | +3.67 |

## Interpretation

### Universal

Universal is not completely flat, but it is restrained. It mostly adds level and a clean high-air lift. It is a good baseline target for a broadly acceptable streaming master.

Calibration target:

- Keep tone mostly neutral.
- Add a visible but not aggressive air shelf.
- Small stereo widening is acceptable.
- Avoid obvious compression identity.

### Clarity

Clarity is less about loudness and more about contrast. It reduces the center/mid body and adds top-end air, which makes detail feel more separated.

Calibration target:

- Pull down `500 Hz - 2 kHz` more than the current app likely does.
- Add `8-16 kHz` air.
- Do not make it much louder than Universal.
- Slightly narrower side/mid may be part of the perceived focus.

### Oomph

Oomph is the most dramatic tonal reference. It is not merely "more low end"; it is sub-forward with a very large low-mid/mid scoop. It is also substantially narrower.

Calibration target:

- Strong `20-60 Hz` lift.
- Significant `250 Hz - 2 kHz` cut.
- Less top-end than Clarity.
- Narrower stereo image.
- More level push than Universal.

This is the clearest proof that our current presets need more separation. A preset called Oomph should not be a tiny variation.

### Tape

Tape is mostly a density/glue preset. It is louder, has much lower crest factor, lifts quiet material, and has a strong top-air profile with reduced presence.

Calibration target:

- Compression/glue must be preset-driven.
- Crest factor should drop meaningfully.
- Quiet/body floor should rise.
- Presence around `2-4 kHz` can soften.
- Top air can still lift, but it should feel saturated/smoothed rather than simply bright.

## Current App Gap

The current Rust preset table already contains intent fields such as:

- `compressor_threshold_dbfs`
- `compressor_ratio`
- `transient_punch`
- `target_lufs`

But in the current implementation notes, some of these are captured but not applied. That matches the listening issue: the labels promise dynamics/personality changes, while the audible result is still dominated by modest EQ, modest width, and modest saturation changes.

The app currently appears to need these changes before preset identity will feel convincing:

1. Wire preset-specific compressor threshold/ratio into the live chain.
2. Make preset compression/density part of the preset, not only an advanced global control.
3. Add or wire transient behavior so Punch/Oomph/Loud can feel percussively different.
4. Increase tonal contrast for the bold presets, especially Oomph and Clarity.
5. Treat Universal as the conservative baseline, not the average of all presets.

## Proposed Calibration Targets

These are not final values. They are first-pass direction targets derived from the references.

| Preset | Loudness Push | Compression | Width | EQ Character |
|---|---|---|---|---|
| Universal | Moderate | Light | Slightly wide | Neutral with air |
| Clarity | Low/moderate | Light | Slightly focused/narrow | Mid/body cut, air lift |
| Oomph | Strong | Medium | Narrower | Sub lift, big low-mid/mid scoop |
| Tape | Strong | Strong/glue | Neutral | Softer presence, denser top/air |

## Conservative Preset Mapping

This mapping intentionally backs off from the measured references. The online renders are useful as "how far is obviously different?" anchors, but the app should land slightly more conservative so the presets remain broadly usable.

### Target Delta Philosophy

- Use roughly 50-70% of the measured tonal moves.
- Keep Universal close to transparent.
- Make Clarity and Oomph clearly different even when volume-matched.
- Make Tape different through dynamics and density first, not just EQ.
- Prefer preset-specific compressor behavior over extreme static EQ.

### Reference-to-App Delta Map

| Preset | Reference Behavior | Conservative App Target |
|---|---|---|
| Universal | +1.6 dB RMS, mostly neutral, +1.5 dB top air, slightly wider | +1.0 to +1.5 dB input/loudness push, +0.8 to +1.2 dB air, tiny width lift |
| Clarity | Little level push, -1.5 to -2.3 dB mids, +2.1 dB air, slightly narrower | +0.5 to +1.0 dB push, -0.8 to -1.4 dB low-mid/mids, +1.4 to +1.8 dB air, neutral/slightly focused width |
| Oomph | +2.3 dB RMS, +3.8 dB sub, -5.4 to -5.9 dB low-mid/mids, much narrower | +1.5 to +2.2 dB push, +2.0 to +2.8 dB sub/low shelf, -2.5 to -3.8 dB low-mid/mids, modestly narrower |
| Tape | +2.1 dB RMS, crest -1.8 dB, quiet floor +3.2 dB, softened 2-4k, bright air | +1.2 to +1.8 dB push, crest reduction around -0.8 to -1.3 dB, quiet floor lifted, -1.0 to -1.8 dB presence, +1.5 to +2.4 dB air with saturation |

### Suggested First-Pass Rust Preset Values

These are starting points for `PresetCalibration`, not final production values. They assume the current four EQ anchor areas:

- `low_shelf_db`: broad low/sub weight
- `low_mid_db`: body / mud control
- `presence_db`: mid-forwardness / vocal-guitar focus
- `air_db`: top-end openness

| Preset | low_shelf_db | low_mid_db | presence_db | air_db | warmth/sat | stereo_width | baseline_gain_push_db | compressor target |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Universal | +0.2 | -0.1 | +0.0 | +1.1 | 0.03 | 1.04 | +1.2 | light, transparent |
| Clarity | +0.2 | -1.0 | -0.8 | +1.7 | 0.025 | 1.00-1.04 | +0.8 | light, preserve transients |
| Oomph | +2.4 | -3.0 | -2.6 | -0.8 | 0.045 | 0.92-0.98 | +1.8 | medium, controlled low/mid density |
| Tape | -0.2 | +0.3 | -1.4 | +2.0 | 0.09-0.12 | 0.98-1.00 | +1.5 | glue, audible crest reduction |

### Dynamics Mapping

The app should not rely on EQ alone for these differences. A conservative dynamics map should be wired so preset selection changes compressor behavior even before the user touches advanced controls.

| Preset | Compression Direction | Audible Goal |
|---|---|---|
| Universal | High threshold, low ratio | Clean level lift without obvious pumping |
| Clarity | Similar/light compression, maybe less low-band compression | Detail and separation, not density |
| Oomph | Lower threshold, moderate ratio, controlled low/mid bands | Bigger low end without mud; tighter body |
| Tape | Lower threshold, softer ratio/knee, slower release | Glue, density, smaller crest factor |

Implementation suggestion:

- Treat preset compressor settings as the base.
- Let user `compression_density` scale the base rather than replace it.
- Keep `compression_density = 0` as a true bypass only if the user explicitly disables compression.
- If the UI defaults to a preset, the preset should carry its own dynamic identity.

### Acceptance Check

After tuning, rerun the private fixture analysis and look for this approximate separation:

| Preset | Expected App Result |
|---|---|
| Universal | mostly neutral, `8-16k` about +0.8 to +1.2 dB after match |
| Clarity | mids clearly reduced, air clearly lifted, not much louder than Universal |
| Oomph | strongest tonal contrast; sub/low lift and mid scoop obvious within 5 seconds |
| Tape | crest factor reduced more than Universal/Clarity; quiet sections feel denser |

## Full Preset Personality Map

The four measured references can act as anchor points for the whole preset set:

- Universal = neutral baseline
- Clarity = detail / air / cleanup
- Oomph = weight / low-end impact
- Tape = glue / saturation / density

The remaining presets should be inferred from their product role, not guessed independently. This keeps the whole preset set coherent.

### Product Principle

Presets are not just safe technical defaults. They are creative direction buttons.

At default intensity, each preset should be immediately legible in a short A/B: `Tape` should feel glued and characterful, `Oomph` should feel heavier and more carved, `Clarity` should feel cleaner and more open, and `Punch` should feel tighter and more forward. It is better for presets to start slightly too opinionated and let the user pull them back with `Intensity` than for them to be so cautious that the user has to build the character manually.

The `Intensity` control exists to scale preset character down or up. It should not be required just to make a preset audible.

### Safety Principle

Creatively bold does not mean unsafe.

At default intensity, no factory preset should clip or create an invalid master. Presets that add gain, compression, saturation, or density must also carry appropriate compressor/limiter behavior so the output remains controlled. Preset-level compression and limiting are therefore not optional polish; they are part of the preset's identity and part of the safety contract.

Minimum expectation:

- Factory presets should not clip at default intensity.
- Factory presets should have preset-specific compression/limiting behavior where needed.
- `Loud`, `Punch`, `Oomph`, and `Tape` cannot rely on EQ/input gain alone.
- If a preset adds level or harmonic density, it must also manage peak safety.
- User overrides may push into unsafe territory, but the factory preset starting point should not.

### Preset Roles

| Preset | Role | Primary Difference | What Should Not Happen |
|---|---|---|---|
| Universal | Balanced baseline | Transparent lift, mild air | Should not sound bland or overly processed |
| Clarity | Detail and openness | Mid cleanup, air, vocal/guitar definition | Should not become thin or harsh |
| Tape | Glue and analog character | Saturation, crest reduction, softened presence | Should not simply be darker Universal |
| Spatial | Width and depth | Wider side image, low-mid cleanup, gentle air | Should not get louder or phasey |
| Oomph | Weight and size | Sub/low lift, low-mid scoop, controlled width | Should not become muddy |
| Warmth | Smooth and full | Fuller body, softened presence/air, gentle glue | Should not become muffled |
| Punch | Attack and forwardness | Transients, tighter lows, presence bite | Should not just become Loud |
| Loud | Dense and aggressive | Level push, compression, limiting, forward tone | Should not erase all dynamics |

### Conservative Target Table

These values are still starting targets. They are intentionally bolder than the current table, but softer than the external reference renders.

| Preset | low_shelf_db | low_mid_db | presence_db | air_db | saturation/warmth | width | gain push | compression identity |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Universal | +0.2 | -0.1 | +0.0 | +1.1 | 0.03 | 1.04 | +1.2 | light transparent |
| Clarity | +0.2 | -1.0 | -0.8 | +1.7 | 0.025 | 1.02 | +0.8 | light, transient-preserving |
| Tape | -0.2 | +0.3 | -1.4 | +2.0 | 0.10 | 0.99 | +1.5 | glue, crest reduction |
| Spatial | +0.1 | -0.8 | -0.3 | +1.3 | 0.04 | 1.16 | +1.0 | light, clean sides |
| Oomph | +2.4 | -3.0 | -2.6 | -0.8 | 0.045 | 0.95 | +1.8 | medium, low/mid control |
| Warmth | +0.8 | +0.7 | -1.8 | -0.8 | 0.08 | 0.98 | +1.0 | soft glue |
| Punch | +0.8 | -1.8 | +1.6 | +0.8 | 0.035 | 1.04 | +1.6 | faster attack/release, transient-forward |
| Loud | +0.4 | -1.6 | +1.8 | +1.2 | 0.055 | 1.03 | +2.5 | strongest density/limiting |

### Dynamics Targets

The compressor behavior is probably the most important missing distinction. Suggested relative mapping:

| Preset | Threshold Direction | Ratio Direction | Attack/Release Feel | Expected Result |
|---|---|---|---|---|
| Universal | Higher | Lower | Medium/transparent | Gentle polish |
| Clarity | Higher | Lower | Faster recovery | Keeps articulation open |
| Tape | Lower | Moderate | Slower/smoother | Glue and density |
| Spatial | Higher | Lower | Transparent | Width without pumping |
| Oomph | Lower | Moderate | Controlled low/mid behavior | Weight without mud |
| Warmth | Medium-low | Low/moderate | Smooth | Softened, rounded body |
| Punch | Medium-low | Moderate/high | Fast and lively | More impact and forwardness |
| Loud | Lowest | Highest | Controlled/dense | Assertive loud master |

### Inferred Preset Notes

#### Spatial

Spatial should sit between Universal and Clarity tonally, but with the widest side image. It should feel wider and more dimensional, not louder. The low-mids should clean up enough that the width does not feel cloudy.

Target feel:

- Wider than Universal by a meaningful amount.
- Slightly less mid/body than Universal.
- Similar or slightly brighter top.
- Light compression only.

#### Warmth

Warmth is not Tape. Tape should be character/density; Warmth should be smoothness and fatigue reduction. It should make a bright track easier to listen to without crushing it.

Target feel:

- Fuller lower body.
- Softer presence and air.
- Gentle saturation.
- Less width than Spatial, roughly neutral.

#### Punch

Punch should be about impact and articulation. Compared to Oomph, it should have less sub scoop drama and more upper-mid/presence attack. Compared to Loud, it should preserve more movement.

Target feel:

- Tight lows, not huge lows.
- Cut some mud.
- Add presence bite.
- Add transient behavior once available.

#### Loud

Loud should be the most assertive preset, but it should still sound like a master, not a smashed preview. It gets the most gain push and density, with less tonal extremity than Oomph.

Target feel:

- Most level push.
- Strongest compression/limiting.
- Forward presence and air.
- Some dynamics remain.

## Suggested Claude Task List

1. Wire preset compressor fields into the live Rust chain:
   - `compressor_threshold_dbfs`
   - `compressor_ratio`
   - preset-scaled compression density

2. Retune the `PresetCalibration` table using the conservative target table above.

3. Preserve user overrides:
   - user EQ sliders add to preset EQ
   - user width can override or scale preset width
   - user compression density scales preset compression rather than erasing it

4. Add a regression/contract test that proves presets are meaningfully distinct:
   - Universal vs Clarity: high-air and mid/body profile differ
   - Universal vs Oomph: sub and low-mid profile differ strongly
   - Universal vs Tape: crest/density behavior differs
   - Punch vs Loud: Punch keeps more crest than Loud

5. Rerun the private preset analysis on app-rendered outputs and compare against this report's acceptance checks.

6. Add preset safety checks:
   - no factory preset clips at default intensity on the private fixture
   - limiter/ceiling behavior is active after preset gain, EQ, saturation, and compression
   - peak control is verified after the full chain, not before it
   - a preset can be stylistically bold, but it cannot start broken

## Recommended Next Implementation Slice

Do not tune only the EQ table. That will probably keep the presets feeling too similar.

Recommended order:

1. Apply preset-specific compressor threshold and ratio in `ChainCoeffs::from_settings`.
2. Decide how preset compression interacts with the user `compression_density` control:
   - likely: preset provides the base, user control scales it.
3. Retune the four known presets against the measured reference identities.
4. Render the same fixture through the app and rerun this analysis.
5. Iterate by listening plus objective deltas.

## Good Enough Target

The goal is not to null-match the online masters. The target is:

- Universal, Clarity, Oomph, and Tape should be immediately distinguishable in a 5-second A/B.
- Preset differences should survive volume match.
- Oomph and Tape should have obvious dynamics/feel differences, not just tonal differences.
- Clarity should sound clearer without just becoming harsh.
- Universal should stay useful and conservative.
