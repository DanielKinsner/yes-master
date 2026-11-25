# Smart Match: execution plan

Decided with the owner (Dan) on 2026-11-24, after Checkpoint A of the
[Smart Balance plan](2026-11-24-smart-balance-execution-plan.md). This plan
**supersedes** that one from its Slice 3 onward. Slices 0 to 2 of the old plan
are done and stay valid; their record is the old plan's "Execution status"
section.

**What this is.** A decision-complete plan for **Smart Match**: the user picks a
reference song, and YES Master nudges the track's tone partway toward it, then
shows what it did. Written for an executor that was not in the planning
conversation.

**Drift rule.** If this plan contradicts current code or docs, verify reality,
follow reality, and note the discrepancy in the deviation log. Do not improvise
a new decision.

**Plans describe outcomes.** Code structure suggested here is a starting
hypothesis. The decisions, contract, gates and stop triggers are not.

**Where it runs.** The office Windows PC, on the existing branch and worktree.
Private test audio lives only in the main checkout at
`C:\Users\SM - Dan\Documents\GitHub\yes-master` and is referenced by absolute
path, as before.

## Why the plan changed

Checkpoint A measured 55 released songs
(`docs/reviews/2026-11-24-smart-balance-reference-range.md`). It showed that one
"healthy range" for all music is too blunt:

- Each band can sit 5 to 10 dB either side of typical on a finished master, so
  the range would leave almost everything alone, including the owner's test
  song.
- Where it would act, it would act on style: solo piano, scooped metal mids, a
  dark film score.
- The range edges move 4 to 9 dB under resampling.
- The planning session's reading of the LANDR example was wrong. The owner's
  song already had less low-mid than typical and LANDR took it further. That was
  taste, not a correction toward a norm.

The lesson: **tone needs a target that someone chooses.** Smart Match lets the
user choose it. The meter, the comparison math, the measuring tool and the song
library built in Slices 1 and 2 all carry over.

## In plain language

The reference works like a reference track in a mix session. You pick a song you
want yours to sit next to. The app measures both in the same six bands, closes
part of the gap in each band, and then your Style applies on top as it always
did.

## Decisions locked

The owner has delegated routine choices on this workstream to the planning
session ("Just go recommended on all 4. I've been letting you run this",
2026-11-24). Rows marked *planner* were decided on that basis.

| # | Decision | Consequence |
| --- | --- | --- |
| M1 | Phase 1 is Smart Match (reference matching). The range-based Smart Balance design is dropped. | No user-facing text says "Smart Balance". |
| M2 | Rule-based, from measurements. No trained model, no "AI" wording. | Unchanged from the old plan. |
| M3 | The decision code takes measured profiles and returns decisions. | It must not care whether a profile came from a whole file or a short listen (the plugin door). |
| M4 | With no reference chosen, or with the match switched off, output is bit-identical to today. | There is no behavior change for anyone who does not use it. |
| M5 | Proof is measured pass/fail gates with material held back from tuning. No listening sitting. | Do not propose one. |
| M6 | The name is **Smart Match**, shown with a small **Beta** tag. | |
| M7 | Standard view: a "Choose reference…" button, the reference's name, an on/off switch and one plain result line. Advanced adds a six-band strip and a **Match amount** slider. | Copy is locked below. |
| M8 | Match amount defaults to 50%. Advanced can set 0 to 100%. Hard caps per band: 4 dB cut, 3 dB boost. | Raising a cap or the default needs the owner. Lowering is allowed and logged. |
| M9 | The project saves the reference's six measured numbers and its name. It does not save the audio or depend on the file path. | A project reopens with the same match on any machine, with the reference file absent. |
| M10 | A reference that is very different from the track is still applied, capped, with a plain notice. | A reference is refused only when it cannot be measured. |
| M11 | Tone only. | Loudness and stereo width are not matched in this plan. |
| M12 | Saved user presets never contain or change a reference. *(planner)* | A reference belongs to a song. Applying a preset leaves the current reference alone. |
| M13 | The reference and its on/off state are **Standard-managed**. Match amount is an Advanced value. *(planner)* | Standard↔Advanced moves and Reset all keep the reference. A non-default amount counts as an Advanced edit, like other Advanced values. |
| M14 | It listens to the two songs, never to the user's knobs. | Manual EQ, Style, Intensity and Adapt strength do not change the correction. Smart Match works with Adapt strength at 0. The Style and the existing guardrail trims apply unchanged on top. |
| M15 | Album mode stays unadapted. | The control is disabled there with an explanation. |
| M16 | It never boosts what the track does not have. | A track with essentially no sub gets no sub boost, whatever the reference. |
| M17 | Every number the user sees comes from the same computation the audio uses. | The frontend never recomputes a correction. |
| M18 | Mobile apps and the browser try-it get no Smart Match UI. | Their engines must still build and behave exactly as before. |
| M19 | No public claim, landing page or marketing change. | |
| M20 | Work stays on the existing branch in small commits. No push and no merge to `main` without the owner's word. | |
| M21 | Smart Match must not depend on, or switch on, any gated system. | `ADAPTIVE_COMPRESSION`, `CONFIDENCE_GATING` and `ALBUM_CHARACTER` stay false. |
| M22 | The "30-second listen versus whole song" comparison is measured and reported, not gated. | |
| M23 | The song library and its statistics stay provisional. Their product role is now small: the typical shape used by the missing-band rule (M16), and test material. *(planner)* | The range edges from Checkpoint A are not used by the product. |
| M24 | Checkpoint B is graded by the planning session, not the owner. Checkpoint C stays with the owner. *(planner)* | See "Lanes". |

### Locked copy

Row label `Smart Match` with tag `Beta`. `<name>` is the reference's file name
without its extension, shown as plain text, shortened with an ellipsis beyond 60
characters and available in full on hover or focus.

| State | Controls shown | Line shown |
| --- | --- | --- |
| No reference | `Choose reference…` | `Pick a song you want this track to sit next to.` |
| Measuring the reference | (busy) | `Listening to your reference…` |
| On, track analysis not ready | name, switch, `Change…`, remove | `Listening to your track…` |
| On, correction applied | same | `Matching "<name>": cleared 1.2 dB of mud, added 0.8 dB of air.` |
| On, nothing to change | same | `Already close to "<name>". Nothing to change.` |
| On, a cap limited the match | same | The applied sentence, then ` Your reference is very different from this track, so the match is partial.` |
| Switched off, reference kept | same | `Match to "<name>" is off.` |
| Reference refused | previous state stays | `That file can't be used as a reference: <reason>.` |
| Album mode (disabled) | | `Smart Match applies to Track Master, not Album renders.` |

Refusal reasons: `it is shorter than 30 seconds`, `it is silent`,
`it could not be read`, `its sample rate is too low`.

The remove control's accessible name is `Remove reference`.

Correction sentence: list moves of 0.3 dB or more, largest first, at most three,
joined by commas. The phrases are lowercase because they follow the
`Matching "<name>":` lead-in. One decimal place.

| Band | Cut wording | Boost wording |
| --- | --- | --- |
| Sub, 20–80 Hz | `tamed X dB of sub rumble` | `added X dB of deep bass` |
| Bass, 80–250 Hz | `tamed X dB of boom` | `added X dB of bass` |
| Low mids, 250–800 Hz | `cleared X dB of mud` | `added X dB of warmth` |
| Mids, 800 Hz–2.5 kHz | `eased X dB of midrange` | `added X dB of midrange` |
| Presence, 2.5–6.5 kHz | `softened X dB of harshness` | `added X dB of presence` |
| Air, 6.5–16 kHz | `softened X dB of brightness` | `added X dB of air` |

Advanced only: slider label `Match amount`; strip column labels Sub, Bass,
Low mids, Mids, Presence, Air; strip caption
`Tone only. Loudness and width are not matched.`

Export receipt line: `Smart Match: off`, or
`Smart Match: "<name>" at 50% · already close`, or
`Smart Match: "<name>" at 50% · cleared 1.2 dB mud, added 0.8 dB air`.

## Lanes

### Owner lane

| # | Item | Blocks |
| --- | --- | --- |
| O1 | At Checkpoint C: run the hand test, then decide on merge and push. | Merge to `main` |
| O2 | Later, outside this plan: decide whether landing-page plates are re-shot. | Nothing here |

### Planning-session lane

| # | Item | Blocks |
| --- | --- | --- |
| P1 | Grade Checkpoint B and tell the executor whether to build the interface. | Slice 5 |

### Executor lane

Slices 3 to 8 below, in order.

## The contract

Tests must pin each rule.

1. **Inputs.** The track's six band shares (the resolved `SourceProfile`), the
   reference's six band shares (stored in settings), the match amount, and the
   on/off state. Nothing else.
2. **Gap.** Convert both sets of shares to dB. Take reference minus track per
   band. Remove the overall offset by subtracting the median of the six
   differences, so a difference in one band does not smear across the others.
3. **Correction.** Intended correction per band is `amount × gap`.
4. **Caps.** Limit each band to −4.0 dB and +3.0 dB.
5. **Dead zone.** Intended corrections smaller than the dead zone become exactly
   zero.
6. **Missing bands.** A boost fades to zero as the track's own band falls far
   below the library's typical shape, because the track does not have that
   content. Cuts are not affected.
7. **Activation.** Active only when a reference is stored, the match is on, the
   amount is above zero, and a source profile is resolved. Album resolves to no
   profile, so it is inert there without a new code path.
8. **Realization.** The intended six-band correction becomes EQ gain. Starting
   hypothesis: add a correction term to the seven existing bands' effective
   gains in `ChainCoeffs::from_settings`, solving for those gains from the
   bands' real responses at the current `eq_bands` frequencies. That adds no
   audio stage, latency or per-sample cost. A separate corrective stage is
   acceptable only if the accuracy gate cannot be met otherwise; it then must be
   skipped entirely when idle and pass the performance gate.
9. **Identity.** No reference, switched off, amount zero, or every correction
   zero: coefficients and rendered bytes are identical to today.
10. **Partial flag.** The "match is partial" notice is shown exactly when a cap
    limited at least one band at the current amount.
11. **Truth.** The readout and the receipt report the corrections the chain used,
    from one shared computation.
12. **Reference measurement.** A reference is measured once, through the same
    meter the app uses for the track, off the audio thread. Playback continues
    while it runs. A newer pick supersedes an older one still in flight. A
    refused or failed pick leaves the previous reference in place.

### Parameters

| Parameter | Start | You may move it | Planning session or owner needed for |
| --- | --- | --- | --- |
| Cut cap / boost cap | 4.0 / 3.0 dB | Lower only | Raising |
| Default match amount | 0.5 | Not at all | Any change |
| Dead zone | 0.25 dB | 0.1 to 0.5 dB | Outside that |
| Missing-band fade, start / end below typical | 8 / 14 dB | 6–10 / 12–18 dB | Outside that |
| Reference minimum length / sample rate | 30 s / 32 kHz | Not at all | Any change |

The typical shape comes from the committed provisional statistics
(`docs/reviews/evidence/2026-11-24-smart-balance-reference-range/`). The tool
emits the table; numbers are never hand-copied; a test asserts they agree.

Tune only on development material. Freeze parameters and record the commit
before any holdout run.

## Slices

Per-slice verification lists are a **floor, not a ceiling**. After each slice,
re-read the conditional rules in `CLAUDE.md` and `docs/TESTING.md` against your
actual diff. Run Cargo suites serially with `--target-dir target\codex-rc`.

### Slice 3: the decision function

Pure logic, not connected to audio.

- Implement contract rules 1 to 7 and 10 as one deterministic function with no
  dependency on gated statics, buildable for desktop, mobile and wasm.
- Product logic must not depend on the evidence runner. Shared math (shares to
  dB, offset removal) may move to a product module that the runner then uses.
- Tests, one per rule, including:
  - identical reference and track give exactly zero in every band;
  - a reference that differs in one band corrects that band and moves every
    other band by at most 0.5 dB;
  - correction is linear in amount up to the caps, and zero at amount zero;
  - caps hold at extreme inputs, at 100%;
  - a track far below typical in a band gets no boost there;
  - the partial flag is set exactly when a cap binds;
  - non-finite, zero or degenerate shares on either side give "inert", never a
    panic.
- Verify: Rust lane.

### Slice 4: the setting and the chain

The first slice that can change sound, and only when a reference is active.

- Add the Smart Match state to `AdvancedSettings` with `#[serde(default)]`:
  the reference's six shares, its display name, on/off, and the amount.
  Absent means off.
- Treat the stored name as plain display text. It is never used as a path.
- Forward compatibility: a project saved by this build with no reference must
  still open in the public 0.9.2 build. The tagged 0.9.2 `types.rs` has no
  `deny_unknown_fields` (checked 2026-11-24). Confirm on the actual
  project-loading path at tag `yes-master-v0.9.2-manual-10`, choose
  serialization accordingly, and record what you found.
- Wire contract rules 7 to 9 into `ChainCoeffs::from_settings`. Smart Match must
  work with Adapt strength at 0.
- **Cache invariant.** Every cache, hash or equality check keyed on settings
  must distinguish Smart Match states, including a change of reference numbers
  or amount. Search for them all (start at `audio.rs::settings_landing_hash`
  and the hook's readout identity); do not trust this list. Add a regression
  that changing the reference invalidates the preview loudness result.
- **Preset invariant (M12).** A saved user preset never contains a reference,
  and applying one never changes the current reference. Search for every place
  presets are saved, listed and applied.
- **Wasm ripple, expected.** `engine-stamp.test.ts` goes red when tracked
  sources change. Add any new module to the `web/tryit/wasm` crate, run
  `npm run build:tryit-wasm`, commit `src/tryit/engine/`, and confirm
  `npm test -- src/tryit` passes. The browser demo's behavior must not change.
- Tests:
  - the existing `preset_byte_identity` and `preset_fingerprint` suites pass
    with their goldens untouched;
  - for every preset, coefficients with no reference, with the match off, with
    amount zero, and with a self-match are identical to before;
  - realization accuracy (gate G4) and level invariance (gate G5);
  - album stays inert;
  - with user-moved band frequencies the correction never has the wrong sign;
  - project save and load round-trips the reference and the amount.
- Verify: Rust lane; slow fixture lane (`AMS_REAL_FIXTURE_PATH` override);
  iPhone bridge lane; Android bridge lane; frontend tests for the try-it
  engine; `chain_bench` against the saved Slice 0 baseline.

### Slice 5: reference measurement, readout and receipt

- Add a desktop command that measures a chosen file as a reference and returns
  its display name and six shares, or a refusal reason from the locked list. It
  uses the same meter as the track analysis (contract rule 12).
- Extend `GuardrailReadout` with an optional Smart Match section so the existing
  `guardrail_readout` command carries it: the state (one of the copy-table
  states), six bands (gap to the reference, correction applied), the amount and
  the partial flag.
- Add a Smart Match digest to `RenderedMeasurements` and `ExportReport`,
  following the `source_profile_digest` pattern, using the locked receipt
  wording.
- **Mirror invariant.** Every place that mirrors these Rust types must match:
  search for them, starting with `src/bindings.ts`,
  `src/bindings-drift.test.ts`, `src-tauri/tests/wire_shape.rs`,
  `src/lib/preview-mock.ts`, the export-receipt helpers, and the Kotlin and
  Swift wire models.
- Verify: Rust lane, `npm test`, `npm run build`, both bridge lanes.

### Slice 6: measured evaluation

Build a runner that produces the evidence, then run it. Commit numbers only.
Songs from the free library may be named, since their credits are already
committed. The owner's private files appear as anonymous IDs.

| Gate | Pass condition |
| --- | --- |
| G1 Off identity | Every preset; no reference, match off, and amount zero: coefficients and rendered bytes equal the baseline. No golden file is updated. |
| G2 Self-match identity | A song matched to its own measurement: bytes equal match off. |
| G3 Known-answer recovery | Take a song, make a copy with a known tone change from an independent filter (one band ±4 dB, a two-band case, and a tilt), and match the copy to the original. Where the measured gap is 1 dB or more: the correction opposes the change in every case, equals `amount × measured gap` within 0.5 dB before caps, and other bands move 0.5 dB or less in at least 90% of cases. |
| G4 Realization accuracy | The six-band change measured through the real EQ coefficients is within 0.5 dB of intended, for every single-band cap case and 20 seeded mixed cases, at 44.1, 48 and 96 kHz. |
| G5 Level invariance | Track or reference at +12 and −12 dB gives the same correction within 0.1 dB. |
| G6 Gap closed end to end | For the G3 copies rendered with Universal at default Intensity and the match at 50%, the re-measured gap in the changed band is between 35% and 65% of the original gap. |
| G7 Amount and caps | Corrections grow with amount and never exceed the caps at 100%. The partial flag matches cap activity in every evaluated case. |
| G8 Performance | `chain_bench` with no reference is within 2% of baseline; with an active match it is at least 95% of that. Reference changes go through the existing coefficient-update path with no new transition mechanism. |
| G9 Safety | Silent, very short, mono and low-sample-rate files are refused as references with the right reason. A track with an empty band never gets that band boosted. Nothing panics. |
| G10 Persistence | A project reopens with the same match when the reference file is absent. A saved preset contains no reference. |

Also report, without gating:

- **Cross-genre matches.** Match each development corpus piece to five library
  songs of other genres. Report the corrections, how often caps bind, and
  anything that looks wrong.
- **The owner's song against LANDR.** Match `It’s a coat-original-test.wav` to
  each of the three LANDR masters in `tests for presets\`. Report whether the
  correction points the same way as the move LANDR made.
- **30-second listen versus whole song** (M22). For each corpus piece, the
  correction from the loudest contiguous 30 seconds of the track against the
  whole track, toward three fixed references: largest per-band difference, and
  the share within 0.5 dB.

**Holdout protocol.** Tune on the library's build split and the five
development corpus pieces. Freeze parameters and record the commit. Then run G3
and G6 on the library's held-out songs and the three holdout corpus pieces,
**once**. A holdout failure is a stop trigger. Do not retune against the
holdout.

Write the results to `docs/reviews/<date>-smart-match-evaluation.md`, including
failures and what the evidence does not establish.

### Checkpoint B

Stop. Report each gate with pass or fail and its number, the three reports,
parameter values and any that moved, and the deviation log. The planning session
grades this checkpoint and tells you whether to continue.

### Slice 7: the interface

- Plumb Smart Match through the existing settings-update path in
  `src/hooks/useTrackMaster.ts`, using the file-dialog pattern already there.
  Read the readout from the existing `guardrailReadout`.
- **Standard** (`src/components/StandardView.tsx`): one row with the label, the
  Beta tag, the controls and the result line from the copy table. Standard must
  still fit the supported minimum window, 1360×740, with every existing control
  visible and reachable before scrolling. If it cannot fit without shrinking or
  removing an existing control, that is a stop trigger.
- **Standard-managed** (`src/lib/standard-managed.ts`, M13): the reference and
  its on/off state are not non-managed edits and survive the reset functions
  and Advanced's Reset all. A non-default amount is a non-managed edit and is
  reset to 50% by those functions.
- **Advanced** (`src/components/AdvancedPanel.tsx`): the same controls, the
  six-band strip (per band: the gap to the reference and how much of it was
  closed) and the Match amount slider. Values are available to keyboard and
  screen-reader users. Leave the debug-only `AdaptiveReadout` as it is.
- **Album**: disabled with the locked explanation, following the existing Album
  Adapt strength pattern.
- Choosing, changing, switching and removing a reference are each one undo
  step, like other settings changes.
- Use the locked copy exactly. Build sentences from backend numbers.
- Tests: component tests for every copy-table state; hook integration for pick,
  superseded pick, refusal, toggle, remove, stale readout and album;
  `standard-managed` tests; new preview scenarios (none, active, partial, off)
  with reachability at both sizes.
- Verify: `npm test`, `npm run build`, `npm run verify:headless` (inspect the
  captures), and `npm run verify:landing-assets`. If that gate reports stale
  captures, run `npm run capture:landing` and commit the result. Do not touch
  or fake the owner's real-session plates; if the gate needs them re-shot,
  report it at Checkpoint C as owner item O2.

### Slice 8: documents and contracts

- `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`, `docs/ARCHITECTURE.md`: what Smart
  Match is, where it sits, what it does not do (tone only), and its Album,
  reset, preset and Standard-managed behavior.
- `docs/TESTING.md`: the measurement tool, the evaluation runner and the lanes a
  Smart Match change requires. Say plainly that the range-based Smart Balance
  design was dropped at Checkpoint A, and why.
- `docs/OPEN_THREADS_AND_DECISIONS.md`, `docs/IDEAS_BACKLOG.md`,
  `docs/CHANGELOG.md` (an internal entry, not a public announcement).
- `CLAUDE.md` **and** `AGENTS.md`, kept byte-identical: add Smart Match to the
  product contracts as "off until the user picks a reference; do not raise its
  caps or default amount without an owner decision".
- No user-facing text says "Smart Balance". Internal names in code may stay
  where renaming would be churn.

### Checkpoint C

Stop. Report all lanes run and their results, the gate table, the hand test
below, and the deviation log. Ask the owner about merge and push. Do neither
without the answer.

## Hand test for the owner

Run from the branch with `npm run tauri dev`.

1. Load "It’s a coat-original-test.wav" from the `tests for presets` folder.
2. In Standard, find **Smart Match** with its Beta tag, a `Choose reference…`
   button, and the line "Pick a song you want this track to sit next to."
3. Play Mastered. Click `Choose reference…` and pick
   "LANDR-It’s a coat-original-test-Warm-High.wav" from the same folder. You
   should see "Listening to your reference…" briefly, then a sentence that
   starts `Matching "LANDR-It’s a coat-original-test-Warm-High":`. There should
   be no click, and the playhead should keep moving.
4. Flip the switch off and on. The sound should change, and the line should read
   `Match to "…" is off.` while it is off.
5. Open Advanced. You should see the same controls, a six-column strip, and a
   Match amount slider at 50%. Drag it to 100%: the numbers grow, and no band
   goes past a 4 dB cut or a 3 dB boost.
6. Create a master. The receipt should carry a `Smart Match: "…" at …` line.
7. Save the project, close the app, reopen the project. The match should still
   be there.
8. Remove the reference. The row returns to `Choose reference…`.

## Do not touch

- Preset calibration and preset voicing in `dsp.rs`.
- Every `TBD-CALIBRATION` constant, and the existing deadbands and caps, in
  `guardrails.rs`.
- The gate statics `ADAPTIVE_COMPRESSION`, `CONFIDENCE_GATING`,
  `ALBUM_CHARACTER` and their defaults.
- Golden and snapshot references under `src-tauri/tests/golden/`, and the
  six-band analysis golden. Never set `YES_MASTER_UPDATE_GOLDEN` in this work.
- `compute_spectral_balance_6band` itself. Both songs must be measured by that
  exact meter.
- The loudness landing, limiter and ceiling contracts.
- Landing page, release workflows, version numbers, updater configuration.
- Private audio. Nothing under `private-audio-fixtures/`, `tests for presets/`
  or `test-output/` is committed, moved or uploaded. The held folder
  `private-audio-fixtures\_held-fma-stream-2026-11-24\` is not used at all.

## Stop and ask

Stop only for these. Otherwise execute without asking.

1. You are not on the office PC, or the test audio is missing.
2. Any golden, snapshot or fingerprint reference would have to change.
3. A gate can only pass by raising a cap, changing the default amount, or moving
   a parameter outside its allowed bounds.
4. A holdout run fails.
5. Standard cannot fit the new row at 1360×740 without altering an existing
   control.
6. The work would need a preset retune, a gate flip, or a public claim.
7. Push, merge to `main`, release, deployment or spending.
8. The worktree contains changes you did not make, or `main` has moved in a way
   that conflicts with this branch.
9. A project saved with no reference would fail to open in 0.9.2.

## Deviation log

At every checkpoint, report "where the plan was unclear or wrong and what I did
about it": every judgment call, every plan claim that did not match reality,
every parameter moved and why, and every ripple beyond a slice's stated
footprint. Keep the log in the evaluation note and in an "Execution status"
section of this file.

## Parking lot

Not in this plan.

- The range-based Smart Balance, and a genre picker with built-in targets.
- Matching loudness, dynamics or stereo width to a reference.
- Smart drive and loudness selection (the September research).
- Activating or calibrating the Adaptive Compressor, confidence gating or album
  character.
- Genre detection and any trained model.
- Time-varying matching, or finer than six bands.
- Built-in or bundled reference songs.
- Smart Match in Album mode, on mobile, or in the browser try-it.
- A DAW plugin. Only the M3 seam and the M22 measurement belong here.
- Landing page, marketing, pricing or release work.

## Execution status

Executor: the Smart Balance session, on `claude/smart-balance-2e58b2`.
Started 2026-11-24 from `762daf9`. `main` had not moved since the branch was
cut (`5a18eaf`). Nothing pushed or merged.

### Slice 3: done

The decision function is `src-tauri/src/smart_match.rs`, pure and
allocation-free, with no gated static and no I/O. `decide` takes the track's
six shares, the reference's six shares, the on/off state and the amount, and
returns the gap, the six corrections, which bands a cap changed and the
partial flag. Fifteen unit tests in the module pin rules 1 to 7 and 10,
including every case the plan lists. Two integration tests in
`tests/smart_match.rs` pin the typical shape against the committed statistics
and the product's median against the runner's.

### Deviation log

1. **The order the rules apply in.** The contract lists caps (4), the dead
   zone (5) and missing bands (6) in that order. The function applies, per
   band: `amount × gap`, then the missing-band fade (boosts only), then the
   caps, then the dead zone. The fade goes before the caps so that a cap
   bounds the value that would otherwise be applied. That makes the partial
   flag (rule 10) mean "a cap limited the match", not "this track has no
   content in this band". Otherwise every track with an empty band would show
   the partial notice. The dead zone goes last, so nothing smaller than it
   survives any step; this also covers rule 5's "intended corrections". The
   partial flag is set exactly when a cap changed a band's value, and a test
   checks that over 2,000 random cases.
2. **The gap is reported even when the match is off.** When both profiles can
   be used, the decision returns the gap even if the match is off or the
   amount is zero, so a display can show how far apart the songs are. The
   corrections are still all zero in that case, so this changes no sound.
3. **Shares are normalized before they become dB.** The meter's shares already
   sum to about 1. Normalizing only matters for a hand-edited project that
   stores a reference at another scale. The gap and the fade remove the overall
   offset anyway, so scale cannot change a result; a test pins that.
4. **The shared math moved into the product module.** That is the band names,
   the band edges, the share floor, the shares-to-dB step and the offset
   removal. The measuring tool now re-exports them and uses `remove_offset`
   for its deviation step (plan Slice 3: "product logic must not depend on the
   evidence runner"). The product's median of six is allocation-free; a test
   checks it against the tool's general median.
5. **How the typical shape gets into the code.** The tool gained
   `--emit-typical-shape <stats.json>`. It writes
   `src-tauri/src/smart_match_typical_shape.rs`, which `smart_match.rs` pulls
   in with `include!`. A test checks two things: the numbers equal the
   committed statistics, and the file is exactly what the tool emits. So the
   table cannot be typed by hand.

   Bootstrapping: the tool lives in the same crate, so to compile it the first
   time I created that file with all-zero values, then ran the tool, which
   overwrote it. The committed file is the tool's output.
6. **The order of reasons for doing nothing.** When the match is inactive, the
   decision reports the first reason that applies, in this order:
   1. no reference;
   2. an unusable reference;
   3. switched off;
   4. zero amount;
   5. no track profile;
   6. an unusable track.

   Slice 5 maps these onto the copy-table states. Album resolves to "no track
   profile" (rule 7).
7. **Parameters are a struct.** `SmartMatchParams::DEFAULT` holds the plan's
   starting values. `within_plan_bounds` encodes the "You may move it" column,
   so the evaluation runner can try other values and prove each one is
   allowed. No parameter has moved.
