# YES Master

**Master your track in real time — and see exactly what it did.** Drop in a
finished track or album, hear the full mastering chain as you listen, shape it to
taste, and export a technically-checked master. It runs entirely on your machine,
so every change is instant — no upload, no waiting, no account.

No black box. YES Master *measures what it did and shows you* — live meters,
quality checks, and a post-render receipt — so you can push as hard as you want
and still know the truth about the file you're shipping.

Desktop (Windows + macOS) is the product: a Tauri 2 + React/TypeScript frontend
over a Rust DSP engine. iPhone and Android companions in `apps/` reuse that same
engine through native bridges, with output bit-parity-pinned to desktop.

License: source-available, proprietary — see `LICENSE`.

## What it is today

**Two views, one engine:**

- **Standard** (default): pick a Style (Universal / Clarity / Tape / Oomph) and
  a loudness (−14 / −11 / −9 LUFS), audition Original vs Mastered at the same
  playhead, Create Master. Exports a fixed known-safe format: 44.1 kHz /
  24-bit WAV at a −1 dBTP ceiling. No blocking review ceremony; cosmetic
  warnings are suppressed, genuine integrity problems are never hidden.
- **Advanced**: the full surface — eight presets, intensity, 7-band visual
  EQ/tone, width/warmth, explicit compressor modes (Preset / Manual / Off,
  with per-band detail), delivery profiles and formats (44.1/48/96 kHz,
  16/24-bit), live metering (peak, LUFS, gain reduction, spectrum), and a
  warning-aware export review with a post-render receipt (delivered LUFS,
  true peak, dynamic range, check results).

**The engine** (Rust, shared by all platforms):

- **Real-time audition** — the full chain (subsonic filter → EQ → multiband
  compression → width/warmth → lookahead limiter → LUFS landing) runs live
  during playback; control changes apply *while the audio is playing*.
- **Analysis** — decode → dynamics → stereo field → tonal balance → deep
  per-window scan (crest, momentary loudness, 31-band detail), with real
  progress events to the UI.
- **Adaptive mastering (Tier 1, shipped + owner-listened)** — each track
  resolves a source profile that drives *reduce-only* guardrails: trim preset
  brightness/low boosts, scale compression density, weighted by per-axis
  confidence and an Adapt Strength control. Presets stay recognizable by
  construction (per-axis trim caps) — it tames, never overcooks.
- **Safety, enforced by tests** — exports never overwrite source files or prior
  renders; Volume Match is audition-only and can never change export level;
  warnings are advisory unless the output is technically invalid; quality
  checks measure the *rendered file*, not assumptions.

**Album Master** (Advanced): album-wide intent with per-track overrides,
album delivery format with mixed-rate resampling, continuous + per-track
renders with a manifest.

**Projects:** `.ams.json` save/open, autosaved recent session with restore,
waveform/peaks rebuilt from referenced source files (audio is referenced from
disk, never embedded).

## Where it's heading

The active plan is a staged shippability push, executed by coding agents
against hyperdetailed specs, with CI (Windows + macOS + Android lanes on
every push) as the verification floor and owner listening sessions as the
only human gates:

1. **Correctness first** (done): audio-corruption and overwrite-protection
   fixes, frontend state races, cross-platform DSP snapshot verification on
   real macOS hardware.
2. **Contract hardening** (in progress): a single supported-format contract
   wired from UI copy to decoder; identity-carrying progress events; a
   cross-language parity fixture pinning the Standard export recipe across
   TypeScript, desktop Rust, iPhone, and Android so no platform can drift
   silently.
3. **Adaptive compressor (MVP feature work)**: per-band track-aware
   compression with transient protection (PSR-based) and already-mastered
   stand-down — built strictly reduce-only so its output space is bounded by
   the already-validated preset sound, shipped behind a runtime gate, and
   enabled only after a by-ear A/B calibration session. Spec:
   `docs/plans/2026-06-12-adaptive-compressor-mvp-spec.md`.
4. **Public desktop release** (desktop-first): a free public beta (Mac +
   Windows together, v0.9.0) with signed installers on GitHub Releases and a
   Tauri updater, then a paid 1.0 ($29 founder → $49). Code signing,
   notarization, and autoupdate are now launch-blocking, not deferred. See
   `docs/plans/2026-07-07-beta-execution-plan.md`.
5. **Mobile follow-on**: the iPhone and Android apps mirror Standard's fixed
   export and already share the engine; store-readiness (privacy manifest,
   signing, lifecycle hardening) is planned but sequenced after desktop. Plans:
   `docs/plans/2026-06-12-iphone-shippability-plan.md`,
   `docs/plans/2026-06-12-android-shippability-plan.md`.

Overall scope, stated plainly: a mastering tool a musician can trust on real
releases — competitive with cloud mastering services on adaptiveness, but
local, inspectable, and honest about what it measured and what it changed.
Not in scope: DAW features, recording/editing, cloud processing, or a
certified-engineer replacement.

The full findings registry behind the plan is
`docs/reviews/2026-06-12-master-shippability-audit.md`; the slice-by-slice
execution spec is `docs/plans/2026-06-12-shippability-roadmap.md`.

## Build from source

Prerequisites: Node 22+, Rust (stable), and the Tauri 2 platform
prerequisites for your OS.

```powershell
npm install
npm run tauri dev      # development app
npm run build:windows  # Windows MSI + NSIS bundles
npm run build:mac      # macOS app + dmg (on a Mac)
```

## Verification lanes

Fast lane, from the repo root:

```powershell
npm test
npm run build
npm run build:windows
```

From `src-tauri`:

```powershell
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

Mobile bridge lanes (required when shared Rust types, `yes_master_lib`
behavior, adaptive/profile resolution, or `#[tauri::command]` signatures
change — desktop lanes do not compile the bridges, so drift is silent without
them):

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

```powershell
cd apps/android-native/rust
cargo test
cargo ndk -t arm64-v8a --platform 29 check
```

The Android cross-check needs the toolchain described in
`docs/ANDROID_NATIVE_SPEC.md` (JDK 17, SDK + NDK r27.2, cargo-ndk 4.1.2 or newer).

CI runs the Windows, macOS (including iPhone Swift tests), and Android JVM
lanes plus cross-platform DSP snapshot diagnostics on every push to `main`.

## Slow fixture lane

Only when local private fixtures exist under `private-audio-fixtures/`
(private audio is local-only and must never be committed):

```powershell
cd src-tauri
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

Required before merging work that touches DSP, render, LUFS landing, WAV
writing, export checks, or source/master parity.

## Documentation

Active canon (read in this order):

- `docs/PRODUCT.md` — product source of truth
- `docs/APP_BEHAVIOR.md` — what the app does today
- `docs/ARCHITECTURE.md`
- `docs/TESTING.md`
- `docs/RELEASE_STABILIZATION.md` — active gates and shipped slices

Active plans: `docs/plans/2026-06-12-*.md` (desktop / iPhone / Android
shippability, the adaptive-compressor spec, and the master roadmap).

Historical handoffs, old phase plans, and prior-session reviews under `docs/`
are working records, not active product spec. When prose and code disagree,
current code plus the canon docs above win.
