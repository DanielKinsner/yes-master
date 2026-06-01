import { convertFileSrc, invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  AnalysisResult,
  ExportReport,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "../../../src/bindings";
import { buildIphoneSimplePlan } from "./simple-mode";

export type IphoneInvoke = <T>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

export interface IphoneRenderRequest {
  trackId: string;
  trackPath: string;
  settings: MasteringSettings;
  outputPath: string;
}

export interface IphonePreviewRequest {
  trackId: string;
  trackPath: string;
  settings: MasteringSettings;
}

export interface IphoneBackend {
  importTrack(path: string): Promise<ImportedTrack>;
  analyzeTrack(trackId: string, path: string): Promise<AnalysisResult>;
  renderMaster(request: IphoneRenderRequest): Promise<RenderJob>;
  prepareMasterPreview(request: IphonePreviewRequest): Promise<RenderJob>;
  runExportChecks(
    report: ExportReport,
    sourceAnalysis: AnalysisResult | null,
    settings: MasteringSettings | null,
  ): Promise<QualityCheck[]>;
}

interface BrowserAudioImport {
  file: File;
  sourceFormat: string;
}

const browserAudioImports = new Map<string, BrowserAudioImport>();
const browserPreviewSettings = buildIphoneSimplePlan().exportSettings;

export function createIphoneBackend(invoke: IphoneInvoke): IphoneBackend {
  return {
    importTrack: (path) =>
      invoke<ImportedTrack>("iphone_import_track", {
        path,
      }),

    analyzeTrack: (trackId, path) =>
      invoke<AnalysisResult>("iphone_analyze_track", {
        trackId,
        path,
      }),

    renderMaster: ({ trackId, trackPath, settings, outputPath }) =>
      invoke<RenderJob>("iphone_render_master", {
        trackId,
        trackPath,
        settings,
        outputPath,
      }),

    prepareMasterPreview: ({ trackId, trackPath, settings }) =>
      invoke<RenderJob>("iphone_prepare_master_preview", {
        trackId,
        trackPath,
        settings,
      }),

    runExportChecks: (report, sourceAnalysis, settings) =>
      invoke<QualityCheck[]>("iphone_run_export_checks", {
        report,
        sourceAnalysis,
        settings,
      }),
  };
}

export function createBrowserPreviewIphoneBackend(): IphoneBackend {
  return {
    async importTrack(path) {
      const imported = browserAudioImports.get(path);
      const displayName = imported?.file.name ?? fileNameFromPath(path);
      return {
        id: `browser-${hashBrowserPath(path)}`,
        path,
        display_name: displayName,
        source_format: imported?.sourceFormat ?? extensionFromName(displayName),
        duration_seconds: null,
        sample_rate: null,
        channels: null,
      };
    },

    async analyzeTrack(trackId) {
      return {
        track_id: trackId,
        lufs_integrated: -14.6,
        lufs_short_term_max: -10.2,
        true_peak_dbtp: -4.0,
        dynamic_range_lu: 5.2,
        spectral_balance: { low: 0.32, mid: 0.42, high: 0.26 },
        transient_density: 0.55,
        stereo_width: 1,
        recommended_universal: browserPreviewSettings,
        measured_at_iso: new Date().toISOString(),
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
    },

    async renderMaster({ trackId, outputPath, settings }) {
      return browserRenderJob(trackId, "master", outputPath, settings);
    },

    async prepareMasterPreview({ trackId, trackPath, settings }) {
      return browserRenderJob(trackId, "preview", trackPath, settings);
    },

    async runExportChecks() {
      return [];
    },
  };
}

export function createDefaultIphoneBackend(): IphoneBackend {
  return hasIphoneNativeRuntime()
    ? createIphoneBackend(tauriInvoke)
    : createBrowserPreviewIphoneBackend();
}

export const iphoneBackend = createDefaultIphoneBackend();

export async function pickIphoneAudioPath(): Promise<string | null> {
  if (!hasIphoneNativeRuntime()) {
    return pickBrowserAudioPath();
  }

  const selected = await open({
    directory: false,
    multiple: false,
    pickerMode: "document",
    fileAccessMode: "copy",
    title: "Import audio",
    filters: [
      {
        name: "Audio",
        extensions: ["wav", "aiff", "aif", "flac", "mp3", "m4a", "aac", "ogg", "opus"],
      },
    ],
  });
  return Array.isArray(selected) ? selected[0] ?? null : selected;
}

export async function pickIphoneOutputPath(
  defaultPath = "YES-Master.wav",
): Promise<string | null> {
  if (!hasIphoneNativeRuntime()) {
    return defaultPath;
  }

  return save({
    title: "Export master",
    defaultPath,
    filters: [
      {
        name: "WAV",
        extensions: ["wav"],
      },
    ],
  });
}

export function toIphoneAudioUrl(path: string): string {
  if (path.startsWith("blob:") || path.startsWith("data:")) return path;
  return convertFileSrc(path);
}

function hasIphoneNativeRuntime(): boolean {
  return Boolean(
    typeof window !== "undefined" &&
      (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__,
  );
}

function pickBrowserAudioPath(): Promise<string | null> {
  if (typeof document === "undefined") return Promise.resolve(null);

  return new Promise((resolve) => {
    const input = document.createElement("input");
    let resolved = false;

    function finish(path: string | null) {
      if (resolved) return;
      resolved = true;
      input.remove();
      resolve(path);
    }

    input.type = "file";
    input.accept = [
      ".wav",
      ".aiff",
      ".aif",
      ".flac",
      ".mp3",
      ".m4a",
      ".aac",
      ".ogg",
      ".opus",
      "audio/*",
    ].join(",");
    input.style.display = "none";
    input.addEventListener(
      "change",
      () => {
        const file = input.files?.[0] ?? null;
        if (!file) {
          finish(null);
          return;
        }
        const path = URL.createObjectURL(file);
        browserAudioImports.set(path, {
          file,
          sourceFormat: extensionFromName(file.name),
        });
        finish(path);
      },
      { once: true },
    );
    input.addEventListener("cancel", () => finish(null), { once: true });
    document.body.appendChild(input);
    input.click();
  });
}

function browserRenderJob(
  trackId: string,
  kind: "preview" | "master",
  outputPath: string,
  settings: { advanced?: { bit_depth?: number | null; target_sample_rate?: number | null } },
): RenderJob {
  return {
    id: `browser-${kind}-${Date.now()}`,
    kind,
    target_tracks: [trackId],
    status: { status: "done" },
    progress: 1,
    started_at_iso: new Date().toISOString(),
    output_paths: [outputPath],
    measurements: {
      lufs_integrated: -14,
      true_peak_dbtp: -1,
      dynamic_range_lu: 5,
      sample_rate: settings.advanced?.target_sample_rate ?? 48_000,
      bit_depth: settings.advanced?.bit_depth ?? 24,
    },
  };
}

function fileNameFromPath(path: string): string {
  return path.split(/[\\/]/).pop() ?? "Imported track";
}

function extensionFromName(name: string): string {
  const extension = name.split(".").pop();
  return extension && extension !== name ? extension.toLowerCase() : "audio";
}

function hashBrowserPath(path: string): string {
  let hash = 0;
  for (const char of path) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return hash.toString(36);
}
