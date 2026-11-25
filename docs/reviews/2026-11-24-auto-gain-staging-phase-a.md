# Auto gain staging, Phase A: three designs measured offline

Date: 2026-11-24. Plan:
[auto gain staging execution plan](../plans/2026-11-24-auto-gain-staging-execution-plan.md),
Slices 0–1, stopped at Checkpoint A for the planning session's grade.
Branch `claude/auto-gain-staging-bf8b35`. Nothing is pushed or merged, and no
product code changed.

## Bottom line

**The rule, applied exactly as written, selects S (turn-down first).**

- G (two gains) fails **R1** on 2 of 594 comparisons, by −0.125 and −0.21 dB
  against a −0.1 dB limit, and **R4** on 3 of 126 cases. A failed R1 sends the
  rule to S, and S passes R1, R3 and R7.
- G passes the other five conditions, most by a wide margin. It hits the
  target in 674 of 732 cases against today's 594. It gives back a median
  2.6 dB of peak-to-loudness ratio where today turns a song down by 1 dB or
  more. Its Styles are further apart. On the reference song it is
  indistinguishable in tone from today.
- S fixes only the "flattened, then turned down" case. It lands on target in
  21 fewer cases than today does. On the reference song, its tone moves up to
  0.9 dB from today's, because at low targets it turns the song down before
  the Style, so the Style's own compressor and saturation barely act.

The planning session decides what to build. The evidence for that call is
below, including where each design fails.

## What was run

- **Runner:** `src-tauri/examples/auto_gain_staging_research/`, tested in
  `src-tauri/tests/auto_gain_staging_research.rs`, documented in
  `docs/TESTING.md` ("Auto Gain Staging Research").
- **Graded run:** development run 2 at `bbe56f0`. Development run 1 differs
  only in how G resolved its drive (see "Two runs" below).
- **Material: 61 development songs, 732 cases** (song × the four Standard
  Styles at default Intensity × −14, −11 and −9 LUFS). Each case was rendered
  as T, S and G through Standard's export recipe: 44.1 kHz, 24-bit, −1 dBTP,
  and the export's own qualified finalizer.
  - 41 free-library songs from the build split. The 14 held-out songs were
    identified by content hash and never decoded.
  - 7 September corpus pieces. All are development, as the plan says.
  - 13 owner files: the reference song, nine other masters of it (four
    BandLab, three LANDR, two YES exports), and three other owner songs.
    `YES-compressor-on.wav` is byte-identical to the Universal-75 YES export
    and is counted once.
  - The owner's Suno and unmastered-mix folders were empty.
- **Reference level (A6):** the app's analysis of the reference song,
  **−12.3313 LUFS and −1.8827 dBTP**. Level in for that song is exactly 0.0 dB.
- **Parameters, unchanged from the plan:** clean limit 6.0 dB, tolerance
  0.2 LU, at most 4 passes, level-in range ±24 dB. Nothing was tuned.
- **Quiet, dynamic pieces (source below −18 LUFS): 5, one short of the six
  the plan asks for.** They are Rich, Imagining Imaginal and Baby Bird (stem
  sums), Brahms Symphony No. 2 III (Musopen) and A Winters Fairytale (Small
  Colin). The planning session said it would gather more before the holdout.
- **The render is trustworthy.** No product code could change in Phase A, so
  the runner renders the Style stages once with a pass-through limiter and then
  feeds a real limiter that signal, times the drive. With no drive this split
  is bit-identical to the production chain. That was checked on every song and
  Style (first 20 s), on three whole songs, and in the tests. The Style
  stages' coefficients do not depend on the target (checked on every song).
  The gated systems were off.

## The decision rule

| # | Condition | G | S |
| --- | --- | --- | --- |
| R1 | Where T and the design both land within 0.2 LU, PLR never more than 0.1 dB below T's | **Fail.** 594 cases; 2 below −0.1 dB (worst −0.210 dB); median +2.54 dB | **Pass.** 573 cases; worst +0.000 dB; median +3.41 dB |
| R2 | Within tolerance in at least as many cases as T | **Pass.** G 674, T 594 | **Fail.** S 573, T 594 |
| R3 | Where T turns down ≥ 1 dB, median PLR gain ≥ 1.0 dB | **Pass.** 555 cases; median +2.61 dB (p10 +1.13) | **Pass.** median +3.57 dB (p10 +1.21) |
| R4 | Where T misses by > 0.5 LU, the design reaches the target or stops at the clean limit, every case | **Fail.** 126 cases: 70 reached, 53 stopped at the clean limit, 3 neither | **Fail.** 126 cases: S never raises the level, so it reaches none |
| R5 | Sources louder than −14 LUFS: median Style distinctness ≥ T's | **Pass.** 3.93 against 3.58 | **Pass.** 4.03 against 3.58 |
| R6 | Reference song: level in exactly 0.0 dB, tone bands within 0.5 dB of T's at each loudness | **Pass.** Level in 0.0; largest difference 0.364 dB (all Styles, all loudnesses) | **Fail.** Largest difference 0.897 dB |
| R7 | Every render finite and under the ceiling by the qualified check | **Pass.** 732 of 732 | **Pass.** 732 of 732 |

**Selection.** G fails R1, so the rule turns to S. S satisfies R1, R3 and R7,
so the rule selects **S**. S's failures on R2, R4 and R6 are not part of its
qualification. Per the plan, S then covers only the overshoot case, and the
quiet-song case stays open.

### Why G fails where it fails

- **R1 (2 cases).**
  - Owner song 15b03e81c8, Oomph at −9: PLR 8.009 against 8.134.
  - Small Colin – A Winters Fairytale, Universal at −14: PLR 12.467 against
    12.677.

  Both are quieter songs that level in raised (+2.2 and +2.6 dB), so the
  Style's saturation and compressor worked harder. In both, G's limiter did
  less than today's (0.94 against 2.28 dB, and idle against 1.52 dB). The loss
  comes from level in, not from the limiter drive. This is the September
  finding in a milder form, where the peak term bounds level in. The next
  closest comparisons are −0.074 and −0.041 dB.
- **R4 (3 cases).** In each, four passes were not enough to reach the target:
  - Funkorama, Tape and Oomph at −11;
  - C. Scott – Determinate, Universal at −9 (reached −9.37).

  The export's final peak check (below) makes the delivered loudness rise
  unevenly with drive, so the conservative step converges slowly. The plan
  allows 2 to 6 passes. Raising the limit was not tried, because the task
  forbids tuning to make a design qualify.

### Two runs, and why

Run 1 (`ecb9c09`, built from the working tree just before that commit, which
differs by formatting only) resolved G's drive on the chain's output before
delivery. Contract rule 3 says the drive is chosen so the **delivered**
loudness equals the target. Run 1 showed the gap: the export's final
true-peak check turned many of G's masters down by 0.2–2.8 dB after the
limiter, and the resolver never saw it. That one cause accounted for 30 of
G's 34 R4 failures and 59 of its 66 R2 losses.

Run 2 (`bbe56f0`) puts the 44.1 kHz conversion and the export finalizer
inside every resolution pass. It changed no rule, parameter or product code.

| | Run 1 (chain output) | Run 2 (delivered, graded) |
| --- | --- | --- |
| G R1 | Fail (3 cases; worst −0.210 dB) | Fail (2 cases; worst −0.210 dB) |
| G R2 | Fail (583 against 594) | Pass (674 against 594) |
| G R3 | Pass (+2.62 dB) | Pass (+2.61 dB) |
| G R4 | Fail (34 neither) | Fail (3 neither) |
| G R5, R6, R7 | Pass | Pass |
| S, all conditions | As in run 2 | R1, R3, R5, R7 pass; R2, R4, R6 fail |
| Selected | S | S |

T and S are identical, row for row, in both runs (1,464 rows). Both
committed summaries are in
`docs/reviews/evidence/2026-11-24-auto-gain-staging-phase-a/`.

## What each design does, in numbers

Medians over all 732 cases:

| | Section contrast change | Paired attack change (median / p10) | Limiter load | LRA | Within ±0.2 LU | 90th-percentile miss |
| --- | --- | --- | --- | --- | --- | --- |
| T (today) | −1.42 dB | −2.54 / −3.64 dB | 2.68 dB | 3.2 LU | 594 | 2.29 LU |
| S | −0.34 dB | −0.61 / −1.71 dB | 1.44 dB | 3.9 LU | 573 | 2.29 LU |
| G | −0.67 dB | −1.46 / −2.57 dB | 0.08 dB | 3.7 LU | 674 | 0.19 LU |

G's limiter is idle in 355 of 732 renders. When it works, it never exceeds
the clean limit (largest load 6.000 dB). No design delivered a master more
than 0.2 LU above its target.

## What G costs (this PC: i9-12900K, single thread)

Times are medians of three runs, Universal, for the chain, the 44.1 kHz
conversion and the final check (decode and analysis excluded).

| Song | Loudness | Today (production) | T, split render | G | G's limiter passes |
| --- | --- | --- | --- | --- | --- |
| Zane Little – Sideways City (2:13) | −14 / −11 / −9 | 2.97 / 2.87 / 2.95 s | 3.18 / 3.32 / 3.23 s | 3.34 / 3.24 / 4.71 s | 1 / 1 / 2 |
| Reference song (4:05) | −14 / −11 / −9 | 5.48 / 5.17 / 5.03 s | 6.06 / 5.76 / 5.88 s | 6.39 / 6.07 / 6.11 s | 1 / 1 / 1 |
| Scott Buckley – Unraveling (7:24) | −14 / −11 / −9 | 8.60 / 8.96 / 8.70 s | 9.69 / 10.31 / 9.83 s | 10.87 / 11.54 / 21.17 s | 1 / 1 / 3 |

- **With one pass**, G costs about what today's export costs: at most 1.2 s
  more than the same split render. That covers the extra loudness and peak scan of the
  stage pass.
- **Each extra pass** costs one limiter run, one conversion and one final
  check: about 1.5 s on a 2-minute song and 5–6 s on a 7-minute one.
- **Across the development set**, G needed 1 pass in 563 renders, 2 in 100,
  3 in 48 and 4 in 21: 0.35 extra passes per render on average.
- The research split render is about 10% slower than production, because it
  copies the signal between the two halves. A product build would not pay
  that.

## Four instructive songs

Universal at −14 / −11 / −9 LUFS. PLR is true peak minus loudness: higher
means more punch kept.

- **The owner's reference song** (−12.3 LUFS, level in 0.0 dB).
  - Today: PLR 6.81 dB at every target. The limiter takes 2.4 dB, then the
    landing turns the master down 6.3 / 3.3 / 1.3 dB.
  - G: PLR 8.69 / 8.69 / 8.07 dB. The limiter is idle at −14 and −11 and takes
    0.75 dB at −9. Section contrast is −0.6 dB against today's −1.3.
  - S: PLR 11.98 / 9.94 / 7.94 dB. At −14 it turns the song down 7.2 dB before
    the Style, so the Style's dynamics hardly act.
- **A loud released master: Contra – Secta** (−5.5 LUFS, +1.0 dBTP, level in
  −6.9 dB).
  - Today: PLR 4.90 dB everywhere. The limiter takes 4.4 dB, and the paired
    attacks lose 3.3 dB.
  - G: PLR 8.74 / 8.74 / 8.16 dB, attacks +1.4 dB.
  - S: PLR 11.35 / 10.02 / 8.01 dB.
- **A quiet, dynamic stem mix: Rich** (−27.4 LUFS, −6.0 dBTP).
  - Today misses every target. It delivers −20.3 LUFS for −14, −11 and −9.
  - S cannot help, because it never raises the level.
  - G reaches −14.07 at −14 (limiter 4.6 dB, section contrast −2.9 dB). At −11
    and −9 it stops at the clean limit at −13.0 LUFS, which is what the locked
    line "Reached −13.0 LUFS of −11.0" would report.
- **A YES Master export of the reference song** (−9.0 LUFS, −2.6 dBTP: the
  "flattened, then turned down" fingerprint).
  - Today flattens it again: PLR 5.31 dB, section contrast −1.8 dB.
  - G: PLR 8.83 / 8.83 / 8.14 dB.
  - S: PLR 11.56 / 9.86 / 8.02 dB.

## Surprises

1. **The export's final peak check, not the limiter, often decides the
   delivered loudness.**
   - On Funkorama the limiter holds samples at −1.1 dBFS, but the qualified
     true-peak meter reads up to +1.8 dBTP. The export then turns the master
     down 2.4–2.8 dB, so today misses −11 and −9 by 2.2 LU.
   - Smaller pull-downs (0.2–1 dB) are common at −9 on dense songs.
   - Today hides this whenever it overshoots and turns down anyway. Any design
     that aims at the target (G, and S near the target) meets it head-on.
   - This is today's behavior, and its code is on the do-not-touch list.
2. **S re-voices the Styles at low targets.** S moves a loud song's whole
   turn-down in front of the Style, often 7–13 dB at −14 LUFS, so the Styles'
   compressor and saturation see a much quieter signal than they were voiced
   for. That is how S keeps the most PLR. It is also why S fails R6, and why
   S's Styles differ from today's at Low loudness. G keeps every Style
   meeting the song at the reference level.
3. **S lands on target less often than today** (573 against 594). After S's
   turn-down, the final peak check can still pull the master under the target
   (median 0.34 LU, worst 0.74 LU in those 21 cases), and S may never raise
   the level.
4. **Ten of the 61 songs are the reference song** (the original and nine
   masters of it). They do not sway the medians. Without them:
   - R3's median gain is +2.58 dB for G and +3.41 dB for S;
   - R5's medians are T 3.65, S 4.14, G 4.00.

## What this does not establish

- **Sound.** These are measurements; whether G or S sounds better is the
  owner's call at Checkpoint C.
- **The product build.** Phase B must reproduce these numbers through the
  production export path within 0.1 dB (Slice 4).
- **The holdout.** No holdout run was made; the plan runs it once in Slice 4.
- **The quiet case.** Only 5 quiet, dynamic pieces were available.
- **Timings** are this PC only, single-threaded, and exclude decoding and
  analysis. Live audition timing is Slice 5's.
- **Distinctness** uses the preset-fingerprint distance adapted to real songs
  (no THD term). See the deviation log.

## Grade (planning session, 2026-11-24)

In order:

1. **The rule as written selected S.**
2. **The planning session overrode it and chose G.** The fallback clause let
   S qualify without checking that it keeps the reference song's tone (R6
   0.897 dB) or lands on target as often as today (R2 573 against 594). S also
   cannot help quiet songs. G's two R1 misses (0.125 and 0.21 dB) come from
   level in letting the Style act at its designed level, with the limiter
   doing less than today's. This is an override, not a pass.
3. **R1a was tried and dropped.** It said G's limiter load is never more than
   0.1 dB above T's. It failed in 26 of 594 cases (worst +1.37 dB), all on
   songs that level in turned down, while those same cases kept more dynamics
   than today. It judged which stage works, not the result.
4. **R1b and R1c replace R1, frozen before any holdout run.** Both hold on the
   development data:
   - **R1b:** PLR never more than 0.5 dB below T's. Worst −0.21 dB.
   - **R1c, section contrast:** 2 of 594 cases (0.34%) worse than T by more
     than 0.5 dB, none by more than 1.0 dB. Worst −0.737 dB.
   - **R1c, paired-attack median:** 5 of 594 cases (0.84%) worse by more than
     0.5 dB, none by more than 1.0 dB. Worst −0.602 dB.

   R2–R7 are unchanged, and R4 must hold before the freeze. S is dropped.
