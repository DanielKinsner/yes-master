import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
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
} from "../../../src/bindings";
import {
  iphoneSimpleExportProfileOptions,
  iphoneSimpleLoudnessOptions,
  iphoneSimpleToneOptions,
  type IphoneSimpleExportProfile,
  type IphoneSimpleLoudness,
  type IphoneSimpleTone,
} from "./simple-mode";
import "./styles.css";

type IphoneOperation = "idle" | "importing" | "exporting" | "preparing-preview";

export default function App({
  backend = iphoneBackend,
  pickAudioPath = pickIphoneAudioPath,
  pickOutputPath = pickIphoneOutputPath,
  toAudioUrl = convertFileSrc,
}: {
  backend?: IphoneBackend;
  pickAudioPath?: () => Promise<string | null>;
  pickOutputPath?: () => Promise<string | null>;
  toAudioUrl?: (path: string) => string;
}) {
  const [state, setState] = useState<IphoneAppState>(initialIphoneAppState);
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [exportChecks, setExportChecks] = useState<QualityCheck[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [masterPreviewPath, setMasterPreviewPath] = useState<string | null>(null);
  const [operation, setOperation] = useState<IphoneOperation>("idle");
  const operationRef = useRef<IphoneOperation>("idle");
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const plan = useMemo(() => toIphoneSimplePlan(state), [state]);
  const hasTrack = state.track !== null;
  const isImporting = operation === "importing";
  const isExporting = operation === "exporting";
  const trackDuration = state.track?.durationSeconds ?? 0;
  const playheadMax = Math.max(trackDuration, state.playheadSeconds, 0);
  const auditionPath =
    state.playback === "mastered" && masterPreviewPath
      ? masterPreviewPath
      : state.track?.path;
  const auditionUrl = auditionPath ? toAudioUrl(auditionPath) : null;
  const sampleRate = plan.exportSettings.advanced.target_sample_rate;
  const bitDepth = plan.exportSettings.advanced.bit_depth;
  const targetLufs = plan.exportSettings.advanced.lufs_offset_db;

  useEffect(() => {
    if (!audioRef.current || !auditionUrl) return;
    if (Math.abs(audioRef.current.currentTime - state.playheadSeconds) > 0.25) {
      audioRef.current.currentTime = state.playheadSeconds;
    }
  }, [auditionUrl, state.playheadSeconds]);

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
      setMasterPreviewPath(null);
      const imported = await backend.importTrack(path);
      setState((current) => attachIphoneTrack(current, toIphoneTrack(imported)));
      setMessage("Analyzing...");
      const nextAnalysis = await backend.analyzeTrack(imported.id, imported.path);
      setAnalysis(nextAnalysis);
      setState((current) => markIphoneAnalysisReady(current));
      setMessage(null);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      finishOperation();
    }
  }

  async function exportMaster() {
    if (!state.track) return;
    if (!startOperation("exporting")) return;
    setMessage("Exporting...");
    setExportChecks([]);
    try {
      const outputPath = await pickOutputPath();
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

  async function switchToMasteredPreview() {
    if (!state.track) return;
    if (!startOperation("preparing-preview")) return;
    setMessage("Preparing Mastered...");
    try {
      const job = await backend.prepareMasterPreview({
        trackId: state.track.id,
        trackPath: state.track.path,
        settings: withSourceAnalysis(buildAuditionPreviewSettings(plan), analysis),
      });
      const previewPath = job.output_paths[0] ?? null;
      if (!previewPath) {
        throw new Error("Mastered preview did not produce an audio file.");
      }
      setMasterPreviewPath(previewPath);
      setState((current) => switchIphonePlayback(current, "mastered"));
      setMessage(null);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      finishOperation();
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
    setMasterPreviewPath(null);
    setState((current) => switchIphonePlayback(update(current), "original"));
  }

  return (
    <main className="iphone-app" aria-label="YES Master iPhone Simple">
      <section className="phone-frame">
        <header className="app-header">
          <div>
            <p className="eyebrow">YES Master</p>
            <h1>Simple</h1>
          </div>
          <span className="status-chip">
            {state.analysisStatus === "ready" ? "Ready" : "Local"}
          </span>
        </header>

        <section className="import-strip">
          {hasTrack ? (
            <div className="track-strip-row">
              <div>
                <p className="track-label">Track</p>
                <h2>{state.track?.displayName}</h2>
              </div>
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
          ) : (
            <button
              className="import-button"
              data-testid="iphone-import"
              type="button"
              disabled={operation !== "idle"}
              onClick={importTrack}
            >
              {isImporting ? "Importing..." : "Import Track"}
            </button>
          )}
        </section>

        <section
          className="wave-panel"
          aria-label={masterPreviewPath ? "Audition ready" : "Audition"}
        >
          <div className="waveform" aria-hidden="true">
            {Array.from({ length: 36 }, (_, index) => (
              <span
                key={index}
                style={{ "--bar": `${32 + ((index * 19) % 54)}%` } as CSSProperties}
              />
            ))}
          </div>
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
              onChange={(event) => {
                const playheadSeconds = Number(event.currentTarget.value);
                setState((current) =>
                  setIphonePlayhead(current, playheadSeconds),
                );
              }}
            />
            <span>{formatTime(trackDuration)}</span>
          </div>
          {auditionUrl ? (
            <audio
              className="audio-preview"
              controls
              data-testid="iphone-audio-preview"
              ref={audioRef}
              src={auditionUrl}
              onTimeUpdate={(event) =>
                setState((current) =>
                  setIphonePlayhead(current, event.currentTarget.currentTime),
                )
              }
            />
          ) : null}
          <div className="transport-row">
            <SegmentButton
              active={state.playback === "original"}
              testId="playback-original"
              onClick={() =>
                setState((current) => switchIphonePlayback(current, "original"))
              }
            >
              Original
            </SegmentButton>
            <SegmentButton
              active={state.playback === "mastered"}
              testId="playback-mastered"
              onClick={switchToMasteredPreview}
            >
              Mastered
            </SegmentButton>
          </div>
        </section>

        <ControlGroup title="Tone">
          {iphoneSimpleToneOptions.map((option) => (
            <SegmentButton
              key={option.id}
              active={state.selectedTone === option.id}
              testId={`tone-${option.id}`}
              onClick={() =>
                updateAuditionSettings((current) =>
                  selectIphoneTone(current, option.id as IphoneSimpleTone),
                )
              }
            >
              {option.label}
            </SegmentButton>
          ))}
        </ControlGroup>

        <ControlGroup title="Loudness">
          {iphoneSimpleLoudnessOptions.map((option) => (
            <SegmentButton
              key={option.id}
              active={state.selectedLoudness === option.id}
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

        <section className="toggle-stack">
          <ToggleRow
            active={state.volumeMatch}
            label="Volume Match"
            testId="volume-match"
            onClick={() =>
              updateAuditionSettings((current) => toggleIphoneVolumeMatch(current))
            }
          />
          <ToggleRow
            active={state.lufsPreview}
            label="LUFS Preview"
            testId="lufs-preview"
            onClick={() =>
              updateAuditionSettings((current) => toggleIphoneLufsPreview(current))
            }
          />
        </section>

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

        <button
          className="export-button"
          data-testid="iphone-export"
          type="button"
          disabled={!hasTrack || operation !== "idle"}
          onClick={exportMaster}
        >
          {isExporting ? "Exporting..." : "Export Master"}
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
      </section>
    </main>
  );
}

function toIphoneTrack(track: {
  id: string;
  path: string;
  display_name: string;
  source_format: string;
  duration_seconds: number | null;
}): IphoneTrack {
  return {
    id: track.id,
    displayName: track.display_name,
    path: track.path,
    sourceFormat: track.source_format,
    durationSeconds: track.duration_seconds,
  };
}

function buildExportReport(track: IphoneTrack, job: RenderJob): ExportReport {
  const outputPath = job.output_paths[0] ?? "";
  const measurements = job.measurements;
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
  onClick,
  testId,
}: {
  active: boolean;
  children: ReactNode;
  onClick: () => void;
  testId: string;
}) {
  return (
    <button
      aria-pressed={active}
      className={active ? "segment is-active" : "segment"}
      data-testid={testId}
      type="button"
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function ToggleRow({
  active,
  label,
  onClick,
  testId,
}: {
  active: boolean;
  label: string;
  onClick: () => void;
  testId: string;
}) {
  return (
    <button
      aria-pressed={active}
      className="toggle-row"
      data-testid={testId}
      type="button"
      onClick={onClick}
    >
      <span>{label}</span>
      <span className={active ? "switch is-on" : "switch"} aria-hidden="true" />
    </button>
  );
}

function formatSampleRate(sampleRate: number | null) {
  if (!sampleRate) return "Source";
  return `${(sampleRate / 1000).toFixed(sampleRate % 1000 === 0 ? 0 : 1)} kHz`;
}

function formatBitDepth(bitDepth: number | null) {
  if (!bitDepth) return "Source bit";
  return `${bitDepth}-bit`;
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
