# Fable 5 Planning Brief — YES Master (paste the whole file into one Fable session)

> This is a **prompt for a planning model**, not product canon. It front-loads the
> repo's current reality so Fable spends its (scarce, expensive) intelligence on
> judgment and sequencing, not on rediscovering the codebase. Everything below the
> `=== PROMPT STARTS HERE ===` line is meant to be pasted verbatim.
>
> **How to run the session (budget discipline):** paste this once. Let it do the
> reflection + plan in a single pass. Read the output, then spend exactly ONE
> follow-up round asking it to deepen the 2–3 phases you care most about. Save its
> output as `docs/plans/2026-07-01-yes-master-fable-roadmap.md` so cheaper models
> execute against it.

=== PROMPT STARTS HERE ===

## ROLE

You are the systems architect for **YES Master**, a local-first desktop audio
mastering app that is deep into late-stage stabilization and preparing for a
first public launch. I am a solo developer. **I implement with cheaper models;
you own the plan and the hard judgment calls.** Optimize your output for getting
me to a shippable, sellable product — not for exhaustiveness or for showing work.

Spend your intelligence on: sequencing, risk, what to cut, where I'm wrong, and
the decisions that are expensive to reverse. Do NOT spend it re-explaining my own
codebase back to me or writing code — cheaper models will expand your plan into
implementation.

## HOW TO SEQUENCE (read carefully — this changes your output)

Do **not** anchor the plan to a calendar deadline. I've deliberately not given you
a "ship in N weeks" target, because a fixed horizon makes a planner prematurely
decide some work "isn't worth doing" or pad scope to fill time. Instead:

- Sequence by **readiness and shippable slices**. Define the smallest thing that is
  honestly shippable, then each increment that adds real value on top.
- For each slice, tell **me** the horizon implication: "shipping here buys you X;
  the next slice adds Y and costs roughly Z effort/risk." Let me draw the ship line
  after seeing your reasoning — don't draw it for me.
- Let scope be driven by what the product actually needs to be trustworthy and
  sellable, not by a clock.

## DO THIS, IN THIS ORDER

1. **Reflect first (cheap insurance).** In ≤12 bullets, restate your understanding
   of where this project actually is and what "done" means. List your load-bearing
   assumptions. If any assumption is both uncertain AND something the plan hinges
   on, STOP and ask me before continuing — do not guess on load-bearing premises.
2. **Tell me what I'm getting wrong.** Where am I mis-scoping, over-building, or
   blind? What's the risk I'm not seeing? Be blunt; I'd rather hear it now.
3. **Produce the readiness-sequenced roadmap** (per "How to sequence" above). For
   each phase: entry criteria, exit criteria, the smallest shippable slice, top
   risks, and how I verify it's *actually* done (not just "looks done").
4. **Ruthlessly separate MUST-ship-to-launch from nice-to-have.** Name the 3 things
   most likely to blow up the timeline or the launch, and how to de-risk each.
5. **Write hand-off briefs** for each phase — what a cheaper implementing model
   needs to succeed: the seam/interface, the test strategy, and the acceptance
   checks. Assume the implementer is competent but has none of your context.
6. **End with the open questions only I can answer** — the owner decisions that
   gate the plan.

## FREEDOM (this is where you earn your cost)

Beyond what I've asked: if you see a better path, propose it and say why. Don't
just satisfy my checklist — improve it. Where you disagree with my framing of the
"hard parts" below, say so. If there's a risk, a sequencing insight, or a
simplification I clearly haven't considered, that is exactly what I'm paying for.

---

## WHAT YES MASTER IS

A local desktop mastering app (Tauri + Rust engine + React/TypeScript UI) for real
tracks and real albums. Same Rust engine (`yes_master_lib`) also powers CI-tested
native iPhone (Swift + Rust bridge) and Android (Kotlin + Rust bridge) companion
apps, plus a Vercel-deployed marketing landing page in-repo. **Desktop (Mac +
Windows) ships first.** The quality bar is "private-solid": good enough the owner
would trust it on personal releases. It is not a certified-engineer replacement and
not a throwaway toy.

Core promise: *drop in audio, hear what the mastering chain is doing in real time,
shape it fast, and export a technically-checked master without risking the source
file.* Make the safe path obvious without preventing expert/taste-led choices.

## DEFINITION OF SHIPPABLE (the finish line)

Two distinct milestones — treat them separately in your plan:

**A) Free public beta (first launch).** Full-featured app, no watermark, given away
time-boxed from the landing page to harvest reviews/testimonials/a warm email list.
Requires: signed + notarized Mac and Windows installers auto-published to GitHub
Releases; the landing "Download" button wired to real artifacts; the Manual
Listening Gate signed off by ear; a real-machine macOS build confirmed; and
verified real-time snappiness on a mid-tier Mac AND Windows laptop (the marketing
headline is literally "real-time").

**B) Paid 1.0 (the flip).** Single SKU, $29 founder → $49, sold direct via Lemon
Squeezy (merchant-of-record), installers on GitHub Releases. Adds: a **free/paid
export gate** (permanent free demo = full play + chain + receipt visible, but
export/render hard-locked until purchase), **one-time online license activation**
(cached, offline grace period), and a **Tauri updater**. The export gate + license
check are the SAME engineering seam.

Release-candidate (engine) meaning, already documented: Track Master
import/analyze/audition/export stable; real-time controls respond during playback;
exports not objectively worse by default on already-processed material without clear
warnings; warnings visible before "done"; private-fixture slow lane run for
DSP/export changes; Windows packaging works locally (done — v0.1.0 built/installed
2026-06-22); a parallel macOS build/install confirmation still owner-pending.

## CURRENT STATE (accurate as of 2026-07-01)

**Architecture**
- Desktop: Tauri (Rust core `src-tauri/` + shared lib `yes_master_lib`), React/TS
  frontend (`src/`). All export renders funnel through ONE chokepoint:
  `render_track_master` and `render_album_plan` in `src-tauri/src/engine.rs` (this
  is confirmed and is the seam the export gate will use).
- Mobile: `apps/iphone-native` (Swift UI + Rust bridge) and `apps/android-native`
  (Kotlin + Rust bridge that also re-uses the iPhone facade crate as an rlib). Both
  mirror Standard's fixed export recipe and resolve the adaptive source profile like
  desktop. Engine output is bit-parity-pinned across platforms.
- Landing: in-repo React page (`src/LandingPage.tsx` + `Hero.tsx`/`Nav.tsx`),
  Vercel-deployed; the desktop binary routes a browser visit to it.

**What's shipped and stable**
- Standard view is the default workflow: import → analyze (real progress) →
  audition Original vs Mastered at the same playhead → pick Style
  (Universal/Clarity/Tape/Oomph) + loudness (−14/−11/−9 LUFS) + Intensity → Create
  Master (fixed 44.1 kHz / 24-bit WAV at −1 dBTP, no blocking review).
- Advanced view: full preset set, EQ/tone, width/warmth, compressor modes
  (Preset/Manual/Off), right-rail Quality Check + Delivery Profile/Format + export
  review (`Export Master` clean / `Export With Review` when warnings exist).
- Album Master (Advanced): album intent, per-track override, arc kinds, album-wide
  delivery format with mixed-rate resampling, continuous + per-track renders with a
  manifest, mixed mono/stereo channel-count resolution, above-stereo fold-down.
- Original/Mastered switching preserves playhead. Volume Match is optional, off by
  default, and does not change export level. Exports never overwrite source or prior
  renders. Cross-platform golden verification resolved via an arch/OS-independent
  tolerance golden (can't spuriously fail per-OS anymore).

**Adaptive engine (the calibration-sensitive part)**
- Loudness landing adapts to current track/render measurements. Tier-1 guardrails
  (reduce-only trims to preset brightness/low-boost + reduce-only compression-density
  scale, confidence-weighted, per-axis trim caps so a preset stays recognizable) are
  SHIPPED and owner-listened/accepted 2026-06-11.
- Preset tone tilt is FIXED per preset (not reference-matched). Re-voiced to an
  owner-directed "85% lean" (commit 659bea5, 2026-06-22; `custom` untouched);
  Windows byte-identity verified; macOS snapshot work closed/superseded; **owner
  listening on the 85% lean is still pending.**
- Adaptive Compressor MVP (per-band, transient-protected, already-mastered
  stand-down, reduce-only): BUILT but GATED OFF by default. Has 9 `TBD-CALIBRATION`
  constants awaiting an owner listening sitting (this is "AC-5"). Phase-B per-window
  confidence gating: BUILT but OFF by default. Both stay OFF for v1 unless a
  listening sitting flips them.

**The launch decisions (locked with owner 2026-06-30)**
- Free time-boxed public beta → paid 1.0 at a publicly-stated flip date. Single SKU
  $29 founder → $49, one-time perpetual license; beta users lock the founder price.
- Sell direct via Lemon Squeezy (MoR, handles global tax). Installers on GitHub
  Releases. Microsoft Store later; skip Mac App Store.
- Signing from the first public build: macOS Apple Developer ID + notarization
  ($99/yr); Windows Azure Trusted Signing (~$10/mo). One-time online activation.
  Tauri updater pulling from GitHub Releases.
- Positioning: lead with **performance + simplicity + honesty**, NOT privacy. Local
  is a speed/convenience footnote, not a stance. Hero conversion asset = an
  interactive Original/Mastered A/B embedded in the landing page.
- Brand resolved: the name is **YES Master** (not "Y.E.S. Master / Your Endgame
  Sound").
- Already shipped toward launch: landing rewrite (brand + positioning + email
  capture; provider endpoint pending); a drafted release pipeline
  (`.github/workflows/release.yml` + `docs/RELEASE_SIGNING_SETUP.md`, auto-activates
  signing when owner adds secrets); drafted legal docs (EULA/privacy/refund, pending
  owner fill-in + lawyer pass).

**Fresh risk surface (from a 2026-06-30 repo-wide adversarial review: 47 confirmed
findings, 0 critical, 5 high). The engine core is clean; the risk is all in this
week's new commercial-layer code:**
- Release pipeline (`release.yml`) has **no test/lint/version gate** before it
  builds, signs, and drafts a public release (4 independent finders converged).
- `npm run verify:landing` checks DOM selectors from a deleted hero design — it
  can't pass or meaningfully fail; it's silently protecting nothing.
- Live audio-thread coefficient swap **allocates on every knob/preset tweak during
  playback** (`sources.rs:519`) — a non-allocating, already-unit-tested sibling
  exists and just isn't wired up. Directly hits the "real-time must stay responsive"
  non-negotiable.
- Draft EULA/refund docs describe a free/paid export lock + license activation that
  **do not exist in the app yet** — a misrepresentation risk if published before the
  gate ships.
- Mediums include: Signal Chain saturation display drifted from the DSP table after
  the 85% lean; undo/redo live-chain re-push has no test pin; iPhone Swift wire model
  silently drops adaptive-traceability digest fields; several cross-platform
  supported-extension lists are independent hardcodes. CSS debt is held (needs a
  real browser to verify).

## HARD CONSTRAINTS / NON-NEGOTIABLES

- Solo developer. **Limited listening time and limited test material.** Owner
  strongly prefers calibrating DSP/presets with *measurement + automated tests*
  rather than extended listening — "the math should suffice." Any objective finding
  should become a mechanical test; only genuinely taste/listening-dependent calls
  need ears.
- Real-time / near-real-time audition must stay responsive (it's now also the
  marketing headline).
- Original/Mastered switching must preserve playhead. Volume Match optional, off by
  default, must not change export level. Exports never overwrite by default. Export
  warnings advisory unless the export is technically invalid.
- Adaptive Compressor + Phase-B stay gated OFF until an owner listening signoff — do
  not enable or change their calibration constants without one.
- No private audio or rendered private masters in git.
- Commit in small, testable chunks. Prefer current code reality over historical
  prose. Track Master stabilization comes before new feature expansion.
- Owner-gated work (by-ear listening signoffs, provisioning accounts/keystores,
  brand/legal calls) cannot be done by an implementing model — plan around that.

## WHAT I THINK THE HARD PARTS ARE (address these, but challenge them)

1. **The listening bottleneck is now the v1 ship gate.** The Manual Listening Gate
   (normal / already-mastered / long-source sweeps + clean-vs-warning export by ear)
   is launch-blocking per my own non-negotiable, but I have little listening time and
   prefer measurement. Design the plan so it does NOT stall on subjective signoff:
   (a) define objective proxies/automated checks that de-risk as much of the gate as
   possible, and (b) structure ONE efficient owner listening sitting that closes the
   maximum number of gates at once (Manual Listening Gate + 85%-lean confirmation +
   already-mastered signoff, and optionally AC-5/Phase-B). Give me a concrete
   session script and a pass/fail rubric.
2. **Adaptive Compressor (AC-5) + Phase-B: ship gated-OFF-and-documented for v1, or
   invest calibration now?** Give me decision criteria and what deferring actually
   costs (product honesty, marketing claims, future re-work).
3. **The free/paid export gate + license activation** is the most important NEW,
   security-sensitive seam (export-lock leakage sinks the whole funnel). It depends
   on a Lemon Squeezy account I don't have yet. Sequence so the free beta ships
   without it, and the paid flip adds it cleanly at the single `engine.rs` chokepoint.
4. **First signed release correctness.** The release pipeline currently has no test
   gate; SmartScreen/notarization reputation only accrues if I sign from the first
   build and don't ship a broken/mismatched-version installer. What's the minimum
   trustworthy pipeline?
5. **The "real-time" headline is a literal promise.** It needs verification on
   mid-tier Mac + Windows laptops; the plan should have a fallback ("lead with
   simple") if it isn't genuinely snappy on modest hardware.
6. **Scope boundary.** Mobile is parked (shared engine = ~free to defer). Album
   Master and the Advanced/Studio tier split are v2. Pressure-test whether that's
   right, or whether any of it should pull forward.

## OPEN OWNER DECISIONS ALREADY ON THE TABLE (seed for your step 6)

- Product-canon refresh (PRODUCT.md/APP_BEHAVIOR.md): mobile product promise, Album
  Master promise beyond consistent loudness, honest adaptive-engine wording, and the
  landing page's official product role.
- Whether to broaden the "desktop Mac + Windows" non-negotiable to formally include
  iPhone/Android/web (CI already runs the mobile lanes).
- macOS build/install confirmation on a real machine; Apple + Azure signing
  enrollment timing; Lemon Squeezy account + payout country + sole-trader vs company.
- Whether to publish legal docs ahead of the export-gate code, or hold them.
- Preset/UI calls pending a listening note: are Universal/Clarity/Tape/Oomph audibly
  distinct at matched loudness post-85%-lean; mode-pill label ("Preset" vs
  "Adaptive"); eight presets vs a curated grouping.

=== PROMPT ENDS HERE ===

---

## Appendix (for me, not for Fable) — why this brief is shaped this way

- **Reflection step is mandatory and first.** A scarce model earns its cost by
  catching a wrong premise before building 40 steps on it — that's the cheapest
  insurance available. Never cut it to "save budget."
- **Two-milestone finish line (free beta vs paid 1.0)** prevents Fable from smearing
  launch-blocking work together with revenue-layer work — the export gate should NOT
  gate the free beta.
- **Readiness-not-calendar sequencing** is the owner's explicit correction: a fixed
  horizon biases a planner into cutting or padding. Fable proposes slices and tells
  us the horizon implications; the owner draws the ship line.
- **The "hard parts" list is the narrow guarantee; the FREEDOM clause is the broad
  license.** Together they resolve the narrow-vs-broad tension: coverage of known
  landmines + room for the model to surface unknown ones.
- **Current-state packet front-loads reality** so zero Fable tokens are spent on
  archaeology. Update the "as of" date and the risk-surface section before re-using.
