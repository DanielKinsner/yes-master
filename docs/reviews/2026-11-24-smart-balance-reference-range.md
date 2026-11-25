# Smart Balance reference range (PROVISIONAL) — 2026-11-24

**Status: provisional.** This range comes from a free, openly licensed stand-in
library, as decision D8 now allows (changed by the owner on 2026-11-24). It is
to be re-measured if the owner adds their own songs. Smart Balance ships off by
default, so a provisional range reaches only people who switch it on.

Plan: `docs/plans/2026-11-24-smart-balance-execution-plan.md`, Slice 2.
Evidence: `docs/reviews/evidence/2026-11-24-smart-balance-reference-range/`.
Statistics: `smart-balance-reference-stats.json`, with `"provisional": true`.
Attribution: `library-sources.json`.

## Bottom line

- **55 songs were measured** (41 build, 14 held out). The 40-song minimum is met.
- **The healthy range is wide.** After removing overall loudness, each band may
  sit about 5 to 10 dB above or below the library's typical shape and still
  count as healthy. Released masters legitimately differ that much, even
  without the orchestral and ambient tracks.
- **The edges are not pinned down.** All twelve edges fail the stability check.
  Under resampling, each edge moves within a 90% interval 4 to 9 dB wide,
  against a 1 dB bar. With about 40 songs this diverse, the edges are rough.
- **"It's a coat" sits inside the range in every band.** Smart Balance, as
  specified, would leave it alone. It would not reproduce LANDR's low-mid move
  on it. Relative to this library the song already has less bass and low-mid
  energy than typical, and LANDR took it further in the same direction.
- **Some finished, released mixes in the test corpus would be corrected.** The
  solo piano piece would get 3 dB cuts in bass and low mids, at the caps.
  Metalmania's scooped mids would get boosts of about 2 dB. Aphelion's dark top
  end would get a 2 dB air boost. Those read as genre choices, not flaws.
- **Forecast for gate G3** (healthy masters left alone), in-sample on the build
  split. With the plan's starting parameters it would fail. With the most
  lenient parameters the plan allows, it still fails on large moves, unless
  the boost cap is also lowered to 1 dB. Details below. This is a forecast, not
  a gate result.

## What was measured

| | Count |
| --- | --- |
| Tracks in the library folder | 57 (46 MP3, 10 FLAC, 1 WAV), 30 artists |
| Skipped: byte-identical to a test-corpus piece | 2 (Aphelion, Metalmania) |
| Skipped: not audio (`_sources.json`) | 1 |
| Accepted | **55**: 41 build, 14 held out |

Two library tracks are the same files as two pieces of the September test
corpus (`docs/reviews/evidence/2026-09-15-mastering-quality/corpus.json`).
Aphelion is a corpus **holdout** piece, and it would have landed in the library's
build split, so it would have shaped the range it is later checked against.
Metalmania is a corpus **development** piece, and it would have landed in the
library's held-out split. The run therefore excludes any library file with the
same bytes as a file in the corpus originals folder (`--exclude-matching`).
Both pieces are still placed against the range below, as corpus pieces.

The split is 75/25 by a seeded hash of each file's content (seed `20261124`).
The range comes from the 41 build songs only. A second run gave byte-identical
statistics.

## The healthy range, in plain words

Each song is measured in six bands. Each band's level is compared with the
library's typical level for that band. The song's overall offset is then
removed (the median of its six differences), so a quiet or loud song is not
mistaken for a dull or bright one. The range is the library median plus or
minus the larger of twice the robust spread and 1.5 dB (D16).

| Band | Healthy range (dB from typical) | Build songs inside | Low edge, 90% interval | High edge, 90% interval |
| --- | --- | --- | --- | --- |
| Sub, 20–80 Hz | −7.0 to +7.3 | 85% | −12.4 to −4.7 | +4.5 to +12.1 |
| Bass, 80–250 Hz | −5.7 to +6.1 | 95% | −7.1 to −3.0 | +3.6 to +8.1 |
| Low mids, 250–800 Hz | −5.8 to +6.5 | 85% | −8.5 to −3.8 | +4.4 to +9.3 |
| Mids, 800 Hz–2.5 kHz | −5.6 to +4.6 | 88% | −9.0 to −3.1 | +2.7 to +8.2 |
| Presence, 2.5–6.5 kHz | −9.6 to +9.6 | 98% | −12.4 to −5.4 | +4.9 to +12.0 |
| Air, 6.5–16 kHz | −8.1 to +9.5 | 83% | −13.6 to −4.9 | +5.7 to +13.9 |

Inside in all six bands at once: 24 of 41 build songs (59%).

So, for example, a mix whose low mids sit 6 dB above typical still counts as
healthy. Smart Balance would only act on mixes further out than that.

## Where the test songs sit

Position in each band, in dB from the library's typical shape. Where a band is
outside the range, the distance beyond the edge is in brackets. "Preview" is
what the plan's contract rules 3 to 7 would do at default strength (0.5) with
the starting parameters. It is a preview of rules not yet built (Slice 3), not
product output.

### The eight corpus pieces

| Piece (split, kind) | Sub | Bass | Low mids | Mids | Presence | Air | Preview |
| --- | --- | --- | --- | --- | --- | --- | --- |
| It's a coat (dev, owner source) | −0.3 | −1.9 | −3.8 | +0.3 | +1.3 | +6.5 | none |
| In This Moment, piano (dev, finished mix) | −3.1 | +17.6 (+11.6) | +20.2 (+13.7) | +3.1 | −9.0 | −9.1 (−1.1) | bass −3.0, low mids −3.0, air +0.5 |
| Funkorama (dev, finished mix) | +4.6 | +2.3 | −2.3 | −3.4 | −4.4 | +5.2 | none |
| Metalmania (dev, finished mix) | +2.1 | +0.3 | −9.5 (−3.7) | −8.7 (−3.1) | −0.3 | +1.0 | low mids +1.9, mids +1.6 |
| Baby Bird (dev, unmastered stem sum) | −6.1 | +2.0 | +2.6 | +0.5 | −3.8 | −0.5 | none |
| Aphelion (holdout, finished mix) | +3.2 | +0.9 | +0.7 | −0.7 | −5.7 | −12.2 (−4.2) | air +2.0 |
| Rich (holdout, unmastered stem sum) | −13.0 (−6.1) | −3.1 | +4.2 | +8.2 (+3.6) | +3.1 | −3.2 | sub +2.0, mids −1.8 |
| Imagining Imaginal (holdout, unmastered stem sum) | −22.2 (−15.2) | −2.9 | +6.2 | +0.1 | −0.1 | +3.8 | none (sub too far below to boost) |

### "It's a coat": source, LANDR, BandLab, and YES output

| File | Sub | Bass | Low mids | Mids | Presence | Air | Preview |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Source, `It’s a coat-original-test.wav` | −0.3 | −1.9 | −3.8 | +0.3 | +1.3 | +6.5 | none |
| LANDR Balanced High | −0.1 | −2.8 | −4.8 | +0.1 | +1.0 | +5.1 | none |
| LANDR Open High | −0.4 | −3.4 | −5.2 | +0.4 | +2.0 | +5.9 | none |
| LANDR Warm High | +0.8 | −2.2 | −4.7 | −0.2 | +0.2 | +3.8 | none |
| BandLab Clarity | +1.1 | −1.8 | −4.3 | −1.1 | +1.7 | +9.3 | none |
| BandLab Oomph | +4.9 | −0.8 | −6.7 (−0.9) | −2.5 | +0.8 | +7.8 | low mids +0.4 |
| BandLab Tape | −0.1 | −1.1 | −2.5 | +0.1 | +0.5 | +10.8 (+1.3) | air −0.7 |
| BandLab Universal | −0.4 | −1.8 | −3.8 | +0.4 | +1.4 | +8.1 | none |
| YES output: Universal, Intensity 75 | −0.6 | −1.8 | −3.4 | +0.6 | +1.7 | +8.0 | none |
| YES output: compressor off | −0.6 | −1.8 | −3.6 | +0.6 | +1.7 | +8.0 | none |
| YES output: compressor on | −0.6 | −1.8 | −3.4 | +0.6 | +1.7 | +8.0 | none |

All three LANDR masters lowered bass and low mids further below typical than
the source already was. They also pulled the air band back toward typical.
BandLab raised air. With this range Smart Balance would not touch the source.
The plan's Slice 6 question ("does Smart Balance move It's a coat toward what
LANDR did?") should therefore be expected to answer "no, it leaves it alone".

## Is the library skewed?

No single genre dominates. The problem is the opposite: the genres pull in
different directions. Median position by genre tag, build split only:

| Genre tag (build songs) | Sub | Bass | Low mids | Mids | Presence | Air |
| --- | --- | --- | --- | --- | --- | --- |
| Cinematic/orchestral/classical (5) | −0.1 | +2.4 | +3.7 | +3.9 | −4.3 | **−15.6** |
| Other: ambient/cinematic (3) | −0.6 | +3.5 | +4.7 | +1.3 | −4.6 | **−13.0** |
| Electronic/dance (8) | +1.2 | −0.3 | −0.8 | −1.0 | +0.0 | +4.3 |
| Hip-hop/beats (7) | +2.6 | −1.8 | −1.2 | −1.1 | −0.3 | +3.2 |
| Jazz/funk/soul (5) | −2.0 | −1.0 | −0.0 | −0.3 | +2.2 | +2.7 |
| Metal (3) | −3.3 | +1.1 | −2.8 | −0.2 | **+4.9** | +3.0 |
| Pop/indie (4) | −0.1 | +0.2 | +1.9 | −1.0 | −2.4 | −0.1 |
| Rock (4) | +0.6 | +0.5 | +0.6 | −0.0 | +2.7 | −1.9 |
| Acoustic/folk/country (2) | −4.9 | +0.7 | +2.0 | −1.5 | −1.6 | +0.5 |

- **Orchestral and ambient tracks** have about 13 to 16 dB less air than
  typical, so they form the air band's long low tail. They are **not** what
  makes the range wide. Measured on the same positions without those 8 songs,
  the air half-width would shrink from 8.8 to 7.3 dB and presence from 9.6 to
  7.4 dB. The other four bands change by 0.7 dB or less. Most of the width is
  ordinary variety among the remaining genres.
- **17 of 41 build songs** are outside in at least one band. Seven are
  orchestral or ambient, mostly for a dark top end. The other ten are one or
  two bands on other masters: for example a rock master's sub at +12.0, an
  electronic track's sub at −13.8, a jazz master's bass at −7.3.
- **Five artists** each have three songs in the build split (15 of 41).
- **17 of 41 build songs** peak at full scale (0 dBFS), consistent with the
  known clipping in loud masters.
- Acoustic/folk/country has only 2 build songs. Its other 5 fell in the
  held-out split.

## Forecast for gate G3, in-sample

G3 requires, on the held-out songs at default strength: at least 70% get zero
correction in every band, at least 95% get no band correction above 1.0 dB, and
none above 2.0 dB. The figures below use the **build** songs, the songs the
range was made from. Held-out results are usually worse.

| Parameters | Zero in every band | No band above 1 dB | Any band above 2 dB |
| --- | --- | --- | --- |
| Starting: multiplier 2.0, min half-width 1.5, dead zone 0.25, fade 6–12 | 68% | 78% | 2 songs |
| Multiplier 3.0 | 78% | 83% | 0 |
| Most lenient allowed: multiplier 3.0, min half-width 3.0, dead zone 0.5, fade 4–8 | 83% | 85% | 0 |
| Same, plus boost cap lowered to 1.0 dB (allowed: "lower only") | 83% | 98% | 0 |

These figures use the preview rules above. Gate G3 is still owed in Slice 6,
on the held-out songs, once, with frozen parameters.

## Known limits of this library

- Nobody listened to these songs. Production quality rests on source
  reputation and objective checks. Several loud masters clip.
- Genre labels come from source tags, not from listening.
- For one track (Townhouse Woods, "Hard Country") the uploader may not be the
  rights holder. Its licence was read from the Internet Archive item's own
  metadata.
- The library leans toward independent releases. Mainstream-label pop, hip-hop
  and metal are not represented.
- 46 of 57 files are MP3s (256 kbps and up). The meter's top edge is 16 kHz.

## What this does not establish

- That the range is right for any particular genre. It is one range for all
  of them.
- Anything about the 14 held-out songs. Their positions were not examined, so
  they stay clean for gates G3 and G4.
- Any sound. Nothing was rendered, and no product behavior changed.

The plan asks for all eight corpus pieces here, so the three corpus holdout
pieces (Aphelion, Rich, Imagining Imaginal) have now been seen. They stay
outside tuning, as the holdout protocol requires.

## Reproduce

From `src-tauri` in the worktree, with the main checkout at `$M`:

```powershell
cargo run --release --target-dir target\codex-rc --example smart_balance_reference -- --references "$M/private-audio-fixtures/reference-masters/free-cc-2026-11-24" --exclude-matching "$M/test-output/mastering-quality-20260915/originals" --provisional --output "$M/test-output/smart-balance/reference"
cargo run --release --target-dir target\codex-rc --example smart_balance_reference -- --stats "$M/test-output/smart-balance/reference/smart-balance-reference-stats.json" --place "$M/test-output/mastering-quality-20260915/sources" --place "$M/tests for presets" --output "$M/test-output/smart-balance/placement"
```

The per-file ledger and the placement report name files and stay under the
ignored `test-output/`. The deviation log for Slices 0 to 2 is in the plan's
"Execution status" section.
