import { invoke, listen } from "./tauri-runtime";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type {
  AlbumArc,
  AlbumPlan,
  AnalysisProgress,
  AnalysisResult,
  CompressionPlan,
  ExportReport,
  GuardrailReadout,
  ImportedTrack,
  LandingStatus,
  LoopRegion,
  MasteringSettings,
  PlaybackTick,
  PresetKind,
  ProjectState,
  QualityCheck,
  RenderJob,
  TrackId,
  UserPreset,
  WaveformPeaks,
} from "../bindings";

export const ADAPTIVE_COMPRESSION_GATE_EVENT = "yes-master:adaptive-compression-gate";

function publishAdaptiveCompressionGate(enabled: boolean) {
  if (typeof window === "undefined") return;
  window.dispatchEvent(
    new CustomEvent<boolean>(ADAPTIVE_COMPRESSION_GATE_EVENT, { detail: enabled }),
  );
}

/// Phase B: render_album_plan return shape.
export interface AlbumTrackRenderRecord {
  track_id: TrackId;
  position: number;
  output_path: string;
  measured_lufs: number;
  source_sample_rate: number;
  rendered_sample_rate: number;
  source_channels: number;
  rendered_channels: number;
}

export interface AlbumRenderReport {
  album_wav_path: string;
  manifest_path: string;
  requested_sample_rate: number | null;
  rendered_sample_rate: number;
  source_sample_rates: number[];
  bit_depth: number;
  rendered_channels: number;
  source_channels: number[];
  tracks: AlbumTrackRenderRecord[];
}

export interface AlbumTrackRenderInput {
  track_id: TrackId;
  source_path: string;
  settings: MasteringSettings;
}

// Tauri 2 auto-converts camelCase invoke arg keys to snake_case Rust parameter
// names. So `trackId` here lands as `track_id` in the Rust handler signature.
// Sending snake_case keys directly does NOT work — Tauri's command arg parser
// rejects them with "missing required key <camelCaseName>". Phase 11.3 fix.

export const api = {
  importTracks: (paths: string[]) =>
    invoke<ImportedTrack[]>("import_tracks", { paths }),

  analyzeTracks: (tracks: Array<{ id: TrackId; path: string }>, batchId?: string) =>
    invoke<AnalysisResult[]>("analyze_tracks", { tracks, batchId: batchId ?? null }),

  renderTrackPreview: (
    trackId: TrackId,
    trackPath: string,
    settings: MasteringSettings,
  ) =>
    invoke<RenderJob>("render_track_preview", {
      trackId,
      trackPath,
      settings,
    }),

  renderTrackMaster: (
    trackId: TrackId,
    trackPath: string,
    settings: MasteringSettings,
    outputPath?: string,
  ) =>
    invoke<RenderJob>("render_track_master", {
      trackId,
      trackPath,
      settings,
      outputPath: outputPath ?? null,
    }),

  prepareWaveform: (
    trackId: TrackId,
    trackPath: string,
    targetPixels?: number,
  ) =>
    invoke<WaveformPeaks>("prepare_waveform", {
      trackId,
      trackPath,
      targetPixels: targetPixels ?? null,
    }),

  runExportChecks: (
    report: ExportReport,
    sourceAnalysis?: AnalysisResult | null,
    settings?: MasteringSettings | null,
  ) =>
    invoke<QualityCheck[]>("run_export_checks", {
      report,
      sourceAnalysis: sourceAnalysis ?? null,
      settings: settings ?? null,
    }),

  openOutput: (outputPath: string) =>
    invoke<null>("open_output", { outputPath }),

  saveProject: (path: string, state: ProjectState) =>
    invoke<null>("save_project", { path, state }),

  autosaveSession: (state: ProjectState) =>
    invoke<null>("autosave_session", { state }),

  loadRecentSession: () =>
    invoke<ProjectState | null>("load_recent_session"),

  loadProject: (path: string) =>
    invoke<ProjectState>("load_project", { path }),

  saveUserPreset: (
    name: string,
    kind: PresetKind,
    settings: MasteringSettings,
  ) =>
    invoke<UserPreset>("save_user_preset", { name, kind, settings }),

  listUserPresets: () => invoke<UserPreset[]>("list_user_presets"),

  deleteUserPreset: (id: string) =>
    invoke<null>("delete_user_preset", { id }),

  evictSourceProfile: (trackId: TrackId) =>
    invoke<null>("evict_source_profile", { trackId }),

  // Phase B owner-calibration gate (runtime, no rebuild). Off by default; flip on
  // to A/B the confidence-gated adaptive voicing, then lock the constants by ear.
  setConfidenceGating: (enabled: boolean) =>
    invoke<boolean>("set_confidence_gating", { enabled }),

  confidenceGatingEnabled: () =>
    invoke<boolean>("confidence_gating_enabled"),

  // Adaptive Compressor owner-calibration gate. Default off: gate-on work is
  // available for listening sessions without changing the validated preset path.
  setAdaptiveCompression: async (enabled: boolean) => {
    const next = await invoke<boolean>("set_adaptive_compression", { enabled });
    publishAdaptiveCompressionGate(next);
    return next;
  },

  adaptiveCompressionEnabled: () =>
    invoke<boolean>("adaptive_compression_enabled"),

  playTrack: (
    trackId: TrackId,
    trackPath: string,
    startPositionSec?: number,
  ) =>
    invoke<null>("play_track", {
      trackId,
      trackPath,
      startPositionSec: startPositionSec ?? null,
    }),

  playMaster: (
    trackId: TrackId,
    trackPath: string,
    settings: MasteringSettings,
    startPositionSec?: number,
    previewLufsLanding = true,
    // B2: album mode is non-adaptive; the backend derives + injects the profile
    // and caches album-ness for the subsequent settings-only update_chain calls.
    album = false,
  ) =>
    invoke<null>("play_master", {
      trackId,
      trackPath,
      settings,
      startPositionSec: startPositionSec ?? null,
      previewLufsLanding,
      album,
    }),

  updateChain: (
    settings: MasteringSettings,
    previewLufsLanding = true,
    // B2 / live-mode-sync: album-ness of the CURRENT mode, sent with every live
    // edit so a Track<->Album switch mid-Mastered-audition resolves correctly
    // (album mode stays non-adaptive) without waiting for the next playMaster.
    album = false,
  ) =>
    invoke<null>("update_chain", { settings, previewLufsLanding, album }),

  /// Read-only per-axis adaptive-trim summary. B2: the backend resolves the
  /// profile from its store (keyed by trackId); album mode is non-adaptive. The
  /// FE sends raw settings + the track id, not a pre-injected profile.
  guardrailReadout: (
    settings: MasteringSettings,
    trackId?: TrackId | null,
    album = false,
  ) =>
    invoke<GuardrailReadout>("guardrail_readout", {
      settings,
      trackId: trackId ?? null,
      album,
    }),

  resolveCompressionPlan: (
    settings: MasteringSettings,
    trackId?: TrackId | null,
    album = false,
  ) =>
    invoke<CompressionPlan>("resolve_compression_plan", {
      settings,
      trackId: trackId ?? null,
      album,
    }),

  /// Prewarm the backend decode cache for a track. Fire-and-forget
  /// from track-select / track-import handlers so the PCM is ready
  /// by the time the user clicks Mastered (eliminates the 1-2 s
  /// freeze on first click for long WAVs). Idempotent; safe to
  /// call repeatedly on the same track.
  prewarmDecode: (trackPath: string) =>
    invoke<null>("prewarm_decode", { trackPath }),

  pausePlayback: () => invoke<null>("pause_playback"),
  resumePlayback: () => invoke<null>("resume_playback"),
  stopPlayback: () => invoke<null>("stop_playback"),
  seekPlayback: (positionSec: number) =>
    invoke<null>("seek_playback", { positionSec }),
  setLoopRegion: (region: LoopRegion | null) =>
    invoke<null>("set_loop_region", { region }),

  // Phase B — album-mode planning + render.
  planAlbum: (
    title: string,
    analyses: AnalysisResult[],
    durations: number[],
    arc: AlbumArc,
    intensity: number,
    deliverySampleRate?: number | null,
    deliveryBitDepth?: number | null,
  ) =>
    invoke<AlbumPlan>("plan_album", {
      request: {
        title,
        analyses,
        durations,
        arc,
        intensity,
        delivery_sample_rate: deliverySampleRate ?? null,
        delivery_bit_depth: deliveryBitDepth ?? null,
      },
    }),

  renderAlbumPlan: (
    plan: AlbumPlan,
    tracks: AlbumTrackRenderInput[],
    outputDir?: string,
  ) =>
    invoke<AlbumRenderReport>("render_album_plan", {
      request: { plan, tracks },
      outputDir: outputDir ?? null,
    }),
};

export function onPlaybackTick(
  handler: (tick: PlaybackTick) => void,
): Promise<UnlistenFn> {
  return listen<PlaybackTick>("playback:tick", (event) => handler(event.payload));
}

export interface RenderProgressEvent {
  track_id: TrackId;
  kind: "preview" | "master" | "album";
  fraction: number;
}

export function onRenderProgress(
  handler: (event: RenderProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<RenderProgressEvent>("render:progress", (event) =>
    handler(event.payload),
  );
}

/// Real analysis progress: stage callbacks from the ACTUAL phase boundaries
/// inside the backend analyzer (decode → dynamics → stereo → tonal → deep
/// scan), batch-rescaled to 0..1. Replaces the paced-timer stage display
/// whenever events arrive.
export type AnalysisProgressEvent = AnalysisProgress;

export function onAnalysisProgress(
  handler: (event: AnalysisProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<AnalysisProgressEvent>("analysis:progress", (event) =>
    handler(event.payload),
  );
}

/// Edge-triggered: true while Mastered audition is playing hotter than the
/// loudness target because the corrective landing gain is still being
/// measured; false once it crossfades in. Drives the "landing loudness…"
/// note by the meters.
export type LandingStatusEvent = LandingStatus;

export function onLandingStatus(
  handler: (pending: boolean, event: LandingStatusEvent) => void,
): Promise<UnlistenFn> {
  return listen<LandingStatusEvent>("landing:status", (event) =>
    handler(event.payload.pending, event.payload),
  );
}
