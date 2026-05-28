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

describe("console layout CSS", () => {
  it("aligns the preset tiles and signal chain on the same 8-column grid", () => {
    expect(block(".tile-row")).toContain(
      "grid-template-columns: repeat(auto-fit, minmax(90px, 1fr))",
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

  it("gives Album Master its own workspace and console grid rows", () => {
    expect(appTsx).toContain('tm.mode === "album" ? " workspace-album" : ""');
    expect(appTsx).toContain('tm.mode === "album" ? " is-album" : ""');
    expect(block(".workspace-album")).toContain(
      "grid-template-rows: auto minmax(0, 1fr)",
    );
    expect(block(".track-master-console")).toContain("overflow: hidden");
    expect(block(".track-master-console.is-album")).toContain("46px");
    expect(block(".track-master-console.is-album")).toContain(
      "minmax(288px, 1.3fr)",
    );
    expect(block(".track-master-console.is-album")).toContain(
      "minmax(162px, 0.72fr)",
    );
    expect(block(".album-panel-controls")).toContain("display: grid");
    expect(block(".album-export-btn")).toContain("white-space: nowrap");
  });

  it("keeps Album Master chrome compact and avoids inferred story chips", () => {
    expect(appTsx).not.toContain("<AlbumHeader");
    expect(appTsx).not.toContain("showStoryTags");
    expect(appTsx).not.toContain("StoryTags");
    expect(albumPanelTsx).toContain('className="album-panel-summary"');
    expect(albumPanelTsx).toContain('className="section-label album-panel-mode"');
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
    expect(block(".analysis-summary-headline")).toContain("white-space: nowrap");
  });

  it("keeps manual compressor knobs compact inside the right rail", () => {
    expect(block(".compressor-knob-grid")).toContain("gap: 0.35rem");
    expect(block(".compressor-knob-grid")).toContain("padding: 0.32rem");
    expect(block(".compressor-knob-grid .knob-vis")).toContain("transform: scale(0.9)");
  });
});
