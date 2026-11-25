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
| E2 Decision rule holds | R1 to R7 hold on the development material through the production path. |
| E3 Holdout | R1, R2, R4 and R7 hold on the holdout songs, run once. |
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

Executor: the auto gain staging session, on `claude/auto-gain-staging-bf8b35`,
started 2026-11-24 from `a8231b5`. Nothing pushed or merged. No file under
`src/` or `src-tauri/src/` changed.

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
