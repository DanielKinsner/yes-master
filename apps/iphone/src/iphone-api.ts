import {
  convertFileSrc,
  invoke as tauriInvoke,
  isTauri,
} from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AnalysisResult,
  ExportReport,
  ImportedTrack,
  MasteringSettings,
  PlaybackTick,
  QualityCheck,
  RenderJob,
  WaveformPeaks,
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
  prepareWaveform(
    trackId: string,
    trackPath: string,
    targetPixels?: number,
  ): Promise<WaveformPeaks>;
  playOriginal(
    trackId: string,
    trackPath: string,
    startPositionSec?: number,
  ): Promise<void>;
  playMastered(
    trackId: string,
    trackPath: string,
    settings: MasteringSettings,
    startPositionSec?: number,
    previewLufsLanding?: boolean,
  ): Promise<void>;
  updateMasteringChain(
    settings: MasteringSettings,
    previewLufsLanding?: boolean,
  ): Promise<void>;
  pausePlayback(): Promise<void>;
  resumePlayback(): Promise<void>;
  stopPlayback(): Promise<void>;
  seekPlayback(positionSec: number): Promise<void>;
  onPlaybackTick(handler: (tick: PlaybackTick) => void): Promise<UnlistenFn>;
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

    prepareWaveform: (trackId, trackPath, targetPixels) =>
      invoke<WaveformPeaks>("iphone_prepare_waveform", {
        trackId,
        trackPath,
        targetPixels,
      }),

    playOriginal: (trackId, trackPath, startPositionSec) =>
      invoke<void>("iphone_play_track", {
        trackId,
        trackPath,
        startPositionSec: startPositionSec ?? null,
      }),

    playMastered: (
      trackId,
      trackPath,
      settings,
      startPositionSec,
      previewLufsLanding = true,
    ) =>
      invoke<void>("iphone_play_master", {
        trackId,
        trackPath,
        settings,
        startPositionSec: startPositionSec ?? null,
        previewLufsLanding,
      }),

    updateMasteringChain: (settings, previewLufsLanding = true) =>
      invoke<void>("iphone_update_chain", {
        settings,
        previewLufsLanding,
      }),

    pausePlayback: () => invoke<void>("iphone_pause_playback"),
    resumePlayback: () => invoke<void>("iphone_resume_playback"),
    stopPlayback: () => invoke<void>("iphone_stop_playback"),
    seekPlayback: (positionSec) =>
      invoke<void>("iphone_seek_playback", {
        positionSec,
      }),
    onPlaybackTick: (handler) =>
      listen<PlaybackTick>("playback:tick", (event) => handler(event.payload)),

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

    async prepareWaveform(trackId, _trackPath, targetPixels = 140) {
      return syntheticIphoneWaveform(trackId, targetPixels);
    },

    async playOriginal() {},
    async playMastered() {},
    async updateMasteringChain() {},
    async pausePlayback() {},
    async resumePlayback() {},
    async stopPlayback() {},
    async seekPlayback() {},
    async onPlaybackTick() {
      return () => {};
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
  // No save() dialog on iOS: the dialog plugin writes a 0-byte placeholder and
  // exports it BEFORE the render bytes exist, so the user gets an empty file.
  // Return the suggested filename; iphone_render_master lands it in the app's
  // Files-visible Documents/YES Master folder. (Browser preview also just uses
  // the name.)
  return defaultPath;
}

export function toIphoneAudioUrl(path: string): string {
  if (path.startsWith("blob:") || path.startsWith("data:")) return path;
  return convertFileSrc(path);
}

function hasIphoneNativeRuntime(): boolean {
  if (isTauri()) return true;

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

function syntheticIphoneWaveform(
  trackId: string,
  targetPixels: number,
): WaveformPeaks {
  const count = Math.max(48, Math.min(180, targetPixels));
  const channel: number[] = [];
  for (let index = 0; index < count; index += 1) {
    const position = index / Math.max(1, count - 1);
    const phrase = Math.sin(position * Math.PI);
    const pulse = Math.sin(position * Math.PI * 13) * 0.22;
    const fine = Math.sin(position * Math.PI * 41) * 0.08;
    channel.push(Math.max(0.08, Math.min(0.94, phrase * 0.72 + pulse + fine)));
  }
  return {
    track_id: trackId,
    channels: [channel],
    samples_per_pixel: 512,
    total_samples: count * 512,
    sample_rate: 44_100,
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
