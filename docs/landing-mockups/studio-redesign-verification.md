# Studio marketing redesign — verification

Branch: `codex/studio-marketing-redesign`. Base: `c006f3e`.
Owner request: implement `YES_Master_Studio_Handoff` on the marketing website,
with Standard/Advanced laptop heroes, editable page copy, responsive layouts,
closed release/signup gates, and no deployment.

## Design comparison

Inspected the approved theme and all six reference boards. Implemented the
shared studio background and matching transparent laptops as separate images.
Used the handoff page copy and unchanged Standard/Advanced source captures,
plus the existing original style artwork and preserved Sign-off photograph.

Desktop retains the left-copy/right-laptop composition and cropped Advanced
reveal. Mobile stacks the full laptop displays below the copy; style cards
become a 2×2 grid. All screenshot crops enlarge to the same full original and
support keyboard closing/focus return. No image-model UI redraws were used.

The supplied laptop layers reconstruct the reference rather than reproducing
its flattened artwork exactly: screen perspective and keyboard framing differ.
The canonical handoff headline replaces the reference's noncanonical headline.
Report and album panels are labelled illustrations; the receipt contains no
invented numbers or success statuses. Album output labels describe actual
output types instead of promising example filenames. The closing preserves
the existing availability and optional signup notices below the reference CTA.
"Watch demo" is an unavailable placeholder with an on-page explanation.

Rendered and inspected at 1600, 1440, 1024, 768, 390 and 320 CSS pixels. Fixed
the Intensity crop's missing intrinsic dimensions (which prevented native lazy
loading) and changed cropped hero overflow to `clip` so focus/capture scrolling
cannot silently scroll the artwork and hide its headline inside its own band.
All six render widths have no horizontal overflow and all visible imagery loads.

Local evidence: `test-output/studio-redesign/renders/` contains hero, full-page,
Advanced hero and each section's images at all six widths. Baseline renders are
in `test-output/studio-redesign/baseline/before/`.

## Verification

- `npm test`: 82 files, 831 tests passed.
- `npm run build`: typecheck and production build passed.
- `npm run verify:landing-assets`: passed; eager artwork is 770 KB within the
  existing 1.5 MB budget; all supplied image bytes are manifest-bound.
- Landing browser lane: 13 viewports, no overflow, no axe violations at desktop
  and phone sizes, 200% zoom, keyboard acquisition, all anchors, closed release,
  unavailable demo, FAQ keyboard toggles, six screenshot dialogs per width.
- `/app` headless lane: 31 scenario/viewport checks passed.
- `npm run build:windows`: MSI and NSIS build checks; no installation performed.
- `cargo fmt --check`, warning-denying clippy, `cargo test --lib`, and full
  `cargo test`, using `target/codex-rc`: passed. Library: 448 passed, 2 ignored;
  integration suites passed with their existing ignored diagnostic tests.
- No DSP/export edits; the private-fixture lane and native listening were not
  run. No shared crate type/signature edits requiring mobile bridge work.

The native source, DSP, feature flags, release/signup configuration, domain,
noindex metadata and dependencies have no changes. No tag, push or deployment.
The supplied handoff directory remains untracked and untouched; only assets
used by the website are copied into the tracked implementation.

Two pre-existing emitted Vite shadow-config files caused the repository's
configuration guard to fail. They were preserved in
`test-output/studio-redesign/pre-existing/`, away from the config search path.
The initial production browser run overlapped a Windows frontend rebuild;
its transient 404 was discarded and the final lane is run after builds finish.
