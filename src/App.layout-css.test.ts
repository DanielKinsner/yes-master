import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/App.css"), "utf8");
const appTsx = readFileSync(resolve(process.cwd(), "src/App.tsx"), "utf8");
const albumPanelTsx = readFileSync(
  resolve(process.cwd(), "src/components/AlbumPanel.tsx"),
  "utf8",
);

function block(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = css.match(new RegExp(`${escaped}\\s*\\{([^}]*)\\}`, "m"));
  if (!match) throw new Error(`CSS block not found: ${selector}`);
  return match[1];
}

/// EVERY source block whose selector list starts with this exact selector —
/// for structural claims that must hold across all declarations, because
/// `block()` reads only the FIRST match and a later block can win the cascade
/// (that first-match blind spot is how the transparent TOOLS row went green;
/// audit U-01). Effective-style claims belong to the browser probe in
/// scripts/verify-app-headless.mjs, never to source text.
function blocks(selector: string): string[] {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const matches = [...css.matchAll(new RegExp(`${escaped}\\s*\\{([^}]*)\\}`, "gm"))];
  if (matches.length === 0) throw new Error(`CSS block not found: ${selector}`);
  return matches.map((m) => m[1]);
}

describe("console layout CSS", () => {
  it("aligns the preset tiles and signal chain on the same 8-column grid", () => {
    expect(block(".tile-row")).toContain(
      "grid-template-columns: repeat(auto-fit, minmax(104px, 1fr))",
    );
    expect(block(".signal-chain-track")).toContain("display: grid");
    expect(block(".signal-chain-track")).toContain(
      "grid-template-columns: repeat(8, minmax(0, 1fr))",
    );
    expect(block(".chain-link")).toContain("display: none");
  });

  it("keeps the preset header and save plus visually compact", () => {
    expect(block(".presets .section-label")).toContain("font-size: 0.72rem");
    expect(block(".preset-save-plus")).toContain("width: 18px");
    expect(block(".preset-save-plus")).toContain("height: 18px");
    expect(block(".preset-save-plus")).toContain("font-size: 0.72rem");
  });

  it("keeps Album Master's center column identical to Track Master's (Slice 13b)", () => {
    // Album chrome moved to the sidebar, so the album layer no longer forks
    // the workspace/console grid. That structural identity is what makes the
    // waveform card sit at the same Y in both modes (the done-criterion; the
    // pixel proof is the live-preview DOM measurement in the deviation log).
    // No is-album / workspace-album divergence may creep back.
    expect(appTsx).not.toContain("workspace-album");
    expect(appTsx).not.toContain("is-album");
    expect(css).not.toContain(".track-master-console.is-album");
    expect(css).not.toContain(".workspace-album");
    // One shared console grid still bounds the waveform deck in both modes.
    expect(block(".track-master-console")).toContain("overflow: hidden");
    expect(block(".track-master-console")).toContain("grid-template-rows");
    // Album identity + export receipt now render inside the sidebar.
    expect(appTsx).toContain("albumHeader={");
    expect(appTsx).toContain("<AlbumPanel");
    expect(appTsx).toContain("albumReceipt={");
    expect(appTsx).toContain("<AlbumExportReceipt");
    // The flow cluster survives as a stacked grid at rail width.
    expect(block(".album-panel-controls")).toContain("display: grid");
    expect(appTsx).toContain('exportMode={tm.mode === "album" ? "album" : "track"}');
  });

  it("keeps the metadata diet — each fact said once in its one home (Slice 13c)", () => {
    // Identity facts are one quiet line, not boxed chips (sidebar album
    // chips keep .meta-chip — that's a different scope, deliberately).
    expect(appTsx).toContain('className="track-meta-line"');
    expect(appTsx).not.toContain('className="track-meta-chips"');
    expect(block(".track-meta-line")).toContain("var(--text-2)");
    // The per-track "Analyzed" echo chip is gone — an unanalyzed track
    // surfaces via Source Check's "Awaiting analysis" row + busy pills.
    expect(appTsx).not.toContain('status-pill status-ok">Analyzed');
    // The footer speaks only while busy: no permanent Ready/Idle pill, no
    // Analyzed/Quality summary dots (quality lives in Source Check).
    expect(appTsx).not.toContain("StatusDot");
    expect(appTsx).not.toContain('"Quality —"');
    // Pass 2 (2026-08-19): the footer's processing pill is gone entirely —
    // it was the fourth copy of the analysis stage on one screen. The bar is
    // live meters only; the header SessionStatus pill owns coarse state.
    // (The SessionStatus pill and the output-settings dialog legitimately
    // keep their own "Ready" strings — different homes, different questions.)
    expect(appTsx).not.toContain("let processing: string | null = null;");
    expect(appTsx).not.toContain("status-processing-label");
    expect(appTsx).not.toContain('className="sidebar-status"');
  });

  it("keeps Album Master chrome compact and avoids inferred story chips", () => {
    expect(appTsx).not.toContain("<AlbumHeader");
    expect(appTsx).not.toContain("showStoryTags");
    expect(appTsx).not.toContain("StoryTags");
    expect(albumPanelTsx).toContain('className="album-panel-summary"');
    expect(albumPanelTsx).not.toContain("album-rate-select");
    expect(albumPanelTsx).not.toContain("album-depth-select");
    expect(albumPanelTsx).not.toContain("album-track-lane");
    expect(block(".album-panel-head")).toContain("display: grid");
    expect(block(".album-panel-summary")).toContain("align-items: baseline");
  });

  it("keeps live meters and delivery selects truthful at rail size", () => {
    expect(block(".master-readouts")).toContain(
      "grid-template-columns: repeat(3, minmax(0, 1fr))",
    );
    expect(block(".rail-card-select")).toContain("max-width: none");
    expect(block(".loudness-profile-select")).toContain("padding: 0.35rem 1.8rem");
    expect(appTsx).toContain("Live peak");
    expect(appTsx).toContain("Live LUFS");
    expect(appTsx).toContain("Preview LUFS");
    expect(appTsx).not.toContain("Export LUFS</span>");
  });

  it("keeps preview controls in the track header and leaves the waveform deck clean", () => {
    expect(appTsx).toContain('className="track-header-meta-row"');
    expect(appTsx).toContain('className="track-header-actions"');
    expect(appTsx).toContain("<SessionStatus");
    expect(appTsx).toContain('className="track-preview-toolbar"');
    expect(block(".track-header")).toContain("display: block");
    expect(block(".track-header-primary")).toContain(
      "grid-template-columns: minmax(0, 1fr) max-content",
    );
    expect(block(".track-header-actions")).toContain("justify-items: end");
    expect(css).toContain("--center-switch-width: 260px");
    expect(css).toContain("--center-switch-height: 38px");
    expect(block(".top-header-tabs")).toContain("width: var(--center-switch-width)");
    expect(block(".track-toolbar-group-compare")).toContain(
      "width: var(--center-switch-width)",
    );
    expect(block(".top-tab")).toContain("min-height: var(--center-switch-height)");
    expect(block(".ab-toggle button")).toContain(
      "min-height: var(--center-switch-height)",
    );
    expect(block(".track-preview-toolbar")).toContain("display: flex");
    expect(block(".track-preview-toolbar")).toContain("justify-content: flex-end");
    expect(block(".wf-deck-transport")).toContain("grid-column: 1");
    expect(block(".wf-deck-transport")).toContain("grid-row: 1");
    expect(block(".wf-card")).toContain("grid-column: 2");
    expect(block(".wf-card")).toContain("grid-row: 1");
    expect(block(".wf-deck-meters")).toContain("grid-column: 3");
    expect(block(".wf-deck-meters")).toContain("grid-row: 1");
  });

  it("keeps readiness out of the old footer strip", () => {
    expect(appTsx).toContain("session-status");
    expect(appTsx).not.toContain("StaleBar");
    expect(appTsx).not.toContain("liveUpdateStats.applied");
    expect(appTsx).not.toContain("<UndoRedoTools");
    expect(css).not.toContain(".console-footer-row");
    expect(css).not.toContain(".undo-redo-bar");
    expect(css).not.toContain(".stale-bar");
    expect(css).not.toContain(".live-update-badge");
    expect(block(".session-status")).toContain("border-radius: 999px");
    expect(block(".track-master-console")).not.toContain("38px");
  });

  it("keeps the console insight compact enough for the metadata rail", () => {
    expect(css).toContain(
      "flex-wrap: nowrap",
    );
    expect(css).toContain(
      "grid-template-columns: 86px minmax(0, 1fr) 174px",
    );
    expect(block(".source-insight-headline")).toContain("white-space: nowrap");
  });

  it("keeps manual compressor knobs compact inside the right rail", () => {
    expect(block(".compressor-knob-grid")).toContain("gap: 0.35rem");
    expect(block(".compressor-knob-grid")).toContain("padding: 0.32rem");
    expect(block(".compressor-knob-grid .knob-vis")).toContain("transform: scale(0.9)");
  });

  it("keeps Standard's center and rail on one shared column gap (seam alignment)", () => {
    // lib/rail-alignment.ts derives Delivery's top from Preview bottom + the
    // rail gap while Loudness's top is Intensity bottom + the steps gap. The
    // two gaps must be the SAME variable or the second seam silently drifts.
    expect(block(".standard-view")).toContain("--std-col-gap");
    expect(block(".std-steps")).toContain("gap: var(--std-col-gap)");
    expect(block(".std-rail")).toContain("gap: var(--std-col-gap)");
    expect(block(".std-center")).toContain("gap: var(--std-col-gap)");
  });

  it("keeps Standard center from becoming its own scroll region", () => {
    expect(block(".std-center")).toContain("overflow: hidden");
    expect(block(".std-center")).not.toContain("scrollbar-gutter");
  });

  it("keeps Standard rail cards and CTA on one shared left/right edge", () => {
    expect(block(".std-rail-card")).toContain("width: 100%");
    expect(css).toContain(".std-rail-export { width: 100%");
    expect(block("button.primary.std-create-master")).toContain("width: 100%");
  });

  it("lets Standard's center column compress instead of scrolling on short viewports", () => {
    // The owner wants no center scroll in any mode — the tall fixed minimums
    // must stay height-fluid even though seam alignment can now remeasure
    // against the current scroll position.
    expect(block(".std-wave-deck")).toContain("min-height: clamp(");
    expect(block(".std-tile")).toContain("min-height: clamp(");
    expect(block(".std-steps .std-step-intensity")).toContain("min-height: clamp(");
    expect(block(".std-tile-icon")).toContain("9vh");
    expect(block(".std-rail .master-out .lufs-meter")).toContain("min-height: 112px");
  });

  // Both of these were found by actually LOOKING at the 1360x740 screenshots
  // the headless lane produces, after every automated assertion on that
  // viewport had passed. No-overflow was green, reachability was green, and
  // the window still looked broken — which is the argument for the visual pass
  // existing at all.
  it("clamps the workspace title without depending on window HEIGHT", () => {
    // The clamp used to live ONLY inside `@media (min-width: 1280px) and
    // (min-height: 820px)`. The supported minimum is 1360x740, so the height
    // condition failed and the clamp switched off exactly where it was needed:
    // a long album filename wrapped to eight lines and pushed the transport
    // and waveform off the top of the workspace.
    const clamp = block(".console-hero .track-title");
    expect(clamp).toContain("white-space: nowrap");
    expect(clamp).toContain("text-overflow: ellipsis");
    expect(clamp).toContain("overflow: hidden");

    // And it must be declared OUTSIDE any height-gated media query. Asserting
    // the properties alone would pass again the moment someone moves the rule
    // back inside one, which is exactly how this regressed the first time.
    const declaration = css.indexOf(".console-hero .track-title");
    const heightGate = css.indexOf("@media (min-width: 1280px) and (min-height: 820px)");
    expect(declaration).toBeGreaterThan(-1);
    expect(heightGate).toBeGreaterThan(-1);
    expect(declaration).toBeLessThan(heightGate);
  });

  it("keeps the sticky export cluster opaque above the TOOLS row", () => {
    // The mask used to run 0% transparent -> 95% at 35%, and the TOOLS row
    // sits inside that first 35%. At 1360x740 the Delivery Format card
    // scrolled straight through it: "DELIVERY FORMAT" rendered on top of
    // "TOOLS". Full opacity has to be reached ABOVE that row.
    const group = block(".right-rail-export-group");
    expect(group).toContain("var(--bg-1) 18%");
    expect(group).not.toContain("rgba(17, 21, 31, 0.95) 35%");

    // Belt and braces: no .right-rail-tools source block may declare a
    // transparent background. This is a STRUCTURAL tripwire only — the first
    // block being opaque proved nothing when a later unconditional block won
    // the cascade with `transparent` (audit U-01). The authoritative check is
    // the browser-computed alpha probe in verify-app-headless.mjs
    // (clean-tools-overlap), because specificity, media conditions, and
    // source order all participate in what actually paints.
    for (const toolsBlock of blocks(".right-rail-tools")) {
      expect(toolsBlock).not.toContain("background: transparent");
    }
  });

  it("keeps muted text (--text-2) at WCAG AA contrast on its lightest ground (audit A-02 / review #2)", () => {
    // The axe lane measures the live page; this computes the same ratio from
    // the tokens so a palette edit fails fast without a browser run. --text-2
    // labels small text (TOOLS summary, receipt path/blurb), so the 4.5:1
    // small-text bar applies. The worst case is the LIGHTEST ground it paints
    // on: --bg-2 — the receipt's .receipt-file-open / .receipt-style panels
    // (the first version of this test measured --bg-1 and let a 4.52:1
    // borderline through; the live lane then failed on --bg-2 — review #2).
    const token = (name: string): string => {
      const match = css.match(new RegExp(`${name}:\\s*#([0-9a-fA-F]{6})`));
      if (!match) throw new Error(`token ${name} not found as a 6-digit hex`);
      return match[1];
    };
    const luminance = (hex: string): number => {
      const channel = (index: number) => {
        const c = parseInt(hex.slice(index, index + 2), 16) / 255;
        return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
      };
      return (
        0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
      );
    };
    const l1 = luminance(token("--text-2"));
    const l2 = luminance(token("--bg-2"));
    const ratio =
      (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
    // 4.8 not 4.5: the live lane samples browser-composited pixels, so a
    // ratio that only just clears AA in token math can still fail there
    // (that is exactly how 4.52:1 shipped and then failed). The margin
    // keeps token edits honestly clear of the line, not camped on it.
    expect(ratio).toBeGreaterThanOrEqual(4.8);
  });

  it("keeps first-run hints out of the Standard Preview rail", () => {
    expect(block(".first-run-overlay")).toContain("pointer-events: none");
    expect(block(".first-run-overlay .hint-chip")).toContain("pointer-events: auto");
    expect(block(".first-run-overlay-flip")).toContain(
      "right: calc(var(--rail-width-right) + 24px)",
    );
    expect(block(".first-run-overlay-advanced")).toContain(
      "right: calc(var(--rail-width-right) + 24px)",
    );
  });
});
