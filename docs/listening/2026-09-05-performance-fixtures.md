# September 5 performance fixtures and owner additions

These are existing local test files supplied by the owner for the
[listening follow-up plan](../plans/2026-09-05-listening-follow-up.md).
**Do not regenerate them or add their audio to git.**

The supplied folder shorthand was `Documents\Vera\_save created documents here`.
Read-only inspection found the files on this machine at:

```text
C:\Users\Daniel Kinsner\OneDrive\Documents\Vera_save created documents here
```

The actual directory name is `Vera_save created documents here`. On another
machine resolve its Documents location and reuse these filenames; do not assume
the local OneDrive prefix or generate replacements when a path differs.

## Provenance and verified headers

The owner reports that every file uses the same 48 kHz source, looped and trimmed
to the specified duration; higher rates were upsampled with soxr and contain no
source content above 24 kHz. This provenance is owner-supplied. Header-only
`ffprobe` inspection confirmed all 15 durations, sample rates, channel counts,
and bit depths below. No audio was rendered, regenerated, modified, or listened
to during this verification.

| Filename | Duration (seconds) | Sample rate (Hz) | Channels | PCM bits |
| --- | ---: | ---: | ---: | ---: |
| `TEST-10min-48khz.wav` | 600 | 48000 | 2 | 24 |
| `TEST-15min-48khz.wav` | 900 | 48000 | 2 | 24 |
| `TEST-20min-48khz.wav` | 1200 | 48000 | 2 | 24 |
| `TEST-30min-loop.wav` | 1800 | 48000 | 2 | 16 |
| `TEST-60min-48000hz.wav` | 3600 | 48000 | 2 | 24 |
| `TEST-60min-96000hz.wav` | 3600 | 96000 | 2 | 24 |
| `TEST-3min-192khz.wav` | 180 | 192000 | 2 | 24 |
| `TEST-5min-192khz.wav` | 300 | 192000 | 2 | 24 |
| `TEST-10min-192khz.wav` | 600 | 192000 | 2 | 24 |
| `TEST-3min-MONO-48khz.wav` | 180 | 48000 | 1 | 24 |
| `TEST-5min-MONO-96khz.wav` | 300 | 96000 | 1 | 24 |
| `TEST-5min-STEREO-192khz.wav` | 300 | 192000 | 2 | 24 |
| `TEST-2min-STEREO-384khz.wav` | 120 | 384000 | 2 | 24 |
| `TEST-1min-STEREO-705khz.wav` | 60 | **705600** | 2 | 24 |
| `TEST-1min-STEREO-768khz.wav` | 60 | 768000 | 2 | 24 |

The MONO files are true one-channel files. The 30-minute file is the only 16-bit
file. `705khz` is the filename spelling; its actual rate is **705.6 kHz**.

These files are useful for controlled frame-count/load comparisons and the Mono
label regression. Their upsampled source is not evidence of behavior on original
ultrasonic-rich program material. Silence, filtering, and limiter activity can
also affect cost, so pair them with existing synthetic branch/edge-case probes
where needed without recreating this fixture set.

## Additional owner observations and directions

- At **384 / 705.6 / 768 kHz**, the owner observed increased lag, **never a timeout**.
  These are additional load cases, not new timing benchmarks or platform guarantees.
- Capture the **full error text to a durable diagnostic log on the next occurrence**
  of the transient bottom-right error during a rapid switch into the 60-minute
  96 kHz track. The reproduction can remain intermittent while the evidence improves.
- A true one-channel source must show **Mono**, not “Narrow — Mono-leaning stereo
  image.” The plan identifies the existing channel metadata needed for this fix.
- Present the album-manifest decision as exactly two choices: **`metadata/`
  subfolder**, or **remove the manifest and rely on the in-app Album receipt**.
  Keeping the manifest is not a default owner decision. The answer belongs in
  the owner decision log when supplied.

## Interpreting the throughput hypothesis

The reported elapsed times imply these approximate **source-frame** throughputs:

| Case | Source frames | Owner elapsed time | Frames per second |
| --- | ---: | ---: | ---: |
| 10 min × 48 kHz | 28,800,000 | 17 s | 1,694,118 |
| 15 min × 48 kHz | 43,200,000 | 22 s | 1,963,636 |
| 20 min × 48 kHz | 57,600,000 | 35 s | 1,645,714 |
| 60 min × 96 kHz | 345,600,000 | 210 s | 1,645,714 |
| 3 min × 192 kHz | 34,560,000 | 22 s | 1,570,909 |

This supports a **roughly frame-count-driven bottleneck** as a starting hypothesis,
not proof that only frame count matters or that sample rate never affects cost.
Three minutes at 192 kHz contains **20% more frames** than ten minutes at 48 kHz;
it equals twelve minutes at 48 kHz by frame count. Channels, settings, active
branches, resampling/output rate, allocation, and concurrent work remain variables.

The inspected path spawns a background worker with `std::thread::Builder`, then
runs the mastering chain through a serial frame loop, optional sample-rate
conversion, and one ebur128 loudness/true-peak feed. “No thread::spawn text” is
not evidence of no threads. See plan step 2A for the stage breakdown and the
conditions on any parallel measurement experiment.
