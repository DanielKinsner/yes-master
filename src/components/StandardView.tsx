// src/components/StandardView.tsx
import type { Preset } from "../bindings";
import {
  STANDARD_LOUDNESS,
  STANDARD_STYLES,
  loudnessToTarget,
  presetToStyle,
  styleToPreset,
  targetToLoudness,
} from "../lib/standard-mapping";
import type { useTrackMaster } from "../hooks/useTrackMaster";
import { Knob, intensityLabel } from "./Knob";
import { WaveformLoading, WaveformView } from "./Waveform";
import { effectiveLoudnessTarget } from "../lib/effective-settings";
import { standardExportNotes } from "../lib/standard-export";

export function StyleTiles({
  preset,
  onSelect,
}: {
  preset: Preset;
  onSelect: (preset: Preset) => void;
}) {
  const activeStyle = presetToStyle(preset);
  return (
    <div className="std-tiles" role="group" aria-label="Style">
      {STANDARD_STYLES.map((s) => (
        <button
          key={s.id}
          type="button"
          className={"std-tile" + (s.id === activeStyle ? " is-active" : "")}
          data-tone={s.tone}
          aria-pressed={s.id === activeStyle}
          onClick={() => onSelect(styleToPreset(s.id))}
        >
          <span className="std-tile-label">{s.label}</span>
          <span className="std-tile-subtitle">{s.subtitle}</span>
        </button>
      ))}
    </div>
  );
}

export function LoudnessSegmented({
  targetLufs,
  onSelect,
}: {
  targetLufs: number | null;
  onSelect: (targetLufs: number) => void;
}) {
  const active = targetToLoudness(targetLufs);
  return (
    <div className="std-seg" role="group" aria-label="Loudness">
      {STANDARD_LOUDNESS.map((l) => (
        <button
          key={l.id}
          type="button"
          className={"std-seg-option" + (l.id === active ? " is-active" : "")}
          aria-pressed={l.id === active}
          onClick={() => onSelect(loudnessToTarget(l.id))}
        >
          <span className="std-seg-label">{l.label}</span>
          <span className="std-seg-lufs">{l.lufs} LUFS</span>
        </button>
      ))}
    </div>
  );
}

type TM = ReturnType<typeof useTrackMaster>;

function StandardExportButton({ tm }: { tm: TM }) {
  const notes = tm.lastExportReceipt
    ? standardExportNotes(tm.lastExportReceipt.checks)
    : null;
  return (
    <div className="std-export">
      <button
        type="button"
        className="primary std-create-master"
        disabled={!tm.selectedAnalysis || tm.isExporting || tm.isRendering}
        onClick={() => { void tm.exportStandardMaster(); }}
      >
        {tm.isExporting ? "Creating Master…" : "Create Master"}
      </button>
      {notes?.invalid && (
        <p className="std-export-block" role="alert">
          Saved, but this master has a problem — re-render: {notes.invalidMessage}
        </p>
      )}
      {notes?.integrityNote && (
        <p className="std-export-note">{notes.integrityNote}</p>
      )}
    </div>
  );
}

export function StandardView({ tm }: { tm: TM }) {
  const s = tm.selectedSettings;
  return (
    <div className="standard-view">
      <section className="std-hero">
        <div className="std-hero-head">
          {tm.tracks.length > 1 ? (
            <select
              className="std-track-select"
              aria-label="Track"
              value={tm.selectedTrackId ?? ""}
              onChange={(e) => tm.selectTrack(e.target.value)}
            >
              {tm.tracks.map((t) => (
                <option key={t.id} value={t.id}>{t.display_name}</option>
              ))}
            </select>
          ) : (
            <span className="std-track-chip">{tm.selectedTrack?.display_name ?? "No track"}</span>
          )}
          <button type="button" className="ghost-btn" onClick={tm.openImportDialog}>Import</button>
        </div>

        <button
          type="button"
          className="std-play"
          aria-label={tm.transport.isPlaying ? "Pause" : "Play"}
          onClick={tm.togglePlay}
        >
          {tm.transport.isPlaying ? "❚❚" : "►"}
        </button>

        <div className="std-wave">
          {tm.selectedWaveform ? (
            <WaveformView
              peaks={tm.selectedWaveform}
              isLoading={tm.isLoadingWaveform}
              isAnalyzing={tm.isAnalyzing}
              analysisProgress={tm.analysisProgress}
              currentTimeSec={tm.transport.currentTimeSec}
              durationSec={tm.selectedTrack?.duration_seconds ?? 0}
              region={tm.selectedRegion}
              onSeek={tm.seek}
              onSetRegion={tm.setRegion}
              onClearRegion={tm.clearRegion}
            />
          ) : (
            <WaveformLoading
              isAnalyzing={tm.isAnalyzing}
              isLoadingWaveform={tm.isLoadingWaveform}
              analysisProgress={tm.analysisProgress}
            />
          )}
        </div>

        <div className="std-ab">
          <div className="ab-toggle">
            <button className={tm.transport.playbackKind === "source" ? "on" : ""} onClick={() => tm.setPlaybackKind("source")}>Original</button>
            <button className={tm.transport.playbackKind === "master" ? "on" : ""} onClick={() => tm.setPlaybackKind("master")}>Mastered</button>
          </div>
          <button
            type="button"
            className={`toolbar-toggle ${tm.transport.volumeMatch ? "is-on" : ""}`}
            aria-pressed={tm.transport.volumeMatch}
            onClick={() => tm.setVolumeMatch(!tm.transport.volumeMatch)}
          >
            <span className="toolbar-toggle-box" aria-hidden />
            <span>Volume Match</span>
          </button>
        </div>
      </section>

      <section className="std-controls">
        <div className="std-step">
          <span className="std-step-label">1 · Style</span>
          <StyleTiles preset={s.preset} onSelect={tm.setPreset} />
        </div>

        <div className="std-step">
          <span className="std-step-label">2 · Intensity</span>
          <Knob
            label=""
            size="lg"
            value={s.intensity}
            min={0}
            max={1}
            step={0.01}
            defaultValue={0.5}
            format={(v) => `${Math.round(v * 100)}%`}
            caption={intensityLabel(s.intensity)}
            onChange={tm.setIntensity}
            centerValue
          />
        </div>

        <div className="std-step">
          <span className="std-step-label">3 · Loudness</span>
          <LoudnessSegmented targetLufs={effectiveLoudnessTarget(s)} onSelect={tm.setLoudnessTarget} />
        </div>

        <StandardExportButton tm={tm} />
      </section>
    </div>
  );
}
