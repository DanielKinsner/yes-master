import { invoke, listen } from "./tauri-runtime";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type {
  AlbumArc,
  AlbumPlan,
  AnalysisProgress,
  AnalysisResult,
  AudioOutputDevice,
  CompressionPlan,
  ExportReport,
  GuardrailReadout,
  ImportedTrack,
  JobStatus,
  LandingStatus,
  LoopRegion,
  MasteringSettings,
  PlaybackDeviceLost,
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
  /** Absent in older reports; null means no loudness target was requested. */
  target_lufs?: number | null;
  true_peak_dbtp?: number;
  ceiling_dbtp?: number;
  source_sample_rate: number;
  rendered_sample_rate: number;
  source_channels: number;
  rendered_channels: number;
  /** True when the track rendered with its own settings/target instead of
   * the album intent (Override toggle) — mirrored into the manifest. */
  override_album: boolean;
}

export interface AlbumRenderReport {
  job_id: string;
  status: JobStatus;
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
  /** D9 full sound exemption: render with the track's own settings/target —
   * no arc offset, no character bias — while keeping album delivery format. */
  override_album: boolean;
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

  listAudioOutputDevices: () =>
    invoke<AudioOutputDevice[]>("list_audio_output_devices"),

  setAudioOutputDevice: (deviceId: string | null) =>
    invoke<null>("set_audio_output_device", { deviceId }),

  // Dismisses the device-loss banner on the backend too — without this the
  // next playback tick re-carries device_lost and revives the banner.
  clearDeviceLost: () => invoke<null>("clear_device_lost"),
  // Slice 7b: download + install the available update and relaunch. Fired ONLY
  // by the user clicking the update toast's action (never automatically).
  installUpdate: () => invoke<null>("install_update"),
  // Audit L-02: the backend latches the available version before emitting the
  // edge-triggered startup event, so a frontend that attached its listener too
  // late can still recover the notice by query. Reading never consumes.
  availableUpdateVersion: () => invoke<string | null>("available_update_version"),
  // Audit L-03: manual recovery when an install fails. The backend opens the
  // FIXED GitHub Releases index (src/lib/release-links.ts) — the command takes
  // no URL, so the frontend cannot steer the opener anywhere else.
  openReleasePage: () => invoke<null>("open_release_page"),

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

  // Collision-free export suggestion: first free of <name>.wav, <name>-2.wav, …
  // in the remembered export directory ("never overwrite by default" is the
  // app's guard, not the OS replace prompt's).
  suggestExportFilename: (directory: string, fileName: string) =>
    invoke<string>("suggest_export_filename", { directory, fileName }),

  // "What am I running?" — version · git hash · build time, stamped into the
  // binary at compile time. Shown in the Help dialog.
  buildInfo: () => invoke<string>("build_info"),

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

  /// Write a plain-text diagnostics report (recent logs + session summary,
  /// assembled locally — nothing is transmitted) to a user-chosen path.
  /// Returns the written path.
  /// Pass 4 (2026-08-19): synthesise (once) and return the demo track path.
  prepareDemoTrack: () => invoke<string>("prepare_demo_track"),
  saveDiagnosticsReport: (targetPath: string) =>
    invoke<string>("save_diagnostics_report", { targetPath }),

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

  cancelRender: (jobId: string) =>
    invoke<null>("cancel_render", {
      jobId,
    }),
};

export function onPlaybackTick(
  handler: (tick: PlaybackTick) => void,
): Promise<UnlistenFn> {
  return listen<PlaybackTick>("playback:tick", (event) => handler(event.payload));
}

export function onPlaybackDeviceLost(
  handler: (event: PlaybackDeviceLost) => void,
): Promise<UnlistenFn> {
  return listen<PlaybackDeviceLost>("playback:device-lost", (event) =>
    handler(event.payload),
  );
}

export interface RenderProgressEvent {
  job_id: string;
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

/// Edge-triggered: requested preview landing or Volume Match is being measured.
/// Drives the "Measuring preview level…" note, including while paused.
export type LandingStatusEvent = LandingStatus;

export function onLandingStatus(
  handler: (pending: boolean, event: LandingStatusEvent) => void,
): Promise<UnlistenFn> {
  return listen<LandingStatusEvent>("landing:status", (event) =>
    handler(event.payload.pending, event.payload),
  );
}

/// Slice 7b: fires when the backend's startup check finds a newer release.
/// Payload is the new version string.
export function onUpdaterAvailable(
  handler: (version: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("updater:available", (event) => handler(event.payload));
}
