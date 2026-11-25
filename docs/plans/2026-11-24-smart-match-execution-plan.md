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
| On, Match amount at 0 *(added at Checkpoint B)* | same | `Match amount is at 0, so Smart Match is idle.` |
| On, track analysed without a six-band profile *(added at Checkpoint B)* | same | `Not enough audio to analyze this track.` |

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
`Tone only. Loudness and width are not matched.` A second caption line,
added at Checkpoint B:
`Moves are applied before the compressor and limiter, so the final change can be smaller on loud material.`

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
| G9 Safety | Silent, very short and low-sample-rate files are refused as references with the right reason. A mono file is accepted, because it can be measured (M10; mono was wrongly listed among the refusals until Checkpoint B). A track with an empty band never gets that band boosted. Nothing panics. |
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

### Deviation log (Slice 3)

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

### Slice 4: done

Smart Match state lives on `AdvancedSettings.smart_match`:
`SmartMatchSettings { reference: Option<{ name, spectral_6 }>, enabled, amount }`.
It defaults to absent and is never written when absent.
`ChainCoeffs::from_settings` asks `smart_match::decide_for_settings` for the
decision. When the decision changes something, `smart_match::realize` solves
for EQ gains, and the gains are added to the seven bands. Otherwise nothing
changes, bit for bit.

Gate-relevant results (the full Slice 6 run will restate them):

- **Off identity.** No reference, switched off, amount zero and a
  self-match give equal coefficients and bit-identical rendered samples for
  all nine presets at 44.1, 48 and 96 kHz.
- **Realization (G4).** The worst error is 0.015 dB, across every single-band
  cap case and 20 seeded mixed cases at all three rates, with every filter
  placement. This is measured through the chain's real coefficients on a
  240-point-per-band grid.
- **Level (G5).** ±12 dB on the track or on the reference moves no correction
  by more than 0.1 dB (synthetic audio through the real meter).
- **Speed (`chain_bench`, release, 7 alternating runs per configuration,
  against the saved `5a18eaf` binary).**
  - No reference: coat −0.12% and Doors +0.16% against baseline.
  - Active match: coat 96.0% and Doors 100.9% of no-reference.
  - Context, not gated: with both Advanced shelves on, the match needs its own
    stage, and costs coat 92.6% and Doors 97.8%.

Lanes run for Slice 4 (office PC):

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings` | Clean |
| Slow fixture lane (`AMS_RUN_REAL_FIXTURE=1`, `AMS_REAL_FIXTURE_PATH` = `It’s a coat-original-test.wav`) | 46 suites, 778 passed, 0 failed, 35 ignored. The four real-fixture contract tests ran. `preset_byte_identity` and `preset_fingerprint` passed with their goldens untouched |
| iPhone bridge (`cargo check --all-targets`; `cargo test`) | 46 passed, 0 failed, 1 ignored |
| Android bridge (`cargo test`; `cargo ndk -t arm64-v8a --platform 29 check`) | 26 passed, 0 failed; the NDK check built |
| Try-it engine (`npm run build:tryit-wasm`; `npx vitest run src/tryit`) | Rebuilt; 32 passed |
| `npm test`; `npm run build` | 920 passed; the build (which includes the wire-shape type gate) succeeded |
| `chain_bench` against the saved Slice 0 binary | See "Speed" above |

### Deviation log (Slice 4)

8. **Realization needs one extra filter.** Rule 8's starting hypothesis was to
   adjust only the seven existing bands. That cannot meet G4. In a prototype
   on the gate's cases, the seven alone missed:
   - mixed targets by 2.1 to 4.3 dB;
   - single-band cap cases by 1.2 to 4.1 dB.

   The 80 Hz peak sits on the border between the meter's sub band (20–80 Hz)
   and bass band (80–250 Hz), and the 200 Hz shelf lifts both, so the two
   bands cannot be moved separately. I added one corrective peaking filter at
   140 Hz with Q 0.9, the centre of the bass band, as the plan allows when the
   gate cannot be met otherwise. With it the worst G4 error is 0.015 dB.
9. **Where that filter runs.** It runs in whichever Advanced shelf slot is
   idle, Warmth first and then Presence/Air. Those are identity biquads the
   chain processes anyway, so the usual case adds no per-sample work. Only
   when both sliders are in use does the filter get its own stage.
   - The stage is skipped while idle. Its state is zeroed on every coefficient
     install that leaves it idle, which equals having run an identity filter,
     so re-activation is clean.
   - All of this happens inside the existing coefficient-update path; there is
     no new transition mechanism.
   - The first build always used the separate stage. It missed G8 on coat
     (92.9%); slot reuse brought it to 96.0%.
10. **The solver changed three times during testing.**
    - The regularization weight (1e-3) biased results short by up to 0.4 dB
      once Universal's 24 Hz high-pass made the sub band less responsive. It
      is now 1e-6, which only picks the smallest of the exact fits.
    - At 96 kHz, f32 rounding in the low bands' coefficients made a 0.05 dB
      forward-difference slope pure noise. It is now a central difference of
      ±0.25 dB.
    - A band layout the user has squeezed (mid at 2.68 kHz, high-mid at
      2.57 kHz) made the plain Gauss-Newton solve diverge. It is now
      Levenberg-Marquardt: a step is kept only if it lowers the miss.

    A release-build solve takes 6 to 10 µs.
11. **What "realized" is measured as.** The target is each meter band's power
    change for pink noise (equal power per octave), relative to the same EQ
    without the match. The meter sums FFT power uniformly across a band, so
    pink weighting is what the meter itself would report for pink noise. Real
    songs are not pink inside a band; G3 and G6 test real songs end to end.
12. **One definition of the seven band shapes.** `dsp::eq_band_coeffs` now
    builds the seven EQ bands for both `from_settings` and the solver. This is
    a byte-identical refactor: the preset byte-identity snapshots pass
    untouched.
13. **The setting is omitted when absent, not written as null; 0.9.2
    compatibility.** At tag `yes-master-v0.9.2-manual-10`, `load_project`
    calls `read_session`. That clamps floats and then calls
    `serde_json::from_value::<ProjectState>`. No settings type there has
    `deny_unknown_fields`; only `export_format::EncodingWire` does. So:
    - a project with a reference also opens in 0.9.2, which ignores the field;
    - a project without one is written exactly as before (a test pins that).

    Stop trigger 9 does not apply.
14. **Presets (M12).** The backend strips `smart_match` when a preset is saved
    and again when presets are read, so a hand-edited file cannot carry one
    either. The frontend's `applyUserPreset` keeps the song's current state,
    through a pure helper (`src/lib/smart-match.ts`) with its own tests.
    Saving needed no frontend change, because the backend is the authority.
15. **Caches.** The search found three that key on settings:
    - `audio.rs::settings_landing_hash` hashes the serialized settings, so it
      includes Smart Match. I added a finiteness guard and the regression
      test: changing the reference, the amount or the switch misses the
      cache.
    - `audio/preparation_cache.rs` keys on `ChainCoeffs` equality, which
      includes the new filter and stage flag.
    - The hook's readout identity is the settings object's identity, so any
      Smart Match change re-requests the readout.
16. **Wire samples, without the golden variable.** The plan forbids setting
    `YES_MASTER_UPDATE_GOLDEN`. `tests/wire_shape.rs` regenerates
    `src/wire-samples.json` with that same variable, so I did not set it.
    Instead I inserted the new keys with a script. The unchanged
    `wire_shape` test confirms the committed JSON equals what Rust emits.
    The sample is a self-match, so no other sampled value changed.
17. **The browser engine.** It now compiles `smart_match.rs` and the
    generated table; all three stamp lists (`build.rs`,
    `build-tryit-wasm.mjs`, `engine-stamp.ts`) name both files. The browser
    never stores a reference, so it renders as before, and the try-it tests
    pass. Its wasm grew from 424 KB to about 451 KB, because the solver is
    compiled in.
18. **`chain_bench` gained `--smart-match` and `--with-shelves`.** Without
    flags it runs the baseline configuration. The new binary is kept beside
    the baseline in `test-output/smart-balance/baseline/`.
19. **The phone bridges.** They build settings with `..Default::default()`, so
    Smart Match is absent and inert there (M18).

### Slice 5: done

- **Reference command.** `smart_match_reference::measure_smart_match_reference(path)`
  decodes and measures a chosen file with the same `decode_full` and the same
  meter call the track analysis uses. It returns
  `{ status: "measured", reference: { name, spectral_6 } }` or
  `{ status: "refused", reason }`, where the reason is one of `too_short`,
  `silent`, `unreadable` or `low_sample_rate`. Each reason maps to its locked
  text. The command runs on a blocking worker, never on the audio thread.
- **Readout.** `GuardrailReadout.smart_match` carries:
  - the state (`no_reference`, `track_not_ready`, `applied`,
    `nothing_to_change`, `switched_off`, `zero_amount` or `album_disabled`);
  - the amount and the partial flag;
  - six bands, each with its gap, its applied correction and whether a cap
    bound it.

  It is built by `smart_match::readout_for_settings` from
  `decide_for_settings`, the decision the chain uses.
- **Receipt.** `RenderedMeasurements` and `ExportReport` carry
  `smart_match_digest`, built by `smart_match::receipt_digest` with the locked
  wording. The engine fills it from the render's own resolved settings, and
  the frontend's `buildExportReport` copies it onto the receipt.
- **Mirrors.**
  - `src/bindings.ts`: the new types, and `SMART_MATCH_REFUSAL_REASONS`.
  - `src/lib/api.ts`: `measureSmartMatchReference`.
  - `src/bindings-drift.test.ts`: compile-time key checks for the readout,
    its bands, and both measurement variants.
  - `src-tauri/tests/wire_shape.rs` and `src/wire-samples.json`.
  - `src/lib/export-receipt.ts`.
  - Kotlin `WireModels.kt` and Swift `NativeMasteringBridge.swift`: the
    optional digest.
- **Tests.** `tests/smart_match_readout.rs` covers:
  - every state;
  - that readout numbers equal the decision's;
  - the receipt wording, including the plan's own example line and all
    twelve phrases;
  - a real engine render carrying the line, and a render without Smart Match
    writing no such key.

  Unit tests in `smart_match_reference.rs` cover the refusals, mono and the
  display name. The frontend has a receipt-helper test.

Lanes run for Slice 5 (office PC):

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test --target-dir target\codex-rc` | 47 suites, 789 passed, 0 failed, 35 ignored |
| iPhone bridge (`cargo check --all-targets`; `cargo test`) | 46 passed, 0 failed, 1 ignored |
| Android bridge (`cargo test`; `cargo ndk -t arm64-v8a --platform 29 check`) | 26 passed, 0 failed; the NDK check built |
| `npm test` | 921 passed |
| `npm run build` | Succeeded (including the wire-shape type gate) |
| Try-it engine | Rebuilt; 32 passed |

### Deviation log (Slice 5)

20. **Mono references are accepted (planning session: please confirm).** G9
    lists mono among the files to refuse. But M10 says a reference is refused
    only when it cannot be measured, and the locked refusal list has no mono
    reason. A mono file measures normally, so it is accepted. G9 will check
    that mono measures without trouble, and that every other refusal carries
    the right reason.
21. **A copy-table gap: amount zero (planning session: please decide the
    line).** The decision has a state the copy table lacks: a reference is
    stored and switched on, but the amount is 0%. The readout reports it as
    `zero_amount`. The receipt treats it as `Smart Match: off`. Slice 7 needs
    a line for it; reusing `Match to "<name>" is off.` would be one option.
22. **"Listening to your track…" can last forever (flagged for Slice 7).** A
    track whose analysis finished without a six-band profile reads the same as
    a track still being analysed: `track_not_ready`. That happens for a silent
    track, or one under about 1,024 frames. Slice 7 can tell the two apart
    from the analysis the hook already holds.
23. **The dead zone moved from 0.25 to 0.3 dB**, inside the plan's 0.1–0.5 dB
    bounds. It now equals the smallest move the locked result line and receipt
    name (0.3 dB), and a compile-time assertion keeps the two together.
    Without the change, a 0.25–0.29 dB move would change the sound while being
    absent from both texts, which breaks contract rule 11. I made it for
    design consistency, before any evaluation run, not as tuning on results.
24. **The receipt line is omitted, not written as null, when no reference is
    stored.** Phone receipts and receipts that never used Smart Match keep
    their exact shape. When a reference is stored but inactive, the line is
    `Smart Match: off`. Slice 7 decides whether the desktop receipt shows
    `Smart Match: off` when the line is absent.
25. **The receipt phrase rule.** The locked receipt examples drop "of" from
    the sentence phrases ("cleared 1.2 dB mud"), so I applied the same rule to
    all twelve phrases. Selection follows the sentence's rule: 0.3 dB or more,
    largest first, at most three, with ties in band order.
26. **The readout is omitted when a song has no Smart Match state**, so phone
    and browser readouts are unchanged. `album_disabled` is reported only when
    a Smart Match state exists on an Album surface; the frontend knows which
    mode it is in either way.
27. **Contract rule 12 is split.** The command itself is stateless. The rest
    of rule 12 belongs to whatever stores the result, which is the hook in
    Slice 7:
    - a newer pick superseding one still in flight;
    - keeping the previous reference after a refusal.
28. **The preview mock's answer for the new command is deferred to Slice 7.**
    `src/lib/preview-mock.ts` is a landing-capture input, so editing it fails
    the screenshot gate until a re-capture, which Slice 7 does anyway. The new
    readout and receipt fields are optional, so the mock already matches the
    types.
29. **Wire samples.** As in Slice 4, I inserted the new keys with a script,
    and the unchanged `wire_shape` test verified them. One frontend test
    (`App.compressor-mode.test.tsx`) spread the raw JSON sample into a typed
    readout. JSON imports type enum strings as plain `string`, so that test
    now uses a typed view of the sample.
30. **What was not compiled.** The Kotlin and Swift models gained only the
    optional digest field. The Android and iPhone Rust bridge lanes ran. The
    Kotlin app's own unit tests did not run, and the Swift app was not built:
    it needs a Mac.
31. **A commit I did not make.** While I worked, the planning session
    committed `d45d5d1` on this branch for the owner. It disables Vercel
    preview builds for this branch, because the branch was pushed as a backup,
    and it touches only `vercel.json`. My commits sit on top of it with no
    conflict, so stop trigger 8 does not apply. I have not pushed anything.

### Slice 6: done; stopped at Checkpoint B (stop trigger 4)

The evaluation runner is `src-tauri/src/smart_match_evaluation.rs`, run by
`examples/smart_match_evaluation.rs`, and tested in
`tests/smart_match_evaluation.rs`. That test file covers the independent
equalizer's accuracy, the tilt, the loudest-window finder and the gate
arithmetic. The parameters were frozen at `c61dbfb`. The development phase ran
twice on that same code; the second run added the diagnostic columns and gave
identical gate results. The holdout phase ran once.

The results are in
[the evaluation note](../reviews/2026-11-24-smart-match-evaluation.md), with
numbers-only evidence in
`docs/reviews/evidence/2026-11-24-smart-match-evaluation/`.

| Gate | Development | Holdout |
| --- | --- | --- |
| G1 | Pass, 54/54 bit-identical | — |
| G2 | Pass, 18/18 | — |
| G3 | Pass: 761 bands, 0 wrong direction, other bands quiet in 100% | Pass: 277 bands, 0 wrong direction, 100% |
| G4 | Pass, worst 0.018 dB over 288 cases | — |
| G5 | Pass, worst 0.0000001 dB | — |
| G6 | **Failed as written, 62/761 outside 35–65%; accepted with a documented limitation** | **Failed as written, 27/277 outside; accepted with a documented limitation** |
| G7 | Pass, 669 pairs × 5 amounts | — |
| G8 | Pass (`chain_bench`, Slice 4) | — |
| G9 | Pass | — |
| G10 | Pass | — |

**G6 in one paragraph.** Smart Match's EQ stage closes about half the gap, as
designed: the median remaining is 0.51 in development and 0.52 on the holdout,
with only 6 and 4 bands out of range. The full chain then gives part of it
back on loud released masters, because Universal's compressor and limiter
relax when the band that drives them is cut. Released masters account for
60 of the 62 development misses and 26 of the 27 holdout misses. The corpus
mixes miss 2 of 82 and 1 of 47, and the owner's song misses none of 18. No
parameter the plan lets me move reaches that interaction, so nothing was
tuned.

Lanes for Slice 6: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean; `cargo test` 48 suites, 794 passed, 0 failed, 35 ignored. Both bridge lanes were re-run, because the runner module also compiles into the bridges' copy of the library (results recorded in the Checkpoint B report).

### Checkpoint B

Stopped as the plan directs, after the holdout failure (stop trigger 4).
Slice 7 has not started. For the planning session (row M24, item P1):

- grade G6's failure and its decomposition;
- confirm deviation 20 (mono accepted);
- decide deviation 21 (the amount-zero line);
- look at the cross-genre finding: a mismatched reference produces drastic,
  capped moves, with the partial notice as the only warning.

### Deviation log (Slice 6)

32. **How G3 and G6 are measured.** The plan names the cases but not the
    method, so I chose it and fixed it in code before any result came in.
    Copies come from an independent FFT equalizer (65,536-sample sqrt-Hann
    overlap-add), with three kinds of change:
    - brick-wall gains on the meter's band edges, each band ±4 dB;
    - low mids +4 dB with presence −4 dB;
    - a tilt of +1 dB per octave.

    Then:
    - A band counts when its measured gap is 1 dB or more. A boost that the
      missing-band rule reduced is reported, not graded: M16 deliberately
      refuses those.
    - G3's "equals amount × gap before caps" is checked on counted bands where
      no cap bound.
    - G6's re-measured gap is the original gap minus what the match changed
      end to end. That is the matched render minus the unmatched render of the
      same copy, through Universal at default Intensity and default Adapt.
    - The full chain renders whole songs, without the export's loudness
      landing, which does not change band balance.
    - A copy's profile is its original's analysis profile with the copy's own
      measured shares.
33. **G6 also reports two numbers that are not graded**, added after the
    first development run and before the freeze:
    - the EQ stage alone, which separates Smart Match's own accuracy from what
      the chain's dynamics do;
    - the copy rendered with the match against the original rendered without
      it.

    The graded definition did not change.
34. **G1 and G2 use real songs**: 30-second excerpts of the owner's song and
    one library song, all nine presets. G2's self-match uses the song's own
    measured shares.
35. **G5** scales the five development corpus pieces, and three fixed library
    references, by ±12 dB before metering.
36. **G7** covers every known-answer pair and every cross-genre pair at five
    amounts. The partial flag is checked against an independent
    recomputation, and pairs within 1e-4 dB of a cap edge are skipped.
37. **G9** tests synthetic WAVs (silent, 10 seconds, 22.05 kHz, mono), a file
    that is not audio, the five development pieces with their sub band
    removed, and hostile values. Mono is accepted (entry 20).
38. **G10** uses a real song as the reference file. The project is saved, the
    file deleted, and the project reopened: the stored match and the decision
    are unchanged. A preset is then saved and listed, and carries no
    reference.
39. **G8 comes from `chain_bench`** (Slice 4). The runner does not time
    anything.
40. **I labelled the corpus genres myself**, because the corpus list has none.
    The cross-genre picks are one build song per other genre, five per piece.
41. **The fixed references** for G5 and the 30-second report are the first
    build song, by label, in each of three genres.
42. **The 30-second report for the holdout corpus pieces ran in the holdout
    phase**, after the freeze, so holdout material was not examined during
    development.
43. **No parameter was tuned on results.** G6 failed in development. A probe
    on its worst cases showed Smart Match's EQ landing within about 0.1 to
    0.4 dB of the request on the real song. The full chain then gave back up
    to two thirds of it:

    | Case | Intended | EQ alone, on the song | After the full chain |
    | --- | --- | --- | --- |
    | C. Scott – Determinate, sub +4 | −2.0 dB | −1.93 dB | −0.70 dB |
    | Fonkdongo – TamosReady, air +4 | −2.0 dB | −1.60 dB | −1.02 dB |

    The plan's movable parameters are the caps (lower only), the dead zone and
    the fade. None of them reaches that interaction, so I froze at `c61dbfb`,
    unchanged since Slice 5, and ran the holdout once as the protocol says.

### Checkpoint B: graded by the planning session

The planning session graded Checkpoint B on 2026-11-24 (row M24). It inspected
the four commits, re-ran the identity, fingerprint, contract, wire-shape and
Smart Match suites, and checked the committed evidence. Its rulings:

1. **G6 failed as written; accepted with a documented limitation.** It stays
   recorded as a failure. Any EQ ahead of the compressor and limiter is
   partly given back on heavily limited material, and the diagnostics show
   that the EQ stage itself does what was asked. The planning session says the
   gate itself was its own mistake, because it did not separate
   already-limited masters from the mixes the product is built for. No
   retuning, and no second holdout run. Slice 8 states the limitation in
   `docs/PRODUCT.md` and `docs/APP_BEHAVIOR.md`. The Advanced strip gets a
   second locked caption line (see "Locked copy").
2. **A bug to fix before Slice 7.** `SignalChainReadout::from_coeffs` reports
   Warmth or Air as on when Smart Match borrows their idle slot. Those two
   flags must reflect only the user's controls, and `eq_active` must be true
   whenever Smart Match changes anything. Regression tests must cover all
   three filter placements, and every consumer of those flags must be
   checked.
3. **Deviation 20 confirmed.** Mono references are accepted; the G9 text is
   corrected above.
4. **Deviation 21 decided:** a locked line for Match amount at 0.
5. **Deviation 22 decided:** a locked line for a track analysed without a
   six-band profile.
6. **Accepted as reported:**
   - the 92.6% speed figure with both Advanced shelves on;
   - the drastic but capped moves from a badly mismatched reference (M10
     stands).

   Both stay visible in the evaluation note.
7. **Carried to Checkpoint C:** the uncompiled Swift model change, as an open
   Mac item.
8. **How decisions are written.** Dan approves recommendations; he does not
   evaluate technical options himself. Such choices are recorded as "approved
   on the planning session's recommendation". The change of feature is said
   in one plain sentence: Smart Balance (automatic tone fixing) was dropped,
   and Smart Match (the user picks a reference song) replaces it.


### Checkpoint B ruling 2: done (`f3698c8`)

`ChainCoeffs` now records where Smart Match's corrective filter runs
(`SmartMatchSlot`: idle, the Warmth slot, the Presence/Air slot, or a stage of
its own). `warmth_active` and `air_active` reflect only the user's controls,
and `eq_active` is true whenever Smart Match changes anything, including a
chain where only the corrective filter is active. Regression tests cover all
three placements through the real readout, a flat Style, and a
corrective-filter-only chain. Consumers checked: `SignalChain.tsx` reads only
these flags; the preview mock and the wire samples carry no Smart Match
state, and the readout's keys did not change. The try-it engine was rebuilt.

### Slice 7: done (`e79bfde`)

- **Hook** (`useTrackMaster`): the pick (the existing audio file dialog, then
  `measure_smart_match_reference`), a newer pick superseding an older one
  still being measured, refusals that keep the previous reference, the
  switch, remove and Match amount. Each is one settings edit, so one undo
  step. The row keeps the last readout for the same track, dimmed, while a
  fresh one is on its way. Album mode disables every action.
- **View logic** (`src/lib/smart-match.ts`): every copy-table state, built from
  the readout's numbers with the locked copy, including the two lines added
  at Checkpoint B. Names past 60 characters are shortened, with the full name
  on hover and focus.
- **Standard**: one row in the right column under Loudness.
- **Advanced**: the same row as a rail card after Delivery Profile, the
  six-band strip (gap and closed, capped bands marked, every cell labelled for
  screen readers), the Match amount slider and both locked captions. The
  debug `AdaptiveReadout` is untouched.
- **Standard-managed**: the reference and its switch are managed; a
  non-default amount is an Advanced edit, reset to 50%.
- **Receipt**: the render's own `smart_match_digest` under Mastering style.
- **Tests**:
  - 9 hook integration tests: pick, cancel, superseded pick, refusal and
    failure, toggle/remove with undo, amount at 0, stale readout, no six-band
    profile, Album;
  - view, wording and settings-edit tests in `smart-match.test.ts`;
  - `standard-managed` tests;
  - 23 component tests (`SmartMatchRow.test.tsx`), covering every copy-table
    state, the controls, the strip, the slider and the captions;
  - StandardView placement and routing tests;
  - two receipt tests.

  A mutation check (the superseded-pick guard removed) made the superseded
  test fail, as it should.
- **Headless**: five Smart Match scenarios at 1440×900 and 1360×740 (pick and
  interactions in Standard, active, off, Advanced, Advanced partial). The
  Standard geometry probe now also checks the Smart Match controls, the card
  sitting below Loudness, and the line inside its card and the window.

### Deviation log (Slice 7)

44. **Standard placement: a stacked right column.** At 1360×740 the only
    spare height in Standard was inside the Loudness card. The right column
    became a flex column, with Loudness and then the Smart Match card. I
    measured it with and without the row (the row hidden by injected CSS):
    the Intensity card stays 283.3 px, the Loudness options stay 40.5 px, and
    the zone chips and knob do not move. The line ends at 738.9 px, inside
    740. Stop trigger 5 was not reached. The headless probe now checks this at
    both sizes.
45. **The line is clamped in Standard.** At the minimum window it shows two
    lines (four from 860 px tall), so a long applied line with the partial
    notice usually ends in an ellipsis there. The full line is on hover, read
    by screen readers (`role="status"`), and shown unclamped in Advanced. At
    1440×900 the full partial line shows.
46. **The name chip is squeezed in Standard at the minimum window.** It shows
    only a few letters ("War…" in the capture). The full name is on hover and
    keyboard focus, and the line repeats it. Logged as polish in
    `docs/IDEAS_BACKLOG.md`.
47. **Resets.** Tone reset (the EQ and Intensity ↺) leaves Smart Match alone.
    Reset all and the Advanced-to-Standard return keep the reference and its
    switch, and put a non-default amount back to 50% (M13). A song with no
    reference gets no Smart Match state from either.
48. **Removing a reference keeps a non-default amount.** The amount is an
    Advanced edit, so it survives as `{ reference: null, enabled: true,
    amount }` and the next pick uses it. At the default amount, removal leaves
    no Smart Match state at all.
49. **The receipt line (deviation 24's open question).** The desktop receipt
    shows no `Smart Match: off` when the render carried no line, so songs that
    never used Smart Match keep their receipt exactly. The line sits under
    Mastering style in the visible column, not in the collapsed Advanced
    details, because Smart Match is a setting the user chose (hand-test step
    6 looks for it). It always shows the render's digest, never a
    frontend-built one.
50. **The 0% line shows in Standard too.** The amount is set only in Advanced,
    but the row reports it wherever it appears.
51. **Small placement calls.** The Match amount slider is disabled with no
    reference. The Advanced card sits after Delivery Profile.
52. **The refusal line's lifetime.** A refusal is per track and replaces the
    line until the next pick, switch or removal. An amount change does not
    clear it.
53. **Undo coalescing.** Each Smart Match action is one undo step when the
    actions are more than 300 ms apart. Two edits inside that window merge,
    as for every other control. The hook tests wait 320 ms between actions
    to model separate clicks.
54. **The preview mock's sample reference has a neutral name.** The `/app`
    browser preview is reachable on the public site (`vercel.json` rewrites
    `/app`), so the sample reference is "Warm Reference (browser preview)",
    not a filename containing another company's brand. The mock imitates the
    backend's visible rules for the browser only; it is not the decision
    function.
55. **`SmartMatchRow.tsx` joined the landing capture inputs.** The list must
    name every non-test component, and nothing checks that automatically, so
    it would otherwise have been missed. The three scripted captures were
    recaptured (the Standard one now shows the empty Smart Match row). The
    owner's real-session plates are untouched. They predate Smart Match, so
    once it ships the Standard plate no longer shows the current Standard
    view: an owner item (O2) for release time, not for this branch.
56. **Test fakes.** Three App test files and StandardView's `fakeTm` hand-build
    the hook's return value; they gained the Smart Match fields. No product
    code was loosened for them.

### Slice 8: done

- `docs/PRODUCT.md`: a Smart Match (Beta) section, including the G6
  limitation, and the "deferred reference-track UX" line narrowed to what is
  still parked.
- `docs/APP_BEHAVIOR.md`: a Smart Match subsection under Track Master, with
  every line, the controls, the minimum-window clamp, the strip, the G6
  limitation, persistence, resets, the receipt and the signal-chain readout.
- `docs/ARCHITECTURE.md`: the frontend and backend pieces, and where the
  correction sits in the chain.
- `docs/TESTING.md`: the Smart Match headless scenarios; why Smart Balance was
  dropped at Checkpoint A; the measurement tool's role now; the evaluation
  runner; and a table of the lanes a Smart Match change requires.
- `docs/OPEN_THREADS_AND_DECISIONS.md`, `docs/IDEAS_BACKLOG.md` and
  `docs/CHANGELOG.md` (an internal entry, marked as not merged and not a
  release).
- `CLAUDE.md` and `AGENTS.md`, byte-identical: Smart Match joins the product
  contracts.
- No user-facing text says "Smart Balance" (searched the app sources).

### Deviation log (Slice 8)

57. **The measurement tool keeps its old name.** `smart_balance_reference` and
    its TESTING section keep "Smart Balance" in their names, because renaming
    would be churn. The section now opens by saying Smart Balance was dropped
    and why, and that the tool still generates Smart Match's typical shape.
58. **The contract line says a little more than the plan's phrase.** Besides
    "off until the user picks a reference; do not raise its caps or default
    amount without an owner decision", it says that a song without a
    reference must render bit-identically, and it points to the G6 limitation
    in `docs/PRODUCT.md`.

### Checkpoint C

Stopped here. Slices 7 and 8 are built (`e79bfde`, `a267ffb`) on top of the
Checkpoint B fix (`f3698c8`). Nothing is pushed or merged.

Lanes run for Checkpoint C (office PC), on the committed code:

| Lane | Result |
| --- | --- |
| `cargo fmt --check` | Clean |
| `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
| Rust suite with the slow fixture lane (`AMS_RUN_REAL_FIXTURE=1`, `AMS_REAL_FIXTURE_PATH` = `It’s a coat-original-test.wav`) | 48 suites, 796 passed, 0 failed, 35 ignored. The four real-fixture contract tests ran. `preset_byte_identity` and `preset_fingerprint` passed with their goldens untouched |
| iPhone bridge (`cargo check --all-targets`; `cargo test`) | 46 passed, 0 failed, 1 ignored |
| Android bridge (`cargo test`; `cargo ndk -t arm64-v8a --platform 29 check`) | 26 passed, 0 failed; the NDK check built |
| `npm test` | 92 files, 982 passed, 0 failed (includes the try-it engine stamp test) |
| `npm run build` | Succeeded (typecheck included) |
| `npm run verify:headless` | Passed: 50 scenario/viewport checks, including the five Smart Match scenarios at both sizes. Captures inspected (Standard at 1360×740 and 1440×900, Advanced partial) |
| `npm run verify:landing-assets` | Passed after `npm run capture:landing`: 3 scripted captures bound to the new digest; owner plates untouched |
| Try-it engine | No Rust changed since `f3698c8`, where it was rebuilt; its stamp test passes inside `npm test` |

The gate table is unchanged from Checkpoint B: G1–G5 and G7–G10 pass; G6
failed as written and is accepted with a documented limitation (see the
evaluation note).

Open at Checkpoint C:

- **O1 (owner):** the hand test above, then the merge and push decision.
- **Mac:** the Swift model change from Slice 5 (the optional receipt digest
  in `NativeMasteringBridge.swift`) has not been compiled. It needs a Mac
  build before a Mac release.
- **O2 at release time:** the owner's real-session landing plates predate
  Smart Match (deviation 55).
