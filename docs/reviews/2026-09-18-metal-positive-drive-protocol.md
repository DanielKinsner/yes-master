# Metal: reproduce the retained positive-drive lead on the corrected chain

Frozen after the new Universal 50 character measurements, before new candidate
audio. This is a separate, bounded follow-up to the 24-file coverage checkpoint.
It does not alter that checkpoint's selection, limits or reported failures.

## Why these two renders

Metal's zero-offset single still fails the top tone band at Universal 50/-14,
as it did at Universal 75. September 16's -3/-6/-12 follow-up at Universal 75
did not repair this. The **already completed historical C1 grid**, using the
older filter-state arithmetic, contains a **+3 dB** candidate at both Universal
50 and 75 that meets every existing character limit and -14 target. +6 dB
fails the median-attack limit. These are development leads, not proof for the
current corrected chain. Reuse those findings instead of repeating the grid.

Test exactly **two new full renders** of Metal: Universal 50 and Universal 75,
-14 LUFS, **+3 dB relative to the normalized single rule**, compensated after
the chain in the same way as the previous negative-offset experiments. Retain
the two current controls and two zero-offset singles by exact file/evidence
identity. No other drive points, sources, targets or preset changes in this
checkpoint. A positive offset means more drive than the experimental single;
it is not a change to manual input gain or a claim that more drive is better.

Use the same source, original requested settings, source anchors, coefficient
strings and operating drive from `c1-metal-v2`. Bind its report and metrics,
including the historical +3/+6 rows, by hash. Preserve the corrected production
chain/SRC/finalizer and 48 kHz stereo float/-1 dBTP delivery. Source normalization,
automatic drive, experimental offset and scalar landing remain separate.

## Acceptance and bounds

The original C1 section/paired-attack/tone/stereo limits and 0.2 LU target
tolerance remain unchanged. Each new candidate uses its matched corrected
current control for tone/stereo bounds. Require independently verified complete
outputs and matching source/settings provenance for all four retained controls
and singles before executing; a pending, failed or mismatched record stops the
follow-up. Independently check both new whole files with the unchanged SOXR16/64,
LUFS, frame/rate/channel/finite/full-scale/hash checks.

Record each indicator and whether character and target separately qualify.
Do **not** feed +3 into `control-inclusive-preserving-v1`: its frozen candidate
set excludes positive offsets. A success is evidence for a future explicitly
bidirectional bounded search, not a retroactive selection success. A failure
stays failed; no additional points or threshold changes to rescue it.

Copy the release executable and check a 25 GiB reserve plus the two-file estimate
before launch. Run after the Universal 50 verification has completed. Render
alone, then run references and character reporting. Count decode/normalization,
chain/SRC/finalization, process wall/CPU/memory, metrics and references separately.
Reuse original analysis and retained measurements without assigning them zero
historical cost. No app latency, listening, unseen holdout, shipping policy,
saturation-curve adoption or production behavior claim follows from this probe.
