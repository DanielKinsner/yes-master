# Auto gain staging: live audition, interface, documents (Checkpoint C)

Date: 2026-11-25. Plan:
[auto gain staging execution plan](../plans/2026-11-24-auto-gain-staging-execution-plan.md),
Checkpoint B's three items and Slices 5–7. Branch
`claude/auto-gain-staging-bf8b35`. Nothing pushed or merged.
[The engine evaluation](2026-11-25-auto-gain-staging-evaluation.md) has
Checkpoint B.

## Bottom line

**Auto gain staging is built end to end and off by default: engine, export,
live audition and the Advanced switch. It is ready for the owner's hand
test.**

> **Checkpoint D (2026-11-27):** Slice 8 added a fast estimate in audition
> for priority Clean. On the reference song it plays within about 1.4 s of
> a settings change; Reach target has no estimate. See
> [Slice 8](#slice-8-fast-settling-checkpoint-d) below; the settling times
> in this bottom line are Checkpoint C's.

- **What works, measured:**
  - Audition resolves the gains with the export's own resolution: a test
    pins the two bit for bit.
  - While a new resolution is in flight, level in applies at once, the
    limiter drive holds, and the line reads "Measuring this song…".
  - No callback deadline misses and no callback allocations with the switch
    on; Original/Mastered keeps the playhead.
- **What to know before the hand test:**
  - **Settling is slow.** A new combination of settings takes 8–11 s to
    settle on a 2-minute song, 15 s on a 4-minute song and 27–43 s on a
    7-minute song, against today's 0.9–2.4 s. Settings already resolved
    apply in under 0.2 s, and playback continues while it works.
  - **E4 still fails, by a hair.** With the planning session's faster step
    for Reach target, 19 of the 20 acceptance cases meet every condition.
    One took 0.09 dB more limiting than allowed, because it landed closer to
    the target than the reference did.
- **Open:** the Mac build (Swift receipt fields uncompiled), and the owner's
  ears.

Commits on top of `5b63cea` (Checkpoint B's three items):

- `560a7a2`: the E4 re-check evidence and the Reach-target export timing;
- `97a2854`: Slice 5, live audition;
- `6586a1b`: Slice 6, the interface;
- `01be25b`: the scripted landing captures;
- the commit carrying this note: Slice 7, the documents.

## Lanes at Checkpoint C (this PC, on the committed code)

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
| `cargo test --target-dir target\codex-rc`, slow fixture lane on the reference song (`AMS_RUN_REAL_FIXTURE=1`, `AMS_REAL_FIXTURE_PATH`) | 863 passed, 0 failed, 36 ignored (the opt-in native and private probes) |
| iPhone bridge: `cargo check --all-targets`, `cargo test` | Clean; 46 passed |
| Android bridge: `cargo test`, `cargo ndk -t arm64-v8a --platform 29 check` | 26 passed; clean |
| Try-it engine: `npm run build:tryit-wasm`, `npx vitest run src/tryit` | Rebuilt; 32 passed |
| `npm test` | 1,005 passed (94 files) |
| `npm run build` | Pass |
| `npm run verify:headless` | Pass: 54 scenario/viewport checks, including the two new ones |
| `npm run verify:landing-assets` | Pass, after `npm run capture:landing` retook the three scripted captures (the owner's plates untouched) |
| Native, opt-in, muted | `mastering_quality_auto_gain_staging_live` on 3 songs; `mastering_quality_streaming_callback_bench` off and on |

## Checkpoint B's three items

### 1. E4, Reach target

The planning session kept E4 recorded as failed as written, kept Clean
unchanged, and let Reach target alone use a faster step and up to 8 passes.

- **The step:** a secant through the last two passes, aimed half a tolerance
  under the target, at least the remaining shortfall and at most 6 dB. It
  can overshoot; the landing trims it.
- **Acceptance, on the 20 cases the 12-pass diagnostic reached:**

| Condition | Result |
| --- | --- |
| Within 0.2 LU of the target | 20 of 20 (worst: Small Colin – All On Me, Clarity, −9.17 for −9.0) |
| At most 8 passes | 20 of 20, all in 4 (the reference needed 7–10) |
| Limiter load at most 0.5 dB above the 12-pass reference's | **19 of 20.** Small Colin – All On Me, Universal, −9: 10.66 dB against 10.07 dB, 0.59 dB more |
| Never above the target after landing (E5) | 20 of 20 |

The failing case landed at −9.06 LUFS where the reference stopped at −9.20.
The secant aims closer to the target than the old step stopped, and the
last 0.14 LU cost 0.59 dB of limiting on this song. Nothing was tuned to make
it pass. **Graded at Checkpoint C: failed as written by 0.09 dB and accepted
by the planning session.**

- **All cases where Clean stopped at the clean limit:**

| | Within 0.2 LU | Stopped by the peak check | Above target | Passes |
| --- | --- | --- | --- | --- |
| Development, 136 | 129 (was 109) | 7, all Funkorama | 0 | 2: 12, 3: 76, 4: 46, 5: 1, 6: 1 |
| Holdout, 50 | 50 (was 48) | 0 | 0 | 2: 3, 3: 28, 4: 19 |

Evidence: `evidence/2026-11-25-auto-gain-staging-evaluation/reach-target-recheck.json`.

### The slowest Reach-target export, and its progress

The real Track export of Scott Buckley – Unraveling (7:24) with Universal,
single thread, median of three runs:

| Target | Switch off | Reach target | Receipt line |
| --- | --- | --- | --- |
| −14 | 9.65 s | 11.50 s | level in −0.8 dB · limiter drive −2.4 dB · limiting 0.0 dB |
| −11 | 9.81 s | 11.68 s | level in −0.8 dB · limiter drive +0.6 dB · limiting 2.7 dB |
| −9 | 9.64 s | **21.60 s** | level in −0.8 dB · limiter drive +3.6 dB · limiting 5.7 dB |

- **Progress keeps moving.** In every run it rose monotonically to 100%,
  with 49–61 updates, and advanced on every resolution pass.
- **Its longest pause is 3.2–3.4 s,** inside the first pass (45% → 48%),
  while that pass's sample-rate conversion and final check run without
  updates. The pause before the first update is about 0.5 s; after the last,
  0.01 s.

Evidence: `evidence/2026-11-25-auto-gain-staging-phase-c/reach-target-export-timing.json`.


### 2. The no-progress stop

When a Reach-target pass adds at least 1 dB of drive and the delivered
loudness rises by less than 0.1 LU, the search stops. The line reads
`Reached −11.3 LUFS of −9.0. This song's peaks stop it going louder under
your ceiling.`, and the receipt appends ` · stopped by the peak check`.

- It applies to Reach target only: Clean was frozen, and Clean already stops
  at the clean limit (deviation 35).
- It ends 7 development cases (all Funkorama, at 9.9–26.1 dB of limiting)
  and no holdout case.

### 3. `Limiter::detector_level` is pinned

`the_limiter_level_helper_matches_the_limiters_own_detection`, over 4
signals (noise, a near-Nyquist tone, sparse impulses, a clipped wave) at 44.1,
48 and 96 kHz with 1, 2 and 6 channels:

- a gain putting the detector level exactly at the ceiling reduces nothing;
- 0.1% more reduces about 0.0087 dB;
- 1 dB more reduces about 1 dB.

Changing the helper's 1.012 factor to 1.015 fails it.

## Slice 5: live audition

- **On the existing preview worker.** When the switch is on and the settings
  have no verified resolution, the worker resolves first (the export's own
  resolution, `gain_staging::resolve_preview`), then measures Volume Match
  and the landing with the resolved gains. No second worker.
- **Rule 8.** Level in comes from the track analysis (now kept in the
  profile store) and applies at once. The limiter drive holds what the chain
  already plays, never rises on an unverified value, and starts at 0 for a
  new song or after the switch was off. The line reads "Measuring this
  song…" until the resolution applies.
- **The crossfade.** Resolved gains reach the playing chain through the
  existing coefficient crossfade.
- **Cache.** Resolutions are kept per file, modification time and settings.
  The audio command thread only `try_lock`s it, so it never waits.
- **Superseded work** is cancelled, never stored and never shown.
- **The line** comes from the loaded song's live state
  (`auto-gain-staging:status`), or, when the song is not playing, from
  `auto_gain_staging_readout`, which shares the permit and the cache.

### Time to a settled sound

Office PC, muted, production `AudioPlayer`, Universal at Intensity 0.75,
export-loudness preview on.
`evidence/2026-11-25-auto-gain-staging-phase-c/slice-5-audition.json`.

| Song | Today: a target edit | Switch on | Target edit, switch on | Clean → Reach target | Settings already resolved |
| --- | --- | --- | --- | --- | --- |
| Sideways City (2:13) | 0.89–0.96 s | 10.7 s | 8.1–8.2 s | 4.3 s | 0.13 s |
| Reference song (4:05) | 1.42–1.44 s | 15.4 s | 14.8–14.9 s | 7.7 s | 0.19 s |
| Unraveling (7:24) | 2.37–2.38 s | 42.6 s | 26.6–27.2 s | 14.8 s | 0.12 s |

- Each new combination runs the export's resolution over the whole song (a
  stage capture, 2–4 limiter passes with the export's conversion and final
  check), then today's landing measurement renders the chain again. The
  first switch-on also measures the song's level once.
- Ideas to make it faster are recorded in `docs/IDEAS_BACKLOG.md`; none was
  tried.

### Tests and probes

| Requirement | Evidence | Result |
| --- | --- | --- |
| Allocation-free callback | `auto_gain_staging_drive_updates_crossfade_without_allocating`: the switch and four drives through the crossfade | 0 allocations |
| No deadline misses | `mastering_quality_streaming_callback_bench`, off and with `YES_MASTER_CALLBACK_AUTO_GAIN_STAGING=1` (drive moves on every edit, the resolution as the concurrent worker) | 0 and 0 misses in 606 and 607 callbacks; worst callback 18% and 24% of its buffer. The Realtek output granted 480–1056 frames for 256 requested, so this does not establish small-buffer headroom |
| No rise on a pending value | `pending_applies_level_in_at_once_and_never_raises_the_drive` | Pass |
| Superseded resolution discarded | `a_superseded_resolution_is_discarded`; the hook's superseded test | Pass |
| Original/Mastered keeps the playhead | native probe, 4 swaps per song | Pass (≤ 0.01 s) |
| Volume Match still compares at matched loudness | `the_worker_resolves_first_and_measures_the_resolved_chain`: matched within 1 LU of the source; native probe: Volume Match on settles in 0.19 s | Pass |
| Audition equals export | `audition_resolves_exactly_what_the_export_resolves` (Clean −9, Reach −9, Clean −14) | Bit-identical gains and delivered loudness |

## Slice 6: interface

- **Advanced only,** in the Advanced Controls card under the gain, loudness
  and ceiling knobs: `Auto gain staging` with a Beta tag, the switch, the
  line from the copy table (with the new peak-check line), and `Loudness
  priority` (`Clean` / `Reach target`) while on. Standard is unchanged.
- **An Advanced edit** under `standard-managed.ts` while on. The card's
  reset, Reset all and the return to Standard turn it off.
- **Album:** the switch is disabled, with the locked line.
- **Tests:** every copy-table state; the row; the hook's pending,
  superseded, live-line and Album paths; two new headless scenarios at both
  window sizes (reachability, axe, Reach target, the switch).

## Slice 7: documents

`docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`, `docs/ARCHITECTURE.md`,
`docs/TESTING.md`, `docs/OPEN_THREADS_AND_DECISIONS.md`,
`docs/IDEAS_BACKLOG.md`, `docs/CHANGELOG.md`, and the line in `CLAUDE.md` and
`AGENTS.md` (byte-identical): "Auto gain staging ships off by default; do not
turn it on by default, raise its clean limit, or move its reference level
without an owner decision."

## Slice 8: fast settling (Checkpoint D)

Date: 2026-11-27. The planning session's ruling after the F1/F2 trade-off
(its option 1): **a fast estimate for priority Clean only; Reach target has
no estimate.**

### What changed, in plain words

- **Clean:** after a settings change, audition plays a fast estimate of the
  gains, then the exact result. The estimate runs the export's own search
  on parallel chunks of the song and emulates the export's final check with
  the cheap (lowpass) half of the qualified peak meter. The line reads the
  estimate's numbers with ` · refining…` until the exact result lands; the
  exact result changes what plays only when the drives differ by 0.3 dB or
  more.
- **Reach target has no fast estimate.** As before Slice 8: level in at
  once, the drive holds or falls, and "Measuring this song…" until the
  exact result lands.
- **Why not Reach target.** On bright material the export's final check
  reads a higher peak (the qualified meter's finite-sinc half) than the
  limiter allows for, and holds the loudness down. Reach target's search
  then stops early at its no-progress peak check. An estimate that never
  reads the finite-sinc half cannot see that, so it can land far above the
  exact result: 2.44 dB on Funkorama (Clarity, −9 LUFS), about 10 dB on a
  synthetic bright signal. Reading that half costs about 1.5 s per read on a
  4-minute song, which breaks the 2-second budget. Clean cannot land high
  that way: its drive never passes the clean limit, and a low peak reading
  makes its search stop sooner, not later.
- **Exports never use the estimate.**

### The gates

| Gate | Requirement | Result |
| --- | --- | --- |
| F1, development (Clean, 900 cases, 75 songs) | within 0.5 dB in ≥ 90%, within 1.0 dB in ≥ 99%, never > 1.5 dB above | 99.0%, 99.67%, most above +0.25 dB: **pass** |
| F1, holdout (Clean, 228 cases, 19 songs; frozen `5d80431`) | the same | 100%, 100%, most above +0.23 dB: **pass** |
| F2, settings change to the estimate playing (Clean) | ≤ 2.0 s (2- and 4-minute songs), ≤ 4.0 s (7-minute song) | 0.84–1.16 s, 1.40–1.49 s, 2.15–2.18 s; the 7-minute switch-on 4.41 s: **failed as written by 0.41 s, accepted by the planning session** |
| F3, the exact result replacing the estimate | through the crossfade; no change below 0.3 dB; how often ≥ 1.0 dB | unit-tested; ≥ 0.3 dB on 15 of 900 development and 0 of 228 holdout cases; ≥ 1.0 dB on 3 of 900 (all Funkorama, estimate lower) and 0 of 228 |
| F4, exports with the switch on | bit-identical to `08509ab` | 360 of 360 exports identical (10 songs × 4 Styles × 3 loudnesses × off, Clean, Reach target) |
| F5, Slice 5's probes | no callback allocation, no deadline miss, no rise on a value from other settings, superseded work discarded, playhead kept, Volume Match matched | all pass (see the lanes below) |

- **This is the holdout's second use, for a different measure.** Its first
  was the Checkpoint B engine gates: the decision rule (E3), and the E4
  Reach-target re-check on its Reach cases after the planning session's
  change. Nothing about the estimate was tuned on it; it ran once.
- **F1's Clean-limit agreement:** on the holdout the export stopped at the
  clean limit on 50 cases, and the estimate stopped there on all 50.
- **"Never higher" on Clean.** The cap holds by construction: a Clean
  search never tries a drive past the clean limit, whatever loudness its
  passes report (pinned by `a_clean_search_never_drives_past_its_cap`).
  That the estimate never lands *above the exact drive* holds only on the
  measured songs, not by construction: the argument needs the delivered
  loudness to rise with the drive by at most 1 dB per dB, and the estimate's
  chunked loudness differs slightly from the export's. The largest measured
  overshoot was +0.25 dB (development) and +0.23 dB (holdout), within the
  search's 0.2 LU tolerance plus that small difference.

### Lanes at Checkpoint D (this PC, `5d80431` plus the documents)

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`, without and with `--features gain-staging-research` | Pass |
| `cargo test` with the slow lane on the reference song | 868 passed, 0 failed, 40 ignored (golden byte-identity and fingerprint included) |
| Callback allocations (`auto_gain_staging_drive_updates_crossfade_without_allocating`) | 0 |
| `mastering_quality_streaming_callback_bench`, off and with `YES_MASTER_CALLBACK_AUTO_GAIN_STAGING=1` (now the estimate, then the exact result, as the concurrent worker) | 0 and 0 deadline misses in 607 and 754 callbacks. The Realtek output granted 480–1056 frames, so this does not establish small-buffer headroom |
| Native live probe, three songs × three runs | Pass: the line, the cached revisit, playhead within 0.01 s over 4 swaps, Volume Match settled in 0.19 s |
| Rule 8, superseded work, Reach target's missing estimate | `gain_staging_live` tests pass (rule 8 pending for both priorities; `a_superseded_resolution_is_discarded`; `reach_target_never_produces_an_estimate`) |
| Mobile bridges | iOS `cargo check --all-targets` and 46 tests; Android 26 tests and the arm64 NDK check: pass |
| Try-it | Rebuilt with no change; its tests pass |
| `npm test`, `npm run build` | 1006 passed; build passes |
| `npm run verify:headless`, `npm run verify:landing-assets` | Pass (54 app checks; landing assets bound to their recorded digest) |

### The wait in practice (office PC, median of three runs)

Seconds from a settings change (the backend command; the interface adds its
180 ms debounce before it):

| Song | Clean: estimate playing | Clean: exact result | Reach target: exact result | Today, switch off |
| --- | --- | --- | --- | --- |
| Sideways City (2:13) | 0.84–1.16 | 8.9–11.8 | 3.9 (settled sound 4.3) | 0.90–0.95 |
| Reference song (4:05) | 1.40–1.49 | 16.0–16.8 | 7.1 (settled sound 7.8) | 1.49–1.50 |
| Unraveling (7:24) | 2.15–2.18; switch-on 4.41 | 28.6–47.5 | 12.6 (settled sound 14.9) | 2.38–2.43 |

- The Reach target column is a switch from Clean to Reach target at the
  same loudness.
- On the reference song with Clean you hear a close-to-final sound about
  1.4 s after a change; the exact result arrives about 16 s after it, and
  on these songs it changed nothing audible (the drives agreed within
  0.02 dB).
- The exact result now lands about 1.2 s later than at Checkpoint C (16.0
  against 14.8 s on the reference song), because the estimate runs first
  on the same worker.

### How the choice was made

| Estimate reading | F1, development, all 1036 cases | F2, reference song |
| --- | --- | --- |
| 4x true peak | fails (most above +5.78 dB) | not timed |
| Lowpass half | fails on Reach target only (+2.44 dB, one case) | 1.36–1.53 s |
| Verified: lowpass, then both halves once on the stopping pass | passes (most above +0.33 dB) | 2.60–3.04 s: fails |
| Both halves on every pass | follows the export, even on the synthetic signal | slower still |

`docs/reviews/evidence/2026-11-27-auto-gain-staging-slice-8/` has the
numbers: `f1-development.json`, `f1-holdout.json`, `f2-audition.json`.

## Hand test for the owner

To be run from this plan's worktree with `npm run tauri dev`.

1. Load "It’s a coat-original-test.wav". In Advanced, choose Universal and the
   High loudness, and export a master with **Auto gain staging** off.
2. Turn the switch on. The line should settle on values for level in, limiter
   drive and limiting. Play Mastered and flip the switch a few times: the
   loudness should stay the same, and the on version should have more punch.
3. Export again. The receipt should show the same loudness as step 1, a peak
   closer to the ceiling, and the `Auto gain staging:` line.
4. Try each of the four Standard Styles with the switch on, then off. With it
   on, the Styles should be easier to tell apart.
5. Load one of your Suno songs and repeat step 2.
6. If you have a quiet, unmastered mix, load it and pick High. With priority
   `Clean` the line should say what loudness it reached. Switch to
   `Reach target`: it should get louder and the export should warn you.

The part only the owner can judge is whether the on version sounds better.

What to expect while testing:

- After each change with the switch on and priority `Clean`, the line
  reads "Measuring this song…" for about 1.5 s on the reference song, then
  shows the estimate's numbers with ` · refining…` while you already hear a
  close-to-final sound. The exact numbers replace them about 16 s after the
  change.
- With `Reach target` there is no fast estimate: "Measuring this song…"
  stays until the exact result, about 7 s on the reference song after a
  switch from Clean. Flipping back to settings already measured is
  instant.
- "High" in Standard is −9 LUFS. In Advanced, the loudness knob sets the
  same target.
- Step 6's export warning comes from the existing export checks; this work
  did not change them or verify which ones fire on a heavily limited master.

## What this does not establish

- **Sound.** The hand test.
- **Mac.** The Swift receipt model gained two optional fields and was not
  compiled; it needs a Mac build.
- **Typical unmastered band mixes.** The quiet material is classical and
  piano heavy; nothing here establishes how G treats a typical unmastered
  band mix.
- **Small-buffer audio interfaces.** The callback bench ran on a device that
  granted large buffers.
- **Timings** are this PC only.

## Evidence

- `evidence/2026-11-25-auto-gain-staging-evaluation/reach-target-recheck.json`
- `evidence/2026-11-25-auto-gain-staging-phase-c/slice-5-audition.json`
- `evidence/2026-11-25-auto-gain-staging-phase-c/reach-target-export-timing.json`

Private ledgers and logs stay under `test-output/auto-gain-staging/`.
