import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { open, save, getCurrentWebview } from "../lib/tauri-runtime";
import { rememberView, rememberedView } from "../lib/view-by-track";
import {
  ADAPTIVE_COMPRESSION_GATE_EVENT,
  api,
  onAnalysisProgress,
  onLandingStatus,
  onPlaybackDeviceLost,
  onPlaybackTick,
  onRenderProgress,
} from "../lib/api";
import {
  browserExportLocationStore,
  defaultExportPath,
  lastExportDirectory,
  rememberExportDirectory,
} from "../lib/export-location";
import {
  applyAdvancedWithProfileFlip,
  applyChainDispatchOverrides,
  applyDeliveryProfileSelection,
  applyExplicitLoudnessTarget,
  applyLoudnessTargetSelection,
} from "../lib/settings-transitions";
import {
  appendToPast,
  applyRedo,
  applyUndo,
  shouldCoalesceCommit,
} from "../lib/history-stack";
import { resetToneSettings } from "../lib/tone-reset";
import {
  resetForStandardReturn as resetForStandardReturnSettings,
  resetToStandardManaged as resetToStandardManagedSettings,
} from "../lib/standard-managed";
import { standardExportSettings } from "../lib/standard-export";
import { buildExportReport } from "../lib/export-receipt";
import { userErrorMessage, type UserErrorContext } from "../lib/user-errors";
import {
  AUDIO_DIALOG_FILTER,
  SUPPORTED_FORMATS_COPY,
  supportedAudioExtensionFromName,
} from "../lib/supported-formats";
import type {
  AdvancedSettings,
  AlbumArcKind,
  AnalysisResult,
  CompressionPlan,
  GuardrailReadout,
  ImportedTrack,
  LoopRegion,
  MasteringSettings,
  PlaybackDeviceLost,
  Preset,
  PresetKind,
  ProjectMode,
  ProjectState,
  QualityCheck,
  RenderJob,
  TrackId,
  UserPreset,
  ViewMode,
  WaveformPeaks,
} from "../bindings";
import { EQ_BAND_DEFAULTS, EQ_BAND_RANGES } from "../bindings";

// The autosave and Save-As paths must serialize the SAME project snapshot.
// These were two byte-identical literals edited in lockstep — a real
// autosave/save-as drift hazard once a field lands in only one of them.
function buildProjectState(args: {
  mode: ProjectMode;
  tracks: ImportedTrack[];
  settingsMap: ProjectState["track_settings"];
  albumIntent: ProjectState["album_intent"];
  albumArcKind: AlbumArcKind;
  albumIntensity: number;
  albumTitle: string;
  albumSampleRate: number | null;
  albumBitDepth: number | null;
  overrideAlbum: Iterable<TrackId>;
  selectedTrackId: TrackId | null;
  viewByTrackId: Record<TrackId, ViewMode>;
}): ProjectState {
  return {
    schema_version: 1,
    mode: args.mode,
    tracks: args.tracks,
    track_order: args.tracks.map((t) => t.id),
    track_settings: args.settingsMap,
    album_intent: args.albumIntent,
    album_arc_kind: args.albumArcKind,
    album_intensity: args.albumIntensity,
    album_title: args.albumTitle,
    album_sample_rate: args.albumSampleRate,
    album_bit_depth: args.albumBitDepth,
    track_override_album: Array.from(args.overrideAlbum),
    selected_track_id: args.selectedTrackId,
    view_by_track_id: args.viewByTrackId,
    last_saved_iso: new Date().toISOString(),
  };
}

/// Band id (as the UI names it) → the `eq_bands` key it edits.
const EQ_BAND_KEY = {
  sub: "sub_hz",
  low: "low_hz",
  "low-mid": "low_mid_hz",
  mid: "mid_hz",
  "high-mid": "high_mid_hz",
  high: "high_hz",
  sparkle: "sparkle_hz",
} as const;

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
  eq_bands: { ...EQ_BAND_DEFAULTS },
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
    // Explicit so a fresh track is honestly "on at 0.5" rather than a null that
    // displays as 50% but reads ambiguously (B4). Durable "off" = 0.0.
    adaptive_strength: 0.5,
  },
};

function suggestedMasterFilename(track: ImportedTrack): string {
  const withoutExtension = track.display_name.replace(/\.[^/.]+$/, "");
  const safeBase =
    withoutExtension.replace(/[^a-z0-9-_]+/gi, "_").replace(/^_+|_+$/g, "") ||
    "master";
  return `${safeBase}__master.wav`;
}

function ensureWavExtension(path: string): string {
  return /\.wav$/i.test(path) ? path : `${path}.wav`;
}

function projectDisplayName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

function extensionLabel(path: string): string {
  const name = path.split(/[\\/]/).filter(Boolean).pop() ?? path;
  const ext = name.includes(".") ? name.split(".").pop()?.trim() : "";
  return ext ? ext.toUpperCase() : "no extension";
}

function unsupportedDropMessage(paths: string[]): string {
  const rejected = Array.from(new Set(paths.map(extensionLabel))).join(", ");
  return `Unsupported file type${paths.length === 1 ? "" : "s"}: ${rejected}. Supported audio: ${SUPPORTED_FORMATS_COPY}.`;
}

function missingAnalysisTracks(
  targetTracks: ImportedTrack[],
  results: AnalysisResult[],
): ImportedTrack[] {
  const analyzedIds = new Set(results.map((result) => result.track_id));
  return targetTracks.filter((track) => !analyzedIds.has(track.id));
}

function analysisGapSummary(missingTracks: ImportedTrack[]): string {
  const count = missingTracks.length;
  const names = missingTracks
    .map((track) => track.display_name || projectDisplayName(track.path))
    .join(", ");
  return `${count} track${count === 1 ? "" : "s"} still ${
    count === 1 ? "needs" : "need"
  } analysis: ${names}`;
}

async function chooseAlbumExportFolder(): Promise<string | null> {
  const store = browserExportLocationStore();
  const selected = await open({
    directory: true,
    defaultPath: lastExportDirectory(store, "album") ?? undefined,
    multiple: false,
    title: "Choose album export folder",
  });
  const outputDir = Array.isArray(selected) ? selected[0] ?? null : selected;
  if (outputDir) rememberExportDirectory(store, "album", outputDir);
  return outputDir;
}

export type PlaybackKindUI = "source" | "master";

export interface ExportReceipt {
  trackId: TrackId;
  outputPath: string;
  checks: QualityCheck[];
  job: RenderJob;
  kind: "track";
}

export interface ProjectFeedback {
  tone: "ok" | "info" | "warn";
  message: string;
}

type RenderProgressKind = "preview" | "master" | "album";

export interface RenderProgressState {
  job_id: string;
  fraction: number;
  kind: RenderProgressKind;
}

export interface RenderFeedback {
  kind: RenderProgressKind;
  message: string;
}

function isCancelledStatus(status: { status: string }): boolean {
  return status.status === "cancelled";
}

function cancelledRenderMessage(kind: RenderProgressKind): string {
  if (kind === "album") return "Album export cancelled. No files were written.";
  if (kind === "preview") return "Audit render cancelled. No file was written.";
  return "Export cancelled. No file was written.";
}

export function shouldPushLiveChainForSettingsEdit({
  trackId,
  editingAlbumIntent,
  loadedTrackId,
  loadedKindByTrack,
  overrideAlbum,
}: {
  trackId: TrackId | null;
  editingAlbumIntent: boolean;
  loadedTrackId: TrackId | null;
  loadedKindByTrack: Record<TrackId, PlaybackKindUI>;
  overrideAlbum: Set<TrackId>;
}): boolean {
  if (editingAlbumIntent) {
    return Object.entries(loadedKindByTrack).some(
      ([id, kind]) => kind === "master" && !overrideAlbum.has(id as TrackId),
    );
  }
  if (!trackId) return false;
  const kindForTrack = loadedKindByTrack[trackId];
  if (kindForTrack === "source") return false;
  if (kindForTrack === "master") return true;
  return loadedTrackId === trackId;
}

/// Normalize a caught value to its display message. Errors expose `.message`
/// (no `"Error: "` prefix); anything else stringifies. Used by every catch
/// that calls `setError`, so thrown `Error`s and raw string rejections render
/// identically.
export function messageOf(err: unknown, context?: UserErrorContext): string {
  return userErrorMessage(err, context);
}

export function playbackErrorMessage(
  err: unknown,
  kind: PlaybackKindUI,
): string {
  const raw = String(err);
  if (
    kind === "master" &&
    /audio thread reply timeout|mastered preview did not become ready/i.test(raw)
  ) {
    return "Mastered preview is still preparing for this file. Wait a moment and try Mastered again, or export the master directly.";
  }
  return raw;
}

const MASTERED_REQUIRES_ANALYSIS_MESSAGE =
  "Analyze this track before using Mastered playback.";

const EOF_RESTART_EPSILON_SEC = 0.05;

function isPausedAtEffectiveEnd(
  positionSec: number,
  durationSec: number,
  isPlaying: boolean,
): boolean {
  if (!Number.isFinite(durationSec) || isPlaying) return false;
  if (durationSec <= EOF_RESTART_EPSILON_SEC) {
    return positionSec >= durationSec;
  }
  return positionSec >= durationSec - EOF_RESTART_EPSILON_SEC;
}

let analysisBatchSeq = 0;

function nextAnalysisBatchId(): string {
  analysisBatchSeq += 1;
  return `analysis-${Date.now()}-${analysisBatchSeq}`;
}

const ANALYSIS_PROGRESS_STAGES = [
  { label: "Analyzing audio", progress: 0.14 },
  { label: "Reading tonal balance", progress: 0.32 },
  { label: "Checking dynamics", progress: 0.5 },
  { label: "Evaluating stereo field", progress: 0.66 },
  { label: "Building mastering context", progress: 0.82 },
  { label: "Preparing preview", progress: 0.94 },
] as const;

export function useTrackMaster() {
  const [tracks, setTracks] = useState<ImportedTrack[]>([]);
  const [selectedTrackId, setSelectedTrackId] = useState<TrackId | null>(null);
  // F6: per-track remembered view (Standard/Advanced). Only explicit user
  // choices are written here (see rememberTrackView); persisted in ProjectState.
  const [viewByTrackId, setViewByTrackId] = useState<Record<TrackId, ViewMode>>({});
  const [analysisMap, setAnalysisMap] = useState<Record<TrackId, AnalysisResult>>({});
  const [waveformMap, setWaveformMap] = useState<Record<TrackId, WaveformPeaks>>({});
  const [settingsMap, setSettingsMap] = useState<Record<TrackId, MasteringSettings>>({});
  const [staleSet, setStaleSet] = useState<Set<TrackId>>(new Set());
  // Which tracks are in an in-flight analysis batch (2026-08-19): the
  // "analyzing" flag is per TRACK. The selected track's pill / Insight card /
  // waveform slot follow ITS batch; another track's batch does not light
  // them over a finished result.
  const [analyzingTrackIds, setAnalyzingTrackIds] = useState<TrackId[]>([]);
  const [analysisStageIndex, setAnalysisStageIndex] = useState(0);
  const [isLoadingWaveform, setIsLoadingWaveform] = useState(false);
  const [isRendering, setIsRendering] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [projectFeedback, setProjectFeedback] = useState<ProjectFeedback | null>(null);
  const [transport, setTransport] = useState({
    isPlaying: false,
    currentTimeSec: 0,
    playbackKind: "source" as PlaybackKindUI,
    loop: false,
    deviceLost: false,
    volumeMatch: false,
    exportLufsPreview: false,
    // Phase 12.2 live clipping meter — post-output-gain peak since the last
    // tick, in dBFS. -120 means "no signal" (silence sentinel from backend).
    // Stored here so the StaleBar's indicator can flash red on clipping
    // without DevTools or an export round-trip.
    peakDbfs: -120,
    // Per-channel post-output peak (dBFS) for the stereo MASTER OUT meter.
    // -120 = silence sentinel; mono sources mirror the same value on both.
    peakLeftDbfs: -120,
    peakRightDbfs: -120,
    // Phase 12.2 per-band compressor GR readouts. -120 = silence sentinel
    // ("no reduction in the window"). Driven by PlaybackTick → snapshot →
    // atomic-swap on the backend audio thread.
    compressionGr: { low: -120, mid: -120, high: -120 },
    // Phase 12.2 P3 — live BS.1770 momentary LUFS. -120 = silence sentinel.
    lufsMomentary: -120,
    // Phase 12.2 P3+ — live BS.1770-4 integrated LUFS over the current
    // playback session.  Resets when a new playback starts.
    lufsIntegrated: -120,
    // L4b — live FFT spectrum, log-binned dB values. Empty array
    // means no spectrum yet (idle / Original playback / pre-L4b
    // backend). The frontend EQ panel uses this to draw bars under
    // the response curve.
    spectrumDb: [] as number[],
  });
  const [lastExportReceipt, setLastExportReceipt] = useState<ExportReceipt | null>(null);
  const [mode, setMode] = useState<ProjectMode>("track");
  const [albumIntent, setAlbumIntent] = useState<MasteringSettings>(DEFAULT_SETTINGS);
  const [sessionLoaded, setSessionLoaded] = useState(false);
  const [hadPriorSession, setHadPriorSession] = useState<boolean | null>(null);
  const [overrideAlbum, setOverrideAlbum] = useState<Set<TrackId>>(new Set());
  // Phase B — Album Master mode controls. Stored on the hook because the
  // AlbumPlan is rebuilt at export time from current tracks + analyses +
  // arc + intensity + delivery choices.
  const [albumArcKind, setAlbumArcKind] = useState<AlbumArcKind>("cinematic");
  const [albumIntensity, setAlbumIntensityState] = useState<number>(1.0);
  const [albumTitle, setAlbumTitle] = useState<string>("");
  const setAlbumArc = useCallback((kind: AlbumArcKind) => setAlbumArcKind(kind), []);
  const setAlbumIntensity = useCallback((v: number) => {
    setAlbumIntensityState(Math.max(0, Math.min(2, v)));
  }, []);
  // Album delivery format. `null` = Auto (backend resolves: rate = highest
  // source rate, bit depth = first-track effective).
  const [albumSampleRate, setAlbumSampleRateState] = useState<number | null>(
    null,
  );
  const [albumBitDepth, setAlbumBitDepthState] = useState<number | null>(null);
  const setAlbumSampleRate = useCallback(
    (v: number | null) => setAlbumSampleRateState(v),
    [],
  );
  const setAlbumBitDepth = useCallback(
    (v: number | null) => setAlbumBitDepthState(v),
    [],
  );
  const [userPresets, setUserPresets] = useState<UserPreset[]>([]);
  const [savingPreset, setSavingPreset] = useState(false);
  const [isDragOver, setIsDragOver] = useState(false);
  const [loadedTrackId, setLoadedTrackId] = useState<TrackId | null>(null);
  const [loadedKindByTrack, setLoadedKindByTrack] = useState<Record<TrackId, PlaybackKindUI>>({});
  const [playbackDeviceLost, setPlaybackDeviceLost] = useState<PlaybackDeviceLost | null>(null);
  const [regionByTrack, setRegionByTrack] = useState<Record<TrackId, LoopRegion | null>>({});
  // Phase 12.1 render progress: backend emits "render:progress" with a 0-1
  // fraction during render_track_preview / render_track_master. Used to
  // render a real progress bar instead of an indeterminate "Rendering…".
  const [renderProgress, setRenderProgress] = useState<RenderProgressState | null>(null);
  const [renderFeedback, setRenderFeedback] = useState<RenderFeedback | null>(null);
  const [cancelRequestedJobId, setCancelRequestedJobId] = useState<string | null>(null);
  const clearIncompleteRenderProgress = useCallback(
    (kind: RenderProgressKind) => {
      setRenderProgress((prev) =>
        prev?.kind === kind && prev.fraction < 1.0 ? null : prev,
      );
    },
    [],
  );
  // "Landing loudness…": true while Mastered audition plays hotter than the
  // loudness target because the corrective landing gain is still being
  // measured in the background. Edge-triggered from the backend.
  const [landingPending, setLandingPending] = useState(false);
  // Real analysis progress from the backend's "analysis:progress" events
  // (actual phase boundaries). The paced-timer stages below remain only as
  // a fallback for the moments before the first real event lands.
  const [realAnalysisProgress, setRealAnalysisProgress] = useState<{
    label: string;
    progress: number;
  } | null>(null);
  const isAnyAnalyzing = analyzingTrackIds.length > 0;
  // Per-track (2026-08-19): true only while the SELECTED track is in an
  // in-flight batch. Consumers (header pill, Insight card, waveform slot,
  // Standard) all speak about the selected track.
  const isAnalyzing =
    selectedTrackId !== null && analyzingTrackIds.includes(selectedTrackId);
  const analysisProgress = isAnalyzing
    ? (realAnalysisProgress ?? ANALYSIS_PROGRESS_STAGES[analysisStageIndex])
    : null;
  // Phase 7.4 undo/redo: snapshot-based history of the undoable state pieces.
  // Refs (not state) so commitToHistory mutations don't trigger re-renders by
  // themselves; we bump `historyVersion` separately when undo/redo state
  // changes so canUndo/canRedo derived values re-evaluate.
  type HistorySnapshot = {
    settingsMap: Record<string, MasteringSettings>;
    albumIntent: MasteringSettings;
    overrideAlbum: string[];
  };
  const historyPast = useRef<HistorySnapshot[]>([]);
  const historyFuture = useRef<HistorySnapshot[]>([]);
  // Coalesce window: consecutive commits within this many ms collapse into the
  // first snapshot, so a slider drag becomes ONE undo step rather than N.
  const lastCommitAt = useRef<number>(0);
  const [historyVersion, setHistoryVersion] = useState(0);
  const HISTORY_MAX = 100;
  const HISTORY_COALESCE_MS = 300;

  // Settings → backend bridge. Two transformations every payload needs
  // before it reaches the audio chain:
  //
  //   1. Inject `source_lufs_integrated` from the analysis result for
  //      this track. Currently unused by the VM math (which estimates
  //      from chain gain stages instead), but kept future-friendly for
  //      a real measure-and-target loop.
  //
  //   2. Override `volume_match` with the session-level transport state.
  //      VM is a UI A/B utility, not a per-track creative choice — the
  //      checkbox the user is looking at always wins. Without this
  //      override, VM "got lost" when the user clicked around: per-track
  //      `settings.volume_match` stayed on the value it had when last
  //      toggled FROM that specific track, so switching tracks made VM
  //      disagree with the checkbox until the user re-toggled. Once
  //      lost, it stayed lost. This forces consistency on every payload
  //      that goes to the backend.
  //
  // The volume_match read uses a ref, not transport.volumeMatch directly,
  // so setVolumeMatch can update the ref synchronously and have the
  // updateSettings call in the same tick already see the new value
  // (otherwise React's setState batching would have us reading the OLD
  // transport.volumeMatch and overriding the toggle BACK to its prior
  // state — exactly the bug this override was meant to prevent).
  //
  // Renaming preserved as `withSourceLufs` to keep diffs small; the
  // comment block above is the source of truth for what it actually does.
  // The SET of in-flight analysis batch ids (§6). Tracking a set rather than a
  // single "current" id means a batch finishing never drops a *different* batch's
  // analysis:progress events — the old single-ref design nulled the ref when the
  // most-recent batch finished first, silently dropping an earlier batch's progress.
  // batchId -> the track ids that batch covers.
  const inFlightAnalysisBatchesRef = useRef<Map<string, TrackId[]>>(new Map());
  const selectedTrackIdRef = useRef<TrackId | null>(null);
  selectedTrackIdRef.current = selectedTrackId;

  // F6 per-track view memory. A ref keeps the reader callback stable while
  // always seeing the latest map. `rememberTrackView` is the ONLY writer, and
  // the force-bounce path never calls it, so a bounce can't clobber a choice.
  const viewByTrackIdRef = useRef<Record<TrackId, ViewMode>>({});
  viewByTrackIdRef.current = viewByTrackId;
  const rememberTrackView = useCallback((trackId: TrackId | null, view: ViewMode) => {
    setViewByTrackId((prev) => rememberView(prev, trackId, view));
  }, []);
  const rememberedTrackView = useCallback(
    (trackId: TrackId | null): ViewMode | null => rememberedView(viewByTrackIdRef.current, trackId),
    [],
  );
  const volumeMatchRef = useRef(false);
  const exportLufsPreviewRef = useRef(false);
  // A/B swaps are async and can overlap under rapid clicking. Only the newest
  // switch owns user-facing failure state; the backend already cancels older
  // play epochs, so surfacing a late timeout/cancellation from one of them
  // would report an error even after the requested source is playing.
  const playbackKindRequestRef = useRef(0);
  // Internal "force WYSIWYG" flag, distinct from the user-facing Advanced
  // `Preview LUFS` toggle (`transport.exportLufsPreview`). Standard auditions
  // with the loudness landing + limiter applied without ever mutating that
  // visible toggle; `effectivePreviewLanding()` ORs the two for the live chain.
  const forceWysiwygRef = useRef(false);
  useEffect(() => {
    volumeMatchRef.current = transport.volumeMatch;
  }, [transport.volumeMatch]);
  useEffect(() => {
    exportLufsPreviewRef.current = transport.exportLufsPreview;
  }, [transport.exportLufsPreview]);
  // Single source of truth for "should live playback apply the export landing?".
  // ORs the user toggle with the forced flag, but yields to Volume Match (an
  // input-referenced A/B aid that is mutually exclusive with the landing — the
  // surrounding setters already keep one off when the other is on).
  const effectivePreviewLanding = useCallback(
    () => exportLufsPreviewRef.current || (forceWysiwygRef.current && !volumeMatchRef.current),
    [],
  );
  const publishAnalyzingTrackIds = useCallback(() => {
    const ids = new Set<TrackId>();
    for (const list of inFlightAnalysisBatchesRef.current.values()) {
      for (const id of list) ids.add(id);
    }
    setAnalyzingTrackIds(Array.from(ids));
  }, []);

  const beginAnalysis = useCallback(
    (trackIds: TrackId[]) => {
      const batchId = nextAnalysisBatchId();
      inFlightAnalysisBatchesRef.current.set(batchId, trackIds);
      publishAnalyzingTrackIds();
      return batchId;
    },
    [publishAnalyzingTrackIds],
  );

  // A begun batch is ALWAYS finished — callers put this in `finally` with no
  // guard. (The session-restore path used to skip it when its effect had
  // been cancelled mid-flight; the batch then stayed in this map forever and
  // the UI said "analyzing" over a finished result — owner report 2026-08-19.)
  const finishAnalysis = useCallback(
    (batchId: string) => {
      inFlightAnalysisBatchesRef.current.delete(batchId);
      publishAnalyzingTrackIds();
    },
    [publishAnalyzingTrackIds],
  );

  // React-state glue around `applyChainDispatchOverrides` (Vitest-
  // tested). Pulls volumeMatchRef + analysisMap from the hook's
  // closure; the override rules themselves live in the pure helper.
  const withSourceLufs = useCallback(
    (id: TrackId | null, settings: MasteringSettings): MasteringSettings =>
      applyChainDispatchOverrides(settings, id, analysisMap, volumeMatchRef.current),
    [analysisMap],
  );

  // Live-edit updateChain dispatcher: rAF-gated, single-in-flight, latest-
  // wins. Slider drags and EQ tweaks can fire updateSettings dozens of
  // times per frame; without this gate every one of those mutations
  // would issue a Tauri IPC call and push another LiveCoeffUpdate at the
  // audio command thread. The backend handles latest-only correctly
  // (Fix A on the output thread, Fix C for LUFS workers), but reducing
  // raw IPC + serialization volume keeps the JS bridge and the React
  // commit phase responsive under heavy interaction.
  //
  // Behavior:
  //   - Caller passes the LATEST desired (settings, previewLufsLanding)
  //     for the live chain; we never replay older values.
  //   - At most one Tauri call is outstanding at a time; subsequent
  //     calls overwrite the pending slot and wait their turn.
  //   - The next pending dispatch waits for the next animation frame
  //     so multiple state mutations within one frame coalesce into a
  //     single IPC after the frame paints.
  //   - On Tauri error we clear the in-flight gate AND attempt to
  //     drain the pending slot, so a transient failure can't strand
  //     the user's latest setting.
  //   - `attempts` is incremented at the call-site (counted as
  //     "user-initiated edits"); `applied` increments here on each
  //     completed IPC. Gap = edits that were coalesced into a later
  //     send — a meaningful liveness signal, not a bug.
  const updateChainInFlight = useRef(false);
  const updateChainPending = useRef<{
    settings: MasteringSettings;
    preview: boolean;
    album: boolean;
  } | null>(null);
  const updateChainRafScheduled = useRef(false);
  const lastPlaybackTickRef = useRef<{
    trackId: TrackId | null;
    positionSec: number;
    isPlaying: boolean;
    receivedAtMs: number;
  }>({
    trackId: null,
    positionSec: 0,
    isPlaying: false,
    receivedAtMs: 0,
  });

  const sendUpdateChain = useCallback(
    (settings: MasteringSettings, preview: boolean) => {
      // Capture album-ness at edit time so a Track<->Album switch mid-audition is
      // reflected on the very next live edit (the backend otherwise reuses the
      // flag cached at the last playMaster). Coalescing keeps the latest pending,
      // so the surviving send carries the most-recent edit's mode.
      updateChainPending.current = {
        settings,
        preview,
        album: mode === "album",
      };
      const drain = () => {
        const next = updateChainPending.current;
        if (!next) {
          updateChainInFlight.current = false;
          return;
        }
        updateChainPending.current = null;
        updateChainInFlight.current = true;
        api
          .updateChain(next.settings, next.preview, next.album)
          .then(() => {
            drain();
          })
          .catch((err) => {
            updateChainInFlight.current = false;
            setError(messageOf(err));
            // Drain anyway — a fresher setting may have arrived during
            // the failed call and should still try to land.
            if (updateChainPending.current) drain();
          });
      };
      if (updateChainInFlight.current || updateChainRafScheduled.current) return;
      updateChainRafScheduled.current = true;
      // Use rAF where available (browser/Tauri); fall back to a 16 ms
      // timer in node-test environments that lack requestAnimationFrame.
      const scheduleDrain = () => {
        updateChainRafScheduled.current = false;
        drain();
      };
      if (typeof requestAnimationFrame === "function") {
        requestAnimationFrame(scheduleDrain);
      } else {
        setTimeout(scheduleDrain, 16);
      }
    },
    [mode],
  );

  useEffect(() => {
    let unlistenTick: (() => void) | undefined;
    let unlistenDeviceLost: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;
    let unlistenLanding: (() => void) | undefined;
    let unlistenAnalysis: (() => void) | undefined;
    let renderProgressClearTimer: ReturnType<typeof setTimeout> | undefined;
    onLandingStatus((pending, event) => {
      const selectedId = selectedTrackIdRef.current;
      if (event.track_id && selectedId && event.track_id !== selectedId) {
        return;
      }
      setLandingPending(pending);
    }).then((fn) => {
      unlistenLanding = fn;
    });
    onAnalysisProgress((evt) => {
      if (!inFlightAnalysisBatchesRef.current.has(evt.batch_id)) return;
      setRealAnalysisProgress({ label: evt.label, progress: evt.fraction });
    }).then((fn) => {
      unlistenAnalysis = fn;
    });
    onPlaybackTick((tick) => {
      setLoadedTrackId(tick.is_loaded ? tick.track_id : null);
      const selectedId = selectedTrackIdRef.current;
      if (tick.is_loaded && selectedId && tick.track_id !== selectedId) {
        return;
      }
      const deviceLost = tick.device_lost ?? false;
      if (deviceLost) {
        setPlaybackDeviceLost({
          track_id: tick.track_id,
          position_sec: tick.position_sec,
        });
      } else if (tick.is_playing) {
        setPlaybackDeviceLost(null);
      }
      lastPlaybackTickRef.current = {
        trackId: tick.track_id,
        positionSec: tick.position_sec,
        isPlaying: tick.is_playing && !deviceLost,
        receivedAtMs: Date.now(),
      };
      setTransport((t) => ({
        ...t,
        currentTimeSec: tick.position_sec,
        isPlaying: tick.is_playing && !deviceLost,
        deviceLost,
        peakDbfs: tick.peak_dbfs,
        peakLeftDbfs: tick.peak_left_dbfs ?? tick.peak_dbfs,
        peakRightDbfs: tick.peak_right_dbfs ?? tick.peak_dbfs,
        compressionGr: {
          low: tick.gr_low_db,
          mid: tick.gr_mid_db,
          high: tick.gr_high_db,
        },
        lufsMomentary: tick.lufs_momentary,
        lufsIntegrated: tick.lufs_integrated,
        spectrumDb: tick.spectrum_db ?? t.spectrumDb,
      }));
    }).then((fn) => {
      unlistenTick = fn;
    });
    onPlaybackDeviceLost((event) => {
      const selectedId = selectedTrackIdRef.current;
      if (event.track_id && selectedId && event.track_id !== selectedId) {
        return;
      }
      setPlaybackDeviceLost(event);
      setLandingPending(false);
      lastPlaybackTickRef.current = {
        trackId: event.track_id,
        positionSec: event.position_sec,
        isPlaying: false,
        receivedAtMs: Date.now(),
      };
      setTransport((t) => ({
        ...t,
        currentTimeSec: event.position_sec,
        isPlaying: false,
        deviceLost: true,
        peakDbfs: -120,
        peakLeftDbfs: -120,
        peakRightDbfs: -120,
        compressionGr: { low: -120, mid: -120, high: -120 },
        lufsMomentary: -120,
        lufsIntegrated: -120,
        spectrumDb: [],
      }));
    }).then((fn) => {
      unlistenDeviceLost = fn;
    });
    onRenderProgress((evt) => {
      // Any fresh progress supersedes a still-pending "clear the bar" timer
      // from a prior completion. Without cancelling on EVERY event, render A
      // reaching 1.0 schedules a 600ms clear that can fire mid render B (which
      // started <600ms later) and wipe B's bar. The effect cleanup cancels a
      // still-pending clear on unmount.
      if (renderProgressClearTimer) {
        clearTimeout(renderProgressClearTimer);
        renderProgressClearTimer = undefined;
      }
      setCancelRequestedJobId((prev) => (prev === evt.job_id ? prev : null));
      setRenderProgress({ job_id: evt.job_id, fraction: evt.fraction, kind: evt.kind });
      // Clear the bar shortly after reaching 1.0 so it doesn't linger.
      if (evt.fraction >= 1.0) {
        renderProgressClearTimer = setTimeout(() => {
          setRenderProgress(null);
          setCancelRequestedJobId(null);
        }, 600);
      }
    }).then((fn) => {
      unlistenProgress = fn;
    });
    return () => {
      unlistenTick?.();
      unlistenDeviceLost?.();
      unlistenProgress?.();
      unlistenLanding?.();
      unlistenAnalysis?.();
      if (renderProgressClearTimer) clearTimeout(renderProgressClearTimer);
    };
  }, []);

  useEffect(() => {
    if (!isAnyAnalyzing) {
      setAnalysisStageIndex(0);
      // A finished (or failed) analysis must not leak its last real event
      // into the next run's first frames.
      setRealAnalysisProgress(null);
      return;
    }
    setAnalysisStageIndex(0);
    const timer = window.setInterval(() => {
      setAnalysisStageIndex((current) =>
        Math.min(current + 1, ANALYSIS_PROGRESS_STAGES.length - 1),
      );
    }, 1400);
    return () => window.clearInterval(timer);
  }, [isAnyAnalyzing]);

  // Phase 7.3: load user presets on mount; subsequent saves/deletes refresh
  // the list directly so we don't need to re-fetch.
  useEffect(() => {
    let cancelled = false;
    api
      .listUserPresets()
      .then((presets) => {
        if (!cancelled) setUserPresets(presets);
      })
      .catch((err) => {
        console.warn("Failed to load user presets", err);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // (Phase A4 hotfix-2 removed an analysis-arrival re-push effect here.
  // Live chain updates inject source LUFS through `withSourceLufs`; if analysis
  // arrives later, the normal settings/update path rebuilds the chain.)

  // Phase 7.2: load the autosaved session on mount, then enable autosave.
  useEffect(() => {
    let cancelled = false;
    api
      .loadRecentSession()
      .then(async (session) => {
        if (cancelled || !session || session.schema_version !== 1) {
          setHadPriorSession(false);
          setSessionLoaded(true);
          return;
        }
        const restoredTracks = session.tracks ?? [];
        setHadPriorSession(restoredTracks.length > 0);
        if (restoredTracks.length > 0) {
          setTracks(restoredTracks);
          // F5 (owner smoke): land on the track the user was on, not always
          // the first. Fall back to the first track when the persisted id is
          // absent (older session) or stale (track since removed).
          const restoredSelection =
            restoredTracks.find((t) => t.id === session.selected_track_id) ??
            restoredTracks[0];
          setSelectedTrackId(restoredSelection.id);
          // Prewarm the auto-selected track's decode cache so the
          // user clicking Mastered immediately after a session
          // restore doesn't pay the 1-2 s cold-decode freeze.
          // Fire-and-forget; the cold path still works if it fails.
          api.prewarmDecode(restoredSelection.path).catch(() => {
            /* opportunistic; cold decode path still works */
          });
        }
        if (session.track_settings) setSettingsMap(session.track_settings);
        if (session.mode) setMode(session.mode);
        if (session.album_intent) setAlbumIntent(session.album_intent);
        setAlbumArcKind(session.album_arc_kind ?? "cinematic");
        setAlbumIntensityState(Math.max(0, Math.min(2, session.album_intensity ?? 1.0)));
        setAlbumTitle(session.album_title ?? "");
        setAlbumSampleRateState(session.album_sample_rate ?? null);
        setAlbumBitDepthState(session.album_bit_depth ?? null);
        if (session.track_override_album) {
          setOverrideAlbum(new Set(session.track_override_album));
        }
        setViewByTrackId(session.view_by_track_id ?? {});

        // Best-effort re-analyze + re-waveform for restored tracks.
        if (restoredTracks.length > 0) {
          const batchId = beginAnalysis(restoredTracks.map((t) => t.id));
          try {
            const results = await api.analyzeTracks(
              restoredTracks.map((t) => ({ id: t.id, path: t.path })),
              batchId,
            );
            if (!cancelled) {
              const map: Record<TrackId, AnalysisResult> = {};
              for (const r of results) map[r.track_id] = r;
              setAnalysisMap((prev) => ({ ...prev, ...map }));
            }
          } catch (err) {
            console.warn("Session restore: analyze failed", err);
          } finally {
            // Unconditional: the batch was begun on THIS hook instance's
            // ref, so it must be removed from it even if the effect was
            // cancelled mid-flight (Fast Refresh re-run) — otherwise
            // isAnalyzing sticks for the life of the session.
            finishAnalysis(batchId);
          }
          for (const t of restoredTracks) {
            if (cancelled) break;
            try {
              const wf = await api.prepareWaveform(t.id, t.path, 1200);
              setWaveformMap((prev) => ({ ...prev, [t.id]: wf }));
            } catch (err) {
              console.warn(`Session restore: waveform for ${t.display_name} failed`, err);
            }
          }
        }
        setSessionLoaded(true);
      })
      .catch((err) => {
        console.warn("Session load failed", err);
        setHadPriorSession(false);
        setSessionLoaded(true);
      });
    return () => {
      cancelled = true;
    };
  }, [beginAnalysis, finishAnalysis]);

  // One snapshot builder shared by both autosave paths — the debounced save
  // below and the explicit analysis-complete save (Q29) — so they can never
  // serialize divergent state.
  const snapshotProjectState = useCallback(
    () =>
      buildProjectState({
        mode,
        tracks,
        settingsMap,
        albumIntent,
        albumArcKind,
        albumIntensity,
        albumTitle,
        albumSampleRate,
        albumBitDepth,
        overrideAlbum,
        selectedTrackId,
        viewByTrackId,
      }),
    [
      mode,
      tracks,
      settingsMap,
      albumIntent,
      albumArcKind,
      albumIntensity,
      albumTitle,
      albumSampleRate,
      albumBitDepth,
      overrideAlbum,
      selectedTrackId,
      viewByTrackId,
    ],
  );

  // Phase 7.2: debounced autosave on relevant state changes.
  useEffect(() => {
    if (!sessionLoaded) return;
    const handle = setTimeout(() => {
      api.autosaveSession(snapshotProjectState()).catch((err) => {
        console.warn("Autosave failed", err);
      });
    }, 1500);
    return () => clearTimeout(handle);
  }, [sessionLoaded, snapshotProjectState]);

  // Q29: fire an explicit autosave the instant analysis completes, so the
  // window the owner measured (analysis latency + the 1.5 s debounce) where
  // fresh work is not yet on disk collapses to zero. Fires only on the
  // analyzing true->false edge with tracks present, and is idempotent with
  // the debounced save.
  const wasAnalyzingRef = useRef(false);
  useEffect(() => {
    // App-wide edge (any batch), not the selected track's — a save must
    // follow every completed analysis, whichever track it was for.
    const justFinished = wasAnalyzingRef.current && !isAnyAnalyzing;
    wasAnalyzingRef.current = isAnyAnalyzing;
    if (!sessionLoaded || !justFinished || tracks.length === 0) return;
    api.autosaveSession(snapshotProjectState()).catch((err) => {
      console.warn("Analysis-complete autosave failed", err);
    });
  }, [isAnyAnalyzing, sessionLoaded, tracks.length, snapshotProjectState]);

  const selectedTrack = useMemo(
    () => tracks.find((t) => t.id === selectedTrackId),
    [tracks, selectedTrackId],
  );
  const selectedAnalysis = selectedTrackId ? analysisMap[selectedTrackId] : undefined;
  const selectedWaveform = selectedTrackId ? waveformMap[selectedTrackId] : undefined;
  const selectedIsOverriding = selectedTrackId
    ? overrideAlbum.has(selectedTrackId)
    : false;
  const followingAlbumIntent = mode === "album" && !selectedIsOverriding && !!selectedTrackId;
  const selectedSettings: MasteringSettings = followingAlbumIntent
    ? albumIntent
    : (selectedTrackId ? settingsMap[selectedTrackId] : undefined) ?? DEFAULT_SETTINGS;
  const previewStale = selectedTrackId ? staleSet.has(selectedTrackId) : false;
  const selectedRegion: LoopRegion | null = selectedTrackId
    ? regionByTrack[selectedTrackId] ?? null
    : null;

  // Read-only per-axis adaptive-trim summary for the "what was trimmed" UI.
  // B2: computed in Rust (single source of truth), which resolves the profile
  // from its store by track id — the SAME profile the chain applies — so the FE
  // sends raw settings + the track id, not a pre-injected profile. Album mode is
  // non-adaptive. Recomputes on settings/analysis/mode change with a latest-wins
  // guard; depends on selectedAnalysis so a late-arriving analysis refetches.
  // Optional-chained so it is inert wherever the command isn't available (tests).
  const [guardrailReadout, setGuardrailReadout] = useState<GuardrailReadout | null>(
    null,
  );
  const [compressionPlan, setCompressionPlan] = useState<CompressionPlan | null>(null);
  const [adaptiveCompressionGate, setAdaptiveCompressionGate] = useState(false);
  const guardrailReadoutReq = useRef(0);
  const compressionPlanSurface = useRef<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    Promise.resolve(api.adaptiveCompressionEnabled?.())
      .then((enabled) => {
        if (!cancelled) {
          setAdaptiveCompressionGate(typeof enabled === "boolean" ? enabled : false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setAdaptiveCompressionGate(false);
        }
      });

    if (typeof window === "undefined") {
      return () => {
        cancelled = true;
      };
    }

    const onAdaptiveCompressionGate = (event: Event) => {
      const enabled = (event as CustomEvent<boolean>).detail;
      if (typeof enabled === "boolean") {
        setAdaptiveCompressionGate(enabled);
      }
    };
    window.addEventListener(ADAPTIVE_COMPRESSION_GATE_EVENT, onAdaptiveCompressionGate);
    return () => {
      cancelled = true;
      window.removeEventListener(
        ADAPTIVE_COMPRESSION_GATE_EVENT,
        onAdaptiveCompressionGate,
      );
    };
  }, []);

  useEffect(() => {
    if (!selectedTrackId) {
      setGuardrailReadout(null);
      setCompressionPlan(null);
      compressionPlanSurface.current = null;
      return;
    }
    const reqId = ++guardrailReadoutReq.current;
    void selectedAnalysis; // dep: refetch when analysis lands so the store is ready
    const album = mode === "album";
    const surface = `${selectedTrackId}:${album}:${adaptiveCompressionGate}`;
    if (compressionPlanSurface.current !== surface) {
      setCompressionPlan(null);
    }
    Promise.resolve(
      api.guardrailReadout?.(selectedSettings, selectedTrackId, album),
    )
      .then((r) => {
        if (guardrailReadoutReq.current === reqId) {
          setGuardrailReadout(r ?? null);
        }
      })
      .catch(() => {
        if (guardrailReadoutReq.current === reqId) {
          setGuardrailReadout(null);
        }
      });
    Promise.resolve(
      api.resolveCompressionPlan?.(selectedSettings, selectedTrackId, album),
    )
      .then((plan) => {
        if (guardrailReadoutReq.current === reqId) {
          compressionPlanSurface.current = surface;
          setCompressionPlan(plan ?? null);
        }
      })
      .catch(() => {
        if (guardrailReadoutReq.current === reqId) {
          compressionPlanSurface.current = surface;
          setCompressionPlan(null);
        }
      });
  }, [
    selectedTrackId,
    selectedSettings,
    selectedAnalysis,
    mode,
    adaptiveCompressionGate,
  ]);

  const estimatedPlaybackPositionSec = useCallback(() => {
    const tick = lastPlaybackTickRef.current;
    let positionSec = tick.positionSec;
    if (tick.isPlaying && tick.trackId === selectedTrackId) {
      positionSec += Math.max(0, Date.now() - tick.receivedAtMs) / 1000;
    }
    const duration = selectedTrack?.duration_seconds;
    if (Number.isFinite(duration)) {
      positionSec = Math.min(positionSec, duration as number);
    }
    return Math.max(0, positionSec);
  }, [selectedTrack?.duration_seconds, selectedTrackId]);

  const markStale = useCallback((id: TrackId) => {
    setStaleSet((prev) => {
      const next = new Set(prev);
      next.add(id);
      return next;
    });
  }, []);

  const markFresh = useCallback((id: TrackId) => {
    setStaleSet((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
  }, []);

  // Phase 7.4 — snapshot the undoable state pieces and push onto the past
  // stack. Called BEFORE each mutation so the popped state on undo is the
  // pre-mutation state. Coalesces consecutive commits within
  // HISTORY_COALESCE_MS into a single snapshot (the FIRST one in the burst)
  // so a slider drag is one undo step, not N. New commits always clear the
  // redo stack (standard undo/redo semantics).
  const commitToHistory = useCallback(() => {
    const now = Date.now();
    if (shouldCoalesceCommit(lastCommitAt.current, now, HISTORY_COALESCE_MS)) {
      // Inside a drag burst — extend the window but don't add a new snapshot.
      lastCommitAt.current = now;
      return;
    }
    lastCommitAt.current = now;
    const snapshot: HistorySnapshot = {
      settingsMap: { ...settingsMap },
      albumIntent: { ...albumIntent },
      overrideAlbum: Array.from(overrideAlbum),
    };
    historyPast.current = appendToPast(
      historyPast.current,
      snapshot,
      HISTORY_MAX,
    );
    historyFuture.current = [];
    setHistoryVersion((v) => v + 1);
  }, [settingsMap, albumIntent, overrideAlbum]);

  // Restore a snapshot. Helper used by both undo and redo.
  const restoreSnapshot = useCallback(
    (snapshot: HistorySnapshot) => {
      const restoredOverride = new Set(snapshot.overrideAlbum as TrackId[]);
      setSettingsMap(snapshot.settingsMap);
      setAlbumIntent(snapshot.albumIntent);
      setOverrideAlbum(restoredOverride);
      // After restoring state, push the restored settings to the live audio
      // chain if the affected track is currently playing as Mastered. Without
      // this, undo would change the UI state but the audible output would lag
      // until the user toggled Original/Master or made another adjustment.
      const id = selectedTrackId;
      const followingAlbum =
        !!id && mode === "album" && !snapshot.overrideAlbum.includes(id as string);
      if (
        shouldPushLiveChainForSettingsEdit({
          trackId: id,
          editingAlbumIntent: followingAlbum,
          loadedTrackId,
          loadedKindByTrack,
          overrideAlbum: restoredOverride,
        })
      ) {
        const effective = followingAlbum
          ? snapshot.albumIntent
          : snapshot.settingsMap[id as string] ?? DEFAULT_SETTINGS;
        const effectiveForChain = withSourceLufs(id as TrackId, effective);
        sendUpdateChain(effectiveForChain, effectivePreviewLanding());
      }
    },
    [
      selectedTrackId,
      loadedKindByTrack,
      loadedTrackId,
      mode,
      withSourceLufs,
      sendUpdateChain,
      effectivePreviewLanding,
    ],
  );

  // §5 — re-push the live chain when the Album<->Track mode flips while a
  // Mastered preview is loaded, so the album-ness flag / album-intent-vs-track
  // settings take effect immediately instead of waiting for an unrelated edit.
  // This runs as an effect (not a setMode wrapper) because sendUpdateChain reads
  // the album flag from the `mode` closure; a synchronous re-push would carry
  // the stale pre-switch flag.
  const prevModeRef = useRef(mode);
  useEffect(() => {
    const prev = prevModeRef.current;
    if (prev === mode) return;
    prevModeRef.current = mode;
    const id = selectedTrackIdRef.current;
    if (!id) return;
    const followingAlbum = mode === "album" && !overrideAlbum.has(id);
    if (
      shouldPushLiveChainForSettingsEdit({
        trackId: id,
        editingAlbumIntent: followingAlbum,
        loadedTrackId,
        loadedKindByTrack,
        overrideAlbum,
      })
    ) {
      const effective = followingAlbum
        ? albumIntent
        : settingsMap[id] ?? DEFAULT_SETTINGS;
      sendUpdateChain(withSourceLufs(id, effective), effectivePreviewLanding());
    }
  }, [
    mode,
    overrideAlbum,
    loadedTrackId,
    loadedKindByTrack,
    albumIntent,
    settingsMap,
    withSourceLufs,
    sendUpdateChain,
    effectivePreviewLanding,
  ]);

  const undo = useCallback(() => {
    const current: HistorySnapshot = {
      settingsMap: { ...settingsMap },
      albumIntent: { ...albumIntent },
      overrideAlbum: Array.from(overrideAlbum),
    };
    const result = applyUndo(historyPast.current, historyFuture.current, current);
    if (result.restored === null) return; // empty past — no-op
    historyPast.current = result.past;
    historyFuture.current = result.future;
    // Reset the coalesce window so the NEXT user edit always commits a new
    // snapshot rather than collapsing into the just-restored state.
    lastCommitAt.current = 0;
    restoreSnapshot(result.restored);
    setHistoryVersion((v) => v + 1);
  }, [settingsMap, albumIntent, overrideAlbum, restoreSnapshot]);

  const redo = useCallback(() => {
    const current: HistorySnapshot = {
      settingsMap: { ...settingsMap },
      albumIntent: { ...albumIntent },
      overrideAlbum: Array.from(overrideAlbum),
    };
    const result = applyRedo(historyPast.current, historyFuture.current, current);
    if (result.restored === null) return; // empty future — no-op
    historyPast.current = result.past;
    historyFuture.current = result.future;
    lastCommitAt.current = 0;
    restoreSnapshot(result.restored);
    setHistoryVersion((v) => v + 1);
  }, [settingsMap, albumIntent, overrideAlbum, restoreSnapshot]);

  const canUndo = historyPast.current.length > 0;
  const canRedo = historyFuture.current.length > 0;
  // historyVersion intentionally referenced here so the closures above
  // re-evaluate canUndo / canRedo on each render after a history change.
  void historyVersion;

  const updateSettings = useCallback(
    (id: TrackId, mutate: (prev: MasteringSettings) => MasteringSettings) => {
      const editingAlbumIntent = mode === "album" && !overrideAlbum.has(id);
      // Phase 7.4: capture pre-mutation state for undo. Coalesces within
      // HISTORY_COALESCE_MS so slider drags are one undo step.
      commitToHistory();
      // Compute `nextSettings` from the CURRENT-RENDER closure values, not
      // from inside a setState updater. React 18's batched-updates model
      // makes side-effect assignments inside `setState((prev) => ...)`
      // unreliable when the call site needs to read the result synchronously;
      // pulling the current state into a local variable here removes that
      // hazard entirely so the api.updateChain call below always has a
      // defined value.
      let nextSettings: MasteringSettings;
      if (editingAlbumIntent) {
        nextSettings = mutate(albumIntent);
        setAlbumIntent(nextSettings);
      } else {
        const current = settingsMap[id] ?? DEFAULT_SETTINGS;
        nextSettings = mutate(current);
        setSettingsMap((prev) => ({ ...prev, [id]: nextSettings }));
        markStale(id);
      }

      // Push to live chain when the edit reaches the currently-playing master.
      // Source playback is an explicit no-push case; if the frontend kind is
      // unknown but the backend tick says this track is loaded, keep the
      // conservative Mastered fallback used by earlier live-chain fixes.
      const shouldPush = shouldPushLiveChainForSettingsEdit({
        trackId: id,
        editingAlbumIntent,
        loadedTrackId,
        loadedKindByTrack,
        overrideAlbum,
      });
      if (shouldPush) {
        // Volume Match needs the current track's source-LUFS — see the
        // `withSourceLufs` helper at the top of this hook.
        const settingsForChain = withSourceLufs(id, nextSettings);
        sendUpdateChain(settingsForChain, effectivePreviewLanding());
      }
    },
    [
      mode,
      overrideAlbum,
      markStale,
      loadedKindByTrack,
      loadedTrackId,
      albumIntent,
      settingsMap,
      withSourceLufs,
      commitToHistory,
      sendUpdateChain,
      effectivePreviewLanding,
    ],
  );

  const toggleOverrideAlbum = useCallback(
    (id: TrackId) => {
      commitToHistory();
      const wasOverriding = overrideAlbum.has(id);
      setOverrideAlbum((prev) => {
        const next = new Set(prev);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        return next;
      });
      if (!wasOverriding) {
        // Entering override — seed per-track settings from the current album
        // intent so the user has a sensible starting point to deviate from.
        setSettingsMap((prev) => ({ ...prev, [id]: { ...albumIntent } }));
      }
    },
    [overrideAlbum, albumIntent, commitToHistory],
  );

  // Stable-ref to importFiles so the drag-drop listener effect doesn't
  // re-attach on every render of the hook.
  const importFilesRef = useRef<(paths: string[]) => Promise<void>>(async () => {});

  const analyzeKnownTracks = useCallback(
    async (targetTracks: ImportedTrack[]): Promise<AnalysisResult[]> => {
      if (targetTracks.length === 0) return [];
      const batchId = beginAnalysis(targetTracks.map((t) => t.id));
      try {
        const results = await api.analyzeTracks(
          targetTracks.map((t) => ({ id: t.id, path: t.path })),
          batchId,
        );
        setAnalysisMap((prev) => {
          const next = { ...prev };
          for (const r of results) next[r.track_id] = r;
          return next;
        });
        return results;
      } finally {
        finishAnalysis(batchId);
      }
    },
    [beginAnalysis, finishAnalysis],
  );

  const rebuildWaveforms = useCallback(async (targetTracks: ImportedTrack[]) => {
    if (targetTracks.length === 0) return;
    setIsLoadingWaveform(true);
    try {
      for (const track of targetTracks) {
        const wf = await api.prepareWaveform(track.id, track.path, 1200);
        setWaveformMap((prev) => ({ ...prev, [track.id]: wf }));
      }
    } finally {
      setIsLoadingWaveform(false);
    }
  }, []);

  const reanalyzeTracks = useCallback(
    async (targetTracks: ImportedTrack[]) => {
      if (targetTracks.length === 0) return;
      setError(null);
      for (const track of targetTracks) {
        markStale(track.id);
      }
      try {
        const results = await analyzeKnownTracks(targetTracks);
        const missing = missingAnalysisTracks(targetTracks, results);
        await rebuildWaveforms(targetTracks);
        if (missing.length > 0) {
          setError(`${analysisGapSummary(missing)}.`);
        }
      } catch (err) {
        setError(messageOf(err));
      }
    },
    [analyzeKnownTracks, rebuildWaveforms, markStale],
  );

  const reanalyzeTrack = useCallback(
    async (id: TrackId) => {
      const track = tracks.find((t) => t.id === id);
      if (!track) return;
      await reanalyzeTracks([track]);
    },
    [tracks, reanalyzeTracks],
  );

  const reanalyzeAll = useCallback(async () => {
    await reanalyzeTracks(tracks);
  }, [tracks, reanalyzeTracks]);

  const importFiles = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) return;
      setError(null);
      try {
        const imported = await api.importTracks(paths);
        if (imported.length === 0) return;

        setTracks((prev) => [...prev, ...imported]);
        setSettingsMap((prev) => {
          const next = { ...prev };
          for (const t of imported) next[t.id] = DEFAULT_SETTINGS;
          return next;
        });

        const newIds = imported.map((t) => t.id);
        for (const id of newIds) markStale(id);

        // Always jump to the newly imported track (not only when nothing was
        // selected before). Replicate selectTrack's transport/meter reset
        // INLINE — we cannot call selectTrack here because `imported[0]` is not
        // in `tracks` state yet on this render. Without the reset, a prior
        // track's "playing" transport and live meter readings leak onto the
        // fresh import: an active play button and moving meters with nothing
        // playing. Reset the meters back to their silence sentinels too.
        const selected = imported[0];
        setSelectedTrackId(selected.id);
        // Sync the ref synchronously — it is otherwise only refreshed on the
        // next render. A late playback tick for the PREVIOUS track can land in
        // the window before React commits; without this the tick guard would
        // read the stale (old) selected id, match it, and re-paint the old
        // track's "playing" state + meters onto the fresh import — exactly the
        // stale state the reset just below clears.
        selectedTrackIdRef.current = selected.id;
        setTransport((t) => ({
          ...t,
          isPlaying: false,
          currentTimeSec: 0,
          loop: false,
          peakDbfs: -120,
          peakLeftDbfs: -120,
          peakRightDbfs: -120,
          compressionGr: { low: -120, mid: -120, high: -120 },
          lufsMomentary: -120,
          lufsIntegrated: -120,
          spectrumDb: [],
        }));
        api.stopPlayback().catch(() => {
          /* swallow — best-effort */
        });
        api.setLoopRegion(null).catch(() => {
          /* swallow — best-effort */
        });
        // Prewarm the newly auto-selected import. Same rationale as selectTrack:
        // the user is likely to click Mastered shortly after import.
        // Fire-and-forget.
        api.prewarmDecode(selected.path).catch(() => {
          /* opportunistic; cold decode path still works */
        });

        const results = await analyzeKnownTracks(imported);
        const missing = missingAnalysisTracks(imported, results);
        setSettingsMap((prev) => {
          const next = { ...prev };
          for (const r of results) {
            const current = next[r.track_id] ?? DEFAULT_SETTINGS;
            if (current.preset.kind === "universal") {
              next[r.track_id] = r.recommended_universal;
            }
          }
          return next;
        });

        await rebuildWaveforms(imported);
        if (missing.length > 0) {
          setError(`${analysisGapSummary(missing)}.`);
        }
      } catch (err) {
        setError(messageOf(err, { name: projectDisplayName(paths[0] ?? "") }));
      }
    },
    [markStale, analyzeKnownTracks, rebuildWaveforms],
  );

  // Keep the ref in sync with the latest importFiles closure so the long-lived
  // drag-drop listener always calls the freshest version (with the current
  // `selectedTrackId` selection logic etc.).
  importFilesRef.current = importFiles;

  // Tauri's window-level drag/drop listener. Attaches once on mount, lives for
  // the lifetime of the hook. Filters dropped paths by audio extension so we
  // ignore non-audio files quietly instead of failing import.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (cancelled) return;
        const payload = event.payload as {
          type: "enter" | "over" | "drop" | "leave";
          paths?: string[];
        };
        if (payload.type === "enter") {
          setIsDragOver(true);
        } else if (payload.type === "leave") {
          setIsDragOver(false);
        } else if (payload.type === "drop") {
          setIsDragOver(false);
          const all = payload.paths ?? [];
          const audio = all.filter((p) => supportedAudioExtensionFromName(p));
          if (audio.length > 0) {
            importFilesRef.current(audio).catch((err) => {
              console.warn("drag-drop import failed", err);
            });
          } else if (all.length > 0) {
            setError(unsupportedDropMessage(all));
          }
        }
      })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
        }
      })
      .catch((err) => {
        console.warn("Failed to attach drag-drop listener", err);
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const openImportDialog = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [AUDIO_DIALOG_FILTER],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      await importFiles(paths);
    } catch (err) {
      setError(messageOf(err));
    }
  }, [importFiles]);

  const selectTrack = useCallback(
    (id: TrackId) => {
      setSelectedTrackId(id);
      selectedTrackIdRef.current = id;
      setTransport((t) => ({
        ...t,
        isPlaying: false,
        currentTimeSec: 0,
        loop: false,
        peakDbfs: -120,
        peakLeftDbfs: -120,
        peakRightDbfs: -120,
        compressionGr: { low: -120, mid: -120, high: -120 },
        lufsMomentary: -120,
        lufsIntegrated: -120,
        spectrumDb: [],
      }));
      if (loadedTrackId && loadedTrackId !== id) {
        api.stopPlayback().catch(() => {
          /* swallow — best-effort */
        });
      }
      api.setLoopRegion(null).catch(() => {
        /* swallow — best-effort */
      });
      // Prewarm the backend decode cache for the newly-selected track
      // so the PCM is ready by the time the user clicks Mastered. The
      // decode runs on the Tauri blocking pool — fire-and-forget here,
      // idempotent on repeat selections of the same track. Eliminates
      // the 1-2 s freeze on first Mastered click for long WAVs.
      const selected = tracks.find((t) => t.id === id);
      if (selected) {
        api.prewarmDecode(selected.path).catch(() => {
          /* swallow — prewarm is opportunistic; cold-decode path
             still works if this fails */
        });
      }
    },
    [loadedTrackId, tracks],
  );

  const removeTrack = useCallback(
    (id: TrackId) => {
      setTracks((prev) => prev.filter((t) => t.id !== id));
      setAnalysisMap((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      setWaveformMap((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      setSettingsMap((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      setStaleSet((prev) => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
      setLoadedKindByTrack((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      setRegionByTrack((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      setOverrideAlbum((prev) => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
      api.evictSourceProfile(id).catch(() => {
        /* best-effort backend cache cleanup */
      });
      if (loadedTrackId === id) {
        api.stopPlayback().catch(() => {
          /* swallow — best-effort */
        });
        // The loaded chain is gone — clear the predicate and reset the live
        // transport/meter state so the auto-selected sibling does not inherit a
        // stale "playing" indicator, frozen meter values, or a still-armed loop
        // (§3). Mirrors selectTrack's reset, including disarming the backend loop.
        setLoadedTrackId(null);
        setTransport((t) => ({
          ...t,
          isPlaying: false,
          currentTimeSec: 0,
          loop: false,
          peakDbfs: -120,
          peakLeftDbfs: -120,
          peakRightDbfs: -120,
          compressionGr: { low: -120, mid: -120, high: -120 },
          lufsMomentary: -120,
          lufsIntegrated: -120,
          spectrumDb: [],
        }));
        api.setLoopRegion(null).catch(() => {
          /* swallow — best-effort */
        });
      }
      if (selectedTrackId === id) {
        const remaining = tracks.filter((t) => t.id !== id);
        setSelectedTrackId(remaining.length > 0 ? remaining[0].id : null);
      }
    },
    [loadedTrackId, selectedTrackId, tracks],
  );

  const setPreset = useCallback(
    (preset: Preset) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => ({ ...prev, preset }));
    },
    [selectedTrackId, updateSettings],
  );

  const setIntensity = useCallback(
    (intensity: number) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => ({ ...prev, intensity }));
    },
    [selectedTrackId, updateSettings],
  );

  const setEqBand = useCallback(
    (
      band: "sub" | "low" | "low-mid" | "mid" | "high-mid" | "high" | "sparkle",
      db: number,
    ) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => {
        const next = { ...prev };
        if (band === "sub") next.eq_sub_db = db;
        else if (band === "low") next.eq_low_db = db;
        else if (band === "low-mid") next.eq_low_mid_db = db;
        else if (band === "mid") next.eq_mid_db = db;
        else if (band === "high-mid") next.eq_high_mid_db = db;
        else if (band === "high") next.eq_high_db = db;
        else next.eq_sparkle_db = db;
        return next;
      });
    },
    [selectedTrackId, updateSettings],
  );

  // 2026-08-18 — a Visual EQ node carries gain AND frequency, and a drag
  // moves both. They must land as ONE settings mutation: `updateSettings`
  // computes `next` from the current-render snapshot (see its comment), so
  // two calls inside one pointer event would each start from the same
  // `prev` and the second would silently drop the first. One call = one
  // undo step = one live-chain push. Hz is clamped to the band's range here
  // as a courtesy; the engine clamps again on the way in.
  const setEqBandPoint = useCallback(
    (
      band: "sub" | "low" | "low-mid" | "mid" | "high-mid" | "high" | "sparkle",
      db: number,
      hz: number,
    ) => {
      if (!selectedTrackId) return;
      const key = EQ_BAND_KEY[band];
      const [lo, hi] = EQ_BAND_RANGES[key];
      const clampedHz = Math.round(Math.max(lo, Math.min(hi, hz)));
      updateSettings(selectedTrackId, (prev) => {
        const next = { ...prev };
        if (band === "sub") next.eq_sub_db = db;
        else if (band === "low") next.eq_low_db = db;
        else if (band === "low-mid") next.eq_low_mid_db = db;
        else if (band === "mid") next.eq_mid_db = db;
        else if (band === "high-mid") next.eq_high_mid_db = db;
        else if (band === "high") next.eq_high_db = db;
        else next.eq_sparkle_db = db;
        next.eq_bands = { ...EQ_BAND_DEFAULTS, ...(prev.eq_bands ?? {}), [key]: clampedHz };
        return next;
      });
    },
    [selectedTrackId, updateSettings],
  );

  // Fast reset for the Visual EQ + Intensity area: flatten intensity and all
  // seven EQ bands in a SINGLE updateSettings mutation so it lands as one
  // undo step (vs. eight coalesced setEqBand/setIntensity calls) and pushes
  // the live chain once. Routes through updateSettings, so album-intent vs.
  // per-track and the stale/live-push bookkeeping are handled identically to
  // every other tone edit.
  const resetToneControls = useCallback(() => {
    if (!selectedTrackId) return;
    updateSettings(selectedTrackId, (prev) => resetToneSettings(prev));
  }, [selectedTrackId, updateSettings]);

  // Advanced -> Standard return: reset every non-managed field to its default
  // (broader than `resetToneSettings`), preserving the Standard-managed
  // {preset, intensity, loudness target, delivery format}. Routes through
  // updateSettings so the live-chain / album-intent bookkeeping is identical
  // to every other edit.
  const resetToStandardManaged = useCallback(() => {
    if (!selectedTrackId) return;
    updateSettings(selectedTrackId, (prev) => resetToStandardManagedSettings(prev));
  }, [selectedTrackId, updateSettings]);

  // Owner 2026-08-19: the Advanced→Standard return's reset. Same as Reset
  // all, plus an Advanced-only style (Spatial / Warmth / Punch / Loud /
  // custom) lands on Universal at the same intensity, because Standard has
  // no tile for it. One undo step like every other settings edit.
  const resetForStandardReturn = useCallback(() => {
    if (!selectedTrackId) return;
    updateSettings(selectedTrackId, (prev) => resetForStandardReturnSettings(prev));
  }, [selectedTrackId, updateSettings]);

  // UI-truthfulness contract (B7): when the user edits a field that a
  // non-Custom DeliveryProfile would shadow at render time, the
  // displayed value MUST become the value export uses. The pure logic
  // lives in `src/lib/settings-transitions.ts::applyAdvancedWithProfileFlip`
  // (Vitest-tested) — this callback is the React-state glue.
  const setAdvanced = useCallback(
    (advanced: AdvancedSettings) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) =>
        applyAdvancedWithProfileFlip(prev, advanced),
      );
    },
    [selectedTrackId, updateSettings],
  );

  const setInputGain = useCallback(
    (db: number) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => ({ ...prev, input_gain_db: db }));
    },
    [selectedTrackId, updateSettings],
  );

  const setOutputGain = useCallback(
    (db: number) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => ({ ...prev, output_gain_db: db }));
    },
    [selectedTrackId, updateSettings],
  );

  const [albumRendering, setAlbumRendering] = useState<boolean>(false);
  const [albumExportReport, setAlbumExportReport] =
    useState<import("../lib/api").AlbumRenderReport | null>(null);

  // U10 — live album plan preview for the sidebar sequence overview.
  //
  // The sequence overview must show what will ACTUALLY render (roles, arc
  // offsets), not a client-side re-derivation that could drift from the
  // backend planner. So it asks the same `plan_album` the export uses.
  //
  // Read-only and disposable: it never feeds a render, and a failure just
  // leaves the overview without roles/offsets rather than surfacing an error —
  // a preview that cannot compute must not look like a broken export.
  const [albumPlanPreview, setAlbumPlanPreview] =
    useState<import("../bindings").AlbumPlan | null>(null);

  useEffect(() => {
    if (mode !== "album" || tracks.length === 0) {
      setAlbumPlanPreview(null);
      return;
    }
    const analyses = tracks
      .map((t) => analysisMap[t.id])
      .filter((a): a is AnalysisResult => !!a);
    if (analyses.length !== tracks.length) {
      setAlbumPlanPreview(null);
      return;
    }

    let cancelled = false;
    // Debounced: reorder and the flow-amount slider both fire rapidly, and the
    // planner is a backend round trip.
    const timer = setTimeout(() => {
      const durations = tracks.map((t) => t.duration_seconds ?? 0);
      const arc: import("../bindings").AlbumArc = {
        kind: "preset",
        preset: albumArcKind,
      };
      void api
        .planAlbum(
          albumTitle.trim() || tracks[0]?.display_name || "Album",
          analyses,
          durations,
          arc,
          albumIntensity,
          albumSampleRate,
          albumBitDepth,
        )
        .then((plan) => {
          if (!cancelled) setAlbumPlanPreview(plan);
        })
        .catch(() => {
          if (!cancelled) setAlbumPlanPreview(null);
        });
    }, 180);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [
    mode,
    tracks,
    analysisMap,
    albumArcKind,
    albumIntensity,
    albumTitle,
    albumSampleRate,
    albumBitDepth,
  ]);

  /// Phase B: build + render the album via the new AlbumPlan path. Picks
  /// up the current tracks, per-track analyses, per-track settings,
  /// current arc + intensity, and hands it to the backend. Returns the
  /// AlbumRenderReport via `albumExportReport` state. This is the only
  /// frontend album export path; the older simple-album hook was removed
  /// after the UI converged on AlbumPlan export.
  const exportAlbumPlan = useCallback(async () => {
    if (tracks.length === 0) return;
    setAlbumRendering(true);
    setError(null);
    setRenderFeedback(null);
    setCancelRequestedJobId(null);
    try {
      const analyses = tracks
        .map((t) => analysisMap[t.id])
        .filter((a): a is AnalysisResult => !!a);
      if (analyses.length !== tracks.length) {
        throw new Error(
          "Analyze all tracks before exporting the album (some are missing analysis).",
        );
      }
      const durations = tracks.map((t) => t.duration_seconds ?? 0);
      const outputDir = await chooseAlbumExportFolder();
      if (!outputDir) return;
      const arc: import("../bindings").AlbumArc = {
        kind: "preset",
        preset: albumArcKind,
      };
      const title = albumTitle.trim() || tracks[0]?.display_name || "Album";
      const plan = await api.planAlbum(
        title,
        analyses,
        durations,
        arc,
        albumIntensity,
        albumSampleRate,
        albumBitDepth,
      );
      const renderTracks: import("../lib/api").AlbumTrackRenderInput[] =
        plan.tracks.map((entry) => {
          const isOverride = overrideAlbum.has(entry.track_id);
          const settings = isOverride
            ? settingsMap[entry.track_id] ?? albumIntent
            : albumIntent;
          const sourceTrack = tracks.find((t) => t.id === entry.track_id);
          return {
            track_id: entry.track_id,
            source_path: sourceTrack?.path ?? "",
            settings,
            // D9 full sound exemption: the backend skips arc offset +
            // character bias for overridden tracks so the on-screen promise
            // ("its own settings will be applied at export") is kept.
            override_album: isOverride,
          };
        });
      const report = await api.renderAlbumPlan(plan, renderTracks, outputDir);
      setAlbumExportReport(report);
      if (isCancelledStatus(report.status)) {
        setRenderFeedback({
          kind: "album",
          message: cancelledRenderMessage("album"),
        });
      }
    } catch (err) {
      setError(messageOf(err));
    } finally {
      clearIncompleteRenderProgress("album");
      setCancelRequestedJobId(null);
      setAlbumRendering(false);
    }
  }, [
    tracks,
    analysisMap,
    settingsMap,
    albumIntent,
    overrideAlbum,
    albumArcKind,
    albumIntensity,
    albumTitle,
    albumSampleRate,
    albumBitDepth,
    clearIncompleteRenderProgress,
  ]);

  /// Phase A3 — pick a delivery profile. Replaces lufs_offset_db /
  /// ceiling_dbtp / bit_depth at render time when non-`custom`. Picking
  /// `custom` doesn't touch the user's existing advanced fields.
  const setDeliveryProfile = useCallback(
    (profile: MasteringSettings["delivery_profile"]) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) =>
        applyDeliveryProfileSelection(prev, profile),
      );
    },
    [selectedTrackId, updateSettings],
  );

  const setLoudnessTarget = useCallback(
    (targetLufs: number | null) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) =>
        applyExplicitLoudnessTarget(prev, targetLufs),
      );
    },
    [selectedTrackId, updateSettings],
  );

  const setLoudnessTargetProfile = useCallback(
    (profileId: string) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) =>
        applyLoudnessTargetSelection(prev, profileId),
      );
    },
    [selectedTrackId, updateSettings],
  );

  const setDeliveryBitDepth = useCallback(
    (bitDepth: number | null) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => ({
        ...prev,
        delivery_profile: "custom",
        advanced: { ...prev.advanced, bit_depth: bitDepth },
      }));
    },
    [selectedTrackId, updateSettings],
  );

  const setDeliverySampleRate = useCallback(
    (sampleRate: number | null) => {
      if (!selectedTrackId) return;
      updateSettings(selectedTrackId, (prev) => ({
        ...prev,
        delivery_profile: "custom",
        advanced: { ...prev.advanced, target_sample_rate: sampleRate },
      }));
    },
    [selectedTrackId, updateSettings],
  );

  const updatePreview = useCallback(async () => {
    if (!selectedTrackId || !selectedTrack) return;
    if (!selectedAnalysis) {
      setError(MASTERED_REQUIRES_ANALYSIS_MESSAGE);
      return;
    }
    setIsRendering(true);
    setError(null);
    setRenderFeedback(null);
    setCancelRequestedJobId(null);
    try {
      // Phase 5: Mastered playback runs through the live chain, so a "preview
      // render" no longer needs to swap the audio source. The button still
      // produces an offline WAV (useful when auditing the would-be master in
      // another player) and clears the stale flag for export bookkeeping.
      // WYSIWYG: the offline preview WAV runs the SAME adapted chain as export
      // and live audition. B2: the backend derives + injects the source profile
      // (keyed by track id) inside render_track_preview, so the FE just sends the
      // raw settings — no FE-side profile injection.
      const job = await api.renderTrackPreview(
        selectedTrackId,
        selectedTrack.path,
        selectedSettings,
      );
      if (isCancelledStatus(job.status)) {
        setRenderFeedback({
          kind: "preview",
          message: cancelledRenderMessage("preview"),
        });
        return;
      }
      markFresh(selectedTrackId);
    } catch (err) {
      setError(messageOf(err));
    } finally {
      clearIncompleteRenderProgress("preview");
      setCancelRequestedJobId(null);
      setIsRendering(false);
    }
  }, [
    selectedTrackId,
    selectedTrack,
    selectedAnalysis,
    selectedSettings,
    markFresh,
    clearIncompleteRenderProgress,
  ]);

  const runExport = useCallback(
    async (exportSettings: MasteringSettings) => {
      if (!selectedTrackId || !selectedAnalysis) return;
      setError(null);
      try {
        if (!selectedTrack) return;
        const store = browserExportLocationStore();
        // Never suggest a name that already exists in the remembered export
        // dir — "exports never overwrite prior renders by default" is the
        // app's guard, not the OS replace prompt's. Backend picks the first
        // free <name>.wav / <name>-2.wav / …; any failure (or no remembered
        // dir yet) falls back to the base suggestion unchanged.
        const baseFilename = suggestedMasterFilename(selectedTrack);
        const exportDir = lastExportDirectory(store, "track");
        const uniqueFilename = exportDir
          ? await Promise.resolve(
              api.suggestExportFilename?.(exportDir, baseFilename),
            )
              .then((name) => name || baseFilename)
              .catch(() => baseFilename)
          : baseFilename;
        const chosenPath = await save({
          defaultPath: defaultExportPath(store, "track", uniqueFilename),
          filters: [{ name: "WAV audio", extensions: ["wav"] }],
        });
        if (!chosenPath) return;
        const chosenOutputPath = ensureWavExtension(chosenPath);
        rememberExportDirectory(store, "track", chosenOutputPath);
        setIsExporting(true);
        setRenderFeedback(null);
        setCancelRequestedJobId(null);
        // B2: the backend derives + injects the source profile inside
        // render_track_master (keyed by track id); the FE sends raw settings.
        const job = await api.renderTrackMaster(
          selectedTrackId,
          selectedTrack.path,
          exportSettings,
          chosenOutputPath,
        );
        if (isCancelledStatus(job.status)) {
          setRenderFeedback({
            kind: "master",
            message: cancelledRenderMessage("master"),
          });
          return;
        }
        const outputPath = job.output_paths[0] ?? "";
        const report = buildExportReport({
          trackId: selectedTrackId,
          outputPath,
          job,
          sourceAnalysis: selectedAnalysis,
          sourceFormat: selectedTrack?.source_format ?? "unknown",
          exportSettings,
        });
        const checks = await api.runExportChecks(report, selectedAnalysis, exportSettings);
        setLastExportReceipt({
          trackId: selectedTrackId,
          outputPath,
          checks,
          job,
          kind: "track",
        });
      } catch (err) {
        setError(messageOf(err));
      } finally {
        clearIncompleteRenderProgress("master");
        setCancelRequestedJobId(null);
        setIsExporting(false);
      }
    },
    [selectedTrackId, selectedAnalysis, selectedTrack, clearIncompleteRenderProgress],
  );

  const exportMaster = useCallback(
    () => runExport(selectedSettings),
    [runExport, selectedSettings],
  );

  const exportStandardMaster = useCallback(
    () => runExport(standardExportSettings(selectedSettings)),
    [runExport, selectedSettings],
  );

  const cancelActiveRender = useCallback(async () => {
    const jobId = renderProgress?.job_id;
    if (!jobId || cancelRequestedJobId === jobId) return;
    setCancelRequestedJobId(jobId);
    try {
      await api.cancelRender(jobId);
    } catch (err) {
      setCancelRequestedJobId(null);
      setError(messageOf(err));
    }
  }, [renderProgress?.job_id, cancelRequestedJobId]);

  const playWithKind = useCallback(
    async (kind: PlaybackKindUI, positionSec: number) => {
      if (!selectedTrack || !selectedTrackId) return;
      if (kind === "source") {
        await api.playTrack(selectedTrackId, selectedTrack.path, positionSec);
      } else {
        if (!selectedAnalysis) {
          throw new Error(MASTERED_REQUIRES_ANALYSIS_MESSAGE);
        }
        // Phase 5: mastered playback streams the source through the live DSP
        // chain — no offline render required, settings changes are audible
        // immediately via updateChain.
        //
        // Inject source-LUFS so Volume Match resolves to the proper
        // "match the source's measured loudness" path on the first chain
        // build.
        try {
          await api.playMaster(
            selectedTrackId,
            selectedTrack.path,
            withSourceLufs(selectedTrackId, selectedSettings),
            positionSec,
            effectivePreviewLanding(),
            // B2: album mode is non-adaptive. The backend caches this and reuses
            // it for the settings-only update_chain dispatches that follow.
            mode === "album",
          );
        } catch (err) {
          throw new Error(playbackErrorMessage(err, kind));
        }
      }
      setLoadedKindByTrack((prev) => ({ ...prev, [selectedTrackId]: kind }));
      setPlaybackDeviceLost(null);
      setTransport((t) => ({ ...t, deviceLost: false }));
    },
    [
      selectedTrack,
      selectedTrackId,
      selectedSettings,
      selectedAnalysis,
      withSourceLufs,
      mode,
      effectivePreviewLanding,
    ],
  );

  const togglePlay = useCallback(async () => {
    if (!selectedTrack || !selectedTrackId) return;
    setError(null);
    try {
      const loadedCorrectTrack = loadedTrackId === selectedTrackId;
      const loadedCorrectKind = loadedKindByTrack[selectedTrackId] === transport.playbackKind;
      // Detect end-of-track: when a song finishes the sink empties but the
      // backend still reports is_loaded=true, so the previous code path
      // called resumePlayback() on a dead sink and nothing happened.  If
      // the playhead is at (or essentially at) the duration AND we're not
      // currently playing, treat this as "re-load and play from start".
      const duration = selectedTrack.duration_seconds ?? Infinity;
      const isAtEnd = isPausedAtEffectiveEnd(
        transport.currentTimeSec,
        duration,
        transport.isPlaying,
      );
      if (!loadedCorrectTrack || !loadedCorrectKind || isAtEnd) {
        const startPosition =
          isAtEnd || !loadedCorrectTrack ? 0 : transport.currentTimeSec;
        await playWithKind(transport.playbackKind, startPosition);
      } else if (transport.isPlaying) {
        await api.pausePlayback();
      } else {
        await api.resumePlayback();
        setPlaybackDeviceLost(null);
        setTransport((t) => ({ ...t, deviceLost: false }));
      }
    } catch (err) {
      setError(messageOf(err));
    }
  }, [
    selectedTrack,
    selectedTrackId,
    loadedTrackId,
    loadedKindByTrack,
    transport.playbackKind,
    transport.isPlaying,
    transport.currentTimeSec,
    playWithKind,
  ]);

  // Spacebar = toggle play/pause anywhere the app is focused (DAW-style
  // transport). Only skip when the user is actually typing TEXT — the
  // preset-name field, a textarea, or a contenteditable — so a literal space
  // still types (Phase 12.1 Dan feedback flagged that as mandatory). Number/
  // range inputs (knobs), selects, and buttons don't need space for entry, so
  // it drives play/stop there too; previously focus parking on a knob or the
  // Delivery Profile select swallowed it, which made play feel waveform-only.
  // preventDefault stops page scroll and the focused control's own space action.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.code !== "Space" && e.key !== " ") return;
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      const inputType = (target as HTMLInputElement | null)?.type;
      const isTextEntry =
        tag === "TEXTAREA" ||
        (target?.isContentEditable ?? false) ||
        (tag === "INPUT" &&
          inputType !== "range" &&
          inputType !== "number" &&
          inputType !== "checkbox" &&
          inputType !== "radio");
      if (isTextEntry) return;
      e.preventDefault();
      void togglePlay();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [togglePlay]);

  // Phase 7.4 — Ctrl/Cmd+Z = undo, Ctrl/Cmd+Shift+Z (or Ctrl/Cmd+Y) = redo.
  // Skips when focus is in a text-editable field so the system-native
  // undo in those inputs still works.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const isCtrl = e.ctrlKey || e.metaKey;
      if (!isCtrl) return;
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      const isTextField =
        tag === "TEXTAREA" ||
        (tag === "INPUT" &&
          (target as HTMLInputElement | null)?.type !== "range") ||
        (target?.isContentEditable ?? false);
      if (isTextField) return;
      const key = e.key.toLowerCase();
      if (key === "z") {
        e.preventDefault();
        if (e.shiftKey) redo();
        else undo();
      } else if (key === "y") {
        e.preventDefault();
        redo();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [undo, redo]);

  const setPlaybackKind = useCallback(
    async (kind: PlaybackKindUI) => {
      if (!selectedTrackId) return;
      if (kind === "master" && !selectedAnalysis) {
        setError(MASTERED_REQUIRES_ANALYSIS_MESSAGE);
        return;
      }
      const requestId = playbackKindRequestRef.current + 1;
      playbackKindRequestRef.current = requestId;
      setTransport((t) => ({ ...t, playbackKind: kind }));
      // Mid-playback swap: only call backend play methods while audio is
      // actively running. A loaded-but-paused track should let the user choose
      // Original/Mastered without starting transport.
      if (loadedTrackId === selectedTrackId && transport.isPlaying) {
        setError(null);
        try {
          await playWithKind(kind, estimatedPlaybackPositionSec());
        } catch (err) {
          if (playbackKindRequestRef.current === requestId) {
            setError(messageOf(err));
          }
        }
      }
    },
    [
      selectedTrackId,
      selectedAnalysis,
      loadedTrackId,
      transport.isPlaying,
      estimatedPlaybackPositionSec,
      playWithKind,
    ],
  );

  const seek = useCallback(
    async (positionSec: number) => {
      if (!selectedTrack) return;
      const clamped = Math.max(0, positionSec);
      lastPlaybackTickRef.current = {
        trackId: selectedTrack.id,
        positionSec: clamped,
        isPlaying: transport.isPlaying,
        receivedAtMs: Date.now(),
      };
      setTransport((t) => ({ ...t, currentTimeSec: clamped }));
      // If the track ran to the end the sink is empty and seekPlayback is a
      // no-op on the backend. Re-prepare the source at the click position
      // so the next play actually starts from the new playhead.
      const duration = selectedTrack.duration_seconds ?? Infinity;
      const wasAtEnd = isPausedAtEffectiveEnd(
        transport.currentTimeSec,
        duration,
        transport.isPlaying,
      );
      if (loadedTrackId === selectedTrack.id) {
        try {
          if (wasAtEnd) {
            // Re-prep at the new offset (this also unpauses; the user
            // intends "play from here" after a finish).
            await playWithKind(transport.playbackKind, clamped);
          } else {
            await api.seekPlayback(clamped);
          }
        } catch (err) {
          setError(messageOf(err));
        }
      }
    },
    [selectedTrack, loadedTrackId, transport.currentTimeSec, transport.isPlaying, transport.playbackKind, playWithKind],
  );

  const toggleLoop = useCallback(async () => {
    const nextLoop = !transport.loop;
    setTransport((t) => ({ ...t, loop: nextLoop }));
    try {
      if (nextLoop && selectedRegion) {
        await api.setLoopRegion(selectedRegion);
      } else {
        await api.setLoopRegion(null);
      }
    } catch (err) {
      setError(messageOf(err));
    }
  }, [transport.loop, selectedRegion]);

  // F3 (owner smoke): looping is Advanced-only, but the armed state used to
  // survive a switch to Standard — which has no loop UI at all, leaving a
  // hidden backend loop silently wrapping playback. Standard entry disarms
  // loop + the backend region; the per-track region memory is kept so
  // re-entering Advanced shows the region again (re-arm is explicit).
  const disarmLoop = useCallback(async () => {
    setTransport((t) => (t.loop ? { ...t, loop: false } : t));
    try {
      await api.setLoopRegion(null);
    } catch (err) {
      setError(messageOf(err));
    }
  }, []);

  const setRegion = useCallback(
    async (region: LoopRegion) => {
      if (!selectedTrackId) return;
      setRegionByTrack((prev) => ({ ...prev, [selectedTrackId]: region }));
      if (transport.loop) {
        try {
          await api.setLoopRegion(region);
        } catch (err) {
          setError(messageOf(err));
        }
      }
    },
    [selectedTrackId, transport.loop],
  );

  const clearRegion = useCallback(async () => {
    if (!selectedTrackId) return;
    setRegionByTrack((prev) => {
      const next = { ...prev };
      delete next[selectedTrackId];
      return next;
    });
    if (transport.loop) {
      try {
        await api.setLoopRegion(null);
      } catch (err) {
        setError(messageOf(err));
      }
    }
  }, [selectedTrackId, transport.loop]);

  const setVolumeMatch = useCallback(
    (on: boolean) => {
      // Update the ref synchronously BEFORE setTransport / updateSettings
      // fire. The ref is what `withSourceLufs` reads when wrapping the
      // settings payload for the audio chain — without this synchronous
      // write, the override below would clobber the new VM value back to
      // the prior transport.volumeMatch (React batches setTransport, so
      // the deferred-effect-driven ref update lands one tick later).
      volumeMatchRef.current = on;
      if (on) {
        exportLufsPreviewRef.current = false;
      }
      setTransport((t) => ({
        ...t,
        volumeMatch: on,
        exportLufsPreview: on ? false : t.exportLufsPreview,
      }));
      // Route through updateSettings so the DSP chain picks up the change
      // (live for Mastered playback via api.updateChain, persisted to
      // settingsMap or albumIntent depending on mode). Source playback is
      // unaffected — it never goes through the chain.
      if (selectedTrackId) {
        updateSettings(selectedTrackId, (prev) => ({
          ...prev,
          volume_match: on,
        }));
      }
    },
    [selectedTrackId, updateSettings],
  );

  const setExportLufsPreview = useCallback(
    (on: boolean) => {
      exportLufsPreviewRef.current = on;
      if (on) {
        volumeMatchRef.current = false;
      }
      setTransport((t) => ({
        ...t,
        volumeMatch: on ? false : t.volumeMatch,
        exportLufsPreview: on,
      }));
      if (
        shouldPushLiveChainForSettingsEdit({
          trackId: selectedTrackId,
          editingAlbumIntent: mode === "album" && !selectedIsOverriding,
          loadedTrackId,
          loadedKindByTrack,
          overrideAlbum,
        })
      ) {
        // Route through the same gated dispatcher as live edits — if a
        // sweep is in flight, this toggle merges into the latest-wins
        // pending slot instead of slipping in out of order. Push the
        // EFFECTIVE landing (not the raw `on`): exportLufsPreviewRef was
        // already set to `on` above, so effectivePreviewLanding() reflects
        // this toggle while still ORing in the forced-WYSIWYG (Standard)
        // flag. Pushing raw `on` would drop Standard's landing when the
        // user toggle goes off.
        sendUpdateChain(withSourceLufs(selectedTrackId, selectedSettings), effectivePreviewLanding());
      }
    },
    [
      mode,
      selectedIsOverriding,
      selectedTrackId,
      loadedTrackId,
      loadedKindByTrack,
      overrideAlbum,
      selectedSettings,
      withSourceLufs,
      sendUpdateChain,
      effectivePreviewLanding,
    ],
  );

  const setForceWysiwyg = useCallback(
    (on: boolean) => {
      forceWysiwygRef.current = on;
      // Re-land the live chain if a master is currently auditioning, so
      // entering/leaving Standard mid-playback recomputes the landing.
      if (
        shouldPushLiveChainForSettingsEdit({
          trackId: selectedTrackId,
          editingAlbumIntent: mode === "album" && !selectedIsOverriding,
          loadedTrackId,
          loadedKindByTrack,
          overrideAlbum,
        })
      ) {
        sendUpdateChain(withSourceLufs(selectedTrackId, selectedSettings), effectivePreviewLanding());
      }
    },
    [
      mode,
      selectedIsOverriding,
      selectedTrackId,
      loadedTrackId,
      loadedKindByTrack,
      overrideAlbum,
      selectedSettings,
      withSourceLufs,
      sendUpdateChain,
      effectivePreviewLanding,
    ],
  );

  const clearError = useCallback(() => setError(null), []);
  const clearProjectFeedback = useCallback(() => setProjectFeedback(null), []);
  const clearPlaybackDeviceLost = useCallback(() => {
    // Clear the backend latch FIRST: MarkDeviceLost paused the sink and set
    // a persistent device_lost bit that every 50 ms tick mirrors, so a
    // local-only clear was revived by the next tick (owner smoke F7 —
    // "could not click dismiss, the dialogue didn't go away"). Fire and
    // forget; a failed IPC just means the banner legitimately returns.
    void api.clearDeviceLost().catch(() => {});
    setPlaybackDeviceLost(null);
    setTransport((t) => ({ ...t, deviceLost: false }));
  }, []);
  const clearExportReceipt = useCallback(() => setLastExportReceipt(null), []);

  const saveUserPreset = useCallback(
    async (name: string): Promise<boolean> => {
      const trimmed = name.trim();
      if (!trimmed) {
        setError("Preset name cannot be empty");
        return false;
      }
      setSavingPreset(true);
      setError(null);
      try {
        const kind: PresetKind = mode === "album" ? "album" : "track";
        const base = followingAlbumIntent ? albumIntent : selectedSettings;
        // A preset is a tonal recipe — strip transport/session state so it's
        // portable across tracks: source_lufs_integrated is frontend-injected
        // per-track (re-injected on the next chain update), and Volume Match is
        // a transport preference, not a preset value. applyUserPreset likewise
        // preserves the current VM toggle rather than taking the preset's.
        const snapshot = {
          ...base,
          source_lufs_integrated: null,
          volume_match: false,
        };
        const created = await api.saveUserPreset(trimmed, kind, snapshot);
        setUserPresets((prev) => [...prev, created]);
        return true;
      } catch (err) {
        setError(messageOf(err));
        return false;
      } finally {
        setSavingPreset(false);
      }
    },
    [mode, followingAlbumIntent, albumIntent, selectedSettings],
  );

  const deleteUserPreset = useCallback(async (id: string) => {
    try {
      await api.deleteUserPreset(id);
      setUserPresets((prev) => prev.filter((p) => p.id !== id));
    } catch (err) {
      setError(messageOf(err));
    }
  }, []);

  const applyUserPreset = useCallback(
    (preset: UserPreset) => {
      commitToHistory();
      const editingAlbumIntent = mode === "album" && !selectedIsOverriding;
      // Volume Match is a transport preference, not part of a preset — keep
      // the user's current monitoring toggle instead of the preset's value.
      const currentVolumeMatch = editingAlbumIntent
        ? albumIntent?.volume_match ?? false
        : selectedSettings.volume_match;
      const applied = { ...preset.settings, volume_match: currentVolumeMatch };
      if (editingAlbumIntent) {
        // Apply to album intent.
        setAlbumIntent(applied);
      } else if (selectedTrackId) {
        setSettingsMap((prev) => ({
          ...prev,
          [selectedTrackId]: applied,
        }));
        markStale(selectedTrackId);
      }
      // Push to live chain if currently playing the affected master. Same
      // shared predicate as settings edits, so Source and unknown loaded-kind
      // cases stay consistent across presets, undo/redo, and sliders.
      const shouldPush = shouldPushLiveChainForSettingsEdit({
        trackId: selectedTrackId,
        editingAlbumIntent,
        loadedTrackId,
        loadedKindByTrack,
        overrideAlbum,
      });
      if (shouldPush) {
        sendUpdateChain(
          withSourceLufs(selectedTrackId, applied),
          effectivePreviewLanding(),
        );
      }
    },
    [
      mode,
      selectedIsOverriding,
      selectedTrackId,
      markStale,
      loadedKindByTrack,
      loadedTrackId,
      overrideAlbum,
      commitToHistory,
      withSourceLufs,
      sendUpdateChain,
      albumIntent,
      selectedSettings,
      effectivePreviewLanding,
    ],
  );

  // Phase 12.2 P3 — explicit Save As / Open Project for .ams.json files.
  // Autosave still runs every 1.5 s into app_data/session.json; these flows
  // let the user park a named snapshot anywhere on disk and reload it.
  const saveProjectAs = useCallback(async () => {
    try {
      const defaultName =
        (selectedTrack?.display_name ?? "untitled-project").replace(
          /[^a-z0-9-_]+/gi,
          "_",
        ) + ".ams.json";
      const path = await save({
        defaultPath: defaultName,
        filters: [
          {
            name: "YES Master project",
            extensions: ["ams.json", "json"],
          },
        ],
      });
      if (!path) {
        setProjectFeedback({ tone: "info", message: "Save project canceled." });
        return;
      }
      const state = buildProjectState({
        mode,
        tracks,
        settingsMap,
        albumIntent,
        albumArcKind,
        albumIntensity,
        albumTitle,
        albumSampleRate,
        albumBitDepth,
        overrideAlbum,
        selectedTrackId,
        viewByTrackId,
      });
      await api.saveProject(path, state);
      setError(null);
      setProjectFeedback({
        tone: "ok",
        message: `Project saved to ${projectDisplayName(path)}.`,
      });
    } catch (err) {
      setProjectFeedback(null);
      setError(messageOf(err));
    }
  }, [
    selectedTrack,
    mode,
    tracks,
    settingsMap,
    albumIntent,
    albumArcKind,
    albumIntensity,
    albumTitle,
    albumSampleRate,
    albumBitDepth,
    overrideAlbum,
    selectedTrackId,
    viewByTrackId,
  ]);

  const openProjectFromDisk = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "YES Master project",
            extensions: ["ams.json", "json"],
          },
        ],
      });
      if (!selected) {
        setProjectFeedback({ tone: "info", message: "Open project canceled." });
        return;
      }
      const path = Array.isArray(selected) ? selected[0] : selected;
      if (!path) {
        setProjectFeedback({ tone: "info", message: "Open project canceled." });
        return;
      }
      const state = await api.loadProject(path);
      if (state.schema_version !== 1) {
        setProjectFeedback(null);
        setError(`Unsupported project schema: v${state.schema_version}`);
        return;
      }
      let analysisRecoveryFailed = false;
      let missingAnalysis: ImportedTrack[] = [];
      let waveformRecoveryFailures = 0;
      setTracks(state.tracks ?? []);
      setSettingsMap(state.track_settings ?? {});
      setMode(state.mode);
      if (state.album_intent) setAlbumIntent(state.album_intent);
      setAlbumArcKind(state.album_arc_kind ?? "cinematic");
      setAlbumIntensityState(Math.max(0, Math.min(2, state.album_intensity ?? 1.0)));
      setAlbumTitle(state.album_title ?? "");
      setAlbumSampleRateState(state.album_sample_rate ?? null);
      setAlbumBitDepthState(state.album_bit_depth ?? null);
      setOverrideAlbum(new Set(state.track_override_album ?? []));
      setViewByTrackId(state.view_by_track_id ?? {});
      if (state.tracks && state.tracks.length > 0) {
        // F5 (owner smoke): same selection restore as the session path —
        // land on the saved selection, fall back to the first track.
        const restoredSelection =
          state.tracks.find((t) => t.id === state.selected_track_id) ??
          state.tracks[0];
        setSelectedTrackId(restoredSelection.id);
        // Prewarm the auto-selected track from the opened project
        // so first-Mastered-click is snappy. Same opportunistic
        // pattern as session restore + import auto-select.
        api.prewarmDecode(restoredSelection.path).catch(() => {
          /* opportunistic; cold decode path still works */
        });
      } else {
        setSelectedTrackId(null);
      }
      // Best-effort re-analyze + re-waveform for the restored tracks so the
      // user lands in a working state without manually pressing Analyze.
      if (state.tracks && state.tracks.length > 0) {
        const batchId = beginAnalysis(state.tracks.map((t) => t.id));
        try {
          const results = await api.analyzeTracks(
            state.tracks.map((t) => ({ id: t.id, path: t.path })),
            batchId,
          );
          const nextAnalysis: Record<TrackId, AnalysisResult> = {};
          for (const r of results) nextAnalysis[r.track_id] = r;
          setAnalysisMap(nextAnalysis);
          missingAnalysis = missingAnalysisTracks(state.tracks, results);
        } catch (err) {
          analysisRecoveryFailed = true;
          console.warn("Re-analyze on open failed", err);
        } finally {
          finishAnalysis(batchId);
        }
        for (const t of state.tracks) {
          try {
            const wf = await api.prepareWaveform(t.id, t.path, 1200);
            setWaveformMap((prev) => ({ ...prev, [t.id]: wf }));
          } catch (err) {
            waveformRecoveryFailures += 1;
            console.warn(`Waveform re-decode failed for ${t.display_name}`, err);
          }
        }
      }
      const recoveryNotes: string[] = [];
      if (analysisRecoveryFailed) {
        recoveryNotes.push("analysis could not be refreshed");
      }
      if (missingAnalysis.length > 0) {
        recoveryNotes.push(analysisGapSummary(missingAnalysis));
      }
      if (waveformRecoveryFailures > 0) {
        recoveryNotes.push(
          `${waveformRecoveryFailures} waveform${
            waveformRecoveryFailures === 1 ? "" : "s"
          } could not be rebuilt`,
        );
      }
      setError(null);
      setProjectFeedback({
        tone: recoveryNotes.length > 0 ? "warn" : "ok",
        message:
          recoveryNotes.length > 0
            ? `Project opened from ${projectDisplayName(path)}; ${recoveryNotes.join(
                "; ",
              )}.`
            : `Project opened from ${projectDisplayName(path)}.`,
      });
    } catch (err) {
      setProjectFeedback(null);
      setError(messageOf(err));
    }
  }, [beginAnalysis, finishAnalysis]);

  const reorderTracks = useCallback((fromIndex: number, toIndex: number) => {
    setTracks((prev) => {
      if (
        fromIndex < 0 ||
        fromIndex >= prev.length ||
        toIndex < 0 ||
        toIndex >= prev.length ||
        fromIndex === toIndex
      ) {
        return prev;
      }
      const next = prev.slice();
      const [moved] = next.splice(fromIndex, 1);
      next.splice(toIndex, 0, moved);
      return next;
    });
  }, []);

  return {
    tracks,
    selectedTrackId,
    selectedTrack,
    selectedAnalysis,
    selectedWaveform,
    selectedSettings,
    previewStale,
    isAnalyzing,
    isAnyAnalyzing,
    analysisProgress,
    isLoadingWaveform,
    isRendering,
    isExporting,
    error,
    projectFeedback,
    playbackDeviceLost,
    transport,
    lastExportReceipt,
    renderProgress,
    renderFeedback,
    cancelRequestedJobId,
    landingPending,
    hadPriorSession,
    undo,
    redo,
    canUndo,
    canRedo,

    openImportDialog,
    importFiles,
    reanalyzeTrack,
    reanalyzeAll,
    selectTrack,
    removeTrack,
    // F6 per-track view memory (Standard/Advanced).
    viewByTrackId,
    rememberTrackView,
    rememberedTrackView,
    setPreset,
    setIntensity,
    setEqBand,
    setEqBandPoint,
    resetToneControls,
    resetToStandardManaged,
    resetForStandardReturn,
    setAdvanced,
    setInputGain,
    setOutputGain,
    setDeliveryProfile,
    setLoudnessTarget,
    setLoudnessTargetProfile,
    setDeliveryBitDepth,
    setDeliverySampleRate,
    // Phase B — Album Master controls.
    albumArcKind,
    albumIntensity,
    albumTitle,
    albumRendering,
    albumExportReport,
    // U10: read-only inputs for the sidebar sequence overview.
    albumPlanPreview,
    analysisByTrackId: analysisMap,
    albumSampleRate,
    albumBitDepth,
    setAlbumArc,
    setAlbumIntensity,
    setAlbumTitle,
    setAlbumSampleRate,
    setAlbumBitDepth,
    exportAlbumPlan,
    updatePreview,
    guardrailReadout,
    compressionPlan,
    exportMaster,
    exportStandardMaster,
    cancelActiveRender,
    togglePlay,
    seek,
    setPlaybackKind,
    toggleLoop,
    setVolumeMatch,
    setExportLufsPreview,
    setForceWysiwyg,
    selectedRegion,
    setRegion,
    clearRegion,
    disarmLoop,
    clearError,
    clearProjectFeedback,
    clearPlaybackDeviceLost,
    clearExportReceipt,
    mode,
    setMode,
    reorderTracks,
    albumIntent,
    overrideAlbum,
    selectedIsOverriding,
    followingAlbumIntent,
    toggleOverrideAlbum,
    userPresets,
    savingPreset,
    saveUserPreset,
    deleteUserPreset,
    applyUserPreset,
    isDragOver,
    saveProjectAs,
    openProjectFromDisk,
  };
}
