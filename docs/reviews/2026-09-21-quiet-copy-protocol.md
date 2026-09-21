# Corrected bidirectional policy: quiet-copy development checkpoint

Frozen before new copy outcomes from `1ca596b1`. Dan requested continued local
research on September 21. This checkpoint implements the active plan's next C1
diagnostic; no shipping selector or reference policy is adopted.

## Fixed matrix

Piano, Imaginal, Metal and Rich; Universal 50; -14 LUFS, -1 dBTP, 48 kHz stereo
float delivery. Retain all sixteen verified original-level outputs from the
bidirectional checkpoint. For each source construct unclipped -40 and -80 dB
copies in memory (f64 scalar multiplication rounded to f32). For each copy
render current processing and normalized singles at offsets 0/-3/+3: **32 new
whole WAVs**, no other points. Preserve every earlier file and failed witness.

Measure actual copy integrated LUFS separately from peak-normalized PCM LUFS.
Recompute the automatic drive from that normalized descriptor using the existing
single rule: target minus normalized LUFS minus preset push minus retained
saturation small-signal gain. Do not reuse an invented actual-source LUFS below
the gate. Retain original-source analysis/guards, source settings, source-anchored
section/transient windows and original preset coefficients. This isolates the
processing/selection experiment; fresh production analysis invariance remains
outside its scope.

Current processing consumes the attenuated copy without automatic normalization
or drive. Singles consume the normalized copy; user trim remains separate and
unchanged. Measure descriptor/normalization separately. Verify old source,
settings, coefficients, offsets, drive and whole-file evidence before audio.
Record actual source/file hashes and the exact synthetic transform.

## Unchanged qualification and diagnostic comparison

First apply unchanged C1 character limits against each copy's own corrected
current control, using original-source scoring anchors. Reuse the frozen
`control-inclusive-bidirectional-v1` rule over the four independently checked
copy outputs. Compare selected policy/offset, target feasibility and reason
against the original-level result. Required selection identity changes remain
failures, even if a new choice has higher loudness or some better metric.

Separately score the same outputs against the original-level control's tone/
stereo reference. Label this **fixed-original-reference diagnostic**, not a new
selector version or production fix. This can distinguish moving reference
limits from actual candidate response differences; it is not deployable for an
arbitrary imported track whose hypothetical original gain is unknown.

Keep previously declared numerical copy budgets: 0.001 LU delivered loudness
and normalized descriptor drift, 0.001 dB drive/normalization compensation,
0.01 dB anchored section/paired-attack/band/side differences, 0.00001 correlation,
0.0001 maximum absolute delivered PCM difference. Compare all 24 normalized
copy outputs with their exact original counterpart. Current processing is
explicitly level dependent and does not inherit a PCM-invariance requirement.
Missing required candidate measurements fail, never default to zero.

## Complete-file validation and unavailable loudness

All new whole files receive unchanged independent finite-zero-padded SOXR16/64
peak checks (0.002 dB numerical tolerance), native qualified ceiling, exact
frames/rate/channels, finite/full-scale and hash checks. Finite integrated LUFS
uses the existing 0.11 LU independent agreement tolerance. Use the same pinned
FFmpeg binary. Preserve independent logs.

For below-gate current outputs, native unavailable integrated LUFS is serialized
as null with an explicit reason. Independently require FFmpeg's no-integrated-
blocks signature (I=-70, integrated threshold=0) and a full-file independent
peak below -70 dBFS. This signature is availability evidence, not a -70 LUFS
measurement. Add controlled silent/very-quiet/finite-loudness regression probes
before using this extension. Output validity and measurement availability are
separate: a technically valid below-gate file cannot be labeled a target hit.
The unchanged selector requires finite delivery LUFS; record its explicit
rejection/no-fallback result without changing that historical contract.

## Work and recovery bounds

32 new outputs maximum, 25 GiB free reserve plus estimated audio bytes. Decode
each original once, prepare each copy and descriptor once, retain each candidate.
Copy the release executable and freeze the job/protocol/source/evidence hashes
before launching. Native rendering runs alone; then character and independent
reference stages run serially. Save per-source preparation, chain/SRC/finalization,
character/reference cost and child wall/CPU/peak memory. Budgets are descriptive
for this diagnostic, with a 2 GiB native-memory qualification; never relax a
quality criterion because a check is slow. No additional search follows a miss
inside this checkpoint.

Unseen holdout stays closed. Full eight-source/two-target coverage, broader
preset/control policy, native responsiveness and final C1/C2 qualification remain
open. A failed invariant should direct the next bounded mechanism investigation,
not a larger grid, a new source fixture request or a silently loosened threshold.
