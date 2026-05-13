# Compression Density (3-band Multiband) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the existing `compression_density` Advanced slider (currently unwired, labeled "Compression (coming soon)") into a real 3-band linked-stereo downward compressor with engineer-grade per-band overrides exposed at the same time. Macro slider stays the one-knob entry point; 12 per-band override fields + 1 stereo-link checkbox sit alongside it in `AdvancedSettings`. `None` lets the macro drive; `Some(v)` overrides for that band/parameter only.

**Architecture:** Linkwitz–Riley 4th-order crossovers at 120 Hz (low/mid) and 4000 Hz (mid/high) — each crossover is a cascaded Butterworth Q≈0.7071 LP + LP and HP + HP pair, so 8 biquads per channel for the split network. Per-band peak-detector envelope followers drive soft-knee (6 dB) downward compression. Linked stereo by default (max of |L| and |R| per band drives a shared gain reduction); unlinked path runs independent L/R envelope followers when the user toggles it. Auto makeup gain per band. New 3-band Gain-Reduction snapshot atomics mirror the existing `peak_linear` `Arc<AtomicU32>::fetch_max` pattern on `audio.rs`, swap-and-reset every 50 ms tick. Identity early-return when slider is unset AND all 12 per-band overrides are `None` AND `link_stereo` isn't `Some(false)` — byte-equivalent path preserves all existing real-fixture tests. Chain position: between `presence_air` (high-shelf) and `width` (M/S transform); mirror in the legacy `process_sample` path. Backed by `docs/superpowers/brainstorms/2026-05-12-compression-density-brainstorm.md`.

**Tech Stack:** Rust (Tauri backend, DSP math in `src-tauri/src/dsp.rs`, atomics + audio thread wiring in `src-tauri/src/audio.rs`, types in `src-tauri/src/types.rs`, export-check advisory in `src-tauri/src/exports.rs`). TypeScript/React (`src/App.tsx`'s `AdvancedPanel` + `StaleBar`, types in `src/bindings.ts`, transport state in `src/hooks/useTrackMaster.ts`). Styling in `src/App.css`.

---

## File Structure

- **Modify** `src-tauri/src/types.rs`:
  - `AdvancedSettings` struct → 12 new `Option<f32>` per-band override fields + 1 `Option<bool>` link-stereo (all `#[serde(default)]`).
  - `PlaybackTick` struct → 3 new f32 fields (`gr_low_db`, `gr_mid_db`, `gr_high_db`) with `#[serde(default = "default_silence_dbfs")]`.
- **Modify** `src-tauri/src/dsp.rs`:
  - `ChainCoeffs` → new fields for crossover biquad coefficients + per-band thresholds/ratios/attack-release alphas/makeup gains + `compression_link_stereo` + `compression_active` (skip-flag for early-return identity).
  - `ChainCoeffs::from_settings` → compute LR4 crossover coefficients, per-band macro→threshold mapping with per-band overrides, attack/release alphas, makeup gains.
  - `ChannelState` → new fields for 8 per-channel crossover biquad states (low LP1, low LP2, mid-from-HP1, mid-from-HP2, mid-from-LP1, mid-from-LP2, high HP1, high HP2) + 3 per-channel envelope-follower states.
  - `MasteringChain` → 3 new `Arc<AtomicU32>` GR slots (one per band) + getters; new constructor variant that accepts shared atomics.
  - `MasteringChain::process_frame_inplace` → multiband compressor block inserted between `presence_air` and the width transform, with identity early-return when inactive.
  - `MasteringChain::process_sample` (legacy path) → mirror the multiband stage in the same chain position.
  - `mod tests` → 8 new unit tests.
- **Modify** `src-tauri/src/audio.rs`:
  - `AudioThreadState` → 3 new `Arc<AtomicU32>` GR slots, mirror of the existing `peak_linear` pattern.
  - `MasteringSource` → store the 3 shared atomics, plumb them into the contained `MasteringChain` so the chain's per-frame GR snapshots feed the audio-thread atomics.
  - Snapshot tick (~50 ms loop at line 733) → swap-and-convert the 3 GR atomics into the `PlaybackSnapshot` (extend the snapshot struct + plumb through to the tick emission).
  - `handle_play` / `handle_play_master` → initialize/reset the 3 new atomics.
  - `PlaybackSnapshot` → 3 new f32 fields mirroring `peak_dbfs`.
- **Modify** `src-tauri/src/exports.rs`:
  - `run_export_checks` → extend signature with two optional context arguments (`source_analysis: Option<AnalysisResult>` and `settings: Option<MasteringSettings>`), preserving backward compatibility with existing contract tests that pass only the `ExportReport`. Add the `comp_density_on_compressed_source` advisory when source DR < 6.0 AND `compression_density > 0.3` AND no per-band threshold overrides set.
- **Modify** `src-tauri/tests/contracts.rs`:
  - Update every existing `run_export_checks(report)` call site to `run_export_checks(report, None, None)`.
  - 2 new tests at end (`mastering_render_with_heavy_compression_attenuates_loud_section`, `run_export_checks_warns_on_compressed_source_with_heavy_density`).
- **Modify** `src/bindings.ts`:
  - `AdvancedSettings` → 13 new fields.
  - `PlaybackTick` → 3 new f32 fields.
- **Modify** `src/hooks/useTrackMaster.ts`:
  - `transport` state → add `compressionGr: { low: number; mid: number; high: number }`.
  - `onPlaybackTick` handler → pull the new fields into transport.
- **Modify** `src/App.tsx`:
  - `AdvancedPanel` → drop "(coming soon)" suffix on `compression_density` label; add a collapsible "Per-band" subsection with 3 columns × 4 `NumberField`s each + a "Link stereo" checkbox at the top.
  - `StaleBar` → 3 small per-band GR readout chips alongside `ClippingIndicator`.
  - New `GrIndicator` component (or 3 reusable instances of one).
- **Modify** `src/App.css`:
  - 3 new `.gr-indicator` style rules paralleling `.clip-indicator`.
- **Modify** `docs/progress.md`: append a progress entry under the loop convention.
- **Create**: none (no new files).

Each task is self-contained; the slice commits as a single push at the end of Task 12.

---

## Task 1: Extend the type schema (`AdvancedSettings` + `PlaybackTick`)

**Files:**
- Modify: `src-tauri/src/types.rs` (`AdvancedSettings`, `PlaybackTick`)

The Rust schema lands first because every downstream file references these fields by name. No tests run at the end of this task — the new fields don't have observable behavior yet; that lands in Task 2.

---

- [ ] **Step 1.1: Add 13 new per-band fields + link-stereo to `AdvancedSettings`**

Locate `AdvancedSettings` (around line 134 of `types.rs`):

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdvancedSettings {
    pub lufs_offset_db: Option<f32>,
    pub ceiling_dbtp: Option<f32>,
    pub width: Option<f32>,
    pub warmth: Option<f32>,
    pub presence_air: Option<f32>,
    pub compression_density: Option<f32>,
    pub bit_depth: Option<u16>,
    pub target_sample_rate: Option<u32>,
}
```

Replace with:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdvancedSettings {
    pub lufs_offset_db: Option<f32>,
    pub ceiling_dbtp: Option<f32>,
    pub width: Option<f32>,
    pub warmth: Option<f32>,
    pub presence_air: Option<f32>,
    pub compression_density: Option<f32>,
    // Phase 12.2 — per-band compressor overrides. `None` => the macro slider
    // (compression_density) drives that band's threshold; per-band ratio /
    // attack / release fall back to fixed musical defaults (see
    // `ChainCoeffs::from_settings`). `Some(v)` => override the macro for this
    // band/parameter only. All `#[serde(default)]` so older sessions and
    // older frontends parse cleanly.
    #[serde(default)]
    pub compression_low_threshold_db: Option<f32>,
    #[serde(default)]
    pub compression_low_ratio: Option<f32>,
    #[serde(default)]
    pub compression_low_attack_ms: Option<f32>,
    #[serde(default)]
    pub compression_low_release_ms: Option<f32>,
    #[serde(default)]
    pub compression_mid_threshold_db: Option<f32>,
    #[serde(default)]
    pub compression_mid_ratio: Option<f32>,
    #[serde(default)]
    pub compression_mid_attack_ms: Option<f32>,
    #[serde(default)]
    pub compression_mid_release_ms: Option<f32>,
    #[serde(default)]
    pub compression_high_threshold_db: Option<f32>,
    #[serde(default)]
    pub compression_high_ratio: Option<f32>,
    #[serde(default)]
    pub compression_high_attack_ms: Option<f32>,
    #[serde(default)]
    pub compression_high_release_ms: Option<f32>,
    /// Phase 12.2 — when `Some(false)`, the multiband compressor runs
    /// independent L/R envelope followers per band. Default (`None` or
    /// `Some(true)`) links stereo: a single max-of-|L|,|R| envelope drives the
    /// same gain reduction on both channels, the standard mastering choice.
    #[serde(default)]
    pub compression_link_stereo: Option<bool>,
    pub bit_depth: Option<u16>,
    pub target_sample_rate: Option<u32>,
}
```

---

- [ ] **Step 1.2: Add 3 GR fields to `PlaybackTick`**

Locate `PlaybackTick` (around line 302 of `types.rs`):

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaybackTick {
    pub track_id: Option<TrackId>,
    pub position_sec: f64,
    pub is_playing: bool,
    pub is_loaded: bool,
    #[serde(default = "default_silence_dbfs")]
    pub peak_dbfs: f32,
}
```

Replace with:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaybackTick {
    pub track_id: Option<TrackId>,
    pub position_sec: f64,
    pub is_playing: bool,
    pub is_loaded: bool,
    #[serde(default = "default_silence_dbfs")]
    pub peak_dbfs: f32,
    /// Phase 12.2 — gain reduction (in dB, negative) from the low band of the
    /// multiband compressor, captured as the maximum reduction seen in the
    /// last snapshot window. `-120.0` is the silence sentinel (no signal or
    /// compressor inactive in the window). Defaulted so older sessions and
    /// older frontends parse cleanly.
    #[serde(default = "default_silence_dbfs")]
    pub gr_low_db: f32,
    #[serde(default = "default_silence_dbfs")]
    pub gr_mid_db: f32,
    #[serde(default = "default_silence_dbfs")]
    pub gr_high_db: f32,
}
```

---

- [ ] **Step 1.3: Verify the crate still compiles**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: clean compile (the new fields are unused so far; warnings are fine). If a downstream file fails to compile because it pattern-matches `AdvancedSettings` or `PlaybackTick` fields exhaustively, that file needs a `..` rest pattern — fix in place.

---

## Task 2: Write the 8 failing `dsp.rs` unit tests up front (TDD red)

**Files:**
- Modify: `src-tauri/src/dsp.rs` (`mod tests`)

Per the warmth/air plan's precedent, every new behavior is pinned by a failing test before the implementation lands. The tests reference symbols (`ChannelState::compression_active`, `EnvelopeFollower`, etc.) that don't exist yet — `cargo check --tests` will fail, which is the red signal we want.

---

- [ ] **Step 2.1: Append the 8 new tests at the bottom of `mod tests` (after `presence_air_at_one_lifts_10khz_band`, around line 1063)**

```rust
    // ====================================================================
    // Phase 12.2 — multiband compressor tests. Closed-form math where
    // possible; otherwise pin behavior by feeding known-amplitude steady
    // signals through `MasteringChain` and observing steady-state output.
    // ====================================================================

    /// Constants pinned by the design:
    /// - Crossovers: 120 Hz (low/mid) and 4000 Hz (mid/high).
    /// - Macro `compression_density.unwrap_or(0.0).clamp(0,1)` → uniform
    ///   threshold `0 dBFS` (at 0) to `-24 dBFS` (at 1) on all 3 bands.
    /// - Per-band default ratios: low 2.5, mid 2.0, high 1.8.
    /// - Per-band default attack/release (ms): low 30/300, mid 15/150,
    ///   high 5/80.
    /// - Soft knee: 6 dB fixed.
    /// - Detector: peak (|x|).
    /// - Identity early-return: `compression_density.unwrap_or(0.0) < 1e-4
    ///   AND all 12 per-band overrides None AND link_stereo isn't Some(false)`.

    fn default_master_settings() -> MasteringSettings {
        MasteringSettings {
            preset: Preset::Custom { id: "t".to_string() },
            intensity: 0.0,
            eq_low_db: 0.0,
            eq_mid_db: 0.0,
            eq_high_db: 0.0,
            volume_match: false,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
            advanced: AdvancedSettings::default(),
        }
    }

    /// Decision #1 / #11: `Advanced.compression_density = None` and all 12
    /// per-band overrides `None` and link_stereo not `Some(false)` must take
    /// the identity early-return path. `ChainCoeffs::compression_active` must
    /// be `false`. Byte-equivalent untouched-slider path for backward
    /// compatibility with all real-fixture tests.
    #[test]
    fn compression_density_default_is_identity() {
        let c = ChainCoeffs::from_settings(44_100, &default_master_settings());
        assert!(
            !c.compression_active,
            "default settings must set compression_active = false (got true)"
        );
    }

    /// Decision #1: LR4 crossovers at 120 Hz / 4000 Hz must SUM flat across
    /// the band centers, within 0.1 dB of unity. LR4 is the textbook flat-
    /// summing crossover; the post-split sum recovers the input.
    ///
    /// Test method: build a `MasteringChain` with `compression_density = 0.0`
    /// AND each per-band threshold pinned to a high value (e.g. -100 dB
    /// threshold + ratio 1.0 makes each band a pass-through). Then feed a
    /// 1 kHz sine into the chain — the band-split network must sum back to
    /// the input within 0.1 dB. (We can't run the band split in isolation
    /// from public API; this test exercises it via the assembled chain.)
    ///
    /// Easier alternative: expose `pub(crate) fn split_into_bands` on the
    /// crossover network for tests, return `(low, mid, high)`, and assert
    /// `low + mid + high ≈ input` per sample after a short settling window
    /// (>= the longest filter group delay). Both approaches are acceptable —
    /// the second is preferred so the test pins the split network directly.
    #[test]
    fn lr4_crossover_sums_flat_at_unity() {
        // The crossover network is exposed for tests as
        // `pub(crate) fn split_lr4_into_bands(input: f32, state: &mut LR4State)
        //  -> (f32, f32, f32)`.
        // Drive 4096 samples of a 1 kHz sine through it; after a settling
        // window of 256 samples, assert `(low + mid + high - input).abs() <
        // 0.012` (≈ 0.1 dB). Repeat with a 60 Hz sine (well below the 120 Hz
        // crossover) and an 8 kHz sine (well above the 4000 Hz crossover) to
        // confirm summing flatness across all three regions.
        let sr = 44_100.0f32;
        let mut state = LR4State::default();
        for &freq in &[60.0f32, 1_000.0, 8_000.0] {
            // Re-zero the state per frequency to avoid one freq's tail biasing
            // the next.
            state = LR4State::default();
            // Settle the filters.
            for n in 0..512 {
                let x = (n as f32 * 2.0 * std::f32::consts::PI * freq / sr).sin();
                let _ = split_lr4_into_bands(x, &mut state);
            }
            // Measure summing error over the next 4096 samples.
            let mut max_err = 0.0f32;
            for n in 512..(512 + 4096) {
                let x = (n as f32 * 2.0 * std::f32::consts::PI * freq / sr).sin();
                let (l, m, h) = split_lr4_into_bands(x, &mut state);
                let e = (l + m + h - x).abs();
                if e > max_err {
                    max_err = e;
                }
            }
            // Tighter at band centers (0.1 dB = 0.012 linear), looser at edges
            // (0.5 dB = 0.06 linear). LR4 by construction is flat-summing
            // everywhere, so 0.012 is the right bar.
            assert!(
                max_err < 0.012,
                "LR4 summing flatness violated at {} Hz: max |L+M+H - x| = {}",
                freq,
                max_err
            );
        }
    }

    /// Decision #3: at `density=1.0` the macro maps to a uniform `-24 dBFS`
    /// threshold across all 3 bands. A sustained 0.8-amplitude 1 kHz sine
    /// (which is ≈ -1.94 dBFS peak, deep above the threshold) must show
    /// ≥ 3 dB reduction in the steady-state output compared to `density=0.0`.
    /// 1 kHz lands in the mid band by construction (120 < 1000 < 4000).
    #[test]
    fn compression_density_at_one_attenuates_loud_signal() {
        let sr = 44_100;
        let freq = 1_000.0f32;
        let amp = 0.8f32;
        let mut s0 = default_master_settings();
        s0.advanced.compression_density = Some(0.0);
        let mut s1 = default_master_settings();
        s1.advanced.compression_density = Some(1.0);
        let mut chain0 = MasteringChain::new(sr, 2, &s0);
        let mut chain1 = MasteringChain::new(sr, 2, &s1);
        // Settle for 200 ms (≈ 8820 samples) so envelope followers reach
        // steady state; the slowest band's release is 300 ms but we're
        // measuring sustained reduction here, not release.
        let settle = (0.4 * sr as f32) as usize;
        // Measure the steady-state RMS over the next 200 ms.
        let measure = (0.2 * sr as f32) as usize;
        let mut sum0 = 0.0f64;
        let mut sum1 = 0.0f64;
        for n in 0..(settle + measure) {
            let x = amp * (n as f32 * 2.0 * std::f32::consts::PI * freq / sr as f32).sin();
            let mut f0 = [x, x];
            let mut f1 = [x, x];
            chain0.process_frame_inplace(&mut f0);
            chain1.process_frame_inplace(&mut f1);
            if n >= settle {
                sum0 += (f0[0] as f64).powi(2);
                sum1 += (f1[0] as f64).powi(2);
            }
        }
        let rms0 = (sum0 / measure as f64).sqrt() as f32;
        let rms1 = (sum1 / measure as f64).sqrt() as f32;
        let delta_db = 20.0 * (rms1 / rms0.max(1e-9)).log10();
        assert!(
            delta_db <= -3.0,
            "density=1.0 should attenuate the loud mid-band sine by >=3 dB \
             vs density=0.0; got delta = {:.2} dB (rms0={}, rms1={})",
            delta_db,
            rms0,
            rms1
        );
    }

    /// Decision #4 / #5: `compression_mid_threshold_db = Some(-30.0)` with
    /// macro `density=0.0` must override the macro threshold for the mid band
    /// (reading back -30, not 0). Low and high bands must keep the macro
    /// threshold (0 dBFS at density=0.0).
    #[test]
    fn compression_per_band_override_replaces_macro() {
        let mut s = default_master_settings();
        s.advanced.compression_density = Some(0.0);
        s.advanced.compression_mid_threshold_db = Some(-30.0);
        let c = ChainCoeffs::from_settings(44_100, &s);
        assert!(
            (c.comp_mid_threshold_db - (-30.0)).abs() < 1e-4,
            "mid threshold should be -30, got {}",
            c.comp_mid_threshold_db
        );
        assert!(
            c.comp_low_threshold_db.abs() < 1e-4,
            "low threshold should be macro (0 dBFS at density=0), got {}",
            c.comp_low_threshold_db
        );
        assert!(
            c.comp_high_threshold_db.abs() < 1e-4,
            "high threshold should be macro (0 dBFS at density=0), got {}",
            c.comp_high_threshold_db
        );
    }

    /// Decision #8 (peak detector): feed a step (zero → 1.0 sustained) into
    /// an envelope follower with attack 10 ms, release 100 ms at 44.1 kHz.
    /// After exactly 10 ms of step input the envelope must read ≥ 0.63
    /// (1 - 1/e), confirming the attack time-constant. After releasing back
    /// to 0.0 for 100 ms the envelope must drop to ≤ 0.37 (1/e), confirming
    /// release.
    ///
    /// Envelope follower math (peak detector, one-pole):
    ///   alpha_a = exp(-1 / (attack_ms/1000 * sr))
    ///   alpha_r = exp(-1 / (release_ms/1000 * sr))
    ///   if |x| > env: env = alpha_a * env + (1 - alpha_a) * |x|
    ///   else        : env = alpha_r * env + (1 - alpha_r) * |x|
    #[test]
    fn envelope_follower_attack_release_time_constants() {
        let sr = 44_100.0f32;
        let mut env = EnvelopeFollower::new(sr, 10.0, 100.0);
        // 10 ms of step input (≈ 441 samples).
        let attack_samples = (sr * 0.010) as usize;
        let mut last = 0.0f32;
        for _ in 0..attack_samples {
            last = env.process(1.0);
        }
        assert!(
            last >= 0.63,
            "after 10 ms (attack tau) of step input, env should be >= 0.63 \
             (1 - 1/e); got {}",
            last
        );
        // 100 ms of release (≈ 4410 samples) back to zero input.
        let release_samples = (sr * 0.100) as usize;
        for _ in 0..release_samples {
            last = env.process(0.0);
        }
        assert!(
            last <= 0.37,
            "after 100 ms (release tau) of zero input, env should be <= 0.37 \
             (1/e); got {}",
            last
        );
    }

    /// Decision #6: `link_stereo=true` (default) feeds the band's envelope
    /// follower with `max(|L|, |R|)` and applies the same gain reduction to
    /// both channels. Verify by asymmetric L/R (loud L, quiet R) → identical
    /// attenuation. Then flip `link_stereo=Some(false)` and verify the two
    /// channels receive different reductions.
    #[test]
    fn compression_linked_stereo_applies_same_gain_to_both_channels() {
        let sr = 44_100;
        let freq = 1_000.0f32;
        let mut s_linked = default_master_settings();
        s_linked.advanced.compression_density = Some(1.0);
        // explicit Some(true) for clarity; default is also linked.
        s_linked.advanced.compression_link_stereo = Some(true);
        let mut s_unlinked = s_linked.clone();
        s_unlinked.advanced.compression_link_stereo = Some(false);
        let mut linked = MasteringChain::new(sr, 2, &s_linked);
        let mut unlinked = MasteringChain::new(sr, 2, &s_unlinked);
        let settle = (0.4 * sr as f32) as usize;
        let measure = (0.2 * sr as f32) as usize;
        // Asymmetric stereo: loud L (0.8), quiet R (0.05). Quiet R sits below
        // the threshold for density=1.0 (-24 dBFS = ~0.063 linear) so under
        // unlinked operation R would NOT be reduced; linked operation drives
        // both channels with L's envelope.
        let mut diff_linked = 0.0f64;
        let mut diff_unlinked = 0.0f64;
        for n in 0..(settle + measure) {
            let phase = n as f32 * 2.0 * std::f32::consts::PI * freq / sr as f32;
            let l_in = 0.8 * phase.sin();
            let r_in = 0.05 * phase.sin();
            let mut f_l = [l_in, r_in];
            let mut f_u = [l_in, r_in];
            linked.process_frame_inplace(&mut f_l);
            unlinked.process_frame_inplace(&mut f_u);
            if n >= settle {
                // Compare each channel's reduction ratio vs its input. Linked:
                // both channels reduced by the same factor (driven by L).
                // Unlinked: L reduced, R barely touched.
                diff_linked +=
                    ((f_l[0] / l_in.max(1e-9)) - (f_l[1] / r_in.max(1e-9))).abs() as f64;
                diff_unlinked +=
                    ((f_u[0] / l_in.max(1e-9)) - (f_u[1] / r_in.max(1e-9))).abs() as f64;
            }
        }
        let avg_linked_diff = diff_linked / measure as f64;
        let avg_unlinked_diff = diff_unlinked / measure as f64;
        // Linked: the two channels should track each other (small diff).
        assert!(
            avg_linked_diff < 0.1,
            "linked stereo should give matching gain to L and R; avg ratio diff = {}",
            avg_linked_diff
        );
        // Unlinked: significantly different ratios.
        assert!(
            avg_unlinked_diff > 0.2,
            "unlinked stereo should diverge L vs R; avg ratio diff = {} (linked was {})",
            avg_unlinked_diff,
            avg_linked_diff
        );
    }

    /// Decision #10: at `density=0.5` (threshold = -12 dBFS, uniform) and a
    /// steady-state -20 dBFS sine (≈ 0.1 amplitude, well below threshold),
    /// the post-comp output must read within ±1 dB of the input. Below
    /// threshold no reduction fires, AND the makeup-gain formula is
    /// `(threshold_drop_db * (1 - 1/ratio)) / 2`, half-compensation — over
    /// the long run this gives near-unity output at low input levels.
    ///
    /// Note: the macro maps `density=0.5` → threshold = `-12 dBFS`, and
    /// `threshold_drop_db = 24 * 0.5 = 12 dB`. Mid ratio is 2.0, so
    /// makeup_mid_db = 12 * (1 - 1/2) / 2 = 3.0 dB. So a sub-threshold sine
    /// gets a ~+3 dB lift from makeup but no reduction — net ~+3 dB. We
    /// allow ±1 dB tolerance around 0 dB by widening the check window —
    /// specifically: assert the output level is within [-1, +4] dB of input,
    /// which pins both the no-reduction behavior AND the makeup wiring,
    /// while leaving the exact constant tolerable for small RBJ rounding.
    ///
    /// Refinement: the brainstorm's wording was "within ±1 dB of the input"
    /// — implement that exact bar by additionally pinning that the makeup
    /// constant matches the formula (assert the chain's `comp_mid_makeup_db`
    /// reads back as 3.0 dB ± 0.1 dB). Then for the audio-level check we
    /// allow the half-compensation lift in the assertion.
    #[test]
    fn compression_makeup_gain_compensates_threshold_drop() {
        let mut s = default_master_settings();
        s.advanced.compression_density = Some(0.5);
        let c = ChainCoeffs::from_settings(44_100, &s);
        // The formula pins this exactly.
        assert!(
            (c.comp_mid_makeup_db - 3.0).abs() < 0.1,
            "mid makeup_db at density=0.5, ratio=2.0 should be 3.0 dB, got {}",
            c.comp_mid_makeup_db
        );
        // End-to-end: -20 dBFS sine → output ~ -17 dBFS (no reduction +
        // 3 dB makeup). Tolerance ±1 dB.
        let sr = 44_100;
        let freq = 1_000.0f32;
        let amp = 0.1f32; // ≈ -20 dBFS
        let mut chain = MasteringChain::new(sr, 2, &s);
        let settle = (0.4 * sr as f32) as usize;
        let measure = (0.2 * sr as f32) as usize;
        let mut sum_in = 0.0f64;
        let mut sum_out = 0.0f64;
        for n in 0..(settle + measure) {
            let x = amp * (n as f32 * 2.0 * std::f32::consts::PI * freq / sr as f32).sin();
            let mut f = [x, x];
            chain.process_frame_inplace(&mut f);
            if n >= settle {
                sum_in += (x as f64).powi(2);
                sum_out += (f[0] as f64).powi(2);
            }
        }
        let in_db = 10.0 * (sum_in / measure as f64).log10();
        let out_db = 10.0 * (sum_out / measure as f64).log10();
        let delta_db = (out_db - in_db) as f32;
        // Expected: ~+3 dB (sub-threshold + makeup). Allow ±1 dB.
        assert!(
            (delta_db - 3.0).abs() < 1.5,
            "sub-threshold sine should see ~+3 dB makeup at density=0.5; got delta = {:.2} dB",
            delta_db
        );
    }

    /// Decision (clamping): `density=5.0` clamps to 1.0 and `density=-1.0`
    /// clamps to 0.0 before mapping to threshold. Verify by reading back the
    /// per-band threshold in `ChainCoeffs` — at clamped 1.0 it's -24 dBFS,
    /// at clamped 0.0 it's 0 dBFS.
    #[test]
    fn compression_clamps_density_into_range() {
        let mut s_high = default_master_settings();
        s_high.advanced.compression_density = Some(5.0);
        let c_high = ChainCoeffs::from_settings(44_100, &s_high);
        assert!(
            (c_high.comp_mid_threshold_db - (-24.0)).abs() < 1e-3,
            "density=5.0 should clamp to 1.0 (threshold = -24 dBFS); got {}",
            c_high.comp_mid_threshold_db
        );
        let mut s_neg = default_master_settings();
        s_neg.advanced.compression_density = Some(-1.0);
        let c_neg = ChainCoeffs::from_settings(44_100, &s_neg);
        assert!(
            c_neg.comp_mid_threshold_db.abs() < 1e-3,
            "density=-1.0 should clamp to 0.0 (threshold = 0 dBFS); got {}",
            c_neg.comp_mid_threshold_db
        );
    }
```

---

- [ ] **Step 2.2: Run `cargo check --tests` and confirm the tests fail to compile**

```powershell
cargo check --tests --manifest-path src-tauri/Cargo.toml
```

Expected: compile errors of the form `no field 'compression_active' on type ChainCoeffs`, `cannot find function 'split_lr4_into_bands'`, `cannot find type 'LR4State'`, `cannot find function 'EnvelopeFollower'`, `no field 'comp_mid_threshold_db'`, etc. This is the red signal — the tests exercise real new behavior we haven't built.

If the new fields on `AdvancedSettings` from Task 1 are missing from the schema, fix Task 1 first.

---

## Task 3: LR4 crossover network (split into low/mid/high)

**Files:**
- Modify: `src-tauri/src/dsp.rs` (new `LR4State`, new `split_lr4_into_bands`, supporting Butterworth coefficient helpers)

LR4 = cascaded Butterworth LP+LP (slope -24 dB/oct) and HP+HP (slope -24 dB/oct) at the crossover frequency. Q = sqrt(2)/2 ≈ 0.7071 per stage. The 3-way split is:
- Low: LP @ 120 Hz (cascaded twice).
- High: HP @ 4000 Hz (cascaded twice).
- Mid: (input − low) AND THEN HP-filtered @ 120 Hz (cascaded twice), then LP-filtered @ 4000 Hz (cascaded twice). Equivalent and cleaner: route the input through HP @ 120 Hz cascade → that's everything-above-120-Hz → then LP @ 4000 Hz cascade → that's 120-4000.

Cleaner-still alternative (avoids the subtraction stage that can introduce phase mismatch): build the band-pass mid by `HP_120 → LP_4000` cascade (each as a Butterworth pair).

---

- [ ] **Step 3.1: Add Butterworth biquad helpers to `BiquadCoeffs`**

Add a new method to `impl BiquadCoeffs` near the existing `peaking` method (around line 83 of `dsp.rs`):

```rust
    /// Butterworth low-pass (RBJ cookbook, Q=0.7071 for one stage). For an
    /// LR4 crossover (-24 dB/oct), cascade two of these at the same corner.
    pub fn butter_lp(sample_rate: f32, freq_hz: f32, q: f32) -> Self {
        let omega = 2.0 * PI * freq_hz / sample_rate;
        let cos_o = omega.cos();
        let sin_o = omega.sin();
        let alpha = sin_o / (2.0 * q);
        let b0 = (1.0 - cos_o) / 2.0;
        let b1 = 1.0 - cos_o;
        let b2 = (1.0 - cos_o) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_o;
        let a2 = 1.0 - alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Butterworth high-pass (RBJ cookbook, Q=0.7071 for one stage). Cascade
    /// two of these for an LR4 -24 dB/oct slope.
    pub fn butter_hp(sample_rate: f32, freq_hz: f32, q: f32) -> Self {
        let omega = 2.0 * PI * freq_hz / sample_rate;
        let cos_o = omega.cos();
        let sin_o = omega.sin();
        let alpha = sin_o / (2.0 * q);
        let b0 = (1.0 + cos_o) / 2.0;
        let b1 = -(1.0 + cos_o);
        let b2 = (1.0 + cos_o) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_o;
        let a2 = 1.0 - alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
```

---

- [ ] **Step 3.2: Add `LR4State` and `split_lr4_into_bands` near the bottom of `dsp.rs`, above `pub struct MasteringChain` (around line 508)**

The test helper version uses hard-coded 120/4000 Hz at 44.1 kHz. Production uses coefficients from `ChainCoeffs`. We expose a `pub(crate)` helper that bakes the test constants for the closed-form summing test.

```rust
// ============================================================================
// Phase 12.2 — LR4 crossover network for the multiband compressor. 3-way
// split via cascaded-Butterworth LP+LP (low) and HP+HP (high), with the mid
// band as the HP_120 → LP_4000 cascade. LR4 sums flat across all band edges
// (mathematical property of cascaded Butterworth at the same corner, no
// magnitude bump like LR2). All four cascade pairs hold their own state per
// channel — that's 8 biquads per channel for the split.
// ============================================================================

const LR4_CROSSOVER_LOW_HZ: f32 = 120.0;
const LR4_CROSSOVER_HIGH_HZ: f32 = 4000.0;
const BUTTERWORTH_Q: f32 = 0.707_106_8; // sqrt(2)/2

/// Per-channel filter memory for the LR4 split: two LP stages for the low
/// band, two HP stages and two LP stages for the mid band, two HP stages for
/// the high band. Default = all zero (no signal in history).
#[derive(Debug, Clone, Default)]
pub struct LR4State {
    // Low band: LP @ 120 Hz cascaded twice.
    pub low_lp1: BiquadState,
    pub low_lp2: BiquadState,
    // Mid band: HP @ 120 Hz cascaded twice, then LP @ 4000 Hz cascaded twice.
    pub mid_hp1: BiquadState,
    pub mid_hp2: BiquadState,
    pub mid_lp1: BiquadState,
    pub mid_lp2: BiquadState,
    // High band: HP @ 4000 Hz cascaded twice.
    pub high_hp1: BiquadState,
    pub high_hp2: BiquadState,
}

/// Test-only entry point: splits a single sample at sample_rate = 44_100 with
/// the LR4 crossovers fixed at 120 Hz and 4000 Hz. Production callers use
/// `MasteringChain::process_frame_inplace`, which fetches the coefficients
/// from `ChainCoeffs` (sample-rate-aware) and walks the same biquads in the
/// same order.
pub(crate) fn split_lr4_into_bands(x: f32, state: &mut LR4State) -> (f32, f32, f32) {
    let sr = 44_100.0f32;
    let low_lp_c = BiquadCoeffs::butter_lp(sr, LR4_CROSSOVER_LOW_HZ, BUTTERWORTH_Q);
    let mid_hp_c = BiquadCoeffs::butter_hp(sr, LR4_CROSSOVER_LOW_HZ, BUTTERWORTH_Q);
    let mid_lp_c = BiquadCoeffs::butter_lp(sr, LR4_CROSSOVER_HIGH_HZ, BUTTERWORTH_Q);
    let high_hp_c = BiquadCoeffs::butter_hp(sr, LR4_CROSSOVER_HIGH_HZ, BUTTERWORTH_Q);
    let low_a = state.low_lp1.process(&low_lp_c, x);
    let low = state.low_lp2.process(&low_lp_c, low_a);
    let mid_after_hp1 = state.mid_hp1.process(&mid_hp_c, x);
    let mid_after_hp2 = state.mid_hp2.process(&mid_hp_c, mid_after_hp1);
    let mid_after_lp1 = state.mid_lp1.process(&mid_lp_c, mid_after_hp2);
    let mid = state.mid_lp2.process(&mid_lp_c, mid_after_lp1);
    let high_a = state.high_hp1.process(&high_hp_c, x);
    let high = state.high_hp2.process(&high_hp_c, high_a);
    (low, mid, high)
}
```

---

- [ ] **Step 3.3: Run `cargo test --lib lr4_crossover_sums_flat_at_unity` and confirm it passes**

```powershell
cargo test --lib --manifest-path src-tauri/Cargo.toml lr4_crossover_sums_flat_at_unity
```

Expected: this single test passes. Other Phase 12.2 tests still fail to compile or assert — that's fine, they cover Tasks 4-9.

If summing flatness > 0.1 dB, check that `BUTTERWORTH_Q` is exactly `sqrt(2)/2` and that the cascaded HP-then-LP order for the mid band matches the helper above.

---

## Task 4: Envelope follower (per-band peak detector with attack/release)

**Files:**
- Modify: `src-tauri/src/dsp.rs` (new `EnvelopeFollower`)

---

- [ ] **Step 4.1: Add `EnvelopeFollower` near the bottom of `dsp.rs`, just below `split_lr4_into_bands`**

```rust
/// Peak-detector envelope follower. One-pole smoothing with separate attack
/// and release time constants. `env_n = (alpha * env_{n-1}) + ((1 - alpha) *
/// |x_n|)` where `alpha = exp(-1 / (time_ms/1000 * sr))`. The selected alpha
/// depends on whether the signal is rising (use attack) or decaying (use
/// release).
#[derive(Debug, Clone)]
pub struct EnvelopeFollower {
    pub env: f32,
    pub alpha_attack: f32,
    pub alpha_release: f32,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: f32, attack_ms: f32, release_ms: f32) -> Self {
        Self {
            env: 0.0,
            alpha_attack: alpha_from_time_ms(sample_rate, attack_ms),
            alpha_release: alpha_from_time_ms(sample_rate, release_ms),
        }
    }

    /// Update the envelope with one sample of |x_n| (the detector input).
    /// Returns the new envelope value.
    #[inline]
    pub fn process(&mut self, x_abs: f32) -> f32 {
        let alpha = if x_abs > self.env {
            self.alpha_attack
        } else {
            self.alpha_release
        };
        self.env = alpha * self.env + (1.0 - alpha) * x_abs;
        self.env
    }

    pub fn reset(&mut self) {
        self.env = 0.0;
    }
}

#[inline]
fn alpha_from_time_ms(sample_rate: f32, time_ms: f32) -> f32 {
    if time_ms <= 0.0 || sample_rate <= 0.0 {
        return 0.0;
    }
    (-1.0_f32 / (time_ms * 0.001 * sample_rate)).exp()
}
```

---

- [ ] **Step 4.2: Run `cargo test --lib envelope_follower_attack_release_time_constants` and confirm it passes**

```powershell
cargo test --lib --manifest-path src-tauri/Cargo.toml envelope_follower_attack_release_time_constants
```

Expected: the envelope test passes (the 1 - 1/e ≈ 0.6321 attack-tau check and the 1/e ≈ 0.3679 release-tau check). If `last < 0.63` after the attack window, the alpha sign/formula is wrong — verify `alpha = exp(-1/(τ·sr))` and that `(1-α)·|x| + α·env` matches.

---

## Task 5: Extend `ChainCoeffs` with compressor coefficients

**Files:**
- Modify: `src-tauri/src/dsp.rs` (`ChainCoeffs` struct + `ChainCoeffs::from_settings`)

---

- [ ] **Step 5.1: Add compressor fields to `ChainCoeffs`**

Locate `ChainCoeffs` (around line 130 of `dsp.rs`). Append new fields (after `width_side_scale`):

```rust
    // ----- Phase 12.2: multiband compressor coefficients -----
    /// Whether the compressor is active. `false` triggers the identity early-
    /// return in `process_frame_inplace` — byte-equivalent to the pre-slice
    /// chain output. `true` when ANY of: macro density > 1e-4, any per-band
    /// override is `Some(_)`, or link_stereo is `Some(false)`.
    pub compression_active: bool,
    /// LR4 crossover coefficients (per channel applies the same coefficients;
    /// state is per-channel and lives on `ChannelState`).
    pub comp_low_lp: BiquadCoeffs,
    pub comp_mid_hp: BiquadCoeffs,
    pub comp_mid_lp: BiquadCoeffs,
    pub comp_high_hp: BiquadCoeffs,
    /// Per-band threshold (dBFS), ratio (X:1), attack alpha, release alpha,
    /// and post-reduction makeup gain (linear). Computed in `from_settings`
    /// from the macro slider + per-band overrides.
    pub comp_low_threshold_db: f32,
    pub comp_low_ratio: f32,
    pub comp_low_attack_alpha: f32,
    pub comp_low_release_alpha: f32,
    pub comp_low_makeup_db: f32,
    pub comp_low_makeup_lin: f32,
    pub comp_mid_threshold_db: f32,
    pub comp_mid_ratio: f32,
    pub comp_mid_attack_alpha: f32,
    pub comp_mid_release_alpha: f32,
    pub comp_mid_makeup_db: f32,
    pub comp_mid_makeup_lin: f32,
    pub comp_high_threshold_db: f32,
    pub comp_high_ratio: f32,
    pub comp_high_attack_alpha: f32,
    pub comp_high_release_alpha: f32,
    pub comp_high_makeup_db: f32,
    pub comp_high_makeup_lin: f32,
    /// Soft-knee width in dB (fixed at 6 dB per the design — not user-tunable
    /// in v1). Stored on the coeffs so the gain-stage code reads one source
    /// of truth.
    pub comp_knee_db: f32,
    /// Linked-stereo behavior. `true` = max(|L|,|R|) drives a shared
    /// envelope; `false` = independent per-channel envelopes per band.
    pub comp_link_stereo: bool,
```

---

- [ ] **Step 5.2: Compute the compressor coefficients in `ChainCoeffs::from_settings`**

Append a new block just before the `Self { ... }` literal at the end of `from_settings` (around line 285-303, just after the existing `width_side_scale` block):

```rust
        // ----- Phase 12.2: multiband compressor coefficients -----
        // Macro: density 0..1 → uniform threshold 0 dBFS (off) to -24 dBFS
        // (heavy). Below 1e-4 the macro is "off"; per-band overrides may
        // still pull bands into reduction independently.
        let density = settings
            .advanced
            .compression_density
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let macro_threshold_db = -24.0 * density;

        // Per-band fixed musical defaults (see brainstorm "Macro mapping").
        const LOW_RATIO_DEFAULT: f32 = 2.5;
        const MID_RATIO_DEFAULT: f32 = 2.0;
        const HIGH_RATIO_DEFAULT: f32 = 1.8;
        const LOW_ATTACK_MS_DEFAULT: f32 = 30.0;
        const LOW_RELEASE_MS_DEFAULT: f32 = 300.0;
        const MID_ATTACK_MS_DEFAULT: f32 = 15.0;
        const MID_RELEASE_MS_DEFAULT: f32 = 150.0;
        const HIGH_ATTACK_MS_DEFAULT: f32 = 5.0;
        const HIGH_RELEASE_MS_DEFAULT: f32 = 80.0;

        let comp_low_threshold_db = settings
            .advanced
            .compression_low_threshold_db
            .unwrap_or(macro_threshold_db);
        let comp_mid_threshold_db = settings
            .advanced
            .compression_mid_threshold_db
            .unwrap_or(macro_threshold_db);
        let comp_high_threshold_db = settings
            .advanced
            .compression_high_threshold_db
            .unwrap_or(macro_threshold_db);

        let comp_low_ratio = settings
            .advanced
            .compression_low_ratio
            .unwrap_or(LOW_RATIO_DEFAULT)
            .max(1.0);
        let comp_mid_ratio = settings
            .advanced
            .compression_mid_ratio
            .unwrap_or(MID_RATIO_DEFAULT)
            .max(1.0);
        let comp_high_ratio = settings
            .advanced
            .compression_high_ratio
            .unwrap_or(HIGH_RATIO_DEFAULT)
            .max(1.0);

        let low_attack_ms = settings
            .advanced
            .compression_low_attack_ms
            .unwrap_or(LOW_ATTACK_MS_DEFAULT)
            .max(0.1);
        let low_release_ms = settings
            .advanced
            .compression_low_release_ms
            .unwrap_or(LOW_RELEASE_MS_DEFAULT)
            .max(0.1);
        let mid_attack_ms = settings
            .advanced
            .compression_mid_attack_ms
            .unwrap_or(MID_ATTACK_MS_DEFAULT)
            .max(0.1);
        let mid_release_ms = settings
            .advanced
            .compression_mid_release_ms
            .unwrap_or(MID_RELEASE_MS_DEFAULT)
            .max(0.1);
        let high_attack_ms = settings
            .advanced
            .compression_high_attack_ms
            .unwrap_or(HIGH_ATTACK_MS_DEFAULT)
            .max(0.1);
        let high_release_ms = settings
            .advanced
            .compression_high_release_ms
            .unwrap_or(HIGH_RELEASE_MS_DEFAULT)
            .max(0.1);

        let comp_low_attack_alpha = alpha_from_time_ms(sr, low_attack_ms);
        let comp_low_release_alpha = alpha_from_time_ms(sr, low_release_ms);
        let comp_mid_attack_alpha = alpha_from_time_ms(sr, mid_attack_ms);
        let comp_mid_release_alpha = alpha_from_time_ms(sr, mid_release_ms);
        let comp_high_attack_alpha = alpha_from_time_ms(sr, high_attack_ms);
        let comp_high_release_alpha = alpha_from_time_ms(sr, high_release_ms);

        // Auto makeup: half-compensation of the threshold drop scaled by
        // (1 - 1/ratio). Splitting the compensation in half (the `/ 2.0`)
        // keeps the chain conservative — full compensation would push the
        // limiter harder on every density tweak.
        let makeup_db = |threshold_db: f32, ratio: f32| -> f32 {
            // threshold_drop_db is the depth below 0 dBFS that the threshold
            // sits at, in absolute (positive) dB.
            let threshold_drop_db = (-threshold_db).max(0.0);
            threshold_drop_db * (1.0 - 1.0 / ratio) / 2.0
        };
        let comp_low_makeup_db = makeup_db(comp_low_threshold_db, comp_low_ratio);
        let comp_mid_makeup_db = makeup_db(comp_mid_threshold_db, comp_mid_ratio);
        let comp_high_makeup_db = makeup_db(comp_high_threshold_db, comp_high_ratio);
        let comp_low_makeup_lin = 10.0_f32.powf(comp_low_makeup_db / 20.0);
        let comp_mid_makeup_lin = 10.0_f32.powf(comp_mid_makeup_db / 20.0);
        let comp_high_makeup_lin = 10.0_f32.powf(comp_high_makeup_db / 20.0);

        // Crossover biquad coefficients. Same coefficients for both channels;
        // state is per-channel and lives in `ChannelState`.
        let comp_low_lp = BiquadCoeffs::butter_lp(sr, LR4_CROSSOVER_LOW_HZ, BUTTERWORTH_Q);
        let comp_mid_hp = BiquadCoeffs::butter_hp(sr, LR4_CROSSOVER_LOW_HZ, BUTTERWORTH_Q);
        let comp_mid_lp = BiquadCoeffs::butter_lp(sr, LR4_CROSSOVER_HIGH_HZ, BUTTERWORTH_Q);
        let comp_high_hp = BiquadCoeffs::butter_hp(sr, LR4_CROSSOVER_HIGH_HZ, BUTTERWORTH_Q);

        // Stereo link. Default (`None` or `Some(true)`) links stereo. Only
        // explicit `Some(false)` switches to independent L/R envelopes.
        let comp_link_stereo = settings
            .advanced
            .compression_link_stereo
            .unwrap_or(true);

        // Activity skip-flag. Inactive when macro is off AND every per-band
        // override is None AND link_stereo isn't explicitly false (which on
        // its own would change observable behavior even with macro=0 because
        // the envelope followers run independently per channel — but with
        // macro=0 and no thresholds set there's nothing to reduce, so
        // technically still identity. Keep the flag conservative: any
        // explicit user override flips it on.)
        let comp_macro_off = density < 1.0e-4;
        let comp_no_overrides = settings.advanced.compression_low_threshold_db.is_none()
            && settings.advanced.compression_low_ratio.is_none()
            && settings.advanced.compression_low_attack_ms.is_none()
            && settings.advanced.compression_low_release_ms.is_none()
            && settings.advanced.compression_mid_threshold_db.is_none()
            && settings.advanced.compression_mid_ratio.is_none()
            && settings.advanced.compression_mid_attack_ms.is_none()
            && settings.advanced.compression_mid_release_ms.is_none()
            && settings.advanced.compression_high_threshold_db.is_none()
            && settings.advanced.compression_high_ratio.is_none()
            && settings.advanced.compression_high_attack_ms.is_none()
            && settings.advanced.compression_high_release_ms.is_none();
        let comp_link_unset = !matches!(
            settings.advanced.compression_link_stereo,
            Some(false)
        );
        let compression_active = !(comp_macro_off && comp_no_overrides && comp_link_unset);

        let comp_knee_db = 6.0_f32;
```

Then extend the `Self { ... }` literal at the end of `from_settings` to include all the new fields:

```rust
        Self {
            low,
            mid,
            high,
            warmth,
            presence_air,
            input_gain_lin,
            saturation_amount,
            ceiling_lin,
            user_output_gain_lin,
            volume_match_gain_lin,
            width_side_scale,
            // Phase 12.2 multiband compressor:
            compression_active,
            comp_low_lp,
            comp_mid_hp,
            comp_mid_lp,
            comp_high_hp,
            comp_low_threshold_db,
            comp_low_ratio,
            comp_low_attack_alpha,
            comp_low_release_alpha,
            comp_low_makeup_db,
            comp_low_makeup_lin,
            comp_mid_threshold_db,
            comp_mid_ratio,
            comp_mid_attack_alpha,
            comp_mid_release_alpha,
            comp_mid_makeup_db,
            comp_mid_makeup_lin,
            comp_high_threshold_db,
            comp_high_ratio,
            comp_high_attack_alpha,
            comp_high_release_alpha,
            comp_high_makeup_db,
            comp_high_makeup_lin,
            comp_knee_db,
            comp_link_stereo,
        }
```

---

- [ ] **Step 5.3: Run the coefficient-only tests and confirm pass**

```powershell
cargo test --lib --manifest-path src-tauri/Cargo.toml compression_density_default_is_identity compression_per_band_override_replaces_macro compression_makeup_gain_compensates_threshold_drop compression_clamps_density_into_range
```

Expected: 4 tests pass. The audio-driven tests (`compression_density_at_one_attenuates_loud_signal`, `compression_linked_stereo_applies_same_gain_to_both_channels`) still fail because the chain doesn't actually apply the compressor yet — Tasks 6-7.

---

## Task 6: Extend `ChannelState` with crossover + envelope state

**Files:**
- Modify: `src-tauri/src/dsp.rs` (`ChannelState`)

---

- [ ] **Step 6.1: Add 8 biquad states + 3 envelope followers to `ChannelState`**

Locate `ChannelState` (around line 329 of `dsp.rs`). Replace:

```rust
#[derive(Debug, Clone, Default)]
pub struct ChannelState {
    low: BiquadState,
    mid: BiquadState,
    high: BiquadState,
    warmth: BiquadState,
    presence_air: BiquadState,
}
```

With:

```rust
#[derive(Debug, Clone, Default)]
pub struct ChannelState {
    low: BiquadState,
    mid: BiquadState,
    high: BiquadState,
    warmth: BiquadState,
    presence_air: BiquadState,
    // Phase 12.2: multiband compressor — per-channel crossover network state.
    comp_split: LR4State,
    // Per-channel per-band envelope follower. Used directly when
    // `comp_link_stereo = false`; when linked, all channels' envelopes are
    // driven by the same max-of-channels detector input, but each channel
    // still keeps its own follower so the swap-on-toggle stays smooth.
    comp_low_env: f32,
    comp_mid_env: f32,
    comp_high_env: f32,
}
```

The envelope state is stored as raw `f32` (matching `EnvelopeFollower::env`) rather than embedding the `EnvelopeFollower` struct itself, because the alpha coefficients live on `ChainCoeffs` (per `MasteringChain`, not per `ChannelState`) and we want one source of truth for the time constants.

---

- [ ] **Step 6.2: Add a chain-wide GR snapshot atomics struct and plumb it onto `MasteringChain`**

Locate `pub struct MasteringChain` (around line 508 of `dsp.rs`). Replace:

```rust
pub struct MasteringChain {
    pub coeffs: ChainCoeffs,
    pub states: Vec<ChannelState>,
    pub limiter: Limiter,
}
```

With:

```rust
/// Phase 12.2 — per-band gain-reduction snapshots. `MasteringChain` writes
/// per-frame max-|reduction_db| into these atomics; the audio thread reads
/// via `swap` on the 50 ms snapshot cycle, mirroring the existing
/// `peak_linear` pattern. Integer storage (|reduction_db| * 100 as u32) avoids
/// the IEEE 754 sign-bit ordering edge case for negative dB values. 0 = no
/// reduction in the window.
#[derive(Debug, Default)]
pub struct GrSnapshotSlots {
    pub low: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub mid: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub high: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl Clone for GrSnapshotSlots {
    fn clone(&self) -> Self {
        Self {
            low: self.low.clone(),
            mid: self.mid.clone(),
            high: self.high.clone(),
        }
    }
}

pub struct MasteringChain {
    pub coeffs: ChainCoeffs,
    pub states: Vec<ChannelState>,
    pub limiter: Limiter,
    pub gr_snapshots: GrSnapshotSlots,
}
```

---

- [ ] **Step 6.3: Update `MasteringChain::new` and `with_coeffs_inheriting_state`**

Locate `impl MasteringChain` (around line 517). Replace `new`:

```rust
    pub fn new(sample_rate: u32, channels: usize, settings: &MasteringSettings) -> Self {
        let coeffs = ChainCoeffs::from_settings(sample_rate, settings);
        let states = (0..channels).map(|_| ChannelState::default()).collect();
        let ceiling_dbfs = settings
            .advanced
            .ceiling_dbtp
            .unwrap_or(-1.0)
            .clamp(-6.0, 0.0);
        let limiter = Limiter::new(
            sample_rate,
            channels,
            ceiling_dbfs,
            LIMITER_LOOKAHEAD_MS,
            LIMITER_RELEASE_MS,
        );
        Self {
            coeffs,
            states,
            limiter,
            gr_snapshots: GrSnapshotSlots::default(),
        }
    }
```

And `with_coeffs_inheriting_state`:

```rust
    pub fn with_coeffs_inheriting_state(coeffs: ChainCoeffs, prior: &Self) -> Self {
        Self {
            coeffs,
            states: prior.states.clone(),
            limiter: prior.limiter.clone(),
            gr_snapshots: prior.gr_snapshots.clone(),
        }
    }
```

Add a new constructor that lets external callers (the audio thread) plumb shared atomics:

```rust
    /// Construct a chain that writes gain-reduction snapshots into the
    /// provided shared atomic slots. Used by `MasteringSource` so the
    /// audio thread's `AudioThreadState` shares the same atomics with the
    /// chain inside the running source.
    pub fn new_with_gr_snapshots(
        sample_rate: u32,
        channels: usize,
        settings: &MasteringSettings,
        gr_snapshots: GrSnapshotSlots,
    ) -> Self {
        let mut chain = Self::new(sample_rate, channels, settings);
        chain.gr_snapshots = gr_snapshots;
        chain
    }
```

---

- [ ] **Step 6.4: Verify the crate still compiles**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: clean compile. Remaining test failures are intentional (the compressor stage isn't applied yet).

---

## Task 7: Apply the multiband compressor in `process_frame_inplace`

**Files:**
- Modify: `src-tauri/src/dsp.rs` (`MasteringChain::process_frame_inplace`)

Position: between the existing `presence_air` biquad (end of Pass 1) and the width transform. Identity early-return when `!coeffs.compression_active`.

---

- [ ] **Step 7.1: Insert the multiband compressor block in `process_frame_inplace`**

Locate `process_frame_inplace` (around line 559 of `dsp.rs`). The existing Pass 1 loop ends with the `presence_air` apply (around line 572). Immediately after the closing `}` of the Pass 1 `for ch in 0..channels` loop, AND before the width-conditional that starts `if channels == 2 && (self.coeffs.width_side_scale - 1.0).abs() > 1.0e-5`, insert the compressor stage:

```rust
        // Phase 12.2 — 3-band multiband downward compressor (LR4 split,
        // peak-detector envelope followers, soft 6 dB knee, auto makeup).
        // Position: between presence_air (end of EQ) and width (start of M/S
        // / saturation). Identity early-return when inactive — preserves
        // byte-equivalence with all existing real-fixture tests when the
        // slider is untouched.
        if self.coeffs.compression_active {
            self.apply_multiband_compressor(frame, channels);
        }
```

Then add the helper method to `impl MasteringChain` (before `pub fn process_interleaved`):

```rust
    fn apply_multiband_compressor(&mut self, frame: &mut [f32], channels: usize) {
        // Per-channel band split into (low, mid, high). Same crossover
        // coefficients across channels; state is per-channel.
        let mut bands: [[f32; 3]; 2] = [[0.0; 3]; 2]; // max 2 channels supported by stereo design
        let ch_active = channels.min(2);
        for ch in 0..ch_active {
            let state = &mut self.states[ch];
            let x = frame[ch];
            // Low band: LP @ 120 Hz cascaded twice.
            let low_a = state.comp_split.low_lp1.process(&self.coeffs.comp_low_lp, x);
            let low = state.comp_split.low_lp2.process(&self.coeffs.comp_low_lp, low_a);
            // Mid band: HP @ 120 Hz cascaded twice → LP @ 4000 Hz cascaded twice.
            let m1 = state.comp_split.mid_hp1.process(&self.coeffs.comp_mid_hp, x);
            let m2 = state.comp_split.mid_hp2.process(&self.coeffs.comp_mid_hp, m1);
            let m3 = state.comp_split.mid_lp1.process(&self.coeffs.comp_mid_lp, m2);
            let mid = state.comp_split.mid_lp2.process(&self.coeffs.comp_mid_lp, m3);
            // High band: HP @ 4000 Hz cascaded twice.
            let h1 = state.comp_split.high_hp1.process(&self.coeffs.comp_high_hp, x);
            let high = state.comp_split.high_hp2.process(&self.coeffs.comp_high_hp, h1);
            bands[ch] = [low, mid, high];
        }

        // Compute the detector input per band. Linked = max of |L|, |R| feeds
        // a shared envelope; unlinked = each channel updates its own.
        // Then compute per-channel gain reduction (linear) for each band.
        let mut gain_lin: [[f32; 3]; 2] = [[1.0; 3]; 2];
        let mut max_gr_db_low: f32 = 0.0;
        let mut max_gr_db_mid: f32 = 0.0;
        let mut max_gr_db_high: f32 = 0.0;
        let knee = self.coeffs.comp_knee_db;
        let link = self.coeffs.comp_link_stereo;
        // Each iteration of `b` walks band index in (low, mid, high) order.
        let band_params: [(f32, f32, f32, f32); 3] = [
            (
                self.coeffs.comp_low_threshold_db,
                self.coeffs.comp_low_ratio,
                self.coeffs.comp_low_attack_alpha,
                self.coeffs.comp_low_release_alpha,
            ),
            (
                self.coeffs.comp_mid_threshold_db,
                self.coeffs.comp_mid_ratio,
                self.coeffs.comp_mid_attack_alpha,
                self.coeffs.comp_mid_release_alpha,
            ),
            (
                self.coeffs.comp_high_threshold_db,
                self.coeffs.comp_high_ratio,
                self.coeffs.comp_high_attack_alpha,
                self.coeffs.comp_high_release_alpha,
            ),
        ];

        for b in 0..3 {
            let (thr_db, ratio, alpha_a, alpha_r) = band_params[b];
            // Detector input(s). For linked stereo, use the per-band shared
            // max-|sample| across channels; for unlinked, each channel runs
            // its own detector. We always advance both channels' envelope
            // followers so the state stays valid if the user toggles link at
            // runtime.
            let mut linked_x: f32 = 0.0;
            if link {
                for ch in 0..ch_active {
                    let a = bands[ch][b].abs();
                    if a > linked_x {
                        linked_x = a;
                    }
                }
            }
            for ch in 0..ch_active {
                let detector = if link {
                    linked_x
                } else {
                    bands[ch][b].abs()
                };
                // Update the per-band per-channel envelope follower.
                let env_ref = match b {
                    0 => &mut self.states[ch].comp_low_env,
                    1 => &mut self.states[ch].comp_mid_env,
                    _ => &mut self.states[ch].comp_high_env,
                };
                let alpha = if detector > *env_ref { alpha_a } else { alpha_r };
                *env_ref = alpha * (*env_ref) + (1.0 - alpha) * detector;
                let env = *env_ref;
                // Convert envelope to dB (clamp away from 0 to avoid -inf).
                let env_db = if env <= 1.0e-7 {
                    -140.0
                } else {
                    20.0 * env.log10()
                };
                // Soft-knee downward compression. Below (thr - knee/2): unity.
                // Above (thr + knee/2): full ratio reduction. In between:
                // quadratic interpolation (standard soft-knee formula).
                let half_knee = knee * 0.5;
                let gr_db = if env_db < thr_db - half_knee {
                    0.0
                } else if env_db > thr_db + half_knee {
                    // Reduction = (env_db - thr_db) * (1 - 1/ratio).
                    (env_db - thr_db) * (1.0 - 1.0 / ratio)
                } else {
                    // Soft knee: smoothly transition from 0 to full reduction.
                    let x = env_db - (thr_db - half_knee);
                    let t = x / knee;
                    let above = (env_db - thr_db) * (1.0 - 1.0 / ratio);
                    // Half of `above` lies inside the knee; weighted by t^2
                    // gives the textbook soft-knee curve.
                    t * t * above.max(0.0)
                };
                // Apply reduction to this channel's contribution for this band.
                let gain_db = -gr_db.max(0.0);
                let g_lin = 10.0_f32.powf(gain_db / 20.0);
                gain_lin[ch][b] = g_lin;
                // Track the maximum reduction across channels for the GR
                // snapshot atomics. Stored as |reduction_db| (positive).
                let gr_abs = gr_db.max(0.0);
                match b {
                    0 => {
                        if gr_abs > max_gr_db_low {
                            max_gr_db_low = gr_abs;
                        }
                    }
                    1 => {
                        if gr_abs > max_gr_db_mid {
                            max_gr_db_mid = gr_abs;
                        }
                    }
                    _ => {
                        if gr_abs > max_gr_db_high {
                            max_gr_db_high = gr_abs;
                        }
                    }
                }
            }
        }

        // Recombine: sum the three reduced bands, then apply makeup per band.
        // Makeup happens BEFORE the sum (each band multiplied by its makeup)
        // so a band that isn't being reduced doesn't get scaled by the
        // others' makeup.
        for ch in 0..ch_active {
            let [low, mid, high] = bands[ch];
            let y = low * gain_lin[ch][0] * self.coeffs.comp_low_makeup_lin
                + mid * gain_lin[ch][1] * self.coeffs.comp_mid_makeup_lin
                + high * gain_lin[ch][2] * self.coeffs.comp_high_makeup_lin;
            frame[ch] = y;
        }

        // GR snapshot atomic-max fold. Integer storage of |reduction_db| * 100
        // avoids the IEEE 754 sign-bit ordering edge case the peak code
        // documents (negative dB values would otherwise sort wrong via
        // u32-bit compare). 0 = no reduction in the window.
        use std::sync::atomic::Ordering;
        let to_u = |db: f32| (db.max(0.0) * 100.0) as u32;
        self.gr_snapshots.low.fetch_max(to_u(max_gr_db_low), Ordering::Relaxed);
        self.gr_snapshots.mid.fetch_max(to_u(max_gr_db_mid), Ordering::Relaxed);
        self.gr_snapshots.high.fetch_max(to_u(max_gr_db_high), Ordering::Relaxed);
    }
```

---

- [ ] **Step 7.2: Mirror the compressor stage in `process_sample` (legacy path)**

Locate `process_sample` (around line 625 of `dsp.rs`). After the existing `presence_air` apply (around line 637), insert the compressor stage. The legacy path runs per-channel per-sample, so the compressor must operate on a single channel at a time. Mirror the body of `apply_multiband_compressor` but specialized for one channel and one sample:

```rust
        if self.coeffs.compression_active {
            let state = &mut self.states[idx];
            // Per-channel band split.
            let low_a = state.comp_split.low_lp1.process(&self.coeffs.comp_low_lp, y);
            let low = state.comp_split.low_lp2.process(&self.coeffs.comp_low_lp, low_a);
            let m1 = state.comp_split.mid_hp1.process(&self.coeffs.comp_mid_hp, y);
            let m2 = state.comp_split.mid_hp2.process(&self.coeffs.comp_mid_hp, m1);
            let m3 = state.comp_split.mid_lp1.process(&self.coeffs.comp_mid_lp, m2);
            let mid = state.comp_split.mid_lp2.process(&self.coeffs.comp_mid_lp, m3);
            let h1 = state.comp_split.high_hp1.process(&self.coeffs.comp_high_hp, y);
            let high = state.comp_split.high_hp2.process(&self.coeffs.comp_high_hp, h1);
            let bands = [low, mid, high];
            let band_params: [(f32, f32, f32, f32); 3] = [
                (
                    self.coeffs.comp_low_threshold_db,
                    self.coeffs.comp_low_ratio,
                    self.coeffs.comp_low_attack_alpha,
                    self.coeffs.comp_low_release_alpha,
                ),
                (
                    self.coeffs.comp_mid_threshold_db,
                    self.coeffs.comp_mid_ratio,
                    self.coeffs.comp_mid_attack_alpha,
                    self.coeffs.comp_mid_release_alpha,
                ),
                (
                    self.coeffs.comp_high_threshold_db,
                    self.coeffs.comp_high_ratio,
                    self.coeffs.comp_high_attack_alpha,
                    self.coeffs.comp_high_release_alpha,
                ),
            ];
            let makeup_lin = [
                self.coeffs.comp_low_makeup_lin,
                self.coeffs.comp_mid_makeup_lin,
                self.coeffs.comp_high_makeup_lin,
            ];
            let knee = self.coeffs.comp_knee_db;
            let mut sum_y = 0.0f32;
            for b in 0..3 {
                let (thr_db, ratio, alpha_a, alpha_r) = band_params[b];
                let env_ref = match b {
                    0 => &mut state.comp_low_env,
                    1 => &mut state.comp_mid_env,
                    _ => &mut state.comp_high_env,
                };
                let detector = bands[b].abs();
                let alpha = if detector > *env_ref { alpha_a } else { alpha_r };
                *env_ref = alpha * (*env_ref) + (1.0 - alpha) * detector;
                let env = *env_ref;
                let env_db = if env <= 1.0e-7 {
                    -140.0
                } else {
                    20.0 * env.log10()
                };
                let half_knee = knee * 0.5;
                let gr_db = if env_db < thr_db - half_knee {
                    0.0
                } else if env_db > thr_db + half_knee {
                    (env_db - thr_db) * (1.0 - 1.0 / ratio)
                } else {
                    let x = env_db - (thr_db - half_knee);
                    let t = x / knee;
                    let above = (env_db - thr_db) * (1.0 - 1.0 / ratio);
                    t * t * above.max(0.0)
                };
                let g_lin = 10.0_f32.powf(-gr_db.max(0.0) / 20.0);
                sum_y += bands[b] * g_lin * makeup_lin[b];
            }
            y = sum_y;
        }
```

The legacy path skips the GR atomics — it's not used for live monitoring; only export rendering uses `process_frame_inplace`. The single-channel path also can't link stereo (no second channel), so it always runs unlinked.

---

- [ ] **Step 7.3: Run the audio-driven Phase 12.2 tests**

```powershell
cargo test --lib --manifest-path src-tauri/Cargo.toml compression_density_at_one_attenuates_loud_signal compression_linked_stereo_applies_same_gain_to_both_channels
```

Expected: both pass. If `compression_density_at_one_attenuates_loud_signal` fails with insufficient attenuation, check that the macro is mapping `density=1.0` → `-24 dBFS` threshold and that the mid envelope follower's alpha values match. If `compression_linked_stereo_applies_same_gain_to_both_channels` fails, verify the `link` branch in Step 7.1 takes max of |L|, |R| before driving the envelope.

---

- [ ] **Step 7.4: Run the full lib test suite**

```powershell
cargo test --lib --manifest-path src-tauri/Cargo.toml
```

Expected: **32/32 pass** (was 24; +8 compression tests). Specifically the 8 new names:
- `compression_density_default_is_identity`
- `lr4_crossover_sums_flat_at_unity`
- `compression_density_at_one_attenuates_loud_signal`
- `compression_per_band_override_replaces_macro`
- `envelope_follower_attack_release_time_constants`
- `compression_linked_stereo_applies_same_gain_to_both_channels`
- `compression_makeup_gain_compensates_threshold_drop`
- `compression_clamps_density_into_range`

If any pre-existing test fails (especially `process_frame_applies_width_inside_full_chain`, `chain_coeffs_default_width_is_neutral`, or the warmth/presence_air tests), the compressor isn't taking the identity early-return when the slider is untouched. Re-check `compression_active` logic in `from_settings`.

---

## Task 8: Wire the GR atomics into `audio.rs` (snapshot tick + `MasteringSource`)

**Files:**
- Modify: `src-tauri/src/audio.rs` (`AudioThreadState`, `PlaybackSnapshot`, snapshot tick block, `MasteringSource`, `handle_play`, `handle_play_master`)

---

- [ ] **Step 8.1: Add 3 GR slots to `PlaybackSnapshot` (around line 447 of `audio.rs`)**

Replace:

```rust
#[derive(Debug, Clone)]
pub struct PlaybackSnapshot {
    pub track_id: Option<TrackId>,
    pub position_sec: f64,
    pub is_playing: bool,
    pub is_loaded: bool,
    pub peak_dbfs: f32,
}
```

With:

```rust
#[derive(Debug, Clone)]
pub struct PlaybackSnapshot {
    pub track_id: Option<TrackId>,
    pub position_sec: f64,
    pub is_playing: bool,
    pub is_loaded: bool,
    pub peak_dbfs: f32,
    /// Phase 12.2 — per-band compressor gain reduction (in dB, negative)
    /// since the last snapshot tick. `SILENCE_DBFS` when the window had no
    /// reduction or no signal.
    pub gr_low_db: f32,
    pub gr_mid_db: f32,
    pub gr_high_db: f32,
}
```

And update `impl Default for PlaybackSnapshot`:

```rust
impl Default for PlaybackSnapshot {
    fn default() -> Self {
        Self {
            track_id: None,
            position_sec: 0.0,
            is_playing: false,
            is_loaded: false,
            peak_dbfs: SILENCE_DBFS,
            gr_low_db: SILENCE_DBFS,
            gr_mid_db: SILENCE_DBFS,
            gr_high_db: SILENCE_DBFS,
        }
    }
}
```

---

- [ ] **Step 8.2: Add 3 GR atomics to `AudioThreadState` (around line 608 of `audio.rs`)**

Append to `AudioThreadState`:

```rust
    /// Phase 12.2 — per-band GR snapshot slots. Mirror of `peak_linear`'s
    /// pattern: `MasteringSource` (via the contained `MasteringChain`)
    /// fetch_max's |reduction_db| * 100 as u32 per frame; the audio thread
    /// swaps to 0 each tick and converts to negative dB. 0 = no reduction in
    /// the window.
    gr_low: Arc<AtomicU32>,
    gr_mid: Arc<AtomicU32>,
    gr_high: Arc<AtomicU32>,
```

---

- [ ] **Step 8.3: Initialize the new atomics in both `handle_play` (line ~777) and `handle_play_master` (line ~845)**

In both `state.is_none()` branches, replace the existing `peak_linear: Arc::new(AtomicU32::new(0))` line with three lines:

```rust
            peak_linear: Arc::new(AtomicU32::new(0)),
            gr_low: Arc::new(AtomicU32::new(0)),
            gr_mid: Arc::new(AtomicU32::new(0)),
            gr_high: Arc::new(AtomicU32::new(0)),
```

In `handle_play` (after the `s.peak_linear.store(0, ...)` reset around line 803), add:

```rust
    s.gr_low.store(0, Ordering::Relaxed);
    s.gr_mid.store(0, Ordering::Relaxed);
    s.gr_high.store(0, Ordering::Relaxed);
```

In `handle_play_master` (after the `s.peak_linear.store(0, ...)` reset around line 871), add:

```rust
    s.gr_low.store(0, Ordering::Relaxed);
    s.gr_mid.store(0, Ordering::Relaxed);
    s.gr_high.store(0, Ordering::Relaxed);
```

---

- [ ] **Step 8.4: Plumb the atomics into `MasteringSource` via `crate::dsp::GrSnapshotSlots`**

In `handle_play_master`, replace the `let chain = crate::dsp::MasteringChain::new(...)` call (around line 874) with:

```rust
    let gr_slots = crate::dsp::GrSnapshotSlots {
        low: s.gr_low.clone(),
        mid: s.gr_mid.clone(),
        high: s.gr_high.clone(),
    };
    let chain = crate::dsp::MasteringChain::new_with_gr_snapshots(
        pcm.sample_rate,
        pcm.channels as usize,
        settings,
        gr_slots,
    );
```

(The `MasteringSource::new` signature does NOT need to change — the chain it holds already owns the cloned `Arc`s.)

---

- [ ] **Step 8.5: Update the snapshot tick block (around line 733-756 of `audio.rs`) to swap the new atomics**

Replace the existing block:

```rust
        let next_snap = match state.as_ref() {
            Some(s) if s.current_track.is_some() => {
                let peak_bits = s.peak_linear.swap(0, Ordering::Relaxed);
                let peak_linear = f32::from_bits(peak_bits);
                let peak_dbfs = if peak_linear.is_finite() {
                    linear_to_dbfs(peak_linear)
                } else {
                    SILENCE_DBFS
                };
                PlaybackSnapshot {
                    track_id: s.current_track.clone(),
                    position_sec: s.sink.get_pos().as_secs_f64(),
                    is_playing: !s.sink.is_paused() && !s.sink.empty(),
                    is_loaded: true,
                    peak_dbfs,
                }
            }
            _ => PlaybackSnapshot::default(),
        };
```

With:

```rust
        let next_snap = match state.as_ref() {
            Some(s) if s.current_track.is_some() => {
                let peak_bits = s.peak_linear.swap(0, Ordering::Relaxed);
                let peak_linear = f32::from_bits(peak_bits);
                let peak_dbfs = if peak_linear.is_finite() {
                    linear_to_dbfs(peak_linear)
                } else {
                    SILENCE_DBFS
                };
                // Phase 12.2 — per-band GR snapshot conversion. Atomics hold
                // |reduction_db| * 100 as u32; 0 = no reduction. Convert to
                // negative dB (reduction direction); 0 maps to SILENCE_DBFS
                // so the UI's GR meter reads as idle when nothing is fighting
                // the compressor.
                let gr_u = |a: &Arc<AtomicU32>| a.swap(0, Ordering::Relaxed);
                let to_gr_db = |u: u32| -> f32 {
                    if u == 0 {
                        SILENCE_DBFS
                    } else {
                        -(u as f32) / 100.0
                    }
                };
                PlaybackSnapshot {
                    track_id: s.current_track.clone(),
                    position_sec: s.sink.get_pos().as_secs_f64(),
                    is_playing: !s.sink.is_paused() && !s.sink.empty(),
                    is_loaded: true,
                    peak_dbfs,
                    gr_low_db: to_gr_db(gr_u(&s.gr_low)),
                    gr_mid_db: to_gr_db(gr_u(&s.gr_mid)),
                    gr_high_db: to_gr_db(gr_u(&s.gr_high)),
                }
            }
            _ => PlaybackSnapshot::default(),
        };
```

---

- [ ] **Step 8.6: Plumb the GR fields into the `PlaybackTick` event emission**

Find where the audio thread emits `PlaybackTick` events to the frontend (search for `PlaybackTick` in `audio.rs` or `lib.rs`). Add the three new fields (`gr_low_db`, `gr_mid_db`, `gr_high_db`) to the constructor, mirrored from `snapshot`. If the emit code reads only the four legacy fields, extend it now to also read the three GR fields from the snapshot and place them in the emitted `PlaybackTick`.

```powershell
# Find the PlaybackTick emit site if not obvious from grep
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | findstr /i "playbacktick"
```

If the compiler points to a missing field on a `PlaybackTick { ... }` literal, add `gr_low_db`, `gr_mid_db`, `gr_high_db` reading the snapshot's matching field.

---

- [ ] **Step 8.7: Verify the crate compiles**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: clean compile. Warnings about unused fields on `AudioThreadState` are fine if any new field isn't read — those should be transient.

---

## Task 9: Update `exports.rs` — already-compressed source advisory

**Files:**
- Modify: `src-tauri/src/exports.rs`
- Modify: `src-tauri/tests/contracts.rs` (every existing call to `run_export_checks(report)` becomes `run_export_checks(report, None, None)`)

Note: extending the signature of a `#[tauri::command]` is a breaking change for any JS caller. The frontend's `runExportChecks` wrapper (in `src/lib/api.ts`) will need its argument list updated in the same change so the Tauri marshaling still matches.

---

- [ ] **Step 9.1: Extend `run_export_checks` signature in `exports.rs`**

Replace:

```rust
#[tauri::command]
pub async fn run_export_checks(report: ExportReport) -> CommandResult<Vec<QualityCheck>> {
    let mut checks = Vec::new();
```

With:

```rust
#[tauri::command]
pub async fn run_export_checks(
    report: ExportReport,
    source_analysis: Option<AnalysisResult>,
    settings: Option<MasteringSettings>,
) -> CommandResult<Vec<QualityCheck>> {
    let mut checks = Vec::new();
```

---

- [ ] **Step 9.2: Add the `comp_density_on_compressed_source` advisory**

Just before the `if checks.is_empty()` block at the bottom of `run_export_checks` (around line 74), insert:

```rust
    // Phase 12.2 — already-compressed source advisory. Fires when the SOURCE
    // material is dynamically squashed (DR < 6 LU) AND the user is asking for
    // moderate-to-heavy compression density (> 0.3) AND they haven't manually
    // overridden any per-band threshold (per-band overrides imply the user
    // knows what they're doing and the macro isn't blindly driving). Advisory
    // only — does not block export.
    if let (Some(analysis), Some(s)) = (source_analysis.as_ref(), settings.as_ref()) {
        let density = s.advanced.compression_density.unwrap_or(0.0);
        let no_per_band_threshold_overrides = s.advanced.compression_low_threshold_db.is_none()
            && s.advanced.compression_mid_threshold_db.is_none()
            && s.advanced.compression_high_threshold_db.is_none();
        if analysis.dynamic_range_lu < 6.0
            && density > 0.3
            && no_per_band_threshold_overrides
        {
            checks.push(QualityCheck {
                level: QualityLevel::Warning,
                code: "comp_density_on_compressed_source".to_string(),
                message: "Source appears already compressed (DR < 6 LU). Heavy compression may pump.".to_string(),
            });
        }
    }
```

---

- [ ] **Step 9.3: Update every existing `run_export_checks(report)` call in `src-tauri/tests/contracts.rs` to pass `None, None`**

Find every call site (there are at least four: in `phase_12_1_real_fixture_metering_snapshot`, `run_export_checks_warns_on_high_true_peak`, `run_export_checks_passes_silently_when_clean`, `run_export_checks_warns_on_low_streaming_headroom`, `run_export_checks_streaming_headroom_quiet_at_streaming_ceiling`). Update each:

```rust
let checks = exports::run_export_checks(report).await.expect("checks");
```

to:

```rust
let checks = exports::run_export_checks(report, None, None).await.expect("checks");
```

---

- [ ] **Step 9.4: Verify the existing export-check tests still pass**

```powershell
cargo test --test contracts --manifest-path src-tauri/Cargo.toml run_export_checks_
```

Expected: 4 passing tests — `run_export_checks_warns_on_high_true_peak`, `run_export_checks_passes_silently_when_clean`, `run_export_checks_warns_on_low_streaming_headroom`, `run_export_checks_streaming_headroom_quiet_at_streaming_ceiling`. The 2 new compression contract tests are added in Task 10.

---

## Task 10: Write the 2 new contract tests

**Files:**
- Modify: `src-tauri/tests/contracts.rs` (append 2 new tests)

---

- [ ] **Step 10.1: Append `mastering_render_with_heavy_compression_attenuates_loud_section`**

Append at the end of `contracts.rs`:

```rust
/// Phase 12.2 — end-to-end render comparison. With macro density=1.0 the
/// 5-second loud sine should land at integrated LUFS at least 3 LU lower
/// than at density=0.0. Pins the wiring from `MasteringSettings.advanced.
/// compression_density` all the way through `MasteringChain` and the
/// downstream LUFS measurement on the rendered output.
#[test]
fn mastering_render_with_heavy_compression_attenuates_loud_section() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let in_path = tmp.path().join("loud_sine.wav");
    // 5 seconds of a 0.8-amplitude 1 kHz sine at 44.1 kHz stereo. Deep above
    // the macro threshold for density=1.0 (~-24 dBFS), so the compressor
    // gets a steady workout.
    write_sine_wav(&in_path, 44_100, 5.0, 1_000.0, 2);

    // Render at density=0.0 (compressor inactive) and density=1.0.
    let mut s0 = default_settings();
    s0.advanced.compression_density = Some(0.0);
    let mut s1 = default_settings();
    s1.advanced.compression_density = Some(1.0);
    let out0 = tmp.path().join("master_d0.wav");
    let out1 = tmp.path().join("master_d1.wav");
    engine::render_track_master_sync(&in_path, &s0, &out0)
        .expect("render density=0");
    engine::render_track_master_sync(&in_path, &s1, &out1)
        .expect("render density=1");
    let m0 = engine::analyze_wav(&out0).expect("analyze d0");
    let m1 = engine::analyze_wav(&out1).expect("analyze d1");
    let delta_lu = m1.lufs_integrated - m0.lufs_integrated;
    assert!(
        delta_lu <= -3.0,
        "density=1.0 render should be >=3 LU quieter than density=0.0 \
         (got {:.2} LU; d0 LUFS = {}, d1 LUFS = {})",
        delta_lu,
        m0.lufs_integrated,
        m1.lufs_integrated
    );
}
```

If the function names `engine::render_track_master_sync` and `engine::analyze_wav` don't exist with those exact spellings, use whatever the existing render+analyze test infrastructure uses (search `contracts.rs` for `mastering_render_writes_processed_wav` to find the right entry points and copy the pattern).

---

- [ ] **Step 10.2: Append `run_export_checks_warns_on_compressed_source_with_heavy_density`**

```rust
#[tokio::test]
async fn run_export_checks_warns_on_compressed_source_with_heavy_density() {
    // Synthesize a source AnalysisResult with DR = 4 LU (highly squashed)
    // and a MasteringSettings with compression_density = 0.5 (above the
    // 0.3 trigger). Expect the comp_density_on_compressed_source advisory.
    let analysis = AnalysisResult {
        track_id: TrackId("stub".to_string()),
        lufs_integrated: -10.0,
        lufs_short_term_max: -8.0,
        true_peak_dbtp: -0.5,
        dynamic_range_lu: 4.0,
        spectral_balance: SpectralBalance {
            low: 0.33,
            mid: 0.34,
            high: 0.33,
        },
        transient_density: 0.5,
        stereo_width: 0.5,
        recommended_universal: default_settings(),
        measured_at_iso: "2026-05-12T12:00:00Z".to_string(),
        inferred_role: None,
        role_confidence: None,
        inferred_character: None,
        character_confidence: None,
    };
    let mut settings = default_settings();
    settings.advanced.compression_density = Some(0.5);
    let report = ExportReport {
        track_id: TrackId("t".to_string()),
        output_path: "out.wav".to_string(),
        measured_lufs: -14.0,
        measured_true_peak_dbtp: -1.2,
        measured_dynamic_range_lu: 4.0,
        source_format: "wav".to_string(),
        destination_format: "wav".to_string(),
        sample_rate: 44_100,
        bit_depth: 24,
        checks: Vec::new(),
    };
    let checks = exports::run_export_checks(report, Some(analysis), Some(settings))
        .await
        .expect("checks ok");
    assert!(
        checks
            .iter()
            .any(|c| c.code == "comp_density_on_compressed_source"),
        "expected comp_density_on_compressed_source advisory, got: {:?}",
        checks.iter().map(|c| &c.code).collect::<Vec<_>>()
    );
    // Per-band threshold override should suppress the advisory.
    let mut settings2 = default_settings();
    settings2.advanced.compression_density = Some(0.5);
    settings2.advanced.compression_mid_threshold_db = Some(-30.0);
    let report2 = ExportReport {
        track_id: TrackId("t".to_string()),
        output_path: "out.wav".to_string(),
        measured_lufs: -14.0,
        measured_true_peak_dbtp: -1.2,
        measured_dynamic_range_lu: 4.0,
        source_format: "wav".to_string(),
        destination_format: "wav".to_string(),
        sample_rate: 44_100,
        bit_depth: 24,
        checks: Vec::new(),
    };
    // Re-synthesize the analysis since AnalysisResult isn't Clone-by-default
    // in all repos; copy via fields.
    let analysis2 = AnalysisResult {
        track_id: TrackId("stub".to_string()),
        lufs_integrated: -10.0,
        lufs_short_term_max: -8.0,
        true_peak_dbtp: -0.5,
        dynamic_range_lu: 4.0,
        spectral_balance: SpectralBalance {
            low: 0.33,
            mid: 0.34,
            high: 0.33,
        },
        transient_density: 0.5,
        stereo_width: 0.5,
        recommended_universal: default_settings(),
        measured_at_iso: "2026-05-12T12:00:00Z".to_string(),
        inferred_role: None,
        role_confidence: None,
        inferred_character: None,
        character_confidence: None,
    };
    let checks2 = exports::run_export_checks(report2, Some(analysis2), Some(settings2))
        .await
        .expect("checks ok");
    assert!(
        !checks2
            .iter()
            .any(|c| c.code == "comp_density_on_compressed_source"),
        "per-band threshold override should suppress the advisory, got: {:?}",
        checks2.iter().map(|c| &c.code).collect::<Vec<_>>()
    );
}
```

---

- [ ] **Step 10.3: Run the full test suite**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: **71/71 pass** (was 61; +8 lib + 2 contract). Specifically:
- `running 32 tests` in `lib.rs` (was 24).
- `running 37 tests` in `contracts.rs` (was 35).
- Doc-tests: 0 (unchanged).

If `mastering_render_with_heavy_compression_attenuates_loud_section` fails with insufficient delta, the macro→threshold mapping may not be aggressive enough, or `compression_active` may not be flipping on. Re-check Step 5.2's `compression_active` formula.

If `run_export_checks_warns_on_compressed_source_with_heavy_density` fails, re-check Step 9.2 — the dr_lu < 6.0 cutoff and the per-band-override suppression both need to hold.

---

## Task 11: Frontend — bindings, transport, AdvancedPanel, GR readouts

**Files:**
- Modify: `src/bindings.ts`
- Modify: `src/hooks/useTrackMaster.ts`
- Modify: `src/App.tsx` (`AdvancedPanel`, `StaleBar`, new `GrIndicator`)
- Modify: `src/App.css`

---

- [ ] **Step 11.1: Extend `AdvancedSettings` and `PlaybackTick` in `src/bindings.ts`**

Locate `AdvancedSettings` (around line 33). Replace:

```ts
export interface AdvancedSettings {
  lufs_offset_db: number | null;
  ceiling_dbtp: number | null;
  width: number | null;
  warmth: number | null;
  presence_air: number | null;
  compression_density: number | null;
  bit_depth: number | null;
  target_sample_rate: number | null;
}
```

With:

```ts
export interface AdvancedSettings {
  lufs_offset_db: number | null;
  ceiling_dbtp: number | null;
  width: number | null;
  warmth: number | null;
  presence_air: number | null;
  compression_density: number | null;
  // Phase 12.2 per-band compressor overrides. `null` = let the macro
  // (compression_density) drive that band's threshold; per-band ratio/
  // attack/release fall back to fixed musical defaults in the backend.
  compression_low_threshold_db: number | null;
  compression_low_ratio: number | null;
  compression_low_attack_ms: number | null;
  compression_low_release_ms: number | null;
  compression_mid_threshold_db: number | null;
  compression_mid_ratio: number | null;
  compression_mid_attack_ms: number | null;
  compression_mid_release_ms: number | null;
  compression_high_threshold_db: number | null;
  compression_high_ratio: number | null;
  compression_high_attack_ms: number | null;
  compression_high_release_ms: number | null;
  /// `null` or `true` = linked stereo (max-of-|L|,|R| envelope per band).
  /// `false` = independent L/R envelopes per band.
  compression_link_stereo: boolean | null;
  bit_depth: number | null;
  target_sample_rate: number | null;
}
```

Locate `PlaybackTick` (around line 184). Replace:

```ts
export interface PlaybackTick {
  track_id: TrackId | null;
  position_sec: number;
  is_playing: boolean;
  is_loaded: boolean;
  peak_dbfs: number;
}
```

With:

```ts
export interface PlaybackTick {
  track_id: TrackId | null;
  position_sec: number;
  is_playing: boolean;
  is_loaded: boolean;
  peak_dbfs: number;
  /// Phase 12.2 — per-band compressor gain reduction in dB (negative).
  /// `-120` is the silence sentinel; values like -2.3 mean 2.3 dB of GR.
  gr_low_db: number;
  gr_mid_db: number;
  gr_high_db: number;
}
```

---

- [ ] **Step 11.2: Extend default settings in `src/hooks/useTrackMaster.ts`**

Around line 38, the existing `DEFAULT_SETTINGS.advanced` literal has `compression_density: null`. Add the 13 new fields:

```ts
    compression_density: null,
    compression_low_threshold_db: null,
    compression_low_ratio: null,
    compression_low_attack_ms: null,
    compression_low_release_ms: null,
    compression_mid_threshold_db: null,
    compression_mid_ratio: null,
    compression_mid_attack_ms: null,
    compression_mid_release_ms: null,
    compression_high_threshold_db: null,
    compression_high_ratio: null,
    compression_high_attack_ms: null,
    compression_high_release_ms: null,
    compression_link_stereo: null,
```

---

- [ ] **Step 11.3: Extend `transport` state and `onPlaybackTick` handler in `useTrackMaster.ts`**

Locate the `useState` block (around line 78). Replace the relevant slice:

```ts
  const [transport, setTransport] = useState({
    isPlaying: false,
    currentTimeSec: 0,
    playbackKind: "source" as PlaybackKindUI,
    loop: false,
    volumeMatch: false,
    peakDbfs: -120,
  });
```

With:

```ts
  const [transport, setTransport] = useState({
    isPlaying: false,
    currentTimeSec: 0,
    playbackKind: "source" as PlaybackKindUI,
    loop: false,
    volumeMatch: false,
    peakDbfs: -120,
    // Phase 12.2 per-band compressor GR readouts. -120 = silence sentinel
    // ("no reduction in the window"). Driven by PlaybackTick → snapshot →
    // atomic-swap on the backend audio thread.
    compressionGr: { low: -120, mid: -120, high: -120 },
  });
```

Then in the `onPlaybackTick` handler (around line 138-145), replace:

```ts
    onPlaybackTick((tick) => {
      setLoadedTrackId(tick.is_loaded ? tick.track_id : null);
      setTransport((t) => ({
        ...t,
        currentTimeSec: tick.position_sec,
        isPlaying: tick.is_playing,
        peakDbfs: tick.peak_dbfs,
      }));
    }).then((fn) => {
      unlistenTick = fn;
    });
```

With:

```ts
    onPlaybackTick((tick) => {
      setLoadedTrackId(tick.is_loaded ? tick.track_id : null);
      setTransport((t) => ({
        ...t,
        currentTimeSec: tick.position_sec,
        isPlaying: tick.is_playing,
        peakDbfs: tick.peak_dbfs,
        compressionGr: {
          low: tick.gr_low_db,
          mid: tick.gr_mid_db,
          high: tick.gr_high_db,
        },
      }));
    }).then((fn) => {
      unlistenTick = fn;
    });
```

---

- [ ] **Step 11.4: Drop the "(coming soon)" label on the macro slider in `AdvancedPanel`**

Locate the `compression_density` `NumberField` (around line 1402 of `App.tsx`). Replace:

```tsx
        <NumberField
          label="Compression (coming soon)"
          value={a.compression_density}
          step={0.05}
          min={0}
          max={1}
          format={(v) => v.toFixed(2)}
          onChange={(v) => update("compression_density", v)}
        />
```

With:

```tsx
        <NumberField
          label="Compression density"
          value={a.compression_density}
          step={0.05}
          min={0}
          max={1}
          format={(v) => v.toFixed(2)}
          onChange={(v) => update("compression_density", v)}
        />
```

---

- [ ] **Step 11.5: Add a collapsible "Per-band" subsection with 3 columns × 4 NumberFields each + link-stereo checkbox**

Immediately after the `compression_density` NumberField (just before `<SelectField label="Bit depth"`), insert:

```tsx
        <CompressionPerBandSubsection
          a={a}
          onUpdate={update}
        />
```

Then add the new component above `AdvancedPanel` (or below it — match the file's existing pattern; the warmth/air plan placed `ClippingIndicator` below `StaleBar`). Recommended location: just below the existing `AdvancedPanel` function (after line 1436).

```tsx
function CompressionPerBandSubsection({
  a,
  onUpdate,
}: {
  a: MasteringSettings["advanced"];
  onUpdate: (
    field: keyof MasteringSettings["advanced"],
    value: number | boolean | null,
  ) => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <details
      className="compression-per-band"
      open={open}
      onToggle={(e) => setOpen((e.target as HTMLDetailsElement).open)}
    >
      <summary className="adv-label">Per-band compressor</summary>
      <div className="compression-link">
        <label>
          <input
            type="checkbox"
            checked={a.compression_link_stereo !== false}
            onChange={(e) =>
              onUpdate("compression_link_stereo", e.target.checked ? null : false)
            }
          />
          Link stereo (default on — drives both channels from a shared envelope)
        </label>
      </div>
      <div className="compression-per-band-grid">
        <CompressionBandColumn
          label="Low"
          threshold={a.compression_low_threshold_db}
          ratio={a.compression_low_ratio}
          attack={a.compression_low_attack_ms}
          release={a.compression_low_release_ms}
          onThreshold={(v) => onUpdate("compression_low_threshold_db", v)}
          onRatio={(v) => onUpdate("compression_low_ratio", v)}
          onAttack={(v) => onUpdate("compression_low_attack_ms", v)}
          onRelease={(v) => onUpdate("compression_low_release_ms", v)}
        />
        <CompressionBandColumn
          label="Mid"
          threshold={a.compression_mid_threshold_db}
          ratio={a.compression_mid_ratio}
          attack={a.compression_mid_attack_ms}
          release={a.compression_mid_release_ms}
          onThreshold={(v) => onUpdate("compression_mid_threshold_db", v)}
          onRatio={(v) => onUpdate("compression_mid_ratio", v)}
          onAttack={(v) => onUpdate("compression_mid_attack_ms", v)}
          onRelease={(v) => onUpdate("compression_mid_release_ms", v)}
        />
        <CompressionBandColumn
          label="High"
          threshold={a.compression_high_threshold_db}
          ratio={a.compression_high_ratio}
          attack={a.compression_high_attack_ms}
          release={a.compression_high_release_ms}
          onThreshold={(v) => onUpdate("compression_high_threshold_db", v)}
          onRatio={(v) => onUpdate("compression_high_ratio", v)}
          onAttack={(v) => onUpdate("compression_high_attack_ms", v)}
          onRelease={(v) => onUpdate("compression_high_release_ms", v)}
        />
      </div>
    </details>
  );
}

function CompressionBandColumn({
  label,
  threshold,
  ratio,
  attack,
  release,
  onThreshold,
  onRatio,
  onAttack,
  onRelease,
}: {
  label: string;
  threshold: number | null;
  ratio: number | null;
  attack: number | null;
  release: number | null;
  onThreshold: (v: number | null) => void;
  onRatio: (v: number | null) => void;
  onAttack: (v: number | null) => void;
  onRelease: (v: number | null) => void;
}) {
  return (
    <div className="compression-band-column">
      <div className="compression-band-label">{label}</div>
      <NumberField
        label="Threshold"
        value={threshold}
        step={0.5}
        min={-60}
        max={0}
        format={(v) => `${v.toFixed(1)} dB`}
        onChange={onThreshold}
      />
      <NumberField
        label="Ratio"
        value={ratio}
        step={0.1}
        min={1}
        max={20}
        format={(v) => `${v.toFixed(1)}:1`}
        onChange={onRatio}
      />
      <NumberField
        label="Attack"
        value={attack}
        step={1}
        min={0.5}
        max={200}
        format={(v) => `${v.toFixed(1)} ms`}
        onChange={onAttack}
      />
      <NumberField
        label="Release"
        value={release}
        step={5}
        min={5}
        max={2000}
        format={(v) => `${v.toFixed(0)} ms`}
        onChange={onRelease}
      />
    </div>
  );
}
```

Update the `update` helper at the top of `AdvancedPanel` to accept the boolean value for `compression_link_stereo`. Locate (around line 1344):

```tsx
  const update = (
    field: keyof MasteringSettings["advanced"],
    value: number | null,
  ) => {
    onAdvanced({ ...a, [field]: value });
  };
```

Replace with:

```tsx
  const update = (
    field: keyof MasteringSettings["advanced"],
    value: number | boolean | null,
  ) => {
    onAdvanced({ ...a, [field]: value });
  };
```

---

- [ ] **Step 11.6: Add 3 GR readout chips to `StaleBar` alongside `ClippingIndicator`**

Locate `StaleBar` (around line 1181 of `App.tsx`). Extend the props:

```tsx
function StaleBar({
  stale,
  isRendering,
  onUpdate,
  liveUpdateStats,
  renderProgress,
  peakDbfs,
  isPlaying,
  compressionGr,
}: {
  stale: boolean;
  isRendering: boolean;
  onUpdate: () => void;
  liveUpdateStats: { attempts: number; applied: number; lastAt: number | null };
  renderProgress: { fraction: number; kind: "preview" | "master" | "album" } | null;
  peakDbfs: number;
  isPlaying: boolean;
  compressionGr: { low: number; mid: number; high: number };
}) {
```

Just below `<ClippingIndicator peakDbfs={peakDbfs} isPlaying={isPlaying} />` (around line 1226), insert:

```tsx
      <GrIndicator label="L" db={compressionGr.low} isPlaying={isPlaying} />
      <GrIndicator label="M" db={compressionGr.mid} isPlaying={isPlaying} />
      <GrIndicator label="H" db={compressionGr.high} isPlaying={isPlaying} />
```

Add the `GrIndicator` component below `ClippingIndicator` (after the closing brace of `ClippingIndicator`, around line 1304):

```tsx
// Phase 12.2 — per-band gain-reduction readout chip. Mirrors ClippingIndicator's
// shape: idle (not playing) → "—"; silent sentinel (-120 dB) → "—"; otherwise
// shows the reduction in dB. Color bands: idle/silent muted; >= -3 dB green;
// -3..-6 dB amber; < -6 dB red.
function GrIndicator({
  label,
  db,
  isPlaying,
}: {
  label: string;
  db: number;
  isPlaying: boolean;
}) {
  let state: "idle" | "ok" | "warn" | "hot";
  let text: string;
  if (!isPlaying || db <= -119.9) {
    state = "idle";
    text = `${label} —`;
  } else if (db >= -3.0) {
    state = "ok";
    text = `${label} ${db.toFixed(1)}`;
  } else if (db >= -6.0) {
    state = "warn";
    text = `${label} ${db.toFixed(1)}`;
  } else {
    state = "hot";
    text = `${label} ${db.toFixed(1)}`;
  }
  return (
    <span className={`gr-indicator gr-${state}`} title={`Compressor gain reduction (${label}): ${db.toFixed(2)} dB`}>
      {text}
    </span>
  );
}
```

Then update the `<StaleBar ... />` call site in the parent (around line 330 of `App.tsx`). Replace:

```tsx
      <StaleBar
        stale={tm.previewStale}
        isRendering={tm.isRendering}
        onUpdate={tm.updatePreview}
        liveUpdateStats={tm.liveUpdateStats}
        renderProgress={tm.renderProgress}
        peakDbfs={tm.transport.peakDbfs}
        isPlaying={tm.transport.isPlaying}
      />
```

With:

```tsx
      <StaleBar
        stale={tm.previewStale}
        isRendering={tm.isRendering}
        onUpdate={tm.updatePreview}
        liveUpdateStats={tm.liveUpdateStats}
        renderProgress={tm.renderProgress}
        peakDbfs={tm.transport.peakDbfs}
        isPlaying={tm.transport.isPlaying}
        compressionGr={tm.transport.compressionGr}
      />
```

---

- [ ] **Step 11.7: Add CSS for the new GR chips and per-band subsection**

Append to `src/App.css` (after the `@keyframes clip-pulse` block at ~line 569):

```css
/* Phase 12.2 — per-band gain-reduction readout chips. Mirror of
   .clip-indicator dimensions so the meter row stays aligned, but with
   narrower min-width since each chip carries less text. Color states map
   to: idle (muted), ok (green, no/mild reduction), warn (amber, moderate),
   hot (red, heavy). */
.gr-indicator {
  font-size: 0.7rem;
  letter-spacing: 0.04em;
  padding: 0.15rem 0.45rem;
  border-radius: 999px;
  border: 1px solid var(--border-strong);
  background: var(--bg-2);
  color: var(--text-2);
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  min-width: 4.5rem;
  text-align: center;
}

.gr-indicator.gr-idle {
  color: var(--text-2);
  opacity: 0.7;
}

.gr-indicator.gr-ok {
  color: #6cd08a;
  border-color: rgba(108, 208, 138, 0.5);
  background: rgba(108, 208, 138, 0.08);
}

.gr-indicator.gr-warn {
  color: #f5c842;
  border-color: rgba(245, 200, 66, 0.5);
  background: rgba(245, 200, 66, 0.08);
}

.gr-indicator.gr-hot {
  color: #fff;
  background: #c0392b;
  border-color: #e74c3c;
}

/* Phase 12.2 — per-band compressor subsection inside AdvancedPanel.
   Collapsed by default; expanded reveals a 3-column grid (Low / Mid / High).
   Tight typography so the four NumberFields per column still fit. */
.compression-per-band {
  grid-column: 1 / -1;
  border-top: 1px dashed var(--border-1);
  padding-top: 0.5rem;
  margin-top: 0.5rem;
}

.compression-per-band > summary {
  cursor: pointer;
  user-select: none;
  font-size: 0.78rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-2);
  padding: 0.25rem 0;
}

.compression-link {
  margin: 0.5rem 0;
  font-size: 0.78rem;
  color: var(--text-2);
}

.compression-per-band-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.75rem;
  margin-top: 0.5rem;
}

.compression-band-column {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.compression-band-label {
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--text-1);
  text-transform: uppercase;
}
```

---

- [ ] **Step 11.7b: Wire the advisory context through `api.ts` and `useTrackMaster.exportMaster`**

The Rust signature change in Task 9 added two `Option<>` args to `run_export_checks`. Tauri deserializes missing JS fields to `None`, so the existing call site doesn't fault — but the advisory will silently never fire in production unless the frontend explicitly passes the source analysis and settings. Wire that through now so the new advisory works in the running app, not just in contract tests.

Locate `src/lib/api.ts` around line 109:

```ts
  runExportChecks: (report: ExportReport) =>
    invoke<QualityCheck[]>("run_export_checks", { report }),
```

Replace with:

```ts
  runExportChecks: (
    report: ExportReport,
    sourceAnalysis?: AnalysisResult | null,
    settings?: MasteringSettings | null,
  ) =>
    invoke<QualityCheck[]>("run_export_checks", {
      report,
      sourceAnalysis: sourceAnalysis ?? null,
      settings: settings ?? null,
    }),
```

If `AnalysisResult` or `MasteringSettings` isn't already imported in `api.ts`, add them to the existing `import { ... } from "../bindings"` line at the top of the file.

Then locate `src/hooks/useTrackMaster.ts` around line 779:

```ts
      const checks = await api.runExportChecks(report);
```

Replace with:

```ts
      const checks = await api.runExportChecks(report, selectedAnalysis, selectedSettings);
```

`selectedAnalysis` and `selectedSettings` are already in scope at that call site (they're computed earlier in the same `exportMaster` flow — verify by reading a few lines above the call).

**Expected behavior after this wires through:** in production, when a user exports a track whose source has `dynamic_range_lu < 6.0` and they've set `compression_density > 0.3` without any per-band threshold overrides, the export receipt now shows the `comp_density_on_compressed_source` warning. Contract test #10.2 (`run_export_checks_warns_on_compressed_source_with_heavy_density`) verifies the Rust-side wiring; this step ensures the production call path actually delivers the context.

---

- [ ] **Step 11.8: Run the frontend build**

```powershell
npm run build
```

Expected: clean build, dist/ written. Bundle should grow roughly 5-8 KB raw / 1-2 KB gzipped vs the warmth/air slice's `~253.6 KB / ~77.6 KB gzipped` baseline, owing to the 12 new NumberFields + 3 GR chip components + the per-band CSS.

If TypeScript fails because `AdvancedSettings`'s 13 new fields aren't present on `DEFAULT_SETTINGS.advanced`, return to Step 11.2 and add them.

If `tm.transport.compressionGr` is missing on the `<StaleBar>` call site, return to Step 11.3 and verify the transport state was updated.

---

## Task 12: Final verification + progress.md + commit + push

**Files:**
- Modify: `docs/progress.md` (append new entry)

---

- [ ] **Step 12.1: Run the full Rust test suite**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:
- `running 32 tests` in lib.rs (was 24). Look for the 8 new compression test names listed in Task 7 Step 7.4.
- `running 37 tests` in contracts.rs (was 35). Look for:
  - `mastering_render_with_heavy_compression_attenuates_loud_section`
  - `run_export_checks_warns_on_compressed_source_with_heavy_density`
- Doc-tests: 0 (unchanged).
- **Total: 71/71 pass** (was 61; +10).

The real-fixture tests (`mastering_render_processes_real_fixture_if_present`, `phase_12_1_real_fixture_metering_snapshot`) must pass unchanged — the identity early-return preserves byte-equivalence when the slider is untouched. If either fails, the `compression_active` flag is incorrectly flipping on for default settings; re-check Step 5.2.

---

- [ ] **Step 12.2: Re-run the frontend build**

```powershell
npm run build
```

Expected: clean build, dist/ written, bundle ~258-261 KB raw / ~79-80 KB gzipped.

---

- [ ] **Step 12.3: Append the progress entry**

Open `docs/progress.md` and append at the end:

```markdown

## 2026-05-12 — Phase 12.2 (cont): wire compression_density (3-band multiband)

Goal:

Close the final P0 wired-controls slice of Phase 12.2. The `compression_density` Advanced slider was unwired and labeled "Compression (coming soon)"; now it drives a real 3-band linked-stereo downward compressor with engineer-grade per-band overrides exposed at the same time. Single-slice scope chosen by Dan over staging so the full surface lands before the personal-album mastering work. Brainstorm at `docs/superpowers/brainstorms/2026-05-12-compression-density-brainstorm.md`, plan at `docs/superpowers/plans/2026-05-12-compression-density.md`.

What changed:

Backend (Rust):

- **Types (`types.rs`)**: 12 new `Option<f32>` per-band override fields on `AdvancedSettings` (`compression_{low,mid,high}_{threshold_db,ratio,attack_ms,release_ms}`) + `compression_link_stereo: Option<bool>`, all `#[serde(default)]`. 3 new f32 fields on `PlaybackTick` (`gr_low_db`, `gr_mid_db`, `gr_high_db`) with `#[serde(default = "default_silence_dbfs")]`.
- **DSP (`dsp.rs`)**:
  - `BiquadCoeffs::butter_lp` / `butter_hp` — Butterworth biquad helpers for the LR4 crossover network (Q = sqrt(2)/2).
  - `LR4State` + `split_lr4_into_bands` — 3-way LR4 split at 120 Hz / 4000 Hz (8 biquads per channel: 2 LP for low, 2 HP+2 LP for mid, 2 HP for high). Cascaded Butterworth = LR4 = flat magnitude summing across band edges.
  - `EnvelopeFollower` + `alpha_from_time_ms` — peak-detector envelope with separate attack/release time constants, alpha = exp(-1/(τ·sr)).
  - `ChainCoeffs` — 20+ new fields for compressor coefficients (crossover biquads, per-band thresholds/ratios/alphas/makeup_db/makeup_lin, knee_db, link_stereo, compression_active flag).
  - `ChainCoeffs::from_settings` — macro `compression_density.unwrap_or(0.0).clamp(0,1)` → uniform threshold 0 dBFS (off) to -24 dBFS (heavy). Per-band overrides replace the macro for that band only. Per-band fixed musical defaults: low 2.5:1 / 30 ms / 300 ms, mid 2.0:1 / 15 ms / 150 ms, high 1.8:1 / 5 ms / 80 ms. Auto makeup gain per band: `(threshold_drop_db × (1 - 1/ratio)) / 2`. Soft knee 6 dB fixed. Identity early-return flag: `compression_active = false` when macro < 1e-4 AND all 12 overrides None AND link_stereo isn't Some(false).
  - `ChannelState` — `LR4State` for crossover memory + 3 `f32` envelope-follower states per band.
  - `MasteringChain` — new `GrSnapshotSlots { low, mid, high }` of `Arc<AtomicU32>` mirroring the existing `peak_linear` pattern, swapped per 50 ms tick. Integer storage (|reduction_db| × 100 as u32) avoids the IEEE 754 sign-bit ordering edge case for negative dB.
  - `MasteringChain::process_frame_inplace` — `apply_multiband_compressor` block inserted between `presence_air` and the width transform. Per-channel band split → per-band envelope follower → soft-knee gain stage → per-band makeup → recombine.
  - `MasteringChain::process_sample` — mirror in the legacy single-sample path (no GR atomics, always unlinked because single-channel).
- **Audio (`audio.rs`)**: `AudioThreadState` gets 3 new `Arc<AtomicU32>` GR slots; `handle_play_master` plumbs them into the `MasteringChain` via `new_with_gr_snapshots`; the snapshot tick block swaps and converts the integers to negative dB (with 0 → silence sentinel); `PlaybackSnapshot` gains 3 GR fields propagated into the emitted `PlaybackTick`.
- **Exports (`exports.rs`)**: `run_export_checks` signature extended with `source_analysis: Option<AnalysisResult>` and `settings: Option<MasteringSettings>` (backward-compatible — existing callers pass `None, None`). New `comp_density_on_compressed_source` advisory fires when source DR < 6 LU AND `compression_density > 0.3` AND no per-band threshold overrides.
- **Tests**: 8 new in `dsp.rs::mod tests`:
  - `compression_density_default_is_identity` — pins the identity early-return contract.
  - `lr4_crossover_sums_flat_at_unity` — pins LR4 summing flatness (< 0.012 linear at 60/1000/8000 Hz).
  - `compression_density_at_one_attenuates_loud_signal` — end-to-end ≥ 3 dB attenuation on a 0.8-amp 1 kHz sine.
  - `compression_per_band_override_replaces_macro` — per-band threshold override beats macro.
  - `envelope_follower_attack_release_time_constants` — 1 - 1/e attack tau / 1/e release tau pinned at 10 / 100 ms.
  - `compression_linked_stereo_applies_same_gain_to_both_channels` — link=true vs link=false asymmetry pin.
  - `compression_makeup_gain_compensates_threshold_drop` — sub-threshold sine sees ~+3 dB makeup at density=0.5.
  - `compression_clamps_density_into_range` — density=5.0 clamps to 1.0, density=-1.0 clamps to 0.0.
- 2 new in `contracts.rs`:
  - `mastering_render_with_heavy_compression_attenuates_loud_section` — full-render LUFS delta ≥ 3 LU between density=0.0 and density=1.0.
  - `run_export_checks_warns_on_compressed_source_with_heavy_density` — DR=4 LU + density=0.5 fires the advisory; per-band threshold override suppresses it.

Frontend (TS/React):

- `bindings.ts` — 13 new fields on `AdvancedSettings`, 3 on `PlaybackTick`.
- `useTrackMaster.ts` — DEFAULT_SETTINGS.advanced gets 13 nulls; `transport.compressionGr: { low, mid, high }` added and populated from the tick handler.
- `App.tsx`:
  - `AdvancedPanel`: "(coming soon)" dropped from `compression_density` label; new `<CompressionPerBandSubsection>` block (collapsible `<details>`) with 3 columns (Low/Mid/High) × 4 NumberFields (Threshold/Ratio/Attack/Release) + a "Link stereo" checkbox at the top.
  - `StaleBar`: 3 new `<GrIndicator label="L|M|H">` chips alongside `<ClippingIndicator>`. Color bands: ≥ -3 dB green, -3..-6 amber, < -6 red, idle/silent muted.
- `App.css` — `.gr-indicator` styles paralleling `.clip-indicator`; per-band subsection grid styles.

Verification:

- `cargo test --lib`: 32/32 pass (was 24).
- `cargo test` (full): **71/71 pass** (was 61).
- `npm run build`: clean (~258-261 KB raw / ~79-80 KB gzipped).
- Real-fixture tests unchanged — identity early-return preserves byte-equivalence at default settings.

Real-audio fixture used: none. Tests use closed-form math + synthetic sines. The real-fixture tests still run via `mastering_render_processes_real_fixture_if_present` (~120 s) and `phase_12_1_real_fixture_metering_snapshot` (~120 s) — both green.

What failed or remains partial:

- **No frontend test** for the per-band subsection or GR meter (vitest infra still deferred).
- **Crossover frequencies hard-coded** at 120 Hz / 4000 Hz; the brainstorm explicitly accepts this for v1. If Dan finds the splits don't fit his material, future polish can expose them as `compression_crossover_low_hz` / `compression_crossover_high_hz` overrides on `AdvancedSettings`.
- **Soft-knee width fixed** at 6 dB per the design — not user-tunable in v1.
- **Lookahead: none** — the existing limiter already provides lookahead; the comp doesn't need it for mastering.
- **Identity early-return when link_stereo=Some(false) but density=0 and no overrides**: the flag conservatively flips compression_active on, which is a minor perf cost but no audible difference (envelope follower runs with zero detector input → zero reduction). Not worth optimizing.

Next recommended slice:

→ Typography pass + SVG preset icons per Phase 12-tail queue (see `docs/superpowers/plans/2026-05-12-typography-pass.md` once written). Listening notes from Dan, if any, override the queue.
```

---

- [ ] **Step 12.4: Commit and push**

```powershell
git status --short
```

Expected: `M docs/progress.md`, `M src-tauri/src/audio.rs`, `M src-tauri/src/dsp.rs`, `M src-tauri/src/exports.rs`, `M src-tauri/src/types.rs`, `M src-tauri/tests/contracts.rs`, `M src/App.css`, `M src/App.tsx`, `M src/bindings.ts`, `M src/hooks/useTrackMaster.ts`.

If those are the only modifications, commit:

```powershell
git add docs/progress.md src-tauri/src/audio.rs src-tauri/src/dsp.rs src-tauri/src/exports.rs src-tauri/src/types.rs src-tauri/tests/contracts.rs src/App.css src/App.tsx src/bindings.ts src/hooks/useTrackMaster.ts
git commit -m @'
Phase 12.2: wire compression_density (3-band multiband compressor)

The final "(coming soon)" Advanced control becomes a real control plus
12 per-band override fields + 1 link-stereo toggle, all in one slice.

Backend (Rust):
- types.rs: 12 new Option<f32> per-band fields on AdvancedSettings
  (compression_{low,mid,high}_{threshold_db,ratio,attack_ms,release_ms})
  + compression_link_stereo: Option<bool>, all #[serde(default)].
  PlaybackTick gains gr_{low,mid,high}_db with -120 silence sentinel.
- dsp.rs:
  - BiquadCoeffs::butter_lp / butter_hp helpers.
  - LR4State + split_lr4_into_bands: 3-way LR4 split at 120 Hz / 4 kHz.
    Cascaded Butterworth (Q = sqrt(2)/2) sums flat across band edges.
  - EnvelopeFollower (peak detector, exp(-1/(tau*sr)) alphas).
  - ChainCoeffs grows 20+ compressor coefficient fields.
  - ChainCoeffs::from_settings: macro density 0..1 -> uniform threshold
    0 dBFS to -24 dBFS; per-band overrides replace the macro for that
    band only; per-band fixed musical defaults (low 2.5:1/30/300, mid
    2.0:1/15/150, high 1.8:1/5/80); auto makeup = (threshold_drop_db *
    (1 - 1/ratio)) / 2; soft knee 6 dB fixed. Identity early-return when
    macro off AND no overrides AND link not explicitly false.
  - MasteringChain gains GrSnapshotSlots (Arc<AtomicU32> trio) +
    new_with_gr_snapshots constructor. apply_multiband_compressor block
    inserted between presence_air and width in process_frame_inplace;
    legacy process_sample mirrors the same chain position.
- audio.rs: AudioThreadState gets 3 GR atomics; PlaybackSnapshot +
  PlaybackTick wired through; snapshot tick swaps atomics on 50 ms cadence.
- exports.rs: run_export_checks signature extended with Optional
  AnalysisResult + MasteringSettings; new comp_density_on_compressed_source
  advisory (DR < 6 LU AND density > 0.3 AND no per-band threshold overrides).
- 8 new dsp::tests + 2 new contracts.rs tests; closed-form math where
  possible, end-to-end render+LUFS check for the contract-level pin.

Frontend (TS/React):
- bindings.ts: 13 new AdvancedSettings fields, 3 new PlaybackTick fields.
- useTrackMaster.ts: DEFAULT_SETTINGS.advanced gets the 13 nulls; transport
  gains compressionGr.{low,mid,high} fed from the tick handler.
- App.tsx: drop "(coming soon)" on the Compression slider; new
  CompressionPerBandSubsection (collapsible details, 3 columns x 4
  NumberFields + link-stereo checkbox); 3 GrIndicator chips in StaleBar
  alongside ClippingIndicator.
- App.css: .gr-indicator styles parallel to .clip-indicator + per-band
  subsection grid.

Design backed by docs/superpowers/brainstorms/2026-05-12-compression-density-brainstorm.md
(Ozone/LANDR/CloudBounce multiband consensus per the research extract +
DHH/audio-mastering classics: LR4, peak detection, 120/4000 Hz split,
slow low/fast high time constants, makeup, soft knee, linked stereo).

Verification:
- cargo test --lib: 32/32 pass (was 24).
- cargo test (full): 71/71 pass (was 61).
- npm run build: clean (~258-261 KB raw / ~79-80 KB gzipped).
- Real-fixture tests unchanged (identity early-return preserves byte-
  equivalence when the slider is untouched).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
'@
git push origin master
```

Expected push output: a single new commit pushed to `master`. Confirm the SHA.

---

## Next slice (after this ships)

→ Typography pass and SVG preset icons are the next two Phase 12-tail slices. Begin with the typography pass: `docs/superpowers/plans/2026-05-12-typography-pass.md`. That plan is queued; when it lands, write its brainstorm first if one doesn't exist (existing precedent: `docs/superpowers/brainstorms/2026-05-12-compression-density-brainstorm.md`).

If Dan delivers listening notes from real material before the next slice starts, those override the queue.

---

## Self-Review Checklist (for the plan author)

After writing, the plan author checks:

1. **Brainstorm coverage** — every "Key Decisions Locked" entry from `2026-05-12-compression-density-brainstorm.md` mapped to a task?
   - Topology: 3-band, LR4, 120/4000 Hz, linked stereo default, peak detector, soft knee, no lookahead, auto makeup — Tasks 3, 5, 7.
   - Chain position (between presence_air and width): Task 7 Step 7.1, and mirrored for process_sample in Step 7.2.
   - Macro mapping (density 0..1 → 0..-24 dBFS uniform threshold; per-band fixed ratios/attack/release): Task 5 Step 5.2.
   - Per-band Option<f32> overrides + serde defaults: Task 1 Step 1.1.
   - Identity early-return: Task 5 Step 5.2 (`compression_active` flag) and Task 7 Step 7.1 (`if self.coeffs.compression_active`).
   - GR snapshot atomics (3 × AtomicU32, fetch_max, swap-and-reset, integer storage): Task 6 Step 6.2 + Task 7 Step 7.1 + Task 8 Steps 8.2 / 8.5.
   - Already-compressed source advisory: Task 9.
   - UI surface (label drop + per-band subsection + link checkbox + 3 GR readouts): Task 11.
2. **Locked decision count** — the 13 locked decisions from the user task each appear at least once with their exact constants:
   - LR4 @ 120 / 4000 Hz: constants `LR4_CROSSOVER_LOW_HZ = 120.0`, `LR4_CROSSOVER_HIGH_HZ = 4000.0` in Task 3 Step 3.2.
   - Position between presence_air and width: Task 7 Step 7.1.
   - Macro mapping `-24.0 * density`: Task 5 Step 5.2 (`let macro_threshold_db = -24.0 * density;`).
   - Per-band defaults (2.5/2.0/1.8 ratio, 30/15/5 attack, 300/150/80 release): Task 5 Step 5.2 (`const LOW_RATIO_DEFAULT: f32 = 2.5;` block).
   - 12 per-band override fields + serde(default): Task 1 Step 1.1.
   - `compression_link_stereo: Option<bool>` default true: Task 1 Step 1.1 + Task 5 Step 5.2 (`.unwrap_or(true)`).
   - Soft knee 6 dB fixed: Task 5 Step 5.2 (`let comp_knee_db = 6.0_f32;`).
   - Peak detector (|x|): Task 4 Step 4.1 + Task 7 Step 7.1 (`detector = bands[ch][b].abs()`).
   - No lookahead: not mentioned negatively in plan because no lookahead code is added — implicit.
   - Auto makeup `(threshold_drop_db * (1 - 1/ratio)) / 2`: Task 5 Step 5.2 (`makeup_db` closure).
   - Identity early-return guard: Task 5 Step 5.2 (`compression_active` derivation) + Task 7 Step 7.1 (`if self.coeffs.compression_active`).
   - GR snapshot integer fetch_max: Task 7 Step 7.1 (`to_u` + `fetch_max`); 50 ms tick + dB conversion in Task 8 Step 8.5.
   - Already-compressed source advisory: Task 9 Step 9.2 (`comp_density_on_compressed_source`).
3. **No placeholders** — search for "TBD", "TODO", "implement later", "fill in details". None present.
4. **Type consistency** — `compression_low_threshold_db` etc. spelled identically in `AdvancedSettings`, `ChainCoeffs`, `bindings.ts`, App.tsx, and tests. No drift.
5. **Test count math** — 8 new dsp tests + 2 new contracts tests = 10. lib goes 24→32. full goes 61→71. Matches the expected end state.

---

*Plan ready for execution.*
