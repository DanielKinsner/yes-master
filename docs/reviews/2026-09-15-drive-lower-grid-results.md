# C1 corrected-chain lower grid: completed comparison

The [separately frozen grid](2026-09-15-drive-lower-grid-protocol.md) finds a
character-qualified candidate for each of its three known development tracks.
Coat needs only the single-rule candidate. Piano and Imaginal trade loudness for
the existing dynamics constraints. This supports a single-first, bounded-search
approach with explicit shortfalls; it does not complete C1 or adopt sonic limits.

## Protected whole outputs

All **12 complete outputs** pass independent peak/LUFS checks, exact frame/rate/
channel checks, finite PCM and zero full-scale samples. Three zero-offset renders
reproduce the retained normalized single-rule controls exactly and reuse their
identified checks. **Nine new WAVs receive fresh independent SOXR16/64 checks.**
The highest native qualified bound is -1.0000098 dBTP against the -1 request.
Controls use the production chain with the experimental single-rule drive;
they are not the application's current automatic policy.

No coefficient, preset, character limit or source anchor changed. The current
f64 filter-state chain, production SRC and finalizer are exercised. The earlier
f32 grid and the failed -24/-48 numerical-convergence diagnostic remain intact.

## Fixed hypothetical selections

All targets are -14 LUFS. The preserving rule uses the unchanged individual
section/attack/tone/stereo limits; it does not maximize crest or minimize limiting.

| Source | Selected offset from single rule | Delivered LUFS | Target miss | Result against existing character limits |
| --- | ---: | ---: | ---: | --- |
| Coat | 0 dB | -14.000 | 0 LU | Pass; extra search does not change selection |
| Piano | -3 dB | -14.804 | -0.804 LU | Pass; section loss improves from 1.586 to 0.652 dB |
| Imaginal | -6 dB | -18.142 | -4.142 LU | Pass; paired attack median/p10 change improves from -3.842/-6.311 to -0.478/-1.592 dB |

The target-first rule selects zero offset on all three. It hits the target while
retaining Piano's section-contrast failure and Imaginal's two attack failures.
No verified control fallback is needed within this small preserving comparison.
This is not evidence that a qualified candidate always exists on other tracks.

Imaginal at -3 dB still fails the attack lower-decile limit; at -12 dB its dynamics
pass but bands 7 and 8 fail the tone allowance. More attenuation is not uniformly
better. No unsampled intermediate candidate is inferred and the grid is not
expanded to obtain a better result.

## Work that may be avoided

The table sums observed chain processing, SRC and qualified finalization. It
excludes source analysis, metric calculation, disk I/O and independent references.
Rendering overlapped the independent checker and metrics, so these are one-pass
loaded observations, not controlled throughput or native interaction budgets.

| Source | Single candidate | All four candidates |
| --- | ---: | ---: |
| Coat | 21.681 s | 84.086 s |
| Piano | 16.938 s | 68.178 s |
| Imaginal | 32.433 s | 123.923 s |

For Coat, the frozen selection rule necessarily accepts the first candidate:
the other three account for **62.405 seconds of measured audio evaluation work**
that this decision could skip. This is evidence for conditional search, not a
62-second measured product speedup. Piano and Imaginal justify investigating
additional candidates. Actual preparation, settings settling, playback deadlines
and total native session cost still require the integrated policy's benchmarks.

## Recommendation, provenance and remaining work

Recommend character preservation with visible loudness shortfalls for automatic
mastering, preserving deliberate manual drive. The owner choice remains pending;
these findings do not supply an answer or establish an audible preference.
Next: broaden corrected-chain selection across the existing sources, presets and
quiet copies; settle numerical limits, search budget, fallback and control mapping;
freeze before new C2 holdout material. C3 still needs its production integration
and affected fixture/bridge/native/UI evidence. Saturation adoption remains separate.

Evidence under `test-output/mastering-quality-implementation-20260915/`:

- `c1-lower-drive-job-v1.json`, `c1-lower-drive-v1/report.json`;
- `c1-lower-drive-metrics-v1.json`, `c1-lower-drive-independent-v1/comparison.json`;
- `c1-lower-drive-summary-v2.json`, joining all 12 WAV hashes and measurements;
- strict example Clippy/build logs and Python compilation checks pass. No
  production source or frontend changed, so unchanged full fixture/bridge/UI
  results are reused rather than rerun for this research-only checkpoint.

The first summary attempt rejected Windows slash spelling differences when
joining paths. Its failed log is retained; v2 normalizes path identity and still
requires exact hashes and every original assertion. No output was rerendered
or tolerance changed to repair that reporting error.

Nine new WAVs occupy approximately 919 MB. Preflight passed the 25 GiB reserve
before building/copying the executable; subsequent observed free space was
24.993 GiB. Thus the preflight did **not** maintain its post-run reserve across
the intervening build/host writes. Build and copy the probe **before** future
output-budget preflights, and check the reserve again at launch. All prior
research and private sources are preserved; no cleanup was performed.

Copied executable SHA-256:
`f72362beb4a29e56a20a18f5093799479c2c17fe7f23ee711b09d58fce998d69`.
