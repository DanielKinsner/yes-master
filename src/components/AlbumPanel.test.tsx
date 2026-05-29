import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AlbumPanel } from "./AlbumPanel";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

function baseProps() {
  return {
    tracks: [],
    albumArcKind: "cinematic" as const,
    albumIntensity: 1.0,
    albumTitle: "",
    albumRendering: false,
    albumExportReport: null,
    albumSampleRate: null,
    albumBitDepth: null,
    onAlbumArc: vi.fn(),
    onAlbumIntensity: vi.fn(),
    onAlbumTitle: vi.fn(),
    onExportAlbum: vi.fn(),
    onAlbumSampleRate: vi.fn(),
    onAlbumBitDepth: vi.fn(),
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

describe("AlbumPanel delivery format", () => {
  it("renders sample-rate and bit-depth selects defaulting to Auto", async () => {
    const { container } = await renderNode(<AlbumPanel {...baseProps()} />);
    const rate = container.querySelector(
      "#album-rate-select",
    ) as HTMLSelectElement | null;
    const depth = container.querySelector(
      "#album-depth-select",
    ) as HTMLSelectElement | null;
    expect(rate?.value).toBe("auto");
    expect(depth?.value).toBe("auto");
  });

  it("calls onAlbumSampleRate with a number when a rate is picked", async () => {
    const props = baseProps();
    const { container } = await renderNode(<AlbumPanel {...props} />);
    const rate = container.querySelector(
      "#album-rate-select",
    ) as HTMLSelectElement;
    await act(async () => {
      rate.value = "44100";
      rate.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(props.onAlbumSampleRate).toHaveBeenCalledWith(44100);
  });

  it("calls onAlbumBitDepth with null when Auto is reselected", async () => {
    const props = { ...baseProps(), albumBitDepth: 24 };
    const { container } = await renderNode(<AlbumPanel {...props} />);
    const depth = container.querySelector(
      "#album-depth-select",
    ) as HTMLSelectElement;
    await act(async () => {
      depth.value = "auto";
      depth.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(props.onAlbumBitDepth).toHaveBeenCalledWith(null);
  });
});
