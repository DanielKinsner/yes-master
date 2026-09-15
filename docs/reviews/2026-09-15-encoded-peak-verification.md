# B4 encoded delivery: measurement, parity and correction experiment

Local development evidence, September 15, 2026. No release, listening verdict or
automatic lossy correction policy is activated. PCM protection remains enforced;
valid lossy delivery remains advisory against its requested ceiling.

## Production corrections

- Completed FLAC/AIFF decode must match every protected input PCM sample and
  frame, at the expected rate/channels. Negative checks reject a one-LSB change,
  changed length/channels, and cancellation. This preserves the existing 16/24-bit
  delivery contract, including the explicitly labeled 24-bit fallback for float.
- Every encoded receipt uses the qualified finite-file peak estimator over the
  actual decoded PCM. EBU integrated loudness and LRA remain separate measurements.
  Scratch float WAVs permit bounded reads without retaining another whole song
  in memory; cancellation and owned-file cleanup remain active.
- MP3 readback had a second under-reading cause: Symphonia 0.5.5 clamps decoded
  synthesis samples to [-1, 1]. One retained 128 kbps music case independently
  peaks at +0.525022 dBTP, while the old decoded PCM reports only +0.266655 under
  the qualified estimator. This is a decoder defect, not an estimator tolerance.
  A scoped `symphonia_readback` facade 0.6.0 / locked MP3 bundle 0.6.1 removes the
  clamp and preserves gapless frame counts. General import/playback still uses
  0.5.5; this does not claim that separate migration or native listening proof.
  The new decoder requires Rust 1.85, now declared by the desktop crate; current
  builds use Rust 1.98. No test on the minimum compiler is claimed.
- Track advisories compare actual peak with the actual requested ceiling,
  including small misses below the old generic warning thresholds. Target
  shortfall wording no longer promises a safe copy when decoded delivery exceeds
  the ceiling. It recommends review/lower output ceiling.
- Album reports and manifests now carry `continuous_peak`, independently of
  component-track results. PCM uses the complete protected programme; encoded
  routes replace it with the complete decoded continuous file. The programme
  ceiling is the least restrictive component ceiling, as in B3's join policy.
  The receipt displays that result and any excess. Optional serialization keeps
  older receipts readable. Per-track target differences above 0.25 LU and peak
  excesses above numerical tolerance are visible, including small excesses.

## Frozen stimuli and independent checks

`make_codec_inputs.py` uses deterministic tonal/HF/transient stress (four seconds),
50 ms tone/end-impulse inputs, and an eight-second extract of the already-seen
frozen owner control. This is codec characterization, not a listening corpus or
unseen mastering holdout. Source and output hashes are checked before reuse.

The native probe protects PCM with the production finalizer, encodes actual
files, and records decoded measurements, frame counts, formats, hashes and cost.
The independent checker decodes with FFmpeg 9.0.1, then checks finite zero-extended
SOXR 16/64x reconstruction plus sample peak per channel using the full SOXR build.
The 2048-sample zero extension includes exterior reconstruction ringout.
Production encoding remains the packaged FFmpeg 8.0.3 candidate.

| Format | Cases | Independent ceiling misses | Highest independent peak |
| --- | ---: | ---: | ---: |
| MP3 | 72 | 23 | +0.525022 dBTP |
| M4A AAC | 48 | 28 | +7.424334 dBTP |
| ADTS AAC | 48 | 28 | +7.424334 dBTP |
| Ogg Vorbis | 36 | 14 | +0.212524 dBTP |
| FLAC | 56 | 0 | -1.038147 dBTP |
| AIFF | 56 | 0 | -1.038147 dBTP |

All use a requested -1 dBTP ceiling. Lossy cases cover mono/stereo, MP3
32/44.1/48 kHz at 128/192/256/320 kbps, AAC 44.1/48 kHz at those four rates,
and Vorbis 44.1/48 kHz at quality 4/6/8. Lossless cases cover 16/24 bit and
32/44.1/48/88.2/96/176.4/192 kHz. Not every source class covers every rate;
the frozen input manifest records the exact matrix. Short/HF stresses account
for extreme AAC growth; do not generalize those maxima to ordinary music.

Initial independent result: **315/316** technical passes, retaining the MP3 clamp
failure. After changing only MP3 receipt decoding, the same encoded bytes pass
**316/316** technical checks. This includes exact lossless PCM, declared frame
behavior and no qualified-meter under-read beyond the independent 0.002 dB
numerical tolerance. It does **not** mean every lossy ceiling passed. Independent
measurements/hashes were reused only after asserting unchanged input/output bytes
and preserved decoded-file hashes. No historical failure was overwritten.

FFmpeg 9.0.1 supplies exact expected non-ADTS frame counts. ADTS retains declared
priming/padding. The production M4A FFmpeg 8 readback still permits its previously
documented decoder padding; this is distinct from independent gapless file proof.

## Headroom versus bounded correction

Before encoding corrections, the saved rule sets per-format headroom to the
maximum observed nonnegative growth on synthetic inputs only. It then compares
that uniform attenuation with exact measured-excess correction, at most two
re-encodes, always from the original retained float master. No cumulative lossy
transcode or repeatedly attenuated quantized source is used. Actual failed cases
remain in the reports; exhaustion means an honest advisory for valid output.

Synthetic headroom was MP3 0.364 dB, AAC about 8.465 dB, Vorbis 0.318 dB. In the
204 headroom outputs, production bounds still miss on 9/72 MP3 and 10/36 Vorbis;
AAC meets that bound at a substantial level cost. These are development results
using known music; they do not establish a universal headroom policy.

Of 105 initial measured corrections, 45 still exceed the production bound.
After the second correction, **18/45** remain above that bound. Independent
decoding/reconstruction of all 45 passes the technical measurement checks and
confirms **10/45** exceed the requested ceiling: MP3 3/14, M4A 2/9, ADTS 2/11,
Vorbis 3/11. The conservative-bound and independent counts intentionally differ.
The highest remaining independent peak is +0.250422 dBTP (AAC).

One first-round MP3 gain was derived from the old clipped readback, then its
retained bytes were remeasured using the corrected decoder before round two.
Those exact gains remain frozen; this comparison is not a complete rerun of a
new-decoder correction policy. Many failures on other cases/formats independently
reject a claim that two exact-excess re-encodes enforce the ceiling.

Encode-plus-readback totals were 141.93 seconds for 316 initial files, 148.92 for
309 headroom/first-correction files, and 23.65 for 45 second corrections. These
short-input runs overlapped development/verification and are descriptive costs,
not controlled full-song performance comparisons. Whole-file PCM preparation
and independent reconstruction are recorded separately in the raw reports.

**Recommendation:** retain advisory lossy output with truthful decoded results.
Neither measured synthetic headroom nor two corrective encodes justify a new
automatic policy here. Working formats remain available. Lossless PCM remains
enforced. No unresolved sonic preference is treated as answered by this result.

## Replayable evidence

Scripts: `scripts/research/mastering-quality-implementation-20260915/`
contains `make_codec_inputs.py`, `check_codec_matrix.py`,
`make_codec_corrections.py`, `recheck_codec_readback.py`.
Native examples: `mastering_quality_codec_matrix`, `mastering_quality_codec_readback`.

Ignored evidence root: `test-output/mastering-quality-implementation-20260915/`.
Preserve `b4-codec-inputs-v2`, `b4-codec-matrix-v1`,
`b4-codec-independent-v1`, `b4-codec-unclipped-v1`,
`b4-codec-independent-unclipped-v1`, `b4-codec-correction-inputs-v1/v2`,
`b4-codec-correction-v1/v2`, `b4-codec-correction-unclipped-v1`, and
`b4-correction-independent-v2`. Failed generation using a no-SOXR essentials
build and the original measurement failure remain retained.

Independent decoder SHA-256:
`72a489eccd008c2ec2c0a5856c5c75bc3d8bbfa90166c4566865c246445e6aa3`.
SOXR reference executable:
`05f4251bce9293c2ab492cb17ca7724a0ffd0d06c881ba2ee83b82a89c2fc740`.
Initial native probe:
`1d90c50bddd3ab3a0c640e593eb97688696a000cbb2b56ad4a570fce1940512d`.
Unclipped retained-file readback probe:
`e996a4d7fa3505a4f425de93d15e3a39d41fed8b7ca87e962cafb4263f48e01a`.
Final second-correction probe:
`134d15756c5758f4cd5760fed99603446b420e0121d6431d879825327d9ffde6`.
The raw provenance is authoritative for exact bytes and per-case facts.

Local native format/album, wire, bridge and rendered-UI verification is recorded
in the checkpoint ledger. It does not establish installed-app, Mac, subjective
listening, minimum-compiler or release readiness.
