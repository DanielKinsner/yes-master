# Repo-Wide Audit & Remediation Plan — YES Master (macOS-weighted)

**Date:** 2026-06-23
**Scope:** whole repo at `main` (`9d42581`), with extra weight on the recent macOS work.
**Method:** dynamic multi-agent audit — 21 finder units across 8 dimensions, each candidate
finding adversarially verified by **3 independent skeptics** (majority-of-3 keeps). 85 candidates →
**68 confirmed**, 17 refuted/dropped, 8 speculative. Every item below cites `file:line` and survived
verification; refuted findings are not listed.

> **So what (read this first).** The codebase is in genuinely good late-stabilization shape — no
> critical bugs, no secrets, no export-overwrite-by-default hole, no audio in git. The real risks are
> **(1)** a cluster of *missing pinning tests* on your own non-negotiables (the adaptive gate, the
> volume-match export invariant, compressor-Off scope) — they're correct today but one careless edit
> from breaking with nothing to catch it; **(2)** the cross-platform golden tests are **arm64-only**,
> so an Intel Mac would throw spurious "DSP drift" failures; **(3)** a handful of small, real
> correctness bugs in the live-audition/transport path; and **(4)** the wire-parity drift gate has
> large coverage holes that let Rust↔TS field drift ship silently. Plus the usual dead-code and CSS
> debt. **Start with the tripwire tests (Workstream B)** — they're additive, zero behavior change, and
> they harden the exact guarantees you've said matter most.

---

## Remediation status (executed 2026-06-23)

Batches A–H executed in small pushed commits with per-fix TDD + CI verification.

- **Batch A (§7–§10) — DONE.** Tripwire tests for the adaptive gate (OFF default),
  VM export invariant (FE; backend already pinned), compressor-Off scope, VM-off
  default, and the wav write primitive.
- **Batch B (§11–§13) — DONE.** Drift gate now recurses into nested structs and
  registers ProjectState, album payloads, export/quality/preset, device/waveform
  (+ nested ImportedTrack). Proven to bite in both directions.
- **Batch C (§3,§4,§5) — DONE.** removeTrack transport/meter/loop reset; live
  ceiling reaches the audition limiter (regression-tested; **owner ear-check
  recommended, see below**); Album↔Track mid-audition re-push.
- **Batch D (§6) — DONE.** Analysis-batch progress set; Export-Anyway guard;
  Android Gson crash-guard (+ a follow-up fix for a restore-path regression the
  guard surfaced). §6d (album TOCTOU) skipped — already mitigated by the existing
  dedup; §6e (iPhone double-decode) is perf-only, noted.
- **Batch E (§14–§16) — DONE.** §15 real fix: mobile facade rejects `..` on
  output_dir/source_path before create_dir_all/decode. §14: desktop contract
  pinned (absolute paths intentionally allowed; base-confinement is an owner
  decision). §16: already mitigated in current code.
- **Batch F (§1,§2) — DONE.** Per-OS exact-byte goldens replaced with
  arch/OS-independent tolerance goldens; Swift VolumeMatch finite-guard.
  Confirmed green on macOS-arm64 CI; still fails on synthetic drift.
- **Batch G (§17–§19) — DONE.** Landing style names; ENGINE_REFERENCE regen;
  macOS-SHA thread closed.
- **Batch H (D4) — PARTIAL.** 3.2 MB PNGs + clean Rust/FE/mobile dead symbols
  deleted; 4 list items were genuinely used and kept; a verified-dead-but-test-
  entangled tail is deferred (OPEN_THREADS #11b).
- **Batch I (D7) — DEFERRED.** CSS refactors need browser verification the agent
  sandbox can't provide; the plan's `.toast` rec is imprecise (would drop live
  styling). See OPEN_THREADS #11a.
- **Owner-gated backlog — left parked** as instructed (S5.4 canon, preset-
  fingerprint harness, stereo_width disposition, AC-5 calibration).

---

## How to read this

Each finding is tagged **Fix now** or **Hold / owner-gated**, derived from your `CLAUDE.md`
non-negotiables and `RELEASE_STABILIZATION.md`:

- **Fix now** = real bugs, security hardening, and *additive* guardrail tests. None of these change
  DSP voicing or stability-critical behavior.
- **Hold / owner-gated** = anything touching preset calibration/voicing, taste/listening work, the
  adaptive default-gate flip, mobile store-readiness, or product-canon decisions only you can make.

Severity counts (confirmed): **High 6 · Medium 22 · Low 40**.
By dimension: D1 correctness 10 · D2 parity 11 · D3 security 5 · D4 dead-code 17 · D5 spec 4 ·
D6 forgotten 6 · D7 CSS 10 · D8 stabilization 5.

### Coverage caveats (no silent caps)
- The **arm64-vs-Intel SHA gap surfaced ~7 times** independently (D1/D2/D5/D6/D8). That's a strong
  signal, but it is **one root cause across three test sites** — consolidated into §1 below.
- One verifier agent (`verify:D3:34.0`) died mid-response (API connection drop); that finding kept
  its other two votes, so the majority verdict is unaffected.
- **Not inspected:** private audio fixtures (gitignored, correctly), `target/` build artifacts,
  `node_modules/`, `graphify-out/`. Generated `Cargo.lock`/`package-lock.json` were read only for
  dependency hygiene.

---

# macOS / cross-platform — the focus area

## 1. Per-OS golden tests are architecture-blind (Windows/macOS only, not Intel vs Apple Silicon)
**Severity: Medium (latent) · Fix now (decision-gated) · D1/D2/D5/D6/D8 — consolidated**

Three separate golden mechanisms branch on `cfg!(target_os = "macos")` with **no `target_arch`
axis**, and all macOS values were recorded on **Apple Silicon (arm64)**:

| Site | What it pins | macOS value recorded on |
|---|---|---|
| `src-tauri/src/dsp.rs:4048-4057` (`expected_platform_sha`) + 8 callers `4123-4225` | 7 preset chain-output SHA-256 byte-identity hashes | arm64 (comments at `4124-4127`, …) |
| `src-tauri/src/deep_analysis.rs:848-895` (`expected_macos` `[u32;16]` fixture, selector `886-890`, assert `894`) | deep-analysis window-metric bits | arm64 |
| `src-tauri/src/analysis.rs:1000-1035` (`expected_platform_spectral_balance_6band`, assert `1031`) | 6-band spectral golden (exact f32 equality) | arm64 |

The chain runs f32 transcendentals (`powf`, `tanh`, `exp`, biquad `sin`/`cos`) whose last ULP differs
between x86_64 and arm64 libm. A single differing ULP propagates through the IIR feedback and changes
the hash.

**Why it matters:** an Intel Mac (or any x86_64 macOS runner) compiles the `macos` branch and compares
against arm64-recorded values → spurious `"chain-output SHA changed; investigate DSP drift"` failures
on code that is actually fine. CI is arm64-only **today**, so this is latent — but it's exactly the
gap `HANDOFF_2026-06-22_MACOS_VERIFICATION.md:363-368` flags as unresolved.

**Fix (owner decision 2026-06-23 — do it right; no Intel Mac needed, zero audio change):** the
brittleness is the *exact-byte* comparison, not the missing architecture. Replace the per-OS exact-byte
SHA (and the `deep_analysis` `[u32;16]` and `analysis` 6-band exact-equality goldens) with an
**architecture- and OS-independent golden**: assert the rendered buffer matches a single stored
reference **within a tight tolerance** (e.g. `max_abs_delta < 1e-6` — comfortably above the ~6e-8
Windows↔macOS rounding, far below any structural drift), or hash a buffer quantized to a coarse grid so
last-ULP libm differences collapse. **Change only the test's comparison, never the real-time DSP**
(switching the engine's libm would alter audio). Verify the new golden is identical on Windows and
macOS-arm64 (Windows local + macOS CI) AND still fails on a synthetic structural change (prove the
tripwire bites). x86_64 Macs then pass by construction — no Intel hardware ever required.

## 2. VolumeMatch gain formula hand-mirrored Swift↔Android-Rust; non-finite guard only on one side
**Severity: Low · Fix now · D2**

`apps/android-native/rust/src/audition.rs:346-353` (`volume_match_linear_gain`) and
`apps/iphone-native/YESMasterNative/VolumeMatch.swift:5-12` carry the same linear-gain formula by
hand, but the **non-finite/NaN guard exists only in the Rust copy**. A non-finite measured LUFS on
iOS could produce a non-finite gain.

**Fix:** add the same finite-guard clamp to the Swift copy; add a shared wire-sample test case (an
extreme/non-finite input) that both platforms run.

---

# Fix now — correctness bugs (D1)

## 3. `removeTrack` leaks stale "playing" transport + live meters onto the auto-selected sibling
**Severity: High · `src/hooks/useTrackMaster.ts:1410-1462`**

When you remove the currently selected+playing track, the code calls `api.stopPlayback()` (`:1452`)
and reselects the first remaining track (`:1456-1458`) — but it **never resets the React transport
state** (playing flag, live meters). The new sibling inherits a stale "playing" indicator and frozen
meter values.

**Fix:** in the `loadedTrackId === id` branch, also reset transport/meter state (set playing=false,
clear meters, clear loaded-chain predicate) before/with the reselection. Add a test: load+play track
A, `removeTrack(A)`, assert sibling B shows stopped transport and cleared meters.

## 4. Live ceiling / delivery-profile change leaves the audition limiter at the OLD ceiling
**Severity: Medium · `src-tauri/src/dsp.rs:2124-2152` (live coeff path) ← `sources.rs:519-525` ← `audio.rs:1654-1716`** · 3/3 verified

The real brick-wall ceiling lives only in `Limiter` (built in `Limiter::new` from
`effective_ceiling_dbtp()`, `dsp.rs:1508/2089`). The **live** update path
(`with_coeffs_inheriting_state` / `overwrite_with_coeffs_inheriting_state`) **clones `prior.limiter`
verbatim** and overwrites only `ChainCoeffs` — and `ChainCoeffs.ceiling_lin` is snapshot-only, never
read in `process_frame_inplace`. So changing the delivery profile mid-playback (e.g. StreamingUniversal
−1.0 dBTP → VinylPremaster −3.0 dBTP) or editing `advanced.ceiling_dbtp` routes through `UpdateChain`,
not a fresh `play_master`, and the audition limiter keeps the **old** ceiling until playback restarts.

**Why it matters:** violates your "live A/B must match export" guarantee — the user auditions a louder
ceiling than the file they'll actually ship. (Export and `play_master` both rebuild the limiter
correctly via `MasteringChain::new`.)

**Fix:** carry the effective ceiling in `LiveCoeffUpdate` and set `limiter.ceiling_lin` in the
inherit-state paths (or rebuild the limiter when `effective_ceiling_dbtp()` changes). Add a regression
test that pushes a coeff update with a different ceiling and asserts the live chain clamps to the new
value. **Note (owner decision 2026-06-23):** complete the fix + regression test and keep going — an
owner ear-check afterward is recommended but is **NOT a blocker**.

## 5. Album↔Track mode switch mid-audition doesn't re-push the live chain
**Severity: Medium · `src/hooks/useTrackMaster.ts:2513`**

Switching Album↔Track while a Mastered preview is playing doesn't re-push the live chain, so the stale
album-ness flag keeps applying until the user makes an unrelated edit or toggles A/B.

**Fix:** trigger a live chain re-push on mode switch when a Mastered preview is active.

## 6. Smaller correctness bugs (Low — fix now, low effort)
- **Concurrent analysis batches drop real progress** — `useTrackMaster.ts:477-485`: `finishAnalysis`
  can null `currentAnalysisBatchRef` while another batch runs, dropping its `analysis:progress` events.
- **"Export Anyway" bypasses the export re-entrancy guard** — `src/components/RightRail.tsx:83-86`:
  the review path doesn't honor the in-flight guard the primary button enforces (double-fire risk).
- **Android landing-measurement can crash on malformed native JSON** —
  `apps/android-native/.../AuditionController.kt:345-349`: no try/guard around Gson parse.
- **iPhone live audition decodes the source twice** — `apps/iphone-native/rust/src/live_stream.rs:355-365`
  (decode then re-decode for adaptive context). Perf, not correctness.
- **Album per-track WAV uniqueness TOCTOU** — `src-tauri/src/album_render.rs:674-682`: dedup check then
  `replace_with_tmp` can overwrite a same-named file that appears in between. (See also §10.)

---

# Fix now — missing stabilization tripwires (D8, additive tests only)

These are your non-negotiables. The behavior is correct **today**; the risk is that nothing pins it.
All fixes here are **additive tests** — zero behavior change. **This is the highest-leverage workstream.**

## 7. Adaptive-compressor gate is runtime-flippable with no "stays OFF" pinning test
**Severity: High · `src-tauri/src/guardrails.rs:75,85-102` + `src-tauri/src/lib.rs:126-127`**

`set_adaptive_compression` **is a registered Tauri command** that flips a runtime flag; the "gated OFF
by default" guarantee rests entirely on the static default. No test asserts the default is OFF.

**Fix:** add a test that the fresh-process default of `is_adaptive_compression_enabled()` is `false`,
and that no startup path (other than the explicit `ADAPTIVE_COMPRESSION` env in
`init_adaptive_compression_from_env`) flips it.

## 8. Export-level Volume-Match invariant rests on a single Rust line, with no FE guard or test
**Severity: High · `src/hooks/useTrackMaster.ts:1722-1779` (+ default `:102`)**

"Volume Match must not change export level" is enforced by one Rust line; the UI forwards
`volume_match: true` into the export payload (`:2065`, `:2205`) with no frontend-side guard or pinning
test that export level is identical with VM on vs off.

**Fix:** add a backend test (export with `volume_match` on vs off → identical output level/LUFS) and an
FE test that the export payload's effective level is VM-independent.

## 9. Compressor-Off bypass scope correct but unpinned at the UI boundary
**Severity: Medium · `src/components/AdvancedPanel.tsx:423-429`**

The "Off" tab writes only `compression_mode` (correct — it must not touch limiter/landing/format), but
nothing tests that it leaves those fields untouched.

**Fix:** snapshot test that selecting Compressor Off mutates only `compression_mode` and leaves
limiter/ceiling/LUFS/format fields intact.

## 10. Volume-Match-off default + export-overwrite primitive are unpinned (Low / Speculative)
- `useTrackMaster.ts:92-133`: VM-off default is only the in-file `DEFAULT_SETTINGS` literal — no test
  pins `volume_match: false` as the fresh-track default.
- *(Speculative)* `wav_writer.rs` `replace_with_tmp` will clobber an existing final path; overwrite
  protection lives entirely in callers, with no caller-agnostic regression test on the write primitive.
  Worth a defensive test even though no current caller triggers it.

---

# Fix now — wire/bindings parity gate has coverage holes (D2)

Your `bindings-drift.test.ts` is the safety net for the Rust↔TS hand-mirrored wire types. It has real
holes that let field drift ship silently — a direct risk to the "engine output bit-parity" promise.

## 11. Drift gate doesn't recurse into nested struct types
**Severity: High · `src/bindings-drift.test.ts:39-40, 80-92`**

The gate checks top-level shape but **does not recurse**, so field drift in nested types —
`SourceProfile`, `SpectralBalance6`, `CompressionBandPlan`, `CompressionPlanReason`, `Confidence`,
`AxisConfidence` — ships undetected.

## 12. `ProjectState` is entirely outside the gate
**Severity: High · `src/bindings.ts:463-477`**

A save/load/autosave wire type hand-mirrored Rust↔TS is not covered at all — drift here corrupts
project files.

## 13. More ungated hand-mirrored wire types (Medium)
- `src/lib/api.ts:36-63, 281-289` — album render payloads (`AlbumPlanRenderRequest`,
  `AlbumRenderReport`, `AlbumTrackRenderInput`, `AlbumTrackRenderRecord`).
- `src/bindings.ts:378-391` — `AudioOutputDevice`, `WaveformPeaks`.
- `src/bindings.ts:434-487` — `ExportReport`, `QualityCheck`, `UserPreset`.
- *(Low)* `bindings-drift.test.ts:70-78` — the `AnalysisResult.deep_analysis` allowlist silently
  depends on serde **not** using `skip_serializing_if`; a future such attr breaks the invariant.

**Fix (11–13 together):** make the drift gate recurse into nested structs and extend its type registry
to cover `ProjectState`, the album payloads, the export/quality/preset types, and the device/waveform
types. This is one focused PR that closes the whole class.

---

# Fix now — security / data-safety hardening (D3)

Local-first desktop reduces blast radius, but these are footguns.

## 14. Path-traversal guard rejects only `..`, not absolute paths or symlinks
**Severity: Medium · `src-tauri/src/files.rs:16-18` (enforced at `files.rs:25`, `exports.rs:186`, `project.rs:15/35`, `engine.rs:821`, `audio.rs:52/79/116`)**

The guard blocks `..` components but allows absolute paths, so frontend-supplied paths can read/write
anywhere the process can.

**Fix:** canonicalize and confine to an allowed base dir (or rely on Tauri capability scoping +
canonical-prefix check). Add a test with an absolute path and a symlink.

## 15. Android render bridge passes `output_dir` with no containment
**Severity: Medium · `apps/android-native/rust/src/lib.rs:116-148`** — same class as §14, mobile side.

## 16. Lower-severity IO footguns (Low)
- `exports.rs:180-225` `open_output` spawns the OS file manager on a frontend-supplied path with no
  canonical/base confinement.
- `wav_writer.rs:180-206` / `engine.rs:1054` / `album_render.rs:244-265` — export overwrite protection
  is TOCTOU (existence check precedes truncating create/rename).

---

# Fix now — documentation drift (D5/D6, trivial edits)

## 17. Public landing page lists the wrong style names
**Severity: High · `src/landing/CrossPlatform.tsx:6`**

`{ title: "Four styles", body: "Balanced, Warm, Open, Punch." }` — none are the app's real Standard
styles (**Universal / Clarity / Tape / Oomph**). Contradicts `docs/landing-brief.md`, the app vocabulary,
and the parity fixture. *(Owner decision 2026-06-23: the landing is a mockup and IS in scope to change —
but the **Hero section look is locked**; keep the rest of the landing visually consistent with Hero.)*
**Fix:** correct to the real four styles (Universal / Clarity / Tape / Oomph). While in the landing,
keep every visual change consistent with the established Hero-section look.

## 18. `ENGINE_REFERENCE.md` is stale after the 85% lean
**Severity: Medium · `docs/ENGINE_REFERENCE.md:34-38` and `:130`**

The preset dB table predates the 2026-06-22 re-voicing, and the §5 worked example is now factually
wrong (Universal's low boost crossed the +0.5 dB character floor it cites). *(This is also flagged as
owner doc-check #21.)*
**Fix:** regenerate the table + example from the shipped 85%-lean constants.

## 19. `OPEN_THREADS_AND_DECISIONS.md` still lists the macOS SHA task as red/open
**Severity: Low (doc drift I confirmed during synthesis) · `docs/OPEN_THREADS_AND_DECISIONS.md:25,95`**

Thread #1 reads *"Most actionable open thread (CI is red)"* for the macOS byte-identity SHAs — but
`HANDOFF_2026-06-22_MACOS_VERIFICATION.md` (RESOLUTION) and commits `cc03d56`/`88853dc` record it as
**done and green**. Move thread #1 to the SHIPPED/closed list. (Thread #18 `stereo_width` is correctly
still open — see §22.)

---

# Hold / owner-gated — forgotten or deferred work (D5/D6)

Not bugs; surfaced so nothing is silently dropped. Each needs your call.

- **AC-5 adaptive-compressor calibration + default-gate flip** *(speculative/expected)* — built, never
  run; 9 `TBD-CALIBRATION` constants unlocked, gate OFF. This is intentionally Wave-10/owner-gated.
- **PRODUCT.md S5.4 canon refresh half-landed** — `docs/PRODUCT.md:210-243`: Mobile, Album Master, and
  Public-Surface product definitions still "pending owner definition." Owner interrogation needed.
- **Preset-fingerprint measurement harness unbuilt** — `docs/plans/2026-06-15-...:86-100`: the mechanical
  Wave-10 retune gate was promised but not built. (Pairs with the preset-research work in your memory.)
- **`stereo_width` computed-and-carried-but-unread** — open thread #18: wire it as a width co-trigger or
  delete the inert field. Decision, not a bug.
- *(Speculative)* `deep_analysis::window_detail_features` allocates a fresh FFT scratch `Vec` per window
  (no reuse) right beside a reused `mono_scratch`, contradicting the per-window cost budget. Perf only;
  matches the long-standing `deep_analysis.rs` TODO.

---

# Hold / batch — dead code & bloat (D4, 17 confirmed)

Low risk, low urgency; do as one mechanical "delete-only" commit (your refactor-backlog style). Reduces
future-edit surface. None are behavior-bearing.

**Rust**
- `dsp.rs:1700-1735` `EnvelopeFollower` — dead in prod (only a self-referential test).
- `profile_store.rs:45-50` `insert` — unused; duplicates `set(id, Some(p))`.
- `types.rs:332/336/382/422` — `DeliveryProfile::is_custom` + three `display_name()` methods never called.
- `deep_analysis.rs:190-191,223,316` `harsh_share`/`sibilant_share` — computed every analysis, never read.
- `deep_analysis.rs:195-199,214-222` — 4 of 5 `AxisStrata` aggregates computed but unread (only
  `crest.dispersion` is consumed).
- `audio.rs:1035-1058` `LandingGainCache::get_or_compute`/`len` — test-only, re-implements prod path.
- `types.rs:926-927` `CommandError::NotImplemented` — dead error variant, never constructed.

**Frontend**
- `src/components/fields.tsx:112-249` — `NumberField` `autoReadout`/`autoLabel` props never passed.
- `src/components/AdvancedPanel.tsx:438-483` — `autoThreshold/autoRatio/autoAttack/autoRelease` label
  strings computed but never read; and `:593` duplicates the `bandLabel` helper (`:619-621`) inline.
- `src/assets/landing/advanced-angled-ui.png` + `hero-fullscreen-control-room.png` — **~3.2 MB of
  unreferenced PNGs**. Delete (biggest single space win).
- `src/lib/standard-managed.ts:111-121` `shouldForceAdvancedOnStandardEntry` — single-caller-in-test
  wrapper around `forceAdvancedOnStandardEntry`.
- `src/components/Knob.tsx:25,42` — `KnobTone` "pink" member + `TONE_COLOR` entry, never used.
- `src/App.tsx:1876-1878` — re-exports `AdvancedPanel` only to keep test imports on `"./App"`.

**Mobile**
- `apps/iphone-native/.../NativeMasteringBridge.swift:48,70,74` — dead `supportedImportSummary`,
  `fixedExportSummary`, `bridgeVersion`.
- `apps/android-native/.../NativeBridge.kt:20,32` — dead `bridgeVersion()` + extern.
- `apps/iphone-native/.../RenderStorage.swift:6,15,23,32` — dead preview-pruning machinery.
- `apps/iphone-native/rust/Cargo.toml:9` — `cdylib` crate-type nothing links.

---

# Hold / batch — CSS & styling debt (D7, 10 confirmed)

All in `src/App.css` unless noted. Cosmetic; batch as a styling-debt PR. Ground-level fixes given.

- `:747-751` `.empty-foot` uses **two `!important`** to win a specificity war it created vs
  `.empty-state p` — fix the base selector instead.
- `:2372-2394` `.toast` base positioning is dead — `.toast-stack .toast` immediately re-undoes it; every
  toast renders in the stack. Delete the base block.
- `:5256-5342` `.std-tile` **re-implements the entire `.tile` material system verbatim** (`:1744-1858`).
  Share one class (~90 dup lines).
- `:1783-1786` + `:1821-1829` `.tile:hover` split into two non-adjacent duplicate blocks — merge.
- `:4039-4059, 4659-4661` dead CSS on permanently-hidden `.chain-link` (self-cancelling margin +
  never-applied media override + animated `.is-hot`).
- `:4482-4486` `.wf-overview` paired magic offset (`calc(100% - 30px)` + `margin-right: 30px`) re-undoing
  the base full-width rule (`:1329-1338`).
- `:505,518,4317,5045,5669,5673` rail widths `300px`/`260px` hard-coded in 5+ places incl. a
  `calc(300px + 24px)` magic offset — tokenize (the repo already tokenizes other widths).
- `:349,1048,1134,2320,2369,2387,2432,5597` + `FirstRunOverlay.tsx:57` — overlay z-indexes are
  un-tokenized magic numbers (multiple layers share 100/120). Introduce a z-index scale.
- `src/components/AdvancedPanel.tsx:213-247` `.adaptive-readout` has a className but all layout is in a
  static inline style block — move to the class.
- `:4381-4384` redundant media-query restatement of the base `.console-hero .track-header { display: block }`.

---

# Speculative (8) — unconfirmed, awareness only

Minority skeptic support only; listed for awareness, **not** scheduled. (Several overlap items already
captured above as defensive tests.)
- Seek right after a cold (uncached) `PlayMaster` may block behind synchronous decode and trip the 2s
  seek-reply timeout. *(D1)*
- `decode_to_peaks` field named `total_samples` actually carries decoded **frames**. *(D1, naming)*
- Album continuous WAV streams per-track buffers without `write_wav`'s non-finite guard. *(D1)*
- Album source-rate/channel probe arrays can desync if a plan `track_id` is absent from inputs. *(D1)*
- AC-5 calibration never run (captured under owner-gated above). *(D6)*
- `deep_analysis` per-window FFT scratch not reused (captured under hold/perf above). *(D6)*
- `write_wav`/`replace_with_tmp` clobber + no caller-agnostic test (captured in §10). *(D8)*
- Album per-track Volume-Match guard rests on a single reassignment line. *(D8)*

---

# Suggested remediation sequence

Grouped into small commit batches — each is a handful of tiny commits straight to `main` (no pull
requests). Recommended order:

1. **Batch A · Stabilization tripwire tests** *(§7–§10, additive only, no behavior change)* — **do first.**
   Pins the adaptive gate OFF, the volume-match export invariant, compressor-Off scope, and VM-off
   default. Highest leverage, lowest risk.
2. **Batch B · Bindings-drift gate hardening** *(§11–§13)* — recurse into nested structs + register the
   missing wire types. Closes the silent-parity-drift class.
3. **Batch C · Live-audition correctness fixes** *(§3, §4, §5)* — transport leak, live ceiling staleness,
   mode-switch re-push. §4 touches the real-time chain → include a regression test; owner ear-check is
   non-blocking (complete it and continue).
4. **Batch D · Small correctness fixes** *(§6)* — analysis-batch progress, Export-Anyway re-entrancy,
   Android Gson guard, album TOCTOU.
5. **Batch E · Security hardening** *(§14–§16)* — path confinement (desktop + Android), open_output base
   check, overwrite-primitive test.
6. **Batch F · macOS / cross-platform goldens** *(§1, §2)* — replace the exact-byte goldens with an
   arch/OS-independent tolerance (or rounded-hash) golden; add the Swift VolumeMatch finite-guard.
7. **Batch G · Doc-drift fixes** *(§17–§19)* — landing style names (keep Hero look), ENGINE_REFERENCE
   regen, OPEN_THREADS close-out.
8. **Batch H · Dead-code delete batch** *(D4)* — mechanical, delete-only.
9. **Batch I · CSS debt batch** *(D7)* — styling-only.
10. **Owner-gated backlog (leave parked — needs Dan)** — S5.4 canon refresh, preset-fingerprint harness,
    `stereo_width` disposition, AC-5 calibration. No code before your listening/decision notes.

**If you only do three things:** Batch A (tripwires), Batch B (parity gate), Batch C/§4 (the live-ceiling
bug that breaks "A/B matches export").
