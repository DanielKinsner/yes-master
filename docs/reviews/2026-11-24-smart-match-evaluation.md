# Smart Match evaluation (Checkpoint B)

Smart Match plan, Slice 6
([plan](../plans/2026-11-24-smart-match-execution-plan.md)). Branch
`claude/smart-balance-2e58b2`. The parameters were frozen at `c61dbfb`. Nothing
is pushed or merged. The library behind the typical shape is the provisional
free library from Checkpoint A.

## Bottom line

**Graded at Checkpoint B: G6 failed as written and was accepted with a
documented limitation.** It stays recorded as a failure. The planning session
accepted it as a property of the chain order: any EQ ahead of the compressor
and limiter is partly given back on heavily limited material. The
diagnostics below show that the EQ stage itself does what was asked. The
planning session says the gate itself was its own mistake, because it did not
separate already-limited masters from the mixes the product is built for.
Nothing was retuned, and the holdout was not run again. The limitation is
stated in the product documents and in a second caption line under the
Advanced strip.

**Everything Smart Match itself does passes. One end-to-end gate, G6, fails,
on development and on the holdout alike.** The cause is not Smart Match's EQ.
On loud, already-released masters, the chain's compressor and limiter give
back part of any tone change in the band that drives them, mostly sub, bass
and air. On the mixes YES Master is built for, the gate passes almost
everywhere:

- in development, 80 of 82 corpus bands land in range, including all 18 of
  the owner's song;
- on the holdout, 46 of 47 do.

| | End to end (graded) | EQ stage alone (diagnostic) |
| --- | --- | --- |
| Development: bands outside 35–65% | 62 / 761 | 6 / 761 |
| Holdout: bands outside 35–65% | 27 / 277 | 4 / 277 |

The holdout failure is the plan's stop trigger 4, so I have stopped here. No
parameter was tuned on results. None the plan lets me move (caps lower only,
dead zone, missing-band fade) reaches the compressor and limiter interaction.

## Gates

Development material is the library's 41 build songs plus the five
development corpus pieces. Holdout material is the library's 14 held-out
songs plus the three holdout corpus pieces, run once at the frozen commit.

| Gate | Result | Number |
| --- | --- | --- |
| G1 Off identity | **Pass** | 54 of 54 preset × off-state renders bit-identical, with equal coefficients (2 real songs × 9 presets × no reference / switched off / amount zero). The Slice 4 test adds every preset at 44.1, 48 and 96 kHz on synthetic audio. Golden files untouched |
| G2 Self-match identity | **Pass** | 18 of 18 self-matched renders bit-identical to match off |
| G3 Known-answer recovery | **Pass** (development and holdout) | Development: 761 counted bands in 644 cases over 46 songs. Holdout: 277 bands in 238 cases over 17 songs. In both: 0 wrong direction; the correction equals 0.5 × the measured gap exactly (worst 0.000 dB); unchanged bands stayed ≤ 0.5 dB in 100% of single- and two-band cases. Not graded: 73 (development) and 31 (holdout) boosts that the missing-band rule reduced |
| G4 Realization accuracy | **Pass** | 288 cases (12 single-band caps and 20 mixed, × 3 rates × 3 filter placements); worst error 0.018 dB |
| G5 Level invariance | **Pass** | 60 comparisons at ±12 dB; worst change 0.0000001 dB |
| G6 Gap closed end to end | **Failed as written; accepted with a documented limitation** (development and holdout) | Development: 62 of 761 bands outside 35–65% (sub 21, bass 10, low mids 2, mids 1, presence 1, air 27); range 0.33–0.83, median 0.55. Holdout: 27 of 277 outside (sub 6, bass 4, low mids 2, mids 1, presence 1, air 13); range 0.40–0.87, median 0.55 |
| G7 Amount and caps | **Pass** | 669 pairs × 5 amounts: 0 shrinking corrections, 0 over a cap, 0 partial-flag mismatches. 344 pairs reach a cap at 100% |
| G8 Performance | **Pass** | `chain_bench` against the saved Slice 0 binary, 7 alternating runs: no reference −0.12% (coat) and +0.16% (Doors) against baseline; active match 96.0% and 100.9% of no-reference. Context, not gated: with both Advanced shelves on, the corrective stage of its own runs at 92.6% and 97.8% |
| G9 Safety | **Pass** | Silent, 10-second and 22.05 kHz files refused with the right reasons; a non-audio file "could not be read"; mono accepted, as the corrected gate text says (confirmed at Checkpoint B); 5 songs with the sub band removed got no sub boost and rendered finite; hostile values inert |
| G10 Persistence | **Pass** | Reopened with the same match and the same decision after the reference file was deleted; a saved preset carries no reference, on disk or when listed |

### Where G6 fails, and why

G6's graded number is how much of the gap is left after matching at 50%:
(original gap − what the match changed end to end) / original gap. It is
measured by the meter on the copy rendered with and without the match, through
Universal. Two further numbers break it down:

| G6 measure | Development | Holdout |
| --- | --- | --- |
| End to end, graded: median (range), outside | 0.553 (0.332–0.825), 62 | 0.550 (0.400–0.871), 27 |
| EQ stage alone, diagnostic | 0.509 (0.318–0.690), 6 | 0.518 (0.384–0.710), 4 |
| Against the original rendered without a match, context | 0.466 (0.152–0.686), 36 | 0.466 (0.216–0.754), 14 |

| Material | Development: outside | Holdout: outside |
| --- | --- | --- |
| Free-library songs (released masters) | 60 / 679 | 26 / 230 |
| Corpus pieces (mixes, stem sums, the owner's source) | 2 / 82 | 1 / 47 |

A probe on the worst development cases:

| Case | Intended | EQ alone, on the song | After the full chain |
| --- | --- | --- | --- |
| C. Scott – Determinate, sub +4 dB | −2.00 dB | −1.93 dB | −0.70 dB |
| Fonkdongo – TamosReady, air +4 dB | −2.00 dB | −1.60 dB | −1.02 dB |
| C. Scott – Determinate, bass +4 dB | −2.00 dB | −1.95 dB | −1.11 dB |

Smart Match's EQ does what it was asked on the real song. The full chain then
gives much of it back. Universal adds its gain push before its compressor and
limiter, so a loud master is driven into them. When Smart Match cuts the band
that dominates the limiting, the gain reduction relaxes and gives part of the
cut back. That happens to any EQ placed before the dynamics, including the
Style's own and the user's manual EQ. Changing it means touching the
dynamics, limiter or ceiling contracts, which the plan does not allow.

The six development EQ-stage misses are two kinds:

- three are air boosts on orchestral or classical songs, at 0.65–0.69. Their
  air energy sits low in the band, where the shelves have not reached full
  depth;
- three are tilt cases in the bass band: two at 0.66, and one (Komiku) at
  0.32, which closed slightly too much.

Both kinds are the solver's pink-noise weighting showing on spectra far from
pink. The
holdout's four EQ-stage misses were not examined one by one: the holdout runs
once and is not used for diagnosis.

## Ungated reports

### Cross-genre matches (development)

Each development corpus piece was matched at 50% to five build songs of other
genres, one per genre: 25 pairs. **20 of the 25 show the partial notice**, and
42 bands are capped. The moves are large because the references are very
different. For example, the solo piano piece (In This Moment) matched to a
hip-hop track has a 19 dB bass gap and a 21 dB low-mid gap. It gets the full
4 dB cut in both and the full 3 dB boost in presence and air. It would become
a much thinner piece, which is what it was asked to do.

What looks wrong, or worth a decision:

- **A mismatched reference produces drastic, capped moves.** The owner's song
  matched to a very dark ambient piece (Chris Zabriskie) loses 4 dB of air.
  Metalmania matched to it would lose 4 dB of air and gain 3 dB in low mids
  and mids. The caps hold, and the partial notice is the only warning. That
  is M10 as written.
- Nothing else stood out: no panics, no boost into an empty band, no sign
  errors.

### The owner's song against the three LANDR masters

Moves are in dB, offset removed, for sub, bass, low mids, mids, presence and
air. Each row's three lines are what LANDR did, what Smart Match decides at
50%, and what that does to a Universal render.

| Reference | Sub | Bass | Low mids | Mids | Presence | Air |
| --- | --- | --- | --- | --- | --- | --- |
| Balanced (`82c3f44bd2`): LANDR | +0.7 | −0.3 | −0.4 | +0.5 | +0.3 | −0.8 |
| Smart Match decides | +0.4 | 0 | 0 | 0 | 0 | −0.4 |
| Rendered | +0.3 | 0 | 0 | 0 | 0 | −0.4 |
| Open (`8a1a18e349`): LANDR | +0.3 | −1.0 | −1.0 | +0.5 | +1.1 | −0.3 |
| Smart Match decides | 0 | −0.5 | −0.5 | 0 | +0.5 | 0 |
| Rendered | 0 | −0.4 | −0.5 | 0 | +0.6 | 0 |
| Warm (`63fb321974`): LANDR | +1.8 | +0.4 | −0.2 | +0.2 | −0.4 | −2.0 |
| Smart Match decides | +0.9 | 0 | 0 | 0 | 0 | −1.0 |
| Rendered | +0.7 | 0 | 0 | −0.1 | 0 | −1.0 |

**In 7 of the 8 bands where LANDR moved 0.5 dB or more, Smart Match's
correction points the same way.** The exception is Open's +0.5 dB in mids: half
of that is 0.25 dB, under the 0.3 dB dead zone. The owner's song is not
limited heavily, so the rendered moves track the decision closely.

### 30-second listen against the whole song (M22)

For each corpus piece, I compared the correction from the loudest contiguous
30 seconds with the correction from the whole song, toward three fixed
references (18 band corrections each).

| Piece | Largest difference | Share within 0.5 dB |
| --- | --- | --- |
| Owner's song (`c24e08dd52`) | 1.47 dB | 44% |
| In This Moment | 1.21 dB | 72% |
| Funkorama | 0.43 dB | 100% |
| Metalmania | 1.13 dB | 78% |
| Baby Bird | 0.51 dB | 94% |
| Aphelion (holdout) | 2.42 dB | 33% |
| Rich (holdout) | 2.43 dB | 22% |
| Imagining Imaginal (holdout) | 1.67 dB | 72% |

A 30-second listen points the same way, but it can differ by up to about
2.4 dB from the whole-song correction on songs whose sections differ. That
matters for the plugin door (M3).

## Parameters

| Parameter | Value | Moved? |
| --- | --- | --- |
| Cut cap / boost cap | 4.0 / 3.0 dB | No |
| Default match amount | 0.5 | No |
| Dead zone | **0.3 dB** | Yes, from 0.25, within 0.1–0.5. Done in Slice 5, before any evaluation run, so every applied move is one the locked text names (deviation log entry 23) |
| Missing-band fade, start / end | 8 / 14 dB | No |
| Reference minimum length / sample rate | 30 s / 32 kHz | No |

No parameter was tuned on evaluation results.

## What was measured, and how

**Material.**

- *Development* (tuning allowed):
  - the 41 build-split songs of the provisional free library;
  - the five development corpus pieces: the owner's "It’s a coat" source
    (shown by its anonymous ID), In This Moment, Funkorama, Metalmania and
    Baby Bird.
- *Holdout* (run once, after freezing):
  - the library's 14 held-out songs;
  - the three holdout corpus pieces: Aphelion, Rich and Imagining Imaginal.

**Same meter, same decision.** Every profile comes from the app's own track
analysis or `smart_match::measure_shares`. Every correction comes from
`smart_match::decide_with` or from the chain itself.

**Known-answer copies (G3, G6).** Each song gets 14 copies, each with a known
tone change:

- each of the six bands ±4 dB;
- low mids +4 dB with presence −4 dB;
- a tilt of +1 dB per octave around 1 kHz.

The changes are made with an independent FFT equalizer, unrelated to the
chain's biquads, so the chain is never graded against itself. It uses
65,536-sample sqrt-Hann overlap-add with brick-wall gains on the meter's own
band edges. Each copy is then matched back to its original at the default 50%.

- A band **counts** when the copy changed it, its measured gap is at least
  1 dB, and, for a boost, the missing-band rule did not engage. A boost that
  rule reduced is reported, not graded: M16 deliberately refuses to boost
  content a track lacks.
- **G3** checks every counted band three ways:
  - the correction opposes the change;
  - it equals 0.5 × the measured gap within 0.5 dB, where no cap bound;
  - in single- and two-band cases, every unchanged band moves 0.5 dB or less,
    in at least 90% of cases.
- **G6** renders each copy through the full chain twice: Universal, default
  Intensity, default Adapt, with and without the match. The *re-measured gap*
  is the original gap minus the change the match made end to end, measured by
  the meter on the two renders. It must be 35–65% of the original gap. This
  isolates Smart Match from the Style and the guardrails, which treat both
  renders alike. A context number compares the matched copy with the original
  rendered without a match, so it includes the guardrails reacting to the
  copy's own tone.
- A copy's profile is its original's analysis profile with the copy's own
  measured six shares. A tone filter does not move dynamics or width, so the
  copies are not re-analysed.

**The other gates.**

- **G1 and G2:** 30-second excerpts of the owner's song and one library song,
  all nine presets.
  - G1 tests three off states: no reference, switched off, amount zero.
  - G2 tests a self-match, made from the song's own measured shares.
  - Both compare coefficients and every rendered sample's bits.
- **G4:** the chain's real coefficients, measured through
  `band_power_change_db` on a 240-point-per-band grid. It covers every
  single-band cap case and 20 seeded mixed cases, at 44.1, 48 and 96 kHz, in
  all three places the corrective filter can run.
- **G5:** each development corpus piece scaled ±12 dB before metering,
  against three fixed references. The references are also scaled ±12 dB.
- **G7:** every known-answer pair and every cross-genre pair, at amounts 0.1,
  0.25, 0.5, 0.75 and 1.0. The partial flag is checked against an
  independent recomputation of cap activity.
- **G8:** `chain_bench`, from Slice 4. The runner does not time anything.
- **G9:**
  - synthetic WAVs: silent; 10 seconds; 22.05 kHz; mono;
  - a file that is not audio;
  - the five development pieces with their sub band removed, matched to
    their originals and also rendered;
  - hostile values: NaN shares, empty shares, NaN and infinite amounts.
- **G10:**
  1. a real song copied to a scratch file and measured as a reference;
  2. a project saved with that reference;
  3. the file deleted and the project reopened;
  4. then a preset saved and listed.

**Fixed references** for G5 and the 30-second report are the first build song,
by label, in each of three genres. The **corpus genres** in the cross-genre
report are my labels, because the corpus list has no genre field:

- coat: unlabelled;
- piano and Aphelion: cinematic/orchestral/classical;
- Funkorama: jazz/funk/soul;
- Metalmania: metal;
- Baby Bird: rock;
- Rich: hip-hop/beats;
- Imaginal: other.

## What this evidence does not establish

- **How it sounds.** Nobody listened. The gates prove the decision and the EQ
  do what the contract says, measured by the app's own meter. They do not
  prove a matched master sounds better. Per M5, no listening sitting is
  proposed; Checkpoint C has the owner's hand test.
- **Real songs are not pink inside a band.** The EQ is solved for pink noise
  within each band. G6 measures the result on real songs end to end, and G4
  measures it on the coefficients; that difference is the gap between them.
- **The library is provisional and leans independent** (Slice 2 limits). Its
  role here is small: the missing-band rule's typical shape, and test
  material.
- **Known-answer copies are filtered copies.** A copy differs from its
  original only in tone, by construction. Two real songs also differ in
  arrangement, dynamics and width. The cross-genre report is the closer look
  at that, and it is not gated.
- **Speed.** `chain_bench` measures the per-sample path on this office PC. The
  cost of the solver itself (about 10 µs per settings change) is measured
  separately; nothing here measures a laptop.
- **The interface.** Nothing in Slices 3–6 is visible yet. The copy, the
  Standard row, Advanced, undo and the hand test are Slice 7 and Checkpoint C.
- **Swift.** The iPhone app's Swift model change was not compiled; that needs
  a Mac. The Rust bridge lanes for both phones did run.

## Deviation log

The full log, entries 1–43 across Slices 3–6, is in the plan's "Execution
status" section. These are the entries a grader most needs:

- **8–10.** The seven existing bands cannot separate sub from bass, so I
  added one corrective peaking filter at 140 Hz. It runs in an idle Advanced
  shelf slot when one is free, so it costs nothing on the usual path. The
  solver is damped Gauss-Newton.
- **13.** The setting is omitted when absent. Projects open in 0.9.2, which I
  checked at its tag.
- **16 and 29.** Wire samples were updated by script and verified by the
  unchanged test, without setting `YES_MASTER_UPDATE_GOLDEN`.
- **20 (please confirm).** Mono references are accepted, under M10, although
  G9 lists mono.
- **21 (please decide).** Amount zero has no line in the copy table.
- **22.** "Listening to your track…" can persist for a track that has no
  six-band profile.
- **23.** The dead zone moved from 0.25 to 0.3 dB.
- **24–28.** The receipt and readout are omitted when unused. The receipt
  phrase rule. Rule 12's split between the command and the hook. The preview
  mock's new case is deferred to Slice 7.
- **31.** A commit I did not make: the planning session's `vercel.json`
  change.
- **32–43.** How G3 and G6 are measured, the two added diagnostics, and the
  decision not to tune.

## Reproduce

From `src-tauri`, with `$M` as the main checkout:

```powershell
cargo run --release --target-dir target\codex-rc --example smart_match_evaluation -- --phase development --library "$M/private-audio-fixtures/reference-masters/free-cc-2026-11-24" --library-sources ../docs/reviews/evidence/2026-11-24-smart-balance-reference-range/library-sources.json --corpus-json ../docs/reviews/evidence/2026-09-15-mastering-quality/corpus.json --corpus-root "$M/test-output/mastering-quality-20260915" --owner-dir "$M/tests for presets" --output "$M/test-output/smart-balance/evaluation" --evidence ../docs/reviews/evidence/2026-11-24-smart-match-evaluation
```

The holdout phase takes the same arguments with `--phase holdout --frozen-commit
c61dbfbb84caff29b6ac6a036c9f405917d76e62`. The holdout has now been spent:
another run on it would not be a holdout.

Committed evidence (numbers only; free-library and corpus songs named, the
owner's files as anonymous IDs):
`docs/reviews/evidence/2026-11-24-smart-match-evaluation/smart-match-evaluation-development.json`
and `…-holdout.json`. The private ledgers, with paths, are in the main
checkout's `test-output/smart-balance/evaluation/`.
