# Fresh source analysis contract: development audit v1

Frozen before fresh analysis of gain-copy files. The current headroom matrix
retains the original source profile/guards. Its numerical consistency cannot
establish consistency when the application freshly analyzes a quiet source.
This audit addresses that separate dependency while preserving every prior run.

Use exactly Piano/Imaginal/Metal/Rich at source gain 0/-40/-80 dB. Analyze each
actual copy and its peak-normalized counterpart through the unchanged complete
production `analysis.rs::analyze_one` body with deep analysis enabled: 24 analyses.
The original four files are reused, and twenty derived float source WAVs are
kept locally. These are analysis inputs, not masters. Decode source PCM and
multiply/normalize in f64 with a single f32 cast per transform, as before.
Bind normalized input PCM hashes to the already-qualified static-reference
inputs. Include the production analysis source directly in an offline example;
do not edit shipping analysis or substitute a simplified measurement pipeline.

Record actual native integrated availability separately from the analysis API's
sanitized LUFS value. Record the complete analysis result, deep presence/window
count, derived SourceProfile, and Universal 50 coefficients after replacing only
the retained source profile and source LUFS with the fresh facts. Preserve all
other requested settings, confidence/calibration gates and user controls.
This is coefficient resolution only; do not render another master or infer
audible equivalence from coefficient similarity.

Compare each analysis against the same source's original-level analysis in its
own domain (actual versus normalized). Require matching profile presence and
shape; diagnostic numeric tolerances are 0.01 for profile DR dB/LRA LU and
0.00001 for spectral shares, correlation and stereo width. Missing/nonfinite
values are explicit. Record coefficient debug identity, structure identity and
maximum absolute numeric difference separately; do not convert a changed
coefficient string into an audio-equivalence claim. Also compare normalized
original analysis with actual original analysis to expose normalization's own
effect. These are diagnostic sensitivity bounds, not sonic adoption criteria.

Keep normalized-source availability separate from actual-file loudness. In
particular, an available normalized LUFS result does not make a below-gate actual
source available. No change to UI sentinels, source profile derivation, cache
identity, manual controls, Density/Adapt, Album, or shipping selection is adopted.

Run after the headroom independent checks, serially. Record total child
wall/CPU/peak memory and per-analysis work. Require <=2 GiB native peak memory
and 25 GiB free reserve plus derived-file estimate before starting. Preserve
raw input files, reports, logs, binary and hashes. Stop at 24 analyses. No new
presets, targets, holdout, sonic retune, public push, release or private-audio commit.
