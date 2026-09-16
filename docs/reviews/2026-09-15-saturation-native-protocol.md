# E native antialias-filter prototype protocol

Declared before native comparison. Production DSP/presets/gates remain unchanged.
The isolated Python experiments already identified the long 8x filter as a
fidelity reference and the shorter 8x filter as a cost comparator. Reuse their
exact coefficient tables; do not redesign filters or repeat the entire grid.

Implement a small causal native f64 polyphase interpolator, the same positive
f64 curve used in the isolated reference, and a decimating FIR. Prepare phase
tables and drive/denominator once; retain bounded per-channel ring state. Process
one frame without allocation. This is a native mechanism/cost prototype, not
production f32 DSP or an audio-device callback test.

Compare both retained 8x filters on four finite stereo signals of 32,768 frames
at 44.1 kHz: first/last-frame impulses on separate channels, a DC finite step,
997 Hz tone with unequal channel levels, and a 19 kHz burst ending at the last
frame. Keep above-full-scale amplitude 1.4 valid. Amount is 0.0715. No hidden
edge crop, gain normalization or per-chunk flushing is allowed.

Validate the complete causal output including all filter ringout against an
independently traversed SciPy upfirdn convolution using the frozen coefficients
and identical f64 positive curve. Maximum absolute difference must be <=1e-10.
Require sample-identical native output for block partitions of 1, 257 and 1,024
frames. Input N yields N+2L frames, where L=(taps-1)/8 is the combined causal
group delay in source frames. Report delay separately from signal length.

Measure phase-table preparation, processor-state construction, and processing
separately; repeat processing three times with identical fresh state. Record
per-256-frame processing duration and compare to its 44.1 kHz time interval only
as an isolated cost ratio. No OS device deadline/locked-block pass follows.
Kernel/source/output/executable hashes and failed results remain in fresh ignored
evidence. Whole-chain limiter redistribution, final output protection, selected
C comparisons, real callbacks and listening remain separate E requirements.
