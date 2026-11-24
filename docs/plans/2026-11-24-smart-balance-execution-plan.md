# Smart Balance: execution plan

Decided with the owner (Dan) on 2026-11-24. Written for an executor that was not
in the planning conversation. Verified against `main` at `ebee875`.

**What this is.** A decision-complete plan for the first "smarter mastering"
feature: **Smart Balance**, an automatic tone correction that runs before the
chosen Style, plus a readout that shows what it did. It serves the long-open
Tier-2 "smart" milestone (`docs/OPEN_THREADS_AND_DECISIONS.md`, row 10;
`docs/IDEAS_BACKLOG.md`, "Adaptive engine — Tier-2").

**Drift rule.** If this plan contradicts current code or docs, verify reality,
follow reality, and note the discrepancy in the deviation log. Do not improvise
a new decision. Owner decisions in the table below are not yours to reopen.

**Plans describe outcomes.** Per `CLAUDE.md`, the code structure suggested here
is a starting hypothesis. Replace it when evidence supports that, and say so.
The decisions, contracts, gates and stop triggers are not hypotheses.

**Where it runs.** Only on the office Windows PC. The test audio is not in git
and exists only there (see "Verified ground"). On any other machine, stop.

## In plain language

Today a Style works like a creative LUT dropped straight onto ungraded footage.
The only adaptation is dialing the LUT back when the footage already looks that
way. Smart Balance is the auto white balance that runs first: it measures the
song's tone, nudges anything unhealthy back toward a healthy range, and then the
Style applies on top as it always did.

The owner's own test song shows the gap. All three saved LANDR masters cut about
1 dB at 120–500 Hz and added about 1 dB of deep bass, whatever the style. YES
Master left that region within 0.2 dB, because it cannot make a corrective move
on its own (`docs/reviews/2026-09-14-yes-export-vs-landr.md`, "Tone and stereo").

## Decisions locked

| # | Decision | Consequence |
| --- | --- | --- |
| D1 | "Smarter" means rule-based decisions from measurements. No trained model, no chat assistant. | Nothing in the product or docs is called "AI". |
| D2 | The engine gets smarter first; a DAW plugin comes later. | The decision code takes a measured profile and returns decisions. It must not care whether the profile came from a whole file or a short listen. |
| D3 | Phase 1 is Smart Balance plus its readout. | Reference matching, smart drive and the Adaptive Compressor are out of scope (see Parking lot). |
| D4 | It ships as a switch that is **off by default** during the beta. | With the switch off, output is bit-identical to today. Flipping the default is a later owner decision. |
| D5 | Proof is measured pass/fail gates, with songs held back from tuning. No formal listening sitting. | The owner tries the switch in normal use. Do not propose a listening session. |
| D6 | The name is **Smart Balance**, shown with a small **Beta** tag. | "Smart Match" is reserved for phase 2. |
| D7 | Standard view: a switch with one plain result line. Advanced view: the switch plus a tone strip showing six bands against the healthy range and what moved. | Copy is locked below. |
| D8 | The healthy range is measured from the owner's own folder of at least 40 released songs. | Only numbers are committed. No audio, file names, artists or titles enter git. |
| D9 | Gentle. The share of the out-of-range excess that gets corrected equals Adapt Strength (default 0.5, so half). Hard caps per band: 3 dB cut, 2 dB boost. | Anything more aggressive needs the owner. Less aggressive is allowed and logged. |
| D10 | It listens to the song, never to the user's knobs. | Manual EQ, Style and Intensity do not change the correction. The Style and the existing guardrail trims apply unchanged on top. |
| D11 | Album mode stays unadapted, as today. | In Album the control is disabled with an explanation. |
| D12 | It is an ordinary settings field, default off, with one special rule: it is **Standard-managed**, the fifth such field after style, intensity, loudness and format. | Old projects open with it off. Standard↔Advanced moves and Reset all preserve it. No new persistence behavior is invented. |
| D13 | If analysis fails, or the clip is too short or silent, it stands by and says so. | Export is never blocked by Smart Balance. |
| D14 | Mobile apps and the browser try-it get no Smart Balance UI in this plan. | Their engines must still build and behave exactly as before. |
| D15 | No public claim. | No landing page, store, changelog-for-users or marketing copy changes here. |
| D16 | Healthy range per band = library median ± the larger of (2 × robust spread) and 1.5 dB. | It may be widened on evidence. It may never be narrowed without the owner. |
| D17 | Cuts are freer than boosts, and it never boosts what is not there. | A band that is essentially absent (solo piano has no sub) gets no boost. |
| D18 | Every number the user sees is resolved by the backend from the same computation the audio uses. | The frontend never recomputes a correction. |
| D19 | Work happens on branch `feat/smart-balance` in small commits. | No push and no merge to `main` without the owner's word at a checkpoint. |
| D20 | A "30-second listen versus whole song" comparison is measured and reported, not gated. | It is the number the future plugin needs. It changes no product behavior. |
| D21 | Smart Balance must not depend on, or switch on, any gated system. | `ADAPTIVE_COMPRESSION`, `CONFIDENCE_GATING` and `ALBUM_CHARACTER` stay false. |

### Locked copy

Standard row, label `Smart Balance` with tag `Beta`:

| State | Line shown |
| --- | --- |
| Off | `Fixes muddy, dull or harsh mixes before your Style is applied.` |
| On, analysis not ready | `Listening to your track…` |
| On, nothing out of range | `Already balanced. Nothing to fix.` |
| On, correction applied | Built from the table below, for example `Cleared 1.2 dB of mud, added 0.8 dB of air.` |
| On, no usable analysis | `Not enough audio to analyze this track.` |
| Album mode (disabled) | `Smart Balance applies to Track Master, not Album renders.` |
| Advanced, Adapt strength at 0 | `Adapt strength is at 0, so Smart Balance is idle.` |

Correction sentence: list moves of 0.3 dB or more, largest first, at most three,
joined by commas. Capitalize the first phrase only. One decimal place.

| Band | Cut wording | Boost wording |
| --- | --- | --- |
| Sub, 20–80 Hz | `Tamed X dB of sub rumble` | `Added X dB of deep bass` |
| Bass, 80–250 Hz | `Tamed X dB of boom` | `Added X dB of bass` |
| Low mids, 250–800 Hz | `Cleared X dB of mud` | `Added X dB of warmth` |
| Mids, 800 Hz–2.5 kHz | `Eased X dB of midrange` | `Added X dB of midrange` |
| Presence, 2.5–6.5 kHz | `Softened X dB of harshness` | `Added X dB of presence` |
| Air, 6.5–16 kHz | `Softened X dB of brightness` | `Added X dB of air` |

Tone strip caption: `Healthy range measured from N released masters.` with the
real count. Column labels: Sub, Bass, Low mids, Mids, Presence, Air.

Export receipt line: `Smart Balance: off`, or `Smart Balance: on · already
balanced`, or `Smart Balance: on · cleared 1.2 dB mud, added 0.8 dB air`.

## Verified ground (2026-11-24, `ebee875`)

| Claim | Evidence |
| --- | --- |
| Active adaptation only reduces preset moves | `src-tauri/src/guardrails.rs`, module header |
| The three gated systems are off | statics in `guardrails.rs`, `confidence.rs`, `album.rs` |
| No corrective tone, reference matching or genre code exists | repo search |
| Each EQ band already sums preset + guardrail trim + user offset | `dsp.rs`, `ChainCoeffs::from_settings`, the `effective_*_db` lines |
| The source profile carries six level-invariant band shares | `types.rs`, `SourceProfile`, `SpectralBalance6`; edges 20/80/250/800/2500/6500/16k Hz in `analysis.rs::compute_spectral_balance_6band` |
| Album resolves to no profile, so adaptation is inert there | `profile_store.rs::resolve_effective_profile` |
| Standard holds a hard "non-managed fields are default" invariant | `src/lib/standard-managed.ts` |
| The existing per-axis adaptive readout is debug-only | `AdaptiveReadout` in `src/components/AdvancedPanel.tsx`, `src/lib/debug-flags` |
| September's smart-drive research is merged and unfinished | `docs/reviews/2026-09-21-research-main-integration.md`; `4c139b4e` is in `main` |
| Test audio is present on this PC only | `tests for presets/` (11 files), `test-output/mastering-quality-20260915/`, `private-audio-fixtures/`; all git-ignored |
| The corpus has a recorded development/holdout split | `docs/reviews/evidence/2026-09-15-mastering-quality/corpus.json`, five development, three holdout |
| Toolchain is ready here | Rust 1.100.0, `wasm32-unknown-unknown`, wasm-bindgen 0.2.128, Android targets |
| Another agent commits to this checkout in parallel | `ebee875` landed during planning |

## Lanes

### Owner lane

| # | Item | Blocks |
| --- | --- | --- |
| O1 | Put 40 or more released songs you trust, across the genres you want YES Master to serve, into `private-audio-fixtures/reference-masters/` on the office PC. MP3, AAC/M4A, FLAC, WAV, AIFF and OGG work. Apple Lossless and copy-protected files do not. | Slice 2 and everything after it |
| O2 | Grade checkpoint A (the measured healthy range). | Slice 3 |
| O3 | Grade checkpoint B (the measured results) and say whether the UI should be built. | Slice 7 |
| O4 | At checkpoint C: run the hand test, then decide on merge and push. | Merge to `main` |
| O5 | Later, outside this plan: decide when the default flips to on. | Nothing here |
| O6 | Later, outside this plan: decide whether the landing-page plates should be re-shot to show the new row. | Nothing here |

### Executor lane

Slices 0 to 8 below, in order. Slice 1 can be built before O1 is done. Slice 2
cannot.

## The contract

These rules define Smart Balance. Tests must pin each one.

1. **Inputs.** The source's six band shares (`SpectralBalance6` from the resolved
   `SourceProfile`), the effective Adapt Strength, the switch, and the committed
   healthy-range table. Nothing else. In particular not the preset, Intensity,
   manual EQ, loudness target or any gated analysis.
2. **Deviation.** Express each band in dB against the library's typical shape,
   then remove the overall offset robustly, so one loud band does not make the
   other five look quiet. Starting hypothesis: subtract the library median per
   band, then subtract the median of the six results.
3. **Range.** Inside the range a band's correction is exactly zero.
4. **Correction.** Outside the range, the intended correction is
   `strength × (distance beyond the nearest range edge)`, toward the range. It
   therefore never crosses the edge, and never reaches the center.
5. **Caps.** Intended correction per band is limited to −3.0 dB and +2.0 dB.
6. **Dead zone.** Intended corrections smaller than the dead zone become exactly
   zero, so near-misses stay bit-identical and no pointless filter is created.
7. **Missing bands.** A boost fades to zero as a band falls far below the range,
   because that is an arrangement choice or a structurally empty band (for
   example air at low sample rates), not a flaw to fix.
8. **Activation.** Active only when the switch is on, a source profile is
   resolved, and effective strength is above zero. This is the same condition
   the guardrails use, so Album and "no analysis" are inert without new code
   paths.
9. **Realization.** The intended six-band correction becomes EQ gain. Starting
   hypothesis: add a correction term to the seven existing bands' effective
   gains in `ChainCoeffs::from_settings`, solving for those gains from the
   bands' real responses at the current `eq_bands` frequencies. That adds no
   audio stage, no latency and no per-sample cost. A separate corrective stage
   is acceptable only if the accuracy gate cannot be met otherwise; it then
   must be skipped entirely when idle and pass the performance gate.
10. **Identity.** Switch off, or on with every correction zero, produces
    coefficients and rendered bytes identical to today.
11. **Truth.** The readout and receipt report the corrections the chain used,
    from one shared computation.

### Parameters

| Parameter | Start | You may move it | Owner needed for |
| --- | --- | --- | --- |
| Cut cap / boost cap | 3.0 / 2.0 dB | Lower only | Raising |
| Strength mapping | fraction = Adapt Strength | Not at all | Any change |
| Range multiplier (× robust spread) | 2.0 | 2.0 to 3.0 | Below 2.0 |
| Minimum range half-width | 1.5 dB | 1.5 to 3.0 dB | Below 1.5 |
| Dead zone | 0.25 dB | 0.1 to 0.5 dB | Outside that |
| Missing-band fade start / end (below lower edge) | 6 / 12 dB | 4–8 / 8–15 dB | Outside that |
| Sub-band boost cap | 2.0 dB | Down to 1.0 dB | Raising |
| Minimum library size | 40 songs | Not at all | Fewer |

Tune only on development material. Freeze every parameter, and record the
commit, before any holdout run.

## Slices

Per-slice verification lists are a **floor, not a ceiling**. After each slice,
re-read the conditional rules in `CLAUDE.md` and `docs/TESTING.md` against your
actual diff. A slice that grows beyond its predicted footprint grows its
verification with it. Run Cargo suites serially in one target directory, with
`--target-dir target\codex-rc`, as `docs/TESTING.md` describes.

### Slice 0: preflight (no code)

- Confirm the machine: the three audio locations in "Verified ground" exist.
- `git status`, `git fetch`. If the checkout holds another agent's uncommitted
  work, stop and ask. Create `feat/smart-balance` from current `origin/main`.
- Establish a baseline on that base: from `src-tauri`, `cargo fmt --check`,
  strict Clippy, `cargo test`; from the root, `npm test` and `npm run build`.
  If the baseline is red, report it and stop. Do not fix unrelated failures.
- Record baseline `chain_bench` output for the performance gate.

### Slice 1: reference measurement tool

Measurement only. No product behavior changes.

- Add a local-only runner in the style of `src-tauri/examples/private_reference_tuning.rs`.
  It takes a folder (searched recursively) and an output directory.
- For each decodable file it measures the six band shares **through the same
  code the app's analysis uses**. A second implementation of the measurement is
  not acceptable, because the range would then describe a different meter.
- It skips, with a logged reason and without failing: files under 30 seconds,
  silent files, undecodable files, and non-audio files.
- Outputs: a private per-file ledger under `test-output/` (ignored), and an
  aggregate statistics file that is safe to commit. The aggregate holds song
  count, per-band median, robust spread, range edges, edge stability from a
  seeded bootstrap, the seed, and split membership by anonymous hash prefix.
- Deterministic: the same folder gives byte-identical statistics.
- Follow the output-path rules in `docs/TESTING.md` ("Evidence-runner output
  paths") and the pattern pinned by `src-tauri/tests/evidence_runner_paths.rs`.
- Tests: aggregate math on synthetic spectra, determinism, skip rules, path
  safety.
- Verify: `cargo fmt --check`, strict Clippy, focused tests, then `cargo test`.

### Slice 2: measure the library and record the range

Depends on O1.

- Run the tool on `private-audio-fixtures/reference-masters/`. Reference it by
  absolute path if you are in a worktree. Never copy private audio.
- Fewer than 40 accepted songs is a stop trigger.
- Split the library 75/25 by seeded hash into **build** and **held-out healthy**.
  The range is computed from the build split only.
- Commit the aggregate statistics and a review note,
  `docs/reviews/<date>-smart-balance-reference-range.md`. The note states the
  range per band in plain words and shows where these sit against it: the eight
  corpus pieces, the owner's `It’s a coat-original-test.wav`, and the saved
  LANDR and BandLab masters in `tests for presets/`. (That file name contains a
  curly apostrophe and spaces; quote it and do not retype the apostrophe.)
- The note must also say plainly if the library looks skewed (for example one
  genre dominating, or an unstable edge).
- Verify: docs checks only. Confirm `git status` shows no audio and no private
  file names staged.

### Checkpoint A

Stop. Report to the owner in plain language: how many songs were measured, the
healthy range, where the owner's test song sits, anything odd, and the deviation
log. Wait for the owner before Slice 3.

### Slice 3: the decision function

Pure logic. Not yet connected to audio.

- Implement contract rules 1 to 7 as one deterministic function with no
  dependency on gated statics, usable on desktop, mobile and wasm.
- The range table is emitted by the Slice 1 tool from the committed statistics.
  Numbers are never hand-copied. A test asserts table and statistics agree.
- Tests, one per rule, including:
  - in range gives exactly zero in every band;
  - a +6 dB flaw in one band corrects that band and moves every other band by
    at most 0.5 dB;
  - correction scales with strength and is zero at strength zero;
  - caps hold at extreme inputs;
  - a band far below range gets no boost; air is ignored when it is
    structurally zero (see the existing test
    `spectral_balance_6band_air_is_structurally_zero_below_13khz_rates`);
  - non-finite, zero or degenerate shares give "inert", never a panic.
- Verify: Rust lane.

### Slice 4: the setting and the chain

This is the first slice that can change sound, and only when the switch is on.

- Add `smart_balance: Option<bool>` to `AdvancedSettings` with
  `#[serde(default)]`. `None` and `Some(false)` both mean off.
- Forward compatibility: a project saved by this build with Smart Balance never
  switched on must still open in the public 0.9.2 build. The tagged 0.9.2
  `types.rs` has no `deny_unknown_fields` (checked 2026-11-24 with
  `git show yes-master-v0.9.2-manual-10:src-tauri/src/types.rs`), which suggests
  an unknown field is ignored. Confirm that on the actual project-file loading
  path at that tag, choose serialization accordingly, and record what you found.
- Wire contract rules 8 to 10 into `ChainCoeffs::from_settings`.
- **Cache invariant.** Every cache, hash or equality check keyed on settings
  must tell Smart Balance on from off. Search for them all (start at
  `audio.rs::settings_landing_hash` and the hook's readout identity); do not
  trust this list. Add a regression that toggling the switch invalidates the
  preview loudness result.
- **Wasm ripple, expected.** `engine-stamp.test.ts` goes red when tracked
  sources change. Add any new module to the `web/tryit/wasm` crate, run
  `npm run build:tryit-wasm`, commit `src/tryit/engine/`, and confirm
  `npm test -- src/tryit` passes. The browser demo's behavior must not change.
- Tests:
  - the existing `preset_byte_identity` and `preset_fingerprint` suites pass
    with their goldens untouched;
  - for every preset, coefficients with the field `None`, `Some(false)`, and
    `Some(true)` with an in-range profile are identical to before;
  - realization accuracy (gate G5);
  - level invariance (gate G6);
  - album stays inert;
  - with user-moved band frequencies the correction never has the wrong sign.
- Verify: Rust lane; slow fixture lane (`AMS_RUN_REAL_FIXTURE`, per
  `docs/TESTING.md`); iPhone bridge lane; Android bridge lane; frontend tests
  for the try-it engine; `chain_bench` against the Slice 0 baseline.

### Slice 5: readout and receipt

- Extend `GuardrailReadout` with an optional tone-balance section so the
  existing `guardrail_readout` command carries it. It holds the state (one of
  the copy-table states), six bands (position against the range, range edges,
  correction applied) and the library song count.
- Add a tone-balance digest to `RenderedMeasurements` and `ExportReport`,
  following the existing `source_profile_digest` pattern, using the locked
  receipt wording.
- **Mirror invariant.** Every place that mirrors these Rust types must match:
  search for them, starting with `src/bindings.ts`, `src/bindings-drift.test.ts`,
  `src-tauri/tests/wire_shape.rs`, `src/lib/preview-mock.ts`, the export-receipt
  helpers, and the Kotlin and Swift wire models.
- Verify: Rust lane, `npm test`, `npm run build`, both bridge lanes.

### Slice 6: measured evaluation

Build a runner that produces the evidence, then run it. Commit numbers only;
private songs appear as anonymous IDs.

| Gate | Pass condition |
| --- | --- |
| G1 Off identity | Every preset, switch off: coefficients and rendered bytes equal the baseline. No golden file is updated. |
| G2 In-range identity | Switch on, in-range source: bytes equal switch off. |
| G3 Healthy masters left alone | On the held-out healthy songs at default strength: at least 70% get zero correction in every band; at least 95% get no band correction above 1.0 dB; none above 2.0 dB. |
| G4 Induced flaws recovered | Take held-out healthy songs, apply a known ±6 dB flaw to one band with an independent filter, re-measure. Where the flaw leaves the range by 1 dB or more: the correction opposes it in at least 90% of cases, matches `strength × excess` within 0.5 dB before caps, and other bands move 0.5 dB or less in at least 90% of cases. |
| G5 Realization accuracy | The six-band change measured through the real EQ coefficients is within 0.5 dB of intended, for every single-band cap case and 20 seeded mixed cases, at 44.1, 48 and 96 kHz. |
| G6 Level invariance | The same source at +12 and −12 dB gets the same correction within 0.1 dB. |
| G7 Full render | For flawed fixtures rendered with Universal at default Intensity, the flawed band's excess shrinks by at least half the intended correction. |
| G8 Performance | `chain_bench` with the switch off is within 2% of baseline; switch on is at least 95% of switch off. Toggling the switch goes through the existing coefficient-update path with no new transition mechanism. |
| G9 Safety | Silent, very short, mono and low-sample-rate sources never panic and never boost an empty band. |

Also report, without gating:

- **Corpus sanity.** For each development piece, the correction per band. More
  than two bands at a cap on one song suggests a target mismatch: investigate
  and report.
- **30-second listen versus whole song** (D20). For each corpus piece and ten
  library songs, the correction from the loudest contiguous 30 seconds against
  the whole file: largest per-band difference, and the share within 0.5 dB.
- **The owner's song.** Whether Smart Balance moves "It’s a coat" toward what
  the three LANDR masters did. One song is context, not proof.

**Holdout protocol.** Tune on the build split and the five development pieces.
Freeze parameters and record the commit. Then run G3 and G4 on the held-out
songs, and the corpus sanity report on the three holdout pieces, **once**. A
holdout failure is a stop trigger. Do not retune against the holdout.

Write the results to `docs/reviews/<date>-smart-balance-evaluation.md`,
including failures and what the evidence does not establish.

### Checkpoint B

Stop. Report in plain language: each gate with pass or fail and its number, the
three reports, parameter values and any that moved, and the deviation log. The
owner decides whether the UI gets built.

### Slice 7: the interface

- Plumb the switch through the existing settings-update path in
  `src/hooks/useTrackMaster.ts`. Read the readout from the existing
  `guardrailReadout`.
- **Standard** (`src/components/StandardView.tsx`): one row with the label, the
  Beta tag, the switch and the result line. Standard must still fit the
  supported minimum window, 1360×740, with every existing control visible and
  reachable before scrolling. If it cannot fit without shrinking or removing an
  existing control, that is a stop trigger.
- **Standard-managed** (`src/lib/standard-managed.ts`): Smart Balance is not a
  non-managed edit. `hasNonManagedEdits` ignores it; the reset functions and
  Advanced's Reset all preserve it.
- **Advanced** (`src/components/AdvancedPanel.tsx`, beside Adapt strength): the
  switch and the tone strip. Each column shows the healthy range, the track's
  position, and the corrected position when one applies. Values are available
  to keyboard and screen-reader users. Leave the debug-only `AdaptiveReadout`
  as it is.
- **Album**: the control is disabled with the locked explanation, following the
  existing Album Adapt strength pattern.
- Use the locked copy exactly. Build the sentence from backend numbers.
- Tests: component tests for every copy-table state; hook integration for
  toggle, stale readout and album; `standard-managed` tests; new preview
  scenarios for off, active and in-range, with reachability at both sizes.
- Verify: `npm test`, `npm run build`, `npm run verify:headless` (inspect the
  captures), and `npm run verify:landing-assets`. If that gate reports stale
  captures, run `npm run capture:landing` and commit the result. Do not touch
  or fake the owner's real-session plates; if the gate needs them re-shot,
  report it at checkpoint C as owner item O6.

### Slice 8: documents and contracts

Update, in the same branch:

- `docs/PRODUCT.md` (Adaptive Mastering): what Smart Balance is, that it is off
  by default, and what it does not do.
- `docs/APP_BEHAVIOR.md`: the control in both views, Album, reset and
  Standard-managed behavior.
- `docs/ARCHITECTURE.md`: where the correction sits and the shared computation.
- `docs/TESTING.md`: the measurement tool, the evaluation runner, and the lanes
  a Smart Balance change requires.
- `docs/OPEN_THREADS_AND_DECISIONS.md`: what was completed and what remains.
- `docs/IDEAS_BACKLOG.md`: mark the corrective target-curve item as started.
- `docs/CHANGELOG.md`: an internal entry. It is not a public announcement.
- `CLAUDE.md` **and** `AGENTS.md`, kept byte-identical: add Smart Balance to the
  product contracts as "ships off by default; do not flip the default or raise
  its caps without an owner decision".

State only what was verified. Make no public quality claim.

### Checkpoint C

Stop. Report: all lanes run and their results, the gate table again, the hand
test below ready for the owner, and the deviation log. Ask the owner about merge
and push. Do neither without the answer.

## Hand test for the owner

Run from the branch with `npm run tauri dev`.

1. Load "It’s a coat-original-test.wav" from the `tests for presets` folder.
2. In Standard, find **Smart Balance** with its Beta tag. It should be off, with
   the line "Fixes muddy, dull or harsh mixes before your Style is applied."
3. Play Mastered. Flip the switch on. You should hear no click, the playhead
   should keep moving, and within about a second the line should become either
   a sentence such as "Cleared 1.2 dB of mud, added 0.8 dB of air." or "Already
   balanced. Nothing to fix."
4. Switch Original and Mastered. The playhead should hold.
5. Open Advanced. Next to Adapt strength you should see the same switch and a
   six-column tone strip whose numbers agree with the Standard sentence.
6. Create a master. The receipt should carry a "Smart Balance: on · …" line.
7. Flip the switch off. The line returns to the off text. The executor will
   have already proved that an off export is bit-identical to today's.

## Do not touch

- Preset calibration and preset voicing in `dsp.rs`.
- Every `TBD-CALIBRATION` constant, and the existing deadbands and caps, in
  `guardrails.rs`.
- The gate statics `ADAPTIVE_COMPRESSION`, `CONFIDENCE_GATING`,
  `ALBUM_CHARACTER` and their defaults.
- Golden and snapshot references: `src-tauri/tests/golden/` (including
  `preset_byte_identity` and `preset_fingerprint.json`) and the six-band
  analysis golden. Never set `YES_MASTER_UPDATE_GOLDEN` in this work.
- `compute_spectral_balance_6band` itself. It is byte-exact and golden-pinned,
  and the healthy range is only valid for that exact meter.
- The loudness landing, limiter and ceiling contracts.
- Landing page, release workflows, version numbers, updater configuration.
- Private audio. Nothing under `private-audio-fixtures/`, `tests for presets/`
  or `test-output/` is committed, moved or uploaded.

## Stop and ask

Stop only for these. Otherwise execute without asking.

1. You are not on the office PC, or the test audio is missing.
2. The reference folder has fewer than 40 usable songs when Slice 2 starts.
3. Any golden, snapshot or fingerprint reference would have to change.
4. A gate can only pass by narrowing the range, raising a cap, changing the
   strength mapping, or moving a parameter outside its allowed bounds.
5. A holdout run fails.
6. Standard cannot fit the new row at 1360×740 without altering an existing
   control.
7. The work would need a preset retune, a gate flip, or a public claim.
8. Push, merge to `main`, release, deployment or spending.
9. The checkout contains another agent's uncommitted work, or `main` has moved
   in a way that conflicts with this branch.
10. A project saved with Smart Balance never enabled would fail to open in 0.9.2.

## Deviation log

At every checkpoint, report "where the plan was unclear or wrong and what I did
about it": every judgment call, every plan claim that did not match reality,
every parameter moved and why, and every ripple beyond a slice's stated
footprint. Keep the log in the evaluation note so it travels with the evidence.

## Parking lot

Not in this plan. Do not drift into any of it.

- Reference-track matching ("Smart Match"). It will reuse this machinery with a
  user-supplied target.
- Smart drive and loudness selection (the September research).
- Activating or calibrating the Adaptive Compressor, confidence gating or album
  character.
- Genre or style detection, and any trained model.
- Time-varying correction, finer than six bands, resonance or sibilance control.
- Per-style or per-genre healthy ranges.
- Smart Balance in Album mode, on mobile, or in the browser try-it.
- A DAW plugin. Only the D2 seam and the D20 measurement belong here.
- Flipping the default to on.
- Landing page, marketing, pricing or release work.
- Restructuring the Advanced "Adapt" controls.

## Execution status

### 2026-11-24: Slices 0 and 1 done; stopped before Slice 2

Branch `claude/smart-balance-2e58b2` (a desktop-app worktree branch standing in
for `feat/smart-balance`), cut from local `main` at `5a18eaf`. Not pushed, not
merged.

**Stopped on stop trigger 2.** `private-audio-fixtures/reference-masters/` on
the office PC held 0 files when Slice 2 was due. Slice 2 needs 40 or more
accepted songs (owner item O1).

**Slice 0 baseline at `5a18eaf`** (office Windows PC, Rust 1.100.0, Node 24.15):

| Check | Result |
| --- | --- |
| `cargo fmt --check` | Clean |
| `cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings` | Clean |
| `cargo test --target-dir target\codex-rc` | Every suite passed (513 library tests; 0 failures) |
| `npm test` | 917 tests in 90 files passed |
| `npm run build` | Succeeded |
| `chain_bench`, release, 7 runs each | `It’s a coat-original-test.wav` (44.1 kHz): median **81.47×** realtime, range 81.15–81.77. `Doors Open.wav` (48 kHz): median **88.98×**, range 84.60–89.64 |

The baseline `chain_bench` binary and raw output are kept, uncommitted, in
`test-output/smart-balance/baseline/` on the office PC. G8 can then compare
old and new binaries back to back.

**Slice 1 delivered.** `src-tauri/src/smart_balance_reference.rs`, the runner
`examples/smart_balance_reference.rs`, tests in
`tests/smart_balance_reference.rs`, path tests added to
`tests/evidence_runner_paths.rs`, and usage in `docs/TESTING.md` ("Smart
Balance Reference Measurement"). Smoke runs on real private folders decoded
MP3 and WAV and logged skips. Two runs on the same 30 real files gave
byte-identical statistics. Those folders hold corpus stems and one source
with its LANDR, BandLab and YES masters. They are not a reference library, so
their statistics were discarded.

### Deviation log (Slices 0–1)

1. **Branch and base.** The plan says to branch `feat/smart-balance` from
   `origin/main`. The plan commit `5a18eaf` was not pushed, so the work started
   from local `main` at `5a18eaf`, on the worktree branch named above, as the
   starting session instructed.
2. **Worktree paths.** Private audio exists only in the main checkout and is
   passed by absolute path. Private evidence goes to the main checkout's
   ignored `test-output/smart-balance/`.
3. **Correction: the BandLab masters do exist.** An earlier version of this
   entry said `tests for presets/` held no BandLab masters. That was wrong. The
   search looked for "bandlab" in file names, and these files have no service
   prefix. BandLab's style names (Clarity, Oomph, Tape, Universal) match YES
   Master's preset names, so the files look like YES exports. Per
   `docs/reviews/2026-09-14-landr-waves-dsp-assessment.md` ("Scope and
   evidence"), the owner identified the files as follows:
   - the source: `It’s a coat-original-test.wav`;
   - three LANDR masters: the `LANDR-` files;
   - four BandLab masters: `It’s a coat-clarity-test.wav`, `-oomph-test`,
     `-tape-test` and `-universal-test`.

   The three remaining files are YES Master exports:
   `It_s_a_coat-original-test_YESmaster_mastered_universal intensity75.wav`,
   `YES-compressor-on.wav` and `YES-compressor-off.wav`. Slice 2's note places
   the source, the three LANDR masters and the four BandLab masters against the
   range. It may include the YES exports as optional context, labeled as YES
   output.
4. **Two skip rules added** beyond the plan's four: sample rate under 32 kHz
   (the meter's air band is cut off by Nyquist, which would bias the air
   statistic) and byte-identical duplicates (one song counted twice would bias
   the medians). "Silent" means a peak under −60 dBFS, or the meter finding no
   energy.
5. **Statistics choices the plan left open.** Robust spread is 1.4826 × the
   median absolute deviation (the normal-consistent sigma). The typical shape is
   the per-band median of dB shares. Deviation follows the plan's starting
   hypothesis (minus the per-band median, then minus the song's median of six).
   Shares are floored at 1e-9 (−90 dB) before conversion. Committed values are
   rounded to four decimals. Edge stability uses 1,000 seeded bootstrap
   resamples. An edge is flagged unstable when its bootstrap 90% interval is
   wider than 1.0 dB.
6. **Split.** The holdout is exactly `round(0.25 × n)` songs, chosen by rank of
   a seeded hash of each file's content, not its name (seed `20261124`).
   Membership is recorded as 10-hex-digit prefixes of the file's FNV-1a 64-bit
   content hash.
7. **Where the tool lives.** It is a crate module, so it calls the
   `pub(crate)` meter directly. `analysis.rs` is untouched. It is a try-it
   engine stamp source, so editing it would have forced a wasm rebuild.
8. **Deferred to Slice 3.** Emitting the range table from the committed
   statistics is not built yet. Note for that slice: the repo checks out with
   `core.autocrlf=true`, so the "table and statistics agree" test should parse
   the JSON rather than compare raw bytes.
9. **Docs earlier than Slice 8.** The `docs/TESTING.md` section was written
   with the tool, because it documents a runner that now exists.

## Kickoff prompt

> Read `docs/plans/2026-11-24-smart-balance-execution-plan.md` in full, then
> `CLAUDE.md` and the sections of `docs/TESTING.md` it points to. Execute the
> plan from Slice 0. Its decisions are locked and its stop triggers are the only
> reasons to pause. Stop at each checkpoint and report in plain language with
> the deviation log. Do not push or merge.
