# Paired prepared-measurement reuse checkpoint

Freeze before timing. Use the same four bound prepared buffers and completed
-9 deliveries from the target-reuse checkpoint, only after all four independent
whole-file checks pass. This isolates an existing production API's performance;
it does not change DSP, protection, a sonic rule or application cache behavior.

For each buffer, decode once and run the existing `output_protection::prepare`
once, recording that preparation cost. Compare public `finalize` (fresh facts)
with existing `finalize_prepared` (reused facts), 48 kHz stereo float, -9 LUFS,
-1 dBTP. Both receive an identical fresh clone of the immutable buffer. Include
that clone in both observed finalization times. Alternate fresh/reused and
reused/fresh order for two rounds per source: sixteen paired-path outputs total.
Do not repeat until a desired speedup appears. Report both observations and the
one-time preparation, not just the favorable timing.

Every complete output must match the already independently verified target-reuse
WAV **bit for bit**, including exact frames/channels/rate and the returned
loudness/gain/peak contract. Preserve source/prepared/output hashes and failures.
Do not create additional WAV copies when equality permits exact evidence reuse.
Do not relax peak or identity checks. If samples differ, stop the reuse claim and
retain the failure for investigation; the existing reference does not certify it.

Invoke a specifically ignored Rust unit benchmark so private prepared-measurement
APIs stay private. The module is compiled only for tests. Build/copy the release
test executable before the zero-new-WAV/25 GiB reserve preflight. Record source
revision, binary hash, whole child wall/CPU/peak memory and per-path timings.
This is offline whole-buffer work; map lookups and device playback/application
remain distinct measurements. It does not establish an installed-app speedup.
