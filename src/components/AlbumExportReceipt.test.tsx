import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";

import { AlbumExportReceipt } from "./AlbumExportReceipt";
import type { AlbumRenderReport } from "../lib/api";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

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

describe("AlbumExportReceipt", () => {
  it("shows delivered per-track loudness, target and peak without claiming a shortfall is failure", async () => {
    const report = {
      job_id: "details", status: { status: "done" }, album_wav_path: "album.wav", manifest_path: "manifest.json",
      requested_sample_rate: null, rendered_sample_rate: 48000, source_sample_rates: [48000],
      bit_depth: 24, rendered_channels: 2, source_channels: [2],
      tracks: [{ track_id: "one", position: 1, output_path: "C:/Masters/01-Song.wav", measured_lufs: -15.2,
        target_lufs: -14, true_peak_dbtp: -1.0, ceiling_dbtp: -1,
        source_sample_rate: 48000, rendered_sample_rate: 48000, source_channels: 2, rendered_channels: 2,
        override_album: false }],
    } as AlbumRenderReport;
    const { container } = await renderNode(<AlbumExportReceipt report={report} />);
    expect(container.querySelector("summary")?.textContent).toContain("Track results");
    expect(container.textContent).toContain("01-Song.wav");
    expect(container.textContent).toContain("-15.2 LUFS");
    expect(container.textContent).toContain("Target -14.0 LUFS");
    expect(container.textContent).toContain("-1.00 dBTP");
    expect(container.textContent).toContain("1.2 LU below target");
    expect(container.textContent).not.toContain("failed");
  });

  it("shows the rendered delivery format and upsample advisory", async () => {
    const report: AlbumRenderReport = {
      job_id: "album-render-job",
      status: { status: "done" },
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
          override_album: true,
        },
      ],
    };

    const { container } = await renderNode(<AlbumExportReceipt report={report} />);

    const receipt = container.querySelector(".album-export-receipt");
    expect(receipt?.textContent).toContain("rendered 48 kHz / 24-bit / stereo");
    expect(receipt?.textContent).toContain("requested Auto");
    expect(receipt?.textContent).toContain("Upsampled source 44.1 kHz");
    expect(receipt?.textContent).toContain("Upmixed source mono");
    expect(receipt?.textContent).toContain(
      "Override: track 1 rendered with its own settings",
    );
  });

  it("shows fold-down advisory for above-stereo album sources", async () => {
    const report: AlbumRenderReport = {
      job_id: "album-render-job",
      status: { status: "done" },
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
          override_album: false,
        },
      ],
    };

    const { container } = await renderNode(<AlbumExportReceipt report={report} />);

    const receipt = container.querySelector(".album-export-receipt");
    expect(receipt?.textContent).toContain("Folded source 4 ch to stereo");
    expect(receipt?.textContent).toContain("Target not recorded");
    expect(receipt?.textContent).toContain("Not recorded");
    expect(receipt?.textContent).not.toContain("Override:");
  });

  it("renders cancelled album exports without a blank path", async () => {
    const report: AlbumRenderReport = {
      job_id: "album-render-job",
      status: { status: "cancelled" },
      album_wav_path: "",
      manifest_path: "",
      requested_sample_rate: null,
      rendered_sample_rate: 48_000,
      source_sample_rates: [48_000],
      bit_depth: 24,
      rendered_channels: 2,
      source_channels: [2],
      tracks: [],
    };

    const { container } = await renderNode(<AlbumExportReceipt report={report} />);

    const receipt = container.querySelector(".album-export-receipt");
    expect(receipt?.textContent).toContain("Export cancelled");
    expect(receipt?.textContent).toContain("No album files were written.");
    expect(container.querySelector(".album-export-receipt-path")).toBeNull();
  });
});
