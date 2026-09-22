# Loudness availability: complete-block diagnostic v1

Frozen from `2a9362fb` before new controlled outcomes. Local research only.
The prior quiet-copy protocol, its 31/32 result, classifiers and selection
failures remain unchanged. No mastering rerender or holdout source is required.

## Mechanism and proposed rule

The [FFmpeg source matching the pinned binary's reported revision](https://raw.githubusercontent.com/FFmpeg/FFmpeg/b18a9c2971/libavfilter/f_ebur128.c)
initializes integrated loudness to -70 and updates it when 400 ms loudness
passes the absolute gate (lines 510 and 662–676). Logged momentary values use
one decimal place (line 727). A transient sample peak cannot identify whether
a loudness block passes that gate. The local ebur128 0.1.10 history code returns
negative infinity when there are no blocks above threshold; NaN is a distinct
error. Preserve these distinctions.

A separate `complete-block-availability-v1` classifier will inspect complete
FFmpeg logs. Supported inputs are 48 kHz stereo float, 0.4–600 seconds, no input
timestamp offset, no resampling, absolute LUFS display. Require exactly one final
summary, a finite I/threshold pair, and exactly floor(frames/4800) consecutive
100 ms readouts with expected last-sample timestamps (0.0006 s rounding budget).
Missing, duplicate, truncated, reordered or malformed records produce unknown.

- I=-70, threshold=0, and every displayed momentary value <=-70.1 establish
  unavailable integrated loudness. This leaves a rounding margin below -70.
- A nonzero integrated threshold, I>=-70 and at least one momentary display
  >=-70 establish a finite integrated result, retaining 0.11 LU agreement.
- Otherwise return unknown with no invented independent LUFS or availability.
- Native negative infinity means unavailable, native NaN/+infinity means error.
  Never turn unknown/error or unavailable into a target hit or selector candidate.

Peak safety remains the previously verified independent SOXR16/64 check; this
classifier neither relaxes that ceiling nor replaces its evidence. Native null
is not sufficient for an independent pass. A finite native value against an
unavailable independent result must fail, including a fabricated -70 value.

## Bounded qualification

Generate twelve deterministic stereo float fixtures at 48 kHz: 2 s silence;
2 s 1 kHz tone at -90 dB peak; 10 ms 1 kHz bursts at 0.8 s with -60/-40 dB
peaks; six 2 s tones at -72/-70.2/-70.04/-69.96/-69.8/-68 dB peaks; a 0.3 s
-20 dB tone (unsupported duration); and a 10 ms -60 dB burst at 1.99 s in a
2 s file. Calculate in f64 then store f32. Save all files/logs/hashes locally.

Use the existing ebur128 0.1.10 crate in an offline meter-only example, recording
integrated availability, relative threshold, gating-block count/energy and
momentary maxima separately from FFmpeg. No app DSP is changed. Control tests
must cover unavailable audio with peaks above -70, finite results near the gate,
rounded boundary unknowns, short input unknowns, fabricated loudness, omitted/
duplicated/reordered log records, missing summaries and nonfinite native values.
All known-status classified cases must agree; unknowns must stay explicit.

After controls, apply the same rule to all 32 retained quiet-copy WAVs using
fresh whole-file FFmpeg loudness logs and native meter observations. Bind exact
WAV hashes/frames to the prior report and reuse only its unchanged passing
technical peak/format evidence. Save a new additive report; do not rewrite the
old availability result or feed this diagnostic into a new sonic selector.

Run native and FFmpeg measurement serially and record elapsed costs. No new
mastering outputs, drive points, presets or source fixtures. The resulting
availability facts inform an explicit fallback contract; they cannot repair
the independently observed moving-reference/character-selection failures.
