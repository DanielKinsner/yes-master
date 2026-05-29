# Album Master Sample-Rate Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Album Master export at a chosen delivery sample rate + bit depth and accept tracks with differing source sample rates, by resampling every track to one common album rate.

**Architecture:** Add explicit `delivery_sample_rate` / `delivery_bit_depth` to `AlbumPlan` (serde-default, backward-compatible). Resolve one album format up front, then resample each track from its source rate to the album rate inside the existing album render loop — reusing the already-tested `sample_rate::convert_interleaved` and the shared landing helpers. Track Master's render path is NOT touched. A small "Delivery Format" control on the Album panel feeds the choice through `plan_album`.

**Tech Stack:** Rust (Tauri backend, `rubato` SRC via `convert_interleaved`, `hound` WAV, `ebur128`), TypeScript/React frontend, Vitest + `cargo test`.

**Design doc:** `docs/superpowers/specs/2026-05-29-album-master-sample-rate-parity-design.md`

**Branch:** `album-sample-rate-parity` (created in Task 0). Commit + push after each task; do NOT merge to `main` (user reviews first).

**Verification commands** (run from repo root unless noted; backend uses an isolated target dir to avoid locking a running app on Windows):

```powershell
npm test
npm run build
cd src-tauri
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
cargo fmt --check
cargo clippy --all-targets --target-dir target\codex-rc -- -D warnings
```

---

## File Structure

- `src-tauri/src/types.rs` — add two fields to `AlbumPlan` + Default + unit tests.
- `src-tauri/src/album.rs` — update the two `AlbumPlan` struct literals to default the new fields.
- `src-tauri/src/decode.rs` — add a cheap header-only `probe_sample_rate` helper.
- `src-tauri/src/album_render.rs` — pure `resolve_album_sample_rate` helper; resample wiring; remove the mixed-rate hard-fail; apply album format downstream.
- `src-tauri/src/engine.rs` — add the two optional fields to `PlanAlbumRequest` and stamp them onto the plan in `plan_album`. (Track Master render path untouched.)
- `src-tauri/tests/album_sample_rate.rs` — new integration tests (mirror `album_render.rs`).
- `src/bindings.ts` — add the two fields to the `AlbumPlan` interface.
- `src/lib/api.ts` — extend `planAlbum` with the two optional args.
- `src/hooks/useTrackMaster.ts` — album delivery-format state + plumbing into `planAlbum`.
- `src/components/AlbumPanel.tsx` — "Delivery Format" selects + receipt display + tests.
- `docs/RELEASE_STABILIZATION.md`, `docs/APP_BEHAVIOR.md` — note parity landed; channel-mismatch still deferred.

---

## Task 0: Create the working branch

- [ ] **Step 1: Branch off current `main` in the yes-master repo**

Run:
```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" checkout -b album-sample-rate-parity
```
Expected: `Switched to a new branch 'album-sample-rate-parity'`.

- [ ] **Step 2: Confirm clean baseline**

Run:
```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" status
```
Expected: on `album-sample-rate-parity`; only pre-existing untracked scratch (`.claude/`, `test-output/`) and the cosmetic `src-tauri/Cargo.toml` line-ending entry, if present. Do not stage those.

---

## Task 1: Add delivery fields to `AlbumPlan` (backward-compatible)

**Files:**
- Modify: `src-tauri/src/types.rs` (the `AlbumPlan` struct at ~line 434 and its `Default` impl at ~line 450)
- Test: `src-tauri/src/types.rs` (`#[cfg(test)]` module, add tests)

- [ ] **Step 1: Write the failing tests**

Add to the existing `#[cfg(test)] mod tests` in `types.rs`:

```rust
#[test]
fn album_plan_deserializes_without_delivery_fields_to_none() {
    // A v1 AlbumPlan JSON (pre-parity) has no delivery_* keys.
    let json = r#"{
        "title": "Legacy",
        "arc": { "kind": "preset", "preset": "cinematic" },
        "tracks": [],
        "transitions": [],
        "intensity": 1.0
    }"#;
    let plan: AlbumPlan = serde_json::from_str(json).expect("legacy AlbumPlan must deserialize");
    assert_eq!(plan.delivery_sample_rate, None);
    assert_eq!(plan.delivery_bit_depth, None);
}

#[test]
fn album_plan_roundtrips_explicit_delivery_fields() {
    let plan = AlbumPlan {
        delivery_sample_rate: Some(44_100),
        delivery_bit_depth: Some(16),
        ..AlbumPlan::default()
    };
    let json = serde_json::to_string(&plan).expect("serialize");
    let back: AlbumPlan = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.delivery_sample_rate, Some(44_100));
    assert_eq!(back.delivery_bit_depth, Some(16));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```powershell
cd "C:\Users\SM - Dan\Documents\GitHub\yes-master\src-tauri"
cargo test --lib --target-dir target\codex-rc album_plan_ 2>&1
```
Expected: compile error (no field `delivery_sample_rate` on `AlbumPlan`).

- [ ] **Step 3: Add the fields + update Default**

In `types.rs`, the `AlbumPlan` struct — add after `pub intensity: f32,`:

```rust
    /// Album-wide delivery sample rate in Hz. `None` = Auto (highest
    /// source rate among the album's tracks). `Some(44100|48000|96000)`
    /// converts every track to this rate. Added for Album SR parity;
    /// `serde(default)` keeps pre-parity saved projects deserializing.
    #[serde(default)]
    pub delivery_sample_rate: Option<u32>,
    /// Album-wide delivery bit depth. `None` = Auto (first track's
    /// `effective_bit_depth()`, the historical behavior).
    #[serde(default)]
    pub delivery_bit_depth: Option<u16>,
```

In the `Default for AlbumPlan` impl, add to the returned struct:

```rust
            delivery_sample_rate: None,
            delivery_bit_depth: None,
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc album_plan_ 2>&1
```
Expected: both new tests PASS. (Build will still fail elsewhere — the literals in `album.rs` — that's Task 2.)

- [ ] **Step 5: Commit (after Task 2 compiles — see note)**

The crate won't compile until `album.rs` literals are fixed (Task 2). Do Task 2 Step 1–2 now, then commit Task 1 + Task 2 together:

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src-tauri/src/types.rs src-tauri/src/album.rs
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): add serde-default delivery_sample_rate/bit_depth to AlbumPlan"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push -u origin album-sample-rate-parity
```

---

## Task 2: Fix `AlbumPlan` literals in `album.rs`

**Files:**
- Modify: `src-tauri/src/album.rs` (the n==0 early-return literal ~line 461 and the final literal ~line 494)

- [ ] **Step 1: Update both struct literals**

In `build_album_plan_with_names`, the early-return literal (`if n == 0`):

```rust
        return AlbumPlan {
            title,
            arc,
            tracks: Vec::new(),
            transitions: Vec::new(),
            intensity,
            delivery_sample_rate: None,
            delivery_bit_depth: None,
        };
```

And the final returned literal at the end of the function:

```rust
    AlbumPlan {
        title,
        arc,
        tracks,
        transitions,
        intensity,
        delivery_sample_rate: None,
        delivery_bit_depth: None,
    }
```

(Delivery is stamped later in `plan_album` from the request — the planner itself defaults to Auto.)

- [ ] **Step 2: Verify the crate compiles + lib tests pass**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc 2>&1
```
Expected: compiles; all lib tests PASS (including Task 1's two).

- [ ] **Step 3: Commit** — done jointly with Task 1 Step 5 above.

---

## Task 3: Add a header-only `probe_sample_rate` helper

**Files:**
- Modify: `src-tauri/src/decode.rs`
- Test: `src-tauri/src/decode.rs` (`#[cfg(test)]` module — add one if absent)

- [ ] **Step 1: Write the failing test**

Add a test module at the bottom of `decode.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn write_silence_wav(path: &Path, sample_rate: u32) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).expect("create");
        for _ in 0..1024 {
            w.write_sample(0_i16).expect("write");
        }
        w.finalize().expect("finalize");
    }

    #[test]
    fn probe_sample_rate_reads_header_rate() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("probe.wav");
        write_silence_wav(&p, 44_100);
        assert_eq!(probe_sample_rate(&p).expect("probe"), 44_100);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc probe_sample_rate 2>&1
```
Expected: compile error (`probe_sample_rate` not found).

- [ ] **Step 3: Implement the helper**

Add to `decode.rs` (mirrors `decode_full`'s probe prologue but stops before the decode loop):

```rust
/// Read just the container/codec header to learn the source sample rate
/// without decoding any audio. Used by the album render path to resolve
/// the Auto delivery rate (= highest source rate) cheaply, before any
/// track is processed.
pub fn probe_sample_rate(path: &Path) -> CommandResult<u32> {
    let file = std::fs::File::open(path).map_err(|e| CommandError::Io(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| CommandError::Decode(e.to_string()))?;
    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| CommandError::Decode("no decodable track".to_string()))?;
    Ok(track.codec_params.sample_rate.unwrap_or(44_100))
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc probe_sample_rate 2>&1
```
Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src-tauri/src/decode.rs
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(decode): add header-only probe_sample_rate helper"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 4: Pure `resolve_album_sample_rate` helper

**Files:**
- Modify: `src-tauri/src/album_render.rs`
- Test: `src-tauri/src/album_render.rs` (add a `#[cfg(test)]` module)

- [ ] **Step 1: Write the failing tests**

Add at the bottom of `album_render.rs`:

```rust
#[cfg(test)]
mod resolve_tests {
    use super::*;

    #[test]
    fn explicit_request_overrides_sources() {
        assert_eq!(resolve_album_sample_rate(Some(44_100), &[48_000, 96_000]), 44_100);
    }

    #[test]
    fn auto_picks_highest_source_rate() {
        assert_eq!(resolve_album_sample_rate(None, &[44_100, 48_000, 44_100]), 48_000);
    }

    #[test]
    fn auto_with_no_sources_falls_back_to_48k() {
        assert_eq!(resolve_album_sample_rate(None, &[]), 48_000);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc resolve_album 2>&1
```
Expected: compile error (`resolve_album_sample_rate` not found).

- [ ] **Step 3: Implement the helper**

Add near the top of `album_render.rs` (module-level fn):

```rust
/// Resolve the album-wide delivery sample rate. An explicit request wins;
/// otherwise Auto = the highest source rate among the tracks (quality-safe:
/// never forces a needless downsample, deterministic for mixed sources).
/// Empty source list falls back to 48 kHz.
fn resolve_album_sample_rate(requested: Option<u32>, source_rates: &[u32]) -> u32 {
    if let Some(rate) = requested {
        return rate;
    }
    source_rates.iter().copied().max().unwrap_or(48_000)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc resolve_album 2>&1
```
Expected: all three PASS.

- [ ] **Step 5: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src-tauri/src/album_render.rs
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): add resolve_album_sample_rate (Auto = highest source rate)"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 5: Wire resampling into the album render path

**Files:**
- Modify: `src-tauri/src/album_render.rs` (`render_album_plan_impl`, ~lines 93–327)
- Test: `src-tauri/tests/album_sample_rate.rs` (new)

This is the core change. It (a) resolves one album format up front, (b) removes the mixed-rate hard-fail, (c) resamples each track to the album rate, (d) writes everything downstream at the album format.

- [ ] **Step 1: Write the failing integration tests**

Create `src-tauri/tests/album_sample_rate.rs`. It mirrors `album_render.rs`'s harness (synthesize WAVs → `build_album_plan` → set delivery → `render_album_plan_impl` → assert headers + manifest):

```rust
//! Album Master sample-rate parity: mixed-source resampling + explicit
//! delivery rate/bit-depth honored end to end.

use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::PathBuf;
use tempfile::TempDir;
use yes_master_lib::album;
use yes_master_lib::album_render::render_album_plan_impl;
use yes_master_lib::engine::{AlbumPlanRenderRequest, AlbumTrackRenderInput};
use yes_master_lib::types::{
    AdvancedSettings, AlbumArc, AlbumArcKind, AnalysisResult, DeliveryProfile, InferenceConfidence,
    MasteringSettings, Preset, SpectralBalance, TrackCharacter, TrackId, TrackRole, ISO_PLACEHOLDER,
};

fn default_master_settings() -> MasteringSettings {
    MasteringSettings {
        preset: Preset::Universal,
        intensity: 0.5,
        eq_sub_db: 0.0,
        eq_low_db: 0.0,
        eq_low_mid_db: 0.0,
        eq_mid_db: 0.0,
        eq_high_mid_db: 0.0,
        eq_high_db: 0.0,
        eq_sparkle_db: 0.0,
        volume_match: false,
        source_lufs_integrated: None,
        input_gain_db: 0.0,
        output_gain_db: 0.0,
        delivery_profile: DeliveryProfile::Custom,
        album: None,
        advanced: AdvancedSettings::default(),
    }
}

fn fake_analysis(id: &str) -> AnalysisResult {
    AnalysisResult {
        track_id: TrackId(id.to_string()),
        lufs_integrated: -14.0,
        lufs_short_term_max: -10.0,
        true_peak_dbtp: -1.0,
        dynamic_range_lu: 8.0,
        spectral_balance: SpectralBalance { low: 0.33, mid: 0.34, high: 0.33 },
        transient_density: 0.5,
        stereo_width: 0.5,
        recommended_universal: default_master_settings(),
        measured_at_iso: ISO_PLACEHOLDER.to_string(),
        inferred_role: Some(TrackRole::AlbumTrack),
        role_confidence: Some(InferenceConfidence::Moderate),
        inferred_character: Some(TrackCharacter::Balanced),
        character_confidence: Some(InferenceConfidence::Moderate),
        spectral_balance_6band: None,
        transient_flux: Some(0.5),
        stereo_correlation: None,
        dynamic_range_p95_p10_db: None,
        lufs_short_term_max_3s: None,
        energy_density_score: Some(0.5),
    }
}

fn write_sine_mono(path: &PathBuf, sample_rate: u32, seconds: f32) {
    let spec = WavSpec { channels: 1, sample_rate, bits_per_sample: 16, sample_format: SampleFormat::Int };
    let mut w = WavWriter::create(path, spec).expect("create wav");
    let frames = (sample_rate as f32 * seconds) as usize;
    let omega = 2.0 * std::f32::consts::PI * 440.0 / sample_rate as f32;
    for i in 0..frames {
        let s = 0.3 * (omega * i as f32).sin();
        w.write_sample((s * 32767.0) as i16).expect("write");
    }
    w.finalize().expect("finalize");
}

/// Build a 2-track plan, optionally set explicit delivery, render, return the report.
fn render_two_track_album(
    tmp: &TempDir,
    rate_a: u32,
    rate_b: u32,
    delivery_sample_rate: Option<u32>,
    delivery_bit_depth: Option<u16>,
) -> yes_master_lib::engine::AlbumRenderReport {
    let a = tmp.path().join("a.wav");
    let b = tmp.path().join("b.wav");
    write_sine_mono(&a, rate_a, 1.0);
    write_sine_mono(&b, rate_b, 1.0);

    let analyses = [fake_analysis("a"), fake_analysis("b")];
    let refs: Vec<&AnalysisResult> = analyses.iter().collect();
    let mut plan = album::build_album_plan(
        "Parity".to_string(),
        &refs,
        &[1.0, 1.0],
        AlbumArc::Preset { preset: AlbumArcKind::Cinematic },
        1.0,
    );
    plan.delivery_sample_rate = delivery_sample_rate;
    plan.delivery_bit_depth = delivery_bit_depth;

    let request = AlbumPlanRenderRequest {
        plan,
        tracks: vec![
            AlbumTrackRenderInput { track_id: TrackId("a".into()), source_path: a.to_string_lossy().into(), settings: default_master_settings() },
            AlbumTrackRenderInput { track_id: TrackId("b".into()), source_path: b.to_string_lossy().into(), settings: default_master_settings() },
        ],
    };
    let out_dir = tmp.path().join("out");
    render_album_plan_impl(&request, &out_dir, None).expect("render")
}

#[test]
fn mixed_source_rates_resample_to_common_album_rate() {
    // 44.1 kHz + 48 kHz sources, explicit 48 kHz delivery — must NOT error,
    // and every output WAV must be 48 kHz.
    let tmp = TempDir::new().expect("tempdir");
    let report = render_two_track_album(&tmp, 44_100, 48_000, Some(48_000), None);
    for rec in &report.tracks {
        let spec = hound::WavReader::open(&rec.output_path).expect("open track").spec();
        assert_eq!(spec.sample_rate, 48_000, "per-track WAV must be 48 kHz");
    }
    let album_spec = hound::WavReader::open(&report.album_wav_path).expect("open album").spec();
    assert_eq!(album_spec.sample_rate, 48_000, "album.wav must be 48 kHz");
}

#[test]
fn explicit_cd_delivery_downsamples_48k_sources_to_44100_16bit() {
    let tmp = TempDir::new().expect("tempdir");
    let report = render_two_track_album(&tmp, 48_000, 48_000, Some(44_100), Some(16));
    let album_spec = hound::WavReader::open(&report.album_wav_path).expect("open album").spec();
    assert_eq!(album_spec.sample_rate, 44_100);
    assert_eq!(album_spec.bits_per_sample, 16);
    let manifest = std::fs::read_to_string(&report.manifest_path).expect("manifest");
    let parsed: serde_json::Value = serde_json::from_str(&manifest).expect("json");
    assert_eq!(parsed["sample_rate"], 44_100);
    assert_eq!(parsed["bit_depth"], 16);
}

#[test]
fn auto_delivery_picks_highest_source_rate() {
    // No explicit delivery; sources are 44.1 + 48 → album should be 48 kHz.
    let tmp = TempDir::new().expect("tempdir");
    let report = render_two_track_album(&tmp, 44_100, 48_000, None, None);
    let album_spec = hound::WavReader::open(&report.album_wav_path).expect("open album").spec();
    assert_eq!(album_spec.sample_rate, 48_000);
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run:
```powershell
cargo test --target-dir target\codex-rc --test album_sample_rate 2>&1
```
Expected: FAIL — `mixed_source_rates_*` and `auto_*` fail with the current hard error `"album sample-rate mismatch ... (resampling not yet supported)"`; the CD test fails on a 48 kHz album header.

- [ ] **Step 3: Implement the render-path changes**

In `album_render.rs::render_album_plan_impl`:

**(a) Add the import** at the top (alongside the existing `use crate::...` lines):

```rust
use crate::sample_rate::convert_interleaved;
```

**(b) Resolve the album format up front.** Replace the current `bit_depth` block (the `let bit_depth = request.plan.tracks.first()...unwrap_or(24);` at ~line 108–114) with:

```rust
    // Resolve the single album delivery format BEFORE processing any track.
    // Sample rate: explicit request wins, else Auto = highest source rate
    // (probed cheaply from headers). Bit depth: explicit request wins, else
    // the historical first-track effective_bit_depth().
    let mut source_rates: Vec<u32> = Vec::with_capacity(request.plan.tracks.len());
    for t in &request.plan.tracks {
        if let Some(input) = settings_by_id.get(t.track_id.as_str()) {
            let probed = crate::decode::probe_sample_rate(Path::new(&input.source_path))?;
            source_rates.push(probed);
        }
    }
    let album_sample_rate =
        resolve_album_sample_rate(request.plan.delivery_sample_rate, &source_rates);
    let bit_depth = request.plan.delivery_bit_depth.unwrap_or_else(|| {
        request
            .plan
            .tracks
            .first()
            .and_then(|t| settings_by_id.get(t.track_id.as_str()))
            .map(|input| input.settings.effective_bit_depth())
            .unwrap_or(24)
    });
```

**(c) Remove the mixed-rate hard-fail and resample instead.** In the per-track loop, the current block (~lines 163–176) tracks `common_sr`/`common_channels` and errors on any sample-rate mismatch. Replace it so only the **channel** mismatch errors (sample-rate differences are now resampled):

```rust
        if i == 0 {
            common_channels = pcm.channels.max(1);
        } else if pcm.channels != common_channels {
            return Err(CommandError::Other(format!(
                "album channel mismatch on {}: {} ch vs album {} ch",
                input.source_path, pcm.channels, common_channels
            )));
        }
```

(Delete the `common_sr` assignment and its mismatch `else if` branch entirely. `common_sr` is no longer used — `album_sample_rate` replaces it everywhere below.)

**(d) Resample each track after its DSP chain, before landing.** The existing code already has `let mut samples = pcm.samples;` before the chunked `chain.process_interleaved` loop. Immediately AFTER that loop and BEFORE `measure_and_apply_ceiling_bounded_landing` (~line 240), insert a plain reassignment (NOT a new `let` — that would shadow the buffer the chain just wrote):

```rust
        // Resample this track from its source rate to the album delivery
        // rate. Ordering mirrors Track Master: chain → SRC → measure → land.
        // `convert_interleaved` would copy even on a match, so guard it to
        // avoid a needless full-buffer clone on already-matching tracks.
        if pcm.sample_rate != album_sample_rate {
            samples = convert_interleaved(&samples, pcm.sample_rate, album_sample_rate, pcm.channels)?;
        }
```

Then change the landing + per-track WAV write + LUFS measure that follow to use `album_sample_rate` instead of `pcm.sample_rate`, and `bit_depth` (already resolved). Specifically:

```rust
        measure_and_apply_ceiling_bounded_landing(
            &mut samples,
            album_sample_rate,
            pcm.channels,
            &shadowed,
        )?;
        // ... per_track_path computed as before ...
        write_wav(&per_track_path, &samples, album_sample_rate, pcm.channels, bit_depth)?;
        let measured_lufs = measure_integrated_lufs(&samples, album_sample_rate, pcm.channels)?;
```

> Note: `samples` is now declared `let mut samples = ...` by the resample step above, so remove the earlier `let mut samples = pcm.samples;` shadowing conflict — keep a single binding. The chain loop runs on `pcm.samples` first; restructure as: `let mut samples = pcm.samples;` (before the chain loop, as today) then the resample step reassigns `samples = convert_interleaved(&samples, ...)?` (drop the inner `let`). Use a plain reassignment, not a new `let`, to avoid shadowing the buffer the chain wrote.

**(e) Pass 2 + manifest use the album format.** Replace remaining `common_sr` uses (~lines 281, 291, 309):

```rust
    let spec = wav_spec(common_channels, album_sample_rate, bit_depth)?;
    // ... gap math:
    let gap_frames = (gap_seconds * album_sample_rate as f32) as usize;
    // ... manifest:
        sample_rate: album_sample_rate,
```

Remove the now-unused `let mut common_sr: u32 = 0;` declaration.

- [ ] **Step 4: Run the new tests + the existing album tests to verify green**

Run:
```powershell
cargo test --target-dir target\codex-rc --test album_sample_rate 2>&1
cargo test --target-dir target\codex-rc --test album_render 2>&1
```
Expected: all PASS — including the pre-existing `album_render` smoke tests (uniform 48 kHz sources still produce a 48 kHz album, unchanged behavior).

- [ ] **Step 5: Full backend gate**

Run:
```powershell
cargo test --target-dir target\codex-rc 2>&1
cargo fmt --check
cargo clippy --all-targets --target-dir target\codex-rc -- -D warnings
```
Expected: all green. Fix any clippy/fmt issues before committing.

- [ ] **Step 6: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src-tauri/src/album_render.rs src-tauri/tests/album_sample_rate.rs
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): resample tracks to one delivery rate; drop mixed-rate hard-fail"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 6: Plumb delivery format through `plan_album`

**Files:**
- Modify: `src-tauri/src/engine.rs` (`PlanAlbumRequest` ~line 352, `plan_album` ~line 364)
- Test: covered by frontend + the existing `album_sample_rate` tests (the command wrapper is a thin pass-through)

- [ ] **Step 1: Extend `PlanAlbumRequest`**

Add the two optional fields (serde-default so the current frontend payload still deserializes until the frontend is updated in Task 8):

```rust
#[derive(Debug, Deserialize)]
pub struct PlanAlbumRequest {
    pub title: String,
    pub analyses: Vec<AnalysisResult>,
    pub durations: Vec<f64>,
    pub arc: AlbumArc,
    pub intensity: f32,
    #[serde(default)]
    pub delivery_sample_rate: Option<u32>,
    #[serde(default)]
    pub delivery_bit_depth: Option<u16>,
}
```

- [ ] **Step 2: Stamp them onto the built plan in `plan_album`**

```rust
#[tauri::command]
pub async fn plan_album(request: PlanAlbumRequest) -> CommandResult<AlbumPlan> {
    let refs: Vec<&AnalysisResult> = request.analyses.iter().collect();
    let mut plan = crate::album::build_album_plan(
        request.title,
        &refs,
        &request.durations,
        request.arc,
        request.intensity,
    );
    plan.delivery_sample_rate = request.delivery_sample_rate;
    plan.delivery_bit_depth = request.delivery_bit_depth;
    Ok(plan)
}
```

- [ ] **Step 3: Verify compile + tests**

Run:
```powershell
cargo test --lib --target-dir target\codex-rc 2>&1
cargo clippy --all-targets --target-dir target\codex-rc -- -D warnings
```
Expected: green.

- [ ] **Step 4: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src-tauri/src/engine.rs
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): accept delivery format in plan_album request"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 7: Frontend types + API

**Files:**
- Modify: `src/bindings.ts` (`AlbumPlan` interface ~line 175)
- Modify: `src/lib/api.ts` (`planAlbum` ~line 172)

- [ ] **Step 1: Add fields to the `AlbumPlan` TS interface**

```ts
export interface AlbumPlan {
  title: string;
  arc: AlbumArc;
  tracks: AlbumTrackEntry[];
  transitions: TransitionSpec[];
  intensity: number;
  delivery_sample_rate?: number | null;
  delivery_bit_depth?: number | null;
}
```

- [ ] **Step 2: Extend `planAlbum` to forward the choice**

```ts
  planAlbum: (
    title: string,
    analyses: AnalysisResult[],
    durations: number[],
    arc: AlbumArc,
    intensity: number,
    deliverySampleRate?: number | null,
    deliveryBitDepth?: number | null,
  ) =>
    invoke<AlbumPlan>("plan_album", {
      request: {
        title,
        analyses,
        durations,
        arc,
        intensity,
        delivery_sample_rate: deliverySampleRate ?? null,
        delivery_bit_depth: deliveryBitDepth ?? null,
      },
    }),
```

- [ ] **Step 3: Typecheck**

Run:
```powershell
cd "C:\Users\SM - Dan\Documents\GitHub\yes-master"
npm run build 2>&1
```
Expected: build succeeds (no new callers broken — `planAlbum`'s new args are optional).

- [ ] **Step 4: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src/bindings.ts src/lib/api.ts
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): frontend AlbumPlan delivery fields + planAlbum args"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 8: Album delivery-format state in the hook

**Files:**
- Modify: `src/hooks/useTrackMaster.ts` (album state block ~line 1047; `exportAlbumPlan` ~line 1073; return object ~line 1844)

- [ ] **Step 1: Add state mirroring `albumIntensity`**

After the `albumIntensity` state declarations (~line 1053):

```ts
  // Album delivery format. `null` = Auto (backend resolves: rate = highest
  // source rate, bit depth = first-track effective). Not persisted in track
  // settings — rebuilt at export like arc/intensity.
  const [albumSampleRate, setAlbumSampleRateState] = useState<number | null>(null);
  const [albumBitDepth, setAlbumBitDepthState] = useState<number | null>(null);
  const setAlbumSampleRate = useCallback(
    (v: number | null) => setAlbumSampleRateState(v),
    [],
  );
  const setAlbumBitDepth = useCallback(
    (v: number | null) => setAlbumBitDepthState(v),
    [],
  );
```

- [ ] **Step 2: Pass them into `planAlbum`**

In `exportAlbumPlan`, update the `api.planAlbum(...)` call:

```ts
      const plan = await api.planAlbum(
        title,
        analyses,
        durations,
        arc,
        albumIntensity,
        albumSampleRate,
        albumBitDepth,
      );
```

Add `albumSampleRate, albumBitDepth` to the `useCallback` dependency array for `exportAlbumPlan`.

- [ ] **Step 3: Export the new state + setters from the hook**

Add to the hook's returned object (~line 1844, near `albumIntensity`):

```ts
    albumSampleRate,
    albumBitDepth,
    setAlbumSampleRate,
    setAlbumBitDepth,
```

- [ ] **Step 4: Typecheck**

Run:
```powershell
npm run build 2>&1
```
Expected: success.

- [ ] **Step 5: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src/hooks/useTrackMaster.ts
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): hold album delivery-format choice in useTrackMaster"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 9: "Delivery Format" controls + receipt on the Album panel

**Files:**
- Modify: `src/components/AlbumPanel.tsx`
- Modify: `src/App.tsx` (where `<AlbumPanel .../>` is rendered — thread the new props)
- Test: `src/components/AlbumPanel.test.tsx` (create if absent, else add cases)

- [ ] **Step 1: Write the failing component test**

Create/extend `src/components/AlbumPanel.test.tsx`:

```tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { AlbumPanel } from "./AlbumPanel";

const baseProps = {
  tracks: [],
  albumArcKind: "cinematic" as const,
  albumIntensity: 1.0,
  albumTitle: "",
  albumRendering: false,
  albumExportReport: null,
  albumSampleRate: null,
  albumBitDepth: null,
  onAlbumArc: vi.fn(),
  onAlbumIntensity: vi.fn(),
  onAlbumTitle: vi.fn(),
  onExportAlbum: vi.fn(),
  onAlbumSampleRate: vi.fn(),
  onAlbumBitDepth: vi.fn(),
};

describe("AlbumPanel delivery format", () => {
  it("renders sample-rate and bit-depth selects defaulting to Auto", () => {
    render(<AlbumPanel {...baseProps} />);
    expect(screen.getByLabelText(/sample rate/i)).toHaveValue("auto");
    expect(screen.getByLabelText(/bit depth/i)).toHaveValue("auto");
  });

  it("calls onAlbumSampleRate with a number when a rate is picked", () => {
    render(<AlbumPanel {...baseProps} />);
    fireEvent.change(screen.getByLabelText(/sample rate/i), { target: { value: "44100" } });
    expect(baseProps.onAlbumSampleRate).toHaveBeenCalledWith(44100);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```powershell
npm test -- AlbumPanel 2>&1
```
Expected: FAIL (no such selects / props).

- [ ] **Step 3: Add the controls to `AlbumPanel.tsx`**

Extend the props type:

```tsx
type AlbumPanelProps = {
  tracks: ImportedTrack[];
  albumArcKind: AlbumArcKind;
  albumIntensity: number;
  albumTitle: string;
  albumRendering: boolean;
  albumExportReport: AlbumRenderReport | null;
  albumSampleRate: number | null;
  albumBitDepth: number | null;
  onAlbumArc: (kind: AlbumArcKind) => void;
  onAlbumIntensity: (v: number) => void;
  onAlbumTitle: (v: string) => void;
  onExportAlbum: () => void;
  onAlbumSampleRate: (v: number | null) => void;
  onAlbumBitDepth: (v: number | null) => void;
};
```

Destructure the four new props in the component signature. Then add this control group inside `<div className="album-panel-controls">`, after the intensity control:

```tsx
        <label className="adv-label" htmlFor="album-rate-select">
          Sample rate
        </label>
        <select
          id="album-rate-select"
          className="loudness-profile-select"
          value={albumSampleRate === null ? "auto" : String(albumSampleRate)}
          onChange={(e) =>
            onAlbumSampleRate(e.target.value === "auto" ? null : parseInt(e.target.value, 10))
          }
        >
          <option value="auto">Auto</option>
          <option value="44100">44.1 kHz</option>
          <option value="48000">48 kHz</option>
          <option value="96000">96 kHz</option>
        </select>
        <label className="adv-label" htmlFor="album-depth-select">
          Bit depth
        </label>
        <select
          id="album-depth-select"
          className="loudness-profile-select"
          value={albumBitDepth === null ? "auto" : String(albumBitDepth)}
          onChange={(e) =>
            onAlbumBitDepth(e.target.value === "auto" ? null : parseInt(e.target.value, 10))
          }
        >
          <option value="auto">Auto</option>
          <option value="16">16-bit</option>
          <option value="24">24-bit</option>
          <option value="32">32-bit</option>
        </select>
```

Also add the rendered format to the export receipt (inside the `albumExportReport` block, in `album-export-receipt-meta`):

```tsx
          <span className="album-export-receipt-meta">
            {albumExportReport.tracks.length} tracks · manifest:{" "}
            {albumExportReport.manifest_path}
          </span>
```

(Manifest path already shows; rendered rate/depth live in the manifest JSON. Optional: if `AlbumRenderReport` is later extended with the rendered format, surface it here. Not required for this task — out of scope, no fabricated fields.)

- [ ] **Step 4: Thread props in `App.tsx`**

Find the `<AlbumPanel ... />` render site and pass the new props from the hook:

```tsx
        albumSampleRate={albumSampleRate}
        albumBitDepth={albumBitDepth}
        onAlbumSampleRate={setAlbumSampleRate}
        onAlbumBitDepth={setAlbumBitDepth}
```

(destructure `albumSampleRate, albumBitDepth, setAlbumSampleRate, setAlbumBitDepth` from the `useTrackMaster()` hook result alongside the existing album fields.)

- [ ] **Step 5: Run tests + build**

Run:
```powershell
npm test -- AlbumPanel 2>&1
npm test 2>&1
npm run build 2>&1
```
Expected: AlbumPanel tests PASS; full suite green; build succeeds.

- [ ] **Step 6: Commit**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add src/components/AlbumPanel.tsx src/components/AlbumPanel.test.tsx src/App.tsx
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "feat(album): Delivery Format (rate + bit depth) controls on Album panel"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

---

## Task 10: Docs + full verification

**Files:**
- Modify: `docs/RELEASE_STABILIZATION.md`, `docs/APP_BEHAVIOR.md`

- [ ] **Step 1: Update docs**

In `docs/APP_BEHAVIOR.md`, where Album Master's same-sample-rate limitation is described, replace it with: Album Master now resamples each track to one album delivery rate (chosen via the Album panel's Delivery Format control; Auto = highest source rate), honoring delivery sample rate + bit depth. Channel-count mismatch (mono vs stereo) remains a hard error (deferred).

In `docs/RELEASE_STABILIZATION.md`, under Deferred, remove "full Album Master delivery parity" wording for sample rate, and note channel-mismatch parity remains the deferred item.

- [ ] **Step 2: Run the full gate**

Run:
```powershell
cd "C:\Users\SM - Dan\Documents\GitHub\yes-master"
npm test
npm run build
cd src-tauri
cargo test --target-dir target\codex-rc
cargo fmt --check
cargo clippy --all-targets --target-dir target\codex-rc -- -D warnings
```
Expected: all green.

- [ ] **Step 3: Commit + push**

```powershell
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" add docs/APP_BEHAVIOR.md docs/RELEASE_STABILIZATION.md
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" commit -m "docs(album): record sample-rate parity; channel parity still deferred"
git -C "C:\Users\SM - Dan\Documents\GitHub\yes-master" push
```

- [ ] **Step 4: Hand back to the user**

Summarize what landed, the branch name, and that it's ready for their review before merging to `main`. Do NOT merge.

---

## Deferred within this plan (spec items intentionally not built here)

The design's "Error Handling & Honesty" section mentioned two extras. Both are
deliberately held back to keep this change focused and avoid widening data models;
flagged to the user for an explicit "now vs later" call:

- **Upsample advisory.** A gentle "this meets delivery specs but doesn't restore
  detail lost to lossy sources" note when the album rate exceeds all source rates.
  Surfacing it cleanly needs the rendered-vs-source rates in `AlbumRenderReport`
  (currently absent). Fast-follow.
- **Requested-vs-rendered integrity check.** The render already writes exactly the
  resolved `album_sample_rate`, so a mismatch isn't reachable through normal flow;
  a defensive assertion is low value until the report model carries the requested
  rate. Fast-follow.

Track Master's existing upsample copy/advisory is unaffected — it is not touched.

## Manual verification (user, before merge)

1. Import two tracks with different source rates (e.g. one 44.1 kHz, one 48 kHz) into Album Master.
2. Confirm export no longer errors with "resampling not yet supported".
3. Set Delivery Format = 44.1 kHz / 16-bit, export, and confirm the written `album.wav` and per-track WAVs report 44.1 kHz / 16-bit (e.g. via file properties or a quick `hound`/ffprobe check) and play back correctly.
4. Set Delivery Format = Auto with mixed sources, confirm the album renders at the higher rate.
5. Listen: confirm no obvious artifacts from the resample.
</content>
