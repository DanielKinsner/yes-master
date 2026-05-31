import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  AnalysisResult,
  ExportReport,
  ImportedTrack,
  MasteringSettings,
  QualityCheck,
  RenderJob,
} from "../../../src/bindings";

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

export const iphoneBackend = createIphoneBackend(tauriInvoke);

export async function pickIphoneAudioPath(): Promise<string | null> {
  const selected = await open({
    directory: false,
    multiple: false,
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

export async function pickIphoneOutputPath(): Promise<string | null> {
  return save({
    title: "Export master",
    defaultPath: "YES-Master.wav",
    filters: [
      {
        name: "WAV",
        extensions: ["wav"],
      },
    ],
  });
}
