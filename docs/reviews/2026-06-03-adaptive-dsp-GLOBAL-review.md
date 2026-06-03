# Global Review — Adaptive DSP Tier-1 Guardrails

Date: 2026-06-03
Branch: `feat/adaptive-dsp-guardrails` (not merged)
Method: **independent verification of two prior reviews against the current code**, by an 8-agent read-only review fleet (6 claim-verifiers + 2 fresh-eyes auditors), reconciled with a prior manual read. Every claim below was re-checked against the tree as it stands now — not taken from the reviews on trust. The point of this pass was to catch any claim the reviews got *wrong, overstated, or that's already fixed*, and to find what they *missed*.

Sources reconciled:
- `2026-06-02-adaptive-dsp-tier1-review.md` (Claude multi-agent, already self-reconciled with the desktop review + a Codex pass)
- `2026-06-03-adaptive-dsp-desktop-review.md` (independent desktop review)
- This pass's 8-agent verification fleet + fresh-eyes audits

---

## Meta-verdict

**The two reviews are trustworthy.** Of 38 distinct claims verified, the breakdown is:
- **CONFIRMED:** 33 (including all of the "what's genuinely good" invariants)
- **PARTIAL / nuance:** 3 (F7 wording, F3 narrowness, the pink-neutral test claim)
- **REFUTED:** 0
- **NON-finding (code correctly matches spec):** 2 (auto-Compressor-Off correctly *not* implemented; all spec numeric constants match)

So this is not a "the reviewers were wrong" situation. It's the opposite: the core engineering is **clean and the criticisms are real**. The agreement between two independent reviews held up under adversarial re-checking.

**What this pass adds on top of the reviews — the reason it was worth running:**

1. 🔴 **Album export is worse than "deferred."** Both reviews logged album-export-skips-injection as a *documented deferral* (medium). The fleet found that album-mode **live audition DOES inject** (`useTrackMaster.ts:1306`) while album **export does NOT** — so in album mode, **what you hear ≠ what ships**, and the *same track* masters differently standalone vs in an album. That's a genuine audition≠deliverable break, not a clean deferral. **Upgraded to HIGH.**
2. 🟠 **The "opt-out" is a trap.** `null` strength silently resolves to 0.6 (on). The Advanced **reset button, slider double-click, and clearing the field all write `null`** → they silently *re-enable* adaptation. "Off" is only durably representable by dragging to exactly 0, and a project saved at `null` reloads as 0.6-on. (New, MEDIUM.)
3. 🟠 **Delivered masters are untraceable.** `ExportReport` records no `adaptive_strength` / "guardrails applied" flag. Given adaptation silently changes preset character and differs across surfaces, there's no way to tell two otherwise-identical masters apart. (New, MEDIUM.)
4. 🟡 **A second LRA hazard, distinct from the one both reviews found.** Beyond the `LRA=0` sentinel: when P95-P10 is missing, `unwrap_or(dynamic_range_lu)` aliases an **LU-scaled** value into the **dB-thresholded** DR ramp — so even a *non-zero* fallback (e.g. 4 LU) mis-triggers against the 8/3 *dB* thresholds. (New, LOW — narrow input window.)
5. ✅ **A real certification.** The fleet's clean probes independently confirm the math is not just "looks right": no NaN/Inf/divide-by-zero paths, no panics, **the profile is never derived from already-processed audio (no guardrail chasing its tail)**, and the inert path is **float-for-float identical** to the pre-feature chain (byte-identity is safe by construction, not by luck).

**Bottom line (unchanged from both reviews, now verified twice over):** the idea and the core math are right; the branch is **not ready for the listening gate** until the wiring/ownership is centralized, two measurement hazards are gated, and the per-axis readout ships. All fixable in a focused pass.

---

## Consolidated findings — verified against current code

Severity = this pass's reconciled severity. "Verdict" = independent re-check result.

| # | Finding | Reviews' call | **This pass** | Current evidence |
|---|---|---|---|---|
| **F1** | Slow lanes (`fixture_matrix.rs`, `reference_tuning.rs`) never set `advanced.source_profile` → the mandated already-mastered evidence lane renders the **old** chain | High | **CONFIRMED · HIGH** | `fixture_matrix.rs:117-127`, `reference_tuning.rs:294-304` set `source_lufs_integrated`/`compression_mode` but no `source_profile`; gate `dsp.rs:756-762`. No back-door injection (clones `recommended_universal` which carries `AdvancedSettings::default()`). |
| **F2** | Offline **preview/audit WAV** skips injection while export injects → WYSIWYG break | High | **CONFIRMED · HIGH** | `useTrackMaster.ts:1217` preview passes raw `selectedSettings`; `:1247` export wraps `injectSourceProfile`; live `settings-transitions.ts:198-201` injects; backend `engine.rs:335-363` forwards settings, derives nothing. *This is the artifact users A/B against the master — see New-3.* |
| **F3** | Non-finite LRA coerced to `0.0` → density ramp reads "maximally dense" → spurious **full 60%** comp trim | High | **CONFIRMED · HIGH** (narrow trigger) | `analysis.rs:159` coerces; `types.rs:144` `unwrap_or` + `settings-transitions.ts:218-220` forward; `guardrails.rs:110-112` `max(dr,lra)` ramp → full. **Nuance:** EBU LRA only goes non-finite on degenerate/too-short input, so the misfire hits short/pathological sources, not a normal long dynamic master. Compound confirmed: missing P95-P10 *also* falls back to the same `0.0`. |
| **F4** | Brightness deadband `0.20` fires on a pink-tilted **neutral** master (presence+air ≈ 0.278) → neutral loses ~40% air at default strength | High | **CONFIRMED · HIGH** (idealized) | Band edges `analysis.rs:361`; trigger/deadband `guardrails.rs:95,30`. Recomputed the 1/f log-band integral independently: **0.2777**, above 0.20 → `bright_raw≈0.65`. **Nuance:** that assumes a clean 1/f across the full 20–16k span; real masters that roll off the top octave or tilt bass-heavy may land *under* 0.20. So "neutral over-trims" is true for idealized pink; whether a *typical* master does depends on its tilt — which is exactly why the readout + calibration matters. |
| **F5** | 6-band FFT reads only the first **~5.5–6 s** (single FFT, `1<<18` cap) despite a "**up to 30 seconds**" comment; DR/LRA/correlation are whole-track | High | **CONFIRMED · HIGH** | `analysis.rs:322` ("30s" comment), `:325-328` caps `fft_size` at 262144 (5.46s@48k / 5.94s@44.1k), `:340-354` single FFT from frame 0. The other measures iterate the whole buffer. An intro can classify the whole song. |
| **F6** | Album Master export never injects a per-track profile | Medium (deferred) | **CONFIRMED → UPGRADED · HIGH** | `album_render.rs:65-103` never touches `source_profile`; `useTrackMaster.ts:1120-1131` album builder omits `injectSourceProfile`. **But** album *live* playback DOES inject (`:1306`) → audition≠deliverable *in album mode* + same-track-differs-standalone-vs-album. See New-1. |
| **F7** | Density "macro" is a single scalar (engagement+threshold+ratio); "≥40% comp" is true of the macro not dB-GR; cap saturates at strength 0.6 | Medium | **PARTIAL · LOW** | Numeric claims **confirmed**: `dsp.rs:942-952`; at `density_raw=1.0`, strength 0.6 → `min(0.6,0.60)=0.60` → mult 0.4, identical to strength 1.0 (default already saturates on a fully-dense trigger). **Overstated bit:** "overdrive" is the same threshold/ratio mechanism, not a separate 4th saturation stage (`saturation_amount` `dsp.rs:831` is *not* density-driven). Tier-2 direction, not a Tier-1 defect. |
| **F8** | When a Delivery LUFS target is active, the post-chain **LUFS-landing** stage can add broadband makeup that recoups trim-induced loudness loss → final A/B can exceed any single per-axis cap | Medium (06-02 marked "unique") | **CONFIRMED · MEDIUM** | Held up under three refutation attempts. `engine.rs:618-679` measures post-chain then lands when a target exists; `engine.rs:106-127` delta can be **positive** (bounded by TP headroom, which can exceed a per-axis cap); applied last at `dsp.rs:2041-2045`; live preview mirrors it (`audio.rs:865-953`). Brickwall limiter has **no makeup** (`dsp.rs:1420-1440`); compressor auto-makeup *drops* when density trimmed (`dsp.rs:1073-1076`); Volume Match off-by-default + attenuate-only. **Real but narrow:** only with an effective target (incl. **Custom + explicit `lufs_offset_db`**, not only named profiles). ⚠️ Stale comment `dsp.rs:2039-2040` says "down-only" — contradicts actual post-B6 upward-makeup behavior. Line drift: reviews cited `engine.rs:1607-1644`; actual is `:618-679`. |
| **F9** | Rust `SourceProfile::from_analysis` is **dead code**; TS twin is the only injector; `types.rs` comment falsely claims backend-derived export | Medium | **CONFIRMED · MEDIUM** (root-cause, see New-2) | `grep` finds exactly one hit (its own def, `types.rs:140`), zero callers. `types.rs:115-117` claims "export: backend from the track AnalysisResult" — false; export injects TS-side at `useTrackMaster.ts:1247`; `bindings.ts:262-266` has the correct wording. |
| **F10** | `stereo_width` (side-energy) computed + carried but **never read** by `compute()` (correlation-only width trigger) | Medium | **CONFIRMED · MEDIUM** | Field `types.rs:131-132` populated `:147`; `compute()` `guardrails.rs:114-119` reads only `stereo_correlation`. Inert payload, not a correctness bug — either wire it as a co-trigger or remove it from profile+spec. |
| **F11** | No "what was trimmed & why" readout → listening A/B can judge the global dial but can't attribute per-axis | Medium | **CONFIRMED · MEDIUM (gating)** | Trims are private to `SourceGuardrails`; UI exposes only the global slider (`App.tsx:2137-2145`). This is the **gating dependency for calibration**, not polish — the numbers can't be tuned by ear without it. |
| **F12** | Spec table over-claims: lists `presence_db` in bright "Touches" (code correctly leaves it); promises a `stereo_width` secondary signal that isn't implemented | Low | **CONFIRMED · LOW** | `dsp.rs:768` `presence_db` untrimmed (only `high_mid`/`air`/`sparkle` wrapped, `:769-773`); spec table lines 179/182 over-promise. **The code is the *correct* behavior** (leaving the musically-sensitive 1.5k band alone); the spec is wrong, not the code. |
| **F13** | `EQ_BOOST_FLOOR_DB` global not per-band; **two** copies of the default strength (Rust const + TS `?? 0.6`); `0.42` low deadband may over-fire on bass-forward genres | Low | **CONFIRMED · LOW** | `guardrails.rs:60` (global floor), `:17` const vs `App.tsx:2139` bare `0.6` literal (no shared source). Drift risk if the default is retuned. |
| **F14** | iPhone shares the engine, never injects, runs un-adapted | scoped out | **CONFIRMED · NOT A DEFECT** | `IPHONE_APP.md:16,61` ("Do not add adaptive/smart analysis to iPhone v1"), `IPHONE_APP_OVERVIEW.md:36`. Intentional. |

---

## New findings — caught by this pass, in neither prior review

| # | Finding | Severity | Evidence | Fix |
|---|---|---|---|---|
| **N1** | **Album mode: live audition is adaptive, export is not.** Same track masters differently standalone vs album; album audition ≠ album deliverable. | **HIGH** | Album live `useTrackMaster.ts:1306` injects; album export `:1120-1131` + `album_render.rs:65-103` do not. | Inject per-track in `exportAlbumPlan`, or derive server-side via `from_analysis`. Add album==track render-equality test. |
| **N2** | **No backend fallback.** `from_analysis` is dead, so correctness of "on by default" rests entirely on the frontend remembering to inject at *every* render entry point — and it already forgot three (preview, album, slow lanes). | **HIGH** (root cause) | `types.rs:140` zero callers; `dsp.rs:756-761` treats `None` as inert. | Make the backend authoritative: derive `source_profile` from the track `AnalysisResult` server-side; treat the FE-supplied profile as an optional override. **This one fix closes F1, F2, F6/N1, F9 by construction.** |
| **N3** | **The audit-WAV — the artifact users A/B against the master — is the one that diverges.** Button copy says "audit it in another player," but it runs non-adaptive while live + export are adaptive. | **HIGH** | `useTrackMaster.ts:1217` vs live `:1306` / export `:1247`; copy `RightRail.tsx:110`. | Wrap `injectSourceProfile` in `updatePreview` (falls out of N2). |
| **N4** | **`null` strength silently re-enables 0.6.** Advanced reset (`App.tsx:2058`), slider double-click (`:2756`), and field-clear (`:2795`) all write `null`; display masks it as `0.6` with no "Off" pill. "Opt out" only persists if you drag to exactly 0; a project saved at `null` reloads adaptive-on. | **MEDIUM** | `dsp.rs:751-755` `unwrap_or(0.6)`; `App.tsx:2139` `?? 0.6`. | Pick one durable "off" representation; exclude `adaptive_strength` from the reset null-sweep; label the default honestly ("Auto 60%"). |
| **N5** | **Non-zero LRA scale-aliasing.** When P95-P10 is missing, `unwrap_or(dynamic_range_lu)` judges an **LU** value against the **dB** DR thresholds (8/3) — distinct from the `LRA=0` sentinel both reviews found. | **LOW** | `types.rs:144` / `settings-transitions.ts:218`; `guardrails.rs:105-109`. Narrow window (>~21 ms for FFT but <~250 ms for P95-P10). | Carry `Option<f32>` for P95-P10; use only `lra_raw` when it's `None` instead of aliasing. |
| **N6** | **Export receipt carries no adaptation record** — delivered masters aren't traceable as adaptive vs not, or at what strength. | **MEDIUM** | `ExportReport` `types.rs:770-781` + `useTrackMaster.ts:1258-1271` omit it. | Add `effective_adaptive_strength` (+ a one-line profile digest) to `ExportReport` and the receipt. |
| **N7** | **Adapt Strength control is editable in album mode but a complete no-op there** (writes into `albumIntent`, but album render never injects). | **LOW** | `App.tsx:2137-2145` renders in album mode; album render inert. | Wire album (N1) so it's real, or hide/disable the control in album mode. |

---

## Certified good (independently re-verified — do not change)

These were claimed solid by both reviews; this pass re-derived them adversarially and they hold:

- **The four safety invariants** — reduce-only, never-flip-sign, never-narrow, mono-never-trims-width — proven from the code (`guardrails.rs:90-178`), not just asserted.
- **Byte-identity is float-exact.** With no profile, the trim closures are pure identity and the `effective_*_db` expressions reduce to the **exact pre-feature form** (verified against `git 6e1e858~1`) — same op order, no reordering, no added ops. SHA snapshots are safe by construction.
- **`floor_boost`** passes cuts/zero through, never raises a tiny boost, floors at `min(0.5, preset_db)`. **Caps** cap the *trim* (`min(raw·s, cap)`), not the multiplier.
- **EQ band mapping is correct and *more* correct than the spec** — trims `high_mid`/`air`/`sparkle`, correctly leaves `presence_db` (1.5k) alone.
- **Manual overrides genuinely preserved** — user `eq_*_db` added after the trim; Manual compression skips density trim; explicit `advanced.width` bypasses the width trim.
- **Level-invariance is correct** — shares/ratios, no LUFS-normalization needed.
- **No tail-chasing.** The profile is always derived from the *source* `AnalysisResult`, keyed by track id, never recomputed from processed audio; persists correctly across settings edits; live and export build it from the same cached source analysis.
- **No NaN/Inf/divide-by-zero/panic** in the adaptive path: all-silent → `None` (inert); correlation denom guarded; percentile indices in-bounds; strength clamped (NaN → dropped → inert); the only unwraps are defaulted/sort-safe.
- **IPC contract is clean** — `SourceProfile` / `SpectralBalance6` / `AdvancedSettings` match field-for-field and optionality across `bindings.ts` ↔ `types.rs`. `source_profile` is **not** persisted into presets/projects (no stale-profile leak).
- **The deferred auto-Compressor-Off is correctly *not* implemented** (deferral honored); all spec numeric constants match the code.

---

## Corrections & nuances for the record (so the reviews aren't over-read)

- **F3 / F4 are narrower than the headline.** F3 fires on short/degenerate sources (unmeasurable LRA), not normal long dynamic masters. F4's 0.278 is an *idealized* full-span 1/f figure; real masters with top rolloff/bass tilt may sit under the deadband. Both are still worth fixing — but "every neutral track loses 40% of its air" is the worst case, not the guaranteed case.
- **F7's "overdrive as a 4th stage" is overstated** — it's the same threshold/ratio mechanism. The numeric saturation-at-0.6 point stands.
- **F8 had line drift** (cited `engine.rs:1607-1644`, actual `:618-679`) and lives alongside a **stale "down-only" comment** (`dsp.rs:2039-2040`) that contradicts the real upward-makeup behavior. Also active for **Custom + explicit `lufs_offset_db`**, not only named delivery profiles.
- **F12 is the spec being wrong, not the code.** The code's behavior (leaving `presence_db` alone) is the *safer* choice.

---

## Recommended sequence (reconciles both reviews + new findings)

Both reviews independently converged on the same headline: **fix the wiring/ownership and ship the readout *before* the listening session**, because the session can't produce calibrated numbers without attribution. This pass agrees and sharpens the order:

### P0 — before the listening gate (correctness + release evidence)
1. **Make the backend own & derive `source_profile`** (wire the dead `from_analysis`); treat any FE-supplied profile as an override. → closes **F1, F2/N3, F6/N1, F9/N2** and the entire "a new render path forgot to inject" class **by construction**. *This is the single highest-leverage change; everyone converges on it.*
2. **Gate the LRA hazards (F3 + N5):** represent unknown LRA *and* missing P95-P10 as `Option`/"unknown" and zero their ramp contribution (don't alias LU into dB). Add the regression test (`healthy DR + LRA=0/None ⇒ no density trim`) — note it would **fail today**, so it's catching a latent bug, not just adding coverage.
3. **Decide album scope explicitly (N1).** It's currently audition≠deliverable in album mode — worse than a clean deferral. Either wire it (preferred — falls out of #1) or hard-disable the album Adapt-Strength control (N7) and say so in UI/docs.

### P1 — calibration-enabling (still before listening)
4. **Ship the per-axis "what was trimmed & why" readout (F11).** The trims are already computed — surface them (label as *chain* trims, pre-landing per F8). Gating dependency.
5. **Recalibrate brightness (F4)** toward measured pink/reference, and **Welch-average the 6-band FFT** across the track (F5). Add a real-pink-derived no-trim test (the existing one is a synthetic-neutral *unit* test — see C6 "PARTIAL").
6. **Fix the `null`-strength opt-out (N4):** make "off" durable, stop reset/clear from re-arming, label the default honestly.

### P2 — hardening & Tier-2 direction
7. Add **slope / side-mid co-triggers** (F10 + strategy); reword or re-anchor the density cap (F7); add **export-receipt traceability (N6)**; collapse the dual default-strength (F13b); fix the stale `dsp.rs:2039` comment + false `types.rs:115` doc (F9); fix the spec table (F12).
8. **Add the missing tests** (all CONFIRMED absent): WYSIWYG/parity, slow-lane profile-present, **album==track render equality**, multi-axis composition *under active LUFS landing*, manual-override preservation at the adaptive layer, render-level `profile=Some + strength=0` byte-identity, LRA-sentinel regression.

**Tier-2 north star (after listening locks v1 numbers):** build the neutral reference from Dan's own reference masters in the slow-fixture lane; replace inferred per-octave deadbands with *measured* per-band shares (optionally per-preset); add a PSR/crest closed-loop for the only honest "don't crush it further" dynamics defense.

---

## Two product calls that are yours alone (only the owner's ears/taste resolve these)

1. **On-by-default at 0.6.** With the readout missing and **four surfaces that disagree** (live = adaptive, track-export = adaptive, album-export = non-adaptive, audit-WAV = non-adaptive), the user's "preset as designed" baseline is *unstable* — silently altered on some paths, silently not on others, with no on-screen or in-receipt indication of which they're hearing. Defensible to ship on (single-user calibrator, bounded blast radius), but it's a house-sound identity decision. Fixing the wiring (P0 #1) collapses the four surfaces to one, which is what makes "on by default" *safe* to ship.
2. **Album adaptation scope.** Tied to N1 — either it's in (wire it) or it's out (disable the control + say so). "Editable but inert" is the one option to avoid.

---

## Confidence

**High** on the findings (every claim re-checked against current code with concrete file:line evidence; line drift noted where the reviews had aged). **High** on the certified-good list (re-derived, not trusted). The remaining uncertainty is exactly where it should be — **taste calibration of the provisional numbers**, which is the listening gate's job, now unblocked once the readout ships.

*Produced by an 8-agent read-only verification fleet over `feat/adaptive-dsp-guardrails`; no files mutated, no builds, no branch changes (a second agent was live in the tree during review).*
