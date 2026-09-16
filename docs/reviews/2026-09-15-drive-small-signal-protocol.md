# C1 diagnostic: preset response at very low operating drive

Declared before this diagnostic. The whole-chain E experiment retains Piano's
section-contrast failure and Imaginal's attack failures even when saturation
aliasing changes. A source-versus-master metric can also include intentional EQ,
crossover, stereo or other preset response. Determine whether the proposed
character limits distinguish that response from level-dependent processing.
This is a diagnostic, not an expanded candidate grid or changed acceptance rule.

Reuse the four matched cases/settings from the completed E whole-chain job:
Coat/Piano/Imaginal single candidates at -14 and Coat current processing at -9.
Retain the exact source normalization, base drive and requested coefficients.
Render the unchanged current production chain at additional **0, -24 and -48 dB**
input attenuation, compensating that additional factor after the chain and
before SRC/qualified finalization. No coefficient, compressor mode, curve,
Density or Adapt override is allowed. Record actual limiter and compressor gain
reduction. The factor compensation makes raw-LUFS gating an avoidable diagnostic
confound; it does not become a proposed production processing stage.

The zero-offset delivered PCM must match the retained E current control exactly.
Reuse its existing WAV on success. Preserve both new complete 48 kHz float outputs
per case and independently verify peak/LUFS and frame counts. Use the same frozen
source section/attack anchors and report source deltas, differences versus the
current control, and differences between the two low-drive outputs individually.
Do not change the C1 limits or retroactively relabel a failed selected candidate.

Only call the tested low-drive responses numerically converged if, after removal
of final scalar landing, their maximum absolute sample difference is **<=1e-5**.
Report an unavailable or failed comparison explicitly. Tiny limiter reduction
alone does not prove linearity of other stages, and a converged low-drive limit
does not prove preferred sound. If convergence fails, retain it and investigate
before using the limit as a reference.

Separate useful information from acceptance: a stable preset contribution may
explain a source-metric failure, but it does not authorize weakening a sonic
limit, choosing a fallback, or adopting a global score. Any revised candidate
rule needs its own frozen development protocol and subsequent unseen holdout.
The automatic-dynamics owner choice remains separate; continue independent work.
