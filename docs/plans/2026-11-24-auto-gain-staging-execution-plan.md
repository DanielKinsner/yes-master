# Auto gain staging: execution plan

Written by the planning session on 2026-11-24 for an executor that was not in
the planning conversation. It follows the
[Smart Match plan](2026-11-24-smart-match-execution-plan.md), whose work is
built and waiting on the owner's hand test.

**What this is.** A decision-complete plan to make the mastering chain
level-aware: it should limit a song only as much as the chosen loudness needs.
This is the unfinished September "smart drive" work, restarted with a simpler
design and a decision rule fixed in advance.

**Approval.** Dan approved the direction and the product rules below on the
planning session's recommendation. He does not evaluate the technical options
himself. Record it that way, and never tell him "you decided".

**Drift rule.** If this plan contradicts current code or docs, verify reality,
follow reality, and note the discrepancy in the deviation log. Do not improvise
a new decision.

**Plans describe outcomes.** Code structure here is a starting hypothesis. The
decisions, contracts, gates and stop triggers are not.

**Where it runs.** The office Windows PC, in a new worktree cut from
`claude/smart-balance-2e58b2`, on its own branch. Private audio lives only in
the main checkout at `C:\Users\SM - Dan\Documents\GitHub\yes-master` and is
referenced by absolute path. Do not work in, build in, or change the Smart
Match worktree: the owner hand-tests from it.

## The problem, in plain words

Every Style adds a fixed push, however loud the song already is. Then the app
measures the result and turns it up or down to land on the chosen loudness.

- **A loud song** is pushed into the saturation and the limiter, flattened, and
  then turned down. Turning it down does not bring the punch back.
- **A quiet song** is not pushed hard enough, and the final gain stops at the
  ceiling, so it can miss the chosen loudness.

Evidence already in the repo:

- `docs/reviews/2026-09-15-mastering-quality-final-synthesis.md`, section 1.
- `docs/reviews/2026-09-14-dynamics-stage-investigation.md`: on the owner's song
  at −9 LUFS, peak-to-average contrast was 8.95 dB with all stages, 9.42 with
  the compressor off, 11.14 with saturation bypassed, and 10.74 with the limiter
  bypassed. The squash comes mostly from saturation and limiting.
- `docs/reviews/2026-09-14-yes-export-vs-landr.md`: the same export's peaks
  ended at −2.6 dBTP under a −1 dBTP ceiling. That unused 1.6 dB is the
  fingerprint of "flattened, then turned down".
- `docs/reviews/2026-11-24-smart-match-evaluation.md`, gate G6: on loud masters
  the compressor and limiter gave back up to about two thirds of a tone change.

## The design, in plain words

Two automatic gains, which is ordinary gain staging:

1. **Level in.** Before the Style's EQ, compressor and saturation, bring the
   song to the level the Styles were voiced at. Every song then meets the Style
   at the same level, so a Style behaves the same on a quiet mix and a loud
   master.
2. **Limiter drive.** After the Style and just before the limiter, add only as
   much gain as the chosen loudness needs. If the loudness can be reached with
   the peaks under the ceiling, the limiter does nothing.

The existing final loudness landing and peak protection stay as the last,
exact step.

September studied a single gain in front of the whole chain. That forced one
knob to do two jobs: pushing a quiet song to its loudness meant driving the
saturation and compressor hard, which changed the tone
(`docs/reviews/2026-09-16-single-first-development-results.md`). Splitting the
gain is this plan's new hypothesis. Phase A tests it before anything is built
into the product.

## Decisions locked

| # | Decision | Consequence |
| --- | --- | --- |
| A1 | The goal is: use as much limiting as the chosen loudness needs, and no more. | No preset is retuned. No limiter, saturation or compressor algorithm changes. |
| A2 | It ships as a switch in Advanced, **Auto gain staging**, with a Beta tag, off by default. | Off is bit-identical to today. Standard shows no new control and follows the default. Turning it on by default is a later owner decision, after his hand test. |
| A3 | It never refuses loudness. By default it reaches the chosen loudness if that takes no more than the **clean limit** of limiting; otherwise it stops at the clean limit and says what it reached. | A song is never squashed past the clean limit unless the user asks. |
| A4 | Advanced offers **Loudness priority: Clean / Reach target**. "Reach target" goes past the clean limit, with the existing export warnings. | Users can still push a song as hard as they like. Nothing that exists today is removed: input gain, output gain, ceiling and custom targets all stay. |
| A5 | The clean limit is the most the limiter may take off the loudest peak automatically. It starts at 6 dB. | Raising it needs the planning session. |
| A6 | The reference level is anchored to the owner's reference song, `It’s a coat-original-test.wav`: that song's level-in gain is exactly 0 dB. | The Style stages meet that song exactly as they do today. |
| A7 | With no loudness target, in Album mode, or with no usable loudness measurement, the feature is inert. | No new behavior on those paths. |
| A8 | Audition stays responsive, Original/Mastered keeps the playhead, and Volume Match stays audition-only. | Gains are resolved off the audio thread. The callback never waits, allocates or boosts on an unverified value. |
| A9 | Every number shown comes from the computation the audio used. | The frontend never recomputes a gain. |
| A10 | Proof is measured gates with held-back songs, then the owner's ears. | This feature changes how a master sounds when on, so the hand test at the end is required. Do not propose any other listening session. |
| A11 | It must work together with Smart Match and must not change Smart Match's decisions. | Smart Match reads the two songs' tone, which gain does not change. |
| A12 | Mobile apps and the browser try-it get no new control. | Their engines must still build and behave exactly as before. |
| A13 | No public claim, landing page or marketing change. | |
| A14 | No push and no merge without the planning session. | The planning session publishes the branch after each checkpoint it has graded. Nothing merges to `main` without the owner. |
| A15 | It must not depend on, or switch on, any gated system. | `ADAPTIVE_COMPRESSION`, `CONFIDENCE_GATING` and `ALBUM_CHARACTER` stay false. |
| A16 | Checkpoints A and B are graded by the planning session. Checkpoint C is the owner's hand test. | |

### Locked copy

Advanced only. Switch label `Auto gain staging`, tag `Beta`.

| State | Line shown |
| --- | --- |
| Off | `Sets the level for this song automatically, so it is limited only as much as your loudness needs.` |
| On, still measuring | `Measuring this song…` |
| On, no limiting needed | `Level in −3.3 dB · limiter drive 0.0 dB · no limiting needed.` |
| On, limiting | `Level in +2.1 dB · limiter drive +4.5 dB · limiting up to 3.1 dB.` |
| On, stopped at the clean limit | `Reached −11.2 LUFS of −9.0. Going louder needs more than 6 dB of limiting.` |
| On, no loudness target | `Choose a loudness target to use auto gain staging.` |
| On, no usable measurement | `Not enough audio to measure this track.` |
| Album mode (disabled) | `Auto gain staging applies to Track Master, not Album renders.` |

Numbers use one decimal place and a real minus sign. The priority control is
labelled `Loudness priority`, with choices `Clean` and `Reach target`, and is
shown only while the switch is on.

Export receipt line: `Auto gain staging: off`, or
`Auto gain staging: level in −3.3 dB · limiter drive 0.0 dB · limiting 0.0 dB`,
with ` · stopped at the clean limit` appended when that applies.

## The contract

Tests must pin each rule.

1. **Inputs.** The source's integrated loudness and true peak from the app's
   own analysis, the loudness target, the ceiling, the switch, the priority
   choice, and the measured level of the signal just before the limiter. It
   does not read the preset, Intensity, EQ or Smart Match directly; those act
   through the measured pre-limiter signal.
2. **Level in.** `level_in = min(NOMINAL_LUFS − source loudness,
   NOMINAL_PEAK − source peak)`, limited to the existing ±24 dB input range. It
   adds to the Style's own push and to the user's input gain. The peak term
   stops a quiet, very dynamic mix from being pushed until its transients clip
   in the saturation stage.
3. **Limiter drive.** A gain applied after the Style stages and immediately
   before the limiter, chosen so the delivered loudness equals the target.
   - If the target is reachable with the pre-limiter peak under the ceiling,
     the drive is exactly the difference and the limiter stays idle.
   - Otherwise the drive is increased until the target is reached.
4. **Clean limit.** The limiter's load is how far the driven pre-limiter peak
   exceeds the ceiling, in dB. With priority `Clean` the drive stops when that
   load reaches the clean limit. With `Reach target` it does not stop there.
5. **Never above the target.** The resolved drive must not leave the delivered
   loudness above the target by more than the landing tolerance. The existing
   landing trims any small remainder.
6. **Activation.** Active only when the switch is on, a loudness target exists,
   the surface is Track Master, and a usable source measurement exists.
7. **Identity.** Switch off or inert: coefficients and rendered bytes are
   identical to today. The limiter-drive stage is skipped entirely when idle.
8. **Pending.** While a new resolution is in flight during audition, level in
   applies at once (it comes from analysis), and the limiter drive may hold or
   fall but never rises on an unverified value. A pending result is never
   labelled settled.
9. **Truth.** The readout and receipt report the gains and the limiter load the
   chain used.
10. **Unchanged stages.** The limiter, saturation, compressor, loudness landing
    and peak-protection code paths keep their algorithms and contracts. This
    plan adds gain in front of them; it does not edit them.

### Parameters

| Parameter | Start | You may move it | Planning session needed for |
| --- | --- | --- | --- |
| `NOMINAL_LUFS`, `NOMINAL_PEAK` | The owner's reference song's own analysed loudness and true peak (about −12.3 LUFS and −1.9 dBTP) | Not at all | Any change |
| Clean limit | 6.0 dB | Lower only | Raising |
| Target tolerance when resolving | 0.2 LU | 0.1 to 0.25 LU | Outside that |
| Maximum resolution passes | 4 | 2 to 6 | Outside that |

Tune only on development material. Freeze parameters and record the commit
before any holdout run.

## Test material

Reference everything by absolute path. Commit numbers only. Free-library and
corpus songs may be named; the owner's files appear as anonymous IDs.

| Set | Where | Role |
| --- | --- | --- |
| Free library, 55 finished masters | `private-audio-fixtures\reference-masters\free-cc-2026-11-24\` | Loud, already-mastered input. Keep the existing seeded 41 / 14 split: the 14 are holdout. |
| September corpus, 8 pieces | `test-output\mastering-quality-20260915\` | Development only. September's holdout was used up (`docs/reviews/2026-09-15-drive-prototype-protocol.md`). Includes three quiet, dynamic stem mixes. |
| Owner's songs | `tests for presets\`, `private-audio-fixtures\*.wav` | Development. The reference song anchors A6. |
| Owner's Suno songs | `private-audio-fixtures\suno-set\` | If any are present at Slice 0, split 75 / 25 by content hash. Empty is not a blocker; say so in the report. |
| Owner's unmastered mixes | `private-audio-fixtures\unmastered-mixes\` | Same rule as the Suno set. |

Do not use `private-audio-fixtures\_held-fma-stream-2026-11-24\`.

If fewer than six quiet, dynamic pieces (source loudness below −18 LUFS) are
available across all sets, say so at Checkpoint A. The planning session will
gather more before the holdout run.

## Phase A: prove the design offline

No product behavior changes in this phase. Build a research runner in the style
of `src-tauri/examples/smart_match_evaluation.rs`. September's metric
definitions and tooling are reusable: `scripts/research/`, and the runners kept
on the tags `archive/codex/mastering-quality` and
`archive/codex/mastering-bidirectional-research` (read them with `git show`; do
not merge the tags).

### Slice 0: preflight

- Confirm the audio locations. Record what the two owner folders contain.
- Baseline lanes on the new branch, as in the Smart Match plan's Slice 0. If the
  baseline is red, report and stop.
- Measure the owner's reference song with the app's analysis and record the
  two nominal values.

### Slice 1: three candidates, measured

For every development song, each of the four Standard Styles at default
Intensity, and each Standard loudness (−14, −11, −9 LUFS), render and measure:

- **T, today.** The production chain unchanged.
- **S, turn-down first.** A fallback design. Where today's chain overshoots the
  target and the landing turns it down, move that reduction in front of the
  chain and repeat until the overshoot is within tolerance. It never raises the
  drive.
- **G, two gains.** This plan's design, with priority `Clean`.

For each render record: delivered loudness, true peak, peak-to-loudness ratio,
loudness range, section contrast loss and attack loss as September defined
them, tone-band deviation from the source, mid/side change, the resolved gains,
the limiter load, and the number of full-track renders needed to resolve.

Also record **Style distinctness**: for each song and loudness, the pairwise
distance between the four Standard Styles' outputs, using the distance already
defined for `src-tauri/tests/preset_fingerprint.rs`.

### Decision rule, fixed in advance

Build **G** if all of these hold on the development material:

| # | Condition |
| --- | --- |
| R1 | Where T and G both land within tolerance of the target, G's peak-to-loudness ratio is never more than 0.1 dB below T's. |
| R2 | G lands within tolerance of the target in at least as many cases as T. |
| R3 | Where T's landing turns the result down by 1 dB or more, G's median gain in peak-to-loudness ratio is at least 1.0 dB. |
| R4 | Where T misses the target by more than 0.5 LU, G reaches the target or stops at the clean limit in every case. |
| R5 | On songs louder than −14 LUFS at the source, median Style distinctness under G is at least that under T. |
| R6 | On the owner's reference song, level in is 0.0 dB, and at each loudness G's tone bands are within 0.5 dB of T's. |
| R7 | Every G render is finite and under the ceiling by the existing qualified peak check. |

If G fails R1, R6 or R7 but **S** satisfies R1, R3 and R7, build S instead.
S then covers only the overshoot case, and the quiet-song case is reported as
open. If neither qualifies, stop.

Report every condition with its number, whether or not it passed.

**Amended at Checkpoint A (planning session, 2026-11-24), frozen before any
holdout run.** As written, the rule selected S. The planning session overrode
it and chose G; see "Checkpoint A: graded" in the execution status for the
history and the reasons. From Slice 4 onward the conditions are:

| # | Condition |
| --- | --- |
| R1b | Where T and G both land within tolerance, G's peak-to-loudness ratio is never more than 0.5 dB below T's. |
| R1c | Over the same cases, compare G with T on section contrast change and on the paired-attack median. On each measure, G may be worse than T by more than 0.5 dB in at most 5% of cases, and by more than 1.0 dB in none. |
| R2–R7 | Unchanged. R4 must hold on the development material before the freeze; the maximum resolution passes may rise to 6, or the resolver may step better. |

R1 as written and R1a (G's limiter load never more than 0.1 dB above T's)
are retired. S is dropped and is not built. The clean limit bounds the
limiter's load, so there is no separate load condition.

### Checkpoint A

Stop. Report in plain language: which design the rule selects, each condition
with pass or fail and its number, what G costs in extra renders and seconds on
this PC, the three or four most instructive songs (include a loud master, a
quiet dynamic mix and the owner's reference song), anything that surprised you,
and the deviation log. The planning session grades this and tells you whether
to build.

## Phase B: engine and export

Build the design the rule selected. The slices below describe G; for S, drop
the limiter-drive stage and the clean-limit rules and keep the rest.

### Slice 2: setting and chain

- Add the switch and the priority choice to `AdvancedSettings` with
  `#[serde(default)]`. Absent means off. Follow the forward-compatibility
  finding already recorded in the Smart Match plan's execution status: a
  project that never used the feature must still open in 0.9.2.
- Add level in to the input gain, and a limiter-drive gain immediately before
  the limiter, in `ChainCoeffs` and `MasteringChain`. The drive stage is skipped
  when idle (contract rule 7).
- The resolved gains reach the chain as backend-resolved values, not as wire
  fields the frontend can set.
- **Cache invariant.** Every cache, hash or equality check keyed on settings
  must distinguish the switch, the priority and the resolved gains. Search for
  them all; start at `audio.rs::settings_landing_hash`.
- **Wasm ripple, expected.** Rebuild the try-it engine if tracked sources
  change, as in the Smart Match plan.
- Tests: off identity against the existing goldens, which stay untouched; the
  level-in rule on synthetic levels and peaks; the reference song gives exactly
  0.0 dB; idle drive stage adds no per-sample work; Smart Match's decision is
  unchanged with the switch on.

### Slice 3: resolve and export

- Resolve the limiter drive for an export from a whole-track measurement of the
  signal just before the limiter, then render. Reuse the existing prepared
  measurement path where it is valid
  (`docs/reviews/2026-09-16-preset-and-target-reuse-results.md`).
- The existing landing and final-rate peak protection run unchanged afterward.
  A render that would exceed the ceiling still errors as it does today.
- Receipts carry the digest line and, when the clean limit stopped the drive,
  the delivered loudness against the target.
- Tests: contract rules 3 to 6 on synthetic and real material; the mirror
  invariant for every type that crosses to TypeScript, Kotlin or Swift.
- Verify: Rust lane, slow fixture lane, both bridge lanes, `npm test`,
  `npm run build`, `chain_bench` against a fresh baseline.

### Slice 4: engine evaluation

Re-run the Phase A measurements through the production export path instead of
the research runner, and confirm they agree with Phase A within 0.1 dB on every
recorded number. Then freeze parameters and run the holdout once.

| Gate | Pass condition |
| --- | --- |
| E1 Off identity | Every preset, switch off or inert: coefficients and bytes equal the baseline. No golden file changes. |
| E2 Decision rule holds | R1b, R1c and R2 to R7 hold on the development material through the production path. |
| E3 Holdout | R1b, R1c, R2, R4 and R7 hold on the holdout songs, run once. |
| E4 Clean limit | With priority `Clean` the limiter load never exceeds the clean limit. With `Reach target` the target is reached wherever the ceiling allows it. |
| E5 Never above target | No delivered loudness exceeds the target by more than the tolerance. |
| E6 Smart Match unchanged | With a reference active, Smart Match's corrections are identical with the switch on and off. |
| E7 Performance | `chain_bench` with the switch off is within 2% of baseline; on is at least 95% of off. Report export time against today for a short, a medium and a long song. |
| E8 Safety | Silent, very short, mono, already-clipped and extremely quiet sources never panic, never produce non-finite audio, and resolve to inert or to a bounded gain. |

Write the results to `docs/reviews/<date>-auto-gain-staging-evaluation.md`,
including failures and what the evidence does not establish.

### Checkpoint B

Stop and report as at Checkpoint A, with the gate table. The planning session
grades it.

## Phase C: live audition, interface, documents

### Slice 5: live audition

- Resolve the gains on the existing preview worker, never in the audio
  callback. Extend that worker's result; do not add a second worker.
- Apply contract rule 8. Updates go through the existing coefficient-update
  crossfade.
- Report the time from a settings change to a settled sound, against today's
  time to a settled loudness, on a short, a medium and a long song.
- Tests and probes: allocation-free callback; no deadline misses in the existing
  native callback probes; no rise on a pending value; a superseded resolution is
  discarded; Original/Mastered switching keeps the playhead; Volume Match still
  compares at matched loudness.

### Slice 6: interface

- Advanced only, beside the existing gain controls: the switch, its line from
  the copy table, and the priority control. Leave Standard unchanged.
- The switch is an Advanced edit under `src/lib/standard-managed.ts`, like other
  Advanced values, until the default is flipped.
- Tests: every copy-table state, the hook's pending and superseded paths, album
  disabled state, and new preview scenarios with reachability at both window
  sizes.
- Verify: `npm test`, `npm run build`, `npm run verify:headless`, and
  `npm run verify:landing-assets` (recapture the deterministic captures if the
  gate asks; never touch the owner's plates).

### Slice 7: documents

Update `docs/PRODUCT.md`, `docs/APP_BEHAVIOR.md`, `docs/ARCHITECTURE.md`,
`docs/TESTING.md`, `docs/OPEN_THREADS_AND_DECISIONS.md`, `docs/IDEAS_BACKLOG.md`
and `docs/CHANGELOG.md`. In `CLAUDE.md` and `AGENTS.md`, kept byte-identical,
add: "Auto gain staging ships off by default; do not turn it on by default,
raise its clean limit, or move its reference level without an owner decision."
State only what was verified.

### Checkpoint C

Stop. Report all lanes and gates, the hand test below, and the deviation log.

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

## Do not touch

- Preset calibration and voicing in `dsp.rs`.
- The limiter, saturation and compressor algorithms, and the loudness landing
  and peak-protection contracts.
- Every `TBD-CALIBRATION` constant and the existing Adapt deadbands and caps in
  `guardrails.rs`.
- The gate statics and their defaults.
- Golden and snapshot references under `src-tauri/tests/golden/`. Never set
  `YES_MASTER_UPDATE_GOLDEN` in this work.
- Smart Match's decision, caps and copy.
- Landing page, release workflows, version numbers, updater configuration.
- Private audio, and the Smart Match worktree.

## Stop and ask

Stop only for these. Otherwise execute without asking.

1. You are not on the office PC, or the test audio is missing.
2. Any golden, snapshot or fingerprint reference would have to change.
3. Switch off is not bit-identical to today.
4. Neither design qualifies under the decision rule.
5. A holdout run fails.
6. A gate can only pass by raising the clean limit, moving the reference level,
   or editing a stage this plan says to leave unchanged.
7. The native callback probes show a deadline miss or an allocation that today's
   build does not.
8. The work would need a preset retune, a gate flip, or a public claim.
9. Push, merge, release, deployment or spending.
10. The worktree contains changes you did not make.

## Deviation log

At every checkpoint, report "where the plan was unclear or wrong and what I did
about it": every judgment call, every plan claim that did not match reality,
every parameter moved and why, and every ripple beyond a slice's stated
footprint. Keep it in an "Execution status" section of this file and in the
evaluation note.

## Parking lot

Not in this plan.

- **Setting Adapt's triggers from real music.** Measured on 2026-11-24 against
  the free library: the "already bright" trigger fired on 0 of 55 songs and the
  "already bassy" trigger on 42 of 55, 34 of them at full strength. That is the
  next plan, kept separate so each change to the sound can be judged alone.
- Turning auto gain staging on by default.
- Retuning any preset, or changing the saturation curve (September's item E).
- A deliberate hard-clipping option past the ceiling.
- Auto gain staging in Album mode, on mobile, or in the browser try-it.
- The Adaptive Compressor, confidence gating and album character.
- A DAW plugin.
- Landing page, marketing, pricing or release work.

## Execution status

### Paused 2026-11-25 (owner out of usage until Friday)

Paused by the planning session in Slice 8. Nothing pushed or merged.

- **Finished:**
  - The Checkpoint C rulings are recorded (below and in the Phase C note).
  - The owner's listening pack: 16 files and `READ ME.txt` in
    `test-output/auto-gain-staging/listening-pack/` (private). Doors Open
    at High is not a fair pair (A 0.4 dB quieter).
- **Half-done, Slice 8 (committed as work in progress):**
  - Built: the fast estimate (`gain_staging::estimate_preview`: parallel
    chunked stage capture and limiter passes, emulated final check), the
    worker sending the estimate then the exact result, the 0.3 dB refine
    rule (`gain_staging_live::audition_gains`), the `estimated` readout and
    the ` · refining…` line, and the probe's F2 timing.
  - Verified at the WIP commit: `cargo clippy --all-targets` clean;
    `gain_staging_live` tests 7 of 7; `tsc` clean; the affected frontend
    tests (146) pass. **Not run:** the full Rust suite, full `npm test`,
    headless, landing gate, bridges, native probes.
  - **No gate is measured yet.** F1's development run was stopped before a
    song finished. F2–F5 are not measured. The estimate now plays in
    audition when the switch is on, before passing F1.
- **Findings worth keeping:**
  - A 4x true-peak reading is useless on a limited master: it reads the
    limiter's own ceiling (−1.10 dBTP) at any drive.
  - The recorded-lowpass half of the qualified peak meter equals the full
    qualified reading on normal music at a fraction of the cost; the
    estimate uses it (`EstimatePeak::Lowpass`).
  - Funkorama and bright noise are dominated by the sinc half (1.4–1.9 dB
    higher on Funkorama), so the estimate misses the final check's
    pull-down there. On a synthetic bright signal with Reach target it came
    out 10 dB *above* the exact drive. F1 must show how often this happens
    on real songs.
  - On normal songs the estimate matched the exact drive to within 0.3 dB
    in 0.5–2.9 s, against 3.5–24 s for the exact resolution.
- **Partial results** (private, under
  `test-output/auto-gain-staging/slice8/`): `export-hashes-08509ab.json`
  (360 exports made with product code equal to `08509ab`, for F4),
  `research-08509ab.exe`, `f1_analysis.py`, `f4_compare.py`.
- **Next step to resume:** rebuild the runner and run F1 on development:
  `--path production --estimate-check "4x:0,lp:0,lp:0.1,lp:0.25"` (with the
  usual discovery arguments and `--quiet`), then
  `python f1_analysis.py estimate-check.json`. If no variant passes F1,
  try one qualified check on the estimate's last pass and weigh it against
  F2, then report the trade-off as the ruling asks.

Executor: the auto gain staging session, on `claude/auto-gain-staging-bf8b35`,
started 2026-11-24 from `a8231b5`. Nothing pushed or merged by the executor.
Phase A changed no product code; Phases B and C did (see each slice).

### Slice 0: done

- **Machine and audio.** The office Windows PC. Every location in "Test
  material" exists. `private-audio-fixtures\suno-set\` and
  `private-audio-fixtures\unmastered-mixes\` are **empty**, so nothing was
  split.
- **Baseline at `a8231b5`**, recorded in
  `test-output/auto-gain-staging/baseline/`:

  | Lane | Result |
  | --- | --- |
  | `cargo fmt --check` | Clean |
  | `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
  | Rust suite with the slow fixture lane (`AMS_RUN_REAL_FIXTURE=1`, `AMS_REAL_FIXTURE_PATH` = `It’s a coat-original-test.wav`) | 48 suites, 796 passed, 0 failed, 35 ignored; the four real-fixture tests ran |
  | `npm ci`; `npm test` | 92 files, 982 passed |
  | `npm run build` | Succeeded |
  | `chain_bench`, release, 7 runs each | Coat (44.1 kHz) median **83.42×** realtime (81.85–83.94); Doors Open (48 kHz) median **91.24×** (87.29–91.43). The binary is kept as `chain_bench-a8231b5.exe` for E7 |

- **Nominal values (A6):** the app's analysis of `It’s a coat-original-test.wav`
  gives **−12.3312845 LUFS** and **−1.882721 dBTP**. Level in for it is
  exactly 0.0 dB.

### Slice 1: done (`ecb9c09`, `682d030`, `bbe56f0`)

The research runner is `src-tauri/examples/auto_gain_staging_research/`,
tested by `src-tauri/tests/auto_gain_staging_research.rs` (13 tests) and
documented in `docs/TESTING.md`. It measured 61 development songs × 4 Styles
× 3 loudnesses × T, S and G: 2,196 delivered masters per run, all finite and
under the ceiling.

The results and the condition table are in
[the Phase A note](../reviews/2026-11-24-auto-gain-staging-phase-a.md), with
numbers-only evidence in
`docs/reviews/evidence/2026-11-24-auto-gain-staging-phase-a/`.

### Checkpoint A

**The rule selects S.** G fails R1 (2 of 594 cases; worst −0.21 dB) and R4
(3 of 126). S satisfies R1, R3 and R7. Stop trigger 4 does not apply, because
S qualifies. Stopped here for the planning session's grade. Phase B has not
started.

| # | G | S |
| --- | --- | --- |
| R1 | Fail: 2 below −0.1 dB, worst −0.210 dB (594 cases) | Pass: worst +0.000 dB |
| R2 | Pass: 674 against T's 594 | Fail: 573 against 594 |
| R3 | Pass: median +2.61 dB (555 cases) | Pass: median +3.57 dB |
| R4 | Fail: 70 reached, 53 stopped at the clean limit, 3 neither (126 cases) | Fail: 0 reached |
| R5 | Pass: 3.93 against 3.58 | Pass: 4.03 against 3.58 |
| R6 | Pass: level in 0.0 dB; largest tone difference 0.364 dB | Fail: 0.897 dB |
| R7 | Pass: 732 of 732 | Pass: 732 of 732 |

Lanes at Checkpoint A (office PC, on the committed runner):

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
| `cargo test --target-dir target\codex-rc` | 49 suites, 820 passed, 0 failed, 35 ignored. The 24 new tests are the runner's 13 and the resampler's 11, which its `#[path]` include compiles in a second time |
| `npm test` | 92 files, 982 passed |
| Slow fixture lane, bridges, try-it | Not re-run: no file under `src-tauri/src/` or `src/` changed since the baseline |

For the planning session:

- only 5 quiet, dynamic pieces (below −18 LUFS) were available, one short of
  six;
- the export's final true-peak check, which is today's code, often decides
  the delivered loudness (surprise 1 in the note);
- S re-voices the Styles at low targets and lands on target in 21 fewer cases
  than today (surprises 2 and 3);
- G's cost: about today's time when one limiter pass suffices, plus 1.5–6 s
  for each extra pass; 0.35 extra passes per render across the set.

### Checkpoint A: graded by the planning session

The planning session graded Checkpoint A on 2026-11-24. It checked the
committed evidence against the report and re-ran the runner's tests. In order:

1. **The rule as written selected S.** G failed R1 (2 of 594 cases, worst
   −0.21 dB) and R4 (3 of 126 cases). S satisfied R1, R3 and R7.
2. **The planning session overrode it and chose G.** Its reasons:
   - The fallback clause was the planning session's own mistake. It let S
     qualify without asking whether S keeps the reference song's tone or lands
     on target as often as today. S fails both: R6 at 0.897 dB, and R2 at 573
     against 594. S also cannot help any quiet song.
   - G's two R1 misses are 0.125 and 0.21 dB. They come from level in letting
     the Style act at its designed level on quieter songs, and in both the
     limiter did less than today's. That is the design working as intended,
     measured against a threshold written too tightly.

   This is recorded as an override, not as a pass.
3. **R1a was tried and dropped.** It said G's limiter load is never more than
   0.1 dB above T's where both land within tolerance. On the development data
   it failed in 26 of 594 cases (worst +1.37 dB).
   - All 26 are songs that level in turned down; 19 of them are at −9 LUFS.
   - With less input, the Style's saturation and compressor make less
     density, so the limiter carries more (median load 3.21 dB against 2.74).
   - The result still keeps more dynamics than today on those cases: PLR
     median +0.34 dB, section contrast +0.15 dB, paired attacks +0.36 dB.

   The planning session dropped R1a as the wrong measure: it asked which stage
   does the work rather than judging the result.
4. **R1b and R1c are the frozen conditions** (see "Amended at Checkpoint A"
   under the decision rule). On the development data:
   - **R1b holds.** The worst case is −0.21 dB against a −0.5 dB limit.
   - **R1c holds:**

     | Measure | Worse than T by more than 0.5 dB | By more than 1.0 dB | Worst case |
     | --- | --- | --- | --- |
     | Section contrast change | 2 of 594 (0.34%) | 0 | −0.737 dB (Small Colin – A Winters Fairytale, Tape, −14) |
     | Paired-attack median | 5 of 594 (0.84%) | 0 | −0.602 dB (owner 15b03e81c8, Tape) |

5. **Carried forward:**
   - R4 must hold on the development material before the freeze.
   - S is dropped.
   - The limiter / final-peak-check disagreement is recorded in
     `docs/OPEN_THREADS_AND_DECISIONS.md` and `docs/IDEAS_BACKLOG.md`. The
     limiter and the final check stay on the do-not-touch list.
   - The planning session added `d0554a7` (no preview builds for this branch)
     and published the branch.
   - More quiet, dynamic material is being gathered into
     `private-audio-fixtures\quiet-dynamic-2026-11-24\`, to split 75/25 by
     content hash at Slice 4.

### Deviation log (Slices 0–1)

1. **Branch.** The desktop app cut the worktree branch
   `claude/auto-gain-staging-bf8b35` from `claude/smart-balance-2e58b2` at
   `a8231b5`, as the plan asks ("a new worktree … on its own branch"). Used
   as it is.
2. **Where the runner lives.** The plan says "in the style of
   `smart_match_evaluation.rs`", which is a crate module. This runner is an
   example only (`examples/auto_gain_staging_research/`), and
   `tests/auto_gain_staging_research.rs` compiles its core module into the
   normal suite. Reason: no file under `src/` changes, so Phase A cannot
   change product behavior, and nothing ripples into the phone bridges or the
   try-it engine. The app's resampler is crate-private, so both crates include
   `src/sample_rate.rs` by path, as `tests/realtime_meter.rs` already does.
3. **How the limiter drive is rendered.** A product change was not allowed in
   Phase A, so a render is split in two: the real chain with its limiter
   swapped for a pass-through one (same lookahead, a ceiling of +200 dBFS),
   which captures the exact signal that reaches the limiter, then a real
   `Limiter` built as the chain builds it, fed that signal times the drive.
   - With no drive the split is bit-identical to the production chain. The
     runner checks it on the first 20 s of every song for every Style, the
     tests check it on synthetic music at 44.1 and 48 kHz, and the timing
     mode checks it over whole songs.
   - With a drive, the drive is one f32 multiply immediately before a real
     limiter: the stage Phase B would add.
   - Level in multiplies the chain's input gain (the Style push plus the
     user's input gain). Phase B would add it in dB before one `powf`; the
     two differ by float rounding only. Slice 4 re-runs everything through the
     product and must agree within 0.1 dB.
   - The limiter's two timing constants are private in `dsp.rs`. The runner
     mirrors them (3 ms, 50 ms); the bit-identity check fails if they drift.
4. **Limiter load.** Measured as the limiter's largest gain reduction over the
   delivered render. To cap the drive at the clean limit without trial
   renders, the runner replicates the limiter's own detector (every sample,
   and 1.012 × the four-phase reconstructed peak). A test pins the replica
   against the limiter's measured reduction within 0.01 dB at 0.5, 3 and
   6 dB, and idle just below the threshold.
5. **How G resolves the drive** (contract rules 3–5). The Style stages run
   once and their loudness is measured. The first drive is the difference to
   the target, capped at the clean limit. When the limiter stays idle at that
   drive, this is exactly the contract's "the drive is the difference". Each
   further step adds the remaining delivered shortfall, still capped. A dB of
   drive adds at most a dB through a limiter, and the export's landing never
   leaves a master above the target, so no step overshoots. Resolution stops
   within 0.2 LU below the target, at the clean limit, or after four limiter
   passes. (Run 1 measured loudness before delivery; see entry 16.)
6. **How S resolves.** The plan gives S no pass limit. It uses the same
   tolerance (0.2 LU) and pass limit (4 full renders, T's included). Its
   overshoot is the chain output's loudness at the source rate minus the
   target. When T does not overshoot by more than 0.2 LU, S is T.
7. **What "today" is.** Standard's export as the app sends it: the frontend
   defaults (`useTrackMaster.ts`), Intensity 0.5, Adapt at its default with
   the backend-resolved profile, Volume Match off, then
   `standardExportSettings`: Custom profile, the target, −1 dBTP, 24-bit,
   44.1 kHz. The chain, the 44.1 kHz conversion and the qualified finalizer
   are the export's own functions. The confidence, adaptive-compressor and
   album-character gates are checked off at start-up.
8. **"Within tolerance"** for R1, R2 and R4 is |delivered − target| ≤ 0.2 LU,
   the plan's resolving tolerance, read from the export finalizer's own
   loudness.
9. **Metrics.** September's definitions, at 48 kHz, through the app's own
   resampler rather than scipy's `resample_poly`. They are consistent within
   this study but not numerically identical to September's tables.
10. **Style distinctness.** `preset_fingerprint.rs` defines its distance on
    synthetic fixtures. On real songs the runner uses the same formula and
    weights on: band tilt over the app's six bands (from the Welch spectrum,
    top band closed at 20 kHz), crest (delivered sample peak minus loudness),
    width change, and half-weighted delivered loudness. The THD term needs the
    harness's 1 kHz probe tone and is left out. A song's distinctness at one
    loudness is the mean of the six pairwise distances. R5 compares medians
    over (song, loudness) cases whose source is louder than −14 LUFS.
11. **R6 read strictly:** all four Styles at each loudness on the reference
    song, not only one Style.
12. **Owner material.** `tests for presets\` holds the reference song plus ten
    files mastered from it (four BandLab, three LANDR, three YES exports). The
    plan lists the folder as development material, so all are used.
    - Two files are byte-identical (`YES-compressor-on.wav` and the
      Universal-75 YES export), so 13 owner songs count, not 14.
    - Ten of the 61 songs are therefore the reference song, which could weigh
      the medians toward it. The Phase A note gives R3 and R5 without them as
      context. The graded numbers include them, and leaving them out changes
      no outcome.
13. **Corpus.** All eight September pieces are development, as the plan
    states. The Smart Match runner had treated three as holdout; September's
    holdout was used up. The corpus "coat" piece is the reference song and is
    counted once.
14. **Suno and unmastered folders** were empty at Slice 0, so nothing was
    split.
15. **Nominal values** come from the app's analysis of the reference song,
    full precision. Level in for that song is therefore exactly 0.0 dB, and G
    reuses T's stage pass for it.
16. **Correction after run 1: G now resolves on the delivered loudness.** Run
    1 resolved G on the chain's output loudness at the source rate. It assumed
    the export's landing would only trim a small remainder. That assumption
    was wrong, and it departed from contract rule 3 ("chosen so the delivered
    loudness equals the target").
    - The export's qualified true-peak check often turned G's masters down
      0.2–2.8 dB after the limiter. That one cause made 30 of G's 34 R4
      failures and 59 of its 66 R2 losses.
    - `bbe56f0` puts the 44.1 kHz conversion and the export's finalizer inside
      every resolution pass. That is the graded run.
    - No rule, parameter or product code changed, and the selection is S in
      both runs.
    - Run 1 stays reproducible with `--g-resolve chain-output`. Its condition
      tables are committed beside run 2's summary. T and S are identical, row
      for row, in both runs.
17. **Run 1's provenance.** Its binary was built from the working tree just
    before `ecb9c09`, which differs from it by rustfmt formatting only. Its
    summary records `682d030`, the HEAD when it finished.
18. **Not tried: a higher pass limit.** The plan allows 2–6 passes, and more
    would likely clear G's three R4 cases. The task forbids tuning to make a
    design qualify, and it would not change the R1 result.
19. **Evidence size.** The committed run 2 summary is the runner's output
    with floats rounded to four decimals and written compactly (1.5 MB instead
    of 3.6 MB). It records the SHA-256 of the full-precision file, which stays
    in the private ledger folder.

### R4 before the freeze: the pass limit is 6

The planning session allowed the maximum resolution passes to rise to 6, or
the resolver to step better, as long as R4 holds on the development material
before the freeze.

- The limit is now 6, inside the plan's 2–6 bound. The step itself is
  unchanged: it adds the delivered shortfall, which can never overshoot the
  target or add limiting that is not needed (A1). A larger step could.
- Only two development songs had renders that ran out of passes at 4
  (Funkorama and C. Scott – Determinate). Re-run at 6:
  - all five such renders now reach the target (3) or stop at the clean
    limit (2), so R4 holds;
  - the other 31 G rows on those two songs are unchanged.

  Every other development render resolved in 4 passes or fewer, so the change
  cannot touch it.

### Slices 2 and 3: done (`6901451`)

**Setting (`types.rs`).**

- `AdvancedSettings.auto_gain_staging: Option<{ enabled, priority: clean |
  reach_target }>`. It is omitted when absent, so a project that never used it
  is written exactly as before, and the public 0.9.2 build reads it (the
  Smart Match finding).
- The resolved gains are `AdvancedSettings.gain_staging_resolved`,
  `serde(skip)`, like `source_confidence` and `compression_guards`. The
  frontend can never set them.
- `MasteringSettings::active_gain_staging()` returns them only with the
  switch on, a loudness target, Track Master and finite values (contract rule
  6).

**Chain (`dsp.rs`).**

- Level in is added to the Style push and the user's input gain, in dB, before
  the existing `powf`.
- A limiter-drive multiply sits immediately before the limiter and is skipped
  entirely when idle (rule 7).
- Inert adds nothing, not even `+0.0`, so coefficients and audio are
  bit-identical.
- `process_frame_inplace` is now four inlined calls: the stages, the drive,
  the limiter and the post-limiter stages. This is a pure refactor; the golden
  byte-identity references are untouched.

**Resolution (`gain_staging.rs`, `engine.rs`).**

- Level in reads `analysis::measure_source_level`. The track analysis now
  calls the same function, so the reference song gives exactly 0.0 dB; the
  slow fixture lane pins it on the real file.
- An export with the feature active works like this:
  - the Style stages run once;
  - each pass runs a fresh limiter (built as the chain builds it), the
    post-limiter stages, the export's sample-rate conversion and its qualified
    finalizer;
  - the drive is chosen on the delivered loudness (rule 3), capped at the clean
    limit for Clean, within ±24 dB, and in at most 6 passes;
  - the last pass is the delivered audio, so no extra render.
- A test pins that the delivered audio equals a chain built from the resolved
  gains, bit for bit (rule 9).
- The landing and the final peak protection run unchanged (rule 10).

**Receipt.**

- `RenderedMeasurements` and `ExportReport` gain `auto_gain_staging_digest`
  (the locked line) and `auto_gain_staging_target_lufs` (present only when the
  clean limit stopped the drive).
- Both are omitted when absent. They are mirrored in `bindings.ts`, the wire
  samples, `export-receipt.ts`, Kotlin `WireModels.kt` and Swift
  `NativeMasteringBridge.swift`.

**Cache invariant.**

- `settings_landing_hash` serializes the settings, so it covers the switch and
  the priority. The resolved gains are added explicitly, together with a
  finiteness guard and a regression test.
- The preparation cache keys on coefficient equality, which already includes
  level in and the drive.
- The live audition never resolves gains yet (that is Slice 5). The switch
  without resolved gains is inert there, so audition sounds as today.

**Tests:** `src-tauri/tests/auto_gain_staging.rs` (15) and module tests (3).

Lanes for Slices 2–3 (office PC, on the committed code):

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
| Rust suite with the slow fixture lane (`AMS_REAL_FIXTURE_PATH` = `It’s a coat-original-test.wav`) | 50 suites, 851 passed, 0 failed, 35 ignored. The four real-fixture tests and the reference-song level-in test ran; `preset_byte_identity` and `preset_fingerprint` passed with their goldens untouched |
| iPhone bridge (`cargo check --all-targets`; `cargo test`) | 46 passed, 0 failed, 1 ignored |
| Android bridge (`cargo test`; `cargo ndk -t arm64-v8a --platform 29 check`) | 26 passed, 0 failed; the NDK check built |
| Try-it engine (`npm run build:tryit-wasm`; `npx vitest run src/tryit`) | Rebuilt (stamp `66493ff2aeb7`); 32 passed |
| `npm test`; `npm run build` | 92 files, 983 passed; the build (with the type gate) succeeded |

### Slice 4: done (`4a0fd0c`, `998b6bc`)

- **Production path.** The runner's `--path production` renders T and G
  through `engine::mastering_render_to_path` and reads the WAV back.
- **Agreement with Phase A.** Within 0.1 dB on every recorded number over
  1,464 rows:
  - audio measures: worst 0.000005 dB;
  - level in: identical;
  - drive and load: within 0.05 dB, read from the receipt's one decimal.
- **Development run (E2).** Through production, 75 songs and 900 cases,
  binary built at `4a0fd0c`. All of R1b, R1c and R2–R7 hold.
- **Freeze.** Parameters frozen at `998b6bc`: clean limit 6.0 dB, tolerance
  0.2 LU, 6 passes, reference level as in A6. Its measurement code is
  identical to `4a0fd0c`; the diff only adds timing tools and docs.
- **Holdout (E3).** Run once at `998b6bc`, through production, on 19 songs
  (228 cases). R1b, R1c, R2, R4 and R7 hold.
- **Write-up.**
  [The evaluation note](../reviews/2026-11-25-auto-gain-staging-evaluation.md),
  with numbers-only evidence in
  `docs/reviews/evidence/2026-11-25-auto-gain-staging-evaluation/`.

### Checkpoint B

Stopped here for the planning session's grade. Slices 5–7 have not started.

| Gate | Result |
| --- | --- |
| E1 Off identity | Pass |
| E2 Decision rule, development, production path | Pass (R1b, R1c, R2–R7) |
| E3 Holdout, run once | Pass (R1b, R1c, R2, R4, R7) |
| E4 Clean limit | Clean: pass (largest load 6.0 dB). **Reach target: fail as written.** 109 of 136 development and 48 of 50 holdout cases reach the target. Of the 27 development misses, 7 (Funkorama) are blocked by the final peak check; the other 20 reach the target with 7–10 passes in a 12-pass diagnostic, against the frozen 6. |
| E5 Never above target | Pass (0 of 1,314 renders) |
| E6 Smart Match unchanged | Pass (test and 94 songs) |
| E7 Performance | Pass. Off −0.40% and −0.56% against the baseline; on 97.1% and 102.6% of off |
| E8 Safety | Pass |

For the planning session:

- **E4's Reach-target half.** Allowing more passes would go outside the plan's
  2–6. A bigger step for Reach target only would be a resolver change. Either
  is the planning session's call; Clean is unaffected.
- **Funkorama** shows today's limiter / final-check disagreement; it is
  already recorded as its own item.
- **Saved user presets** carry the switch and priority (deviation 21).
- **Mac:** the Swift receipt model change is uncompiled.

Lanes for Slice 4 (on the committed code). The Rust suite, both bridges, the
try-it engine, `npm test` and `npm run build` are the Slices 2–3 runs above;
no product code changed since `6901451`.

| Lane | Result |
| --- | --- |
| `cargo fmt --check`; `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
| Runner tests (`tests/auto_gain_staging_research.rs`) | 15 passed (26 with the resampler's included tests) |

### Deviation log (Phase B, Slices 2–4)

20. **Where the resolved gains live.** They sit in a `serde(skip)` field of
    `AdvancedSettings`, the pattern the backend-resolved confidence and
    compressor guards already use. They are applied only through
    `active_gain_staging()`, so a stale value cannot act while the switch is
    off, in Album, or without a target.
21. **Saved user presets carry the switch and the priority.** They are
    ordinary Advanced values like input gain. Smart Match's rule of stripping
    its state from presets (its M12) was specific to Smart Match. The resolved
    gains are never saved. The planning session may want a different rule;
    nothing depends on it yet.
22. **The chain's per-frame function was split** into the stages, the drive,
    the limiter and the post-limiter stages. The plan's starting hypothesis
    was only "a gain immediately before the limiter"; the split lets the export
    run the stages once and the limiter several times without duplicating
    code.
    - It is a refactor with `#[inline(always)]`.
    - The byte-identity goldens pass untouched; E7 measures the cost.
23. **The export renders no extra pass.** The resolver's last pass is the
    delivered audio, so a song that needs one pass costs about what today's
    export costs. A test pins that this audio equals what a chain built from
    the resolved gains produces, bit for bit, through the same conversion and
    finalizer.
24. **Level in is measured from the decoded file at export.** Measuring it
    there, rather than reading the analysis cache, keeps
    `mastering_render_*` usable without the app's profile store (tests,
    runners). It reads the same `ebur128` measurement the analysis reports,
    from the same samples, and costs one extra true-peak pass per export.
    The analysis was refactored to call that one function.
25. **Both gains are bounded to ±24 dB,** level in by contract rule 2 and the
    drive by the same range (E8, "a bounded gain"). This also bounds
    Reach target.
26. **The receipt has two fields.** One is the locked line. The other is the
    target, present only when the clean limit stopped the drive ("the delivered
    loudness against the target"); the delivered loudness is the receipt's
    existing measured loudness.
    - `Auto gain staging: off` appears when the setting exists but is off or
      inert.
    - Nothing appears when the song never used the feature, so receipts and
      the `export_report` golden keep their shape (as Smart Match did).
27. **Wire samples** were refreshed from the `wire_shape` test's own
    generated JSON, after checking that only the new keys differ. The sample
    switches the feature on without resolved gains, so nothing downstream
    changes. `YES_MASTER_UPDATE_GOLDEN` was never set.
28. **Live audition is unchanged until Slice 5.** With the switch on but no
    resolved gains, the live chain is inert.
29. **How Slice 4 renders through production.**
    - T and G go through `engine::mastering_render_to_path` to a scratch WAV
      and are decoded back.
    - T is checked against the research render at the level of 24-bit sample
      codes, with the writer's deterministic dither replayed by
      `DeliveryPcm`.
    - G's drive and load are read from the receipt line, so they come in
      0.1 dB steps. Level in is recomputed exactly with the export's own
      function.
    - T's landing amount, which a receipt does not carry, comes from the
      research render that was just proven identical.
30. **The pass limit is 6** (see "R4 before the freeze").
31. **`chain_bench --auto-gain-staging`** times the chain with level in
    (−2 dB) and an active drive (+1.5 dB) for E7.
32. **The quiet folder's split.** It held 19 pieces with a 19-entry
    `_sources.json` at Slice 4. The 75/25 content-hash split gave 14 to
    development and 5 to the holdout. The pieces are open-licensed, so they
    are named in the evidence, like the library.
    - Their source list is committed as attribution
      (`docs/reviews/evidence/2026-11-25-auto-gain-staging-evaluation/quiet-dynamic-sources.json`),
      with IDs and split; every file's SHA-256 matched it.
    - The set is classical and piano heavy, with no band mixes. That limit is
      stated in the evaluation note.
33. **`--priority reach`** is a diagnostic-only runner option: the research
    path with priority Reach target, used once with `--max-passes 12` on the
    E4 misses. It is part of no gate and changed no frozen parameter.
34. **R6 in the holdout summary reads "fail (0 cases)"** only because the
    reference song is development material. E3 does not include R6.

### Checkpoint B: graded by the planning session

Graded 2026-11-25. Continue to Phase C after three items; still no push or
merge. Each item below was approved on the planning session's
recommendation.

1. **E4, Reach target.** E4 stays recorded as failed as written. Clean keeps
   its never-overshoot step and 6 passes. Reach target only may use a faster
   step, may overshoot, and may run 8 passes. Acceptance, on the 20
   development cases the 12-pass diagnostic reached: within 0.2 LU of the
   target in every one, in at most 8 passes; limiter load no more than 0.5 dB
   above the 12-pass reference's; never above the target after landing (E5).
2. **No-progress stop.** When a pass adds at least 1 dB of drive and the
   delivered loudness rises by less than 0.1 LU, stop. Line: `Reached −11.3
   LUFS of −9.0. This song's peaks stop it going louder under your
   ceiling.` Receipt: ` · stopped by the peak check`.
3. **Pin `Limiter::detector_level`** with a regression across signals, rates
   and channel counts.

Also: report the slowest Reach-target export on the long song and confirm
the export's progress keeps moving across passes; carry the uncompiled Swift
fields to Checkpoint C as an open Mac item; keep the quiet-material limit.

### The three items: done (`5b63cea`)

- **Reach target's step** is a secant through the last two passes, aimed half
  a tolerance under the target, at least the remaining shortfall and at most
  6 dB per pass; at most 8 passes. Clean is unchanged.
- **The no-progress stop** applies to Reach target only (deviation 35).
- **`Limiter::detector_level`** is pinned by
  `the_limiter_level_helper_matches_the_limiters_own_detection`: 4 signals ×
  44.1/48/96 kHz × 1/2/6 channels. A gain putting the detector level exactly
  at the ceiling reduces nothing; 0.1% more reduces about 0.0087 dB; 1 dB
  more reduces about 1 dB. Changing the 1.012 factor to 1.015 fails it.

**E4 re-check, through the production resolver at `5b63cea`**
(`docs/reviews/evidence/2026-11-25-auto-gain-staging-evaluation/reach-target-recheck.json`):

| | Result |
| --- | --- |
| The 20 acceptance cases | **19 of 20 meet every condition.** All 20 land within 0.2 LU (worst −9.17 for −9.0), all in 4 passes, none above the target. **One fails the load condition:** Small Colin – All On Me, Universal, −9: 10.66 dB of limiting against the reference's 10.07 dB, 0.59 dB more where 0.5 dB is allowed. It landed at −9.06 LUFS where the reference stopped at −9.20; the last 0.14 LU cost the extra limiting. Not tuned. |
| Development, 136 cases (where Clean stopped) | 129 within 0.2 LU (was 109). 7 stopped by the peak check, all Funkorama. 0 above the target. Passes: 2 (12), 3 (76), 4 (46), 5 (1), 6 (1). |
| Holdout, 50 cases | 50 within 0.2 LU (was 48). 0 stopped by the peak check. 0 above the target. Passes: 2 (3), 3 (28), 4 (19). |

E4 as written stays failed. Under the amended acceptance it fails on one case
by 0.09 dB; that is for the planning session.

**The slowest Reach-target export on the long song** (Scott Buckley –
Unraveling, 7:24, Universal, single thread): 21.6 s at −9 LUFS against 9.6 s
with the switch off; 11.5–11.7 s at −14 and −11. Progress rose monotonically
to 100% with 49–61 updates and advanced on every pass; its longest pause was
3.4 s, inside a pass, while that pass's conversion and final check ran.

### Slice 5: done

- Resolution on the existing preview worker, before its Volume Match and
  landing measurements, with the export's own resolution
  (`gain_staging::resolve_preview`, pinned bit-identical to the export).
- Rule 8: level in from the analysis (stored in the profile store) applies at
  once; the drive holds what the chain plays and never rises on an
  unverified value; the line reads "Measuring this song…" until it settles.
- Updates go through the existing coefficient crossfade; superseded work is
  cancelled, never stored and never shown.
- Time to a settled sound, office PC (Universal, Intensity 0.75):

| Song | Today, target edit | Switch on | Target edit, on | Already resolved |
| --- | --- | --- | --- | --- |
| Sideways City (2:13) | 0.89–0.96 s | 10.7 s | 8.1–8.2 s | 0.13 s |
| Reference song (4:05) | 1.42–1.44 s | 15.4 s | 14.8–14.9 s | 0.19 s |
| Unraveling (7:24) | 2.37–2.38 s | 42.6 s | 26.6–27.2 s | 0.12 s |

- Probes: 0 callback allocations through the crossfade; 0 deadline misses
  off and on (606 / 607 callbacks; the device granted 480–1056 frames for
  256 requested); Original/Mastered keeps the playhead (≤ 0.01 s); Volume
  Match within 1 LU of the source with the resolved chain.

### Slice 6: done

- Advanced Controls card, under the knobs: the switch with its Beta tag, the
  locked line (including the peak-check line) and `Loudness priority` while
  on. Standard unchanged; Album shows it disabled.
- An Advanced edit under `standard-managed.ts` while on; the card's reset,
  Reset all and the return to Standard turn it off.
- `npm run verify:headless` passed with two new scenarios at both window
  sizes. The landing gate asked for new scripted captures, retaken with
  `npm run capture:landing`; the owner's plates were not touched.

### Slice 7: done

`PRODUCT`, `APP_BEHAVIOR`, `ARCHITECTURE`, `TESTING`, `OPEN_THREADS`,
`IDEAS_BACKLOG`, `CHANGELOG`, and the line in `CLAUDE.md` and `AGENTS.md`
(byte-identical). Phase C note:
`docs/reviews/2026-11-25-auto-gain-staging-phase-c.md`.

### Checkpoint C

Stopped here for the owner's hand test (in the Phase C note and above).
Nothing pushed or merged.

Commits on top of `5b63cea` (Checkpoint B's three items):

- `560a7a2`: the E4 re-check evidence and the Reach-target export timing;
- `97a2854`: Slice 5, live audition;
- `6586a1b`: Slice 6, the interface;
- `01be25b`: the scripted landing captures;
- the commit carrying the Phase C note: Slice 7, the documents.

Lanes (this PC):

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

Open:

- **E4 under the amended acceptance:** one of 20 cases, 0.09 dB over the
  load condition. The planning session's call.
- **Audition settling time** (above). Ideas recorded in
  `docs/IDEAS_BACKLOG.md`; none tried.
- **Mac:** the Swift receipt model's two optional fields are uncompiled.
- **The quiet material** is classical and piano heavy; nothing establishes
  how G treats a typical unmastered band mix.

### Checkpoint C: graded by the planning session

Graded 2026-11-25: the build is accepted, with two more pieces of work before
the owner's hand test. Approved on the planning session's recommendation:

- **E4's one case** (Small Colin – All On Me, Universal, −9, 0.59 dB more
  limiting than the 12-pass reference) is **failed as written by 0.09 dB and
  accepted by the planning session**: it landed 0.14 LU closer to the target
  than the reference did, and that cost the extra limiting.
- **Audition settling time** needs work before the owner listens in the app,
  because a sound that shifts 15 seconds after each tweak makes tuning by ear
  confusing. That is Slice 8.
- **First, a listening pack**: Universal at default Intensity, the reference
  song and three of the owner's songs, Medium and High, switch off and on
  (Clean), through the production export, kept private.
- **Slice 8, fast settling:** a fast estimate of the limiter drive first, then
  the exact resolution, on the existing preview worker. Rule 8 is amended for
  this slice only: an estimate that has passed gate F1 may raise the drive in
  audition; a value left over from different settings still may not. Exports
  never use the estimate. Gates F1–F5; F1 is checked once on the holdout (its
  second use, for a different measure). If F1 and F2 cannot both be met, stop
  and report the best trade-off.

### Deviation log (Checkpoint B items and Phase C)

35. **The no-progress stop applies to Reach target only.** Clean was frozen
    by the Checkpoint B grade, and Clean already stops at the clean limit, so
    a Clean search never adds 1 dB without progress before stopping there.
36. **Reach target's step** is a secant through the last two passes, its
    slope clamped to 0.1–1, aimed at the target minus half the tolerance, and
    clamped between the remaining shortfall and 6 dB (or the shortfall when
    that is larger). The first pass, and Clean, are unchanged.
37. **The level in audition plays while pending** comes from the analysis,
    newly stored in the profile store at analysis time. The resolution itself
    measures the decoded file once per song, exactly as the export does
    (deviation 24), so the settled gains equal the export's.
38. **A readout command and a status event were added,** beyond the plan's
    "extend the worker's result": `auto_gain_staging_readout` resolves the
    selected song when it is not playing, so the line has something true to
    show, and `auto-gain-staging:status` carries the loaded song's live line,
    edge-triggered like the landing status. Both share the one permit and the
    one cache with the worker.
39. **Preparing the preview level also resolves** when the switch is on, so
    the prepared landing is measured on the chain audition will play.
40. **Switching off keeps `{enabled: false, priority}`** in the settings, so
    the receipt reads `Auto gain staging: off` and the priority survives;
    Standard's reset and the card's reset clear it.
41. **The row reuses Smart Match's look** through shared CSS selectors, with
    its own class names so nothing that looks for Smart Match's switch can
    match it.
42. **Test support beyond the slice's files:** three App test fakes gained
    the hook's new view (as they did for Smart Match), and one reset-payload
    expectation gained `auto_gain_staging: null`.
43. **Runner options:** `--reach-check` (the E4 re-check) and
    `--timing-reach` (Reach-target export timing with the progress trace).
44. **Native probe and bench variant (test-only):**
    `mastering_quality_auto_gain_staging_live`, and
    `YES_MASTER_CALLBACK_AUTO_GAIN_STAGING=1` for the callback bench.
45. **Audition settling time was reported, not optimized.** The plan asks for
    the report; speeding it up would change how audition resolves and is
    recorded for the planning session.
