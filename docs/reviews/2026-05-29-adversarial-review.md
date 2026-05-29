# Adversarial Read-Only Review — YES Master

- **Date:** 2026-05-29
- **Scope:** Whole repo, read-only. No code modified.
- **Method:** Grounded in CLAUDE.md + docs/{ARCHITECTURE,APP_BEHAVIOR,TESTING}.md, then verified every claim against code. Ran the full verification gate. Fanned out four parallel read-only subagents (DSP, dead-code, wiring, CSS/UX); **every reported finding below was independently re-verified by hand** — several subagent findings were downgraded or dropped (noted inline).
- **Reviewer stance:** Precision over recall. Genuine correctness bugs are separated from hygiene/maintainability nits. Hunches are labelled.

---

## TL;DR (the "so what")

**This is a healthy, unusually well-tested codebase.** The verification gate is fully green, and the parts that would be catastrophic if wrong — the BS.1770 loudness math, the true-peak limiter ceiling, the EQ filter coefficients, the sample-rate conversion, the WAV dither/quantization — are **correct and conformant**. I found **no P0 or P1 bug that corrupts audio or measurements in the live (production) path.**

The real findings are a tier down: **maintainability and drift risk**, a few **sub-dB DSP voicing nuances**, **type/display mirrors that can rot into bugs**, and **accessibility debt**. The single highest-value item is that the **compressor calibration table is duplicated by hand in Rust and TypeScript** and will silently drift the next time presets are retuned (which is in the active queue).

Headline counts: **0 × P0, 0 × P1** (after right-sizing subagent severities), **~3 × P2**, **~20 × P3**.

> Note on subagent severities: the DSP and dead-code agents each filed two "P1"s. On inspection, **all four were over-escalated** (a sliding-window limiter that actually does cover its peaks; a true-peak estimator that is effectively 4× oversampling; two currently-in-sync duplicate tables). They are recorded below at their real severity with the reasoning.

---

## 1. Verification Gate — REAL results

Run exactly as requested. All green. (`target\codex-rc` was already warm, so the cargo run was fast.)

| Command | Result |
|---|---|
| `cargo test --target-dir target\codex-rc` | **PASS** — 208 lib + 71 integration/contract/doc tests, **0 failed**. (2 `dump_*` diagnostics are `ignored` by design.) |
| `cargo fmt --check` | **PASS** — clean (exit 0) |
| `cargo clippy --all-targets --target-dir target\codex-rc -- -D warnings` | **PASS** — **no warnings** (exit 0) |
| `npm test` (`vitest run`) | **PASS** — 149 tests across 18 files |
| `npm run build` (`tsc -b && vite build`) | **PASS** — 62 modules, type-check clean |

There is nothing to fix in the gate itself. The release-gate cleanup item in CLAUDE.md (Rust formatting decision + Clippy install/gate) appears **already satisfied**: `cargo fmt --check` and `clippy -D warnings` both pass locally.

---

## 2. DSP Correctness (highest-priority dimension)

### 2a. Verified CORRECT (so coverage is on record)

Each of these was checked against the reference and/or worked numerically; all are right:

- **BS.1770-4 K-weighting** (`dsp.rs:139-192`): coefficients are computed with the bilinear transform `k = tan(π·f0/fs)` and are therefore **sample-rate-correct at 44.1k/48k/96k** — *not* hardcoded for 48k (a common bug; this code avoids it). The RLB stage correctly keeps `b = [1, −2, 1]` un-normalized, matching the published Annex-1 reference (`a1 ≈ −1.99004745`, `a2 ≈ 0.99007225` at 48k).
- **Momentary + Integrated LUFS** (`dsp.rs:1573-1824`): 400 ms blocks, 100 ms step (75% overlap), −70 LUFS absolute gate, −10 LU relative gate computed from the abs-gated mean, mean-of-energies (not mean-of-dB), −0.691 offset, L=R=1.0 weights. Conformant.
- **RBJ biquads** (`dsp.rs:29-217`): peaking / low-shelf / high-shelf / Butterworth HP+LP all match the cookbook including `a0` normalization and the shelf `2√A·α` term.
- **Ceiling-bounded LUFS landing** (`engine.rs:106-211`): provably never overshoots the ceiling — upward gain is `min(target−measured, ceiling−TP)`, so post-gain TP lands *at* the ceiling, never above; downward applies fully. Relies (correctly) on the exact identity that a uniform linear gain shifts integrated-LUFS and true-peak by the same `20·log10(g)`.
- **Authoritative true-peak / LUFS** for analysis, landing, and the export receipt all use the external **`ebur128` crate** (`analysis.rs:52`, `engine.rs:180`, `audio.rs:912`) — a real polyphase TP meter. The hand-rolled meters in `dsp.rs` drive only the realtime UI (`sources.rs`) and are test-verified conformant.
- **Sample-rate conversion** (`sample_rate.rs`): rubato `Fft::new(source, target, …)` direction is correct; output buffer sized via `process_all_needed_output_len` then truncated; test pins 44.1k→48k at exactly 4800 frames / 0.1 s.
- **Limiter ceiling enforcement** (`dsp.rs:1318-1404`): lookahead + Lagrange-4 inter-sample scan; instant attack, exponential release; the ceiling tests pass for the right reasons.
- **M/S width** (`dsp.rs:1203-1213`): lossless, exactly identity at scale 1.0.
- **WAV quantization/dither** (`wav_writer.rs:76-93`): TPDF (sum of two uniforms, ±2 LSB) added pre-quantization; **symmetric scaling fixed** (`INT16_SCALE = 32768`, clamp to `32767`) so negatives reach `i16::MIN` with **no off-by-one**; 32-bit float passes through. Byte-hash pinned.
- **NaN/Inf & empty-buffer guards**: `sanitize_lufs`, landing non-finite guards, ring-sum `.max(0.0)` drift clamp, width `len < 2` guard, analysis empty/short guards. No panics found on degenerate input.

### 2b. Real but low-severity findings

**[P3] Soft-knee compressor zeroes the lower half of the knee**
- **Where:** `dsp.rs:2109-2118` (and duplicated at `dsp.rs:2291-2300`).
- **Why:** In the knee region the gain reduction is `t² · above.max(0.0)` where `above = (env_db − thr)·(1 − 1/ratio)`. Because `above` is negative below threshold, `.max(0.0)` makes GR **exactly 0** from `thr − knee/2` through `thr`. The textbook quadratic knee (Reiss/McPherson, Giannoulis) gives a small *positive* GR at the threshold center (`(1−1/R)·W/8`). So the implemented "soft 6 dB knee" only softens the *upper* half; it under-compresses just above threshold and is not the centered curve the comment implies.
- **Impact:** Objective deviation, but sub-dB and on a creative/preset compressor — a voicing nuance, not output corruption. It is *consistent* across both code paths and pinned by the byte-identity SHA tests (so it ships deterministically).
- **Fix:** Use the canonical form `gr_db = (1 − 1/ratio) · (env_db − thr + knee/2)² / (2·knee)` inside the knee branch; factor the two copies into one helper (see 4d).
- **Confidence:** High that it deviates; Medium that it is audible. *(Taste-dependent — capture a listening note before any preset recalibration, per CLAUDE.md working style.)*

**[P3] Limiter inter-sample estimator uses a Lagrange-4 kernel, not the BS.1770 polyphase FIR** *(subagent rated P1 — downgraded)*
- **Where:** `dsp.rs:1372` + `LAGRANGE_INTERSAMPLE_COEFFS`.
- **Why:** It evaluates inter-sample points at x ∈ {0.25, 0.5, 0.75} — i.e. it *is* effectively 4× oversampling, just with a cubic-Lagrange kernel instead of the standard windowed-sinc FIR. The kernel is slightly optimistic (≈0.1–0.3 dB on worst-case full-band transients), so the limiter's internal TP estimate can read marginally low.
- **Why it's only P3:** the **authoritative export TP check uses `ebur128`** (a real TP meter) and warns the user if the rendered master exceeds the ceiling — so any small leak is always surfaced. The code comment is candid about the approximation.
- **Fix (optional):** swap in a ≥4× polyphase FIR if strict TP conformance is ever required, or trim the internal ceiling by the kernel's worst-case error.
- **Confidence:** High (it is not the sinc FIR); impact is small with a backstop.

**[P3] 6-band spectral-balance edges misorder below ~13 kHz Nyquist**
- **Where:** `analysis.rs:360-361` — `top = min(Nyquist, 16000)`; `edges = [20, 80, 250, 800, 2500, 6500, top]`.
- **Why:** for any source under ~13 kHz sample rate, `top < 6500`, so the air-band test `freq ≥ 6500 && freq < top` can never fire and all energy ≥ 6500 Hz is dropped (`continue`). Latent: real music is ≥44.1k, and this feeds only **role/character inference**, not audio.
- **Fix:** early-return `None` (or clamp/sort edges) when `top ≤ edges[5]`.
- **Confidence:** High mechanically; near-zero real-world impact.

**[P3] Dynamic-range percentile is truncated nearest-rank**
- **Where:** `analysis.rs:471-472` — `rms_db[(len*10)/100]` / `rms_db[((len*95)/100).min(len-1)]`.
- **Why:** integer truncation biases toward lower percentiles for some `len`. Feeds inference only; cosmetic.
- **Fix:** `((len-1) as f32 * 0.95).round() as usize`.
- **Confidence:** High; negligible impact.

### 2c. Explicitly DISPROVEN (so they aren't re-raised)

- **Limiter ISP loop "skips boundary pairs" (subagent P1) — NOT a real leak.** The buffer is a sliding ring: every physical adjacent sample-pair passes through interior positions `1..frames-3` (which *are* scanned) several frames before it reaches the output at position 0, and instant-attack gain reduction then holds while the peak remains in-buffer. Only the absolute first/last frames of the whole signal are uncovered (silence/fade). Verified by tracing the ring indexing.
- **K-weighting wrong off-48k — disproven** (it's bilinear-transform / sample-rate-aware; see 2a).
- **`process_sample` low_mid skip is a live EQ bug — disproven** (it has no production caller; see 4b).

---

## 3. Wiring Gaps (UI control → state → command → output)

Overall the wiring is **tight**: `ChainCoeffs::from_settings` genuinely reads every `advanced.*` field, and the value flow from controls → `MasteringSettings` → `invoke` → chain is intact. `transient_amount`, compressor mode/density, per-band overrides, width, warmth, presence_air, input/output gain, ceiling, delivery profile, and all seven EQ bands are all live. Findings are type/display drift, not silent dead controls.

**[P3] `AlbumTrackEntry` TS type omits `album_character` (and `AlbumCharacter` has no TS type at all)**
- **Where:** Rust `types.rs:430` defines `album_character: Option<AlbumCharacter>`; TS `bindings.ts:166-173` lacks it. `grep AlbumCharacter src/` → 0 hits.
- **Why it matters:** `bindings.ts` is **hand-written** ("Phase 1.2 will replace this with tauri-specta"). The value survives today only because JS preserves unknown JSON keys round-tripping through `render_album_plan`; the TS type is lying about the shape and will mislead the first UI that tries to read/edit per-track character.
- **Fix:** add `AlbumCharacter` union + `album_character?` to the TS type (or land the tauri-specta codegen).
- **Confidence:** High.

**[P3] Track-mode Width can't reach the full DSP range; `SignalChain` shows the wrong Spatial width**
- **Where:** Width `NumberField` is `max={1.5}` (`App.tsx:2096-2103`) but the chain clamps to `[0, 2]` (`dsp.rs:859-863`) and album bias pushes to 2.0. Separately, `SignalChain.presetDefaultWidth` returns `1.3` for Spatial and `1.0` for everything else (`SignalChain.tsx:45-52`), but the real baselines are Spatial `1.16`, Oomph `0.95`, Tape `0.99`, Warmth `0.98` (`dsp.rs`).
- **Why:** range is unreachable in Track mode; the "neutral/width" stage indicator is computed from a **stale display-mirror constant** and is wrong for ~4 presets. (Another instance of the duplication theme — see §4a.)
- **Fix:** raise the field `max` to 2.0; replace `presetDefaultWidth` with the real per-preset table.
- **Confidence:** High.

**[P3] Saved user presets capture session-transient `volume_match` / `source_lufs_integrated`**
- **Where:** snapshot at `useTrackMaster.ts:1596` → `saveUserPreset`.
- **Why:** a preset saved while Volume Match was on carries `volume_match: true` and a track-specific `source_lufs_integrated` into other tracks. Live playback force-overrides VM and the landing hash strips both, so audio doesn't break (VM never changes export level by contract) — but a foreign `source_lufs_integrated` in a *shared* preset is meaningless.
- **Fix:** strip both fields before `saveUserPreset` (mirror `settings_landing_hash`).
- **Confidence:** Medium.

**[P3] `MasteringSettings.album` is always `None` on the track path**
- **Where:** `types.rs:532` / `bindings.ts:214`. Every production constructor sets `album: None`; album rendering uses the separate `render_album_plan` request path. A serialized "looks wired, isn't used" field. Informational; no fix required.
- **Confidence:** High.

**[P3 / UX] Four EQ bands (Sub 80 Hz, Low-Mid 400 Hz, High-Mid 3.5 kHz, Sparkle 12 kHz) are drag-only and hidden in compact mode**
- **Where:** the knob row (`App.tsx:1811-1846`) exposes only Low/Mid/High; the other four are reachable only by dragging unlabeled Visual-EQ nodes (`VisualEqPanel.tsx`), and node value labels are suppressed in `compact` mode.
- **Why it's only P3:** the plumbing is correct end-to-end (these *do* reach the DSP), and the dsp.rs comments explicitly call them "Drag-only on Visual EQ" — so this is **working-as-designed discoverability**, not a wiring gap. Flagged for a product call.
- **Confidence:** High (drag-only); design-intent acknowledged.

**[Note, not a live bug] `compression_mode` TS type is wider than the wire contract**
- TS types it `compression_mode?: CompressionMode | null` (`bindings.ts:41`); Rust is a non-`Option` enum with `#[serde(default)]` (`types.rs:603-604`). An explicit `null` would fail to deserialize — but **every** frontend write path emits a literal `"preset"/"manual"/"off"` and reads use `?? "preset"`, so null is never sent. Harmless today; tighten the TS type to drop `| null` to keep it honest.

---

## 4. Dead Code & Bloat

**[P2] Compressor calibration table is duplicated by hand in Rust and TypeScript — drift risk** *(highest-value cleanup; subagent rated P1, but values currently agree → P2)*
- **Where:** Rust `PRESET_*` constants `dsp.rs:352-596` (threshold/ratio/attack/release) + engagement/overdrive math `dsp.rs:905-915`; mirrored **verbatim** in `compressor-auto.ts:23-36` + `:52-60`. I diffed every value — all nine presets match today (e.g. loud `−23/3.5/15/180`, punch `−20/2.8/10/100`).
- **Why it matters:** these numbers are **actively retuned** (the constants cite "Private reference tuning (2026-05-26)" and "reference tuning" is in the CLAUDE.md jump-fix queue). The TS copy powers the per-band compressor **UI readout**. The next Rust-side retune that doesn't also edit the TS file will make the UI display numbers that differ from what the chain actually applies — a silent, hard-to-notice divergence.
- **Fix:** single source of truth — expose the live Rust calibration via a small command (or extend the analyze return), or codegen the TS constants from the Rust table.
- **Confidence:** High.

**[P3] Legacy `process_sample` is a ~147-line drifted dead path**
- **Where:** `dsp.rs:2187-2333`. **No production caller** (`grep` shows only tests; all real paths use `process_interleaved` → `process_frame_inplace`). It deliberately **skips the `low_mid` band** (`dsp.rs:2199`, pinned by its own test) and uses a **soft-clip instead of the lookahead limiter** (`dsp.rs:2326`), and computes band gain with `powf` where the frame path uses `exp` (`dsp.rs:2301` vs `2126`).
- **Why it matters:** it isn't a live bug, but it's a maintenance liability and a latent trap — anyone who wires it up inherits a wrong EQ curve and a weaker ceiling, and its own passing test lends it false credibility.
- **Fix:** retire it, or bring it to parity and de-duplicate against `process_frame_inplace`.
- **Confidence:** High.

**[P3] `EnvelopeFollower` is test-only, and the one-pole envelope is implemented three times**
- **Where:** `dsp.rs:1481-1510` (`EnvelopeFollower`) is referenced only at `dsp.rs:3501` (a test). The production compressor (`dsp.rs:2101`) and the transient shaper (`dsp.rs:1543`) each re-implement the same `alpha·env + (1−alpha)·x` one-pole inline.
- **Fix:** either use `EnvelopeFollower` in the real code or delete it; consider one shared follower.
- **Confidence:** High.

**[P3] Duplicated 8-second preview-window slice in `audio.rs`**
- **Where:** `audio.rs:895-903` (inline in `export_landing_gain_lin_for_preview`) duplicates the `preview_landing_window` helper (`audio.rs:960-969`); the worker path actually **double-windows** (helper output is fed to the inline copy). Plus a no-op `safe_channels = channels_usize.max(1)` on `audio.rs:897` (`channels_usize` is already `max(1)`).
- **Why only P3:** idempotent today (input ≤ 8 s ⇒ inline window is the whole buffer). Confusing and fragile if the window size ever changes in one place.
- **Fix:** call the helper from both sites; drop `safe_channels`.
- **Confidence:** High.

**[P3] Overly broad `pub` on test-only items**
- `preset_calibration` (`dsp.rs:598`) is `pub` but only called internally → `pub(crate)`.
- `mastering_render` / `mastering_render_to_path` (`engine.rs:422`, `:441`) are convenience shims called only by tests/examples. *Caveat:* they must stay `pub` for the integration-test crate boundary, so this is cosmetic, not removable — verify before changing.
- **Confidence:** High on the observation; the fix is constrained by the test crate boundary.

---

## 5. Export Checks (`exports.rs`) — #1 jump-fix-queue feature

The advisory-vs-technical-failure split matches APP_BEHAVIOR.md, and the thresholds are internally consistent (TP, loudness, DR, bit-depth<16 = Critical, sample-rate mismatch = Critical, non-finite = Critical; quality rows advisory).

**[P3] Comment/message vs code mismatch on the true-peak check**
- **Where:** `exports.rs:23-50`. The `> -0.1 dBTP` branch emits `QualityLevel::Warning`, but the code comment (`:34`) calls −0.1 "the critical … threshold," and the message (`:28`) says platforms "reject masters above **-1.0** dBTP" while the branch actually fires at −0.1 (the −1.0…−0.1 band is the *separate* `streaming_headroom_low` warning).
- **Why only P3:** the `Warning` level is intentional and test-pinned (`run_export_checks_warns_on_high_true_peak`); only the wording/intent is inconsistent.
- **Fix:** reword the comment/message (or, if a peak over 0 dBFS should block, that's a deliberate product decision to make — currently advisory, consistent with "users may overcook their own track").
- **Confidence:** High.

---

## 6. CSS / UX / UI Hygiene

(All "dead CSS" claims were spot-checked with grep against `*.tsx`; the sampled fragments returned **0** matches, confirming the method.)

### Accessibility — real, but lower urgency for a single-user desktop app *(subagent rated several P1; recorded as P2)*

**[P2] Form inputs are not programmatically labelled**
- `GainField` / `NumberField` / `SelectField` render the label as `<span className="adv-label">` with the `<input>` as a *sibling*, not a `<label htmlFor>` (verified `App.tsx:2627`). The Delivery Profile `<select id="delivery-profile-select">` has an `id` but no matching `<label>` (verified `App.tsx:1998-2001`). The main waveform `<svg role="slider">` has no `aria-label` (`App.tsx:1362`); the *overview* SVG correctly does. `OverrideBanner` toggle buttons lack `aria-label`/`aria-pressed` (`App.tsx:832`).
- **Fix:** use `<label htmlFor>` (a `useId()` is already used in `Knob.tsx`); add `aria-label` to the main waveform; add `aria-pressed` to the override toggles. *(Note: `AlbumPanel.tsx` already does this correctly — copy that pattern.)*
- **Confidence:** High.

**[P2] Focus rings removed without a `:focus-visible` replacement**
- `.panel-reset-button:focus-visible { outline: none }` (`App.css:2954`) and `.preset-save-name:focus { outline: none }` (`App.css:1811`) suppress the global `:focus-visible` ring (`App.css:5133`), leaving keyboard users a weak border-color-only cue.
- **Fix:** drop `outline: none` (let the global ring through) or add a scoped `outline` on `:focus-visible`.
- **Confidence:** High.

### Dead CSS — [P3], confirmed unused

Roughly a dozen orphaned blocks in `App.css` (UI that was replaced). Confirmed-unused fragments include: `.sidebar-head`/`.brand`/`.mode-pill`/`.mode-toggle` (467-521), `.add-btn` (565-578), `.story-tags`/`.tag.conf-*` (1300-1331), `.io-gain` (1015-1028), `.transport`/`-left`/`-right` (1562-1588), `.export-bar`/`.export-btn`/`.advanced-toggle` (2342-2365), `.clip-indicator`/`.gr-indicator` states (1034-1119), `.compression-per-band*` (1122-1188), `.quality-summary*` (3197-3253), `.slider-row`/`-label`/`-input`/`-value`/`-number` (2227-2300), `.workspace-section-label`/`.track-sub` (1002-1013, 3827-3834), `.analysis-summary-icon`/`-chevron`/`-status` (1219-1271). **Grep-confirm each before deleting** (classNames can be built dynamically).

### Conflicts & theme debt — [P3]

- `.wf` declared three times with accumulating properties (`App.css:1438`, `1444`, `1467`) — consolidate.
- `.empty-foot` uses `!important` to beat a sibling it already out-specifies (`App.css:740-743`) — drop the `!important`.
- `var(--border-1)` is undefined (`App.css:1127`) — but it's inside a dead block, so moot once that block is removed; grep for other `--border-1` uses.
- `z-index` is an ungoverned set (`1,2,5,20,100,120,200,300`) with the scale only in comments — define `--z-*` tokens.
- `export-review-*` block mixes raw `px` + hardcoded red `rgba()` against the rest-of-file `rem` + `var(--accent-danger)` convention (`App.css:3617-3675`).
- Reduced-motion: live state dots convey state via animation only; under `prefers-reduced-motion` they become static with no non-motion differentiation (`App.css:5138`). *(Confidence: Medium — design call.)*

---

## 7. Docs vs Code

**[P3] ARCHITECTURE.md signal chain is stale by one+ stage**
- The doc lists a "9-step" chain and groups "Saturation/warmth/width" as one step. The code (`dsp.rs:1914-2008`) runs **input gain → 7-band EQ → compressor → transient shaper → width → saturation → limiter → volume-match → user output trim → export-landing gain** — i.e. ~10 distinct stages, with **width applied before saturation** and a **transient shaper** the doc doesn't mention. The "Compressor Off bypasses step 4 only" invariant itself **holds** in code (`Off` skips only `apply_multiband_compressor`).
- **Fix:** refresh the step list in ARCHITECTURE.md.
- **Confidence:** High.

---

## Appendix — Methodology & Caveats

- **Subagents:** four read-only agents (DSP/dead-code/wiring/CSS). Their raw severities were **not** trusted; each finding here was re-verified against code and re-graded. Four subagent "P1"s were downgraded (limiter ISP coverage, limiter TP kernel, the two duplicate tables).
- **Not exhaustively read:** `engine.rs` beyond ~line 930 (render orchestration; no DSP math expected there but not line-by-line confirmed); the full 4,579-line `dsp.rs` test module; `project.rs`/`files.rs`/`settings.rs` internals beyond their command signatures and the serde contract.
- **Open hunch (not counted):** no explicit denormal flush (FTZ/DAZ) around the IIR filters — a *performance* tail concern on long decays, not correctness. Not benchmarked.
- **Not run:** the slow private-fixture lane (`AMS_RUN_REAL_FIXTURE=1`) and manual listening gate — out of scope for a read-only pass, and the DSP voicing nuance (§2b knee) is exactly the kind of thing those lanes exist to judge.
