# YES Master — Open Threads & Owner Decisions

> **2026-09-22 — Preset preview delay and loudness-control investigation.**
> The owner's supplied Coat source matches the installed Windows session's
> frame counts. Three cold 96 kHz preview preparations total 30.6–32.8 seconds
> across logged stages, explaining most of the reported approximately 40-second
> wait. The [investigation](reviews/2026-09-22-preset-preview-investigation.md)
> identifies duplicate measurement, reuse and gain-handoff optimization candidates,
> confirms shared target state, and explains the hidden live-meter labels.
> The end-of-preparation hiccup remains unproven: existing callback probes cancel
> before this completion boundary. All 63 focused frontend tests pass. This is
> investigation evidence and a concrete next diagnostic, with no app changes,
> new listening verdict, installed fix or performance improvement claimed.

> **2026-09-22 — Real Mac launch verification conducted.** The downloaded
> `e6db1f4` universal candidate installs on the M4 and passes native import,
> audition, WAV/M4A plus Album export, external playback and project/error
> recovery checks. Five complete outputs pass independent checks. The local
> combined branch preserves the earlier Mac fixes and corrects a Mac label
> squeeze; 902 frontend tests, 711 desktop tests, explicit encoder matrices,
> 40 app browser cases and actual device timing/lifecycle checks pass. Corrected
> ARM build `fb89d63` is packaged and installed, with the original session restored.
> The owner explicitly postponed its final native recheck and offline journey
> after the Mac locked, and authorized integrating/pushing the fixes and status
> document to `main` for cross-machine handoff. See [the scoped evidence](reviews/2026-09-22-mac-launch-verification.md).
> Those postponed checks remain unverified. Existing release gates remain;
> this authorization does not activate a release or website deployment.
> **Main transfer completed:** `60c526a` is pushed and verified on GitHub. Its
> [exact-main CI](https://github.com/DanielKinsner/yes-master/actions/runs/35744259967)
> is running at handoff, separate from the passing local/candidate evidence.

> **2026-09-22 — Launch candidate prepared; public release remains NO-GO.** Dan requested all
> applicable tests, end-to-end computer use and visual polish. The isolated
> [launch preparation record](reviews/2026-09-21-launch-preparation.md) records
> Advanced layout integration, dependency/CI repairs, current export copy,
> responsive corrections and completed local suites. Frozen candidate `e6db1f46`
> has verified Windows/universal Mac draft assets and an installed Windows M4A
> check; full CI `35700809284` completes successfully. The draft audit retains the
> unresolved recipient-permission/release gate. Research policies remain unadopted. Real Mac,
> final listening, relinking, key recovery and public updater evidence remain
> separate; downloads are not activated by preparation.

> **2026-09-21 — Advanced control alignment corrected after owner review.**
> The original `ab420654` label/value collision also blocked reset clicks.
> The first fix (`68983921`) passed collision checks but the owner rejected
> its uneven wrapping. The revised two-row grid keeps every label/readout on
> one line and puts compact reset icons in a fixed slot beside each slider.
> Auto retains an inactive icon so editing cannot move the controls. Full
> labels, numeric formats and reset-to-Auto semantics are preserved.
> Browser checks now cover minimum header spacing, row/track alignment and
> stable Auto/mixed/edited layouts in Track/Album at 1360, 1440 and 1920 widths.
> These focused checks and 32 affected component/behavior tests pass. Visual
> captures include all three states at each width; evidence is under ignored
> `test-output/text-overlap/` in the installation worktree. The full
> `npm run verify:headless` gate passes (landing and 38 app scenarios).
> Windows packages built at `b7eebf49`; the installed app/encoder were verified
> and the app reopened with a responsive window. The previous autosaved session
> was backed up locally. Enlarged browser captures establish the visual review;
> native window responsiveness does not substitute for native visual inspection.
> Work remains on `codex/fix-advanced-text-overlap`; no DSP change, main merge
> or public push is included.

> **2026-09-21 — Completed research main push authorized.** Dan requested
> committing and pushing the completed checkpoint. The [integration record](reviews/2026-09-21-research-main-integration.md)
> reviews the 16-commit research delta, verification and existing remote CI
> failures. A main-branch Vercel guard preserves the no-deployment boundary.
> Subsequent experiments remain local on a separate research branch; no sonic
> policy, release, private-audio commit or deployment is authorized.

> **2026-09-18 — Broader coverage and Metal follow-up verified.**
> Dan authorized continuing evidence collection. The [completed results](reviews/2026-09-18-universal50-and-metal-results.md)
> add 24 independently verified whole files and reuse eight Funk/Rich outputs,
> completing original-level Universal 50 current/single pairs at -14/-9 across
> all eight development sources. The fixed selector yields four qualified targets,
> five character-qualified shortfalls and seven explicit character fallbacks.
> All eight -9 singles fail a character limit. A separate two-file reproduction
> of the retained Metal +3 dB lead passes every unchanged limit at Universal 50/75
> and -14, preserving more attack than current processing while clearing the
> zero-offset candidate's tone failure. Tone margins are narrow (0.031/0.028 dB).
> This supports a future bounded search in both directions, with processing
> choice separate from delivery gain. It does not rewrite the fixed selector
> or adopt +3 as a shipping constant. Native evaluation costs, whole-file checks,
> source/settings hashes and preflight corrections are recorded. C1's final
> source/preset/control contract, quiet-copy coverage for the corrected policy,
> runtime freeze, C2 holdout and C3 adoption remain open. Main stays `ab420654`;
> normal app behavior, preset intent, gated features and release status are unchanged.

> **2026-09-16 — Broader presets, target reuse and recovery verified.**
> The [new results](reviews/2026-09-16-preset-and-target-reuse-results.md) complete
> sixteen broader preset/target outputs and four prepared-target outputs, all
> independently checked whole files. Only one of eight broader control/single
> pairs meets both the frozen character and target limits; two retain qualified
> character with a shortfall and five remain explicit character fallbacks.
> Dense-preset intent is preserved; no universal shipping constraint is inferred.
> Reusing prepared PCM preserves its character across a target change. Retaining
> the existing prepared peak/loudness facts then cuts paired finalization means
> from 7.636–8.272 s to 0.256–0.319 s with **16 bit-identical complete outputs**.
> Including one-time preparation, two evaluations use 40.0–52.7% less observed work.
> These are offline stage measurements, not an app-wide speedup. Develop processing
> choice separately from delivery gain and retain valid buffer/facts; C1's final
> source/preset contract, C2 and C3 remain open. No new shipping policy is enabled.
> After the power interruption, Realtek opens and fresh muted callback checks
> pass (659 gain / 609 EQ callbacks, zero misses/errors). Earlier endpoint-open
> failures remain preserved with cause unresolved; fixed-256/listening proof is
> not inferred. Local main stays `ab420654`, with experiments on the separate branch.

> **2026-09-16 — Dynamics research checkpoint verified; experimental direction chosen.**
> Dan chose the recommended dynamics-preserving direction for experiments when
> reaching a target would flatten punch/section contrast. He wants a polished
> result without routine explanatory caveats for users. Keep diagnostics in
> research evidence, retain accurate meters/results and deliberate strong-processing
> controls. This does not adopt shipping thresholds, a selector or a UI redesign.
> The [new results](reviews/2026-09-16-single-first-development-results.md) record
> 12 single-first and 10 follow-up rows passing complete independent output checks
> (16 new WAVs, preserved/reused anchors). Funk, Aphelion and Baby satisfy the
> frozen single-candidate character/target limits; Metal and Rich retain tradeoffs.
> Extra search costs 74.707 s of evaluation without a clear replacement for their
> current processing under those limits. Include the corrected control among
> eligible results in the next frozen selector. That [selector](reviews/2026-09-16-control-inclusive-selector-protocol.md)
> is now implemented and verified by 11 regressions plus existing whole-file
> reselection: Rich keeps current processing; Metal remains an explicit character
> fallback; the three qualifying singles remain selected. C1 contract/regression work,
> C2 holdout and C3 adoption remain open. The power-outage recovery verified all
> saved report hashes; no completed audio experiment needs restarting.

> **2026-09-16 — Local fixes merged; separate dynamics research resumed.**
> Local main is `ab420654` after verified selective integration. No push occurred.
> `codex/mastering-dynamics-research` starts at that exact main state; the original
> `codex/mastering-quality` remains at `03df038a`. The next
> [frozen development checkpoint](reviews/2026-09-16-single-first-development-protocol.md)
> measures current processing and the existing single candidate on the remaining
> five development sources, with two retained reproduction anchors. Existing
> character limits remain experimental. Comparison-control cost is counted;
> native response, offline evaluation and independent verification stay separate.

> **2026-09-16 — Selective local integration authorized.** Dan approved preparing
> and testing the fixes, merging the verified selection into local main, then
> resuming dynamic-range/quality research on a separate branch. See the
> [integration record](reviews/2026-09-16-mastering-fixes-integration.md).
> The selected result on `codex/mastering-fixes-integration` is verified for local
> main: 897 frontend tests, 716 desktop tests including four private-fixture
> checks, headless UI, both bridges and Windows packaging pass. Four fresh
> whole-song device outputs match prior WAVs exactly and pass independent checks.
> Native lifecycle and callback checks pass on the working monitor fallback;
> the current default Realtek endpoint refuses stream opening and its cause
> remains unresolved. The original research branch and evidence are preserved.
> Resume experiments on `codex/mastering-dynamics-research` after local integration.
> The review pause below is
> superseded. No push, release, deployment, spending, private-audio commit or
> automatic sonic-policy adoption is authorized by this direction.

> **2026-09-16 — Owner review first; sound experiments paused.**
> Dan wants to review the local correctness, playback, measurement, performance
> and readout changes before any main integration, with possible beta inclusion.
> The [review guide](reviews/2026-09-16-mastering-fixes-review.md) pins the current
> code/research snapshot at `f88f042b` on `codex/mastering-quality`, based on local
> main `7534f612`, and separates app changes from interleaved experiment tooling.
> Pause new sound experiments and DSP variations. After the fixes are approved
> and merged, pursue sound research on a separate branch from verified main.
> The automatic sound-priority choice is deferred; do not ask for it during this
> review. The LANDR comparison motivated investigation, and the owner now separates
> demonstrated defects from broader mastering improvements. It does not establish
> a fundamental architectural failure or the competitor's internal processing.
> Preserve all frozen evidence and private fixtures. B3's remaining validation
> limits and existing release gates remain explicit; unfinished C/E research is
> not a new blanket beta requirement. No merge/push/release/deployment is authorized
> now. The active next step is owner review, superseding the continuation below.

> **2026-09-15 — Mastering-quality implementation active locally.**
> Dan authorized implementation, testing, internal docs and small local commits
> in the existing checkout. `codex/mastering-quality` starts at local `7534f612`;
> no push/main merge/release/deployment/spend or private-audio commit is authorized.
> The [checkpoint ledger](reviews/2026-09-15-mastering-quality-implementation-evidence.md)
> records A1/A2, B1, D1/D2 and C0 complete locally; B2's qualified core is integrated
> with explicit reference/official-sequence limits. Frozen evidence/fixtures remain intact.
>
> **B3 remains in progress.** Track, rendered preview and Album finalizers pass,
> including programme joins and component PCM parity. Production Original/Mastered
> conversion now uses the actual opened device configuration and the encoding's
> pre-encode PCM rate. [Device preparation](reviews/2026-09-15-device-gain-preparation.md)
> fixes the independently confirmed whole-Imaginal device-rate ceiling miss.
> All 32 retained-output core checks and 16 first-placement original-source cases
> pass whole-file independent verification. The first placement at `9ebd2f81`
> subsequently exposed brief 9.37-12 dB transition dips. The corrected placement
> preserves compensated DSP crossfades and verifies actual landed/SRC PCM using
> cached raw facts; all eight equal-output transition cases now match exactly.
> Full desktop/private fixtures (505 library tests), strict Clippy, both bridges
> and 606 loaded native callbacks pass. All **16 corrected whole-source outputs**
> now match their prepared PCM exactly and independently pass peak/LUFS checks.
> A subsequent [combined-gain correction](reviews/2026-09-15-combined-gain-transition.md)
> removes a 1.023 dB bump during inverse device-correction/Volume Match edits.
> Six normal/interrupted cases now stay within 6.472e-7 dB; 507 library tests,
> full private fixtures, 655 rapid-gain and 606 EQ-edit native callbacks pass.
> Broader dynamic boundaries and the final-result UI contract remain open.
>
> Latest actual-device preparation/application is **44.962/45.142 s** initially,
> **2.010/2.244 s** for a new target, and **75/191 ms** for a cached target. Raw DSP
> is reused; actual gain/SRC and whole-file delivery checks still run on new targets.
> Source/file and device caches cap retained PCM at 288/192 MiB. The loaded native
> session observes **48.741 s wall / 47.516 s CPU / 722 MiB peak working set**.
> Preparation, output application, callback responsiveness and session cost remain
> separate; these are not isolated speed comparisons or new listening evidence.
>
> B4 has 316 independent encoded technical checks, exact FLAC/AIFF PCM parity and
> unclipped MP3 receipt readback. Lossy ceiling misses remain explicit/advisory;
> bounded headroom/re-encoding experiments do not justify automatic adoption.
> [C1 development](reviews/2026-09-15-drive-prototype-protocol.md) has 517 original
> coarse/refined deliveries plus 960 quiet-copy renders, complete metrics and
> 243 independently passing quiet-copy selected/control files. The separate filter-precision
> correction reduces tested normalized-copy differences to at most 5.96e-7 with
> unchanged coefficients, at 19-29% extra chain CPU in loaded paired trials.
> Its full validation and preserved old references are recorded in
> [gain-copy diagnosis](reviews/2026-09-15-drive-gain-consistency.md).
> A [normalized-reference selector experiment](reviews/2026-09-15-drive-selector-reference-experiment.md)
> completes all eight sources: preserving candidate/fallback consistency improves
> from 73/96 to 96/96, while original-group fallbacks increase from 18 to 21.
> Character/fallback limits remain open. The [actual-analysis diagnostic](reviews/2026-09-15-source-analysis-gain-results.md)
> completes 48 analyses: normalized-copy coefficients match 48/48, versus 33/48
> with actual-level analysis. Original versus normalized-source coefficients
> match 24/24. No analysis policy, automatic sonic policy, control mapping or
> unseen-holdout result is adopted.
> A [low-drive diagnostic](reviews/2026-09-15-drive-small-signal-results.md)
> retains four exact controls and eight new independently protected outputs.
> Lower drive clears Piano's contrast failure but lands at -18.7 for a -14
> request; Imaginal's attacks recover with a large miss and changed tone.
> All four low-drive pairs fail the frozen numerical convergence limit, so no
> linear reference or revised selection rule is claimed.
> The subsequent [fixed lower-drive grid](reviews/2026-09-15-drive-lower-grid-results.md)
> adds three exact controls and nine freshly independently protected whole files.
> With unchanged character limits, Coat/Piano/Imaginal select -14.000/-14.804/
> -18.142 LUFS for a -14 request. The single rule suffices for Coat; its other
> candidates cost 62.405 s of additional audio evaluation. Target-first retains
> Piano/Imaginal dynamics failures. These are development findings; owner
> direction, broader corrected-chain validation and C2/C3 remain open.
> [Saturation calibration](reviews/2026-09-15-saturation-calibration-results.md)
> retains isolated continuity/antialias comparisons, eight independently verified
> native finite-filter cases and **16 independently passing whole-song outputs**.
> Four current controls are exact and four f64-only controls pass numerical
> isolation. Filters redistribute limiter activity without resolving C1 character
> failures. The serial long-filter chain costs 68-148 s on these songs; table reuse
> is negligible. A separate accumulation trial improves long-filter processing
> 8.12% but regresses the short filter 11.10%.
> A separate AVX experiment subsequently reduces paired short/long processing
> cost 16.61/31.79% on this host, with all eight outputs exact against the prior
> four-accumulator version and independently qualified. Its scalar fallback is
> also exact; the existing whole-song timings remain serial-filter evidence.
> Earlier block-interval exceedances,
> actual-device/selected-C evidence and owner calibration/listening remain open.
> Preset intent and gated constants are unchanged.
> No installed/Mac/release verdict follows from this local implementation.

> **2026-09-15 — Revised mastering plan: useful upfront preparation is an owner priority.**
> Dan accepts longer initial analysis/preparation for demonstrated audio quality,
> fidelity and subsequent interaction benefits. The [revised plan](plans/2026-09-15-mastering-quality-implementation-plan.md)
> makes C0 an early preparation/quality/performance experiment alongside A/B1/D1;
> offline C no longer waits for B2/B3 production integration. It compares a
> single-render source-relative rule, attenuate-only processing and bounded
> search, using checked research references before production adoption. Initial
> wait, prediction-settling time and live audio responsiveness are separate
> measures. Numeric sonic limits/control mapping remain open; the agent owns
> candidate calibration and validation. This request updates the plan, with
> production implementation and existing calibration gates unchanged.

> **2026-09-15 — Mastering implementation planning draft prepared.**
> The [code-grounded plan](plans/2026-09-15-mastering-quality-implementation-plan.md)
> separates SRC correctness, qualified final-output protection, automatic-drive
> prototyping, existing-control accuracy and saturation calibration. Receiving-
> machine restoration verified 3,543 objects / 4,507 paths. A fresh locked Windows
> harness build reproduced On/Off/257-frame WAVs byte for byte after diagnosing
> source line-ending differences in an isolated copy. Current production is
> unchanged. Automatic dynamics limits and control mapping remain proposals;
> planning is not production adoption, new listening approval or release work.

> **2026-09-15 — Final mastering research synthesis; next step is implementation planning.**
> Claude's blind report and unblinded reconciliation are archived with original
> seals, scripts/settings and raw measurements. The [authoritative synthesis](reviews/2026-09-15-mastering-quality-final-synthesis.md)
> resolves disagreements and corrects overclaims/reference-meter errors in both
> investigations. Recommended order: SRC correctness; final-rate peak protection
> independent of LUFS targeting; bounded source/target-aware drive; existing
> Density/readout accuracy; separately calibrated saturation/antialiasing.
> The [fresh-agent prompt](prompts/2026-09-15-mastering-quality-planning-handoff.md)
> and [transfer guide](reviews/2026-09-15-mastering-quality-transfer.md) support
> another machine. Fresh On reproduction and five pruned Claude witnesses match
> their recorded WAV hashes. Preserve defaults, gates and prior listening verdicts.
> This closes the requested research/reconciliation, not production implementation
> or release activation. The independent sibling is a folder without Git; no
> cleanup was performed by the final pass. Earlier entries below are history.

> **2026-09-15 — Independent report received; targeted follow-up prepared.**
> Claude Fable completed its report in the separate review workspace. All 18
> sealed files match their recorded hashes. The report overlaps on excessive
> upstream processing and peak discrepancies, recommends an attenuate-only
> policy, and reports additional no-target/short-input ceiling failures; these
> are its findings, not yet a completed cross-investigation replication.
> [The explicit unblinded follow-up](prompts/2026-09-15-mastering-quality-targeted-replication.md)
> directs matched-condition tests and a separate reconciliation report. A verified
> 74-file text/source checkpoint is retained under ignored `test-output/`; unique
> rendered audio remains in the review workspace. Keep that workspace through
> reconciliation and verified evidence preservation. Nothing was deleted or
> changed in production; the targeted follow-up itself has not been run here.

> **2026-09-15 — Independent review prepared at the owner's request.**
> A separate source/audio workspace and [neutral question-led prompt](prompts/2026-09-15-independent-mastering-audit.md)
> support a fresh Claude Fable assessment without earlier findings or experiment
> tooling. [Coordinator instructions](reviews/2026-09-15-independent-audit-setup.md)
> explain context isolation, sealing results and later reconciliation. The review
> itself has not been launched; no independent verdict or production change is claimed.

> **2026-09-15 UTC / September 14 Pacific — Mastering-quality recommendation completed.**
> [The expanded investigation](reviews/2026-09-15-mastering-quality-recommendation.md)
> retains byte-identical owner exports and adds seven licensed pieces, 192 pilot
> delivery measurements, input-level robustness and 504 native mechanism cases.
> It recommends source/target-aware drive ahead of global density/Adapt retuning;
> exploratory -14 tests meet target and independent ceiling in all 12 tested
> source/level conditions. -9 tradeoffs, input bounds and peak failures remain.
> Separate SRC buffer/short-input defects and native-versus-independent peak
> discrepancies are demonstrated; an isolated SRC correction passes 45 conditions.
> Production adoption and its application/bridge/fixture checks remain separate.
> Defaults, calibration gates and prior listening verdicts are unchanged. No new
> owner questionnaire is needed. [Portable scripts](../scripts/research/mastering-quality-20260915/README.md)
> accompany the recorded evidence; the older audio archive remains intact.

> **2026-09-14 — Density Auto thumb question added to the continuation.**
> The owner asks whether Auto should display the preset's default and how that
> should interact with Adapt strength. [The UI follow-up](prompts/2026-09-14-mastering-quality-continuation.md#additional-owner-question-where-should-density-auto-place-the-thumb)
> recommends a thumb tied to requested density (preset default while Auto), with
> source-adaptive processing shown separately. Adapt must not move the requested
> value. Preserve null Auto state and current sound; wording/implementation remain
> pending. No UI or DSP change was made in this handoff update.

> **2026-09-14 — Owner requests an evidence-led quality recommendation.**
> The owner has limited test music and reports that the five A–E clips are too
> subtle for a useful naked-ear choice. This is not a listening pass, preference
> ranking or proof of equivalence. [The continuation prompt](prompts/2026-09-14-mastering-quality-continuation.md)
> asks the next agent to acquire suitable additional material, run objective
> mechanism/robustness tests, use LANDR as an empirical benchmark, and make a
> reasoned recommendation. Do not stop at “it is subjective” or repeat the same
> listening task as the next gate. Existing defaults and calibration gates remain.

> **2026-09-14 — Research prepared for cross-machine continuation.**
> [The transfer guide](reviews/2026-09-14-research-handoff.md) indexes the retained
> pricing, competitor, control and DSP proposals and versioned measurement CSVs.
> The owner explicitly authorized GitHub transport of the preset audio/evidence.
> A separate research draft holds the verified archive; ordinary Git clones keep
> audio/build outputs ignored. This is evidence transport, not preset approval
> or application release activation. Follow the guide to download and restore.

> **2026-09-14 — Dynamics stages investigated with exact export reproduction.**
> [Native diagnostic research](reviews/2026-09-14-dynamics-stage-investigation.md)
> reproduces both owner On/Off WAVs byte for byte. Eighteen local renders isolate
> compressor/saturation/limiter combinations, density/adaptation settings and
> input trim. Saturation and limiting contribute more to reduced peak contrast
> than the compressor switch on this fixture; Adapt demonstrably eases compressor
> gain reduction. Source/target-aware drive and saturation mapping are the next
> research priorities. Five anonymous equal-volume clips are ready locally.
> No DSP/default/gate/UI changes or listening verdict; no repeat owner export needed.

> **2026-09-14 — Density/adaptation integration proposed.**
> In response to the owner's design question, [the recommendation](reviews/2026-09-14-yes-export-vs-landr.md#proposed-integration-of-density-and-adaptation)
> keeps compression amount in Advanced and presents global “Adapt to track”
> primarily as automatic behavior, with strength in expanded controls. Preserve
> current defaults/saved values, fix the Auto thumb, and show backend-resolved
> effects. This is a proposal, not an adopted redesign or a DSP change.

> **2026-09-14 — Owner Compressor On/Off comparison completed.**
> [Measured pair](reviews/2026-09-14-yes-export-vs-landr.md#completed-owner-comparison-compressor-preset-versus-off)
> holds Universal 75 / Adapt 50% per owner instructions. Both measure -9.0 LUFS;
> Off raises LRA from 3.1 to 3.6 LU and crest from 8.95 to 9.42 dB. On exactly
> matches the earlier YES export. Compressor mode contributes to the density,
> but most source/LANDR peak-contrast difference remains with it Off. Remaining
> stage attribution is unresolved. These files need not be requested again.
> No DSP, preset or gate changes; all source files remain unchanged.

> **2026-09-14 — Density/adaptation controls reviewed after export comparison.**
> [The follow-up](reviews/2026-09-14-yes-export-vs-landr.md#follow-up-preset-density-and-adapt-strength)
> confirms named-preset density defaults to 0.50, while Adapt strength (default
> 50%) reduces eligible preset processing. Intensity is separate. Code review
> found an Auto density thumb parked at zero and an idle “Effective compression”
> summary that omits adaptive trims; these presentation issues remain unmodified.
> 24 existing frontend tests passed. The export does not identify which dynamics
> stage caused its lower peak contrast; no DSP or preset changes were made.

> **2026-09-14 — Actual YES export compared with LANDR references.**
> [Independent measurements](reviews/2026-09-14-yes-export-vs-landr.md) of the
> owner's Universal Intensity 75 file show -9.0 LUFS / 3.1 LU LRA / -2.6 dBTP,
> versus LANDR High at -9.4 to -9.5 LUFS / 3.8 to 3.9 LU / -0.3 dBTP. YES has
> lower peak contrast and more relative side/upper-treble energy on this source.
> Exact build and other export settings remain unspecified; no listening winner
> or general ranking is claimed. DSP and source audio are unchanged.

> **2026-09-14 — LANDR/Waves DSP research and reference measurements refreshed.**
> The owner supplied the existing reference folder and asked whether competitor
> analysis time suggests missing processing. [The assessment](reviews/2026-09-14-landr-waves-dsp-assessment.md)
> remeasures the source plus seven saved masters, checks native source adaptation,
> and separates vendor disclosures from proprietary unknowns. It recommends a
> fair current-engine benchmark, then calibrated per-band dynamics and bounded
> corrective tonal balancing. No fresh YES/Waves render or listening verdict is
> claimed; DSP, preset voicing and calibration gates remain unchanged.

> **2026-09-14 — Desktop-plus-phone bundle proposed in pricing discussion.**
> The owner proposed one desktop and one mobile experience for one purchase price,
> and asked about mechanical competitor audio tests and the perception of $49.
> [The pricing follow-up](reviews/2026-09-14-pricing-assessment.md#follow-up-mechanical-comparison-a-phone-bundle-and-the-meaning-of-49)
> records the existing reference runner, a fair benchmark approach and the bundle's
> value. Final price, purchase/restore terms and availability remain unsettled;
> this discussion does not certify mobile releases or impose a device-count limit.

> **2026-09-14 — Pricing reopened for competitive assessment.** The owner
> explicitly said the former $29 founder / $49 standard figures are not fixed.
> [The product and market assessment](reviews/2026-09-14-pricing-assessment.md)
> recommends testing $49 introductory / $79 standard for a qualified paid 1.0;
> that is agent judgment, not an adopted price. The existing free-beta duration
> is unchanged. Final prices and purchase terms remain open.

> **2026-09-14 (afternoon) — Website try-it rebuilt with the desktop's source analysis.**
> On `vera/web-tryit`: the browser engine now compiles the desktop's `analysis.rs`,
> `guardrails.rs` and `confidence.rs` by path, so the demo adapts each style to the
> whole track exactly as Track Master does (confidence and Adaptive Compressor stay
> off, as on the desktop). Worker pool + render cache + neighbour prefetch make
> style/loudness switches ~30 ms once warm; decode is at the file's own rate; seek
> and keyboard control added; no verdict/warning text anywhere; scratch page and
> duplicate binaries deleted; `npm test` now fails if the checked-in WASM drifts
> from the desktop sources (`npm run build:tryit-wasm` fixes it). Six Tauri
> commands are `cfg`-gated for the wasm target; no desktop behavior changed.
> Volume Match stays in the card, off by default (owner decision, same day).
> [Evidence](reviews/2026-09-14-web-tryit-rebuild.md). Not a deployment.

> **2026-09-14 — Website audition improvements authorized and implemented locally.**
> On `vera/web-tryit`, loading now waits for Play; explicit Original/Mastered,
> retained pause position, app shortcuts, excerpt/seek controls, compact layout
> and worker-based preview updates implement the owner's requested follow-through.
> Desktop DSP simplification stays deferred. The browser preview's source-analysis
> limits are visible. [Changes and verification](reviews/2026-09-14-web-tryit-audition.md)
> include a pre-existing beta-copy test mismatch; this does not activate a release.

> **2026-09-14 — Main integration authorized; local production files excluded.**
> The owner requested all necessary launch changes on main and appropriate ignore
> rules. The local `YES_Master_Video_Packet/` and private signing/recovery material
> are ignored; the packet remains on disk. Resumed installed Windows checks passed
> Standard/Advanced M4A export, Album cancellation/retry with advancing audition,
> and external playback. The historical pause did not recur; its cause remains open.
> [Exact installed evidence](listening/2026-09-14-windows-launch-candidate.md).
> Completed CI at `f5976299` passed all three encoder and archived-source rebuild
> jobs, but failed downstream on the missing independent-decoder environment and
> renamed beta-date row. Both setup issues are corrected. Later `cf1ddc21` also
> exposed an intermittent Intel Mac Album PCM comparison failure; strict equality
> remains and failed synthetic files will now be retained for diagnosis. Overall
> CI/release readiness is not green. This integration does not activate downloads
> or publish a desktop release.

> **2026-09-14 — Launch fixes and replacement Windows review package prepared.**
> Fixed M4A edit-list duration rounding and both Mac encoder configuration
> failures. Both Mac architectures passed encoder/application-format qualification
> in run `34861723641`; the overall CI run remains pending. New source-rebuild
> coverage runs at `f5976299`. The installed Windows candidate remains exact
> `cf3ddcc9`, with verified signatures and local modified-library rebuild proof.
> See [the launch ledger](plans/2026-09-14-launch-readiness.md) and
> [candidate record](listening/2026-09-14-windows-launch-candidate.md).
> User Escape stopped desktop input. Preserve the September 9 baseline PASS;
> the changed installed behavior, Mac hardware, recipient permission, key recovery
> and actual release/updater transaction still need their applicable evidence.

> **2026-09-14 — Beta duration settled.** The owner selected **eight weeks from
> actual launch**. This supersedes the provisional October 31 end date. At the
> verified public release transaction, set the public beta end date to the
> actual publication date plus 56 calendar days; do not start the clock during
> candidate preparation. Engineering/CI follow-through is authorized.

> **2026-09-14 — Launch readiness refreshed; demo removal prepared locally.**
> The owner asked to get YES Master launched without a demo video. Removed the
> inactive hero demo button and unavailable-video note; release metadata remains
> closed. Live GitHub main is `af406855`; its CI run `34402480068` completed
> **failure**, not merely pending: both Mac encoder builds stop at
> `configure_host[@]: unbound variable`, and Windows encoder qualification reports
> an M4A frame count of 88332/88337. Desktop/bridge jobs were skipped by dependency.
> These are observed failures; the Windows decoder/encoder cause is not established.
> GitHub still has only the frozen beta.1 draft. The production site's HTTP entry
> point returns 200, which does not prove release availability or browser behavior.
> See the [current launch assessment](plans/2026-09-14-launch-readiness.md).

> **2026-09-09 — Export formats implemented; Windows candidate installed.**
> [Live implementation evidence](plans/2026-09-09-export-formats-evidence.md)
> records U2–U5 completion, the qualified Windows encoder, 859 frontend and 643
> Rust passes, 38 headless checks and independently verified native Track/Album
> files. [Candidate `7cd36ab6`](listening/2026-09-09-export-candidate.md) has exact
> NSIS/MSI hashes, verified updater signatures and source/relink rebuild proof.
> U1 Mac and U6/U7 release gates remain open, including one unexplained audition
> pause, disconnected/player/listening/accessibility and exact remote CI evidence.
> Owner confirmed Mac verification must wait for a later Mac session. Continue
> Windows and independent release preparation. Existing installed Windows
> `07021f1b` owner PASS remains valid for its named scope. No release activity authorized.

> **2026-09-09 — Export and beta launch implementation plan written.**
> [Export formats and public beta launch plan](plans/2026-09-09-0857-feat-export-formats-beta-launch-plan.md)
> details encoder qualification, all desktop export flows, installed package checks,
> candidate preparation and activation. The proposed launch set adds FLAC, AAC/M4A,
> standalone AAC, Ogg Vorbis and AIFF to WAV/MP3; ALAC/Opus are explicit follow-ups.
> These are planning recommendations, not implemented capabilities or publication approval.

> **2026-09-09 — Installed Windows owner check PASSED, all five checks.**
> Installed and launched 0.9.2 from exact `07021f1b` with a verified clean build
> stamp; existing session preserved. 240 focused tests and 19 checks through the
> real Windows audio output passed, plus independently decoded WAV/MP3 exports.
> [Five simple owner checks and precise evidence limits](listening/2026-09-09-installed-check.md)
> replace a broad repeat questionnaire for this sitting. The owner reported
> "actually ran through all those tests. all 5 passed". The named Windows baseline
> is approved; do not reopen that same questionnaire. Continue with broader
> export support and launch preparation, testing the changed behavior. This
> signoff does not certify a future format expansion or Mac release artifact.

> **2026-09-09 — Owner clarified broader export-format intent.** MP3 was an
> example of the desired range of delivery formats, not the complete request.
> The intended product should export the major audio formats users need,
> covering its supported import formats and additional useful delivery formats.
> Current picker extensions are WAV, MP3, M4A, AAC, FLAC and OGG; implemented
> desktop export choices are WAV and MP3. The September 5 MP3 checkpoint remains
> completed evidence for that increment, not closure of this broader requirement.
> See [the expanded export scope](plans/2026-09-05-export-options.md#broader-export-format-scope--owner-clarification-2026-09-09)
> for the distinction between the owner's requirement and the proposed format set.

> **September 6 end-to-end launch audit:** native Mac import/audition, WAV/MP3,
> Album cancellation/retry/order, project persistence and damaged-input recovery
> were exercised with Computer Use. Export selectors/title branding, Standard
> meter containment, stale Album retry receipts and direct access to Album files
> were corrected. Independent measurements checked five delivered files.
> See the [audit ledger](reviews/2026-09-06-launch-readiness.md) and
> [owner checklist](reviews/2026-09-06-launch-readiness.html). Local checks do not
> clear the replacement-candidate, exact installer/updater, MP3 distribution or
> targeted listening gates. No publication or new owner decision is implied.

> **September 5 follow-through implemented locally (verification continued September 6 UTC).**
> Return to start/Home, pointer/keyboard reorder with insertion and edge scrolling,
> stable Track/Album header/timeline geometry, incremental analysis/profile readiness,
> selected-track automatic Preview LUFS, and WAV/MP3 mastering export are implemented.
> This supersedes the implementation-pending annotations in the historical interview
> entries below. WAV remains default; MP3 defaults to 320 kbps with 256/192/128.
> Track/Standard encoder tests preceded Album batch/continuous implementation and
> tests, as authorized. No settings reset, converter, preset retuning or release.
> See [checkpoint 9](plans/2026-09-05-listening-follow-up.md#checkpoint-9--completed-owner-follow-through)
> for actual evidence and the remaining platform/listening limits.

The single place where **genuinely-open work** and **decisions only the owner can
make** are logged so nothing gets silently dropped. Companion to
**docs/CHANGELOG.md** (shipped history) and **docs/IDEAS_BACKLOG.md** (wishlist).

Generated 2026-06-22 from the docs-hygiene recon. Update this file as threads
close rather than letting them rot in scattered docs.

> **Transport follow-up — accepted interaction contract, now implemented in checkpoint 9.**
> Add Return to start beside Loop under Play/time (center it in Standard, where
> Loop is absent). No end button for symmetry. Jump to 0:00 while preserving
> playing/paused state; a finished track stays paused. Keep Original/Mastered
> and processing settings. Disarm an active loop first, retaining its drawn
> region so it can be re-enabled. Keep Home consistent with the new action.
> Improve row dragging with a discoverable handle, insertion feedback and edge
> scrolling, preserving selection/settings/playback and keyboard reordering;
> implemented scope is Standard, Track and Album. Pointer capture replaces the
> previous Album-only HTML drag path; verification is recorded in checkpoint 9.
>
> **Right-rail overflow and mode-switch Width flash — corrected locally.**
> Reproduced Album's hidden Adapt Strength tooltip expanding a 279px scrollport
> to 388px. Bound the tooltip to the shared control grid without hiding overflow.
> Track/Album sections and Export now align in matching Manual states at
> 1360×740, 1440×900 and 1920×1080, with hover/focus checked. Width retains the
> same track's previous position while a mode change resolves, with pending
> accessibility/hover status; obsolete replies remain rejected. New browser
> coverage measures the rail itself and samples transitions. Native Windows
> Track/Album captures also show matching rails and no horizontal scrollbar.
> Vertical scrolling remains available where controls exceed window height. Ceiling
> shows -1 Auto on Track versus -1 Custom on Album: current captions distinguish
> unset automatic from explicit settings, not just equal displayed numbers;
> read-only session inspection confirmed both null and explicit -1 settings.
> Both modes use the same caption rules; no sound setting is changed to align labels.
> Owner asked why Render audit WAV is needed. It produces a temporary offline
> track WAV using the shared renderer, not an improvement to live preview and
> not an Album-plan proof. **Owner decision:** remove the entire Tools section
> and audit-render action; ordinary Export is sufficient for checking a saved
> file. Internal renderer/verification tools remain. UI removal is implemented
> in checkpoint 7. Checkpoint 8 records the overflow/Width fix and evidence limits.

> **2026-09-05 — MP3 purpose clarified in owner interview.** MP3 is a smaller-file,
> broadly playable choice within the normal mastering export. The owner rejects
> a separate conversion workflow or Original-source export selector. This
> supersedes the earlier interpretation of the listening notes as requiring
> conversion without mastering. Preserve existing DSP/delivery behavior at 0%
> Intensity; do not add bypass or special workaround explanations. Normal meters
> and export claims remain truthful. **Quality choice settled:** 320, 256, 192
> and 128 kbps, with 320 kbps selected by default in the same selector in Standard
> and Advanced. WAV remains the default export format. MP3 is not implemented
> by this decision.
> **Explicit follow-up clarification:** 0% was only a hypothetical workaround,
> not an MP3 default or requirement. MP3 uses the user's chosen preset, Intensity
> and controls just like WAV. Selecting the encoding never resets mastering
> settings or selects 0%.
> **Scope and sequencing settled:** implement Track Master MP3 in Standard and
> Advanced first. The implementing agent tests the encoder, normal mastering
> export and delivered-file contracts, then proceeds to Album batch/continuous
> export once those pass. The owner explicitly authorizes that continuation;
> no additional owner approval or owner testing is required for this dependency.
> Verify Album's own cases as well. This does not authorize publication/release
> or replace any separate listening/signoff requirement for sonic changes.

> **2026-09-05 — Album manifest presentation settled in owner interview.**
> The owner accepted `metadata/manifest.json`, with the concern that users
> should not receive an unexplained file. Keep audio files separate and explain
> the supporting export details in the in-app receipt using plain language:
> these files are not needed to play or share the audio. Update paths, links and
> cleanup together; preserve existing exports. Implemented in local checkpoint 6.
> **Filename choice also settled:** Track uses `Song_mastered.wav`; Album uses
> `01-Song_mastered.wav`, with one underscore. Album numbers follow the final
> left-rail arrangement at export, not worker completion order. Code inspection
> confirms the current rail tracks feed the plan in order and the plan assigns
> one-based positions used by export filenames. Apply the suffix to new default
> names only; preserve existing exports and collision/source protection.

> **2026-09-05 — Loading workflow settled in owner follow-up interview.** Make
> each imported track usable as soon as its analysis/profile is ready, show
> progress for remaining tracks, and prioritize playback over background work.
> The owner accepted this recommendation. Implementation must verify concurrent
> analysis keeps audition responsive and measure first-ready/batch timing;
> this decision is not evidence that the behavior is already implemented.
> Later settings changes can still require Preview LUFS measurement.
>
> **Automatic Preview LUFS also settled in the follow-up interview:** when
> Preview LUFS is already enabled for the selected mode, start measurement as
> soon as the selected track is ready. Show "Measuring," cancel obsolete work
> when switching tracks, and protect playback with bounded background work.
> Other imported tracks wait until selected; do not queue a full-track render
> for every import. Implementation and resource/lifecycle verification remain
> pending; this does not change the feature's enabled state or defaults.

> **2026-09-05 — Listening follow-up implementation underway locally.** Warm VM
> settings edits reproduced a 16.26 dB audio jump. Retaining applied same-source
> attenuation corrects that regression; strict Clippy and the full ordinary Rust
> suite pass. Native transition/stress and remaining fixture evidence are still
> open. See the [checkpoint ledger](plans/2026-09-05-listening-follow-up.md#implementation-evidence).
> Existing unrelated UI/capture work is preserved; no push or release authorized.

> **Local performance checkpoint:** playback PCM sharing eliminates repeated
> whole-source allocations; full-chain DSP still dominates Preview LUFS waits.
> Failure context now persists in diagnostics. Ordinary Rust and four actual
> existing-file contracts passed. See [stage evidence](listening/2026-09-05-follow-up-evidence.md)
> and [concrete unresolved output options](plans/2026-09-05-export-options.md).
> A further local checkpoint bounds preview workers across source/device changes
> and cancels obsolete renders between processing blocks; full fixture and bridge
> checks pass. Callback deadlines, audible dropout and targeted new listening
> remain open.

> **Native/UI follow-through:** Width pending display, inactive Standard region,
> mono Insight and automatically opened Album receipt are committed locally.
> Combined frontend 846 tests and 31 headless checks pass. Five-minute native
> high-rate transport/edit/A-B checks and an actual four-track Album export with
> independent FFmpeg receipt validation are recorded in the checkpoint ledger.
> This adds mechanical evidence, not owner musical signoff or a dropout verdict.

> **Landing design is parked.** Discussion reminders are in
> [LANDING_DESIGN_NOTES.md](LANDING_DESIGN_NOTES.md); they are not a task list.
> The owner is switching focus to software work.

> **2026-09-05 — Agent operating rules revised with owner approval.** Read
> task-relevant documentation, choose implementation details autonomously, and
> explore DSP/performance alternatives within authorized work. Use focused tests
> during iteration and affected integration lanes at coherent checkpoints; make
> coherent commits and preserve concurrent work instead of blindly pulling.
> Continue authorized work after progress reports, and update internal docs for
> decisions already made without another approval request. These rules supersede
> older generic full-reading, every-lane-per-commit, and stop-on-any-DSP rules.
> Product contracts, intentional sonic choices, gated feature decisions, and
> applicable publication/release authorization remain in force. See `AGENTS.md`
> and [the verification scope matrix](TESTING.md#choose-verification-by-scope).

> **2026-09-05 — Owner listening pass CONDUCTED; evidence and follow-up recorded.**
> Daniel listened in the native Windows app via `npm run tauri dev`, with Focusrite
> USB output/studio monitors, and reported build `e600a21` as verified. Read
> [the reconciled answers](listening/2026-09-05-owner-handoff.md) and
> [the follow-up plan](plans/2026-09-05-listening-follow-up.md); unchanged source
> exports and the reusable guide are linked from [the record index](listening/README.md).
> Normal A/B/playhead continuity, normal musical contrast, and the observed Track
> Master export comparison passed. The owner broadly likes the sound and wants
> taste left alone; the overall selection remains “Not ready / stopped here.”
> Remaining work is the recorded VM level jump, high-rate/long-file behavior,
> Width/Loud mechanical checks, loop/UI/export findings, and narrow coverage gaps.
> **Do not treat the listening session as pending owner, repeat the whole guide,
> or request details already supplied in prose.** New fixes need targeted evidence,
> not automatic approval from this earlier build. The required analysis/loading discussion
> and export choices were subsequently settled in the interview recorded above;
> implementation and verification continue under those decisions. This documentation integration does not retune
> presets, implement application fixes, or activate a release.

> **2026-09-05 — Listening-plan additions and performance exploration.** The
> owner supplied the existing load fixtures; [all 15 headers were verified](listening/2026-09-05-performance-fixtures.md).
> Do not regenerate those files. Add 384/705.6/768 kHz lag-without-timeout cases,
> capture the full intermittent switch error to the diagnostic log, and correct
> true-mono Source Insight labeling. The plan explicitly permits investigation
> and experiments with better DSP/measurement implementations for performance,
> with measured results and preserved correctness/sound contracts. Intentional
> sonic tradeoffs remain owner decisions; this is not a preset-retuning request.
> The manifest question offered exactly two options. The subsequent interview
> selected `metadata/` with a plain-language receipt explanation (recorded above). These additions update the plan, not application code.

> **2026-09-05 — Keep the live layout; remove only the Sign-off section.**
> The owner preferred the existing live page over the consolidation experiment.
> PR #31 was closed without merging; `codex/landing-tightening` remains parked.
> The owner then identified the pencil/blank-sheet photograph and its Sign-off
> caption on main for removal. This change removes that section and its unused
> component only. The surrounding export and sound-character sections, other
> studio imagery, native app and release/signup gates retain their live behavior.

> **2026-09-04 — Studio marketing website publication AUTHORIZED.** The
> owner approved the redesigned studio site and explicitly requested subtle
> animation, push/merge to `main`, and publication as the new live website.
> This supersedes the earlier no-deploy instruction for the marketing design.
> The existing `yes-master.vercel.app` domain, noindex, closed installer and
> signup gates remain unchanged. This is not authorization to activate the beta,
> tag/publish installers, spend on release services, retune DSP or enable flags.
> Release gates remain open; the September 5 listening results are recorded above.
> Visual/artwork evidence:
> `docs/landing-mockups/studio-redesign-verification.md`.

> **2026-09-04 — Correctness work authorized; release activity parked.**
> Owner requested implementation of the fresh solo audit findings, with
> regression tests, and deferred listening to **one combined pass toward the
> end**. No release spending, tagging, publication, or deployment is requested.
> Work/evidence ledger: `docs/plans/2026-09-04-audio-correctness.md`, merged
> to `main` for the owner's machine switch. Preset calibration and gated adaptive
> features remain unchanged. The requested combined listening session was
> subsequently conducted September 5; its results and limits are recorded above.
> Existing historical approvals do not approve subsequent fixes automatically.
> Follow-up implementation now includes bounded live metering, faster high-rate
> limiting, background cold Volume Match, worker reuse across A/B switches, and
> per-track Album target/peak receipts. See the same ledger for regression and
> local build evidence; September 5 follow-up findings now replace the missing-pass item.
> Owner tests via `npm run tauri dev` and accepts the current meter presentation.
> Minor follow-ups: explain the 0.2 LU independent LRA difference and include
> the numeric target on Custom export receipts. Neither is a confirmed audio
> defect. Private fixtures/local evidence do not travel with git.

> **2026-09-04 — Display-scaling owner hand-test CLOSED / PASS.** The
> installed Windows build stamped `e5943f4` was checked on the 4K office
> monitor at **250 %** and **300 %** scaling. Standard remained fully visible
> and manipulable without a center scrollbar; Advanced retained its intended
> center/right-rail scrolling; the 300 % result was crisp and complete. An
> accidental 350 % observation is not part of the gate. The follow-up fix
> makes startup zoom use the actual post-maximize webview client area instead
> of the monitor work area, with regression coverage. Remaining owner lane:
> beta.1 disposition + beta.2 tag, candidate install / final by-ear check /
> updater proof, publishing, and the landing activation phrase.

> **2026-09-01 — Friday ship plan executed (S1–S7); hero CTA stays
> copy-first (D10).** The decision-complete plan
> `docs/plans/2026-09-01-friday-ship-execution-plan.md` carries the owner's
> in-session decisions **D1–D11** in its §1 table: no custom domain (public
> URLs use the deployed origin), Dependabot PRs closed not merged, the beta.1
> disposition and beta.2 tag **not yet decided** (D3), window sizing in three
> tiers (D4), the Style/Preset vocabulary rule (D9), and the three taste
> calls (D11). Every executor slice shipped on `main` — `docs/CHANGELOG.md`
> (2026-09-01, ship-review entry) and the go/no-go §9 ledger name the exact
> SHAs and lanes. Still owner lane (plan §5): the beta.1 disposition + beta.2
> tag, install / by-ear check / updater
> proof, publishing, and the trigger phrase that unlocks the landing flip.
> **D10 in one line:** the fixed nav CTA is the all-axis acquisition control
> at every viewport; the hero CTA is gated horizontally only and may sit
> below the fold on short phones by design, because the introductory copy
> comes first. Owner-queue row T-03 is closed; the rule is permanent in
> `docs/TESTING.md` "Landing quality gates". No layout change.

> **2026-08-31 — Adversarial audit: `v0.9.2-beta.1` @ `c750da6` is
> AUDIT-BLOCKED / NO-GO pending owner disposition (not yet rejected).**
> Confirmed release-bound blockers (hostile-import panic, updater notice
> loss/recovery, sticky-TOOLS transparency, receipt accessibility, RustSec
> unsound blind spot). Remediation runs on
> `codex/launch-readiness-remediation` per
> `docs/superpowers/plans/2026-08-31-adversarial-audit-remediation-and-launch-readiness.md`
> (Claude's verified triage sits beside it). Beta.1's tag, draft release
> (379883047), and evidence stay frozen until the owner's Task-12
> disposition; U15–U17 do not run against beta.1.

> **2026-08-19 — The 2026-07-27 candidate (`v0.9.1-beta.1` @ `34f7c88`) is
> SUPERSEDED; the freeze is lifted. Owner decision.** Three weeks of
> feature/design work landed on `main` after the tag (movable EQ bands,
> bipolar knobs, Source Insight, the premium restyle, the four UI polish
> passes, the stuck-analysis fix), so the tagged bytes no longer match what
> will ship. Version is now **0.9.2** (three manifests + preview mock). Next
> candidate: install + the one listening sitting on current `main`, then
> re-run U14's gates and re-tag **`v0.9.2-beta.1`**; the 07-27 draft
> releases (`v0.9.1-beta.1` and the three stray `-manual-*` drafts) are stale
> and should be deleted on GitHub before the new tag. U15–U17 remain
> owner-lane. The freeze rule itself stands: it re-enters force at the next
> tag. Details: `docs/plans/beta-go-no-go.md` §Candidate freeze.

> *(Historical, 2026-07-27 — superseded above.)* **⚠ CANDIDATE FREEZE IN FORCE (U14 closed).** `v0.9.1-beta.1`
> sits at merge commit `34f7c88`; remote CI is green at that tip and the
> tag-triggered Release run `30294627200` produced the complete 9-asset draft
> release (unpublished). **Owner decision, same day:** the first tag run
> exposed three latent Release-workflow bugs (empty-but-set Apple env vars
> breaking ad-hoc codesign and forcing blank notarization; a draft-blind
> audit lookup that had never actually passed); the owner approved merging
> the proven branch fixes and moving the tag rather than deferring to U16.
> **Do not commit to `main` until U17 closes or the candidate is rejected** —
> branch and queue instead. Everything remaining is owner-lane: U15
> installed/listening gates → U16 release/updater transaction → U17 public
> activation. Details: `docs/plans/beta-go-no-go.md` §Candidate freeze and
> the U14 row in `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md`.

> **2026-08-25 — Owner decisions (in-session, Dan): listening gates approved;
> one final pre-launch check remains; mobile remediation unlocked.**
> 1. **Listening gates collapsed.** The owner has done meaningful listening
>    tests and approves the current sound. The separate owner-gated listening
>    threads — #4 (Manual Listening Gate), #5 (Reference Retune notes),
>    #6 (already-mastered matrix signoff), #7 (85%-lean distinctness confirm)
>    and Part B Q1/Q2 — are **closed as approved**, collapsed into **one final
>    by-ear check immediately before beta activation** (part of the U15
>    sitting). Do not treat them as open gates or re-queue them.
>    **Unchanged:** AC-5, Phase-B confidence, and album character stay gated
>    OFF — those are post-beta *calibration* sittings (D7), not launch gates.
> 2. **Space carve-out approved and shipped** (`167f066`): Space passes
>    through to checkbox/radio inputs (their only keyboard toggle); transport
>    everywhere else. Regression test verified red against the pre-fix handler.
> 3. **Single-key shortcuts (A / L / ?) ship as-is** for beta; a
>    disable/remap preference is post-beta polish.
> 4. **Mobile in-app remediation unlocked: U18 (iPhone) and U19 (Android)**
>    from the quality plan's parked chunk are now agent-lane work. U20
>    (mobile release evidence) stays parked; no distribution or parity claim
>    changes — desktop still ships first.

> **2026-08-20 — Adversarial-review follow-up (agent fixes shipped; owner
> clicks remain).** The confirmed findings from the 2026-08-20 review are
> fixed on `main` (loop-arm guard, npm + Rust advisory bumps in all three
> lockfiles, CI security-audit job, release-workflow hardening, go/no-go +
> README reconciliation — see `docs/CHANGELOG.md` 2026-08-20). **Owner
> actions still open, all free GitHub repo settings + one click each,
> recommended before the public beta:** enable Dependabot security alerts (+
> security updates), secret scanning with push protection, and private
> vulnerability reporting on the public repo (Settings → Code security);
> delete the four stale 07-27 draft releases; and note the hardened
> `release.yml` gets its proving run on the next tag/dispatch. Two feel
> decisions are queued in `docs/OWNER_INPUT_QUEUE.md` (Space-on-checkbox,
> single-key shortcut preference).

> **2026-07-03:** a hardening push is in flight on
> `harden/2026-07-03-hostile-input` — see
> `docs/plans/2026-07-03-hardening-plan.md`. It carries threads 11a, 11b, #13,
> the doc-accuracy checks (Part B 20/21), the preset-fingerprint harness idea
> (pairs thread #5), and RS-09 classification (Part B Q17) as riders, and it
> resolved Part B Q8 and (partially) Q11.

> **2026-07-05:** the owner's first hands-on smoke test (13 findings) was
> independently validated and the objective fixes shipped — see
> `docs/plans/2026-07-05-owner-smoke-fixes.md` for verdicts, refuted
> hypotheses, and the fix ledger. The design decisions it surfaced are Part B
> Q23–Q30 below.

> **2026-07-06:** a lean adversarial audit (July-push commits + audio/DSP hot
> areas + PRODUCT conformance + stabilization-risk; every finding
> skeptic-verified) confirmed 21 findings and the fix batch shipped same-day
> on main: pristine-process gate-default tripwires (a skeptic flipped the
> album-character gate ON and the whole suite stayed green — now impossible),
> one pinned decision point for play start position (F4 floor + loop-wrap
> confinement), cold-thread loop arm buffering, Mastered seek now keeps the
> newest live coefficients and restarts its LUFS meters like Original, album
> renders got the promised 1% progress throttle + per-stage timing line,
> above-stereo sources fold to stereo at decode (the audit's one HIGH:
> the stereo-native chain was mastering only a 5.1 file's front pair), and
> Help/comment copy drift was corrected. **The held items also shipped
> same-day:** the limiter bundle (ISP skip margin 1.2 → 1.26 with a
> coefficient-pinned test, warmup-aware ring indexing, denormal flush on
> envelopes/biquads) landed byte-safe — all 33 test binaries incl. the preset
> byte-identity snapshots and the fixture slow lane passed with NO snapshot
> regen needed — and the device-loss banner now emits on the backend LATCH
> edge, so a skipped stale mark shows no transient banner at all. Nothing
> from the 2026-07-06 audit remains open except Q26, which stays parked
> behind the 7a flip and is now tripwired by
> `src-tauri/tests/owner_gates_default.rs`.

> **2026-07-07 — Beta launch plan grilled & locked (Fable 5 session).** The
> decision-complete execution plan is `docs/plans/2026-07-07-beta-execution-plan.md`
> (strategy source: `docs/plans/2026-06-30-launch-plan.md`). Fifteen owner
> decisions (D1–D15) were locked; the load-bearing ones:
> - **D1** Beta = free public beta, **Mac + Windows together** (not Windows-first).
> - **D2** ~8-week timebox with a concrete flip date on the landing page; beta
>   users keep the $29 founder price.
> - **D3** Owner accounts (Apple Developer, Azure Trusted Signing, email provider,
>   GitHub Actions billing) start at payday (~2026-07-10); all secret-dependent
>   engineering is payday-gated, everything else proceeds now.
> - **D4** Email provider chosen at payday (Buttondown recommended); signup form
>   stays safe-disabled until wired.
> - **D5** Download flow = **ungated download button + optional email signup
>   beside it** — NOT email-gated. This **overrides** the "email-gated" phrasing
>   still in launch-plan §5/§7 (that phrasing is now stale; D5 wins).
> - **D6** Legal drafts are **not a beta gate** (owner researches independently).
> - **D7** One minimal owner listening sitting covers everything beta-blocking;
>   **AC-5, Phase-B gating, and album-character ship gated OFF in beta**, and are
>   calibrated post-beta. Keeps threads **#2, #3, 7a and Part B Q3/Q4/Q6 parked**.
> - **D8** Canon edits **A–G approved** (executed as beta-plan Slice 2) — closes
>   **#8** and Part B **Q7, Q9, Q10, Q11, Q12, Q13** (annotated inline below).
> - **D9** Mobile promise (owner's words): phones go live when the owner judges
>   them ready; they are Standard mode on phones by design and never fully mimic
>   desktop (4 presets, 3 loudness levels, fixed safe export). Closes Q7.
> - **D10** UX fixes **Q24** (per-track view memory), **Q25** (album subfolder),
>   **Q29** (autosave at analysis-complete) are in beta scope (Slices 3/4/5).
> - **D11** Technical calls confirmed: **Q27** export-during-export stays a hard
>   block + tooltip; **Q30** device-loss threshold stays 2 s; **Q16** min-window
>   is documented, not a layout slice; **Q23** width slider keeps the honest-Auto
>   treatment; **Q18** stereo_width stays Wave 10; **Q19** parked stays parked.
> - **D12** macOS ships as a universal binary; Intel-Mac smoke test is
>   non-blocking (the tolerance golden already removed Intel risk).
> - **D13** Beta version number = **0.9.0** (1.0.0 reserved for the paid flip).
> - **D14** Premium-parity UI pass is in beta scope (beta-plan Slice 9).
> - **D15** Owner verification devices: M4 MacBook Pro, current Windows box,
>   iPhone 16. No Android device — **Android stays parked**.
> - **D16 (owner, 2026-07-20)** The public beta must have a **$0 launch path**.
>   Apple Developer notarization and Azure Artifact Signing are post-beta trust
>   upgrades, not beta blockers. The beta may ship macOS ad-hoc / Windows
>   unsigned with clear Gatekeeper/SmartScreen guidance and release checksums.
>   The free Tauri updater signature remains mandatory. The download stays
>   ungated; optional email capture cannot block launch. This supersedes the
>   paid-account/email blocking parts of D3/D4.
>
> Post-beta parking lot (explicitly NOT this plan): paid-flip export gate +
> Lemon Squeezy activation, AC-5 / Phase-B / album-character calibration,
> Tier-1/Tier-2 adaptive follow-ons, mobile revival, Microsoft Store,
> Advanced/Studio tier.

> **2026-07-24 — Public Beta Quality Program opened; canon reconciled (U1).**
> The active forward queue is now
> `docs/plans/2026-07-24-001-feat-public-beta-quality-plan.md` (20 units in five
> sequential chunks, C1–C5, with a chunk status ledger inside the plan). U1
> landed the ground-truth reconciliation:
> - **Document precedence is now stated, not inferred.**
>   `docs/plans/2026-07-07-beta-execution-plan.md` is **executed history** — read
>   it for why D1–D16 were decided, not as an open queue.
>   The 2026-07-24 quality plan is the **active forward queue**.
>   `docs/plans/beta-go-no-go.md` remains the **live release gate**; the quality
>   plan feeds it rather than replacing it. Recorded in AGENTS.md/CLAUDE.md
>   Required Reading.
> - **`docs/OWNER_INPUT_QUEUE.md` created.** Owner-blocked questions accumulate
>   there and are answered in batches; an agent never stalls a chunk and never
>   invents an owner decision. Seeded with five open questions: founder-window
>   dates/terms, newsletter provider + consent/retention/sender, beta end date,
>   public announcement date/publication authorization, and whether
>   `docs/legal/` drafts ship as-is. **When one is answered it moves here** and
>   the queue row is struck through.
> - **`docs/CAPABILITY_EVIDENCE_MATRIX.md` created** — every public landing
>   claim, platform statement, pricing statement, and beta promise bound to a
>   named evidence source (KTD5/R5). The audit found 26 claims: 12 proved, 6 to
>   qualify, 1 to remove as unconditional (the `/releases/latest` download CTA —
>   there is no verified full release behind it today), 4 owner-blocked, 2
>   settled policies with no public surface yet. U5/U6/U7 execute the verdicts;
>   U1 only made the list exist.
> - **Four canon conflicts named and fixed** (documentation reconciliation only,
>   no product decision changed): the landing brief forbade the "coming to your
>   pocket" language the shipped page uses; the brief's Cross-platform pillar
>   claimed mobile in the present tense; AGENTS.md/CLAUDE.md still called
>   landing-page scope an open owner decision while their own Non-Negotiables
>   (and D16) already said it was in scope; and the two beta documents had no
>   stated precedence. The brief now carries an explicit **Mobile status**
>   section permitting exactly one date-free "not currently available" sentence.
> - **An exact-commit evidence ledger** (R17/KTD12) was added to
>   `docs/plans/beta-go-no-go.md` §9 rather than creating a fourth release
>   checklist. It also carries the candidate-freeze declaration; the freeze is
>   **NOT in force** (no candidate tagged).
>
> Nothing in this block changes an owner decision. Where a decision was genuinely
> absent, the conservative default was implemented and the question was queued.

> **State of the project (2026-06-22):** late stabilization. The two big mechanical
> queues (the final repo-wide review A1–E3, and the shippability roadmap S0–S7 +
> AC-1…AC-4) are **fully shipped on main**. What remains is almost entirely
> **owner-gated taste/listening work** plus a handful of explicit decisions. The
> macOS cross-platform golden verification is **resolved** (closed below; the
> 2026-06-23 audit replaced the per-OS exact-byte SHAs with an arch/OS-independent
> tolerance golden, so it can never spuriously fail again). One owner listening
> sitting could close most of the remaining listening gates at once.

---

## Part A — Open threads (loose-ends ledger)

### OPEN — actionable now

_None purely-mechanical — the macOS byte-identity SHA thread (former #1) is
resolved; see "Confirmed SHIPPED" below. Remaining threads are owner-gated._

### OPEN — owner-gated (listening / taste)

> Post-beta calibration only (D7). The formerly-open listening gates (old
> rows 4, 5, 6, 7) were **approved 2026-08-25** and now live under "Closed as
> approved" below — do not treat them as open, and do not retune presets
> without a new listening note.

| # | Thread | Source | Action |
|---|---|---|---|
| 2 | **AC-5 Adaptive Compressor calibration + default-gate flip** | adaptive-compressor-mvp-spec §5; HANDOFF_2026-06-13_AC5_CALIBRATION_PREP | Owner listening session. AC-1…AC-4 shipped; queue private fixtures, run OFF-vs-ON A/B, capture keep/adjust/reject per constant, land a single AC-5 flip commit (lock 9 constants, flip default, regen snapshots, update PRODUCT/APP_BEHAVIOR). Ship gate-OFF until then. |
| 3 | **Phase-B confidence gating** calibration + default flip | RELEASE_STABILIZATION Active Gates; AC-5 prep | Owner-gated; bundle into the AC-5 sitting (same fixtures/ears). Decide whether the default flips ON in a separate commit. No code change before listening notes. |
| 7a | **Album character system listening + flip decision** | 2026-07-03 hardening plan D7 | The album genre-inference system (per-track character labels → loudness pulls + EQ/width/warmth/intensity biases, `album.rs`) was gated OFF by default on 2026-07-03 (owner decision; it had never had a listening gate and silently altered tracks, contradicting the album promise). Owner A/B listening session decides whether it returns as a visible opt-in. Any flip commit removes or demotes the filename-keyword label override. |

### Closed as approved — 2026-08-25 owner listening signoff (history)

The owner listened across sessions and signed off on the current sound; these
four gates are **closed**, collapsed into one final by-ear spot-check on the
installed candidate immediately before beta activation (U15 / go-no-go §5).
Rows preserved for their dated history — they are not open work:

| # | Thread (closed) | Source | What the gate had asked |
|---|---|---|---|
| 4 | **Manual Listening Gate** (normal / already-mastered / long-source sweeps + clean-vs-warning export by ear) | Jump-Fix #1; deferred to Wave 10 | Owner by-ear pass incl. 8 kHz + 11.025 kHz sources; sweep Intensity/EQ/gain/compressor/Preview-LUFS/Volume-Match during playback; export clean + warning case, compare. |
| 5 | **Reference Retune listening notes** (Oomph least-matched) | Jump-Fix #2; deferred to Wave 10 | Re-run the private reference-tuning runner after the 85% lean, capture per-preset notes. The paired preset-fingerprint harness (shipped 2026-07-03) still mechanically gates any FUTURE retune. |
| 6 | **Already-mastered matrix listening signoff** | Jump-Fix #3; deferred to Wave 10 | Owner by-ear signoff against aggregate evidence post-85%-lean. |
| 7 | **"Characterless presets" regression check** | 2026-06-15 stabilization plan | Confirm Universal/Clarity/Tape/Oomph audibly distinct at matched loudness post-85%-lean. |

### OPEN — roadmap / scope (no listening required)

| # | Thread | Source | Action |
|---|---|---|---|
| 8 | **PRODUCT.md / APP_BEHAVIOR.md canon refresh (S5.4)** — Standard-first workflow, Mobile, Album Master, honest Adaptive wording | roadmap S5.4 | **CLOSED 2026-07-07 (beta plan D8)** — canon interrogation locked; PRODUCT.md edits A–G executed in beta-plan Slice 2. (APP_BEHAVIOR.md was not in the A–G edit set; adaptive wording was already reconciled 2026-07-06 `ab707f5` — reopen a doc-accuracy check only if it drifts.) |
| 9 | **Tier-1 adaptive follow-ons** (tilt-vs-reference brightness, density-cap reshape, stereo_width co-trigger, per-axis EQ floors, LRA→Option) | ADAPTIVE_DSP_NEXT_STEPS; roadmap S10.4 | Wave 10 taste work. Tilt-vs-reference is highest-value. Gate every constant change behind a listening note + fingerprint test. |
| 10 | **Tier-2 "smart" milestone** (measured-neutral per-preset, PSR/crest loop, corrective curve, reference matching, resonance/sibilance) | ADAPTIVE_DSP_NEXT_STEPS; roadmap S10.4/5 | Post-v1. B3 loss-budget + PSR transient protection are the most defensible first steps. See IDEAS_BACKLOG. |
| 11 | **Mobile store-readiness** (iPhone PrivacyInfo/real-progress/error-enum; Android signing+keystore; background-audio) | roadmap Wave 8 / S8.x | **Keystore-free batch EXECUTED 2026-07-04** (branch `feat/iphone-store-readiness`): S8.4 Android items verified already shipped (monochrome `ic_launcher.xml:5`, share/play intents `MainActivity.kt:352`, ext check `MasteringViewModel.kt:83`); S8.1 `ITSAppUsesNonExemptEncryption:false` added + required-reason API audit clean (only file-timestamp APIs, covered by the existing C617.1 manifest; no UserDefaults/boot-time/disk-space anywhere); S8.2 closed-as-shipped (indeterminate `ProcessingSpinner` + honest labels, `ContentView.swift:277-297`; no percent staging remains); S8.3a typed FFI error codes replace message sniffing (pinned both sides of the wire; `CommandError::with_context` preserves the class through analyze aggregation); S8.3b closed-as-shipped (`importTrack` bumps `landingGeneration` first, `AuditionController.swift:161-162`, and `refreshLanding` rejects stale generations `:431` — Android-guard equivalent already present); IP-05 leftovers: silent adaptive fallback structurally removed (live attach now fails like render), Documents-folder flags dropped (masters live in App Support; moving them to Documents is an owner product call — restore flags in that commit). **Still owner-gated: S8.5 Android signing (keystore), S8.6 background-audio (decision), S8.1 archive validation + Swift-side test run (Mac lane — CI billing currently blocked).** |
| 11a | **CSS styling-debt batch (audit D7, 10 items)** | 2026-06-23 audit Batch I | **9 of 10 CLOSED** (8 on 2026-07-03, `fdf1940`; `.wf-overview` on 2026-07-04, `119ffea`) — every rec re-derived against current CSS first, then applied and verified live in the browser preview (tokens resolve, served sheet correct, app shell intact; 492 FE tests green). The re-derivation caught both imprecise recs: `.toast` was fixed the corrected way (strip dead positioning, keep live material). **`.wf-overview` CLOSED 2026-07-04** (`119ffea`) — re-derivation showed no parent restructure was needed: the margin half of the paired offset is visually inert (stretch-aligned flex column, definite-width block box), so it was dropped and the width calc kept; visual A/B in the browser preview at the affected breakpoint (1440x900) measured a bounding box identical to the fraction of a pixel, and the base breakpoint untouched. **1 deferred with reason:** `.std-tile` is NO LONGER a verbatim `.tile` copy — it has intentional-looking drift (fluid clamp sizing, ✓ check chip, longer motion curves), so consolidation would regress Standard's visuals (owner-eye decision). |
| ~~11b~~ | ~~Dead-code tail (audit D4 leftovers)~~ | 2026-06-23 audit Batch H | **CLOSED 2026-07-03** — all six executed with test callers retargeted and full lanes (frontend, desktop Rust, iPhone check+tests, Android host+ndk): `profile_store::insert` deleted (tests use `set`); `LandingGainCache::get_or_compute` rewritten to delegate to the real `get`/`insert` (was re-implementing the hash dispatch); `DeepAnalysis.harsh_share`/`sibilant_share` + `harsh_sibilant_from_bands` deleted (per-window `harsh_31`/`sibilant_31` stay — Phase-B confidence consumes those); AdvancedPanel auto-label strings deleted; `shouldForceAdvancedOnStandardEntry` wrapper deleted (invariant tests compose the production pieces); iphone `cdylib` crate-type removed. |

### UNKNOWN — verify before acting

| # | Thread | Action |
|---|---|---|
| 12 | Does the shipped **Standard view** satisfy the original "Simple Mode" ask, or is a further-simplified mode still wanted? | Confirm with owner. Likely already-shipped-in-substance. |
| ~~13~~ | ~~review-2026-05-28 findings 8 & 15~~ | **CLOSED 2026-07-03** (hardening-push recon, read-only code verification): finding 8 is resolved — the session pill has three visually distinct states (`session-status-idle/live/busy` with distinct colors, halos, and animations, App.css + `SessionStatus` in App.tsx); finding 15 is resolved/justified in current `audio.rs`. No follow-up needed. |

### Confirmed SHIPPED (closed — listed so they aren't re-opened)

- **macOS byte-identity / cross-platform goldens (former thread #1).** The per-OS
  exact-byte SHAs (and the `deep_analysis` `[u32;16]` + `analysis` 6-band
  exact-equality goldens) were replaced by an architecture/OS-independent
  tolerance golden in the 2026-06-23 audit (Batch F), so an Intel Mac can no
  longer spuriously fail and no per-OS SHA regen is ever needed again. The
  earlier Apple-Silicon SHA recording (`cc03d56`/`88853dc`) is superseded.
- AdaptiveReadout debug-flag gating (`src/lib/debug-flags.ts`, default OFF).
- 2026-06-16 final repo-wide review queue (A1–E3, slices #1–#20).
- Shippability roadmap mechanical waves S0–S7 + AC-1…AC-4.
- Album channel-count parity (mixed mono/stereo + above-stereo fold-down).
- Realtime sweep confirmation gate (responsive sweep accepted; diagnostic counters removed).

---

## Part B — Owner decisions (flags)

Things the recon could not resolve without you. Grouped; **the preset/listening
cluster can mostly close in one sitting.**

### Presets & listening
1. Is the **85% preset lean** (commit `659bea5`) **final**, or still a candidate pending the macOS regen/listen?
2. Post-85%-lean, are Universal/Clarity/Tape/Oomph **audibly distinct at matched loudness**?
3. **AC-5** session: keep/adjust/reject each of the 9 `TBD-CALIBRATION` constants; flip the default gate ON?
4. In the same sitting, does **Phase-B `CONFIDENCE_GATING`** flip ON (separate commit)?
5. **Mode-pill label**: keep "Preset" or relabel to "Adaptive" in the calibrated UI?
6. **Eight presets vs a curated grouping** (UX-08) — change anything? (Needs a listening note first.)

### Product canon (S5.4 — needs an interrogation)
7. What is the **mobile** product promise (audience, scope, deliberate absences)?
   **CLOSED 2026-07-07 (beta plan D9):** phones go live when the owner judges
   them ready; Standard mode on phones by design, never fully mimicking desktop
   — 4 presets, 3 loudness levels, fixed safe export; no album mastering /
   advanced controls / custom delivery formats on phones. Canon edit C applied.
8. ~~What does **Album Master** promise beyond consistent loudness?~~
   **ANSWERED 2026-07-03:** *"One coherent record — consistent loudness, one
   delivery format, honest per-track receipts, nothing silently altered."*
   Arc = user-chosen expressive bonus. DDP/cue/ISRC/gapless out of v1.
   Override = full sound exemption (own settings + own target, album delivery
   format kept, manifest marks it). See `docs/plans/2026-07-03-hardening-plan.md` D4/D9.
9. How should the **adaptive engine** be described honestly in PRODUCT.md / APP_BEHAVIOR.md?
   **CLOSED 2026-07-07 (beta plan D8):** the Adaptive Mastering section was
   already reconciled to honest shipped wording (2026-07-06, `ab707f5`); the
   beta plan confirms no further rewrite is needed for beta.
10. What is the **marketing landing page's** product role (marketing-only vs download/onboarding), and should PRODUCT.md name it as a public surface?
    **CLOSED 2026-07-07 (beta plan D8, edit D):** the landing page is a supported
    product surface — marketing + ungated download hub + optional email capture
    (later checkout). PRODUCT.md Public Surface updated in beta-plan Slice 2.

### Agent-file scope
11. Is the **landing page / web build in-scope for agent work** (add `verify:landing` + `docs/landing-brief.md` to Required Reading), or hands-off like the Next.js storefront?
    **PARTIALLY ANSWERED 2026-07-03:** in scope for **security & verification**
    purposes (the beta-capture backend gets a security pass — hardening plan D3).
    The general-reading question was later resolved by the task-scoped reading
    policy below.
    **Security pass executed 2026-07-03:** finding — there is **no capture
    backend yet** (`src/landing/signup-config.ts` ships `SIGNUP_ENDPOINT = ""`;
    the form is safe-disabled; MCP-verified no YES Master Supabase project
    exists). Built-bundle grep: no secrets/stale endpoints. The form's safety
    posture is now pinned (`src/landing/BetaSignup.test.tsx`). When you pick a
    mailing provider (Buttondown/MailerLite/Kit per signup-config comments) or
    build a Supabase backend, re-run the hardening plan's Workstream F
    checklist against it before going live.
    **UPDATED 2026-07-20 (D16):** the landing page is in agent scope. The
    ungated GitHub Releases download is wired. Email capture remains optional
    and safe-disabled until a provider is chosen; it cannot block beta launch.
    **UPDATED 2026-09-05:** read landing guidance for relevant landing work;
    there is no blanket requirement to read it for unrelated engine tasks.
12. Broaden the "Local desktop app for Mac and Windows" non-negotiable to acknowledge iPhone + Android (and web)? *(Recommended yes — the CI already runs the mobile lanes.)*
    **CLOSED 2026-07-07 (beta plan D8, edit G):** yes — CLAUDE.md/AGENTS.md first
    non-negotiable broadened to acknowledge the CI-tested iPhone/Android bridges
    + landing page (desktop still ships first). *(Note: this non-negotiable was
    already broadened on 2026-07-06 `ab707f5`; edit G refines it and adds the
    launch-plan docs to Required Reading + confirms landing in agent scope.)*

### Packaging / platform
12a. **Path sandboxing (audit §14).** The desktop path guard intentionally rejects only `..` and ALLOWS absolute paths — required for the native-file-picker model (the user imports/exports arbitrary locations). Base-dir confinement / symlink-target rejection would break that, so it's a product decision: keep the current model, or introduce a sandboxed mode? (The mobile bridges ARE now `..`-guarded — audit §15.) Default: keep current desktop model.
13. Confirm **macOS build/install status** — add a parallel macOS-packaging release criterion to PRODUCT.md?
    **CLOSED 2026-07-07 (beta plan D8, edit F):** yes — PRODUCT.md
    Release-Candidate Meaning gains a macOS packaging criterion parallel to the
    Windows line. D16 later superseded the OS-signing requirement for the $0
    beta: ad-hoc/unsigned is acceptable with install guidance. The real-machine
    macOS confirm on the M4 remains owner lane (D15).
14. **Android signing/bundleRelease** is blocked on you providing a keystore — when?
15. Mobile **background-audio** behavior (UIBackgroundModes / foreground service) — v1 limitation or build it?

### Roadmap owner-decision queue
16. **Min window size** for 1366×768 laptops — document the requirement (cheap default) or schedule a layout slice?
    **CLOSED 2026-07-07 (beta plan D11):** document it, no layout slice —
    beta-plan Slice 6 documents the supported minimum in APP_BEHAVIOR.md and
    reconciles the Tauri min-size (was `minWidth 1440 / minHeight 860`, which
    exceeded a 1366-wide target).
    **RESOLVED 2026-07-08 (Slice 6 — experiment/verify/keep):** min-size lowered
    to **1360×740** and verified at 1366×768 in the browser preview — the desk
    tiles cleanly (sidebar/main/rail, no horizontal scroll) and every Standard
    control (preset tiles, intensity knob, Create Master) stays fully visible;
    only cosmetic bottom padding trims via `overflow: hidden` at the floor, no
    control is clipped. Kept and documented in APP_BEHAVIOR.md — no layout
    rework. (Advanced scrolls vertically on short viewports as it already did
    below ~1230 px.)
17. ~~**RS-09 limiter flush/tail** (~3 ms export-byte change) — defer + document (default), or accept?~~
    **RESOLVED 2026-07-03 — fixed, not deferred.** The 2026-07-03 DSP math
    audit confirmed (with adversarial verification) that every export was
    silently dropping its final ~3 ms (stuck in the limiter lookahead ring)
    and shipping a ~3 ms silent lead-in — an objective bug under the
    two-tier policy, not a preference. `MasteringChain::flush_render_tail`
    now drains the lookahead on both export paths; output keeps source
    length and sample alignment. Spot-listen entry in the hardening plan.
18. **Stereo_width disposition** — wire it as a width co-trigger or delete the inert carried field?
    **CLOSED 2026-07-07 (beta plan D11) — deferred-confirmed:** stays Wave 10;
    leave the carried field as-is for beta.
19. Confirm the **parked items stay parked**: P2 one-pole/soft-knee hoist + P4 tauri-specta (on the do-not-do list) — leave alone?
    **CLOSED 2026-07-07 (beta plan D11):** confirmed parked — P2 + P4 stay on the
    do-not-touch list.
    **UPDATED 2026-09-05:** the authorized performance investigation may reassess
    P2's hoist if profiling supports it and correctness/sound contracts are
    preserved. The historical deferral is not a ban on that investigation.
    This does not activate unrelated P4/tauri-specta or other parked backlog.

### Low-risk doc-accuracy checks (I can do these on request)
20. ~~`IPHONE_APP_OVERVIEW.md` preset vocabulary vs the shipped Standard 8-preset set~~ **CLOSED 2026-07-03 (verified, no drift):** the iPhone app deliberately ships a curated four-preset picker — `ContentView.swift:4-7` maps exactly Universal/Clarity/Tape/Oomph — and the doc describes that accurately. Not a drift from desktop's 8; a deliberate mobile subset.
21. ~~`ENGINE_REFERENCE.md` preset-calibration table predates the 85% lean~~ **CLOSED 2026-07-03 (verified, already fixed):** the table was regenerated from the shipped 85%-lean constants (noted in the doc itself, commit `659bea5`); spot-checked Oomph row (+5.30/−3.0/−2.6/−2.05, width 0.84, target −12.0) against `dsp.rs` `PresetCalibration` — exact match.

### 2026-07-05 owner smoke test (see `docs/plans/2026-07-05-owner-smoke-fixes.md` for full context)
23. **Width/advanced slider re-model (F10).** Your bipolar proposal (center =
    Auto, left = subtract, right = add) is sound but changes the wire meaning
    of `advanced.width` (old files with `0.0` legitimately mean mono) — needs
    an `advanced_schema_version` gate first. **Shipped meanwhile:** honest Auto
    thumb + `Auto · 1.11` readout + visible reset-to-Auto chip.
    *Recommendation: live with the honest slider first; re-model only if it
    still confuses.*
    **CLOSED 2026-07-07 (beta plan D11):** keep the shipped honest-Auto
    treatment; no bipolar re-model for beta.
24. **Per-track view memory (F6).** Spec: `view_by_track_id` in ProjectState;
    force-bounce to Advanced still happens for dirty tracks but stops
    overwriting the remembered view; only explicit choices persist.
    *Recommendation: implement as specced — say the word.*
    **DECIDED 2026-07-07 (beta plan D10) — IN PROGRESS:** implement as specced;
    beta-plan Slice 3.
25. **Album filename scheme (F13).** Title already reaches the backend;
    options: (i) `<Title>-NN-<stem>.wav` prefix, (ii) album-titled subfolder,
    (iii) both; also the continuous file + empty-title fallback.
    *Recommendation: (ii) subfolder.*
    **DECIDED 2026-07-07 (beta plan D10):** option (ii) album-titled subfolder;
    beta-plan Slice 4.
26. **Album-character width bias latent bug.** With the (gated-OFF) character
    system active and Width on Auto, `album_render.rs` reinterprets Auto as
    1.0 before adding the offset — silently discarding the preset baseline.
    *Recommendation: "Auto stays Auto" (skip the offset when width is None) +
    regression test, landed before 7a ever flips.*
27. **Export-during-export UX (F8).** Current hard-block is intentional.
    *Recommendation: keep, add a tooltip; a queue means overlapping ~GB
    render jobs.*
    **CLOSED 2026-07-07 (beta plan D11):** keep the hard block, add a plain
    tooltip; beta-plan Slice 5.
28. **Reorder arrows (F12).** Shipped hidden-until-hover/focus (keyboard
    reorder preserved). *Alternative: remove entirely (loses keyboard path).*
29. **Autosave immediacy (F5 tail).** The 7–10 s you measured is analysis
    latency + a 1.5 s debounce, not a slow save. *Recommendation: fire an
    explicit autosave at analysis-complete; small and harmless.*
    **DECIDED 2026-07-07 (beta plan D10):** fire an explicit autosave at
    analysis-complete; beta-plan Slice 5.
30. **Device-loss threshold.** Shipped at 2 s (40 ticks) because the code
    documents legitimate 1–2 s cold-decode stalls. If you want faster true
    loss detection on the flaky Focusrite, 1.5 s is the defensible floor.
    **CLOSED 2026-07-07 (beta plan D11):** device-loss threshold stays 2 s.

### Branding (parked)
22. **"Y.E.S. Master" / "Your Endgame Sound"** vs the current **"YES Master"** — a brand decision you parked. It cascades across PRODUCT.md, AGENTS.md/CLAUDE.md, README, and the landing copy. Until you call it, docs keep the current "YES Master" naming.

### 2026-09-05 website analytics boundary

- **OWNER DECISION:** Enable Vercel Web Analytics on the production
  `yesdsp.com` marketing site.
- This supersedes the earlier blanket landing-page prohibition on behavioral
  analytics only for aggregate public-site page traffic. The desktop app's
  no-telemetry promise remains unchanged.
- The implementation must fail closed outside `https://yesdsp.com` and
  `https://www.yesdsp.com`: no analytics in Tauri, `/app`, localhost, or Vercel
  preview deployments. Initial scope is page analytics only, with no custom
  product events or audio-derived data.
