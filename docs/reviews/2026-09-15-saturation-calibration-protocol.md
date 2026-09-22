# Saturation calibration: isolated mechanism protocol

Status: frozen for the first new isolated experiment; no production adoption.

The completed `sat_probe.json` and `sat_interaction_coat_t9.json` establish the
positive-amount zero-boundary jump and interaction with the limiter. Preserve
them. This experiment compares remedies; it does not repeat their questionnaire
or establish a preferred mastering sound.

## Frozen comparisons

- Control: the existing positive curve `tanh((1 + 2a)x) / tanh(1 + 2a)`, identity
  at `a = 0`. Use float64 for this isolated numerical experiment; existing native
  evidence already checks the formula. This is not a Rust callback benchmark.
- Continuity candidates: blend identity with the unchanged positive curve using
  `smoothstep(clamp(a/k, 0, 1))`, with **k = 0.005 and 0.02**. Above k the positive
  curve remains exactly the control. These thresholds are experimental choices,
  not preset migrations. Compare these without antialiasing first.
- Antialiasing candidates: retain the current positive curve at **2x, 4x, 8x**.
  Use a symmetric Kaiser-beta-10 FIR, **32 × factor + 1 taps**, cutoff
  **0.92 / factor** (normalized to the higher rate's Nyquist), unit DC sum,
  equal up/down filtering. Store exact coefficients and hashes. Isolated coherent
  signals use periodic extension and aligned delay removal; a production stream
  must separately account for finite edges, filter state and latency.
- Rates: 44.1/48/96 kHz. Tone frequencies near 997/7001/11003/19001 Hz, rounded
  to the nearest odd bin of 32,768 samples. Amplitudes -24/-6 dBFS and 1.4.
- Amounts for continuity: 0, 1e-8, 0.001, 0.005, 0.02, 0.03575, 0.0715, 0.13,
  0.25. Antialiasing uses 0.03575/0.0715/0.13. No clipping of floating input.

## Reference and reporting

Compute the transfer curve's Fourier series from one densely sampled analytic
sine period. Construct its ideal bandlimited base-rate output by retaining only
harmonics below Nyquist. Require 131,072/262,144-point coefficient convergence
within **1e-10 absolute** for the retained harmonics before using the reference.
This reference does not use the candidate SRC or its filter. Use coherent FFT
bins to separate fundamental gain, legitimate in-band harmonics and folded
nonharmonic energy. Also compare the full waveform to the analytic bandlimited
reference after fundamental-level matching; report filter attenuation separately.
Keep actual gain, harmonic distortion and alias energy separate. A lower THD
number is not itself a better master.

Report preparation/kernel cost and per-signal offline processing time, with
Python/library versions and source hashes. A coefficient table can be calculated
once per factor and reused; no per-sample filter design is justified.

## Next acceptance gates

Isolated alias reduction alone cannot authorize production adoption. Promising
methods need matched whole-chain delivery through the qualified finalizer,
limiter-work redistribution, existing fixed section/attack/tone/stereo metrics,
target feasibility, finite boundaries, exact latency and native CPU/memory.
Re-evaluate any selected C policy, including its quiet-copy cases. Freeze a new
development iteration if methods or limits change. Focused owner calibration/
listening is required for curve/preset migration. Gated Adaptive Compressor,
Phase-B confidence and album-character constants remain untouched.
