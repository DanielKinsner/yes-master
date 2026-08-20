# YES Master — Changelog & Project Ledger

A concise, **milestone-level** record of what has shipped, newest first. This is
the "what happened" companion to:

- **docs/IDEAS_BACKLOG.md** — "what we could still build" (surfaced/forgotten ideas).
- **docs/OPEN_THREADS_AND_DECISIONS.md** — "what's still open / owner-gated."

Detailed handoffs, reviews, and execution records live under **docs/archive/**;
this ledger summarizes them so the corpus is scannable without opening 40 files.
Dates are milestone dates, not exact commit times.

---

## 2026-08-20 — Adversarial-review fixes: loop guard, dependency advisories, CI security gate, release hardening

A full-repo adversarial review (Codex; docs in `docs/reviews/2026-08-20-*`)
was triaged finding-by-finding and the confirmed items fixed:

- **Loop bug (real):** the `L` shortcut could arm a loop with no region drawn
  — the button is disabled in that state but the keyboard path bypassed it,
  and drawing a region later silently started looping. `toggleLoop` now
  requires a region to arm (disarm always allowed) and rolls the optimistic
  flag back if the backend rejects the change. The old test encoded the bug;
  replaced with the intended contract + a rollback test.
- **Dependency advisories cleared:** npm `brace-expansion` → 5.0.9 and
  `nanoid` → 3.3.18 (both dev-only, `npm audit` now 0 at every level);
  `plist` 1.9.0 → 1.10.0 / `quick-xml` 0.39.4 → 0.41.0 (RUSTSEC-2026-0194/5,
  DoS, low exposure — Tauri plumbing, no user-XML path) in **all three**
  Cargo lockfiles (desktop, iPhone, Android). All lanes re-run green.
- **CI security gate added:** a `security-audit` job fails on high+ npm
  advisories and on actual RustSec vulnerabilities in the three lockfiles
  (cargo-audit 0.22.2 pinned; unmaintained-crate warnings warn, not fail).
  CI could previously stay green after a new advisory published.
- **Release workflow hardened:** workflow token dropped to `contents: read`
  with job-level write only where the draft is created/audited; every action
  SHA-pinned (the workflow holds the updater signing key); Azure secrets
  scoped to the Windows leg; `artifact-signing-cli` pinned `0.11.0 --locked`.
  Next draft run doubles as the proving run.
- **Docs reconciled:** go/no-go top boxes no longer show superseded 0.9.1
  evidence as current (banner + reset candidate boxes, history kept); README
  updated to the 0.9.2 candidate and the active plan.
- **Not fixed on purpose** (owner feel decisions, queued in
  `docs/OWNER_INPUT_QUEUE.md`): Space-on-checkbox carve-out and a
  disable/remap preference for the single-key shortcuts. The review's framing
  of the global Space transport as a defect was rejected — it is the
  Phase 12.1 owner-requested DAW behavior.

## 2026-08-19 — Candidate `v0.9.1-beta.1` superseded; version 0.9.2

Owner decision: the 2026-07-27 candidate no longer represents `main` (~70
commits of feature/design work landed after the tag), so it is superseded
and the candidate freeze is lifted until the next tag. Version bumped to
**0.9.2** in the three desktop manifests, the three Cargo.locks, the preview
mock build stamp, the lockfile and the bug-form placeholder
(`version-coherence.test.ts` green). Next: install + the one listening
sitting on current `main`, re-run U14's gates, re-tag `v0.9.2-beta.1`, delete
the stale 07-27 drafts on GitHub. Recorded in `docs/OPEN_THREADS_AND_DECISIONS.md`
and `docs/plans/beta-go-no-go.md` §Candidate freeze.

## 2026-08-19 — Pass 4: moments & onboarding

- **Keyboard**: ← / → seek 5 s (Shift: 30 s), Home jumps to start, **A**
  flips Original/Mastered, **L** toggles the loop (Advanced only), **?**
  opens a Shortcuts overlay. One catalogue (`lib/shortcuts.ts`) feeds the
  handlers, the overlay and Help, so the overlay can never advertise a dead
  key; all shortcuts yield to text entry and the seek/A/L keys also yield to
  focused knobs / number fields / selects.
- **"Try a demo track"** beside Import in the empty state: the engine
  synthesises a 24 s loop (Am–F–C–G pad, bass, drums; mixed quiet, peaks
  −8 dBFS so the master has room to move) once into app-data
  `demo/yes-master-demo.wav` and imports it (`demo.rs`,
  `prepare_demo_track`). A bundled CC track can replace the generator later
  without changing the contract.
- **Standard per-card reset**: a quiet ↺ on each step label — Style →
  Universal, Intensity → 50%, Loudness → Low — disabled at the default.
- **Export Complete card**: visible **Show file** + **Done** actions; the
  preview mock now carries delivered measurements so captures show the
  Results section; the Mastering-target chip is neutral (fact, not warning).
- Settings/Help (one `ChromeDialog`) and toasts (one `Toast` + stack) were
  already consistent — verified, no change.

## 2026-08-19 — Pass 3: colour & state semantics

- **Amber rule**: on the console chrome amber means EQ/bipolar *cut* and
  nothing else; warnings keep amber only inside warning surfaces (export
  gate, receipt rows, concern chips). Source Insight's REVIEW pill is a
  neutral **New** status chip; the compressor hint's rule is neutral.
- **Edited markers**: Tone Shape, input/output gain and compressor knobs
  show a small dot beside the label while off their default (`Knob`
  `editedIndicator`; Intensity deliberately not); the active Styles tile
  says **edited** when Advanced edits sit on top of the style.
- Cobalt-for-live already held for the loudness picker and tiles (raised
  neutral / preset accent) — verified, no change.

## 2026-08-19 — Stuck-analysis root cause, per-track "analyzing", deep-scan progress, header rhythm

- **Root cause of "analysis running for minutes on one small file"** (owner
  report): the session-restore path skipped `finishAnalysis` when its effect
  had been cancelled mid-flight (Fast Refresh re-running App's effects in
  `tauri dev` does exactly that), so the batch stayed in the in-flight set
  for the life of the session and the pill/card said "analyzing" over a
  finished result. Measured: the Rust analysis of a mono 48 kHz 1:51 file
  takes 0.7 s release / 0.9 s debug. Fix: every begun batch is finished
  unconditionally (source-pinned in `useTrackMaster.integration.test.tsx`).
- **"Analyzing" is per track**: `isAnalyzing` is true only while the
  *selected* track is in an in-flight batch (`isAnyAnalyzing` remains for
  the stage timer and the analysis-complete autosave). Another track's
  batch no longer lights the pill/Insight card/waveform slot.
- **Deep-scan sub-progress**: the costliest stage now emits 0.80→0.98 as
  the windowed scan runs (`scan_windows_with_progress`; output identical,
  snapshots green, iPhone + Android lanes green), so the bar moves instead
  of parking at 80%.
- **Track header rhythm**: a touch more air under the title; the meta row is
  one quiet line; the session pill sits at text height with no fill (the
  dot carries the state); INSIGHT's label shares the title's left gutter.

## 2026-08-19 — Pass 2: hierarchy, density, laptop size

- **Analysis progress has two owners, not four**: the header SessionStatus
  pill (coarse, with progress) and the waveform slot (rich). The sidebar
  footer line and the bottom bar's "Processing · …" pill are gone; the
  bottom bar is live meters only (pinned by a count test in
  `App.playback-kind.test.tsx`).
- **Flow layout (≤1279 wide or ≤819 tall, i.e. the 1360×740 floor)**: title +
  meta row take the full width and the preview toolbar sits beneath, so
  Album Master's row (identity · Follow/Override · pill) is one line and
  Track/Album headers land the waveform at the same Y. The Follows/Overrides
  status chip is assistive-tech-only at every size (the segmented control
  states the choice).
- **Signal-chain stages jump to their controls**: EQ / Warmth / Air / Comp /
  Width / Saturation / Limiter are buttons that scroll the owning section
  into view and flash it (`STAGE_JUMP_TARGETS`, `#jump-*` ids); Source stays
  a plain node.
- **Rail density**: compressor advisory + adaptive guidance are hairline
  notes (2px rule, no box); per-band plan rows + preset summary read as a
  compact table; sidebar "Overrides" is neutral, not warn-coloured.
- **T4 type rung** 0.6 → 0.64rem (captions/pills/subtext) for laptop
  legibility.

## 2026-08-19 — Owner tune of the alive pass + Standard-return gate for Advanced-only styles

- **Quieter**: preset tile hover ~40% as dramatic; knob hover glow + drag
  effect ~30% less; MASTER OUT bars +6% saturation only while Mastered is
  audible.
- **One heading rung**: Standard's step labels / rail titles / TRACKS now sit
  on Advanced's T1 rung (size, weight, white). The ✓ chip on Standard's
  selected Style tile is gone — border/glow/tinted label already say it.
- **Advanced→Standard with an Advanced-only style** (Spatial / Warmth /
  Punch / Loud / custom) now goes through the Back-to-Standard confirm
  (it used to land on "no style selected"). The dialog names the style and
  says it becomes Universal at the same intensity/loudness; Reset all on
  the Advanced rail still keeps the style (`needsStandardReturnReset` /
  `resetForStandardReturn` in `lib/standard-managed.ts`, pinned by
  `App.transitions.test.tsx` 7b).

## 2026-08-19 — Alive pass 1: A/B flip moment, meter ballistics, live GR, console motion

Plan: `docs/superpowers/plans/2026-08-19-ui-alive-pass-1.md` (Passes 2–4
roadmap in its appendix). Presentation only — no audio-path, playhead,
render, or export semantics changed; the L10 audio crossfade on A/B is as it
was.

- **A/B flip lands**: the selected side of Original/Mastered blooms once on
  flip; while Mastered is audible the waveform's played span runs brighter
  (the `.app` root now carries `data-playback-kind` / `data-playing`,
  pinned by `App.playback-kind.test.tsx`).
- **MASTER OUT ballistics**: rate-limited fall (24 dB/s) + peak-hold pips
  (1 s hold, 12 dB/s decay; red above −1 dBFS). Pure helper
  `lib/meter-ballistics.ts`, unit-tested; `hooks/useMeterBallistics.ts`.
- **Live per-band gain reduction** in the Per-band Compressor card while the
  master plays (`lib/gain-reduction.ts`); the playback tick had carried
  `gr_*_db` since Phase 12.2 with no surface.
- **Console motion**: rail + sidebar arrive with the deck on entering
  Advanced; a clean export draws its check; Standard's ✓ pops. Every effect
  is registered in `App.delight.test.tsx` (reduced-motion opt-out, no reflow
  properties, named purpose).

## 2026-08-19 — Bipolar knobs, global Advanced reset, one type hierarchy

- **Bipolar boost/cut knobs**: Tone Shape Low/Mid/High and the rail's
  Input/Output gain knobs grow their arc from the centre and follow the
  Visual EQ's signed palette — blue above 0 dB, amber below, neutral at rest
  (`Knob` `bipolar`; `--eq-boost` / `--eq-cut` tokens pinned to the TS
  constants by `Knob.bipolar.test.tsx`). Visual EQ nodes are one size.
- **Reset all** on the Advanced rail (Track and Album Master): one header
  action puts every non-managed control back to its first-open state
  (`resetToStandardManaged`), one undo step, behind a confirm dialog; style /
  intensity / loudness target / delivery format stay.
- **Insight review**: opening the INSIGHT disclosure acknowledges the analysis
  (the REVIEW pill disappears as it opens); clicking the pill opens it. The
  in-panel "Mark reviewed" / Unreviewed affordances are gone.
- **Type hierarchy**: the rail's three stacked density override layers
  collapsed into one tokenised block with four rungs (heading / control /
  value / subtext); deck section labels and Tone Shape knob titles sit on the
  same rungs as the rail; Preset Density uses the feel-control slider layout.
- **Visual EQ at any aspect**: the panel's viewBox width now follows its
  rendered aspect ratio (height fixed in user units), so labels, nodes and
  strokes are no longer smeared horizontally on wide decks (4K); the drag
  torture script normalises node x to the plot area accordingly. Rail dials
  54px; played waveform span softened ~25%; the 1360x740 flow layout clamps
  the deck dials so Intensity no longer runs under Tone Shape.
- **Album header fix**: at console sizes under 1700px the two preview toggles
  stack so Album Master's one-line badge row no longer runs under the
  Original/Mastered switch; the waveform card's Y is unchanged in both modes.

## 2026-08-18 — Premium design pass, blue/amber dynamic EQ, movable EQ bands

- **Desktop shell restyle** (`design/premium-pass`, merged): hairline seams,
  top-lit graphite surfaces, cobalt accent reserved for what is live, raised
  neutral segmented controls, gradient waveform with a lit played span,
  refined buttons/chips/selects/meters/overlays, one staggered console reveal.
- **Dynamic EQ redraw**: boost is blue above 0 dB, cut is amber below (two
  clipped copies of the same curve), hollow ring nodes coloured by role.
- **Landing redesign**: owner-supplied 4K console hero, app-matched tokens,
  timeline workflow, framed captures, real preset artwork, definition-list
  terms; every July claims test still pins the copy. Headline is now
  "Master your track in real time. Hear every move. Ship with proof."
- **EQ bands movable in frequency** (`feat/eq-variable-bands`): each of the
  seven bands' Hz is a setting (`MasteringSettings.eq_bands`), clamped by the
  engine to a per-band window; defaults equal the historical constants so
  presets, projects and byte-identity snapshots are unchanged. Node drag =
  gain + frequency in one mutation (one undo step); double-click resets both.
  Rust/TS goldens re-blessed; iPhone + Android bridges rebuilt and tested.

## 2026-07-08 — Export receipt rebuilt into the "Export Complete" card

Owner-approved premium slice (an explicit exception to the UI-polish freeze).
The Track Master export receipt (Advanced view) was rebuilt into the
"Export Complete" card design, frontend only, in eight small commits — one per
section. Everything is wired to data the app already has: nothing invented,
nothing hardcoded.

- **Track** — display name + the 13c metadata-diet identity line
  (duration · format · sample rate · channels).
- **File saved** — filename + full path with a copy-to-clipboard button
  alongside the existing reveal-in-file-manager action.
- **Mastering style** — the preset's own orb art (same PNGs as the Styles
  tiles) + its real label/blurb, beside a radial intensity dial showing the
  real percentage and the real intensity label (50% is "Moderate", not the
  mock's fictional "Aggressive"). `PRESET_OPTIONS` moved to
  `lib/preset-copy.ts` as the single source for that copy.
- **Results** — Integrated loudness / True peak / **Dynamic range** from
  measured render values; the old row was mislabelled "LRA" but the engine
  measures dynamic range, not EBU Loudness Range. Mastering Target chip = real
  delivery-profile name + target LUFS.
- **Quality check** — the receipt's honesty surface: source measurements with
  per-axis icons driven by the REAL export checks. A warned/critical export
  can never render as an all-green card; format-only checks (bit depth, sample
  rate) surface as their own honest rows. Logic in a pure, unit-tested helper
  (`lib/receipt-quality.ts`).
- **Audio format** — bit depth · sample rate · file type (channels omitted:
  the delivered channel count isn't measured on the receipt).
- **Footer** — export timestamp + real build stamp (version · git hash · build
  time) from the `build_info` command; no external link, no fabricated engine
  version.
- **Layout** — the mock's two-column premium card, responsive (collapses to one
  column on narrow windows), verified at 1920×1080 and 1360×740.

No Rust changes: the card is fed by props from the render site
(selectedTrack / selectedSettings / selectedAnalysis) rather than widening the
export-receipt payload.

## 2026-07-04 — Local diagnostics (log + save-a-report)

The first "product meets the world" slice from the blind-spot review: the
backend previously had no logging at all (two eprintln sites), so field
failures were invisible. Now: a rotating local log (~2 MiB cap; startup,
panics, decode/render/session failures, device loss — never hot paths), a
"Save diagnostics report" button in Help (one plain-text file: version/OS,
log tails, session summary), and the #1 support case made explainable —
a discarded autosaved session now logs WHY it was skipped. Privacy stance
unchanged and stated in the report itself: no telemetry, nothing leaves
the machine unless the user saves and shares the file.

## 2026-07-04 — Hardening follow-ups + iPhone store-readiness batch

Same-day continuation of the hardening push (branches
`harden/2026-07-04-loop-region-followup`, `feat/iphone-store-readiness`):

- **loop_region hostile-IPC fix.** The D5 "plausible" finding confirmed real:
  the audio thread never cleared an armed loop region on track switch — only a
  racing fire-and-forget frontend IPC message did. The clear now rides on the
  Play/Stop commands themselves (same-track O/M swaps and arm-then-play keep
  the region); policy pinned by three tests.
- **`.wf-overview` offset closed (ledger 11a → 9/10).** The margin half of the
  paired 30px offset proved visually inert; browser-preview A/B measured a
  bounding box identical to the fraction of a pixel. Only `.std-tile` stays
  parked (owner-eye).
- **iPhone store-readiness (Wave 8, keystore-free).**
  `ITSAppUsesNonExemptEncryption:false` declared; required-reason API audit
  clean against the existing privacy manifest; progress honesty (S8.2) and the
  stale-landing guard (S8.3b) closed as already-shipped with evidence; typed
  FFI error codes replace message sniffing (S8.3a — pinned on both sides of
  the wire, with `CommandError::with_context` preserving the error class
  through analyze aggregation); the silent adaptive-context fallback is
  structurally removed (live attach now fails like render); the untruthful
  Documents-folder flags dropped. Swift halves compile/run on the mac CI lane.
  Remaining owner-gated: Android signing (keystore), background-audio
  (decision), archive validation (Mac). **Note: CI has been billing-blocked
  since 2026-07-04 — every lane, including the first macOS run of the
  fingerprint tolerance goldens, needs a re-run once GitHub billing is fixed.**

## 2026-07-03/04 — Hostile-input hardening push + D5 follow-ups

A grill-driven hardening sweep (plan + owner decisions:
`docs/plans/2026-07-03-hardening-plan.md`) executed in ~50 small commits,
followed by two Codex-built feature slices reviewed/merged 2026-07-04.
**Five real bugs found and fixed**, each with a proving test:

- **RS-09 — limiter tail truncation.** Every export silently dropped its final
  ~3 ms and carried a ~3 ms silent lead-in; a chain flush now keeps output
  sample-aligned with the source.
- **Unloadable-session save.** A `1e39` in a `.ams.json` became `+inf` in an
  f32, re-serialized as `null`, and made every later save/autosave a file the
  app could never reopen (silent session loss). Loads clamp raw floats to
  f32-finite; saves verify read-back before renaming into place.
- **Never-overwrite finalize race.** Overlapping exports to the same name
  clobbered the earlier render at finalize; the writer now diverts to a
  `__{n}` sibling and reports the actual path.
- **True-peak ceiling breach.** High-crest material at hot delivery targets
  shipped up to +1.68 dBTP over the profile's −1 dBTP promise (limiter caps
  sample peaks; inter-sample peaks sailed through, and the landing only
  ceiling-bounded boosts). The landing now applies a level-only safety trim in
  both directions; 27-cell landing matrix pins the contract.
- **Hostile-decode panic.** A crafted 0 Hz WAV crashed symphonia itself; a
  panic boundary + rate validation + NaN/Inf sample sanitization now guard all
  decode entry points.

Also shipped: album promise proofs (loudness consistency, manifest
truthfulness, no-partial-output incl. pass-2/manifest failures, named hostile
errors); Override = full sound exemption end-to-end with honest receipts; the
album character system gated OFF pending owner listening; adaptive-ruin
property proofs (reduce-only/capped/floored, incl.
`intentional_brightness_is_never_buried`); the **preset fingerprint harness**
(safety bounds at intensity 0.5 AND 1.0, pairwise distinctness floor across
all 28 pairs, OS-independent tolerance golden, owner MD/CSV report); hostile
project-JSON + export-I/O batteries; orphaned render-tmp startup sweep; a
capture-endpoint security pass (finding: no backend exists yet — form pinned
safe-disabled, go-live checklist parked); dead-code tail (11b) and CSS
styling-debt batch (11a, 8/10, browser-verified); D5 runtime-abuse review
(33 abuse patterns verified clean). **Codex follow-ups:** export cancellation
(job registry, `cancel_render`, job-scoped progress, cancelled = no output)
and playback device-loss surfacing (ticker stall detector,
`playback:device-lost`, recovery banner). Owner-gated remainder: Wave-10
listening sittings (3-entry Spot-Listen Queue in the plan doc).

---

## 2026-06-23 — Repo-wide audit remediation (macOS-weighted)

Executed the 2026-06-23 audit plan
(`docs/reviews/2026-06-23-repo-wide-audit-macos-focus.md`), Batches A–H, in
small pushed commits with per-fix TDD and CI verification. All behavior-
preserving except the explicit Batch C bug fixes; no preset/DSP voicing changed.

- **Cross-platform goldens made arch/OS-independent (§1).** Replaced the per-OS
  exact-byte preset SHAs + the `deep_analysis` `[u32;16]` and `analysis` 6-band
  exact-equality goldens with a single reference compared within a tight
  tolerance (decimated `max_abs_delta < 1e-6` for the chain output; scale-aware
  value tolerance for the metrics). An Intel/x86_64 Mac now passes by
  construction — no per-OS SHA regen ever needed again. Real-time DSP untouched;
  confirmed green on macOS-arm64 CI and still fails on a synthetic drift.
- **Live-audition correctness (Batch C).** §3 `removeTrack` no longer leaks a
  stale playing/meter/loop state onto the auto-selected sibling; §4 a live
  delivery-profile/ceiling change now reaches the audition limiter (was clamping
  to the OLD ceiling until restart — broke "A/B matches export"); §5 an
  Album↔Track switch mid-audition re-pushes the live chain.
- **Stabilization tripwire tests (Batch A)** pinning the non-negotiables:
  adaptive-compressor gate OFF by default, Volume-Match export-level invariance,
  compressor-Off scope, VM-off default, and the wav write-primitive contract.
- **Wire-drift gate hardened (Batch B):** recurse into nested struct types and
  register the previously-ungated wire types (`ProjectState`, album render
  payloads, `ExportReport`/`QualityCheck`/`UserPreset`, device/waveform).
- **Small correctness (Batch D):** concurrent analysis-batch progress no longer
  dropped; "Export Anyway" honors the export re-entrancy guard; Android native-
  JSON Gson parses are crash-guarded.
- **Security hardening (Batch E):** the shared mobile facade now rejects `..`
  traversal on `output_dir`/`source_path` before `create_dir_all`/decode (closed
  a sandbox-escape on both bridges); desktop path contract pinned + Swift
  VolumeMatch finite-guard.
- **Doc drift (Batch G):** landing style names corrected to Universal/Clarity/
  Tape/Oomph; `ENGINE_REFERENCE.md` preset table + worked example regenerated
  from the shipped 85%-lean constants; the macOS-SHA open thread closed.
- **Dead-code delete (Batch H):** 3.2 MB of unreferenced landing PNGs + verified-
  dead Rust/FE/mobile symbols removed (4 list items were found genuinely used
  and kept).

## 2026-06-22 — Preset 85% lean + docs hygiene

- **Presets re-voiced to the "85% lean."** All 8 character presets
  (universal / clarity / tape / spatial / oomph / warmth / punch / loud) set to
  `baseline + 0.85·(research delta)`; `custom` left untouched. This backs off the
  earlier full-100% pass after listening. Windows byte-identity snapshots
  regenerated; Windows desktop build rebuilt and installed. (`659bea5`)
- **macOS verification handoff** added — `docs/HANDOFF_2026-06-22_MACOS_VERIFICATION.md`.
  The 7 macOS snapshot SHAs were initially Windows placeholders (`3d6818a`);
  real Apple Silicon SHAs landed in `cc03d56`, and the follow-up deep-analysis
  per-OS fixture bits landed in `88853dc`.
- **README** sharpened (local-first, "no black box" framing). (`8d2e20a`)
- **Docs hygiene pass**: 31 completed/superseded docs archived under
  `docs/archive/` with status banners; this ledger system (CHANGELOG /
  IDEAS_BACKLOG / OPEN_THREADS_AND_DECISIONS) added. (`24fcb07`)

## ~2026-06-18 → 06-22 — Public marketing landing page

- Shipped a public **landing page** (`src/LandingPage.tsx` / `.css`); the desktop
  binary routes a browser visit to it; **Vercel** deploy configured
  (`vercel.json`, `.vercelignore`); responsive verification lane
  (`npm run verify:landing`). Source-of-truth copy in `docs/landing-brief.md`.

## 2026-06-16 — Final review queue + album parity

- **Final repo-wide review implementation** complete — Waves A–E, slices #1–#20,
  all landed with commit SHAs.
  (`docs/archive/plans/2026-06-16-final-review-implementation-plan.md`)
- **Album channel-count parity** shipped: mixed mono/stereo resolution + above-
  stereo fold-down to stereo delivery, with delivery-profile parity tests.
- The three listening gates (manual listening, reference-retune, already-mastered
  matrix) were **deferred to Wave 10** (owner-gated) — see OPEN_THREADS.

## 2026-06-15 — Owner listening/UX triage

- Owner listening/UX triage (L1–L15) captured and largely executed on the
  mechanical-tail branch.
  (`docs/archive/LISTENING_FINDINGS_2026-06-15.md`)

## 2026-06-12 — Shippability planning + adaptive-compressor spec

- **Public-release planning**: master shippability audit (P0–P3 registry) +
  shippability roadmap (Waves 0–10) + desktop / iPhone / Android shippability
  plans. **Desktop-first v0.1.0** decision.
- **Adaptive Compressor MVP** spec written (per-band, reduce-only, gated rollout
  AC-1…AC-5). Plumbing (AC-1…AC-4) landed but the feature is **gated OFF** pending
  the owner ear-calibration session (AC-5).
  (`docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md`)

## 2026-06-08 → 06-11 — Standard view + adaptive Tier-1 accepted

- **Standard view** shipped and stabilized (the default, simplified workflow:
  pick Style + Loudness + Intensity → Create Master, fixed 44.1 kHz / 24-bit
  WAV at −1 dBTP).
- **Adaptive DSP Tier-1** (reduce-only guardrails, one Adapt Strength dial,
  per-axis trim caps) accepted by owner listening (2026-06-11).
- **Refactor backlog** (19 items) executed in full (2026-06-09).
  (`docs/archive/reviews/2026-06-10-consolidated-refactor-backlog.md`)

## 2026-06-02 → 06-04 — Adaptive DSP Tier-1/Tier-2 build

- Tier-1 guardrails designed, reviewed, and merged; Tier-2 Phase-A (additive/
  neutral) landed and **Phase-B confidence gating built but OFF by default**,
  pending owner calibration.
- Deep-analysis (per-window scan) computed but **not yet wired into the sound**.

## 2026-05-27 → 05-29 — Release-candidate finish + reviews

- Track Master stabilization (import / analyze / audition / export), overwrite
  protection, already-mastered fixture matrix, and a stack of adversarial master
  reviews — all consolidated and actioned.
  (`docs/archive/RELEASE_EVIDENCE_2026-05-28.md`,
  `docs/archive/reviews/2026-05-29-master-review.md`)

---

### Still in flight (not shipped)

See **docs/OPEN_THREADS_AND_DECISIONS.md** for the live list. Headlines:
the AC-5 adaptive-compressor listening session + Phase-B gate flip, the three
Wave-10 listening signoffs, the PRODUCT.md S5.4 canon refresh
(mobile/album/adaptive wording), and mobile store-readiness.
