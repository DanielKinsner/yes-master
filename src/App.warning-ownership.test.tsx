// U10(a) — one owner, one location, per warning.
//
// The unit's acceptance says "remove duplicate warnings and give each warning,
// blocker, and advisory one clear owner and location". An audit of the existing
// surfaces found three places where one fact was presented twice at the same
// moment. These tests pin each of them, and each was verified RED against the
// pre-fix source before the fix landed:
//
//   DUP-1  RightRail: the QUALITY CHECK panel and the inline pre-export review
//          gate rendered the same label AND the same detail message, plus a
//          REVIEW badge each, simultaneously in one rail.
//   DUP-2  ExportReceiptCard: a full-screen visual modal with no dialog
//          semantics, so its Quality check rows and the rail's EXPORT CHECK
//          rows — same checks, same payload — were both in the accessibility
//          tree at once.
//   DUP-3  Album sidebar row: a `★` override mark next to the track name AND
//          an "Overrides" sequence chip in the same row, for the same fact.
//
// The invariant these lock in: a warning may be presented once. A surface that
// re-states one must be modal, so only one copy is ever presented.

import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { RightRail } from "./components/RightRail";
import { ExportReceiptCard } from "./components/ExportReceiptCard";
import type {
  AnalysisResult,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const mocks = vi.hoisted(() => ({
  tm: null as Record<string, unknown> | null,
}));

vi.mock("./hooks/useTrackMaster", () => ({
  useTrackMaster: () => {
    if (!mocks.tm) throw new Error("mock tm not configured");
    return mocks.tm;
  },
}));

const SETTINGS: MasteringSettings = {
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

const HOT_ANALYSIS: AnalysisResult = {
  track_id: "track-1",
  lufs_integrated: -5.5,
  lufs_short_term_max: -4.8,
  true_peak_dbtp: 0.2,
  dynamic_range_lu: 3.3,
  spectral_balance: { low: 0.3, mid: 0.4, high: 0.3 },
  transient_density: 0.5,
  stereo_width: 0.5,
  recommended_universal: SETTINGS,
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

const TRUE_PEAK_MESSAGE =
  "True peak reached -0.2 dBTP, above the -1.0 dBTP ceiling.";
const DYNAMICS_MESSAGE = "Dynamic range is 3.3 LU; heavy limiting detected.";

const WARNING_CHECKS: QualityCheck[] = [
  { code: "true_peak_high", level: "warning", message: TRUE_PEAK_MESSAGE },
  { code: "dynamic_range_low", level: "warning", message: DYNAMICS_MESSAGE },
];

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

function occurrences(haystack: string, needle: string): number {
  let count = 0;
  let from = 0;
  for (;;) {
    const at = haystack.indexOf(needle, from);
    if (at === -1) return count;
    count += 1;
    from = at + needle.length;
  }
}

/**
 * Text a user is actually PRESENTED — `textContent` minus every subtree that
 * is `inert`, `aria-hidden`, or `hidden`.
 *
 * Raw `textContent` is the wrong instrument for a duplication test: content
 * behind a modal is still in the DOM, so a raw count says "twice" about
 * something no user meets twice. This walks the tree the way the platform
 * presents it, which is the claim under test. (U9 made the same distinction
 * when it re-measured a visible-text assertion in App.chrome.test.tsx.)
 */
function presentedText(root: Element): string {
  let out = "";
  const walk = (node: Node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      out += node.textContent ?? "";
      return;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return;
    const el = node as Element;
    if (
      el.hasAttribute("inert") ||
      el.getAttribute("aria-hidden") === "true" ||
      el.hasAttribute("hidden")
    ) {
      return;
    }
    node.childNodes.forEach(walk);
  };
  root.childNodes.forEach(walk);
  return out;
}

afterEach(() => {
  document.body.innerHTML = "";
  mocks.tm = null;
});

describe("U10(a) DUP-1 — the pre-export review gate does not restate the panel", () => {
  it("presents each warning exactly once with the gate open", async () => {
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_ANALYSIS}
        lastChecks={WARNING_CHECKS}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={vi.fn()}
        previewStale
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );

    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    expect(exportButton).toBeTruthy();

    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    // The gate is open...
    expect(container.textContent).toContain("Review before export");

    // ...and the standing rail is inert behind it, so each warning is
    // presented once, not twice. Before the fix every count here was 2.
    const text = presentedText(container);
    expect(occurrences(text, TRUE_PEAK_MESSAGE)).toBe(1);
    expect(occurrences(text, DYNAMICS_MESSAGE)).toBe(1);
    // The friendly one-liner label is the same story.
    expect(occurrences(text, "True peak above safe ceiling")).toBe(1);
    expect(occurrences(text, "Low loudness range (LRA)")).toBe(1);
    // And a single REVIEW badge, not one per surface.
    expect(occurrences(text, "REVIEW")).toBe(1);

    // The standing rail is genuinely inert — not merely covered by a scrim.
    // This is what keeps the second copy out of the tab order and out of the
    // accessibility tree, and it is the load-bearing half of the fix.
    const standing = container.querySelector(".right-rail-standing");
    expect(standing?.hasAttribute("inert")).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });

  it("carries real dialog semantics so assistive tech sees one owner", async () => {
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_ANALYSIS}
        lastChecks={WARNING_CHECKS}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={vi.fn()}
        previewStale
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );
    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const dialog = container.querySelector(".export-review-panel");
    expect(dialog?.getAttribute("role")).toBe("dialog");
    expect(dialog?.getAttribute("aria-modal")).toBe("true");
    // Labelled by its own visible title, not an invented string.
    const labelledBy = dialog?.getAttribute("aria-labelledby");
    expect(labelledBy).toBeTruthy();
    expect(container.querySelector(`#${labelledBy}`)?.textContent).toContain(
      "Review before export",
    );
    // Focus moved into the gate — a dialog role the keyboard did not honour
    // would be a claim, not a behaviour.
    expect(document.activeElement).toBe(dialog);

    await act(async () => {
      root.unmount();
    });
  });

  it("treats Escape as Adjust Settings, never as Export Anyway", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={HOT_ANALYSIS}
        lastChecks={WARNING_CHECKS}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );
    const exportButton = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      exportButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(container.textContent).toContain("Review before export");

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });

    expect(container.textContent).not.toContain("Review before export");
    // Dismissing a warning gate must never be a shortcut to the export.
    expect(onExport).not.toHaveBeenCalled();

    await act(async () => {
      root.unmount();
    });
  });
});

describe("U10(a) DUP-2 — the export receipt owns its checks while it is up", () => {
  const receiptJob: RenderJob = {
    id: "job-1",
    job_id: "job-1",
    kind: "master",
    target_tracks: ["track-1"],
    status: { status: "done" },
    progress: 1,
    started_at_iso: "2026-07-25T00:00:00Z",
    output_paths: ["/out/master.wav"],
  };

  const track: ImportedTrack = {
    id: "track-1",
    path: "/audio/track-1.wav",
    display_name: "track-1.wav",
    source_format: "wav",
    duration_seconds: 120,
    sample_rate: 44_100,
    channels: 2,
  };

  it("is a real modal, not a visual-only overlay", async () => {
    const { container, root } = await renderNode(
      <ExportReceiptCard
        receipt={{
          trackId: "track-1",
          outputPath: "/out/master.wav",
          checks: WARNING_CHECKS,
          job: receiptJob,
          kind: "track",
        }}
        track={track}
        settings={SETTINGS}
        analysis={HOT_ANALYSIS}
        onClose={vi.fn()}
      />,
    );

    const dialog = container.querySelector(".receipt");
    expect(dialog?.getAttribute("role")).toBe("dialog");
    expect(dialog?.getAttribute("aria-modal")).toBe("true");
    const labelledBy = dialog?.getAttribute("aria-labelledby");
    expect(labelledBy).toBeTruthy();
    // The label is the receipt's own headline, so it stays honest when the
    // export is flagged ("Export saved — needs attention") rather than
    // announcing a fixed cheerful string.
    expect(container.querySelector(`#${labelledBy}`)?.textContent).toBeTruthy();
    expect(document.activeElement).toBe(dialog);

    await act(async () => {
      root.unmount();
    });
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    const { root } = await renderNode(
      <ExportReceiptCard
        receipt={{
          trackId: "track-1",
          outputPath: "/out/master.wav",
          checks: WARNING_CHECKS,
          job: receiptJob,
          kind: "track",
        }}
        track={track}
        settings={SETTINGS}
        analysis={HOT_ANALYSIS}
        onClose={onClose}
      />,
    );

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });
    expect(onClose).toHaveBeenCalledTimes(1);

    await act(async () => {
      root.unmount();
    });
  });
});

describe("U10(a) DUP-3 — an album row states an override once", () => {
  const albumTrack: ImportedTrack = {
    id: "album-track-1",
    path: "/audio/album-track-1.wav",
    display_name: "album-track-1.wav",
    source_format: "wav",
    duration_seconds: 120,
    sample_rate: 44_100,
    channels: 2,
  };

  function albumState(): Record<string, unknown> {
    return {
      mode: "album",
      setMode: vi.fn(),
      saveProjectAs: vi.fn(),
      openProjectFromDisk: vi.fn(),
      tracks: [albumTrack],
      selectedTrackId: null,
      selectedTrack: null,
      selectedAnalysis: undefined,
      selectedWaveform: undefined,
      selectedSettings: undefined,
      selectedRegion: null,
      selectTrack: vi.fn(),
      removeTrack: vi.fn(),
      openImportDialog: vi.fn(),
      isAnalyzing: false,
      isLoadingWaveform: false,
      isDragOver: false,
      isExporting: false,
      isRendering: false,
      previewStale: false,
      updatePreview: vi.fn(),
      exportMaster: vi.fn(),
      error: null,
      clearError: vi.fn(),
      lastExportReceipt: null,
      clearExportReceipt: vi.fn(),
      reorderTracks: vi.fn(),
      // The track opts out of album intent — the fact under test.
      overrideAlbum: new Set([albumTrack.id]),
      albumArcKind: "cinematic",
      albumIntensity: 1,
      albumTitle: "",
      albumRendering: false,
      albumExportReport: null,
      albumSampleRate: null,
      albumBitDepth: null,
      setAlbumArc: vi.fn(),
      setAlbumIntensity: vi.fn(),
      setAlbumTitle: vi.fn(),
      setAlbumSampleRate: vi.fn(),
      setAlbumBitDepth: vi.fn(),
      exportAlbumPlan: vi.fn(),
      transport: {
        isPlaying: false,
        currentTimeSec: 0,
        playbackKind: "source",
        loop: false,
        volumeMatch: false,
        exportLufsPreview: false,
        peakDbfs: -120,
        compressionGr: { low: -120, mid: -120, high: -120 },
        lufsMomentary: -120,
        lufsIntegrated: -120,
        spectrumDb: [],
      },
      renderProgress: null,
      undo: vi.fn(),
      redo: vi.fn(),
      canUndo: false,
      canRedo: false,
      setPreset: vi.fn(),
      setIntensity: vi.fn(),
      setEqBand: vi.fn(),
      setAdvanced: vi.fn(),
      setInputGain: vi.fn(),
      setOutputGain: vi.fn(),
      setDeliveryProfile: vi.fn(),
      setLoudnessTarget: vi.fn(),
      setDeliveryBitDepth: vi.fn(),
      setDeliverySampleRate: vi.fn(),
      togglePlay: vi.fn(),
      seek: vi.fn(),
      setPlaybackKind: vi.fn(),
      toggleLoop: vi.fn(),
      setVolumeMatch: vi.fn(),
      setExportLufsPreview: vi.fn(),
      setRegion: vi.fn(),
      clearRegion: vi.fn(),
      albumIntent: null,
      selectedIsOverriding: false,
      followingAlbumIntent: false,
      toggleOverrideAlbum: vi.fn(),
      userPresets: [],
      savingPreset: false,
      saveCurrentPreset: vi.fn(),
      deleteUserPresetById: vi.fn(),
      hadPriorSession: true,
      setForceWysiwyg: vi.fn(),
      resetToStandardManaged: vi.fn(),
      exportStandardMaster: vi.fn(),
      saveUserPreset: vi.fn(),
    };
  }

  it("shows the override once, as the sequence chip that explains itself", async () => {
    mocks.tm = albumState();
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    await act(async () => {
      root.render(<App />);
    });

    const row = container.querySelector(".track-pick");
    expect(row).toBeTruthy();
    const rowText = row?.textContent ?? "";

    // The star mark is gone — it said the same thing with less meaning.
    expect(container.querySelector(".override-mark")).toBeNull();
    expect(rowText).not.toContain("★");

    // Exactly one override indicator survives in the row.
    //
    // Counted as elements, not as substrings: the surviving chip deliberately
    // carries BOTH a visible "Overrides" and an `.sr-only` sentence that
    // expands it. Those are one indicator rendered for two audiences — each
    // user is presented exactly one — so a substring count would flag the
    // correct pattern as a duplicate.
    expect(row?.querySelectorAll(".seq-chip.is-override")).toHaveLength(1);
    expect(rowText).toContain(
      "Overrides the album settings — renders with its own sound and its own target.",
    );

    await act(async () => {
      root.unmount();
    });
  });
});
