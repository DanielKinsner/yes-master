import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AnalysisResult, MasteringSettings } from "../bindings";
import { MasterOutPanel, RightRail } from "./RightRail";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const DEFAULT_SETTINGS: MasteringSettings = {
  preset: { kind: "universal" },
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
  },
};

const HOT_SOURCE_ANALYSIS: AnalysisResult = {
  track_id: "track-1",
  lufs_integrated: -10.5,
  lufs_short_term_max: -8.8,
  true_peak_dbtp: 0.2,
  dynamic_range_lu: 3.3,
  spectral_balance: { low: 0.3, mid: 0.4, high: 0.3 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: DEFAULT_SETTINGS,
  measured_at_iso: "2026-05-20T00:00:00.000Z",
  inferred_role: null,
  role_confidence: null,
  inferred_character: null,
  character_confidence: null,
  spectral_balance_6band: null,
  transient_flux: null,
  stereo_correlation: null,
  dynamic_range_p95_p10_db: null,
  lufs_short_term_max_3s: null,
  energy_density_score: null,
};

const CLEAN_SOURCE_ANALYSIS: AnalysisResult = {
  ...HOT_SOURCE_ANALYSIS,
  true_peak_dbtp: -1.2,
  lufs_integrated: -14.0,
  dynamic_range_lu: 8.0,
};

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

describe("MasterOutPanel", () => {
  it("does not show source analysis as live output while idle", async () => {
    const { container, root } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-120}
        peakLeftDbfs={-120}
        peakRightDbfs={-120}
        isPlaying={false}
        lufsMomentary={-120}
        lufsIntegrated={-120}
      />,
    );

    expect(container.textContent).toContain("idle");
    expect(container.textContent).not.toContain("-10.5");
    expect(container.textContent).not.toContain("0.2");
    const labels = [...container.querySelectorAll(".lufs-bar-label")].map(
      (e) => e.textContent,
    );
    expect(labels).toEqual(["L", "R"]);
    expect(
      (container.querySelector(".lufs-bar-fill") as HTMLElement | null)?.style.height,
    ).toBe("0%");

    await act(async () => {
      root.unmount();
    });
  });

  it("shows live playback tick values only while playing", async () => {
    const { container, root } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-8.5}
        peakLeftDbfs={-8.5}
        peakRightDbfs={-8.5}
        isPlaying
        lufsMomentary={-9.7}
        lufsIntegrated={-10.5}
      />,
    );

    expect(container.textContent).toContain("LIVE");
    expect(container.textContent).toContain("Momentary LUFS");
    expect(container.textContent).toContain("Since-play LUFS");
    expect(container.textContent).toContain("Live peak dBFS");
    expect(container.textContent).toContain("-9.7");
    expect(container.textContent).toContain("-10.5");
    expect(container.textContent).toContain("-8.5");
    expect(container.querySelector(".readout")?.getAttribute("title")).toContain(
      "Short-window live loudness",
    );

    await act(async () => {
      root.unmount();
    });
  });

  // Alive pass 1 (2026-08-19) — peak-hold pips ride the L/R bars while
  // playing, vanish when idle, and turn red above the -1 dBFS ceiling.
  it("renders a peak-hold pip per channel while playing", async () => {
    const { container, root } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-6}
        peakLeftDbfs={-6}
        peakRightDbfs={-9}
        isPlaying
        lufsMomentary={-14}
        lufsIntegrated={-14.5}
      />,
    );
    const pips = container.querySelectorAll(".lufs-bar-hold");
    expect(pips.length).toBe(2);
    // -6 dB on a -36..0 scale sits at 83.33% from the bottom.
    expect((pips[0] as HTMLElement).style.bottom).toBe("83.33333333333334%");
    await act(async () => {
      root.unmount();
    });
  });

  it("hides the peak-hold pip while idle", async () => {
    const { container, root } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-6}
        peakLeftDbfs={-6}
        peakRightDbfs={-6}
        isPlaying={false}
        lufsMomentary={-14}
        lufsIntegrated={-14.5}
      />,
    );
    expect(container.querySelectorAll(".lufs-bar-hold").length).toBe(0);
    await act(async () => {
      root.unmount();
    });
  });

  it("flags a hold above the -1 dBFS ceiling", async () => {
    const { container, root } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-0.3}
        peakLeftDbfs={-0.3}
        peakRightDbfs={-12}
        isPlaying
        lufsMomentary={-10}
        lufsIntegrated={-10}
      />,
    );
    const pips = container.querySelectorAll(".lufs-bar-hold");
    expect(pips[0].classList.contains("is-hold-over")).toBe(true);
    expect(pips[1].classList.contains("is-hold-over")).toBe(false);
    await act(async () => {
      root.unmount();
    });
  });

  it("uses simpler readout labels in Standard mode", async () => {
    const { container, root } = await renderNode(
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-8.5}
        peakLeftDbfs={-8.5}
        peakRightDbfs={-8.5}
        isPlaying
        lufsMomentary={-9.7}
        lufsIntegrated={-10.5}
        meterMode="standard"
      />,
    );

    expect(container.textContent).toContain("Loudness");
    expect(container.textContent).toContain("Since Play");
    expect(container.textContent).toContain("Peak");
    expect(container.textContent).not.toContain("Momentary LUFS");
    expect(container.querySelector(".readout")?.getAttribute("title")).toContain(
      "not the selected target",
    );

    await act(async () => {
      root.unmount();
    });
  });
});

describe("RightRail source checks", () => {
  it("keeps the primary action as Export Master when current checks are clean", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={CLEAN_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Master",
    );

    expect(exportButton).toBeTruthy();

    await act(async () => {
      root.unmount();
    });
  });

  it("labels the primary action Export With Review when source checks warn", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    );

    expect(exportButton).toBeTruthy();
    expect(onExport).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });

  it("opens review instead of exporting when warnings are present", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;

    expect(exportButton).toBeTruthy();

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).not.toHaveBeenCalled();
    expect(container.textContent).toContain("Review before export");
    expect(container.textContent).toContain("Source true peak 0.2 dBTP");
    expect(container.textContent).not.toContain("Source loudness range (LRA) 3.3 LU");

    const adjustButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Adjust Settings",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      adjustButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).not.toHaveBeenCalled();
    expect(container.textContent).not.toContain("Review before export");

    await act(async () => {
      root.unmount();
    });
  });

  it("exports from the review panel when the user chooses Export Anyway", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const anywayButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Anyway",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      anywayButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.unmount();
    });
  });

  it("does not double-fire Export Anyway while an export is already in flight (§6)", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    // An export is now under way — re-render with isExporting=true. The review
    // panel stays open (analysis/lastChecks unchanged).
    await act(async () => {
      root.render(
        <RightRail
          analysis={HOT_SOURCE_ANALYSIS}
          lastChecks={undefined}
          canExport
          isExporting={true}
          isRendering={false}
          onExport={onExport}
          previewStale={false}
          canRenderPreview
          onUpdatePreview={vi.fn()}
        />,
      );
    });

    const anywayButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Anyway",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      anywayButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });

  it("exports immediately when current checks are clean", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={CLEAN_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Master",
    ) as HTMLButtonElement | undefined;

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).toHaveBeenCalledTimes(1);
    expect(container.textContent).not.toContain("Review before export");

    await act(async () => {
      root.unmount();
    });
  });

  it("uses the Album export action directly in album mode", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        exportMode="album"
        analysis={HOT_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Export Album",
    ) as HTMLButtonElement | undefined;

    expect(exportButton).toBeTruthy();

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onExport).toHaveBeenCalledTimes(1);
    expect(container.textContent).not.toContain("Review before export");

    await act(async () => {
      root.unmount();
    });
  });

  it("shows render progress with a cancel action while exporting", async () => {
    const onCancelRender = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={CLEAN_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting
        isRendering={false}
        renderProgress={{ job_id: "render-job-1", kind: "master", fraction: 0.42 }}
        onExport={vi.fn()}
        onCancelRender={onCancelRender}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    expect(container.textContent).toContain("Master render 42%");
    const cancel = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Cancel",
    ) as HTMLButtonElement | undefined;
    expect(cancel?.disabled).toBe(false);

    await act(async () => {
      cancel?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onCancelRender).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.unmount();
    });
  });

  // 2026-08-18: SOURCE CHECK (measurements + Re-analyze) moved out of the rail
  // into Source Insight under the track title — see SourceInsight.test.tsx.
  // The pre-export gate above still derives its rows from the same analysis.

  it("keeps audit WAV rendering disabled until analysis exists", async () => {
    const onUpdatePreview = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={undefined}
        lastChecks={undefined}
        canExport={false}
        isExporting={false}
        isRendering={false}
        onExport={vi.fn()}
        previewStale
        canRenderPreview={false}
        onUpdatePreview={onUpdatePreview}
      />,
    );

    const auditButton = container.querySelector<HTMLButtonElement>(".right-rail-audit")!;
    expect(auditButton.disabled).toBe(true);
    expect(auditButton.title).toBe("Analyze a track first.");
    await act(async () => {
      auditButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(onUpdatePreview).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });
});

describe("MasterOutPanel landing note", () => {
  function panel(landingPending: boolean) {
    return (
      <MasterOutPanel
        isAnalyzing={false}
        peakDbfs={-8.5}
        peakLeftDbfs={-8.5}
        peakRightDbfs={-8.5}
        isPlaying
        lufsMomentary={-9.7}
        lufsIntegrated={-10.5}
        landingPending={landingPending}
      />
    );
  }

  it("shows the landing note while the corrective gain is pending", async () => {
    const { container, root } = await renderNode(panel(true));
    const note = container.querySelector(".landing-note");
    expect(note?.textContent).toContain("Measuring preview level");
    await act(async () => { root.unmount(); });
  });

  it("hides the note once the landing has settled (and by default)", async () => {
    const { container, root } = await renderNode(panel(false));
    expect(container.querySelector(".landing-note")).toBeNull();
    await act(async () => { root.unmount(); });
  });
});

describe("RightRail export-in-progress tooltip (Q27)", () => {
  it("explains the disabled export button while an export is running", async () => {
    const { container, root } = await renderNode(
      <RightRail
        analysis={CLEAN_SOURCE_ANALYSIS}
        lastChecks={undefined}
        canExport
        isExporting={true}
        isRendering={false}
        onExport={vi.fn()}
        previewStale={false}
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = container.querySelector<HTMLButtonElement>("button.right-rail-export");
    expect(exportButton).toBeTruthy();
    expect(exportButton?.disabled).toBe(true);
    expect(exportButton?.getAttribute("title")).toBe(
      "An export is already running — it finishes or fails before the next one starts.",
    );

    await act(async () => {
      root.unmount();
    });
  });
});
