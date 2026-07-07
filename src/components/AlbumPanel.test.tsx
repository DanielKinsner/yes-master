import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AlbumPanel } from "./AlbumPanel";
import type { ImportedTrack } from "../bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

function track(overrides: Partial<ImportedTrack> = {}): ImportedTrack {
  return {
    id: "t1",
    path: "C:/audio/t1.wav",
    display_name: "Track One",
    source_format: "wav",
    duration_seconds: 180,
    sample_rate: 44_100,
    channels: 2,
    ...overrides,
  };
}

function baseProps() {
  return {
    tracks: [],
    albumArcKind: "cinematic" as const,
    albumIntensity: 1.0,
    albumTitle: "",
    onAlbumArc: vi.fn(),
    onAlbumIntensity: vi.fn(),
    onAlbumTitle: vi.fn(),
  };
}

async function renderNode(node: ReactNode): Promise<{
  container: HTMLDivElement;
  root: Root;
}> {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(node);
  });
  return { container, root };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("AlbumPanel", () => {
  it("renders compact album flow controls without export format fields", async () => {
    const { container } = await renderNode(<AlbumPanel {...baseProps()} />);
    expect(container.textContent).toContain("Album flow");
    expect(container.textContent).toContain("Flow amount");
    expect(container.querySelector("#album-rate-select")).toBeNull();
    expect(container.querySelector("#album-depth-select")).toBeNull();
    expect(container.textContent).not.toContain("Export Album");
  });

  it("renders the title as an in-place heading with a dim prompt (Slice 13)", async () => {
    const { container } = await renderNode(<AlbumPanel {...baseProps()} />);
    const input = container.querySelector<HTMLInputElement>(".album-title-input");
    expect(input).toBeTruthy();
    // Kept as an <input> for a11y + edit-in-place; prompt, not a boxed field.
    expect(input?.tagName).toBe("INPUT");
    expect(input?.getAttribute("placeholder")).toBe("Name this album");
    expect(input?.getAttribute("aria-label")).toBe("Album title");
    expect(input?.getAttribute("maxlength")).toBe("120");
  });

  it("renders album stats as metadata chips and fixes 1-track pluralization (Slice 13)", async () => {
    const one = await renderNode(
      <AlbumPanel {...baseProps()} tracks={[track({ duration_seconds: 65 })]} />,
    );
    const chips = Array.from(
      one.container.querySelectorAll(".album-panel-chips .meta-chip"),
    ).map((c) => c.textContent);
    expect(chips).toContain("1 track");
    expect(chips).not.toContain("1 tracks");
    // Duration renders as its own chip (M:SS) next to the count.
    expect(chips).toContain("1:05");

    const many = await renderNode(
      <AlbumPanel
        {...baseProps()}
        tracks={[track({ id: "a" }), track({ id: "b" }), track({ id: "c" })]}
      />,
    );
    const manyChips = Array.from(
      many.container.querySelectorAll(".album-panel-chips .meta-chip"),
    ).map((c) => c.textContent);
    expect(manyChips).toContain("3 tracks");
  });
});
