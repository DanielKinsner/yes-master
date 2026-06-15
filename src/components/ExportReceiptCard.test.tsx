import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ExportReceiptCard, formatBitDepth, formatSampleRate } from "./ExportReceiptCard";
import type { ExportReceipt } from "../hooks/useTrackMaster";
import type { QualityCheck, RenderJob } from "../bindings";

vi.mock("../lib/api", () => ({
  api: { openOutput: vi.fn(async () => undefined) },
}));

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT =
  true;

function job(): RenderJob {
  return {
    id: "job-1",
    kind: "master",
    target_tracks: ["t1"],
    status: { status: "done" },
    progress: 1,
    started_at_iso: "2026-06-09T00:00:00+00:00",
    output_paths: ["out/track.master.wav"],
    measurements: {
      lufs_integrated: -13.5,
      true_peak_dbtp: -1.25,
      dynamic_range_lu: 8.5,
      sample_rate: 48_000,
      bit_depth: 24,
      effective_adaptive_strength: 0.5,
      source_profile_digest: "bass +1.2",
      confidence_digest: null,
      compression_digest:
        "compression eased low 20% / mid 16% / high 25%; stand-down 0.00; density confidence 1.00",
    },
  };
}

function receipt(checks: QualityCheck[]): ExportReceipt {
  return {
    trackId: "t1",
    outputPath: "out/track.master.wav",
    checks,
    job: job(),
    kind: "track",
  };
}

let root: Root | null = null;

function render(node: React.ReactElement): HTMLElement {
  const container = document.createElement("div");
  document.body.appendChild(container);
  act(() => {
    root = createRoot(container);
    root.render(node);
  });
  return container;
}

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.innerHTML = "";
});

describe("ExportReceiptCard", () => {
  it("renders the clean medallion with delivered measurements", () => {
    const container = render(
      <ExportReceiptCard receipt={receipt([])} onClose={() => {}} />,
    );
    expect(container.querySelector(".receipt-medallion-clean")).not.toBeNull();
    expect(container.textContent).toContain("Export complete");
    expect(container.textContent).toContain("Master -13.5 LUFS");
    expect(container.textContent).toContain("Source · bass +1.2");
    expect(container.textContent).toContain("Compression · compression eased low 20%");
  });

  it("renders the review medallion for a single warning without double-pluralizing", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([
          { level: "warning", code: "lufs_very_loud", message: "loud" },
        ])}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-medallion-review")).not.toBeNull();
    expect(container.textContent).toContain("1 item to review");
    expect(container.textContent).not.toContain("reviews");
  });

  it("pluralizes only the noun for multiple warnings", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([
          { level: "warning", code: "lufs_very_loud", message: "loud" },
          { level: "warning", code: "true_peak_high", message: "peak" },
        ])}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-medallion-review")).not.toBeNull();
    expect(container.textContent).toContain("2 items to review");
    expect(container.textContent).not.toContain("reviews");
  });

  it("renders needs-attention and qualifies the header for criticals", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([
          { level: "critical", code: "sample_rate_mismatch", message: "rate" },
        ])}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-medallion-attention")).not.toBeNull();
    expect(container.textContent).toContain("Export saved — needs attention");
  });
});

describe("format helpers", () => {
  it("formats sample rates and bit depths", () => {
    expect(formatSampleRate(44_100)).toBe("44.1 kHz");
    expect(formatSampleRate(48_000)).toBe("48 kHz");
    expect(formatSampleRate(800)).toBe("800 Hz");
    expect(formatBitDepth(24)).toBe("24-bit");
    expect(formatBitDepth(32)).toBe("32-bit float");
  });
});
