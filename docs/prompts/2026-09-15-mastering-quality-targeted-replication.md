# Targeted replication and reconciliation of the mastering findings

The independent phase is complete. This is an explicitly **unblinded follow-up**.
I authorize you to read the main YES Master checkout, Codex's research and its
retained evidence. This current user instruction supersedes the previous
restriction against accessing those materials. It does not authorize production
changes or changes to your sealed independent report.

Your task is to test Codex's specific claims, reconcile them with your own
findings, and recommend the best-supported next engineering work. Agreement is
not the goal: confirm, narrow, refute or leave each claim unresolved according
to the evidence. A failure to discover something in the blind phase is not a
failed replication unless you exercised the relevant conditions.

## Workspaces and references

Main checkout:
`C:/Users/Daniel Kinsner/OneDrive/Documents/GitHub/yes-master`

Your review workspace:
`C:/Users/Daniel Kinsner/OneDrive/Documents/GitHub/yes-master-independent-audit-20260915`

Read these in order:

1. Your `audit-output/INDEPENDENT_REPORT.md`, `PROTOCOL.md`,
   `SEALED_MANIFEST.json` and `EVIDENCE_MANIFEST.json`. Verify the seal before
   starting; preserve the originals byte for byte.
2. Main checkout `AGENTS.md`, then
   `docs/reviews/2026-09-15-mastering-quality-recommendation.md` and
   `docs/reviews/2026-09-15-mastering-quality-protocol.md`.
3. The relevant compact data under
   `docs/reviews/evidence/2026-09-15-mastering-quality/` and the reproduction
   package `scripts/research/mastering-quality-20260915/README.md`.
4. For a specific claim, its retained native files/scripts under
   `test-output/mastering-quality-20260915/`. Do not write into that directory
   or overwrite any previous experiment output. Earlier owner reproduction
   details are in `docs/reviews/2026-09-14-dynamics-stage-investigation.md`.

Both investigations started with production revision
`a4fb621d88a95b8af549467fb499943acb4274d5`. Inspect current source differences
instead of assuming current main is still identical. Preserve unrelated work.
Keep the original baseline available for replication; report later-build results
separately if the application has changed.

Fixture mapping (verify the file hashes before relying on it):

| Your input | Codex ID | Piece |
| --- | --- | --- |
| input01 | coat | It's a coat |
| input02 | piano | In This Moment |
| input03 | aphelion | Aphelion |
| input04 | funk | Funkorama |
| input05 | metal | Metalmania |
| input06 | baby | Baby Bird |
| input07 | rich | Rich |
| input08 | imaginal | Imagining Imaginal |

Codex's main matrix uses Universal **75%**, Density 50%, Adapt 50%, -1 dBTP,
targets -9 and -14 LUFS, source-rate processing and 48 kHz PCM24 delivery.
Your main matrix often uses Universal **50%**. Codex also retained an Intensity
50 screen. Match actual settings, source profile, rate, PCM bytes and measurement
method before declaring a disagreement. Baseline byte identity validates that
case; it does not automatically validate every later harness variation.

## Claims to test directly

### 1. SRC buffer handling: a separate issue from ordinary peak regrowth

Codex reports a defect in Rubato 1.0.1's `process_all_into_buffer` helper:
the delay-trimming copy can retain the wrong number of useful samples. In the
48→44.1 kHz case, a 1911-frame output block and 955-frame delay leave 956 useful
frames; output frame 955 is allegedly wrong. A 100 Hz, 0.5-peak sine reportedly
shows about 0.43 amplitude error. Very short conversions can retain startup
delay and become nearly silent; floating frame-count arithmetic can add a frame.

Evidence: `src-checks.json`, the retained `src-check/` and `src-check-integer/`
directories, and the isolated prototype `src/src_fixed.rs` in Codex's package.
The tested matrix covers 5/10/30/100/2000 ms inputs, three frequencies and
44.1→48, 48→44.1 and 96→48 kHz rate pairs.

Reproduce with your own minimal harness and independently calculated expected
length/content. Inspect the actual locked dependency and call site. Distinguish
this alleged indexing/delay defect from legitimate filter-dependent peak
regrowth. Test the isolated correction against the same original converter,
including boundaries; challenge both implementations. Explain whether your
earlier format/alignment pass covered these particular conditions.

### 2. Delivered ceiling, measurement disagreement and skipped landing

Codex reports 25/192 pilot deliveries failing its predeclared FFmpeg criterion
of <=-0.9 dBTP for a -1 request: all 24 Imaginal variants and Funkorama's
`continuous_drive_t14`. It also reports discrepancies supported by soxr
reconstruction, with native-meter feed-block size ruled out in those cases.

Evidence: `music-results.csv`, `peak-verification.json`, `src-regrowth.json`,
the named retained finalist WAVs and associated native/FFmpeg logs.

Your report also finds a Custom/no-loudness-target ceiling failure and a short
input whose landing is skipped. Reproduce those paths and inspect the actual
production export path, not just a diagnostic approximation. Check both the
pre-quantization samples and the file the user would receive.

On byte-identical files, compare the engine meter, your meter, FFmpeg and a
documented reconstruction reference. Resolve differing 16x results by examining
filter response, precision, rates, alignment and sample handling; oversampling
factor alone does not make a meter ground truth. Support any standards-compliance
claim with the applicable primary specification and its actual tolerances.

Test your proposed 0.5 dB margin against the complete relevant failing set,
including the synthetic cases you described. Distinguish a demonstrated
correction from an empirical margin that passed a limited set. Keep filter
regrowth, estimator differences, skipped ceiling enforcement and buffer errors
separate when explaining causes and proposed fixes.

### 3. Automatic drive: compare the two proposed policies fairly

Codex recommends a bounded source/target-aware search. Its exploratory version
centres input trim at `-18 - measured source LUFS`, then tests offsets from
-12 to +6 dB in 1.5 dB steps while retaining the production +/-24 dB clamp.
It selects the least drive reaching the target within 0.2 LU, otherwise the
smallest target error, with the documented tie rule. -18 is a tested search
coordinate, not an adopted normalization setting.

It reports all 12 -14 LUFS conditions for coat, piano, Rich and Imaginal at
0/-6/-12 dB source gain passing target and independent ceiling checks, with
per-song crest spread below 0.013 dB. At -9, gain bounds, dynamics costs and
peak failures remain. Evidence: `centered-validation.json`, `robustness.json`
and the corresponding retained jobs/settings/files.

Compare current production, your attenuate-only rule, and Codex's bounded
source-centred rule on matched conditions at both targets. Include the quiet
copies and the piano/sparse-vocal regressions. Verify your reported synthetic
target regression and any proposed damping/iteration; do not call an untested
adjustment a resolved regression.

Assess achieved loudness, delivered peaks, section/short-term contrast, attacks,
tonal/stereo changes, processing activity and runtime. More crest alone is not
a winner. Reused sources are reconciliation material, not fresh holdouts.

The owner wants dependable decisions **behind the existing interface**. Do not
default to adding a new knob or mode to avoid the engineering decision. Explain
whether the already-selected loudness target expresses the relevant user intent,
what processing limits are justified, and when an honest target-missed result
should remain. Recommend a policy with explicit benefits and costs; keep it
experimental in this task.

### 4. Saturation mapping, aliasing and interaction with limiting

Codex reports that the current positive mapping
`tanh(x*(1+2a))/tanh(1+2a)` is discontinuous with the bypass at zero amount.
Its -24 dBFS/1 kHz probe gives roughly +2.357 dB fundamental gain for an
almost-zero positive amount and +2.922 dB at Universal 75's amount 0.0715.
Halving the coefficient therefore does not halve the shaping.

Verify the actual mapping and boundary behavior. Compare native spectra with
the analytical curve; separate intended harmonics from folded components.
Check the retained 4x/8x reference's filters and assumptions. These are
mechanism measurements, not a claimed perceptual improvement of that many dB.

The gentle continuous prototype `tanh(2a*x)/(2a)` reportedly reduces shaping
but, at matched -9 delivery on coat, raises limiter maximum/mean reduction to
6.60/0.97 dB and lowers LRA to 2.6 versus control 3.1. Reproduce or challenge
the interaction. Do not recommend a new curve solely from isolated THD/alias
numbers or adopt saturation bypass as a production feature.

### 5. Density, Adapt and the claims to keep current behavior

Verify whether the measurements support retaining their separate functions
and current defaults while coordinating whole-chain drive. Distinguish
requested density, source-resolved processing, compressor makeup and measured
gain reduction. Compressor Off also removes crossover/makeup effects; its
output difference is not pure compression attribution.

Reconcile any broad "compressor read-out passes" claim with the adapted case:
matching the unadapted UI formula alone does not establish that a source-adapted
effective summary is accurate. Keep this a scoped correctness assessment,
not a new UI design project.

## Execution and deliverable

Work autonomously on focused replication. Reuse preserved music, binaries and
logs where their hashes/settings match; make a fresh isolated directory for
new tests. Do not repeat every matrix when a smaller causal test answers the
question. Record failures and competing explanations. Distinguish measured
changes from listening judgments and installed-app or real-time claims.

Keep production source/defaults/calibration gates unchanged. No new UI features,
deployment, release, public push, spending, third-party audio upload or deferred
refactor. Do not reopen the owner's prior listening questionnaire.

Write new work under `audit-output/reconciliation-20260915/` (choose a fresh
suffix if it already exists). Produce `RECONCILIATION.md` with a table:

**Claim | Exact matched test | Result | Confirmed/narrowed/refuted/unresolved |
Cause of any disagreement | Recommended action | Confidence/remaining limit**.

Finish with a ranked recommendation, explicit regressions and the smallest
appropriate production follow-up slices. Mark any conclusion changed after
unblinding. Seal the new report/scripts/evidence manifest separately. Preserve
the original 18 sealed files and all unique original evidence.

Finally, provide a cleanup inventory: reproducible build caches, duplicate inputs,
unique evidence and the minimum archive needed for another machine. Do not delete
the review workspace or its evidence in this task. Cleanup follows verified
preservation and completion of the reconciliation.
