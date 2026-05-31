import { useMemo, useState, type CSSProperties, type ReactNode } from "react";
import {
  attachIphoneTrack,
  initialIphoneAppState,
  markIphoneAnalysisReady,
  selectIphoneExportProfile,
  selectIphoneLoudness,
  selectIphoneTone,
  switchIphonePlayback,
  toggleIphoneLufsPreview,
  toggleIphoneVolumeMatch,
  toIphoneSimplePlan,
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

export default function App({
  backend = iphoneBackend,
  pickAudioPath = pickIphoneAudioPath,
  pickOutputPath = pickIphoneOutputPath,
}: {
  backend?: IphoneBackend;
  pickAudioPath?: () => Promise<string | null>;
  pickOutputPath?: () => Promise<string | null>;
}) {
  const [state, setState] = useState<IphoneAppState>(initialIphoneAppState);
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [exportChecks, setExportChecks] = useState<QualityCheck[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const plan = useMemo(() => toIphoneSimplePlan(state), [state]);
  const hasTrack = state.track !== null;
  const sampleRate = plan.exportSettings.advanced.target_sample_rate;
  const bitDepth = plan.exportSettings.advanced.bit_depth;
  const targetLufs = plan.exportSettings.advanced.lufs_offset_db;

  async function importTrack() {
    setMessage("Importing...");
    try {
      const path = await pickAudioPath();
      if (!path) {
        setMessage(null);
        return;
      }
      const imported = await backend.importTrack(path);
      setState((current) => attachIphoneTrack(current, toIphoneTrack(imported)));
      setMessage("Analyzing...");
      const nextAnalysis = await backend.analyzeTrack(imported.id, imported.path);
      setAnalysis(nextAnalysis);
      setState((current) => markIphoneAnalysisReady(current));
      setMessage(null);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function exportMaster() {
    if (!state.track) return;
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
        settings: plan.exportSettings,
        outputPath,
      });
      const report = buildExportReport(state.track, job);
      const checks = await backend.runExportChecks(
        report,
        analysis,
        plan.exportSettings,
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
    }
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
            <div>
              <p className="track-label">Track</p>
              <h2>{state.track?.displayName}</h2>
            </div>
          ) : (
            <button
              className="import-button"
              data-testid="iphone-import"
              type="button"
              onClick={importTrack}
            >
              Import Track
            </button>
          )}
        </section>

        <section className="wave-panel" aria-label="Audition">
          <div className="waveform" aria-hidden="true">
            {Array.from({ length: 36 }, (_, index) => (
              <span
                key={index}
                style={{ "--bar": `${32 + ((index * 19) % 54)}%` } as CSSProperties}
              />
            ))}
          </div>
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
              onClick={() =>
                setState((current) => switchIphonePlayback(current, "mastered"))
              }
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
                setState((current) =>
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
                setState((current) =>
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
                setState((current) =>
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

        <section className="toggle-stack">
          <ToggleRow
            active={state.volumeMatch}
            label="Volume Match"
            testId="volume-match"
            onClick={() => setState((current) => toggleIphoneVolumeMatch(current))}
          />
          <ToggleRow
            active={state.lufsPreview}
            label="LUFS Preview"
            testId="lufs-preview"
            onClick={() => setState((current) => toggleIphoneLufsPreview(current))}
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
              {formatSampleRate(sampleRate)} · {bitDepth ?? 24}-bit
            </strong>
          </div>
        </section>

        <button
          className="export-button"
          data-testid="iphone-export"
          type="button"
          disabled={!hasTrack}
          onClick={exportMaster}
        >
          Export Master
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
