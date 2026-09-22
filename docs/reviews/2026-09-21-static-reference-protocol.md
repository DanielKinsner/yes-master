# Static preset reference: development diagnostic v1

Frozen from `d9b6f1f8` before new reference outcomes. This follows the completed
availability diagnostic. User requested continued local research. No shipping
selector, source/preset intent, historical result or character limit is changed.

## Reference definition

Construct an explicit static reference from the original Universal 50 settings
for Piano, Imaginal, Metal and Rich. Verify the same source hashes, production
numeric settings and baseline coefficient identities as the prior bidirectional
matrix. Preserve source analysis/guards and source-rate coefficients.

Retain the two subsonic HP stages, seven EQ bands, warmth/presence filters,
the active compressor's exact LR4 branches, fixed per-band makeup and their
recombination, then the existing guarded M/S width transform. Omit time-varying
compressor gain and transient shaping, nonlinear saturation and limiter from
this reference only. Input/output gain and saturation's small-signal scalar
are global gains and are omitted; source-relative scoring removes global gain.
This is a defined static component of the preset, not an assertion that the
complete preset becomes this response at low drive or that it is a preferred master.

Use existing public production BiquadCoeffs/BiquadState with independent offline
states. Never change normal DSP, compressor Off behavior or preset parameters.
Serialize exact f32 coefficients and stage topology for independent checking.

## Bounded matrix and mechanical checks

For each of four sources, render reference-only PCM from peak-normalized
0/-40/-80 dB copies. Transform f64 gain -> f32 copy -> f64 peak division -> f32.
Keep source-rate frames and stereo channels exact. Twelve whole-song reference
buffers maximum, plus twelve short synthetic outputs (three per coefficient set).
These float WAVs are analysis buffers, not delivered masters: values above full
scale are permitted, finite values are required, and no playback/export claim
or delivery-ceiling qualification is attached to them.

Each coefficient set first processes three 65,536-frame synthetic signals:
asymmetric stereo impulses; multitone stereo (70/1000/9000 Hz); and deterministic
LCG noise. The source-rate fixture definitions and hashes are frozen in the job.
Run an independent SciPy double-state lfilter implementation with f32 rounding
at the same stage boundaries. Require maximum absolute PCM disagreement divided
by max(reference peak, 1e-12) <=1e-5 on all 24 outputs, exact frames/rate/channels,
finite samples and exact file hashes. Unknown/missing evidence fails.

For the eight attenuated whole-song references, require maximum PCM difference
against their matching original reference <=1e-4 after division by the original
reference peak. Retain the previous character-copy budgets: section, paired
attack, tone-band and side/mid differences <=0.01 dB, correlation <=0.00001.
All required reference comparisons must pass before diagnostic rescoring.

## Diagnostic rescoring, after qualification only

Reuse the sixteen original and thirty-two quiet-copy mastering outputs without
rerendering. Bind existing measurements, source anchors and complete-file checks;
use the additive complete-block availability evidence for quiet-copy technical
eligibility while preserving the old failed report. Calculate each static
reference's source-relative tone/stereo deltas. Substitute that reference into
the existing control-relative bound formula without changing its floor/slack.
Keep source-anchored dynamics and target tolerances unchanged. Label all such
rows `diagnostic-static-reference-v1`; do not rewrite historical qualifications.

Run the existing bounded bidirectional ranking with the new diagnostic failures
and explicit technical eligibility. Compare the twelve selections, including
policy/offset identity, character qualification, target feasibility and selected
LUFS. Score copy identity against the original under this same new reference,
not against a different old rule. Also compare original choices with historical
choices to expose changed policy. Current processing remains level dependent
and eligible under the same rules; reference stability does not prove selected
audio stability or solve unavailable-current fallback.

## Cost and stopping

25 GiB free reserve plus estimated reference bytes; native-memory budget 2 GiB.
Build/copy the offline executable and freeze job/protocol/inputs before execution.
Run native reference generation, independent mechanical checks, then source-
anchored metrics/rescoring serially. Record actual work and storage. No new
mastering outputs, drive points, source analysis, holdout or broader presets.
If mechanics fail, preserve the witness and resolve that issue before scoring.
If scoring remains inconsistent, retain that result and identify the remaining
mechanism; do not loosen bounds or enlarge the grid to obtain a pass.
