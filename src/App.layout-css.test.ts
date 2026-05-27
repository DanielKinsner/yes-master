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

  it("keeps the header transport controls from overlapping the insight summary", () => {
    expect(block(".track-header")).toContain("display: grid");
    expect(block(".track-header")).toContain(
      "grid-template-columns: minmax(0, 1fr) max-content",
    );
    expect(block(".track-header-controls")).toContain("min-width: max-content");
  });

  it("keeps the console insight card narrow enough for playback controls", () => {
    expect(css).toContain(
      "grid-template-columns: minmax(220px, 1fr) minmax(220px, 340px)",
    );
    expect(css).toContain(
      "grid-template-columns: minmax(300px, 1fr) minmax(260px, 360px)",
    );
    expect(block(".analysis-summary-headline")).toContain("font-size: 0.78rem");
  });

  it("keeps manual compressor knobs compact inside the right rail", () => {
    expect(block(".compressor-knob-grid")).toContain("gap: 0.35rem");
    expect(block(".compressor-knob-grid")).toContain("padding: 0.32rem");
    expect(block(".compressor-knob-grid .knob-vis")).toContain("transform: scale(0.9)");
  });
});
