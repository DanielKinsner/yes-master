# 2026-07-05 Owner Smoke-Test Findings — Independent Validation + Fix Plan

Successor to `docs/plans/2026-07-04-owner-smoke-findings-handoff.md` (the
orientation doc). That doc was a map; this is the route. Every finding and
every claimed cause was re-validated against current code by independent
review agents (two passes on the three highest-impact clusters), and the
high-impact symptoms were exercised **live against the fresh 2026-07-04
21:11 installed build** (rebuilt + silent-reinstalled first, per the
handoff's instruction) via the WebView2 CDP port.

## Method (what "validated" means here)

- Rebuilt (`npm run build:windows`) and silent-reinstalled the NSIS setup;
  confirmed the installed exe timestamp moved 10:34 → 21:11 (diagnostics
  build live).
- Backed up the owner's `session.json`, seeded deterministic test sessions
  (generated 2/4/8/16/24/60-min stereo WAVs at 44.1k/48k), drove the real
  app over CDP (`--remote-debugging-port`), and restored the owner's session
  afterwards.
- Fan-out code validation: one adversarial reviewer per finding cluster,
  then a second-opinion pass attacking the first pass on clusters A
  (width), B (swap/device-loss), and D (export perf). All file:line cites
  below re-checked against current code; the load-bearing ones I re-read
  myself.

## Reproduction results on the fresh install (healthy machine)

| Probe | Result |
| --- | --- |
| O/M flips, 5 × 4 s apart, 44.1k WAV, defaults | playhead smooth (worst frozen gap 139 ms), no dialogs |
| O/M flips, 48k WAV + Warmth 0.23 + Volume Match ON | smooth, no dialogs |
| 20 rapid flips down to 250 ms spacing | smooth, no dialogs |
| Export curve 2/4/8/16/24-min WAV → local disk | **linear**: 1259/1250/1266/1283/1304 ms per source-minute; 24-min took **31.3 s** |
| 8-min export → OneDrive-synced folder | 10.07 s vs 10.13 s local — **no OneDrive penalty** |
| 8-min export while playback running | 11.5 s (+14 %), playhead stayed smooth |
| 630-click storm (presets + O/M + toggles) | frame times flat at 5.6 ms p50 before/during/after; JS heap flat; process RAM flat ~300 MB |
| Cold Mastered click on 24-min and 60-min WAV | device-loss banner did **not** fire (this machine decodes inside the 1 s window) |
| Settings device dropdown (healthy state) | fully populated (7 devices), change → "Audio output saved", change-back → "Using system default" — works |

Net: **F4/F7/F9/F11 do not reproduce on healthy hardware.** The owner's
symptoms are load/hardware-dependent — but the *mechanisms* behind F4 and
F7 are real and confirmed in code (below), and they fire exactly on a
machine like the owner's (struggling Focusrite USB device, busy host).

## Verdict per finding

| # | Symptom | Verdict | Mechanism |
| --- | --- | --- | --- |
| F1 | Loop won't start before region | **Confirmed in code** | tick only checks `pos >= end_sec` → seek start (`audio.rs:2080-2092`); no forward branch. Asymmetric with click-past behavior (same branch). |
| F2 | Presets made demo "mono-ish, weird reverb" | **No DSP bug** | Narrowing presets (Tape .93 / Oomph .84 / Warmth .98) expose pre-existing phase junk in a badly-mixed source. `apply_width_stereo` is textbook M/S, clamped [0,2] — inversion impossible (pinned by unit tests). Second, lesser mechanism: preset *switches* crossfade two chains' outputs for 512 frames (~10 ms) — a transient wash, not reverb. |
| F3 | Loop lasso live in Standard, no loop button there | **Confirmed** | `showRegionHint` only changes hint text, not the pointer handlers (`Waveform.tsx:190-224`). Worse: a loop armed in Advanced **stays armed** after switching to Standard, with zero Standard UI to see/disarm it. |
| F4 | O/M flip jumps backward / stutters | **Confirmed mechanism** | New sink seeks to the FE-estimated position captured at IPC-send time (`useTrackMaster.ts:2111-2130` → `audio.rs:2552`); the old sink kept advancing during the swap, so the UI (driven by `sink.get_pos()`) jumps backward by IPC latency + swap work. Invisible on a fast machine (<30 ms); big on a loaded one. The 22 ms swap fade is NOT the stutter source. |
| F5 | Reverts to first track; ~7-10 s to store state | **Confirmed / explained** | `useTrackMaster.ts:791` unconditionally selects `restoredTracks[0]`; `ProjectState` has no `selected_track_id`. The 7-10 s is analysis latency + a **1.5 s** debounce (`useTrackMaster.ts:855-871`) — there is no long save timer. |
| F6 | Advanced becomes sticky/global | **Confirmed** | Single global nav state; entering Standard force-bounces to Advanced when the *selected* track has non-managed edits; only explicit "Back to Standard" leaves. The bounce also **writes localStorage** (`useNavigationMachine.ts:117-122`), so a never-chosen Advanced persists across restart. |
| F7 | Error dialog on A/B; won't dismiss; device dropdown dead | **Confirmed (three stacked bugs)** | (1) `PlaybackDeviceLossDetector` fires after 20×50 ms of frozen-but-playing playhead with **no guard for a busy audio thread**; a cold tier-3 decode on first Mastered click is documented at 1-2 s (`audio.rs:1500-1501`) — inside the trip window. (2) `MarkDeviceLost` is processed *after* the in-flight PlayMaster in the same drain → **pauses the just-started sink** and latches `device_lost`. (3) Dismiss (`clearPlaybackDeviceLost`) clears FE state only; the next 50 ms tick re-arms the banner because the backend latch never clears and the paused sink keeps `is_playing=false`. Dismiss is structurally a no-op. Dropdown: works when healthy; during the latched state, re-picking the *same* device fires no `onChange` (HTML select semantics) → recovery dead-end. |
| F8 | Export button grayed during export | **By design** | FE gate `isExporting || isRendering`; no queue. |
| F9 | 24-min export slow, 3 % CPU, lag builds with clicks | **Not reproduced; pipeline exonerated** | See measurements. Code is honestly linear (7-8 O(N) passes). No listener/sink leaks (all cleanups verified; click storm flat). Owner-side suspects: busy host + Focusrite underruns. Shipping instrumentation so his next report answers it. |
| F10 | Width 0.05 drastic; 0 doesn't revert | **Confirmed — semantics + rendering, no race** | Width is the only advanced slider whose Auto = *preset* value (guarded), not 0 — any touch **replaces** it. The slider thumb renders at **min (0.0)** when Auto (`fields.tsx:135 value ?? min`), so 0.05 looks like a nudge but means "near-mono instead of ~1.11". Slider-to-0 = full mono, NOT Auto; only double-click / clearing the number field / undo restores `null`. No timing race exists (coalescing verified end-to-end). |
| F11 | 4 min ≈ 30 s but 24 min ≈ 15 min | **Refuted as pipeline behavior** | Measured 31.3 s for the 24-min WAV through the same command, owner-like settings. Per-minute cost flat 1.25→1.30 s. OneDrive target added nothing on this machine. Environmental on the owner's box; instrumentation will prove where. |
| F12 | Move up/down noisy; want drag | **Drag already exists** | Full HTML5 drag-to-reorder shipped (`App.tsx:955-991`, wired to the same reducer); the arrows are just always-visible at 0.78 opacity (`App.css:667-695`). The complaint is the visual noise + discoverability. |
| F13 | Album render ignores album title | **Confirmed feature gap** | `plan.title` reaches only `manifest.json`. Per-track files are `NN-<source-stem>.wav` (`album_render.rs:791-794`), continuous file is `album_continuous_<ts>.wav` (`:1004-1010`). The code comments claiming `NN-<sanitized_title>.wav` are drift — no track-title field ever existed. |

## Refuted handoff hypotheses (recorded so they stay dead)

1. **"`set_audio_output_device` early-returns on a same-name re-pick, so
   no-op re-picks can't recover"** — backwards. The `!=` guard
   (`audio.rs:1933-1937`) is a **stale-command race barrier** (caller
   writes the shared name *before* sending; a fresh call always matches and
   falls through to reopen). A fresh same-device re-pick **does** recover.
   The real dropdown dead-end is that re-selecting the already-selected
   option fires no `change` event at all.
2. **Super-linear export pipeline** — measured linear (see table); no
   quadratic code exists; buffers are pre-sized (no growth reallocation).
3. **OneDrive write-stall as the primary export cost** — no penalty
   measured here (Files-on-Demand writes locally). Still plausible on the
   owner's box under sync churn — the per-stage timing log will settle it.
4. **Export/playback lock contention** — render runs on the blocking pool,
   touches no audio-thread state; measured +14 % only, playback smooth.
5. **Leaked sinks / event listeners driving click lag** — all `listen`/
   `addEventListener` cleanups verified; detached swap sinks drain in 22 ms;
   630-click storm shows zero accumulation.
6. **Swap-fade silent lead-in reads as the stutter** — it's 22 ms; the
   audible artifact is the backward position jump.
7. **F10 as a stale-state race** — no race; pure semantics + thumb
   rendering.
8. **"A track-title field should have been used in album filenames"** — no
   such field exists anywhere in the album model; the comment was
   aspirational drift.

## Objective fixes (shipping now, in this order, one commit each)

1. **F7a — device-loss detector hardening (Rust).**
   `PlaybackSnapshot` gains `play_generation: u64`, bumped at the end of
   `handle_play` / `handle_play_master`; the detector treats a generation
   change like a track change (full reset). `MarkDeviceLost` is skipped —
   with a diagnostics line — when a play command completed within the last
   stall window (`last_play_completed_at`), because the frozen-playhead
   evidence predates that command; the skip also bumps the generation so
   the detector re-arms fresh and a *real* dead device still fires ~2 s
   later. `DEVICE_LOSS_STALL_TICKS` 20 → 40 (1 s → 2 s): the code itself
   documents legitimate 1-2 s cold-decode stalls, so a 1 s threshold fires
   inside documented-legit latency — that's a miscalibration, not a taste
   call. Tests: detector generation-reset; drain-order race (PlayMaster
   then MarkDeviceLost → sink stays playing); stale-timestamp path still
   marks loss.
2. **F7b — Dismiss that dismisses (Rust + FE).** New
   `clear_device_lost` command clears the backend latch (sink stays paused;
   Play/space resumes exactly as the owner observed). FE
   `clearPlaybackDeviceLost` invokes it before clearing local state. Tests
   both sides, including the previously-missing "tick after dismiss doesn't
   revive the banner".
3. **F4 — swap position clamp (Rust).** On `is_swap` only, seek the new
   sink to `max(requested, old_sink.get_pos())` so the timeline can never
   run backward on an O/M flip. Non-swap plays are untouched (explicit
   seeks must stay exact). Pure-function extraction + unit test; playback
   behavior only — no rendered bytes change.
4. **F1 — loop forward-seek (Rust).** Add the symmetric branch: playhead
   before the region while loop armed → seek to region start. This removes
   the click-before/click-past asymmetry the owner called out (click-past
   already snaps into the region via the same branch). Mechanical test.
5. **F3 — loop stays Advanced (FE).** Gate the shift-drag region gesture
   behind a real prop (Standard passes false — today `showRegionHint`
   changes only the hint text), and disarm loop + clear the backend region
   on entering Standard so no hidden armed loop survives where no UI shows
   it. Owner stated this preference verbatim ("looping should stay in
   advanced"). Tests: shift-drag in Standard is inert; Standard entry
   disarms.
6. **F5 — selected-track persistence (Rust types + FE).**
   `ProjectState.selected_track_id: Option<TrackId>` (`serde(default)`,
   backward-compatible), saved by `buildProjectState`, honored on restore
   with first-track fallback. Touches shared crate types → iPhone and
   Android bridge lanes run. Round-trip + fallback tests.
7. **F10 — width slider honesty (Rust + FE, no DSP change).**
   `guardrail_readout` gains `effective_auto_width` (the exact
   `trim_width(preset_width)` the chain resolves); the Width field then
   (a) renders the thumb at the resolved auto value while on Auto instead
   of at 0, (b) shows `Auto · 1.11`-style readout, (c) gets a visible
   "reset to Auto" affordance (double-click stays). Warmth/Presence get the
   same affordance with auto=0. This is slider *semantics/labeling* — not
   listening-gated. The bipolar re-model is [OWNER] (below).
8. **F9/F11 — export observability + mechanical wins (Rust).**
   Per-stage `Instant` timing (decode/process/tail/SRC/measure/landing/
   write + output parent dir) logged to diagnostics for track and album
   renders; 1 MiB `BufWriter` for the WAV writers (`wav_writer.rs` **and**
   the album continuous writer at `album_render.rs:875` — byte-identical
   output, snapshot tests prove it; ~135k→~1k flushes on a 24-min file);
   throttle `render:progress` emits to ≥1 % or ≥100 ms (track + album emit
   sites). Next time the owner sees a 15-minute export, the log names the
   stage and the path.
9. **F12 — de-noise reorder arrows (CSS).** Hide `.track-reorder-controls`
   at rest, reveal on row hover / `:focus-within` — drag (which already
   exists and the empty-state hint already advertises) becomes the visible
   primary path; keyboard a11y keeps the buttons. Trivially reversible;
   full-removal option logged below.
10. **Docs.** User-guide "what Width does" explainer (F2 answer);
    album-render comment drift fixed; install-path/silent-install runbook
    note; OPEN_THREADS updated with the proposals below.

Nothing above changes rendered bytes, so **no snapshot regen and no
Spot-Listen line is required**; the fixture lane still runs before merge.

## [OWNER] proposals (recommended defaults; nothing silently picked)

1. **Width/advanced slider re-model (F10 full fix).** Owner's proposal
   (center = Auto, left = subtract from preset, right = add, explicit
   negatives) is sound but is a wire-format change:
   `advanced.width: Option<f32>` currently stores an absolute; old files
   with `width: 0.0` legitimately mean mono. Requires an
   `advanced_schema_version` field + load-time gate before semantics can
   flip, plus reconciling `album_render.rs:317`'s `unwrap_or(1.0)` (see 4).
   **Recommendation:** ship item 7 above now (honest thumb + readout +
   reset affordance) and only do the bipolar re-model if the owner still
   wants it after living with the honest slider — most of the confusion was
   the dishonest rendering.
2. **Per-track view memory (F6).** Add
   `view_by_track_id: HashMap<TrackId, ViewMode>` to `ProjectState`; nav
   machine consults it on selection change; force-bounce to Advanced (for a
   dirty track) still happens but **stops writing** the remembered view —
   only explicit choices persist. Original always-clean invariant
   (2026-06-08 design spec) is per-track and stays enforced.
   **Recommendation:** implement as specced; it's what the owner described,
   and it composes with the F5 fix already shipped. Not shipped now because
   the machine + persistence redesign deserves the owner's eyes on the spec
   first.
3. **Album filename scheme (F13).** Mechanism is small
   (`plan.title` is already on the backend; `sanitize_for_filename` handles
   unicode/reserved/empty; never-overwrite suffixing unaffected). Pick one:
   (i) prefix: `<AlbumTitle>-NN-<source-stem>.wav`, continuous file
   `<AlbumTitle>_continuous_<ts>.wav`; (ii) album-titled subfolder,
   filenames unchanged; (iii) both. Empty title falls back to `untitled`
   (or first source stem — sub-decision). **Recommendation: (ii) subfolder**
   — it organizes multi-album exports, keeps per-track names short, and
   can't collide with the `NN-` test pin; happy to implement same-day on a
   word.
4. **Album-character width bias latent bug.** With character biases active
   (system currently gated OFF) and Width on Auto, `album_render.rs:316-319`
   reinterprets Auto as 1.0 before adding the offset — silently discarding
   the preset's width baseline (e.g. Spatial 1.45). Options: skip the
   offset when width is None ("Auto stays Auto" — one line), or apply the
   offset to the preset baseline. **Recommendation: "Auto stays Auto"** +
   regression test, landed before D7 ever flips the gate.
5. **Export-during-export UX (F8).** Current hard-block is defensible and
   honest. Options: leave as-is + tooltip "an export is already running";
   or a 1-deep queue. **Recommendation: leave as-is with the tooltip**;
   overlapping ~1 GB render jobs is a real cost on long tracks.
6. **Reorder arrows (F12).** Shipped: hidden until hover/focus. Option: 
   remove entirely (loses keyboard reorder — HTML5 drag has no keyboard
   path). **Recommendation: keep as shipped.**
7. **Autosave immediacy (F5 tail).** Debounce is 1.5 s; the owner's 7-10 s
   was analysis latency. Option: fire an explicit autosave at
   analysis-complete so state is durable the moment analysis lands.
   **Recommendation: yes, small and harmless — say the word.**
8. **Device-loss threshold.** Shipped at 2 s (40 ticks). If the owner wants
   snappier true-loss detection on his flaky USB box, 1.5 s is the floor
   I'd defend given the documented 1-2 s legit stalls.

## Deferred / backlog (sized, not shipped)

- **Arc-backed decoded PCM** — every play/swap clones the full PCM into
  `decoded_cache` (`audio.rs:2462-2466`); ~250 MB+ alloc churn per O/M
  click on long tracks. ~150 lines across the two source types; the
  measured click-storm showed no user-visible cost on healthy hardware, so
  this is a polish item, not a fix.
- **Backend render-in-flight gate for previews** — the F8 gate is FE-only
  for previews; a Rust-side "latest wins + cancel in-flight" guard would
  harden knob-storm behavior (~50 lines, reuses cancel infrastructure).
- **`flush_render_tail` front-drain memmove** — one full-buffer memcpy per
  export to strip ~3 ms; measurable but small next to I/O.
- **Device-lost as derived staleness (timestamp) instead of a latched bit**
  — eliminates the split-brain class F7 belongs to; wave-scale refactor.
- **`SetOutputDevice` reopen hardcodes 44.1 kHz** + pause→re-pick→Resume is
  a silent no-op (nothing re-appended) — adjacent bugs worth a small
  follow-up ticket.
- **Legacy app-data identifiers** (`com.albummasteringstudio.app`,
  `local.album-mastering-studio`) still under %APPDATA% — cleanup candidate.

## Environment notes (Part 0 closure)

- App data (session/logs/renders) lives under
  `%APPDATA%\com.yesmaster.desktop` — Roaming AppData, **not** in the
  OneDrive Known-Folder-Move set. Exports go wherever the user points the
  dialog; this machine's `Documents` **is** OneDrive-redirected, but no
  write penalty was measurable today.
- No custom window-close handling exists (single window, no
  `on_window_event`); orphaned `msedgewebview2` processes are a runtime
  concern, not app code.
- Install runbook: NSIS installs per-user to `%LOCALAPPDATA%\YES Master`;
  silent install `& ".\YES Master_0.1.0_x64-setup.exe" /S`; silent
  uninstall `& "$env:LOCALAPPDATA\YES Master\uninstall.exe" /S`.

## Verification

Fast lane per CLAUDE.md (`npm test`, `npm run build`,
`npm run build:windows`, `cargo fmt --check` / `clippy -D warnings` /
`cargo test` ×2 under `target\codex-rc`), plus the iPhone and Android
bridge lanes for the shared-type commits (F5, F7a/b, F10), plus the
`AMS_RUN_REAL_FIXTURE=1` slow lane before merge (WAV-writer path touched,
even though bytes are pinned identical). CI is billing-blocked; all lanes
local.
