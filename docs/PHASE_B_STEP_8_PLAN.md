# Phase B+ Step 8 — Validation Sound Tests

This is the implementing spec for the **next** session that picks the project
up. Phase A1–A5 and Phase B Steps 1–7 are done and on master. Step 8 is the
remaining unfinished workstream — a battery of programmatic tests that validate
the listening-quality work end-to-end, without requiring Dan's ears.

Read this top-to-bottom, then start. No further plan-doc work is needed.

---

## Bootstrap — where everything lives

### Repo state at the time this doc was written

- **Repo path (Dan's old machine):** `C:\Users\SM - Dan\Documents\GitHub\album-mastering-studio-claude-build`
- **Remote:** `https://github.com/Dannytownkins/album-mastering-studio-claude-build.git`
- **Master HEAD:** `e947751` — Merge phase-b-album-master Step 7
- **Branch:** `phase-b-album-master` is at `f6e1d31` (Step 7), already merged into master. Continue Step 8 on a new branch from master.
- **Tests at this snapshot:** 74 lib + 39 contracts + 2 album_render = **115/115** green. `npm run build` clean. `phase_12_1_real_fixture_metering_snapshot` (Track Master byte-identity verifier) passes.

### Required reading order (fresh agent)

1. `CLAUDE.md` — repo non-negotiables. **Do not read source from `../album-mastering-studio/` (Codex repo) unless you explicitly need a reference value not captured here.** Even then, treat it as read-only and don't import.
2. `docs/PRODUCT.md` — product canon. Read-only unless Dan explicitly asks for an edit.
3. `docs/HANDOFF.md` — rolling pointer to the latest dated handoff. The most recent dated handoff at the time of writing is `HANDOFF_2026-05-13_session.md`; the work done since then is captured in `docs/progress.md` tail entries and the commit log.
4. `docs/ALBUM_MASTER_PLAN.md` — the design doc Phase B was built from. Anti-features list is still binding.
5. `docs/album-mastering-port-plan-v2.md` — note: this lives at `C:\Users\SM - Dan\Downloads\album-mastering-port-plan-v2.md` on Dan's old machine, NOT in the repo. If it's needed on the new machine, Dan can drop it back into Downloads or paste into a fresh handoff doc. The phases (A1–A5 + B) are all complete; only Step 8 from the post-B+ extension remains.
6. **This doc** (`docs/PHASE_B_STEP_8_PLAN.md`).
7. `docs/progress.md` tail entry for the most recent completed phase.

### Don't read by default

- `docs/reference/` — Codex-path optional context, not startup material.
- `docs/research/` — pre-A1 research dumps. The numbers from these landed in the code already; reading them now mostly produces conflicting prior states.
- `../album-mastering-studio/` — Codex repo. The Codex source files Step 8 might reference are listed inline below; you don't need to read anything else from over there.

### Where Phase B code lives

| Concern | Path |
|---|---|
| AlbumPlan types | `src-tauri/src/types.rs` (search for `AlbumPlan`, `AlbumArc`, `TransitionSpec`, `AlbumTrackEntry`, `AlbumCharacter`) |
| Plan math | `src-tauri/src/album.rs` (resample_arc_curve, character_loudness_offset, mastering_bias_for, infer_album_characters, build_album_plan) |
| Render pipeline | `src-tauri/src/engine.rs` — `render_album_plan_impl` (~line 1048), `apply_album_shadow`, `plan_album` Tauri command, `render_album_plan` Tauri command |
| Frontend bindings | `src/bindings.ts` (search for `AlbumPlan`, `AlbumArc`, `AlbumCharacter`) |
| Hook state | `src/hooks/useTrackMaster.ts` (search for `albumArcKind`, `albumIntensity`, `exportAlbumPlan`) |
| UI component | `src/components/AlbumPanel.tsx` |
| Browser preview mock | `src/lib/preview-mock.ts` (handles plan_album / render_album_plan offline) |
| Existing tests | `src-tauri/src/album.rs::mod tests`, `src-tauri/tests/album_render.rs` |

### Where the per-phase numbers came from

If a number in the code doesn't make sense, the Codex source files it was ported from are:

| Our code | Codex source |
|---|---|
| `AlbumArcKind::curve()` | `../album-mastering-studio/src/album_mastering_studio/arc.py:70-99` (ARC_PRESETS dict) |
| `character_loudness_offset` | `arc.py:287-299` (`_character_loudness_offset`) |
| `mastering_bias_for` | `arc.py:302-352` (`_mastering_bias`) |
| `infer_album_characters` | `../album-mastering-studio/src/album_mastering_studio/character.py:24-58` |
| `heavy_score / acoustic_score / transition_score` | `character.py:146-172` |
| `resample_arc_curve` | `arc.py:202-218` (`_resample_curve`) |
| Preset 13-number calibration | `../album-mastering-studio/src/album_mastering_studio/mastering.py:96-357` (PRESETS dict) |
| K-weighting reference | ITU-R BS.1770-4 Annex 1, and `pyloudnorm.k_filter` equivalent |
| Delivery profiles | `../album-mastering-studio/src/album_mastering_studio/standards.py` |

### Verification commands

Run these in order before changing anything. If any fail, stop and diagnose.

```powershell
# Repo root: C:\Users\SM - Dan\Documents\GitHub\album-mastering-studio-claude-build

# 1. Backend lib unit tests (~1 second).
cd src-tauri
cargo test --lib

# 2. Backend full suite including the ~4-minute real-fixture metering snapshot.
cargo test

# 3. Frontend production build.
cd ..
npm run build

# 4. (optional) Browser-preview smoke. Vite alone, no Tauri.
npm run dev
# Then in another shell:
agent-browser open http://localhost:1420
agent-browser set viewport 1600 940
agent-browser screenshot test-output/sanity.png
```

Expected at master HEAD `e947751`:

- `cargo test --lib`: 74/74 pass
- `cargo test`: 115/115 pass (lib + contracts + album_render)
- `npm run build`: clean, ~280 KB raw / ~87 KB gzipped main chunk
- Preview opens with a one-track mock loaded and meters animating when you click play.

---

## Step 8 — the goal

Write integration tests that validate the listening-quality work programmatically. The Phase A and Phase B work changed real audio output across many dimensions. Manual listening from Dan catches what we shipped; these tests catch **regressions** when future changes silently shift behavior.

Stop condition for Step 8:

- All seven test files below exist under `src-tauri/tests/` (or `src-tauri/src/dsp.rs::mod tests` for the K-weighting curve test, since that one needs access to crate-internal helpers).
- `cargo test` runs them all and all pass.
- They run in under ~3 minutes total (additive to the existing ~4 minutes for `phase_12_1_real_fixture_metering_snapshot`).
- Each test has a clear assertion message that names the property it's checking, so when it fails in the future the diff tells you what regressed.

Commit each test file as its own commit on a `phase-b-step-8-validation` branch off master. Merge to master only after all seven pass.

---

## The seven tests

### Test 1 — Per-preset character signature

**Goal:** detect "did I wire preset X to the wrong calibration tuple" regressions.

**File:** `src-tauri/tests/preset_signature.rs` (new integration test).

**Signal:** synthesize 2 seconds of pseudo-white noise at 48 kHz (deterministic LCG, peak `-12 dBFS`) as the input — same input through every preset. Stereo (interleaved).

**Procedure:** for each of the 8 presets at default intensity 0.5, build a `MasteringChain` and process the input. After processing, compute:

- Integrated LUFS via `measure_integrated_lufs` (already in `engine.rs`).
- Magnitude of a Goertzel filter at 200 Hz, 400 Hz, 1500 Hz, 6000 Hz (one per band the preset table tunes). A 4-line Goertzel implementation per frequency is fine; no FFT needed.

**Assertions (concrete dB targets, per preset):**

- **Universal** (low_shelf=0, low_mid=0, presence=0, air=+0.8): the 6 kHz band must be ≥ +0.5 dB above the input; the 200 Hz / 400 Hz / 1500 Hz bands within ±0.5 dB of the input.
- **Clarity** (low_shelf=-0.2, low_mid=-0.7, presence=+0.9, air=+2.2): 400 Hz < -0.3 dB; 1500 Hz > +0.5 dB; 6 kHz > +1.5 dB.
- **Tape** (low_shelf=+1.2, low_mid=+0.25, presence=-0.65, air=-0.15, warmth=0.095): 200 Hz > +0.8 dB; 1500 Hz < -0.3 dB; AND the output must contain measurable saturation (compute the 3rd-harmonic of an injected 1 kHz sine — see helper below — and assert it's > -50 dBFS).
- **Spatial** (low_shelf=+0.9, low_mid=-0.65, presence=-0.15, air=+1.35, width=1.13): 6 kHz > +1.0 dB; AND when fed a stereo `(L, -L)` antiphase signal, the post-chain side-signal RMS must be > 1.1× the pre-chain side-signal RMS (verifies the M/S widener engages).
- **Oomph** (low_shelf=+0.6, low_mid=-1.25, presence=+1.1, air=+0.85): 400 Hz < -1.0 dB (the mud-zone cut); 1500 Hz > +0.5 dB.
- **Warmth** (low_shelf=+0.8, low_mid=+0.1, presence=-1.2, air=-0.9, warmth=0.075): 1500 Hz < -0.8 dB; 6 kHz < -0.5 dB; with saturation detectable (1 kHz 3rd-harmonic > -55 dBFS).
- **Punch** (low_shelf=+1.0, low_mid=-1.9, presence=+1.8, air=+1.2): 400 Hz < -1.5 dB (deepest mud-zone cut); 1500 Hz > +1.0 dB.
- **Loud** (low_shelf=+0.4, low_mid=-1.5, presence=+1.7, air=+1.35): 400 Hz < -1.0 dB; 1500 Hz > +1.0 dB; 6 kHz > +0.8 dB.

Goertzel snippet (one frequency, single-precision, mono input — call once per band):

```rust
fn goertzel_mag_db(samples: &[f32], sample_rate: f32, freq_hz: f32) -> f32 {
    let omega = 2.0 * std::f32::consts::PI * freq_hz / sample_rate;
    let coeff = 2.0 * omega.cos();
    let mut q1 = 0.0_f32;
    let mut q2 = 0.0_f32;
    for &s in samples {
        let q0 = coeff * q1 - q2 + s;
        q2 = q1;
        q1 = q0;
    }
    let mag = (q1 * q1 + q2 * q2 - coeff * q1 * q2).max(1e-30).sqrt();
    // Normalize by sample count so it's comparable to RMS-level metrics.
    20.0 * (mag / samples.len() as f32).log10()
}
```

3rd-harmonic-of-1-kHz-sine helper (one preset, mono input, used for saturation detection):

```rust
fn third_harmonic_db_at_1khz(samples: &[f32], sample_rate: f32) -> f32 {
    goertzel_mag_db(samples, sample_rate, 3000.0)
}
```

For the saturation test, feed a 2-second `0.5 · sin(2π·1000·t)` mono signal (duplicated to stereo) and check `third_harmonic_db_at_1khz(output_left_channel, 48_000.0)` against the threshold.

### Test 2 — Inter-preset loudness consistency

**Goal:** catch "preset X is unintentionally 4 LU louder than the rest" regressions like the Dan-flagged Tape-too-loud listening note that landed in `2026-05-12`.

**File:** `src-tauri/tests/preset_loudness_balance.rs`.

**Signal:** same 2-second pink-ish noise input (Paul Kellet pink filter on deterministic LCG white at 48 kHz, peak `-12 dBFS`, stereo).

**Procedure:** for each of the 8 presets at intensity 0.5, build a `MasteringChain` and process the input. Measure integrated LUFS of the output via `measure_integrated_lufs`.

**Assertion:** the per-preset integrated LUFS values must satisfy `max - min < 4.0 LU`. No preset is allowed to land more than 4 LU away from the loudest or quietest sibling.

This is a regression net — the Codex preset calibration was tuned to be roughly loudness-matched at default intensity. A change that breaks that balance is suspicious and the test forces a re-think.

### Test 3 — Delivery profile end-to-end

**Goal:** verify the `effective_*` shadow plumbing from `MasteringSettings` through `ChainCoeffs` and `mastering_render_with_progress` honors the profile values for every non-Custom variant.

**File:** `src-tauri/tests/delivery_profile_render.rs`.

**Signal:** a 5-second 1 kHz sine at peak `0.3` (≈ `-10 dBFS` RMS) stereo. Written to a tempfile WAV.

**Procedure:** for each delivery profile (StreamingUniversal, AppleMusic, Cd, VinylPremaster, LoudRock, BroadcastEu, BroadcastUs):

1. Build `MasteringSettings { preset: Universal, delivery_profile: <P>, ... }`.
2. Call `mastering_render_with_progress(track_id, source_path, &settings, out_dir, RenderKind::Master, None)`.
3. Open the rendered WAV via `hound::WavReader`. Decode samples.
4. Measure integrated LUFS via `measure_integrated_lufs`.
5. Read the WAV spec — `bits_per_sample`.

**Assertions per profile:**

- Measured LUFS within `±1.0 LU` of the profile's `target_lufs()`. (The 1 LU tolerance is generous because the chain doesn't loudness-target precisely yet — landing is best-effort attenuation only, no upward gain. Tighten when an iterative loudness-targeting pass lands in a later phase.)
- Output WAV `bits_per_sample` matches `profile.output_bit_depth()`.
- For Custom: explicit advanced fields (lufs_offset_db = -12.0, bit_depth = 16) must be honored.

### Test 4 — Album arc curve trace

**File:** `src-tauri/tests/album_arc_trace.rs`.

**Goal:** verify the Cinematic curve `(0.32, 0.52, 0.78, 1.00, 0.70, 0.46)` actually shapes per-track LUFS through the full pipeline, not just in `build_album_plan`.

**Signal:** 6 identical synthetic mono WAVs (2 s of 1 kHz sine at peak `0.3`, 48 kHz), each named `arc-trace-N.wav`. Same source so character / energy / source-comp don't muddy the assertion.

**Procedure:**

1. Build 6 fake `AnalysisResult`s with `energy_density_score = Some(0.5)` (neutral) and `inferred_character = None` (no character bias).
2. Call `album::build_album_plan` with `AlbumArc::Preset { preset: Cinematic }` and `intensity = 1.0`.
3. Build the `AlbumPlanRenderRequest` and call `engine::render_album_plan_impl`.
4. For each rendered per-track WAV, measure integrated LUFS.

**Assertions:**

- Per-track measured LUFS must follow the cinematic shape: `LUFS[3]` (the peak position) is the **highest**; `LUFS[0]` and `LUFS[5]` are both below `LUFS[3]` by at least `2.0 LU` each.
- The non-decreasing segment indices 0→3 must be monotonically non-decreasing (within ±0.5 LU jitter tolerance for measurement noise).

### Test 5 — Album character promotion + bias landing

**File:** `src-tauri/tests/album_character_bias.rs`.

**Goal:** verify the position-aware character system actually lands different EQ on a `HeavyDjent` track vs the same-intrinsic track in slot 1, and verify the `AcousticFolk → ReturnAcoustic` promotion fires.

**Signal:** 4 synthetic tracks, 2 s each at 48 kHz stereo:

- Track 1 (`"acoustic-intro.wav"`): full-range pink-ish noise at peak 0.25 (deterministic LCG → Paul Kellet pink).
- Track 2 (`"djent-banger.wav"`): a saw-like high-pass signal — square wave at 110 Hz with the fundamental + ~6 odd harmonics, peak 0.5 (high `energy_density`, low crest, percussive).
- Track 3 (`"return-quiet.wav"`): same pink-ish signal as track 1 but at peak 0.15 (lower energy).
- Track 4 (`"closer.wav"`): pink-ish at peak 0.2.

Filename hints push the inference: `acoustic-intro` → `AcousticFolk`, `djent-banger` → `HeavyDjent`, `return-quiet` → (no hint, but it'll get promoted to `ReturnAcoustic` because it's `AcousticFolk` in the back half after the heavy).

**Procedure:** call `album::build_album_plan_with_names` with these 4 tracks, the filename strings, Cinematic arc, intensity 1.0. Render via `render_album_plan_impl`. Measure per-track LUFS and FFT-band magnitudes (use the Goertzel helper from Test 1).

**Assertions:**

- `plan.tracks[0].album_character == Some(AcousticFolk)`.
- `plan.tracks[1].album_character == Some(HeavyDjent)`.
- `plan.tracks[2].album_character == Some(ReturnAcoustic)` — verifies the back-half-after-heavy promotion.
- Heavy track's measured 400 Hz Goertzel magnitude is at least `1.0 dB lower` than the AcousticFolk track 1's, even though the input pink+square signals have similar low-mid energy — because heavy's `mastering_bias_for` cuts low_mid by -0.55 dB on top of whatever the preset does, and the saw harmonics put 110 Hz × 3 ≈ 330 Hz energy in the bias band.
- Heavy track's measured integrated LUFS is at least `1.5 LU louder` than ReturnAcoustic's (heavy +0.82 LUFS offset vs return -1.05 LUFS offset = 1.87 LU expected gap).

### Test 6 — TPDF dither absence-of-harmonics

**File:** `src-tauri/tests/dither_absence_of_harmonics.rs`.

**Goal:** the existing `tpdf_dither_decorrelates_quantization_at_minus_90_dbfs` unit test in `engine.rs::mod tests` checks distinct-value count; this stronger version uses Goertzel filters to assert "no harmonics of 1 kHz appear above the noise floor in the dithered 16-bit output."

**Signal:** 1 second of a 1 kHz sine at `-90 dBFS RMS` mono at 48 kHz.

**Procedure:**

1. Render to 16-bit WAV through the engine.rs writer (which already uses TPDF dither — Phase A4).
2. Decode the rendered WAV back to f32 samples.
3. Run a Goertzel filter at 1 kHz (fundamental), 2 kHz, 3 kHz, 4 kHz, 5 kHz (harmonics).
4. Compute the noise floor: average Goertzel magnitudes at 7 random non-harmonic frequencies (e.g. 1234 Hz, 2345 Hz, 4567 Hz, 6789 Hz, 8901 Hz, 11000 Hz, 14500 Hz).

**Assertions:**

- The 1 kHz fundamental magnitude is `> noise_floor + 15 dB` (signal is detectable).
- Every harmonic (2k–5k Hz) magnitude is `< noise_floor + 6 dB` (no clear harmonic peaks above the noise floor).

This is the textbook "dither replaces correlated quantization distortion with uncorrelated white noise" property, verified spectrally.

### Test 7 — K-weighting transfer function at canonical frequencies

**File:** add to `src-tauri/src/dsp.rs::mod tests` (the K-weighting builders are crate-internal; this test needs access to `biquad_magnitude_db_at`, which already lives in the test module).

**Goal:** verify the BS.1770-4 K-weighting cascade response at canonical frequencies matches the published reference (within ±0.3 dB) so a future filter-form change can't silently break conformance.

**Procedure:** at 48 kHz sample rate, compute the cascade magnitude `mag_pre(f) + mag_rlb(f)` (dB add since they're in series) at:

- 20 Hz
- 60 Hz
- 100 Hz
- 1000 Hz
- 2000 Hz
- 6000 Hz
- 10000 Hz

**Assertions (target dB, ±0.3 dB tolerance):**

| Hz | Target dB |
|---|---|
| 20 | -14.5 |
| 60 | -3.8 |
| 100 | -1.2 |
| 1000 | 0.0 |
| 2000 | +1.5 |
| 6000 | +3.5 |
| 10000 | +4.0 |

(These are reference values from the published BS.1770-4 curve at 48 kHz, approximate to 1 decimal place. If you need the exact figures Codex used as their ground truth, run their `loudness.py` with the same sample rate; do NOT depend on Codex source running for this test — paste the numbers into the test file.)

---

## What to do NOT to do in Step 8

Per the original codex port plan v2 and the `docs/ALBUM_MASTER_PLAN.md` anti-features list — Step 8 is **only tests**. Do NOT:

- Add new product features (no "Album Insights" card, no per-transition spinner, no Custom-arc editor).
- Refactor the existing render path or DSP modules. The tests should pass against the code AS-IT-IS. If a test fails because the code is genuinely wrong, surface that to Dan rather than silently fixing.
- Add new external crate dependencies (Goertzel and FFT-free assertions are deliberate to keep build time down).
- Touch frontend code. Step 8 is backend-only.
- Read or copy code from `../album-mastering-studio/` — by Step 8 every numeric value we depend on is already in our codebase.
- Re-run the existing `phase_12_1_real_fixture_metering_snapshot` test — it takes ~4 minutes and is unrelated; the new tests should not piggyback on real-fixture audio.

If a test you write reveals a real bug in the code, **stop and report** rather than fixing it. The bug fix is a separate slice with its own commit and a clear "found via Step 8 Test N" footer.

---

## Test infrastructure / synthesis helpers

Each test file should keep its synthesis helpers local (not factor them into a shared module yet — premature). Common patterns:

- **Deterministic LCG white noise:**
  ```rust
  let mut state: u32 = 0xCAFE_BABE;
  let next = |s: &mut u32| -> f32 {
      *s = s.wrapping_mul(1103515245).wrapping_add(12345);
      ((*s >> 16) & 0x7FFF) as f32 / 32768.0 - 0.5
  };
  ```
- **Paul Kellet pink:** see `src-tauri/src/dsp.rs::mod tests::momentary_lufs_pink_noise_at_minus_23_dbfs_reads_minus_23_within_half_lu` for the reference six-stage IIR.
- **Stereo synthesis:** generate mono then interleave `[L, R, L, R, ...]` with the same value for both, or vary L/R for stereo-correlation / widener tests.
- **Mono WAV write:** see `tests/album_render.rs::write_wav_mono` for a working hound-based helper. For stereo, pass `WavSpec { channels: 2, ... }` and write samples interleaved.
- **Temp dirs:** the `tempfile` crate is already a dev-dependency in `src-tauri/Cargo.toml`. Use `tempfile::TempDir::new()` and let the destructor clean up.

---

## Acceptance / commit structure

Each test is one commit on a `phase-b-step-8-validation` branch off master, in the order listed above. Commit message format:

```
Phase B+ Step 8.<N>: <test name>

<one-paragraph description of what the test verifies>

<key assertions in bullet form>

cargo test --test <file>: 1/1 pass.
cargo test (full): <N+old>/<N+old> pass.
```

After all 7 land, merge `phase-b-step-8-validation` into master via:

```bash
git checkout master
git merge --no-ff phase-b-step-8-validation -m "<merge message>"
git push origin master
```

Stop conditions for Step 8 closure, all required:

- All 7 test files exist and pass individually (`cargo test --test <file>` succeeds for each).
- `cargo test` runs every test green (lib + contracts + album_render + new step-8 tests).
- `npm run build` clean.
- Track Master single-track byte-identity preserved: `phase_12_1_real_fixture_metering_snapshot` still passes verbatim.

If any pre-existing test breaks because of test-data interference (filesystem path collisions, port conflicts, etc.), the new test is the one to fix — never weaken the existing assertions.

---

## After Step 8

Step 8 is the final scheduled item from the codex port plan v2. After Step 8 closes, the open queue is empty and the next direction comes from Dan's listening pass. Likely candidates for future work, in rough priority:

1. Album mode UI polish: per-transition Gap-seconds spinner, drag-to-reorder, "Album: -1.05 LUFS / ×0.94 intensity" badge on the workspace.
2. Real-album smoke against `private-audio-fixtures/` when Dan has tracks loaded.
3. Sample-rate resampling (currently DeliveryProfile carries `output_sample_rate()` but the renderer writes at source SR — captured but not applied).
4. Custom-arc UI editor (the data model supports `AlbumArc::Custom { lufs_offsets }` but there's no UI).

None of these are blocking. Phase A1–A5 + Phase B + Phase B+ Step 6/7 is the full ported feature set.

---

## Memory / preferences carried forward

If memory isn't loading on the new machine (different hostname / home path):

- **Repo autonomy:** high. Install deps, run tests, commit + push to master when work is verified. **Do NOT merge feature branches to master without explicit "merge X to master" from Dan** — the post-Phase-B revert chain is what happens when you skip that gate.
- **No check-in chatter:** when Dan says "dive in" / "keep iterating", chain commits. Don't `AskUserQuestion` between every slice.
- **Listening calls are Dan's:** sound-quality decisions only happen when Dan signals "I listened to it." Don't fake them.
- **PRODUCT.md is canon:** read-only without explicit ask.
- **Codex repo (`../album-mastering-studio/`) is reference-only:** don't copy source.

Commit message footer:
```
Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```
