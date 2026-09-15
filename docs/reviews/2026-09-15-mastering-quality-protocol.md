# Mastering quality investigation: frozen pilot protocol

Written before new-corpus rendering, September 14 Pacific / September 15 UTC.
Starting checkout: `a4fb621d88a95b8af549467fb499943acb4274d5`, clean `main`,
matching remote main. All prototypes and audio stay under ignored
`test-output/mastering-quality-20260915/`. Production defaults/gates stay unchanged.

## Hypotheses, in priority order

1. Excess upstream drive causes avoidable peak suppression when final landing
   attenuates the result. Reducing drive to the minimum that meets delivery
   should retain more local/section contrast, without deleting preset stages.
2. Saturation's discontinuous zero/positive mapping and small-signal gain cause
   more shaping than the small amount suggests. A continuous unity-slope mapping
   should reduce this, but may sacrifice loudness or intended colour.
3. Lower density or stronger Adapt helps, but does not solve whole-chain drive.
   Keep their separate functions; test their interaction with drive.
4. Input-level sensitivity creates inconsistent automatic results. A bounded
   source/target-aware drive search should reduce that sensitivity.

## Corpus and split (chosen before output measurements)

Development: owner's It's a coat; Scott Buckley, In This Moment (solo piano);
Kevin MacLeod, Funkorama (bass/drums) and Metalmania (dense guitars/drums);
Admiral Bob, Baby Bird (vocal/acoustic band stems).

Holdout: Scott Buckley, Aphelion (orchestral/electronic transitions); Hans Atom
featuring Adisa McKenzie, Rich (rock/electronic vocal stems); SackJo22, Imagining
Imaginal (vocal/bowl/percussion stems). Holdout results do not tune candidate
ranges or the selection rule. If a download is unavailable, record the failure
and substitution before rendering. Gain shifts are repeated conditions, not songs.

Finished downloadable mixes are already-mastered regression material, with
undocumented mastering. Locally summed stems are **constructed unmastered test
mixes**, not claimed to be artist-approved premaster balances; lossy stems remain
labelled. The owner source's earlier processing history is unspecified. Save credits, exact
licenses, page and media URLs, original bytes/format, hashes and every transformation.

## Bounded candidates

- Control: production Universal 75, requested density 0.5, Adapt 0.5, -1 dBTP.
- Drive grid: additional trim -12 through +6 dB in 1.5 dB steps (includes zero),
  target -9 and -14 LUFS. Select the lowest drive reaching target within 0.2 LU
  after existing ceiling-bounded scalar landing. If none reaches, select the
  least target error, tie within 0.05 LU by lower drive; report infeasibility.
  Include the control as fallback; do not assume response is monotonic.
- Saturation mapping: unchanged control; zero diagnostic bypass; half positive
  coefficient; unity-small-signal-slope `tanh(k*x)/k`, k=2*amount, identity at
  zero. These are isolated research options, not adopted calibration.
- Density: 0.25, 0.5, 0.75 at Adapt 0.5; Adapt 0, 0.5, 1 at density 0.5;
  compressor Off diagnostic. Preserve all other preset coefficients.
- First test each mechanism alone. Then drive search with continuous saturation
  and with Adapt 1.0. Do not conflate compressor bypass with isolated GR removal.
- Robustness: selected development and holdout sources at 0/-6/-12 dB input,
  no input clipping. Frozen search grid may be insufficient at the quiet end;
  report that limitation rather than secretly extend the grid after selection.

## Explicit evaluation criteria

Delivery: finite samples, exact expected duration/channels, requested ceiling
(independent FFmpeg true peak, report precision), target error <=0.2 LU where
feasible. Native meter used for selection; independent meter verifies finalists.

Processing economy: pre-landing loudness/attenuation, compressor band GR,
limiter GR/activity, saturation level changes, while meeting delivery.

Preservation: whole-file and active 400 ms crest; 3 s short-term loudness spread;
fixed section contrast; RMS-matched band energies and stereo correlation/M:S.
Report changes individually. Higher crest is not a universal score. Flag lost
target, >1 LU extra section-contrast loss, >1 dB broad-band tonal change or >1 dB
M:S change against control for review, not automatic proof of audible damage.

Mechanisms: coherent sine/multitone/level sweeps, bursts, impulses, silence/tails,
sample rates 44.1/48/96 kHz; saturation zero continuity, harmonics versus folded
alias energy, limiter attack/recovery/ceiling, finite output, chunk invariance and
very-low-level behavior. Separate intended nonlinear harmonics from aliasing.

LANDR: reuse the three saved same-source masters with verified hashes. Their
-0.3 dBTP ceiling differs from YES -1.0. Gain-only playback matching; no added
limiter. No new service upload, account, purchase or current-engine claim.

## Decision and evidence

Rank options by delivery reliability, avoidable processing, robustness and
preservation, with explicit regressions and confidence. No scalar quality score,
unvalidated perceptual model, claimed listening win, or repeat A-E gate.
Record source and harness hashes, compiler/dependencies, settings, selection
tables and reproducible commands. Retain previous byte-identical reproduction
and confirm it on this checkout before relying on the extended runner.

## Exploratory follow-up, declared after the first grid

The frozen absolute trim grid reached its +6 dB boundary on quiet constructed
mixes and attenuated copies. It is therefore not a robust automatic algorithm.
Test one source-level-centred grid on coat, piano, Rich and Imagining Imaginal,
at 0/-6/-12 dB source gain: centre input trim at `-18 - measured source LUFS`,
then use the same -12..+6 dB grid and selection rule with current saturation,
density 0.5 and Adapt 0.5. -18 is a search coordinate, not a delivery target or
an adopted normalization default. Retain the production +/-24 dB input bound.
Report boundary hits and actual saturation/limiter activity. These sources are
now exploratory validation material, not a fresh second holdout. No tuning of
the formula to individual songs is allowed in this follow-up.

Mechanism discoveries also justify separate SRC probes: compare current
Rubato whole-buffer processing with the same FFT resampler using explicit delay
removal and length accounting; test 5/10/30/100/2000 ms signals and 44.1/48/96 kHz
rate pairs. This isolated correction does not change production code. Compare
the current saturation curve with 4x/8x oversampling on analytical tones; these
are alias-rejection references, not full-chain production candidates.
