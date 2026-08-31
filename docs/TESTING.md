# Testing

## Fast Lane

This section is the desktop fast lane. `npm run verify:fast` is the umbrella
all-lanes runner: frontend, desktop Rust, iPhone bridge, and Android bridge.
Use `verify:frontend`, `verify:rust`, `verify:iphone`, or `verify:android` to
run one lane directly.

Run from repo root:

```powershell
npm test
npm run build
npm run build:windows
```

Run from `src-tauri`:

```powershell
cargo fmt --check
cargo clippy --target-dir target\codex-rc --all-targets -- -D warnings
cargo test --lib --target-dir target\codex-rc
cargo test --target-dir target\codex-rc
```

Use this lane for normal UI, state, packaging-script, and backend contract work.
The explicit `target\codex-rc` directory avoids collisions with a running debug
app that may lock the default target executable on Windows.

## Landing Marketing Proof (U7)

```powershell
npm run verify:landing-assets   # the gate — offline, no browser, milliseconds
npm run capture:landing         # recapture after changing a captured surface
npm run optimize:landing-art    # re-derive hero/icon variants from the master art
```

A screenshot on a marketing page has no expiry. The two desktop UI images
shipped from a 2026-06-28 capture and then U9, U10 and U11 changed the console
underneath them. Nothing failed, because nothing could.

`src/assets/landing/manifest.json` binds every capture to a **capture-input
digest**: a content hash over the explicit file list in
`scripts/lib/landing-assets.mjs` (`CAPTURE_INPUTS`) — the app shell, every
component, `preview-mock.ts`, and the capture script itself. Edit any of them
without recapturing and the gate fails.

**It is a content digest, not `HEAD`.** HEAD advances for reasons that cannot
affect a screenshot — docs, Rust, CI config — and a gate that cries wolf on
every commit is a gate people mute. An unrelated commit leaves valid proof
valid; this is verified by a negative control.

The gate catches four failures: **stale** (inputs changed), **missing**
(manifest promises a file that is absent), **altered** (bytes differ from the
capture), and **heavy** (eager imagery over 1.5 MB). It also fails if any
mobile UI image is imported by the landing page (R7).

**Eager budget.** Every capture is below the fold and `loading="lazy"` with a
declared intrinsic size; the hero art ships as a 1280w/2560w `srcSet` derived
from the 2026-08-18 4K console render. Eager imagery stays well under the
1.5 MB budget. Do not raise the budget — shrink the art. The pristine hero
master lives at `hero-control-room-studio-source.jpg`, is imported by nothing,
and reaches no bundle; the shipped variants are always re-derived from it so
repeated runs cannot stack generation loss.

## Visual EQ drag torture (manual, browser)

```powershell
node scripts/verify-eq-drag.mjs test-output   # needs the dev server on :5199 (or --url)
```

Pushes all seven EQ nodes to their frequency floors/ceilings and ±12 dB in the
real browser build and asserts clamping (via the live readout), band ordering
at the extremes, double-click reset, the panel reset button, one-step undo,
user-preset round trip, Standard↔Advanced round trip, album-mode drag, and a
clean console. Not a gate — run it after touching `VisualEqPanel.tsx` or the
EQ setters in `useTrackMaster.ts`.

## Headless Web Lane (landing + `/app`)

The default repeatable browser check for anything that renders. One command,
no prerequisites, no manually started server:

```powershell
npm run verify:headless
```

It builds once, starts its own preview server on a free port, runs the landing
responsive suite and the `/app` scenario suite against it, and tears the server
down on every exit path. Evidence (screenshots + `summary.json`) lands under
`test-output/headless/<timestamp>/`.

**Browser runtime.** The lane uses Playwright's **bundled Chromium**, pinned by
the `playwright` entry in `package-lock.json` — deliberately not the installed
Chrome channel, whose version is whatever Chrome last auto-updated to and which
does not exist on a CI runner at all. Install it once per machine:

```powershell
npx playwright install --with-deps chromium
```

**A missing browser fails the lane.** This is a gate; a gate that skips itself
when a dependency is absent reports green and is worse than nothing.

## RustSec Gate (audit S-01/S-02)

CI runs `cargo audit --deny unsound --ignore RUSTSEC-2024-0429 --file <lock>`
over all three lockfiles (desktop, iPhone, Android). `--deny unsound` fails
closed: a new unsoundness advisory turns CI red instead of warning into the
void (that blind spot let RUSTSEC-2026-0190 in `anyhow` sit unnoticed). The
single exception is **exactly** `RUSTSEC-2024-0429` — the Linux-only
GTK3/`glib 0.18` path Tauri pulls in — because Linux is deferred for this
beta; remove the `--ignore` when upstream leaves `glib 0.18`. Never widen the
exception to a category or suppress all warnings. Locally:

```powershell
cargo audit --deny unsound --ignore RUSTSEC-2024-0429 --file src-tauri/Cargo.lock
cargo audit --deny unsound --ignore RUSTSEC-2024-0429 --file apps/iphone-native/rust/Cargo.lock
cargo audit --deny unsound --ignore RUSTSEC-2024-0429 --file apps/android-native/rust/Cargo.lock
```

The static half of the gate (`release-readiness.test.ts`) pins all three CI
commands to that exact shape.

**Accessibility scans (audit A-02).** Axe-gated scenarios run the committed
`axe-core` build (pinned by the lockfile, injected from `node_modules`, never a
CDN) against WCAG 2.0/2.1 A + AA tags on the live settled state. Any violation
fails the lane; `incomplete` results are persisted in `summary.json` as review
evidence, not passes. An `axeCoverage` ledger requires each expected
state/viewport pair to be scanned exactly once with a positive rule total, so a
silently skipped or empty scan is a failure. Do not suppress a rule to green
the lane — a real violation gets its own focused fix commit.

**Landing quality gates (added U8).** The matrix is now **13 viewports** —
`320×568` was added because S-A3 names it and nothing covered it. At every
viewport the lane also asserts: both acquisition CTAs are fully on screen, no
in-page link points at a missing section, every external link is https and
carries `rel` on `_blank`, exactly one `h1` with no skipped heading levels, a
`<main>` landmark, a skip link targeting `#main`, title/description/canonical
sanity, and — **at phone widths only** — touch targets of at least 24px.

The touch rule is width-scoped on purpose: on a desktop width the input is a
mouse, and failing a 20px nav link there is noise. Links sitting *inline inside
a sentence* are exempt (WCAG 2.5.8), because their size comes from the prose
and padding them wrecks the paragraph.

**Accessibility scan.** axe-core runs at `1440×900` and `390×844` against
`wcag2a/wcag2aa/wcag21a/wcag21aa`. Best-practice rules are excluded — they are
opinions, and a gate that fails on an opinion gets disabled. **Serious and
critical violations fail the lane**; anything lower is recorded in
`summary.json` for review. Two widths, not twelve: axe findings are
overwhelmingly width-independent, so more runs buy repetition, not coverage.

**Keyboard path.** The first Tab stop must be the skip link, it must become
visible when focused and show a focus indicator, and the acquisition CTA must
be reachable within 40 stops without focus falling to `<body>`.

**200% zoom.** WCAG 1.4.4 is usually discharged as a "recorded manual check"
that nobody repeats. The lane zooms to 200%, re-measures overflow, and confirms
the CTA is still on screen.

**Reduced motion.** Every load-bearing sentence must be present with
`prefers-reduced-motion: reduce`. Content that only exists after an animation
is content some visitors never get.

**Indexability follows release state.** While no verified release exists the
page must carry `noindex` — indexing a page whose download is closed sends
search traffic somewhere it cannot be served. The check fails in *both*
directions, so U17 cannot forget to remove it and nobody can remove it early.

**Failed requests** fail the lane, except `ERR_ABORTED`: that is the browser
cancelling a superseded `srcset` candidate on a viewport change, which the
responsive hero produces by design.

**Production smoke (`npm run verify:landing-smoke -- <url>`).** Adds real
network checks that every outbound link resolves. Off by default because a
hermetic CI lane should not depend on third-party availability; U17 runs it
against the deployed URL.

**Landing release state (added U5).** At every viewport the landing suite
asserts that no anchor on the page links to `/releases/latest`, that the
`[data-release-state]` host exists, and — while no release is verified — that
zero download actions are rendered, an inactive action is present, and its
reason is *visibly painted* (non-zero box, not `display:none`/`hidden`/
transparent) and associated via `aria-describedby`. It also fails if the
visitor-facing reason contains `draft`, `unverified`, or `candidate`: the
five-state release model is internal, and leaking it is a build-detail
violation of `docs/landing-brief.md`.

This lane observes **S-A1** and **S-I1** on the real page. It cannot observe
**S-A2** (needs a verified release) or **S-B1** (needs a real draft); those are
injected-state cases in `src/landing/BetaDownload.test.tsx` and
`src/lib/release-readiness.test.ts`, and `summary.json` carries a
`scenarioCoverage` map that says so explicitly — so a green run is never
mistaken for coverage it does not have.

**`/app` preview scenarios.** Deterministic states selected with
`?scenario=<name>` (see `PREVIEW_SCENARIOS` in `src/lib/preview-mock.ts`):
`clean`, `empty`, `warning`, `long-copy`, `export-success`, `export-cancel`,
`album-1`, `album-4`, `album-12`, `album-long`, `album-warning`. `?empty=1`
still works as an alias for `empty`. Every scenario is checked at 1440×900 and,
where layout matters, at the supported minimum 1360×740.

**Export-receipt shell geometry (audit U-02).** The `warning` scenario's
reduced-motion pass measures the OPEN receipt after two settled animation
frames, before any reachability scrolling: the card must fit the viewport,
Done / Show file / × must be initially visible and topmost at their centres,
the close target must be ≥24×24 px, and scrolling `.receipt-scroll-region`
(the only scrolling part of the card) must not move the pinned action/footer
rows. The structural sibling-order half of this contract is pinned in
`src/components/ExportReceiptCard.test.tsx`.

**Console policy.** Any console error or warning fails the lane. The preview
mock warns on an unhandled command or listen channel, so a drifted mock contract
turns the lane red instead of producing noise nobody reads. Deliberately
unsupported native-only behavior (`install_update`) is logged at *info* level
with a named prefix and allowlisted in `scripts/verify-app-headless.mjs`.

**Reachability (`mustReach`, added U10(c)).** A separate question from both
"does the text exist" (`textContent`) and "does the page overflow sideways". A
control can exist, in the DOM, at the bottom of a scroll container, underneath
a sticky footer — present to every assertion and unreachable to every user. A
`mustReach` target is scrolled into view and then checked three ways:

1. it has a real layout box (non-zero width and height);
2. after scrolling, that box is inside the viewport;
3. it is the topmost element at its own centre — nothing is covering it.

Check 3 is the one worth having, and it is easy to write wrongly. Reachable
means the topmost element is the target **or a descendant** of it. Treating an
*ancestor* as acceptable ("it contains the button, so we must have hit it")
silently passes a `::after { inset: 0 }` sheet laid over a sticky export group:
the pseudo-element is not a node, so `elementFromPoint` returns the element
that generated it — the button's own parent. The first draft of this check did
exactly that and reported green while Playwright's click on the same button
timed out.

Current targets: the Delivery Format card and the Export action (Export Master
/ Export Album) in `clean`, `long-copy`, and every album scenario, at both
1440×900 and 1360×740. Per-target geometry is recorded in `summary.json` under
`reachability`, so a pass is a measurement rather than an absence of
complaints. Failures name the coverer.

**Reduced-motion variants (`reducedMotionVariant`, added U11).** A scenario so
marked runs a second time with Playwright's `reducedMotion: "reduce"`, writing
a `<label>-reduced-motion-<viewport>.png` alongside the normal shot. **The
assertions are identical in both passes**, which is the point: reduced motion
is meant to change how a state arrives, never which state you arrive at, so a
scenario that only passes with motion enabled is a real failure rather than an
expected difference. Currently on `clean` (the quality-verdict badge),
`warning` (the review-gate overlay entrance), and `album-4` (the album
sequence arc).

**Scenario IDs.** Every scenario carries the plan's `scenarioId`
(S-D1/S-E1/S-F1/S-F2/S-F3), and `summary.json` ends with a `scenarioCoverage`
map from ID to the scenario/viewport checks that covered it — so "the scenario
set is closed" is answerable from a file instead of from memory.

**What this lane does NOT prove.** Native dialogs, real audio device behavior,
installer signing, updater installation, and anything about how the product
sounds. Those are the installed-machine and owner-listening layers. Browser
results are labelled `evidenceLayer: "browser-headless"` in every summary so
they cannot be mistaken for native proof.

To prove the lane still fails when it should:

```powershell
node scripts/verify-app-headless.mjs --url http://127.0.0.1:5177 --force-fail album-12
```

It must exit nonzero and name the scenario, route, viewport, and screenshot path.

## iPhone Native Bridge Lane

When changing shared Rust types, `yes_master_lib` behavior, adaptive/profile
resolution, or `#[tauri::command]` signatures that the phone bridge may depend
on, also run:

```powershell
cd apps/iphone-native/rust
cargo check --all-targets
cargo test
```

The desktop lanes do not compile this bridge. A shared struct or signature can
drift here while all desktop checks stay green.

## Android Native Bridge Lane

When changing shared Rust types, the iPhone facade crate, `yes_master_lib`
behavior, adaptive/profile resolution, or the Android bridge crate, also run:

```powershell
cd apps/android-native/rust
cargo test
cargo ndk -t arm64-v8a --platform 29 check
```

This lane needs the Android toolchain from `docs/ANDROID_NATIVE_SPEC.md`:
JDK 17 on `JAVA_HOME`, SDK/NDK r27.2 via `local.properties` or
`ANDROID_HOME`, Rust Android targets, and `cargo-ndk` 4.1.2 or newer.
`--platform 29` matches `minSdk`; cargo-ndk's default API 21 sysroot predates
`libaaudio`, so the audition link fails without it.

## Slow Fixture Lane

Private audio must live under:

```text
private-audio-fixtures/
```

Run from `src-tauri`:

```powershell
$env:AMS_RUN_REAL_FIXTURE = "1"
cargo test --target-dir target\codex-rc
Remove-Item Env:\AMS_RUN_REAL_FIXTURE
```

Use this lane before merging changes to:

- DSP chain behavior.
- Compressor mode.
- Limiter/ceiling behavior.
- LUFS landing.
- Export checks.
- WAV writing.
- Source/master parity.
- Decode/playback paths that affect audition trust.

## Manual Listening Gate

Automated tests cannot approve taste.

Before calling Track Master private-solid, manually verify with audio playing:

- Intensity sweeps.
- EQ/tone sweeps.
- Output gain sweeps.
- Compressor threshold/density sweeps.
- Preview LUFS off and on.
- Original/Mastered switching.
- Volume Match off and on.
- Export and open output.

## Evidence-runner output paths (both private runners)

The two private runners below take `--output`. A few rules, pinned by
`src-tauri/tests/evidence_runner_paths.rs`:

- **Relative output is expected and supported.** `--output ../test-output/...`
  from `src-tauri` is the documented form; the runner resolves it to an
  absolute path before deriving any render destination. (Before 2026-07-24 the
  matrix runner did not, and the documented command failed outright with
  `path traversal not allowed` — the raw `..` reached the render layer's
  traversal guard.)
- **Forward slashes, on every platform.** Windows accepts `/`, macOS does not
  accept `\`. A backslash path on macOS is one filename containing backslashes,
  not a path — so the commands below use `/` and work on both.
- **The output may not be inside the private source directory** (the manifest's
  folder, or the reference folder). Rendered masters do not belong next to
  private source audio. The runner rejects it *before* creating anything, and
  re-checks after creating in case a symlink or junction redirected it.
- Reports and renders land under the requested directory only, and every
  returned path is absolute so the evidence is unambiguous.

## Already-Mastered Regression Matrix

Use the local-only runner documented in `docs/PRIVATE_AUDIO_FIXTURES.md`:

```powershell
cd src-tauri
cargo run --example private_fixture_matrix -- --manifest ../private-audio-fixtures/manifest.json --output ../test-output/private-fixture-matrix
```

To validate the Phase B confidence gate in the same headless lane:

```powershell
cd src-tauri
$env:YES_MASTER_CONFIDENCE_GATING = "1"
cargo run --example private_fixture_matrix -- --manifest ../private-audio-fixtures/manifest.json --output ../test-output/private-fixture-matrix-confidence
Remove-Item Env:\YES_MASTER_CONFIDENCE_GATING
```

If the full private manifest is too slow for an interactive run, create a
local-only subset manifest under ignored private fixture storage and record that
the evidence is representative rather than complete.

Required coverage:

| Case | Preset | Compressor | Expected Evidence |
| --- | --- | --- | --- |
| Already-processed source | Universal | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Universal | Off | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Loud | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Loud | Off | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Clarity | Preset | Source/render LUFS, TP, LRA, warning codes |
| Already-processed source | Clarity | Off | Source/render LUFS, TP, LRA, warning codes |

The goal is not to prevent bold masters. The goal is to catch cases where the
app makes the output objectively flatter/hotter and then fails to make the user
review that fact.

## Private Reference Tuning

Use the local-only reference runner after preset/DSP retunes:

```powershell
cd src-tauri
cargo run --example private_reference_tuning -- --references "../tests for presets" --output ../test-output/private-reference-tuning
```

To compare reference gaps with Phase B confidence gating enabled:

```powershell
cd src-tauri
$env:YES_MASTER_CONFIDENCE_GATING = "1"
cargo run --example private_reference_tuning -- --references "../tests for presets" --output ../test-output/private-reference-tuning-confidence
Remove-Item Env:\YES_MASTER_CONFIDENCE_GATING
```

Use the ledger as evidence, but do not treat it as a listening substitute. The
runner output and rendered WAVs are private/ignored and must not be committed.

## Preset Fingerprint Harness

The committable, synthetic-fixture counterpart to the private runners:
`src-tauri/tests/preset_fingerprint.rs` renders every factory preset against
deterministic in-test fixtures (pink bed, drum loop, tonal pad, 1 kHz probe)
and gates three properties in CI:

- **safety bounds** — band-tilt/crest/LUFS/width/THD caps at default intensity;
- **distinctness** — a minimum character distance across every preset pair;
- **tolerance golden** — the full fingerprint table pinned OS-independently
  (`tests/golden/preset_fingerprint.json`).

After a deliberate, owner-approved retune:

```powershell
cd src-tauri
$env:YES_MASTER_UPDATE_GOLDEN = "1"
cargo test --test preset_fingerprint
Remove-Item Env:\YES_MASTER_UPDATE_GOLDEN
# then commit the JSON diff and queue the listening note
```

To arm a listening sitting with the owner-readable report
(`test-output/preset-fingerprints/`, git-ignored):

```powershell
cd src-tauri
cargo test --test preset_fingerprint write_owner_fingerprint_report -- --ignored
```

## Tooling Notes

- Clippy is part of the hard local gate. If it is missing on a fresh toolchain,
  install it with `rustup component add clippy`.
- Windows packaging should produce MSI and NSIS artifacts under ignored
  `src-tauri/target/release/bundle/` and should not leave
  `src-tauri/target/release/produce_dialog_smoke.exe` registered as an app
  binary.
