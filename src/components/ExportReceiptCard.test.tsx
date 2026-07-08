import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ExportReceiptCard, formatBitDepth, formatSampleRate } from "./ExportReceiptCard";
import type { ExportReceipt } from "../hooks/useTrackMaster";
import type {
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "../bindings";

vi.mock("../lib/api", () => ({
  api: { openOutput: vi.fn(async () => undefined) },
}));

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT =
  true;

function job(): RenderJob {
  return {
    id: "job-1",
    job_id: "job-1",
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

function track(): ImportedTrack {
  return {
    id: "t1",
    path: "C:/music/lumberjack-final.mp3",
    display_name: "lumberjack-final",
    source_format: "mp3",
    duration_seconds: 252, // 4:12
    sample_rate: 48_000,
    channels: 2,
  };
}

// Matches the shape of DEFAULT_SETTINGS (useTrackMaster) — the receipt reads
// preset, intensity, and delivery_profile off it. Warmth @ 0.5 mirrors the
// mock so the "Moderate" label / 50% assertions are meaningful.
function settings(overrides: Partial<MasteringSettings> = {}): MasteringSettings {
  return {
    preset: { kind: "warmth" },
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
      compression_mode: "preset",
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
      adaptive_strength: 0.5,
    },
    ...overrides,
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
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-medallion-clean")).not.toBeNull();
    expect(container.textContent).toContain("Export complete");
    expect(container.textContent).toContain("Master -13.5 LUFS");
    expect(container.textContent).toContain("Source · bass +1.2");
    expect(container.textContent).toContain("Compression · compression eased low 20%");
  });

  it("renders the Track section identity line from the source track", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-track-name")?.textContent).toBe(
      "lumberjack-final",
    );
    // 13c dialect: one quiet line, spec order duration · format · sr · channels.
    expect(container.querySelector(".track-meta-line")?.textContent).toBe(
      "4:12 · MP3 · 48 kHz · Stereo",
    );
  });

  it("copies the saved file path to the clipboard and confirms", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText },
      configurable: true,
    });
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        onClose={() => {}}
      />,
    );
    const copyBtn = container.querySelector<HTMLButtonElement>(".receipt-file-copy");
    expect(copyBtn).not.toBeNull();
    await act(async () => {
      copyBtn!.click();
    });
    expect(writeText).toHaveBeenCalledWith("out/track.master.wav");
    expect(container.querySelector(".receipt-file-copy.is-copied")).not.toBeNull();
  });

  it("renders the preset's real name + blurb and the real intensity label", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings({ preset: { kind: "warmth" }, intensity: 0.5 })}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-style-name")?.textContent).toBe("Warmth");
    // Real blurb, not the mock's marketing paragraph.
    expect(container.querySelector(".receipt-style-blurb")?.textContent).toBe(
      "Fuller, smoother body",
    );
    expect(container.querySelector(".receipt-dial-pct")?.textContent).toBe("50%");
    // 50% is "Moderate" here — the mock's "Aggressive" is wrong.
    expect(container.querySelector(".receipt-intensity-label")?.textContent).toBe(
      "Moderate",
    );
    expect(container.querySelector(".receipt-style-orb-img")).not.toBeNull();
  });

  it("renders the review medallion for a single warning without double-pluralizing", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([
          { level: "warning", code: "lufs_very_loud", message: "loud" },
        ])}
        track={track()}
        settings={settings()}
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
        track={track()}
        settings={settings()}
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
        track={track()}
        settings={settings()}
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
