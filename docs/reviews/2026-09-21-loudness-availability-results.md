# Complete-block loudness availability results

The [frozen diagnostic](2026-09-21-loudness-availability-protocol.md) completed
with tooling `cd2da419`. The new `complete-block-availability-v1` method agrees
with native ebur128 measurements on **all 32 retained quiet-copy files**:
28 finite integrated results and four independently confirmed unavailable
results. There are no unknown retained-file classifications. The prior 31/32
qualification and its failed peak guard remain unchanged as historical evidence.

[Joined measurements](evidence/2026-09-21-dynamics-research/block-availability-v1.json)
and [verification](evidence/2026-09-21-dynamics-research/block-availability-verification-v1.json)
bind the source files, copied native meter, pinned FFmpeg, complete logs,
protocol and reused peak/format checks. No music was rerendered.

## What was wrong with the earlier availability check

The [matching FFmpeg revision](https://raw.githubusercontent.com/FFmpeg/FFmpeg/b18a9c2971/libavfilter/f_ebur128.c)
gates integrated loudness using 400 ms loudness blocks. It initializes the
integrated display to -70 and logs momentary values to one decimal place.
Sample peak is a different quantity: a brief peak above -70 dBFS does not
establish any loudness block above the absolute gate. The new classifier requires
complete consecutive block logs, a consistent final summary and a rounding
margin, returning unknown whenever those facts cannot establish availability.

For the retained Metal -80 current control, all **1,904** expected readouts are
present. Maximum displayed momentary loudness is **-82.9 LUFS**; the fresh
native maximum is **-82.864016 LUFS**, with no gating-block count/energy result
and explicitly unavailable integrated loudness. The old independent peak is
still **-69.071192 dBFS**. The failed peak guard was too restrictive for this
unavailable-loudness case; the new complete-block evidence resolves the meter
question without altering that earlier failure or inventing a -70 measurement.

## Controlled qualification

Twelve deterministic stereo float signals were measured by both implementations.
Ten receive matching classified results; two deliberately remain unknown.

| Control | Native | New independent result |
| --- | --- | --- |
| Silence; -90 dB tone | Unavailable | Unavailable |
| 10 ms burst, -60 dB peak | Unavailable | Unavailable; momentary maximum -76.0 |
| 10 ms burst, -40 dB peak | -56.0164 LUFS | Finite, -56.0 display |
| -72 / -70.2 dB tones | Unavailable | Unavailable |
| -70.04 dB tone | Unavailable | Unknown: rounded momentary -70.0 |
| -69.96 dB tone | -69.9533 LUFS | Finite: I=-70.0, threshold=-80.0 |
| -69.8 / -68 dB tones | Finite | Finite, matching within 0.11 LU |
| 0.3 s tone | Unavailable | Unknown: unsupported duration |
| Last-10-ms burst, -60 dB peak | Unavailable | Unavailable |

The two near-gate tones demonstrate why the display alone is insufficient:
I=-70 can be a default unavailable display or a rounded finite measurement.
Unknown is not a pass, a guessed measurement, or a selector candidate. Tests
also reject missing, duplicated, reordered and malformed records; missing or
duplicate summaries; unsupported inputs; fabricated -70 native values; and
native NaN/positive infinity. The old peak-guard regression still requires its
historical failure.

## Cost, boundaries and next work

Controlled native wall time is 0.103 s; controls including FFmpeg take 0.708 s.
Retained native wall time is 23.487 s and fresh FFmpeg reference work totals
62.898 s, executed sequentially. This is meter-only work on retained complete
files, not mastering throughput or an app responsiveness benchmark. Native
LUFS availability matches the prior reports, with finite differences <=0.001 LU.
All prior passing SOXR16/64 peak and exact float/frames/rate/channels checks
are reused only after file hashes and formats match. No peak checks were weakened.

The release meter example builds and passes strict Clippy/Rust formatting.
New parser regressions and the preserved historical availability regressions
pass. Python compilation, artifact bindings, internal links and whitespace
checks pass. Application DSP, selectors, preset coefficients, controls, saved
values and production source remain unchanged; this is not native playback,
listening, holdout or shipping-policy evidence.

Availability is now understood for this supported 48 kHz stereo 0.4–600 s
diagnostic scope. The selector still requires finite delivered loudness, so the
Imaginal/Rich below-gate fallback failures do not disappear. C1's next mechanism
study is a reproducible reference for tone/stereo comparison, separate from
target attainment and fallback eligibility.

Code inspection narrows that study: the production compressor includes an LR4
split/recombination network and fixed per-band makeup even when gain reduction
is zero (`dsp.rs`, `apply_multiband_compressor`). Simply turning compression
Off would remove those contributions and would not represent the same preset's
static response. The next reference-only prototype should explicitly retain
that network, the existing EQ and width, isolate time-varying gain/transient
shaping and saturation, and prove its response before using it for scoring.
This is an engineering starting hypothesis, not a frozen sonic rule. The earlier
failed low-drive convergence experiment remains unusable as a proven linear
reference. Broader control contracts, C2 holdout and C3 integration remain open.
