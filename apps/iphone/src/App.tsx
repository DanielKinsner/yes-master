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
  type IphoneBackend,
} from "./iphone-api";
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
}: {
  backend?: IphoneBackend;
  pickAudioPath?: () => Promise<string | null>;
}) {
  const [state, setState] = useState<IphoneAppState>(initialIphoneAppState);
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
      await backend.analyzeTrack(imported.id, imported.path);
      setState((current) => markIphoneAnalysisReady(current));
      setMessage(null);
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

        <button className="export-button" type="button" disabled={!hasTrack}>
          Export Master
        </button>
        {message ? <p className="status-message">{message}</p> : null}
      </section>
    </main>
  );
}

function toIphoneTrack(track: {
  id: string;
  display_name: string;
  duration_seconds: number | null;
}): IphoneTrack {
  return {
    id: track.id,
    displayName: track.display_name,
    durationSeconds: track.duration_seconds,
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
