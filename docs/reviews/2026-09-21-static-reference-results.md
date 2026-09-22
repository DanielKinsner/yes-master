# Static preset reference results

The [frozen diagnostic](2026-09-21-static-reference-protocol.md) completes with
all 24 independent PCM comparisons passing and all eight quiet-reference copies
within their unchanged numerical/character budgets. This establishes a
reproducible reference component for these four fixed source/settings contexts.
It does not adopt that component as preferred sound or a shipping scoring rule.

[Scoring evidence](evidence/2026-09-21-dynamics-research/static-reference-v1.json)
and [verification](evidence/2026-09-21-dynamics-research/static-reference-verification-v1.json)
retain the mechanical checks, complete buffers, dependencies and selection audit.
No new mastering output was generated: twelve full references and twelve short
synthetic reference outputs are analysis buffers, not delivery/listening files.

## Mechanics and candidate qualification

Maximum native/SciPy PCM disagreement relative to reference peak is
**1.15638e-7**, against 1e-5. Maximum quiet-copy reference disagreement is
**3.08367e-7**, against 1e-4. All eight source-anchored metric comparisons pass;
the largest paired-attack difference is 0.000002015 dB and largest tone-band
difference is 0.0000001085 dB. Exact input PCM hashes, coefficients, source-rate
frames/stereo shape and finite outputs match. This is an explicit static
implementation, not a reclassification of the failed low-drive convergence study.

All **24 normalized-candidate quiet-copy character classifications** now match
their original-level counterpart under this same reference. Dynamics limits and
tone/stereo floor/slack remain unchanged; the reference definition changes.
Metal's original zero-offset candidate becomes eligible: its static upper-band
reference delta is 1.167058 dB, giving a 1.417058 dB allowance, versus 0.981036 dB
with the old original current-control reference. This changes the original
selection from +3 to zero; it is not proof that the newly eligible tone is preferred.

## Selection results

| Source | Original level | -40 dB copy | -80 dB copy |
| --- | --- | --- | --- |
| Piano | Single -3; -14.875 LUFS | Same | Same |
| Imaginal | Current; -21.050 LUFS | Current; -25.329 | No eligible fallback |
| Metal | Single 0; -14.000 LUFS | Same | Same |
| Rich | Current; -20.276 LUFS | Current; -22.155 | No eligible fallback |

Three of four original choices are retained. Six of eight copies match their
original choice under the new reference. Across twelve groups: three qualified
targets, seven character-qualified shortfalls and two explicit missing fallbacks.
Stable reference measurements do not make the selected current-control audio
stable: Imaginal/Rich still shift by -4.279/-1.878 LU on their -40 dB copies.
All four -80 current controls are independently unavailable for integrated LUFS;
the selector's finite-loudness eligibility requirement remains intact.

## The remaining mechanism is candidate construction

Every normalized Imaginal candidate still fails a dynamics limit. The best
lower-decile attack loss in the three-point pool is **4.132509 dB**, beyond 3.5.
Every normalized Rich candidate also fails dynamics: its smallest section
contrast loss is **2.201290 dB**, beyond 0.75. Changing only tone/stereo reference
or tie-breaking cannot make an acceptable normalized candidate exist in either
pool. Their eligible level-dependent current controls explain the remaining
delivery drift; unavailable controls explain the two missing fallbacks.

The next experiment should change how a single candidate's operating drive is
derived, rather than retune this reference or add arbitrary search offsets.
The qualified static reference provides a directly measured signal peak that
can inform a source-relative headroom candidate, with preset input push and the
known saturation small-signal scalar accounted for separately. It must be a new
frozen experiment with actual mastering/output/character verification; a static
peak prediction cannot guarantee the full nonlinear chain's output ceiling.

## Cost and limits

Native reference generation: **16.456 s wall / 15.875 s CPU / 914.2 MiB peak**,
within the 2 GiB memory budget. Independent mechanical checking takes 55.086 s;
anchored metrics and diagnostic scoring take 34.785 s. Stages ran sequentially.
The 24 reference WAVs use **1,078,067,688 bytes**. These are offline analysis
costs, not an app-wide speedup or a qualified complete preparation budget.

The release example builds, strict Clippy and Rust formatting pass, and four
focused reference regressions pass. Python compilation, artifact bindings,
internal links and whitespace pass. Production source, historical evidence,
selectors, preset settings and source audio remain unchanged. A macro-syntax
build error was corrected before the probe/job was frozen; no experimental
outcome or acceptance criterion changed. Raw buffers/jobs/logs remain ignored
under `test-output/dynamics-research-20260921/static-reference-v1/`.

Fresh production analysis invariance, manual/dense/no-target/Off/Adapt/Album
contracts, runtime integration, C2 holdout and C3/listening remain open. No
automatic sonic policy or release action is enabled.
