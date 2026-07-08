import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ExportReceiptCard, formatBitDepth, formatSampleRate } from "./ExportReceiptCard";
import type { ExportReceipt } from "../hooks/useTrackMaster";
import type {
  AnalysisResult,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "../bindings";

vi.mock("../lib/api", () => ({
  api: {
    openOutput: vi.fn(async () => undefined),
    buildInfo: vi.fn(async () => "v0.1.0 · abc1234 · 2026-06-10 12:00"),
  },
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

// buildQualityRows reads only these three source measurements — a lean cast
// keeps the fixture readable (values mirror the mock's source rows).
function analysis(): AnalysisResult {
  return {
    true_peak_dbtp: 2.5,
    lufs_integrated: -7.5,
    dynamic_range_lu: 6.6,
  } as unknown as AnalysisResult;
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
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-verified-clean")).not.toBeNull();
    expect(container.textContent).toContain("Export complete");
    expect(container.textContent).toContain("File processed and verified");
    expect(container.textContent).toContain("-13.5 LUFS");
    expect(container.textContent).toContain("Source · bass +1.2");
    expect(container.textContent).toContain("Compression · compression eased low 20%");
  });

  it("renders the Results panel from measured values, labelling dynamic range (not LRA)", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings({ delivery_profile: "streaming-universal" })}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    const rows = [...container.querySelectorAll(".receipt-result-row")].map(
      (r) => r.textContent,
    );
    expect(rows).toContain("Integrated loudness-13.5 LUFS");
    expect(rows).toContain("True peak-1.25 dBTP");
    expect(rows).toContain("Dynamic range8.5 LU");
    // We measure dynamic range, not EBU Loudness Range — "LRA" must not appear.
    expect(container.textContent).not.toContain("LRA");
    // Mastering Target chip = real profile name + real target LUFS.
    expect(container.querySelector(".receipt-target-chip")?.textContent).toBe(
      "Streaming (Spotify / YouTube / Tidal / Amazon) · -14.0 LUFS",
    );
  });

  it("shows the Mastering Target name alone for a profile with no target (custom)", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings({ delivery_profile: "custom" })}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-target-chip")?.textContent).toBe("Custom");
  });

  it("renders the Track section identity line from the source track", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        analysis={analysis()}
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
        analysis={analysis()}
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
        analysis={analysis()}
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
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-verified-review")).not.toBeNull();
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
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-verified-review")).not.toBeNull();
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
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    expect(container.querySelector(".receipt-verified-attention")).not.toBeNull();
    expect(container.textContent).toContain("Export saved — needs attention");
  });

  it("renders three clean source rows when the export has no flags", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    const rows = container.querySelectorAll(".receipt-quality-row");
    expect(rows).toHaveLength(3);
    expect(container.querySelectorAll(".receipt-quality-row.is-ok")).toHaveLength(3);
    expect(container.querySelector(".receipt-quality-row.is-warning")).toBeNull();
    expect(container.querySelector(".receipt-quality-row.is-critical")).toBeNull();
    // Source values from analysis.
    expect(container.textContent).toContain("2.5 dBTP");
    expect(container.textContent).toContain("-7.5 LUFS");
    expect(container.textContent).toContain("6.6 LU");
  });

  it("renders the warning-state Quality Check variant — never all-green when flagged", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([
          {
            level: "warning",
            code: "lufs_very_loud",
            message: "Integrated loudness is -6.0 LUFS.",
          },
        ])}
        track={track()}
        settings={settings()}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    // The honest state + colour reaches the loudness row and its message shows.
    const warned = container.querySelector(".receipt-quality-row.is-warning");
    expect(warned).not.toBeNull();
    expect(warned?.textContent).toContain("Source loudness");
    expect(warned?.textContent).toContain("Integrated loudness is -6.0 LUFS.");
    // Not an all-green card: at least one row is non-ok.
    const okRows = container.querySelectorAll(".receipt-quality-row.is-ok").length;
    const total = container.querySelectorAll(".receipt-quality-row").length;
    expect(okRows).toBeLessThan(total);
  });

  it("renders the footer with the export timestamp and real build stamp, no external link", async () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    // Flush the async build_info effect.
    await act(async () => {});
    const footer = container.querySelector(".receipt-footer");
    expect(footer?.textContent).toContain("Exported");
    // From job.started_at_iso (2026-…); year is timezone-stable.
    expect(footer?.textContent).toContain("2026");
    // Real version · git hash · build time — the mock's "Engine v2.4.1" is fiction.
    expect(footer?.textContent).toContain("v0.1.0 · abc1234 · 2026-06-10 12:00");
    expect(footer?.textContent).not.toContain("Engine v2.4.1");
    expect(footer?.textContent).not.toContain("yesmaster.app");
  });

  it("renders the Audio Format panel (bit depth, sample rate, file type; no channels)", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([])}
        track={track()}
        settings={settings()}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    const format = container.querySelector('[aria-label="Audio format"]');
    expect(format?.textContent).toContain("Bit depth");
    expect(format?.textContent).toContain("24-bit");
    expect(format?.textContent).toContain("Sample rate");
    expect(format?.textContent).toContain("48 kHz");
    expect(format?.textContent).toContain("File type");
    expect(format?.textContent).toContain("WAV");
    // Channels row deliberately omitted (not measured on the receipt).
    expect(format?.textContent).not.toContain("Channels");
  });

  it("surfaces a format-only critical (sample-rate mismatch) as its own honest row", () => {
    const container = render(
      <ExportReceiptCard
        receipt={receipt([
          {
            level: "critical",
            code: "sample_rate_mismatch",
            message: "Rendered 44100 Hz does not match 48000 Hz.",
          },
        ])}
        track={track()}
        settings={settings()}
        analysis={analysis()}
        onClose={() => {}}
      />,
    );
    const critical = container.querySelector(".receipt-quality-row.is-critical");
    expect(critical).not.toBeNull();
    expect(critical?.textContent).toContain("Sample rate");
    expect(critical?.textContent).toContain("Rendered 44100 Hz does not match 48000 Hz.");
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
