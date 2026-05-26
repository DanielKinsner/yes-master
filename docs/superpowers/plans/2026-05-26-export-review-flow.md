# Export Review Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a warning-aware export flow so YES Master shows `Export With Review` when current quality rows are not clean, opens a review panel, and lets the user choose `Adjust Settings` or `Export Anyway`.

**Architecture:** Keep this slice in the frontend. `RightRail` already owns source/export quality rows and the export button, so add a small review-state UI there and avoid changing Rust render/export logic. Use existing `QualityCheck` data when available and source-derived preflight rows when no export receipt exists.

**Tech Stack:** React 19, TypeScript, Vitest, existing `RightRail` component tests, Tauri command layer unchanged.

---

## Scope

This plan implements the first stabilization slice only: export review UX. It does not implement compressor modes, fixture matrices, realtime diagnostic cleanup, or DSP changes.

The intended behavior:

- If current quality rows are all clean, the primary button reads `Export Master`.
- If any current quality row is warning or critical, the primary button reads `Export With Review`.
- First click on `Export With Review` does not render; it opens an inline review panel.
- `Adjust Settings` closes the review panel and does not export.
- `Export Anyway` calls the existing `onExport`.
- `Exporting...` continues to display while an export is in progress.
- Disabled behavior for no analysis, rendering, and exporting stays intact.

## Files

- Modify: `src/components/RightRail.tsx`
  - Export a reusable quality-row type.
  - Reuse one row derivation path for Quality Check and export review state.
  - Add local review panel state.
  - Add warning-aware button label and handlers.

- Modify: `src/components/RightRail.test.tsx`
  - Add tests for clean export button, warning review button, review panel, `Adjust Settings`, `Export Anyway`, and disabled states.

- Modify: `src/App.album-export.test.tsx`
  - Add one App-level test proving `RightRail` integration labels the export button as `Export With Review` when the selected source analysis has review-level metrics.

- Optional modify: `src/App.css`
  - Add compact styles for `.export-review-panel`, `.export-review-list`, `.export-review-actions`, `.export-review-row`.
  - Keep styling small and consistent with existing right-rail panels.

## Current Code Landmarks

- `src/components/RightRail.tsx`
  - `RightRail` renders the export button near the top of the file.
  - `QualityCheckPanel` currently derives rows internally from either `lastChecks` or `analysis`.
  - `derivePreflightChecks` already marks source true peak, loudness, and dynamic range as ok/warn/crit.
  - `friendlyCheckLabel` already converts backend export check codes into short labels.

- `src/components/RightRail.test.tsx`
  - Already has a `HOT_SOURCE_ANALYSIS` fixture that produces review rows.
  - Already renders `RightRail` directly.

- `src/App.album-export.test.tsx`
  - Mocks `useTrackMaster`.
  - Good place for one app-integration test after component tests are green.

---

### Task 1: Add Direct RightRail Tests For Button Labels

**Files:**
- Modify: `src/components/RightRail.test.tsx`

- [ ] **Step 1: Add a clean source-analysis fixture**

Add this below `HOT_SOURCE_ANALYSIS`:

```ts
const CLEAN_SOURCE_ANALYSIS: AnalysisResult = {
  ...HOT_SOURCE_ANALYSIS,
  true_peak_dbtp: -1.2,
  lufs_integrated: -14.0,
  dynamic_range_lu: 8.0,
};
```

- [ ] **Step 2: Add tests for clean vs review button labels**

Add these tests inside `describe("RightRail source checks", () => { ... })`:

```tsx
  it("keeps the primary action as Export Master when current checks are clean", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={CLEAN_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Master",
    );

    expect(exportButton).toBeTruthy();

    await act(async () => {
      root.unmount();
    });
  });

  it("labels the primary action Export With Review when source checks warn", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    );

    expect(exportButton).toBeTruthy();
    expect(onExport).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });
```

- [ ] **Step 3: Run the focused test and verify it fails**

Run:

```powershell
npm test -- src/components/RightRail.test.tsx
```

Expected: the clean-label test passes or nearly passes, and the warning-label test fails because the current button still says `Export Master`.

- [ ] **Step 4: Commit the failing tests**

Do not commit if unrelated files are dirty.

```powershell
git add src/components/RightRail.test.tsx
git commit -m "test: cover export review button label"
```

---

### Task 2: Refactor Quality Rows For Reuse

**Files:**
- Modify: `src/components/RightRail.tsx`

- [ ] **Step 1: Add a reusable type near the imports**

Add below the `RightRailProps` type:

```ts
type QualityRow = {
  key: string;
  ok: boolean;
  warn: boolean;
  crit: boolean;
  label: string;
  detail: string;
};
```

- [ ] **Step 2: Add reusable helpers above `QualityCheckPanel`**

Add this above `function QualityCheckPanel`:

```ts
function qualityRowsFor(
  checks: QualityCheck[] | undefined,
  analysis: AnalysisResult | undefined,
): QualityRow[] {
  return checks && checks.length > 0
    ? checks.map((c, i) => ({
        key: `${c.code}-${i}`,
        ok: c.level === "info",
        warn: c.level === "warning",
        crit: c.level === "critical",
        label: friendlyCheckLabel(c),
        detail: c.message,
      }))
    : derivePreflightChecks(analysis);
}

function hasReviewRows(rows: QualityRow[]): boolean {
  return rows.some((row) => row.warn || row.crit);
}
```

- [ ] **Step 3: Update `QualityCheckPanel` to use the helper**

Replace the current `rows` assignment in `QualityCheckPanel`:

```ts
  const rows = checks && checks.length > 0
    ? checks.map((c, i) => ({
        key: `${c.code}-${i}`,
        ok: c.level === "info",
        warn: c.level === "warning",
        crit: c.level === "critical",
        label: friendlyCheckLabel(c),
        detail: c.message,
      }))
    : derivePreflightChecks(analysis);
```

with:

```ts
  const rows = qualityRowsFor(checks, analysis);
```

- [ ] **Step 4: Change `derivePreflightChecks` return type**

Replace the inline return type:

```ts
function derivePreflightChecks(analysis: AnalysisResult | undefined): {
  key: string;
  ok: boolean;
  warn: boolean;
  crit: boolean;
  label: string;
  detail: string;
}[] {
```

with:

```ts
function derivePreflightChecks(analysis: AnalysisResult | undefined): QualityRow[] {
```

- [ ] **Step 5: Run tests and verify existing behavior still passes except new label test**

Run:

```powershell
npm test -- src/components/RightRail.test.tsx
```

Expected: still failing on the new `Export With Review` label test only.

---

### Task 3: Add Warning-Aware Button Label And Click Gate

**Files:**
- Modify: `src/components/RightRail.tsx`

- [ ] **Step 1: Import `useEffect` and `useState`**

Replace:

```ts
import type { ReactNode } from "react";
```

with:

```ts
import { useEffect, useState, type ReactNode } from "react";
```

- [ ] **Step 2: Add row/review state inside `RightRail`**

Inside `RightRail`, after props are destructured and before `return`, add:

```ts
  const qualityRows = qualityRowsFor(lastChecks, analysis);
  const needsReview = canExport && hasReviewRows(qualityRows);
  const [reviewOpen, setReviewOpen] = useState(false);

  useEffect(() => {
    setReviewOpen(false);
  }, [analysis?.track_id, lastChecks]);

  const exportLabel = isExporting
    ? "Exporting..."
    : needsReview
    ? "Export With Review"
    : "Export Master";

  const handlePrimaryExport = () => {
    if (!canExport || isExporting || isRendering) return;
    if (needsReview) {
      setReviewOpen(true);
      return;
    }
    onExport();
  };

  const handleExportAnyway = () => {
    setReviewOpen(false);
    onExport();
  };
```

- [ ] **Step 3: Wire the primary button to the new handler and label**

Replace:

```tsx
          onClick={onExport}
```

with:

```tsx
          onClick={handlePrimaryExport}
```

Replace:

```tsx
          {isExporting ? "Exporting…" : "Export Master"}
```

with:

```tsx
          {exportLabel}
```

Use three periods in `Exporting...` for the first implementation so the test can match ASCII. If the codebase prefers the ellipsis glyph later, update tests and UI text together.

- [ ] **Step 4: Add the inline review panel below the primary button**

Immediately after the primary export button, add:

```tsx
        {reviewOpen && needsReview && (
          <section className="export-review-panel" aria-label="Export review">
            <header className="export-review-head">
              <span className="export-review-title">Review before export</span>
              <span className="quality-badge badge-warn">REVIEW</span>
            </header>
            <ul className="export-review-list">
              {qualityRows
                .filter((row) => row.warn || row.crit)
                .map((row) => (
                  <li
                    key={row.key}
                    className={
                      "export-review-row " + (row.crit ? "is-crit" : "is-warn")
                    }
                    title={row.detail}
                  >
                    <span className="quality-check-glyph" aria-hidden>
                      {row.crit ? "✗" : "△"}
                    </span>
                    <span>{row.label}</span>
                  </li>
                ))}
            </ul>
            <div className="export-review-actions">
              <button
                type="button"
                className="ghost-btn"
                onClick={() => setReviewOpen(false)}
              >
                Adjust Settings
              </button>
              <button
                type="button"
                className="primary"
                onClick={handleExportAnyway}
              >
                Export Anyway
              </button>
            </div>
          </section>
        )}
```

- [ ] **Step 5: Run the focused test**

Run:

```powershell
npm test -- src/components/RightRail.test.tsx
```

Expected: Task 1 tests pass.

- [ ] **Step 6: Commit**

```powershell
git add src/components/RightRail.tsx src/components/RightRail.test.tsx
git commit -m "feat: label risky exports for review"
```

---

### Task 4: Add Review Panel Interaction Tests

**Files:**
- Modify: `src/components/RightRail.test.tsx`

- [ ] **Step 1: Import no new test helpers**

Use existing `act`, `renderNode`, and plain DOM events. Do not add Testing Library for this slice.

- [ ] **Step 2: Add an interaction test for opening review and adjusting settings**

Add inside `describe("RightRail source checks", () => { ... })`:

```tsx
  it("opens review instead of exporting when warnings are present", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;

    expect(exportButton).toBeTruthy();

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).not.toHaveBeenCalled();
    expect(container.textContent).toContain("Review before export");
    expect(container.textContent).toContain("Source true peak 0.2 dBTP");
    expect(container.textContent).toContain("Source dynamic range 3.3 LU");

    const adjustButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Adjust Settings",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      adjustButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).not.toHaveBeenCalled();
    expect(container.textContent).not.toContain("Review before export");

    await act(async () => {
      root.unmount();
    });
  });
```

- [ ] **Step 3: Add an interaction test for Export Anyway**

Add inside the same describe block:

```tsx
  it("exports from the review panel when the user chooses Export Anyway", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const anywayButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Anyway",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      anywayButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.unmount();
    });
  });
```

- [ ] **Step 4: Add a clean-path click test**

Add inside the same describe block:

```tsx
  it("exports immediately when current checks are clean", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={CLEAN_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Master",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).toHaveBeenCalledTimes(1);
    expect(container.textContent).not.toContain("Review before export");

    await act(async () => {
      root.unmount();
    });
  });
```

- [ ] **Step 5: Run the focused test**

Run:

```powershell
npm test -- src/components/RightRail.test.tsx
```

Expected: all `RightRail` tests pass.

- [ ] **Step 6: Commit**

```powershell
git add src/components/RightRail.test.tsx
git commit -m "test: cover export review confirmation"
```

---

### Task 5: Style The Review Panel

**Files:**
- Modify: `src/App.css`

- [ ] **Step 1: Find the right-rail export styles**

Run:

```powershell
rg -n "right-rail-export|quality-check|right-rail-tools" src/App.css
```

Expected: line numbers for the current right-rail export and quality-check CSS.

- [ ] **Step 2: Add compact review styles near the right-rail export styles**

Add:

```css
.export-review-panel {
  margin-top: 10px;
  padding: 12px;
  border: 1px solid rgba(248, 113, 113, 0.35);
  border-radius: 8px;
  background: rgba(127, 29, 29, 0.16);
}

.export-review-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 10px;
}

.export-review-title {
  color: var(--text);
  font-size: 0.82rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.export-review-list {
  display: grid;
  gap: 7px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.export-review-row {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--muted);
  font-size: 0.86rem;
}

.export-review-row.is-crit {
  color: #fecaca;
}

.export-review-row.is-warn {
  color: #fed7aa;
}

.export-review-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 12px;
}

.export-review-actions .primary,
.export-review-actions .ghost-btn {
  width: 100%;
}
```

If `--text` or `--muted` are not defined in `src/App.css`, use the existing local text color variables from nearby right-rail styles. Do not introduce a new palette.

- [ ] **Step 3: Run CSS/layout tests**

Run:

```powershell
npm test -- src/App.layout-css.test.ts src/components/RightRail.test.tsx
```

Expected: all selected tests pass.

- [ ] **Step 4: Commit**

```powershell
git add src/App.css
git commit -m "style: add export review panel"
```

---

### Task 6: Add App-Level Integration Coverage

**Files:**
- Modify: `src/App.album-export.test.tsx`

- [ ] **Step 1: Import `AnalysisResult` and `MasteringSettings` types**

Replace:

```ts
import type { ImportedTrack, QualityCheck, RenderJob } from "./bindings";
```

with:

```ts
import type {
  AnalysisResult,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "./bindings";
```

- [ ] **Step 2: Add settings and hot analysis fixtures below `track`**

Add:

```ts
const settings: MasteringSettings = {
  preset: { kind: "universal" },
  intensity: 0.5,
  eq_sub_db: 0,
  eq_low_db: 0,
  eq_low_mid_db: 0,
  eq_mid_db: 0,
  eq_high_mid_db: 0,
  eq_high_db: 0,
  eq_sparkle_db: 0,
  volume_match: false,
  input_gain_db: 0,
  output_gain_db: 0,
  delivery_profile: "streaming-universal",
  advanced: {
    lufs_offset_db: null,
    ceiling_dbtp: null,
    width: null,
    warmth: null,
    presence_air: null,
    compression_density: null,
    compression_low_threshold_db: null,
    compression_low_ratio: null,
    compression_low_attack_ms: null,
    compression_low_release_ms: null,
    compression_mid_threshold_db: null,
    compression_mid_ratio: null,
    compression_mid_attack_ms: null,
    compression_mid_release_ms: null,
    compression_high_threshold_db: null,
    compression_high_ratio: null,
    compression_high_attack_ms: null,
    compression_high_release_ms: null,
    compression_link_stereo: null,
    bit_depth: null,
    target_sample_rate: null,
  },
};

const hotAnalysis: AnalysisResult = {
  track_id: track.id,
  lufs_integrated: -10.5,
  lufs_short_term_max: -8.8,
  true_peak_dbtp: 0.2,
  dynamic_range_lu: 3.3,
  spectral_balance: { low: 0.3, mid: 0.4, high: 0.3 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: settings,
  measured_at_iso: "2026-05-26T00:00:00.000Z",
  inferred_role: null,
  role_confidence: null,
  inferred_character: null,
  character_confidence: null,
  spectral_balance_6band: null,
  transient_flux: null,
  stereo_correlation: null,
  dynamic_range_p95_p10_db: null,
  lufs_short_term_max_3s: null,
  energy_density_score: null,
};
```

- [ ] **Step 3: Add an App-level test**

Add inside `describe("album export actions", () => { ... })`:

```tsx
  it("surfaces Export With Review from the app shell when selected source checks warn", async () => {
    mocks.tm = {
      ...baseTrackMasterState(),
      mode: "track",
      selectedTrackId: track.id,
      selectedTrack: track,
      selectedAnalysis: hotAnalysis,
      selectedSettings: settings,
    };

    const { container, root } = await renderApp();

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    );

    expect(exportButton).toBeTruthy();

    await act(async () => {
      root.unmount();
    });
  });
```

- [ ] **Step 4: Run the App export tests**

Run:

```powershell
npm test -- src/App.album-export.test.tsx
```

Expected: all selected tests pass.

- [ ] **Step 5: Commit**

```powershell
git add src/App.album-export.test.tsx
git commit -m "test: cover app export review integration"
```

---

### Task 7: Full Frontend Verification

**Files:**
- No file changes expected.

- [ ] **Step 1: Run all frontend tests**

Run:

```powershell
npm test
```

Expected:

```text
Test Files  14 passed
Tests  100+ passed
```

The exact count may increase from 99 because this plan adds tests. Failures in RightRail/App export tests should be fixed before continuing.

- [ ] **Step 2: Run production build**

Run:

```powershell
npm run build
```

Expected: TypeScript build and Vite build complete successfully.

- [ ] **Step 3: Commit verification-only changes only if a test snapshot or generated committed file changed**

No commit is expected for this task unless the prior task missed a file.

---

### Task 8: Manual Desktop Smoke

**Files:**
- No file changes expected unless manual smoke reveals a bug.

- [ ] **Step 1: Start the app**

Run:

```powershell
npm run tauri dev
```

Expected: YES Master opens in a desktop window.

- [ ] **Step 2: Verify no-analysis state**

Without loading a track:

- Primary export button is disabled.
- Button text is `Export With Review` if the source check panel shows awaiting-analysis review, or `Export Master` if implementation chooses to suppress review labels until `canExport` is true.
- Disabled title still communicates `Analyze a track first.`

Preferred behavior for this slice: because `needsReview` includes `canExport`, no-analysis state should show disabled `Export Master`.

- [ ] **Step 3: Verify warning state**

Load a track whose source analysis produces review rows, such as true peak above -1.0 dBTP or dynamic range below 6 LU.

Expected:

- Quality panel badge says `REVIEW`.
- Primary button says `Export With Review`.
- Clicking it opens the review panel.
- The panel lists the same warning labels visible in Quality Check.
- Clicking `Adjust Settings` closes the review panel and does not open a save dialog.
- Clicking `Export With Review` again and then `Export Anyway` opens the existing save/export flow.

- [ ] **Step 4: Verify clean state**

Load or synthesize a source whose analysis rows are clean.

Expected:

- Quality panel badge says `SAFE`.
- Primary button says `Export Master`.
- Clicking it opens the existing save/export flow immediately.

- [ ] **Step 5: Stop the dev server**

Close the app window and stop the terminal process with `Ctrl+C`.

---

### Task 9: Final Gate And Commit

**Files:**
- All files modified by this slice.

- [ ] **Step 1: Run final commands**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test --lib
```

Expected:

- Frontend tests pass.
- Production build passes.
- Rust lib tests still pass.

Full `cargo test` is optional for this UI-only slice, but run it if any shared type or binding file changed.

- [ ] **Step 2: Check git status**

Run:

```powershell
git status --short
```

Expected: only intended files are modified.

- [ ] **Step 3: Final commit**

Use a single final commit only if prior task commits were not made. If prior task commits exist, skip this step.

```powershell
git add src/components/RightRail.tsx src/components/RightRail.test.tsx src/App.album-export.test.tsx src/App.css
git commit -m "feat: add export review flow"
```

---

## Self-Review Checklist

- Spec coverage:
  - Warning button label is covered by Task 1 and Task 3.
  - Review panel is covered by Task 3 and Task 4.
  - Adjust Settings behavior is covered by Task 4.
  - Export Anyway behavior is covered by Task 4.
  - App-level integration is covered by Task 6.
  - Styling is covered by Task 5.
  - Full verification is covered by Task 7 through Task 9.

- Placeholder scan:
  - This plan names the concrete files, helpers, tests, commands, and expected behaviors for every task.

- Type consistency:
  - `QualityRow`, `qualityRowsFor`, and `hasReviewRows` are defined before use.
  - `RightRail` props are unchanged.
  - Existing `QualityCheck` and `AnalysisResult` binding types are reused.

## Notes For The Implementing Agent

- Do not change Rust export behavior in this slice.
- Do not add compressor mode in this slice.
- Do not remove diagnostic counters in this slice.
- Keep source warning review advisory, not blocking.
- If a critical row exists, still allow `Export Anyway` because the current product canon says quality checks are advisory when the export is technically possible. A later slice can decide whether specific critical technical failures should bypass this UI and hard-stop before render.
