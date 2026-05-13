# Typography Pass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bump text sizes consistently across Album Mastering Studio so the UI feels less cramped on Dan's monitor while preserving visual hierarchy. Dan's note: "UI overall could use larger text overall." This is a pure-CSS slice — no JS, no schema, no logic changes.

**Architecture:** Two-layer change.

1. **Base bump.** Raise the root `font-size` on `:root` from `14px` to `16px`. Because almost every selector in `App.css` uses `rem` units, this single change proportionally enlarges the entire UI by ~14% (16/14 = 1.143×). That handles "larger overall" in one stroke.
2. **Floor-lift on the smallest labels.** A handful of selectors currently use `0.65rem` or `0.7rem`, which even at the new 16px base land at ~10.4 px and ~11.2 px — still uncomfortably small for slider micro-labels and uppercase tag chips. These get a targeted bump to `0.78rem` (~12.5 px) so the floor of the type scale lands at a readable size without making them compete with primary copy.

Hierarchy is preserved because every change is proportional — `track-title` (1.3 rem) stays larger than `track-sub` (0.8 rem) stays larger than `track-meta` (0.7 rem → 0.78 rem) etc. The page title (`empty-state h1` @ 1.6 rem) and drop-overlay title (1.4 rem) ride the base bump and end up at 25.6 px / 22.4 px — still proudly large, not oversized for a 240 px sidebar + workspace layout.

**Tech Stack:** CSS only (`src/App.css`). No `src/App.tsx` logic edits. No new dependencies, no font-family changes, no responsive breakpoints added.

---

## File Structure

- **Modify** `src/App.css`:
  - `:root` font-size: `14px` → `16px`.
  - Floor-lift selectors using `0.65rem` and `0.7rem` micro-text to `0.78rem` where they're slider labels, badge chips, tag text, meta lines, or section labels that Dan reads frequently.
  - A few selectors intentionally stay at their current sub-`0.8rem` value (decorative chrome where lifting would crowd the layout — see Task 1 inventory).
- **Modify** `src/App.tsx`: **none expected.** Grep confirms there are zero `fontSize` inline-style declarations. Task 3 re-confirms before commit.
- **Modify** `docs/progress.md`: append a progress entry under the loop convention.
- **Create**: none.

This slice ships as a single commit at the end after Dan's eyes-on smoke confirms the new sizes feel right.

---

## Task 1: Inventory current font-sizes and pin the before/after table

**Files:**
- Read-only: `src/App.css`

This task pins the source of truth so the executor in Task 2 isn't re-deciding values per selector. Every CSS edit in Task 2 must match exactly one row of this table.

---

- [ ] **Step 1.1: Read `src/App.css` end-to-end and confirm the inventory below matches the live file**

The before/after table was assembled by grepping `font-size` against `src/App.css` at plan-write time. Before doing any edits, re-run a quick eye-pass through the file to confirm no `font-size` declarations have been added or moved since. If anything differs, stop and reconcile against the live file before proceeding to Task 2 (do not silently update entries — the divergence itself is information).

```bash
# Sanity check: count and list font-size lines.
# Should match the row count of the table below (62 declarations).
```

Use the Grep tool against `src/App.css` with pattern `font-size` to verify line count.

---

- [ ] **Step 1.2: Confirm the before/after table is internally consistent**

For each row: the **Action** column is the source of truth. `keep` rows are intentional no-ops (decorative chrome, headings already big, inputs that inherit from `:root`). `lift` rows go from `0.65rem`/`0.7rem` micro-text to `0.78rem`. The `:root` row is the base bump.

**Pixel values in the After column assume the new 16px base.** Where a selector is `keep`, the pixel value still grows from the base bump alone (e.g., `0.85rem` becomes 11.9 px → 13.6 px without any change to its declaration).

| # | Line | Selector | Current | After | Action | Notes |
|---|---:|---|---:|---:|---|---|
| 1 | 24 | `:root` | `14px` | `16px` | **base bump** | The one declaration that drives everything else. |
| 2 | 41 | `button` | `inherit` | `inherit` | keep | Inherits the new 16px naturally. |
| 3 | 48 | `input, select` | `inherit` | `inherit` | keep | Same. |
| 4 | 77 | `.brand` | `0.85rem` | `0.85rem` | keep | Sidebar brand — rides base bump. |
| 5 | 84 | `.mode-pill` | `0.65rem` | `0.78rem` | **lift** | Sidebar uppercase pill — small + tracked. |
| 6 | 109 | `.mode-toggle button` | `0.7rem` | `0.78rem` | **lift** | Sidebar Track/Album toggle text. |
| 7 | 132 | `.section-label` | `0.65rem` | `0.78rem` | **lift** | "TRACKS" / "PRESETS" sidebar headers. |
| 8 | 145 | `.add-btn` | `0.75rem` | `0.75rem` | keep | Sidebar `+ Add` button — rides base bump. |
| 9 | 165 | `.track-empty` | `0.85rem` | `0.85rem` | keep | "No tracks yet" placeholder. |
| 10 | 203 | `.track-index` | `0.7rem` | `0.7rem` | keep | Track list left-rail — Dan said to leave the track list alone unless it conflicts; bigger here would force the `1.6rem` index column wider. |
| 11 | 227 | `.track-name` | `0.85rem` | `0.85rem` | keep | Track list — rides base bump. |
| 12 | 234 | `.track-meta` | `0.7rem` | `0.7rem` | keep | Track list secondary line — stays paired with `.track-index`. |
| 13 | 241 | `.override-mark` | `0.85em` | `0.85em` | keep | `em` not `rem` — relative to parent; do not touch. |
| 14 | 249 | `.track-remove` | `1rem` | `1rem` | keep | The × button glyph. |
| 15 | 264 | `.sidebar-status` | `0.75rem` | `0.75rem` | keep | Sidebar footer. |
| 16 | 287 | `.empty-state h1` | `1.6rem` | `1.6rem` | keep | Empty-state hero title — rides base bump to 25.6 px. |
| 17 | 300 | `.empty-foot` | `0.85rem` | `0.85rem` | keep | Hint under hero. |
| 18 | 317 | `button.primary` | `0.9rem` | `0.9rem` | keep | Primary CTA — already prominent. |
| 19 | 349 | `.album-stat` | `0.85rem` | `0.85rem` | keep | Album header stats. |
| 20 | 373 | `.override-banner` | `0.8rem` | `0.8rem` | keep | Override banner body. |
| 21 | 410 | `.override-toggle button` | `0.78rem` | `0.78rem` | keep | Override toggle. |
| 22 | 436 | `.track-title` | `1.3rem` | `1.3rem` | keep | Current track H1. |
| 23 | 446 | `.track-sub` | `0.8rem` | `0.8rem` | keep | Sub-line under track title. |
| 24 | 456 | `.track-badge` | `0.7rem` | `0.78rem` | **lift** | Tracked-uppercase badge — fix small-tracked-text legibility. |
| 25 | 490 | `.undo-redo-bar .ghost-btn` | `0.78rem` | `0.78rem` | keep | Undo/redo bar. |
| 26 | 510 | `.live-update-badge` | `0.7rem` | `0.78rem` | **lift** | Live-update chip near StaleBar — Dan glances at it during slider drags. |
| 27 | 527 | `.clip-indicator` | `0.7rem` | `0.78rem` | **lift** | Phase 12.2 clipping/GR readout — explicitly called out in the brief ("StaleBar text"). |
| 28 | 573 | `.analysis-summary` | `0.78rem` | `0.78rem` | keep | Already at the floor. |
| 29 | 586 | `.analysis-summary > summary` | `0.72rem` | `0.78rem` | **lift** | Disclosure pill — same family as section-label. |
| 30 | 614 | `.tag` | `0.72rem` | `0.78rem` | **lift** | Story tag chips. |
| 31 | 679 | `.wf-hint` | `0.7rem` | `0.78rem` | **lift** | Waveform hint line. |
| 32 | 691 | `.wf-empty` | `0.85rem` | `0.85rem` | keep | "Drop audio" inside waveform card. |
| 33 | 720 | `.play-btn` | `1rem` | `1rem` | keep | Transport play glyph. |
| 34 | 731 | `.time` | `0.85rem` | `0.85rem` | keep | Transport timer. |
| 35 | 745 | `.icon-btn` | `0.9rem` | `0.9rem` | keep | Transport icon buttons. |
| 36 | 772 | `.ab-toggle button` | `0.8rem` | `0.8rem` | keep | A/B toggle. |
| 37 | 790 | `.vm-toggle` | `0.8rem` | `0.8rem` | keep | Volume Match checkbox label. |
| 38 | 843 | `.tile-label` | `0.85rem` | `0.85rem` | keep | Preset tile names — bumps via base. (Brief: "Preset tile names: bump if currently small" — at 13.6 px after base bump these are no longer small.) |
| 39 | 851 | `.tile-blurb` | `0.7rem` | `0.78rem` | **lift** | One-line blurb under preset name — small + Dan reads during preset shopping. |
| 40 | 870 | `.user-preset-empty` | `0.78rem` | `0.78rem` | keep | At the floor. |
| 41 | 893 | `.user-preset-apply` | `0.78rem` | `0.78rem` | keep | At the floor. |
| 42 | 903 | `.user-preset-kind` | `0.7rem` | `0.78rem` | **lift** | Kind chip under user preset. |
| 43 | 912 | `.user-preset-delete` | `0.9rem` | `0.9rem` | keep | Delete glyph. |
| 44 | 934 | `.user-preset-name` | `0.8rem` | `0.8rem` | keep | Save input. |
| 45 | 966 | `.slider-label` | `0.8rem` | `0.8rem` | keep | Macro slider labels — already comfortable. |
| 46 | 977 | `.slider-value` | `0.8rem` | `0.8rem` | keep | Macro slider numeric value. |
| 47 | 984 | `.adv-number` | `0.78rem` | `0.78rem` | keep | Advanced number field digit text — at floor. (Brief called out "Number fields' digit text: bump to match the new base." Floor + base bump = 12.5 px, which matches the new macro slider value at 12.8 px.) |
| 48 | 1009 | `.slider-number` | `0.8rem` | `0.8rem` | keep | Macro number field digits. |
| 49 | 1042 | `.stale-bar` | `0.8rem` | `0.8rem` | keep | StaleBar body text. |
| 50 | 1089 | `.ghost-btn` | `0.8rem` | `0.8rem` | keep | Ghost buttons. |
| 51 | 1120 | `.advanced-toggle` | `0.85rem` | `0.85rem` | keep | "Advanced ▾" toggle. |
| 52 | 1152 | `.adv-label` | `0.7rem` | `0.78rem` | **lift** | AdvancedPanel uppercase labels — brief specifically called out "slider labels in AdvancedPanel." |
| 53 | 1175 | `.adv-value` | `0.8rem` | `0.8rem` | keep | AdvancedPanel slider readout. |
| 54 | 1194 | `.micro-btn` | `0.7rem` | `0.78rem` | **lift** | Micro buttons (reset / etc.) inside AdvancedPanel. |
| 55 | 1225 | `.drop-overlay-title` | `1.4rem` | `1.4rem` | keep | Drag-drop overlay hero — rides base bump. |
| 56 | 1233 | `.drop-overlay-hint` | `0.85rem` | `0.85rem` | keep | Drop overlay hint. |
| 57 | 1253 | `.toast` | `0.85rem` | `0.85rem` | keep | Error toast body. |
| 58 | 1261 | `.toast-close` | `1.1rem` | `1.1rem` | keep | Toast × glyph. |
| 59 | 1300 | `.receipt h2` | `1.05rem` | `1.05rem` | keep | Export receipt heading. |
| 60 | 1313 | `.receipt-path` | `0.78rem` | `0.78rem` | keep | At the floor. |
| 61 | 1352 | `.check-row` | `0.85rem` | `0.85rem` | keep | Receipt check row body. |
| 62 | 1368 | `.check-level` | `0.7rem` | `0.78rem` | **lift** | Receipt check level chip ("INFO" / "WARNING" / "CRITICAL"). |

**Count summary:**
- Base bump: 1 row (`:root`).
- Lift to `0.78rem`: 14 rows.
- Keep: 47 rows (decorative chrome, inherit-from-base inputs, or already at/above the floor).

**Net edits in Task 2: 15 `Edit` calls against `src/App.css`** (1 base + 14 lifts).

---

## Task 2: Apply the CSS edits, one selector at a time

**Files:**
- Modify: `src/App.css`

Each step is a single `Edit` call with exact `old_string` / `new_string`. Steps are ordered top-to-bottom by line number so the executor can scan once. If any `old_string` fails to match uniquely, stop and re-inventory against the live file — do not "fix" by adding more context blindly; the divergence may matter.

---

- [ ] **Step 2.1: Base bump (`:root`)**

```css
/* old */
  font-size: 14px;
}
```
```css
/* new */
  font-size: 16px;
}
```

The `}` after the line makes this unique in the file (the `:root` block ends there).

`old_string`:
```
  font-size: 14px;
}
```
`new_string`:
```
  font-size: 16px;
}
```

---

- [ ] **Step 2.2: `.mode-pill`**

`old_string`:
```
.mode-pill {
  font-size: 0.65rem;
```
`new_string`:
```
.mode-pill {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.3: `.mode-toggle button`**

`old_string`:
```
.mode-toggle button {
  flex: 1;
  background: transparent;
  color: var(--text-2);
  border: none;
  padding: 0.35rem 0.5rem;
  font-size: 0.7rem;
```
`new_string`:
```
.mode-toggle button {
  flex: 1;
  background: transparent;
  color: var(--text-2);
  border: none;
  padding: 0.35rem 0.5rem;
  font-size: 0.78rem;
```

(More context included because `.mode-toggle button` shares enough of its body with `.mode-toggle button.on` that we want an unambiguous anchor.)

---

- [ ] **Step 2.4: `.section-label`**

`old_string`:
```
.section-label {
  font-size: 0.65rem;
```
`new_string`:
```
.section-label {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.5: `.track-badge`**

`old_string`:
```
.track-badge {
  font-size: 0.7rem;
```
`new_string`:
```
.track-badge {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.6: `.live-update-badge`**

`old_string`:
```
.live-update-badge {
  font-size: 0.7rem;
```
`new_string`:
```
.live-update-badge {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.7: `.clip-indicator`**

`old_string`:
```
.clip-indicator {
  font-size: 0.7rem;
```
`new_string`:
```
.clip-indicator {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.8: `.analysis-summary > summary`**

`old_string`:
```
.analysis-summary > summary {
  cursor: pointer;
  user-select: none;
  display: inline-block;
  padding: 0.15rem 0.4rem;
  border: 1px solid var(--border-strong);
  border-radius: 3px;
  background: var(--bg-2);
  color: var(--text-1);
  font-size: 0.72rem;
```
`new_string`:
```
.analysis-summary > summary {
  cursor: pointer;
  user-select: none;
  display: inline-block;
  padding: 0.15rem 0.4rem;
  border: 1px solid var(--border-strong);
  border-radius: 3px;
  background: var(--bg-2);
  color: var(--text-1);
  font-size: 0.78rem;
```

---

- [ ] **Step 2.9: `.tag`**

`old_string`:
```
.tag {
  font-size: 0.72rem;
```
`new_string`:
```
.tag {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.10: `.wf-hint`**

`old_string`:
```
.wf-hint {
  margin: 0.3rem 0 0;
  font-size: 0.7rem;
```
`new_string`:
```
.wf-hint {
  margin: 0.3rem 0 0;
  font-size: 0.78rem;
```

---

- [ ] **Step 2.11: `.tile-blurb`**

`old_string`:
```
.tile-blurb {
  font-size: 0.7rem;
```
`new_string`:
```
.tile-blurb {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.12: `.user-preset-kind`**

`old_string`:
```
.user-preset-kind {
  color: var(--text-2);
  font-size: 0.7rem;
```
`new_string`:
```
.user-preset-kind {
  color: var(--text-2);
  font-size: 0.78rem;
```

---

- [ ] **Step 2.13: `.adv-label`**

`old_string`:
```
.adv-label {
  font-size: 0.7rem;
```
`new_string`:
```
.adv-label {
  font-size: 0.78rem;
```

---

- [ ] **Step 2.14: `.micro-btn`**

`old_string`:
```
.micro-btn {
  background: transparent;
  color: var(--text-2);
  border: 1px solid var(--border-strong);
  padding: 0.15rem 0.4rem;
  border-radius: 3px;
  font-size: 0.7rem;
```
`new_string`:
```
.micro-btn {
  background: transparent;
  color: var(--text-2);
  border: 1px solid var(--border-strong);
  padding: 0.15rem 0.4rem;
  border-radius: 3px;
  font-size: 0.78rem;
```

---

- [ ] **Step 2.15: `.check-level`**

`old_string`:
```
.check-level {
  font-size: 0.7rem;
```
`new_string`:
```
.check-level {
  font-size: 0.78rem;
```

---

## Task 3: Re-confirm no inline `fontSize` in App.tsx

**Files:**
- Read-only: `src/App.tsx`

---

- [ ] **Step 3.1: Grep for inline `fontSize` declarations**

Use the Grep tool with pattern `fontSize` against `src/App.tsx`.

**Expected result:** zero matches (this was confirmed at plan-write time; re-verify because App.tsx may have changed between plan write and execution).

**If matches appear:** for each match, evaluate whether the inline size is intentionally overriding a CSS rule (e.g., dynamically-computed waveform overlay text) or is just legacy. If intentional and proportional to the surrounding base, leave it. If it's a hardcoded `px` value of a label that should ride the base bump, swap the `px` value out for a `rem` equivalent — but only touch the `fontSize:` field; do not refactor the surrounding component logic.

**If still zero matches:** proceed to Task 4 unchanged.

---

## Task 4: Verification (build + Dan eyes-on smoke)

**Files:**
- None modified in this task — verification only.

This task is the gate on the slice. It explicitly requires Dan's eyeballs on the running app before commit, because typography is subjective.

---

- [ ] **Step 4.1: Run `npm run build` and confirm clean TS + flat bundle**

```bash
cd "C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\album-mastering-studio-claude-build"
npm run build
```

**Expected:**
- TypeScript compiles clean (no `tsc` errors).
- `dist/` written.
- Bundle size approximately flat. The session's last recorded number is ~253.65 KB / ~77.57 KB gzipped. This change is pure CSS text — expect ±100 bytes raw, ±20 bytes gzipped. Anything larger than that points at an unintended change somewhere else (stop and diff).

**If TS errors appear:** they should not — this slice does not touch `.ts`/`.tsx`. If errors show, they're from drift since the last green build, not from this slice. Stop and reconcile against `master` before continuing.

---

- [ ] **Step 4.2: STOP — ask Dan to launch the dev app and confirm the new sizes feel right**

This is a hard stop. Do not commit before Dan confirms. Typography is subjective and the whole point of this slice is "feels right on Dan's monitor."

Surface this to Dan with a message of the form:

> Typography pass is staged but not committed. Please run `npm run tauri dev` and click around — empty state, a loaded track, the Advanced panel, a preset shop, the export receipt. The base went from 14 px to 16 px and 14 small-label selectors were lifted from 0.65 / 0.7 / 0.72 rem to 0.78 rem. If anything feels too big, too small, or visually unbalanced, name the selector(s) and I'll adjust before commit.

**If Dan flags one or more selectors:**
- Adjust the table in Task 1 (mark which row(s) Dan vetoed, with a one-line reason).
- Apply the corresponding `Edit` calls — either revert to original `rem` value or pick a new target between current and new.
- Re-run Step 4.1 (`npm run build`).
- Loop back to Step 4.2 (ask Dan again) until Dan confirms.

**If Dan confirms first try:**
- Proceed to Task 5.

**Do not skip this step even if you're confident.** The brief explicitly says "subjective UI work" — Dan's eye is the verification, not the build.

---

## Task 5: Append progress.md entry + commit + push

**Files:**
- Modify: `docs/progress.md` (append a new entry)

---

- [ ] **Step 5.1: Append the progress entry**

Open `docs/progress.md` and append at the end. Use the warmth/air entry's shape — Goal / What changed / Verification / What failed or remains partial / Next recommended slice.

```markdown

## 2026-05-12 — Phase 12.2 P1: typography pass

Goal:

Close the first P1 polish slice after the Phase 12.2 wired-controls campaign. Dan's note from the listening session: "UI overall could use larger text overall." Pure-CSS slice — no JS, no schema, no logic. Plan at `docs/superpowers/plans/2026-05-12-typography-pass.md`.

What changed:

Frontend (`src/App.css`):

- **Base bump.** `:root` `font-size: 14px` → `16px`. Because almost every selector in `App.css` uses `rem` units, this single change proportionally enlarges the entire UI by ~14% (16/14 ≈ 1.143×).
- **Floor-lift on 14 micro-labels.** Selectors at `0.65rem` / `0.7rem` / `0.72rem` lifted to `0.78rem` so the smallest UI text lands at ~12.5 px after the base bump instead of ~10.4 px. Lifted: `.mode-pill`, `.mode-toggle button`, `.section-label`, `.track-badge`, `.live-update-badge`, `.clip-indicator`, `.analysis-summary > summary`, `.tag`, `.wf-hint`, `.tile-blurb`, `.user-preset-kind`, `.adv-label`, `.micro-btn`, `.check-level`.
- **Intentionally kept at current values:** track-list left-rail text (would force the index column wider), headings (already prominent, ride the base bump), inputs and buttons that `inherit` from `:root` (ride the base bump for free).

Frontend (`src/App.tsx`):

- No changes. Confirmed zero inline `fontSize` declarations at plan-execute time.

Verification:

- `npm run build`: clean. Bundle ~253.6 KB / ~77.6 KB gzipped (flat — pure CSS text edits).
- **Dan eyes-on smoke** (`npm run tauri dev`): Dan confirmed the new sizes feel right across empty state, loaded track, Advanced panel, preset shop, and export receipt. [If Dan adjusted any selector mid-loop, note the adjustment here.]

What failed or remains partial:

- **No automated typography regression test.** Vitest infra still deferred (HANDOFF infra #13).
- **No responsive breakpoints added.** Dan runs the app at a single resolution on a single monitor; if the app is ever opened on a small laptop screen, the 16 px base may need a media-query backstop. Out of scope for this slice.

Next recommended slice:

SVG preset icons (HANDOFF P1 #7). Plan path: `docs/superpowers/plans/2026-05-12-svg-preset-icons.md`.
```

---

- [ ] **Step 5.2: Commit + push**

```bash
cd "C:\Users\Daniel Kinsner\OneDrive\Documents\GitHub\album-mastering-studio-claude-build"
git status --short
```

Expected: `M docs/progress.md`, `M src/App.css`. If anything else shows modified, stop and investigate before committing.

```bash
git add docs/progress.md src/App.css
git commit -m "$(cat <<'EOF'
Phase 12.2 P1: typography pass — bump base + lift micro-labels

Dan: "UI overall could use larger text overall." Pure-CSS slice.

src/App.css:
- :root font-size 14px -> 16px. Because almost every selector uses
  rem units, this scales the entire UI proportionally (~1.143x).
- 14 micro-label selectors (0.65/0.7/0.72rem -> 0.78rem) so the
  smallest text lands at ~12.5 px instead of ~10.4 px after the
  base bump: .mode-pill, .mode-toggle button, .section-label,
  .track-badge, .live-update-badge, .clip-indicator,
  .analysis-summary > summary, .tag, .wf-hint, .tile-blurb,
  .user-preset-kind, .adv-label, .micro-btn, .check-level.
- Track-list left-rail and headings intentionally untouched.

Verification:
- npm run build: clean (~253.6 KB / ~77.6 KB gzipped, flat).
- Dan eyes-on smoke on `npm run tauri dev` confirmed sizes feel
  right across empty state, loaded track, Advanced panel,
  preset shop, and export receipt.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
git push origin master
```

Expected push output: one new commit pushed to `master`. Confirm the SHA is shown.

---

## Next slice (after this ships)

SVG preset icons — Dan's reference screenshot from the parallel Codex build had distinct icons per preset; adds visual hierarchy to the preset tile row. Plan path: `docs/superpowers/plans/2026-05-12-svg-preset-icons.md` (to be written when this slice ships).

---

## Self-Review Checklist (for the plan author)

After writing, the plan author checks:

1. **Brief coverage** — every concrete recommendation in the brief mapped to a task?
   - Base body 14 → 16 px: Step 2.1.
   - Slider labels in AdvancedPanel (11–12 → 13–14 px equivalent): `.adv-label` lift in Step 2.13 lands at 12.5 px and rides the base bump effects on its siblings (`.adv-value`, `.adv-control`).
   - StaleBar text (clipping indicator + live-update badge): Steps 2.6 and 2.7.
   - Number fields' digit text: explicitly addressed in the inventory (row 47: `.adv-number` stays at `0.78rem`, which is now the floor — matches the brief's "bump to match the new base" via the base bump itself).
   - Preset tile names: row 38 — base bump alone takes them to 13.6 px.
   - Track list left alone: rows 10 / 11 / 12 — all `keep` per brief.
   - Page title / primary CTAs untouched: rows 16, 18, 22, 55, 59 — all `keep`.

2. **Hard constraints honored.**
   - No `src/App.tsx` logic changes: Task 3 only re-confirms zero `fontSize` matches; no edits queued.
   - No new dependencies, fonts, font-families: none added in Task 2.
   - Pure CSS: only `src/App.css` is modified in Tasks 1–4.
   - No responsive breakpoints added: confirmed in the "what remains partial" note in Step 5.1.
   - Boundaries: no edits to `docs/PRODUCT.md`, `private-audio-fixtures/`, or anything Phase 12.2-guardrailed.

3. **Dan eyes-on smoke is a HARD GATE.** Step 4.2 is explicit: stop, ask Dan, loop until confirmed before Task 5 fires. The task structure literally cannot reach commit without it.

4. **No placeholders.** Search the plan for "TBD", "TODO", "fill in details", "implement later" — none present.

5. **Numeric consistency.** Every "lift" target is `0.78rem`. Every "keep" row is annotated with why. The base bump is `14px → 16px` everywhere it's mentioned (table row 1, Step 2.1, the progress entry, the commit message).

6. **Reversibility.** This slice can be reverted in one commit (`git revert`) without touching code paths or schema. That's the whole point of doing it as pure CSS.

---

*Plan ready for execution after `compression_density` ships.*
