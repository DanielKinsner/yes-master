// Named acceptance scenarios from the public-beta quality plan (U10(d)).
//
// The plan requires each scenario family to be "instantiated as a named test
// case ... with input, action, and expected result", and the traceability table
// names the proving unit and evidence layer for each. This file is the
// FRONTEND-UNIT instance of the three that have one. The rest of each
// scenario's proof lives where the table says it does:
//
//   S-D1  frontend-unit (here) + browser-headless (`empty` scenario)
//   S-F1  frontend-unit (here) + browser-headless (`clean`, `warning`,
//         `long-copy`, `export-success`, `export-cancel`)
//   S-F3  frontend-unit (here) + browser-headless (`export-success` /
//         `export-cancel`) + native-synthetic
//         (src-tauri/src/exports.rs::suggest_export_filename tests)
//
//   S-E1  browser-headless only — `S-E1-rapid-ab` in
//         scripts/verify-app-headless.mjs. Rapid switching against a live
//         transport is not a thing a jsdom unit can honestly claim.
//   S-F2  browser-headless only — the `album-1` / `album-4` / `album-12` /
//         `album-long` / `album-warning` scenarios at 1440x900 and 1360x740.
//
// Deliberately NOT re-implemented here: behaviour already pinned elsewhere.
// Restating an existing assertion in a differently-named test does not make a
// scenario more proved, it makes the suite slower and the coverage picture
// harder to read. Where a leg is already covered, the case below says where.

import { act } from "react";
import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { EmptyState } from "./components/EmptyState";
import { RightRail } from "./components/RightRail";
import { SourceInsight } from "./components/SourceInsight";
import { formatUserError, userErrorMessage } from "./lib/user-errors";
import { supportedAudioExtensionFromName } from "./lib/supported-formats";
import {
  defaultExportPath,
  exportDirectoryFromPath,
  lastExportDirectory,
  rememberExportDirectory,
  type ExportLocationStore,
} from "./lib/export-location";
import type { AnalysisResult, MasteringSettings, QualityCheck } from "./bindings";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

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

function analysisFor(overrides: Partial<AnalysisResult>): AnalysisResult {
  return {
    track_id: "scenario-track",
    lufs_integrated: -14,
    lufs_short_term_max: -12,
    true_peak_dbtp: -1.5,
    dynamic_range_lu: 9,
    spectral_balance: { low: 0.33, mid: 0.34, high: 0.33 },
    transient_density: 0.5,
    stereo_width: 0.5,
    recommended_universal: SETTINGS,
    measured_at_iso: "2026-07-25T00:00:00.000Z",
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
    ...overrides,
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

function memoryStore(): ExportLocationStore & { map: Map<string, string> } {
  const map = new Map<string, string>();
  return {
    map,
    getItem: (key) => map.get(key) ?? null,
    setItem: (key, value) => void map.set(key, value),
  };
}

afterEach(() => {
  document.body.innerHTML = "";
});

// ---------------------------------------------------------------------------
// S-D1
// ---------------------------------------------------------------------------
//
// INPUT   Start with no track, then select an unsupported, missing, empty,
//         mono, stereo, long, and already-mastered source.
// ACTION  Observe the state the UI offers for each.
// EXPECT  An accurate next action or a recoverable error; never an impossible
//         mastering/export state.
describe("S-D1 — source states never produce an impossible action", () => {
  it("no track: offers import, and offers no export", async () => {
    const { container, root } = await renderNode(<EmptyState onAdd={vi.fn()} />);
    // The next action is stated, not implied.
    const importButton = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Import audio",
    );
    expect(importButton).toBeTruthy();
    // And the accepted inputs are named rather than discovered by failure.
    expect(container.textContent).toContain("Supports WAV");
    await act(async () => root.unmount());
  });

  it("no analysis: export is disabled and says why, in text", async () => {
    // The impossible state this guards is the one a beta tester finds in the
    // first minute: a live-looking Export button on a track the engine has not
    // measured yet.
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
        onUpdatePreview={vi.fn()}
      />,
    );
    const exportButton = container.querySelector<HTMLButtonElement>(
      "button.right-rail-export",
    );
    expect(exportButton?.disabled).toBe(true);
    // Discoverable without a pointer: an associated description, not a title.
    const describedBy = exportButton?.getAttribute("aria-describedby");
    expect(describedBy).toBeTruthy();
    expect(container.querySelector(`#${describedBy}`)?.textContent).toContain(
      "Analyze a track first",
    );
    await act(async () => root.unmount());
  });

  it("unsupported source: rejected by extension before anything claims to work", () => {
    expect(supportedAudioExtensionFromName("mix.wav")).toBe("wav");
    expect(supportedAudioExtensionFromName("mix.flac")).toBe("flac");
    expect(supportedAudioExtensionFromName("notes.txt")).toBeNull();
    expect(supportedAudioExtensionFromName("no-extension")).toBeNull();
    expect(supportedAudioExtensionFromName("")).toBeNull();
  });

  it("missing or unreadable source: a recoverable message naming the next move", () => {
    // Each of these is a real backend error prefix. The requirement is not
    // "show an error" — it is that the message tells the user what to DO.
    const missing = formatUserError(new Error("io error: file not found"), {
      name: "moved.wav",
    });
    expect(missing.message).toContain("try again");
    expect(missing.detail).toContain("io error");

    const undecodable = formatUserError(new Error("decode error: bad header"), {
      name: "empty.wav",
    });
    // An empty/corrupt file names the file and offers two concrete recoveries.
    expect(undecodable.message).toContain("empty.wav");
    expect(undecodable.message).toContain("Re-import it or use Re-analyze");

    // An unknown failure still degrades to something honest rather than a
    // stack trace or a blank toast.
    const unknown = userErrorMessage(new Error("kaboom"));
    expect(unknown).toContain("Something went wrong.");
    expect(unknown).toContain("kaboom");
  });

  it("already-mastered source: flagged for review, never blocked from export", async () => {
    // The product promise is that the user may overcook their own track. An
    // already-hot source must therefore REVIEW, not refuse — a hard block here
    // would be the impossible state in the other direction.
    const hot = analysisFor({
      lufs_integrated: -5.2,
      true_peak_dbtp: 0.4,
      dynamic_range_lu: 3.1,
    });
    const { container, root } = await renderNode(
      <RightRail
        analysis={hot}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={vi.fn()}
        previewStale
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );
    const exportButton = container.querySelector<HTMLButtonElement>(
      "button.right-rail-export",
    );
    expect(exportButton?.disabled).toBe(false);
    expect(exportButton?.textContent).toContain("Export With Review");
    await act(async () => root.unmount());
  });

  it("mono, stereo, and long sources all reach a ready, exportable state", async () => {
    // Channel count and duration are source facts, not gates. The scenario
    // asks that none of them strands the user; the album channel-count
    // resolution itself is native behaviour and is proved in the Rust lane.
    for (const analysis of [
      analysisFor({ stereo_width: 0 }), // mono-ish source
      analysisFor({ stereo_width: 0.9 }), // wide stereo source
      analysisFor({ lufs_short_term_max: -3 }), // long/dynamic source
    ]) {
      const { container, root } = await renderNode(
        <RightRail
          analysis={analysis}
          lastChecks={undefined}
          canExport
          isExporting={false}
          isRendering={false}
          onExport={vi.fn()}
          previewStale
          canRenderPreview
          onUpdatePreview={vi.fn()}
        />,
      );
      const exportButton = container.querySelector<HTMLButtonElement>(
        "button.right-rail-export",
      );
      expect(exportButton?.disabled).toBe(false);
      await act(async () => root.unmount());
    }
  });
});

// ---------------------------------------------------------------------------
// S-F1
// ---------------------------------------------------------------------------
//
// INPUT   Complete Standard and Advanced single-track workflows, including
//         warnings and validation.
// ACTION  Inspect naming, selected-state announcement, warning readability,
//         and export collision safety.
// EXPECT  All controls named, selected states announced, warning details
//         readable, exports collision-safe.
describe("S-F1 — single-track workflow is named, announced, and readable", () => {
  it("warning details are readable as text, not only as colour and tooltip", async () => {
    const checks: QualityCheck[] = [
      {
        code: "true_peak_high",
        level: "warning",
        message: "True peak reached -0.2 dBTP, above the -1.0 dBTP ceiling.",
      },
    ];
    // 2026-08-18: export checks are presented in Source Insight (under the
    // track title), in its "Last export" group, once the disclosure is open.
    const { container, root } = await renderNode(
      <SourceInsight
        analysis={analysisFor({ true_peak_dbtp: -0.2 })}
        lastChecks={checks}
        unreviewed={false}
        onAcknowledge={vi.fn()}
      />,
    );
    await act(async () => {
      container
        .querySelector<HTMLButtonElement>(".source-insight-toggle")
        ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    // Severity and explanation both exist as text a screen reader will reach.
    expect(container.textContent).toContain("Caution.");
    expect(container.textContent).toContain(
      "True peak reached -0.2 dBTP, above the -1.0 dBTP ceiling.",
    );
    await act(async () => root.unmount());
  });

  it("the export gate announces itself as a decision, with both ways out", async () => {
    const onExport = vi.fn();
    const { container, root } = await renderNode(
      <RightRail
        analysis={analysisFor({ true_peak_dbtp: 0.3, dynamic_range_lu: 3 })}
        lastChecks={undefined}
        canExport
        isExporting={false}
        isRendering={false}
        onExport={onExport}
        previewStale
        canRenderPreview
        onUpdatePreview={vi.fn()}
      />,
    );
    const open = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.trim() === "Export With Review",
    ) as HTMLButtonElement | undefined;
    await act(async () => {
      open?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const dialog = container.querySelector(".export-review-panel");
    expect(dialog?.getAttribute("role")).toBe("dialog");
    const names = Array.from(container.querySelectorAll("button")).map((b) =>
      b.textContent?.trim(),
    );
    expect(names).toContain("Adjust Settings");
    expect(names).toContain("Export Anyway");
    // Opening the gate is not exporting.
    expect(onExport).not.toHaveBeenCalled();
    await act(async () => root.unmount());
  });

  // Standard's leg of S-F1 — Create Master naming, its disabled reasons, the
  // done card and its actions — is pinned in StandardView.test.tsx (37 tests),
  // and the accessible-name/selected-state sweep is App.a11y-semantics.test.tsx
  // (10 tests). Both run in the same lane; duplicating them here would add
  // runtime, not proof.
});

// ---------------------------------------------------------------------------
// S-F3
// ---------------------------------------------------------------------------
//
// INPUT   Export twice to the same intended destination, then save and reload
//         the project.
// ACTION  Observe the second export's suggested destination and the reloaded
//         project's state.
// EXPECT  Prior renders are not overwritten by default; receipt actions
//         resolve correctly; the restored project is usable.
describe("S-F3 — a second export does not land on the first", () => {
  it("remembers the export directory and re-suggests inside it", () => {
    const store = memoryStore();
    rememberExportDirectory(store, "track", "/Users/dan/Masters/song__master.wav");
    expect(lastExportDirectory(store, "track")).toBe("/Users/dan/Masters");
    // The next export starts in the same folder — which is exactly why the
    // collision check below has to exist.
    expect(defaultExportPath(store, "track", "song__master.wav")).toBe(
      "/Users/dan/Masters/song__master.wav",
    );
  });

  it("keeps track and album export locations separate", () => {
    // A shared key would make an album export silently retarget the next track
    // export into the album folder.
    const store = memoryStore();
    rememberExportDirectory(store, "track", "/Users/dan/Masters/song.wav");
    rememberExportDirectory(store, "album", "/Users/dan/Albums/Record");
    expect(lastExportDirectory(store, "track")).toBe("/Users/dan/Masters");
    expect(lastExportDirectory(store, "album")).toBe("/Users/dan/Albums/Record");
  });

  it("derives the directory from Windows and POSIX paths alike", () => {
    expect(exportDirectoryFromPath("C:\\Users\\dan\\Music\\take.wav")).toBe(
      "C:\\Users\\dan\\Music",
    );
    expect(exportDirectoryFromPath("/Users/dan/Music/take.wav")).toBe(
      "/Users/dan/Music",
    );
    expect(exportDirectoryFromPath("take.wav")).toBeNull();
  });

  // The collision check itself — first free of name.wav / name-2.wav / … — is
  // the backend's, and is proved on both sides of the boundary:
  //   frontend wiring   useTrackMaster.integration.test.tsx, "suggests a
  //                     collision-free filename when the remembered export dir
  //                     has a prior render" (plus its degrade-on-failure leg)
  //   native            src-tauri/src/exports.rs::suggest_export_filename unit
  //                     tests
  // Receipt-action resolution is pinned in ExportReceiptCard.test.tsx and
  // StandardView.test.tsx; save/reload recovery text in the open-project tests.
});
