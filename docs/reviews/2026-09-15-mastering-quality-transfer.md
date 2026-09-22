# Restore tonight's mastering research on another machine

The [final synthesis](2026-09-15-mastering-quality-final-synthesis.md) and
[fresh-agent prompt](../prompts/2026-09-15-mastering-quality-planning-handoff.md)
are the starting points. All conclusions and small evidence records travel with
Git. Audio is in the existing **private research draft**, not public Git blobs.
The owner authorized this transport and the push to main. Do not publish the draft.

## What a pull provides

- Final synthesis, corrections, ranked implementation proposals and acceptance
  boundaries; the original research, comparison prompts and live status.
- Codex's portable experiment source, locked dependencies and expected source/
  corpus hashes, plus the original compact CSV/JSON evidence and plots.
- 511 byte-preserved Claude text/source/settings/measurement files, including
  both reports, both seals, raw results and locked native harnesses.
- Final reference-check scripts, safe restore tool, supplement manifest and
  recorded verification. Scoped `.gitattributes` preserve sealed bytes.

A clone does **not** contain audio. Reading/planning can begin immediately;
rendering/verification needs the archives below. Source URLs/credits remain
available, but restoring the frozen corpus avoids depending on changed hosts.

## Two complementary archives

Both use repository `DanielKinsner/yes-master`, draft tag
`research-transfer-2026-09-14`. Authenticate `gh` as the owner or an authorized
collaborator. Draft assets are not public downloads. Use the stable tag with
`gh release`, not a transient draft URL. No new release or publication is needed.

### Original September 14 owner/LANDR archive

`YES-Master-private-research-2026-09-14.zip`: **1,834,667,666 bytes**, SHA-256
`a76a3ae0a95751df0c91e747a363c776ad073cb5ece22bfb54acd6de7e3b8c53`.
It holds 147 owner/service/earlier-reproduction files and its own restore helper.
The previous download-back/full-restore verification is recorded in the
[original transfer guide](2026-09-14-research-handoff.md#restore).
That archive is preserved unchanged. Follow its restore command for that ZIP;
it has a different layout from the supplement.

### September 15 expanded corpus and independent-review supplement

Three content-addressed ZIPs contain **3,543 unique files**, restoring **4,507
paths** across the main repository's ignored research directories and the
independent sibling workspace. Identical files are stored once in the archive,
then restored to their expected paths. There are 3,579,372,787 unique uncompressed
bytes, approximately 6.0 GB when restoring all duplicate paths. Compressed total:
3,045,172,401 bytes. Each part is independently hash-verifiable.

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| `YES-Master-research-2026-09-15-part01.zip` | 1,477,454,344 | `eeba06f1dae01ddb5336ebd18b018d10376da129f91c6ee081747ecd00ff72c5` |
| `YES-Master-research-2026-09-15-part02.zip` | 1,331,471,156 | `cfee18ecdc2ab52d1c7b4f51c176df3871c8cc1b2b4db299f5e1e08b44575a20` |
| `YES-Master-research-2026-09-15-part03.zip` | 236,246,901 | `26e005800c55d7ca3e8596826f95434483a31061af4c1990a762acf2d7d16cbd` |

The [supplement manifest](evidence/2026-09-15-final-review/RESEARCH_20260915_MANIFEST.json)
maps every object hash to its restored path. It is also a draft asset named
`RESEARCH_20260915_MANIFEST.json`. The Git copy is the expected manifest; compare
its hash with a downloaded copy if using the latter.

Included: all eight review inputs and owner baseline; expanded original downloads,
stems and prepared source WAVs; canonical 452-file independent source snapshot;
all small experiment records; both locked review harnesses; retained small
synthetics and SRC probes; all 25 re-metered pilot failures; selected control and
comparison WAVs; regenerated no-target, short-file and pre/post-SRC witnesses.
The Windows review executables are retained privately as optional historical
artifacts; rebuild from source for another platform or when checking provenance.

Excluded: multi-gigabyte build caches and bulk derivable music renders, most
gain/rate copies, redundant partial-download chunks and downloaded standards PDFs.
Settings/scripts and original result/deletion hashes preserve the reconstruction
route. **This is a sufficient selected reproduction bundle, not an image of every
local directory.** Claude had already pruned its own intermediates. The final
pass did not delete any existing files or the independent workspace.

## Download and restore the supplement

From the receiving main checkout, after reviewing/preserving local changes and
updating it safely from main. These PowerShell commands use the receiving path;
they do not assume the original machine's username or location.

```powershell
$repoPath = (Get-Location).Path
$auditPath = Join-Path (Split-Path $repoPath -Parent) 'yes-master-independent-audit-20260915'
$downloadPath = Join-Path $repoPath 'test-output/research-transfer-download-20260915'
gh release view research-transfer-2026-09-14 --repo DanielKinsner/yes-master --json isDraft,assets
gh release download research-transfer-2026-09-14 --repo DanielKinsner/yes-master --pattern 'YES-Master-research-2026-09-15-part*.zip' --dir $downloadPath
python scripts/research/final-review-20260915/restore.py --download-dir $downloadPath --manifest docs/reviews/evidence/2026-09-15-final-review/RESEARCH_20260915_MANIFEST.json --repo $repoPath --audit $auditPath --proof test-output/supplement-restore-proof.json
```

Use `--verify-only` to stream-check all archives/members without writing the
restored tree. Normal restore checks archive/member hashes, validates target
paths and rejects overwriting different existing files. Same-hash existing files
are accepted. If an existing independent workspace differs, choose fresh roots;
do not erase it. Leave roughly 10 GB for supplement download/restore, plus the
original archive and working/build space. A complete new rendering grid needs
considerably more space. Standard-library Python suffices to restore.

On macOS/Linux the same Python command works with explicit native `--repo`,
`--audit` and `--download-dir` paths. `gh` syntax is unchanged. The historical
whole-WAV reproduction is verified on Windows; cross-platform exact identity is
not established. Diagnose discrepancies before using new measurements.

## Build and verify, without altering historical evidence

Codex's [reproduction README](../../scripts/research/mastering-quality-20260915/README.md)
gives a fresh-directory setup, source-hash checks, analysis requirements, locked
build and On/Off/257-frame identity checks. Use that package for new experiments,
not the already populated archived evidence directory.

To rebuild the restored Claude harness on Windows:

```powershell
cargo build --release --locked --manifest-path "$auditPath/audit-output/harness/Cargo.toml" --target-dir "$auditPath/audit-output/build/target"
cargo build --release --locked --manifest-path "$auditPath/audit-output/reconciliation-20260915/harness/Cargo.toml" --target-dir "$auditPath/audit-output/reconciliation-20260915/build/target"
python scripts/research/final-review-20260915/verify.py
```

The final verifier requires Python NumPy 2.2.6, SciPy 1.17.1, SoundFile 0.12.1,
and FFmpeg 7.1.1 with SOXR for matched reference results. Original Rust is 1.95.0
with each checked-in Cargo lockfile. Install analysis packages in an isolated
environment as described by the reproduction package. The verifier resolves
historical paths against restored roots, uses an OS-appropriate executable name,
and writes to `test-output/mastering-quality-final-review-20260915/`. It refuses
overwriting a completed verification JSON; select a new output directory in a
working copy if rerunning changed checks. The archive restores its critical
witnesses, and the verifier checks the owner hash before reusing them.

Other original Claude scripts retain historical absolute paths. Copy them into
a new experiment and adapt paths there; do not edit sealed originals. In
particular, do not run the archived pruning scripts as part of restoration.

## Verification record

See the [final verification addendum](2026-09-15-mastering-quality-final-verification.md)
and the adjacent machine-readable upload/download/restore records. They distinguish
archive integrity, fresh render identity, measurement checks and unperformed
application/platform checks. Keeping the draft private is part of the transport
contract; publication and cleanup remain separate actions.
