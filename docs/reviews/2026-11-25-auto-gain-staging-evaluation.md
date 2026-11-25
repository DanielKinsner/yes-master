# Auto gain staging: engine evaluation (Checkpoint B)

Date: 2026-11-25. Plan:
[auto gain staging execution plan](../plans/2026-11-24-auto-gain-staging-execution-plan.md),
Slices 2–4. Branch `claude/auto-gain-staging-bf8b35`. Nothing pushed or merged.
The interface (Slice 6) and live audition (Slice 5) are not built.
[Phase A](2026-11-24-auto-gain-staging-phase-a.md) has the design study and
the Checkpoint A grade.

**Since then:** the planning session graded Checkpoint B, and its three items
and Slices 5–7 are in the [Phase C note](2026-11-25-auto-gain-staging-phase-c.md).
With the planning session's faster step, E4's Reach-target half failed one
case by 0.09 dB under the amended acceptance; the planning session accepted
it at Checkpoint C. This note keeps the Checkpoint B state as it was.

## Bottom line

**G is built into the engine and export, off by default. Seven of the eight
gates pass; E4 fails as written in one of its two halves.**

- **What passes:**
  - The holdout (E3) passes, run once at frozen commit `998b6bc`.
  - The decision rule as frozen at Checkpoint A (R1b, R1c, R2–R7) holds on
    the development material through the real export (E2).
  - Switched off or inert, the chain renders exactly as today (E1), and
    exports are as fast as before (E7).
- **What fails:** E4's Reach-target half. Clean never takes more than the
  6 dB clean limit. But Reach target, which goes past the clean limit,
  reached the target in only 109 of 136 development cases and 48 of 50
  holdout cases. The 7 cases on one song (Funkorama) are blocked by today's
  final peak check, which no amount of limiting gets past. The rest need
  7–10 resolution passes; the frozen limit is 6.

## The gates

| Gate | Result | Evidence |
| --- | --- | --- |
| **E1** Off identity | **Pass** | Every factory preset with the switch off, unresolved, zero, without a target or in Album: equal coefficients and bit-identical renders (`tests/auto_gain_staging.rs`). Golden byte-identity and fingerprint references pass untouched, and no golden file changed. The production export with the switch absent matches the Phase A research render exactly on every song: 0.0 difference on all 28 recorded T numbers over 61 songs, and equal 24-bit PCM on all 94 development and holdout songs. |
| **E2** Decision rule, development, production path | **Pass** | 75 songs, 900 cases. See the table below. |
| **E3** Holdout (R1b, R1c, R2, R4, R7), run once | **Pass** | 19 held-out songs, 228 cases, at `998b6bc`. See the table below. |
| **E4** Clean limit / Reach target | **Clean: pass. Reach target: fail as written** | Clean: the largest limiter load is 6.0 dB in development and in the holdout. Reach target, run wherever Clean stopped: 109 of 136 development cases and 48 of 50 holdout cases within 0.2 LU. Of the 27 development misses, a 12-pass diagnostic reaches 20 (7–10 passes); the 7 Funkorama cases stay short even with up to 36 dB of limiting, because the final peak check pulls them down (see "Findings"). |
| **E5** Never above target | **Pass** | 0 of 1,036 development renders and 0 of 278 holdout renders (Clean and Reach target) land more than 0.2 LU above the target. |
| **E6** Smart Match unchanged | **Pass** | With a reference active, Smart Match's decision and corrective filter are identical with the switch on and off: in the test and on every one of the 94 songs. |
| **E7** Performance | **Pass** | `chain_bench`, 7 interleaved runs each, against the saved `a8231b5` binary. Off: −0.40% (reference song) and −0.56% (an owner fixture at 48 kHz), against a 2% limit. On: 97.1% and 102.6% of off, against ≥ 95%. Export times are in the table below. |
| **E8** Safety | **Pass** | Silent, 50 ms, already-clipped, mono and extremely quiet sources, under both priorities: no panic, finite audio, and either inert or with both gains within ±24 dB (`hostile_sources_are_inert_or_bounded`). |

### The decision rule through the production path

| # | Development (75 songs, 900 cases) | Holdout (19 songs, 228 cases) |
| --- | --- | --- |
| R1b | Pass. 605 cases, worst −0.210 dB | Pass. 127 cases, worst −0.082 dB |
| R1c, section contrast | Pass. 2 worse by more than 0.5 dB (0.33%), 0 by more than 1.0 dB; worst −0.737 dB | Pass. 0 and 0; worst −0.305 dB |
| R1c, paired-attack median | Pass. 5 (0.83%) and 0; worst −0.602 dB | Pass. 0 and 0; worst −0.299 dB |
| R2 | Pass. G 764 within ±0.2 LU, T 605 | Pass. G 178, T 127 |
| R3 | Pass. 557 cases, median +2.607 dB (p10 +1.124) | Not part of E3 (112 cases, median +2.781 dB) |
| R4 | Pass. 281 cases: 145 reached, 136 stopped at the clean limit, 0 neither | Pass. 100 cases: 50 reached, 50 stopped, 0 neither |
| R5 | Pass. 3.930 against 3.584 | Not part of E3 (4.333 against 4.259) |
| R6 | Pass. Level in exactly 0.0 dB; largest tone-band difference 0.364 dB | Not applicable: the reference song is development material |
| R7 | Pass. 900 of 900 | Pass. 228 of 228 |

### Agreement with Phase A

The plan asks that the production path agree with Phase A within 0.1 dB on
every recorded number. It does, over all 1,464 T and G rows of the 61 Phase A
songs. The Phase A baseline is development run 2, with the six-pass re-run of
two songs merged in.

- **Audio measures:** the largest difference is 0.000005 dB, across loudness,
  true peak, PLR, LRA, section contrast, attacks, the nine tone bands,
  mid/side and the fingerprint terms.
- **Level in:** identical.
- **Drive and limiter load:** within 0.05 dB, because Slice 4 reads them from
  the receipt line, which has one decimal.
- **The stopped flag:** agrees everywhere.

### Export time on this PC (single thread, decoding included)

| Song | Loudness | Switch off | Switch on |
| --- | --- | --- | --- |
| Zane Little – Sideways City (2:13) | −14 / −11 / −9 | 3.39 / 3.41 / 3.38 s | 3.59 / 3.85 / 5.52 s |
| Reference song (4:05) | −14 / −11 / −9 | 5.61 / 5.53 / 5.45 s | 6.53 / 6.26 / 6.21 s |
| Scott Buckley – Unraveling (7:24) | −14 / −11 / −9 | 9.64 / 9.48 / 9.86 s | 11.15 / 11.34 / 23.21 s |

With one resolution pass, an export takes 0.2–1.9 s more: the source
measurement and the stage capture. Each extra pass adds a limiter run, a
conversion and a final check, up to about 13 s on the 7-minute song at −9.

## Material

- **Development (75 songs):**
  - the 61 Phase A songs: 41 library build, 7 corpus, 13 owner;
  - 14 of the 19 open-licensed quiet, dynamic pieces gathered on 2026-11-24
    (`private-audio-fixtures\quiet-dynamic-2026-11-24\`, split 75/25 by
    content hash).

  19 development sources are below −18 LUFS, against Phase A's 5.
- **Holdout (19 songs):** the library's 14 held-out songs and the quiet
  folder's other 5.
- **The owner's Suno and unmastered folders** were still empty.
- **The quiet set's known limit: it is classical and piano heavy.**
  - It holds 9 solo piano, 4 chamber, 3 choral, 2 orchestral and 1 ambient
    piece, from −32.3 to −18.1 LUFS, with a loudness range of 6.0 to 21.1 LU.
  - It has no folk, small-group jazz, or unmastered pop or rock mixes: every
    candidate of those kinds measured too loud.
  - So it tests quiet, dynamic input well. It does not represent a typical
    unmastered band mix, and nothing here establishes how G treats one.
  - Its source list (licences and origins, verified by the planning session)
    is committed as attribution in `quiet-dynamic-sources.json`, with each
    piece's content-hash ID and its side of the split. The audio is not
    committed.

## Findings

1. **Reach target needs more than 6 passes when it limits hard.** Past
   about 6 dB of limiting, each dB of drive adds well under a dB of delivered
   loudness. So the step that can never overshoot (add the remaining
   shortfall) closes the gap slowly. With up to 12 passes, 20 of the 27 development misses reach the
   target at 7–10 passes, with 0.2–1.6 dB more limiting. Two ways out, either
   one the planning session's call: a pass limit above the plan's 2–6, or a
   bigger step for Reach target only. A bigger step can overshoot, but the
   landing trims it, and the user asked for the loudness. Clean is not
   affected.
2. **Funkorama cannot be brought to −9 or −11 at all.**
   - Its limiter holds samples at −1.1 dBFS, while the export's final
     true-peak check reads up to 1.8 dBTP and turns the master down
     2.4–2.8 dB.
   - Even at 36 dB of limiting it stays near −11.3 LUFS for a −9 target.
   - This is today's behavior. It is recorded in
     `docs/OPEN_THREADS_AND_DECISIONS.md` and `docs/IDEAS_BACKLOG.md`, and the
     limiter and final check are on the do-not-touch list.
3. **Level in ties G to the export's source measurement.** The export now
   runs the analysis's loudness and true-peak measurement once more on the
   decoded file. That is part of the extra 0.2–1.9 s.

## What this does not establish

- **Sound.** Whether G sounds better is the owner's hand test (Checkpoint C).
- **Live audition and the interface.** Slices 5 and 6 are not built. With
  the switch on, audition currently plays today's sound, because no gains are
  resolved there yet.
- **Mac.** The Swift receipt model gained two optional fields but was not
  compiled; it needs a Mac build.
- **Album, mobile and the browser try-it** stay inert by design, and are not
  evaluated beyond building and their existing tests.
- **Typical unmastered band mixes.** The quiet material is classical and
  piano heavy (see "Material").
- **Timings** are this PC only.

## Evidence

Numbers only, in `docs/reviews/evidence/2026-11-25-auto-gain-staging-evaluation/`:

- `slice-4-development-summary.json` and `slice-4-holdout-summary.json`;
- `agreement-with-phase-a.json`;
- `reach-target-diagnostic.json`;
- `e7-performance.json`;
- `quiet-dynamic-sources.json` (attribution, IDs and split).

Private ledgers stay under `test-output/auto-gain-staging/`.
