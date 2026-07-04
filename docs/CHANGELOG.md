# YES Master — Changelog & Project Ledger

A concise, **milestone-level** record of what has shipped, newest first. This is
the "what happened" companion to:

- **docs/IDEAS_BACKLOG.md** — "what we could still build" (surfaced/forgotten ideas).
- **docs/OPEN_THREADS_AND_DECISIONS.md** — "what's still open / owner-gated."

Detailed handoffs, reviews, and execution records live under **docs/archive/**;
this ledger summarizes them so the corpus is scannable without opening 40 files.
Dates are milestone dates, not exact commit times.

---

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
