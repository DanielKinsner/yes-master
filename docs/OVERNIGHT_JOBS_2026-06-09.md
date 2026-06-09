# Overnight Jobs — 2026-06-09

Context: fable-5 is available on the Max plan until **June 21**; usage resets
in the morning, so tonight's unspent quota vanishes. These four jobs are
sequenced for unattended overnight runs — each produces a reviewable artifact
(findings doc, committed tests, graph) rather than taste decisions that need
eyes. Run each in its **own session** so one failure can't take down the
others' results. Machine stays on; read results over coffee.

What deliberately does NOT run overnight: the flagship visual-polish pass.
It is taste-driven — every decision needs the owner's eyes. That is
tomorrow's work, on fresh quota, with a human in the loop.

---

## Job 1 — yes-master refactor: Phase 1 survey (run first)

Paste into a fresh session at the yes-master repo root, prepended with
`ultracode` (no budget cap tonight — quota expires at reset):

> I want a behavior-preserving refactor pass on this repo (yes-master: Tauri
> desktop mastering app + iPhone native app sharing the Rust DSP engine via a
> path dependency).
>
> **Hard constraint — the engine is locked.** The adaptive DSP engine is
> validated; its output must not change. Every refactor step must keep
> `cargo test` (src-tauri — especially `preset_signature`, `contracts.rs`,
> and the byte-identical confidence tests) and `npm test` green *unchanged*.
> If a refactor would alter any rendered output, test expectation, or the FFI
> contract the iPhone bridge depends on, don't do it — flag it as a finding.
>
> **Phase 1 — Survey, no code changes.** A knowledge graph of the repo lives
> in `graphify-out/` (query with `graphify query "<question>"`; read
> `GRAPH_REPORT.md` first). Use it for structure questions, then read actual
> source before forming any finding — treat INFERRED graph edges as leads,
> not facts. Evaluate at minimum:
> 1. The three communities the graph flagged with cohesion < 0.09 —
>    *Confidence & Deep Analysis*, *Reference Tuning & Exports*, *Native
>    Bridge Types* — real tangles worth splitting, or clustering noise?
> 2. The god nodes (`AuditionController` 53 edges,
>    `default_master_settings()` 36, `MasteringSettings` 31,
>    `settings_with_intensity()` 27) — is any one a change-bottleneck where a
>    seam or extraction would pay off?
> 3. The desktop↔iPhone seam (`apps/iphone-native/rust/src/lib.rs` →
>    `src-tauri` engine) — is the FFI surface as small as it could be, and is
>    the contract pinned by tests on both sides?
> 4. Leftover scaffolding from the May–June review cycles: dead code,
>    duplicated helpers, oversized files.
> 5. The view-mode/project-mode coordination in App.tsx — the effect-based
>    entry-guard bounce caused the silent Album→Standard trap (fixed
>    2026-06-09); evaluate consolidating view+mode into one explicit state
>    machine so illegal states are impossible by construction rather than
>    corrected by reaction.
>
> Rank findings by payoff vs. risk (risk = proximity to the locked engine).
> Write the ranked plan to `docs/reviews/2026-06-10-refactor-survey.md` and
> **stop — do not change code.** Phase 2 executes only after owner approval.

## Job 2 — ECP: adversarial audit against Spec v1.0

Repo: `Documents\GitHub\ecp` (DanielKinsner/ecp). Paste with `ultracode`:

> Adversarially audit this repo against the canonical product.md (Spec v1.0).
> Three sweeps:
> 1. **Spec conformance** — map every spec requirement to the code that
>    implements it, or flag it missing/divergent. No requirement skipped.
> 2. **Test truthfulness** — run BOTH `unittest discover` AND `pytest tests/`
>    (the unittest runner silently skips pytest-style tests — known blind
>    spot). Report real coverage, not what one runner claims.
> 3. **Tooling integrity** — the `/ecp:audit` plugin is mis-pointed at a
>    deleted pre-prune directory and runs stale code. Locate the stale
>    pointer and fix it.
>
> Output: a findings doc ranked by severity, committed to the repo. Fixes in
> tiny incremental commits on main (repo convention), each finding reviewed
> in the doc before its fix lands.

## Job 3 — yes-master: test-harden the iPhone FFI seam

The knowledge graph's #1 architectural finding: the iPhone bridge
(`apps/iphone-native/rust/src/lib.rs`) path-depends on the desktop engine —
highest-risk seam in the repo. Test generation is ideal unattended work
because the agent verifies its own output by running the suite. Paste with
`ultracode`:

> The iPhone bridge (apps/iphone-native/rust/src/lib.rs) path-depends on the
> desktop engine (src-tauri). Audit the FFI contract test coverage on both
> sides — every exported `yes_master_native_*` function, every error path,
> and the Swift-side parity tests (SupportedExtensions, NativeLoudness,
> NativeMasteringBridge). Write the missing contract tests.
>
> **Hard constraint: the engine is locked** — tests assert CURRENT behavior,
> never change DSP code. Every test must pass before its commit. Feature
> branch (`feature/ffi-contract-tests`), tiny commits.

## Job 4 — graphify ECP + merge hygiene (cheap; run last or while watching Job 1)

1. In the ECP repo: `/graphify "C:\Users\SM - Dan\Documents\GitHub\ecp"` —
   build ECP's knowledge graph, commit it the same way as yes-master's
   (track `graph.json`, `cache/`, `manifest.json`, labels, report; gitignore
   `.graphify_python`, `.graphify_root`, `graph.html`).
2. yes-master: `feature/graphify-knowledge-graph` branch is still unmerged —
   merge it to main once tonight's work settles.
3. After all merges: `/graphify --update` in yes-master and commit the
   refreshed `graphify-out/` so the map matches main.

---

## Morning checklist

- [ ] Read `docs/reviews/2026-06-10-refactor-survey.md` → approve/trim Phase 2 scope
- [ ] Read ECP findings doc → verify the `/ecp:audit` plugin fix
- [ ] Review `feature/ffi-contract-tests` branch → merge if green
- [ ] Confirm both repos' graphs are committed and current
- [ ] Fresh quota: start the flagship visual-polish pass (eyes-on work)
