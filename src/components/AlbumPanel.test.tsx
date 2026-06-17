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

  it("shows the rendered delivery format and upsample advisory", async () => {
    const props = {
      ...baseProps(),
      albumExportReport: {
        album_wav_path: "C:/Masters/album_continuous.wav",
        manifest_path: "C:/Masters/manifest.json",
        requested_sample_rate: null,
        rendered_sample_rate: 48_000,
        source_sample_rates: [44_100, 48_000],
        bit_depth: 24,
        rendered_channels: 2,
        source_channels: [1, 2],
        tracks: [
          {
            track_id: "track-1",
            position: 1,
            output_path: "C:/Masters/01-track.wav",
            measured_lufs: -14,
            source_sample_rate: 44_100,
            rendered_sample_rate: 48_000,
            source_channels: 1,
            rendered_channels: 2,
          },
        ],
      },
    };

    const { container } = await renderNode(<AlbumPanel {...props} />);

    const receipt = container.querySelector(".album-export-receipt");
    expect(receipt?.textContent).toContain("rendered 48 kHz / 24-bit / stereo");
    expect(receipt?.textContent).toContain("requested Auto");
    expect(receipt?.textContent).toContain("Upsampled source 44.1 kHz");
    expect(receipt?.textContent).toContain("Upmixed source mono");
  });

  it("shows fold-down advisory for above-stereo album sources", async () => {
    const props = {
      ...baseProps(),
      albumExportReport: {
        album_wav_path: "C:/Masters/album_continuous.wav",
        manifest_path: "C:/Masters/manifest.json",
        requested_sample_rate: null,
        rendered_sample_rate: 48_000,
        source_sample_rates: [48_000],
        bit_depth: 24,
        rendered_channels: 2,
        source_channels: [2, 4],
        tracks: [
          {
            track_id: "track-1",
            position: 1,
            output_path: "C:/Masters/01-track.wav",
            measured_lufs: -14,
            source_sample_rate: 48_000,
            rendered_sample_rate: 48_000,
            source_channels: 4,
            rendered_channels: 2,
          },
        ],
      },
    };

    const { container } = await renderNode(<AlbumPanel {...props} />);

    const receipt = container.querySelector(".album-export-receipt");
    expect(receipt?.textContent).toContain("Folded source 4 ch to stereo");
  });
});
