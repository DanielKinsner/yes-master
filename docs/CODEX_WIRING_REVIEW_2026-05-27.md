# YES Master — Wiring & Cleanup Review (for Codex)

**Date:** 2026-05-27
**Scope:** Frontend (`src/`) wiring + dead code + bugs, with a backend (`src-tauri/`) command-wiring pass. UI/visual polish at the end.
**Branch state at review:** `main`, working tree dirty (`docs/*.md`, `src/hooks/useTrackMaster.ts`).

## How this was verified

- `npx tsc -b` → **clean** (exit 0).
- `npx vitest run` → **124/124 passing**, 15 files.
- This means everything below is cleanup/correctness-hardening, not a broken build. The dead code survives compilation because `noUnusedLocals` (`tsconfig.json:18`) does **not** flag dead `export`s or unused keys on a returned object — that is the exact blind spot these items live in. Consider adding `knip` or `ts-prune` to CI.

> Re-evaluation note: an earlier draft called the right-rail bottom a large "dead-zone." Screenshots with `ADVANCED CONTROLS` expanded show the rail is mostly full; the remaining gap is just the space above the bottom-sticky export group when rail content is short. Softened accordingly below.

## Post-review status

This review is now historical evidence, not a literal open queue. The safe-delete
items in section 1 were consumed after review:

- `65381d5 chore: remove unused album and rail wiring` removed the unused rail
  panels, orphaned advanced state, unused album intent updater, and old simple
  album render path.
- `58c25d7 chore: remove realtime diagnostic counters` removed
  `get_diag_counters` and the temporary diagnostic API/backend wiring after the
  realtime responsiveness sweep was accepted.

Still treat section 2 and section 3 as live risk areas unless newer code proves
otherwise: loudness-target control semantics, live-chain update predicates,
duplicate live-loudness readouts, and A/B/live-chain regression coverage.

---

## 1. Dead code — safe to delete

| # | Item | Location | Evidence | Confidence |
|---|------|----------|----------|------------|
| 1.1 | `LevelsPanel` component | `src/components/RightRail.tsx:507-577` | Exported; **zero** render sites or tests (`grep` finds only the definition). ~70 lines. | High |
| 1.2 | `StereoWidthGauge` component | `src/components/RightRail.tsx:252-395` | Exported; referenced only in its own comments + CSS, never mounted. ~145 lines of SVG. Its doc comment claims "dragging Width moves the needle in real time" — but nothing renders it. | High |
| 1.3 | Rail threshold constants | `src/components/RightRail.tsx:44-46` | `CLIP_THRESHOLD_DBFS`, `HEADROOM_WARN_DBFS`, `SILENCE_FLOOR_DBFS` are consumed **only** by `LevelsPanel`. Die with 1.1. | High |
| 1.4 | `advancedOpen` / `setAdvancedOpen` / `toggleAdvanced` | `src/hooks/useTrackMaster.ts:165, 1465-1467, 1701, 1739` | Advanced panel uses a native `<details>` (`App.tsx:1937`), so this state is orphaned. Only surviving reference is a test mock (`App.album-export.test.tsx:195-196`). | High |
| 1.5 | `updateAlbumIntent` | `src/hooks/useTrackMaster.ts:1680-1685, 1749` | Exported from the hook; **zero** real callers (album edits route through `setAlbumArc/Intensity/Title`). Only a test mock references it. | High |
| 1.6 | `engine::render_album_master` (Tauri command + fn) | `src-tauri/src/engine.rs:320`, registered `src-tauri/src/lib.rs:67` | No frontend `invoke("render_album_master")` and no `api.ts` method. The album path uses `plan_album` + `render_album_plan` (see hook comment at `useTrackMaster.ts:1026-1031`: "the older simple-album hook was removed"). Only `preview-mock.ts:269` still mocks it. | High |
| 1.7 | `get_diag_counters` chain | `src-tauri/src/lib.rs:80`, `audio::get_diag_counters`, `api.ts:186` `getDiagCounters` | API method + backend command exist but have **no caller** in app code (only def + mock + comment). These are the "temporary realtime diagnostic counters" already flagged for removal in `docs/APP_BEHAVIOR.md` (Current Gaps #3) and `CLAUDE.md` jump-fix #4. | High — but gated on the realtime sweep being verified clean first (see that doc). |

**When deleting 1.1/1.2:** also remove the now-orphaned mock cases and any CSS rules keyed to `.levels`, `.stereo-width-panel`, `.stereo-gauge-*` in `App.css` (the comment at `App.css:1142` references the meters column).

---

## 2. Redundant / duplicated logic

### 2.1 Two loudness-target controls writing the same field (UX + wiring)
- Rail `LUFS target` `NumberField` → `update("lufs_offset_db", v)` at `src/App.tsx:1957-1965`.
- Center `LOUDNESS TARGET` dropdown → `onAdvanced({ ...advanced, lufs_offset_db: profile.lufs })` at `src/App.tsx:1676`.
- **Both write `lufs_offset_db`, but only the center control flips `delivery_profile` → `"custom"`** (`App.tsx:1672-1674`). The rail slider mutates the value without the profile flip.
- This is why the screenshots show `DELIVERY PROFILE: Custom` while `LOUDNESS TARGET` reads "Streaming default (-14)" — a presented contradiction. It's intentional per the B7 comments, but it's the highest-confusion surface in the app and editing the same value from two places with **divergent side effects** is a real inconsistency.
- **Recommendation:** route both through one handler so the profile-flip behavior is identical regardless of entry point, or drop one control. Decision needed from Dan before changing calibration semantics.

### 2.2 "Should I push to the live chain?" predicate copy-pasted 3×
Same `loadedKindByTrack[id] === "master" || (loadedTrackId === id && kind !== "source")` logic in:
- `updateSettings` — `src/hooks/useTrackMaster.ts:673-683`
- `applyUserPreset` — `:1520-1531`
- `restoreSnapshot` — `:577-581`

Extract `isPlayingMasterFor(id)` to kill drift risk. Pure, easily unit-tested.

### 2.3 Duplicate live-loudness readout
- `BottomStatusBar` renders `LIVE PEAK` / `LIVE LUFS` (`src/App.tsx:147-154`).
- `MASTER OUT` panel renders "Live Peak" + "Since Play" from the same `transport` fields (`RightRail.tsx:227-243`).
- Both bottom-bar values read `—` in every screenshot (playback stopped). Pick one home; the bottom bar duplicates the meter that's already on screen.

---

## 3. Bugs / correctness risks

### 3.1 `loadedKindByTrack` is populated asymmetrically (watch item)
- The map is only ever **set** to `"source"`/`"master"` in `playWithKind` (`useTrackMaster.ts:1204`) and **deleted** on track removal (`:920-924`). It is never reset when the user toggles back to Original without re-loading.
- Several gates treat `kind !== "source"` as "probably master" (`:682`, `:1530`). Combined with the tick-driven `loadedTrackId`, the logic is "belt-and-suspenders" by the author's own comments — which usually means a past "edits not audible until I re-toggle" bug lived here.
- **Not confirmed broken**, but it's the highest-entropy part of the live-chain wiring. Worth a focused test: edit a control while paused-on-Master, while playing-Original, and right after an undo, asserting `update_chain` fires (or doesn't) as intended.

### 3.2 Insight chevron implies expand when there's nothing to expand
- `AnalysisSummary` renders a `⌄` chevron unconditionally (`App.tsx:1017`) inside a `<details>`, but the body only exists when `rest.length > 0` (`:1019`). With a one-line insight, clicking toggles nothing.
- Low severity; hide the chevron when `rest.length === 0`.

### 3.3 `Transport` receives both `loop` and `loopEnabled`
- `loop={transport.loop}` and `loopEnabled={!!selectedRegion}` (`App.tsx:558-565`). Two loop-ish props is easy to mis-wire later. Confirm both are genuinely distinct ("looping active" vs. "a region exists") and named to reflect that.

---

## 4. Visual / UX polish (lower priority)

1. **Surface the unused panels into the rail gap.** Items 1.1/1.2 are fully-built `LEVELS` and `STEREO WIDTH` panels that already exist. If they're wanted, mounting one in the space above the sticky export group (visible in screenshot 2 between `DELIVERY FORMAT` and `TOOLS`) both fills the gap and recovers the work. Otherwise delete per §1.
2. **`ACTIVE 5` pill reads like a debug counter** next to the `ANALYZED` status pill on the A/B row (`App.tsx:766-771`). Note: the `live: 7/7` badge bottom-center is `import.meta.env.DEV`-gated (`App.tsx:1798`) and is tree-shaken from production — leave it. `ACTIVE n` is **not** gated and ships; consider whether it should.
3. **A/B row is crowded:** `Original | Mastered` + 2 checkboxes + 2 pills share one flex row. Group the transport toggles apart from the status pills.
4. **Insight card** spans full center width for a single line of text; tighten to content width (and see 3.2).

---

## 5. Confirmed healthy (no action)

- Drag-drop import, autosave/session restore, explicit Save As / Open Project.
- Undo/redo incl. live re-push on restore (`useTrackMaster.ts:567-636`).
- Volume Match / Preview LUFS mutual exclusion (`:1402-1463`).
- Warning-aware export-review state machine (`RightRail.tsx:60-86`).
- rAF-gated, latest-wins `update_chain` dispatcher (`:296-340`).
- Backend command registration otherwise matches `api.ts` (all invoked commands exist; `open_output` is wired at `App.tsx:2745`).

---

## Suggested order for Codex

1. **Pure deletes (no behavior change):** §1.1–1.5 frontend dead code, then §1.6 backend `render_album_master`. Run `tsc -b` + `vitest` + `cargo test` after.
2. **§1.7 diag counters** — only after confirming the realtime sweep is verified clean (per `APP_BEHAVIOR.md` gap #3).
3. **§2.2 / §2.3** — mechanical de-dup, add the `isPlayingMasterFor` unit test.
4. **§2.1 + §3.1** — these touch audible/loudness behavior; get a decision from Dan and add tests before changing.
5. **§4 visual polish** last.
