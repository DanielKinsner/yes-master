> **ARCHIVED 2026-06-22 - historical record, not active spec.**
> Adversarial repo review; findings fixed/routed; complete. _(Status: COMPLETE.)_ See docs/CHANGELOG.md for the project ledger.

# Codex Adversarial Repo Review - 2026-06-03

Read-only review of the current repo, latest commits through `18737a5`, and the adaptive DSP/filter path. This file intentionally lists only items that may need investigation or change.

---

## Validation & disposition (2026-06-03 — adversarial cross-check)

Each finding below was independently validated against the real code (reproduce
vs. refute, every citation fact-checked). The original text is kept verbatim for
the record; this block records what actually held up.

| # | Verdict | Priority (doc → real) | Disposition |
|---|---------|-----------------------|-------------|
| 1 | **Holds.** Citations accurate; the FFT read only the leading ~5.5 s. | P1 → **P2** | **Fixed** — `compute_spectral_balance_6band` now Welch-averages the whole track; regression test added. |
| 2 | **Holds, narrowed:** live-audition-only (export / offline preview / guardrail readout resolve album-ness independently) and self-healing on the next Mastered click. | P1/P2 → **P3** | **Fixed** — album flag now carried on `update_chain`; integration test added. |
| 3 | **Holds, narrowed:** only HARD failures leak (the soft too-short/silent path already clears via `set(id, None)`); in-session + persisted-id only; audition-only (export gated by `!selectedAnalysis`); re-import is immune (fresh UUID); the store is in-memory (no cross-session staleness). | P2 → **P3** | **Fixed** — `profile_store::prune_failed_profiles` evicts stale entries for failed re-analyses; unit tests added. |
| 4 | **Largely overstated.** The doc misquotes `APP_BEHAVIOR.md` — which makes quality rows *advisory once a file can be written* — and the three "Critical" rows are settings-derived and structurally unreachable on the Track-Master path (the WAV writer rejects bad bit depth before writing; rendered vs. requested sample rate are equal by construction; LUFS is `sanitize_lufs`'d finite). True write-invalidity already blocks before write. Only residual: a header-truth nit. | P2 → **P3 (cosmetic)** | **Partially addressed** — receipt header now reflects a critical ("Export saved — needs attention"). No file quarantine / pre-render gate warranted. |
| 5 | **Holds, accurate.** Not a code bug — provisional-but-conservative constants, mechanically tested, neutral = identity. | release gate | **Owner action** — run the listening matrix; no source change. |

Corrections to the original text below: §2 is live-audition-only and self-healing
(not an export risk); §3's real gap is hard-failure-skip specifically (the soft
`None` path already clears) and re-import cannot trigger it; §4's "undermines the
blocking contract" framing is inaccurate per the misquote above.

---

## 1. Adaptive Tonal Decisions Use Only The First ~5.5s

**Priority:** P1

`compute_spectral_balance_6band` derives the 6-band tonal profile from a capped FFT window. On longer tracks, the cap makes the tonal read represent the first ~5.5 seconds at 48 kHz, while DR, LRA, and correlation are whole-track measurements.

Evidence:

- `src-tauri/src/analysis.rs:321` computes `total_frames`.
- `src-tauri/src/analysis.rs:322-325` documents the `1 << 18` cap and the first-~5.5s behavior.
- `src-tauri/src/types.rs:141-153` turns that spectral read into `SourceProfile`.
- The profile then drives adaptive EQ/density/width for preview and export.

Why it matters:

A bright intro with a dark body, a dark intro with a bright chorus, or an ambient lead-in before a dense drop can cause the adaptive filter to trim based on the wrong section of the song. That is especially risky because the output is presented as track-aware mastering.

Suggested change:

- Replace the single initial-window read with whole-track Welch-style averaging or another representative whole-track spectral summary.
- Add a regression fixture/synthetic test where the first 5.5s has intentionally different tonality from the rest of the file.
- Confirm adaptive trims follow the whole-track/body character, not only the intro.

## 2. Mode Switching Can Leave Live Playback In The Previous Adaptive Context

**Priority:** P1/P2

Track Master / Album Master mode is changed with frontend `setMode`, but the backend audio thread only learns the album/non-album adaptive flag when `play_master` is called. Later settings-only `update_chain` calls reuse the cached `live_album` value.

Evidence:

- `src/App.tsx:303-311` mode tabs call `onModeChange("track")` / `onModeChange("album")`.
- `src/hooks/useTrackMaster.ts:1347-1356` sends `mode === "album"` only on `playMaster`.
- `src-tauri/src/audio.rs:1873-1875` caches that value as `live_album`.
- `src-tauri/src/audio.rs:1263-1269` applies profiles during `update_chain` using cached `s.live_album`.
- `docs/ADAPTIVE_DSP_NEXT_STEPS.md:33-35` already notes the edge case.

Why it matters:

If a user switches Track Master to Album Master while Mastered playback is already loaded, subsequent live edits can still resolve as Track Master and remain adaptive. The reverse can also happen: switching back to Track Master can keep audition non-adaptive until a fresh `play_master`.

Suggested change:

- Send the current album flag with `update_chain`, or reload/re-prime Mastered playback on mode change while a mastered stream is loaded.
- Add an integration test for: play Mastered in Track mode, switch to Album mode, change an advanced control, and assert backend update resolves as non-adaptive.
- Add the inverse test: play Mastered in Album mode, switch to Track mode, change Adapt Strength or an advanced control, and assert adaptive resolution is restored.

## 3. Failed Re-Analysis Can Leave Stale Adaptive State

**Priority:** P2

The profile store clears stale profiles only for tracks that produce an `AnalysisResult` whose `SourceProfile::from_analysis` returns `None`. If analysis fails for a track, that track is skipped before `populate_profile_store` sees it, so any existing cached profile for that `TrackId` remains.

Evidence:

- `src-tauri/src/engine.rs:45-73` returns partial successes and skips failures.
- `src-tauri/src/engine.rs:80-86` populates or clears profiles only for successful results.
- `src-tauri/src/profile_store.rs:41-54` correctly supports clear-on-`None`, but failures never reach that path.
- `src/hooks/useTrackMaster.ts:1833-1845` catches project-open re-analysis failure without clearing prior analysis/profile state.

Why it matters:

A moved file, decode failure, or project-open refresh failure can leave a prior profile attached to the same logical track ID. Later readout/live/render paths can then apply adaptation derived from old audio.

Suggested change:

- Clear requested track IDs from `SourceProfileStore` before analysis, or explicitly clear failed IDs after partial-success analysis.
- On project-open analysis failure, clear `analysisMap` for the opened project tracks rather than leaving prior session data in place.
- Add a mechanical test where a store has a profile, analysis is requested for the same `TrackId` and fails, then subsequent profile lookup returns `None`.

## 4. Critical Export Checks Are Detected After The File Is Written

**Priority:** P2

The export flow renders the file first, then builds an `ExportReport`, then runs export checks. Some checks are marked `Critical`, but by the time they exist the output file has already been written and the right rail review flow treats review rows as export-review/advisory UI.

Evidence:

- `src/hooks/useTrackMaster.ts:1284-1289` calls `renderTrackMaster` with the chosen output path.
- `src/hooks/useTrackMaster.ts:1298-1316` builds the report and then calls `runExportChecks`.
- `src-tauri/src/exports.rs:74-105` marks low bit depth, sample-rate mismatch, and non-finite LUFS as critical.
- `src/components/RightRail.tsx:74-86` opens review rows first, then `Export Anyway` calls the same export path.
- `docs/APP_BEHAVIOR.md:93-95` says technical failures should block export when the app cannot write a valid file.

Why it matters:

The app can create an output file and only afterward label the render technically invalid. That undermines the "warnings are advisory, technical invalidity is blocking" contract.

Suggested change:

- Treat post-render critical checks as failed exports: do not present them as successful receipts, and consider deleting/quarantining the just-written file.
- Where possible, preflight technically invalid settings before render.
- Add a test that simulates a critical post-render check and asserts the UI/backend reports export failure rather than "export complete with critical row."

## 5. Adaptive Calibration Is Still Not Release-Locked By Evidence

**Priority:** P2/P3

The adaptive guardrail constants are implemented and tested mechanically, but docs still mark owner listening signoff/calibration as pending.

Evidence:

- `docs/ADAPTIVE_DSP_NEXT_STEPS.md:3-6` says guardrail numbers are provisional placeholders and not ear-validated.
- `docs/ADAPTIVE_DSP_NEXT_STEPS.md:106-120` lists listening A/B and slow fixture lane calibration as pending.
- `src-tauri/src/guardrails.rs` owns the current deadbands/caps/default strength.

Why it matters:

Adaptive DSP is now user-facing enough to affect trust. Mechanical tests prove wiring and invariants, but they do not prove the default `60%` strength, deadbands, or trim caps sound right across already-mastered, bright, dense, wide, and neutral sources.

Suggested change:

- Run and record the private fixture listening matrix before treating adaptive DSP as release-stable.
- Capture per-fixture notes for Adapt `0` vs `60%` and any candidate deadband changes.
- Avoid deadband/default changes without those notes.

## Verification Gaps From This Review

- Full fast lane was not completed in this read-only pass: `npm run build`, `npm run build:windows`, and full `cargo test` were not run.
- Slow fixture lane was not run: `AMS_RUN_REAL_FIXTURE=1 cargo test`.
- No live browser/Tauri audition or listening pass was performed.
