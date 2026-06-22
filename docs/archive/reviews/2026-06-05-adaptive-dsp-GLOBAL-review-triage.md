> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> GLOBAL adaptive-DSP review triage; B2 refactor closed the findings. _(Status: COMPLETE.)_ See docs/CHANGELOG.md for the project ledger.

# Triage — `2026-06-03-adaptive-dsp-GLOBAL-review.md` against current `main`

Date: 2026-06-05
Source under triage: `docs/reviews/2026-06-03-adaptive-dsp-GLOBAL-review.md` (written against the
**pre-merge** `feat/adaptive-dsp-guardrails` branch).
Method: 8-cluster read-only verification fleet; every finding re-checked against current `main` by
symbol/content (the review's line numbers are stale). No code changed during triage.

Task #2 of `docs/archive/AGENT_WORK_AND_REVIEW_QUEUE_2026-06-05.md` asked to extract each finding and tag it
`objective-fix` / `needs-owner-ear` / `already-fixed` / `stale` / `unclear`, with current code evidence,
**before** any code change.

## Verdict

The GLOBAL review predates the **B2 backend-ownership refactor** (and the B4/B5/B6/B11 follow-ups), which
closed almost all of it. Of 20 distinct findings (F6/N1 are one item):

| Tag | Count | IDs |
|---|---|---|
| **already-fixed** | 15 | F1, F2, F3, F4, F6/N1, F8, F9, F11, F12, F13, N2, N3, N5, N6, N7 |
| **stale** (premise reversed/superseded) | 3 | F5, F14, N4 |
| **needs-owner-ear** (taste/calibration only) | 2 | F7, F10 |
| **objective-fix** | 0 | — |
| **unclear** | 0 | — |

**There is no open adaptive-DSP code defect to fix.** The only actionable objective residue is a single
stale doc-comment surfaced inside F5 (see "Residual" below); the two `needs-owner-ear` items are
constants/wiring decisions reserved for the owner.

## Already-fixed (15)

| # | Original | Closed on `main` by |
|---|---|---|
| **F1** | Slow lanes never set `source_profile` → evidence renders un-adapted | `fixture_matrix.rs:132` / `reference_tuning.rs:319` derive via `from_analysis`; presence tests `fixture_matrix.rs:536`, `reference_tuning.rs:746`. |
| **F2** | Preview/audit WAV skips injection while export injects (WYSIWYG break) | `render_track_preview` (`engine.rs:428`) resolves the backend profile identically to `render_track_master` (`:473`); TS injector twins removed; parity test `useTrackMaster.integration.test.tsx:1259`. |
| **F3** | Non-finite LRA → `0.0` sentinel → spurious full density trim | density ramp guards `LRA<=0.5` (`guardrails.rs:144-152`); regression `lra_sentinel_does_not_density_trim_a_dynamic_source` (`guardrails.rs:506`). |
| **F4** | Brightness deadband `0.20` over-trims pink-neutral (~0.278) | owner raised `BRIGHT_DEADBAND` 0.20→0.30 above the pink-neutral share (`guardrails.rs:41`); pinned by `neutral_source_is_identity`. |
| **F6 / N1** | Album live injects but export doesn't (audition≠deliverable) | both surfaces now uniformly non-adaptive — `profile_store.rs` returns `None` for album, `album_render.rs:104` strips the profile; same-track-differs is the documented owner decision; regression tests `resolve_album_is_always_inert`, `album_shadow_strips_source_profile_so_album_stays_unadapted`. |
| **F8** (stale-comment half) | Stale "down-only" landing comment vs upward makeup | landing comments now read "down, or up when ceiling headroom allows" (`dsp.rs:686-689`, `:2074-2075`); old cited line is now an unrelated Width comment. (Upward-makeup *behavior* is owner-only — see note.) |
| **F9** | `from_analysis` dead; `types.rs` comment falsely claims backend-derived export | `from_analysis` has live callers (`engine.rs:128`, slow lanes, iPhone `lib.rs:160`); `types.rs` doc rewritten to match the backend-owned map. |
| **F11** | No per-axis "what was trimmed & why" readout | `GuardrailReadout` carries per-axis trims + context + deadbands + confidence (`guardrails.rs:230-261`); rendered in the Advanced card `AdaptiveReadout` (`App.tsx:2069-2161`). |
| **F12** | Spec over-claims `presence_db` / `stereo_width` | spec table corrected (`docs/plans/2026-06-02-001-…:179,182`); code was always correct (`presence_db` untrimmed `dsp.rs:803`). |
| **F13** (dual-default half) | Two copies of the default strength (Rust const + TS `?? 0.6`) | single shared const `ADAPTIVE_STRENGTH_DEFAULT = 0.5` (`guardrails.rs:23`, `bindings.ts:277`, `App.tsx:2282`); the old `?? 0.6` default fallback is gone. (Per-band floor + `0.42` deadband are owner-only — not defects.) |
| **N2** | No backend fallback; correctness rests on FE injecting everywhere | `populate_profile_store` caches a derived profile per track at analysis (`engine.rs:42`); single chokepoint `apply_resolved_profile` (`profile_store.rs:144`, `fe_override.or(cached)`); contract test `contracts.rs:259`. |
| **N3** | Audit-WAV runs non-adaptive while live+export adaptive | instance of F2 — `updatePreview` → `render_track_preview` now runs the same adapted chain. |
| **N5** | Missing P95-P10 aliases an LU value into the dB DR ramp | `from_analysis` uses a `100.0` dB no-trigger sentinel, not `unwrap_or(dynamic_range_lu)` (`types.rs:151-164`); DR is `Option<f32>`; regression `missing_p95p10_does_not_alias_lra_into_the_db_ramp` (`guardrails.rs:488`). |
| **N6** | Export receipt carries no adaptation record | `effective_adaptive_strength` + `source_profile_digest` + `confidence_digest` on `RenderedMeasurements` and `ExportReport` (`types.rs:796-845`), populated server-side (`engine.rs:796-825`) and copied by the FE receipt + slow lanes. |
| **N7** | Adapt Strength editable-but-inert in album mode | control `disabled` in album mode with an honest note (`App.tsx:2280-2298`). |

## Stale — premise reversed or superseded (3)

| # | Original | Why stale |
|---|---|---|
| **F5** | 6-band FFT reads only first ~5.5 s despite "30 s" comment | now an explicit **whole-track Welch average** (`analysis.rs:387-457`) with regression `spectral_balance_6band_reflects_whole_track_not_just_intro`. The in-place "30s" comment was corrected; **one residual stale comment remains at `types.rs:90`** (see Residual). The whole-track window swap itself was the owner-only change, and it is done. |
| **F14** | iPhone shares engine, never injects, runs un-adapted | reversed — the bridge resolves a backend-derived profile + confidence like desktop (`apps/iphone-native/rust/src/lib.rs:241-245`), with parity tests; the iPhone docs were rewritten to "adaptive hidden in the bridge, no UI controls in v1" (`IPHONE_APP.md`, `IPHONE_APP_OVERVIEW.md`). |
| **N4** | `null` strength silently re-enables `0.6` | explicit `0.5` default everywhere, durable **Off** pill at `0` (`App.tsx:2286`), reset/field-clear no longer write `null` (`App.tsx:2199-2204`, `:2887-2896`), `DEFAULT_SETTINGS` seeds explicit `0.5`. Residual double-click→`null` now surfaces a visible **Auto** pill (owner UX, not a silent trap). |

## Needs-owner-ear — taste/calibration only, do not touch (2)

| # | Finding | Owner lever |
|---|---|---|
| **F7** | Density is a single scalar; realized trim saturates once `strength·density_raw` hits `DENSITY_CAP=0.60`. Current default strength is **0.5**, so the old "default 0.6 already maxes" wording is stale, but the cap-shape/taste concern remains real once strength reaches the cap on fully dense material. "Overdrive as a 4th stage" was overstated (same threshold/ratio mechanism, `dsp.rs:977-980`). | Re-anchoring `DENSITY_CAP` or splitting density into sub-knobs is Tier-2 voicing. No false comment / dead code / wiring bug to fix mechanically. |
| **F10** | `stereo_width` is computed + carried in `SourceProfile` but never read by the width trigger (`guardrails.rs:156-162` reads only `stereo_correlation`). | Wiring it as a width co-trigger (owner taste, named in the queue) or removing a carried IPC field (contract decision) — both owner-only. The spec is now honest about the inert state. |

## Residual objective doc-fix (the only code action from this triage)

- **`src-tauri/src/types.rs:90`** — the `spectral_balance_6band` doc-comment still says
  "*…FFT (Hann-windowed, up to 30 s of mono)*", but `compute_spectral_balance_6band` is now a
  **whole-track Welch average** (`analysis.rs:387-399`). Doc-only, owner-safe (no behaviour/constant
  change). Fixed in the follow-up commit to this triage.

## Note on F8 behaviour (owner-only, no fix)

The post-chain LUFS-landing stage can add upward broadband makeup (bounded by true-peak headroom) when an
effective target exists (incl. Custom + explicit `lufs_offset_db`), which can recoup trim-induced loudness
loss beyond a per-axis cap (`engine.rs:182-199`). This is real and unchanged, but it is exactly the
**total-loudness-loss budget behaviour** reserved to the owner. Documented here; not changed.
