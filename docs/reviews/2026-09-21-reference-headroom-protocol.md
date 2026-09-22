# Reference-headroom single candidate: development v1

Frozen from `336a23c5` before new mastering outcomes. The static-reference
checkpoint is complete and preserved. This is a new candidate-construction
experiment, not an extension of its drive grid or an adopted selector.

## One derived drive, no offset search

For Piano/Imaginal/Metal/Rich, Universal 50, retain source analysis/guards and
the exact verified settings. Use each qualified static reference's measured
source-rate sample peak. With peak-normalized input, derive automatic drive as:

`drive_db = requested_ceiling_db - 20*log10(static_reference_peak) - preset_input_push_db - saturation_small_signal_gain_db`

Calculate in f64 and round once to the production f32 drive. Verify retained
preset push/saturation gain against actual coefficients. These are separate
quantities; do not cancel user trim or alter preset coefficients. The current
matrix has user input/output trim zero and Volume Match off. No general manual
control compatibility is inferred. The formula contains no target-LUFS term;
requested -14 LUFS remains the final delivery target, not the operating level.

The reference excludes dynamic compressor gain, transient shaping and nonlinear
saturation. Its predicted sample headroom is therefore a hypothesis, not an
actual peak guarantee. Run the unchanged complete production research chain,
corrected SRC and final output protection. Measure actual limiter/compressor
activity, delivered loudness, peak, character and cost. Large target misses
remain explicit even if character qualifies.

## Fixed twelve-file matrix and limits

Each source gets 0/-40/-80 dB copies, with the previous f64 -> f32 transform
and peak normalization. Recompute drive from the corresponding already verified
static reference (do not force copied drives to equal originals). Render exactly
one new complete 48 kHz stereo float master per copy: twelve total. No additional
offset, source, preset, target or holdout file. Preserve all previous 48 masters
and reference buffers. Hash-bind jobs, source transformations, reference and
coefficient identities, and verify exact expected delivered frame counts.

All twelve receive fresh full-file finite/full-scale, SOXR16/64 zero-padded
peak checks with the same 0.002 dB numerical tolerance and native <=-1 dBTP
requirement. Use the pinned FFmpeg and complete-block availability classifier;
finite loudness agreement remains 0.11 LU. Preserve all independent logs.

Compare eight quiet deliveries against their new original-level counterparts:
absolute PCM difference <=1e-4, delivered LUFS/drive drift <=0.001, anchored
section/paired-attack/tone/side differences <=0.01 dB and correlation <=0.00001.
Missing measurements fail. Apply unchanged C1 dynamics/target limits and both
the qualified static-reference and historical matched-current tone/stereo
bounds, separately labeled. Do not modify any historical qualification.

## Comparison, not adoption

Report each new candidate against all three retained normalized drive points
and the current control at the matching source level. Also report a separately
labeled normalized-pool diagnostic: among the three existing normalized points
and this headroom candidate, prefer character-qualified target hits, otherwise
the closest character-qualified shortfall; break ties in fixed order zero,
negative, positive, headroom. If none qualify, return no eligible normalized
candidate. Require complete technical verification first; unavailable loudness
cannot win. This diagnostic does not change either existing selector or the
current-control eligibility contract. It exposes the quality/delivery tradeoff
of consistently choosing among source-normalized candidates.

## Work and stopping

Native memory <=2 GiB, 25 GiB reserve plus estimated output bytes. Record child
wall/CPU/peak memory and reference preparation separately. Reuse verified static
references and prior master measurements by exact identity. Run rendering,
character measurements and independent checks serially. Native processing
function must match the previously verified probe. Stop this matrix at twelve
files regardless of outcome. No sonic adoption, production integration, public
push, release, private-audio commit or deployment.
