# Claude Fable review archive

Original files copied byte for byte from the independent review workspace.
`ARCHIVED_FILES.json` inventories 511 text/source/settings/measurement files.
The original reports are historical, including their mistakes. Read the
[final synthesis](../../2026-09-15-mastering-quality-final-synthesis.md) and
[verification corrections](../../2026-09-15-mastering-quality-final-verification.md)
before treating any recommendation as current.

## Read in this order

1. [Sealed blind protocol](audit-output/PROTOCOL.md).
2. [Sealed blind report](audit-output/INDEPENDENT_REPORT.md).
3. [Separate unblinded reconciliation](audit-output/reconciliation-20260915/RECONCILIATION.md).
4. [Blind seal](audit-output/SEALED_MANIFEST.json) and
   [reconciliation seal](audit-output/reconciliation-20260915/SEALED_MANIFEST.json).
5. Detailed `results/`, `cases/`, `specs/`, `scripts/` and locked `harness/` in
   each phase. The original evidence manifests identify retained/pruned audio.

The 18 blind and 20 reconciliation sealed files were hash-verified in the final
pass. A Git attribute disables text conversion here so seals survive a clone.
Archive paths preserve the original `audit-output/` layout. The private
[transfer supplement](../../2026-09-15-mastering-quality-transfer.md) restores
the matching source snapshot, fixtures, small probes and selected WAVs.

Historical scripts can contain the original machine's absolute paths and some
write summaries or delete intermediates. **Do not run these in the sealed
archive or over retained evidence.** Keep this archive unchanged, restore the
private snapshot, then copy a harness/experiment into a new working directory
and adapt paths there. These scripts are evidence, not application dependencies.
The final-review reproduction tool safely regenerates the five key pruned
witnesses into a separate directory; it does not run the pruning scripts.

The preparation only blinded the first phase to prior conclusions. The second
phase read Codex's evidence. Neither is a formal double-blind listening trial.
