# Master Review — YES Master (consolidated, cross-source verified)

- **Date:** 2026-05-29
- **Status:** Supersedes [2026-05-29-adversarial-review.md](2026-05-29-adversarial-review.md). Read-only; no code modified.
- **Sources reconciled (all re-verified against *current* code, 2026-05-29):**
  1. This reviewer's adversarial pass (DSP / dead-code / wiring / CSS-UX; 4 parallel agents; hand-verified).
  2. **Codex Adversarial Review** — `docs/CODEX_ADVERSARIAL_REVIEW_2026-05-29.md` (full 8-finding review).
  3. **Codex Wiring Review** — `docs/CODEX_WIRING_REVIEW_2026-05-27.md` (secondary; mostly already actioned).
- **Reconciliation rule:** A Codex finding is carried forward only if it still reproduces in current code; already-fixed items are listed as *verified-fixed*, not re-raised. Where this pass and Codex independently flag the same thing → **✓ secondary-source verified**.
- **Validated by Codex** (`docs/reviews/2026-05-29-codex-validation-of-master-review.md`, 2026-05-29): all substantive findings confirmed against the current tree; gate re-run green. Three **wording** corrections were raised and are applied here — focus-ring scope (§6), reduced-motion claim withdrawn (§6), and "export corruption" qualified (TL;DR + §2). No substantive finding was overturned.

---

## TL;DR (the "so what")

The core verdict holds: **healthy, well-tested codebase; the build/lint/test gate is green; no major processing, loudness, true-peak, or render-order bug corrupts the master.** (One qualifier, per Codex's validation: the dither bug *does* affect the exported integer-PCM noise floor — it's an export-spec/noise-floor defect, not musical-content corruption. See §2.) But cross-referencing the **full** Codex review (it had been regenerated from an earlier stub) was worth it — **Codex caught four real things my pass missed**, two of them DSP:

1. **Mono live LUFS reads ~+3 LU too hot** — the live meters duplicate mono into a stereo pair, so a mono file is monitored ~3 LU louder than it actually is (and louder than its own export, which measures correctly). A metering-**trust** bug, mono-only, live-display-only. **(P2; Codex rated P1.)**
2. **WAV dither is ~2× the intended amplitude** — and this **corrects a mistake in my own first review**, where my DSP agent rubber-stamped the dither as "correct." Standard TPDF is ±1 LSB; this is ±2 LSB. **(P2.)**
3. **~12 MB of 1024px preset PNGs** ship for 37–78px icons. **(P2 bundle bloat.)**
4. **`csp: null`** disables the Tauri content-security-policy. **(P3 security hygiene, pre-release.)**

Two of Codex's findings **duplicate mine** (legacy `process_sample`; right-rail reachability) → tagged ✓ secondary-source verified. The bulk of Codex's *older* wiring review is **already fixed** in current code (verified) and not re-raised.

Revised counts: **0 P0, 0 P1**, **~5 P2**, **~20 P3**. (The two Codex P1s are recorded at P2 with the scope that justifies the downgrade — see §2/§10.)

> **Candor note:** my first pass's "verified-correct" list wrongly included the WAV dither, and never traced the mono channel path. Both are corrected below. This is exactly why a second independent source is valuable.

---

## 1. Verification Gate — REAL results

| Command | This pass | Codex independent run |
|---|---|---|
| `cargo test` (`target\codex-rc`) | **PASS** — 208 lib + 71 integ/doc, 0 failed | PASS (208 lib; full suite passed on rerun) |
| `cargo fmt --check` | **PASS** | PASS |
| `cargo clippy --all-targets -- -D warnings` | **PASS** — no warnings | PASS |
| `npm test` | **PASS** — 149 tests | PASS — 147 tests (run slightly earlier) |
| `npm run build` | **PASS** | PASS |

*Process note:* Codex hit a one-off Windows linker/open-file error from two `cargo test` runs sharing a target dir — exactly the collision `docs/TESTING.md` mandates `target\codex-rc` to avoid. Re-run passed; not a product issue.

---

## 2. DSP Correctness (highest priority)

**Still verified-correct:** BS.1770 K-weighting (sample-rate-aware), momentary+integrated LUFS gating, RBJ biquads, ceiling-bounded landing (no overshoot), rubato SRC direction, limiter ceiling enforcement, M/S width, `ebur128` for authoritative LUFS/TP, NaN/Inf & empty-buffer guards. **No P0/P1.**

| Sev | Finding | Location | Source / Confidence |
|---|---|---|---|
| **P2** | **Mono live LUFS inflated ~+3.01 LU.** Both live sources duplicate mono into a stereo pair (`r = l`) then feed `process_frame(l, r)`, summing `l²+r² = 2·l²` → +3.01 LU. The export/analysis path uses `ebur128` with the **real** channel count, so a mono file measures correctly there. Result: mono is *monitored* ~3 LU hotter than it *exports*. Stereo is unaffected; export is correct; the A/B is internally consistent (both meters inflated equally). **Codex rated P1; downgraded to P2 because it's mono-only and display-only (the deliverable is correct).** Fix: feed mono as one channel (or halve mono energy) and add a mono regression test. | live meters `sources.rs:131-144` (original) + `405-418` (mastered); meter sum `dsp.rs:1609-1619`; correct path e.g. `analysis.rs:52-64` | **Codex P1 — verified, downgraded.** High |
| **P2** | **WAV TPDF dither is ~2× the standard amplitude.** `tpdf_lsb()` sums two uniforms each in `[-1,1)` LSB → triangular in `[-2,2)` LSB (±2 LSB). Standard TPDF (Lipshitz/Vanderkooy, which the file cites) is ±1 LSB (two ±0.5 LSB RPDFs). The **file header comment itself says "±1 LSB"** (`wav_writer.rs:13`) while the function comment says ±2 LSB — internal contradiction. Net effect: dither noise power ~+6 dB over a correct TPDF (audible-ish only at 16-bit/CD; inaudible at 24-bit). The tests **pin the ±2 LSB behavior** (`wav_writer.rs:449-465`), which is why neither my agent nor the suite flagged it. **This corrects my first review, which called the dither "correct."** Sub-issue: a fresh deterministic RNG per `write_samples_into_writer` call (`:123-124`, pinned by `:305-352`) means segmented album writes repeat the same noise pattern per segment. | `wav_writer.rs:46-56`, `:82-90`, `:123-124` | **Codex P1 — verified, downgraded to P2.** High |
| P3 | Soft-knee compressor zeroes the **lower half** of the knee; not the centered textbook quadratic. Sub-dB voicing nuance, both code paths, taste-dependent. | `dsp.rs:2109-2118` (+ dup `:2291-2300`) | this pass. High/Med |
| P3 | Limiter ISP uses a **Lagrange-4 kernel**, not the BS.1770 polyphase FIR (~0.1–0.3 dB optimistic); backstopped by the authoritative `ebur128` export TP check. | `dsp.rs:1372` | this pass. High |
| P3 | 6-band spectral edges misorder below ~13 kHz Nyquist (air band dropped). Inference-only, latent. | `analysis.rs:360-361` | this pass. High |
| P3 | Dynamic-range percentile truncated nearest-rank. Inference-only, cosmetic. | `analysis.rs:471-472` | this pass. High |

**Disproven (not raised):** limiter ISP "boundary gap"; K-weighting wrong off-48k.

---

## 3. Wiring (UI → state → command → output) — *source: this pass*

| Sev | Finding | Location |
|---|---|---|
| P3 | `AlbumTrackEntry` TS type omits `album_character` (and `AlbumCharacter` has no TS type); survives only via JS keeping unknown keys. Hand-written `bindings.ts` drift. | `bindings.ts:166-173` vs `types.rs:430` |
| P3 | Track-mode Width caps at `1.5` but DSP allows `[0,2]`; `SignalChain.presetDefaultWidth` shows `1.3` for Spatial (real `1.16`) and ignores Oomph/Tape/Warmth — stale display mirror. | `App.tsx:2096`, `SignalChain.tsx:45-52` |
| P3 | User presets capture transient `volume_match` / `source_lufs_integrated`. | `useTrackMaster.ts:1596` |
| P3 | `MasteringSettings.album` always `None` on track path (dead serialized field). | `types.rs:532` |
| P3 | Four EQ bands (Sub/Low-Mid/High-Mid/Sparkle) drag-only + hidden in compact mode — working-as-designed discoverability. | `App.tsx:1811-1846`, `VisualEqPanel.tsx` |
| P3 | **`Transport` receives both `loop` and `loopEnabled`** (looping-active vs region-exists) — distinct, so a naming/clarity nit only. | `App.tsx:753,756`; `:1534,1564` | 
| P3 (watch) | **`loadedKindByTrack` live-chain gating** under-tested; not a confirmed bug. Add a focused regression test (edit while paused-on-Master / playing-Original / post-undo). | `useTrackMaster.ts:144,1323` |

*(Last two: **Codex wiring §3.3 / §3.1, verified still-live.**)*

**Note (not a live bug):** `compression_mode` TS type is `| null` but the Rust enum rejects explicit null; every frontend write emits a literal and reads use `?? "preset"`. Tighten the TS type.

---

## 4. Dead Code & Bloat — *source: this pass*

| Sev | Finding | Location |
|---|---|---|
| **P2** | **Compressor calibration duplicated Rust↔TS** (values + engagement math). In sync today; actively-retuned table → next retune silently drifts the UI readout. **Highest-value cleanup.** | `dsp.rs:352-596` ↔ `compressor-auto.ts:23-36` |
| P3 | Legacy `process_sample` (~147 lines): no production caller; drifted (skips `low_mid`, soft-clip vs limiter, `powf` vs `exp`). Latent trap with a passing test. | `dsp.rs:2187-2333` — **✓ secondary-source verified (Codex P3)** |
| P3 | `EnvelopeFollower` test-only; one-pole envelope implemented 3×. | `dsp.rs:1481-1510` |
| P3 | Duplicated 8 s preview-window slice + no-op `safe_channels` + double-windowing. | `audio.rs:895-903` vs `:960-969` |
| P3 | Overly broad `pub` on test-only items (`preset_calibration`; `mastering_render` shims — but shims must stay `pub` for the test crate). | `dsp.rs:598`, `engine.rs:422,441` |

---

## 5. Export Checks — *source: this pass*

| Sev | Finding | Location |
|---|---|---|
| P3 | True-peak check: comment calls `>−0.1 dBTP` "critical" and message says "reject above **−1.0**," but the branch fires at `−0.1` and emits `Warning` (intentional/test-pinned). Wording mismatch. | `exports.rs:23-50` |

---

## 6. CSS / UX / Accessibility — *source: this pass*

| Sev | Finding | Location |
|---|---|---|
| **P2** | Inputs not programmatically labelled: `GainField`/`NumberField`/`SelectField` use `<span class="adv-label">` not `<label htmlFor>`; Delivery Profile `<select id>` has no label; main waveform `<svg role="slider">` lacks `aria-label`; override toggles lack `aria-label`/`aria-pressed`. (Album panel does it right.) Lower urgency for a desktop app. | `App.tsx:2627,1998,1362,832` |
| **P2** | A global `:focus-visible` ring **does** exist (`App.css:5133`, added to fix near-invisible focus). The narrower real issue: two controls suppress it locally — `.preset-save-name:focus` (`outline:none`, weak border-only cue, and fires on `:focus` not `:focus-visible`) and `.panel-reset-button:focus-visible` (`outline:none`, replaced by border/background). Not app-wide. *(Codex-corrected scope.)* | `App.css:1811-1814,2955-2960` |
| P3 | ~12 dead CSS blocks (grep-confirmed) — list in prior doc. | `App.css` |
| P3 | `.wf` declared 3×; `.empty-foot !important`; `var(--border-1)` undefined (dead block); ungoverned `z-index`; `export-review-*` px/rgba vs rem/vars. | `App.css` (various) |
| — | **Withdrawn (Codex-corrected):** the earlier "reduced-motion state-dots rely on motion" claim is overstated — live/busy/idle differ by color, background, border, and glow plus a text label (`App.css:905-943`), so reduced-motion users are not blocked. Motion is only an extra cue. | `App.css:905-943,5138-5147` |

---

## 7. Layout & Reachability — **✓ secondary-source verified** (both Codex docs)

| Sev | Finding | Location |
|---|---|---|
| P3 (watch) | **Right rail is a scroll surface; verify the sticky Export group never hides an unreachable control at the resize floor.** Launch is `1920×1080` (no overlap there); `1440×860` is only the resize minimum. CSS makes the rail scrollable (`App.css:4453-4455`) and the export group sticky (`:4868-4871`). Add a launch + min-resize screenshot smoke check with Advanced Controls expanded, asserting Delivery Format and Export stay reachable. **I had not covered responsive layout — genuine gap in my pass.** | `tauri.conf.json:17-20`; rail render |

---

## 8. Product / Repo Hygiene & Security — **NEW (Codex), all verified**

| Sev | Finding | Location | Confidence |
|---|---|---|---|
| **P2** | **~12 MB of 1024×1024 preset PNGs ship for 37–78px icons.** All eight statically imported and bundled; per my build log: universal 527 KB + seven at ~1.5–1.7 MB ≈ **12.3 MB**. Downscale to max display size (+density) and convert to WebP/AVIF. | `PresetIcon.tsx:11-18`; render sizes `App.css:1908,4747,4945,5064` | High |
| P3 | **9 generated screenshots tracked under `test-output/`** (`git ls-files` confirms: `current-ui.png`, `preview-1600*.png`, `preview-check*.png`, `ui-tighten-e*.png`, a smoke `.ams.json`). At least one is stale vs current branding; fights the repo's own "evidence stays ignored" convention. Untrack them. | `test-output/` | High |
| P3 | **`csp: null` disables the Tauri CSP.** Acceptable for a local-only app with bundled assets, but a real hygiene gap if it ever loads remote assets / user HTML. Before release: document why it's safe, or add a restrictive CSP. | `tauri.conf.json:23-24` | High |
| P3 | **Large-file concentration** (App.tsx ~2.9k, App.css ~4.5k, useTrackMaster.ts ~1.8k, dsp.rs ~4.3–4.6k, audio.rs ~3.3k lines). Not wrong per se; raises change-isolation risk. Extract only when fixing active bugs, not as a broad refactor. | (whole-file) | Med (soft) |

---

## 9. Docs vs Code — *source: this pass*

| Sev | Finding | Location |
|---|---|---|
| P3 | ARCHITECTURE.md "9-step chain" stale — code runs ~10 stages (transient shaper unlisted; width before saturation). The "Compressor Off bypasses step 4 only" invariant holds. | `docs/ARCHITECTURE.md` vs `dsp.rs:1914-2008` |

---

## 10. Codex Cross-Reference Reconciliation

### 10a. Codex findings INCORPORATED (verified still-live)
| Codex | Sev (Codex → here) | Where folded |
|---|---|---|
| Mono live LUFS +3.01 LU | P1 → **P2** | §2 |
| WAV dither ~2× amplitude + per-call RNG | P1 → **P2** | §2 (corrects my prior "verified-correct") |
| Preset art oversized (~12 MB) | P2 → **P2** | §8 |
| Stale screenshots tracked | P2 → **P3** | §8 |
| `csp: null` | P3 → **P3** | §8 |
| Large-file concentration | P3 → **P3** | §8 |
| Legacy `process_sample` trap | P3 → **P3** | §4 ✓ secondary-source verified |
| Right-rail reachability | P3 → **P3** | §7 ✓ secondary-source verified |

### 10b. Codex (wiring 2026-05-27) items VERIFIED ALREADY FIXED — not re-raised
`LevelsPanel`, `StereoWidthGauge`, rail threshold consts, `advancedOpen`/`toggleAdvanced`, `updateAlbumIntent`, `render_album_master`, `get_diag_counters` (all removed — `grep` finds nothing); dual loudness-target control (consolidated to one source-of-truth dropdown); 3× live-chain predicate (refactored); duplicate `LIVE PEAK`/`LIVE LUFS` bottom bar (removed); insight chevron-to-nowhere (no chevron, body always renders); `ACTIVE n` debug pill (removed).

### 10c. Net assessment of the two passes
**Complementary, with two genuine misses on my side.** Codex was stronger on the realtime metering path (mono LUFS) and the dither spec — areas where my DSP agent verified the math *in isolation* but didn't trace it against the export path or the standard's amplitude. My pass was stronger/unique on: the Rust↔TS compressor drift, contract/type drift (`AlbumTrackEntry`, width mirror), the soft-knee deviation, the EQ/landing/SRC/limiter verifications, dead-code beyond `process_sample`, the export-check wording, and the a11y/dead-CSS sweep. Only `process_sample` and the right-rail item overlap directly (now ✓ secondary-source verified).

---

## Appendix — Method & Caveats
- Every Codex finding re-checked against current code before carry/retire; source-doc line numbers had drifted. The `CODEX_ADVERSARIAL_REVIEW_2026-05-29.md` file was a 21-line stub on first read and a full 8-finding review on the second — this doc reflects the full version.
- **Correction to the 2026-05-29 adversarial review:** its coverage list wrongly marked the WAV dither "correct"; see §2.
- Not exhaustively read: `engine.rs` beyond ~line 930; full `dsp.rs` test module; project/files/settings internals beyond command signatures.
- Not independently verified: Codex §4.3/§4.4 subjective UX polish (crowded A/B row, insight card width) — backlog, not asserted.
- Open hunch (uncounted): no explicit denormal flush (FTZ/DAZ) around the IIR filters — performance, not correctness.
- Not run: slow private-fixture lane and manual listening gate (out of scope for read-only).
