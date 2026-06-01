import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import {
  attachIphoneTrack,
  initialIphoneAppState,
  markIphoneAnalysisReady,
  selectIphoneExportProfile,
  selectIphoneLoudness,
  selectIphoneTone,
  setIphoneCustomExport,
  setIphonePlayhead,
  switchIphonePlayback,
  toggleIphoneLufsPreview,
  toggleIphoneVolumeMatch,
  toIphoneSimplePlan,
  type IphoneCustomExportSettings,
  type IphoneAppState,
  type IphoneTrack,
} from "./app-state";
import {
  iphoneBackend,
  pickIphoneAudioPath,
  pickIphoneOutputPath,
  type IphoneBackend,
} from "./iphone-api";
import type {
  AnalysisResult,
  ExportReport,
  QualityCheck,
  RenderJob,
  WaveformPeaks,
} from "../../../src/bindings";
import {
  iphoneSimpleExportProfileOptions,
  iphoneSimpleLoudnessOptions,
  iphoneSimpleToneOptions,
  type IphoneSimpleExportProfile,
  type IphoneSimpleLoudness,
  type IphoneSimpleTone,
} from "./simple-mode";
import yesMasterMarkUrl from "../src-tauri/icons/icon.png";
import clarityPresetUrl from "../../../src/assets/presets/clarity.png";
import punchPresetUrl from "../../../src/assets/presets/punch.png";
import universalPresetUrl from "../../../src/assets/presets/universal.png";
import warmthPresetUrl from "../../../src/assets/presets/warmth.png";
import "./styles.css";

type IphoneOperation =
  | "idle"
  | "importing"
  | "analyzing"
  | "exporting"
  | "preparing-preview";
type ProcessingStage = "importing" | "analyzing";

interface LoadedAudition {
  masteredSignature: string | null;
  playback: IphoneAppState["playback"];
  resumeDirty: boolean;
  trackId: string;
}

interface IphoneExportReceipt {
  bitDepth: number;
  measuredLufs: number;
  outputPath: string;
  sampleRate: number;
  warningCount: number;
}

const IPHONE_TONE_VISUALS: Record<
  IphoneSimpleTone,
  { accent: string; description: string; image: string; title: string }
> = {
  balanced: {
    accent: "#4d8bff",
    description: "A clean, streaming-ready shape for most mixes.",
    image: universalPresetUrl,
    title: "Balanced",
  },
  warm: {
    accent: "#fb923c",
    description: "Fuller body and smoother top-end finish.",
    image: warmthPresetUrl,
    title: "Warm",
  },
  open: {
    accent: "#22d3ee",
    description: "Air, vocal clarity, and a wider front edge.",
    image: clarityPresetUrl,
    title: "Open",
  },
  punch: {
    accent: "#ef4444",
    description: "Sharper transient impact and forward energy.",
    image: punchPresetUrl,
    title: "Punch",
  },
};

export default function App({
  backend = iphoneBackend,
  pickAudioPath = pickIphoneAudioPath,
  pickOutputPath = pickIphoneOutputPath,
}: {
  backend?: IphoneBackend;
  pickAudioPath?: () => Promise<string | null>;
  pickOutputPath?: (defaultPath?: string) => Promise<string | null>;
}) {
  const [state, setState] = useState<IphoneAppState>(initialIphoneAppState);
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [exportChecks, setExportChecks] = useState<QualityCheck[]>([]);
  const [exportReceipt, setExportReceipt] = useState<IphoneExportReceipt | null>(
    null,
  );
  const [waveform, setWaveform] = useState<WaveformPeaks | null>(null);
  const [isLoadingWaveform, setIsLoadingWaveform] = useState(false);
  const [waveformError, setWaveformError] = useState<string | null>(null);
  const [isAuditionPlaying, setIsAuditionPlaying] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [operation, setOperation] = useState<IphoneOperation>("idle");
  const operationRef = useRef<IphoneOperation>("idle");
  const waveformRequestVersionRef = useRef(0);
  const loadedAuditionRef = useRef<LoadedAudition | null>(null);
  // Guards the playhead slider against the 50 ms playback tick: while the user
  // is actively dragging, ticks must not overwrite the thumb. scrubValueRef
  // holds the latest dragged value so endScrub can seek to it on release.
  const isScrubbingRef = useRef(false);
  const scrubValueRef = useRef(0);
  const plan = useMemo(() => toIphoneSimplePlan(state), [state]);
  // Signature of just the controls that change the mastering chain, so the
  // live-apply effect below fires once per real control change — not on every
  // playhead tick (which also mutates `state`, and therefore `plan`).
  const masteredControlSignature = useMemo(
    () =>
      JSON.stringify({
        tone: state.selectedTone,
        loudness: state.selectedLoudness,
        profile: state.selectedExportProfile,
        volumeMatch: state.volumeMatch,
        lufsPreview: state.lufsPreview,
        customExport: state.customExport,
      }),
    [
      state.selectedTone,
      state.selectedLoudness,
      state.selectedExportProfile,
      state.volumeMatch,
      state.lufsPreview,
      state.customExport,
    ],
  );
  const hasTrack = state.track !== null;
  const analysisReady = state.analysisStatus === "ready";
  const canRenderMaster = hasTrack && analysisReady;
  const isImporting = operation === "importing";
  const isAnalyzing = operation === "analyzing" || (isImporting && hasTrack);
  const isExporting = operation === "exporting";
  const isPreparingPlayback = operation === "preparing-preview";
  const controlsLocked = isImporting || isAnalyzing || isExporting || isPreparingPlayback;
  const processingStage: ProcessingStage = isAnalyzing ? "analyzing" : "importing";
  const trackStripLabel =
    isAnalyzing
      ? "Analyzing..."
      : state.analysisStatus === "ready"
        ? "Ready"
        : state.analysisStatus === "needed"
          ? "Needs analysis"
          : "Track";
  const trackDuration = state.track?.durationSeconds ?? 0;
  const playheadMax = Math.max(trackDuration, state.playheadSeconds, 0);
  const sampleRate = plan.exportSettings.advanced.target_sample_rate;
  const bitDepth = plan.exportSettings.advanced.bit_depth;
  const targetLufs = plan.exportSettings.advanced.lufs_offset_db;
  const selectedToneVisual = IPHONE_TONE_VISUALS[state.selectedTone];
  const heroImportLabel = isImporting ? "Importing..." : "Import Track";
  const heroActionDisabled = hasTrack
    ? !canRenderMaster || controlsLocked
    : operation !== "idle";
  const heroActionAriaLabel =
    operation === "preparing-preview"
      ? "Preparing playback"
      : isAuditionPlaying
        ? "Pause"
        : "Play";
  const exportButtonLabel = isExporting ? "Creating..." : "Create Master";

  useEffect(() => {
    let isActive = true;
    let cleanup: (() => void) | undefined;

    backend
      .onPlaybackTick((tick) => {
        if (!isActive || tick.track_id !== state.track?.id) return;
        setIsAuditionPlaying(tick.is_playing);
        // Don't let the position tick fight an in-progress scrub; the pointer
        // handlers own the playhead until the drag ends.
        if (!isScrubbingRef.current) {
          setState((current) =>
            setIphonePlayhead(current, Math.max(0, tick.position_sec)),
          );
        }
      })
      .then((unlisten) => {
        if (isActive) {
          cleanup = unlisten;
        } else {
          unlisten();
        }
      })
      .catch(() => {});

    return () => {
      isActive = false;
      cleanup?.();
      void backend.stopPlayback();
    };
  }, [backend, state.track?.id]);

  useEffect(() => {
    let isActive = true;
    let cleanup: (() => void) | undefined;

    backend
      .onAudioSessionWarning((warning) => {
        if (isActive) setMessage(warning);
      })
      .then((unlisten) => {
        if (isActive) {
          cleanup = unlisten;
        } else {
          unlisten();
        }
      })
      .catch(() => {});

    return () => {
      isActive = false;
      cleanup?.();
    };
  }, [backend]);

  // Live-apply mastering changes while Mastered is auditioning. Without this,
  // changing tone/loudness/profile/volume-match/lufs-preview only updates the
  // UI and export plan, not the audio, until Pause->Play — which breaks the
  // real-time-audition contract. iPhone controls are all discrete (cards,
  // segments, toggles), so a direct re-apply per change can't flood the bridge
  // the way desktop's continuous knobs would.
  useEffect(() => {
    if (
      !isAuditionPlaying ||
      state.playback !== "mastered" ||
      !state.track ||
      !analysisReady
    ) {
      return;
    }
    void backend.updateMasteringChain(
      withSourceAnalysis(buildAuditionPreviewSettings(plan), analysis),
      state.lufsPreview,
    );
    // Keyed on the control signature so this is one call per actual change;
    // plan/analysis derive from the same state the signature covers.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [masteredControlSignature, isAuditionPlaying, state.playback]);

  // Returning to the foreground after a call / Siri / route change — iOS
  // deactivates the audio session in those cases, leaving playback silently
  // dead for the rest of the app's lifetime. Re-activate it on every return so
  // the next play (or a resume) can be audible again.
  useEffect(() => {
    function reactivate() {
      if (document.visibilityState === "visible") {
        void backend.reactivateAudioSession();
      }
    }
    document.addEventListener("visibilitychange", reactivate);
    window.addEventListener("focus", reactivate);
    return () => {
      document.removeEventListener("visibilitychange", reactivate);
      window.removeEventListener("focus", reactivate);
    };
  }, [backend]);

  async function importTrack() {
    if (!startOperation("importing")) return;
    setMessage("Importing...");
    try {
      const path = await pickAudioPath();
      if (!path) {
        setMessage(null);
        return;
      }
      setAnalysis(null);
      setExportChecks([]);
      setExportReceipt(null);
      clearWaveform();
      loadedAuditionRef.current = null;
      setIsAuditionPlaying(false);
      const imported = await backend.importTrack(path);
      const track = toIphoneTrack(imported);
      setState((current) => attachIphoneTrack(current, track));
      void loadWaveform(track);
      await analyzeTrack(track);
    } catch (error) {
      setMessage(toIphoneErrorMessage(error));
    } finally {
      finishOperation();
    }
  }

  async function retryAnalysis() {
    if (!state.track || !startOperation("analyzing")) return;
    setAnalysis(null);
    setExportChecks([]);
    setExportReceipt(null);
    try {
      await analyzeTrack(state.track);
    } catch (error) {
      setMessage(toIphoneErrorMessage(error));
    } finally {
      finishOperation();
    }
  }

  async function analyzeTrack(track: IphoneTrack) {
    setMessage("Analyzing...");
    const nextAnalysis = await backend.analyzeTrack(track.id, track.path);
    setAnalysis(nextAnalysis);
    setState((current) => markIphoneAnalysisReady(current));
    setMessage(null);
  }

  async function loadWaveform(track: IphoneTrack) {
    const requestVersion = waveformRequestVersionRef.current + 1;
    waveformRequestVersionRef.current = requestVersion;
    setIsLoadingWaveform(true);
    setWaveformError(null);
    try {
      const nextWaveform = await backend.prepareWaveform(track.id, track.path, 140);
      if (requestVersion !== waveformRequestVersionRef.current) return;
      setWaveform(nextWaveform);
    } catch (error) {
      if (requestVersion !== waveformRequestVersionRef.current) return;
      setWaveform(null);
      // Don't swallow it into a fake flat placeholder — record why so the user
      // sees a distinct "unavailable" state and the device log (lib.rs) is
      // actionable. Gated by requestVersion so a superseded load can't stomp it.
      setWaveformError(toIphoneErrorMessage(error));
    } finally {
      if (requestVersion === waveformRequestVersionRef.current) {
        setIsLoadingWaveform(false);
      }
    }
  }

  function clearWaveform() {
    waveformRequestVersionRef.current += 1;
    setWaveform(null);
    setWaveformError(null);
    setIsLoadingWaveform(false);
  }

  async function exportMaster() {
    if (!state.track || !analysisReady) return;
    if (!startOperation("exporting")) return;
    setMessage("Exporting...");
    setExportChecks([]);
    setExportReceipt(null);
    try {
      const outputPath = await pickOutputPath(
        suggestIphoneExportFileName(state.track),
      );
      if (!outputPath) {
        setMessage(null);
        return;
      }
      const job = await backend.renderMaster({
        trackId: state.track.id,
        trackPath: state.track.path,
        settings: withSourceAnalysis(plan.exportSettings, analysis),
        outputPath,
      });
      const report = buildExportReport(state.track, job);
      const checks = await backend.runExportChecks(
        report,
        analysis,
        withSourceAnalysis(plan.exportSettings, analysis),
      );
      setExportChecks(checks);
      const warningCount = checks.filter((check) => check.level !== "info").length;
      setExportReceipt({
        bitDepth: report.bit_depth,
        measuredLufs: report.measured_lufs,
        outputPath,
        sampleRate: report.sample_rate,
        warningCount,
      });
      setMessage(
        warningCount > 0
          ? `Exported with ${warningCount} warning${warningCount === 1 ? "" : "s"}`
          : "Exported",
      );
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      finishOperation();
    }
  }

  async function toggleAuditionPlayback() {
    if (!state.track || !analysisReady) return;
    if (isAuditionPlaying) {
      await pauseAuditionPlayback();
      return;
    }
    if (canResumeAuditionPlayback(state.playback)) {
      await resumeAuditionPlayback(state.playback);
      return;
    }
    await startAuditionPlayback(state.playback);
  }

  async function switchAuditionMode(nextPlayback: IphoneAppState["playback"]) {
    if (!state.track || !analysisReady) return;
    // While stopped, just update the selection; the next Play uses it. While
    // playing, let startAuditionPlayback flip `playback` only on a successful
    // (re)start — otherwise a blocked switch leaves the segment showing a mode
    // the audio isn't actually playing.
    if (!isAuditionPlaying) {
      setState((current) => switchIphonePlayback(current, nextPlayback));
      setMessage(null);
      return;
    }
    await startAuditionPlayback(nextPlayback);
  }

  async function startAuditionPlayback(playback: IphoneAppState["playback"]) {
    if (!state.track || !analysisReady) return;
    if (!startOperation("preparing-preview")) return;
    setMessage(playback === "mastered" ? "Starting Mastered..." : "Starting Original...");
    try {
      if (playback === "mastered") {
        await backend.playMastered(
          state.track.id,
          state.track.path,
          withSourceAnalysis(buildAuditionPreviewSettings(plan), analysis),
          state.playheadSeconds,
          state.lufsPreview,
        );
      } else {
        await backend.playOriginal(
          state.track.id,
          state.track.path,
          state.playheadSeconds,
        );
      }
      setState((current) => switchIphonePlayback(current, playback));
      loadedAuditionRef.current = {
        masteredSignature:
          playback === "mastered" ? masteredControlSignature : null,
        playback,
        resumeDirty: false,
        trackId: state.track.id,
      };
      setIsAuditionPlaying(true);
      setMessage(null);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
      setIsAuditionPlaying(false);
    } finally {
      finishOperation();
    }
  }

  function canResumeAuditionPlayback(playback: IphoneAppState["playback"]) {
    const loaded = loadedAuditionRef.current;
    if (!loaded || !state.track) return false;
    if (loaded.trackId !== state.track.id) return false;
    if (loaded.playback !== playback) return false;
    if (loaded.resumeDirty) return false;
    if (
      playback === "mastered" &&
      loaded.masteredSignature !== masteredControlSignature
    ) {
      return false;
    }
    return true;
  }

  async function resumeAuditionPlayback(playback: IphoneAppState["playback"]) {
    if (!state.track || !startOperation("preparing-preview")) return;
    setMessage("Resuming...");
    try {
      await backend.resumePlayback();
      setState((current) => switchIphonePlayback(current, playback));
      setIsAuditionPlaying(true);
      setMessage(null);
    } catch (error) {
      loadedAuditionRef.current = null;
      setMessage(error instanceof Error ? error.message : String(error));
      setIsAuditionPlaying(false);
    } finally {
      finishOperation();
    }
  }

  async function pauseAuditionPlayback() {
    try {
      await backend.pausePlayback();
      setIsAuditionPlaying(false);
      setMessage(null);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  function startOperation(nextOperation: Exclude<IphoneOperation, "idle">) {
    if (operationRef.current !== "idle") return false;
    operationRef.current = nextOperation;
    setOperation(nextOperation);
    return true;
  }

  function finishOperation() {
    operationRef.current = "idle";
    setOperation("idle");
  }

  function endScrub() {
    if (!isScrubbingRef.current) return;
    isScrubbingRef.current = false;
    // Commit the final dragged position to the audio thread once, on release.
    if (isAuditionPlaying) {
      void backend.seekPlayback(scrubValueRef.current);
    }
  }

  function updateCustomExport(
    nextCustomExport: Partial<IphoneCustomExportSettings>,
  ) {
    updateAuditionSettings((current) =>
      setIphoneCustomExport(current, {
        ...current.customExport,
        ...nextCustomExport,
      }),
    );
  }

  function updateAuditionSettings(
    update: (current: IphoneAppState) => IphoneAppState,
  ) {
    setExportChecks([]);
    setExportReceipt(null);
    setMessage(null);
    setState((current) => update(current));
  }

  return (
    <main className="iphone-app" aria-label="YES Master iPhone">
      <section className="phone-frame">
        <header className="app-header">
          <div className="brand-lockup">
            <BrandMark />
            <span className="brand-name">YES Master</span>
          </div>
          <span className="status-chip">
            {state.analysisStatus === "ready" ? "Ready" : "Local"}
          </span>
        </header>

        <section
          className="hero-panel"
          style={{ "--tone-accent": selectedToneVisual.accent } as CSSProperties}
        >
          <img
            className="hero-watermark"
            src={yesMasterMarkUrl}
            alt=""
            aria-hidden="true"
          />
          {!hasTrack ? (
            <div className="hero-copy">
              <h1>Import, master, export.</h1>
            </div>
          ) : null}
          <div className={hasTrack ? "hero-orb has-track" : "hero-orb is-empty"}>
            <button
              className="hero-action-button"
              aria-label={hasTrack ? heroActionAriaLabel : undefined}
              data-testid={hasTrack ? "iphone-preview-master" : "iphone-import"}
              type="button"
              disabled={heroActionDisabled}
              onClick={hasTrack ? toggleAuditionPlayback : importTrack}
            >
              {!hasTrack ? (
                <span className="hero-upload-glyph" aria-hidden="true" />
              ) : null}
              <span
                className={
                  isAuditionPlaying ? "hero-pause-glyph" : "hero-play-glyph"
                }
                aria-hidden="true"
              />
              {!hasTrack ? <span>{heroImportLabel}</span> : null}
            </button>
          </div>
          <section className="hero-option-row" aria-label="Audition options">
            <CheckOption
              active={state.volumeMatch}
              disabled={controlsLocked}
              label="Volume Match"
              testId="volume-match"
              onClick={() =>
                updateAuditionSettings((current) =>
                  toggleIphoneVolumeMatch(current),
                )
              }
            />
            <CheckOption
              active={state.lufsPreview}
              disabled={controlsLocked}
              label="LUFS Preview"
              testId="lufs-preview"
              onClick={() =>
                updateAuditionSettings((current) =>
                  toggleIphoneLufsPreview(current),
                )
              }
            />
          </section>
        </section>

        {hasTrack ? (
          <section className="track-meta-row">
            <div>
              <p className="track-label">{trackStripLabel}</p>
              <h2>{state.track?.displayName}</h2>
              {state.track ? (
                <div className="track-chip-row" aria-label="Track details">
                  <span>{formatSourceFormat(state.track.sourceFormat)}</span>
                  {state.track.sampleRate ? (
                    <span>{formatSampleRate(state.track.sampleRate)}</span>
                  ) : null}
                  {state.track.channels ? (
                    <span>{formatChannels(state.track.channels)}</span>
                  ) : null}
                  {state.track.durationSeconds ? (
                    <span>{formatTime(state.track.durationSeconds)}</span>
                  ) : null}
                </div>
              ) : null}
            </div>
            <div className="track-action-row">
              {state.analysisStatus === "needed" ? (
                <button
                  className="analyze-track-button"
                  data-testid="iphone-retry-analysis"
                  type="button"
                  disabled={operation !== "idle"}
                  onClick={retryAnalysis}
                >
                  Analyze
                </button>
              ) : null}
              <button
                className="change-track-button"
                data-testid="iphone-change-track"
                type="button"
                disabled={operation !== "idle"}
                onClick={importTrack}
              >
                Change
              </button>
            </div>
          </section>
        ) : null}

        {hasTrack ? (
          <section
            className="audition-panel"
            aria-label="Audition"
          >
            <div className="transport-row mode-switch">
            <SegmentButton
              active={state.playback === "original"}
              disabled={controlsLocked}
              testId="playback-original"
              onClick={() => void switchAuditionMode("original")}
            >
              Original
            </SegmentButton>
            <SegmentButton
              active={state.playback === "mastered"}
              disabled={!canRenderMaster || controlsLocked}
              testId="playback-mastered"
              onClick={() => void switchAuditionMode("mastered")}
            >
              Mastered
            </SegmentButton>
            </div>

            <MiniWaveform
              error={waveformError}
              isLoading={isLoadingWaveform}
              peaks={waveform}
            />

            <div className="playhead-row" aria-label="Playhead">
            <span>{formatTime(state.playheadSeconds)}</span>
            <input
              data-testid="iphone-playhead"
              type="range"
              min="0"
              max={playheadMax}
              step="0.1"
              value={state.playheadSeconds}
              disabled={!hasTrack}
              onPointerDown={() => {
                isScrubbingRef.current = true;
              }}
              onPointerUp={endScrub}
              onPointerCancel={endScrub}
              onChange={(event) => {
                const playheadSeconds = Number(event.currentTarget.value);
                scrubValueRef.current = playheadSeconds;
                setState((current) =>
                  setIphonePlayhead(current, playheadSeconds),
                );
                // Live-seek only for discrete changes (keyboard / a11y steps).
                // A pointer drag defers its seek to release via endScrub so the
                // audio thread isn't flooded with intermediate positions.
                if (isAuditionPlaying && !isScrubbingRef.current) {
                  void backend.seekPlayback(playheadSeconds);
                } else if (!isAuditionPlaying && loadedAuditionRef.current) {
                  loadedAuditionRef.current = {
                    ...loadedAuditionRef.current,
                    resumeDirty: true,
                  };
                }
              }}
            />
            <span>{formatTime(trackDuration)}</span>
            </div>
            <button
              className="native-play-button"
              data-testid="iphone-native-play"
              type="button"
              disabled={!canRenderMaster || controlsLocked}
              onClick={toggleAuditionPlayback}
            >
              <span
                className={
                  isAuditionPlaying ? "hero-pause-glyph" : "hero-play-glyph"
                }
                aria-hidden="true"
              />
              {isAuditionPlaying ? "Pause" : "Play"}
            </button>
          </section>
        ) : null}

        <section className="settings-sheet" aria-label="Mastering settings">
          <TonePicker
            disabled={controlsLocked}
            selectedTone={state.selectedTone}
            onSelect={(tone) =>
              updateAuditionSettings((current) =>
                selectIphoneTone(current, tone),
              )
            }
          />

          <ControlGroup title="Loudness">
            {iphoneSimpleLoudnessOptions.map((option) => (
              <SegmentButton
                key={option.id}
                active={state.selectedLoudness === option.id}
                disabled={controlsLocked}
                testId={`loudness-${option.id}`}
                onClick={() =>
                  updateAuditionSettings((current) =>
                    selectIphoneLoudness(current, option.id as IphoneSimpleLoudness),
                  )
                }
              >
                {option.label}
              </SegmentButton>
            ))}
          </ControlGroup>

          <ControlGroup title="Profile">
            {iphoneSimpleExportProfileOptions.map((option) => (
              <SegmentButton
                key={option.id}
                active={state.selectedExportProfile === option.id}
                disabled={controlsLocked}
                testId={`profile-${option.id}`}
                onClick={() =>
                  updateAuditionSettings((current) =>
                    selectIphoneExportProfile(
                      current,
                      option.id as IphoneSimpleExportProfile,
                    ),
                  )
                }
              >
                {option.label}
              </SegmentButton>
            ))}
          </ControlGroup>

          {state.selectedExportProfile === "custom" ? (
            <section className="custom-export-panel" aria-label="Custom export">
              <label>
                <span>Rate</span>
                <select
                  data-testid="custom-sample-rate"
                  disabled={controlsLocked}
                  value={state.customExport.sampleRate ?? "source"}
                  onChange={(event) =>
                    updateCustomExport({
                      sampleRate: parseOptionalNumber(event.currentTarget.value),
                    })
                  }
                >
                  <option value="source">Source</option>
                  <option value="44100">44.1 kHz</option>
                  <option value="48000">48 kHz</option>
                  <option value="96000">96 kHz</option>
                </select>
              </label>
              <label>
                <span>Depth</span>
                <select
                  data-testid="custom-bit-depth"
                  disabled={controlsLocked}
                  value={state.customExport.bitDepth ?? "source"}
                  onChange={(event) =>
                    updateCustomExport({
                      bitDepth: parseOptionalNumber(event.currentTarget.value),
                    })
                  }
                >
                  <option value="source">Source</option>
                  <option value="16">16-bit</option>
                  <option value="24">24-bit</option>
                </select>
              </label>
              <label>
                <span>Ceiling</span>
                <select
                  data-testid="custom-ceiling"
                  disabled={controlsLocked}
                  value={state.customExport.ceilingDbtp}
                  onChange={(event) =>
                    updateCustomExport({
                      ceilingDbtp: Number(event.currentTarget.value),
                    })
                  }
                >
                  <option value="-1">-1 dBTP</option>
                  <option value="-1.5">-1.5 dBTP</option>
                  <option value="-2">-2 dBTP</option>
                </select>
              </label>
            </section>
          ) : null}

          <section className="master-card" aria-label="Master settings">
            <div>
              <p className="track-label">Target</p>
              <strong>{targetLufs?.toFixed(1) ?? "-14.0"} LUFS</strong>
            </div>
            <div>
              <p className="track-label">Format</p>
              <strong>
                {formatSampleRate(sampleRate)} · {formatBitDepth(bitDepth)}
              </strong>
            </div>
          </section>
        </section>

        <button
          className="export-button"
          data-testid="iphone-export"
          type="button"
          disabled={!canRenderMaster || operation !== "idle"}
          onClick={exportMaster}
        >
          {exportButtonLabel}
        </button>
        {message ? <p className="status-message">{message}</p> : null}
        {exportChecks.some((check) => check.level !== "info") ? (
          <section className="warning-list" aria-label="Export warnings">
            {exportChecks
              .filter((check) => check.level !== "info")
              .map((check) => (
                <p key={check.code}>{check.message}</p>
            ))}
          </section>
        ) : null}
        {exportReceipt ? (
          <ExportReadySheet
            receipt={exportReceipt}
            onClose={() => setExportReceipt(null)}
          />
        ) : null}
        {isImporting || operation === "analyzing" ? (
          <ProcessingOverlay
            stage={processingStage}
            trackName={state.track?.displayName}
          />
        ) : null}
      </section>
    </main>
  );
}

function TonePicker({
  disabled,
  onSelect,
  selectedTone,
}: {
  disabled: boolean;
  onSelect: (tone: IphoneSimpleTone) => void;
  selectedTone: IphoneSimpleTone;
}) {
  return (
    <section className="tone-picker">
      <h3>Style</h3>
      <div className="tone-grid">
        {iphoneSimpleToneOptions.map((option) => {
          const visual = IPHONE_TONE_VISUALS[option.id];
          const active = selectedTone === option.id;
          return (
            <button
              key={option.id}
              aria-pressed={active}
              className={active ? "tone-card is-active" : "tone-card"}
              data-testid={`tone-${option.id}`}
              disabled={disabled}
              style={{ "--tone-accent": visual.accent } as CSSProperties}
              type="button"
              onClick={() => onSelect(option.id)}
            >
              <img src={visual.image} alt="" aria-hidden="true" />
              <div>
                <span>{option.label}</span>
                <small>{visual.description}</small>
              </div>
            </button>
          );
        })}
      </div>
    </section>
  );
}

function BrandMark() {
  return (
    <span className="brand-mark" aria-hidden="true">
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
        <path
          d="M4 6h2v12H4zM8 10h2v8H8zM12 4h2v16h-2zM16 8h2v10h-2zM20 12h2v6h-2z"
          fill="currentColor"
        />
      </svg>
    </span>
  );
}

function toIphoneTrack(track: {
  id: string;
  path: string;
  display_name: string;
  source_format: string;
  duration_seconds: number | null;
  sample_rate?: number | null;
  channels?: number | null;
}): IphoneTrack {
  return {
    id: track.id,
    displayName: track.display_name,
    path: track.path,
    sourceFormat: track.source_format,
    durationSeconds: track.duration_seconds,
    sampleRate: track.sample_rate ?? null,
    channels: track.channels ?? null,
  };
}

function buildExportReport(track: IphoneTrack, job: RenderJob): ExportReport {
  const outputPath = job.output_paths[0] ?? "";
  const measurements = job.measurements;
  if (!outputPath) {
    throw new Error("Export finished without an output file.");
  }
  if (!measurements) {
    throw new Error("Export finished without rendered measurements.");
  }
  return {
    track_id: track.id,
    output_path: outputPath,
    measured_lufs: measurements.lufs_integrated,
    measured_true_peak_dbtp: measurements.true_peak_dbtp,
    measured_dynamic_range_lu: measurements.dynamic_range_lu,
    source_format: track.sourceFormat,
    destination_format: "wav",
    sample_rate: measurements.sample_rate,
    bit_depth: measurements.bit_depth,
    checks: [],
  };
}

function suggestIphoneExportFileName(track: IphoneTrack) {
  const baseName = track.displayName
    .replace(/\.(wav|wave|aiff|aif|flac|mp3|m4a|aac|ogg|opus)$/i, "")
    .replace(/[\\/:*?"<>|]+/g, "-")
    .replace(/\s+/g, " ")
    .trim();

  return baseName ? `${baseName} - YES Master.wav` : "YES-Master.wav";
}

function buildAuditionPreviewSettings(
  plan: ReturnType<typeof toIphoneSimplePlan>,
) {
  if (plan.previewLufsLanding) return plan.auditionSettings;
  return {
    ...plan.auditionSettings,
    advanced: {
      ...plan.auditionSettings.advanced,
      lufs_offset_db: null,
    },
  };
}

function withSourceAnalysis(
  settings: ReturnType<typeof toIphoneSimplePlan>["exportSettings"],
  analysis: AnalysisResult | null,
) {
  const sourceLufs = analysis?.lufs_integrated;
  if (sourceLufs === undefined || sourceLufs === null || !Number.isFinite(sourceLufs)) {
    return settings;
  }
  return {
    ...settings,
    source_lufs_integrated: sourceLufs,
  };
}

function toIphoneErrorMessage(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  if (/analysis|analy[sz]e|decode|source file|no samples/i.test(message)) {
    return "This track needs analysis before preview or export. Try Analyze again, or choose another audio file.";
  }
  return message;
}

function ControlGroup({
  children,
  title,
}: {
  children: ReactNode;
  title: string;
}) {
  return (
    <section className="control-group">
      <h3>{title}</h3>
      <div className="segmented">{children}</div>
    </section>
  );
}

function SegmentButton({
  active,
  children,
  disabled = false,
  onClick,
  testId,
}: {
  active: boolean;
  children: ReactNode;
  disabled?: boolean;
  onClick: () => void;
  testId: string;
}) {
  return (
    <button
      aria-pressed={active}
      className={active ? "segment is-active" : "segment"}
      data-testid={testId}
      disabled={disabled}
      type="button"
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function CheckOption({
  active,
  disabled = false,
  label,
  onClick,
  testId,
}: {
  active: boolean;
  disabled?: boolean;
  label: string;
  onClick: () => void;
  testId: string;
}) {
  return (
    <button
      aria-checked={active}
      className={active ? "check-option is-active" : "check-option"}
      data-testid={testId}
      disabled={disabled}
      role="checkbox"
      type="button"
      onClick={onClick}
    >
      <span className="check-box" aria-hidden="true" />
      <span>{label}</span>
    </button>
  );
}

function ProcessingOverlay({
  stage,
  trackName,
}: {
  stage: ProcessingStage;
  trackName?: string;
}) {
  const isAnalyzing = stage === "analyzing";

  return (
    <section
      aria-label="Track processing"
      aria-live="polite"
      className="processing-scrim"
      data-testid="iphone-processing-overlay"
    >
      <div className="processing-card">
        <div className="processing-mark" aria-hidden="true">
          <span />
        </div>
        <p className="track-label">{isAnalyzing ? "Analyzing" : "Uploading"}</p>
        <h2>{isAnalyzing ? "Reading the track" : "Uploading track"}</h2>
        <p className="processing-copy">
          {trackName ?? "Handing audio to YES Master."}
        </p>
        <div className="processing-steps" aria-hidden="true">
          <span className="is-done">Import</span>
          <span className={isAnalyzing ? "is-active" : ""}>Analyze</span>
        </div>
        <div className="processing-bar" aria-hidden="true" />
      </div>
    </section>
  );
}

function ExportReadySheet({
  onClose,
  receipt,
}: {
  onClose: () => void;
  receipt: IphoneExportReceipt;
}) {
  return (
    <section
      aria-label="Master ready"
      aria-live="polite"
      className="export-ready-sheet"
      data-testid="iphone-export-ready"
    >
      <div className="export-ready-mark" aria-hidden="true" />
      <p className="track-label">
        {receipt.warningCount > 0 ? "Review warnings" : "Master ready"}
      </p>
      <h2>{fileNameFromPath(receipt.outputPath)}</h2>
      <div className="export-ready-stats" aria-label="Export details">
        <span>{receipt.measuredLufs.toFixed(1)} LUFS</span>
        <span>{formatSampleRate(receipt.sampleRate)}</span>
        <span>{formatBitDepth(receipt.bitDepth)}</span>
      </div>
      <p className="export-ready-location">
        Saved to Files › On My iPhone › YES Master
      </p>
      <button data-testid="iphone-export-ready-done" type="button" onClick={onClose}>
        Done
      </button>
    </section>
  );
}

function MiniWaveform({
  error,
  isLoading,
  peaks,
}: {
  error: string | null;
  isLoading: boolean;
  peaks: WaveformPeaks | null;
}) {
  const channel = peaks?.channels[0] ?? [];
  const bars = channel.length > 0 ? downsampleWaveform(channel, 44) : [];

  // A real decode failure must look different from a healthy empty state —
  // otherwise the flat placeholder reads as "here's your waveform" when it
  // isn't. The concrete reason is in the Rust log (iphone_prepare_waveform).
  if (error && bars.length === 0 && !isLoading) {
    return (
      <div
        aria-label="Waveform unavailable"
        className="mini-waveform is-error"
        data-testid="iphone-mini-waveform"
        role="status"
      >
        <span className="mini-waveform-note">Waveform unavailable for this file.</span>
      </div>
    );
  }

  return (
    <div
      aria-label={isLoading ? "Loading waveform" : "Waveform preview"}
      className={isLoading ? "mini-waveform is-loading" : "mini-waveform"}
      data-testid="iphone-mini-waveform"
    >
      {bars.length > 0
        ? bars.map((level, index) => (
            <span
              // The waveform is visual only here; the playhead slider remains
              // the accessible seek control.
              aria-hidden="true"
              key={`${index}-${level.toFixed(3)}`}
              style={{ "--bar-level": level } as CSSProperties}
            />
          ))
        : Array.from({ length: 24 }, (_, index) => (
            <span
              aria-hidden="true"
              key={index}
              style={{ "--bar-level": 0.22 + (index % 5) * 0.11 } as CSSProperties}
            />
          ))}
    </div>
  );
}

function downsampleWaveform(channel: number[], targetBars: number) {
  if (channel.length <= targetBars) return channel.map(normalizeWaveformLevel);
  const bucketSize = channel.length / targetBars;
  return Array.from({ length: targetBars }, (_, bucketIndex) => {
    const start = Math.floor(bucketIndex * bucketSize);
    const end = Math.max(start + 1, Math.floor((bucketIndex + 1) * bucketSize));
    let max = 0;
    for (let index = start; index < end && index < channel.length; index += 1) {
      max = Math.max(max, Math.abs(channel[index] ?? 0));
    }
    return normalizeWaveformLevel(max);
  });
}

function normalizeWaveformLevel(level: number) {
  if (!Number.isFinite(level)) return 0.08;
  return Math.max(0.08, Math.min(1, level));
}

function formatSampleRate(sampleRate: number | null) {
  if (!sampleRate) return "Source";
  return `${(sampleRate / 1000).toFixed(sampleRate % 1000 === 0 ? 0 : 1)} kHz`;
}

function formatBitDepth(bitDepth: number | null) {
  if (!bitDepth) return "Source bit";
  return `${bitDepth}-bit`;
}

function formatSourceFormat(sourceFormat: string) {
  return sourceFormat.trim().toUpperCase() || "AUDIO";
}

function formatChannels(channels: number) {
  if (channels === 1) return "Mono";
  if (channels === 2) return "Stereo";
  return `${channels} ch`;
}

function fileNameFromPath(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

function formatTime(seconds: number | null | undefined) {
  const safeSeconds = Math.max(0, Math.floor(seconds ?? 0));
  const minutes = Math.floor(safeSeconds / 60);
  const remainingSeconds = safeSeconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

function parseOptionalNumber(value: string) {
  return value === "source" ? null : Number(value);
}
