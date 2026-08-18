# YES Master — Changelog & Project Ledger

A concise, **milestone-level** record of what has shipped, newest first. This is
the "what happened" companion to:

- **docs/IDEAS_BACKLOG.md** — "what we could still build" (surfaced/forgotten ideas).
- **docs/OPEN_THREADS_AND_DECISIONS.md** — "what's still open / owner-gated."

Detailed handoffs, reviews, and execution records live under **docs/archive/**;
this ledger summarizes them so the corpus is scannable without opening 40 files.
Dates are milestone dates, not exact commit times.

---

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
