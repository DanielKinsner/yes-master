# SVG Preset Icons Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (inline) or superpowers:subagent-driven-development (dispatched per task). Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Hard rule (carried from `CLAUDE.md`):** Do NOT read or copy anything from the parallel Codex repo. The icons in this plan are sourced exclusively from Lucide (https://lucide.dev, MIT licensed). If a chosen icon doesn't exist in current Lucide, fall back to Phosphor Icons (https://phosphoricons.com, MIT). Do not improvise SVG paths from training-data recall — fetch the exact path data from the upstream source.

**Goal:** Give each of the 8 named preset tiles (Universal, Clarity, Tape, Spatial, Oomph, Warmth, Punch, Loud) a distinct inline SVG icon above its label so Dan can scan the preset row by visual hierarchy rather than reading every label. Custom presets are surfaced through `UserPresetSection` rather than `PresetTiles` and so do not appear in the tile row — they are out of scope here, but the icon component must still accept a `"custom"` kind so it can be reused if `UserPresetSection` ever gets icons.

**Architecture:** Inline SVGs as a single React component file (`src/components/PresetIcon.tsx`) keyed by `Preset["kind"]`. Each icon is a self-contained `<svg>` element with the Lucide path data inlined; no `lucide-react` (or any other) dependency added. Icons inherit color via `stroke="currentColor"` so the active/inactive tile color flows through naturally. The component slots into `PresetTiles` in `src/App.tsx` above the existing `.tile-label`.

**Tech Stack:** TypeScript/React (no Rust changes). New file: `src/components/PresetIcon.tsx`. Modified files: `src/App.tsx`, `src/App.css`.

**Bundle impact (expected):** ~1–2 KB raw SVG-path text for nine icons. Gzipped delta should be under 500 bytes. The pre-icon bundle is ~253.6 KB / ~77.6 KB gzipped per the most recent Phase 12.2 handoff; the post-icon target is ≤ 255.5 KB / ≤ 78.2 KB gzipped. If the build exceeds those bounds something is wrong — likely a dependency was added by mistake.

---

## File Structure

- **Create** `src/components/PresetIcon.tsx`
  - Exports `<PresetIcon kind={...} />` component.
  - Maps each `Preset["kind"]` to a self-contained inline `<svg>`.
  - Lucide MIT license attribution at the top of the file.
- **Modify** `src/App.tsx`
  - Import `PresetIcon` and render it inside each `.tile` button, above `.tile-label`.
- **Modify** `src/App.css`
  - Add `.tile-icon` rule (size, inherited color, vertical rhythm).
  - Adjust `.tile`'s flex gap so the icon doesn't crash into the label.
- **Modify** `docs/progress.md`
  - Append a progress entry under the loop convention.
- **Create**: no other new files.
- **No** new npm dependencies. Do not run `npm install lucide-react` or any other icon package. The SVG path data is copied as static strings.

Each task is self-contained; the slice commits as a single push at the end — **only after Dan eyes-on approval on Task 4's visual smoke**.

---

## Pre-flight checks (do these before Task 1)

- [ ] **PF.1: Confirm the typography pass shipped.**

  This plan is queued behind the typography pass (HANDOFF P1 #6). Read the tail of `docs/progress.md`. If the typography slice has not yet been committed, **stop** and execute that one first (or ask Dan which to run). The icon CSS spacing in Task 3 assumes the post-typography type scale.

  ```bash
  tail -n 80 docs/progress.md
  ```

- [ ] **PF.2: Confirm no `lucide-react` dependency is present.**

  ```bash
  grep -i lucide package.json || echo "OK — no lucide dependency"
  ```

  Expected: `OK — no lucide dependency`. If `lucide-react` is present, **stop and ask Dan** before proceeding. The plan's inline-SVG approach is incompatible with also pulling in the runtime package (we'd be shipping two copies of the same paths).

- [ ] **PF.3: Confirm the 8-preset list in `src/App.tsx`'s `PRESET_OPTIONS` matches `src-tauri/src/types.rs::Preset`.**

  ```bash
  grep -n "kind:" src/App.tsx | head -20
  grep -nE "Universal|Clarity|Tape|Spatial|Oomph|Warmth|Punch|Loud|Custom" src-tauri/src/types.rs
  ```

  Expected: both lists contain the 8 named presets `universal | clarity | tape | spatial | oomph | warmth | punch | loud` plus `custom` for user-saved presets. If either list has drifted, **stop and ask Dan** — adding icons to a mismatched preset set risks orphan icons or unhandled cases.

---

## Task 1: Choose icons and create `src/components/PresetIcon.tsx`

**Files:**
- Create: `src/components/PresetIcon.tsx`

This task adds the new component file but does NOT yet wire it into `App.tsx` — the next task does. Splitting it this way means we can build + type-check + smoke the file standalone before any visible change in the app.

---

- [ ] **Step 1.1: Verify each chosen Lucide icon exists in the current Lucide release.**

  The recommended mapping (Dan can override at execution time — see Task 4 smoke step):

  | Preset kind | Lucide icon name | Why it fits |
  |---|---|---|
  | `universal` | `sparkles` | Universal-default, "magic moment" / safe-recommended affordance. |
  | `clarity` | `eye` | Focus / definition / "see it clearly". |
  | `tape` | `disc` | Reel/disc visual; Lucide doesn't have a stable `cassette-tape` glyph. |
  | `spatial` | `maximize-2` | Outward arrows → width/depth expansion. |
  | `oomph` | `speaker` | Low-end weight, sub-cabinet imagery. |
  | `warmth` | `flame` | Warmth/heat — direct semantic match. |
  | `punch` | `zap` | Lightning bolt → transient impact. |
  | `loud` | `megaphone` | Loudness / level / volume push. |
  | `custom` | `sliders` | User-shaped preset; sliders are a universal "tweakables" signifier. |

  Fetch each icon's SVG from `https://lucide.dev/icons/<name>` (or `https://unpkg.com/lucide-static@latest/icons/<name>.svg`) and confirm:
  - The icon exists at that URL.
  - The license declaration at the Lucide repo root (`https://github.com/lucide-icons/lucide/blob/main/LICENSE`) still reads MIT.
  - The path data uses `stroke` rendering (Lucide's whole point), not `fill` — this matters because our CSS will set `stroke="currentColor"` to inherit the active/inactive tile color.

  If any icon name has been deprecated or renamed in current Lucide:
  1. Try the documented Lucide fallback (e.g., `volume-2` if `speaker` got renamed, `maximize` if `maximize-2` is gone).
  2. If no clean Lucide match remains, fall back to Phosphor Icons (`https://phosphoricons.com`, also MIT) — pick the most semantically similar regular-weight icon and note the swap in a code comment.
  3. If neither Lucide nor Phosphor has a clean match, **stop and ask Dan** before hand-authoring SVG paths. We'd rather drop the icon for that preset than ship a weak custom drawing.

- [ ] **Step 1.2: Create `src/components/PresetIcon.tsx` with the license header.**

  ```tsx
  // Phase 12.2 — Preset tile icons.
  //
  // Icons inlined from Lucide (https://lucide.dev), MIT licensed.
  // Lucide license: https://github.com/lucide-icons/lucide/blob/main/LICENSE
  //
  // We inline the path data instead of depending on `lucide-react` because
  // we only need 9 icons and the package ships 1000+. Each icon below
  // preserves Lucide's standard 24x24 viewBox, 2px stroke width, round
  // line caps and joins, and stroke="currentColor" so the SVG inherits
  // the parent tile's color (handles active/inactive state automatically).
  //
  // If any icon needs to be swapped, copy fresh path data from
  // `https://lucide.dev/icons/<name>` — do NOT improvise paths.
  
  import { Preset } from "../bindings";

  type IconKind = Preset["kind"];

  type IconProps = {
    kind: IconKind;
    className?: string;
    "aria-hidden"?: boolean;
  };

  const STROKE_PROPS = {
    width: 20,
    height: 20,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 2,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
  };

  // Each renderer returns the inner <path>/<circle>/etc. elements for a
  // 24x24 Lucide-style icon. The wrapping <svg> with shared stroke props
  // is added by the dispatcher at the bottom of this file.

  function SparklesPaths() {
    // Lucide "sparkles": https://lucide.dev/icons/sparkles
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE - verified in Step 1.1 */}
      </>
    );
  }

  function EyePaths() {
    // Lucide "eye": https://lucide.dev/icons/eye
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function DiscPaths() {
    // Lucide "disc": https://lucide.dev/icons/disc
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function Maximize2Paths() {
    // Lucide "maximize-2": https://lucide.dev/icons/maximize-2
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function SpeakerPaths() {
    // Lucide "speaker": https://lucide.dev/icons/speaker
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function FlamePaths() {
    // Lucide "flame": https://lucide.dev/icons/flame
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function ZapPaths() {
    // Lucide "zap": https://lucide.dev/icons/zap
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function MegaphonePaths() {
    // Lucide "megaphone": https://lucide.dev/icons/megaphone
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  function SlidersPaths() {
    // Lucide "sliders": https://lucide.dev/icons/sliders
    return (
      <>
        {/* PASTE LUCIDE PATH DATA HERE */}
      </>
    );
  }

  export function PresetIcon({ kind, className, ...rest }: IconProps) {
    let inner: JSX.Element;
    switch (kind) {
      case "universal":
        inner = <SparklesPaths />;
        break;
      case "clarity":
        inner = <EyePaths />;
        break;
      case "tape":
        inner = <DiscPaths />;
        break;
      case "spatial":
        inner = <Maximize2Paths />;
        break;
      case "oomph":
        inner = <SpeakerPaths />;
        break;
      case "warmth":
        inner = <FlamePaths />;
        break;
      case "punch":
        inner = <ZapPaths />;
        break;
      case "loud":
        inner = <MegaphonePaths />;
        break;
      case "custom":
        inner = <SlidersPaths />;
        break;
      default: {
        // Exhaustiveness check — if a new Preset variant is added to
        // bindings.ts and not handled here, the compiler will flag it
        // via the `never` assignment.
        const _exhaustive: never = kind;
        return null;
      }
    }
    return (
      <svg
        {...STROKE_PROPS}
        className={className}
        aria-hidden={rest["aria-hidden"] ?? true}
      >
        {inner}
      </svg>
    );
  }
  ```

  **Important:** the `PASTE LUCIDE PATH DATA HERE` comments are placeholders. Do NOT commit those placeholders — Step 1.3 fills them in.

- [ ] **Step 1.3: Paste verified Lucide path data into each `*Paths()` function.**

  For each icon, fetch the upstream SVG from `https://unpkg.com/lucide-static@latest/icons/<name>.svg` or copy directly from `https://lucide.dev/icons/<name>` (the page shows the raw SVG). Take only the inner elements (the `<path d="…" />`, `<circle …/>`, `<line …/>` children) — strip the outer `<svg>` wrapper, the `xmlns`, the `width`/`height`/`viewBox`/`fill`/`stroke`/`stroke-width`/`stroke-linecap`/`stroke-linejoin` attributes — those are all supplied by `STROKE_PROPS` and the wrapping `<svg>` in `PresetIcon`.

  Convert SVG attribute names to JSX:
  - `stroke-width` → `strokeWidth` (already on the wrapper, drop from children if present)
  - `stroke-linecap` → `strokeLinecap` (drop from children)
  - `stroke-linejoin` → `strokeLinejoin` (drop from children)
  - Self-close all tags (`<path … />`, not `<path …>`).

  Example for `sparkles` (the actual path data will be whatever Lucide ships at fetch time — this is illustrative shape only, do not assume current):

  ```tsx
  function SparklesPaths() {
    // Lucide "sparkles": https://lucide.dev/icons/sparkles
    return (
      <>
        <path d="M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z" />
        <path d="M20 3v4" />
        <path d="M22 5h-4" />
        <path d="M4 17v2" />
        <path d="M5 18H3" />
      </>
    );
  }
  ```

  Repeat for all 9 icons. Use the canonical Lucide path data fetched in Step 1.1.

- [ ] **Step 1.4: Confirm no placeholder text remains in the file.**

  ```bash
  grep -n "PASTE LUCIDE PATH DATA HERE" src/components/PresetIcon.tsx && echo "FAIL — placeholders left" || echo "OK — no placeholders"
  ```

  Expected: `OK — no placeholders`.

- [ ] **Step 1.5: Type-check the new file.**

  ```bash
  npx tsc --noEmit
  ```

  Expected: clean. If the `Preset` type import path is wrong (e.g., the component is at `src/components/` and `bindings.ts` is at `src/`), the relative import `../bindings` is correct — verify against `src/bindings.ts` (created in Phase 11 per the handoff).

  If the compiler flags an exhaustiveness error in the `default` branch's `_exhaustive: never` line, that means `Preset["kind"]` in `bindings.ts` has gained or lost a variant since this plan was written — re-sync the `switch` arms with the current type before continuing.

---

## Task 2: Wire `<PresetIcon />` into `PresetTiles` in `src/App.tsx`

**Files:**
- Modify: `src/App.tsx` (`PresetTiles` component around line 832; import statement near top)

---

- [ ] **Step 2.1: Add the import.**

  Near the existing imports at the top of `src/App.tsx`, add:

  ```tsx
  import { PresetIcon } from "./components/PresetIcon";
  ```

  Place it grouped with other local-module imports (the existing imports from `./hooks/useTrackMaster` and `./bindings` are a good neighborhood).

- [ ] **Step 2.2: Render `<PresetIcon />` inside each tile button.**

  Locate the existing tile render around line 845–858:

  ```tsx
  {PRESET_OPTIONS.map((p) => {
    const active = isPresetActive(selected, p.value);
    return (
      <button
        key={p.label}
        type="button"
        className={"tile " + (active ? "active" : "")}
        onClick={() => onChange(p.value)}
      >
        <span className="tile-label">{p.label}</span>
        <span className="tile-blurb">{p.blurb}</span>
      </button>
    );
  })}
  ```

  Replace with:

  ```tsx
  {PRESET_OPTIONS.map((p) => {
    const active = isPresetActive(selected, p.value);
    return (
      <button
        key={p.label}
        type="button"
        className={"tile " + (active ? "active" : "")}
        onClick={() => onChange(p.value)}
      >
        <PresetIcon kind={p.value.kind} className="tile-icon" />
        <span className="tile-label">{p.label}</span>
        <span className="tile-blurb">{p.blurb}</span>
      </button>
    );
  })}
  ```

  Notes:
  - Icon is the first child inside the button — it visually sits above the label because `.tile` is `flex-direction: column`.
  - `aria-hidden` defaults to `true` inside the `PresetIcon` component, so the icon is decorative and screen readers still read the label.
  - `p.value.kind` is a string discriminant; TypeScript narrows it correctly because `PRESET_OPTIONS` is typed `{ value: Preset; ... }[]`.

---

## Task 3: Style the icon in `src/App.css`

**Files:**
- Modify: `src/App.css` (the `.tile*` block, currently around lines 817–855)

---

- [ ] **Step 3.1: Add a `.tile-icon` rule and tighten `.tile`'s gap.**

  Locate the existing `.tile` rule (around line 817):

  ```css
  .tile {
    background: var(--bg-1);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.75rem 0.85rem;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  ```

  The existing `gap: 0.2rem` is fine for label-above-blurb but too tight for icon-above-label. Add the icon rule immediately after `.tile-blurb` (around line 854), and adjust `.tile`'s vertical rhythm so the icon has room to breathe:

  ```css
  .tile-icon {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--text-2);
    margin-bottom: 0.35rem;
    flex-shrink: 0;
    /* stroke="currentColor" on the inline SVG means the icon color
       inherits from this rule. Active-tile override below picks up the
       accent color from the parent .tile.active. */
  }

  .tile.active .tile-icon {
    color: var(--accent);
  }

  .tile:hover .tile-icon {
    color: var(--text-1);
  }

  .tile.active:hover .tile-icon {
    /* Don't dim back from accent when hovering an already-active tile. */
    color: var(--accent);
  }
  ```

  Notes:
  - Icon dimensions are intentionally smaller than the surrounding text scale so the icon supports rather than competes with the label.
  - `flex-shrink: 0` prevents the icon collapsing when a tile is narrow (the `.tile-row` is `minmax(120px, 1fr)` so tiles get pretty tight on small windows).
  - The hover/active rules cascade from `.tile.active` and `.tile:hover`, which already exist — we don't redefine those, just add icon-color overrides.
  - We do NOT set `stroke` directly here because the SVG uses `stroke="currentColor"`, which reads `color` from this rule.

- [ ] **Step 3.2: Confirm no other selectors regress.**

  Quick visual mental-check of the CSS order:
  1. `.tile` — flex container, default text color.
  2. `.tile:hover` — border + bg change.
  3. `.tile.active` — accent border + bg + color override on label.
  4. `.tile-label` — font weight/size.
  5. `.tile.active .tile-label` — accent color.
  6. `.tile-blurb` — small muted line.
  7. **NEW** `.tile-icon` + active/hover variants.

  No specificity conflicts because `.tile-icon` is a new class.

---

## Task 4: Verification (build + Dan-eyes-on visual smoke)

**Files:**
- Modify: `docs/progress.md` (append) — but ONLY after Task 4.3 passes.

This task is gated. Do not commit before Step 4.3 receives explicit "looks good, ship it" approval from Dan. Removing or relaxing this gate defeats the point of the slice — these icons are a visual decision and Dan's eye is the integration test.

---

- [ ] **Step 4.1: Run the frontend build (clean).**

  ```bash
  cd "C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\album-mastering-studio-claude-build"
  npm run build
  ```

  Expected:
  - Clean type-check (no TS errors).
  - Clean Vite build with no warnings about the new file.
  - Bundle size growth: roughly 1–2 KB raw / ≤ 500 bytes gzipped delta.
  - Final size target: ≤ 255.5 KB / ≤ 78.2 KB gzipped.

  If the bundle grew by more than 5 KB, check:
  - Did anyone import `lucide-react`? (`grep -rn lucide-react src/`)
  - Did anyone paste in giant `<defs>`/`<mask>`/embedded raster data? (`wc -c src/components/PresetIcon.tsx` — should be under 8 KB).

- [ ] **Step 4.2: Run the Rust type-check (defensive — shouldn't be affected).**

  ```bash
  cd "C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\album-mastering-studio-claude-build\src-tauri"
  cargo check --tests
  ```

  Expected: clean. This slice doesn't touch Rust, but a clean check guarantees we haven't accidentally broken `bindings.ts`/`types.rs` parity (the new component reads `Preset["kind"]` from `bindings.ts`).

  If `cargo test` is desired, run it — but the carried-forward pitfall #1 still applies: `npm run tauri dev` running in another terminal will lock the binary. `cargo check --tests` is the cheaper proof of correctness for a frontend-only slice.

- [ ] **Step 4.3: STOP. Ask Dan to visually approve the icon-to-preset mapping.**

  Print this exact message to the chat, then wait:

  > Phase 12.2 P1: SVG preset icons are built and ready for visual review.
  >
  > Please run `npm run tauri dev` and look at the preset tile row. Each of the eight named presets now has an inline SVG icon above the label:
  >
  > - Universal → Sparkles
  > - Clarity → Eye
  > - Tape → Disc
  > - Spatial → Maximize2 (outward arrows)
  > - Oomph → Speaker
  > - Warmth → Flame
  > - Punch → Zap (lightning bolt)
  > - Loud → Megaphone
  >
  > Two questions:
  >
  > 1. Does any icon feel wrong for its preset? (For example: "Tape should look like a cassette, not a disc," or "Oomph should be a subwoofer, not a speaker.")
  > 2. Does the visual weight feel right? (Too big? Too small? Crowds the label? Tile feels unbalanced?)
  >
  > Reply "ship it" if good. Reply with the specific complaint if not — I'll swap icons / tweak sizing without re-running the build pipeline from scratch.

  Behavior depending on Dan's response:

  - **"ship it" or equivalent approval** → proceed to Step 4.4.
  - **Specific icon complaint** → fetch a new Lucide (or Phosphor) icon, replace the `*Paths()` body for that preset, rerun Step 4.1 (build) and Step 4.3 (ask again). Repeat until approval.
  - **Sizing complaint** → adjust the `.tile-icon` `width` / `height` / `margin-bottom` in `src/App.css`. Rerun Step 4.1 and Step 4.3.
  - **"Hate it, revert"** → `git checkout -- src/App.tsx src/App.css && rm src/components/PresetIcon.tsx` and stop. Append a progress.md entry noting the experiment and Dan's rejection. Do NOT commit.

  Under no circumstance should this step be skipped — committing without Dan's eyes-on visual approval is an SLA violation per Dan's stated preferences (UI polish slices are subjective and benefit from his review).

- [ ] **Step 4.4: Append the progress.md entry.**

  Open `docs/progress.md` and append:

  ```markdown

  ## 2026-05-12 — Phase 12.2 P1: SVG preset icons (visual hierarchy)

  Goal:

  Dan's reference screenshot from the parallel Codex build had distinct icons per preset tile. Adding them improves visual scanning of the preset row and gives each preset a memorable visual handle alongside the label.

  What changed:

  Frontend (`src/components/PresetIcon.tsx` — new file):

  - Self-contained inline-SVG component. One `<svg>` per `Preset["kind"]` variant (9 total: universal, clarity, tape, spatial, oomph, warmth, punch, loud, custom).
  - Icons sourced from Lucide (MIT licensed, https://lucide.dev). Path data copied directly — no `lucide-react` dependency added. License attribution at the top of the file.
  - `stroke="currentColor"` on every icon so the SVG inherits the parent tile's `color`, which means active/inactive state, hover, and theme changes all flow through without per-icon CSS.

  Frontend (`src/App.tsx`):

  - `PresetTiles` now renders `<PresetIcon kind={p.value.kind} className="tile-icon" />` as the first child of each `.tile` button, above the existing label and blurb.

  Frontend (`src/App.css`):

  - New `.tile-icon` rule (1.25rem square, muted default color, accent on active, intermediate on hover).
  - No other style regressions.

  Icon mapping (post-Dan-approval):

  - Universal → Sparkles
  - Clarity → Eye
  - Tape → Disc
  - Spatial → Maximize2
  - Oomph → Speaker
  - Warmth → Flame
  - Punch → Zap
  - Loud → Megaphone
  - Custom → Sliders (reserved; not currently rendered since custom presets surface through `UserPresetSection`, not `PresetTiles`)

  Verification:

  - `npm run build`: clean. Bundle <PINNED_SIZE_HERE> / <PINNED_GZIP_HERE> gzipped (delta from pre-slice baseline well under the +5 KB regression bar).
  - `cargo check --tests`: clean (no Rust changes; sanity check only).
  - Dan's visual smoke: approved on <DATE>.

  What failed or remains partial:

  - No automated frontend test for icon presence / mapping (vitest infra still deferred per HANDOFF infra #13).
  - `UserPresetSection` still uses text-only chips. Could reuse `<PresetIcon kind={p.kind === "track" ? ... : ...} />` later, but the `UserPreset.kind` enum is `"track" | "album" | "shared"`, not `Preset["kind"]`, so a small mapping decision is needed first — out of scope here.

  Next recommended slice:

  Phase 12.2 P1 polish (typography + SVG icons) is complete. Stop and ask Dan for the next direction: listening notes / preset rebalancing / `PHASE 12 CONFIRMED — proceed to 13`.
  ```

  Replace `<PINNED_SIZE_HERE>`, `<PINNED_GZIP_HERE>`, and `<DATE>` with the actual numbers from Step 4.1's build output and the current date.

- [ ] **Step 4.5: Commit and push.**

  Verify the working tree is what we expect:

  ```bash
  cd "C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\album-mastering-studio-claude-build"
  git status --short
  ```

  Expected: `A src/components/PresetIcon.tsx`, `M src/App.tsx`, `M src/App.css`, `M docs/progress.md`. Nothing else. If `package.json` or `package-lock.json` is modified, **stop** — a dependency was added by mistake and must be removed before committing.

  ```bash
  git add src/components/PresetIcon.tsx src/App.tsx src/App.css docs/progress.md
  git commit -m "$(cat <<'EOF'
  Phase 12.2 P1: SVG preset icons (visual hierarchy)

  Each named preset tile now has a distinct inline SVG icon above its
  label. Icons sourced from Lucide (MIT) and inlined directly — no new
  dependency. PresetIcon component is keyed by Preset["kind"] and uses
  stroke="currentColor" so active/hover state flows through naturally.

  Mapping (Dan-approved at execution time):
  Universal/Sparkles, Clarity/Eye, Tape/Disc, Spatial/Maximize2,
  Oomph/Speaker, Warmth/Flame, Punch/Zap, Loud/Megaphone. Custom
  (Sliders) is wired in the component for future reuse by
  UserPresetSection but is not rendered in the main tile row.

  Verification:
  - npm run build: clean, bundle delta well under +5 KB.
  - cargo check --tests: clean (no Rust changes).
  - Dan's visual smoke: approved.

  Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
  EOF
  )"
  git push origin master
  ```

  Confirm the new commit SHA in the push output.

---

## Hard constraints (do not violate)

- **Never** read or copy from the parallel Codex repo. Icon designs come from Lucide (or Phosphor as documented fallback). Do not browse the Codex build's icon set for "reference."
- **Open-source license required**. Lucide is MIT; the license attribution comment at the top of `PresetIcon.tsx` is non-negotiable. If Phosphor (also MIT) is used as fallback for any icon, add a parallel attribution comment.
- **Inline SVG only.** Do not introduce external asset files (`.svg` files in `public/`, `src/assets/`, etc.). Inline keeps the bundle self-contained and the icon-color inheritance trivial.
- **No new npm dependencies.** Do not run `npm install lucide-react`, `react-icons`, `@phosphor-icons/react`, or any other icon package. The whole point of this approach is to avoid shipping 100+ unused glyphs.
- **No `docs/PRODUCT.md` edits.** This is a UI-polish slice, not a product-direction change.
- **No `private-audio-fixtures/` touches.** This slice is pure frontend; the fixtures are irrelevant.
- **Do not skip Step 4.3.** Dan's visual approval is a gating step. The slice is subjective polish; shipping without his eye on it violates the stated "P1 — UX polish Dan asked for" framing in the handoff.

---

## Next slice (after this ships)

Phase 12.2 P1 polish (typography + SVG preset icons) is **complete** with this commit. The Phase 12.2 backlog at this point consists of:

- `compression_density` (real envelope-following compressor) — last remaining unwired Advanced control. Per the handoff, ~300–500 lines; needs a fresh brainstorm + spec + plan cycle. **Not** something to dive into without an approved design.
- Preset rebalancing — subjective, needs Dan's ear and listening notes.

The executor should **stop and ask Dan** for the next direction:

1. **Listening notes** → execute whatever Dan flags (preset rebalancing, specific preset tunings, advisory wording, etc.).
2. **`compression_density` plan** → spawn a brainstorming/spec/plan cycle (separate session). Do not start coding.
3. **`PHASE 12 CONFIRMED — proceed to 13`** → if Dan is satisfied with Track Master quality, he writes that sentinel into `progress.md` by hand. Agents never cross phase boundaries autonomously.

Per the goal directive: subjective sound-quality decisions take precedence over the wired-controls queue if listening notes come in first.

---

## Self-Review Checklist (for the plan author)

After writing, the plan author checks:

1. **Source independence** — every reference to icon design points at Lucide (or Phosphor as fallback). No section says "look at the Codex screenshot," "match the existing app," or anything that would require reading the parallel repo. ✓
2. **License attribution required** — the plan instructs the executor to include the Lucide MIT license attribution in the component file and (if used) the Phosphor attribution alongside. ✓
3. **No new dependencies** — both the file structure section and the hard-constraints section forbid `npm install lucide-react` / `react-icons` / `@phosphor-icons/react`. Pre-flight check PF.2 verifies the baseline. Build verification Step 4.1 catches accidental additions via bundle growth. ✓
4. **Inline SVG only** — no `public/icons/`, no external asset files. ✓
5. **Coverage of all 8 named presets + Custom** — the icon mapping table in Step 1.1 covers Universal, Clarity, Tape, Spatial, Oomph, Warmth, Punch, Loud, Custom. The `switch` in `PresetIcon`'s dispatcher has 9 cases plus the exhaustiveness `never` check. ✓
6. **Color-inheritance is correct** — every icon uses `stroke="currentColor"`; CSS sets `color` on `.tile-icon` and overrides on `.tile.active .tile-icon` and `.tile:hover .tile-icon`. No icon-specific color attributes. ✓
7. **Dan-eyes-on gate is explicit and unmissable** — Task 4 Step 4.3 has a STOP marker, a literal message to print to chat, and a guard that says "Do NOT commit without approval." The commit step (4.5) is sequenced after 4.3. ✓
8. **No `docs/PRODUCT.md` edits** — confirmed by inspection of the file structure section. ✓
9. **No private fixtures touched** — confirmed; slice is pure frontend. ✓
10. **No placeholders left** — search the plan for "TBD", "TODO", "implement later", "Add appropriate error handling", "fill in details". The only intentional placeholders are the `PASTE LUCIDE PATH DATA HERE` markers in Step 1.2, which Step 1.3 fills in and Step 1.4 verifies are gone. ✓
11. **Type consistency** — `PresetIcon` accepts `Preset["kind"]` from `bindings.ts`; the dispatcher's `switch` arms match the union exactly; the `never` exhaustiveness check will compile-error if the union drifts. ✓
12. **Phase boundary respected** — the "Next slice" section explicitly tells the executor to stop and ask Dan rather than crossing into Phase 13 work autonomously. ✓

---

*Plan ready for execution. Awaiting typography pass to land first; on completion of that, this is the next P1 polish slice.*
