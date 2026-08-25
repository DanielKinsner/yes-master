# U18 — iPhone impossible states + transport UX: Mac-session execution plan

**Date:** 2026-08-25 · **Status: READY — waiting for a Mac session.**
**Unit:** U18 from `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`
(owner-unlocked 2026-08-25; see `docs/OPEN_THREADS_AND_DECISIONS.md`).

**Why this document exists.** U18 was analyzed and decision-completed on the
Windows machine on 2026-08-25 — the same session that executed U19 (Android).
Swift does not compile on Windows, and this program does not ship unverified
code, so the analysis, the decisions, and the exact work plan are recorded
here for a session on Dan's MacBook to execute mechanically. Every claim
below was read from current source this session; line anchors are for commit
`f768d50`-era files.

**Boundaries (unchanged from the unit):** no distribution work, no parity
claim, no desktop-beta dependency. The U19 Android remediation is the
sibling: mirror its *behavior*, not its widget set.

---

## 1. Current-state facts (verified by reading, 2026-08-25)

All in `apps/iphone-native/YESMasterNative/`:

| # | Fact | Where |
|---|---|---|
| F1 | `AuditionController` publishes `isPlaying`, `isAnalyzing`, `isRendering`, `canPlay` (computed, NOT published — derived in views), but **no position, no duration** | AuditionController.swift:236-238, 89-93 |
| F2 | `LiveAudioEngine` already exposes `positionSeconds`, `durationSeconds`, `seek(toSeconds:)` — the controller uses them internally (restart-from-top, EOF) but never surfaces them | AuditionController.swift:264-265, 281 |
| F3 | The hero "playVisual" is **decorative**: gradient bar + concentric circles; no playhead, no time readout, no seek anywhere in the app | ContentView.swift:205-275 |
| F4 | The 10 Hz `positionTimer` tick only checks EOF; it publishes nothing | AuditionController.swift:279-298 |
| F5 | **Impossible state 1:** the Original/Mastered switch is never disabled — with no track imported, tapping "Mastered" flips engine bypass on nothing and sets status "Switched to mastered at the same spot." (`selectSide` has no guard beyond same-side) | ContentView.swift:409-448; AuditionController.swift:378-384 |
| F6 | **Impossible state 2:** Create Master is disabled only while `isRendering`; with no track it is tappable and error-driven (`.masterUnavailable` after the tap) | ContentView.swift:587-611; AuditionController.swift:480-484 |
| F7 | A11y: the play button is icon-only (no label); the Intensity `Slider` has no `accessibilityLabel`/`Value` (the % lives in a separate `sectionTitle` meta); style/loudness/side buttons don't expose `.isSelected`; the Volume Match `checkboxButton` is a drawn rect + Button with no toggle semantics | ContentView.swift:234-271, 504-540, 409-448, 316-341 |
| F8 | Reduced motion is already respected for heroPulse, play-button scale, and `ProcessingSpinner` (also `accessibilityHidden`); the `processingBanner` transition is NOT gated | ContentView.swift:56, 93-98, 214, 267, 302, 723-729 |
| F9 | No haptics exist anywhere in the app today | (grep: no UIImpactFeedbackGenerator / UINotificationFeedbackGenerator) |
| F10 | Test seams exist and are good: `MasteringRenderer` protocol + `FakeRenderer`, fake stream/output in `makeLoadedController()`; tests await `analysisTask`/`renderTask` | AuditionController.swift:8-15; YESMasterNativeTests/AuditionControllerTests.swift |
| F11 | Interruption/route handling is complete and status-driven (`handleInterruptionBegan/Ended`, `handleAudioRouteLost`) | AuditionController.swift:305-374 |

## 2. Decisions (made; do not re-litigate)

- **D-A. Published transport state.** Add to `AuditionController`:
  `@Published private(set) var playbackPositionSeconds: Double = 0` and
  `@Published private(set) var playbackDurationSeconds: Double = 0`.
  Duration set on successful `engine.load` (and reset on import/failure);
  position updated in `handlePlaybackTick` (the existing 10 Hz timer), on
  `seek`, on `stopPlayback`, and reset on import.
- **D-B. Gates.** Two computed vars, published state only:
  - `canSelectSide` = `importedTrack != nil && analysisResult != nil && !isAnalyzing`
  - `canCreateMaster` = `importedTrack != nil && analysisResult != nil && !isAnalyzing && !isRendering`
  `selectSide` and `createMaster` **early-return without touching the engine
  or error state** when gated (the UI will be disabled anyway; the guard makes
  the impossible state unreachable even programmatically). This matches
  Android, where the A/B segments and transport disable unless `Ready`.
- **D-C. Real transport UI.** Keep the hero art. Under `playVisual`, add a
  seek row: a `Slider` (0…duration) + `m:ss / m:ss` readout. Drag semantics
  mirror Android's `AuditionCard`: local drag state so the thumb tracks the
  finger, one `engine.seek` on release via a new controller
  `func seek(toSeconds:)` (clamps to `0...playbackDurationSeconds`, updates
  the published position immediately). Slider disabled unless
  `canPlay && playbackDurationSeconds > 0`.
- **D-D. Accessibility.** Mirror the Android semantics commit `bf8e402`:
  - Play button: `.accessibilityLabel(importedTrack == nil ? "Import audio" : (isPlaying ? "Pause" : "Play"))`.
  - Seek slider: label "Playback position", `.accessibilityValue("0:32 of 2:10"` — same `formatClock` wording as Android).
  - Intensity slider: label "Intensity", value "NN%".
  - Style/loudness/side buttons: `.accessibilityAddTraits(selected ? [.isSelected] : [])`.
  - Volume Match: `.accessibilityAddTraits(.isButton)` → replace with proper
    toggle semantics: `.accessibilityLabel("Volume Match")` +
    `.accessibilityValue(active ? "on" : "off")` + `.isToggle` trait
    (iOS 17+; on 16 fall back to value-only — deployment target is 16.0).
  - Status: `statusText` changes should be announced — post
    `AccessibilityNotification.Announcement` (iOS 17) / `UIAccessibility.post(notification: .announcement…)`
    from a `statusText` `didSet` hook **only when it changes** and only for
    interruption/EOF/analysis-done messages (not every keystroke of state).
  - Disabled reasons: add computed `createMasterDisabledReason: String?`
    ("Import a track first." / "Analyzing — just a moment." / nil) rendered
    as small text under the button AND as `.accessibilityHint`.
- **D-E. Haptic restraint = no haptics.** The app has none today (F9); U18's
  "haptic restraint" is satisfied by adding **none** and recording that as
  deliberate. Do not add success buzzes.
- **D-F. Reduced motion.** Gate the `processingBanner` transition
  (`.transition(...)` at ContentView.swift:302) behind `reduceMotion` —
  everything else is already gated (F8).
- **D-G. No new test dependency.** No ViewInspector/snapshot library.
  Controller-level XCTests prove the gates and transport; the VoiceOver pass
  is a manual checklist on the Mac/device (record results in this doc when
  done). This mirrors the U19 lane-boundary decision (Robolectric proves
  semantics; real TalkBack stays a device gate).

## 3. Work order (small commits, each with its tests)

1. **Controller: published transport + gates** (`AuditionController.swift`)
   — D-A + D-B, no UI change. Tests (extend `AuditionControllerTests` using
   `makeLoadedController`):
   - duration published after load; position 0 on import.
   - `seek` clamps below 0 / above duration; forwards to the engine; updates
     the published position while paused.
   - gate matrix: `canSelectSide`/`canCreateMaster` false with no track,
     false while analyzing, true when ready, `canCreateMaster` false while
     rendering; **negative control:** `selectSide(.mastered)` with no track
     leaves `selectedSide == .original` and records **zero** engine calls
     (assert on the fake stream), `createMaster` while gated schedules no
     render task and sets no error.
   - interruption-resume still publishes a fresh position (F11 regression).
2. **UI: real seek row + gated switch + gated Create Master**
   (`ContentView.swift`) — D-C + the `.disabled(!controller.canSelectSide)` /
   `.disabled(!controller.canCreateMaster)` wiring + visible disabled reason.
3. **UI: accessibility pass** — D-D + D-F. (No mechanical test per D-G; the
   manual VoiceOver checklist below gates completion.)
4. **Docs + ledger** — update `docs/IPHONE_APP_OVERVIEW.md` transport
   description, mark U18 Complete in the quality-plan ledger with commits,
   and append VoiceOver-checklist results here.

## 4. Verification lanes (Mac)

```bash
cd apps/iphone-native && xcodegen generate
# unit tests (simulator; scheme YESMasterNative from project.yml)
xcodebuild test -project YESMasterNative.xcodeproj -scheme YESMasterNative \
  -destination 'platform=iOS Simulator,name=iPhone 16'
cd rust && cargo check --all-targets && cargo test
```

No shared-crate Rust changes are expected; if any happen anyway, run the
desktop + Android lanes too (CLAUDE.md Verification).

**Manual VoiceOver checklist (10 min, simulator or device):** with VoiceOver
on — (a) empty state: play button announces "Import audio"; Mastered/Create
Master are announced dimmed; (b) after import+analysis: side buttons announce
selection; seek slider announces "Playback position, 0:00 of <length>" and is
adjustable; Intensity announces name + percent; Volume Match announces
on/off; (c) trigger an interruption (siri/timer): the pause status is
announced without touching the screen.

## 5. What NOT to do

- Do not touch `LiveAudioEngine`/the Rust facade — everything needed exists (F2).
- Do not add a Mastered-side auto-switch, autoplay, or render-on-import.
- Do not restyle the hero art or "premiumize" further — this unit is states
  and semantics, not visual design.
- Do not enable AC-5/Phase-B anything (still gated OFF, D7).
