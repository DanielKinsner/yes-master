# 2026-07-04 Owner Smoke-Test Findings — Orientation for Independent Review

The owner ran the first real hands-on smoke test (`docs/OWNER_SMOKE_TEST.html`)
against the installed 2026-07-04 build and reported the findings below. Fable
then formed opinions on likely causes. This document is an **introduction to
the problems, NOT a work order.**

## How to use this document (read first)

1. Read the owner's findings (Part 1) — ground truth about *symptoms*.
2. Read Fable's analysis (Part 2) — a **lead, not a verdict**. Written the
   same day; `[CONFIRMED]` items were read in code, `[HYPOTHESIS]` items were
   not debugged and some will be wrong. Deliberately loose to avoid handing you
   tunnel-vision blinders.
3. **Independently validate** every finding and stated cause — including
   Fable's — against current code. Reproduce where you can. Current code
   reality beats this prose (AGENTS.md).
4. Then write **your own** plan doc (`docs/plans/2026-07-05-...`) with the
   sequencing and fixes *you* decide on. **BRANCH FIRST** (the owner pushes
   local state without checking), small commits.

Several findings are design/taste decisions the owner has opinions on but that
touch shipped behavior — marked `[OWNER]`. Design the options and surface them;
don't silently pick for him. The two-tier DSP policy applies: objective
wrongness → fix + proving test (+ snapshot regen + Spot-Listen line if rendered
bytes change); taste-shaped → listening note only. Preset *calibration* changes
are listening-gated; slider *semantics/labeling* fixes are not.

**Reinstall before validating.** The tested build predates the diagnostics log
(merged later that day, `da532c3`). Rebuild + reinstall (`npm run
build:windows`, then the NSIS setup with `/S`) so reproductions land in the
app-data logs and Help → Save diagnostics report captures device-loss / render
/ decode events. Several findings below (B, D) get far easier with a real
diagnostics report.

There is a **screenshot** the owner referenced for F7 (the error dialog during
A/B) not available to this session — ask for it or reproduce the state before
acting on the dialog's exact wording.

---

## Part 0 — Environment note (add a line to dev docs)

The owner reported the whole PC lagging. Two checks, both real:

- **Fable's check (this session):** NOT dev servers — 4 idle node procs, no
  preview servers, no build processes; RAM fine (78% free of 130 GB). Real
  suspects: **OneDrive sync churn** (this repo lives inside OneDrive; heavy git
  + an `npm install` churns thousands of files that re-sync), **19
  `msedgewebview2` processes** (Tauri uses WebView2 — likely orphaned instances
  from repeated app open/close), and **`vmmemWSL` holding ~5.8 GB**.
- **Earlier observation:** concurrent Claude + Codex agent processes and Adobe
  Creative Cloud were running, and `audiodg.exe` (the Windows audio engine) had
  accumulated an unusual amount of CPU time.

Net: the raw perf/lag numbers from this session are **not clean-room**. The app
must still behave honestly under a struggling audio stack and a busy host, but
don't over-index on absolute timings. Two things worth confirming as real:
(a) does closing the YES Master window fully **reap its WebView2 children**, or
do they leak? (b) is app-data / export output landing **inside OneDrive** and
stalling on sync (see D)?

---

## Part 1 — Owner's findings (verbatim)

Reproduced word-for-word as requested; numbering added, wording unchanged.

**F1 — Loop region, playhead before region.**
> when loop is on, if playhead is before selected region loop wont happen
> until it the playhead reaches selected region. when you click PAST selected
> region, it loops, clicking before region allows. B never inherits a's
> looops but thought id mention

**F2 — Width in presets / what width is doing.**
> id like to check how much "width" is in each preset. also what width is
> truly doing.... i got a really old friends old demo song and granted hes
> bad at mixing... but when the presets were on it it brought DOWN the width
> and gave it a weird reverb... like it made it sound mono ish?... granted its
> the FIRST time ive ever heard it do that and im pretty sure it might just be
> inverting an already wide signal or something... just wanna make sure thats
> done right

**F3 — Loop lasso usable in Standard; no loop button; looping should stay Advanced.**
> i noticed the mouse command that highlights an area for looping is usable in
> standard mode however theres no "loop" button under the playhead. i think
> looping should stay in advanced.

**F4 — Original/Mastered flip jumps backward / stutters.**
> the single biggest thing i heard thats concerning so far is the flip between
> "mastered" and "original" it jumps backward in time/ stutters... jumping
> around on timeline is fine, cancel button is working fine, txt files cant
> even be imported when changing their names somehow... cant drag and drop cant
> view it with windows explorer. overcooking honest is there all along the way

**F5 — Session restore timing + always reverts to first track.**
> restarting a session and it storing what you last did... works for the most
> part. it seems to take between 7-10 seconds after a track is analyzed to
> store information about the new state. also the app always reverts to the
> first track, but remembers where the other tracks settings were, however.
> Lets say a track was in standard mode, and others were in advanced. let me
> try and break it down... not sure if this matters but let me try

**F6 — View mode: Advanced becomes sticky/global for all tracks.**
> lets say track 1 is in standard, track 2 is in advanced, track 3 is in
> standard. so you open the app and because track 1 is in standard it the
> first UI you see is standard mode. ... you can click to track 3 and it stays
> in standard mode... the second you click to track 2... the UI then becomes
> advanced for ALL tracks. even when reopening the app after letting it save
> its state with the 8-10 seconds or whatever. the SETTINGS remain the same...
> but the UI changes and stays in advanced for every subsequent track. i
> personally think it needs to save the ui state the user was in because its
> just a signal that their song hasnt changed from what they potentially
> liked. the second they choose advanced... well thats fine... but prior to
> that all states need to remain in the state they left it in.

**F7 — Timeline choke on A/B; device dropdown dead; error dialog won't dismiss.**
> the only choke on the timeline comes original/mastered a/b testing. you can
> see in the screeenshot the error it shows. then in settings if you click to
> change your device the drop down does not work. on that error dialog i could
> not click dismiss, or rather the dialogue didnt go away.. and the device was
> always working anyway. i could just hit space bar or press play and it
> started up on the same audio device

**F8 — Export button grayed out during export.**
> the export button is literally grayed out during export so i cant start an
> export during an export... not sure if thats good or bad.

**F9 — 24-minute export very slow; low CPU; playback degraded; lag builds up.**
> it took a very long time to export a 24 minute single wav file. i noticed
> during that export the cpu was only using 3-3.5% of the power, about 4 gigs
> of ram and absolutely no gpu. songs can still play during export... but it
> was interesting the song that was exporting was not playing well.
> unfortunately ive noticed this about my particular i/o usb device... it can
> randomly take a crap so this should be looked into but not necessarily taken
> unwarranted action on. the export itself did not have the weird artifacts
> and lag the timeline was showing. it was still lagging even after the export.
> (the 24 minutes track)... when reopening the app, after the 24 minute track
> reanalyzed it was no longer lagging. i was able to replicate the lag by
> exporting, but then had to click a ton of stuff in order for it to start
> building up the lag and it got progressively worse. it seems to resolve
> itself over time but unsure. you can't save over another file in windows
> because windows itself warns you.

**F10 — Width slider on "auto": 0.05 changes sound drastically, 0 doesn't revert.**
> something rather concerning and i dont know what it could be. i found, lets
> say everything is on "auto" in advanced controls... then i slide width only
> .05... the sound changed drastically. then i slide it to 0... the sound
> doesnt go back to before i slid it to 0.05... it then you switch the
> "refresh" icon' or "undo all" the sound snaps back to something way better.
> this COULD be the track it could also be that its creating 0 width or
> lessening width with it set to 0.05... but from what i understand. the width
> and all adanced controls shouldnt override the presets innate width or other
> setting... essentiall it seems like "auto" should be on by default... if we
> want to subtract any of those settings a 0 value should still be auto and a -
> negative value needs to be an option incase a user wants to remove width or
> presence or warmth etc etc. but it needs to be explicitly obvious thats what
> someones doing... a 0.05 positive value shouldnt represent LESS of
> something... so maybe if thats whats actually going on, we need those sliders
> set at auto in the middle. negative values from the preset go to the left
> starting at -.05 and positive values go to the right +0.05 only ever adding
> to the preset.

**F11 — Export time super-linear (4 min ≈ 30 s, 24 min ≈ 15 min).**
> interesting that a 4 min track takes about 30 seconds but the 24 minute
> track took like 15 minutes.

**F12 — Move up / move down UX is noisy; should be drag-to-reorder.**
> i dont love the move up / move down ui/ux. it looks noisy. those should just
> be draggable into position... thats intuitive

**F13 — Album render adds no prefix/suffix, not even the album title.**
> on rendering an albumn. it renders fine. but it doesnt add a prefix or suffix
> to the exported tracks at all. not even the album title you give it in the UI

Adjacent asides the owner made in passing (not bugs, captured so they aren't
lost): "B never inherits A's loops" — the `6497a98` loop-region fix is
**confirmed working** by the owner. "overcooking honest is there all along the
way" — the overcook-honesty guarantee is **confirmed present**. ".txt files
can't be imported even when renamed / can't drag-drop / can't view in
Explorer" — the owner is noting the hostile-import guard behaves correctly.

---

## Part 2 — Fable's analysis (leads, not verdicts — VALIDATE independently)

`[CONFIRMED]` = Fable read the code and the mechanism is real.
`[HYPOTHESIS]` = plausible, unproven — reproduce first.

### A. Width — strongest, most actionable cluster (F2 + F10)

`[CONFIRMED]` **The Width slider is an absolute M/S side-scale override, not an
offset.** `src/components/AdvancedPanel.tsx:334-342` renders Width as
`min=0, max=2, step=0.05`, value `a.width` (`advanced.width: number | null`).
`src-tauri/src/dsp.rs:977` uses
`settings.advanced.width.unwrap_or(guarded_preset_width)` as the M/S
`side_scale`:
- `advanced.width = null` → **auto** = preset baseline width.
- `advanced.width = 0.05` → `side_scale = 0.05` → **near-mono**, *replacing*
  the preset baseline entirely.
- `advanced.width = 0.0` → full mono (even more collapsed — why "0 doesn't
  revert").
- Undo/refresh → `null` → auto → snaps back to the preset value ("snaps back
  to something way better").

The owner's diagnosis is essentially correct. `apply_width_stereo`
(`dsp.rs:1409`) is textbook-correct M/S (`side_scale=1` exact identity, `0`
mono, `>1` widen) — the DSP is fine; the **slider semantics/labeling** are the
problem. The owner's proposed model (center = auto/preset, left = subtract,
right = add, negatives explicit) is sound. **Not unique to Width:** Warmth
(`min=0,max=1`) and Presence/Air (`min=0,max=1`) at `AdvancedPanel.tsx:343-360`
share the same "any touch replaces the preset with an absolute" pattern.
Consider a general "auto-centered advanced control" model rather than a
one-off. `[OWNER]` — this changes shipped interaction and possibly the
`advanced.*` wire meaning; design the options and confirm before landing.

`[CONFIRMED]` **Preset width baselines** (`src-tauri/src/dsp.rs` preset table):
Universal 1.11, Clarity 1.09, Tape **0.93**, Spatial 1.45, Oomph **0.84**,
Warmth 0.98, Punch 1.04, Loud 1.03, Custom 1.0. Presets below 1.0 narrow the
image. On a badly-mixed demo with fake-wide / out-of-phase content, a
narrowing preset (Tape/Oomph/Warmth) collapses the side signal toward mono and
exposes phase cancellation → "mono-ish, weird reverb." It is **not inverting**
— it is narrowing (correct math, jarring on a pathological input).
**Deliverable that answers the owner directly:** the preset fingerprint harness
already measures width per preset (`src-tauri/tests/preset_fingerprint.rs`,
report via `write_owner_fingerprint_report`, see TESTING.md) — regenerate the
report, present the width/correlation column per preset, and add a short "what
width does" explainer to the user guide. Any *retune* of the numbers is
listening-gated; measuring + explaining is not.

### B. Original/Mastered A/B — likely one root cause behind F4 + F7

`[HYPOTHESIS — investigate first]` The O/M swap makes the playhead **stall for
~1 s** (swap fade + re-prep of the new sink at the preserved position), and
that stall **trips the device-loss detector into a false positive.** Last
session shipped `PlaybackDeviceLossDetector` (`src-tauri/src/audio.rs`): ~20
ticks / ~1 s of a playing-but-frozen playhead → `playback:device-lost`. The
owner's tells all line up: "jumps backward/stutters," an error dialog appears
on A/B, "the device was always working anyway," and pressing space just
resumes on the same device. If true, the fix space includes (a) marking the
swap so the detector ignores it (it already ignores track-change / seek /
pause / loop), and (b) the backward-jump/stutter itself — the new sink may seek
to a slightly stale `start_position_sec` captured before the swap, or the
silent lead-in of `build_swap_fade` reads as a stutter. Look at `handle_play` /
`handle_play_master` swap paths + `play_sink_from_start_position`, and how the
frontend computes the position passed on an O/M toggle (`useTrackMaster.ts`
`playWithKind` / `estimatedPlaybackPositionSec`).

`[HYPOTHESIS]` **Banner won't dismiss + Settings device dropdown dead** (F7).
Two candidates: the device-loss banner may only clear on a *successful device
re-pick* (last session's design), so a false-positive loss where the device
never actually changed has no clean dismiss; and `set_audio_output_device`
(`src-tauri/src/audio.rs`) only reopens the stream when the selected name
*differs* from the current — a no-op re-pick returns without recovering. The
Settings dropdown itself not responding may be a separate frontend issue.
Reproduce independently; may or may not be entangled with the detector.

### C. View mode: Advanced is sticky/global (F6) + reverts to first track (F5)

`[CONFIRMED]` **`view` is a single global state, not per-track, and Advanced is
one-directional.** `useNavigationMachine.ts` recomputes on `context-changed`
(fires when `hasNonManagedEdits` changes), and `hasNonManagedEdits` is computed
from the *currently selected* track (`App.tsx:239`). Selecting a track with
advanced edits force-bounces the view to Advanced; selecting a plain track
afterward does **not** bounce back — by design, only an explicit "Back to
Standard" leaves Advanced (see the `isExplicitReturn` comment at
`useNavigationMachine.ts:30-41`). So visiting one Advanced track pins the whole
UI to Advanced. The owner wants **per-track view memory** (a track shows the
view it was left in until the user explicitly chooses Advanced). That is a real
change to the navigation machine + its persistence (`lib/view-mode`,
`ProjectState`). `[OWNER]` — reconcile with whatever reason Advanced was made
sticky (probably: don't yank someone out of Advanced mid-edit).

`[CONFIRMED]` **Always reverts to first track on reopen** (F5). Restore does
not persist the *selected* track id — it selects the first track. Per-track
settings restore fine (keyed by track id); only selection + view don't. Pairs
with the per-track-view request.

`[HYPOTHESIS]` **7–10 s to save new state** (F5). Autosave likely fires on a
debounce after analysis, possibly gated behind waveform rebuild. Find the
trigger (`useTrackMaster.ts` → `api.autosaveSession`), check the debounce
interval and what it awaits. If it's a fixed long debounce, shortening it (or
saving immediately on meaningful state changes) is cheap.

### D. Export performance (F9, F11) — probably TWO separate problems

`[HYPOTHESIS]` **Super-linear render time.** 4 min ≈ 30 s but 24 min ≈ 15 min is
~5× worse per-minute — not linear. With **3% CPU, no saturation**, this smells
**I/O-blocked, not compute-bound.** Prime suspect: export target (or
app-data/temp) is **inside OneDrive**, so each write stalls on cloud sync and
compounds with size. Second suspect: more than one full-signal pass (landing
measure + render + post-measure) or a `Vec` reallocation that grows with
length. **Measure first:** time a 24-min export to a plain local
(non-OneDrive) folder vs a OneDrive folder; watch disk queue depth. The owner
flagged their USB I/O device "can randomly take a crap" — real, but a 5×
super-linear curve is more than jitter. Instrument; don't assume.

`[HYPOTHESIS]` **Progressive UI lag that builds with clicks and resolves over
time** (F9) is likely a *separate* accumulation/leak — leaked rodio
sinks/sources across swaps, growing caches (`decoded_cache`,
`landing_gain_cache`), the spectrum ring/FFT, or WebView2 memory (see the
orphaned-WebView2 note in Part 0). "Replicate by clicking a ton" + "resolves
over time" fits accumulating short-lived state / detached sinks draining
slowly. The diagnostics log + a handle/heap count over a scripted click-storm
would localize it.

`[CONFIRMED / by design]` **Export button grayed during export** (F8) and
**can't overwrite in the Windows save dialog** (F9 tail) are intentional:
concurrent exports are gated; Windows' own dialog enforces the overwrite
warning (and the never-overwrite guarantee is enforced backend-side too). `[OWNER]`
— the grayed button is defensible, but a queue or a clearer "export in
progress" affordance might feel better. Design call.

### E. Album naming (F13)

`[CONFIRMED]` **The album title is never used in exported filenames.**
`src-tauri/src/album_render.rs:791-794` names each per-track WAV
`format!("{:02}-{}.wav", entry.position, safe)` where `safe =
sanitize_for_filename(stem)` and `stem` is the **source file's** stem — the
album title from the UI is not a prefix/suffix anywhere. The code comment on
:791 even claims "NN-<sanitized_title>.wav" — the comment is wrong. Adding
album-title prefixing is straightforward. `[OWNER]` — confirm the scheme
(`<Album> - NN - <track>.wav`? an album-named subfolder? padding?).

### F. Loop (F1, F3)

`[CONFIRMED]` **Loop only fires when the playhead reaches `region.end`**
(`src-tauri/src/audio.rs` loop-enforcement in the ~50 ms thread tick:
`if pos >= region.end_sec { seek to region.start }`). It never seeks *up* to
`region.start` if you start before the region, so a click before the region
plays through until end, then loops. Matches F1 exactly. `[OWNER]` — whether to
also snap forward into the region on loop-enable / play-before-region is a
small UX decision.

`[CONFIRMED]` **The loop lasso (shift-drag) works in Standard but there's no
loop toggle there** (F3). The owner wants looping Advanced-only → gate the
shift-drag region interaction behind Advanced in the waveform component
(`src/components/Waveform.tsx` — `showRegionHint` + the pointer handlers).
Small, clear scope.

### G. Move up/down reorder (F12)

`[HYPOTHESIS — UX, OWNER-aligned]` Replace move-up/down buttons with
drag-to-reorder in the album/track list. Pure frontend; find the list
component rendering the buttons. Owner explicitly wants this; medium effort —
include a keyboard-accessible fallback so it doesn't regress a11y.

---

## Part 3 — Cross-cutting guidance for whoever picks this up

- **Separate objective bugs from design decisions.** Objective (fix + test):
  album title naming (E), loop-region start behavior IF deemed wrong (F1),
  false-positive device loss IF confirmed (B), export super-linearity IF
  confirmed (D). Design/taste (`[OWNER]` confirm or visible proposal first):
  width/advanced-control slider model (A/F10), per-track view memory (C/F6),
  export-during-export UX (F8), drag reorder (F12), loop-in-Standard gating
  (F3), album naming scheme (E).
- **Do not tunnel on Fable's hypotheses.** B and D especially are unproven.
  Reproduce, instrument, let evidence pick the fix. A disproven hypothesis,
  recorded with why, is as valuable as a fix.
- **Reinstall first** so the diagnostics log is live — B and D become much
  easier from a real Save-diagnostics report; consider asking the owner to
  reproduce F4/F7/F9 on the diagnostics build and send it.
- **Nothing here is listening-gated** except preset width *calibration* changes
  — the slider-semantics fix in A does not require a listening note.

Write your own plan when you've validated. This doc is the map, not the route.
