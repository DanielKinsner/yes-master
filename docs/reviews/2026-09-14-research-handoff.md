# September 14 research: continue on another machine

The live browser demo is on `main`. The pricing, DSP and control-design documents
are decision material, not approved changes to prices, presets or controls.

## What travels with Git

Pull `main` to get the research, recorded decisions and proposals:

- [Dynamics stage investigation](2026-09-14-dynamics-stage-investigation.md):
  exact reproduction of both owner exports; 18 diagnostic renders; saturation,
  limiting, density/adaptation and input-drive findings.
- [YES export versus LANDR](2026-09-14-yes-export-vs-landr.md): original/competitor
  comparisons and proposed Density/Adapt presentation.
- [LANDR/Waves assessment](2026-09-14-landr-waves-dsp-assessment.md): documented
  vendor behavior versus proprietary unknowns.
- [Pricing assessment](2026-09-14-pricing-assessment.md): proposed price range,
  browser/software business model and desktop/mobile bundle discussion.
- [Software simplification proposals](../plans/2026-09-14-dsp-software-simplification-proposal.md):
  a separate structural review; desktop simplification remains deferred.
- [Native dynamics measurements](evidence/2026-09-14-dynamics.csv),
  [independent original/YES/LANDR measurements](evidence/2026-09-14-yes-landr.csv),
  [independent owner compressor pair](evidence/2026-09-14-compressor-pair.csv).

The compact CSVs contain measurements and hashes, with no audio or machine-local
absolute paths. The full local evidence and playback files are in the archive
below. A Git commit records notes; it does not approve the recommendations.

## Audio and complete evidence on GitHub

The owner explicitly authorized using GitHub to transport these files. They are
stored as assets of the **Research transfer — September 14, 2026** draft, separate
from the application beta draft. Sign in with the repository-owner account or an
authorized collaborator account to access this draft:

[Open GitHub Releases and select the research transfer draft](https://github.com/DanielKinsner/yes-master/releases)

The Releases list is used here because GitHub can change a draft's temporary
`untagged-*` URL when its target changes. The CLI tag identifier below is stable.

Release tag identifier: `research-transfer-2026-09-14`.
Archive: `YES-Master-private-research-2026-09-14.zip`.
Archive size: **1,834,667,666 bytes** (about 1.83 GB).
SHA-256: `a76a3ae0a95751df0c91e747a363c776ad073cb5ece22bfb54acd6de7e3b8c53`.

The archive contains **147 files**, including all **11 files from
`tests for presets/`**, 18 full diagnostic renders, five matched-volume clips,
measurements, plots, settings, hashes and the diagnostic scripts/Rust harness.
It excludes build caches and unrelated production material. Keeping the archive
as an attachment avoids adding its size to ordinary code clones. The input and
generated-audio folders remain Git-ignored; a normal `git pull` will not populate
them by itself.

### Restore

1. Preserve any existing work on the other machine, then switch to `main` and
   update it with `git pull --ff-only`.
2. Download the ZIP and its `.sha256` companion from the research draft. If using
   an authenticated GitHub CLI, from a checkout root:

   ```sh
   gh release download research-transfer-2026-09-14 --repo DanielKinsner/yes-master --pattern "YES-Master-private-research-2026-09-14.zip*" --dir test-output/research-transfer-download
   ```

3. Extract into a separate temporary folder, **not over the checkout**. From the
   extracted folder, run with Python 3.9 or later:

   ```sh
   python RESTORE_PRIVATE_RESEARCH.py "PATH/TO/yes-master"
   python RESTORE_PRIVATE_RESEARCH.py "PATH/TO/yes-master" --verify-only
   ```

   Use `python3` on macOS if needed. The helper checks every payload hash before
   copying, skips identical existing files, and refuses different existing files.
   Verification should report **147 verified files**. It copies only missing
   files, preserving the relative paths expected by the reports.

4. Open `test-output/yes-stage-ablation-20260914/listening/README.md`, then listen
   to A through E before opening `reveal-key.json`. All five excerpts measured
   -16.0 LUFS and cover the same 195–215 second passage. No by-ear winner has
   been selected. Different source material is still needed before generalizing
   a preferred processing change.

Historical logs/JSON retain the original machine's absolute paths as provenance.
Use archive-manifest relative paths and the report links on the new machine.
The files can be played and the recorded results discussed without rebuilding.
Fresh Rust renders require the toolchain; analysis scripts additionally require
FFmpeg, NumPy, SciPy and Matplotlib. Do not treat a new Mac build as already
validated by the Windows measurements.

## Resume point

The next discussion concerns the measured results and listening preferences.
Do not implement a preset retune, combine controls, activate calibration gates,
adopt a price, or resume the deferred desktop refactor merely because this packet
is present. The latest state remains in [Open Threads](../OPEN_THREADS_AND_DECISIONS.md).
No repeat owner export is needed to recover the completed On/Off comparison.
