# YES Master — Ideas Backlog

A deduped catalog of **forward-looking ideas and features** surfaced from across
the repo (active docs + the archived handoffs/reviews/plans), grouped by theme.

**This is a resurfacing list, not a commitment list.** Many of these were
proposed, half-built, or noted in passing and then lost as docs aged out. Nothing
here is scheduled; it exists so you can scan for plans you forgot to start. Items
that are owner-gated or already-partially-built are noted. Live open *threads*
(things genuinely in progress / decisions owed) live in
**docs/OPEN_THREADS_AND_DECISIONS.md**; this file is the wishlist.

Generated 2026-06-22 from the docs-hygiene recon.

---

## Adaptive engine — Tier-2 "smart" / corrective mastering

- Corrective **target-curve** mastering (LANDR / iZotope / Match-EQ style) that
  pushes toward a target curve — vs the defensive reduce-only Tier-1 that shipped.
- **Measured-neutral reference** built from your own reference masters, ideally
  **per-preset**, replacing textbook pink (makes the deadband number ~irrelevant).
- **Tilt-vs-reference brightness** metric (spectral-slope-vs-pink) so a flat
  spectrum reads as zero excess by construction — the "principled" fix; never built.
- **Genre/style classification** feeding adaptation.
- **Reference-track upload / matching.**
- **Resonance detection + dynamic notching** (Soothe / Gullfoss style); active
  sibilance notch; realtime per-frequency dynamic EQ.
- **Per-band stereo-width matching** (Ozone Width Match style).
- **PSR/crest closed-loop** dynamics defense (transient protection) — inputs
  (per-window sample_peak + loudness_key) are retained, logic never built.
- Holistic **already-mastered stand-down** keyed on per-stratum peak + integrated
  LUFS + crest.
- **Total-loudness-loss budget** (B3) shared across the four adaptive axes so
  multi-axis trims can't exceed a per-axis cap after LUFS landing.
- Higher-resolution / time-varying analysis (finer than 6 bands); 6→31-band
  rollup with per-band tolerance test + album label-stability fixture.
- Wire 31-band harsh/sibilant/tilt features into the guardrails (harsh-only /
  sibilant-only / airy each master differently).
- Welch-style averaged spectral windowing across begin/middle/end instead of a
  single first-window FFT.

## Adaptive engine — Tier-1 finish / controls

- **Per-dimension Adapt Strength** (separate dials per axis: EQ touchier than
  compression / brightness / low / density / width) instead of one global dial.
- Use **stereo_width as a width co-trigger** alongside correlation (currently
  computed/carried but unread) — or remove the inert IPC field.
- Split **density** from a single scalar into sub-knobs (re-anchor DENSITY_CAP);
  reshape density-cap against the engagement curve so "keep N%" is honest in dB.
- **Auto Compressor-Off at extreme density** (v1 only caps density reduction at 60%).
- Per-axis **EQ floors / LOW_DEADBAND** ear-calibration for bass-forward genres.
- transient_density softening of bright/boomy triggers.
- **Spectral slope/tilt** (dB-per-octave) co-trigger alongside share-based tonal detection.
- side/mid co-trigger for the width guardrail instead of correlation-only.
- Represent unknown LRA as `Option<f32>` rather than a 0.0 sentinel.
- Backend-owned/stored per-track analysis profile attached at every chain build
  (dissolves per-call-site injection drift; became the B2 refactor).
- **Album Master per-track adaptive** injection (currently intentionally flat).
- Local calibration measuring our-band neutral shares across your reference
  masters to replace inferred provisional defaults.

## Compressor (adaptive Tier-3)

- Per-band compressor **Advanced card** with per-band "Adaptive" tags +
  plain-language guidance built from machine-readable reasons.
- `resolve_compression_plan(track_id)` backend command so the frontend never
  re-implements the per-band mapping.
- **Export-receipt guard digest** ("density eased 22% low / 8% mid, stand-down
  0.4, confidence 0.9") for render traceability.
- Pre-launch optional **"suggestion layer"** (per-band card recommends density/
  mode, one-click apply, zero render-path change) — distinct from full Tier-3.
- Mode-pill relabel from "Preset" to **"Adaptive"** in calibrated UI copy (owner call).
- A real **owner-facing adaptive-compression toggle** (today only env seed + dev command).
- Runtime **OFF→ON→OFF same-playhead A/B** toggle (`api.setAdaptiveCompression`)
  as the listening primitive over env-seed relaunch.
- A **dev-only runtime flag** (vs compile-time const) for confidence gating so you
  can A/B without source edits (partly realized by `YES_MASTER_CONFIDENCE_GATING`).
- Compact per-axis **confidence digest** in the export receipt once Phase B is on.

## Receipt / metrics / traceability

- Add **effective_adaptive_strength + profile digest** to the export receipt so
  delivered masters are traceable as adapted.
- Measure **post-render stereo correlation** (currently no field/consumer) so the
  receipt shows delivered correlation, not just source.
- Surface the already-stored **delivered LRA** (`RenderedMeasurements.dynamic_range_lu`).
- Seed the dither RNG **continuously across segmented album writes** (per-call reset
  repeats the noise pattern across segments).

## Analysis / DSP internals

- Tier-2 Phase C **"premium" staged-loading UI** whose progress maps to REAL
  analysis phases (decode → long pass → short pass → assemble) + a plain-English
  **AnalysisSummary** on the wire (flags + summary, never raw per-window series).
- Replace **staged wall-clock progress theater** (sleeps, parks at 94%) with honest
  indeterminate/real progress events.
- Explicit **denormal flush (FTZ/DAZ)** around the IIR filters.
- Centered textbook **soft-knee quadratic** instead of zeroing the lower knee half (taste).
- **Per-window TRUE peak** via a 4×-oversampled pass (Phase A only does mono-downmix sample_peak).
- **Persist DeepAnalysis to disk** (Serialize/Deserialize + content/version key +
  invalidation) for faster reopen on multi-track albums.
- **Stereo-aware (channel-summed BS.1770)** loudness key for width-confidence coverage.
- Route desktop **live Preview LUFS** through the shared rendered-rate landing
  helper so audition matches export on non-44.1k sources.
- Hoist a **single reusable mono scratch buffer** in deep-analysis scan_windows
  (cut ~262 MiB worst-case allocation churn).
- **yes-dsp crate extraction** (decouple `MasteringSettings.album` from album types).
- A real **lite-adaptive phone path** (new FFI/Swift wiring) vs running full
  DeepAnalysis on mobile.

## Contracts / bindings / drift

- **tauri-specta** adoption to auto-generate TS bindings from Rust types (replaces
  hand-written `bindings.ts`) — currently parked behind the hand-written drift gate.
- Repo-local **Rust→TS wire-contract drift gate** (dependency-free, 90% of the value).
- **Single supported-audio-format contract** shared across desktop UI, dialog
  filter, fixture scanning, and native bridges (cross-surface parity test).
- **Codegen the TS compressor-calibration constants** from the Rust table (kill the
  dual-table drift).
- Frontend **preset-display contract module** with parity tests for compressor /
  SignalChain-width / standard-mapping mirrors.
- Shared evidence-lane adaptive-settings resolver behind one Rust helper for
  `fixture_matrix` + `reference_tuning`, with a cross-lane equivalence test.

## Tooling / tests / CI

- Add **knip / ts-prune** to CI to catch dead exports/unused object keys.
- **CSS selector-inventory test** that fails on unmatched selectors before deleting
  dead blocks; full dead-CSS selector inventory + deletion.
- **Asset-budget test** capping per-preset and total preset-artwork bytes.
- **Launch-size + minimum-resize** visual reachability smoke check (Delivery Format
  + Export stay scroll-reachable with Advanced Controls expanded).
- **macOS CI job** that continuously proves the Mac build (convert the "Mac build
  never verified" PKG-04 gap); Android Rust host tests + `cargo ndk` arm64 check.
- **Render-level multi-axis composition tests** under active delivery LUFS landing.
- **iPhone render-path parity golden tests** vs the desktop shared Rust render.
- **Preset-fingerprint measurement harness** (per-band tonal delta, dynamics, width,
  saturation at intensity 0.5/1.0) that **gates retunes via mechanical tests**;
  define numeric per-preset target fingerprints (LF weight for Oomph, HF air for
  Clarity). *(High value — makes future preset tuning objective.)*
- Git-archaeology diff of preset tonal tables across history to test "presets had
  more character previously."
- Separate **external-master benchmark lane** (manifest-driven, multi-brand:
  BandLab / LANDR / eMastered / CloudBounce / Ozone) distinct from the 4-preset
  private runner.
- `buildProjectState()` helper to de-dup the byte-identical autosave vs save-as literal.
- Extract shared `src/lib/time-format.ts` with explicit null/non-finite policy.
- Replace substring-sniffing iPhone error display with an explicit error enum + view test.

## Portability / distribution (blind-spot review 2026-07-04)

- **Relink missing sources**: sessions store absolute paths, so machine hops /
  moved folders orphan every track. A per-track "locate file..." affordance
  (plus optional relative-to-session-file path storage) would make `.ams.json`
  genuinely portable.
- **Cloud-placeholder awareness**: detect `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`
  on import and warn ("this file will download from OneDrive before analysis")
  instead of stalling silently; pair with a decode/analysis watchdog timeout.
- **Auto-updater** (Tauri updater plugin + signed release feed): every beta
  install is currently stranded on its version forever. Blocked on CI/release
  budget, not on code.

## Refactor / architecture

- Explicit **Standard/Advanced/Album navigation state machine** making illegal UI
  states unrepresentable (consolidate App.tsx view-mode + project-mode).
- Owner-gated **engine-adjacent cleanup** slice (process_sample,
  analyze_tracks_core_lite, dsp one-pole/soft-knee hoist) activatable by one owner line.
- Regenerate **graphify cache with repo-relative paths** (or untrack) for
  cross-machine cache-warming. *(Relevant now — the archive move staled the cache.)*

## UX / product surfaces

- **Flagship Standard View visual-polish pass** (hero treatment, motion, micro-
  interactions vs `docs/simple-mode-mockup.html`) — flagged as the single biggest
  remaining Standard View item.
- **Simple Mode view**: stripped-down desktop UI riding on the adaptive engine.
- **First-run coachmarks** replacing embedded first-run rail UI.
- **Original↔Mastered fast-toggle crossfade** to remove click/lag.
- **Recent projects MRU / Open Recent** with prune-on-open; native app menu (macOS);
  file-type association + single-instance; export-finished notification when unfocused.
- **Cancelable long operations** via per-job AtomicBool cancellation tokens.
- **Close-guard confirm dialog** + post-export system notification when unfocused.
- **Missing-source relink flow** with per-track Locate…/Remove recovery.
- **Friendly-error mapping layer** translating backend error shapes to action copy.
- **Min-window-size** support for 1366×768 laptops.
- **30-minute import duration cap** with a single-constant threshold + clear error
  (shared with iPhone via the facade).
- **Decouple Loudness Low/Med/High from the export profile** (how-loud vs where-
  it's-going), making presets tone/character only.
- **Friendly Simple-facing preset names** mapping to internal ids; LANDR-style
  visible "analyzing / fitting to your track" step; remember default-mode per user.
- **Retire the Export-With-Review confirm-gate** while keeping advisory warnings +
  technical hard-stops.
- Surface the fully-built **LEVELS + STEREO WIDTH panels** into the right-rail gap
  instead of deleting them.
- Capture **S5.4 canon answers** (mobile audience/exclusions, what "Album Master"
  promises, honest "adaptive" wording) into PRODUCT.md.
- Fold **READING_THE_METERS.md into in-app Help/glossary** (single source of truth).
- AlbumPanel above-stereo fold-down advisory copy + receipt test.
- Persist **Album-panel choices** (arc/title/intensity/sample-rate/bit-depth) in
  the project schema with backwards-compatible old-`.ams.json` handling.
- Return **structured partial-analysis failures** so import/open-project shows calm
  per-track recovery feedback.
- Test a **"Clarity + Punch" hypothesis** for LANDR Open's identity; carry a distinct
  Warmth voicing separate from Tape's glue/density.

## Mobile-specific

- ARM **FPCR flush-to-zero** on the render thread + preallocated coeff double-buffer
  (ping-pong) to remove in-callback Vec clones.
- **File-descriptor passthrough for SAF import** (avoid doubling disk on huge files);
  16 KB ELF-alignment post-build tripwire; single-source the ABI list; x86_64 ABI
  for emulator/Chromebook.
- **Background render story**: foreground service + MediaSession vs documented v1
  limitation; done-screen Share/play intents (parity with iPhone ShareLink);
  monochrome adaptive-icon layer for Android 13.
- Lower **minSdk below 29** for broader device coverage; audition buffer-size knob;
  Play Store packaging with ABI splits.
- iPhone v1 deliberately omits LUFS Preview, export-profile picker, adaptive/
  confidence-gate UI — candidate surfaces for an **iPhone v2 "expert phone surface."**

## Visual polish (cosmetic)

- **Perfectly-round Visual EQ nodes** (SVG `preserveAspectRatio='none'` renders
  slight ellipses; needs plot-scaling rework).
- Standard View **hero track switcher** polish (replace the pragmatic `<select>`).

## Security / packaging

- **Restrictive Tauri CSP** to replace `csp:null` before any remote/user-HTML loading.

## Public surface

- Define the **marketing landing page's product role** (marketing-only vs download/
  onboarding) and document it in canon.
