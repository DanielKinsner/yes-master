# September 5 follow-up implementation evidence

Local Windows work begun at `7a5d71c`; first fix `9238a55`. This record supplements
the reconciled owner handoff, not a replacement listening verdict. No release or
remote CI is claimed. All audio below is the existing owner-supplied fixture set.

## Volume Match

The real `MasteringSource` regression reproduced a 16.26 dB jump after a warm
match followed by an uncached 0.01 dB EQ edit. Retaining the last applied
same-source attenuation fixes the pending interval and the settled result to
within 0.1 dB in that case. Stale cache values cannot replace that fallback;
source changes reset it and VM off remains unity. Export settings are unchanged.

The native dev app imported `TEST-3min-MONO-48khz.wav`, displayed its correct
48 kHz/mono header and `Mono — One-channel source.` in Insight. Mastered playback
with VM on / Preview LUFS off continued through 20 keyboard edits across Intensity,
High EQ and Density, with advancing transport and finite meters. Settings shows
the system-default Focusrite output. These are native control/state observations,
**not** loopback recordings or new owner listening. Actual device rate/buffer and
transient audibility are not established by the screenshot evidence.

## Stage measurements

i9-13900K, application dev opt-level 1, dependency opt-level 3. Seconds, one run
per whole-file case; the diagnostic uses Universal 50%, default Advanced settings,
Streaming Universal delivery (48 kHz). Analysis includes its own decode. The
measurement windows/process launches differ from the owner's stopwatch session.
Numbers are stage timings, not promises of end-to-end UI latency.

| Existing fixture | Decode | Three deep PCM copies | Chain | Tail | SRC | LUFS + true peak feed | Global gate | Analysis |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 3 min / 192 kHz stereo | 0.547 | 0.132 | 16.353 | 0.137 | 0.885 | 0.348 | 0.000085 | 19.590 |
| 10 min / 192 kHz stereo | 1.828 | 0.468 | 73.539 | 0.432 | 2.967 | 1.087 | 0.000031 | 56.896 |
| 60 min / 96 kHz stereo | 7.161 | 1.987 | 173.780 | 1.059 | 10.825 | 7.562 | 0.000122 | 154.756 |

VM's representative eight-second calculation took 0.442–0.508 seconds in these
cases. Global gating is a tiny part of the wait; parallelizing only that reduction
cannot materially address these observed waits. Stateful DSP dominates. The
60-minute source buffer is 2,764,800,000 bytes. Prewarm/cache/A-B/landing setup
previously cloned it; immutable `Arc` sharing eliminates those copies. Three
shared clones measured below the diagnostic's microsecond display resolution,
versus 2.212 seconds for three deep copies in the paired after run. The pointer
identity regression establishes sharing independently of clock precision.

The after runs retain the exact measured LUFS and VM gains:
3 min / 192 kHz: -8.866045735824963 LUFS, gain 0.4944587;
60 min / 96 kHz: -8.596555542392789 LUFS, gain 0.56591356.
The after full-chain runs were slower (17.258 and 184.418 seconds); they do not
show a DSP speedup. Concurrent machine load and single-run variation limit
comparisons. Sharing is a setup allocation improvement, not a faster DSP claim.

Three additional algorithm experiments were discarded: saturation normalization
caching, equal-channel compressor curve reuse, and block minima for limiter
lookahead search. Each retained the exact eight-second rendered SHA-256 but
failed to improve measured throughput. The limiter variant additionally passed
bitwise comparisons across 8–768 kHz, resets, ceiling edits, and mono/stereo,
but was slower. None of these experimental algorithms is in the application.

Diagnostics are opt-in ignored Rust tests in `listening_bench.rs`, using
`YES_MASTER_BENCH_FILE` to point at existing files. They write no audio. Raw logs
are local/ignored under `test-output/listening-*`. Baseline extension logs also
cover the supplied 384/705.6/768 kHz stereo and 48 kHz mono fixtures.

## Limits and next checks

The slow fixture lane and UI integration results are recorded at their completed
checkpoints in the follow-up plan. Callback deadlines, underruns, peak process
memory and five-minute stressed audition remain unproven. Checkpoint 3 now bounds
preview work across source changes with a shared permit and cooperative
cancellation. This does not cover independent import/export worker pools. No
loading-workflow change or automatic import measurement has
been made on the strength of these offline timings.

Width/Loud mechanical tests, saved-file comparison with VM off, and Album native
receipt verification remain distinct from UI mocks and the original owner's
successful normal A/B/export observations. Preset taste remains unchanged.
