// Browser-preview mocks for the Tauri backend.
//
// Only loaded by `tauri-runtime.ts` when `window.__TAURI_INTERNALS__` is
// missing — i.e. when the app is being viewed in a plain browser (Vite
// dev server at localhost:1420, agent-browser screenshots, etc).
// In the real Tauri WebView these mocks are never imported.
//
// The mock seeds a "preview project" with one realistic-looking track so
// the loaded-track UI renders immediately on boot. Playback ticks emit at
// ~50 Hz with bouncing meter values so the live readouts animate.

import type {
  AnalysisResult,
  AudioOutputDevice,
  CompressionPlan,
  GuardrailReadout,
  ImportedTrack,
  MasteringSettings,
  PlaybackTick,
  ProjectState,
  QualityCheck,
  TrackId,
  UserPreset,
  WaveformPeaks,
} from "../bindings";
import { EQ_BAND_DEFAULTS } from "../bindings";
import type { UnlistenFn } from "@tauri-apps/api/event";

const PREVIEW_TRACK_ID = "preview-track-1";
const PREVIEW_DURATION = 245;
let mockSelectedAudioOutput: string | null = null;

// Deterministic clock. The preview used to stamp `new Date()` / `Date.now()`
// into analysis results, preset ids, and job ids, which made two runs of the
// same scenario differ. The headless lane (U3) and the capture manifests (U7)
// both need byte-stable output, so the preview uses a fixed instant and a
// monotonic counter instead of wall-clock time.
const PREVIEW_ISO = "2026-01-01T00:00:00.000Z";
let previewIdCounter = 0;
const nextPreviewId = (prefix: string) => `${prefix}-${++previewIdCounter}`;

// ---------------------------------------------------------------------------
// Deterministic preview scenarios (U3).
//
// Selected with `?scenario=<name>` on the /app URL. Each one is a fixed,
// reproducible starting state so a headless test can assert a named expected
// result instead of whatever the mock happened to seed. `?empty=1` is kept as
// an alias for the `empty` scenario so existing links and docs still work.
//
// These are BROWSER-ONLY. `tauri-runtime.ts` never imports this module inside
// the real WebView, so nothing here can change production behavior.
// ---------------------------------------------------------------------------

export type PreviewScenarioName =
  | "clean"
  | "empty"
  | "warning"
  | "long-copy"
  | "export-success"
  | "export-cancel"
  | "album-1"
  | "album-4"
  | "album-12"
  | "album-long"
  | "album-warning";

interface PreviewScenario {
  /// Human-readable purpose, surfaced to the headless lane for failure output.
  readonly purpose: string;
  /// false => `load_recent_session` returns null (true first-run state).
  readonly seedProject: boolean;
  /// 0 => track mode. >0 => album mode with that many tracks.
  readonly albumTracks: number;
  /// Deliberately long display names, to exercise truncation and wrapping.
  readonly longNames: boolean;
  /// What `run_export_checks` returns.
  readonly exportChecks: "clean" | "warning";
  /// Whether the native save dialog resolves to a path or to cancellation.
  readonly saveDialog: "path" | "cancel";
  /// Album positions (1-based) that override the album settings.
  readonly overridePositions: readonly number[];
}

const BASE_SCENARIO: PreviewScenario = {
  purpose: "Seeded single-track project, no warnings.",
  seedProject: true,
  albumTracks: 0,
  longNames: false,
  exportChecks: "clean",
  saveDialog: "cancel",
  overridePositions: [],
};

export const PREVIEW_SCENARIOS: Record<PreviewScenarioName, PreviewScenario> = {
  clean: BASE_SCENARIO,
  empty: {
    ...BASE_SCENARIO,
    purpose: "True first-run state: no session, no track, empty state visible.",
    seedProject: false,
  },
  warning: {
    ...BASE_SCENARIO,
    purpose: "Export checks report a warning and a critical, both reviewable.",
    exportChecks: "warning",
    // The checks only reach the UI through a COMPLETED export receipt, so this
    // scenario has to take the save-succeeds path. With the default cancel the
    // export never runs and the scenario would silently assert nothing.
    saveDialog: "path",
  },
  "long-copy": {
    ...BASE_SCENARIO,
    purpose:
      "Pathological filename length at the supported minimum desktop size.",
    longNames: true,
  },
  "export-success": {
    ...BASE_SCENARIO,
    purpose: "Save dialog returns a path; export completes and shows a receipt.",
    saveDialog: "path",
  },
  "export-cancel": {
    ...BASE_SCENARIO,
    purpose:
      "Save dialog is cancelled; the UI must NOT show a success receipt.",
    saveDialog: "cancel",
  },
  "album-1": {
    ...BASE_SCENARIO,
    purpose: "Album mode with a single track — the degenerate album case.",
    albumTracks: 1,
  },
  "album-4": {
    ...BASE_SCENARIO,
    purpose: "Album mode, four tracks, one overriding the album settings.",
    albumTracks: 4,
    overridePositions: [3],
  },
  "album-12": {
    ...BASE_SCENARIO,
    purpose:
      "Album mode, twelve tracks — scrolling, reorder boundaries, selection.",
    albumTracks: 12,
    overridePositions: [2, 9],
  },
  "album-long": {
    ...BASE_SCENARIO,
    purpose:
      "Album mode with deliberately long track names and a long album title.",
    albumTracks: 12,
    longNames: true,
    overridePositions: [5],
  },
  "album-warning": {
    ...BASE_SCENARIO,
    purpose: "Album mode where export checks surface a warning.",
    albumTracks: 4,
    exportChecks: "warning",
  },
};

export const DEFAULT_SCENARIO: PreviewScenarioName = "clean";

function currentSearchParams(): URLSearchParams {
  return new URLSearchParams(globalThis.location?.search ?? "");
}

export function activeScenarioName(): PreviewScenarioName {
  const params = currentSearchParams();
  const requested = params.get("scenario");
  if (requested && requested in PREVIEW_SCENARIOS) {
    return requested as PreviewScenarioName;
  }
  if (requested) {
    // An unrecognized scenario is a test-authoring bug, not a UI state. Say so
    // loudly — the headless lane treats any [preview-mock] warning as a failure.
    console.warn(`[preview-mock] unhandled scenario: ${requested}`);
  }
  // Back-compat: `?empty=1` predates the scenario switch.
  if (params.has("empty")) return "empty";
  return DEFAULT_SCENARIO;
}

function activeScenario(): PreviewScenario {
  return PREVIEW_SCENARIOS[activeScenarioName()];
}

const LONG_NAME_TAIL =
  "an-absurdly-long-working-title-that-should-truncate-not-overlap";

const MOCK_AUDIO_OUTPUTS: AudioOutputDevice[] = [
  {
    id: "Preview Speakers",
    name: "Preview Speakers",
    is_default: true,
    is_selected: false,
  },
  {
    id: "Preview Headphones",
    name: "Preview Headphones",
    is_default: false,
    is_selected: false,
  },
];

const DEFAULT_ADVANCED: MasteringSettings["advanced"] = {
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
};

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
  advanced: DEFAULT_ADVANCED,
};

const PREVIEW_TRACK: ImportedTrack = {
  id: PREVIEW_TRACK_ID,
  path: "/preview/sample.wav",
  display_name: "Preview Track (browser preview)",
  source_format: "wav",
  duration_seconds: PREVIEW_DURATION,
  sample_rate: 48000,
  channels: 2,
};

const PREVIEW_ANALYSIS: AnalysisResult = {
  track_id: PREVIEW_TRACK_ID,
  lufs_integrated: -14.6,
  lufs_short_term_max: -10.2,
  true_peak_dbtp: -4.0,
  dynamic_range_lu: 5.2,
  spectral_balance: { low: 0.32, mid: 0.42, high: 0.26 },
  transient_density: 0.55,
  stereo_width: 1.0,
  recommended_universal: DEFAULT_SETTINGS,
  measured_at_iso: PREVIEW_ISO,
  inferred_role: null,
  role_confidence: null,
  inferred_character: null,
  character_confidence: null,
};

// Scenario-driven project construction. Deterministic for a given scenario:
// same ids, same names, same order, every run.
function albumTrackName(position: number, longNames: boolean): string {
  const nn = String(position).padStart(2, "0");
  return longNames
    ? `${nn} - ${LONG_NAME_TAIL}-take-${position}.wav`
    : `${nn} - Preview Track ${position}.wav`;
}

function previewProject(): ProjectState {
  const scenario = activeScenario();

  if (scenario.albumTracks > 0) {
    const tracks: ImportedTrack[] = Array.from(
      { length: scenario.albumTracks },
      (_, index) => {
        const position = index + 1;
        return {
          ...PREVIEW_TRACK,
          id: `${PREVIEW_TRACK_ID}-${position}`,
          path: `/preview/album/${position}.wav`,
          display_name: albumTrackName(position, scenario.longNames),
          // Vary duration a little so the sequence is not visually uniform.
          duration_seconds: PREVIEW_DURATION - 40 + position * 7,
        };
      },
    );
    const order = tracks.map((track) => track.id);
    return {
      schema_version: 1,
      mode: "album",
      tracks,
      track_order: order,
      track_settings: Object.fromEntries(
        order.map((id) => [id, DEFAULT_SETTINGS]),
      ),
      album_intent: DEFAULT_SETTINGS,
      album_arc_kind: "cinematic",
      album_intensity: 1.0,
      album_title: scenario.longNames
        ? `A Deliberately Overlong Album Title For Truncation Testing (${LONG_NAME_TAIL})`
        : "Preview Album",
      album_sample_rate: null,
      album_bit_depth: null,
      track_override_album: scenario.overridePositions
        .filter((position) => position <= order.length)
        .map((position) => order[position - 1]),
      selected_track_id: order[0],
      last_saved_iso: null,
    };
  }

  const track: ImportedTrack = scenario.longNames
    ? {
        ...PREVIEW_TRACK,
        display_name: `${LONG_NAME_TAIL}-final-master-v7-FINAL-actually-final.wav`,
      }
    : PREVIEW_TRACK;

  return {
    schema_version: 1,
    mode: "track",
    tracks: [track],
    track_order: [PREVIEW_TRACK_ID],
    track_settings: { [PREVIEW_TRACK_ID]: DEFAULT_SETTINGS },
    album_intent: null,
    track_override_album: [],
    selected_track_id: PREVIEW_TRACK_ID,
    last_saved_iso: null,
  };
}

// Warning-scenario export checks. One warning and one critical so the review
// surface has to render both levels and their detail text.
const WARNING_EXPORT_CHECKS: QualityCheck[] = [
  {
    level: "warning",
    code: "dynamic_range_low",
    message:
      "Dynamic range is low for this delivery target. The master will read as dense on quiet systems.",
  },
  {
    level: "critical",
    code: "true_peak_over_ceiling",
    message:
      "True peak exceeds the delivery ceiling. Lossy encoders are likely to clip this file.",
  },
];

// Synthesize a stereo waveform that visually reads like a music track —
// a couple of dynamic envelope swells over the duration, with stereo
// asymmetry so the two channels don't look identical.
function syntheticWaveform(targetPixels: number): WaveformPeaks {
  const px = Math.max(64, Math.min(4096, targetPixels));
  const peaks: number[][] = [[], []];
  for (let i = 0; i < px; i++) {
    const t = i / px;
    // Envelope: slow swell over the track + a quieter intro + a chorus bump.
    const envelope =
      0.55 +
      0.25 * Math.sin(t * Math.PI * 2 - Math.PI / 2) +
      0.15 * Math.sin(t * Math.PI * 6) +
      0.05 * Math.sin(t * Math.PI * 27);
    const ampL = Math.max(0.05, envelope) * (0.9 + 0.1 * Math.sin(t * 41));
    const ampR = Math.max(0.05, envelope) * (0.9 + 0.1 * Math.cos(t * 37));
    peaks[0].push(Math.min(0.98, ampL));
    peaks[1].push(Math.min(0.98, ampR));
  }
  return {
    track_id: PREVIEW_TRACK_ID,
    channels: peaks,
    samples_per_pixel: Math.floor((PREVIEW_DURATION * 48000) / px),
    total_samples: PREVIEW_DURATION * 48000,
    sample_rate: 48000,
  };
}

// Mock playback ticker state. The browser-preview meters animate when the
// user "plays" so the LIVE pill, gradient bars, and live integrated LUFS
// have something visually interesting to do.
let mockPlaying = false;
let mockPosition = 0;
let mockLoadedTrackId: TrackId | null = PREVIEW_TRACK_ID;
const TICK_HZ = 20;
const TICK_INTERVAL_MS = Math.floor(1000 / TICK_HZ);

// Monotonic id source for preview imports (see import_tracks).
let mockImportCounter = 0;

export async function mockInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  switch (cmd) {
    case "load_recent_session":
      // The `empty` scenario boots with no session — the true first-run state
      // (EmptyState, auto-select on first import, analysis orb in the main
      // slot). Every other scenario seeds its project.
      if (!activeScenario().seedProject) {
        return null as unknown as T;
      }
      return previewProject() as unknown as T;
    case "load_project":
      return previewProject() as unknown as T;
    case "autosave_session":
    case "save_project":
      return null as unknown as T;
    case "suggest_export_filename":
      // No filesystem in the preview — the base suggestion is always "free".
      return (args?.fileName ?? "master.wav") as unknown as T;
    case "build_info":
      return "0.9.1 · build preview · browser" as unknown as T;

    case "import_tracks": {
      const paths = (args?.paths as string[]) ?? [];
      // Unique per CALL, not per batch index — a second preview import must
      // not mint the same React key as the first (duplicate-key errors).
      const imported: ImportedTrack[] = paths.map((p) => ({
        ...PREVIEW_TRACK,
        id: `${PREVIEW_TRACK_ID}-${++mockImportCounter}`,
        path: p,
        display_name: p.split(/[\\/]/).pop() ?? `Track ${paths.indexOf(p) + 1}`,
      }));
      return imported as unknown as T;
    }

    case "analyze_tracks": {
      // Real analysis takes seconds; resolve slowly AND emit the same
      // staged "analysis:progress" events the desktop engine now sends, so
      // the browser preview exercises the real-progress path end to end.
      // (Browser-only code — Tauri never loads this.)
      const stages: Array<[number, string]> = [
        [0.0, "Analyzing audio"],
        [0.35, "Checking dynamics"],
        [0.55, "Evaluating stereo field"],
        [0.65, "Reading tonal balance"],
        [0.8, "Building mastering context"],
      ];
      for (const [fraction, label] of stages) {
        emitAnalysisProgress(fraction, label);
        await new Promise((resolve) => setTimeout(resolve, 800));
      }
      emitAnalysisProgress(1.0, "Building mastering context");
      const tracks =
        (args?.tracks as Array<{ id: TrackId; path: string }>) ?? [];
      const results: AnalysisResult[] = tracks.map((t) => ({
        ...PREVIEW_ANALYSIS,
        track_id: t.id,
      }));
      return results as unknown as T;
    }

    case "prepare_waveform": {
      // Mirror the real flow's decode gap (analysis ends, THEN peaks land)
      // so the browser preview exercises the orb -> morph transition.
      await new Promise((resolve) => setTimeout(resolve, 800));
      const pixels = (args?.targetPixels as number | null) ?? 1600;
      return syntheticWaveform(pixels) as unknown as T;
    }

    case "list_audio_output_devices":
      return MOCK_AUDIO_OUTPUTS.map((device) => ({
        ...device,
        is_selected: mockSelectedAudioOutput === device.id,
      })) as unknown as T;

    case "set_audio_output_device":
      mockSelectedAudioOutput = (args?.deviceId as string | null) ?? null;
      return null as unknown as T;

    case "play_track":
      mockLoadedTrackId = (args?.trackId as TrackId | undefined) ?? PREVIEW_TRACK_ID;
      mockPosition = (args?.startPositionSec as number | null) ?? mockPosition;
      mockPlaying = true;
      return null as unknown as T;

    case "resume_playback":
      mockPlaying = true;
      return null as unknown as T;

    case "play_master":
      mockLoadedTrackId = (args?.trackId as TrackId | undefined) ?? PREVIEW_TRACK_ID;
      mockPosition = (args?.startPositionSec as number | null) ?? mockPosition;
      mockPlaying = true;
      // Mimic the real engine's landing window: Mastered starts hotter than
      // target while the corrective gain is measured, then settles.
      emitLandingStatus(true);
      setTimeout(() => emitLandingStatus(false), 2500);
      return null as unknown as T;

    case "pause_playback":
      mockPlaying = false;
      return null as unknown as T;

    case "stop_playback":
      mockPlaying = false;
      mockLoadedTrackId = null;
      mockPosition = 0;
      return null as unknown as T;

    case "seek_playback":
      mockPosition = (args?.positionSec as number) ?? 0;
      return null as unknown as T;

    case "set_loop_region":
    case "update_chain":
    case "prewarm_decode":
    case "open_output":
    case "delete_user_preset":
      return null as unknown as T;

    case "save_diagnostics_report":
      // Browser preview has no filesystem — echo the chosen path back so
      // the Help panel's saved-confirmation flow can be exercised.
      return ((args?.targetPath as string) ?? "") as unknown as T;

    case "list_user_presets":
      return [] as unknown as T;

    case "save_user_preset": {
      const preset: UserPreset = {
        id: nextPreviewId("mock-preset"),
        name: (args?.name as string) ?? "Preview Preset",
        kind: (args?.kind as UserPreset["kind"]) ?? "track",
        settings: (args?.settings as MasteringSettings) ?? DEFAULT_SETTINGS,
        created_at_iso: PREVIEW_ISO,
      };
      return preset as unknown as T;
    }

    case "plan_album": {
      // Mock: return a single-track plan from whatever the caller passed.
      const req = (args?.request as Record<string, unknown>) ?? {};
      const analyses = (req.analyses as Array<{ track_id: string }> | undefined) ?? [];
      const tracks = analyses.map((a, i) => ({
        track_id: a.track_id,
        position: i + 1,
        role: i === 0
          ? "opener"
          : i === analyses.length - 1
            ? "closer"
            : "album_track",
        role_locked: false,
        arc_lufs_offset_db: 0,
        intensity_scale: 1.0,
      }));
      return {
        title: req.title ?? "Mock Album",
        arc: req.arc ?? { kind: "preset", preset: "cinematic" },
        tracks,
        transitions: Array(Math.max(0, analyses.length - 1)).fill({
          kind: "direct",
          duration_seconds: 0,
        }),
        intensity: req.intensity ?? 1.0,
      } as unknown as T;
    }

    case "render_album_plan":
      emitRenderProgress(
        nextPreviewId("mock-album-render"),
        PREVIEW_TRACK_ID,
        "album",
      );
      return {
        job_id: nextPreviewId("mock-album-render"),
        status: { status: "done" },
        album_wav_path: "/preview/album.wav",
        manifest_path: "/preview/manifest.json",
        requested_sample_rate: null,
        rendered_sample_rate: 44_100,
        source_sample_rates: [44_100],
        bit_depth: 24,
        rendered_channels: 2,
        source_channels: [2],
        tracks: [],
      } as unknown as T;

    case "render_track_preview":
    case "render_track_master": {
      const trackId = (args?.trackId as TrackId | undefined) ?? mockLoadedTrackId ?? PREVIEW_TRACK_ID;
      const outputPath = (args?.outputPath as string | null | undefined) ?? "/preview/output.wav";
      const jobId = nextPreviewId("mock-render");
      const kind = cmd === "render_track_master" ? "master" : "preview";
      emitRenderProgress(jobId, trackId, kind);
      return {
        id: jobId,
        job_id: jobId,
        kind,
        target_tracks: [trackId],
        status: { status: "done" },
        progress: 1.0,
        started_at_iso: PREVIEW_ISO,
        output_paths: [outputPath],
        // Pass 4 (2026-08-19): delivered measurements, so the preview's
        // Export Complete card shows its Results section like the real one.
        measurements: {
          lufs_integrated: -14.1,
          true_peak_dbtp: -1.02,
          dynamic_range_lu: 7.4,
          sample_rate: 44_100,
          bit_depth: 24,
          effective_adaptive_strength: 0.5,
          source_profile_digest: null,
          confidence_digest: null,
          compression_digest: null,
        },
      } as unknown as T;
    }

    case "cancel_render":
      return null as unknown as T;

    case "run_export_checks":
      return (
        activeScenario().exportChecks === "warning" ? WARNING_EXPORT_CHECKS : []
      ) as unknown as T;

    // -----------------------------------------------------------------------
    // Owner-gated calibration surfaces (U3 contract completion).
    //
    // These gates are OFF in production and stay OFF here. The setters mirror
    // the real echo-back semantics so the UI toggle can be exercised, but the
    // state is browser-local: this module is never loaded inside the Tauri
    // WebView, so nothing here can enable a gated system in the product.
    // -----------------------------------------------------------------------
    case "adaptive_compression_enabled":
      return mockAdaptiveCompression as unknown as T;

    case "set_adaptive_compression":
      mockAdaptiveCompression = Boolean(args?.enabled);
      return mockAdaptiveCompression as unknown as T;

    case "confidence_gating_enabled":
      return mockConfidenceGating as unknown as T;

    case "set_confidence_gating":
      mockConfidenceGating = Boolean(args?.enabled);
      return mockConfidenceGating as unknown as T;

    // -----------------------------------------------------------------------
    // Read-only adaptive readouts. Fixed, plausible values -- the preview does
    // not run the DSP, and inventing a *varying* readout would imply it does.
    // -----------------------------------------------------------------------
    case "guardrail_readout":
      return PREVIEW_GUARDRAIL_READOUT as unknown as T;

    case "resolve_compression_plan":
      return PREVIEW_COMPRESSION_PLAN as unknown as T;

    case "evict_source_profile":
      // Backend cache eviction; the preview holds no profile cache.
      return null as unknown as T;

    case "clear_device_lost":
      // The preview never loses a device (playback:device-lost is a no-op),
      // so clearing the latch is a genuine no-op rather than a stub.
      return null as unknown as T;

    default:
      if (NATIVE_ONLY_COMMANDS.has(cmd)) {
        // Recognized and DELIBERATELY unsupported in a browser. Logged with a
        // distinct prefix the headless lane allowlists, so "we know about this"
        // is mechanically distinguishable from "nobody has looked at this".
        console.info(`[preview-mock] native-only command, not simulated: ${cmd}`);
        return null as unknown as T;
      }
      console.warn(`[preview-mock] unhandled command: ${cmd}`, args);
      return null as unknown as T;
  }
}

// Commands whose real behavior only exists on an installed desktop build.
// Faking them would be worse than declining: a browser cannot prove that an
// update installs, and a green preview must never imply that it did.
const NATIVE_ONLY_COMMANDS = new Set<string>(["install_update"]);

// Owner-gated systems: OFF by default, exactly as they ship.
let mockAdaptiveCompression = false;
let mockConfidenceGating = false;

const PREVIEW_GUARDRAIL_READOUT: GuardrailReadout = {
  active: true,
  strength: 0.62,
  bright_trim: 0.18,
  low_trim: 0.0,
  density_trim: 0.24,
  width_trim: 0.0,
  brightness_share: 0.27,
  low_share: 0.31,
  dynamic_range_db: 5.2,
  bright_deadband: 0.24,
  low_deadband: 0.36,
  width_corr_deadband: 0.35,
  stereo_correlation: 0.52,
  confidence: null,
  effective_auto_width: 1.11,
};

const PREVIEW_COMPRESSION_PLAN: CompressionPlan = {
  active: true,
  low: {
    threshold_db: -18.0,
    ratio: 2.0,
    density_mult: 1.0,
    threshold_lift_db: 0.0,
    ratio_mult: 1.0,
    adaptive: false,
  },
  mid: {
    threshold_db: -16.0,
    ratio: 1.8,
    density_mult: 1.0,
    threshold_lift_db: 0.0,
    ratio_mult: 1.0,
    adaptive: false,
  },
  high: {
    threshold_db: -14.0,
    ratio: 1.6,
    density_mult: 1.0,
    threshold_lift_db: 0.0,
    ratio_mult: 1.0,
    adaptive: false,
  },
  reasons: [],
  guidance: null,
  digest: null,
};

export async function mockListen<T>(
  channel: string,
  handler: (event: { payload: T }) => void,
): Promise<UnlistenFn> {
  if (channel === "playback:tick") {
    const interval = setInterval(() => {
      if (mockPlaying) {
        mockPosition = (mockPosition + 1 / TICK_HZ) % PREVIEW_DURATION;
      }
      // Animate live readouts so the meter has something to do. Sinusoidal
      // bounce around a plausible mastered-track level. When paused, all
      // live signals collapse to the silence sentinel (-120) so the UI
      // renders the same "idle" state the real backend would emit.
      const t = mockPosition;
      // Left/right bounce on slightly different phases so the browser preview
      // shows a moving stereo image, not two identical bars.
      const peakLeft = mockPlaying ? -3.5 + 2.5 * Math.sin(t * 7) : -120;
      const peakRight = mockPlaying ? -3.5 + 2.5 * Math.sin(t * 7 + 0.9) : -120;
      const peakDb = mockPlaying ? Math.max(peakLeft, peakRight) : -120;
      const lufsMomentary = mockPlaying ? -11 + 3 * Math.sin(t * 4) : -120;
      const lufsIntegrated = mockPlaying ? -14.2 + 0.4 * Math.sin(t * 0.6) : -120;
      const tick: PlaybackTick = {
        track_id: mockLoadedTrackId,
        position_sec: mockPosition,
        is_playing: mockPlaying,
        is_loaded: mockLoadedTrackId !== null,
        device_lost: false,
        peak_dbfs: peakDb,
        peak_left_dbfs: peakLeft,
        peak_right_dbfs: peakRight,
        gr_low_db: mockPlaying ? -1.2 + 0.8 * Math.sin(t * 3) : -120,
        gr_mid_db: mockPlaying ? -2.3 + 1.1 * Math.sin(t * 5 + 1) : -120,
        gr_high_db: mockPlaying ? -0.6 + 0.4 * Math.sin(t * 9 + 2) : -120,
        lufs_momentary: lufsMomentary,
        lufs_integrated: lufsIntegrated,
      };
      handler({ payload: tick as unknown as T });
    }, TICK_INTERVAL_MS);
    return () => clearInterval(interval);
  }
  if (channel === "landing:status") {
    const wrapped = (pending: boolean) => handler({ payload: pending as unknown as T });
    landingStatusHandlers.add(wrapped as (pending: boolean) => void);
    return () => landingStatusHandlers.delete(wrapped as (pending: boolean) => void);
  }
  if (channel === "playback:device-lost") {
    return () => {};
  }
  if (channel === "updater:available") {
    // Startup update check (Slice 7). The preview has no real updater, so this
    // is a known no-op — registered explicitly to avoid the unhandled-channel warning.
    return () => {};
  }
  if (channel === "render:progress") {
    const wrapped = (payload: unknown) => handler({ payload: payload as T });
    renderProgressHandlers.add(wrapped);
    return () => renderProgressHandlers.delete(wrapped);
  }
  if (channel === "analysis:progress") {
    const wrapped = (fraction: number, label: string) =>
      handler({ payload: { fraction, label } as unknown as T });
    analysisProgressHandlers.add(wrapped as (fraction: number, label: string) => void);
    return () =>
      analysisProgressHandlers.delete(wrapped as (fraction: number, label: string) => void);
  }
  // Unknown channel — return a no-op unlisten.
  console.warn(`[preview-mock] unhandled listen channel: ${channel}`);
  return () => {};
}

// "Landing loudness…" simulation plumbing: play_master flips pending on,
// then off after a short window (see mockInvoke).
const landingStatusHandlers = new Set<(pending: boolean) => void>();
function emitLandingStatus(pending: boolean): void {
  for (const h of landingStatusHandlers) h(pending);
}

// Real analysis-progress simulation: analyze_tracks emits the same staged
// events the desktop engine sends (see mockInvoke).
const analysisProgressHandlers = new Set<(fraction: number, label: string) => void>();
function emitAnalysisProgress(fraction: number, label: string): void {
  for (const h of analysisProgressHandlers) h(fraction, label);
}

// Render progress. The backend emits "render:progress" during preview, master,
// and album renders; before U3 the preview left the channel unregistered, which
// produced an unhandled-listen warning on every /app boot. Renders resolve
// immediately in the preview, so this emits a single honest 1.0 completion
// rather than a fake ramp -- inventing intermediate percentages would be
// exactly the "no fake progress" failure U11 forbids.
const renderProgressHandlers = new Set<(payload: unknown) => void>();
function emitRenderProgress(
  jobId: string,
  trackId: TrackId,
  kind: "preview" | "master" | "album",
): void {
  const payload = { job_id: jobId, track_id: trackId, kind, fraction: 1.0 };
  for (const h of renderProgressHandlers) h(payload);
}

export async function mockOpen(
  opts?: { directory?: boolean; defaultPath?: string; multiple?: boolean; title?: string },
): Promise<string | string[] | null> {
  // Browser-preview can't access the OS filesystem. Returning null mimics
  // "user cancelled the dialog" so error paths render correctly.
  console.info("[preview-mock] open() returned null (cancelled)", opts);
  return null;
}

export async function mockSave(): Promise<string | null> {
  // The export-success / export-cancel scenarios differ ONLY here. Cancel is
  // the conservative default because a browser has no filesystem: a scenario
  // must opt in to the success path rather than get it by accident.
  if (activeScenario().saveDialog === "path") {
    const chosen = "/preview/exports/preview-master.wav";
    console.info(`[preview-mock] save() returned ${chosen} (scenario)`);
    return chosen;
  }
  console.info("[preview-mock] save() returned null (cancelled)");
  return null;
}

export function mockWebview(): {
  onDragDropEvent: (
    handler: (event: { payload: unknown }) => void,
  ) => Promise<UnlistenFn>;
} {
  return {
    onDragDropEvent: async (handler) => {
      // Browser preview can't receive OS file drops, so expose the handler
      // as a global: devtools (or agent tooling driving the preview) can
      // call __previewDropAudio(["C:/demo.wav"]) to exercise the full
      // import -> analyze -> waveform pipeline, including the analysis orb.
      const g = globalThis as { __previewDropAudio?: (paths: string[]) => void };
      const fire = (paths: string[]) => handler({ payload: { type: "drop", paths } });
      g.__previewDropAudio = fire;
      return () => {
        // StrictMode double-mounts: a stale unlisten must not wipe the
        // re-registered hook — only remove our own registration.
        if (g.__previewDropAudio === fire) delete g.__previewDropAudio;
      };
    },
  };
}
