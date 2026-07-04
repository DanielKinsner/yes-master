# YES Master — Open Threads & Owner Decisions

The single place where **genuinely-open work** and **decisions only the owner can
make** are logged so nothing gets silently dropped. Companion to
**docs/CHANGELOG.md** (shipped history) and **docs/IDEAS_BACKLOG.md** (wishlist).

Generated 2026-06-22 from the docs-hygiene recon. Update this file as threads
close rather than letting them rot in scattered docs.

> **2026-07-03:** a hardening push is in flight on
> `harden/2026-07-03-hostile-input` — see
> `docs/plans/2026-07-03-hardening-plan.md`. It carries threads 11a, 11b, #13,
> the doc-accuracy checks (Part B 20/21), the preset-fingerprint harness idea
> (pairs thread #5), and RS-09 classification (Part B Q17) as riders, and it
> resolved Part B Q8 and (partially) Q11.

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

| # | Thread | Source | Action |
|---|---|---|---|
| 2 | **AC-5 Adaptive Compressor calibration + default-gate flip** | adaptive-compressor-mvp-spec §5; HANDOFF_2026-06-13_AC5_CALIBRATION_PREP | Owner listening session. AC-1…AC-4 shipped; queue private fixtures, run OFF-vs-ON A/B, capture keep/adjust/reject per constant, land a single AC-5 flip commit (lock 9 constants, flip default, regen snapshots, update PRODUCT/APP_BEHAVIOR). Ship gate-OFF until then. |
| 3 | **Phase-B confidence gating** calibration + default flip | RELEASE_STABILIZATION Active Gates; AC-5 prep | Owner-gated; bundle into the AC-5 sitting (same fixtures/ears). Decide whether the default flips ON in a separate commit. No code change before listening notes. |
| 4 | **Manual Listening Gate** (normal / already-mastered / long-source sweeps + clean-vs-warning export by ear) | Jump-Fix #1; deferred to Wave 10 | Owner by-ear pass. Include 8 kHz + 11.025 kHz sources (Nyquist clamp proof). Sweep Intensity/EQ/gain/compressor/Preview-LUFS/Volume-Match during playback; export clean + warning case, compare. Record a listening-note doc. |
| 5 | **Reference Retune listening notes** (Oomph least-matched) | Jump-Fix #2; deferred to Wave 10 | Re-run the private reference-tuning runner **after the 85% lean**, capture per-preset notes (Oomph: bolder without mud/pumping). Don't change export LUFS landing or compressor semantics. **The paired preset-fingerprint harness shipped 2026-07-03** (`src-tauri/tests/preset_fingerprint.rs`: safety bounds + pairwise distinctness floor + tolerance golden; owner report via `write_owner_fingerprint_report`, see TESTING.md) — future retunes are now mechanically gated; the listening notes themselves remain owner-gated. |
| 6 | **Already-mastered matrix listening signoff** | Jump-Fix #3; deferred to Wave 10 | Owner by-ear signoff against existing aggregate evidence; pairs with AC-5 stand-down listening. Re-run since the 85% lean is a DSP change. |
| 7 | **Confirm the 85% lean resolved the "characterless presets" regression** | 2026-06-15 stabilization plan | Part of the Manual Listening Gate: confirm Universal/Clarity/Tape/Oomph are audibly distinct at matched loudness post-85%-lean. If still too similar, capture a listening note before further retune. |
| 7a | **Album character system listening + flip decision** | 2026-07-03 hardening plan D7 | The album genre-inference system (per-track character labels → loudness pulls + EQ/width/warmth/intensity biases, `album.rs`) was gated OFF by default on 2026-07-03 (owner decision; it had never had a listening gate and silently altered tracks, contradicting the album promise). Owner A/B listening session decides whether it returns as a visible opt-in. Any flip commit removes or demotes the filename-keyword label override. |

### OPEN — roadmap / scope (no listening required)

| # | Thread | Source | Action |
|---|---|---|---|
| 8 | **PRODUCT.md / APP_BEHAVIOR.md canon refresh (S5.4)** — Standard-first workflow, Mobile, Album Master, honest Adaptive wording | roadmap S5.4 | Owner interrogation (mobile scope, album promise, adaptive/compressor wording), then rewrite. See Part B Q7–Q10. Tie the compressor-canon update to AC-5. |
| 9 | **Tier-1 adaptive follow-ons** (tilt-vs-reference brightness, density-cap reshape, stereo_width co-trigger, per-axis EQ floors, LRA→Option) | ADAPTIVE_DSP_NEXT_STEPS; roadmap S10.4 | Wave 10 taste work. Tilt-vs-reference is highest-value. Gate every constant change behind a listening note + fingerprint test. |
| 10 | **Tier-2 "smart" milestone** (measured-neutral per-preset, PSR/crest loop, corrective curve, reference matching, resonance/sibilance) | ADAPTIVE_DSP_NEXT_STEPS; roadmap S10.4/5 | Post-v1. B3 loss-budget + PSR transient protection are the most defensible first steps. See IDEAS_BACKLOG. |
| 11 | **Mobile store-readiness** (iPhone PrivacyInfo/real-progress/error-enum; Android signing+keystore; background-audio) | roadmap Wave 8 / S8.x | Owner-gated, out of desktop-v1 scope. Android signing blocked on owner keystore. |
| 11a | **CSS styling-debt batch (audit D7, 10 items)** — `.toast`/`.std-tile`/`.tile:hover` dedup, rail-width + z-index tokenization, dead `.chain-link` CSS, `.adaptive-readout` inline→class | 2026-06-23 audit Batch I | **Deferred — needs a browser.** CSS refactors can't be visually verified in the headless agent sandbox (dev server can't bind; `verify:landing` fails `ERR_CONNECTION_REFUSED`), and the plan's recs are imprecise (e.g. "delete the `.toast` base block" would actually drop live `background/border/padding`, not just the overridden `position`). Do these in a session with a working preview. |
| 11b | **Dead-code tail (audit D4 leftovers)** — `profile_store::insert`, `LandingGainCache` test helpers, `deep_analysis` `harsh_share`/`sibilant_share`, AdvancedPanel `bandFields` auto-strings, `shouldForceAdvancedOnStandardEntry`, iphone `cdylib` crate-type | 2026-06-23 audit Batch H | Verified-dead but **test-entangled / mobile-unverifiable-locally**; left for a focused follow-up delete commit (each needs its test callers updated or a CI round-trip). |

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
8. ~~What does **Album Master** promise beyond consistent loudness?~~
   **ANSWERED 2026-07-03:** *"One coherent record — consistent loudness, one
   delivery format, honest per-track receipts, nothing silently altered."*
   Arc = user-chosen expressive bonus. DDP/cue/ISRC/gapless out of v1.
   Override = full sound exemption (own settings + own target, album delivery
   format kept, manifest marks it). See `docs/plans/2026-07-03-hardening-plan.md` D4/D9.
9. How should the **adaptive engine** be described honestly in PRODUCT.md / APP_BEHAVIOR.md?
10. What is the **marketing landing page's** product role (marketing-only vs download/onboarding), and should PRODUCT.md name it as a public surface?

### Agent-file scope
11. Is the **landing page / web build in-scope for agent work** (add `verify:landing` + `docs/landing-brief.md` to Required Reading), or hands-off like the Next.js storefront?
    **PARTIALLY ANSWERED 2026-07-03:** in scope for **security & verification**
    purposes (the beta-capture backend gets a security pass — hardening plan D3).
    Whether it enters Required Reading for general feature work is still open.
12. Broaden the "Local desktop app for Mac and Windows" non-negotiable to acknowledge iPhone + Android (and web)? *(Recommended yes — the CI already runs the mobile lanes.)*

### Packaging / platform
12a. **Path sandboxing (audit §14).** The desktop path guard intentionally rejects only `..` and ALLOWS absolute paths — required for the native-file-picker model (the user imports/exports arbitrary locations). Base-dir confinement / symlink-target rejection would break that, so it's a product decision: keep the current model, or introduce a sandboxed mode? (The mobile bridges ARE now `..`-guarded — audit §15.) Default: keep current desktop model.
13. Confirm **macOS build/install status** — add a parallel macOS-packaging release criterion to PRODUCT.md?
14. **Android signing/bundleRelease** is blocked on you providing a keystore — when?
15. Mobile **background-audio** behavior (UIBackgroundModes / foreground service) — v1 limitation or build it?

### Roadmap owner-decision queue
16. **Min window size** for 1366×768 laptops — document the requirement (cheap default) or schedule a layout slice?
17. ~~**RS-09 limiter flush/tail** (~3 ms export-byte change) — defer + document (default), or accept?~~
    **RESOLVED 2026-07-03 — fixed, not deferred.** The 2026-07-03 DSP math
    audit confirmed (with adversarial verification) that every export was
    silently dropping its final ~3 ms (stuck in the limiter lookahead ring)
    and shipping a ~3 ms silent lead-in — an objective bug under the
    two-tier policy, not a preference. `MasteringChain::flush_render_tail`
    now drains the lookahead on both export paths; output keeps source
    length and sample alignment. Spot-listen entry in the hardening plan.
18. **Stereo_width disposition** — wire it as a width co-trigger or delete the inert carried field?
19. Confirm the **parked items stay parked**: P2 one-pole/soft-knee hoist + P4 tauri-specta (on the do-not-do list) — leave alone?

### Low-risk doc-accuracy checks (I can do these on request)
20. `IPHONE_APP_OVERVIEW.md` preset vocabulary vs the shipped Standard 8-preset set (flagged for preset-name drift).
21. `ENGINE_REFERENCE.md` preset-calibration table predates the 85% lean — EQ/dB numbers likely stale vs shipped.

### Branding (parked)
22. **"Y.E.S. Master" / "Your Endgame Sound"** vs the current **"YES Master"** — a brand decision you parked. It cascades across PRODUCT.md, AGENTS.md/CLAUDE.md, README, and the landing copy. Until you call it, docs keep the current "YES Master" naming.
