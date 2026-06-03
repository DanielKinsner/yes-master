# Handoff — Adaptive DSP: read this, then code (Tier-1 finish + Tier-2)

Audience: a fresh agent in a new window picking up the adaptive/smart DSP. This is
**self-contained** — review this doc and you can start coding without re-reading the history.

> **Merged to `main` on 2026-06-03 (merge commit `2877c6d`).** Work from `main`
> (or a fresh branch off it); `feat/adaptive-dsp-guardrails` is now historical.
> **Merged ≠ validated:** the guardrail numbers are still *provisional placeholders*
> and the owner's by-ear listening signoff has NOT happened. Do not treat the
> current constants as final, and don't build Tier-2 on the assumption that the
> Tier-1 numbers are locked.

---

## 1. TL;DR — where it stands

YES Master now has an **adaptive/smart DSP**: presets adapt to the source. It is
**Tier-1 defensive** — when a source already has a quality, the matching preset move
is *trimmed* toward neutral; it never adds a move, flips a sign, touches a cut, or
narrows. It is **merged to `main`** (`2877c6d`) and green, but **NOT yet
ear-calibrated** — the guardrail numbers (deadbands, caps, default strength) are
*provisional placeholders*, and the owner's by-ear listening signoff has not
happened. **Being on `main` does not mean it's taste-validated.** Only a human
clears taste; don't lock or hard-code assumptions on the current numbers. Two review rounds were
verified against the code and their real bugs fixed (see `docs/reviews/`). What's
left: the Tier-1 *finish* (one root-cause refactor + calibration) and **Tier-2** (the
corrective "smart" tier). Your job starts at the action plan in §7.

## 2. The mental model

- **4 guardrails:** already-**bright** → trim air/high lift; already-**boomy** → trim
  low/sub lift; already-**dense** → soften compression; already-**wide** → trim widening.
- **Defensive only.** Per-axis caps (EQ 50% / comp 60% / width 70%) + a **+0.5 dB EQ
  character floor** keep presets recognizable. Reduce-only.
- **One Adapt Strength dial** (`0..1`, default **0.6 on**; `0` = byte-identical to the
  old chain). Scales every trim.
- The analysis it reads was **already computed** (6-band spectral shares, P95-P10 DR,
  LRA, stereo correlation/width). We consume it; we didn't invent it. The analysis
  resolution is the ceiling on how "smart" this can get.

## 3. Architecture / data flow

```
import → analyze_one (analysis.rs) → AnalysisResult (6-band shares, DR, LRA, corr, width)
  │
  ├─ BACKEND-OWNED since B2 (2026-06-02): analyze_tracks derives SourceProfile via
  │     SourceProfile::from_analysis and caches it in SourceProfileStore
  │     (profile_store.rs), keyed by TrackId. Every Track-Master chain entry resolves
  │     from it via profile_store::apply_resolved_profile before building the chain:
  │       • render_track_preview / render_track_master  (engine.rs — have track id)
  │       • live play_master (has track id) + update_chain (audio thread resolves by
  │         the loaded track; reads the store fresh so a late analysis is picked up)
  │       • guardrail_readout (gets track id + album)
  │     Precedence: album → None; FE-supplied settings.advanced.source_profile =
  │     override; else the backend-derived cache. ALBUM stays flat (play_master/readout
  │     `album` arg → None; apply_album_shadow also strips). The FE no longer computes
  │     the profile (TS mappers deleted) — single Rust derivation, no dual-mapper drift.
  │
  └─ Rust slow lanes (fixture_matrix / reference_tuning) still build it directly via
        SourceProfile::from_analysis (test/tuning — same Rust derivation, no drift)

ChainCoeffs::from_settings (dsp.rs) reads settings.advanced.{source_profile, adaptive_strength}:
  • SourceGuardrails::compute(profile, strength)  (guardrails.rs) → per-axis multipliers
  • trims the PRESET contribution ONLY (user eq_*_db, Manual comp, explicit width untouched)
  • gated: no profile OR strength 0 ⇒ byte-identical (preset_byte_identity SHA snapshots prove it)

Readout: guardrail_readout command (guardrails.rs) → readout_for(settings) → REALIZED
  per-axis trims (post-floor) → Advanced panel.
```

**Load-bearing invariant:** no profile / strength 0 ⇒ the chain output is byte-for-byte
the old output. The `preset_byte_identity` SHA snapshots enforce it. Don't break them —
gate any new behavior on the profile being present.

## 4. File map

- `src-tauri/src/guardrails.rs` — **the heart.** Trim math, ALL tuning constants
  (deadbands, caps, floor, `ADAPTIVE_STRENGTH_DEFAULT`), `SourceGuardrails`, the
  readout (`readout_for`, `realized_eq_trim`, `guardrail_readout` command — now takes
  `track_id` + `album` and resolves from the store). **Tune here.**
- `src-tauri/src/profile_store.rs` — **B2.** `SourceProfileStore` (the backend-owned
  `TrackId → SourceProfile` cache), `resolve_effective_profile` (album/override/cache
  precedence), `apply_resolved_profile` (the single inject chokepoint). Populated by
  `engine::analyze_tracks` / `populate_profile_store`.
- `src-tauri/src/dsp.rs` — `ChainCoeffs::from_settings` applies trims (EQ ≈ L744,
  compression density ≈ L895, width ≈ L860); `preset_calibration` (preset band table).
- `src-tauri/src/types.rs` — `SourceProfile`, `AdvancedSettings.adaptive_strength`.
- `src-tauri/src/analysis.rs` — `analyze_one`; `compute_spectral_balance_6band` (the
  ~5.5 s `1<<18` window — see B6).
- `src-tauri/src/album_render.rs` — `apply_album_shadow` (strips the profile → album flat).
- `src-tauri/src/audio.rs` — live `update_chain` handler (settings-only, no track id —
  resolves the profile via the audio thread's `SourceProfileStore` + loaded track,
  gated by cached `live_album`); `play_master` (now takes `album`, resolves + caches).
- `src/lib/settings-transitions.ts` — `applyChainDispatchOverrides` (live VM +
  source_lufs only now; the TS profile mappers were deleted in B2 — backend owns it).
- `src/hooks/useTrackMaster.ts` — live dispatch (`withSourceLufs`), `exportMaster`,
  `updatePreview`, album export (`exportAlbumPlan` ≈ L1118), the readout fetch.
- `src/App.tsx` — Adapt Strength control + the readout (in `AdvancedControlsCard`).
- `src/lib/api.ts` / `src/bindings.ts` — `guardrail_readout` wrapper + `GuardrailReadout`/`SourceProfile` types.

## 5. Build / test / verify (exact commands)

Repo root: `C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\yes-master`.

```powershell
npm test                 # frontend (vitest) — currently 162 pass
npm run build            # tsc + vite (also regenerates dist/, which tauri needs)
cd src-tauri
cargo test --lib --target-dir target/codex-rc    # 236 pass; includes byte-identity snapshots
cargo test --target-dir target/codex-rc          # + integration suites
cargo clippy --all-targets --target-dir target/codex-rc -- -D warnings
cd ..
npm run build:windows    # MSI + NSIS bundle (slow; the real Windows gate)
```

- Use `--target-dir target/codex-rc` for cargo so you don't lock a running app's `target/`.
- `cargo test --lib` does **not** compile `generate_handler!` (it's behind the
  `app-runner` feature). To verify a **new Tauri command** registers, run
  `cargo check --all-features` or `npm run build:windows`.

## 6. Done — do not redo

Tier-1 engine (4 guardrails, caps/floor, Adapt Strength UI, byte-identity). Post-review
fixes: slow-lane + preview wiring; `LRA=0` sentinel; bright deadband `0.20→0.30` with a
real-pink regression; the per-axis readout (now realized/post-floor, B8); album
**non-adaptive end-to-end** (audition flat + backend strip + control disabled, tested,
B1/B9); durable on/off default (B4); LU→dB LRA aliasing removed (B11); stale-profile
clear (B10); doc/test hygiene (B12). Gates green (npm 162 / cargo lib 236 / integration /
clippy / build:windows). Owner decisions are locked in `ADAPTIVE_DSP_NEXT_STEPS.md`.

**P0 / B2 — backend-owned `source_profile` — DONE (2026-06-02).** Derivation moved
server-side (`profile_store.rs` + `analyze_tracks`); every Track-Master chain entry
resolves via `apply_resolved_profile`; the live `update_chain` path resolves by the
loaded track in the audio thread; album stays flat (`album` arg + cached `live_album`);
`guardrail_readout` re-touched (NF-2); the TS profile mappers were deleted. Gates green
(cargo lib 246 / contracts 39 / clippy / check --all-features / npm 157 / build).

## 7. What to do next — action plan (from the TRUE master review)

Owner already decided: album OUT (done), deadband quick-bump (done), readout built,
backend refactor (done — see §6). So:

**P0 — DONE (B2, see §6).** ~~Make the backend own `source_profile`.~~ Backend now
derives server-side, FE profile is an override, the live `update_chain` path resolves
via the audio thread's store, `guardrail_readout` re-touched, album kept flat, TS
mappers deleted.

**P1 — calibration-enabling (before the listening session):**
- **Welch-average the 6-band window + recalibrate brightness (B6).** Today the tonal
  read is the first ~5.5 s (`1<<18` cap). Average across the track. **Blast radius:**
  `compute_spectral_balance_6band` also feeds role/character/album-bias inference.
- **Export-receipt traceability (B5).** Add `effective_adaptive_strength` (+ a profile
  digest) to `ExportReport` and the receipt.

**P2 / Tier-2 — the "smart" tier (the real next milestone):**
- **Measured neutral** from the owner's reference masters (`AMS_RUN_REAL_FIXTURE`),
  optionally per-preset; **tilt-vs-reference deadband** (makes the deadband number
  ~irrelevant); **PSR/crest closed loop** (the only honest "won't crush transients");
  **total-loudness-loss budget** (B3); `stereo_width` co-trigger (B7); density-lever
  reshape. Plus the **Simple Mode view** — the stripped-down UI that rides on this
  engine (the other half of the original product idea).

Full detail + options: `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`.
Live backlog: `docs/ADAPTIVE_DSP_NEXT_STEPS.md`.

## 8. Gotchas

- **Don't break `preset_byte_identity`.** Gate any new chain behavior on the profile.
- **Album is intentionally flat.** Don't add injection for album; the strip + FE skip +
  disabled control are deliberate.
- **One profile mapper now (B2 done).** The TS `sourceProfileFromAnalysis` twin was
  deleted; the backend `SourceProfile::from_analysis` (via `profile_store.rs`) is the
  single derivation. The FE may still pass `advanced.source_profile` as an override on
  the wire, but nothing in the app constructs one — `resolve_effective_profile` decides.
- **Album flatness is gated in two ways now:** `apply_album_shadow` strips the profile
  for album renders, AND the live/readout paths pass `album` → `resolve_effective_profile`
  returns `None`. Don't add injection for album.
- **Readout is single-source-of-truth** (reuses `compute()` + the realized-floor math)
  and now resolves the profile from the store via `track_id` + `album` (B2/NF-2 done).
- **Tuning constants live ONLY in `guardrails.rs`.**
- **LF→CRLF** warnings on commit are benign. **`Cargo.toml`** showing "modified" after a
  windows build is EOL-only — `git restore` it.

## 9. Owner's gates (not yours)

- **Listening signoff:** A/B Adapt `0` vs `60%` on already-mastered / bright / dense /
  wide **and** neutral sources; confirm neutral does ~nothing; lock the constants.
- **Slow fixture lane** with private audio (`AMS_RUN_REAL_FIXTURE`) — now exercises the
  adaptive chain.

## 10. Pointers

- Spec: `docs/plans/2026-06-02-001-adaptive-dsp-tier1-guardrails.md`
- Finish/Tier-2 plan (options + rationale): `docs/plans/2026-06-02-002-adaptive-dsp-tier1-finish-and-tier2.md`
- Backlog / entry point: `docs/ADAPTIVE_DSP_NEXT_STEPS.md`
- The review you're coding from: `docs/reviews/2026-06-03-adaptive-dsp-TRUE-master-review.md`
- Original build handoff (historical): `docs/HANDOFF_2026-06-02_ADAPTIVE_DSP_TIER1.md`
