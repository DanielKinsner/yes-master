# Evaluation of the Codex Refactor Survey — 2026-06-10

Evaluates `docs/reviews/codex-2026-06-10-refactor-survey-.md` (the independent
Codex Phase 1 survey), and adjudicates the disputes raised in
`docs/reviews/codex-2026-06-10-refactor-survey-review-review.md` (Codex's
review of the Claude survey, `2026-06-10-refactor-survey.md`).

**Method.** Every evidence citation in the Codex survey was re-opened against
the working tree (HEAD `14f9010`): file:line claims, doc quotes, commit
references. Codex's churn/co-change table was independently reproduced from
`git log -n 200 --name-only`. Each of the six findings got an adversarial
verification pass. Where the two surveys disagree, the disagreement is
adjudicated with source evidence, including against the Claude survey — two
of its claims are conceded below.

---

## Executive Verdict

**Codex's survey is citation-accurate and right on the marquee call, but
incomplete.** Every file:line reference reproduces within 1–5 lines (typical
error: end-of-range undershoot). The churn table reproduces exactly (App.tsx
32 vs 33 is window skew from three doc-only commits — not an error). The
graphify-cache filtering was the right methodology call.

Scorecard for the six Codex findings:

| # | Codex finding | Verdict |
| --- | --- | --- |
| 1 | Navigation state machine | **Validated** — independently converges with Claude Finding 1; state set undercounted (corrections below) |
| 2 | Evidence-lane adaptive resolver | **Validated** — real, near-identical duplication; complements (does not replace) the six mechanical helpers; merge both |
| 3 | useTrackMaster deepening (L) | **Disputed as scoped** — the cited defects would not have been prevented by the split; re-scope to a small pure-parts slice |
| 4 | iPhone bridge internal split | **Disputed** — citations accurate, proposal misaligned with the documented pain; decline in favor of contract pins |
| 5 | Display-mirror parity | **Validated** — and credit: catches the SignalChain width mirror the Claude survey missed; split tripwire tests from module consolidation |
| 6 | Untrack graphify cache | **Core validated, reframed** — technically right, but it relitigates a deliberate owner decision and its churn evidence is a bulk-add artifact; needs owner sign-off |

Three structural omissions in the Codex survey, each verified:

1. **It never noticed `cargo test` does not compile on main.** Grep of the
   survey for `E0063` / `measurements_are_rendered` / `compile` returns zero
   matches, yet its own verification plans for findings 2 and 4 invoke
   `cargo test --target-dir target\codex-rc` — a command that currently fails
   with 8 E0063 errors at `tests/contracts.rs:592,632,655,679,714,748,1785,
   1817`. (Codex's later review-review reproduced this independently after
   reading the Claude survey and endorsed it as the P0 — so its final
   position is correct; its survey missed it.)
2. **No finding for the hand-written `bindings.ts` contract**, even though
   Codex's own co-change table lists `types.rs` ↔ `bindings.ts` at 6 — more
   co-change than the compressor table its Finding 5 worries about. Grep of
   the survey for `bindings` returns only the two table rows and one passing
   mention in the decline list.
3. **No new FFI contract pins**, despite an FFI-focused Finding 4 that
   self-rates FFI proximity "high." The finding reuses existing tests and
   adds none; the genuinely unpinned wire keys (`true_peak_dbtp`,
   `dynamic_range_lu`, `sample_rate`, `bit_depth` on the actual analyze/render
   JSON) go unmentioned.

It also has no dead-code lens at all (no equivalent of the
`activeModifierChips` 134-LOC deletion, the Waveform re-export shim,
`analyze_tracks_core_lite`, the `now_iso()` divergence, the stale
`selectedAnalysis` dep), no ExportReport snapshot, no test-builder
consolidation, and no docs-truth findings (stale CLAUDE.md queue).

**In the other direction, the Claude survey owes Codex two concessions**,
detailed in the adjudication section: the `compression_mode` "live drift"
evidence was overstated, and the quick-win batch wrongly included
engine-adjacent deletions that belong behind an explicit owner gate.

---

## Finding-by-Finding Evaluation

### Codex 1 — Navigation state machine: VALIDATED (with spec corrections)

Two independent surveys, two methodologies, same #1 structural finding —
this is the strongest possible convergence signal, and the owner should read
it as such. All five Codex evidence anchors verify:

- `App.transitions.test.tsx:524` is scenario 9 ("back to Standard from Album
  mode leaves Album and lands in Standard") with the regression comment at
  :525-528 exactly as Codex describes.
- `HANDOFF_2026-06-08_STANDARD_VIEW_COMPLETE.md:53` documents the
  enforced-at-every-entry bounce guard; the second review/fix loop is at
  :124-133 within the :111-134 addendum.
- All four test files in its verification plan exist (including
  `src/lib/view-mode.test.ts` — a file the Claude survey under-weighted;
  view-mode logic has two layers: pure persisted-store in
  `src/lib/view-mode.ts:8-74` and the resolving hook in
  `src/hooks/useViewMode.ts:18-52`).

**Corrections to carry into the merged spec:**

1. Codex's explicit state set `{standard-track, advanced-track,
   advanced-album}` is **undercounted**. `useViewMode` deliberately holds
   `view: null` until `hadPriorSession` resolves, and `App.tsx:81`
   short-circuits the WYSIWYG effect on it — a machine that doesn't model
   the unresolved state regresses the loading-flicker guard. The
   Back-to-Standard confirm modal (`App.tsx:115-121`, `returnConfirm` state
   at :74) is a transient sub-state with cancel semantics. The complete set
   is closer to `{unresolved, standard-track, advanced-track, advanced-album,
   return-confirm-pending}` — which also lets the machine absorb the
   `returnConfirm` useState entirely.
2. Transitions must route through `writePersistedViewMode` (the
   `view-mode.ts` store) or localStorage desyncs — neither survey spelled
   this out; the verifier did.
3. Effort: Codex's M is the upper edge of M, possibly L, once the persistence
   layer and confirm flow are modeled. Verification plan should add
   `App.album-export.test.tsx` and `App.compressor-mode.test.tsx` (both
   assert against the current view+mode coupling).

Codex's "transition functions returning next state + required action" and the
Claude survey's "reducer + transition table" are the same change with
different ergonomics; the reducer framing folds `returnConfirm` in more
naturally. Engine proximity: zero (both surveys agree).

### Codex 2 — Evidence-lane adaptive resolver: VALIDATED (merge with the mechanical-helper extraction)

The duplication is real, verified at `fixture_matrix.rs:117-138`
(`settings_for_matrix_case`) vs `reference_tuning.rs:305-326`
(`settings_for_reference_preset`): six identical logical steps with matching
multi-line comments (clone `recommended_universal`, `volume_match=false`,
`source_lufs_integrated`, `SourceProfile::from_analysis`,
`apply_resolved_confidence(.., false)`). One correction: it is **near**-
identical (~12 of 14 lines), not byte-identical — the divergence is the
preset/compression-mode assignment (`MatrixCase` fields vs `Preset` param +
hard-coded `CompressionMode::Preset`).

All four bug-ledger citations verify verbatim (Tier-1 review :64, deep-
analysis adversarial :52-62, GLOBAL triage :34, Phase B handoff :111 — the
fix shipped as two near-identical blocks in `33d1133`). The behavior is
pinned on both sides today (`matrix_case_resolves_confidence_like_the_app`
at fixture_matrix.rs:553, `reference_settings_resolve_confidence_like_the_app`
at reference_tuning.rs:763), and the `--lib fixture_matrix` test filters in
Codex's verification plan select the right modules (mod paths confirmed).

**Relationship to the Claude survey:** the Claude survey ranked the six
*byte-identical mechanical helpers* (`csv_escape`, `sanitize_path_part`,
`preset_slug`, `normalized_absolute_path`, `lexically_normalize`,
`export_report_for` — ~60+ duplicated lines) and listed this resolver
duplication only as a hunch ("win is small"). Codex's bug-ledger evidence
upgrades it: the resolver duplication is fewer lines but a higher-stakes
class (silent adaptive-chain drift between the evidence lanes and the live
app — the exact defect the 2026-06-02 review caught). The mechanical helpers
explain more of the 10× co-change; the resolver explains the scarier part.

**Merged disposition:** one shared module (the Claude survey's
`evidence_lanes` proposal) hosting both the mechanical helpers and the
resolver wrapper, in one PR with one slow-lane verification — plus one new
test neither survey proposed but the verification surfaced: a **cross-lane
equivalence test** asserting both lanes produce identical
`settings.advanced.source_profile`/`source_confidence` for the same source —
that is the test that would catch future drift between the lanes even when
each lane's own pin stays green. Engine proximity: medium-low — the locked
resolver (`apply_resolved_confidence`, profile_store.rs:162) is untouched;
only its caller-side wrappers collapse.

### Codex 3 — useTrackMaster deepening: DISPUTED AS SCOPED

The orientation line-refs are mostly accurate (corrections: the file is
2,148 lines, not 2,057; the "project open/save :1900-1985" window misses
most of `openProjectFromDisk`, which actually runs :1946-2039). The churn
pain is real. But the finding fails on causation and on scope:

1. **The cited defects would not have been prevented by the proposed split.**
   Verified one by one: the Tier-1 :65 WYSIWYG preview-skip was a missing
   call in the preview path, fixed by moving profile derivation to the
   backend (B2) — a "live-chain dispatch module" doesn't prevent forgetting
   a call. The FINAL-review :36 readout drift is a cross-language contract
   issue; moving the fetcher doesn't reduce FE↔Rust coupling. And the
   deep-analysis :106-114 cache-eviction finding is **already fixed**
   (`removeTrack` calls `api.evictSourceProfile` at useTrackMaster.ts:1066)
   — citing a resolved P3 as live evidence is the survey's one genuinely
   sloppy moment.
2. **It ignores prior extraction work.** The live-chain dispatch already has
   a tested pure layer (`src/lib/settings-transitions.ts`, 208 lines, plus
   `effective-settings.ts`); the remaining hook glue at :385-432 is
   rAF/ref/state-machine code referenced by ~7 other callbacks. Moving it
   means a new sub-hook passing refs and setters around — a layer added, no
   coupling removed. This is the same wall that refuted the
   album/transport/history sub-hook split in the Claude survey's
   verification (the `restoreSnapshot` closure knot at :716-754).
3. **L effort for the full scope is honest — and that's the problem.** Most
   of the L budget is mechanical line-moving with marginal payoff.

**Counter-proposal (S, keeps what's good):** extract only the genuinely pure
parts —

- the export-report constructor (~22 lines at useTrackMaster.ts:1391-1413)
  into `src/lib/export-receipt.ts` with a Vitest that pins the
  `measurements_are_rendered = m != null` semantics (which gates the
  `target_not_reached` advisory downstream — must be preserved exactly);
- the **ProjectState literal duplicated byte-for-byte at :596-605 (autosave)
  and :1917-1926 (saveProjectAs)** into a `buildProjectState()` helper. This
  duplication was found during verification — neither survey had it — and it
  is a real autosave/save-as drift hazard.

Decline the rest until the owner opens it deliberately.

### Codex 4 — iPhone bridge internal split: DISPUTED

The citations are accurate (with end-of-range undershoots: ABI block is
:18-144 not :18-140; `native_preset` is :249-265 not :249-261; both parity
tests end a few lines later than cited). The test at lib.rs:361
(`native_adaptive_context_injects_desktop_profile_fields`) is real and
distinct from the gate-off resolver test at :812 — both exist.

The proposal itself fails the pain test:

1. **Neither documented bridge defect is a layout problem.** Commit
   `27edcf4` (the bridge break) was shared-type drift — B2 added a
   `tauri::State` arg, Tier-1 added two `AdvancedSettings` fields — fixed by
   the `..Default::default()` spread at lib.rs:235-238 and a call-site swap.
   A private-module split prevents neither. The `lufs_target` fifth FFI arg
   (HANDOFF_2026-06-01:33) requires header+Swift sync regardless of where
   the Rust impl lives.
2. **The production surface is ~293 lines.** lib.rs is 863 lines, of which
   570 (66%) are tests pinning exactly the right invariants. A 293-line
   facade with four ABI functions and four helpers does not need a module
   split; Codex's framing presents the full 863 as if it were production
   complexity.
3. **Risk/reward is inverted.** Codex self-rates FFI proximity "high" for
   what is a filing change, and its verification plan's decisive step
   (xcodebuild on a Mac bridge lane) is unrunnable from this Windows
   checkout — the effort grade is "M to write, unverifiable to finish" here.
   Codex itself hedges by sequencing it last and gating it on a Mac lane.

**Disposition: decline**, in favor of the two narrower moves that target the
actual documented pain — the bridge wire-key pin tests (Claude Finding 9;
the four genuinely unpinned Swift-consumed keys) and a desktop-side
bridge-surface canary / fast-lane bridge build (Claude Finding 10 +
CLAUDE.md's own "it broke once already" note). Both are FFI-neutral,
Windows-verifiable, and overlap the already-queued
`feature/ffi-contract-tests` overnight job. The Claude survey declined
bridge-internal reorganization on the same grounds ("no demonstrated pain;
the Default-spread comment is load-bearing") — the two surveys agree once
Codex's finding is reduced to its evidence.

### Codex 5 — Display-mirror parity: VALIDATED (and a genuine catch)

This finding contains the one item Codex caught that the Claude survey
missed (its completeness critic included): **the SignalChain preset-width
mirror.** Verified:

- `src/components/SignalChain.tsx:45-69` — `presetDefaultWidth()` hand-mirrors
  the eight per-preset `stereo_width` baselines, with a comment saying
  "kept in sync by hand until the calibration is sourced from Rust." The
  eight values match `dsp.rs` (:389, :416, :444, :470, :498, :524, :551,
  :578) byte-for-byte today.
- `src/components/SignalChain.test.tsx` exists but pins **zero** width
  values — it asserts DOM shape only. So this mirror has no tripwire at all.
- The historical drift is real: `2026-05-29-codex-validation-of-master-
  review.md:46` records SignalChain hard-coding Spatial at 1.3 while the DSP
  differed. The mirror has drifted before and the table was retuned as
  recently as `88b3796`.

One overstatement to correct: the **Standard-mapping side is already
double-pinned** — `standard-mapping.test.ts:14-27` (forward + reverse) and
the bridge's `native_options_map_to_shared_preset_and_intensity` at iphone
lib.rs:336. The gap there is only the absence of one explicit cross-test
linking the two sources; it is not uncovered.

**Disposition: split the proposal.** The parity-test half is the value and
is S: (1) width parity assertions in SignalChain.test.tsx; (2) a
shadow-table test in compressor-auto.test.ts naming dsp.rs:389-578 as
source-of-record (this supersedes and extends Claude Finding 7); (3) one
cross-test for standard-mapping. The "consolidated display-contract module"
half is taste-driven file churn for three small tables with no shared types
beyond `Preset['kind']` — defer it. Honest framing per the verifier: without
a Rust command or codegen (correctly declined this pass), these are
*tripwires*, not single-source-of-truth fixes — drift becomes loud instead
of silent, which is the achievable win.

### Codex 6 — Untrack graphify cache: CORE VALIDATED, REFRAMED AS OWNER DECISION

The technical claims all verify, and then some:

- `graphify-out/manifest.json` contains **234 occurrences** of the *other*
  machine's absolute paths (`C:\Users\SM - Dan\...`) plus mtimes;
  `cache/stat-index.json` is keyed by the same absolute paths with
  `mtime_ns`; 154 cache files are tracked (79 `cache/ast/` + 74
  `cache/semantic/` + stat-index — the handoff's "63 cache/ast" count is
  wrong, and `cost.json` is tracked despite the handoff calling it
  untracked-by-intent).
- Codex's keep-list is sound: `graph.json`, `GRAPH_REPORT.md`, and the
  labels file grep clean of absolute paths.

Two corrections that change the framing, not the conclusion:

1. **The "158 pair incidents" churn evidence is an artifact.** All 159
   tracked graphify-out files entered in a single commit (`32f116e`) and
   have never been touched since — there is zero demonstrated churn. The
   pain is *predicted* (next regeneration), not measured. Codex presents a
   forecast as churn data.
2. **This relitigates a deliberate owner decision.** Merge `c22d71a`
   (2026-06-09 16:39) explicitly tracked "graph.json + extraction cache +
   manifest + labels + GRAPH_REPORT.md … for every machine/agent" — *after*
   the 2026-06-08 handoff had already flagged the path-keying. The owner
   knew and chose to track.

That said, the decisive argument — which Codex did not make — is that **the
path-keyed cache cannot serve the merge's stated purpose**: keyed to one
Windows user profile, it will 100% miss on any other machine, so it warms
nothing while guaranteeing future diff noise. The honest finding is:

> The tracked cache/manifest cannot warm caches across machines as intended.
> Either (a) untrack `manifest.json` + `cache/` (S — gitignore + one
> `git rm --cached`, ~155 files) and accept cold rebuilds elsewhere, or
> (b) patch graphify to write repo-relative paths so the warm-cache intent
> actually works (M/L — upstream tool change).

**Disposition: hold for owner sign-off** with the (a)/(b) choice stated.
The Claude survey missed this area entirely; credit to Codex for surfacing
it, with the reframe above.

---

## Adjudicating Codex's Review of the Claude Survey

Codex's review-review (`codex-2026-06-10-refactor-survey-review-review.md`)
validated 19 of the Claude survey's 22 findings, independently reproduced the
P0, and raised four disputes. Adjudication, with concessions where due:

### Conceded: the `compression_mode` evidence was overstated

Verified first-hand this session: `types.rs:679-680` is
`#[serde(default)] pub compression_mode: CompressionMode` (non-Option), and
`bindings.ts:41` is `compression_mode?: CompressionMode`. Since
`#[serde(default)]` affects deserialization only, Rust **always serializes**
the field; and TS `?` permits *absence*, not `null`. So the Claude survey's
"an explicit null from the FE would fail deserialization; the TS type lies
about acceptable values" was wrong as a live-drift claim — TS doesn't admit
`null` there, and omission is handled by serde's default. The current state
is benign optionality drift (TS marks maybe-absent what Rust always emits).
The dangerous `| null` drift is historical, fixed in `f7377f0`.

One nit back: the review-review characterizes the claim as asserting the
type still reads `CompressionMode | null` — the Claude survey quoted the
current `?` form correctly; the error was in the failure-mode sentence, not
the type quote. The substance of Codex's correction stands regardless.
**The drift-gate finding itself is unaffected** — it rests on the historical
incident, the hand-sync burden (5–6× co-change), and the absence of any
wire-shape test, all of which both sides now endorse.

### Conceded: engine-adjacent deletions exit the quick-win batch

Codex rejects ranking `process_sample` deletion (Claude 12),
`analyze_tracks_core_lite` deletion (Claude 13), and the dsp.rs
one-pole/soft-knee hoist (Claude 14) as Phase 2 refactor work. Under the
brief's literal wording — "any idea that would alter rendered output, **a
test expectation**, or the FFI contract … never acted on" and "do not
propose … anything touching DSP math" — Codex's classification is the
compliant one: 12 and 13 delete tests (even though those tests pin only the
dead code itself), and 14 edits the lines the math lives on (even though the
expression extraction is zero-bit by construction).

The technical safety analysis was not disputed by anyone — zero production
callers, removal cannot change rendered output, byte-identity SHAs gate the
hoist — and Codex's own review concedes the claims are "true as source
archaeology." **Disposition: all three move out of the quick-win batch into
an explicitly owner-gated "engine-adjacent cleanup" slice** that runs only if
the owner opens it, with the byte-identity verification plans already
written. One line from the owner activates it; absent that, they stay
flagged.

### Concordance, not correction: AdaptiveReadout

The review-review objects that the AdaptiveReadout debug gate "changes
visible behavior." The Claude survey's own Finding 19 already said exactly
that — "owner-sanctioned UI change — not strictly behavior-preserving …
sits in this list only because the owner's own TODO mandates it" — and
excluded it from the quick wins. Both documents agree: it is release-polish
work scheduled by the owner's TODO (App.tsx:1846-1849,
ADAPTIVE_DSP_NEXT_STEPS.md:63-67), riding outside the refactor batch.

### Accepted: the revised quick-win batch

With 12/13 removed, the review-review's 8-item batch is the right
greenlightable bundle and is adopted in the sequence below. The items it
deprioritizes rather than rejects (`now_iso()` unification — note it changes
the reference-tuning report's timestamp *format*, a tooling artifact, not
rendered audio; the Settings/Help copy lift; the desktop bridge canary)
remain valid riders that can attach to adjacent commits.

### Claude survey errata (recorded for the file's own honesty)

- Finding 2 evidence: the `compression_mode` live-drift sentence — corrected
  above.
- Finding 3 evidence: "`ca1a3ae`→`ca1ae3a`" is a typo; the commit is
  `ca1ae3a`.
- Quick Wins: items 8 and 7 of that batch (process_sample,
  analyze_tracks_core_lite) — withdrawn per the concession above.

---

## Reconciled Phase 2 Sequence (three-document synthesis)

Supersedes the sequences in both surveys; each step is tiny, independently
green commits.

| Step | Work | Source |
| --- | --- | --- |
| 0 | Fix the 8 `contracts.rs` ExportReport literals; add the golden receipt JSON snapshot in the adjacent commit | Claude P0+F3; reproduced by Codex review-review |
| 1 | Quick-win batch: delete `activeModifierChips`/`Summary`; retarget Waveform test import; drop stale `selectedAnalysis` dep; sync CLAUDE/AGENTS jump-fix queue; sync WindowMetrics comments; consolidate the duplicated `MasteringSettings` test builders. Riders: `now_iso()` unification, Settings/Help copy lift | Claude quick wins as revised by Codex review-review |
| 2 | Contract pins before movement: Rust↔TS bindings drift gate (corrected evidence); display-mirror tripwires — compressor shadow-table, **SignalChain width parity (new, from Codex)**, standard-mapping cross-test; bridge wire-key pins for the four unpinned Swift keys (fold into the `feature/ffi-contract-tests` branch); optional desktop bridge canary | Claude F2/F7/F9/F10 + Codex F5 |
| 3 | Evidence-lanes shared module: six mechanical helpers **and** the adaptive resolver wrapper, plus the new cross-lane equivalence test | Claude F4 + Codex F2, merged |
| 4 | App.tsx mechanical extractions: `ExportReceiptCard`, then `AdvancedPanel` (PanelResetButton wrinkle noted); plus the S-slice from Codex F3 — `buildExportReport` and the duplicated `buildProjectState()` literal | Claude F5/F6 + re-scoped Codex F3 |
| 5 | View/mode state machine with the merged spec: model `unresolved` and `return-confirm-pending`, absorb `returnConfirm`, route persistence through the view-mode store; verification adds album-export + compressor-mode suites | Claude F1 + Codex F1 + verifier corrections |
| — | **Owner-decision parking lot:** graphify cache — untrack (S) vs fix-the-tool (M/L); engine-adjacent cleanup slice (process_sample, `_lite`, dsp de-dup) — one-line approval activates the already-written plans; AdaptiveReadout flag home; tauri-specta adoption | all three docs |

**Declined after evaluation:** Codex F3 at L scope (pure-parts S slice only);
Codex F4 bridge module split (contract pins instead); the consolidated
display-contract module (tripwire tests only); everything on both surveys'
existing decline lists where they agree (community splits, MasteringSettings
seams, AuditionController, App.css split, useTrackMaster sub-hooks,
dependency/codegen changes this pass).

---

## Bottom Line

Codex's survey is trustworthy on facts and converges with the Claude survey
on the highest-value structural change (the navigation state machine) and on
most declines — meaningful, since the two surveys were run blind to each
other. Its distinctive contributions are the SignalChain width mirror, the
graphify-cache portability problem (after reframing), and the bug-ledger
upgrade of the evidence-lane resolver. Its weaknesses are coverage (no
dead-code/docs lens, no contract-test proposals, and it never ran the
verification commands it prescribes — missing that the test lane doesn't
compile) and two findings (3, 4) whose proposals don't follow from their own
evidence. The Claude survey takes two corrections on the chin
(`compression_mode` wording; quick-win batch composition) and one
reclassification (engine-adjacent deletions → owner-gated).

The reconciled sequence above is the actionable output: step 0 is unblocked
today, steps 1–2 are greenlightable as a batch, and the parking lot holds
everything that needs the owner's voice.

*Evaluated at HEAD `14f9010`, 2026-06-10. No code changed.*
