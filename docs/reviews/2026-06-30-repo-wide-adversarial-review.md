# Repo-Wide Adversarial Review — YES Master (2026-06-30)

**Date:** 2026-06-30
**Scope:** whole repo at `main` (`dae43b3`), weighted toward everything shipped since the
2026-06-23 audit (`docs/reviews/2026-06-23-repo-wide-audit-macos-focus.md`) — the
release/signing pipeline, the draft legal docs, and the landing-page rebrand — while
still sweeping the long-stable core (Rust DSP/engine, frontend, both mobile bridges).
**Method:** dynamic multi-agent audit, 25 finder units across 8 dimensions (D1
correctness, D2 cross-language parity, D3 security, D4 dead code/bloat, D5 spec
conformance, D6 forgotten features, D7 CSS debt, D8 stabilization risk), plus two
loop-until-dry deep dives (DSP math correctness, cross-language parity) that each ran
4 rounds, plus a completeness-critic round that seeded 4 more targeted investigations.
**233 subagents, ~3,839 tool calls.** Every candidate finding was adversarially checked
by 3 independent skeptics (perspective-diverse: does-it-reproduce / is-it-guarded /
does-it-matter); a finding only counts here if ≥2 of 3 said "real." 65 candidates → **47
confirmed**, 7 refuted, 11 speculative (minority support only, listed separately, not
actionable).

> **Run note:** the first attempt at this audit was almost entirely wiped out by a
> transient Anthropic API rate limit a few seconds into the run (28 of ~30 finder
> agents failed before producing a single finding). It was re-run in smaller batches
> and completed cleanly the second time; the numbers above are from the successful run.

> **So what (read this first).** No critical bugs, no secrets in the repo, no
> export-overwrite hole, no live DSP-voicing bugs. The codebase is in genuinely good
> shape for a solo-dev pre-launch app. The real risk surface is **all in the brand-new
> stuff shipped this week**: (1) the landing page's verification script
> (`scripts/verify-landing-responsive.mjs`) checks DOM selectors from a hero design that
> no longer exists — `npm run verify:landing` cannot currently pass *or* meaningfully
> fail, so it's silently not protecting anything; (2) the new GitHub Actions release
> pipeline (`.github/workflows/release.yml`) has **zero test/lint/version-check gate**
> before it builds, signs, and drafts a public release — four independent finder passes
> converged on this same gap; (3) the draft legal docs (EULA, refund policy) describe a
> free/paid export lock and license activation that **do not exist in the app yet**; and
> (4) one real, fixable real-time-audio-safety issue was found in the live mastering
> chain: every live knob/preset tweak during playback allocates on the audio thread,
> even though a non-allocating sibling function already exists and is already
> unit-tested — it's just not wired up at the one call site that matters.
> **Start with the live-audio allocation fix and the release-pipeline test gate** — both
> are small, additive, and directly protect this repo's own non-negotiables
> (real-time responsiveness; not shipping a broken/regressed build).

---

## Severity / dimension breakdown (confirmed findings)

| Severity | Count | | Dimension | Count |
|---|---|---|---|---|
| High | 5 | | D1 correctness | 11 |
| Medium | 15 | | D2 parity | 6 |
| Low | 27 | | D3 security | 3 |
| **Total** | **47** | | D4 dead code | 5 |
| | | | D5 spec conformance | 2 |
| | | | D6 forgotten features | 2 |
| | | | D7 CSS debt | 6 |
| | | | D8 stabilization risk | 9 |
| | | | critic-seeded (cross-cutting) | 3 |

47 confirmed findings are presented below as **~38 line items** — several of the
strongest findings were independently rediscovered by 2–4 different finder lenses
(different dimensions, same root cause). I've merged those and noted the corroboration
instead of listing duplicates; independent rediscovery is a *stronger* confidence
signal than a single finder's claim, not noise.

### Coverage caveats (no silent caps)
- **Not inspected:** private audio fixtures (gitignored, correctly), `target/` build
  artifacts, `node_modules/`, `graphify-out/`, the actual rendered visual output of the
  CSS/landing findings (this sandbox cannot bind a browser — every D7/landing finding
  below is a **static source-code read**, not a visual verification).
- The DSP-math deep dive read `dsp.rs`/`engine.rs`/`analysis.rs`/`deep_analysis.rs`
  line-by-line across 4 rounds; the cross-language-parity deep dive read every wire
  boundary (Rust↔TS↔Swift↔Kotlin) across 4 rounds. Both stopped after 2 consecutive
  empty rounds (the loop-until-dry exit condition), not an arbitrary cap.
- Per CLAUDE.md's working style, no preset/DSP **voicing** was second-guessed — the one
  DSP finding that's closest to a creative-parameter change (the album intensity clamp,
  low-priority below) is flagged for awareness, not prescribed as an immediate change.

---

## Fix now

Bugs, additive guardrail tests, and doc corrections. None of these change DSP voicing.
Two items (marked) touch the live mastering chain / true-peak detection and should go
through the private-fixture slow lane per `CLAUDE.md`'s verification section before
merging, even though neither is a voicing change.

### High severity

#### 1. Release pipeline has no test/lint/version gate before building & drafting a public release
**`​.github/workflows/release.yml`** · confirmed independently by **4 separate finder
passes** (D8, D8-stabilization, and the completeness critic) — the strongest-corroborated
finding in this audit.

`release.yml`'s steps are: checkout → setup node/rust → `npm ci` → (optional signing) →
build+sign+draft via `tauri-apps/tauri-action@v0`. There is no `npm test`, no `cargo
test`, no `cargo clippy`, no `cargo fmt --check` anywhere in the file — contrast with
`ci.yml`'s windows/macos jobs, which run all of these. Three concrete consequences:
- A regression in `cargo test` (including the DSP/export snapshot tests) would not
  block a tagged release from building signed installers.
- `src/lib/version-coherence.test.ts` already exists specifically to pin
  `package.json`/`tauri.conf.json`/`Cargo.toml` to the same version — but it only runs
  inside `ci.yml`'s `windows` job, never inside `release.yml`. The three files happen to
  agree today (`0.1.0`), but nothing in the release pipeline itself would catch future
  drift.
- The pushed **tag** (e.g. `v0.2.0`) is never cross-checked against the version actually
  baked into the manifests — a typo'd tag or a forgotten version bump ships installers
  whose embedded version doesn't match their release name, with no CI signal.
- A manual `workflow_dispatch` run resolves `tagName` to the literal string
  `yes-master-v__VERSION__` with no run id/timestamp — see the separate tag-collision
  finding below, which compounds this.

**Fix (additive only):** add a fast pre-build job (`npm test` + `cargo test --lib`) that
the three matrix legs `needs:`, so a regression fails the workflow red before any
installer is built; add a one-line guard that strips the tag's `v`/prerelease suffix and
asserts it equals `package.json`'s version before building.

#### 2. `npm run verify:landing` cannot currently pass or fail meaningfully — it checks a hero/nav DOM structure that was deleted
**`scripts/verify-landing-responsive.mjs:73-160`** · confirmed independently by **3
separate finder passes** (D1, D4, D6), plus a related but unconfirmed (1/3 vote) D7
finding flagging the same root cause.

The script's structural assertions query `.landing-hero`, `.landing-hero-scene`,
`.landing-hero-copy`, `.landing-hero-hotspots`, and `.landing-nav` — none of these class
names exist anywhere in `src/` (confirmed via repo-wide grep: zero matches). They're
leftovers from a pre-rewrite image-map/hotspot hero design; the current `Hero.tsx` /
`Nav.tsx` use plain Tailwind utility classes. Every `querySelector` call resolves to
`null`, so `metrics.imageFit`/`copyDisplay`/`hotspotsDisplay` are always `null` and can
never equal the expected `"cover"`/`"fill"`/`"none"` strings — **the script fails
unconditionally on every viewport regardless of whether the actual page is correct.**
CLAUDE.md explicitly instructs agents to run this script whenever the landing page is
touched; right now doing so tells you nothing.

A second, related but distinct bug in the same file: `requiredAnchors` (line 22) omits
`#top` while `requiredSections` (line 21) includes it — a duplicated, hand-typed
source-of-truth that's already drifted once.

**Fix:** rewrite the structural checks against the current Tailwind component tree (the
real `<section id="top">`/hero `<img>`/copy column/`<nav>`), dropping the obsolete
overlay-vs-image-map composition model entirely since the rewrite has one composition,
not two. Derive the anchor lists from one shared array instead of two independently
typed ones. Then actually run `npm run verify:landing` once and record a real pass.

#### 3. Live audio-thread coefficient swap allocates on every knob/preset tweak during playback
**`src-tauri/src/sources.rs:519`**, hot path `MasteringSource::next()` (the Rodio
iterator the audio callback thread pulls samples from directly).

```rust
self.pending_chain = Some(crate::dsp::MasteringChain::with_coeffs_inheriting_state(update.coeffs, &self.chain));
```

`with_coeffs_inheriting_state` (`dsp.rs:2087-2102`) clones `prior.limiter` and
`prior.states`, both of which heap-allocate fresh `Vec`s — on every coefficient update,
which fires roughly every 3 ms during live playback (i.e. on essentially every
real-time-audible knob move). Heap allocation on a real-time audio callback thread is a
classic glitch/dropout source, and **this repo already built and unit-tested a
non-allocating sibling** for exactly this purpose:
`overwrite_with_coeffs_inheriting_state` (`dsp.rs:2108-2127`), proven via
pointer-identity assertions (`dsp.rs:3390-3415`) to reuse the target's existing
allocations, with its own doc comment calling itself "the real-time-safe sibling... used
by `MasteringSource` to crossfade... without re-ringing the filters." It just isn't
wired up at the one call site that needs it.

**Why it matters:** directly touches the "real-time/near-real-time audition must stay
responsive" non-negotiable. **Fix (additive, low-risk):** swap the call site from
`with_coeffs_inheriting_state` to the already-built, already-tested
`overwrite_with_coeffs_inheriting_state`. Add a pinning test (a counting allocator under
`#[cfg(test)]`, or reuse the existing pointer-identity pattern exercised through
`MasteringSource::next()` itself) so a future edit can't silently reintroduce the
allocation. **Run the realtime sweep (RELEASE_STABILIZATION's "Realtime Sweep
Confirmation" gate) after this change**, since it touches the live chain update path.

#### 4. Draft legal docs (EULA, Refund Policy) describe a free/paid export lock and license activation that don't exist in the shipped app
**`docs/legal/EULA.md:22-31`**, **`docs/legal/REFUND-POLICY.md:12-14`**.

EULA §2: *"The free version of the Software... does not export/render finished files.
Exporting requires a paid license."* §3: *"The Software performs a one-time online
activation to validate the key..."* Refund Policy repeats: *"only exporting is
locked."* Verified directly against `src-tauri/src/engine.rs` — `render_track_master`
(line 571) and `render_album_plan` (line 734) render unconditionally; there is no
license/activation check anywhere in either call path, and `Cargo.toml` has no HTTP
client capable of "one-time online activation." `docs/plans/2026-06-30-launch-plan.md`
correctly tracks "Export gate (free/paid boundary)" and "License activation" as
**unchecked P0/P1 items** — the legal docs were drafted as the target-state contract,
but as worded today they assert the gate already exists.

**Why it matters:** if either doc ships (bundled with an installer, posted publicly)
before the export gate lands, every install is fully unlocked and unrestricted — a
factual misrepresentation to anyone who reads the EULA/refund terms before installing.

**Fix:** keep §2/§3 as the target-state contract for 1.0, but add an explicit
draft-status caveat that the export-lock/activation behavior described is **not yet
implemented**, and don't bundle/publish either document with a real build until the
P0/P1 engineering items ship (or it's confirmed acceptable to publish ahead of the
code, which is an owner call).

### Medium severity

#### 5. `workflow_dispatch` (manual) release runs always produce the same draft-release tag — repeat runs can silently collide
**`.github/workflows/release.yml:122`** · confirmed independently 3 times (D1, D3, D8).

```yaml
tagName: ${{ github.ref_type == 'tag' && github.ref_name || 'yes-master-v__VERSION__' }}
```

For a manual (non-tag) run, `tagName` is the literal `yes-master-v__VERSION__`,
substituted from `tauri.conf.json`'s static version (currently `0.1.0`) — with no run
id, timestamp, or nonce. Two manual "Run workflow" invocations before the version is
bumped (e.g. a retry after a flaky build, or two people testing) resolve to the
identical tag; `tauri-action` upserts a release by tag, so the second run's artifacts
land in/overwrite the first run's still-open draft rather than failing loudly or
creating a separate release.

**Fix:** for non-tag runs, append a disambiguator — `github.run_id` or
`github.run_number` — to the manual-run `tagName`.

#### 6. SignalChain's saturation display values have drifted from the real DSP table after the 85% lean
**`src/components/SignalChain.tsx:20-43`** (the drift) + **`src/components/SignalChain.test.tsx`** (why it went uncaught).

`presetSaturation()` is a hand-maintained "mirror of the `saturation_amount` table in
`dsp.rs`" — its own comment says "Kept in sync by hand — if those numbers change, update
here too." Three of eight presets are now stale post-retune (worst case **Tape: displays
0.1, engine uses 0.15 — ~33% low**). This is purely a UI display bug (the Signal Chain
strip's "Saturation" glow + "% drive" hover text) — the actual audio output is correct,
only the visualization under-reports. It went uncaught because, unlike the sibling
`stereo_width`/compressor mirrors (which got a generated tripwire test in
`src/lib/preset-mirrors.test.ts` / `src/preset-mirrors.json` specifically because "both
mirrors have drifted before"), saturation was left out of that net — and
`SignalChain.test.tsx` has exactly one test, asserting only DOM structure, never the
saturation values themselves.

**Fix:** extend the existing golden-value pattern — add `saturation` to the JSON
`src-tauri/tests/preset_mirrors.rs` emits, regenerate `src/preset-mirrors.json`, export
`presetSaturation` (mirroring `presetDefaultWidth`), and correct the three stale
literals (universal 0.03→0.055, tape 0.1→0.15, warmth 0.08→0.13).

#### 7. No mechanical pin on undo/redo's live-chain re-push
**`src/hooks/useTrackMaster.ts:992-1031`**

The code comment explicitly documents the regression it prevents: *"Without this, undo
would change the UI state but the audible output would lag until the user toggled
Original/Master."* Despite that, `useTrackMaster.integration.test.tsx` has zero
references to `undo(`, `redo(`, or `restoreSnapshot` — completely unexercised.

**Fix:** add a test — load Mastered, edit, commit history, `undo()`, assert
`api.updateChain` was called with pre-edit settings; mirror for `redo()`.

#### 8. Track-vs-album export routing has no integration test pinning the four independent ternaries together
**`src/App.tsx:376`**

`exportMode`, `canExport`, `isExporting`, and `onExport` are each separately keyed off
`tm.mode === "album"`, four lines apart — new wiring from this week's album/2-up
consolidation. No test renders the real `App`, clicks the rendered button, and asserts
the *album* handler (not the track handler) actually fires in album mode.

**Fix:** one assertion in `App.album-export.test.tsx` per mode, asserting the correct
mock fired and the wrong one didn't.

#### 9. Limiter's inter-sample-peak skip heuristic uses a margin smaller than the interpolator's actual worst case
**`src-tauri/src/dsp.rs:1559-1587`** *(touches true-peak detection — run the private
fixture slow lane after fixing)*

```rust
const ISP_SKIP_MARGIN: f32 = 1.2;
if frames >= 4 && peak * ISP_SKIP_MARGIN > self.ceiling_lin { /* full ISP scan */ }
```

The comment claims inter-sample peaks exceed the raw sample peak by "at most ~+1 dB."
The actual Lagrange-4 x=0.5 tap coefficients (`[-0.0625, 0.5625, 0.5625, -0.0625]`) give
a worst-case gain of 1.25× (+1.94 dB) for a steep two-sample transient edge — not a
contrived case. In the headroom band between roughly −1.94 dB and −1.58 dB below
ceiling, the full scan is **skipped** even though a genuine inter-sample peak there can
exceed the ceiling by up to ~0.36 dB.

**Fix:** raise `ISP_SKIP_MARGIN` to cover the interpolator's real worst-case gain
(≥1.25, derived from the actual tap-coefficient sum, not an empirical estimate), or drop
the skip optimization's safety claim and re-derive it mathematically.

#### 10. iPhone's Swift wire model silently drops the adaptive-traceability digest fields the Rust bridge emits and Android decodes
**`apps/iphone-native/YESMasterNative/NativeMasteringBridge.swift:15-21`** (+ a related
Android-side test-coverage gap, `apps/android-native/rust/src/lib.rs:350-378`)

`NativeRenderedMeasurements` declares only `lufsIntegrated`/`truePeakDbtp`/
`dynamicRangeLu`/`sampleRate`/`bitDepth`; `RenderedMeasurements` in `types.rs:759-782`
also carries `effective_adaptive_strength`, `source_profile_digest`,
`confidence_digest`, `compression_digest` (already proven present on the real native
payload by the iPhone Rust crate's own test). `JSONDecoder` silently ignores unknown
keys, so the iPhone build compiles and runs fine but can never surface the same
adaptive-DSP traceability desktop shows on its export receipt
(`ExportReceiptCard.tsx:103-121`) — the data exists, it's just invisible on iPhone only.
Separately, Android's own JNI-path test never asserts these 4 keys either (a desktop
fixture test covers the Kotlin decode shape, but not Android's actual render output).

**Fix:** add the four optional fields to `NativeRenderedMeasurements` (mirroring
`WireMeasurements` in `WireModels.kt`) plus a Swift test that decodes a real payload and
round-trips the digests; add the equivalent assertion to the Android Rust test.

#### 11. Desktop's own import-extension allowlist is a fourth independent hardcode, pinned only against itself
**`src/lib/supported-formats.ts:1`** (+ siblings: `apps/android-native/.../Repositories.kt:53`,
`apps/iphone-native/.../NativeMasteringBridge.swift:46`, plus two related test-coverage
gaps in the Android/iPhone Rust crates)

`AUDIO_EXTENSIONS = ["wav","mp3","m4a","aac","flac","ogg"]` is the **primary desktop
platform's** copy of "what we can decode" — but `src-tauri/src/decode.rs` has no
allowlist at all; it just hands the extension to `symphonia`'s probe and accepts
whatever the installed codec registry supports. Unlike the mobile bridges (which at
least have a real FFI call backing their lists), desktop's list is pinned only by a
self-referential test (`supported-formats.test.ts` asserts the array equals itself). If
`symphonia`'s enabled codec features ever change, the desktop dialog/empty-state copy
could silently advertise formats the engine can no longer decode, or vice versa.

This is one of **five related findings** across desktop/iPhone/Android extension lists
(the others are lower severity — see the Low section). All five share one fix shape:
derive each platform's advertised list from the real decoder capability instead of a
hand-typed literal.

**Fix:** add a small exported Rust constant/command reporting the canonical
symphonia-backed extension set, and assert `AUDIO_EXTENSIONS` against it.

#### 12. No mechanical check that `package.json` / `tauri.conf.json` / `Cargo.toml` stay in sync before a release build
**`src-tauri/tauri.conf.json:4`** — folded into Fix-now item #1 above (same root cause:
the release pipeline runs no checks). Listed here only because it was independently
flagged with its own evidence: `RELEASE_SIGNING_SETUP.md` documents the "three files
must match" rule as a manual habit with no automated enforcement anywhere in CI.

#### 13. EULA/Refund Policy publish-readiness — see Fix-now #4 above (Refund Policy repeats the same not-yet-true claim; same fix).

### Low severity (mechanical, cheap, low-risk)

- **`write_samples_into_writer` has no non-finite guard on the 32-bit float path**
  (`src-tauri/src/wav_writer.rs:121-157`) — unlike `write_wav`, which checks
  `is_finite()` before writing. `f32::clamp` does not sanitize `NaN`; a NaN sample
  reaching the 32-bit float branch would be written verbatim, silently corrupting the
  master (the 16/24-bit integer branches happen to self-heal via float→int cast
  saturation). Not reachable today — the only caller (`album_render.rs:711/720`)
  already validated the same buffer via a prior `write_wav` call — but it's a latent gap
  in a shared `pub(crate)` primitive. **Fix:** move the finite-check into
  `write_samples_into_writer` itself so every future caller inherits it.
- **Stale doc comment claims the renderer never resamples — it does**
  (`src-tauri/src/types.rs:302-304`). `output_sample_rate()`'s doc comment says "A3
  does NOT resample" but `engine.rs:510-513` explicitly calls `convert_interleaved(...)`
  via `effective_sample_rate()` today — a CD-rate profile on a 48kHz source really does
  get resampled. **Fix:** update the comment; no code change.
- **`JobStatus` enum has 3 of 4 variants never constructed** (`src-tauri/src/types.rs:728-736`)
  — only `Done` is ever built; `Pending`/`Running`/`Failed`/`Cancelled` are dead, and the
  TS binding still advertises a `"cancelled"` variant that can never arrive. **Decision,
  not pure mechanics:** either wire up real progress reporting or collapse the enum —
  flagging for a call rather than prescribing one.
- **`resetGuide` exported and tested but has zero production callers** (`src/lib/first-run-guide.ts:49`)
  — the live reset path is `requestGuideReset`; `resetGuide` is reachable only from its
  own test. **Fix:** delete it and its test, or wire a real caller if a hard-clear
  semantic is actually wanted.
- **`formatSampleRate`/`formatDbtp` duplicated verbatim** between `standard-export.ts`
  and `chrome-content.ts` (`src/lib/chrome-content.ts:7-14`) — `chrome-content.ts`
  already imports from `standard-export.ts`, so this is an easy dedup.
- **Apple signing `.p12`/keychain never cleaned up at end of job** (`.github/workflows/release.yml:85-97`)
  — found independently from both a correctness (D1) and security (D3) lens. Low risk
  today (GitHub-hosted runners are ephemeral) but a footgun if the workflow ever moves
  to persistent runners or a later step broadens an artifact-upload glob. **Fix:** add
  an `if: always()` cleanup step (`rm -f certificate.p12`, `security delete-keychain`).
- **`fail-fast: false` + `releaseDraft: true` means a partial matrix failure leaves an
  incomplete draft with no loud signal** (`.github/workflows/release.yml:29-41,125`) —
  e.g. the Windows leg fails after both macOS legs already attached installers; the
  draft looks complete to anyone skimming the Releases page instead of the Actions tab.
  **Fix:** add a trailing `if: always()` job that checks all 3 matrix results and flags
  an incomplete draft.
- **iPhone error messages (including raw Rust-side file paths) surface verbatim into
  user-visible status text** (`apps/iphone-native/YESMasterNative/AuditionController.swift:522-532`)
  — low impact (sandboxed container paths only), cheap to scrub.
- **`docs/landing-brief.md`** (the documented landing-copy source-of-truth, which says
  "present tense only, no roadmap language, no 'coming soon'") **was not reconciled with
  the shipped rewrite**, which now intentionally uses "coming soon" framing for mobile
  per the 2026-06-30 launch plan. Not a functional bug — the live page is correct — but
  a future agent following the brief literally could regenerate copy that strips the
  now-intentional roadmap framing. **Fix:** one-paragraph addendum to the brief.
- **Per-window stereo correlation falls back to `+1.0` ("perfectly correlated") instead
  of "unknown" when one channel has zero variance** (`src-tauri/src/deep_analysis.rs:627-665`)
  *(touches the live Tier-1 width-confidence axis — re-run the private fixture lane
  after fixing)*. Mathematically, Pearson correlation is undefined (not 1) at zero
  variance; the sibling track-level function (`analysis.rs:567`) already returns `None`
  in the same case. A silent/constant channel currently gets counted as evidence of
  *not* being wide, which can lower width-trim confidence on hard-panned or
  single-channel-active material. **Fix:** return `NaN` (already filtered by
  `is_finite()` elsewhere) instead of `1.0`, mirroring `analysis.rs`'s behavior.
- **`IntegratedLufs` running sum lacks the non-negative clamp its own (copy-pasted)
  comment promises** (`src-tauri/src/dsp.rs:1938-1941`) — the sibling `MomentaryLufs`
  has the `.max(0.0)` guard against f64 cancellation drift over long sessions;
  `IntegratedLufs` (which backs the live audition meter) doesn't. Empirically the
  realistic drift is tiny (verified: exact after 50M simulated frames), so this is
  low-risk, but it's a one-line fix to match the documented invariant.
- **Album per-track intensity shadow permits up to 1.5, double the DSP's actual [0,1]
  clamp** (`src-tauri/src/album_render.rs:298`) — not audible today (downstream
  re-clamps to 1.0 anyway), but it means album `intensity_scale` values between ~0.67×
  and 1.5× all collapse onto the same ceiling instead of producing the differentiated
  per-track intensity the arc/character system intends. Flagging for awareness; not
  prescribing a change without checking intent against the album character design.
- **Five related cross-platform "supported extension list" parity gaps** (D2,
  low-severity siblings of Fix-now #11 above): Android's `ImportPolicy` list
  (`Repositories.kt:53`) isn't pinned to the Rust decoder; iPhone's `knownAudioExtensions`
  (`NativeMasteringBridge.swift:46`) is a third independent hardcode; Android's own
  Rust-side extension test (`apps/android-native/rust/src/lib.rs:324`) omits `"aac"`
  from its coverage loop despite the test's name implying full coverage. None are live
  bugs (each platform's actual gating still calls through to the real Rust decoder) —
  they're all coverage/drift-detection gaps on the *rejection-message* text and test
  completeness, not behavior bugs.
- **CSS — see the Hold section below** (band-aid styling found in `App.css`; deferred
  per the same "needs a browser" rationale as the 2026-06-23 audit's CSS batch).

---

## Hold / needs-browser or owner judgment

#### CSS debt (D7) — needs visual verification this sandbox can't provide
Same rationale as the 2026-06-23 audit's deferred CSS batch (`OPEN_THREADS_AND_DECISIONS.md` #11a):
these are real, but changing CSS without a live preview risks shipping an unverified
visual regression. All found in files touched this week (the 2-up Standard layout,
album-master CSS pass):

- `src/App.css:5375-5384, 5458-5476` — the 2-up Intensity card silently swaps
  `display: grid` for `display: flex` via a higher-specificity `.std-pair` override,
  making the base rule's `grid-area` assignments dead in that context. Two parallel
  positioning systems for the same DOM, switched by a wrapper class.
- `src/App.css:4740-4750, 4872-4878, 5005-5015` — Knob wrapper width re-capped with
  hand-picked `max-width` values at 3 breakpoints, independent of `Knob.tsx`'s own
  `SIZES` table; must be manually re-tuned in lockstep or the knob silently
  crops/squishes.
- `src/App.css:4240-4243, 4263-4267` — `.knob-vis` declared as two non-adjacent rule
  blocks (harmless today, but a future edit to one copy could diverge from the other).
- `src/App.css:893-899` — dead `justify-items: end` left over from a grid→flex
  migration (`justify-items` has no effect on flex containers).
- `src/App.css:4385-4392, 4819-4827, 4903-4910` — `grid-template-rows` pixel heights for
  `.is-album` hand-duplicated across 3 breakpoints with no shared token (same magic-number
  pattern already tracked for rail widths, now spreading to a new property in this
  week's album-CSS pass).
- *(Speculative, 1/3 votes — not confirmed, flagging for awareness only)* the new 2-up
  Intensity knob is resized via `transform: scale(1.12)` rather than a real size
  variant, and `FirstRunOverlay`'s coachmark position re-duplicates the already-known
  `300px` rail-width magic number a second time.

#### Owner-judgment / no action prescribed
- **`Export's Volume Match override is duplicated at 3 call sites** (`engine.rs:383,
  457, 849`) with no shared chokepoint — but all 3 current paths already have pinning
  tests (`tests/export_volume_match.rs`, `tests/album_render.rs:206`), so there's no
  live bug. Only relevant if a 4th render/measurement entry point is ever added — route
  it through a shared `export_settings()` helper then, not now.
- **`compute_spectral_balance` hardcodes 44.1kHz-tuned RC coefficients** (`analysis.rs:311-313`)
  — explicitly documented elsewhere (`deep_analysis.rs:446`) as "retained for Phase-A
  diagnostics... historical tests only"; Phase-B's real guardrails use the sample-rate-aware
  FFT path instead. No render-affecting decision depends on this. Backlog only.
- **Limiter's Lagrange ISP scan misses the newest sample pair at the buffer head**
  (`dsp.rs:1572-1587`) — self-heals one frame later in steady state; only matters for
  the last ~1-2 frames at end-of-track. Low priority, already an "accepted
  approximation" in spirit.

---

## Speculative (11) — minority skeptic support only, not confirmed, **not actionable**

Listed for awareness. Each got 1-of-3 "real" votes, below the ≥2/3 bar this audit
requires to count as confirmed.

- Per-track shadowed `bit_depth` metadata can disagree with the actual album WAV bit
  depth — metadata drift only, nothing currently reads it.
- `useNavigationMachine`'s side-effect dispatch (leaving Album mode) has only
  reducer-level tests, no hook-level test.
- `api.setConfidenceGating`/`confidenceGatingEnabled` have no frontend caller (devtools/
  calibration-only by design, possibly).
- Intensity knob in the 2-up layout resized via CSS `transform: scale()` instead of a
  real size variant (CSS, see Hold section).
- `FirstRunOverlay` duplicates the `300px` rail-width magic number a second time (CSS).
- Landing nav still labels "Mobile" as a bare peer link to Standard/Advanced despite the
  section now being intentionally roadmap-framed — copy nit, not a bug.
- A near-duplicate framing of the verify:landing dead-selector bug (already confirmed
  above via 3 independent passes; this 4th attempt used different wording and landed at
  1/3).
- **Three parallel release.yml matrix jobs race to create/attach to the same draft
  release with no `concurrency:` guard** — plausible (no `concurrency:` block exists),
  but unconfirmed; worth a glance given how much else in this audit converged on
  `release.yml`.
- **macOS signing-identity extraction can silently no-op into an unsigned build** even
  with secrets configured (`release.yml:95` — `grep "Developer ID Application"` with no
  `set -e`/empty-result guard) — plausible, unconfirmed.
- `AVAudioEngineOutput.start()` (iPhone) has no reentrancy guard of its own, relying
  entirely on its single caller's guard — plausible, unconfirmed, low real-world risk
  given the current single call site.
- A second, more detailed restatement of EULA §2/§3's not-yet-true claims (already
  confirmed above; this is a duplicate-with-different-emphasis that landed at 1/3).

---

## Suggested order

1. **Live-audio allocation fix** (Fix-now #3) — small, additive, swaps to an
   already-built/tested safe path. Run the realtime sweep after.
2. **Release-pipeline test/version gate** (Fix-now #1) — additive CI job, protects every
   future release including the first signed one.
3. **`verify:landing` rewrite** (Fix-now #2) — restores a currently-useless safety net
   before the next landing-page change ships blind.
4. **Legal-doc draft-status caveat** (Fix-now #4) — a few sentences, removes a real
   misrepresentation risk before any build is distributed.
5. Everything else in Fix-now — all small, independent, no particular ordering
   dependency; batch as mechanical commits the way past audits in this repo have.
6. CSS batch — hold until a session with a working browser preview is available, per
   established precedent.

**47 confirmed, 0 critical, 5 high, 15 medium, 27 low. No DSP voicing issues. No
secrets, no export-overwrite hole, no data-safety violations found.**
